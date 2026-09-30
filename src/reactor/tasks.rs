//! Application-owned futures with explicit and drop cancellation.
use super::{ensure_runtime, handle};
use hmux_rt::Handle as _;
use std::future::Future;
use std::io;

/// Replace the owner's pending task. Dropping the runtime task cancels it.
pub fn task_start<F>(
    task: &mut Option<hmux_rt::mio::Task>,
    initialize: impl FnOnce() -> io::Result<F>,
) -> io::Result<()>
where
    F: Future<Output = ()> + 'static,
{
    drop(task.take());
    ensure_runtime();
    *task = Some(handle().spawn(initialize()?)?);
    Ok(())
}

pub fn task_is_pending(task: &Option<hmux_rt::mio::Task>) -> bool {
    task.as_ref()
        .is_some_and(hmux_rt::mio::Handle::task_is_pending)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hmux_rt::AsyncFd as _;
    use std::cell::{Cell, RefCell};
    use std::io::{Read, Write};
    use std::os::fd::AsRawFd;
    use std::os::unix::net::UnixStream;
    use std::rc::Rc;
    use std::time::Duration;

    fn poll() {
        super::super::poll_runtime_with_timeout(Some(Duration::ZERO));
    }

    #[test]
    fn tasks_can_spawn_work_owned_by_the_caller() {
        let calls = Rc::new(Cell::new(0));
        let observed = calls.clone();
        let child = Rc::new(RefCell::new(None::<hmux_rt::mio::Task>));
        let retained = child.clone();
        let mut task = None::<hmux_rt::mio::Task>;
        crate::src::reactor::task_start(&mut task, move || {
            Ok(async move {
                crate::src::reactor::task_start(&mut retained.borrow_mut(), move || {
                    Ok(async move {
                        observed.set(1);
                    })
                })
                .unwrap();
            })
        })
        .unwrap();
        assert_eq!(calls.get(), 0);
        poll();
        assert_eq!(calls.get(), 1);
        assert!(!crate::src::reactor::task_is_pending(&task));
        assert!(!crate::src::reactor::task_is_pending(&child.borrow()));
        super::super::shutdown_runtime();
    }

    #[test]
    fn moving_and_cancelling_a_wait_preserves_unread_input() {
        let (mut reader, mut writer) = UnixStream::pair().unwrap();
        reader.set_nonblocking(true).unwrap();
        let fd = reader.as_raw_fd();
        let calls = Rc::new(Cell::new(0));
        let observed = calls.clone();
        let mut task = None::<hmux_rt::mio::Task>;
        crate::src::reactor::task_start(&mut task, move || {
            let source = super::super::descriptor(fd)?;
            let observed = observed.clone();
            Ok(async move {
                source.ready(true, false).await.unwrap();
                observed.set(observed.get() + 1);
            })
        })
        .unwrap();
        poll();
        let mut moved = task;
        assert!(crate::src::reactor::task_is_pending(&moved));
        drop(moved.take());
        drop(moved.take());
        writer.write_all(b"unread").unwrap();
        poll();
        assert_eq!(calls.get(), 0);
        assert_eq!(
            Rc::strong_count(&calls),
            1,
            "cancel releases the future capture"
        );

        let observed = calls.clone();
        crate::src::reactor::task_start(&mut moved, move || {
            let source = super::super::descriptor(fd)?;
            let observed = observed.clone();
            Ok(async move {
                source.ready(true, false).await.unwrap();
                observed.set(observed.get() + 1);
            })
        })
        .unwrap();
        poll();
        assert_eq!(calls.get(), 1);
        assert!(!crate::src::reactor::task_is_pending(&moved));
        let mut bytes = [0; 6];
        reader.read_exact(&mut bytes).unwrap();
        assert_eq!(&bytes, b"unread");
        assert_eq!(Rc::strong_count(&calls), 1);
        super::super::shutdown_runtime();
    }

    #[test]
    fn dispatch_can_replace_and_cancel_its_running_task() {
        let slot = Rc::new(RefCell::new(None::<hmux_rt::mio::Task>));
        let observer = Rc::downgrade(&slot);
        let calls = Rc::new(Cell::new(0));
        let observed = calls.clone();
        crate::src::reactor::task_start(&mut slot.borrow_mut(), move || {
            let observer = observer.clone();
            let observed = observed.clone();
            Ok(async move {
                let slot = observer.upgrade().unwrap();
                observed.set(1);
                let replacement = observed.clone();
                crate::src::reactor::task_start(&mut slot.borrow_mut(), move || {
                    let replacement = replacement.clone();
                    Ok(async move {
                        replacement.set(2);
                    })
                })
                .unwrap();
                super::super::yield_now().await;
                panic!("cancelled dispatch must not resume");
            })
        })
        .unwrap();
        poll();
        assert_eq!(calls.get(), 2);
        assert!(!crate::src::reactor::task_is_pending(&slot.borrow()));
        assert_eq!(Rc::strong_count(&slot), 1);
        super::super::shutdown_runtime();
    }

    #[test]
    fn reused_descriptor_numbers_do_not_share_an_executing_tasks_old_lease() {
        use std::os::fd::{FromRawFd, IntoRawFd, OwnedFd};
        let (old_reader, mut old_writer) = UnixStream::pair().unwrap();
        let (new_reader, mut new_writer) = UnixStream::pair().unwrap();
        old_reader.set_nonblocking(true).unwrap();
        new_reader.set_nonblocking(true).unwrap();
        let fd = old_reader.into_raw_fd();
        super::super::init_runtime();
        // An executing dispatch retains its descriptor lease until it returns.
        let old_source = super::super::descriptor(fd).unwrap();
        super::super::forget_descriptor(fd);
        assert_eq!(unsafe { libc::close(fd) }, 0);
        assert_eq!(unsafe { libc::dup2(new_reader.as_raw_fd(), fd) }, fd);
        let endpoint = unsafe { OwnedFd::from_raw_fd(fd) };
        let calls = Rc::new(Cell::new(0));
        let observed = calls.clone();
        let mut task = None::<hmux_rt::mio::Task>;
        crate::src::reactor::task_start(&mut task, move || {
            let source = super::super::descriptor(fd)?;
            let observed = observed.clone();
            Ok(async move {
                source.ready(true, false).await.unwrap();
                let mut byte = 0_u8;
                assert_eq!(
                    unsafe { libc::read(fd, (&mut byte as *mut u8).cast(), 1) },
                    1
                );
                assert_eq!(byte, b'n');
                observed.set(observed.get() + 1);
            })
        })
        .unwrap();
        poll();
        old_writer.write_all(b"o").unwrap();
        poll();
        assert_eq!(calls.get(), 0);
        new_writer.write_all(b"n").unwrap();
        poll();
        assert_eq!(calls.get(), 1);
        assert!(!crate::src::reactor::task_is_pending(&task));
        drop(old_source);
        drop(endpoint);
        super::super::shutdown_runtime();
    }

    #[test]
    fn dropping_a_handle_releases_unpolled_and_parked_work() {
        let retained = Rc::new(());
        for before_poll in [true, false] {
            let observed = retained.clone();
            let mut task = None::<hmux_rt::mio::Task>;
            crate::src::reactor::task_start(&mut task, move || {
                Ok(async move {
                    let _retained = observed;
                    std::future::pending::<()>().await;
                })
            })
            .unwrap();
            if !before_poll {
                poll();
            }
            assert_eq!(Rc::strong_count(&retained), 2);
            drop(task);
            assert_eq!(Rc::strong_count(&retained), 1);
            poll();
            super::super::shutdown_runtime();
        }

        let mut task = None::<hmux_rt::mio::Task>;
        let result =
            crate::src::reactor::task_start(&mut task, || -> io::Result<std::future::Ready<()>> {
                Err(io::ErrorKind::InvalidInput.into())
            });
        assert!(result.is_err());
        assert!(!crate::src::reactor::task_is_pending(&task));
        super::super::shutdown_runtime();
    }

    #[test]
    fn shutdown_invalidates_old_handles_without_cancelling_new_tasks() {
        let mut old = None::<hmux_rt::mio::Task>;
        crate::src::reactor::task_start(&mut old, || Ok(std::future::pending())).unwrap();
        assert!(crate::src::reactor::task_is_pending(&old));
        super::super::shutdown_runtime();
        assert!(!crate::src::reactor::task_is_pending(&old));

        let calls = Rc::new(Cell::new(0));
        let observed = calls.clone();
        let mut task = None::<hmux_rt::mio::Task>;
        crate::src::reactor::task_start(&mut task, move || {
            Ok(async move {
                observed.set(observed.get() + 1);
            })
        })
        .unwrap();
        drop(old.take());
        drop(old);
        assert!(crate::src::reactor::task_is_pending(&task));
        poll();
        assert_eq!(calls.get(), 1);
        assert!(!crate::src::reactor::task_is_pending(&task));
        super::super::shutdown_runtime();
    }

    #[test]
    fn shutdown_during_dispatch_is_rejected_before_cleanup() {
        let rejected = Rc::new(Cell::new(false));
        let observed = rejected.clone();
        let mut first = None::<hmux_rt::mio::Task>;
        crate::src::reactor::task_start(&mut first, move || {
            Ok(async move {
                let result = std::panic::catch_unwind(super::super::shutdown_runtime);
                observed.set(result.is_err());
                assert!(super::super::runtime_initialized());
            })
        })
        .unwrap();
        let calls = Rc::new(Cell::new(0));
        let observed = calls.clone();
        let mut second = None::<hmux_rt::mio::Task>;
        crate::src::reactor::task_start(&mut second, move || Ok(async move { observed.set(1) }))
            .unwrap();
        poll();
        assert!(rejected.get());
        assert_eq!(calls.get(), 1);
        super::super::shutdown_runtime();
        assert!(!super::super::runtime_initialized());
    }

    #[test]
    fn initializer_transfers_its_only_owner_into_the_future() {
        let owner = refbox::RefBox::new(());
        let observer = owner.downgrade();
        let mut task = None::<hmux_rt::mio::Task>;
        crate::src::reactor::task_start(&mut task, move || {
            Ok(async move {
                let _owner = owner;
                std::future::pending::<()>().await;
            })
        })
        .unwrap();
        poll();
        assert!(observer.is_alive());
        drop(task.take());
        assert!(!observer.is_alive());
        super::super::shutdown_runtime();
    }
}
