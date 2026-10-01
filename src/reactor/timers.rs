//! Timer work is a future owned by the caller's task handle.
use super::{ensure_runtime, handle};
use hmux_rt::Handle as _;
use std::io;
use std::time::{Duration, Instant};

/// Owns one scheduled future. Dropping it cancels the work.
/// Owners that track active work clear their timer from the callback.
pub struct Timer {
    _task: hmux_rt::mio::Task,
}

impl Timer {
    /// Zero duration defers execution to a runtime turn.
    pub fn new(delay: Duration, callback: impl FnOnce() + 'static) -> io::Result<Self> {
        ensure_runtime();
        let now = Instant::now();
        // Preserve the existing behavior: an overflowing deadline is expired.
        let deadline = now.checked_add(delay).unwrap_or(now);
        let sleep = handle().sleep_until(deadline);
        let task = handle().spawn(async move {
            sleep.await.expect("timer wait failed");
            callback();
        })?;
        Ok(Self { _task: task })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    fn poll() {
        super::super::poll_runtime_with_timeout(Some(Duration::ZERO));
    }

    #[test]
    fn moving_then_dropping_a_timer_cancels_pending_work_and_releases_captures() {
        let calls = Rc::new(Cell::new(0));
        for before_poll in [true, false] {
            let observed = calls.clone();
            let timer = Timer::new(Duration::from_secs(60), move || {
                observed.set(observed.get() + 1)
            })
            .unwrap();
            if !before_poll {
                poll();
            }
            let moved = timer;
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
        assert_eq!(
            Rc::strong_count(&calls),
            1,
            "firing releases the callback even while its handle lives"
        );
        drop(timer);
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
                // The running future remains owned until it returns or is cancelled.
                drop(slot.borrow_mut().take());
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
    fn dropping_a_timer_from_a_closed_runtime_does_not_cancel_new_timers() {
        let calls = Rc::new(Cell::new(0));
        let old = Timer::new(Duration::ZERO, || panic!("old timer must not dispatch")).unwrap();
        super::super::shutdown_runtime();
        let observed = calls.clone();
        let timer = Timer::new(Duration::ZERO, move || observed.set(observed.get() + 1)).unwrap();
        drop(old);
        poll();
        assert_eq!(calls.get(), 1);
        super::super::shutdown_runtime();
        assert_eq!(Rc::strong_count(&calls), 1);
        drop(timer);
    }
}
