//! tmux imsg wire protocol: message layer, codec, and server traits.

pub mod codec;
pub mod introspect;
pub mod message;
pub mod traits;

pub use crate::server::Server;
pub use message::{Frame, Message, PROTOCOL_VERSION, msgtype};
pub use traits::{
    NonblockingFrameReader, NonblockingFrameWriter, NonblockingTmuxServer, WriteQueueFull,
};
