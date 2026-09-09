use super::*;
use crate::ImsgDataBuffer;
use crate::ImsgMessage;
use crate::compat::imsg_buffer::{
    ibuf_add_ibuf, ibuf_data, ibuf_get_ibuf, ibuf_get_strbuf, ibuf_rewind, ibufq_pop, ibufq_push,
};
use crate::compat::{ibufq_free, ibufq_new, ibufq_queuelen, msgbuf};
use crate::ffi::{close, socketpair};
use crate::tests::test_fixtures::zeroed;
use ::bytes::BufMut as _;
use ::core::ffi::{CStr, c_char, c_int, c_void};
use std::io::IoSlice;

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
        match unsafe { imsgbuf_get(self.reader()) } {
            Ok(Some(m)) => Some(Message(Box::new(m))),
            Ok(None) => None,
            Err(()) => panic!("imsgbuf_get failed"),
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
    /// The message's payload, header aside.
    fn data(&self) -> Vec<u8> {
        self.0.imsg_message_data().to_vec()
    }
}

impl Drop for Message {
    fn drop(&mut self) {
        unsafe { imsg_free(core::mem::take(&mut *self.0)) };
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
        assert_eq!(imsg_compose(link.writer(), 7, 99, 1234, -1, b"payload"), 1);
        assert_eq!(imsgbuf_queuelen(link.writer()), 1);

        let mut m = link.carry().expect("a message arrived");
        assert_eq!(imsg_get_type(&m.0), 7);
        assert_eq!(imsg_get_id(&m.0), 99);
        assert_eq!(imsg_get_pid(&m.0), 1234);
        assert_eq!(imsg_get_len(&m.0), 7);
        assert_eq!(m.data(), b"payload");
        assert_eq!(imsg_get_fd(&mut m.0), -1);
    }
}

#[test]
fn an_initialized_reader_can_move_away_from_its_original_owner_address() {
    for fdpass in [false, true] {
        let mut link = if fdpass {
            Link::new().with_fdpass()
        } else {
            Link::new()
        };
        let mut moved = std::mem::replace(
            link.reader(),
            imsgbuf {
                w: None,
                pid: 0,
                maxsize: 0,
                fd: -1,
                flags: 0,
            },
        );
        unsafe {
            assert_eq!(imsg_compose(link.writer(), 7, 99, 1234, -1, b"payload"), 1);
            assert_eq!(imsgbuf_flush(link.writer()), 0);
            assert_eq!(imsgbuf_read(&mut moved), 1);
            let message = imsgbuf_get(&mut moved).unwrap().unwrap();
            assert_eq!(message.imsg_message_data(), b"payload");
        }
    }
}

#[test]
fn each_read_uses_the_current_message_size_limit() {
    for fdpass in [false, true] {
        let mut link = if fdpass {
            Link::new().with_fdpass()
        } else {
            Link::new()
        };
        unsafe {
            assert_eq!(imsgbuf_set_maxsize(link.reader(), 3), 0);
            assert_eq!(imsg_compose(link.writer(), 7, 0, 0, -1, b"abc"), 1);
            assert_eq!(link.carry().unwrap().data(), b"abc");
            assert_eq!(imsgbuf_set_maxsize(link.reader(), 1), 0);
            assert_eq!(imsg_compose(link.writer(), 7, 0, 0, -1, b"xy"), 1);
            assert_eq!(imsgbuf_flush(link.writer()), 0);
            clear_errno();
            assert_eq!(imsgbuf_read(link.reader()), -1);
            assert_eq!(errno(), ERANGE);
        }
    }
}

#[test]
fn a_message_composed_without_a_pid_carries_this_process_id() {
    let mut link = Link::new();
    unsafe {
        assert_eq!(imsg_compose(link.writer(), 1, 0, 0, -1, &[]), 1);
        let mut m = link.carry().expect("a message arrived");
        assert_eq!(imsg_get_pid(&m.0), getpid() as pid_t);
        assert_eq!(imsg_get_len(&m.0), 0);
        assert!(m.0.imsg_message_data().is_empty());
    }
}

#[test]
fn a_message_bigger_than_the_limit_is_refused() {
    let mut link = Link::new();
    let big = vec![b'x'; MAX_IMSGSIZE as usize];
    unsafe {
        clear_errno();
        assert_eq!(imsg_compose(link.writer(), 1, 0, 0, -1, &big), -1);
        assert_eq!(errno(), ERANGE);
        assert_eq!(imsgbuf_queuelen(link.writer()), 0);
    }
}

#[test]
fn a_message_may_be_composed_from_several_pieces() {
    let mut link = Link::new();
    unsafe {
        let iov = [
            IoSlice::new(b"one"),
            IoSlice::new(b""),
            IoSlice::new(b"two"),
        ];
        assert_eq!(imsg_composev(link.writer(), 2, 0, 0, -1, &iov), 1);
        let mut m = link.carry().expect("a message arrived");
        assert_eq!(m.data(), b"onetwo");

        // No pieces at all is an empty message.
        assert_eq!(imsg_composev(link.writer(), 2, 0, 0, -1, &[]), 1);
        let mut empty = link.carry().expect("a message arrived");
        assert_eq!(imsg_get_len(&empty.0), 0);
    }
}

#[test]
fn pieces_that_do_not_fit_together_are_refused() {
    let mut link = Link::new();
    let big = vec![b'x'; MAX_IMSGSIZE as usize];
    unsafe {
        let iov = [IoSlice::new(&big)];
        assert_eq!(imsg_composev(link.writer(), 2, 0, 0, -1, &iov), -1);
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
        let iov = [IoSlice::new(&first), IoSlice::new(b"more")];
        // Together they are past the limit, so the create is what refuses.
        assert_eq!(imsg_composev(link.writer(), 2, 0, 0, -1, &iov), -1);

        // Sized for one piece but handed two, the add is what refuses.
        let mut wbuf = imsg_create(link.writer(), 2, 0, 0, 4).expect("a message to fill");
        wbuf.set_imsg_data_buffer_max_size(IMSG_HEADER_SIZE + 4);
        assert_eq!(ibuf_add(&mut *wbuf, b"toolong"), -1);
        ibuf_free(wbuf);
    }
}

#[test]
fn a_message_may_be_composed_from_a_buffer_that_is_handed_over() {
    let mut link = Link::new();
    unsafe {
        let mut buf = ibuf_dynamic(0, 64).expect("a buffer to hand over");
        ibuf_add(&mut *buf, b"handed");
        assert_eq!(imsg_compose_ibuf(link.writer(), 3, 11, 22, buf), 1);
        assert_eq!(imsgbuf_queuelen(link.writer()), 2);

        let mut m = link.carry().expect("a message arrived");
        assert_eq!(imsg_get_type(&m.0), 3);
        assert_eq!(imsg_get_id(&m.0), 11);
        assert_eq!(imsg_get_pid(&m.0), 22);
        assert_eq!(m.data(), b"handed");
    }
}

#[test]
fn a_handed_over_buffer_too_big_for_the_limit_is_refused_and_freed() {
    let mut link = Link::new();
    unsafe {
        let mut buf = ibuf_dynamic(0, (MAX_IMSGSIZE + 1) as size_t).expect("a buffer to hand over");
        buf.put_bytes(0, MAX_IMSGSIZE as usize);
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
        assert_eq!(imsg_get_pid(&m.0), getpid() as pid_t);
    }
}

#[test]
fn a_message_is_forwarded_with_its_header_and_payload() {
    let mut link = Link::new();
    let mut onward = Link::new();
    unsafe {
        imsg_compose(link.writer(), 5, 6, 7, -1, b"body");
        let mut m = link.carry().expect("a message arrived");
        assert_eq!(imsg_forward(onward.writer(), &mut *m.0), 1);

        let mut fwd = onward.carry().expect("the message was forwarded");
        assert_eq!(imsg_get_type(&fwd.0), 5);
        assert_eq!(imsg_get_id(&fwd.0), 6);
        assert_eq!(imsg_get_pid(&fwd.0), 7);
        assert_eq!(fwd.data(), b"body");
    }
}

#[test]
fn an_empty_message_is_forwarded_too() {
    let mut link = Link::new();
    let mut onward = Link::new();
    unsafe {
        imsg_compose(link.writer(), 5, 0, 0, -1, &[]);
        let mut m = link.carry().expect("a message arrived");
        assert_eq!(imsg_forward(onward.writer(), &mut *m.0), 1);
        let mut fwd = onward.carry().expect("the message was forwarded");
        assert_eq!(imsg_get_len(&fwd.0), 0);
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
        buf.put_bytes(0, IMSG_HEADER_SIZE + 32);
        over.0.buf = Some(buf);
        over.0.hdr.type_0 = 1;
        clear_errno();
        assert_eq!(imsg_forward(link.writer(), &mut *over.0), -1);
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
            imsg_compose(link.writer(), 9, 0, 0, libc::dup(other[0]), &[]),
            1
        );
        let mut m = link.carry().expect("a message arrived");
        let got = imsg_get_fd(&mut m.0);
        assert!(got >= 0, "no descriptor came with the message");
        assert_eq!(imsg_get_fd(&mut m.0), -1);
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
        wbuf.set_imsg_data_buffer_max_size(IMSG_HEADER_SIZE + 32);
        assert_eq!(ibuf_add(&mut wbuf, &[b'x'; 32]), 0);
        assert_eq!(ibuf_add(&mut wbuf, b"x"), -1);
        assert_eq!(ibuf_size(&wbuf), IMSG_HEADER_SIZE + 32);
        ibuf_free(wbuf);
    }
}

#[test]
fn adding_to_a_message_answers_how_much_was_added() {
    let mut link = Link::new();
    unsafe {
        let mut wbuf = imsg_create(link.writer(), 1, 0, 0, 0);
        assert_eq!(imsg_add(&mut wbuf, &[]), 0);
        assert_eq!(imsg_add(&mut wbuf, b"four"), 4);

        // A piece past the limit frees the message and answers -1.
        wbuf.as_deref_mut()
            .expect("message to limit")
            .set_imsg_data_buffer_max_size(IMSG_HEADER_SIZE + 4);
        assert_eq!(imsg_add(&mut wbuf, b"more"), -1);
        assert!(wbuf.is_none());
    }
}

#[test]
fn a_message_read_from_a_queue_is_put_back_on_one() {
    let mut link = Link::new();
    unsafe {
        let mut bufq = ibufq_new();
        assert!(matches!(imsg_ibufq_pop(&mut bufq), Ok(None)));

        imsg_compose(link.writer(), 4, 0, 0, -1, b"queued");
        let mut m = link.carry().expect("a message arrived");
        imsg_ibufq_push(&mut *bufq, core::mem::take(&mut *m.0));
        assert_eq!(ibufq_queuelen(&bufq), 1);
        assert!(m.0.buf.is_none());
        assert_eq!(m.0.hdr.type_0, 0);

        let mut back = Message(Box::new(
            imsg_ibufq_pop(&mut bufq)
                .expect("queued message should parse")
                .expect("queued message should be present"),
        ));
        assert_eq!(imsg_get_type(&back.0), 4);
        assert_eq!(back.data(), b"queued");

        // A message with nothing in it comes back off the queue pointing
        // at nothing.
        imsg_compose(link.writer(), 5, 0, 0, -1, &[]);
        let mut empty = link.carry().expect("a message arrived");
        imsg_ibufq_push(&mut *bufq, core::mem::take(&mut *empty.0));
        let mut nothing = Message(Box::new(
            imsg_ibufq_pop(&mut bufq)
                .expect("empty message should parse")
                .expect("empty message should be present"),
        ));
        assert!(nothing.0.imsg_message_data().is_empty());
        assert!(nothing.data().is_empty());
        ibufq_free(bufq);
    }
}

#[test]
fn a_queued_buffer_too_short_for_a_header_is_refused() {
    unsafe {
        let mut bufq = ibufq_new();
        ibufq_push(&mut bufq, ibuf_open(2).expect("a buffer to queue"));
        assert!(imsg_ibufq_pop(&mut bufq).is_err());
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
        ibufq_push(&mut (*w).rbufs, ibuf_open(2).expect("a buffer to queue"));
        assert!(imsgbuf_get(link.reader()).is_err());
    }
}

#[test]
fn a_message_reads_out_as_a_buffer_as_bytes_or_as_a_string() {
    let mut link = Link::new();
    unsafe {
        imsg_compose(link.writer(), 1, 0, 0, -1, b"ab\0cd");
        let mut m = link.carry().expect("a message arrived");
        let mut out = [0u8; 8];
        assert_eq!(imsg_get_strbuf(&mut *m.0, &mut out[..3]), 0);
        assert_eq!(
            CStr::from_ptr(out.as_ptr() as *const c_char).to_bytes(),
            b"ab"
        );
        assert_eq!(imsg_get_buf(&mut *m.0, &mut out[..2]), 0);
        assert_eq!(&out[..2], b"cd");
        assert_eq!(imsg_get_len(&m.0), 0);

        clear_errno();
        assert!(imsg_get_ibuf(&mut *m.0).is_none());
        assert_eq!(errno(), EBADMSG);
    }
}

#[test]
fn a_message_reads_out_as_a_buffer_over_the_whole_payload() {
    let mut link = Link::new();
    unsafe {
        imsg_compose(link.writer(), 1, 0, 0, -1, b"body");
        let mut m = link.carry().expect("a message arrived");
        let inner = imsg_get_ibuf(&mut *m.0).expect("a buffer");
        assert_eq!(ibuf_data(&inner), b"body");
    }
}

#[test]
fn a_message_payload_is_read_out_whole_or_not_at_all() {
    let mut link = Link::new();
    unsafe {
        imsg_compose(link.writer(), 1, 0, 0, -1, b"body");
        let mut m = link.carry().expect("a message arrived");
        let mut out = [0u8; 8];

        clear_errno();
        assert_eq!(imsg_get_data(&mut *m.0, &mut []), -1);
        assert_eq!(errno(), EINVAL);

        clear_errno();
        assert_eq!(imsg_get_data(&mut *m.0, &mut out[..3]), -1);
        assert_eq!(errno(), EBADMSG);

        assert_eq!(imsg_get_data(&mut *m.0, &mut out[..4]), 0);
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
        assert!(matches!(imsgbuf_get(link.reader()), Ok(None)));
        assert!(matches!(imsg_get(link.reader()), Ok(None)));
    }
    assert!(link.next().is_none());
}

#[test]
fn the_older_get_answers_the_whole_message_length() {
    let mut link = Link::new();
    unsafe {
        imsg_compose(link.writer(), 1, 0, 0, -1, b"body");
        assert_eq!(imsgbuf_flush(link.writer()), 0);
        assert_eq!(imsgbuf_read(link.reader()), 1);
        let (m, len) = imsg_get(link.reader()).unwrap().unwrap();
        assert_eq!(len, (IMSG_HEADER_SIZE + 4) as size_t);
        let mut m = Message(Box::new(m));
        assert_eq!(imsg_get_type(&m.0), 1);
    }
}

#[test]
fn writing_to_a_socket_that_has_gone_is_an_error() {
    let mut link = Link::new();
    unsafe {
        imsg_compose(link.writer(), 1, 0, 0, -1, &[]);
        link.writer().fd = -1;
        assert_eq!(imsgbuf_write(link.writer()), -1);
        assert_eq!(imsgbuf_flush(link.writer()), -1);
    }
}

#[test]
fn a_message_buffer_that_passes_descriptors_reads_and_writes_the_other_way() {
    let mut link = Link::new().with_fdpass();
    unsafe {
        imsg_compose(link.writer(), 1, 0, 0, -1, b"fd");
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
        imsg_compose(link.writer(), 1, 0, 0, -1, &[]);
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
        ibuf_fd_set(&mut *wbuf, libc::dup(pair[0]));
        imsg_close(link.writer(), wbuf);
        let mut len: uint32_t = 0;
        let w = link
            .writer()
            .w
            .as_deref_mut()
            .map(|w| w as *mut msgbuf)
            .unwrap();
        let queued = &raw const *(*w).bufs.bufs[0] as *mut ibuf;
        core::ptr::copy_nonoverlapping(
            ibuf_data(&*queued).as_ptr().add(4),
            &raw mut len as *mut u8,
            4,
        );
        assert_ne!(len & IMSG_FD_MARK, 0);
        for fd in pair {
            close(fd);
        }
    }
}

pub(crate) unsafe fn imsgbuf_set_maxsize(imsgbuf: &mut imsgbuf, max: uint32_t) -> c_int {
    unsafe {
        if max as usize > (UINT32_MAX as usize).wrapping_sub(IMSG_HEADER_SIZE) {
            return imsg_fail(ERANGE, -(1 as c_int));
        }
        let max = (max as core::ffi::c_ulong).wrapping_add(IMSG_HEADER_SIZE as core::ffi::c_ulong)
            as uint32_t;
        if max & IMSG_FD_MARK as uint32_t != 0 {
            return imsg_fail(EINVAL, -(1 as c_int));
        }
        imsgbuf.maxsize = max;
        0 as c_int
    }
}

pub(crate) unsafe fn imsg_ibufq_pop(bufq: &mut ibufqueue) -> Result<Option<imsg>, ()> {
    unsafe {
        let Some(buf) = ibufq_pop(bufq) else {
            return Ok(None);
        };
        let Some(m) = imsg_from_ibuf(buf) else {
            return Err(());
        };
        Ok(Some(m))
    }
}

pub(crate) unsafe fn imsg_ibufq_push(bufq: &mut ibufqueue, mut imsg: imsg) {
    unsafe {
        if let Some(mut buf) = imsg.buf.take() {
            ibuf_rewind(&mut *buf);
            ibufq_push(bufq, buf);
        }
    }
}

pub(crate) unsafe fn imsg_get_ibuf(imsg: &mut imsg) -> Option<ibuf> {
    unsafe {
        let size = ibuf_size(&*imsg_buf(imsg));
        if size == 0 as size_t {
            return imsg_fail(EBADMSG, None);
        }
        ibuf_get_ibuf(imsg_buf(imsg), size)
    }
}

pub(crate) fn imsg_get_data(imsg: &mut imsg, data: &mut [u8]) -> c_int {
    unsafe {
        if data.is_empty() {
            return imsg_fail(EINVAL, -(1 as c_int));
        }
        if ibuf_size(&*imsg_buf(imsg)) != data.len() as size_t {
            return imsg_fail(EBADMSG, -(1 as c_int));
        }
        ibuf_get(imsg_buf(imsg), data)
    }
}

pub(crate) fn imsg_get_buf(imsg: &mut imsg, data: &mut [u8]) -> c_int {
    unsafe { ibuf_get(imsg_buf(imsg), data) }
}

pub(crate) fn imsg_get_strbuf(imsg: &mut imsg, str: &mut [u8]) -> c_int {
    unsafe { ibuf_get_strbuf(imsg_buf(imsg), str) }
}

pub(crate) unsafe fn imsg_get_id(imsg: &imsg) -> uint32_t {
    imsg.hdr.peerid
}

pub(crate) unsafe fn imsg_get_pid(imsg: &imsg) -> pid_t {
    imsg.hdr.pid as pid_t
}

pub(crate) unsafe fn imsg_composev(
    imsgbuf: &mut imsgbuf,
    type_0: uint32_t,
    id: uint32_t,
    pid: pid_t,
    fd: c_int,
    iov: &[std::io::IoSlice<'_>],
) -> c_int {
    unsafe {
        let datalen = iov
            .iter()
            .fold(0 as size_t, |sum, piece| sum.wrapping_add(piece.len()));
        let Some(mut wbuf) = imsg_create(imsgbuf, type_0, id, pid, datalen) else {
            return -(1 as c_int);
        };
        if iov
            .iter()
            .all(|piece| ibuf_add(&mut wbuf, piece) != -(1 as c_int))
        {
            ibuf_fd_set(&mut wbuf, fd);
            imsg_close(imsgbuf, wbuf);
            return 1 as c_int;
        }
        ibuf_free(wbuf);
        -(1 as c_int)
    }
}

pub(crate) unsafe fn imsg_compose_ibuf(
    imsgbuf: &mut imsgbuf,
    type_0: uint32_t,
    id: uint32_t,
    pid: pid_t,
    mut buf: Box<ibuf>,
) -> c_int {
    unsafe {
        let mut hdrbuf: Option<Box<ibuf>> = None;
        let len = ibuf_size(&*buf).wrapping_add(IMSG_HEADER_SIZE);
        if len > imsgbuf.maxsize as size_t {
            *__errno_location() = ERANGE;
        } else {
            let hdr = imsg_make_hdr(imsgbuf, type_0, id, pid, len as uint32_t);
            hdrbuf = ibuf_open(IMSG_HEADER_SIZE);
            if let Some(mut hdrbuf_box) = hdrbuf.take() {
                if imsg_add_hdr(&mut *hdrbuf_box, &hdr) != -(1 as c_int) {
                    ibuf_close(imsgbuf_msgbuf(imsgbuf), hdrbuf_box);
                    ibuf_close(imsgbuf_msgbuf(imsgbuf), buf);
                    return 1 as c_int;
                }
                hdrbuf = Some(hdrbuf_box);
            }
        }
        ibuf_free(buf);
        if let Some(hdrbuf) = hdrbuf {
            ibuf_free(hdrbuf);
        }
        -(1 as c_int)
    }
}

pub(crate) unsafe fn imsg_forward(imsgbuf: &mut imsgbuf, msg: &mut imsg) -> c_int {
    unsafe {
        ibuf_rewind(imsg_buf(msg));
        ibuf_get(imsg_buf(msg), &mut [0; size_of::<imsg_hdr>()]);
        let len = ibuf_size(&*imsg_buf(msg));
        let wbuf = imsg_create(
            imsgbuf,
            msg.hdr.type_0,
            msg.hdr.peerid,
            msg.hdr.pid as pid_t,
            len,
        );
        let Some(mut wbuf) = wbuf else {
            return -(1 as c_int);
        };
        if len != 0 as size_t && ibuf_add_ibuf(&mut *wbuf, &*imsg_buf(msg)) == -(1 as c_int) {
            ibuf_free(wbuf);
            return -(1 as c_int);
        }
        imsg_close(imsgbuf, wbuf);
        1 as c_int
    }
}

/// Adds a piece to the message. A piece that does not fit takes the message
/// with it: `msg` is left empty and the answer is -1.
pub(crate) unsafe fn imsg_add(msg: &mut Option<Box<ibuf>>, data: &[u8]) -> c_int {
    unsafe {
        let Some(buf) = msg.as_deref_mut() else {
            return -(1 as c_int);
        };
        if !data.is_empty() && ibuf_add(&mut *buf, data) == -(1 as c_int) {
            ibuf_free(msg.take().expect("the message just looked at"));
            return -(1 as c_int);
        }
        data.len() as c_int
    }
}
