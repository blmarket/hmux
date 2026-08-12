//! Background-job and `wait-for` channel registries.
//!
//! Both are interior-mutable side tables hanging off `ServerState`: the job
//! registry tracks `run-shell`-style children so `list-jobs` can report them,
//! and the wait registry backs `wait-for`'s channel/lock semantics.

use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::os::fd::RawFd;

use hmux_rt::{completion_pair, Completion, CompletionSender};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BackgroundJob {
    pub(crate) command: String,
    pub(crate) fd: RawFd,
    pub(crate) pid: u32,
}

#[derive(Default)]
struct BackgroundJobRegistryState {
    next_id: u64,
    jobs: BTreeMap<u64, BackgroundJob>,
}

#[derive(Default)]
pub(crate) struct BackgroundJobRegistry {
    inner: RefCell<BackgroundJobRegistryState>,
}

impl BackgroundJobRegistry {
    pub(crate) fn register(&self, command: String, fd: RawFd, pid: u32) -> u64 {
        let mut inner = self.inner.borrow_mut();
        let id = inner.next_id;
        inner.next_id = inner.next_id.wrapping_add(1);
        inner.jobs.insert(id, BackgroundJob { command, fd, pid });
        id
    }

    pub(crate) fn remove(&self, id: u64) {
        self.inner.borrow_mut().jobs.remove(&id);
    }

    pub(crate) fn jobs(&self) -> Vec<BackgroundJob> {
        self.inner.borrow().jobs.values().cloned().collect()
    }
}

#[derive(Default)]
struct WaitChannel {
    locked: bool,
    woken: bool,
    /// `wait-for channel` callers still waiting for a `-S`.
    waiters: Vec<CompletionSender<()>>,
    /// `wait-for -L` callers still waiting for the lock, in request order.
    lock_queue: VecDeque<CompletionSender<()>>,
}

/// Whether a `wait-for` channel operation finished at once, or has to wait for
/// another client.
///
/// The pending arm hands back the completion the registry will signal, so the
/// waiting command queue is resumed by whichever driver owns it rather than
/// parking a thread inside the registry.
pub(crate) enum WaitOutcome {
    Ready,
    Pending(Completion<()>),
}

#[derive(Default)]
pub(crate) struct WaitRegistry {
    channels: RefCell<BTreeMap<String, WaitChannel>>,
}

impl WaitRegistry {
    pub(crate) fn signal(&self, name: &str) {
        let mut channels = self.channels.borrow_mut();
        let channel = channels.entry(name.to_string()).or_default();
        channel.woken = true;
        // Every waiter is released together, as the condvar broadcast this
        // replaces did; the wake is consumed as the last of them leaves.
        let waiters = std::mem::take(&mut channel.waiters);
        if !waiters.is_empty() {
            if channel.locked {
                channel.woken = false;
            } else {
                channels.remove(name);
            }
        }
        drop(channels);
        for waiter in waiters {
            waiter.complete(());
        }
    }

    pub(crate) fn wait(&self, name: &str) -> WaitOutcome {
        let mut channels = self.channels.borrow_mut();
        let channel = channels.entry(name.to_string()).or_default();
        if channel.woken {
            if channel.locked {
                channel.woken = false;
            } else {
                channels.remove(name);
            }
            return WaitOutcome::Ready;
        }
        let Ok((completion, sender)) = completion_pair() else {
            return WaitOutcome::Ready;
        };
        channel.waiters.push(sender);
        WaitOutcome::Pending(completion)
    }

    pub(crate) fn lock(&self, name: &str) -> WaitOutcome {
        let mut channels = self.channels.borrow_mut();
        let channel = channels.entry(name.to_string()).or_default();
        if !channel.locked {
            channel.locked = true;
            return WaitOutcome::Ready;
        }
        let Ok((completion, sender)) = completion_pair() else {
            return WaitOutcome::Ready;
        };
        channel.lock_queue.push_back(sender);
        WaitOutcome::Pending(completion)
    }

    pub(crate) fn unlock(&self, name: &str) -> bool {
        let mut channels = self.channels.borrow_mut();
        let Some(channel) = channels.get_mut(name) else {
            return false;
        };
        if !channel.locked {
            return false;
        }
        // Hand the lock straight to the next waiter instead of dropping it and
        // letting every blocked caller race for it.
        let next = channel.lock_queue.pop_front();
        if next.is_none() {
            channel.locked = false;
            if channel.woken && channel.waiters.is_empty() {
                channels.remove(name);
            }
        }
        drop(channels);
        if let Some(next) = next {
            next.complete(());
        }
        true
    }
}
