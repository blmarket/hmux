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
use hmux_rt::Runtime as _;
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
}
fn ensure_runtime() {
    if hmux_rt::mio::Runtime::is_initialized() {
        return;
    }
    let runtime = hmux_rt::mio::Runtime::new().expect("hmux-rt initialization");
    HOST.with(|h| *h.borrow_mut() = Some(runtime));
}
/// Create a task-owned registration; no raw descriptor lookup survives this call.
pub(crate) fn io(fd: BorrowedFd<'_>) -> std::io::Result<hmux_rt::mio::Io> {
    // Retain the open file description through callback cancellation.
    let lease = fd.try_clone_to_owned()?;
    hmux_rt::mio::Io::new(lease)
}
/// Translate an owned I/O error at the remaining C-style callback boundary.
pub(crate) fn io_status(result: std::io::Result<()>) -> i32 {
    match result {
        Ok(()) => 0,
        Err(error) => {
            unsafe {
                *libc::__errno_location() = error.raw_os_error().unwrap_or(libc::EIO);
            }
            -1
        }
    }
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
    // The runtime clears its current handle after destructors run.
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
