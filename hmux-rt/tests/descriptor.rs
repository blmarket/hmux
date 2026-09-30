//! The legacy descriptor bridge uses the runtime's ordinary cancellation path.
use hmux_rt::{Handle as _, Runtime as _, mio};
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
    let rt = mio::Runtime::new().unwrap();
    let (mut peer, fd) = UnixStream::pair().unwrap();
    fd.set_nonblocking(true).unwrap();
    let source = Rc::new(mio::Descriptor::new(&rt.handle(), Rc::new(fd.into())).unwrap());
    let s = source.clone();
    let cancelled = rt
        .handle()
        .spawn(async move {
            s.wait(true, false).await.unwrap();
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
                assert_eq!(source.wait(true, false).await.unwrap(), (true, false));
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
    let source = mio::Descriptor::new(&handle, fd.clone()).unwrap();
    drop(rt);
    assert!(mio::Descriptor::new(&handle, fd).is_err());
    let mut wait = pin!(source.wait(true, false));
    assert!(matches!(
        wait.as_mut().poll(&mut Context::from_waker(Waker::noop())),
        Poll::Ready(Err(_))
    ));
}
