//! Application adapter contracts, including callbacks that destroy their owners.
use hmux2::src::reactor::*;
use hmux2::src::shared::{
    abi::timeval,
    event::{bufferevent, bufferevent_data_callback, bufferevent_event_callback, event},
};
use std::ffi::{c_int, c_short, c_void};
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;

struct Runtime;
impl Runtime {
    fn new() -> Self {
        shutdown_runtime();
        unsafe {
            event_init();
        }
        Self
    }
    fn tick(&self) {
        unsafe {
            event_loop(2);
        }
    }
}
impl Drop for Runtime {
    fn drop(&mut self) {
        shutdown_runtime();
    }
}
#[test]
fn timers_activation_cancellation_and_deadline_queries() {
    let rt = Runtime::new();
    unsafe {
        let mut event = event::default();
        assert_eq!(event_initialized(&event), 0);
        let mut calls = 0usize;
        let calls_ptr = &mut calls as *mut usize;
        event_set(
            &mut event,
            -1,
            0,
            move |_, _| *calls_ptr += 1,
        );
        assert_eq!(event_initialized(&event), 1);
        assert_eq!(event_pending(&event, 1, std::ptr::null_mut()), 0);
        let mut absolute = timeval {
            tv_sec: 0,
            tv_usec: 0,
        };
        assert_eq!(
            event_add(
                &mut event,
                &timeval {
                    tv_sec: 60,
                    tv_usec: 0
                }
            ),
            0
        );
        assert_eq!(event_pending(&event, 1, &mut absolute), 1);
        assert!(absolute.tv_sec > 0);
        event_del(&mut event);
        assert_eq!(event_initialized(&event), 1);
        rt.tick();
        assert_eq!(calls, 0);
        event_active(&mut event, 1, 1);
        event_active(&mut event, 1, 1);
        assert_eq!(calls, 0);
        rt.tick();
        assert_eq!(calls, 1);
        assert_eq!(event_pending(&event, 1, std::ptr::null_mut()), 0);
        event_once(
            -1,
            0,
            move |_, _| *calls_ptr += 1,
            std::ptr::null(),
        );
        rt.tick();
        assert_eq!(calls, 2);
    }
}
struct Owner {
    event: event,
    calls: *mut usize,
}
#[test]
fn activation_can_free_its_embedded_owner() {
    let rt = Runtime::new();
    unsafe {
        let mut calls = 0;
        let owner = Box::into_raw(Box::new(Owner {
            event: Default::default(),
            calls: &mut calls,
        }));
        event_set(&mut (*owner).event, -1, 0, move |_, _| {
            let owner = Box::from_raw(owner);
            *owner.calls += 1;
        });
        event_active(&mut (*owner).event, 1, 1);
        event_active(&mut (*owner).event, 1, 1);
        rt.tick();
        rt.tick();
        assert_eq!(calls, 1);
    }
}
#[derive(Default)]
struct Observations {
    reads: usize,
    writes: usize,
    errors: Vec<c_short>,
    free_on_read: bool,
}
unsafe fn read_cb(stream: *mut bufferevent, arg: *mut c_void) {
    let state = unsafe { &mut *(arg as *mut Observations) };
    state.reads += 1;
    if state.free_on_read {
        unsafe {
            bufferevent_free(stream);
        }
    }
}
unsafe fn write_cb(_: *mut bufferevent, arg: *mut c_void) {
    unsafe {
        (*(arg as *mut Observations)).writes += 1;
    }
}
unsafe fn error_cb(_: *mut bufferevent, flags: c_short, arg: *mut c_void) {
    unsafe {
        (*(arg as *mut Observations)).errors.push(flags);
    }
}
fn pair() -> (UnixStream, UnixStream) {
    let (a, b) = UnixStream::pair().unwrap();
    a.set_nonblocking(true).unwrap();
    b.set_nonblocking(true).unwrap();
    (a, b)
}
#[test]
fn descriptorless_stream_keeps_buffers_without_io_callbacks() {
    let rt = Runtime::new();
    let mut observations = Observations::default();
    let observations_ptr = &mut observations as *mut Observations;
    unsafe {
        let stream = bufferevent_new(
            -1,
            bufferevent_data_callback(move |stream| {
                read_cb(stream, observations_ptr.cast())
            }),
            bufferevent_data_callback(move |stream| {
                write_cb(stream, observations_ptr.cast())
            }),
            bufferevent_event_callback(move |stream, flags| {
                error_cb(stream, flags, observations_ptr.cast())
            }),
        );
        assert!(!stream.is_null());
        assert_eq!(bufferevent_enable(stream, 6), 0);
        assert_eq!(
            evbuffer_add((*stream).input, b"input".as_ptr().cast(), 5),
            0
        );
        assert_eq!(bufferevent_write(stream, b"output".as_ptr().cast(), 6), 0);
        for _ in 0..3 {
            rt.tick();
        }
        assert_eq!(evbuffer_get_length(&*(*stream).input), 5);
        assert_eq!(evbuffer_get_length(&*(*stream).output), 6);
        assert_eq!(observations.reads, 0);
        assert_eq!(observations.writes, 0);
        assert!(observations.errors.is_empty());
        assert_eq!(bufferevent_disable(stream, 6), 0);
        evbuffer_drain((*stream).input, 5);
        evbuffer_drain((*stream).output, 6);
        assert_eq!(bufferevent_enable(stream, 6), 0);
        rt.tick();
        assert_eq!(observations.writes, 0);
        assert!(observations.errors.is_empty());
        bufferevent_free(stream);
        rt.tick();
    }
}
#[test]
fn stream_watermarks_drain_reenable_and_direct_output_append() {
    let rt = Runtime::new();
    let (mut peer, fd) = pair();
    let mut observations = Observations::default();
    let observations_ptr = &mut observations as *mut Observations;
    unsafe {
        let stream = bufferevent_new(
            fd.as_raw_fd(),
            bufferevent_data_callback(move |stream| {
                read_cb(stream, observations_ptr.cast())
            }),
            bufferevent_data_callback(move |stream| {
                write_cb(stream, observations_ptr.cast())
            }),
            bufferevent_event_callback(move |stream, flags| {
                error_cb(stream, flags, observations_ptr.cast())
            }),
        );
        assert!(!stream.is_null());
        bufferevent_setwatermark(stream, 2, 3, 4);
        bufferevent_enable(stream, 2);
        peer.write_all(b"abcdefgh").unwrap();
        rt.tick();
        assert_eq!(evbuffer_get_length(&*(*stream).input), 4);
        assert_eq!(observations.reads, 1);
        rt.tick();
        assert_eq!(observations.reads, 1);
        evbuffer_drain((*stream).input, 4);
        rt.tick();
        assert_eq!(evbuffer_get_length(&*(*stream).input), 4);
        assert_eq!(observations.reads, 2);
        bufferevent_disable(stream, 2);
        evbuffer_drain((*stream).input, 4);
        peer.write_all(b"ijkl").unwrap();
        rt.tick();
        assert_eq!(observations.reads, 2);
        bufferevent_enable(stream, 2);
        rt.tick();
        assert_eq!(observations.reads, 3);
        evbuffer_add((*stream).output, b"reply".as_ptr().cast(), 5);
        rt.tick();
        let mut output = [0; 5];
        peer.read_exact(&mut output).unwrap();
        assert_eq!(&output, b"reply");
        assert_eq!(observations.writes, 1);
        bufferevent_free(stream);
    }
}
#[test]
fn stream_callback_can_free_owner_with_both_directions_ready() {
    let rt = Runtime::new();
    let (mut peer, fd) = pair();
    let mut observations = Observations {
        free_on_read: true,
        ..Default::default()
    };
    let observations_ptr = &mut observations as *mut Observations;
    unsafe {
        let stream = bufferevent_new(
            fd.as_raw_fd(),
            bufferevent_data_callback(move |stream| {
                read_cb(stream, observations_ptr.cast())
            }),
            bufferevent_data_callback(move |stream| {
                write_cb(stream, observations_ptr.cast())
            }),
            bufferevent_event_callback(move |stream, flags| {
                error_cb(stream, flags, observations_ptr.cast())
            }),
        );
        bufferevent_enable(stream, 2);
        bufferevent_write(stream, b"not sent".as_ptr().cast(), 8);
        peer.write_all(b"free now").unwrap();
        rt.tick();
        rt.tick();
        assert_eq!(observations.reads, 1);
        assert_eq!(observations.writes, 0);
    }
}
#[test]
fn partial_writes_resume_after_backpressure_and_deliver_eof_once() {
    let rt = Runtime::new();
    let (mut peer, fd) = pair();
    let mut observations = Observations::default();
    let observations_ptr = &mut observations as *mut Observations;
    unsafe {
        let stream = bufferevent_new(
            fd.as_raw_fd(),
            bufferevent_data_callback(move |stream| {
                read_cb(stream, observations_ptr.cast())
            }),
            bufferevent_data_callback(move |stream| {
                write_cb(stream, observations_ptr.cast())
            }),
            bufferevent_event_callback(move |stream, flags| {
                error_cb(stream, flags, observations_ptr.cast())
            }),
        );
        let payload = (0..2_000_000).map(|i| (i % 251) as u8).collect::<Vec<_>>();
        bufferevent_write(stream, payload.as_ptr().cast(), payload.len());
        rt.tick();
        assert!(evbuffer_get_length(&*(*stream).output) > 0);
        let mut received = Vec::new();
        let mut chunk = [0; 65536];
        for _ in 0..200 {
            loop {
                match peer.read(&mut chunk) {
                    Ok(n) if n > 0 => received.extend_from_slice(&chunk[..n]),
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                    other => panic!("unexpected read: {other:?}"),
                }
            }
            rt.tick();
            if received.len() == payload.len() {
                break;
            }
        }
        assert_eq!(received, payload);
        assert_eq!(observations.writes, 1);
        bufferevent_enable(stream, 2);
        peer.shutdown(std::net::Shutdown::Write).unwrap();
        rt.tick();
        rt.tick();
        assert_eq!(observations.errors, [0x11]);
        bufferevent_free(stream);
    }
}
#[test]
fn stream_temporarily_sets_and_restores_blocking_flags() {
    let _rt = Runtime::new();
    let (_peer, fd) = UnixStream::pair().unwrap();
    unsafe {
        let before = libc::fcntl(fd.as_raw_fd(), libc::F_GETFL);
        let stream = bufferevent_new(fd.as_raw_fd(), None, None, None);
        assert!(!stream.is_null());
        assert_ne!(
            libc::fcntl(fd.as_raw_fd(), libc::F_GETFL) & libc::O_NONBLOCK,
            0
        );
        bufferevent_free(stream);
        assert_eq!(libc::fcntl(fd.as_raw_fd(), libc::F_GETFL), before);
    }
}

#[test]
fn enabling_an_empty_writer_requests_one_callback_without_busy_polling() {
    let rt = Runtime::new();
    let (_peer, fd) = pair();
    let mut observations = Observations::default();
    let observations_ptr = &mut observations as *mut Observations;
    unsafe {
        let stream = bufferevent_new(
            fd.as_raw_fd(),
            None,
            bufferevent_data_callback(move |stream| {
                write_cb(stream, observations_ptr.cast())
            }),
            None,
        );
        rt.tick();
        assert_eq!(observations.writes, 0);
        // Control mode uses this to fill an empty output buffer from its
        // application-owned queue, including when EV_WRITE is already enabled.
        bufferevent_enable(stream, 4);
        rt.tick();
        rt.tick();
        assert_eq!(observations.writes, 1);
        bufferevent_enable(stream, 4);
        rt.tick();
        rt.tick();
        assert_eq!(observations.writes, 2);
        bufferevent_free(stream);
    }
}

struct Reuse {
    event: event,
    replacement: c_int,
    old_fd: c_int,
    calls: usize,
}
unsafe fn reuse_event(state: *mut Reuse, fd: c_int) {
    let state = &mut *state;
    state.calls += 1;
    if state.calls == 1 {
        libc::dup2(state.replacement, state.old_fd);
        let state_ptr = state as *mut Reuse;
        event_set(&mut state.event, fd, 2, move |fd, _| reuse_event(state_ptr, fd));
        event_add(&mut state.event, std::ptr::null());
    }
}
#[test]
fn callback_rearming_a_reused_descriptor_cannot_keep_the_old_lease() {
    let rt = Runtime::new();
    let (mut old_peer, old_fd) = pair();
    let (mut new_peer, replacement) = pair();
    let mut state = Reuse {
        event: Default::default(),
        replacement: replacement.as_raw_fd(),
        old_fd: old_fd.as_raw_fd(),
        calls: 0,
    };
    unsafe {
        let state_ptr = &mut state as *mut Reuse;
        event_set(
            &mut state.event,
            old_fd.as_raw_fd(),
            2,
            move |fd, _| reuse_event(state_ptr, fd),
        );
        event_add(&mut state.event, std::ptr::null());
        old_peer.write_all(b"old").unwrap();
        rt.tick();
        assert_eq!(state.calls, 1);
        new_peer.write_all(b"new").unwrap();
        rt.tick();
        assert_eq!(state.calls, 2);
        event_del(&mut state.event);
    }
}
