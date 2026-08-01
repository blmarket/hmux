//! tmux imsg wire protocol: message layer, codec, and server traits.

pub mod backing;
pub mod codec;
pub mod introspect;
pub mod message;
pub mod traits;

pub use crate::native::NativeServer;
pub use crate::server::Server;
pub use backing::Backing;
pub use message::{msgtype, Frame, Message, PROTOCOL_VERSION};
pub use traits::{
    FrameReader, FrameWriter, NonblockingFrameReader, NonblockingFrameWriter, TmuxServer,
    WriteQueueFull,
};
