//! The boundary the crate reaches its event loop through.
//!
use ::core::ffi::c_short;

use crate::types::timeval;

pub mod buffer;
pub mod notify;
pub mod registry;
pub mod runtime;
pub mod stream;

pub use buffer::Buf;
pub use registry::{IoHandle, SignalHandle, TimerHandle};
pub use runtime::{Base, current, shutdown};
pub use stream::Stream;

/// What a [`Stream`] calls back when it has read something or drained what it
/// was given. The closure carries whatever the callback works on, so nothing
/// is handed back through an untyped pointer.
pub type StreamCb = ::std::rc::Rc<dyn Fn(Stream)>;

/// What a [`Stream`] calls back when the connection fails or ends.
pub type StreamErrorCb = ::std::rc::Rc<dyn Fn(Stream, c_short)>;

/// What an [`IoWatch`] is waiting for.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Interest {
    Read,
    Write,
    ReadWrite,
}

/// Whether an [`IoWatch`] stays armed once it has fired.
///
/// Both are in use: the client's terminal input is persistent, while its
/// output re-arms itself only while there is more to write — making that one
/// persistent busy-loops on a writable descriptor.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum WatchMode {
    Once,
    Persistent,
}

/// A deadline the loop calls back on.
///
/// The handle is embedded by value in structs the tree allocates with
/// `xcalloc` and reads all-zero as "never set up", which is what [`Timer::ZERO`]
/// is and what [`Timer::is_set`] answers `false` for.
pub trait Timer: Copy {
    /// The all-zero state, for a handle that has not been given a callback
    /// yet. This is what `Default` answers with too; it is spelled as a
    /// constant so a `static` can start out this way.
    const ZERO: Self;

    /// Whether the timer has been given a callback. This is what guards
    /// setting one, since setting it again retires the slot the old one lived
    /// in; nothing else needs asking, because arming and disarming a handle
    /// that has no callback are both allowed and do nothing.
    fn is_set(&self) -> bool;

    /// Call back once, `after` from now. A zero `after` means the next turn of
    /// the loop.
    fn arm(&mut self, after: timeval);

    /// Take the deadline off. Doing this to a timer that is not armed is
    /// allowed and does nothing.
    fn disarm(&mut self);

    /// Whether a deadline is outstanding.
    fn is_armed(&self) -> bool;
}

/// A watch on a descriptor becoming readable or writable.
///
/// Unlike [`Timer`], this has nothing to ask about its state: the tree sets a
/// watch's callback at one place per watch rather than lazily, and enabling an
/// already-enabled watch or disabling an already-disabled one does nothing.
pub trait IoWatch: Copy {
    /// The all-zero state, for a handle that has not been given a descriptor
    /// and a callback yet.
    const ZERO: Self;

    /// Put the watch on the loop.
    fn enable(&mut self);

    /// Take it off.
    fn disable(&mut self);
}

/// A watch on a signal. It stays on until it is taken off.
pub trait SignalWatch: Copy {
    /// The all-zero state, for a handle that has not been given a signal and
    /// a callback yet.
    const ZERO: Self;

    /// Take it off.
    fn unwatch(&mut self);
}

/// The loop itself.
///
/// A process has one, and it holds no state of its own worth handing around:
/// [`current`] answers with it wherever it is needed, including the one place
/// the tree threads it from — `client_main` and `server_start` take it as an
/// argument the way the C tree passed its `event_base` around.
pub trait Reactor: Copy {
    /// Turn the loop once, dispatching whatever is ready.
    fn run_once(&mut self);

    /// Re-register the events that survived a fork, answering whether that
    /// worked. This is not "drop everything": what was armed before is armed
    /// after.
    fn reinit(&mut self) -> bool;

    /// Call `callback` once, on the next turn of the loop.
    fn defer(&mut self, callback: impl FnOnce() + 'static);

    /// What the loop is, for the log.
    fn describe(&self) -> String;
}
