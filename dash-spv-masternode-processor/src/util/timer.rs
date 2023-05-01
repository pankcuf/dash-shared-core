use std::time;

#[derive(Default)]
pub struct Timer {
    timer: Option<Box<os_timer::Timer>>,
}

impl Clone for Timer {
    fn clone(&self) -> Self {
        // Self { timer: self.timer.clone() }
        Self { timer: None }
    }
}

impl Timer {
    pub fn new<F: 'static + FnMut()>(callback: F) -> Self {
        Self { timer: os_timer::Timer::new(os_timer::Callback::closure(callback)).map(Box::new) }
    }

    pub fn schedule(&self, timeout: time::Duration, interval: time::Duration) -> bool {
        if let Some(timer) = &self.timer {
            timer.schedule_interval(timeout, interval)
        } else {
            false
        }
    }

    pub fn stop(&mut self) {
        self.timer = Some(Box::new(unsafe {
            os_timer::Timer::uninit()
        }));
    }
}
