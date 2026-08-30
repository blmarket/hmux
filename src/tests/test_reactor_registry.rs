use super::*;
use hmux_rt::TaskRuntime;
use std::ffi::c_void;
use std::io::Write as _;
use std::os::fd::AsRawFd as _;
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The libevent flag a timer callback was handed, which the registry no
/// longer passes on and these callbacks ignore.
const EV_TIMEOUT: c_short = 0x1;

unsafe fn count_timer(_fd: c_int, _events: c_short, argument: *mut c_void) {
    unsafe {
        (&*argument.cast::<AtomicUsize>()).fetch_add(1, Ordering::SeqCst);
    }
}

unsafe fn count_io(_fd: c_int, _events: c_short, argument: *mut c_void) {
    unsafe {
        (&*argument.cast::<AtomicUsize>()).fetch_add(1, Ordering::SeqCst);
    }
}

unsafe fn read_one(fd: c_int, _events: c_short, argument: *mut c_void) {
    unsafe {
        let mut byte = 0;
        assert_eq!(libc::read(fd, (&raw mut byte).cast(), 1), 1);
        (&*argument.cast::<AtomicUsize>()).fetch_add(1, Ordering::SeqCst);
    }
}

struct DisableContext {
    control: RuntimeControl,
    id: std::cell::Cell<usize>,
    calls: AtomicUsize,
}

unsafe fn disable_from_callback(_fd: c_int, _events: c_short, argument: *mut c_void) {
    unsafe {
        let context = &*argument.cast::<DisableContext>();
        if context.calls.fetch_add(1, Ordering::SeqCst) == 0 {
            context.control.disable_io(context.id.get());
        }
    }
}

unsafe fn count_signal(_signo: c_int, _events: c_short, argument: *mut c_void) {
    unsafe {
        (&*argument.cast::<AtomicUsize>()).fetch_add(1, Ordering::SeqCst);
    }
}

fn drive(runtime: &mut TaskRuntime) {
    runtime.dispatch(64).expect("dispatch");
    runtime.poll(Some(Duration::ZERO)).expect("poll");
    runtime.dispatch(64).expect("dispatch");
}

#[test]
fn timer_fires_once_and_can_be_rearmed() {
    let mut runtime = TaskRuntime::new().expect("runtime");
    let control = RuntimeControl::new();
    control.set_task_handle(runtime.handle());
    let calls = AtomicUsize::new(0);
    let argument = (&calls as *const AtomicUsize).cast_mut().cast::<c_void>();
    let id = control.allocate_timer(move || unsafe {
        count_timer(-1, EV_TIMEOUT, argument);
    });

    control.arm_timer(id, timeval::from_secs(0));
    drive(&mut runtime);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(!control.is_timer_armed(id));

    control.arm_timer(id, timeval::from_secs(0));
    drive(&mut runtime);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn disarming_a_timer_cancels_its_pending_task() {
    let mut runtime = TaskRuntime::new().expect("runtime");
    let control = RuntimeControl::new();
    let handle = runtime.handle();
    control.set_task_handle(handle.clone());
    let calls = AtomicUsize::new(0);
    let argument = (&calls as *const AtomicUsize).cast_mut().cast::<c_void>();
    let id = control.allocate_timer(move || unsafe {
        count_timer(-1, EV_TIMEOUT, argument);
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
    let calls = AtomicUsize::new(0);
    let id = control.allocate_io(source.as_raw_fd(), Interest::Read, WatchMode::Once, {
        let argument = (&calls as *const AtomicUsize).cast_mut().cast::<c_void>();
        move |fd, events| unsafe { count_io(fd, events, argument) }
    });
    control.enable_io(id);
    drive(&mut runtime);

    peer.write_all(b"ready").expect("write readiness");
    drive(&mut runtime);

    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(!control.is_io_enabled(id));
}

#[test]
fn persistent_io_rearms_after_a_callback_consumes_part_of_the_input() {
    let mut runtime = TaskRuntime::new().expect("runtime");
    let control = RuntimeControl::new();
    control.set_task_handle(runtime.handle());
    let (source, mut peer) = UnixStream::pair().expect("socket pair");
    source.set_nonblocking(true).expect("nonblocking source");
    peer.set_nonblocking(true).expect("nonblocking peer");
    let calls = AtomicUsize::new(0);
    let id = control.allocate_io(source.as_raw_fd(), Interest::Read, WatchMode::Persistent, {
        let argument = (&calls as *const AtomicUsize).cast_mut().cast::<c_void>();
        move |fd, events| unsafe { read_one(fd, events, argument) }
    });
    control.enable_io(id);
    drive(&mut runtime);

    peer.write_all(b"a").expect("first byte");
    drive(&mut runtime);
    peer.write_all(b"b").expect("second byte");
    drive(&mut runtime);

    assert_eq!(calls.load(Ordering::SeqCst), 2);
    control.disable_io(id);
}

#[test]
fn persistent_io_can_disable_itself_without_a_second_callback() {
    let mut runtime = TaskRuntime::new().expect("runtime");
    let control = RuntimeControl::new();
    control.set_task_handle(runtime.handle());
    let (source, mut peer) = UnixStream::pair().expect("socket pair");
    source.set_nonblocking(true).expect("nonblocking source");
    peer.set_nonblocking(true).expect("nonblocking peer");
    let context = DisableContext {
        control: control.clone(),
        id: std::cell::Cell::new(0),
        calls: AtomicUsize::new(0),
    };
    let id = control.allocate_io(source.as_raw_fd(), Interest::Read, WatchMode::Persistent, {
        let argument = (&context as *const DisableContext)
            .cast_mut()
            .cast::<c_void>();
        move |fd, events| unsafe { disable_from_callback(fd, events, argument) }
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
fn signal_watch_delivers_and_unwatch_is_immediate() {
    let mut runtime = TaskRuntime::new().expect("runtime");
    let control = RuntimeControl::new();
    control.set_task_handle(runtime.handle());
    let signal = libc::SIGRTMIN() + 7;
    let calls = AtomicUsize::new(0);
    let id = control.watch_signal(signal, {
        let argument = (&calls as *const AtomicUsize).cast_mut().cast::<c_void>();
        move |signo, events| unsafe { count_signal(signo, events, argument) }
    });
    drive(&mut runtime);

    assert_eq!(unsafe { libc::raise(signal) }, 0);
    drive(&mut runtime);
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    control.unwatch_signal(id);
    assert_eq!(unsafe { libc::raise(signal) }, 0);
    drive(&mut runtime);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn setting_a_timer_again_retires_the_previous_slot() {
    let mut runtime = TaskRuntime::new().expect("runtime");
    let control = current_control();
    control.set_task_handle(runtime.handle());
    let before = control.slot_counts().0;
    let calls = AtomicUsize::new(0);
    let argument = (&calls as *const AtomicUsize).cast_mut().cast::<c_void>();

    let mut timer = TimerHandle::ZERO;
    for _ in 0..8 {
        timer.set_callback(move || unsafe {
            count_timer(-1, EV_TIMEOUT, argument);
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
    let calls = AtomicUsize::new(0);
    let argument = (&calls as *const AtomicUsize).cast_mut().cast::<c_void>();

    let mut watch = IoHandle::ZERO;
    for _ in 0..8 {
        watch.disable();
        watch.set_callback(
            left.as_raw_fd(),
            Interest::Read,
            WatchMode::Once,
            move |fd, events| unsafe { read_one(fd, events, argument) },
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
    let calls = AtomicUsize::new(0);
    let argument = (&calls as *const AtomicUsize).cast_mut().cast::<c_void>();

    let mut watch = SignalHandle::ZERO;
    for _ in 0..8 {
        watch.set_callback(signal, move |signo, events| unsafe {
            count_signal(signo, events, argument)
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
