//! Mio-backed implementation of the local runtime contracts.

mod readiness;
mod runtime;
mod signals;
mod waits;

pub use readiness::Io;
pub use runtime::{Core, Handle, Runtime, Task};
pub use signals::Signals;
pub use waits::Sleep;

mod descriptor;
pub use descriptor::Descriptor;
