//! tmux imsg wire protocol: message layer, codec, and native server.

pub mod backing;
pub mod codec;
pub mod introspect;
pub mod message;
pub(crate) mod native;
pub mod status_client;
pub mod traits;

pub use backing::Backing;
pub use message::{msgtype, Frame, Message, PROTOCOL_VERSION};
pub use native::NativeServer;
pub use status_client::{PaneStatus, StatusClient, StatusUpdate};
pub use traits::{FrameReader, FrameWriter, NonblockingFrameReader, TmuxServer};
