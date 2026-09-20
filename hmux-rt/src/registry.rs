//! Explicit callback registrations driven by a host-owned TaskRuntime.
//! Release registrations before closing or reusing their descriptors.
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ffi::{c_int, c_short};
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::{Interest as RtInterest, Readiness, Signals, TaskHandle, TaskId, sleep_until};

use super::adapters::{WatchInterest as Interest, WatchMode};
use super::notify::yield_now;

pub const EV_READ: c_short = 0x2;
pub const EV_WRITE: c_short = 0x4;
pub const EV_SIGNAL: c_short = 0x8;

struct TimerSlot {
    generation: u64,
    armed: bool,
    deadline: Option<Instant>,
    callback: TimerCallback,
    task: Option<TaskId>,
}

type TimerCallback = Rc<RefCell<Box<dyn FnMut()>>>;

struct IoSlot {
    generation: u64,
    fd: c_int,
    interest: Interest,
    mode: WatchMode,
    enabled: bool,
    callback: IoCallback,
    task: Option<TaskId>,
}

type IoCallback = Rc<RefCell<Box<dyn FnMut(c_int, c_short)>>>;

struct SignalSlot {
    generation: u64,
    signo: c_int,
    active: bool,
    callback: SignalCallback,
    task: Option<TaskId>,
}

type SignalCallback = Rc<RefCell<Box<dyn FnMut(c_int, c_short)>>>;

pub struct DeferredCall {
    pub epoch: u64,
    pub callback: Box<dyn FnOnce()>,
}

#[derive(Default)]
struct RuntimeControlInner {
    next_id: usize,
    epoch: u64,
    timers: HashMap<usize, TimerSlot>,
    ios: HashMap<usize, IoSlot>,
    signals: HashMap<usize, SignalSlot>,
    deferred: std::collections::VecDeque<DeferredCall>,
    task_handle: Option<TaskHandle>,
    deferred_live: HashMap<usize, Rc<Cell<bool>>>,
}

#[derive(Clone, Default)]
pub struct RuntimeControl {
    inner: Rc<RefCell<RuntimeControlInner>>,
}

impl RuntimeControl {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn begin_epoch(&self) -> u64 {
        let mut inner = self.inner.borrow_mut();
        inner.epoch = inner.epoch.wrapping_add(1);
        inner.epoch
    }

    pub fn take_ready_deferred(&self, epoch: u64) -> Vec<DeferredCall> {
        let mut inner = self.inner.borrow_mut();
        let mut ready = Vec::new();
        let mut waiting = std::collections::VecDeque::new();
        while let Some(call) = inner.deferred.pop_front() {
            if call.epoch <= epoch {
                ready.push(call);
            } else {
                waiting.push_back(call);
            }
        }
        inner.deferred = waiting;
        ready
    }

    pub fn set_task_handle(&self, handle: TaskHandle) {
        self.inner.borrow_mut().task_handle = Some(handle);
    }

    fn allocate_id(inner: &mut RuntimeControlInner) -> usize {
        assert_ne!(inner.next_id, usize::MAX, "reactor handle space exhausted");
        inner.next_id += 1;
        inner.next_id
    }

    pub fn allocate_timer(&self, callback: impl FnMut() + 'static) -> usize {
        let mut inner = self.inner.borrow_mut();
        let id = Self::allocate_id(&mut inner);
        inner.timers.insert(
            id,
            TimerSlot {
                generation: 1,
                armed: false,
                deadline: None,
                callback: Rc::new(RefCell::new(Box::new(callback))),
                task: None,
            },
        );
        id
    }

    pub fn arm_timer(&self, id: usize, after: Duration) {
        let (handle, old_task, generation, deadline) = {
            let mut inner = self.inner.borrow_mut();
            let handle = inner.task_handle.clone();
            let Some(slot) = inner.timers.get_mut(&id) else {
                return;
            };
            slot.generation = next_generation(slot.generation);
            slot.armed = true;
            let deadline = timer_deadline(after);
            slot.deadline = Some(deadline);
            let old_task = slot.task.take();
            (handle, old_task, slot.generation, deadline)
        };
        if let Some(old_task) = old_task
            && let Some(handle) = &handle
        {
            handle.cancel(old_task);
        }
        let Some(handle) = handle else {
            return;
        };
        self.spawn_timer(handle, id, generation, deadline);
    }

    fn fire_timer(&self, id: usize, generation: u64) {
        let callback = {
            let mut inner = self.inner.borrow_mut();
            let Some(slot) = inner.timers.get_mut(&id) else {
                return;
            };
            if slot.generation != generation || !slot.armed {
                return;
            }
            slot.armed = false;
            slot.deadline = None;
            slot.task = None;
            slot.callback.clone()
        };
        callback.borrow_mut()();
    }

    pub fn disarm_timer(&self, id: usize) {
        let (handle, task) = {
            let mut inner = self.inner.borrow_mut();
            let handle = inner.task_handle.clone();
            let Some(slot) = inner.timers.get_mut(&id) else {
                return;
            };
            slot.generation = next_generation(slot.generation);
            slot.armed = false;
            slot.deadline = None;
            (handle, slot.task.take())
        };
        if let (Some(handle), Some(task)) = (handle, task) {
            handle.cancel(task);
        }
    }

    /// Disarms the timer and removes its slot so the handle id is retired.
    pub fn release_timer(&self, id: usize) {
        self.disarm_timer(id);
        let removed = self.inner.borrow_mut().timers.remove(&id);
        drop(removed);
    }

    pub fn timer_deadline(&self, id: usize) -> Option<Instant> {
        self.inner.borrow().timers.get(&id).and_then(|s| s.deadline)
    }

    pub fn is_timer_armed(&self, id: usize) -> bool {
        self.inner
            .borrow()
            .timers
            .get(&id)
            .is_some_and(|slot| slot.armed)
    }

    pub fn allocate_io(
        &self,
        fd: c_int,
        interest: Interest,
        mode: WatchMode,
        callback: impl FnMut(c_int, c_short) + 'static,
    ) -> usize {
        let mut inner = self.inner.borrow_mut();
        let id = Self::allocate_id(&mut inner);
        inner.ios.insert(
            id,
            IoSlot {
                generation: 1,
                fd,
                interest,
                mode,
                enabled: false,
                callback: Rc::new(RefCell::new(Box::new(callback))),
                task: None,
            },
        );
        id
    }

    pub fn enable_io(&self, id: usize) {
        let (handle, old_task, generation, fd, interest, mode) = {
            let mut inner = self.inner.borrow_mut();
            let handle = inner.task_handle.clone();
            let Some(slot) = inner.ios.get_mut(&id) else {
                return;
            };
            if slot.enabled {
                return;
            }
            slot.generation = next_generation(slot.generation);
            slot.enabled = true;
            (
                handle,
                slot.task.take(),
                slot.generation,
                slot.fd,
                slot.interest,
                slot.mode,
            )
        };
        if let (Some(handle), Some(task)) = (&handle, old_task) {
            handle.cancel(task);
        }
        let Some(handle) = handle else {
            return;
        };
        self.spawn_io(handle, id, generation, fd, interest, mode);
    }

    fn spawn_io(
        &self,
        handle: TaskHandle,
        id: usize,
        generation: u64,
        fd: c_int,
        interest: Interest,
        mode: WatchMode,
    ) {
        let control = self.clone();
        let task_handle = handle.clone();
        let task = handle.spawn(async move {
            control
                .run_io_task(task_handle, id, generation, fd, interest, mode)
                .await;
        });
        let mut inner = self.inner.borrow_mut();
        if let Some(slot) = inner.ios.get_mut(&id) {
            if slot.generation == generation && slot.enabled {
                slot.task = Some(task);
            } else {
                handle.cancel(task);
            }
        }
    }

    async fn run_io_task(
        &self,
        task_handle: TaskHandle,
        id: usize,
        generation: u64,
        fd: c_int,
        interest: Interest,
        mode: WatchMode,
    ) {
        let rt_interest = runtime_interest(interest);
        match mode {
            WatchMode::Once => {
                let async_fd = match super::adapters::register_fd(&task_handle, fd, rt_interest) {
                    Ok(async_fd) => async_fd,
                    Err(_) => {
                        self.disable_io(id);
                        return;
                    }
                };
                let readiness = async_fd.readiness().await;
                drop(async_fd);
                let callback = {
                    let mut inner = self.inner.borrow_mut();
                    let Some(slot) = inner.ios.get_mut(&id) else {
                        return;
                    };
                    if slot.generation != generation || !slot.enabled {
                        return;
                    }
                    slot.enabled = false;
                    slot.task = None;
                    slot.callback.clone()
                };
                callback.borrow_mut()(fd, readiness_flags(readiness, interest));
            }
            WatchMode::Persistent => loop {
                if !self.io_is_current(id, generation) {
                    return;
                }
                let async_fd = match super::adapters::register_fd(&task_handle, fd, rt_interest) {
                    Ok(async_fd) => async_fd,
                    Err(_) => {
                        self.disable_io(id);
                        return;
                    }
                };
                let readiness = async_fd.readiness().await;
                drop(async_fd);
                let callback = {
                    let inner = self.inner.borrow();
                    let Some(slot) = inner.ios.get(&id) else {
                        return;
                    };
                    if slot.generation != generation || !slot.enabled {
                        return;
                    }
                    slot.callback.clone()
                };
                callback.borrow_mut()(fd, readiness_flags(readiness, interest));
                yield_now().await;
            },
        }
    }

    fn io_is_current(&self, id: usize, generation: u64) -> bool {
        self.inner
            .borrow()
            .ios
            .get(&id)
            .is_some_and(|slot| slot.generation == generation && slot.enabled)
    }

    pub fn disable_io(&self, id: usize) {
        let (handle, task) = {
            let mut inner = self.inner.borrow_mut();
            let handle = inner.task_handle.clone();
            let Some(slot) = inner.ios.get_mut(&id) else {
                return;
            };
            slot.generation = next_generation(slot.generation);
            slot.enabled = false;
            (handle, slot.task.take())
        };
        if let (Some(handle), Some(task)) = (handle, task) {
            handle.cancel(task);
        }
    }

    /// Disables the watch and removes its slot so the handle id is retired.
    pub fn release_io(&self, id: usize) {
        self.disable_io(id);
        let removed = self.inner.borrow_mut().ios.remove(&id);
        drop(removed);
    }

    #[cfg(test)]
    pub fn is_io_enabled(&self, id: usize) -> bool {
        self.inner
            .borrow()
            .ios
            .get(&id)
            .is_some_and(|slot| slot.enabled)
    }

    pub fn watch_signal(
        &self,
        signo: c_int,
        callback: impl FnMut(c_int, c_short) + 'static,
    ) -> usize {
        let (handle, id, generation) = {
            let mut inner = self.inner.borrow_mut();
            let id = Self::allocate_id(&mut inner);
            inner.signals.insert(
                id,
                SignalSlot {
                    generation: 1,
                    signo,
                    active: true,
                    callback: Rc::new(RefCell::new(Box::new(callback))),
                    task: None,
                },
            );
            (inner.task_handle.clone(), id, 1)
        };
        if let Some(handle) = handle {
            self.spawn_signal(handle, id, generation, signo);
        }
        id
    }

    fn spawn_signal(&self, handle: TaskHandle, id: usize, generation: u64, signo: c_int) {
        let signals = match Signals::new(&handle, &[signo]) {
            Ok(signals) => signals,
            Err(_) => {
                let mut inner = self.inner.borrow_mut();
                if let Some(slot) = inner.signals.get_mut(&id)
                    && slot.generation == generation
                    && slot.active
                {
                    slot.generation = next_generation(slot.generation);
                    slot.active = false;
                }
                return;
            }
        };
        let control = self.clone();
        let task_handle = handle.clone();
        let task = handle.spawn(async move {
            control
                .run_signal_task(task_handle, signals, id, generation, signo)
                .await;
        });
        let mut inner = self.inner.borrow_mut();
        if let Some(slot) = inner.signals.get_mut(&id) {
            if slot.generation == generation && slot.active {
                slot.task = Some(task);
            } else {
                handle.cancel(task);
            }
        }
    }

    async fn run_signal_task(
        &self,
        _task_handle: TaskHandle,
        mut signals: Signals,
        id: usize,
        generation: u64,
        signo: c_int,
    ) {
        loop {
            if signals.recv().await.is_err() {
                return;
            }
            let callback = {
                let inner = self.inner.borrow();
                let Some(slot) = inner.signals.get(&id) else {
                    return;
                };
                if slot.generation != generation || !slot.active {
                    return;
                }
                slot.callback.clone()
            };
            callback.borrow_mut()(signo, EV_SIGNAL);
        }
    }

    pub fn unwatch_signal(&self, id: usize) {
        let (handle, task) = {
            let mut inner = self.inner.borrow_mut();
            let handle = inner.task_handle.clone();
            let Some(slot) = inner.signals.get_mut(&id) else {
                return;
            };
            slot.generation = next_generation(slot.generation);
            slot.active = false;
            (handle, slot.task.take())
        };
        if let (Some(handle), Some(task)) = (handle, task) {
            handle.cancel(task);
        }
    }

    /// Stops the signal watch and removes its slot so the handle id is retired.
    pub fn release_signal(&self, id: usize) {
        self.unwatch_signal(id);
        let removed = self.inner.borrow_mut().signals.remove(&id);
        drop(removed);
    }

    /// Drops everything the reactor is holding on behalf of its callers:
    /// queued deferred calls first, then the timer, io and signal callbacks.
    ///
    /// Those drops reach back into the reactor — releasing a client disarms its
    /// timers — so they have to run while the reactor is still there to answer
    /// them, which is what this is for. Each round is taken out of the control
    /// block before it is dropped, because a drop may queue or release more,
    /// and the rounds repeat until nothing is left.
    pub fn shutdown(&self) {
        let (timers, ios, signals) = {
            let inner = self.inner.borrow();
            (
                inner.timers.keys().copied().collect::<Vec<_>>(),
                inner.ios.keys().copied().collect::<Vec<_>>(),
                inner.signals.keys().copied().collect::<Vec<_>>(),
            )
        };
        for id in timers {
            self.release_timer(id);
        }
        for id in ios {
            self.release_io(id);
        }
        for id in signals {
            self.release_signal(id);
        }
        let deferred = {
            let mut inner = self.inner.borrow_mut();
            for live in inner.deferred_live.drain().map(|(_, live)| live) {
                live.set(false);
            }
            std::mem::take(&mut inner.deferred)
        };
        drop(deferred);
    }

    #[cfg(test)]
    pub fn slot_counts(&self) -> (usize, usize, usize) {
        let inner = self.inner.borrow();
        (inner.timers.len(), inner.ios.len(), inner.signals.len())
    }

    /// Queue for the next host epoch. Cancellation remains effective even if
    /// the host has already taken the ready batch.
    pub fn defer(&self, callback: impl FnOnce() + 'static) -> usize {
        let mut inner = self.inner.borrow_mut();
        let id = Self::allocate_id(&mut inner);
        let epoch = inner.epoch.wrapping_add(1);
        let live = Rc::new(Cell::new(true));
        inner.deferred_live.insert(id, live.clone());
        let weak = Rc::downgrade(&self.inner);
        inner.deferred.push_back(DeferredCall {
            epoch,
            callback: Box::new(move || {
                if let Some(inner) = weak.upgrade() {
                    inner.borrow_mut().deferred_live.remove(&id);
                }
                if live.replace(false) {
                    callback();
                }
            }),
        });
        id
    }

    pub fn cancel_deferred(&self, id: usize) {
        if let Some(live) = self.inner.borrow_mut().deferred_live.remove(&id) {
            live.set(false);
        }
    }

    pub fn respawn_active(&self, handle: &TaskHandle) {
        self.set_task_handle(handle.clone());
        let timers = {
            let mut inner = self.inner.borrow_mut();
            inner
                .timers
                .iter_mut()
                .filter_map(|(&id, slot)| {
                    if !slot.armed {
                        return None;
                    }
                    slot.task = None;
                    slot.generation = next_generation(slot.generation);
                    Some((id, slot.generation, slot.deadline?))
                })
                .collect::<Vec<_>>()
        };
        let ios = {
            let mut inner = self.inner.borrow_mut();
            inner
                .ios
                .iter_mut()
                .filter_map(|(&id, slot)| {
                    if !slot.enabled {
                        return None;
                    }
                    slot.task = None;
                    slot.generation = next_generation(slot.generation);
                    Some((id, slot.generation, slot.fd, slot.interest, slot.mode))
                })
                .collect::<Vec<_>>()
        };
        let signals = {
            let mut inner = self.inner.borrow_mut();
            inner
                .signals
                .iter_mut()
                .filter_map(|(&id, slot)| {
                    if !slot.active {
                        return None;
                    }
                    slot.task = None;
                    slot.generation = next_generation(slot.generation);
                    Some((id, slot.generation, slot.signo))
                })
                .collect::<Vec<_>>()
        };
        for (id, generation, deadline) in timers {
            self.spawn_timer(handle.clone(), id, generation, deadline);
        }
        for (id, generation, fd, interest, mode) in ios {
            self.spawn_io(handle.clone(), id, generation, fd, interest, mode);
        }
        for (id, generation, signo) in signals {
            self.spawn_signal(handle.clone(), id, generation, signo);
        }
    }

    fn spawn_timer(&self, handle: TaskHandle, id: usize, generation: u64, deadline: Instant) {
        let control = self.clone();
        let task_handle = handle.clone();
        let task = handle.spawn(async move {
            sleep_until(&task_handle, deadline).await;
            control.fire_timer(id, generation);
        });
        let mut inner = self.inner.borrow_mut();
        if let Some(slot) = inner.timers.get_mut(&id) {
            if slot.generation == generation && slot.armed {
                slot.task = Some(task);
            } else {
                handle.cancel(task);
            }
        }
    }
}

fn next_generation(generation: u64) -> u64 {
    generation.wrapping_add(1).max(1)
}

fn timer_deadline(duration: Duration) -> Instant {
    let now = Instant::now();
    now.checked_add(duration).unwrap_or_else(|| {
        now.checked_add(Duration::from_secs(100 * 365 * 24 * 60 * 60))
            .expect("timer fallback deadline is representable")
    })
}

fn runtime_interest(interest: Interest) -> RtInterest {
    match interest {
        Interest::Read => RtInterest::READABLE,
        Interest::Write => RtInterest::WRITABLE,
        Interest::ReadWrite => RtInterest::READABLE | RtInterest::WRITABLE,
    }
}

fn readiness_flags(readiness: Readiness, interest: Interest) -> c_short {
    let mut flags = 0;
    if readiness.is_readable() || readiness.intersects(Readiness::READ_CLOSED) {
        flags |= EV_READ;
    }
    if readiness.is_writable() || readiness.intersects(Readiness::WRITE_CLOSED) {
        flags |= EV_WRITE;
    }
    if flags == 0 {
        match interest {
            Interest::Read => EV_READ,
            Interest::Write => EV_WRITE,
            Interest::ReadWrite => EV_READ | EV_WRITE,
        }
    } else {
        flags
    }
}

#[cfg(test)]
#[path = "registry_tests.rs"]
mod tests;
