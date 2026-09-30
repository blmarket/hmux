//! Application compatibility over hmux-rt and hmux-buffer.
//!
//! Streams own their buffers. Task handles own their cancellable futures;
//! callbacks run without registry borrows. Descriptor leases are
//! duplicated once per live endpoint and close after an executing poll finishes.
#![allow(clippy::missing_safety_doc)]
mod buffer;
mod streams;
mod tasks;
mod timers;
pub use buffer::*;
use hmux_rt::{Handle as _, Runtime as _};
use std::cell::RefCell;
use std::collections::HashMap;
use std::os::fd::{FromRawFd, OwnedFd};
use std::rc::{Rc, Weak};
use std::time::Duration;
pub use streams::*;
pub use tasks::Task;
pub use timers::{timer_once_owned, Timer};

#[repr(C)]
pub struct bufferevent_ops {
    _private: [u8; 0],
}
thread_local! {
    static HOST: RefCell<Option<hmux_rt::mio::Runtime>> = const { RefCell::new(None) };
    static HANDLE: RefCell<Option<hmux_rt::mio::Handle>> = const { RefCell::new(None) };
    static FDS: RefCell<HashMap<i32, Weak<hmux_rt::mio::Descriptor>>> = RefCell::new(HashMap::new());
}
pub(crate) fn handle() -> hmux_rt::mio::Handle {
    HANDLE.with(|h| h.borrow().as_ref().expect("runtime initialized").clone())
}
pub(crate) fn runtime_initialized() -> bool {
    HANDLE.with(|h| h.borrow().is_some())
}
fn ensure_runtime() {
    if runtime_initialized() {
        return;
    }
    let runtime = hmux_rt::mio::Runtime::new().expect("hmux-rt initialization");
    HANDLE.with(|h| *h.borrow_mut() = Some(runtime.handle()));
    HOST.with(|h| *h.borrow_mut() = Some(runtime));
}
pub(crate) fn descriptor(fd: i32) -> std::io::Result<Rc<hmux_rt::mio::Descriptor>> {
    if let Some(source) = FDS.with(|f| f.borrow().get(&fd).and_then(Weak::upgrade)) {
        return Ok(source);
    }
    // SAFETY: duplicate retains the open file description through callback cancellation.
    let duplicate = unsafe { libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, 0) };
    if duplicate < 0 {
        return Err(std::io::Error::last_os_error());
    }
    let lease = Rc::new(unsafe { OwnedFd::from_raw_fd(duplicate) });
    let source = Rc::new(handle().descriptor(lease)?);
    FDS.with(|f| {
        let mut f = f.borrow_mut();
        f.retain(|_, value| value.strong_count() != 0);
        f.insert(fd, Rc::downgrade(&source));
    });
    Ok(source)
}
pub fn init_runtime() {
    ensure_runtime();
}

/// Forget a closing endpoint before its descriptor number can be reused.
pub(crate) fn forget_descriptor(fd: i32) {
    FDS.with(|fds| fds.borrow_mut().remove(&fd));
}

pub fn poll_runtime() {
    poll_runtime_with_timeout(None);
}

fn poll_runtime_with_timeout(max_wait: Option<Duration>) {
    ensure_runtime();
    HOST.with(|h| {
        h.borrow_mut()
            .as_mut()
            .expect("runtime initialized")
            .poll(max_wait)
            .expect("hmux-rt poll");
    });
}

/// Drop the runtime after polling has returned. Calling this from a callback
/// panics before cleanup because the runtime owner is still borrowed.
pub fn shutdown_runtime() {
    let runtime = HOST.with(|h| h.borrow_mut().take());
    let Some(runtime) = runtime else { return };
    streams::clear();
    timers::clear();
    FDS.with(|f| f.borrow_mut().clear());
    // Keep the scheduling handle accessible while destructors run.
    drop(runtime);
    HANDLE.with(|h| h.borrow_mut().take());
}
pub(crate) async fn yield_now() {
    let mut yielded = false;
    std::future::poll_fn(|cx| {
        if yielded {
            return std::task::Poll::Ready(());
        }
        yielded = true;
        cx.local_waker().wake_by_ref();
        std::task::Poll::Pending
    })
    .await
}
