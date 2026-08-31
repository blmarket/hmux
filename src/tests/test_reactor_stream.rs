use super::*;
use bytes::Bytes;
use hmux_rt::TaskRuntime;
use std::io::Read as _;
use std::io::Write as _;
use std::os::fd::AsRawFd as _;
use std::os::unix::net::UnixStream;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// A callback that counts the times it is reached.
fn count(calls: &Rc<AtomicUsize>) -> StreamCb {
    let calls = calls.clone();
    Rc::new(move |_stream| {
        calls.fetch_add(1, Ordering::SeqCst);
    })
}

/// A read callback that counts and then gives the stream up from inside
/// itself.
fn free_on_read(registry: &StreamRegistry, calls: &Rc<AtomicUsize>) -> StreamCb {
    let registry = registry.clone();
    let calls = calls.clone();
    Rc::new(move |stream: Stream| {
        calls.fetch_add(1, Ordering::SeqCst);
        registry.free(stream.0);
    })
}

/// An error callback that records what it was told and gives the stream up.
fn free_on_error(registry: &StreamRegistry, events: &Rc<AtomicUsize>) -> StreamErrorCb {
    let registry = registry.clone();
    let events = events.clone();
    Rc::new(move |stream: Stream, what: c_short| {
        events.store(what as usize, Ordering::SeqCst);
        registry.free(stream.0);
    })
}

fn drive(runtime: &mut TaskRuntime) {
    runtime.dispatch(64).expect("dispatch");
    runtime.poll(Some(Duration::ZERO)).expect("poll");
    runtime.dispatch(64).expect("dispatch");
}

fn registry_with_runtime(runtime: &TaskRuntime) -> StreamRegistry {
    let registry = StreamRegistry::new();
    registry.inner.borrow_mut().task_handle = Some(runtime.handle());
    registry
}

#[test]
fn read_burst_is_drained_before_one_callback() {
    let mut runtime = TaskRuntime::new().expect("runtime");
    let registry = registry_with_runtime(&runtime);
    let (source, mut peer) = UnixStream::pair().expect("socket pair");
    source.set_nonblocking(true).expect("nonblocking source");
    peer.set_nonblocking(true).expect("nonblocking peer");
    let calls = Rc::new(AtomicUsize::new(0));
    let id = registry.allocate(source.as_raw_fd(), Some(count(&calls)), None, None);
    registry.enable(id, Interest::Read);
    drive(&mut runtime);

    peer.write_all(b"first").expect("first write");
    peer.write_all(b"second").expect("second write");
    drive(&mut runtime);

    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(registry.input_len(id), 11);
    let data = registry.with_input(id, |buffer| buffer.copy_to_bytes(buffer.len()));
    assert_eq!(data.expect("stream"), Bytes::from_static(b"firstsecond"));
    registry.free(id);
}

#[test]
fn queued_output_wakes_a_write_only_stream_and_notifies_at_low_water() {
    let mut runtime = TaskRuntime::new().expect("runtime");
    let registry = registry_with_runtime(&runtime);
    let (source, mut peer) = UnixStream::pair().expect("socket pair");
    source.set_nonblocking(true).expect("nonblocking source");
    peer.set_nonblocking(true).expect("nonblocking peer");
    let calls = Rc::new(AtomicUsize::new(0));
    let id = registry.allocate(source.as_raw_fd(), None, Some(count(&calls)), None);
    registry.set_watermark(id, 0, 0);
    assert!(registry.write(id, b"output"));
    registry.enable(id, Interest::Write);
    drive(&mut runtime);

    let mut output = [0; 6];
    peer.read_exact(&mut output).expect("read output");
    assert_eq!(&output, b"output");
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(registry.output_len(id), 0);
    registry.free(id);
}

#[test]
fn enabling_an_empty_write_stream_runs_its_callback() {
    let mut runtime = TaskRuntime::new().expect("runtime");
    let registry = registry_with_runtime(&runtime);
    let (source, peer) = UnixStream::pair().expect("socket pair");
    source.set_nonblocking(true).expect("nonblocking source");
    peer.set_nonblocking(true).expect("nonblocking peer");
    let calls = Rc::new(AtomicUsize::new(0));
    let id = registry.allocate(source.as_raw_fd(), None, Some(count(&calls)), None);

    registry.enable(id, Interest::Write);
    drive(&mut runtime);

    assert_eq!(calls.load(Ordering::SeqCst), 1);
    registry.free(id);
}

#[test]
fn descriptorless_streams_remain_inert_until_freed() {
    let mut runtime = TaskRuntime::new().expect("runtime");
    let registry = registry_with_runtime(&runtime);
    let id = registry.allocate(-1, None, None, None);

    registry.enable(id, Interest::ReadWrite);
    drive(&mut runtime);

    assert!(registry.lookup(id).is_some());
    registry.free(id);
}

#[test]
fn read_callback_can_free_the_stream() {
    let mut runtime = TaskRuntime::new().expect("runtime");
    let registry = registry_with_runtime(&runtime);
    let (source, mut peer) = UnixStream::pair().expect("socket pair");
    source.set_nonblocking(true).expect("nonblocking source");
    peer.set_nonblocking(true).expect("nonblocking peer");
    let calls = Rc::new(AtomicUsize::new(0));
    let id = registry.allocate(
        source.as_raw_fd(),
        Some(free_on_read(&registry, &calls)),
        None,
        None,
    );
    registry.enable(id, Interest::Read);
    drive(&mut runtime);

    peer.write_all(b"read").expect("write input");
    drive(&mut runtime);

    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(registry.lookup(id).is_none());
}

#[test]
fn error_callback_can_free_the_stream_after_eof() {
    let mut runtime = TaskRuntime::new().expect("runtime");
    let registry = registry_with_runtime(&runtime);
    let (source, mut peer) = UnixStream::pair().expect("socket pair");
    source.set_nonblocking(true).expect("nonblocking source");
    peer.set_nonblocking(true).expect("nonblocking peer");
    let seen = Rc::new(AtomicUsize::new(0));
    let id = registry.allocate(
        source.as_raw_fd(),
        None,
        None,
        Some(free_on_error(&registry, &seen)),
    );
    registry.enable(id, Interest::Read);
    drive(&mut runtime);

    drop(peer);
    drive(&mut runtime);

    let events = seen.load(Ordering::SeqCst) as c_short;
    assert_ne!(events & STREAM_EVENT_EOF, 0);
    assert_ne!(events & STREAM_EVENT_READING, 0);
    assert!(registry.lookup(id).is_none());
}
