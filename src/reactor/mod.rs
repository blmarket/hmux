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
use hmux_rt::Runtime as _;
use std::cell::RefCell;
use std::collections::HashMap;
use std::os::fd::{FromRawFd, OwnedFd};
use std::rc::{Rc, Weak};
use std::time::Duration;
pub use streams::*;
pub use tasks::{defer, Task};
#[allow(deprecated)]
pub use timers::timer_once;
pub use timers::{timer_once_owned, Timer};

#[repr(C)]
pub struct bufferevent_ops {
    _private: [u8; 0],
}
thread_local! {
    static HOST: RefCell<Option<hmux_rt::mio::Runtime>> = const { RefCell::new(None) };
    static FDS: RefCell<HashMap<i32, Weak<hmux_rt::mio::Descriptor>>> = RefCell::new(HashMap::new());
}
pub(crate) fn handle() -> hmux_rt::mio::Handle {
    HOST.with(|h| h.borrow().as_ref().expect("runtime initialized").handle())
}
pub(crate) fn runtime_initialized() -> bool {
    HOST.with(|h| h.borrow().is_some())
}
fn ensure_runtime() {
    if runtime_initialized() {
        return;
    }
    let runtime = hmux_rt::mio::Runtime::new().expect("hmux-rt initialization");
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
    let source = Rc::new(hmux_rt::mio::Descriptor::new(&handle(), lease)?);
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
    let shutdown = HOST.with(|h| {
        let host = h.borrow();
        let runtime = host.as_ref().expect("runtime initialized");
        runtime.poll(max_wait).expect("hmux-rt poll");
        runtime.is_shutdown()
    });
    if shutdown {
        let runtime = HOST.with(|h| h.borrow_mut().take());
        drop(runtime);
    }
}

pub fn shutdown_runtime() {
    streams::clear();
    timers::clear();
    FDS.with(|f| f.borrow_mut().clear());
    let runtime = HOST.with(|h| {
        // Keep the closed runtime reachable while capture destructors run, so
        // scheduling through them cannot initialize a replacement runtime.
        if let Some(runtime) = h.borrow().as_ref() {
            runtime.shutdown();
        }
        // Dispatch holds a shared borrow. In that case poll_runtime removes
        // the closed owner after the executing callback or future returns.
        h.try_borrow_mut().ok().and_then(|mut host| host.take())
    });
    drop(runtime);
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
