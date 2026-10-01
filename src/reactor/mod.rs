//! Application compatibility over hmux-rt and hmux-buffer.
//!
//! Streams own their buffers. Task handles own their cancellable futures;
//! callbacks run without registry borrows. Descriptor leases are
//! owned by each task and close after an executing poll finishes.
#![allow(clippy::missing_safety_doc)]
mod buffer;
mod streams;
mod tasks;
mod timers;
pub use buffer::*;
use hmux_rt::{Handle as _, Runtime as _};
use std::cell::RefCell;
use std::os::fd::BorrowedFd;
use std::time::Duration;
pub use streams::*;
pub use tasks::task_start;
pub use timers::Timer;

#[repr(C)]
pub struct bufferevent_ops {
    _private: [u8; 0],
}
thread_local! {
    static HOST: RefCell<Option<hmux_rt::mio::Runtime>> = const { RefCell::new(None) };
    static HANDLE: RefCell<Option<hmux_rt::mio::Handle>> = const { RefCell::new(None) };
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
/// Create a task-owned registration; no raw descriptor lookup survives this call.
pub(crate) fn io(fd: BorrowedFd<'_>) -> std::io::Result<hmux_rt::mio::Io> {
    // Retain the open file description through callback cancellation.
    let lease = fd.try_clone_to_owned()?;
    handle().io(lease)
}
pub fn init_runtime() {
    ensure_runtime();
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
