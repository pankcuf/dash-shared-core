use std::ffi::c_void;
use std::net::SocketAddr;
use std::{ptr, thread};
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

use system_configuration::core_foundation::base::{kCFAllocatorDefault, TCFType};
use system_configuration::core_foundation::runloop::{__CFRunLoop, __CFRunLoopSource, CFRunLoopAddSource, CFRunLoopContainsSource, CFRunLoopGetCurrent, CFRunLoopRef, CFRunLoopRemoveSource, CFRunLoopRun, CFRunLoopSourceContext, CFRunLoopSourceCreate, CFRunLoopStop, kCFRunLoopCommonModes};
use system_configuration::core_foundation::string::{CFString, CFStringRef};
use system_configuration::network_reachability::{ReachabilityFlags, SCNetworkReachability};
use system_configuration::sys::network_reachability::{SCNetworkReachabilityContext, SCNetworkReachabilityFlags, SCNetworkReachabilityRef, SCNetworkReachabilityScheduleWithRunLoop, SCNetworkReachabilitySetCallback, SCNetworkReachabilityUnscheduleFromRunLoop};

/// Monitors the reachability of domains, and addresses for both WWAN and WiFi network interfaces.
/// Reachability can be used to determine background information about why a network operation failed,
/// or to trigger a network operation retrying when a connection is established. It should not be used
/// to prevent a user from initiating a network request, as it's possible that an initial request may
/// be required to establish reachability.
/// See Apple's Reachability Sample Code (https://developer.apple.com/library/ios/samplecode/reachability/)
/// @warning Instances of `Manager` must be started with `-startMonitoring` before
/// reachability status can be determined.
// pub const ReachabilityDidChangeNotification: &str = "";
// pub const ReachabilityNotificationStatusItem: &str = "";
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Association {
    Address = 1,
    AddressPair = 2,
    Name = 3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Status {
    Unknown = -1,
    NotReachable = 0,
    ReachableViaWWAN = 1,
    ReachableViaWiFi = 2,
}
unsafe impl Send for Status {}
unsafe impl Sync for Status {}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Command {
    START,
    STOP
}
unsafe impl Send for Command {}
unsafe impl Sync for Command {}

#[derive(Clone, Copy, Debug, PartialEq)]
struct RunLoopSendInfo(*mut __CFRunLoop, *mut __CFRunLoopSource, *mut SCNetworkReachabilityRef);
unsafe impl Send for RunLoopSendInfo {}


#[cfg(target_os = "ios")]
const fn status_for_flags(flags: ReachabilityFlags) -> Status {
    let is_reachable = flags.contains(ReachabilityFlags::REACHABLE);
    let needs_connection = flags.contains(ReachabilityFlags::CONNECTION_REQUIRED);
    let can_auto_connect = flags.contains(ReachabilityFlags::CONNECTION_ON_DEMAND) || flags.contains(ReachabilityFlags::CONNECTION_ON_TRAFFIC);
    let can_connect_without_user_interaction = can_auto_connect && !flags.contains(ReachabilityFlags::INTERVENTION_REQUIRED);
    let is_network_reachable = is_reachable && (!needs_connection || can_connect_without_user_interaction);
    if !is_network_reachable {
        Status::NotReachable
    } else if flags.contains(ReachabilityFlags::IS_WWAN) {
        Status::ReachableViaWWAN
    } else {
        Status::ReachableViaWiFi
    }
}

#[cfg(target_os = "macos")]
const fn status_for_flags(flags: ReachabilityFlags) -> Status {
    let is_reachable = flags.contains(ReachabilityFlags::REACHABLE);
    let needs_connection = flags.contains(ReachabilityFlags::CONNECTION_REQUIRED);
    let can_auto_connect = flags.contains(ReachabilityFlags::CONNECTION_ON_DEMAND) || flags.contains(ReachabilityFlags::CONNECTION_ON_TRAFFIC);
    let can_connect_without_user_interaction = can_auto_connect && !flags.contains(ReachabilityFlags::INTERVENTION_REQUIRED);
    let is_network_reachable = is_reachable && (!needs_connection || can_connect_without_user_interaction);
    if !is_network_reachable {
        Status::NotReachable
    } else {
        Status::ReachableViaWiFi
    }
}

struct MonitorContext {
    host: SocketAddr,
    receiver: Receiver<Command>,
}

impl MonitorContext {
    fn new(host: SocketAddr, receiver: Receiver<Command>) -> Self {
        Self { host, receiver }
    }
    extern "C" fn schedule_callback(info: *const c_void, run_loop_ref: CFRunLoopRef, _run_loop_mode: CFStringRef) {
        println!("MonitorContext::schedule_callback {:?} {:?}", info, run_loop_ref);
        // let context: &mut Self = unsafe { &mut (*(info as *mut _)) };
    }

    extern "C" fn cancel_callback(info: *const c_void, run_loop_ref: CFRunLoopRef, _run_loop_mode: CFStringRef) {
        println!("MonitorContext::cancel_callback {:?} {:?}", info, run_loop_ref);
        // let context: &mut Self = unsafe { &mut (*(info as *mut _)) };
    }

    extern "C" fn perform_callback(info: *const c_void) {
        println!("MonitorContext::perform_callback {:?}", info);
    }

    extern "C" fn copy_ctx_description(_ctx: *const c_void) -> CFStringRef {
        let description = CFString::from_static_string("Reachability callback context");
        let description_ref = description.as_concrete_TypeRef();
        std::mem::forget(description);
        description_ref
    }
}

struct ReachabilityContext<T> where T: Fn(Status) + Sync + Send {
    reachability: SCNetworkReachability,
    status_callback: T,
}

impl<T> ReachabilityContext<T> where T: Fn(Status) + Sync + Send {
    fn new(reachability: SCNetworkReachability, status_callback: T) -> Self {
        Self { reachability, status_callback }
    }

    extern "C" fn reachability_callback(_info: SCNetworkReachabilityRef, flags: SCNetworkReachabilityFlags, context: *mut c_void) {
        let context: &mut Self = unsafe { &mut (*(context as *mut _)) };
        let flags = unsafe { ReachabilityFlags::from_bits_unchecked(flags) };
        //println!("reachability_callback: flags: {:?}", flags);
        let status = status_for_flags(flags);
        (context.status_callback)(status);
    }

    extern "C" fn copy_ctx_description(_ctx: *const c_void) -> CFStringRef {
        let description = CFString::from_static_string("Reachability callback context");
        let description_ref = description.as_concrete_TypeRef();
        std::mem::forget(description);
        description_ref
    }

    extern "C" fn release_context(ctx: *const c_void) where Self: Sized {
        // println!("ManagerContext::release_context {:?}", ctx);
        unsafe { Arc::decrement_strong_count(ctx as *mut Self); }
    }

    extern "C" fn retain_context(ctx_ptr: *const c_void) -> *const c_void where Self: Sized  {
        // println!("ManagerContext::retain_context {:?}", ctx_ptr);
        unsafe { Arc::increment_strong_count(ctx_ptr as *mut Self); }
        ctx_ptr
    }
}

pub trait ReachabilityStatusCallback: Fn(Status) + Send + Sync {}
impl<T: Fn(Status) + Send + Sync + Clone> ReachabilityStatusCallback for T {}

struct Inner {
    pub status: Status,
    handlers: Vec<Arc<dyn ReachabilityStatusCallback<Output=()>>>,
}

impl Inner {
    pub fn update(&mut self, status: Status) {
        self.status = status;
        self.handlers.iter().for_each(|handler| handler(status));
    }
}

unsafe impl Send for Inner {}
unsafe impl Sync for Inner {}

pub struct ReachabilityManager {
    sender: Option<Arc<Sender<Command>>>,
    handle: Option<thread::JoinHandle<()>>,
    inner: Arc<RwLock<Inner>>,
    // inner: Inner,
    pub is_running: bool,
}

impl ReachabilityManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(Inner {
                status: Status::Unknown,
                handlers: vec![]
            })),
            sender: None,
            handle: None,
            is_running: false,
        }
    }

    fn local_host() -> SocketAddr {
        "0.0.0.0:0".parse::<SocketAddr>().unwrap()
    }

    fn create_run_loop_source_context() -> CFRunLoopSourceContext {
        CFRunLoopSourceContext {
            version: 0,
            info: ptr::null_mut(),
            retain: None,
            release: None,
            copyDescription: Some(MonitorContext::copy_ctx_description),
            equal: None,
            hash: None,
            schedule: Some(MonitorContext::schedule_callback),
            cancel: Some(MonitorContext::cancel_callback),
            perform: MonitorContext::perform_callback,
        }
    }

    fn schedule_reachability<F>(context: ReachabilityContext<F>) where F: Fn(Status) + Sync + Send {
        let reachability_ref = context.reachability.as_concrete_TypeRef();
        let reachability_context_arc = Arc::new(context);
        let mut callback_context = SCNetworkReachabilityContext {
            version: 0,
            info: Arc::as_ptr(&reachability_context_arc) as *mut _,
            retain: Some(ReachabilityContext::<F>::retain_context),
            release: Some(ReachabilityContext::<F>::release_context),
            copyDescription: Some(ReachabilityContext::<F>::copy_ctx_description),
        };
        unsafe {
            let run_loop_ref = CFRunLoopGetCurrent();
            if SCNetworkReachabilitySetCallback(reachability_ref, Some(ReachabilityContext::<F>::reachability_callback), &mut callback_context) == 0 {
                panic!("setup_reachability_loop:: can't perform SCNetworkReachabilitySetCallback");
            }
            if SCNetworkReachabilityScheduleWithRunLoop(reachability_ref, run_loop_ref, kCFRunLoopCommonModes) == 0 {
                panic!("setup_reachability_loop:: can't perform SCNetworkReachabilitySetCallback");
            }
        }
    }

    fn unschedule_reachability(loop_info: RunLoopSendInfo) {
        unsafe {
            let run_loop_ref = loop_info.0 as *mut __CFRunLoop;
            let source_ref = loop_info.2 as *mut __CFRunLoopSource;
            let reachability_ref = loop_info.2 as SCNetworkReachabilityRef;
            if SCNetworkReachabilityUnscheduleFromRunLoop(reachability_ref, run_loop_ref, kCFRunLoopCommonModes) == 0 {
                panic!("Reachability.loop::Can't unschedule ")
            }
            if CFRunLoopContainsSource(run_loop_ref, source_ref, kCFRunLoopCommonModes) != 0 {
                CFRunLoopRemoveSource(run_loop_ref, source_ref, kCFRunLoopCommonModes);
            }
            CFRunLoopStop(run_loop_ref);
        }
    }

    fn setup_reachability_loop(host: SocketAddr, sender_info: Sender<RunLoopSendInfo>, inner: Arc<RwLock<Inner>>) {
        let mut run_loop_context = Self::create_run_loop_source_context();
        let reachability = SCNetworkReachability::from(host);
        let reachability_ref = reachability.as_concrete_TypeRef();
        let reachability_context = ReachabilityContext::new(reachability, move |status| {
            if let Ok(mut writer) = inner.try_write() {
                writer.update(status);
            }
        });
        unsafe {
            let run_loop_ref = CFRunLoopGetCurrent();
            let source = CFRunLoopSourceCreate(kCFAllocatorDefault, 0, &mut run_loop_context);
            CFRunLoopAddSource(run_loop_ref, source, kCFRunLoopCommonModes);
            Self::schedule_reachability(reachability_context);
            let loop_info = RunLoopSendInfo(run_loop_ref, source, reachability_ref as *mut _);
            sender_info.send(loop_info).unwrap();
            CFRunLoopRun();
        }
    }

    fn setup_monitoring(host: SocketAddr, receiver: Receiver<Command>, inner: Arc<RwLock<Inner>>) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let context = MonitorContext::new(host, receiver);
            let host = context.host.clone();
            let context_arc = Arc::new(context);
            let (sender_info, receiver_info) = channel::<RunLoopSendInfo>();
            if let Ok(Command::START) = context_arc.receiver.recv() {
                let mut loop_info: Option<RunLoopSendInfo> = None;
                let handle = thread::spawn(move || {
                    Self::setup_reachability_loop(host, sender_info, inner)
                });
                loop {
                    if let Ok(locked_info) = receiver_info.try_recv() {
                        loop_info = Some(locked_info);
                    }
                    if let Ok(Command::STOP) = context_arc.receiver.try_recv() {
                        Self::unschedule_reachability(loop_info.take().unwrap());
                        handle.join().unwrap();
                        break;
                    }
                    thread::sleep(Duration::from_millis(20));
                }
            }
        })
    }

    pub fn add_handler<F: ReachabilityStatusCallback + 'static>(&mut self, handler: F) {
        if let Ok(mut writer) = self.inner.try_write() {
            writer.handlers.push(Arc::new(handler));
        }
    }

    pub fn start_monitoring(&mut self) {
        println!("ReachabilityManager::start");
        let (sender, receiver) = channel();
        self.is_running = true;
        self.sender = Some(Arc::new(sender));
        self.handle = Some(Self::setup_monitoring(Self::local_host(), receiver, self.inner.clone()));
        self.sender.as_ref().unwrap().send(Command::START).unwrap();
    }

    pub fn stop_monitoring(&mut self) {
        println!("ReachabilityManager::stop");
        self.is_running = false;
        self.sender.as_ref().unwrap().send(Command::STOP).unwrap();
        self.handle.take().unwrap().join().unwrap();
    }

    pub fn last_status(&self) -> Status {
        if let Ok(reader) = self.inner.try_read() {
            reader.status
        } else {
            Status::Unknown
        }
    }
}



// cargo test --package reachability --lib test_monitoring -- -Z unstable-options --format=json --show-output --nocapture
#[test]
fn test_monitoring() {
    let mut manager = ReachabilityManager::new();
    manager.add_handler(|status| println!("handler::status {:?}", status));
    manager.start_monitoring();
    println!("test_monitoring -> start_monitoring (expiration 15 sec)...");
    thread::sleep(Duration::from_secs(15));
    manager.stop_monitoring();
    println!("test_monitoring -> end (expiration 15 sec)... {:?}", manager.last_status());
    thread::sleep(Duration::from_secs(15));
    println!("test_monitoring -> end");
}
