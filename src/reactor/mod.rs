//! Application compatibility over hmux-rt and hmux-buffer.
//!
//! Streams own their buffers. Registrations own cancellable local tasks;
//! callbacks run without registry borrows. Descriptor leases are
//! duplicated once per live endpoint and close after an executing poll finishes.
#![allow(clippy::missing_safety_doc)]
mod buffer;
mod streams;
mod tasks;
mod timers;
pub use buffer::*;
use hmux_rt::Runtime as _;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::os::fd::{FromRawFd, OwnedFd};
use std::rc::{Rc, Weak};
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
    static PID: Cell<u32> = const { Cell::new(0) };
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
    PID.with(|p| p.set(std::process::id()));
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
    ensure_runtime();
    let mut runtime = HOST.with(|h| h.borrow_mut().take().expect("recursive runtime dispatch"));
    runtime.poll(None).expect("hmux-rt poll");
    HOST.with(|h| *h.borrow_mut() = Some(runtime));
}

pub fn reset_after_fork() -> std::io::Result<()> {
    ensure_runtime();
    if PID.with(|pid| pid.get() == std::process::id()) {
        return Ok(());
    }
    tasks::stop_tasks();
    streams::stop_tasks();
    timers::stop_tasks();
    FDS.with(|fds| fds.borrow_mut().clear());
    HOST.with(|host| {
        let mut host = host.borrow_mut();
        let runtime = host.as_mut().expect("runtime initialized");
        runtime.reset_after_fork()?;
        HANDLE.with(|handle| *handle.borrow_mut() = Some(runtime.handle()));
        Ok::<_, std::io::Error>(())
    })?;
    PID.with(|pid| pid.set(std::process::id()));
    tasks::restart()?;
    timers::restart()?;
    streams::restart();
    Ok(())
}

pub fn shutdown_runtime() {
    tasks::clear();
    streams::clear();
    timers::clear();
    // Stream and timer captures may enqueue deferred cleanup while clearing.
    tasks::clear();
    FDS.with(|f| f.borrow_mut().clear());
    HANDLE.with(|h| h.borrow_mut().take());
    HOST.with(|h| h.borrow_mut().take());
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
