//! Thread-confined command queue state.
//!
//! A queue has at most one current item, and whoever started it holds it for as
//! long as running it takes — an await, when running it means waiting. Only
//! that owner mutates the queue, so the item it completes is always the one it
//! started, and nothing has to name it.

use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct GroupId(u64);

struct Entry<T> {
    group: GroupId,
    value: T,
}

/// Result of an item reaching a terminal state.
///
/// Each inner vector in `insert_next` is a command group. Failure may discard
/// the unstarted tail of the current group without affecting inserted groups or
/// later independently submitted groups.
pub(in crate::server) struct QueueCompletion<T> {
    pub(in crate::server) discard_group_tail: bool,
    pub(in crate::server) insert_next: Vec<Vec<T>>,
}

impl<T> QueueCompletion<T> {
    pub(in crate::server) fn done() -> Self {
        Self {
            discard_group_tail: false,
            insert_next: Vec::new(),
        }
    }

    pub(in crate::server) fn failed() -> Self {
        Self {
            discard_group_tail: true,
            insert_next: Vec::new(),
        }
    }
}

/// FIFO command queue with tmux-style insertion immediately after the current
/// item.
///
/// This type is deliberately not synchronized. A protocol client or startup
/// runner owns it and runs its items one at a time.
pub(in crate::server) struct CommandQueue<T> {
    next_group: u64,
    pending: VecDeque<Entry<T>>,
    current: Option<GroupId>,
}

impl<T> Default for CommandQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> CommandQueue<T> {
    pub(in crate::server) fn new() -> Self {
        Self {
            next_group: 1,
            pending: VecDeque::new(),
            current: None,
        }
    }

    /// Whether the queue has nothing queued and nothing running.
    pub(in crate::server) fn is_empty(&self) -> bool {
        self.current.is_none() && self.pending.is_empty()
    }

    /// Append an independently submitted command group.
    pub(in crate::server) fn push_back_group<I>(&mut self, values: I)
    where
        I: IntoIterator<Item = T>,
    {
        let group = self.allocate_group();
        for value in values {
            self.pending.push_back(Entry { group, value });
        }
    }

    /// Take the next item, which becomes the current one. The item already
    /// running has to be completed first.
    /// Whether the group the current item belongs to still holds work that has
    /// not started.
    ///
    /// A control client's input line is one group, and tmux drains the whole
    /// client queue before the global queue fires — so a line's notifications
    /// wait behind every marker the line produces.
    pub(in crate::server) fn current_group_has_more(&self) -> bool {
        let Some(current) = self.current else {
            return false;
        };
        self.pending.iter().any(|entry| entry.group == current)
    }

    pub(in crate::server) fn start_next(&mut self) -> Option<T> {
        if self.current.is_some() {
            return None;
        }
        let entry = self.pending.pop_front()?;
        self.current = Some(entry.group);
        Some(entry.value)
    }

    /// Complete the current item, optionally discarding its group tail and
    /// inserting new groups before the previously pending tail.
    pub(in crate::server) fn complete(&mut self, completion: QueueCompletion<T>) {
        let group = self
            .current
            .take()
            .expect("a queue completes the item it started");

        if completion.discard_group_tail {
            self.pending.retain(|entry| entry.group != group);
        }

        let mut inserted = Vec::new();
        for values in completion.insert_next {
            let group = self.allocate_group();
            inserted.extend(values.into_iter().map(|value| Entry { group, value }));
        }
        for entry in inserted.into_iter().rev() {
            self.pending.push_front(entry);
        }
    }

    fn allocate_group(&mut self) -> GroupId {
        let group = GroupId(self.next_group);
        self.next_group = self.next_group.wrapping_add(1).max(1);
        group
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(queue: &mut CommandQueue<&'static str>) -> &'static str {
        let value = queue.start_next().expect("an item to run");
        queue.complete(QueueCompletion::done());
        value
    }

    #[test]
    fn independent_groups_are_fifo() {
        let mut queue = CommandQueue::new();
        queue.push_back_group(["a", "b"]);
        queue.push_back_group(["c"]);

        for expected in ["a", "b", "c"] {
            assert_eq!(run(&mut queue), expected);
        }
        assert!(queue.is_empty());
    }

    #[test]
    fn insertions_run_before_the_existing_tail() {
        let mut queue = CommandQueue::new();
        queue.push_back_group(["source"]);
        queue.push_back_group(["tail"]);

        assert_eq!(queue.start_next(), Some("source"));
        queue.complete(QueueCompletion {
            discard_group_tail: false,
            insert_next: vec![vec!["first", "second"], vec!["finalize"]],
        });

        for expected in ["first", "second", "finalize", "tail"] {
            assert_eq!(run(&mut queue), expected);
        }
    }

    #[test]
    fn nested_insertions_are_depth_first() {
        let mut queue = CommandQueue::new();
        queue.push_back_group(["source"]);

        assert_eq!(queue.start_next(), Some("source"));
        queue.complete(QueueCompletion {
            discard_group_tail: false,
            insert_next: vec![vec!["nested-source", "after"], vec!["outer-finalize"]],
        });

        assert_eq!(queue.start_next(), Some("nested-source"));
        queue.complete(QueueCompletion {
            discard_group_tail: false,
            insert_next: vec![vec!["nested-command"], vec!["nested-finalize"]],
        });

        for expected in [
            "nested-command",
            "nested-finalize",
            "after",
            "outer-finalize",
        ] {
            assert_eq!(run(&mut queue), expected);
        }
    }

    #[test]
    fn a_started_item_keeps_the_queue_parked() {
        // Running an item is the owner's own wait, however long it takes: the
        // queue hands out nothing else until that item is completed.
        let mut queue = CommandQueue::new();
        queue.push_back_group(["confirm", "after"]);

        assert_eq!(queue.start_next(), Some("confirm"));
        assert!(!queue.is_empty());
        assert_eq!(queue.start_next(), None);

        queue.complete(QueueCompletion::done());
        assert_eq!(queue.start_next(), Some("after"));
    }

    #[test]
    fn failure_discards_only_the_current_group_tail() {
        let mut queue = CommandQueue::new();
        queue.push_back_group(["bad", "same-group"]);
        queue.push_back_group(["next-group"]);

        assert_eq!(queue.start_next(), Some("bad"));
        queue.complete(QueueCompletion::failed());

        assert_eq!(queue.start_next(), Some("next-group"));
    }
}
