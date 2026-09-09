//! Coverage for [`crate::compat`] — pure buffer helpers.
//!
//! `imsg_buffer.rs` is the low-level buffer and queue layer beneath `imsg.rs`.
//! This suite stays clear of sockets and the message-reader machinery:
//! every test operates on memory alone through the `ibuf` / `ibufqueue` /
//! `msgbuf` helpers.

use crate::compat::{
    ibuf_add, ibuf_add_ibuf, ibuf_add_n8, ibuf_add_n16, ibuf_add_n32, ibuf_add_n64,
    ibuf_add_strbuf, ibuf_data, ibuf_dynamic, ibuf_free, ibuf_from_buffer, ibuf_from_ibuf,
    ibuf_get, ibuf_get_ibuf, ibuf_get_n16, ibuf_get_n32, ibuf_get_n64, ibuf_get_strbuf, ibuf_left,
    ibuf_open, ibuf_rewind, ibuf_seek, ibuf_set, ibuf_set_h32, ibuf_size, ibuf_truncate,
    ibufq_flush, ibufq_new, ibufq_pop, ibufq_push, ibufq_queuelen, msgbuf_clear, msgbuf_new,
    msgbuf_queuelen,
};
use crate::ffi::__errno_location;
use ::bytes::{Buf as _, BufMut as _};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn errno() -> i32 {
    unsafe { *__errno_location() }
}
fn clear_errno() {
    unsafe { *__errno_location() = 0 };
}

unsafe fn bytes_of(buf: *mut crate::types::ibuf) -> Vec<u8> {
    unsafe { ibuf_data(&*buf).to_vec() }
}

// ---------------------------------------------------------------------------
// ibuf add / get roundtrip
// ---------------------------------------------------------------------------

#[test]
fn ibuf_add_and_get_mixed_widths_roundtrip() {
    unsafe {
        let mut buf_box = ibuf_dynamic(0, 64).expect("a buffer");
        let buf = &raw mut *buf_box;
        assert_eq!(ibuf_add_n8(&mut *buf, 0xAB), 0);
        assert_eq!(ibuf_add_n16(&mut *buf, 0x1234), 0);
        assert_eq!(ibuf_add_n32(&mut *buf, 0x0A0B0C0D), 0);
        assert_eq!(ibuf_add_n64(&mut *buf, 0x0102030405060708), 0);
        assert_eq!(ibuf_size(&*buf), 1 + 2 + 4 + 8);

        let mut n8: u8 = 0;
        // ibuf_get_n* reads in network order where applicable
        // n8 is raw byte, n16/n32/n64 are big-endian on wire then swapped back
        assert_eq!(ibuf_get(&mut *buf, core::slice::from_mut(&mut n8)), 0);
        assert_eq!(n8, 0xAB);
        assert_eq!(ibuf_get_n16(&mut *buf), Some(0x1234));
        assert_eq!(ibuf_get_n32(&mut *buf), Some(0x0A0B0C0D));
        assert_eq!(ibuf_get_n64(&mut *buf), Some(0x0102030405060708));
        assert_eq!(ibuf_size(&*buf), 0);
        ibuf_free(buf_box);
    }
}

#[test]
fn ibuf_set_overwrites_without_moving_write_pos() {
    unsafe {
        let mut buf_box = ibuf_dynamic(0, 32).expect("a buffer");
        let buf = &raw mut *buf_box;
        (*buf).put_bytes(0, 8);
        assert_eq!(ibuf_set_h32(&mut *buf, 0, 0xDEADBEEF), 0);
        assert_eq!(ibuf_set_h32(&mut *buf, 4, 0xCAFEBABE), 0);
        // write position stays at 8
        assert_eq!(ibuf_size(&*buf), 8);
        let bytes = bytes_of(buf);
        assert_eq!(&bytes[0..4], &0xDEADBEEFu32.to_ne_bytes());
        assert_eq!(&bytes[4..8], &0xCAFEBABEu32.to_ne_bytes());

        // generic ibuf_set at offset 2
        let patch = [0xFFu8, 0xFF];
        assert_eq!(ibuf_set(&mut *buf, 2, &patch), 0);
        let patched = bytes_of(buf);
        assert_eq!(patched[2], 0xFF);
        assert_eq!(patched[3], 0xFF);
        ibuf_free(buf_box);
    }
}

#[test]
fn ibuf_seek_finds_slice_and_rejects_out_of_bounds() {
    unsafe {
        let mut buf_box = ibuf_dynamic(0, 16).expect("a buffer");
        let buf = &raw mut *buf_box;
        ibuf_add(&mut *buf, b"hello world");
        let p = ibuf_seek(&mut *buf, 6, 5).unwrap();
        assert_eq!(p, b"world");

        clear_errno();
        assert!(ibuf_seek(&mut *buf, 12, 1).is_none());
        assert_eq!(errno(), 34); // ERANGE

        clear_errno();
        assert!(ibuf_seek(&mut *buf, 0, 12).is_none());
        assert_eq!(errno(), 34);

        (*buf).advance(6);
        assert_eq!(bytes_of(buf), b"world");
        ibuf_rewind(&mut *buf);
        assert_eq!(bytes_of(buf), b"hello world");
        ibuf_free(buf_box);
    }
}

// ---------------------------------------------------------------------------
// truncate / left / from_buffer
// ---------------------------------------------------------------------------

#[test]
fn ibuf_left_and_truncate_grow_and_shrink() {
    unsafe {
        let mut buf_box = ibuf_dynamic(0, 16).expect("a buffer");
        let buf = &raw mut *buf_box;
        assert_eq!(ibuf_left(&*buf), 16);
        ibuf_add(&mut *buf, b"abcd");
        assert_eq!(ibuf_left(&*buf), 12);
        assert_eq!(ibuf_size(&*buf), 4);

        // shrink
        assert_eq!(ibuf_truncate(&mut *buf, 2), 0);
        assert_eq!(bytes_of(buf), b"ab");
        assert_eq!(ibuf_left(&*buf), 14);

        // grow pads with zeroes
        assert_eq!(ibuf_truncate(&mut *buf, 5), 0);
        assert_eq!(bytes_of(buf), b"ab\0\0\0");

        // past max is refused
        clear_errno();
        assert_eq!(ibuf_truncate(&mut *buf, 17), -1);
        assert_eq!(errno(), 34); // ERANGE
        ibuf_free(buf_box);
    }
}

#[test]
fn ibuf_from_buffer_and_from_ibuf_copy_their_input() {
    unsafe {
        let mut outer_box = ibuf_dynamic(0, 16).expect("a buffer");
        let outer = &raw mut *outer_box;
        ibuf_add(&mut *outer, b"abcdef");
        (*outer).advance(2);
        assert_eq!(bytes_of(outer), b"cdef");

        let mut stack = Box::new(crate::types::ibuf::default());
        ibuf_from_ibuf(&mut stack, &*outer);
        assert!(stack.borrowed);
        assert_eq!(ibuf_size(&stack), 4);
        assert_eq!(ibuf_data(&stack), b"cdef");

        // ibuf_get_ibuf also creates a stack view over a slice
        let mut slice = Box::new(crate::types::ibuf::default());
        *slice = ibuf_get_ibuf(&mut *outer, 2).expect("a buffer");
        assert!(slice.borrowed);
        assert_eq!(ibuf_data(&slice), b"cd");
        assert_eq!(bytes_of(outer), b"ef");

        // copying raw bytes directly
        let raw = [0xAAu8, 0xBB, 0xCC];
        let mut wrapped = Box::new(crate::types::ibuf::default());
        ibuf_from_buffer(&mut wrapped, &raw);
        assert!(wrapped.borrowed);
        assert_eq!(ibuf_size(&wrapped), 3);
        ibuf_free(outer_box);
    }
}

#[test]
fn ibuf_add_strbuf_pads_and_reports_overflow() {
    unsafe {
        let mut buf_box = ibuf_dynamic(0, 32).expect("a buffer");
        let buf = &raw mut *buf_box;
        // "hi" into 6 bytes -> "hi\0\0\0\0"
        assert_eq!(ibuf_add_strbuf(&mut *buf, c"hi", 6), 0);
        assert_eq!(bytes_of(buf), b"hi\0\0\0\0");

        // string too long for field -> EOVERFLOW and still consumed space
        clear_errno();
        assert_eq!(ibuf_add_strbuf(&mut *buf, c"toolong", 4), -1);
        assert_eq!(errno(), 75); // EOVERFLOW
        ibuf_free(buf_box);

        // ibuf_get_strbuf requires NUL terminator at end
        let mut buf2_box = ibuf_dynamic(0, 32).expect("a buffer");
        let buf2 = &raw mut *buf2_box;
        ibuf_add(&mut *buf2, b"ab\0\0");
        let mut out = [0u8; 4];
        assert_eq!(ibuf_get_strbuf(&mut *buf2, &mut out[..3]), 0);
        assert_eq!(&out[..3], b"ab\0");

        // missing terminator -> EOVERFLOW and last byte forced to NUL
        ibuf_add(&mut *buf2, b"xy");
        clear_errno();
        assert_eq!(ibuf_get_strbuf(&mut *buf2, &mut out[..2]), -1);
        assert_eq!(errno(), 75);
        assert_eq!(out[1], 0);
        ibuf_free(buf2_box);
    }
}

// ---------------------------------------------------------------------------
// queue and msgbuf helpers (no sockets)
// ---------------------------------------------------------------------------

#[test]
fn ibufq_concat_and_flush_preserve_order() {
    unsafe {
        let mut a = ibufq_new();
        let mut b = ibufq_new();
        assert_eq!(ibufq_queuelen(&a), 0);

        for ch in *b"xyz" {
            let mut ib = ibuf_open(1).expect("a buffer");
            ibuf_add(&mut ib, &[ch]);
            ibufq_push(&mut a, ib);
        }
        assert_eq!(ibufq_queuelen(&a), 3);

        let mut ib = ibuf_open(1).expect("a buffer");
        ibuf_add(&mut ib, b"w");
        ibufq_push(&mut b, ib);
        a.bufs.append(&mut b.bufs);
        assert_eq!(ibufq_queuelen(&a), 4);
        assert_eq!(ibufq_queuelen(&b), 0);

        let mut order = Vec::new();
        while ibufq_queuelen(&a) > 0 {
            let p = ibufq_pop(&mut a).expect("a queued buffer");
            order.extend(bytes_of(&raw const *p as *mut crate::types::ibuf));
            ibuf_free(p);
        }
        assert_eq!(order, b"xyzw");
        assert!(ibufq_pop(&mut a).is_none());

        // flush empties even when not popped one by one
        for ch in *b"ab" {
            let mut ib = ibuf_open(1).expect("a buffer");
            ibuf_add(&mut ib, &[ch]);
            ibufq_push(&mut a, ib);
        }
        ibufq_flush(&mut a);
        assert_eq!(ibufq_queuelen(&a), 0);

        // leak check: free the empty queues
        crate::compat::ibufq_free(a);
        crate::compat::ibufq_free(b);
    }
}

#[test]
fn msgbuf_queues_transfer_ownership_and_clear() {
    unsafe {
        let mut m1 = msgbuf_new();
        let mut m2 = msgbuf_new();
        let m1 = &raw mut *m1;
        let m2 = &raw mut *m2;
        assert_eq!(msgbuf_queuelen(&*m1), 0);

        let mut ib1_box = ibuf_dynamic(0, 16).expect("a buffer");
        let ib1 = &raw mut *ib1_box;
        ibuf_add(&mut *ib1, b"one");
        crate::compat::ibuf_close(&mut *m1, ib1_box);
        assert_eq!(msgbuf_queuelen(&*m1), 1);

        let mut ib2_box = ibuf_dynamic(0, 16).expect("a buffer");
        let ib2 = &raw mut *ib2_box;
        ibuf_add(&mut *ib2, b"two");
        crate::compat::ibuf_close(&mut *m2, ib2_box);
        assert_eq!(msgbuf_queuelen(&*m2), 1);

        (*m1).bufs.bufs.append(&mut (*m2).bufs.bufs);
        assert_eq!(msgbuf_queuelen(&*m1), 2);

        // ibuf_get_ibuf copy between buffers
        let mut src_box = ibuf_dynamic(0, 16).expect("a buffer");
        let src = &raw mut *src_box;
        ibuf_add(&mut *src, b"hello");
        let mut dst_box = ibuf_dynamic(0, 16).expect("a buffer");
        let dst = &raw mut *dst_box;
        assert_eq!(ibuf_add_ibuf(&mut *dst, &*src), 0);
        assert_eq!(bytes_of(dst), b"hello");
        ibuf_free(src_box);
        ibuf_free(dst_box);

        // ibuf_open with zero length has no storage
        let empty = ibuf_open(0).expect("a buffer with no storage");
        assert!(empty.buf.capacity() == 0 || empty.size == 0);
        ibuf_free(empty);

        msgbuf_clear(&mut *m1);
        assert_eq!(msgbuf_queuelen(&*m1), 0);
        msgbuf_clear(&mut *m2);
    }
}
