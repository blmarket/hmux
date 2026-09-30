//! Explicitly cancelled callback timers over hmux-rt's monotonic waits.
use super::{ensure_runtime, handle};
use hmux_rt::Handle as _;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::io;
use std::num::NonZeroU64;
use std::rc::Rc;
use std::time::{Duration, Instant};

type Callback = Rc<RefCell<Box<dyn FnMut()>>>;

/// A movable timer handle. Owners must cancel it in their explicit cleanup path.
/// Dropping the handle does not cancel a registration.
#[derive(Default)]
pub struct Timer {
    id: Option<NonZeroU64>,
    pub(crate) callback: Option<Callback>,
}

struct Registration {
    deadline: Instant,
    callback: Callback,
    task: RefCell<Option<hmux_rt::mio::Task>>,
}

thread_local! {
    static NEXT_ID: Cell<u64> = const { Cell::new(0) };
    static TIMERS: RefCell<HashMap<NonZeroU64, Rc<Registration>>> = RefCell::new(HashMap::new());
}

impl Timer {
    pub const fn new() -> Self {
        Self {
            id: None,
            callback: None,
        }
    }

    pub fn set(&mut self, callback: impl FnMut() + 'static) {
        self.cancel();
        self.callback = Some(Rc::new(RefCell::new(Box::new(callback))));
    }

    pub fn is_initialized(&self) -> bool {
        self.callback.is_some() || self.id.is_some()
    }

    /// Replace any pending wait. Zero duration defers to a runtime turn.
    pub fn arm(&mut self, delay: Duration) -> io::Result<()> {
        self.cancel();
        let callback = self
            .callback
            .clone()
            .expect("timer initialized before arming");
        self.id = Some(schedule(delay, callback)?);
        Ok(())
    }

    /// Cancel pending work, retaining the callback for a later arm.
    pub fn cancel(&mut self) {
        if let Some(id) = self.id.take() {
            remove(id);
        }
    }

    pub fn deadline(&self) -> Option<Instant> {
        self.id
            .and_then(|id| TIMERS.with(|timers| timers.borrow().get(&id).map(|t| t.deadline)))
    }

    pub fn is_pending(&self) -> bool {
        self.deadline().is_some()
    }
}

fn remove(id: NonZeroU64) {
    let registration = TIMERS.with(|timers| timers.borrow_mut().remove(&id));
    if let Some(registration) = registration {
        let task = registration.task.borrow_mut().take();
        drop(task);
    }
}

fn start(id: NonZeroU64, registration: &Rc<Registration>) -> io::Result<()> {
    let h = handle();
    let timer = registration.clone();
    let task = h.clone().spawn(async move {
        h.sleep_until(timer.deadline)
            .await
            .expect("timer wait failed");
        // Remove before dispatch so a callback can rearm or free its owner.
        remove(id);
        (timer.callback.borrow_mut())();
    })?;
    *registration.task.borrow_mut() = Some(task);
    Ok(())
}

fn schedule(delay: Duration, callback: Callback) -> io::Result<NonZeroU64> {
    register(next_id(), delay, callback)
}

fn next_id() -> NonZeroU64 {
    NEXT_ID.with(|next| {
        let id = next.get().checked_add(1).expect("timer IDs exhausted");
        next.set(id);
        NonZeroU64::new(id).unwrap()
    })
}

fn register(id: NonZeroU64, delay: Duration, callback: Callback) -> io::Result<NonZeroU64> {
    ensure_runtime();
    let now = Instant::now();
    // Preserve the existing behavior: an overflowing deadline is expired.
    let registration = Rc::new(Registration {
        deadline: now.checked_add(delay).unwrap_or(now),
        callback,
        task: RefCell::new(None),
    });
    TIMERS.with(|timers| timers.borrow_mut().insert(id, registration.clone()));
    if let Err(error) = start(id, &registration) {
        remove(id);
        return Err(error);
    }
    Ok(id)
}

/// Defer a callback to a runtime turn, without allocating an event handle.
pub fn timer_once(callback: impl FnOnce() + 'static) {
    let mut callback = Some(callback);
    schedule(
        Duration::ZERO,
        Rc::new(RefCell::new(Box::new(move || {
            callback.take().expect("one timer dispatch")();
        }))),
    )
    .expect("schedule deferred callback");
}

/// Keep the record in its Box; only the registration owns the callback.
/// This avoids a cycle through the timer embedded in the record.
pub fn timer_once_owned<T: 'static>(
    mut owner: Box<T>,
    timer_handle: fn(&mut T) -> &mut Timer,
    delay: Option<Duration>,
    callback: impl FnOnce(Box<T>) + 'static,
) {
    timer_handle(&mut owner).cancel();
    timer_handle(&mut owner).callback = None;
    let id = next_id();
    timer_handle(&mut owner).id = Some(id);
    let mut pending = Some((owner, callback));
    let callback: Callback = Rc::new(RefCell::new(Box::new(move || {
        let (owner, callback) = pending.take().expect("one owned timer dispatch");
        callback(owner);
    })));
    register(id, delay.unwrap_or(Duration::ZERO), callback).expect("arm owned timer");
}

pub(super) fn stop_tasks() {
    let timers = TIMERS.with(|timers| timers.borrow().values().cloned().collect::<Vec<_>>());
    for timer in timers {
        let task = timer.task.borrow_mut().take();
        drop(task);
    }
}

pub(super) fn restart() -> io::Result<()> {
    let timers = TIMERS.with(|timers| {
        timers
            .borrow()
            .iter()
            .map(|(&id, t)| (id, t.clone()))
            .collect::<Vec<_>>()
    });
    for (id, timer) in timers {
        start(id, &timer)?;
    }
    Ok(())
}

pub(super) fn clear() {
    loop {
        let timers = TIMERS.with(|timers| {
            timers
                .borrow_mut()
                .drain()
                .map(|(_, t)| t)
                .collect::<Vec<_>>()
        });
        if timers.is_empty() {
            break;
        }
        for timer in timers {
            let task = timer.task.borrow_mut().take();
            drop(task);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hmux_rt::Runtime as _;

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
    fn moving_rearming_and_cancelling_preserves_the_registration() {
        let calls = Rc::new(Cell::new(0));
        let observed = calls.clone();
        let mut timer = Timer::new();
        timer.set(move || observed.set(observed.get() + 1));
        let before = Instant::now();
        timer.arm(Duration::from_secs(60)).unwrap();
        let after = Instant::now();
        let deadline = timer.deadline().unwrap();
        assert!(deadline >= before + Duration::from_secs(60));
        assert!(deadline <= after + Duration::from_secs(60));
        let mut moved = timer;
        assert_eq!(moved.deadline(), Some(deadline));
        moved.arm(Duration::ZERO).unwrap();
        poll();
        assert_eq!(calls.get(), 1);
        assert!(!moved.is_pending());
        moved.arm(Duration::ZERO).unwrap();
        moved.cancel();
        moved.cancel();
        poll();
        assert_eq!(calls.get(), 1);
        assert!(moved.is_initialized());
        super::super::shutdown_runtime();
    }

    #[test]
    fn callbacks_can_rearm_and_free_their_owner() {
        let slot = Rc::new(RefCell::new(Some(Box::new(Timer::new()))));
        let observer = Rc::downgrade(&slot);
        let calls = Rc::new(Cell::new(0));
        let observed = calls.clone();
        slot.borrow_mut().as_mut().unwrap().set(move || {
            let slot = observer.upgrade().unwrap();
            observed.set(observed.get() + 1);
            let mut slot = slot.borrow_mut();
            let timer = slot.as_mut().unwrap();
            assert!(!timer.is_pending(), "remove before dispatch");
            if observed.get() == 1 {
                timer.arm(Duration::MAX).unwrap();
            } else {
                timer.cancel();
                drop(slot.take());
            }
        });
        slot.borrow_mut()
            .as_mut()
            .unwrap()
            .arm(Duration::ZERO)
            .unwrap();
        poll();
        assert_eq!(calls.get(), 2);
        assert!(slot.borrow().is_none());
        super::super::shutdown_runtime();
    }

    struct Owner {
        timer: Timer,
        freed: Rc<Cell<usize>>,
    }
    impl Drop for Owner {
        fn drop(&mut self) {
            self.timer.cancel();
            self.freed.set(self.freed.get() + 1);
        }
    }

    #[test]
    fn owned_callbacks_dispatch_once_and_shutdown_releases_pending_owners() {
        let freed = Rc::new(Cell::new(0));
        let calls = Rc::new(Cell::new(0));
        for delay in [None, Some(Duration::ZERO), Some(Duration::from_secs(60))] {
            let owner = Box::new(Owner {
                timer: Timer::new(),
                freed: freed.clone(),
            });
            let observed = calls.clone();
            timer_once_owned(
                owner,
                |owner| &mut owner.timer,
                delay,
                move |owner| {
                    assert!(!owner.timer.is_pending());
                    observed.set(observed.get() + 1);
                    drop(owner);
                },
            );
        }
        assert_eq!(calls.get(), 0, "spawn must not dispatch inline");
        assert_eq!(freed.get(), 0);
        poll();
        assert_eq!(calls.get(), 2);
        assert_eq!(freed.get(), 2);
        super::super::shutdown_runtime();
        assert_eq!(calls.get(), 2);
        assert_eq!(freed.get(), 3);
    }

    #[test]
    fn shutdown_cancels_deferred_cleanup_and_old_handles_cannot_cancel_new_timers() {
        struct DeferredDrop(Rc<Cell<usize>>);
        impl Drop for DeferredDrop {
            fn drop(&mut self) {
                let freed = self.0.clone();
                timer_once(move || freed.set(freed.get() + 1));
            }
        }
        let freed = Rc::new(Cell::new(0));
        let owner = DeferredDrop(freed.clone());
        timer_once(move || drop(owner));
        let stream_owner = DeferredDrop(freed.clone());
        unsafe {
            super::super::bufferevent_new(
                -1,
                crate::src::shared::event::bufferevent_data_callback(move |_| {
                    let _keep_owner = &stream_owner;
                    panic!("shutdown must not dispatch I/O");
                }),
                None,
                None,
            );
        }
        let mut old = Timer::new();
        old.set(|| {});
        old.arm(Duration::from_secs(60)).unwrap();
        super::super::shutdown_runtime();
        assert!(!old.is_pending());
        let mut new = Timer::new();
        new.set(|| {});
        new.arm(Duration::from_secs(60)).unwrap();
        old.cancel();
        assert!(new.is_pending());
        poll();
        assert_eq!(
            freed.get(),
            0,
            "shutdown must cancel nested deferred cleanup"
        );
        new.cancel();
        super::super::shutdown_runtime();
    }

    #[test]
    fn restarting_tasks_preserves_deadlines_and_dispatches_only_once() {
        let calls = Rc::new(Cell::new(0));
        let observed = calls.clone();
        let mut timer = Timer::new();
        timer.set(move || observed.set(observed.get() + 1));
        timer.arm(Duration::ZERO).unwrap();
        let deadline = timer.deadline();
        stop_tasks();
        assert_eq!(timer.deadline(), deadline);
        restart().unwrap();
        poll();
        assert_eq!(calls.get(), 1);
        assert!(!timer.is_pending());
        poll();
        assert_eq!(calls.get(), 1);
        super::super::shutdown_runtime();
    }
}
