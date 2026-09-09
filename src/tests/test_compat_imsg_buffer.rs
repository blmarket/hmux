use super::*;
use crate::ImsgDataBuffer;
use crate::ffi::{__errno_location, close, socketpair};
use ::core::ffi::{c_char, c_int, c_void};

/// An ibuf that gives itself up at the end of the test.
struct Buf(Option<Box<ibuf>>);

impl Buf {
    fn open(len: size_t) -> Buf {
        Buf(Some(ibuf_open(len).expect("ibuf_open failed")))
    }

    fn dynamic(len: size_t, max: size_t) -> Buf {
        Buf(Some(ibuf_dynamic(len, max).expect("ibuf_dynamic failed")))
    }

    fn borrow_mut(&mut self) -> &mut ibuf {
        self.0.as_deref_mut().expect("a buffer still held")
    }

    /// Hands the buffer over to something that takes ownership of it, such
    /// as a queue.
    fn leak(mut self) -> Box<ibuf> {
        self.0.take().expect("a buffer to hand over")
    }

    /// The bytes between the read and the write position.
    fn bytes(&self) -> Vec<u8> {
        ibuf_data(self).to_vec()
    }
}

impl Drop for Buf {
    fn drop(&mut self) {
        if let Some(buf) = self.0.take() {
            unsafe { ibuf_free(buf) };
        }
    }
}

impl core::ops::Deref for Buf {
    type Target = ibuf;

    fn deref(&self) -> &ibuf {
        self.0.as_deref().expect("a buffer still held")
    }
}

impl core::ops::DerefMut for Buf {
    fn deref_mut(&mut self) -> &mut ibuf {
        self.borrow_mut()
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
            unsafe { msgbuf_new_reader(4, Some(std::rc::Rc::new(read_header)), None) }
                .expect("msgbuf_new_reader failed"),
        )
    }

    fn borrow_mut(&mut self) -> &mut msgbuf {
        &mut self.0
    }

    /// Every message the reader has finished, taken off the queue.
    fn messages(&mut self) -> Vec<Vec<u8>> {
        let mut out = Vec::new();
        loop {
            let Some(p) = (unsafe { msgbuf_get(self.borrow_mut()) }) else {
                return out;
            };
            out.push(Buf(Some(p)).bytes());
        }
    }
}

impl core::ops::Deref for Msgbuf {
    type Target = msgbuf;

    fn deref(&self) -> &msgbuf {
        &self.0
    }
}

impl core::ops::DerefMut for Msgbuf {
    fn deref_mut(&mut self) -> &mut msgbuf {
        self.borrow_mut()
    }
}

/// A four-byte big-endian length, then that many bytes in all.
fn read_header(hdr: &mut ibuf, _limit: Option<uint32_t>, _fd: &mut c_int) -> Option<Box<ibuf>> {
    unsafe {
        let len = ibuf_get_n32(hdr)?;
        ibuf_open(len as size_t)
    }
}

/// A header reader that turns every message down.
fn refuse_header(_hdr: &mut ibuf, _limit: Option<uint32_t>, _fd: &mut c_int) -> Option<Box<ibuf>> {
    unsafe { *__errno_location() = EINVAL };
    None
}

/// A header reader answering a buffer whose write position is already past
/// its own limit, so that the copy into it is refused.
fn broken_header(_hdr: &mut ibuf, _limit: Option<uint32_t>, _fd: &mut c_int) -> Option<Box<ibuf>> {
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
    let mut buf = Buf::open(8);
    assert_eq!(buf.size, 8);
    assert_eq!(buf.max, 8);
    assert_eq!(buf.wpos, 0);
    assert_eq!(buf.fd, -1);
    assert_eq!(unsafe { ibuf_size(&buf) }, 0);
    assert_eq!(unsafe { ibuf_left(&buf) }, 8);
    assert_eq!(buf.bytes(), []);

    let mut empty = Buf::open(0);
    assert_eq!(empty.buf.capacity(), 0);
    assert_eq!(empty.size, 0);
}

#[test]
fn a_dynamic_buffer_grows_up_to_its_limit() {
    let mut buf = Buf::dynamic(0, 8);
    assert_eq!(buf.size, 0);
    assert_eq!(buf.max, 8);
    assert_eq!(buf.buf.capacity(), 0);

    let mut started = Buf::dynamic(4, 8);
    assert_eq!(started.size, 4);
    assert_eq!(started.max, 8);
    assert!(started.buf.capacity() >= 4);
}

#[test]
fn ibuf_implements_buf_and_bufmut() {
    use ::bytes::{Buf as _, BufMut as _};

    let mut buf = Buf::dynamic(0, 8);
    let raw = buf.borrow_mut();
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
fn bufmut_writes_across_initialized_and_spare_storage_without_exceeding_the_limit() {
    use ::bytes::{Buf as _, BufMut as _};
    let mut buf = ibuf_dynamic(3, 8).unwrap();
    buf.put_slice(b"abcdefgh");
    assert_eq!(buf.chunk(), b"abcdefgh");
    assert_eq!(buf.wpos, 8);
    assert_eq!(buf.remaining_mut(), 0);
    assert_eq!(buf.chunk_mut().len(), 0);
    buf.advance(3);
    assert_eq!(buf.chunk(), b"defgh");
    assert_eq!(buf.chunk_mut().len(), 0);
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
    let mut buf = Buf::dynamic(2, 8);
    {
        let first = ibuf_reserve(buf.borrow_mut(), 2);
        assert!(first.is_some());
        assert_eq!(buf.wpos, 2);
        assert_eq!(buf.size, 2);

        let second = ibuf_reserve(buf.borrow_mut(), 4);
        assert!(second.is_some());
        assert_eq!(buf.wpos, 6);
        assert_eq!(buf.size, 6);
        assert_eq!(buf.bytes(), [0, 0, 0, 0, 0, 0]);
    }
}

#[test]
fn reserving_more_than_the_buffer_may_hold_is_refused() {
    let mut buf = Buf::dynamic(0, 8);
    unsafe {
        clear_errno();
        assert!(ibuf_reserve(buf.borrow_mut(), 9).is_none());
        assert_eq!(errno(), ERANGE);

        // Past the limit, and past what the write position may be added
        // to at all.
        assert_eq!(ibuf_add(buf.borrow_mut(), b"a"), 0);
        clear_errno();
        assert!(ibuf_reserve(buf.borrow_mut(), SIZE_MAX as size_t).is_none());
        assert_eq!(errno(), ERANGE);
    }
}

#[test]
fn a_buffer_over_somebody_elses_bytes_is_never_reserved_into() {
    let mut data = *b"abcd";
    let mut buf = empty_ibuf();
    unsafe {
        ibuf_from_buffer(&mut buf, &data);
        assert!(buf.borrowed);
        assert_eq!(ibuf_size(&buf), 4);
        assert_eq!(ibuf_left(&buf), 0);

        clear_errno();
        assert!(ibuf_reserve(&mut buf, 1).is_none());
        assert_eq!(errno(), EINVAL);

        clear_errno();
        assert_eq!(ibuf_truncate(&mut buf, 8), -1);
        assert_eq!(errno(), ERANGE);
    }
}

fn empty_ibuf() -> Box<ibuf> {
    Box::new(ibuf::default())
}

#[test]
fn adding_nothing_adds_nothing() {
    let mut buf = Buf::dynamic(0, 8);
    unsafe {
        assert_eq!(ibuf_add(buf.borrow_mut(), &[]), 0);
        buf.borrow_mut().put_bytes(0, 0);
        assert_eq!(ibuf_size(&buf), 0);
    }
}

#[test]
fn adding_past_the_limit_is_refused() {
    let mut buf = Buf::dynamic(0, 2);
    unsafe {
        assert_eq!(ibuf_add(buf.borrow_mut(), b"abcd"), -1);
        assert!(ibuf_reserve(buf.borrow_mut(), 4).is_none());
        assert_eq!(ibuf_size(&buf), 0);
    }
}

#[test]
fn numbers_are_added_in_network_order_and_in_host_order() {
    let mut buf = Buf::dynamic(0, 64);
    unsafe {
        assert_eq!(ibuf_add_n8(buf.borrow_mut(), 0x12), 0);
        assert_eq!(ibuf_add_n16(buf.borrow_mut(), 0x1234), 0);
        assert_eq!(ibuf_add_n32(buf.borrow_mut(), 0x1234_5678), 0);
        assert_eq!(ibuf_add_n64(buf.borrow_mut(), 0x1234_5678_9abc_def0), 0);
        assert_eq!(
            buf.bytes(),
            [
                0x12, 0x12, 0x34, 0x12, 0x34, 0x56, 0x78, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde,
                0xf0
            ]
        );
    }

    let mut host = Buf::dynamic(0, 64);
    unsafe {
        assert_eq!(ibuf_add_h16(host.borrow_mut(), 0x1234), 0);
        assert_eq!(ibuf_add_h32(host.borrow_mut(), 0x1234_5678), 0);
        assert_eq!(ibuf_add_h64(host.borrow_mut(), 0x1234_5678_9abc_def0), 0);
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
    let mut buf = Buf::dynamic(0, 64);
    unsafe {
        for (retval, width) in [
            (ibuf_add_n8(buf.borrow_mut(), 0x100), 8),
            (ibuf_add_n16(buf.borrow_mut(), 0x1_0000), 16),
            (ibuf_add_n32(buf.borrow_mut(), 0x1_0000_0000), 32),
            (ibuf_add_h16(buf.borrow_mut(), 0x1_0000), 16),
            (ibuf_add_h32(buf.borrow_mut(), 0x1_0000_0000), 32),
        ] {
            assert_eq!(retval, -1, "a value wider than {width} bits was taken");
            assert_eq!(errno(), EINVAL);
        }
        assert_eq!(ibuf_size(&buf), 0);
    }
}

#[test]
fn one_buffer_is_added_to_another() {
    let mut buf = Buf::dynamic(0, 16);
    let mut from = Buf::dynamic(0, 16);
    unsafe {
        ibuf_add(from.borrow_mut(), b"abc");
        assert_eq!(ibuf_add_ibuf(buf.borrow_mut(), &*from), 0);
        assert_eq!(buf.bytes(), b"abc");
    }
}

#[test]
fn a_string_is_added_into_a_field_of_its_own_and_padded_with_zeroes() {
    let mut buf = Buf::dynamic(0, 16);
    unsafe {
        assert_eq!(ibuf_add_strbuf(buf.borrow_mut(), c"ab", 5), 0);
        assert_eq!(buf.bytes(), b"ab\0\0\0");

        clear_errno();
        assert_eq!(ibuf_add_strbuf(buf.borrow_mut(), c"toolong", 4), -1);
        assert_eq!(errno(), EOVERFLOW);

        assert_eq!(ibuf_add_strbuf(buf.borrow_mut(), c"x", 99), -1);
    }
}

#[test]
fn seeking_finds_a_place_inside_what_is_written() {
    let mut buf = Buf::dynamic(0, 16);
    unsafe {
        ibuf_add(buf.borrow_mut(), b"abcd");
        let p = ibuf_seek(buf.borrow_mut(), 1, 2).unwrap();
        assert_eq!(p, b"bc");

        clear_errno();
        assert!(ibuf_seek(buf.borrow_mut(), 5, 0).is_none());
        assert_eq!(errno(), ERANGE);
        assert!(ibuf_seek(buf.borrow_mut(), 2, 3).is_none());
        assert!(ibuf_seek(buf.borrow_mut(), 1, SIZE_MAX as size_t).is_none());
    }
}

#[test]
fn a_number_already_written_is_written_over() {
    let mut buf = Buf::dynamic(0, 64);
    unsafe {
        buf.borrow_mut().put_bytes(0, 32);
        assert_eq!(ibuf_set_n8(buf.borrow_mut(), 0, 0x12), 0);
        assert_eq!(ibuf_set_n16(buf.borrow_mut(), 1, 0x1234), 0);
        assert_eq!(ibuf_set_n32(buf.borrow_mut(), 3, 0x1234_5678), 0);
        assert_eq!(ibuf_set_n64(buf.borrow_mut(), 7, 0x1234_5678_9abc_def0), 0);
        assert_eq!(ibuf_set_h16(buf.borrow_mut(), 15, 0x1234), 0);
        assert_eq!(ibuf_set_h32(buf.borrow_mut(), 17, 0x1234_5678), 0);
        assert_eq!(ibuf_set_h64(buf.borrow_mut(), 21, 0x1234_5678_9abc_def0), 0);
        assert_eq!(
            buf.bytes()[..29],
            [
                0x12, 0x12, 0x34, 0x12, 0x34, 0x56, 0x78, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde,
                0xf0, 0x34, 0x12, 0x78, 0x56, 0x34, 0x12, 0xf0, 0xde, 0xbc, 0x9a, 0x78, 0x56, 0x34,
                0x12
            ]
        );

        assert_eq!(ibuf_set(buf.borrow_mut(), 0, &[]), 0);
        assert_eq!(ibuf_set(buf.borrow_mut(), 64, b"x"), -1);
    }
}

#[test]
fn a_number_too_big_for_the_field_it_is_written_into_is_refused() {
    let mut buf = Buf::dynamic(0, 64);
    unsafe {
        buf.borrow_mut().put_bytes(0, 32);
        assert_eq!(ibuf_set_n8(buf.borrow_mut(), 0, 0x100), -1);
        assert_eq!(ibuf_set_n16(buf.borrow_mut(), 0, 0x1_0000), -1);
        assert_eq!(ibuf_set_n32(buf.borrow_mut(), 0, 0x1_0000_0000), -1);
        assert_eq!(ibuf_set_h16(buf.borrow_mut(), 0, 0x1_0000), -1);
        assert_eq!(ibuf_set_h32(buf.borrow_mut(), 0, 0x1_0000_0000), -1);
        assert_eq!(errno(), EINVAL);
    }
}

#[test]
fn truncating_cuts_what_is_written_or_pads_it_with_zeroes() {
    let mut buf = Buf::dynamic(0, 16);
    unsafe {
        ibuf_add(buf.borrow_mut(), b"abcd");
        assert_eq!(ibuf_truncate(&mut *buf.0.as_deref_mut().unwrap(), 2), 0);
        assert_eq!(buf.bytes(), b"ab");

        assert_eq!(ibuf_truncate(&mut *buf.0.as_deref_mut().unwrap(), 5), 0);
        assert_eq!(buf.bytes(), b"ab\0\0\0");

        assert_eq!(ibuf_truncate(&mut *buf.0.as_deref_mut().unwrap(), 99), -1);
    }
}

#[test]
fn rewinding_puts_the_read_position_back_at_the_start() {
    let mut buf = Buf::dynamic(0, 16);
    unsafe {
        ibuf_add(buf.borrow_mut(), b"abcd");
        buf.borrow_mut().advance(2);
        assert_eq!(buf.bytes(), b"cd");
        ibuf_rewind(buf.borrow_mut());
        assert_eq!(buf.bytes(), b"abcd");

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            buf.borrow_mut().advance(99);
        }));
        assert!(result.is_err());
        assert_eq!(buf.bytes(), b"abcd");
    }
}

#[test]
fn a_buffer_reads_back_what_was_written_into_it() {
    let mut buf = Buf::dynamic(0, 64);
    unsafe {
        ibuf_add_n8(buf.borrow_mut(), 0x12);
        ibuf_add_n16(buf.borrow_mut(), 0x1234);
        ibuf_add_n32(buf.borrow_mut(), 0x1234_5678);
        ibuf_add_n64(buf.borrow_mut(), 0x1234_5678_9abc_def0);
        ibuf_add_h16(buf.borrow_mut(), 0x4321);
        ibuf_add_h32(buf.borrow_mut(), 0x8765_4321);
        ibuf_add_h64(buf.borrow_mut(), 0x0fed_cba9_8765_4321);

        assert_eq!(ibuf_get_n8(buf.borrow_mut()), Some(0x12));
        assert_eq!(ibuf_get_n16(buf.borrow_mut()), Some(0x1234));
        assert_eq!(ibuf_get_n32(buf.borrow_mut()), Some(0x1234_5678));
        assert_eq!(ibuf_get_n64(buf.borrow_mut()), Some(0x1234_5678_9abc_def0));
        assert_eq!(ibuf_get_h16(buf.borrow_mut()), Some(0x4321));
        assert_eq!(ibuf_get_h32(buf.borrow_mut()), Some(0x8765_4321));
        assert_eq!(ibuf_get_h64(buf.borrow_mut()), Some(0x0fed_cba9_8765_4321));
        assert_eq!(ibuf_size(&buf), 0);
    }
}

#[test]
fn reading_past_the_end_is_refused() {
    let mut buf = Buf::dynamic(0, 64);
    unsafe {
        let mut n8: uint8_t = 0;
        clear_errno();
        assert_eq!(
            ibuf_get(buf.borrow_mut(), core::slice::from_mut(&mut n8)),
            -1
        );
        assert_eq!(errno(), EBADMSG);
        assert_eq!(ibuf_get_n8(buf.borrow_mut()), None);
        assert_eq!(ibuf_get_n16(buf.borrow_mut()), None);
        assert_eq!(ibuf_get_n32(buf.borrow_mut()), None);
        assert_eq!(ibuf_get_n64(buf.borrow_mut()), None);
        assert_eq!(ibuf_get_h16(buf.borrow_mut()), None);
        assert_eq!(ibuf_get_h32(buf.borrow_mut()), None);
        assert_eq!(ibuf_get_h64(buf.borrow_mut()), None);
        assert!(ibuf_get_ibuf(buf.borrow_mut(), 1).is_none());
        assert!(ibuf_get_string(buf.borrow_mut(), 1).is_none());
    }
}

#[test]
fn a_buffer_is_read_out_as_a_buffer_over_the_same_bytes() {
    let mut buf = Buf::dynamic(0, 16);
    unsafe {
        ibuf_add(buf.borrow_mut(), b"abcdef");
        let new = ibuf_get_ibuf(buf.borrow_mut(), 3).expect("a buffer");
        assert_eq!(ibuf_data(&new), b"abc");
        assert!(new.borrowed);
        assert_eq!(buf.bytes(), b"def");

        let mut over = empty_ibuf();
        ibuf_from_ibuf(&mut over, &buf);
        assert_eq!(ibuf_size(&over), 3);
        assert_eq!(ibuf_data(&over), b"def");
    }
}

#[test]
fn a_string_is_read_out_and_copied() {
    let mut buf = Buf::dynamic(0, 16);
    unsafe {
        ibuf_add(buf.borrow_mut(), b"abcdef");
        let p = ibuf_get_string(buf.borrow_mut(), 3).expect("string should be available");
        assert_eq!(p.as_bytes(), b"abc");
        assert_eq!(buf.bytes(), b"def");
    }
}

#[test]
fn a_string_field_must_end_in_a_terminator() {
    let mut buf = Buf::dynamic(0, 16);
    let mut out = [0u8; 4];
    unsafe {
        ibuf_add(buf.borrow_mut(), b"ab\0no");
        assert_eq!(ibuf_get_strbuf(buf.borrow_mut(), &mut out[..3]), 0);
        assert_eq!(&out[..3], b"ab\0");

        clear_errno();
        assert_eq!(ibuf_get_strbuf(buf.borrow_mut(), &mut out[..2]), -1);
        assert_eq!(errno(), EOVERFLOW);
        assert_eq!(&out[..2], b"n\0");

        clear_errno();
        assert_eq!(ibuf_get_strbuf(buf.borrow_mut(), &mut out[..0]), -1);
        assert_eq!(errno(), EINVAL);

        assert_eq!(ibuf_get_strbuf(buf.borrow_mut(), &mut out[..4]), -1);
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
    let mut buf = Buf::open(4);
    let pair = Pair::new();
    unsafe {
        assert_eq!(ibuf_fd_avail(&buf), 0);
        assert_eq!(ibuf_fd_get(&mut *buf.0.as_deref_mut().unwrap()), -1);

        let (probe, fd) = CloseProbe::new();
        ibuf_fd_set(&mut *buf.0.as_deref_mut().unwrap(), fd);
        assert_eq!(ibuf_fd_avail(&buf), 1);

        // Setting another one closes the first.
        let second = libc::dup(pair.reader());
        ibuf_fd_set(&mut *buf.0.as_deref_mut().unwrap(), second);
        assert!(probe.closed());

        assert_eq!(ibuf_fd_get(&mut *buf.0.as_deref_mut().unwrap()), second);
        assert_eq!(ibuf_fd_avail(&buf), 0);
        close(second);

        // And a descriptor of -1 leaves the buffer without one.
        ibuf_fd_set(&mut *buf.0.as_deref_mut().unwrap(), -1);
        assert_eq!(buf.fd, -1);
    }
}

#[test]
fn a_freed_buffer_closes_the_descriptor_it_carries() {
    unsafe {
        let (probe, fd) = CloseProbe::new();
        let mut buf = ibuf_open(4).expect("a buffer to carry it");
        ibuf_fd_set(&mut buf, fd);
        assert!(!probe.closed());
        ibuf_free(buf);
        assert!(probe.closed());
    }
}

#[test]
fn a_queue_takes_buffers_at_the_end_and_hands_them_back_from_the_front() {
    unsafe {
        let mut bufq = ibufq_new();
        assert_eq!(ibufq_queuelen(&bufq), 0);
        assert!(ibufq_pop(&mut bufq).is_none());

        let mut first = Buf::open(1).leak();
        let mut second = Buf::open(2).leak();
        let (first_at, second_at) = (&raw const *first, &raw const *second);
        ibufq_push(&mut bufq, first);
        ibufq_push(&mut bufq, second);
        assert_eq!(ibufq_queuelen(&bufq), 2);

        let first = ibufq_pop(&mut bufq).expect("the first buffer back");
        assert_eq!(&raw const *first, first_at);
        assert_eq!(ibufq_queuelen(&bufq), 1);
        let second = ibufq_pop(&mut bufq).expect("the second buffer back");
        assert_eq!(&raw const *second, second_at);
        assert_eq!(ibufq_queuelen(&bufq), 0);
        ibuf_free(first);
        ibuf_free(second);

        ibufq_push(&mut bufq, Buf::open(1).leak());
        ibufq_free(bufq);
    }
}

#[test]
fn one_queue_is_joined_onto_the_end_of_another() {
    unsafe {
        let mut to = ibufq_new();
        let mut from = ibufq_new();
        let mut first = Buf::open(1).leak();
        let mut second = Buf::open(2).leak();
        let (first_at, second_at) = (&raw const *first, &raw const *second);
        ibufq_push(&mut to, first);
        ibufq_push(&mut from, second);

        to.bufs.append(&mut from.bufs);
        assert_eq!(ibufq_queuelen(&to), 2);
        assert_eq!(ibufq_queuelen(&from), 0);
        let first = ibufq_pop(&mut to).expect("the first buffer back");
        assert_eq!(&raw const *first, first_at);
        let second = ibufq_pop(&mut to).expect("the second buffer back");
        assert_eq!(&raw const *second, second_at);
        ibuf_free(first);
        ibuf_free(second);

        // Joining an empty queue on leaves the first one as it was.
        ibufq_push(&mut to, Buf::open(1).leak());
        to.bufs.append(&mut from.bufs);
        assert_eq!(ibufq_queuelen(&to), 1);
        ibufq_flush(&mut to);
        assert_eq!(ibufq_queuelen(&to), 0);
        ibufq_free(to);
        ibufq_free(from);
    }
}

#[test]
fn a_message_buffer_starts_with_two_empty_queues() {
    let mut msgbuf = Msgbuf::new();
    unsafe {
        assert_eq!(msgbuf_queuelen(&*msgbuf), 0);
        assert!(msgbuf_get(msgbuf.borrow_mut()).is_none());
        assert!(msgbuf.rbuf.is_none());

        ibuf_close(msgbuf.borrow_mut(), Buf::open(4).leak());
        assert_eq!(msgbuf_queuelen(&*msgbuf), 1);

        let mut from = ibufq_new();
        ibufq_push(&mut from, Buf::open(4).leak());
        msgbuf.bufs.bufs.append(&mut from.bufs);
        assert_eq!(msgbuf_queuelen(&*msgbuf), 2);
        ibufq_free(from);

        msgbuf_clear(msgbuf.borrow_mut());
        assert_eq!(msgbuf_queuelen(&*msgbuf), 0);
    }
}

#[test]
fn a_reader_needs_a_header_size_that_fits_in_half_its_buffer() {
    unsafe {
        clear_errno();
        assert!(msgbuf_new_reader(0, Some(std::rc::Rc::new(read_header)), None).is_none());
        assert_eq!(errno(), EINVAL);

        clear_errno();
        assert!(
            msgbuf_new_reader(
                (IBUF_READ_SIZE / 2 + 1) as size_t,
                Some(std::rc::Rc::new(read_header)),
                None
            )
            .is_none()
        );
        assert_eq!(errno(), EINVAL);

        let mut msgbuf = Msgbuf::reader();
        assert!(msgbuf.rbuf.is_some());
        assert_eq!(msgbuf.hdrsize, 4);
    }
}

/// Puts `bytes` in the reader's own buffer, as a read would have.
fn filled(msgbuf: &mut Msgbuf, bytes: &[u8]) {
    let msgbuf = msgbuf.borrow_mut();
    let rbuf = msgbuf.rbuf.as_deref_mut().expect("a reader has a buffer");
    rbuf[msgbuf.roff..msgbuf.roff + bytes.len()].copy_from_slice(bytes);
    msgbuf.roff += bytes.len();
}

#[test]
fn the_reader_hands_back_whole_messages_and_keeps_the_rest() {
    let mut msgbuf = Msgbuf::reader();
    unsafe {
        let mut bytes = framed(b"one");
        bytes.extend(framed(b"two"));
        bytes.extend_from_slice(&framed(b"three")[..4]);
        filled(&mut msgbuf, &bytes);

        assert_eq!(ibuf_read_process(msgbuf.borrow_mut(), -1), 1);
        assert_eq!(msgbuf.messages(), [framed(b"one"), framed(b"two")]);
        // The four header bytes of the third message went straight into
        // the message being built, so nothing is left over.
        assert_eq!(msgbuf.roff, 0);
        assert!(msgbuf.rpmsg.is_some());

        filled(&mut msgbuf, b"three");
        assert_eq!(ibuf_read_process(msgbuf.borrow_mut(), -1), 1);
        assert_eq!(msgbuf.messages(), [framed(b"three")]);
        assert_eq!(msgbuf.roff, 0);
        assert!(msgbuf.rpmsg.is_none());
    }
}

#[test]
fn the_reader_waits_for_a_whole_header() {
    let mut msgbuf = Msgbuf::reader();
    unsafe {
        filled(&mut msgbuf, b"ab");
        assert_eq!(ibuf_read_process(msgbuf.borrow_mut(), -1), 1);
        assert!(msgbuf.messages().is_empty());
        assert_eq!(msgbuf.roff, 2);
    }
}

#[test]
fn the_reader_closes_the_descriptor_it_was_handed() {
    let mut msgbuf = Msgbuf::reader();
    unsafe {
        let (probe, fd) = CloseProbe::new();
        filled(&mut msgbuf, &framed(b"one"));
        assert_eq!(ibuf_read_process(msgbuf.borrow_mut(), fd), 1);
        assert!(probe.closed());
    }
}

#[test]
fn a_header_the_reader_turns_down_stops_the_read_and_closes_the_descriptor() {
    unsafe {
        let mut msgbuf = Msgbuf(
            msgbuf_new_reader(4, Some(std::rc::Rc::new(refuse_header)), None).expect("a reader"),
        );
        let (probe, fd) = CloseProbe::new();
        filled(&mut msgbuf, &framed(b"one"));
        assert_eq!(ibuf_read_process(msgbuf.borrow_mut(), fd), -1);
        assert!(probe.closed());
    }
}

#[test]
fn a_message_that_will_not_take_its_own_bytes_stops_the_read() {
    unsafe {
        let mut msgbuf = Msgbuf(
            msgbuf_new_reader(4, Some(std::rc::Rc::new(broken_header)), None).expect("a reader"),
        );
        filled(&mut msgbuf, &framed(b"one"));
        assert_eq!(ibuf_read_process(msgbuf.borrow_mut(), -1), -1);
    }
}

#[test]
fn writing_sends_every_queued_buffer_and_drops_them() {
    let mut msgbuf = Msgbuf::new();
    let pair = Pair::new();
    unsafe {
        assert_eq!(ibuf_write(pair.writer(), msgbuf.borrow_mut()), 0);

        for text in [b"one".as_slice(), b"two".as_slice()] {
            let mut buf = Buf::dynamic(0, 16);
            ibuf_add(buf.borrow_mut(), text);
            ibuf_close(msgbuf.borrow_mut(), buf.leak());
        }
        assert_eq!(ibuf_write(pair.writer(), msgbuf.borrow_mut()), 0);
        assert_eq!(msgbuf_queuelen(&*msgbuf), 0);
        assert_eq!(pair.read(16), b"onetwo");
    }
}

#[test]
fn writing_to_a_descriptor_that_is_not_one_is_an_error() {
    let mut msgbuf = Msgbuf::new();
    unsafe {
        let mut buf = Buf::dynamic(0, 16);
        ibuf_add(buf.borrow_mut(), b"one");
        ibuf_close(msgbuf.borrow_mut(), buf.leak());
        assert_eq!(ibuf_write(-1, msgbuf.borrow_mut()), -1);
        assert_eq!(msgbuf_write(-1, msgbuf.borrow_mut()), -1);
        assert_eq!(msgbuf_queuelen(&*msgbuf), 1);
    }
}

#[test]
fn writing_to_a_full_socket_leaves_the_queue_alone() {
    let mut msgbuf = Msgbuf::new();
    let mut pair = Pair::new();
    pair.nonblocking();
    unsafe {
        let big = vec![b'x'; 1 << 20];
        let mut buf = Buf::dynamic(0, big.len() as size_t);
        ibuf_add(buf.borrow_mut(), &big);
        ibuf_close(msgbuf.borrow_mut(), buf.leak());
        // The first write fills the socket; the queue keeps what is left.
        assert_eq!(ibuf_write(pair.writer(), msgbuf.borrow_mut()), 0);
        assert_eq!(ibuf_write(pair.writer(), msgbuf.borrow_mut()), 0);
        assert_eq!(msgbuf_write(pair.writer(), msgbuf.borrow_mut()), 0);
        assert_eq!(msgbuf_queuelen(&*msgbuf), 1);
        pair.close_writer();
    }
}

#[test]
fn a_write_stops_at_the_descriptor_vector_limit() {
    let mut msgbuf = Msgbuf::new();
    let pair = Pair::new();
    unsafe {
        for _ in 0..IOV_MAX + 1 {
            let mut buf = Buf::dynamic(0, 4);
            ibuf_add(buf.borrow_mut(), b"a");
            ibuf_close(msgbuf.borrow_mut(), buf.leak());
        }
        assert_eq!(ibuf_write(pair.writer(), msgbuf.borrow_mut()), 0);
        assert_eq!(msgbuf_queuelen(&*msgbuf), 1);
        assert_eq!(pair.read(IOV_MAX as usize + 1).len(), IOV_MAX as usize);

        for _ in 0..IOV_MAX {
            let mut buf = Buf::dynamic(0, 4);
            ibuf_add(buf.borrow_mut(), b"b");
            ibuf_close(msgbuf.borrow_mut(), buf.leak());
        }
        assert_eq!(msgbuf_queuelen(&*msgbuf), IOV_MAX as uint32_t + 1);
        assert_eq!(msgbuf_write(pair.writer(), msgbuf.borrow_mut()), 0);
        assert_eq!(msgbuf_queuelen(&*msgbuf), 1);
        assert_eq!(pair.read(IOV_MAX as usize + 1).len(), IOV_MAX as usize);

        assert_eq!(msgbuf_write(pair.writer(), msgbuf.borrow_mut()), 0);
        assert_eq!(msgbuf_queuelen(&*msgbuf), 0);
    }
}

#[test]
fn a_partial_write_leaves_the_rest_of_the_buffer_queued() {
    let mut msgbuf = Msgbuf::new();
    unsafe {
        let mut buf = Buf::dynamic(0, 16);
        ibuf_add(buf.borrow_mut(), b"abcdef");
        ibuf_close(msgbuf.borrow_mut(), buf.leak());
        msgbuf_drain(msgbuf.borrow_mut(), 2);
        assert_eq!(msgbuf_queuelen(&*msgbuf), 1);
        assert_eq!(ibuf_data(&msgbuf.bufs.bufs[0]).to_vec(), b"cdef");

        msgbuf_drain(msgbuf.borrow_mut(), 99);
        assert_eq!(msgbuf_queuelen(&*msgbuf), 0);
    }
}

#[test]
fn a_message_write_carries_a_descriptor_across_and_stops_at_the_next_one() {
    let mut msgbuf = Msgbuf::new();
    let pair = Pair::new();
    let other = Pair::new();
    unsafe {
        assert_eq!(msgbuf_write(pair.writer(), msgbuf.borrow_mut()), 0);

        let mut first = Buf::dynamic(0, 16);
        ibuf_add(first.borrow_mut(), b"one");
        let passed = libc::dup(other.reader());
        ibuf_fd_set(&mut *first.0.as_deref_mut().unwrap(), passed);
        ibuf_close(msgbuf.borrow_mut(), first.leak());

        let mut second = Buf::dynamic(0, 16);
        ibuf_add(second.borrow_mut(), b"two");
        ibuf_fd_set(
            &mut *second.0.as_deref_mut().unwrap(),
            libc::dup(other.reader()),
        );
        ibuf_close(msgbuf.borrow_mut(), second.leak());

        // The first write carries the first buffer and its descriptor; the
        // second buffer waits, because it has one of its own. The reader
        // receiving it below is what proves the descriptor crossed.
        assert_eq!(msgbuf_write(pair.writer(), msgbuf.borrow_mut()), 0);
        assert_eq!(msgbuf_queuelen(&*msgbuf), 1);

        let mut reader = Msgbuf::reader();
        assert_eq!(msgbuf_read(pair.reader(), reader.borrow_mut()), 1);
        assert_eq!(reader.roff, 3);

        assert_eq!(msgbuf_write(pair.writer(), msgbuf.borrow_mut()), 0);
        assert_eq!(msgbuf_queuelen(&*msgbuf), 0);
    }
}

#[test]
fn a_message_write_takes_buffers_without_descriptors_up_to_the_first_one_with() {
    let mut msgbuf = Msgbuf::new();
    let pair = Pair::new();
    unsafe {
        for text in [b"one".as_slice(), b"two".as_slice()] {
            let mut buf = Buf::dynamic(0, 16);
            ibuf_add(buf.borrow_mut(), text);
            ibuf_close(msgbuf.borrow_mut(), buf.leak());
        }
        let mut last = Buf::dynamic(0, 16);
        ibuf_add(last.borrow_mut(), b"three");
        ibuf_fd_set(
            &mut *last.0.as_deref_mut().unwrap(),
            libc::dup(pair.reader()),
        );
        ibuf_close(msgbuf.borrow_mut(), last.leak());

        assert_eq!(msgbuf_write(pair.writer(), msgbuf.borrow_mut()), 0);
        assert_eq!(msgbuf_queuelen(&*msgbuf), 1);
        assert_eq!(pair.read(16), b"onetwo");
    }
}

#[test]
fn reading_needs_a_buffer_to_read_into() {
    let mut msgbuf = Msgbuf::new();
    unsafe {
        clear_errno();
        assert_eq!(ibuf_read(0, msgbuf.borrow_mut()), -1);
        assert_eq!(errno(), EINVAL);

        clear_errno();
        assert_eq!(msgbuf_read(0, msgbuf.borrow_mut()), -1);
        assert_eq!(errno(), EINVAL);
    }
}

#[test]
fn reading_takes_what_arrives_and_hands_back_whole_messages() {
    let mut msgbuf = Msgbuf::reader();
    let pair = Pair::new();
    unsafe {
        let bytes = framed(b"hello");
        assert!(libc::write(pair.writer(), bytes.as_ptr() as *const c_void, bytes.len()) > 0);
        assert_eq!(ibuf_read(pair.reader(), msgbuf.borrow_mut()), 1);
        assert_eq!(msgbuf.messages(), [framed(b"hello")]);
    }
}

#[test]
fn reading_answers_nothing_when_the_other_end_has_gone() {
    let mut msgbuf = Msgbuf::reader();
    let mut pair = Pair::new();
    pair.close_writer();
    unsafe {
        assert_eq!(ibuf_read(pair.reader(), msgbuf.borrow_mut()), 0);
    }

    let mut other = Msgbuf::reader();
    let mut gone = Pair::new();
    gone.close_writer();
    unsafe {
        assert_eq!(msgbuf_read(gone.reader(), other.borrow_mut()), 0);
    }
}

#[test]
fn reading_a_socket_with_nothing_on_it_answers_that_it_is_still_open() {
    let mut msgbuf = Msgbuf::reader();
    let pair = Pair::new();
    pair.nonblocking();
    unsafe {
        assert_eq!(ibuf_read(pair.reader(), msgbuf.borrow_mut()), 1);
        assert_eq!(msgbuf_read(pair.reader(), msgbuf.borrow_mut()), 1);
    }
}

#[test]
fn reading_a_descriptor_that_is_not_one_is_an_error() {
    let mut msgbuf = Msgbuf::reader();
    unsafe {
        assert_eq!(ibuf_read(-1, msgbuf.borrow_mut()), -1);
        assert_eq!(msgbuf_read(-1, msgbuf.borrow_mut()), -1);
    }
}

#[test]
fn a_message_read_keeps_the_first_descriptor_and_closes_the_rest() {
    let mut msgbuf = Msgbuf::reader();
    let pair = Pair::new();
    let other = Pair::new();
    unsafe {
        let sent: [c_int; 2] = [libc::dup(other.reader()), libc::dup(other.reader())];
        let bytes = framed(b"fd");
        send_with_fds(pair.writer(), &bytes, &sent);
        close(sent[0]);
        close(sent[1]);

        assert_eq!(msgbuf_read(pair.reader(), msgbuf.borrow_mut()), 1);
        assert_eq!(msgbuf.messages(), [framed(b"fd")]);
    }
}

/// Sends `bytes` over `fd` with `fds` attached, the way `msgbuf_write`
/// would but with more than one descriptor.
fn send_with_fds(fd: c_int, bytes: &[u8], fds: &[c_int]) {
    unsafe {
        let iov = [IoSlice::new(bytes)];
        let mut space = vec![0u8; libc::CMSG_SPACE((fds.len() * 4) as u32) as usize];
        let mut cmsg = ControlMessage::from_control_message_header(
            &mut space,
            crate::CONTROL_MESSAGE_HEADER_SIZE + std::mem::size_of_val(fds),
            SOL_SOCKET,
            SCM_RIGHTS as c_int,
        )
        .unwrap();
        for (data, fd) in cmsg
            .control_message_data_mut()
            .chunks_exact_mut(size_of::<c_int>())
            .zip(fds)
        {
            data.copy_from_slice(&fd.to_ne_bytes());
        }
        let length = cmsg.control_message_length();
        assert!(crate::message_header::send_socket_message(fd, &iov, &space[..length], 0) > 0);
    }
}

#[test]
fn network_u64_writes_preserve_all_bytes() {
    use ::bytes::BufMut as _;
    for value in [0, 1, u64::MAX, 0x0123_4567_89ab_cdef, 1 << 63]
        .into_iter()
        .chain((0..64).map(|bit| 1u64 << bit))
    {
        let mut buf = Buf::open(8);
        buf.borrow_mut().put_u64(value);
        assert_eq!(buf.bytes(), value.to_be_bytes());
        unsafe {
            assert_eq!(ibuf_set_n64(buf.borrow_mut(), 0, !value), 0);
        }
        assert_eq!(buf.bytes(), (!value).to_be_bytes());
    }
}

#[test]
fn network_u64_reads_preserve_all_bytes_and_advance_the_cursor() {
    use ::bytes::{Buf as _, BufMut as _};
    for value in [0, 1, u64::MAX, 0x0123_4567_89ab_cdef, 1 << 63]
        .into_iter()
        .chain((0..64).map(|bit| 1u64 << bit))
    {
        let mut buf = Buf::open(9);
        buf.borrow_mut().put_slice(&value.to_be_bytes());
        buf.borrow_mut().put_u8(0xa5);
        assert_eq!(buf.borrow_mut().get_u64(), value);
        assert_eq!(buf.bytes(), [0xa5]);
    }
}

#[test]
fn buffer_writes_respect_updated_storage_limits() {
    let mut buf = Buf::dynamic(0, 8);
    buf.set_imsg_data_buffer_max_size(4);
    buf.borrow_mut().put_slice(b"four");
    assert_eq!(buf.borrow_mut().remaining_mut(), 0);
    assert_eq!(unsafe { ibuf_add(buf.borrow_mut(), b"x") }, -1);
    assert_eq!(buf.bytes(), b"four");
    buf.set_imsg_data_buffer_max_size(8);
    assert_eq!(unsafe { ibuf_add(buf.borrow_mut(), b"more") }, 0);
    assert_eq!(buf.bytes(), b"fourmore");
}

/// Puts the bytes of a number at the end of the buffer.
pub(crate) unsafe fn ibuf_add_bytes(buf: &mut ibuf, bytes: &[u8]) -> c_int {
    unsafe { ibuf_add(buf, bytes) }
}

pub(crate) unsafe fn ibuf_add_n8(buf: &mut ibuf, value: uint64_t) -> c_int {
    unsafe {
        if ibuf_too_wide(value, UINT8_MAX as uint64_t) {
            return -(1 as c_int);
        }
        ibuf_add_bytes(buf, &(value as uint8_t).to_ne_bytes())
    }
}

pub(crate) unsafe fn ibuf_add_n16(buf: &mut ibuf, value: uint64_t) -> c_int {
    unsafe {
        if ibuf_too_wide(value, UINT16_MAX as uint64_t) {
            return -(1 as c_int);
        }
        ibuf_add_bytes(buf, &(value as uint16_t).swap_bytes().to_ne_bytes())
    }
}

pub(crate) unsafe fn ibuf_add_n32(buf: &mut ibuf, value: uint64_t) -> c_int {
    unsafe {
        if ibuf_too_wide(value, UINT32_MAX as uint64_t) {
            return -(1 as c_int);
        }
        ibuf_add_bytes(buf, &(value as uint32_t).swap_bytes().to_ne_bytes())
    }
}

pub(crate) unsafe fn ibuf_add_n64(buf: &mut ibuf, value: uint64_t) -> c_int {
    unsafe { ibuf_add_bytes(buf, &value.to_be_bytes()) }
}

pub(crate) unsafe fn ibuf_add_h16(buf: &mut ibuf, value: uint64_t) -> c_int {
    unsafe {
        if ibuf_too_wide(value, UINT16_MAX as uint64_t) {
            return -(1 as c_int);
        }
        ibuf_add_bytes(buf, &(value as uint16_t).to_ne_bytes())
    }
}

pub(crate) unsafe fn ibuf_add_h32(buf: &mut ibuf, value: uint64_t) -> c_int {
    unsafe {
        if ibuf_too_wide(value, UINT32_MAX as uint64_t) {
            return -(1 as c_int);
        }
        ibuf_add_bytes(buf, &(value as uint32_t).to_ne_bytes())
    }
}

pub(crate) unsafe fn ibuf_add_h64(buf: &mut ibuf, value: uint64_t) -> c_int {
    unsafe { ibuf_add_bytes(buf, &value.to_ne_bytes()) }
}

pub(crate) unsafe fn ibuf_add_strbuf(buf: &mut ibuf, str: &std::ffi::CStr, len: size_t) -> c_int {
    unsafe {
        use crate::ffi::strlcpy;

        let Some(b) = ibuf_reserve(buf, len) else {
            return -(1 as c_int);
        };
        let n = strlcpy(b.as_mut_ptr() as *mut c_char, str.as_ptr(), len) as size_t;
        if n >= len {
            return ibuf_fail(EOVERFLOW, -(1 as c_int));
        }
        b[n..].fill(0);
        0 as c_int
    }
}

pub(crate) unsafe fn ibuf_set_n8(buf: &mut ibuf, pos: size_t, value: uint64_t) -> c_int {
    if ibuf_too_wide(value, UINT8_MAX as uint64_t) {
        return -(1 as c_int);
    }
    unsafe { ibuf_set_bytes(buf, pos, &(value as uint8_t).to_ne_bytes()) }
}

pub(crate) unsafe fn ibuf_set_n16(buf: &mut ibuf, pos: size_t, value: uint64_t) -> c_int {
    if ibuf_too_wide(value, UINT16_MAX as uint64_t) {
        return -(1 as c_int);
    }
    unsafe { ibuf_set_bytes(buf, pos, &(value as uint16_t).swap_bytes().to_ne_bytes()) }
}

pub(crate) unsafe fn ibuf_set_n32(buf: &mut ibuf, pos: size_t, value: uint64_t) -> c_int {
    if ibuf_too_wide(value, UINT32_MAX as uint64_t) {
        return -(1 as c_int);
    }
    unsafe { ibuf_set_bytes(buf, pos, &(value as uint32_t).swap_bytes().to_ne_bytes()) }
}

pub(crate) unsafe fn ibuf_set_n64(buf: &mut ibuf, pos: size_t, value: uint64_t) -> c_int {
    unsafe { ibuf_set_bytes(buf, pos, &value.to_be_bytes()) }
}

pub(crate) unsafe fn ibuf_set_h16(buf: &mut ibuf, pos: size_t, value: uint64_t) -> c_int {
    if ibuf_too_wide(value, UINT16_MAX as uint64_t) {
        return -(1 as c_int);
    }
    unsafe { ibuf_set_bytes(buf, pos, &(value as uint16_t).to_ne_bytes()) }
}

pub(crate) unsafe fn ibuf_set_h64(buf: &mut ibuf, pos: size_t, value: uint64_t) -> c_int {
    unsafe { ibuf_set_bytes(buf, pos, &value.to_ne_bytes()) }
}

pub(crate) unsafe fn ibuf_truncate(buf: &mut ibuf, len: size_t) -> c_int {
    unsafe {
        if ibuf_size(buf) >= len {
            buf.wpos = buf.rpos.wrapping_add(len);
            return 0 as c_int;
        }
        if ibuf_on_stack(buf) {
            return ibuf_fail(ERANGE, -(1 as c_int));
        }
        let padding = len.wrapping_sub(ibuf_size(buf));
        let Some(bytes) = ibuf_reserve(buf, padding) else {
            return -(1 as c_int);
        };
        bytes.fill(0);
        0 as c_int
    }
}

pub(crate) unsafe fn ibuf_from_ibuf(buf: &mut ibuf, from: &ibuf) {
    unsafe {
        let data = ibuf_data(from);
        ibuf_from_buffer(buf, data);
    }
}

/// Takes a number out of the buffer as it lies there, without changing its
/// byte order.
pub(crate) unsafe fn ibuf_get_h16(buf: &mut ibuf) -> Option<uint16_t> {
    let mut bytes = [0; size_of::<uint16_t>()];
    if unsafe { ibuf_get(buf, &mut bytes) } < 0 {
        None
    } else {
        Some(uint16_t::from_ne_bytes(bytes))
    }
}

pub(crate) unsafe fn ibuf_get_h32(buf: &mut ibuf) -> Option<uint32_t> {
    let mut bytes = [0; size_of::<uint32_t>()];
    if unsafe { ibuf_get(buf, &mut bytes) } < 0 {
        None
    } else {
        Some(uint32_t::from_ne_bytes(bytes))
    }
}

pub(crate) unsafe fn ibuf_get_h64(buf: &mut ibuf) -> Option<uint64_t> {
    let mut bytes = [0; size_of::<uint64_t>()];
    if unsafe { ibuf_get(buf, &mut bytes) } < 0 {
        None
    } else {
        Some(uint64_t::from_ne_bytes(bytes))
    }
}

pub(crate) unsafe fn ibuf_get_n8(buf: &mut ibuf) -> Option<uint8_t> {
    let mut bytes = [0; 1];
    if unsafe { ibuf_get(buf, &mut bytes) } < 0 {
        None
    } else {
        Some(bytes[0])
    }
}

pub(crate) unsafe fn ibuf_get_n16(buf: &mut ibuf) -> Option<uint16_t> {
    let mut bytes = [0; size_of::<uint16_t>()];
    if unsafe { ibuf_get(buf, &mut bytes) } < 0 {
        None
    } else {
        Some(uint16_t::from_be_bytes(bytes))
    }
}

pub(crate) unsafe fn ibuf_get_n32(buf: &mut ibuf) -> Option<uint32_t> {
    let mut bytes = [0; size_of::<uint32_t>()];
    if unsafe { ibuf_get(buf, &mut bytes) } < 0 {
        None
    } else {
        Some(uint32_t::from_be_bytes(bytes))
    }
}

pub(crate) unsafe fn ibuf_get_n64(buf: &mut ibuf) -> Option<uint64_t> {
    let mut bytes = [0; size_of::<uint64_t>()];
    if unsafe { ibuf_get(buf, &mut bytes) } < 0 {
        None
    } else {
        Some(uint64_t::from_be_bytes(bytes))
    }
}

pub(crate) unsafe fn ibuf_get_string(buf: &mut ibuf, len: size_t) -> Option<std::ffi::CString> {
    unsafe {
        use std::ffi::CString;

        if ibuf_size(buf) < len {
            ibuf_fail(EBADMSG, null_mut::<c_char>());
            return None;
        }
        let bytes = &ibuf_data(buf)[..len];
        let end = bytes
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(bytes.len());
        let string = CString::new(&bytes[..end]).expect("ibuf string has no NUL");
        buf.rpos = buf.rpos.wrapping_add(len);
        Some(string)
    }
}

pub(crate) unsafe fn ibuf_get_strbuf(buf: &mut ibuf, str: &mut [u8]) -> c_int {
    unsafe {
        if str.is_empty() {
            return ibuf_fail(EINVAL, -(1 as c_int));
        }
        if ibuf_get(buf, str) == -(1 as c_int) {
            return -(1 as c_int);
        }
        if *str.last().unwrap() != 0 {
            *str.last_mut().unwrap() = 0;
            return ibuf_fail(EOVERFLOW, -(1 as c_int));
        }
        0 as c_int
    }
}

pub(crate) fn ibufq_new() -> Box<ibufqueue> {
    Box::new(ibufqueue {
        bufs: std::collections::VecDeque::new(),
    })
}

pub(crate) unsafe fn ibufq_free(mut bufq: Box<ibufqueue>) {
    unsafe {
        ibufq_flush(&mut bufq);
        drop(bufq);
    }
}

pub(crate) unsafe fn ibuf_rewind(buf: &mut ibuf) {
    buf.rpos = 0 as size_t;
}
