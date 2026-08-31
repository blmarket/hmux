use super::*;
use crate::compat::{ibuf_add_zero, ibufq_free, ibufq_new, ibufq_queuelen, msgbuf};
use crate::ffi::{close, socketpair};
use crate::tests::test_fixtures::zeroed;
use ::core::ffi::{CStr, c_char, c_int, c_void};
use ::core::ptr::{null, null_mut};

/// A connected pair of unix stream sockets with an `imsgbuf` on each end,
/// all closed at the end of the test.
struct Link {
    fds: [c_int; 2],
    write: Box<imsgbuf>,
    read: Box<imsgbuf>,
}

impl Link {
    fn new() -> Link {
        let mut fds: [c_int; 2] = [-1, -1];
        assert_eq!(
            unsafe { socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, fds.as_mut_ptr()) },
            0
        );
        let mut link = Link {
            fds,
            write: zeroed::<imsgbuf>(),
            read: zeroed::<imsgbuf>(),
        };
        unsafe {
            assert_eq!(imsgbuf_init(&mut link.write, fds[0]), 0);
            assert_eq!(imsgbuf_init(&mut link.read, fds[1]), 0);
        }
        link
    }

    /// Lets both ends pass descriptors.
    fn with_fdpass(mut self) -> Link {
        imsgbuf_allow_fdpass(&mut self.write);
        imsgbuf_allow_fdpass(&mut self.read);
        self
    }

    fn writer(&mut self) -> &mut imsgbuf {
        &mut self.write
    }

    fn reader(&mut self) -> &mut imsgbuf {
        &mut self.read
    }

    /// Sends everything queued and takes the next message off the far end.
    fn carry(&mut self) -> Option<Message> {
        unsafe {
            assert_eq!(imsgbuf_flush(self.writer()), 0);
            assert_eq!(imsgbuf_read(self.reader()), 1);
            self.next()
        }
    }

    /// The next message the reading end has, if it has one.
    fn next(&mut self) -> Option<Message> {
        let mut m = Message(Box::default());
        match unsafe { imsgbuf_get(self.reader(), &raw mut *m.0) } {
            0 => None,
            1 => Some(m),
            other => panic!("imsgbuf_get answered {other}"),
        }
    }
}

impl Drop for Link {
    fn drop(&mut self) {
        unsafe {
            imsgbuf_clear(&mut self.write);
            imsgbuf_clear(&mut self.read);
            for fd in self.fds {
                close(fd);
            }
        }
    }
}

/// A message that frees its buffer at the end of the test.
struct Message(Box<imsg>);

impl Message {
    fn ptr(&mut self) -> *mut imsg {
        &raw mut *self.0
    }

    /// The message's payload, header aside.
    fn data(&mut self) -> Vec<u8> {
        unsafe {
            let len = imsg_get_len(self.ptr());
            if len == 0 {
                return Vec::new();
            }
            ::core::slice::from_raw_parts((*self.ptr()).data as *const u8, len).to_vec()
        }
    }
}

impl Drop for Message {
    fn drop(&mut self) {
        unsafe { imsg_free(&raw mut *self.0) };
    }
}

fn errno() -> c_int {
    unsafe { *__errno_location() }
}

fn clear_errno() {
    unsafe { *__errno_location() = 0 };
}

#[test]
fn a_message_buffer_starts_with_this_process_and_the_default_limit() {
    let mut link = Link::new();
    unsafe {
        assert_eq!(link.writer().pid, getpid() as pid_t);
        assert_eq!(link.writer().maxsize, MAX_IMSGSIZE as uint32_t);
        let fd = link.fds[0];
        assert_eq!(link.writer().fd, fd);
        assert_eq!(link.writer().flags, 0);
        assert!(link.writer().w.is_some());
        assert_eq!(imsgbuf_queuelen(link.writer()), 0);

        imsgbuf_allow_fdpass(link.writer());
        assert_eq!(link.writer().flags, IMSG_ALLOW_FDPASS);
    }
}

#[test]
fn a_composed_message_arrives_with_its_type_id_pid_and_payload() {
    let mut link = Link::new();
    unsafe {
        assert_eq!(
            imsg_compose(link.writer(), 7, 99, 1234, -1, b"payload".as_ptr(), 7),
            1
        );
        assert_eq!(imsgbuf_queuelen(link.writer()), 1);

        let mut m = link.carry().expect("a message arrived");
        assert_eq!(imsg_get_type(m.ptr()), 7);
        assert_eq!(imsg_get_id(m.ptr()), 99);
        assert_eq!(imsg_get_pid(m.ptr()), 1234);
        assert_eq!(imsg_get_len(m.ptr()), 7);
        assert_eq!(m.data(), b"payload");
        assert_eq!(imsg_get_fd(m.ptr()), -1);
    }
}

#[test]
fn a_message_composed_without_a_pid_carries_this_process_id() {
    let mut link = Link::new();
    unsafe {
        assert_eq!(imsg_compose(link.writer(), 1, 0, 0, -1, null(), 0), 1);
        let mut m = link.carry().expect("a message arrived");
        assert_eq!(imsg_get_pid(m.ptr()), getpid() as pid_t);
        assert_eq!(imsg_get_len(m.ptr()), 0);
        assert_eq!((*m.ptr()).data, null_mut());
    }
}

#[test]
fn a_message_bigger_than_the_limit_is_refused() {
    let mut link = Link::new();
    let big = vec![b'x'; MAX_IMSGSIZE as usize];
    unsafe {
        clear_errno();
        assert_eq!(
            imsg_compose(link.writer(), 1, 0, 0, -1, big.as_ptr(), big.len()),
            -1
        );
        assert_eq!(errno(), ERANGE);
        assert_eq!(imsgbuf_queuelen(link.writer()), 0);
    }
}

#[test]
fn a_message_may_be_composed_from_several_pieces() {
    let mut link = Link::new();
    unsafe {
        let iov = [
            iovec {
                iov_base: b"one".as_ptr() as *mut c_void,
                iov_len: 3,
            },
            iovec {
                iov_base: b"two".as_ptr() as *mut c_void,
                iov_len: 3,
            },
        ];
        assert_eq!(
            imsg_composev(link.writer(), 2, 0, 0, -1, iov.as_ptr(), 2),
            1
        );
        let mut m = link.carry().expect("a message arrived");
        assert_eq!(m.data(), b"onetwo");

        // No pieces at all is an empty message.
        assert_eq!(
            imsg_composev(link.writer(), 2, 0, 0, -1, null::<iovec>(), 0),
            1
        );
        let mut empty = link.carry().expect("a message arrived");
        assert_eq!(imsg_get_len(empty.ptr()), 0);
    }
}

#[test]
fn pieces_that_do_not_fit_together_are_refused() {
    let mut link = Link::new();
    let big = vec![b'x'; MAX_IMSGSIZE as usize];
    unsafe {
        let iov = [iovec {
            iov_base: big.as_ptr() as *mut c_void,
            iov_len: big.len(),
        }];
        assert_eq!(
            imsg_composev(link.writer(), 2, 0, 0, -1, iov.as_ptr(), 1),
            -1
        );
        assert_eq!(imsgbuf_queuelen(link.writer()), 0);
    }
}

#[test]
fn a_piece_the_message_will_not_take_stops_the_compose() {
    let mut link = Link::new();
    unsafe {
        // The first piece fills the message the header sized; the second
        // is past the limit the buffer was given.
        let room = MAX_IMSGSIZE as usize - IMSG_HEADER_SIZE;
        let first = vec![b'x'; room];
        let iov = [
            iovec {
                iov_base: first.as_ptr() as *mut c_void,
                iov_len: first.len(),
            },
            iovec {
                iov_base: b"more".as_ptr() as *mut c_void,
                iov_len: 4,
            },
        ];
        // Together they are past the limit, so the create is what refuses.
        assert_eq!(
            imsg_composev(link.writer(), 2, 0, 0, -1, iov.as_ptr(), 2),
            -1
        );

        // Sized for one piece but handed two, the add is what refuses.
        let mut wbuf = imsg_create(link.writer(), 2, 0, 0, 4).expect("a message to fill");
        assert_eq!(imsg_set_maxsize(&raw mut *wbuf, 4), 0);
        assert_eq!(ibuf_add(&raw mut *wbuf, b"toolong".as_ptr(), 7), -1);
        ibuf_free(wbuf);
    }
}

#[test]
fn a_message_may_be_composed_from_a_buffer_that_is_handed_over() {
    let mut link = Link::new();
    unsafe {
        let mut buf = ibuf_dynamic(0, 64).expect("a buffer to hand over");
        ibuf_add(&raw mut *buf, b"handed".as_ptr(), 6);
        assert_eq!(imsg_compose_ibuf(link.writer(), 3, 11, 22, buf), 1);
        assert_eq!(imsgbuf_queuelen(link.writer()), 2);

        let mut m = link.carry().expect("a message arrived");
        assert_eq!(imsg_get_type(m.ptr()), 3);
        assert_eq!(imsg_get_id(m.ptr()), 11);
        assert_eq!(imsg_get_pid(m.ptr()), 22);
        assert_eq!(m.data(), b"handed");
    }
}

#[test]
fn a_handed_over_buffer_too_big_for_the_limit_is_refused_and_freed() {
    let mut link = Link::new();
    unsafe {
        let mut buf = ibuf_dynamic(0, (MAX_IMSGSIZE + 1) as size_t).expect("a buffer to hand over");
        ibuf_add_zero(&raw mut *buf, MAX_IMSGSIZE as size_t);
        clear_errno();
        assert_eq!(imsg_compose_ibuf(link.writer(), 3, 0, 0, buf), -1);
        assert_eq!(errno(), ERANGE);
        assert_eq!(imsgbuf_queuelen(link.writer()), 0);
    }
}

#[test]
fn a_handed_over_buffer_with_no_pid_carries_this_process_id() {
    let mut link = Link::new();
    unsafe {
        let buf = ibuf_dynamic(0, 64).expect("a buffer to hand over");
        assert_eq!(imsg_compose_ibuf(link.writer(), 3, 0, 0, buf), 1);
        let mut m = link.carry().expect("a message arrived");
        assert_eq!(imsg_get_pid(m.ptr()), getpid() as pid_t);
    }
}

#[test]
fn a_message_is_forwarded_with_its_header_and_payload() {
    let mut link = Link::new();
    let mut onward = Link::new();
    unsafe {
        imsg_compose(link.writer(), 5, 6, 7, -1, b"body".as_ptr(), 4);
        let mut m = link.carry().expect("a message arrived");
        assert_eq!(imsg_forward(onward.writer(), m.ptr()), 1);

        let mut fwd = onward.carry().expect("the message was forwarded");
        assert_eq!(imsg_get_type(fwd.ptr()), 5);
        assert_eq!(imsg_get_id(fwd.ptr()), 6);
        assert_eq!(imsg_get_pid(fwd.ptr()), 7);
        assert_eq!(fwd.data(), b"body");
    }
}

#[test]
fn an_empty_message_is_forwarded_too() {
    let mut link = Link::new();
    let mut onward = Link::new();
    unsafe {
        imsg_compose(link.writer(), 5, 0, 0, -1, null(), 0);
        let mut m = link.carry().expect("a message arrived");
        assert_eq!(imsg_forward(onward.writer(), m.ptr()), 1);
        let mut fwd = onward.carry().expect("the message was forwarded");
        assert_eq!(imsg_get_len(fwd.ptr()), 0);
    }
}

#[test]
fn forwarding_past_the_limit_is_refused() {
    let mut link = Link::new();
    unsafe {
        // A message whose payload is bigger than the far end's limit is
        // too big to forward once its own header is put back on.
        assert_eq!(imsgbuf_set_maxsize(link.writer(), 8), 0);
        let mut over = Message(Box::default());
        let mut buf = ibuf_dynamic(0, 64).expect("a buffer to forward");
        ibuf_add_zero(&raw mut *buf, IMSG_HEADER_SIZE as size_t + 32);
        (*over.ptr()).buf = Some(buf);
        (*over.ptr()).hdr.type_0 = 1;
        clear_errno();
        assert_eq!(imsg_forward(link.writer(), over.ptr()), -1);
        assert_eq!(errno(), ERANGE);
    }
}

#[test]
fn a_descriptor_rides_along_with_a_message() {
    let mut link = Link::new().with_fdpass();
    let other = {
        let mut fds: [c_int; 2] = [-1, -1];
        unsafe { socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, fds.as_mut_ptr()) };
        fds
    };
    unsafe {
        assert_eq!(
            imsg_compose(link.writer(), 9, 0, 0, libc::dup(other[0]), null(), 0),
            1
        );
        let mut m = link.carry().expect("a message arrived");
        let got = imsg_get_fd(m.ptr());
        assert!(got >= 0, "no descriptor came with the message");
        assert_eq!(imsg_get_fd(m.ptr()), -1);
        close(got);
        for fd in other {
            close(fd);
        }
    }
}

#[test]
fn the_limit_may_be_raised_but_never_past_what_the_length_field_holds() {
    let mut link = Link::new();
    unsafe {
        assert_eq!(imsgbuf_set_maxsize(link.writer(), 64), 0);
        assert_eq!(link.writer().maxsize, 64 + IMSG_HEADER_SIZE as uint32_t);

        clear_errno();
        assert_eq!(imsgbuf_set_maxsize(link.writer(), UINT32_MAX), -1);
        assert_eq!(errno(), ERANGE);

        // A limit whose top bit is the one the descriptor mark uses.
        clear_errno();
        assert_eq!(imsgbuf_set_maxsize(link.writer(), IMSG_FD_MARK), -1);
        assert_eq!(errno(), EINVAL);
    }
}

#[test]
fn one_message_may_be_given_a_limit_of_its_own() {
    let mut link = Link::new();
    unsafe {
        let mut wbuf = imsg_create(link.writer(), 1, 0, 0, 0).expect("a message to limit");
        assert_eq!(imsg_set_maxsize(&raw mut *wbuf, 32), 0);

        clear_errno();
        assert_eq!(imsg_set_maxsize(&raw mut *wbuf, UINT32_MAX as size_t), -1);
        assert_eq!(errno(), ERANGE);
        ibuf_free(wbuf);
    }
}

#[test]
fn adding_to_a_message_answers_how_much_was_added() {
    let mut link = Link::new();
    unsafe {
        let mut wbuf = imsg_create(link.writer(), 1, 0, 0, 0);
        assert_eq!(imsg_add(&mut wbuf, null(), 0), 0);
        assert_eq!(imsg_add(&mut wbuf, b"four".as_ptr(), 4), 4);

        // A piece past the limit frees the message and answers -1.
        imsg_set_maxsize(
            wbuf.as_deref_mut().map_or(null_mut(), |buf| &raw mut *buf),
            4,
        );
        assert_eq!(imsg_add(&mut wbuf, b"more".as_ptr(), 4), -1);
        assert!(wbuf.is_none());
    }
}

#[test]
fn a_message_read_from_a_queue_is_put_back_on_one() {
    let mut link = Link::new();
    unsafe {
        let mut bufq = ibufq_new();
        assert_eq!(
            imsg_ibufq_pop(&raw mut *bufq, &raw mut *Box::new(imsg::default())),
            0
        );

        imsg_compose(link.writer(), 4, 0, 0, -1, b"queued".as_ptr(), 6);
        let mut m = link.carry().expect("a message arrived");
        imsg_ibufq_push(&raw mut *bufq, m.ptr());
        assert_eq!(ibufq_queuelen(&raw mut *bufq), 1);
        assert!((*m.ptr()).buf.is_none());
        assert_eq!((*m.ptr()).hdr.type_0, 0);

        let mut back = Message(Box::default());
        assert_eq!(imsg_ibufq_pop(&raw mut *bufq, back.ptr()), 1);
        assert_eq!(imsg_get_type(back.ptr()), 4);
        assert_eq!(back.data(), b"queued");

        // A message with nothing in it comes back off the queue pointing
        // at nothing.
        imsg_compose(link.writer(), 5, 0, 0, -1, null(), 0);
        let mut empty = link.carry().expect("a message arrived");
        imsg_ibufq_push(&raw mut *bufq, empty.ptr());
        let mut nothing = Message(Box::default());
        assert_eq!(imsg_ibufq_pop(&raw mut *bufq, nothing.ptr()), 1);
        assert_eq!((*nothing.ptr()).data, null_mut());
        assert!(nothing.data().is_empty());
        ibufq_free(bufq);
    }
}

#[test]
fn a_queued_buffer_too_short_for_a_header_is_refused() {
    unsafe {
        let mut bufq = ibufq_new();
        ibufq_push(&raw mut *bufq, ibuf_open(2).expect("a buffer to queue"));
        let mut m = Box::new(imsg::default());
        assert_eq!(imsg_ibufq_pop(&raw mut *bufq, &raw mut *m), -1);
        ibufq_free(bufq);
    }
}

#[test]
fn a_message_buffer_holding_something_too_short_for_a_header_is_refused() {
    let mut link = Link::new();
    unsafe {
        let w: *mut msgbuf = link
            .reader()
            .w
            .as_deref_mut()
            .map(|w| w as *mut msgbuf)
            .unwrap();
        ibufq_push(
            &raw mut (*w).rbufs,
            ibuf_open(2).expect("a buffer to queue"),
        );
        let mut m = Box::new(imsg::default());
        assert_eq!(imsgbuf_get(link.reader(), &raw mut *m), -1);
    }
}

#[test]
fn a_message_reads_out_as_a_buffer_as_bytes_or_as_a_string() {
    let mut link = Link::new();
    unsafe {
        imsg_compose(link.writer(), 1, 0, 0, -1, b"ab\0cd".as_ptr(), 5);
        let mut m = link.carry().expect("a message arrived");
        let mut out = [0u8; 8];
        assert_eq!(
            imsg_get_strbuf(m.ptr(), out.as_mut_ptr() as *mut c_char, 3),
            0
        );
        assert_eq!(
            CStr::from_ptr(out.as_ptr() as *const c_char).to_bytes(),
            b"ab"
        );
        assert_eq!(imsg_get_buf(m.ptr(), out.as_mut_ptr(), 2), 0);
        assert_eq!(&out[..2], b"cd");
        assert_eq!(imsg_get_len(m.ptr()), 0);

        clear_errno();
        let mut empty = Box::new(ibuf::default());
        assert_eq!(imsg_get_ibuf(m.ptr(), &raw mut *empty), -1);
        assert_eq!(errno(), EBADMSG);
    }
}

#[test]
fn a_message_reads_out_as_a_buffer_over_the_whole_payload() {
    let mut link = Link::new();
    unsafe {
        imsg_compose(link.writer(), 1, 0, 0, -1, b"body".as_ptr(), 4);
        let mut m = link.carry().expect("a message arrived");
        let mut inner = Box::new(ibuf::default());
        assert_eq!(imsg_get_ibuf(m.ptr(), &raw mut *inner), 0);
        assert_eq!(
            ::core::slice::from_raw_parts(ibuf_data(&raw mut *inner) as *const u8, 4),
            b"body"
        );
    }
}

#[test]
fn a_message_payload_is_read_out_whole_or_not_at_all() {
    let mut link = Link::new();
    unsafe {
        imsg_compose(link.writer(), 1, 0, 0, -1, b"body".as_ptr(), 4);
        let mut m = link.carry().expect("a message arrived");
        let mut out = [0u8; 8];

        clear_errno();
        assert_eq!(imsg_get_data(m.ptr(), out.as_mut_ptr(), 0), -1);
        assert_eq!(errno(), EINVAL);

        clear_errno();
        assert_eq!(imsg_get_data(m.ptr(), out.as_mut_ptr(), 3), -1);
        assert_eq!(errno(), EBADMSG);

        assert_eq!(imsg_get_data(m.ptr(), out.as_mut_ptr(), 4), 0);
        assert_eq!(&out[..4], b"body");
    }
}

#[test]
fn a_length_the_header_could_not_hold_stops_the_read() {
    let mut link = Link::new();
    unsafe {
        // A header claiming less than a header's worth of bytes.
        let short: [u32; 4] = [1, 2, 0, 0];
        libc::write(
            link.fds[0],
            short.as_ptr() as *const c_void,
            IMSG_HEADER_SIZE,
        );
        clear_errno();
        assert_eq!(imsgbuf_read(link.reader()), -1);
        assert_eq!(errno(), ERANGE);
    }

    let mut over = Link::new();
    unsafe {
        // And one claiming more than the limit allows.
        let long: [u32; 4] = [1, MAX_IMSGSIZE as u32 + 1, 0, 0];
        libc::write(
            over.fds[0],
            long.as_ptr() as *const c_void,
            IMSG_HEADER_SIZE,
        );
        clear_errno();
        assert_eq!(imsgbuf_read(over.reader()), -1);
        assert_eq!(errno(), ERANGE);
    }
}

#[test]
fn there_is_nothing_to_get_before_anything_has_been_read() {
    let mut link = Link::new();
    unsafe {
        let mut m = Box::new(imsg::default());
        assert_eq!(imsgbuf_get(link.reader(), &raw mut *m), 0);
        assert_eq!(imsg_get(link.reader(), &raw mut *m), 0);
    }
    assert!(link.next().is_none());
}

#[test]
fn the_older_get_answers_the_whole_message_length() {
    let mut link = Link::new();
    unsafe {
        imsg_compose(link.writer(), 1, 0, 0, -1, b"body".as_ptr(), 4);
        assert_eq!(imsgbuf_flush(link.writer()), 0);
        assert_eq!(imsgbuf_read(link.reader()), 1);
        let mut m = Message(Box::default());
        assert_eq!(
            imsg_get(link.reader(), m.ptr()),
            (IMSG_HEADER_SIZE + 4) as ssize_t
        );
    }
}

#[test]
fn writing_to_a_socket_that_has_gone_is_an_error() {
    let mut link = Link::new();
    unsafe {
        imsg_compose(link.writer(), 1, 0, 0, -1, null(), 0);
        link.writer().fd = -1;
        assert_eq!(imsgbuf_write(link.writer()), -1);
        assert_eq!(imsgbuf_flush(link.writer()), -1);
    }
}

#[test]
fn a_message_buffer_that_passes_descriptors_reads_and_writes_the_other_way() {
    let mut link = Link::new().with_fdpass();
    unsafe {
        imsg_compose(link.writer(), 1, 0, 0, -1, b"fd".as_ptr(), 2);
        assert_eq!(imsgbuf_write(link.writer()), 0);
        assert_eq!(imsgbuf_read(link.reader()), 1);
        let mut m = link.next().expect("a message arrived");
        assert_eq!(m.data(), b"fd");
    }
}

#[test]
fn clearing_takes_the_message_buffer_away() {
    let mut link = Link::new();
    unsafe {
        imsg_compose(link.writer(), 1, 0, 0, -1, null(), 0);
        imsgbuf_clear(link.writer());
        assert!(link.writer().w.is_none());
    }
}

#[test]
fn a_message_marks_its_header_with_the_descriptor_it_carries() {
    let mut link = Link::new();
    let pair = {
        let mut fds: [c_int; 2] = [-1, -1];
        unsafe { socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, fds.as_mut_ptr()) };
        fds
    };
    unsafe {
        let mut wbuf = imsg_create(link.writer(), 1, 0, 0, 0).expect("a message to send");
        ibuf_fd_set(&raw mut *wbuf, libc::dup(pair[0]));
        imsg_close(link.writer(), wbuf);
        let mut len: uint32_t = 0;
        let w = link
            .writer()
            .w
            .as_deref_mut()
            .map(|w| w as *mut msgbuf)
            .unwrap();
        let queued = &raw const *(*w).bufs.bufs[0] as *mut ibuf;
        ::core::ptr::copy_nonoverlapping(
            (ibuf_data(queued) as *const u8).add(4),
            &raw mut len as *mut u8,
            4,
        );
        assert_ne!(len & IMSG_FD_MARK, 0);
        for fd in pair {
            close(fd);
        }
    }
}
