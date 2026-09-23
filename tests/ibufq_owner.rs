use hmux2::src::compat::imsg_buffer::{
    ibuf_free, ibuf_open, ibufq_concat, ibufq_flush, ibufq_free, ibufq_new, ibufq_pop, ibufq_push,
    ibufq_queuelen, ibufqueue,
};
use std::ptr::null_mut;

unsafe fn assert_empty(queue: *mut ibufqueue) {
    assert_eq!(ibufq_queuelen(queue), 0);
    assert!((*queue).bufs.tqh_first.is_null());
    assert_eq!((*queue).bufs.tqh_last, &raw mut (*queue).bufs.tqh_first);
}

#[test]
fn standalone_ibufq_owns_linked_buffers_and_keeps_tail_at_its_stable_address() {
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
        assert_eq!((*destination).bufs.tqh_first, first);
        assert_eq!(
            (*destination).bufs.tqh_last,
            &raw mut (*second).entry.tqe_next
        );

        assert_eq!(ibufq_pop(destination), first);
        ibuf_free(first);
        assert_eq!(ibufq_queuelen(destination), 1);
        assert_eq!((*destination).bufs.tqh_first, second);
        assert_eq!(
            (*second).entry.tqe_prev,
            &raw mut (*destination).bufs.tqh_first
        );

        ibufq_concat(destination, source);
        assert_empty(source);
        assert_eq!(ibufq_queuelen(destination), 2);
        assert_eq!((*second).entry.tqe_next, third);
        assert_eq!((*third).entry.tqe_prev, &raw mut (*second).entry.tqe_next);
        assert_eq!(
            (*destination).bufs.tqh_last,
            &raw mut (*third).entry.tqe_next
        );

        assert_eq!(ibufq_pop(destination), second);
        ibuf_free(second);
        assert_eq!((*destination).bufs.tqh_first, third);
        assert_eq!(
            (*third).entry.tqe_prev,
            &raw mut (*destination).bufs.tqh_first
        );

        // The destination destructor flushes its remaining linked buffer.
        ibufq_free(destination);

        let fourth = ibuf_open(1);
        assert!(!fourth.is_null());
        ibufq_push(source, fourth);
        ibufq_flush(source);
        assert_empty(source);
        ibufq_free(source);
    }
}
