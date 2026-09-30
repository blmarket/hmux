//! Explicitly cancelled application futures, recreated after a fork.
use super::{ensure_runtime, handle};
use hmux_rt::Handle as _;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::future::Future;
use std::io;
use std::num::NonZeroU64;
use std::pin::Pin;
use std::rc::Rc;

type Factory = Box<dyn Fn() -> io::Result<Pin<Box<dyn Future<Output = ()>>>>>;

/// A movable cancellation handle. Owners cancel it in their explicit cleanup.
/// Dropping this handle leaves the registered future running.
#[derive(Default)]
pub struct Task {
    id: Option<NonZeroU64>,
}

struct Registration {
    factory: Factory,
    task: RefCell<Option<hmux_rt::mio::Task>>,
}

thread_local! {
    static NEXT_ID: Cell<u64> = const { Cell::new(0) };
    static TASKS: RefCell<HashMap<NonZeroU64, Rc<Registration>>> = RefCell::new(HashMap::new());
}

/// Queue a callback without running it inline or creating a timer.
/// Shutdown or a scheduling failure releases captures without dispatching it.
pub fn defer(callback: impl FnOnce() + 'static) {
    // Retain the callback in the factory until dispatch, so replacing the
    // runtime after fork can recreate an unpolled future without losing it.
    let pending = Rc::new(RefCell::new(Some(callback)));
    let _ = Task::new().start(move || {
        let pending = pending.clone();
        Ok(async move {
            let callback = pending.borrow_mut().take().expect("one deferred dispatch");
            callback();
        })
    });
}

impl Task {
    pub const fn new() -> Self {
        Self { id: None }
    }

    /// Replace pending work. The factory creates fresh runtime resources on
    /// initial registration and after fork; it must not dispatch work inline.
    pub fn start<F>(&mut self, factory: impl Fn() -> io::Result<F> + 'static) -> io::Result<()>
    where
        F: Future<Output = ()> + 'static,
    {
        self.cancel();
        ensure_runtime();
        let id = NEXT_ID.with(|next| {
            let id = next.get().checked_add(1).expect("task IDs exhausted");
            next.set(id);
            NonZeroU64::new(id).unwrap()
        });
        let registration = Rc::new(Registration {
            factory: Box::new(move || Ok(Box::pin(factory()?))),
            task: RefCell::new(None),
        });
        TASKS.with(|tasks| tasks.borrow_mut().insert(id, registration.clone()));
        if let Err(error) = start(id, &registration) {
            remove(id);
            return Err(error);
        }
        self.id = Some(id);
        Ok(())
    }

    pub fn cancel(&mut self) {
        if let Some(id) = self.id.take() {
            remove(id);
        }
    }

    pub fn is_pending(&self) -> bool {
        self.id
            .is_some_and(|id| TASKS.with(|tasks| tasks.borrow().contains_key(&id)))
    }
}

fn remove(id: NonZeroU64) {
    let registration = TASKS.with(|tasks| tasks.borrow_mut().remove(&id));
    if let Some(registration) = registration {
        let task = registration.task.borrow_mut().take();
        drop(task);
    }
}

fn start(id: NonZeroU64, registration: &Rc<Registration>) -> io::Result<()> {
    let future = (registration.factory)()?;
    let task = handle().spawn(async move {
        future.await;
        remove(id);
    })?;
    *registration.task.borrow_mut() = Some(task);
    Ok(())
}

pub(super) fn stop_tasks() {
    let tasks = TASKS.with(|tasks| tasks.borrow().values().cloned().collect::<Vec<_>>());
    for registration in tasks {
        let task = registration.task.borrow_mut().take();
        drop(task);
    }
}

pub(super) fn restart() -> io::Result<()> {
    let tasks = TASKS.with(|tasks| {
        tasks
            .borrow()
            .iter()
            .map(|(&id, task)| (id, task.clone()))
            .collect::<Vec<_>>()
    });
    for (id, registration) in tasks {
        start(id, &registration)?;
    }
    Ok(())
}

pub(super) fn clear() {
    loop {
        let tasks = TASKS.with(|tasks| {
            tasks
                .borrow_mut()
                .drain()
                .map(|(_, task)| task)
                .collect::<Vec<_>>()
        });
        if tasks.is_empty() {
            break;
        }
        for registration in tasks {
            let task = registration.task.borrow_mut().take();
            drop(task);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hmux_rt::Runtime as _;
    use std::io::{Read, Write};
    use std::os::fd::AsRawFd;
    use std::os::unix::net::UnixStream;
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
        assert!(TASKS.with(|tasks| tasks.borrow().is_empty()));
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
            "cancel releases the factory capture"
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
    fn dropping_a_handle_keeps_work_pending_but_shutdown_releases_it() {
        let retained = Rc::new(());
        let observed = retained.clone();
        let mut task = Task::new();
        task.start(move || {
            let observed = observed.clone();
            Ok(async move {
                let _retained = observed;
                std::future::pending::<()>().await;
            })
        })
        .unwrap();
        drop(task);
        poll();
        assert_eq!(Rc::strong_count(&retained), 3);
        super::super::shutdown_runtime();
        assert_eq!(Rc::strong_count(&retained), 1);

        let mut task = Task::new();
        let result = task.start(|| -> io::Result<std::future::Ready<()>> {
            Err(io::ErrorKind::InvalidInput.into())
        });
        assert!(result.is_err());
        assert!(!task.is_pending());
        assert!(TASKS.with(|tasks| tasks.borrow().is_empty()));
        super::super::shutdown_runtime();
    }

    #[test]
    fn pending_descriptor_futures_restart_in_the_child_and_leave_the_parent_live() {
        let (reader, mut writer) = UnixStream::pair().unwrap();
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
                let mut byte = 0_u8;
                assert_eq!(
                    unsafe { libc::read(fd, (&mut byte as *mut u8).cast(), 1) },
                    1
                );
                observed.set(observed.get() + 1);
            })
        })
        .unwrap();
        poll();
        let pid = unsafe { libc::fork() };
        assert!(pid >= 0);
        if pid == 0 {
            unsafe { libc::alarm(5) };
            super::super::reset_after_fork().unwrap();
            assert!(task.is_pending());
            writer.write_all(b"c").unwrap();
            poll();
            let succeeded = calls.get() == 1 && !task.is_pending();
            super::super::shutdown_runtime();
            unsafe { libc::_exit(if succeeded { 0 } else { 1 }) };
        }
        let mut status = 0;
        assert_eq!(unsafe { libc::waitpid(pid, &mut status, 0) }, pid);
        assert!(libc::WIFEXITED(status));
        assert_eq!(libc::WEXITSTATUS(status), 0);
        assert!(task.is_pending());
        writer.write_all(b"p").unwrap();
        poll();
        assert_eq!(calls.get(), 1);
        assert!(!task.is_pending());
        super::super::shutdown_runtime();
    }
}
