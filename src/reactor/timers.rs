//! Callback timers owned by the reactor and cancelled by dropping their handles.
use super::{ensure_runtime, handle};
use hmux_rt::Handle as _;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::io;
use std::marker::PhantomData;
use std::num::NonZeroU64;
use std::time::{Duration, Instant};

/// A movable handle to one scheduled callback. Dropping it stops pending work
/// and releases the callback. Schedule a new timer to repeat or resume work.
pub struct Timer {
    id: NonZeroU64,
    // Registrations belong to the current thread's reactor.
    _local: PhantomData<*mut ()>,
}

struct Registration {
    deadline: Instant,
    callback: Box<dyn FnMut()>,
    task: Option<hmux_rt::mio::Task>,
}

thread_local! {
    static NEXT_ID: Cell<u64> = const { Cell::new(0) };
    static TIMERS: RefCell<HashMap<NonZeroU64, Registration>> = RefCell::new(HashMap::new());
}

impl Timer {
    /// Schedule one callback. Zero duration defers it to a runtime turn.
    pub fn new(delay: Duration, callback: impl FnMut() + 'static) -> io::Result<Self> {
        let id = next_id();
        register(id, delay, Box::new(callback))?;
        Ok(Self {
            id,
            _local: PhantomData,
        })
    }

    pub fn deadline(&self) -> Option<Instant> {
        TIMERS.with(|timers| timers.borrow().get(&self.id).map(|timer| timer.deadline))
    }

    pub fn is_pending(&self) -> bool {
        self.deadline().is_some()
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        remove(self.id);
    }
}

fn remove(id: NonZeroU64) -> Option<Registration> {
    // Captured owners may drop their handles during thread-local teardown.
    let mut registration = TIMERS
        .try_with(|timers| timers.borrow_mut().remove(&id))
        .ok()
        .flatten();
    if let Some(timer) = registration.as_mut() {
        drop(timer.task.take());
    }
    registration
}

fn start(id: NonZeroU64, deadline: Instant) -> io::Result<()> {
    let h = handle();
    let task = h.clone().spawn(async move {
        h.sleep_until(deadline).await.expect("timer wait failed");
        // Remove before dispatch, with no registry borrow held. The callback
        // may drop its handle or replace it with a newly scheduled timer.
        if let Some(mut timer) = remove(id) {
            (timer.callback)();
        }
    })?;
    let previous = TIMERS.with(|timers| {
        timers
            .borrow_mut()
            .get_mut(&id)
            .expect("registered timer")
            .task
            .replace(task)
    });
    drop(previous);
    Ok(())
}

fn next_id() -> NonZeroU64 {
    NEXT_ID.with(|next| {
        let id = next.get().checked_add(1).expect("timer IDs exhausted");
        next.set(id);
        NonZeroU64::new(id).unwrap()
    })
}

fn register(id: NonZeroU64, delay: Duration, callback: Box<dyn FnMut()>) -> io::Result<()> {
    ensure_runtime();
    let now = Instant::now();
    // Preserve the existing behavior: an overflowing deadline is expired.
    let deadline = now.checked_add(delay).unwrap_or(now);
    let registration = Registration {
        deadline,
        callback,
        task: None,
    };
    let previous = TIMERS.with(|timers| timers.borrow_mut().insert(id, registration));
    assert!(previous.is_none(), "timer IDs are never reused");
    if let Err(error) = start(id, deadline) {
        drop(remove(id));
        return Err(error);
    }
    Ok(())
}

/// Queue a callback without running it inline.
#[deprecated(note = "use reactor::defer for deferred execution without a timer")]
pub fn timer_once(callback: impl FnOnce() + 'static) {
    super::defer(callback);
}

/// Transfer a record and its timer to the reactor until dispatch or shutdown.
pub fn timer_once_owned<T: 'static>(
    mut owner: Box<T>,
    timer_handle: fn(&mut T) -> &mut Option<Timer>,
    delay: Option<Duration>,
    mut callback: impl FnMut(Box<T>) + 'static,
) {
    let id = next_id();
    *timer_handle(&mut owner) = Some(Timer {
        id,
        _local: PhantomData,
    });
    let mut owner = Some(owner);
    register(
        id,
        delay.unwrap_or(Duration::ZERO),
        Box::new(move || callback(owner.take().expect("one owned timer dispatch"))),
    )
    .expect("arm owned timer");
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
        for mut timer in timers {
            drop(timer.task.take());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    fn poll() {
        super::super::poll_runtime_with_timeout(Some(Duration::ZERO));
    }

    #[test]
    fn moving_then_dropping_a_timer_cancels_pending_work_and_releases_captures() {
        let calls = Rc::new(Cell::new(0));
        for before_poll in [true, false] {
            let observed = calls.clone();
            let before = Instant::now();
            let timer = Timer::new(Duration::from_secs(60), move || {
                observed.set(observed.get() + 1)
            })
            .unwrap();
            let deadline = timer.deadline().unwrap();
            assert!(deadline >= before + Duration::from_secs(60));
            if !before_poll {
                poll();
            }
            let moved = timer;
            assert_eq!(moved.deadline(), Some(deadline));
            assert_eq!(Rc::strong_count(&calls), 2);
            drop(moved);
            assert_eq!(Rc::strong_count(&calls), 1);
            poll();
            assert_eq!(calls.get(), 0);
        }
        super::super::shutdown_runtime();
    }

    #[test]
    fn pausing_drops_the_old_callback_and_resuming_schedules_a_new_one() {
        let calls = Rc::new(Cell::new(0));
        let observed = calls.clone();
        let mut timer = Some(Timer::new(Duration::ZERO, move || observed.set(100)).unwrap());
        drop(timer.take());
        assert_eq!(Rc::strong_count(&calls), 1);
        let observed = calls.clone();
        timer = Some(Timer::new(Duration::ZERO, move || observed.set(observed.get() + 1)).unwrap());
        poll();
        assert_eq!(calls.get(), 1);
        assert!(!timer.as_ref().unwrap().is_pending());
        assert_eq!(
            Rc::strong_count(&calls),
            1,
            "firing releases the callback even while its handle lives"
        );
        super::super::shutdown_runtime();
    }

    #[test]
    fn callbacks_can_replace_and_drop_their_own_timer() {
        let slot = Rc::new(RefCell::new(None::<Timer>));
        let observer = Rc::downgrade(&slot);
        let calls = Rc::new(Cell::new(0));
        let observed = calls.clone();
        *slot.borrow_mut() = Some(
            Timer::new(Duration::ZERO, move || {
                let slot = observer.upgrade().unwrap();
                assert!(!slot.borrow().as_ref().unwrap().is_pending());
                observed.set(observed.get() + 1);
                let observer = Rc::downgrade(&slot);
                let observed = observed.clone();
                *slot.borrow_mut() = Some(
                    Timer::new(Duration::MAX, move || {
                        observed.set(observed.get() + 1);
                        drop(observer.upgrade().unwrap().borrow_mut().take());
                    })
                    .unwrap(),
                );
            })
            .unwrap(),
        );
        poll();
        assert_eq!(calls.get(), 2);
        assert!(slot.borrow().is_none());
        assert_eq!(Rc::strong_count(&calls), 1);
        super::super::shutdown_runtime();
    }

    #[test]
    fn owned_timers_dispatch_once_and_shutdown_releases_pending_owners() {
        let calls = Rc::new(Cell::new(0));
        let retained = Rc::new(());
        for delay in [Duration::ZERO, Duration::from_secs(60)] {
            let observed = calls.clone();
            timer_once_owned(
                Box::new((None, retained.clone())),
                |owner| &mut owner.0,
                Some(delay),
                move |owner| {
                    assert!(!owner.0.as_ref().unwrap().is_pending());
                    observed.set(observed.get() + 1);
                },
            );
        }
        assert_eq!(Rc::strong_count(&retained), 3);
        poll();
        poll();
        assert_eq!(calls.get(), 1);
        assert_eq!(Rc::strong_count(&retained), 2);
        super::super::shutdown_runtime();
        assert_eq!(Rc::strong_count(&retained), 1);
        assert_eq!(Rc::strong_count(&calls), 1);
    }

    #[test]
    fn shutdown_invalidates_old_handles_without_cancelling_new_timers() {
        let calls = Rc::new(Cell::new(0));
        let old = Timer::new(Duration::ZERO, || panic!("old timer must not dispatch")).unwrap();
        super::super::shutdown_runtime();
        assert!(!old.is_pending());
        let observed = calls.clone();
        let timer = Timer::new(Duration::ZERO, move || observed.set(observed.get() + 1)).unwrap();
        drop(old);
        poll();
        assert_eq!(calls.get(), 1);
        super::super::shutdown_runtime();
        assert!(!timer.is_pending());
        assert_eq!(Rc::strong_count(&calls), 1);
    }

    #[test]
    fn thread_exit_releases_owned_timers_without_reentering_the_destroyed_registry() {
        std::thread::spawn(|| {
            timer_once_owned(
                Box::new(None),
                |timer| timer,
                Some(Duration::from_secs(60)),
                |_| panic!("thread exit must not dispatch"),
            );
        })
        .join()
        .unwrap();
    }
}
