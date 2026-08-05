//! tmux imsg wire protocol: message layer, codec, and server traits.

pub mod codec;
pub mod introspect;
pub mod message;
pub mod traits;

pub use crate::server::Server;
pub use message::{msgtype, Frame, Message, PROTOCOL_VERSION};
pub use traits::{
    FrameReader, FrameWriter, NonblockingFrameReader, NonblockingFrameWriter,
    NonblockingTmuxServer, TmuxServer, WriteQueueFull,
};
