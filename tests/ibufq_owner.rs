use hmux2::src::compat::imsg::{imsgbuf_clear, imsgbuf_init};
use hmux2::src::compat::imsg_buffer::{
    ibuf_add, ibuf_data, ibuf_dynamic, ibuf_fd_get, ibuf_fd_set, ibuf_free, ibuf_from_buffer,
    ibuf_from_ibuf, ibuf_get, ibuf_get_string, ibuf_open, ibuf_read, ibuf_reserve, ibuf_size,
    ibuf_write, ibufq_concat, ibufq_flush, ibufq_free, ibufq_new, ibufq_pop, ibufq_push,
    ibufq_queuelen, ibufqueue, msgbuf_clear, msgbuf_free, msgbuf_new, msgbuf_new_reader,
    msgbuf_write, EINVAL,
};
use std::io::Read;
use std::mem::MaybeUninit;
use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd};
use std::os::unix::net::UnixStream;
use std::ptr::null_mut;

unsafe fn entries(queue: *mut ibufqueue) -> Vec<*mut hmux2::src::compat::imsg_buffer::ibuf> {
    (*queue).bufs.iter().collect()
}

unsafe fn assert_empty(queue: *mut ibufqueue) {
    assert_eq!(ibufq_queuelen(queue), 0);
    assert!((*queue).bufs.is_empty());
}

unsafe extern "C" fn read_two_byte_message(
    _header: *mut hmux2::src::compat::imsg_buffer::ibuf,
    _arg: *mut std::ffi::c_void,
    _fd: *mut std::ffi::c_int,
) -> *mut hmux2::src::compat::imsg_buffer::ibuf {
    ibuf_open(2)
}

#[test]
fn standalone_ibufq_owns_buffers_and_moves_them_between_queues() {
    unsafe {
        ibufq_free(null_mut());

        let destination = ibufq_new();
        let source = ibufq_new();
        assert!(!destination.is_null());
        assert!(!source.is_null());
        assert_empty(destination);
        assert_empty(source);

        let first = ibuf_open(1);
        let second = ibuf_open(1);
        let third = ibuf_open(1);
        assert!(!first.is_null());
        assert!(!second.is_null());
        assert!(!third.is_null());

        ibufq_push(destination, first);
        ibufq_push(destination, second);
        ibufq_push(source, third);
        assert_eq!(ibufq_queuelen(destination), 2);
        assert_eq!(ibufq_queuelen(source), 1);
        assert_eq!(entries(destination), vec![first, second]);

        assert_eq!(ibufq_pop(destination), first);
        ibuf_free(first);
        assert_eq!(ibufq_queuelen(destination), 1);
        assert_eq!(entries(destination), vec![second]);

        ibufq_concat(destination, source);
        assert_empty(source);
        assert_eq!(ibufq_queuelen(destination), 2);
        assert_eq!(entries(destination), vec![second, third]);

        assert_eq!(ibufq_pop(destination), second);
        ibuf_free(second);
        assert_eq!(entries(destination), vec![third]);

        // The destination destructor flushes its remaining owned buffer.
        ibufq_free(destination);

        let fourth = ibuf_open(1);
        assert!(!fourth.is_null());
        ibufq_push(source, fourth);
        ibufq_flush(source);
        assert_empty(source);
        ibufq_free(source);
    }
}

#[test]
fn msgbuf_initializes_and_drops_both_owned_queues() {
    unsafe {
        let owner = msgbuf_new();
        assert!(!owner.is_null());

        let outgoing = ibuf_open(1);
        let incoming = ibuf_open(1);
        assert!(!outgoing.is_null());
        assert!(!incoming.is_null());
        ibufq_push(&raw mut (*owner).bufs, outgoing);
        ibufq_push(&raw mut (*owner).rbufs, incoming);
        assert_eq!(ibufq_queuelen(&raw mut (*owner).bufs), 1);
        assert_eq!(ibufq_queuelen(&raw mut (*owner).rbufs), 1);

        msgbuf_clear(owner);
        assert_empty(&raw mut (*owner).bufs);
        assert_empty(&raw mut (*owner).rbufs);

        // Clear leaves both owners valid for reuse; free then drops the collections.
        let after_clear = ibuf_open(1);
        assert!(!after_clear.is_null());
        ibufq_push(&raw mut (*owner).bufs, after_clear);
        msgbuf_free(owner);
    }
}

unsafe fn queue_payload(queue: *mut ibufqueue, payload: &[u8]) {
    let buf = ibuf_open(payload.len());
    assert!(!buf.is_null());
    assert_eq!(ibuf_add(buf, payload.as_ptr().cast(), payload.len()), 0);
    ibufq_push(queue, buf);
}

#[test]
fn message_writers_preserve_owned_queue_order_and_drain_buffers() {
    unsafe {
        let (writer, mut reader) = UnixStream::pair().unwrap();
        let outgoing = msgbuf_new();
        assert!(!outgoing.is_null());
        queue_payload(&raw mut (*outgoing).bufs, b"first-");
        queue_payload(&raw mut (*outgoing).bufs, b"second");

        assert_eq!(ibuf_write(writer.as_raw_fd(), outgoing), 0);
        let mut output = [0; 12];
        reader.read_exact(&mut output).unwrap();
        assert_eq!(&output, b"first-second");
        assert_eq!(ibufq_queuelen(&raw mut (*outgoing).bufs), 0);
        msgbuf_free(outgoing);

        let (writer, mut reader) = UnixStream::pair().unwrap();
        let outgoing = msgbuf_new();
        assert!(!outgoing.is_null());
        queue_payload(&raw mut (*outgoing).bufs, b"left-");
        queue_payload(&raw mut (*outgoing).bufs, b"right");

        assert_eq!(msgbuf_write(writer.as_raw_fd(), outgoing), 0);
        let mut output = [0; 10];
        reader.read_exact(&mut output).unwrap();
        assert_eq!(&output, b"left-right");
        assert_eq!(ibufq_queuelen(&raw mut (*outgoing).bufs), 0);
        msgbuf_free(outgoing);
    }
}

#[test]
fn boxed_payload_growth_preserves_bytes_and_zeroes_new_capacity() {
    unsafe {
        let buf = ibuf_dynamic(1, 4);
        assert!(!buf.is_null());
        let first = [b'a'];
        assert_eq!(ibuf_add(buf, first.as_ptr().cast(), first.len()), 0);

        let grown = ibuf_reserve(buf, 3) as *mut u8;
        assert!(!grown.is_null());
        assert_eq!(std::slice::from_raw_parts(grown, 3), &[0, 0, 0]);
        grown.copy_from_nonoverlapping(b"bcd".as_ptr(), 3);
        assert_eq!(ibuf_size(buf), 4);
        assert_eq!(
            std::slice::from_raw_parts(ibuf_data(buf) as *const u8, ibuf_size(buf)),
            b"abcd"
        );
        ibuf_free(buf);
    }
}

#[test]
fn string_reads_return_owned_cstrings_and_consume_the_declared_length() {
    unsafe {
        let buf = ibuf_open(4);
        assert!(!buf.is_null());
        let payload = [b'a', 0, b'b', b'c'];
        assert_eq!(ibuf_add(buf, payload.as_ptr().cast(), payload.len()), 0);

        let errno = hmux2::src::ffi::libc::__errno_location();
        *errno = EINVAL;
        let string = ibuf_get_string(buf, payload.len()).unwrap();
        assert_eq!(string.as_bytes(), b"a");
        assert_eq!(*errno, EINVAL);
        assert_eq!(ibuf_size(buf), 0);
        ibuf_free(buf);

        let empty = ibuf_open(0);
        let string = ibuf_get_string(empty, 0).unwrap();
        assert!(string.as_bytes().is_empty());
        ibuf_free(empty);
    }
}

#[test]
fn descriptor_transfer_survives_buffer_drop_and_owned_descriptors_close() {
    unsafe {
        let (attached, mut peer) = UnixStream::pair().unwrap();
        let buf = ibuf_open(0);
        ibuf_fd_set(buf, attached.into_raw_fd());
        let transferred = ibuf_fd_get(buf);
        ibuf_free(buf);
        let mut transferred = UnixStream::from_raw_fd(transferred);
        use std::io::Write;
        transferred.write_all(b"x").unwrap();
        let mut byte = [0];
        peer.read_exact(&mut byte).unwrap();
        assert_eq!(byte, [b'x']);
        drop(transferred);

        let (attached, mut peer) = UnixStream::pair().unwrap();
        peer.set_read_timeout(Some(std::time::Duration::from_secs(1)))
            .unwrap();
        let buf = ibuf_open(0);
        ibuf_fd_set(buf, attached.into_raw_fd());
        ibuf_free(buf);
        assert_eq!(peer.read(&mut byte).unwrap(), 0);
    }
}

#[test]
fn reader_scratch_parses_and_queues_complete_messages() {
    unsafe {
        let (mut writer, reader) = UnixStream::pair().unwrap();
        let owner = msgbuf_new_reader(1, Some(read_two_byte_message), null_mut());
        assert!(!owner.is_null());
        use std::io::Write;
        writer.write_all(b"abcd").unwrap();

        assert_eq!(ibuf_read(reader.as_raw_fd(), owner), 1);
        assert_eq!(ibufq_queuelen(&raw mut (*owner).rbufs), 2);
        for expected in [&b"ab"[..], &b"cd"[..]] {
            let buf = hmux2::src::compat::imsg_buffer::msgbuf_get(owner);
            assert!(!buf.is_null());
            assert_eq!(ibuf_size(buf), expected.len());
            assert_eq!(
                std::slice::from_raw_parts(ibuf_data(buf) as *const u8, ibuf_size(buf)),
                expected
            );
            ibuf_free(buf);
        }
        msgbuf_free(owner);
    }
}

#[test]
fn borrowed_views_keep_source_bytes_and_cursor_separate() {
    unsafe {
        let mut bytes = *b"abcd";
        let mut view = MaybeUninit::uninit();
        ibuf_from_buffer(view.as_mut_ptr(), bytes.as_mut_ptr().cast(), bytes.len());
        let mut view = view.assume_init();
        let mut read = [0u8; 2];
        assert_eq!(ibuf_get(&raw mut view, read.as_mut_ptr().cast(), 2), 0);
        assert_eq!(&read, b"ab");
        assert_eq!(ibuf_size(&raw mut view), 2);

        let owner = ibuf_open(3);
        ibuf_add(owner, b"xyz".as_ptr().cast(), 3);
        let mut subview = MaybeUninit::uninit();
        ibuf_from_ibuf(subview.as_mut_ptr(), owner);
        let mut subview = subview.assume_init();
        assert_eq!(ibuf_size(&raw mut subview), 3);
        drop(subview);
        ibuf_free(owner);
        drop(view);
        assert_eq!(&bytes, b"abcd");
    }
}

#[test]
fn read_header_failure_drops_borrowed_view_without_queueing() {
    unsafe extern "C" fn reject_header(
        header: *mut hmux2::src::compat::imsg_buffer::ibuf,
        arg: *mut std::ffi::c_void,
        _: *mut std::ffi::c_int,
    ) -> *mut hmux2::src::compat::imsg_buffer::ibuf {
        *(arg as *mut bool) = ibuf_size(header) == 1 && *(ibuf_data(header) as *const u8) == b'x';
        null_mut()
    }

    unsafe {
        let (mut writer, reader) = UnixStream::pair().unwrap();
        let mut saw_header = false;
        let owner = msgbuf_new_reader(1, Some(reject_header), (&raw mut saw_header).cast());
        use std::io::Write;
        writer.write_all(b"x").unwrap();
        assert_eq!(ibuf_read(reader.as_raw_fd(), owner), -1);
        assert!(saw_header);
        assert_eq!(ibufq_queuelen(&raw mut (*owner).rbufs), 0);
        msgbuf_free(owner);
    }
}

#[test]
fn imsgbuf_reader_owns_a_live_msgbuf_until_clear() {
    unsafe {
        let mut owner = MaybeUninit::<hmux2::src::compat::imsg::imsgbuf>::zeroed().assume_init();
        assert_eq!(imsgbuf_init(&raw mut owner, -1), 0);
        assert!(!owner.w.is_null());
        imsgbuf_clear(&raw mut owner);
        assert!(owner.w.is_null());
    }
}
