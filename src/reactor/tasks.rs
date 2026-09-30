//! Application-owned futures with explicit and drop cancellation.
use super::{ensure_runtime, handle};
use hmux_rt::Handle as _;
use std::future::Future;
use std::io;

/// Owns one future. Explicit cleanup and dropping the owner both cancel it.
#[must_use = "dropping the task cancels its future"]
#[derive(Default)]
pub struct Task {
    task: Option<hmux_rt::mio::Task>,
}

/// Queue a callback without running it inline or creating a timer.
/// Shutdown or a scheduling failure releases captures without dispatching it.
pub fn defer(callback: impl FnOnce() + 'static) {
    ensure_runtime();
    let _ = handle().defer(callback);
}

impl Task {
    pub const fn new() -> Self {
        Self { task: None }
    }

    /// Replace pending work. Initialize resources once, without dispatching
    /// work inline; the returned future owns those resources.
    pub fn start<F>(&mut self, initialize: impl FnOnce() -> io::Result<F>) -> io::Result<()>
    where
        F: Future<Output = ()> + 'static,
    {
        self.cancel();
        ensure_runtime();
        let future = initialize()?;
        self.task = Some(handle().spawn(future)?);
        Ok(())
    }

    pub fn cancel(&mut self) {
        drop(self.task.take());
    }

    pub fn is_pending(&self) -> bool {
        self.task.as_ref().is_some_and(|task| task.is_pending())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hmux_rt::Runtime as _;
    use std::cell::{Cell, RefCell};
    use std::io::{Read, Write};
    use std::os::fd::AsRawFd;
    use std::os::unix::net::UnixStream;
    use std::rc::Rc;
    use std::time::Duration;

    fn poll() {
        super::super::HOST.with(|host| {
            host.borrow_mut()
                .as_mut()
                .unwrap()
                .poll(Some(Duration::ZERO))
                .unwrap();
        });
    }

    #[test]
    fn deferred_callbacks_queue_without_inline_dispatch_and_release_captures() {
        let order = Rc::new(RefCell::new(Vec::new()));
        let observed = order.clone();
        defer(move || {
            observed.borrow_mut().push(1);
            let nested = observed.clone();
            defer(move || nested.borrow_mut().push(3));
            observed.borrow_mut().push(2);
        });
        assert!(order.borrow().is_empty());
        poll();
        assert_eq!(*order.borrow(), [1, 2, 3]);
        assert_eq!(Rc::strong_count(&order), 1);
        super::super::shutdown_runtime();
    }

    #[test]
    fn moving_and_cancelling_a_wait_preserves_unread_input() {
        let (mut reader, mut writer) = UnixStream::pair().unwrap();
        reader.set_nonblocking(true).unwrap();
        let fd = reader.as_raw_fd();
        let calls = Rc::new(Cell::new(0));
        let observed = calls.clone();
        let mut task = Task::new();
        task.start(move || {
            let source = super::super::descriptor(fd)?;
            let observed = observed.clone();
            Ok(async move {
                source.wait(true, false).await.unwrap();
                observed.set(observed.get() + 1);
            })
        })
        .unwrap();
        poll();
        let mut moved = task;
        assert!(moved.is_pending());
        moved.cancel();
        moved.cancel();
        writer.write_all(b"unread").unwrap();
        poll();
        assert_eq!(calls.get(), 0);
        assert_eq!(
            Rc::strong_count(&calls),
            1,
            "cancel releases the future capture"
        );

        let observed = calls.clone();
        moved
            .start(move || {
                let source = super::super::descriptor(fd)?;
                let observed = observed.clone();
                Ok(async move {
                    source.wait(true, false).await.unwrap();
                    observed.set(observed.get() + 1);
                })
            })
            .unwrap();
        poll();
        assert_eq!(calls.get(), 1);
        assert!(!moved.is_pending());
        let mut bytes = [0; 6];
        reader.read_exact(&mut bytes).unwrap();
        assert_eq!(&bytes, b"unread");
        assert_eq!(Rc::strong_count(&calls), 1);
        super::super::shutdown_runtime();
    }

    #[test]
    fn dispatch_can_replace_and_cancel_its_running_task() {
        let slot = Rc::new(RefCell::new(Task::new()));
        let observer = Rc::downgrade(&slot);
        let calls = Rc::new(Cell::new(0));
        let observed = calls.clone();
        slot.borrow_mut()
            .start(move || {
                let observer = observer.clone();
                let observed = observed.clone();
                Ok(async move {
                    let slot = observer.upgrade().unwrap();
                    observed.set(1);
                    let replacement = observed.clone();
                    slot.borrow_mut()
                        .start(move || {
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
        assert!(!slot.borrow().is_pending());
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
        let mut task = Task::new();
        task.start(move || {
            let source = super::super::descriptor(fd)?;
            let observed = observed.clone();
            Ok(async move {
                source.wait(true, false).await.unwrap();
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
        assert!(!task.is_pending());
        drop(old_source);
        drop(endpoint);
        super::super::shutdown_runtime();
    }

    #[test]
    fn dropping_a_handle_releases_unpolled_and_parked_work() {
        let retained = Rc::new(());
        for before_poll in [true, false] {
            let observed = retained.clone();
            let mut task = Task::new();
            task.start(move || {
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

        let mut task = Task::new();
        let result = task.start(|| -> io::Result<std::future::Ready<()>> {
            Err(io::ErrorKind::InvalidInput.into())
        });
        assert!(result.is_err());
        assert!(!task.is_pending());
        super::super::shutdown_runtime();
    }

    #[test]
    fn shutdown_invalidates_old_handles_without_cancelling_new_tasks() {
        let mut old = Task::new();
        old.start(|| Ok(std::future::pending())).unwrap();
        assert!(old.is_pending());
        super::super::shutdown_runtime();
        assert!(!old.is_pending());

        let calls = Rc::new(Cell::new(0));
        let observed = calls.clone();
        let mut task = Task::new();
        task.start(move || {
            Ok(async move {
                observed.set(observed.get() + 1);
            })
        })
        .unwrap();
        old.cancel();
        drop(old);
        assert!(task.is_pending());
        poll();
        assert_eq!(calls.get(), 1);
        assert!(!task.is_pending());
        super::super::shutdown_runtime();
    }

    #[test]
    fn initializer_transfers_its_only_owner_into_the_future() {
        let owner = refbox::RefBox::new(());
        let observer = owner.downgrade();
        let mut task = Task::new();
        task.start(move || {
            Ok(async move {
                let _owner = owner;
                std::future::pending::<()>().await;
            })
        })
        .unwrap();
        poll();
        assert!(observer.is_alive());
        task.cancel();
        assert!(!observer.is_alive());
        super::super::shutdown_runtime();
    }

    #[test]
    fn shutdown_releases_deferred_work_queued_by_capture_destructors() {
        struct DeferOnDrop(Option<refbox::RefBox<()>>);
        impl Drop for DeferOnDrop {
            fn drop(&mut self) {
                let owner = self.0.take().unwrap();
                defer(move || {
                    drop(owner);
                    panic!("shutdown must not dispatch deferred work");
                });
            }
        }

        let owner = refbox::RefBox::new(());
        let observer = owner.downgrade();
        let guard = DeferOnDrop(Some(owner));
        defer(move || drop(guard));
        super::super::shutdown_runtime();
        assert!(!observer.is_alive());
    }
}
