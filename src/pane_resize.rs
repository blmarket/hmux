//! Coalescing of pane size changes before they are sent to an application.

use crate::types::u_int;
use std::collections::VecDeque;
use std::time::Duration;

const NORMAL_RETRY: Duration = Duration::from_millis(250);
const QUICK_RETRY: Duration = Duration::from_millis(10);

/// A pane's size in terminal cells.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PaneSize {
    /// The number of columns.
    pub width: u_int,
    /// The number of rows.
    pub height: u_int,
}

/// One resize to deliver and the delay before inspecting the queue again.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PaneResizeStep {
    /// The size to deliver.
    pub size: PaneSize,
    /// How long to wait before delivering another queued resize.
    pub retry_after: Duration,
}

#[derive(Clone, Copy)]
struct PaneResize {
    sx: u_int,
    sy: u_int,
    osx: u_int,
    osy: u_int,
}

impl PaneResize {
    fn size(&self) -> PaneSize { PaneSize { width: self.sx, height: self.sy } }
    fn old_size(&self) -> PaneSize { PaneSize { width: self.osx, height: self.osy } }
}

/// The pane resize queue used by hmux.
#[derive(Default)]
pub struct RustPaneResizeQueue {
    queue: VecDeque<PaneResize>,
}

impl RustPaneResizeQueue {
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn clear(&mut self) {
        self.queue.clear();
    }

    pub fn record(&mut self, old: PaneSize, new: PaneSize) {
        self.queue.push_back(PaneResize { sx: new.width, sy: new.height, osx: old.width, osy: old.height });
    }

    pub fn next_step(&mut self) -> Option<PaneResizeStep> {
        let first = *self.queue.front()?;
        let last = *self.queue.back()?;
        if self.queue.len() == 1 {
            self.queue.pop_front();
            return Some(PaneResizeStep {
                size: first.size(),
                retry_after: NORMAL_RETRY,
            });
        }
        if last.size() != first.old_size() {
            self.queue.clear();
            return Some(PaneResizeStep {
                size: last.size(),
                retry_after: NORMAL_RETRY,
            });
        }

        let resize = self.queue[self.queue.len() - 2];
        let last = self.queue.pop_back().expect("the queue has a last resize");
        self.queue.clear();
        self.queue.push_back(last);
        Some(PaneResizeStep {
            size: resize.size(),
            retry_after: QUICK_RETRY,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn size(width: u_int, height: u_int) -> PaneSize {
        PaneSize { width, height }
    }

    #[test]
    fn empty_and_single_resize_queues_are_consumed() {
        let mut queue = RustPaneResizeQueue::default();
        assert_eq!(queue.next_step(), None);
        queue.record(size(80, 24), size(100, 30));
        assert_eq!(
            queue.next_step(),
            Some(PaneResizeStep {
                size: size(100, 30),
                retry_after: NORMAL_RETRY,
            })
        );
        assert!(queue.is_empty());
    }

    #[test]
    fn a_run_ending_at_a_new_size_keeps_only_its_last_size() {
        let mut queue = RustPaneResizeQueue::default();
        queue.record(size(80, 24), size(90, 25));
        queue.record(size(90, 25), size(100, 30));
        assert_eq!(queue.next_step().map(|step| step.size), Some(size(100, 30)));
        assert!(queue.is_empty());
    }

    #[test]
    fn a_round_trip_delivers_an_intermediate_size_then_the_last() {
        let mut queue = RustPaneResizeQueue::default();
        queue.record(size(80, 24), size(90, 25));
        queue.record(size(90, 25), size(100, 30));
        queue.record(size(100, 30), size(80, 24));
        assert_eq!(
            queue.next_step(),
            Some(PaneResizeStep {
                size: size(100, 30),
                retry_after: QUICK_RETRY,
            })
        );
        assert_eq!(queue.next_step().map(|step| step.size), Some(size(80, 24)));
        assert!(queue.is_empty());
    }

    #[test]
    fn clear_discards_a_pending_run() {
        let mut queue = RustPaneResizeQueue::default();
        queue.record(size(80, 24), size(90, 25));
        queue.clear();
        assert_eq!(queue.next_step(), None);
    }
}
