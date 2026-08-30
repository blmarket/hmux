use super::*;
use crate::ffi::{__errno_location, close, socketpair};
use ::core::ffi::{c_char, c_int, c_void};
use ::core::ptr::{null, null_mut};

/// An ibuf that gives itself up at the end of the test.
struct Buf(Option<Box<ibuf>>);

impl Buf {
    fn open(len: size_t) -> Buf {
        Buf(Some(ibuf_open(len).expect("ibuf_open failed")))
    }

    fn dynamic(len: size_t, max: size_t) -> Buf {
        Buf(Some(ibuf_dynamic(len, max).expect("ibuf_dynamic failed")))
    }

    fn ptr(&self) -> *mut ibuf {
        self.0
            .as_deref()
            .map_or(null_mut(), |buf| &raw const *buf as *mut ibuf)
    }

    /// Hands the buffer over to something that takes ownership of it, such
    /// as a queue.
    fn leak(mut self) -> Box<ibuf> {
        self.0.take().expect("a buffer to hand over")
    }

    /// The bytes between the read and the write position.
    fn bytes(&self) -> Vec<u8> {
        unsafe {
            ::core::slice::from_raw_parts(ibuf_data(self.ptr()) as *const u8, ibuf_size(self.ptr()))
                .to_vec()
        }
    }
}

impl Drop for Buf {
    fn drop(&mut self) {
        if let Some(buf) = self.0.take() {
            unsafe { ibuf_free(buf) };
        }
    }
}

impl ::core::ops::Deref for Buf {
    type Target = ibuf;

    fn deref(&self) -> &ibuf {
        self.0.as_deref().expect("a buffer still held")
    }
}

/// A message buffer that frees itself at the end of the test.
struct Msgbuf(Box<msgbuf>);

impl Msgbuf {
    fn new() -> Msgbuf {
        Msgbuf(msgbuf_new())
    }

    /// A message buffer that reads, whose header is four big-endian bytes
    /// holding the whole message's length.
    fn reader() -> Msgbuf {
        Msgbuf(
            unsafe { msgbuf_new_reader(4, Some(read_header), null_mut()) }
                .expect("msgbuf_new_reader failed"),
        )
    }

    fn ptr(&self) -> *mut msgbuf {
        &raw const *self.0 as *mut msgbuf
    }

    /// Every message the reader has finished, taken off the queue.
    fn messages(&self) -> Vec<Vec<u8>> {
        let mut out = Vec::new();
        loop {
            let Some(p) = (unsafe { msgbuf_get(self.ptr()) }) else {
                return out;
            };
            out.push(Buf(Some(p)).bytes());
        }
    }
}

/// A four-byte big-endian length, then that many bytes in all.
unsafe fn read_header(hdr: *mut ibuf, _arg: *mut imsgbuf, _fd: *mut c_int) -> Option<Box<ibuf>> {
    unsafe {
        let mut len: uint32_t = 0;
        ibuf_get_n32(hdr, &raw mut len);
        ibuf_open(len as size_t)
    }
}

/// A header reader that turns every message down.
unsafe fn refuse_header(_hdr: *mut ibuf, _arg: *mut imsgbuf, _fd: *mut c_int) -> Option<Box<ibuf>> {
    unsafe { *__errno_location() = EINVAL };
    None
}

/// A header reader answering a buffer whose write position is already past
/// its own limit, so that the copy into it is refused.
unsafe fn broken_header(_hdr: *mut ibuf, _arg: *mut imsgbuf, _fd: *mut c_int) -> Option<Box<ibuf>> {
    let mut p = ibuf_open(4)?;
    p.max = 2;
    p.wpos = 4;
    Some(p)
}

/// A message the reader takes: its length, then its payload.
fn framed(payload: &[u8]) -> Vec<u8> {
    let len = (payload.len() + 4) as u32;
    let mut out = len.to_be_bytes().to_vec();
    out.extend_from_slice(payload);
    out
}

/// The bytes a buffer somebody else owns has between its read and write
/// positions.
unsafe fn bytes_of(buf: *mut ibuf) -> Vec<u8> {
    unsafe { ::core::slice::from_raw_parts(ibuf_data(buf) as *const u8, ibuf_size(buf)).to_vec() }
}

fn errno() -> c_int {
    unsafe { *__errno_location() }
}

fn clear_errno() {
    unsafe { *__errno_location() = 0 };
}

/// A connected pair of unix stream sockets, closed at the end of the test.
struct Pair([c_int; 2]);

impl Pair {
    fn new() -> Pair {
        let mut fds: [c_int; 2] = [-1, -1];
        assert_eq!(
            unsafe { socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, fds.as_mut_ptr()) },
            0
        );
        Pair(fds)
    }

    fn writer(&self) -> c_int {
        self.0[0]
    }

    fn reader(&self) -> c_int {
        self.0[1]
    }

    /// Makes both ends refuse to wait.
    fn nonblocking(&self) -> &Pair {
        for fd in self.0 {
            unsafe { libc::fcntl(fd, libc::F_SETFL, libc::O_NONBLOCK) };
        }
        self
    }

    fn close_writer(&mut self) {
        if self.0[0] != -1 {
            unsafe { close(self.0[0]) };
            self.0[0] = -1;
        }
    }

    /// What the reading end has to offer, up to `len` bytes.
    fn read(&self, len: usize) -> Vec<u8> {
        let mut out = vec![0u8; len];
        let n = unsafe { libc::read(self.reader(), out.as_mut_ptr() as *mut c_void, len) };
        assert!(n >= 0, "read failed");
        out.truncate(n as usize);
        out
    }
}

impl Drop for Pair {
    fn drop(&mut self) {
        for fd in self.0 {
            if fd != -1 {
                unsafe { close(fd) };
            }
        }
    }
}

#[test]
fn an_open_buffer_is_sized_once_and_for_all() {
    let buf = Buf::open(8);
    assert_eq!(buf.size, 8);
    assert_eq!(buf.max, 8);
    assert_eq!(buf.wpos, 0);
    assert_eq!(buf.fd, -1);
    assert_eq!(unsafe { ibuf_size(buf.ptr()) }, 0);
    assert_eq!(unsafe { ibuf_left(buf.ptr()) }, 8);
    assert_eq!(buf.bytes(), []);

    let empty = Buf::open(0);
    assert_eq!(empty.buf.capacity(), 0);
    assert_eq!(empty.size, 0);
}

#[test]
fn a_dynamic_buffer_grows_up_to_its_limit() {
    let buf = Buf::dynamic(0, 8);
    assert_eq!(buf.size, 0);
    assert_eq!(buf.max, 8);
    assert_eq!(buf.buf.capacity(), 0);

    let started = Buf::dynamic(4, 8);
    assert_eq!(started.size, 4);
    assert_eq!(started.max, 8);
    assert!(started.buf.capacity() >= 4);
}

#[test]
fn ibuf_implements_buf_and_bufmut() {
    use ::bytes::{Buf as _, BufMut as _};

    let buf = Buf::dynamic(0, 8);
    let raw = unsafe { &mut *buf.ptr() };
    raw.put_slice(b"abc");
    assert_eq!(raw.remaining(), 3);
    assert_eq!(raw.chunk(), b"abc");
    assert_eq!(raw.get_u16(), 0x6162);
    assert_eq!(raw.get_u8(), b'c');
    assert_eq!(raw.remaining(), 0);

    raw.put_slice(b"de");
    assert_eq!(raw.remaining(), 2);
    assert_eq!(raw.chunk(), b"de");
}

#[test]
fn a_dynamic_buffer_needs_a_limit_that_holds_what_it_starts_with() {
    clear_errno();
    assert!(ibuf_dynamic(0, 0).is_none());
    assert_eq!(errno(), EINVAL);

    clear_errno();
    assert!(ibuf_dynamic(9, 8).is_none());
    assert_eq!(errno(), EINVAL);
}

#[test]
fn reserving_moves_the_write_position_and_grows_the_buffer() {
    let buf = Buf::dynamic(2, 8);
    unsafe {
        let first = ibuf_reserve(buf.ptr(), 2);
        assert!(!first.is_null());
        assert_eq!(buf.wpos, 2);
        assert_eq!(buf.size, 2);

        let second = ibuf_reserve(buf.ptr(), 4);
        assert!(!second.is_null());
        assert_eq!(buf.wpos, 6);
        assert_eq!(buf.size, 6);
        assert_eq!(buf.bytes(), [0, 0, 0, 0, 0, 0]);
    }
}

#[test]
fn reserving_more_than_the_buffer_may_hold_is_refused() {
    let buf = Buf::dynamic(0, 8);
    unsafe {
        clear_errno();
        assert!(ibuf_reserve(buf.ptr(), 9).is_null());
        assert_eq!(errno(), ERANGE);

        // Past the limit, and past what the write position may be added
        // to at all.
        assert_eq!(ibuf_add(buf.ptr(), b"a".as_ptr(), 1), 0);
        clear_errno();
        assert!(ibuf_reserve(buf.ptr(), SIZE_MAX as size_t).is_null());
        assert_eq!(errno(), ERANGE);
    }
}

#[test]
fn a_buffer_over_somebody_elses_bytes_is_never_reserved_into() {
    let mut data = *b"abcd";
    let mut buf = empty_ibuf();
    unsafe {
        ibuf_from_buffer(&raw mut *buf, data.as_mut_ptr(), 4);
        assert!(buf.borrowed);
        assert_eq!(ibuf_size(&raw mut *buf), 4);
        assert_eq!(ibuf_left(&raw mut *buf), 0);

        clear_errno();
        assert!(ibuf_reserve(&raw mut *buf, 1).is_null());
        assert_eq!(errno(), EINVAL);

        clear_errno();
        assert_eq!(ibuf_set_maxsize(&raw mut *buf, 1), -1);
        assert_eq!(errno(), EINVAL);

        clear_errno();
        assert_eq!(ibuf_truncate(&raw mut *buf, 8), -1);
        assert_eq!(errno(), ERANGE);
    }
}

fn empty_ibuf() -> Box<ibuf> {
    Box::new(ibuf::default())
}

#[test]
fn adding_nothing_adds_nothing() {
    let buf = Buf::dynamic(0, 8);
    unsafe {
        assert_eq!(ibuf_add(buf.ptr(), null::<c_uchar>(), 0), 0);
        assert_eq!(ibuf_add_zero(buf.ptr(), 0), 0);
        assert_eq!(ibuf_size(buf.ptr()), 0);
    }
}

#[test]
fn adding_past_the_limit_is_refused() {
    let buf = Buf::dynamic(0, 2);
    unsafe {
        assert_eq!(ibuf_add(buf.ptr(), b"abcd".as_ptr(), 4), -1);
        assert_eq!(ibuf_add_zero(buf.ptr(), 4), -1);
        assert_eq!(ibuf_size(buf.ptr()), 0);
    }
}

#[test]
fn numbers_are_added_in_network_order_and_in_host_order() {
    let buf = Buf::dynamic(0, 64);
    unsafe {
        assert_eq!(ibuf_add_n8(buf.ptr(), 0x12), 0);
        assert_eq!(ibuf_add_n16(buf.ptr(), 0x1234), 0);
        assert_eq!(ibuf_add_n32(buf.ptr(), 0x1234_5678), 0);
        assert_eq!(ibuf_add_n64(buf.ptr(), 0x1234_5678_9abc_def0), 0);
        assert_eq!(
            buf.bytes(),
            [
                0x12, 0x12, 0x34, 0x12, 0x34, 0x56, 0x78, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde,
                0xf0
            ]
        );
    }

    let host = Buf::dynamic(0, 64);
    unsafe {
        assert_eq!(ibuf_add_h16(host.ptr(), 0x1234), 0);
        assert_eq!(ibuf_add_h32(host.ptr(), 0x1234_5678), 0);
        assert_eq!(ibuf_add_h64(host.ptr(), 0x1234_5678_9abc_def0), 0);
        assert_eq!(
            host.bytes(),
            [
                0x34, 0x12, 0x78, 0x56, 0x34, 0x12, 0xf0, 0xde, 0xbc, 0x9a, 0x78, 0x56, 0x34, 0x12
            ]
        );
    }
}

#[test]
fn a_number_too_big_for_its_width_is_refused() {
    let buf = Buf::dynamic(0, 64);
    unsafe {
        for (retval, width) in [
            (ibuf_add_n8(buf.ptr(), 0x100), 8),
            (ibuf_add_n16(buf.ptr(), 0x1_0000), 16),
            (ibuf_add_n32(buf.ptr(), 0x1_0000_0000), 32),
            (ibuf_add_h16(buf.ptr(), 0x1_0000), 16),
            (ibuf_add_h32(buf.ptr(), 0x1_0000_0000), 32),
        ] {
            assert_eq!(retval, -1, "a value wider than {width} bits was taken");
            assert_eq!(errno(), EINVAL);
        }
        assert_eq!(ibuf_size(buf.ptr()), 0);
    }
}

#[test]
fn one_buffer_is_added_to_another() {
    let buf = Buf::dynamic(0, 16);
    let from = Buf::dynamic(0, 16);
    unsafe {
        ibuf_add(from.ptr(), b"abc".as_ptr(), 3);
        assert_eq!(ibuf_add_ibuf(buf.ptr(), from.ptr()), 0);
        assert_eq!(buf.bytes(), b"abc");
    }
}

#[test]
fn a_string_is_added_into_a_field_of_its_own_and_padded_with_zeroes() {
    let buf = Buf::dynamic(0, 16);
    unsafe {
        assert_eq!(ibuf_add_strbuf(buf.ptr(), c"ab".as_ptr(), 5), 0);
        assert_eq!(buf.bytes(), b"ab\0\0\0");

        clear_errno();
        assert_eq!(ibuf_add_strbuf(buf.ptr(), c"toolong".as_ptr(), 4), -1);
        assert_eq!(errno(), EOVERFLOW);

        assert_eq!(ibuf_add_strbuf(buf.ptr(), c"x".as_ptr(), 99), -1);
    }
}

#[test]
fn seeking_finds_a_place_inside_what_is_written() {
    let buf = Buf::dynamic(0, 16);
    unsafe {
        ibuf_add(buf.ptr(), b"abcd".as_ptr(), 4);
        let p = ibuf_seek(buf.ptr(), 1, 2) as *const u8;
        assert_eq!(::core::slice::from_raw_parts(p, 2), b"bc");

        clear_errno();
        assert!(ibuf_seek(buf.ptr(), 5, 0).is_null());
        assert_eq!(errno(), ERANGE);
        assert!(ibuf_seek(buf.ptr(), 2, 3).is_null());
        assert!(ibuf_seek(buf.ptr(), 1, SIZE_MAX as size_t).is_null());
    }
}

#[test]
fn a_number_already_written_is_written_over() {
    let buf = Buf::dynamic(0, 64);
    unsafe {
        ibuf_add_zero(buf.ptr(), 32);
        assert_eq!(ibuf_set_n8(buf.ptr(), 0, 0x12), 0);
        assert_eq!(ibuf_set_n16(buf.ptr(), 1, 0x1234), 0);
        assert_eq!(ibuf_set_n32(buf.ptr(), 3, 0x1234_5678), 0);
        assert_eq!(ibuf_set_n64(buf.ptr(), 7, 0x1234_5678_9abc_def0), 0);
        assert_eq!(ibuf_set_h16(buf.ptr(), 15, 0x1234), 0);
        assert_eq!(ibuf_set_h32(buf.ptr(), 17, 0x1234_5678), 0);
        assert_eq!(ibuf_set_h64(buf.ptr(), 21, 0x1234_5678_9abc_def0), 0);
        assert_eq!(
            buf.bytes()[..29],
            [
                0x12, 0x12, 0x34, 0x12, 0x34, 0x56, 0x78, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde,
                0xf0, 0x34, 0x12, 0x78, 0x56, 0x34, 0x12, 0xf0, 0xde, 0xbc, 0x9a, 0x78, 0x56, 0x34,
                0x12
            ]
        );

        assert_eq!(ibuf_set(buf.ptr(), 0, null::<c_uchar>(), 0), 0);
        assert_eq!(ibuf_set(buf.ptr(), 64, b"x".as_ptr(), 1), -1);
    }
}

#[test]
fn a_number_too_big_for_the_field_it_is_written_into_is_refused() {
    let buf = Buf::dynamic(0, 64);
    unsafe {
        ibuf_add_zero(buf.ptr(), 32);
        assert_eq!(ibuf_set_n8(buf.ptr(), 0, 0x100), -1);
        assert_eq!(ibuf_set_n16(buf.ptr(), 0, 0x1_0000), -1);
        assert_eq!(ibuf_set_n32(buf.ptr(), 0, 0x1_0000_0000), -1);
        assert_eq!(ibuf_set_h16(buf.ptr(), 0, 0x1_0000), -1);
        assert_eq!(ibuf_set_h32(buf.ptr(), 0, 0x1_0000_0000), -1);
        assert_eq!(errno(), EINVAL);
    }
}

#[test]
fn a_limit_may_be_lowered_but_never_raised() {
    let buf = Buf::dynamic(0, 8);
    unsafe {
        assert_eq!(ibuf_set_maxsize(buf.ptr(), 4), 0);
        assert_eq!(buf.max, 4);

        clear_errno();
        assert_eq!(ibuf_set_maxsize(buf.ptr(), 8), -1);
        assert_eq!(errno(), ERANGE);
    }
}

#[test]
fn truncating_cuts_what_is_written_or_pads_it_with_zeroes() {
    let buf = Buf::dynamic(0, 16);
    unsafe {
        ibuf_add(buf.ptr(), b"abcd".as_ptr(), 4);
        assert_eq!(ibuf_truncate(buf.ptr(), 2), 0);
        assert_eq!(buf.bytes(), b"ab");

        assert_eq!(ibuf_truncate(buf.ptr(), 5), 0);
        assert_eq!(buf.bytes(), b"ab\0\0\0");

        assert_eq!(ibuf_truncate(buf.ptr(), 99), -1);
    }
}

#[test]
fn rewinding_puts_the_read_position_back_at_the_start() {
    let buf = Buf::dynamic(0, 16);
    unsafe {
        ibuf_add(buf.ptr(), b"abcd".as_ptr(), 4);
        assert_eq!(ibuf_skip(buf.ptr(), 2), 0);
        assert_eq!(buf.bytes(), b"cd");
        ibuf_rewind(buf.ptr());
        assert_eq!(buf.bytes(), b"abcd");

        assert_eq!(ibuf_skip(buf.ptr(), 99), -1);
        assert_eq!(errno(), EBADMSG);
    }
}

#[test]
fn a_buffer_reads_back_what_was_written_into_it() {
    let buf = Buf::dynamic(0, 64);
    unsafe {
        ibuf_add_n8(buf.ptr(), 0x12);
        ibuf_add_n16(buf.ptr(), 0x1234);
        ibuf_add_n32(buf.ptr(), 0x1234_5678);
        ibuf_add_n64(buf.ptr(), 0x1234_5678_9abc_def0);
        ibuf_add_h16(buf.ptr(), 0x4321);
        ibuf_add_h32(buf.ptr(), 0x8765_4321);
        ibuf_add_h64(buf.ptr(), 0x0fed_cba9_8765_4321);

        let mut n8: uint8_t = 0;
        let mut n16: uint16_t = 0;
        let mut n32: uint32_t = 0;
        let mut n64: uint64_t = 0;
        assert_eq!(ibuf_get_n8(buf.ptr(), &raw mut n8), 0);
        assert_eq!(n8, 0x12);
        assert_eq!(ibuf_get_n16(buf.ptr(), &raw mut n16), 0);
        assert_eq!(n16, 0x1234);
        assert_eq!(ibuf_get_n32(buf.ptr(), &raw mut n32), 0);
        assert_eq!(n32, 0x1234_5678);
        assert_eq!(ibuf_get_n64(buf.ptr(), &raw mut n64), 0);
        assert_eq!(n64, 0x1234_5678_9abc_def0);
        assert_eq!(ibuf_get_h16(buf.ptr(), &raw mut n16), 0);
        assert_eq!(n16, 0x4321);
        assert_eq!(ibuf_get_h32(buf.ptr(), &raw mut n32), 0);
        assert_eq!(n32, 0x8765_4321);
        assert_eq!(ibuf_get_h64(buf.ptr(), &raw mut n64), 0);
        assert_eq!(n64, 0x0fed_cba9_8765_4321);
        assert_eq!(ibuf_size(buf.ptr()), 0);
    }
}

#[test]
fn reading_past_the_end_is_refused() {
    let buf = Buf::dynamic(0, 64);
    unsafe {
        let mut n8: uint8_t = 0;
        let mut n16: uint16_t = 0;
        let mut n32: uint32_t = 0;
        let mut n64: uint64_t = 0;
        clear_errno();
        assert_eq!(ibuf_get(buf.ptr(), &raw mut n8, 1), -1);
        assert_eq!(errno(), EBADMSG);
        assert_eq!(ibuf_get_n8(buf.ptr(), &raw mut n8), -1);
        assert_eq!(ibuf_get_n16(buf.ptr(), &raw mut n16), -1);
        assert_eq!(ibuf_get_n32(buf.ptr(), &raw mut n32), -1);
        assert_eq!(ibuf_get_n64(buf.ptr(), &raw mut n64), -1);
        assert_eq!(ibuf_get_h16(buf.ptr(), &raw mut n16), -1);
        assert_eq!(ibuf_get_h32(buf.ptr(), &raw mut n32), -1);
        assert_eq!(ibuf_get_h64(buf.ptr(), &raw mut n64), -1);
        let mut new = empty_ibuf();
        assert_eq!(ibuf_get_ibuf(buf.ptr(), 1, &raw mut *new), -1);
        assert!(ibuf_get_string(buf.ptr(), 1).is_none());
    }
}

#[test]
fn a_buffer_is_read_out_as_a_buffer_over_the_same_bytes() {
    let buf = Buf::dynamic(0, 16);
    let mut new = empty_ibuf();
    unsafe {
        ibuf_add(buf.ptr(), b"abcdef".as_ptr(), 6);
        assert_eq!(ibuf_get_ibuf(buf.ptr(), 3, &raw mut *new), 0);
        assert_eq!(
            ::core::slice::from_raw_parts(ibuf_data(&raw mut *new) as *const u8, 3),
            b"abc"
        );
        assert!(new.borrowed);
        assert_eq!(buf.bytes(), b"def");

        let mut over = empty_ibuf();
        ibuf_from_ibuf(&raw mut *over, buf.ptr());
        assert_eq!(ibuf_size(&raw mut *over), 3);
        assert_eq!(
            ::core::slice::from_raw_parts(ibuf_data(&raw mut *over) as *const u8, 3),
            b"def"
        );
    }
}

#[test]
fn a_string_is_read_out_and_copied() {
    let buf = Buf::dynamic(0, 16);
    unsafe {
        ibuf_add(buf.ptr(), b"abcdef".as_ptr(), 6);
        let p = ibuf_get_string(buf.ptr(), 3).expect("string should be available");
        assert_eq!(p.as_bytes(), b"abc");
        assert_eq!(buf.bytes(), b"def");
    }
}

#[test]
fn a_string_field_must_end_in_a_terminator() {
    let buf = Buf::dynamic(0, 16);
    let mut out = [0u8; 4];
    unsafe {
        ibuf_add(buf.ptr(), b"ab\0no".as_ptr(), 5);
        assert_eq!(
            ibuf_get_strbuf(buf.ptr(), out.as_mut_ptr() as *mut c_char, 3),
            0
        );
        assert_eq!(&out[..3], b"ab\0");

        clear_errno();
        assert_eq!(
            ibuf_get_strbuf(buf.ptr(), out.as_mut_ptr() as *mut c_char, 2),
            -1
        );
        assert_eq!(errno(), EOVERFLOW);
        assert_eq!(&out[..2], b"n\0");

        clear_errno();
        assert_eq!(
            ibuf_get_strbuf(buf.ptr(), out.as_mut_ptr() as *mut c_char, 0),
            -1
        );
        assert_eq!(errno(), EINVAL);

        assert_eq!(
            ibuf_get_strbuf(buf.ptr(), out.as_mut_ptr() as *mut c_char, 4),
            -1
        );
    }
}

/// The write end of a fresh pipe, plus a check that its last copy has
/// been closed: the nonblocking read end reports end-of-file exactly
/// then, keyed to the kernel object rather than a reusable fd number.
struct CloseProbe {
    read: c_int,
}

impl CloseProbe {
    fn new() -> (CloseProbe, c_int) {
        let mut fds: [c_int; 2] = [-1, -1];
        unsafe {
            assert_eq!(libc::pipe2(fds.as_mut_ptr(), libc::O_NONBLOCK), 0);
        }
        (CloseProbe { read: fds[0] }, fds[1])
    }

    fn closed(&self) -> bool {
        let mut byte = 0u8;
        unsafe { libc::read(self.read, &raw mut byte as *mut c_void, 1) == 0 }
    }
}

impl Drop for CloseProbe {
    fn drop(&mut self) {
        unsafe {
            close(self.read);
        }
    }
}

#[test]
fn a_buffer_carries_at_most_one_descriptor() {
    let buf = Buf::open(4);
    let pair = Pair::new();
    unsafe {
        assert_eq!(ibuf_fd_avail(buf.ptr()), 0);
        assert_eq!(ibuf_fd_get(buf.ptr()), -1);

        let (probe, fd) = CloseProbe::new();
        ibuf_fd_set(buf.ptr(), fd);
        assert_eq!(ibuf_fd_avail(buf.ptr()), 1);

        // Setting another one closes the first.
        let second = libc::dup(pair.reader());
        ibuf_fd_set(buf.ptr(), second);
        assert!(probe.closed());

        assert_eq!(ibuf_fd_get(buf.ptr()), second);
        assert_eq!(ibuf_fd_avail(buf.ptr()), 0);
        close(second);

        // And a descriptor of -1 leaves the buffer without one.
        ibuf_fd_set(buf.ptr(), -1);
        assert_eq!(buf.fd, -1);
    }
}

#[test]
fn a_freed_buffer_closes_the_descriptor_it_carries() {
    unsafe {
        let (probe, fd) = CloseProbe::new();
        let mut buf = ibuf_open(4).expect("a buffer to carry it");
        ibuf_fd_set(&raw mut *buf, fd);
        assert!(!probe.closed());
        ibuf_free(buf);
        assert!(probe.closed());
    }
}

#[test]
fn a_queue_takes_buffers_at_the_end_and_hands_them_back_from_the_front() {
    unsafe {
        let mut bufq = ibufq_new();
        assert_eq!(ibufq_queuelen(&raw mut *bufq), 0);
        assert!(ibufq_pop(&raw mut *bufq).is_none());

        let first = Buf::open(1).leak();
        let second = Buf::open(2).leak();
        let (first_at, second_at) = (&raw const *first, &raw const *second);
        ibufq_push(&raw mut *bufq, first);
        ibufq_push(&raw mut *bufq, second);
        assert_eq!(ibufq_queuelen(&raw mut *bufq), 2);

        let first = ibufq_pop(&raw mut *bufq).expect("the first buffer back");
        assert_eq!(&raw const *first, first_at);
        assert_eq!(ibufq_queuelen(&raw mut *bufq), 1);
        let second = ibufq_pop(&raw mut *bufq).expect("the second buffer back");
        assert_eq!(&raw const *second, second_at);
        assert_eq!(ibufq_queuelen(&raw mut *bufq), 0);
        ibuf_free(first);
        ibuf_free(second);

        ibufq_push(&raw mut *bufq, Buf::open(1).leak());
        ibufq_free(bufq);
    }
}

#[test]
fn one_queue_is_joined_onto_the_end_of_another() {
    unsafe {
        let mut to = ibufq_new();
        let mut from = ibufq_new();
        let first = Buf::open(1).leak();
        let second = Buf::open(2).leak();
        let (first_at, second_at) = (&raw const *first, &raw const *second);
        ibufq_push(&raw mut *to, first);
        ibufq_push(&raw mut *from, second);

        ibufq_concat(&raw mut *to, &raw mut *from);
        assert_eq!(ibufq_queuelen(&raw mut *to), 2);
        assert_eq!(ibufq_queuelen(&raw mut *from), 0);
        let first = ibufq_pop(&raw mut *to).expect("the first buffer back");
        assert_eq!(&raw const *first, first_at);
        let second = ibufq_pop(&raw mut *to).expect("the second buffer back");
        assert_eq!(&raw const *second, second_at);
        ibuf_free(first);
        ibuf_free(second);

        // Joining an empty queue on leaves the first one as it was.
        ibufq_push(&raw mut *to, Buf::open(1).leak());
        ibufq_concat(&raw mut *to, &raw mut *from);
        assert_eq!(ibufq_queuelen(&raw mut *to), 1);
        ibufq_flush(&raw mut *to);
        assert_eq!(ibufq_queuelen(&raw mut *to), 0);
        ibufq_free(to);
        ibufq_free(from);
    }
}

#[test]
fn a_message_buffer_starts_with_two_empty_queues() {
    let msgbuf = Msgbuf::new();
    unsafe {
        assert_eq!(msgbuf_queuelen(msgbuf.ptr()), 0);
        assert!(msgbuf_get(msgbuf.ptr()).is_none());
        assert!((*msgbuf.ptr()).rbuf.is_none());

        ibuf_close(msgbuf.ptr(), Buf::open(4).leak());
        assert_eq!(msgbuf_queuelen(msgbuf.ptr()), 1);

        let mut from = ibufq_new();
        ibufq_push(&raw mut *from, Buf::open(4).leak());
        msgbuf_concat(msgbuf.ptr(), &raw mut *from);
        assert_eq!(msgbuf_queuelen(msgbuf.ptr()), 2);
        ibufq_free(from);

        msgbuf_clear(msgbuf.ptr());
        assert_eq!(msgbuf_queuelen(msgbuf.ptr()), 0);
    }
}

#[test]
fn a_reader_needs_a_header_size_that_fits_in_half_its_buffer() {
    unsafe {
        clear_errno();
        assert!(msgbuf_new_reader(0, Some(read_header), null_mut()).is_none());
        assert_eq!(errno(), EINVAL);

        clear_errno();
        assert!(
            msgbuf_new_reader(
                (IBUF_READ_SIZE / 2 + 1) as size_t,
                Some(read_header),
                null_mut()
            )
            .is_none()
        );
        assert_eq!(errno(), EINVAL);

        let msgbuf = Msgbuf::reader();
        assert!((*msgbuf.ptr()).rbuf.is_some());
        assert_eq!((*msgbuf.ptr()).hdrsize, 4);
    }
}

/// Puts `bytes` in the reader's own buffer, as a read would have.
fn filled(msgbuf: &Msgbuf, bytes: &[u8]) {
    unsafe {
        let rbuf = (*msgbuf.ptr())
            .rbuf
            .as_mut()
            .expect("a reader has a buffer");
        let p = rbuf.as_mut_ptr().add((*msgbuf.ptr()).roff);
        ::core::ptr::copy_nonoverlapping(bytes.as_ptr() as *const c_char, p, bytes.len());
        (*msgbuf.ptr()).roff += bytes.len();
    }
}

#[test]
fn the_reader_hands_back_whole_messages_and_keeps_the_rest() {
    let msgbuf = Msgbuf::reader();
    unsafe {
        let mut bytes = framed(b"one");
        bytes.extend(framed(b"two"));
        bytes.extend_from_slice(&framed(b"three")[..4]);
        filled(&msgbuf, &bytes);

        assert_eq!(ibuf_read_process(msgbuf.ptr(), -1), 1);
        assert_eq!(msgbuf.messages(), [framed(b"one"), framed(b"two")]);
        // The four header bytes of the third message went straight into
        // the message being built, so nothing is left over.
        assert_eq!((*msgbuf.ptr()).roff, 0);
        assert!((*msgbuf.ptr()).rpmsg.is_some());

        filled(&msgbuf, b"three");
        assert_eq!(ibuf_read_process(msgbuf.ptr(), -1), 1);
        assert_eq!(msgbuf.messages(), [framed(b"three")]);
        assert_eq!((*msgbuf.ptr()).roff, 0);
        assert!((*msgbuf.ptr()).rpmsg.is_none());
    }
}

#[test]
fn the_reader_waits_for_a_whole_header() {
    let msgbuf = Msgbuf::reader();
    unsafe {
        filled(&msgbuf, b"ab");
        assert_eq!(ibuf_read_process(msgbuf.ptr(), -1), 1);
        assert!(msgbuf.messages().is_empty());
        assert_eq!((*msgbuf.ptr()).roff, 2);
    }
}

#[test]
fn the_reader_closes_the_descriptor_it_was_handed() {
    let msgbuf = Msgbuf::reader();
    unsafe {
        let (probe, fd) = CloseProbe::new();
        filled(&msgbuf, &framed(b"one"));
        assert_eq!(ibuf_read_process(msgbuf.ptr(), fd), 1);
        assert!(probe.closed());
    }
}

#[test]
fn a_header_the_reader_turns_down_stops_the_read_and_closes_the_descriptor() {
    unsafe {
        let msgbuf =
            Msgbuf(msgbuf_new_reader(4, Some(refuse_header), null_mut()).expect("a reader"));
        let (probe, fd) = CloseProbe::new();
        filled(&msgbuf, &framed(b"one"));
        assert_eq!(ibuf_read_process(msgbuf.ptr(), fd), -1);
        assert!(probe.closed());
    }
}

#[test]
fn a_message_that_will_not_take_its_own_bytes_stops_the_read() {
    unsafe {
        let msgbuf =
            Msgbuf(msgbuf_new_reader(4, Some(broken_header), null_mut()).expect("a reader"));
        filled(&msgbuf, &framed(b"one"));
        assert_eq!(ibuf_read_process(msgbuf.ptr(), -1), -1);
    }
}

#[test]
fn writing_sends_every_queued_buffer_and_drops_them() {
    let msgbuf = Msgbuf::new();
    let pair = Pair::new();
    unsafe {
        assert_eq!(ibuf_write(pair.writer(), msgbuf.ptr()), 0);

        for text in [b"one".as_slice(), b"two".as_slice()] {
            let buf = Buf::dynamic(0, 16);
            ibuf_add(buf.ptr(), text.as_ptr(), text.len());
            ibuf_close(msgbuf.ptr(), buf.leak());
        }
        assert_eq!(ibuf_write(pair.writer(), msgbuf.ptr()), 0);
        assert_eq!(msgbuf_queuelen(msgbuf.ptr()), 0);
        assert_eq!(pair.read(16), b"onetwo");
    }
}

#[test]
fn writing_to_a_descriptor_that_is_not_one_is_an_error() {
    let msgbuf = Msgbuf::new();
    unsafe {
        let buf = Buf::dynamic(0, 16);
        ibuf_add(buf.ptr(), b"one".as_ptr(), 3);
        ibuf_close(msgbuf.ptr(), buf.leak());
        assert_eq!(ibuf_write(-1, msgbuf.ptr()), -1);
        assert_eq!(msgbuf_write(-1, msgbuf.ptr()), -1);
        assert_eq!(msgbuf_queuelen(msgbuf.ptr()), 1);
    }
}

#[test]
fn writing_to_a_full_socket_leaves_the_queue_alone() {
    let msgbuf = Msgbuf::new();
    let mut pair = Pair::new();
    pair.nonblocking();
    unsafe {
        let big = vec![b'x'; 1 << 20];
        let buf = Buf::dynamic(0, big.len() as size_t);
        ibuf_add(buf.ptr(), big.as_ptr(), big.len());
        ibuf_close(msgbuf.ptr(), buf.leak());
        // The first write fills the socket; the queue keeps what is left.
        assert_eq!(ibuf_write(pair.writer(), msgbuf.ptr()), 0);
        assert_eq!(ibuf_write(pair.writer(), msgbuf.ptr()), 0);
        assert_eq!(msgbuf_write(pair.writer(), msgbuf.ptr()), 0);
        assert_eq!(msgbuf_queuelen(msgbuf.ptr()), 1);
        pair.close_writer();
    }
}

#[test]
fn a_write_stops_at_the_descriptor_vector_limit() {
    let msgbuf = Msgbuf::new();
    let pair = Pair::new();
    unsafe {
        for _ in 0..IOV_MAX + 1 {
            let buf = Buf::dynamic(0, 4);
            ibuf_add(buf.ptr(), b"a".as_ptr(), 1);
            ibuf_close(msgbuf.ptr(), buf.leak());
        }
        assert_eq!(ibuf_write(pair.writer(), msgbuf.ptr()), 0);
        assert_eq!(msgbuf_queuelen(msgbuf.ptr()), 1);
        assert_eq!(pair.read(IOV_MAX as usize + 1).len(), IOV_MAX as usize);

        for _ in 0..IOV_MAX {
            let buf = Buf::dynamic(0, 4);
            ibuf_add(buf.ptr(), b"b".as_ptr(), 1);
            ibuf_close(msgbuf.ptr(), buf.leak());
        }
        assert_eq!(msgbuf_queuelen(msgbuf.ptr()), IOV_MAX as uint32_t + 1);
        assert_eq!(msgbuf_write(pair.writer(), msgbuf.ptr()), 0);
        assert_eq!(msgbuf_queuelen(msgbuf.ptr()), 1);
        assert_eq!(pair.read(IOV_MAX as usize + 1).len(), IOV_MAX as usize);

        assert_eq!(msgbuf_write(pair.writer(), msgbuf.ptr()), 0);
        assert_eq!(msgbuf_queuelen(msgbuf.ptr()), 0);
    }
}

#[test]
fn a_partial_write_leaves_the_rest_of_the_buffer_queued() {
    let msgbuf = Msgbuf::new();
    unsafe {
        let buf = Buf::dynamic(0, 16);
        ibuf_add(buf.ptr(), b"abcdef".as_ptr(), 6);
        ibuf_close(msgbuf.ptr(), buf.leak());
        msgbuf_drain(msgbuf.ptr(), 2);
        assert_eq!(msgbuf_queuelen(msgbuf.ptr()), 1);
        assert_eq!(
            bytes_of(&raw const *(*msgbuf.ptr()).bufs.bufs[0] as *mut ibuf),
            b"cdef"
        );

        msgbuf_drain(msgbuf.ptr(), 99);
        assert_eq!(msgbuf_queuelen(msgbuf.ptr()), 0);
    }
}

#[test]
fn a_message_write_carries_a_descriptor_across_and_stops_at_the_next_one() {
    let msgbuf = Msgbuf::new();
    let pair = Pair::new();
    let other = Pair::new();
    unsafe {
        assert_eq!(msgbuf_write(pair.writer(), msgbuf.ptr()), 0);

        let first = Buf::dynamic(0, 16);
        ibuf_add(first.ptr(), b"one".as_ptr(), 3);
        let passed = libc::dup(other.reader());
        ibuf_fd_set(first.ptr(), passed);
        ibuf_close(msgbuf.ptr(), first.leak());

        let second = Buf::dynamic(0, 16);
        ibuf_add(second.ptr(), b"two".as_ptr(), 3);
        ibuf_fd_set(second.ptr(), libc::dup(other.reader()));
        ibuf_close(msgbuf.ptr(), second.leak());

        // The first write carries the first buffer and its descriptor; the
        // second buffer waits, because it has one of its own. The reader
        // receiving it below is what proves the descriptor crossed.
        assert_eq!(msgbuf_write(pair.writer(), msgbuf.ptr()), 0);
        assert_eq!(msgbuf_queuelen(msgbuf.ptr()), 1);

        let reader = Msgbuf::reader();
        assert_eq!(msgbuf_read(pair.reader(), reader.ptr()), 1);
        assert_eq!((*reader.ptr()).roff, 3);

        assert_eq!(msgbuf_write(pair.writer(), msgbuf.ptr()), 0);
        assert_eq!(msgbuf_queuelen(msgbuf.ptr()), 0);
    }
}

#[test]
fn a_message_write_takes_buffers_without_descriptors_up_to_the_first_one_with() {
    let msgbuf = Msgbuf::new();
    let pair = Pair::new();
    unsafe {
        for text in [b"one".as_slice(), b"two".as_slice()] {
            let buf = Buf::dynamic(0, 16);
            ibuf_add(buf.ptr(), text.as_ptr(), text.len());
            ibuf_close(msgbuf.ptr(), buf.leak());
        }
        let last = Buf::dynamic(0, 16);
        ibuf_add(last.ptr(), b"three".as_ptr(), 5);
        ibuf_fd_set(last.ptr(), libc::dup(pair.reader()));
        ibuf_close(msgbuf.ptr(), last.leak());

        assert_eq!(msgbuf_write(pair.writer(), msgbuf.ptr()), 0);
        assert_eq!(msgbuf_queuelen(msgbuf.ptr()), 1);
        assert_eq!(pair.read(16), b"onetwo");
    }
}

#[test]
fn reading_needs_a_buffer_to_read_into() {
    let msgbuf = Msgbuf::new();
    unsafe {
        clear_errno();
        assert_eq!(ibuf_read(0, msgbuf.ptr()), -1);
        assert_eq!(errno(), EINVAL);

        clear_errno();
        assert_eq!(msgbuf_read(0, msgbuf.ptr()), -1);
        assert_eq!(errno(), EINVAL);
    }
}

#[test]
fn reading_takes_what_arrives_and_hands_back_whole_messages() {
    let msgbuf = Msgbuf::reader();
    let pair = Pair::new();
    unsafe {
        let bytes = framed(b"hello");
        assert!(libc::write(pair.writer(), bytes.as_ptr() as *const c_void, bytes.len()) > 0);
        assert_eq!(ibuf_read(pair.reader(), msgbuf.ptr()), 1);
        assert_eq!(msgbuf.messages(), [framed(b"hello")]);
    }
}

#[test]
fn reading_answers_nothing_when_the_other_end_has_gone() {
    let msgbuf = Msgbuf::reader();
    let mut pair = Pair::new();
    pair.close_writer();
    unsafe {
        assert_eq!(ibuf_read(pair.reader(), msgbuf.ptr()), 0);
    }

    let other = Msgbuf::reader();
    let mut gone = Pair::new();
    gone.close_writer();
    unsafe {
        assert_eq!(msgbuf_read(gone.reader(), other.ptr()), 0);
    }
}

#[test]
fn reading_a_socket_with_nothing_on_it_answers_that_it_is_still_open() {
    let msgbuf = Msgbuf::reader();
    let pair = Pair::new();
    pair.nonblocking();
    unsafe {
        assert_eq!(ibuf_read(pair.reader(), msgbuf.ptr()), 1);
        assert_eq!(msgbuf_read(pair.reader(), msgbuf.ptr()), 1);
    }
}

#[test]
fn reading_a_descriptor_that_is_not_one_is_an_error() {
    let msgbuf = Msgbuf::reader();
    unsafe {
        assert_eq!(ibuf_read(-1, msgbuf.ptr()), -1);
        assert_eq!(msgbuf_read(-1, msgbuf.ptr()), -1);
    }
}

#[test]
fn a_message_read_keeps_the_first_descriptor_and_closes_the_rest() {
    let msgbuf = Msgbuf::reader();
    let pair = Pair::new();
    let other = Pair::new();
    unsafe {
        let sent: [c_int; 2] = [libc::dup(other.reader()), libc::dup(other.reader())];
        let bytes = framed(b"fd");
        send_with_fds(pair.writer(), &bytes, &sent);
        close(sent[0]);
        close(sent[1]);

        assert_eq!(msgbuf_read(pair.reader(), msgbuf.ptr()), 1);
        assert_eq!(msgbuf.messages(), [framed(b"fd")]);
    }
}

/// Sends `bytes` over `fd` with `fds` attached, the way `msgbuf_write`
/// would but with more than one descriptor.
fn send_with_fds(fd: c_int, bytes: &[u8], fds: &[c_int]) {
    unsafe {
        let mut iov = iovec {
            iov_base: bytes.as_ptr() as *mut c_void,
            iov_len: bytes.len(),
        };
        let mut space = vec![0u8; libc::CMSG_SPACE((fds.len() * 4) as u32) as usize];
        let mut msg: msghdr = ::core::mem::zeroed();
        msg.msg_iov = &raw mut iov;
        msg.msg_iovlen = 1;
        msg.msg_control = space.as_mut_ptr() as *mut c_void;
        msg.msg_controllen = space.len();
        let cmsg = msg.msg_control as *mut cmsghdr;
        (*cmsg).cmsg_len = libc::CMSG_LEN((fds.len() * 4) as u32) as size_t;
        (*cmsg).cmsg_level = SOL_SOCKET;
        (*cmsg).cmsg_type = SCM_RIGHTS as c_int;
        ::core::ptr::copy_nonoverlapping(
            fds.as_ptr(),
            &raw mut (*cmsg).__cmsg_data as *mut ::core::ffi::c_uchar as *mut c_int,
            fds.len(),
        );
        msg.msg_controllen = (*cmsg).cmsg_len;
        assert!(sendmsg(fd, &raw const msg, 0) > 0);
    }
}
