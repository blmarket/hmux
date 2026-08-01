//! Thread-confined command queue state.
//!
//! A queue has at most one current item. The owner starts that item, then
//! either completes it immediately or marks it waiting while another event
//! source does work. Workers and UI callbacks return the [`QueueTicket`];
//! only the owner is allowed to mutate the queue.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_QUEUE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct QueueTicket {
    queue: u64,
    item: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum QueueState {
    Empty,
    Ready,
    Running(QueueTicket),
    Waiting(QueueTicket),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CurrentState {
    Running,
    Waiting,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct GroupId(u64);

struct Entry<T> {
    id: u64,
    group: GroupId,
    value: T,
}

struct Current {
    ticket: QueueTicket,
    group: GroupId,
    state: CurrentState,
}

/// An item checked out by the queue owner for execution.
pub(crate) struct Started<T> {
    pub(crate) ticket: QueueTicket,
    pub(crate) value: T,
}

/// Result of an item reaching a terminal state.
///
/// Each inner vector in `insert_next` is a command group. Failure may discard
/// the unstarted tail of the current group without affecting inserted groups or
/// later independently submitted groups.
pub(crate) struct QueueCompletion<T> {
    pub(crate) discard_group_tail: bool,
    pub(crate) insert_next: Vec<Vec<T>>,
}

impl<T> QueueCompletion<T> {
    pub(crate) fn done() -> Self {
        Self {
            discard_group_tail: false,
            insert_next: Vec::new(),
        }
    }

    pub(crate) fn failed() -> Self {
        Self {
            discard_group_tail: true,
            insert_next: Vec::new(),
        }
    }
}

/// Error returned when an event tries to resume or finish the wrong item.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct StaleQueueTicket;

/// FIFO command queue with tmux-style insertion immediately after the current
/// item.
///
/// This type is deliberately not synchronized. A protocol handler or startup
/// runner owns it, while background work communicates through tickets.
pub(crate) struct CommandQueue<T> {
    id: u64,
    next_item: u64,
    next_group: u64,
    pending: VecDeque<Entry<T>>,
    current: Option<Current>,
}

impl<T> Default for CommandQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> CommandQueue<T> {
    pub(crate) fn new() -> Self {
        Self {
            id: NEXT_QUEUE_ID.fetch_add(1, Ordering::Relaxed),
            next_item: 1,
            next_group: 1,
            pending: VecDeque::new(),
            current: None,
        }
    }

    pub(crate) fn state(&self) -> QueueState {
        match self.current.as_ref() {
            Some(Current {
                ticket,
                state: CurrentState::Running,
                ..
            }) => QueueState::Running(*ticket),
            Some(Current {
                ticket,
                state: CurrentState::Waiting,
                ..
            }) => QueueState::Waiting(*ticket),
            None if self.pending.is_empty() => QueueState::Empty,
            None => QueueState::Ready,
        }
    }

    /// Append an independently submitted command group.
    pub(crate) fn push_back_group<I>(&mut self, values: I)
    where
        I: IntoIterator<Item = T>,
    {
        let group = self.allocate_group();
        for value in values {
            let entry = self.entry(group, value);
            self.pending.push_back(entry);
        }
    }

    /// Start the next item. A waiting or running item must be resolved first.
    pub(crate) fn start_next(&mut self) -> Option<Started<T>> {
        if self.current.is_some() {
            return None;
        }
        let entry = self.pending.pop_front()?;
        let ticket = QueueTicket {
            queue: self.id,
            item: entry.id,
        };
        self.current = Some(Current {
            ticket,
            group: entry.group,
            state: CurrentState::Running,
        });
        Some(Started {
            ticket,
            value: entry.value,
        })
    }

    /// Mark the current item as waiting for an external completion event.
    pub(crate) fn wait(&mut self, ticket: QueueTicket) -> Result<(), StaleQueueTicket> {
        let current = self.current_mut(ticket)?;
        current.state = CurrentState::Waiting;
        Ok(())
    }

    /// Mark a waiting item runnable again without completing it.
    pub(crate) fn resume(&mut self, ticket: QueueTicket) -> Result<(), StaleQueueTicket> {
        let current = self.current_mut(ticket)?;
        if current.state != CurrentState::Waiting {
            return Err(StaleQueueTicket);
        }
        current.state = CurrentState::Running;
        Ok(())
    }

    /// Complete the current item, optionally discarding its group tail and
    /// inserting new groups before the previously pending tail.
    pub(crate) fn complete(
        &mut self,
        ticket: QueueTicket,
        completion: QueueCompletion<T>,
    ) -> Result<(), StaleQueueTicket> {
        let current = self.current.take().ok_or(StaleQueueTicket)?;
        if current.ticket != ticket {
            self.current = Some(current);
            return Err(StaleQueueTicket);
        }

        if completion.discard_group_tail {
            self.pending.retain(|entry| entry.group != current.group);
        }

        let mut inserted = Vec::new();
        for values in completion.insert_next {
            let group = self.allocate_group();
            inserted.extend(values.into_iter().map(|value| self.entry(group, value)));
        }
        for entry in inserted.into_iter().rev() {
            self.pending.push_front(entry);
        }
        Ok(())
    }

    fn current_mut(&mut self, ticket: QueueTicket) -> Result<&mut Current, StaleQueueTicket> {
        match self.current.as_mut() {
            Some(current) if current.ticket == ticket => Ok(current),
            _ => Err(StaleQueueTicket),
        }
    }

    fn allocate_group(&mut self) -> GroupId {
        let group = GroupId(self.next_group);
        self.next_group = self.next_group.wrapping_add(1).max(1);
        group
    }

    fn entry(&mut self, group: GroupId, value: T) -> Entry<T> {
        let id = self.next_item;
        self.next_item = self.next_item.wrapping_add(1).max(1);
        Entry { id, group, value }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finish(queue: &mut CommandQueue<&'static str>, started: Started<&'static str>) {
        queue
            .complete(started.ticket, QueueCompletion::done())
            .unwrap();
    }

    #[test]
    fn independent_groups_are_fifo() {
        let mut queue = CommandQueue::new();
        queue.push_back_group(["a", "b"]);
        queue.push_back_group(["c"]);

        for expected in ["a", "b", "c"] {
            let started = queue.start_next().unwrap();
            assert_eq!(started.value, expected);
            finish(&mut queue, started);
        }
        assert_eq!(queue.state(), QueueState::Empty);
    }

    #[test]
    fn insertions_run_before_the_existing_tail() {
        let mut queue = CommandQueue::new();
        queue.push_back_group(["source"]);
        queue.push_back_group(["tail"]);

        let source = queue.start_next().unwrap();
        queue
            .complete(
                source.ticket,
                QueueCompletion {
                    discard_group_tail: false,
                    insert_next: vec![vec!["first", "second"], vec!["finalize"]],
                },
            )
            .unwrap();

        for expected in ["first", "second", "finalize", "tail"] {
            let started = queue.start_next().unwrap();
            assert_eq!(started.value, expected);
            finish(&mut queue, started);
        }
    }

    #[test]
    fn nested_insertions_are_depth_first() {
        let mut queue = CommandQueue::new();
        queue.push_back_group(["source"]);

        let source = queue.start_next().unwrap();
        queue
            .complete(
                source.ticket,
                QueueCompletion {
                    discard_group_tail: false,
                    insert_next: vec![vec!["nested-source", "after"], vec!["outer-finalize"]],
                },
            )
            .unwrap();

        let nested = queue.start_next().unwrap();
        assert_eq!(nested.value, "nested-source");
        queue
            .complete(
                nested.ticket,
                QueueCompletion {
                    discard_group_tail: false,
                    insert_next: vec![vec!["nested-command"], vec!["nested-finalize"]],
                },
            )
            .unwrap();

        for expected in [
            "nested-command",
            "nested-finalize",
            "after",
            "outer-finalize",
        ] {
            let started = queue.start_next().unwrap();
            assert_eq!(started.value, expected);
            finish(&mut queue, started);
        }
    }

    #[test]
    fn waiting_item_keeps_the_queue_parked() {
        let mut queue = CommandQueue::new();
        queue.push_back_group(["confirm", "after"]);

        let confirm = queue.start_next().unwrap();
        queue.wait(confirm.ticket).unwrap();
        assert_eq!(queue.state(), QueueState::Waiting(confirm.ticket));
        assert!(queue.start_next().is_none());

        queue.resume(confirm.ticket).unwrap();
        queue
            .complete(confirm.ticket, QueueCompletion::done())
            .unwrap();
        let after = queue.start_next().unwrap();
        assert_eq!(after.value, "after");
    }

    #[test]
    fn failure_discards_only_the_current_group_tail() {
        let mut queue = CommandQueue::new();
        queue.push_back_group(["bad", "same-group"]);
        queue.push_back_group(["next-group"]);

        let bad = queue.start_next().unwrap();
        queue
            .complete(bad.ticket, QueueCompletion::failed())
            .unwrap();

        let next = queue.start_next().unwrap();
        assert_eq!(next.value, "next-group");
    }

    #[test]
    fn stale_completion_cannot_advance_another_queue() {
        let mut first = CommandQueue::new();
        let mut second = CommandQueue::new();
        first.push_back_group(["first"]);
        second.push_back_group(["second"]);
        let first_item = first.start_next().unwrap();
        let second_item = second.start_next().unwrap();

        assert_eq!(
            second.complete(first_item.ticket, QueueCompletion::done()),
            Err(StaleQueueTicket)
        );
        assert_eq!(second.state(), QueueState::Running(second_item.ticket));
    }
}
