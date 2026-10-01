use super::*;
use crate::src::cmd::queue::{cmdq_append, cmdq_continue, cmdq_get_callback_owned, cmdq_next};
use crate::src::reactor::{poll_runtime, shutdown_runtime};
use crate::src::shared::command::{CMDQ_WAITING, CMD_RETURN_NORMAL, CMD_RETURN_WAIT};
use std::cell::{Cell, RefCell};

/// Run a real local read from a queue item, retaining only observers afterward.
unsafe fn start_read(
    path: &'static CStr,
    callback: impl FnOnce(Weak<UnsafeCell<cmdq_item>>) -> client_file_cb + 'static,
    cancel: Option<Box<dyn FnOnce()>>,
) -> (Weak<UnsafeCell<cmdq_item>>, Weak<UnsafeCell<client_file>>) {
    let file = Rc::new(Cell::new(Weak::new()));
    let observed = file.clone();
    let item = cmdq_get_callback_owned(
        c"local read",
        Some(Box::new(move |item| {
            let callback = callback(Rc::downgrade(item));
            file_read_with_cmdq_wait_init(
                None,
                path.as_ptr(),
                move |file| {
                    observed.set(file);
                    callback
                },
                item,
                cancel,
            );
            CMD_RETURN_WAIT
        })),
    );
    let item = cmdq_append(None, item);
    assert_eq!(cmdq_next(None), 0, "local completion must be deferred");
    assert_ne!((*item.upgrade().unwrap().get()).flags & CMDQ_WAITING, 0);
    (item, file.take())
}

#[test]
fn local_read_success_and_error_are_deferred_and_preserve_queue_order() {
    unsafe {
        for (path, error) in [(c"/dev/null", 0), (c"/dev/null/missing", libc::ENOTDIR)] {
            let order = Rc::new(RefCell::new(Vec::new()));
            let observed = order.clone();
            let (item, file) = start_read(
                path,
                move |item| {
                    Some(Box::new(move |event| {
                        assert!(event.closed);
                        assert_eq!(event.path, Some(path));
                        assert_eq!(event.error, error);
                        assert_eq!(evbuffer_get_length(event.buffer.unwrap()), 0);
                        observed.borrow_mut().push("completion");
                        cmdq_continue(&item.upgrade().unwrap());
                    }))
                },
                None,
            );
            let observed = order.clone();
            cmdq_append(
                None,
                cmdq_get_callback_owned(
                    c"after read",
                    Some(Box::new(move |_| {
                        observed.borrow_mut().push("following command");
                        CMD_RETURN_NORMAL
                    })),
                ),
            );
            assert_eq!(cmdq_next(None), 0);
            assert!(order.borrow().is_empty());
            assert!(
                file.upgrade().is_some(),
                "unindexed local file must survive"
            );

            poll_runtime();
            assert_eq!(*order.borrow(), ["completion"]);
            assert!(file.upgrade().is_none());
            // Only newly fired commands contribute to cmdq_next's count.
            assert_eq!(cmdq_next(None), 1);
            assert!(
                item.upgrade().is_none(),
                "queue removal must release the item"
            );
            poll_runtime();
            assert_eq!(*order.borrow(), ["completion", "following command"]);
            shutdown_runtime();
        }
    }
}

#[test]
fn cancelling_before_dispatch_releases_callback_data_and_allows_item_removal() {
    unsafe {
        let data = Rc::new(());
        let retained = data.clone();
        let cancelled = Rc::new(Cell::new(0));
        let observed = cancelled.clone();
        let (item, file) = start_read(
            c"/dev/null",
            move |_| {
                Some(Box::new(move |_| {
                    let _ = &retained;
                    panic!("cancelled read callback must not run");
                }))
            },
            Some(Box::new(move || observed.set(observed.get() + 1))),
        );
        assert_eq!(Rc::strong_count(&data), 2);
        let owner = file.upgrade().unwrap();
        file_cancel_cmdq_wait(&owner);
        file_cancel_cmdq_wait(&owner);
        drop(owner);
        assert_eq!(Rc::strong_count(&data), 1);
        assert_eq!(cancelled.get(), 1);
        cmdq_continue(&item.upgrade().unwrap());
        assert_eq!(cmdq_next(None), 0);
        assert!(item.upgrade().is_none());
        assert!(
            file.upgrade().is_none(),
            "cancellation releases local completion"
        );

        poll_runtime();
        assert!(file.upgrade().is_none());
        assert_eq!(cancelled.get(), 1);
        shutdown_runtime();
    }
}

#[test]
fn local_completion_can_be_cancelled_after_runtime_shutdown() {
    unsafe {
        let data = Rc::new(());
        let retained = data.clone();
        let cancelled = Rc::new(Cell::new(0));
        let observed = cancelled.clone();
        let (item, file) = start_read(
            c"/dev/null",
            move |_| {
                Some(Box::new(move |_| {
                    let _ = &retained;
                    panic!("shutdown must not deliver completion");
                }))
            },
            Some(Box::new(move || observed.set(observed.get() + 1))),
        );
        shutdown_runtime();
        // Runtime shutdown does not own command cleanup. The command's file
        // wait remains cancellable after its runtime stops.
        let owner = file.upgrade().unwrap();
        file_cancel_cmdq_wait(&owner);
        drop(owner);
        assert!(file.upgrade().is_none());
        assert_eq!(Rc::strong_count(&data), 1);
        assert_eq!(cancelled.get(), 1);
        cmdq_continue(&item.upgrade().unwrap());
        assert_eq!(cmdq_next(None), 0);
        assert!(item.upgrade().is_none());
    }
}

#[test]
fn completion_unlinks_indexed_files_once_even_with_a_retained_lookup() {
    unsafe {
        for cancel in [false, true] {
            let calls = Rc::new(Cell::new(0));
            let observed = calls.clone();
            let file = file_create_with_peer(
                std::ptr::null_mut(),
                42,
                Some(Box::new(move |event| {
                    assert!(event.closed);
                    observed.set(observed.get() + 1);
                })),
            );
            let observer = Rc::downgrade(&file);
            file_fire_done(&file);
            file_fire_done(&file);
            drop(file);
            let lookup = crate::src::client::client_find_file(42).unwrap();
            assert_eq!(calls.get(), 0);
            if cancel {
                // Removing the index owner and releasing the last lookup drops
                // the file's task before it can deliver the callback.
                client_files_remove(&mut *lookup.get());
            } else {
                poll_runtime();
                poll_runtime();
            }
            assert_eq!(calls.get(), usize::from(!cancel));
            assert!(crate::src::client::client_find_file(42).is_none());
            drop(lookup);
            poll_runtime();
            assert_eq!(calls.get(), usize::from(!cancel));
            assert!(observer.upgrade().is_none());
            assert_eq!(Rc::strong_count(&calls), 1);
            shutdown_runtime();
        }
    }
}
