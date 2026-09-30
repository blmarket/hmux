//! The legacy descriptor bridge uses the runtime's ordinary cancellation path.
use hmux_rt::{AsyncFd as _, Handle as _, Runtime as _, mio};
use std::cell::Cell;
use std::future::Future;
use std::io::Write;
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixStream;
use std::pin::pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};
use std::time::Duration;

#[test]
fn cancelled_readiness_does_not_consume_input_and_rearming_needs_no_new_edge() {
    let mut rt = mio::Runtime::new().unwrap();
    let (mut peer, fd) = UnixStream::pair().unwrap();
    fd.set_nonblocking(true).unwrap();
    let source = Rc::new(rt.handle().descriptor(Rc::new(fd.into())).unwrap());
    let s = source.clone();
    let cancelled = rt
        .handle()
        .spawn(async move {
            s.ready(true, false).await.unwrap();
        })
        .unwrap();
    rt.poll(Some(Duration::ZERO)).unwrap();
    drop(cancelled);
    peer.write_all(b"still present").unwrap();
    let calls = Rc::new(Cell::new(0));
    let observed = calls.clone();
    let task = rt
        .handle()
        .spawn(async move {
            for _ in 0..2 {
                assert_eq!(source.ready(true, false).await.unwrap(), (true, false));
                observed.set(observed.get() + 1);
            }
        })
        .unwrap();
    rt.poll(Some(Duration::ZERO)).unwrap();
    assert_eq!(calls.get(), 2);
    drop(task);
}

#[test]
fn regular_file_bridge_rejects_stale_runtime_handles() {
    let rt = mio::Runtime::new().unwrap();
    let handle = rt.handle();
    let fd = Rc::new(OwnedFd::from(std::fs::File::open("Cargo.toml").unwrap()));
    // Regular files do not need epoll registration.
    let source = handle.descriptor(fd.clone()).unwrap();
    drop(rt);
    assert!(handle.descriptor(fd).is_err());
    let mut wait = pin!(source.ready(true, false));
    assert!(matches!(
        wait.as_mut().poll(&mut Context::from_waker(Waker::noop())),
        Poll::Ready(Err(_))
    ));
}

#[test]
fn readiness_rearms_after_a_custom_operation_drains_the_descriptor() {
    use std::future::Future;
    use std::io::Read;
    use std::pin::pin;
    use std::task::{Context, Poll, Waker};

    let mut runtime = mio::Runtime::new().unwrap();
    let (mut peer, mut reader) = UnixStream::pair().unwrap();
    reader.set_nonblocking(true).unwrap();
    let handle = runtime.handle();
    let source = handle
        .io(Rc::new(reader.try_clone().unwrap().into()))
        .unwrap();
    drop(handle); // Readiness belongs to the registration, not the handle.
    let mut context = Context::from_waker(Waker::noop());
    assert!(matches!(
        pin!(source.writable()).poll(&mut context),
        Poll::Ready(Ok(()))
    ));
    peer.write_all(b"x").unwrap();
    runtime.poll(Some(Duration::ZERO)).unwrap();
    assert!(matches!(
        pin!(source.readable()).poll(&mut context),
        Poll::Ready(Ok(()))
    ));
    reader.read_exact(&mut [0]).unwrap();
    let mut wait = pin!(source.readable());
    assert!(wait.as_mut().poll(&mut context).is_pending());
    peer.write_all(b"y").unwrap();
    runtime.poll(Some(Duration::ZERO)).unwrap();
    assert!(matches!(
        wait.as_mut().poll(&mut context),
        Poll::Ready(Ok(()))
    ));
    let mut byte = [0];
    reader.read_exact(&mut byte).unwrap();
    assert_eq!(byte, *b"y");
}

#[test]
fn readiness_shares_waiters_with_byte_reads_and_runtime_drop_invalidates_it() {
    use hmux_rt::AsyncRead as _;
    use std::future::Future;
    use std::pin::pin;
    use std::task::{Context, Poll, Waker};

    let runtime = mio::Runtime::new().unwrap();
    let (_peer, reader) = UnixStream::pair().unwrap();
    reader.set_nonblocking(true).unwrap();
    let source = runtime.handle().io(Rc::new(reader.into())).unwrap();
    let mut context = Context::from_waker(Waker::noop());
    assert!(
        matches!(pin!(source.ready(false, false)).poll(&mut context),
        Poll::Ready(Err(error)) if error.kind() == std::io::ErrorKind::InvalidInput)
    );
    {
        let mut wait = pin!(source.readable());
        assert!(wait.as_mut().poll(&mut context).is_pending());
        assert!(matches!(pin!(source.read(&mut [0])).poll(&mut context),
            Poll::Ready(Err(error)) if error.kind() == std::io::ErrorKind::AlreadyExists));
    }
    let mut byte = [0];
    let mut read = pin!(source.read(&mut byte));
    assert!(read.as_mut().poll(&mut context).is_pending());
    drop(runtime);
    assert!(matches!(read.as_mut().poll(&mut context),
        Poll::Ready(Err(error)) if error.kind() == std::io::ErrorKind::BrokenPipe));
    assert!(matches!(pin!(source.writable()).poll(&mut context),
        Poll::Ready(Err(error)) if error.kind() == std::io::ErrorKind::BrokenPipe));
}
