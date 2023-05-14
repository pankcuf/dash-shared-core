pub mod buffer;
pub mod control_dispatcher;
pub mod control_notification;
pub mod expected_reply;
pub mod layer;
pub mod message_reader;
pub mod notification;
pub mod notification_dispatcher;
pub mod peer;
pub mod peer_selector;
pub mod peer_source;
pub mod pool;
pub mod state;
pub mod state_flags;
pub mod timeout;
pub mod version_carrier;

use std::collections::HashMap;
use std::sync::{Arc, mpsc};
use futures::task::Waker;
use mio::Token;
use mio::net::TcpListener;
pub use self::buffer::{Buffer, PassThroughBufferReader};
pub use self::control_dispatcher::P2PControlDispatcher;
pub use self::control_notification::P2PControlNotification;
pub use self::expected_reply::ExpectedReply;
pub use self::layer::P2P;
pub use self::message_reader::MessageReader;
pub use self::notification::P2PNotification;
pub use self::notification_dispatcher::P2PNotificationDispatcher;
pub use self::peer::{Peer, PeerId, PeerMap};
pub use self::peer_selector::PeerSelector;
pub use self::peer_source::PeerSource;
pub use self::state::{DashP2PState, PeerState};
pub use self::state_flags::PeerStateFlags;
pub use self::timeout::{SharedTimeout, Timeout};
pub use self::version_carrier::VersionCarrier;

pub type ListenerMap = HashMap<Token, Arc<TcpListener>>;
pub type WakerMap = HashMap<PeerId, Waker>;

pub(crate) type P2PControlReceiver = mpsc::Receiver<P2PControlNotification>;
pub(crate) type P2PControlSender = mpsc::Sender<P2PControlNotification>;

pub type P2PNotificationReceiver = mpsc::Receiver<P2PNotification>;
pub type P2PNotificationSender = mpsc::SyncSender<P2PNotification>;
