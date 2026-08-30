use super::*;
use std::io::Read as _;
use std::os::fd::{FromRawFd, RawFd};

fn socket_pair() -> [RawFd; 2] {
    let mut fds = [0; 2];
    assert_eq!(
        unsafe { libc::socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, fds.as_mut_ptr()) },
        0
    );
    fds
}

#[test]
fn owned_segments_are_kept_without_copying() {
    let owned = Bytes::from_static(b"owned");
    let ptr = owned.as_ptr();
    let mut buf = Buf::new();
    buf.append_bytes(owned);
    assert_eq!(buf.pullup(5).as_ptr(), ptr);
}

#[test]
fn borrowed_appends_and_cross_segment_consumption_preserve_order() {
    let mut buf = Buf::new();
    buf.append_bytes(Bytes::from_static(b"one"));
    buf.append_bytes(Bytes::from_static(b"two"));
    buf.append(b"three");
    assert_eq!(buf.copy_to_bytes(11).as_ref(), b"onetwothree");
    assert!(buf.is_empty());
}

#[test]
fn mutable_buffer_appends_into_the_tail() {
    let mut buf = Buf::new();
    buf.put_slice(b"hello");
    assert_eq!(buf.as_slice(), b"hello");
}

#[test]
fn vectored_write_consumes_only_written_bytes() {
    let fds = socket_pair();
    let mut buf = Buf::new();
    buf.append_bytes(Bytes::from_static(b"one"));
    buf.append_bytes(Bytes::from_static(b"two"));
    assert_eq!(buf.write_to_fd(fds[0]).unwrap(), 6);
    let mut out = [0; 6];
    let mut stream = unsafe { std::os::unix::net::UnixStream::from_raw_fd(fds[1]) };
    stream.read_exact(&mut out).unwrap();
    assert_eq!(&out, b"onetwo");
    unsafe { libc::close(fds[0]) };
}

#[test]
fn read_line_handles_crlf_and_retains_tail() {
    let mut buf = Buf::new();
    buf.append(b"one\r\ntwo\nrest");
    assert_eq!(buf.read_line().unwrap().as_ref(), b"one");
    assert_eq!(buf.read_line().unwrap().as_ref(), b"two");
    assert_eq!(buf.as_slice(), b"rest");
}
