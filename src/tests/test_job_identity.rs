use super::*;
use std::cell::Cell;
use std::rc::Rc;

fn inert_job(id: u_int) -> Box<job> {
    Box::new(job {
        id,
        state: JOB_RUNNING,
        flags: 0,
        cmd: None,
        pid: -1,
        tty: [0; 32],
        status: 0,
        fd: -1,
        event: Stream::NONE,
        updatecb: None,
        completecb: None,
    })
}

#[test]
fn freeing_a_job_removes_its_identity_before_dropping_callback_data() {
    let _guard = crate::tests::test_fixtures::globals();
    struct Released(Rc<Cell<u32>>);
    impl Drop for Released {
        fn drop(&mut self) {
            assert_eq!(job_event_by_id(37), None);
            assert_eq!(job_still_running(), 0);
            self.0.set(self.0.get() + 1);
        }
    }
    let released = Rc::new(Cell::new(0));
    let data = Released(released.clone());
    let mut job = inert_job(37);
    job.completecb = Some(Box::new(move |_| drop(data)));
    all_jobs.queue().push_front(job);
    assert_eq!(job_event_by_id(37), Some(Stream::NONE));
    unsafe { job_free(37) };
    unsafe { job_free(37) };
    assert_eq!(released.get(), 1);
}

#[test]
fn update_callbacks_can_remove_their_job_and_stale_stream_events_are_ignored() {
    let _guard = crate::tests::test_fixtures::globals();
    let calls = Rc::new(Cell::new(0));
    let observed = calls.clone();
    let mut job = inert_job(38);
    job.status = 123;
    job.updatecb = Some(Rc::new(move |event| {
        assert_eq!(job_event_by_id(event.id), Some(event.event));
        unsafe { job_free(event.id) };
        assert_eq!(event.status, 123);
        observed.set(observed.get() + 1);
    }));
    all_jobs.queue().push_front(job);
    let read = on_job(38, job_read_callback);
    read(Stream::NONE);
    read(Stream::NONE);
    on_job(38, job_write_callback)(Stream::NONE);
    on_job_error(38, job_error_callback)(Stream::NONE, 0);
    assert_eq!(calls.get(), 1);
    assert_eq!(job_event_by_id(38), None);
}

#[test]
fn completion_callbacks_can_remove_jobs_for_either_event_order() {
    let _guard = crate::tests::test_fixtures::globals();
    for eof_first in [false, true] {
        let calls = Rc::new(Cell::new(0));
        let observed = calls.clone();
        let mut job = inert_job(39);
        job.completecb = Some(Box::new(move |event| {
            assert_eq!(event.status, 7 << 8);
            assert_eq!(job_event_by_id(event.id), Some(event.event));
            unsafe { job_free(event.id) };
            observed.set(observed.get() + 1);
        }));
        all_jobs.queue().push_front(job);
        if eof_first {
            job_error_callback(39);
            assert_eq!(calls.get(), 0);
            job_check_died(-1, 7 << 8);
        } else {
            job_check_died(-1, 7 << 8);
            assert_eq!(calls.get(), 0);
            job_error_callback(39);
        }
        job_error_callback(39);
        job_check_died(-1, 7 << 8);
        assert_eq!(calls.get(), 1);
        assert_eq!(job_event_by_id(39), None);
    }
}

#[test]
fn transferred_tty_names_fit_and_terminate_in_the_callers_buffer() {
    let _guard = crate::tests::test_fixtures::globals();
    for (capacity, expected) in [
        (0, &b""[..]),
        (1, &b"\0"[..]),
        (4, &b"/de\0"[..]),
        (16, &b"/dev/pts/42\0xxxx"[..]),
    ] {
        let mut tty = [0; 32];
        for (target, source) in tty.iter_mut().zip(b"/dev/pts/42") {
            *target = *source;
        }
        let mut job = inert_job(0);
        job.tty = tty;
        let mut output = vec![b'x'; capacity];
        all_jobs.queue().push_front(job);
        assert_eq!(
            { job_transfer(0, Some(&mut output)) },
            Some((-1, -1))
        );
        assert_eq!(job_event_by_id(0), None);
        assert_eq!(output, expected);
    }
}

#[test]
fn job_id_exhaustion_returns_before_startup_and_releases_callback_data() {
    std::thread::spawn(|| {
        struct Released(Rc<Cell<u32>>);
        impl Drop for Released {
            fn drop(&mut self) {
                self.0.set(self.0.get() + 1);
                unsafe { *crate::ffi::__errno_location() = libc::EINVAL };
            }
        }
        NEXT_JOB_ID.set(Some(u_int::MAX));
        assert_eq!(
            crate::entity_id::try_next_entity_id(&NEXT_JOB_ID),
            Some(u_int::MAX)
        );
        let released = Rc::new(Cell::new(0));
        for attempt in 1..=2 {
            let data = Released(released.clone());
            let result = unsafe {
                job_run(
                    Some(c"true"),
                    &[],
                    None,
                    None,
                    None,
                    None,
                    Some(Box::new(move |_| drop(data))),
                    0,
                    80,
                    24,
                )
            };
            assert!(result.is_none());
            assert_eq!(unsafe { *crate::ffi::__errno_location() }, libc::EOVERFLOW);
            assert_eq!(released.get(), attempt);
            assert_eq!(NEXT_JOB_ID.get(), None);
        }
        std::thread::spawn(|| assert_eq!(NEXT_JOB_ID.get(), Some(0)))
            .join()
            .unwrap();
    })
    .join()
    .unwrap();
}
