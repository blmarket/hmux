//! Executor and readiness contracts; process-global behavior is tested separately.
#![feature(local_waker)]

use hmux_rt::mio;
use hmux_rt::{AsyncRead, AsyncWrite, Handle, Runtime};
use std::cell::{Cell, RefCell};
use std::future::{Future, poll_fn};
use std::io::{self, Write};
use std::os::fd::{AsRawFd, OwnedFd};
use std::os::unix::net::UnixStream;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};

fn tick(runtime: &mut mio::Runtime) {
    runtime.poll(Some(Duration::ZERO)).unwrap()
}
fn poll<F: Future + Unpin>(future: &mut F) -> Poll<F::Output> {
    Pin::new(future).poll(&mut Context::from_waker(Waker::noop()))
}
fn pair() -> (UnixStream, Rc<OwnedFd>) {
    let (writer, reader) = UnixStream::pair().unwrap();
    reader.set_nonblocking(true).unwrap();
    (writer, Rc::new(reader.into()))
}

struct Dropped(Rc<Cell<usize>>);
impl Drop for Dropped {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

#[test]
fn task_handles_observe_completion_and_runtime_shutdown() {
    let mut runtime = mio::Runtime::new().unwrap();
    let task = runtime.handle().spawn(async {}).unwrap();
    assert!(task.is_pending());
    tick(&mut runtime);
    assert!(!task.is_pending());

    let count = Rc::new(Cell::new(0));
    let spy = Dropped(count.clone());
    let pending = runtime
        .handle()
        .spawn(async move {
            let _spy = spy;
            std::future::pending::<()>().await;
        })
        .unwrap();
    assert!(pending.is_pending());
    drop(runtime);
    assert!(!pending.is_pending());
    assert_eq!(count.get(), 0, "the task still owns its future");
    drop(pending);
    assert_eq!(count.get(), 1);
}

#[test]
fn deferred_callbacks_run_or_release_captures_on_runtime_shutdown() {
    let mut runtime = mio::Runtime::new().unwrap();
    let count = Rc::new(Cell::new(0));
    let spy = Dropped(count.clone());
    runtime.handle().defer(move || drop(spy)).unwrap();
    assert_eq!(count.get(), 0);
    tick(&mut runtime);
    assert_eq!(count.get(), 1);

    let spy = Dropped(count.clone());
    runtime
        .handle()
        .defer(move || {
            let _spy = spy;
            panic!("shutdown must not dispatch callbacks");
        })
        .unwrap();
    drop(runtime);
    assert_eq!(count.get(), 2);
}

#[test]
fn runtime_drop_releases_callbacks_queued_by_capture_destructors() {
    struct EnqueueOnDrop {
        handle: mio::Handle,
        dropped: Rc<Cell<usize>>,
    }
    impl Drop for EnqueueOnDrop {
        fn drop(&mut self) {
            let spy = Dropped(self.dropped.clone());
            let _ = self.handle.defer(move || {
                let _spy = spy;
                panic!("cleanup must not dispatch queued work");
            });
        }
    }

    let runtime = mio::Runtime::new().unwrap();
    let handle = runtime.handle();
    let count = Rc::new(Cell::new(0));
    let enqueue = EnqueueOnDrop {
        handle: handle.clone(),
        dropped: count.clone(),
    };
    handle
        .defer(move || {
            let _enqueue = enqueue;
            panic!("shutdown must not dispatch callbacks");
        })
        .unwrap();
    drop(runtime);
    assert_eq!(count.get(), 1);
    let spy = Dropped(count.clone());
    assert!(handle.defer(move || drop(spy)).is_err());
    assert_eq!(count.get(), 2);
}

#[test]
fn ready_queue_orders_tasks_and_deferred_callbacks_without_inline_dispatch() {
    let mut runtime = mio::Runtime::new().unwrap();
    let handle = runtime.handle();
    let order = Rc::new(RefCell::new(Vec::new()));
    let observed = order.clone();
    handle.defer(move || observed.borrow_mut().push(1)).unwrap();
    let observed = order.clone();
    let _task = handle
        .spawn(async move {
            observed.borrow_mut().push(2);
        })
        .unwrap();
    let observed = order.clone();
    let nested = handle.clone();
    handle
        .defer(move || {
            observed.borrow_mut().push(3);
            let last = observed.clone();
            nested.defer(move || last.borrow_mut().push(5)).unwrap();
            observed.borrow_mut().push(4);
        })
        .unwrap();
    assert!(order.borrow().is_empty());
    tick(&mut runtime);
    assert_eq!(*order.borrow(), [1, 2, 3, 4, 5]);
}

#[test]
fn cancellation_drops_unpolled_and_parked_futures_without_driving() {
    let mut runtime = mio::Runtime::new().unwrap();
    let handle = runtime.handle();
    let count = Rc::new(Cell::new(0));
    let spy = Dropped(count.clone());
    let task = handle
        .spawn(async move {
            let _spy = spy;
            panic!("cancelled before polling");
        })
        .unwrap();
    drop(task);
    assert_eq!(count.get(), 1);
    tick(&mut runtime);

    let (_writer, fd) = pair();
    let source = handle.io(fd.clone()).unwrap();
    let spy = Dropped(count.clone());
    let task = handle
        .spawn(async move {
            let _spy = spy;
            source.read(&mut [0; 1]).await.unwrap();
        })
        .unwrap();
    tick(&mut runtime);
    drop(task);
    assert_eq!(count.get(), 2);
    assert_eq!(Rc::strong_count(&fd), 1);
    // The kernel registration has been removed, with no intervening turn.
    let _replacement = handle.io(fd).unwrap();
}

#[test]
fn self_waking_work_returns_control_and_cancelled_wakes_are_inert() {
    let mut runtime = mio::Runtime::new().unwrap();
    let polls = Rc::new(Cell::new(0));
    let saved = Rc::new(RefCell::new(None));
    let count = polls.clone();
    let wake = saved.clone();
    let task = runtime
        .handle()
        .spawn(poll_fn(move |cx| {
            count.set(count.get() + 1);
            *wake.borrow_mut() = Some(cx.local_waker().clone());
            for _ in 0..20 {
                cx.local_waker().wake_by_ref();
            }
            Poll::Pending
        }))
        .unwrap();
    tick(&mut runtime);
    let polled = polls.get();
    assert!(polled > 0);
    drop(task);
    saved.borrow().as_ref().unwrap().wake_by_ref();
    tick(&mut runtime);
    assert_eq!(polls.get(), polled);
}

#[test]
fn ordinary_waker_is_deliberately_inert() {
    let mut runtime = mio::Runtime::new().unwrap();
    let polls = Rc::new(Cell::new(0));
    let count = polls.clone();
    let _task = runtime
        .handle()
        .spawn(poll_fn(move |cx| {
            count.set(count.get() + 1);
            cx.waker().wake_by_ref();
            Poll::Pending
        }))
        .unwrap();
    tick(&mut runtime);
    tick(&mut runtime);
    assert_eq!(polls.get(), 1);
}

#[test]
fn self_cancellation_finishes_current_poll_then_drops_resources() {
    let mut runtime = mio::Runtime::new().unwrap();
    let owner = Rc::new(RefCell::new(None));
    let count = Rc::new(Cell::new(0));
    let spy = Dropped(count.clone());
    let slot = owner.clone();
    let observed = count.clone();
    *owner.borrow_mut() = Some(
        runtime
            .handle()
            .spawn(poll_fn(move |cx| {
                let _keep = &spy;
                slot.borrow_mut().take();
                assert_eq!(observed.get(), 0);
                cx.local_waker().wake_by_ref();
                Poll::Pending
            }))
            .unwrap(),
    );
    tick(&mut runtime);
    assert_eq!(count.get(), 1);
    tick(&mut runtime);
}

#[test]
fn future_destructors_can_cancel_other_tasks_and_spawn() {
    struct Cascade {
        child: Option<mio::Task>,
        handle: mio::Handle,
    }
    impl Drop for Cascade {
        fn drop(&mut self) {
            self.child.take();
            let transient = self.handle.spawn(async {}).unwrap();
            drop(transient);
        }
    }
    let runtime = mio::Runtime::new().unwrap();
    let count = Rc::new(Cell::new(0));
    let spy = Dropped(count.clone());
    let child = runtime
        .handle()
        .spawn(async move {
            let _spy = spy;
        })
        .unwrap();
    let cascade = Cascade {
        child: Some(child),
        handle: runtime.handle(),
    };
    let parent = runtime
        .handle()
        .spawn(async move {
            let _cascade = cascade;
        })
        .unwrap();
    drop(parent);
    assert_eq!(count.get(), 1);
}

#[test]
fn timers_wake_in_deadline_and_insertion_order() {
    let mut runtime = mio::Runtime::new().unwrap();
    let seen = Rc::new(RefCell::new(Vec::new()));
    let deadline = Instant::now() + Duration::from_millis(15);
    let mut tasks = Vec::new();
    for i in 0..4 {
        let wait = runtime.handle().sleep_until(deadline);
        let seen = seen.clone();
        tasks.push(
            runtime
                .handle()
                .spawn(async move {
                    wait.await.unwrap();
                    seen.borrow_mut().push(i);
                })
                .unwrap(),
        );
    }
    tick(&mut runtime);
    runtime.poll(Some(Duration::from_millis(100))).unwrap();
    assert_eq!(*seen.borrow(), [0, 1, 2, 3]);
}

fn read_bytes(
    source: &impl AsyncRead,
    limit: usize,
) -> Pin<Box<impl Future<Output = io::Result<Vec<u8>>> + '_>> {
    Box::pin(async move {
        let mut bytes = vec![0; limit];
        let count = source.read(&mut bytes).await?;
        bytes.truncate(count);
        Ok(bytes)
    })
}

#[test]
fn partial_reads_retain_readiness_across_a_pause() {
    let mut runtime = mio::Runtime::new().unwrap();
    let (mut writer, fd) = pair();
    let source = runtime.handle().io(fd.clone()).unwrap();
    let mut wait = read_bytes(&source, 2);
    assert!(poll(&mut wait).is_pending());
    writer.write_all(b"first").unwrap();
    tick(&mut runtime);
    assert!(matches!(poll(&mut wait), Poll::Ready(Ok(bytes)) if bytes == b"fi"));
    drop(wait);
    tick(&mut runtime);
    assert!(
        matches!(poll(&mut read_bytes(&source, 32)), Poll::Ready(Ok(bytes)) if bytes == b"rst")
    );
    let mut wait = read_bytes(&source, 32);
    assert!(poll(&mut wait).is_pending());
    writer.write_all(b"new").unwrap();
    tick(&mut runtime);
    assert!(matches!(poll(&mut wait), Poll::Ready(Ok(bytes)) if bytes == b"new"));
}

#[test]
fn directional_waiter_conflicts_cancellation_and_wouldblock() {
    let mut runtime = mio::Runtime::new().unwrap();
    let (mut writer, fd) = pair();
    let source = runtime.handle().io(fd.clone()).unwrap();
    assert!(
        matches!(runtime.handle().io(fd.clone()), Err(e) if e.kind() == io::ErrorKind::AlreadyExists)
    );
    let mut first = read_bytes(&source, 1);
    assert!(poll(&mut first).is_pending());
    assert!(
        matches!(poll(&mut read_bytes(&source, 1)), Poll::Ready(Err(e)) if e.kind() == io::ErrorKind::AlreadyExists)
    );
    drop(first);
    let mut replacement = read_bytes(&source, 1);
    assert!(poll(&mut replacement).is_pending());
    tick(&mut runtime);
    assert!(matches!(
        poll(&mut Box::pin(source.write(b"w"))),
        Poll::Ready(Ok(1))
    ));
    writer.write_all(b"x").unwrap();
    tick(&mut runtime);
    assert!(matches!(poll(&mut replacement), Poll::Ready(Ok(bytes)) if bytes == b"x"));
    // Completed futures release the waiter even before they are dropped.
    let mut next = read_bytes(&source, 1);
    assert!(poll(&mut next).is_pending());
    writer.write_all(b"y").unwrap();
    tick(&mut runtime);
    assert!(matches!(poll(&mut next), Poll::Ready(Ok(bytes)) if bytes == b"y"));
}

#[test]
fn kernel_readiness_is_serviced_despite_a_self_waking_task() {
    let mut runtime = mio::Runtime::new().unwrap();
    let spinner = runtime
        .handle()
        .spawn(poll_fn(|cx| {
            cx.local_waker().wake_by_ref();
            Poll::Pending
        }))
        .unwrap();
    let (mut writer, fd) = pair();
    let source = runtime.handle().io(fd).unwrap();
    let done = Rc::new(Cell::new(false));
    let mark = done.clone();
    let _reader = runtime
        .handle()
        .spawn(async move {
            source.read(&mut [0; 1]).await.unwrap();
            mark.set(true);
        })
        .unwrap();
    tick(&mut runtime);
    writer.write_all(b"x").unwrap();
    for _ in 0..4 {
        tick(&mut runtime);
    }
    assert!(done.get());
    drop(spinner);
}

#[test]
fn runtime_drop_releases_resources_and_invalidates_old_leaves() {
    let runtime = mio::Runtime::new().unwrap();
    let handle = runtime.handle();
    let (_writer, fd) = pair();
    let source = handle.io(fd.clone()).unwrap();
    let mut sleep = handle.sleep_until(Instant::now());
    drop(runtime);
    assert_eq!(Rc::strong_count(&fd), 1);
    assert!(
        matches!(poll(&mut sleep), Poll::Ready(Err(e)) if e.kind() == io::ErrorKind::BrokenPipe)
    );
    assert!(
        matches!(poll(&mut Box::pin(source.read(&mut [0; 1]))), Poll::Ready(Err(e)) if e.kind() == io::ErrorKind::BrokenPipe)
    );
    assert!(handle.spawn(async {}).is_err());
}

#[test]
fn panic_poisons_runtime_and_resets_drive_state() {
    let mut runtime = mio::Runtime::new().unwrap();
    let _task = runtime
        .handle()
        .spawn(async {
            panic!("probe");
        })
        .unwrap();
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| tick(&mut runtime))).is_err());
    assert!(runtime.handle().spawn(async {}).is_err());
    assert!(runtime.poll(Some(Duration::ZERO)).is_err());
}

#[test]
fn descriptor_eof_and_unsupported_regular_files_are_explicit() {
    let mut runtime = mio::Runtime::new().unwrap();
    let (writer, fd) = pair();
    let source = runtime.handle().io(fd.clone()).unwrap();
    drop(writer);
    tick(&mut runtime);
    assert!(matches!(
        poll(&mut Box::pin(source.read(&mut [0; 1]))),
        Poll::Ready(Ok(_))
    ));
    let mut byte = 0u8;
    assert_eq!(
        unsafe { libc::read(fd.as_raw_fd(), (&mut byte as *mut u8).cast(), 1) },
        0
    );

    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let file = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
            .unwrap();
        assert!(
            matches!(runtime.handle().io(Rc::new(file.into())), Err(e) if e.kind() == io::ErrorKind::Unsupported)
        );
    }
}

#[test]
fn empty_runtime_returns_and_same_process_reset_is_rejected() {
    let mut runtime = mio::Runtime::new().unwrap();
    runtime.poll(None).unwrap();
    assert!(
        matches!(runtime.reset_after_fork(), Err(e) if e.kind() == io::ErrorKind::InvalidInput)
    );
}
