//! Application compatibility and process ownership for hmux-rt.
#![allow(clippy::missing_safety_doc)]
mod buffer;
mod events;
mod streams;
pub use buffer::*;
pub use events::*;
pub use hmux_rt::ByteBuffer;
use hmux_rt::{registry::RuntimeControl, stream::StreamRegistry, TaskRuntime};
use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Duration;
pub use streams::*;
thread_local! {
    static CONTROL: RuntimeControl = RuntimeControl::new();
    static STREAMS: StreamRegistry = StreamRegistry::new();
    static HOST: RefCell<Option<(libc::pid_t,TaskRuntime)>> = const { RefCell::new(None) };
    static BUFFERS: RefCell<HashMap<usize,usize>> = RefCell::new(HashMap::new());
}
fn control() -> RuntimeControl {
    CONTROL.with(Clone::clone)
}
fn streams() -> StreamRegistry {
    STREAMS.with(Clone::clone)
}
fn wake_buffer(buffer: *mut ByteBuffer) {
    let id = BUFFERS.with(|b| b.borrow().get(&(buffer as usize)).copied());
    if let Some(id) = id {
        streams().wake(id);
    }
}
fn ensure_runtime(force: bool) {
    HOST.with(|host| {
        let mut host = host.borrow_mut();
        let pid = unsafe { libc::getpid() };
        if !force && host.as_ref().is_some_and(|(p, _)| *p == pid) {
            return;
        }
        *host = None;
        let runtime = TaskRuntime::new().expect("hmux-rt initialization failed");
        control().respawn_active(&runtime.handle());
        streams().respawn_active(&runtime.handle());
        *host = Some((pid, runtime));
    });
}
fn run_once() {
    ensure_runtime(false);
    let (pid, mut runtime) =
        HOST.with(|h| h.borrow_mut().take().expect("recursive runtime dispatch"));
    let epoch = control().begin_epoch();
    runtime.dispatch(64).expect("hmux-rt dispatch failed");
    for deferred in control().take_ready_deferred(epoch) {
        (deferred.callback)();
    }
    // Always poll: runnable tasks must not starve descriptors or timers.
    runtime
        .poll(Some(if runtime.pending() == 0 {
            Duration::from_millis(10)
        } else {
            Duration::ZERO
        }))
        .expect("hmux-rt poll failed");
    runtime.dispatch(64).expect("hmux-rt dispatch failed");
    runtime
        .flush_cancelled()
        .expect("hmux-rt cancellation failed");
    HOST.with(|h| *h.borrow_mut() = Some((pid, runtime)));
}
/// Release cancelled descriptor/signal tasks without invoking application code.
pub fn flush_cancelled() {
    HOST.with(|h| {
        let mut host = h.borrow_mut();
        if host
            .as_ref()
            .is_some_and(|(pid, _)| *pid != unsafe { libc::getpid() })
        {
            // Drop inherited leaves without issuing epoll_ctl on the parent's
            // kernel poller. event_reinit will rebuild surviving registrations.
            *host = None;
            return;
        }
        if let Some((_, runtime)) = host.as_mut() {
            runtime
                .flush_cancelled()
                .expect("hmux-rt cancellation failed");
        }
    });
}
pub fn shutdown_runtime() {
    control().shutdown();
    streams().shutdown();
    HOST.with(|h| *h.borrow_mut() = None);
    BUFFERS.with(|b| b.borrow_mut().clear());
}
