use super::*;

use crate::TaskRuntime;
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
    let data = registry.with_input(id, |buffer| {
        let bytes = buffer.as_slice().to_vec();
        buffer.drain(buffer.len());
        bytes
    });
    assert_eq!(data.expect("stream"), b"firstsecond".to_vec());
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
    let (source, peer) = UnixStream::pair().expect("socket pair");
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

#[test]
fn read_watermark_and_reenable_resume_without_a_new_edge() {
    let mut runtime = TaskRuntime::new().unwrap();
    let registry = registry_with_runtime(&runtime);
    let (source, mut peer) = UnixStream::pair().unwrap();
    source.set_nonblocking(true).unwrap();
    let calls = Rc::new(AtomicUsize::new(0));
    let id = registry.allocate(source.as_raw_fd(), Some(count(&calls)), None, None);
    registry.set_read_watermark(id, 3, 4);
    registry.enable(id, Interest::Read);
    peer.write_all(b"abcdefgh").unwrap();
    for _ in 0..4 {
        drive(&mut runtime);
    }
    assert_eq!(registry.input_len(id), 4);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    registry.disable(id, Interest::Read);
    registry.with_input(id, |b| b.drain(4));
    registry.wake(id);
    drive(&mut runtime);
    assert_eq!(registry.input_len(id), 0);
    registry.enable(id, Interest::Read);
    for _ in 0..4 {
        drive(&mut runtime);
    }
    assert_eq!(
        registry.with_input(id, |b| b.as_slice().to_vec()).unwrap(),
        b"efgh"
    );
    registry.free(id);
    runtime.flush_cancelled().unwrap();
}

#[test]
fn regular_file_transfer_exceeds_budget_and_cleans_up() {
    let mut runtime = TaskRuntime::new().unwrap();
    let registry = registry_with_runtime(&runtime);
    let path = std::env::temp_dir().join(format!("hmux-rt-stream-{}", std::process::id()));
    let bytes = vec![b'x'; READ_CHUNK * (STREAM_IO_BUDGET + 2)];
    std::fs::write(&path, &bytes).unwrap();
    let source = std::fs::File::open(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
    let events = Rc::new(AtomicUsize::new(0));
    let output = events.clone();
    let id = registry.allocate(
        source.as_raw_fd(),
        None,
        None,
        Some(Rc::new(move |_, f| {
            output.store(f as usize, Ordering::SeqCst);
        })),
    );
    registry.enable(id, Interest::Read);
    for _ in 0..10 {
        drive(&mut runtime);
    }
    assert_ne!(events.load(Ordering::SeqCst) & STREAM_EVENT_EOF as usize, 0);
    assert!(registry.with_input(id, |b| b.as_slice() == bytes).unwrap());
    registry.free(id);
    runtime.flush_cancelled().unwrap();
    assert!(registry.lookup(id).is_none());
}

#[test]
fn large_partial_writes_eventually_drain_and_close_after_cancellation() {
    let mut runtime = TaskRuntime::new().unwrap();
    let registry = registry_with_runtime(&runtime);
    let (source, mut peer) = UnixStream::pair().unwrap();
    source.set_nonblocking(true).unwrap();
    peer.set_nonblocking(true).unwrap();
    let id = registry.allocate(source.as_raw_fd(), None, None, None);
    let bytes = vec![42; 4 * 1024 * 1024];
    registry.write(id, &bytes);
    registry.enable(id, Interest::Write);
    drive(&mut runtime);
    assert!(registry.output_len(id) > 0);
    let mut received = Vec::new();
    for _ in 0..1024 {
        drive(&mut runtime);
        let mut chunk = [0; 65536];
        loop {
            match peer.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => received.extend_from_slice(&chunk[..n]),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => panic!("{e}"),
            }
        }
        if received.len() == bytes.len() {
            break;
        }
    }
    assert_eq!(received, bytes);
    registry.free(id);
    runtime.flush_cancelled().unwrap();
    drop(source);
    assert_eq!(peer.read(&mut [0; 1]).unwrap(), 0);
}

#[test]
fn pty_hangup_delivers_buffered_bytes_then_eof() {
    use std::os::fd::{FromRawFd, OwnedFd};
    let mut master = -1;
    let mut slave = -1;
    assert_eq!(
        unsafe {
            libc::openpty(
                &mut master,
                &mut slave,
                std::ptr::null_mut(),
                std::ptr::null(),
                std::ptr::null(),
            )
        },
        0
    );
    let master = unsafe { OwnedFd::from_raw_fd(master) };
    let slave = unsafe { OwnedFd::from_raw_fd(slave) };
    assert_ne!(
        unsafe { libc::fcntl(master.as_raw_fd(), libc::F_SETFL, libc::O_NONBLOCK) },
        -1
    );
    let mut runtime = TaskRuntime::new().unwrap();
    let registry = registry_with_runtime(&runtime);
    let flags = Rc::new(AtomicUsize::new(0));
    let found = flags.clone();
    let id = registry.allocate(
        master.as_raw_fd(),
        None,
        None,
        Some(Rc::new(move |_, f| {
            found.store(f as usize, Ordering::SeqCst);
        })),
    );
    registry.enable(id, Interest::Read);
    assert_eq!(
        unsafe { libc::write(slave.as_raw_fd(), b"pty".as_ptr().cast(), 3) },
        3
    );
    drop(slave);
    let deadline = std::time::Instant::now() + Duration::from_secs(1);
    while flags.load(Ordering::SeqCst) == 0 && std::time::Instant::now() < deadline {
        drive(&mut runtime);
        runtime.poll(Some(Duration::from_millis(1))).unwrap();
    }
    assert_eq!(
        registry.with_input(id, |b| b.as_slice().to_vec()).unwrap(),
        b"pty"
    );
    assert_ne!(flags.load(Ordering::SeqCst) & STREAM_EVENT_EOF as usize, 0);
    registry.free(id);
    runtime.flush_cancelled().unwrap();
}

#[test]
fn writing_less_than_the_low_watermark_still_notifies_drain() {
    let mut runtime = TaskRuntime::new().unwrap();
    let registry = registry_with_runtime(&runtime);
    let (source, _peer) = UnixStream::pair().unwrap();
    source.set_nonblocking(true).unwrap();
    let calls = Rc::new(AtomicUsize::new(0));
    let id = registry.allocate(source.as_raw_fd(), None, Some(count(&calls)), None);
    registry.set_watermark(id, 512, 0);
    registry.enable(id, Interest::Write);
    drive(&mut runtime);
    let before = calls.load(Ordering::SeqCst);
    registry.write(id, b"small");
    for _ in 0..4 {
        drive(&mut runtime);
    }
    assert_eq!(registry.output_len(id), 0);
    assert_eq!(calls.load(Ordering::SeqCst), before + 1);
    registry.free(id);
    runtime.flush_cancelled().unwrap();
}
