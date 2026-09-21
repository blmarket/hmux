//! Application compatibility over hmux-rt and hmux-buffer.
//!
//! Embedded C-layout handles own no Rust resources. Registrations own cancellable
//! local tasks; callbacks run without registry borrows. Descriptor leases are
//! duplicated once per live endpoint and close after an executing poll finishes.
#![allow(clippy::missing_safety_doc)]
mod buffer;
mod events;
mod streams;
pub use buffer::*;
pub use events::*;
use hmux_rt::{Handle as _, Runtime as _};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::os::fd::{FromRawFd, OwnedFd};
use std::rc::{Rc, Weak};
use std::time::Duration;
pub use streams::*;

#[repr(C)]
pub struct event_base {
    _private: [u8; 0],
}
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
fn handle() -> hmux_rt::mio::Handle {
    HANDLE.with(|h| h.borrow().as_ref().expect("runtime initialized").clone())
}
fn ensure_runtime() {
    if HANDLE.with(|h| h.borrow().is_some()) {
        return;
    }
    let runtime = hmux_rt::mio::Runtime::new().expect("hmux-rt initialization");
    PID.with(|p| p.set(std::process::id()));
    HANDLE.with(|h| *h.borrow_mut() = Some(runtime.handle()));
    HOST.with(|h| *h.borrow_mut() = Some(runtime));
}
fn descriptor(fd: i32) -> std::io::Result<Rc<hmux_rt::mio::Descriptor>> {
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
fn run_once(nonblocking: bool) {
    ensure_runtime();
    let mut runtime = HOST.with(|h| h.borrow_mut().take().expect("recursive runtime dispatch"));
    runtime
        .poll(if nonblocking {
            Some(Duration::ZERO)
        } else {
            None
        })
        .expect("hmux-rt poll");
    HOST.with(|h| *h.borrow_mut() = Some(runtime));
}
pub fn shutdown_runtime() {
    events::clear();
    streams::clear();
    FDS.with(|f| f.borrow_mut().clear());
    HANDLE.with(|h| h.borrow_mut().take());
    HOST.with(|h| h.borrow_mut().take());
}
async fn yield_now() {
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
