#![feature(local_waker)]

use hmux_rt::{Handle as _, Runtime as _, mio};
use std::cell::Cell;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{ContextBuilder, LocalWake, LocalWaker, Waker};
use std::time::{Duration, Instant};

#[test]
#[should_panic(expected = "runtime initialized")]
fn current_handle_panics_before_initialization() {
    mio::Handle::current();
}

#[test]
#[should_panic(expected = "runtime initialized")]
fn current_handle_panics_after_shutdown() {
    let runtime = mio::Runtime::new().unwrap();
    drop(runtime);
    mio::Handle::current();
}

#[test]
fn initialization_registers_the_current_handle_on_its_thread() {
    assert!(!mio::Runtime::is_initialized());
    let mut runtime = mio::Runtime::new().unwrap();
    assert!(mio::Runtime::is_initialized());

    std::thread::spawn(|| {
        assert!(!mio::Runtime::is_initialized());
        let runtime = mio::Runtime::new().unwrap();
        assert!(mio::Runtime::is_initialized());
        drop(runtime);
        assert!(!mio::Runtime::is_initialized());
    })
    .join()
    .unwrap();

    let mut independent = mio::Runtime::new().unwrap();
    let called = Rc::new(Cell::new(false));
    let observed = called.clone();
    let task = independent
        .block_on(async {
            mio::Handle::current()
                .spawn(async move { observed.set(true) })
                .unwrap()
        })
        .unwrap();
    independent.poll(Some(Duration::ZERO)).unwrap();
    assert!(!called.get());
    drop(independent);
    assert!(mio::Runtime::is_initialized());

    runtime.poll(Some(Duration::ZERO)).unwrap();
    assert!(called.get());
    drop(task);
    drop(runtime);
    assert!(!mio::Runtime::is_initialized());
}

#[test]
fn dropping_an_auxiliary_runtime_preserves_a_new_registration() {
    let old = mio::Runtime::new().unwrap();
    let auxiliary = mio::Runtime::new().unwrap();
    drop(old);
    assert!(!mio::Runtime::is_initialized());

    let mut runtime = mio::Runtime::new().unwrap();
    drop(auxiliary);

    let handle = mio::Handle::current();
    let called = Rc::new(Cell::new(false));
    let observed = called.clone();
    let _task = handle.spawn(async move { observed.set(true) }).unwrap();
    runtime.poll(Some(Duration::ZERO)).unwrap();
    assert!(called.get());
    drop(runtime);
    assert!(!mio::Runtime::is_initialized());
    assert!(matches!(
        handle.spawn(async {}),
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe
    ));
}

#[test]
fn current_handle_remains_accessible_during_runtime_cleanup() {
    struct Wake(Rc<Cell<bool>>);

    impl LocalWake for Wake {
        fn wake(self: Rc<Self>) {}
    }

    impl Drop for Wake {
        fn drop(&mut self) {
            // Shutdown has invalidated scheduling, but lookup must still work.
            assert!(mio::Runtime::is_initialized());
            let handle = mio::Handle::current();
            assert!(matches!(
                handle.spawn(async {}),
                Err(error) if error.kind() == io::ErrorKind::BrokenPipe
            ));
            self.0.set(true);
        }
    }

    let runtime = mio::Runtime::new().unwrap();
    let dropped = Rc::new(Cell::new(false));
    let mut sleep = runtime
        .handle()
        .sleep_until(Instant::now() + Duration::from_secs(3600));
    {
        let wake = LocalWaker::from(Rc::new(Wake(dropped.clone())));
        let mut context = ContextBuilder::from_waker(Waker::noop())
            .local_waker(&wake)
            .build();
        assert!(Pin::new(&mut sleep).poll(&mut context).is_pending());
    }
    assert!(!dropped.get());
    drop(runtime);
    assert!(dropped.get());
    assert!(!mio::Runtime::is_initialized());
}
