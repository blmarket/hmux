use super::*;
use hmux_rt::TaskRuntime;
use std::io::Write as _;
use std::os::fd::AsRawFd as _;
use std::os::unix::net::UnixStream;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

fn read_one(fd: c_int, calls: &AtomicUsize) {
    unsafe {
        let mut byte = 0;
        assert_eq!(libc::read(fd, (&raw mut byte).cast(), 1), 1);
    }
    calls.fetch_add(1, Ordering::SeqCst);
}

struct DisableContext {
    control: RuntimeControl,
    id: std::cell::Cell<usize>,
    calls: AtomicUsize,
}

fn drive(runtime: &mut TaskRuntime) {
    runtime.dispatch(64).expect("dispatch");
    runtime.poll(Some(Duration::ZERO)).expect("poll");
    runtime.dispatch(64).expect("dispatch");
}

#[test]
fn disarming_a_timer_cancels_its_pending_task() {
    let mut runtime = TaskRuntime::new().expect("runtime");
    let control = RuntimeControl::new();
    let handle = runtime.handle();
    control.set_task_handle(handle.clone());
    let calls = Rc::new(AtomicUsize::new(0));
    let callback_calls = Rc::clone(&calls);
    let id = control.allocate_timer(move || {
        callback_calls.fetch_add(1, Ordering::SeqCst);
    });

    control.arm_timer(id, timeval::from_secs(60));
    control.disarm_timer(id);
    runtime.dispatch(64).expect("dispatch");
    runtime.poll(Some(Duration::ZERO)).expect("poll");
    runtime.dispatch(64).expect("dispatch");

    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert!(!handle.is_active(1));
    assert!(!control.is_timer_armed(id));
}

#[test]
fn deferred_calls_wait_for_the_following_epoch() {
    let control = RuntimeControl::new();
    let calls = std::sync::Arc::new(AtomicUsize::new(0));
    let epoch = control.begin_epoch();
    let deferred_calls = std::sync::Arc::clone(&calls);
    control.defer(move || {
        deferred_calls.fetch_add(1, Ordering::SeqCst);
    });

    assert!(control.take_ready_deferred(epoch).is_empty());
    let ready = control.take_ready_deferred(epoch + 1);
    assert_eq!(ready.len(), 1);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[test]
fn one_shot_io_disables_before_reentrant_callback_work() {
    let mut runtime = TaskRuntime::new().expect("runtime");
    let control = RuntimeControl::new();
    control.set_task_handle(runtime.handle());
    let (source, mut peer) = UnixStream::pair().expect("socket pair");
    source.set_nonblocking(true).expect("nonblocking source");
    peer.set_nonblocking(true).expect("nonblocking peer");
    let calls = Rc::new(AtomicUsize::new(0));
    let callback_calls = Rc::clone(&calls);
    let id = control.allocate_io(source.as_raw_fd(), Interest::Read, WatchMode::Once, {
        move |_fd, _events| {
            callback_calls.fetch_add(1, Ordering::SeqCst);
        }
    });
    control.enable_io(id);
    drive(&mut runtime);

    peer.write_all(b"ready").expect("write readiness");
    drive(&mut runtime);

    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(!control.is_io_enabled(id));
}

#[test]
fn persistent_io_can_disable_itself_without_a_second_callback() {
    let mut runtime = TaskRuntime::new().expect("runtime");
    let control = RuntimeControl::new();
    control.set_task_handle(runtime.handle());
    let (source, mut peer) = UnixStream::pair().expect("socket pair");
    source.set_nonblocking(true).expect("nonblocking source");
    peer.set_nonblocking(true).expect("nonblocking peer");
    let context = Rc::new(DisableContext {
        control: control.clone(),
        id: std::cell::Cell::new(0),
        calls: AtomicUsize::new(0),
    });
    let callback_context = Rc::downgrade(&context);
    let id = control.allocate_io(source.as_raw_fd(), Interest::Read, WatchMode::Persistent, {
        move |_fd, _events| {
            if let Some(callback_context) = callback_context.upgrade() {
                if callback_context.calls.fetch_add(1, Ordering::SeqCst) == 0 {
                    callback_context
                        .control
                        .disable_io(callback_context.id.get());
                }
            }
        }
    });
    context.id.set(id);
    control.enable_io(id);
    drive(&mut runtime);

    peer.write_all(b"ready").expect("write readiness");
    drive(&mut runtime);

    assert_eq!(context.calls.load(Ordering::SeqCst), 1);
    assert!(!control.is_io_enabled(id));
}

#[test]
fn setting_a_timer_again_retires_the_previous_slot() {
    let mut runtime = TaskRuntime::new().expect("runtime");
    let control = current_control();
    control.set_task_handle(runtime.handle());
    let before = control.slot_counts().0;
    let calls = Rc::new(AtomicUsize::new(0));

    let mut timer = TimerHandle::ZERO;
    for _ in 0..8 {
        let callback_calls = Rc::clone(&calls);
        timer.set_callback(move || {
            callback_calls.fetch_add(1, Ordering::SeqCst);
        });
        timer.disarm();
    }
    assert_eq!(control.slot_counts().0, before + 1);

    timer.arm(timeval::from_secs(0));
    drive(&mut runtime);
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    control.release_timer(timer.0);
    assert_eq!(control.slot_counts().0, before);
}

#[test]
fn setting_a_watch_again_retires_the_previous_slot() {
    let mut runtime = TaskRuntime::new().expect("runtime");
    let control = current_control();
    control.set_task_handle(runtime.handle());
    let before = control.slot_counts().1;
    let (left, mut right) = UnixStream::pair().expect("pair");
    let calls = Rc::new(AtomicUsize::new(0));

    let mut watch = IoHandle::ZERO;
    for _ in 0..8 {
        let callback_calls = Rc::clone(&calls);
        watch.disable();
        watch.set_callback(
            left.as_raw_fd(),
            Interest::Read,
            WatchMode::Once,
            move |fd, _events| read_one(fd, &callback_calls),
        );
        watch.enable();
    }
    assert_eq!(control.slot_counts().1, before + 1);

    right.write_all(b"x").expect("write");
    drive(&mut runtime);
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    control.release_io(watch.0);
    assert_eq!(control.slot_counts().1, before);
}

#[test]
fn watching_a_signal_again_retires_the_previous_slot() {
    let mut runtime = TaskRuntime::new().expect("runtime");
    let control = current_control();
    control.set_task_handle(runtime.handle());
    let before = control.slot_counts().2;
    let signal = libc::SIGRTMIN() + 8;
    let calls = Rc::new(AtomicUsize::new(0));

    let mut watch = SignalHandle::ZERO;
    for _ in 0..8 {
        let callback_calls = Rc::clone(&calls);
        watch.set_callback(signal, move |_signo, _events| {
            callback_calls.fetch_add(1, Ordering::SeqCst);
        });
    }
    assert_eq!(control.slot_counts().2, before + 1);
    drive(&mut runtime);

    assert_eq!(unsafe { libc::raise(signal) }, 0);
    drive(&mut runtime);
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    control.release_signal(watch.0);
    assert_eq!(control.slot_counts().2, before);
}

struct DisarmOnDrop(TimerHandle);

impl Drop for DisarmOnDrop {
    fn drop(&mut self) {
        self.0.disarm();
    }
}

#[test]
fn shutdown_drops_what_the_reactor_holds_while_it_can_still_answer() {
    let control = current_control();

    let mut armed = TimerHandle::ZERO;
    armed.set_callback(|| {});
    armed.arm(timeval {
        tv_sec: 60,
        tv_usec: 0,
    });

    let deferred = DisarmOnDrop(armed);
    control.defer(move || drop(deferred));

    let held = DisarmOnDrop(armed);
    let mut owner = TimerHandle::ZERO;
    owner.set_callback(move || {
        let _ = &held;
    });

    crate::reactor::shutdown();

    assert_eq!(control.slot_counts(), (0, 0, 0));
}
