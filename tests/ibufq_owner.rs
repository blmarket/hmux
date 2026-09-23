use hmux2::src::compat::imsg::{imsgbuf_clear, imsgbuf_init};
use hmux2::src::compat::imsg_buffer::{
    ibuf_add, ibuf_free, ibuf_open, ibuf_write, ibufq_concat, ibufq_flush, ibufq_free, ibufq_new,
    ibufq_pop, ibufq_push, ibufq_queuelen, ibufqueue, msgbuf_clear, msgbuf_free, msgbuf_new,
    msgbuf_write,
};
use std::io::Read;
use std::mem::MaybeUninit;
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;
use std::ptr::null_mut;

unsafe fn entries(queue: *mut ibufqueue) -> Vec<*mut hmux2::src::compat::imsg_buffer::ibuf> {
    (*queue).bufs.iter().collect()
}

unsafe fn assert_empty(queue: *mut ibufqueue) {
    assert_eq!(ibufq_queuelen(queue), 0);
    assert!((*queue).bufs.is_empty());
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
fn imsgbuf_reader_owns_a_live_msgbuf_until_clear() {
    unsafe {
        let mut owner = MaybeUninit::<hmux2::src::compat::imsg::imsgbuf>::zeroed().assume_init();
        assert_eq!(imsgbuf_init(&raw mut owner, -1), 0);
        assert!(!owner.w.is_null());
        imsgbuf_clear(&raw mut owner);
        assert!(owner.w.is_null());
    }
}
