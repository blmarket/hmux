//! Deterministic timer scheduling for the host event loop.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::time::{Duration, Instant};

/// Stable identity for one scheduled timer.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TimerId(u64);

struct Timer<T> {
    deadline: Instant,
    value: T,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Deadline {
    at: Instant,
    id: TimerId,
}

/// A timer removed from the queue after reaching its deadline.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpiredTimer<T> {
    id: TimerId,
    value: T,
}

impl<T> ExpiredTimer<T> {
    pub fn id(&self) -> TimerId {
        self.id
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn into_value(self) -> T {
        self.value
    }
}

/// Timer queue ordered by deadline and then insertion order.
pub struct TimerQueue<T> {
    deadlines: BinaryHeap<Reverse<Deadline>>,
    timers: HashMap<TimerId, Timer<T>>,
    next_id: u64,
}

impl<T> Default for TimerQueue<T> {
    fn default() -> Self {
        Self {
            deadlines: BinaryHeap::new(),
            timers: HashMap::new(),
            next_id: 1,
        }
    }
}

impl<T> TimerQueue<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, deadline: Instant, value: T) -> TimerId {
        let id = self.allocate_id();
        self.deadlines.push(Reverse(Deadline { at: deadline, id }));
        self.timers.insert(id, Timer { deadline, value });
        id
    }

    pub fn cancel(&mut self, id: TimerId) -> Option<T> {
        let value = self.timers.remove(&id).map(|timer| timer.value);
        self.compact_if_sparse();
        value
    }

    pub fn next_deadline(&mut self) -> Option<Instant> {
        self.remove_stale_deadlines();
        self.deadlines.peek().map(|entry| entry.0.at)
    }

    pub fn time_until_next(&mut self, now: Instant) -> Option<Duration> {
        self.next_deadline()
            .map(|deadline| deadline.saturating_duration_since(now))
    }

    pub fn drain_expired(&mut self, now: Instant, output: &mut Vec<ExpiredTimer<T>>) {
        self.remove_stale_deadlines();
        while self
            .deadlines
            .peek()
            .is_some_and(|deadline| deadline.0.at <= now)
        {
            let Reverse(deadline) = self.deadlines.pop().expect("peeked deadline disappeared");
            let Some(timer) = self.timers.remove(&deadline.id) else {
                continue;
            };
            output.push(ExpiredTimer {
                id: deadline.id,
                value: timer.value,
            });
            self.remove_stale_deadlines();
        }
    }

    pub fn is_empty(&self) -> bool {
        self.timers.is_empty()
    }

    pub fn len(&self) -> usize {
        self.timers.len()
    }

    fn allocate_id(&mut self) -> TimerId {
        loop {
            let id = TimerId(self.next_id);
            self.next_id = self.next_id.wrapping_add(1);
            if self.next_id == 0 {
                self.next_id = 1;
            }
            if !self.timers.contains_key(&id) {
                return id;
            }
        }
    }

    fn remove_stale_deadlines(&mut self) {
        while self.deadlines.peek().is_some_and(|deadline| {
            self.timers
                .get(&deadline.0.id)
                .is_none_or(|timer| timer.deadline != deadline.0.at)
        }) {
            self.deadlines.pop();
        }
    }

    fn compact_if_sparse(&mut self) {
        if self.deadlines.len() <= self.timers.len().saturating_mul(2) + 64 {
            return;
        }
        self.deadlines = self
            .timers
            .iter()
            .map(|(&id, timer)| {
                Reverse(Deadline {
                    at: timer.deadline,
                    id,
                })
            })
            .collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timers_expire_by_deadline_then_insertion_order() {
        let start = Instant::now();
        let mut timers = TimerQueue::new();
        let late = timers.set(start + Duration::from_secs(2), "late");
        let first = timers.set(start + Duration::from_secs(1), "first");
        let second = timers.set(start + Duration::from_secs(1), "second");

        assert_eq!(timers.len(), 3);
        assert_eq!(timers.time_until_next(start), Some(Duration::from_secs(1)));

        let mut expired = Vec::new();
        timers.drain_expired(start + Duration::from_millis(999), &mut expired);
        assert!(expired.is_empty());

        timers.drain_expired(start + Duration::from_secs(1), &mut expired);
        assert_eq!(
            expired
                .iter()
                .map(|timer| (timer.id(), *timer.value()))
                .collect::<Vec<_>>(),
            vec![(first, "first"), (second, "second")]
        );
        assert_eq!(timers.next_deadline(), Some(start + Duration::from_secs(2)));

        timers.drain_expired(start + Duration::from_secs(2), &mut expired);
        assert_eq!(expired.last().map(ExpiredTimer::id), Some(late));
        assert!(timers.is_empty());
        assert_eq!(timers.next_deadline(), None);
    }

    #[test]
    fn cancellation_returns_value_and_removes_stale_deadline() {
        let start = Instant::now();
        let mut timers = TimerQueue::new();
        let cancelled = timers.set(start + Duration::from_secs(1), String::from("cancelled"));
        timers.set(start + Duration::from_secs(2), String::from("kept"));

        assert_eq!(timers.cancel(cancelled).as_deref(), Some("cancelled"));
        assert_eq!(timers.cancel(cancelled), None);
        assert_eq!(timers.next_deadline(), Some(start + Duration::from_secs(2)));
        assert_eq!(
            timers.time_until_next(start + Duration::from_secs(3)),
            Some(Duration::ZERO)
        );
    }

    #[test]
    fn expired_timers_append_and_can_yield_owned_values() {
        let now = Instant::now();
        let mut timers = TimerQueue::new();
        timers.set(now, String::from("event"));
        let mut expired = vec![ExpiredTimer {
            id: TimerId(999),
            value: String::from("existing"),
        }];

        timers.drain_expired(now, &mut expired);

        assert_eq!(expired.len(), 2);
        assert_eq!(expired.pop().unwrap().into_value(), "event");
        assert_eq!(expired.pop().unwrap().into_value(), "existing");
    }
}
