//! Buffered streams driven by runtime tasks. Callbacks run outside state borrows.
//! Descriptors are borrowed: free the stream before closing or reusing its fd.
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{c_int, c_short};
use std::rc::Rc;

use crate::{Interest as RtInterest, Readiness, TaskHandle, TaskId};

use super::adapters::WatchInterest as Interest;
use super::notify::{Notify, Select2, SelectResult, yield_now};
pub type StreamCb = Rc<dyn Fn(Stream)>;
pub type StreamErrorCb = Rc<dyn Fn(Stream, c_short)>;
use crate::ByteBuffer;

pub const STREAM_EVENT_READING: c_short = 0x01;
pub const STREAM_EVENT_WRITING: c_short = 0x02;
pub const STREAM_EVENT_EOF: c_short = 0x10;
pub const STREAM_EVENT_ERROR: c_short = 0x20;

const STREAM_IO_BUDGET: usize = 64;
const READ_CHUNK: usize = 64 * 1024;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
#[repr(transparent)]
pub struct Stream(pub usize);

struct StreamState {
    generation: u64,
    fd: c_int,
    read_callback: Option<StreamCb>,
    write_callback: Option<StreamCb>,
    error_callback: Option<StreamErrorCb>,
    read_enabled: bool,
    write_enabled: bool,
    input: Box<ByteBuffer>,
    output: Box<ByteBuffer>,
    read_low: usize,
    read_high: usize,
    low_watermark: usize,
    high_watermark: usize,
    low_notification_armed: bool,
    write_callback_pending: bool,
    notify: Notify,
    task: Option<TaskId>,
    closed: bool,
}

type SharedStream = Rc<RefCell<StreamState>>;

#[derive(Default)]
struct StreamRegistryInner {
    next_id: usize,
    streams: HashMap<usize, SharedStream>,
    task_handle: Option<TaskHandle>,
}

#[derive(Clone, Default)]
pub struct StreamRegistry {
    inner: Rc<RefCell<StreamRegistryInner>>,
}

impl StreamRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Invalidate every stream and cancel its task before releasing storage.
    pub fn shutdown(&self) {
        let ids = self
            .inner
            .borrow()
            .streams
            .keys()
            .copied()
            .collect::<Vec<_>>();
        for id in ids {
            self.free(id);
        }
    }

    pub fn allocate(
        &self,
        fd: c_int,
        read_callback: Option<StreamCb>,
        write_callback: Option<StreamCb>,
        error_callback: Option<StreamErrorCb>,
    ) -> usize {
        let state = Rc::new(RefCell::new(StreamState {
            generation: 1,
            fd,
            read_callback,
            write_callback,
            error_callback,
            read_enabled: false,
            write_enabled: false,
            input: Box::default(),
            output: Box::default(),
            read_low: 0,
            read_high: 0,
            low_watermark: 0,
            high_watermark: 0,
            low_notification_armed: false,
            write_callback_pending: false,
            notify: Notify::new(),
            task: None,
            closed: false,
        }));
        let (id, handle) = {
            let mut inner = self.inner.borrow_mut();
            assert_ne!(inner.next_id, usize::MAX, "stream handle space exhausted");
            inner.next_id += 1;
            let id = inner.next_id;
            inner.streams.insert(id, Rc::clone(&state));
            (id, inner.task_handle.clone())
        };
        if let Some(handle) = handle {
            self.spawn(handle, id, state, 1);
        }
        id
    }

    fn lookup(&self, id: usize) -> Option<SharedStream> {
        self.inner.borrow().streams.get(&id).cloned()
    }

    pub fn free(&self, id: usize) {
        let (state, handle) = {
            let mut inner = self.inner.borrow_mut();
            (inner.streams.remove(&id), inner.task_handle.clone())
        };
        let Some(state) = state else {
            return;
        };
        let task = {
            let mut state = state.borrow_mut();
            state.closed = true;
            state.generation = next_generation(state.generation);
            state.task.take()
        };
        if let (Some(handle), Some(task)) = (handle, task) {
            handle.cancel(task);
        }
    }

    pub fn input_len(&self, id: usize) -> usize {
        self.lookup(id)
            .map(|state| state.borrow().input.len())
            .unwrap_or(0)
    }

    pub fn output_len(&self, id: usize) -> usize {
        self.lookup(id)
            .map(|state| state.borrow().output.len())
            .unwrap_or(0)
    }

    pub fn with_input<R>(
        &self,
        id: usize,
        callback: impl FnOnce(&mut ByteBuffer) -> R,
    ) -> Option<R> {
        let state = self.lookup(id)?;
        Some(callback(&mut state.borrow_mut().input))
    }

    pub fn with_output<R>(
        &self,
        id: usize,
        callback: impl FnOnce(&mut ByteBuffer) -> R,
    ) -> Option<R> {
        let state = self.lookup(id)?;
        Some(callback(&mut state.borrow_mut().output))
    }

    pub fn write(&self, id: usize, bytes: &[u8]) -> bool {
        let Some(state) = self.lookup(id) else {
            return false;
        };
        let notify = {
            let mut state = state.borrow_mut();
            if state.closed {
                return false;
            }
            state.output.append(bytes);
            state.low_notification_armed |= !state.output.is_empty();
            state.notify.clone()
        };
        notify.notify();
        true
    }

    pub fn write_buffer(&self, id: usize, buffer: &mut ByteBuffer) -> bool {
        let Some(state) = self.lookup(id) else {
            return false;
        };
        let notify = {
            let mut state = state.borrow_mut();
            if state.closed {
                return false;
            }
            state.output.transfer(buffer);
            state.low_notification_armed |= !state.output.is_empty();
            state.notify.clone()
        };
        notify.notify();
        true
    }

    pub fn enable(&self, id: usize, interest: Interest) {
        let Some(state) = self.lookup(id) else {
            return;
        };
        let notify = {
            let mut state = state.borrow_mut();
            set_interest(&mut state, interest, true);
            if matches!(interest, Interest::Write | Interest::ReadWrite)
                && state.write_enabled
                && state.write_callback.is_some()
            {
                state.write_callback_pending = true;
            }
            state.notify.clone()
        };
        notify.notify();
    }

    pub fn disable(&self, id: usize, interest: Interest) {
        let Some(state) = self.lookup(id) else {
            return;
        };
        let notify = {
            let mut state = state.borrow_mut();
            set_interest(&mut state, interest, false);
            state.notify.clone()
        };
        notify.notify();
    }

    pub fn wake(&self, id: usize) {
        if let Some(state) = self.lookup(id) {
            let notify = {
                let mut s = state.borrow_mut();
                s.low_notification_armed |= !s.output.is_empty();
                s.notify.clone()
            };
            notify.notify();
        }
    }

    pub fn set_read_watermark(&self, id: usize, low: usize, high: usize) {
        if let Some(state) = self.lookup(id) {
            let notify = {
                let mut s = state.borrow_mut();
                s.read_low = low;
                s.read_high = high;
                s.notify.clone()
            };
            notify.notify();
        }
    }

    pub fn set_watermark(&self, id: usize, low: usize, high: usize) {
        let Some(state) = self.lookup(id) else {
            return;
        };
        let mut state = state.borrow_mut();
        state.low_watermark = low;
        state.high_watermark = high;
        state.low_notification_armed = !state.output.is_empty();
    }

    fn spawn(&self, handle: TaskHandle, id: usize, state: SharedStream, generation: u64) {
        let task_handle = handle.clone();
        let task = handle.spawn(async move {
            run_stream(task_handle, id, state, generation).await;
        });
        if let Some(state) = self.lookup(id) {
            let mut state = state.borrow_mut();
            if state.generation == generation && !state.closed {
                state.task = Some(task);
                return;
            }
        }
        handle.cancel(task);
    }

    pub fn respawn_active(&self, handle: &TaskHandle) {
        self.inner.borrow_mut().task_handle = Some(handle.clone());
        let streams = self
            .inner
            .borrow()
            .streams
            .iter()
            .map(|(&id, state)| (id, Rc::clone(state)))
            .collect::<Vec<_>>();
        for (id, state) in streams {
            let generation = {
                let mut state = state.borrow_mut();
                if state.closed {
                    continue;
                }
                state.task = None;
                state.generation = next_generation(state.generation);
                state.generation
            };
            self.spawn(handle.clone(), id, state, generation);
        }
    }
}

async fn run_stream(task_handle: TaskHandle, id: usize, state: SharedStream, generation: u64) {
    let mut descriptor = None;
    let mut registered = None;
    loop {
        let (fd, read_enabled, write_enabled, has_output, notify) = {
            let state = state.borrow();
            if state.closed || state.generation != generation {
                return;
            }
            (
                state.fd,
                state.read_enabled && (state.read_high == 0 || state.input.len() < state.read_high),
                state.write_enabled,
                !state.output.is_empty(),
                state.notify.clone(),
            )
        };
        let wanted = match (read_enabled, write_enabled && has_output) {
            (false, false) => None,
            (true, false) => Some(RtInterest::READABLE),
            (false, true) => Some(RtInterest::WRITABLE),
            (true, true) => Some(RtInterest::READABLE | RtInterest::WRITABLE),
        };
        if fd < 0 {
            descriptor = None;
            registered = None;
            notify.notified().await;
            continue;
        }
        let Some(wanted) = wanted else {
            descriptor = None;
            registered = None;
            notify.notified().await;
            if let Some(callback) = take_pending_write_callback(&state, generation) {
                callback(Stream(id));
                if !is_current(&state, generation) {
                    return;
                }
            }
            continue;
        };
        if registered != Some(wanted) {
            drop(descriptor.take());
            match super::adapters::register_fd(&task_handle, fd, wanted) {
                Ok(async_fd) => {
                    descriptor = Some(async_fd);
                    registered = Some(wanted);
                }
                Err(_) => {
                    report_error(
                        id,
                        &state,
                        generation,
                        STREAM_EVENT_ERROR | STREAM_EVENT_READING | STREAM_EVENT_WRITING,
                    );
                    return;
                }
            }
        }

        let outcome = {
            let async_fd = descriptor.as_ref().expect("descriptor created");
            Select2::new(async_fd.readiness(), notify.notified()).await
        };
        let readiness = match outcome {
            SelectResult::Left(readiness) => Some(readiness),
            SelectResult::Right(()) => None,
        };
        if !is_current(&state, generation) {
            return;
        }
        if let Some(callback) = take_pending_write_callback(&state, generation) {
            callback(Stream(id));
            if !is_current(&state, generation) {
                return;
            }
        }
        let Some((mut read_enabled, write_enabled, has_output)) =
            current_stream_flags(&state, generation)
        else {
            return;
        };
        let should_write = readiness
            .is_none_or(|value| value.is_writable() || value.intersects(Readiness::WRITE_CLOSED));
        let should_read = readiness
            .is_none_or(|value| value.is_readable() || value.intersects(Readiness::READ_CLOSED));

        if should_write && write_enabled && has_output {
            let result = write_burst(&state, id, fd, generation);
            if result.error {
                report_error(
                    id,
                    &state,
                    generation,
                    STREAM_EVENT_ERROR | STREAM_EVENT_WRITING,
                );
                return;
            }
            if !is_current(&state, generation) {
                return;
            }
            if let Some(callback) = take_pending_write_callback(&state, generation) {
                callback(Stream(id));
                if !is_current(&state, generation) {
                    return;
                }
            }
            let Some((new_read_enabled, _, _)) = current_stream_flags(&state, generation) else {
                return;
            };
            read_enabled = new_read_enabled;
            if result.budget_exhausted {
                notify.notify();
                yield_now().await;
                continue;
            }
        }

        if should_read && read_enabled {
            let result = read_burst(&state, fd, generation);
            if result.bytes != 0 {
                invoke_read(&state, id, generation, result.bytes);
                if !is_current(&state, generation) {
                    return;
                }
            }
            if result.eof {
                report_error(
                    id,
                    &state,
                    generation,
                    STREAM_EVENT_EOF | STREAM_EVENT_READING,
                );
                return;
            }
            if result.error {
                report_error(
                    id,
                    &state,
                    generation,
                    STREAM_EVENT_ERROR | STREAM_EVENT_READING,
                );
                return;
            }
            if !is_current(&state, generation) {
                return;
            }
            if result.budget_exhausted {
                notify.notify();
                yield_now().await;
            }
        }
    }
}

#[derive(Default)]
struct BurstResult {
    bytes: usize,
    eof: bool,
    error: bool,
    budget_exhausted: bool,
}

fn read_burst(state: &SharedStream, fd: c_int, generation: u64) -> BurstResult {
    let mut result = BurstResult::default();
    for operation in 0..STREAM_IO_BUDGET {
        let read = {
            let mut state = state.borrow_mut();
            if state.closed || state.generation != generation || !state.read_enabled {
                return result;
            }
            let limit = if state.read_high == 0 {
                READ_CHUNK
            } else {
                state
                    .read_high
                    .saturating_sub(state.input.len())
                    .min(READ_CHUNK)
            };
            if limit == 0 {
                return result;
            }
            unsafe { state.input.read_from_fd(fd, limit) }
        };
        match read {
            Ok(0) => {
                result.eof = true;
                break;
            }
            Ok(bytes) => result.bytes += bytes,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(error)
                if error.raw_os_error() == Some(libc::EIO)
                    || error.raw_os_error() == Some(libc::ECONNRESET) =>
            {
                result.eof = true;
                break;
            }
            Err(_) => {
                result.error = true;
                break;
            }
        }
        if operation + 1 == STREAM_IO_BUDGET {
            result.budget_exhausted = true;
        }
    }
    result
}

fn write_burst(state: &SharedStream, id: usize, fd: c_int, generation: u64) -> BurstResult {
    let mut result = BurstResult::default();
    for operation in 0..STREAM_IO_BUDGET {
        let write = {
            let mut state = state.borrow_mut();
            if state.closed || state.generation != generation || !state.write_enabled {
                return result;
            }
            if state.output.is_empty() {
                return result;
            }
            unsafe { state.output.write_to_fd(fd) }
        };
        match write {
            Ok(0) => break,
            Ok(bytes) => {
                result.bytes += bytes;
                let callback = {
                    let mut state = state.borrow_mut();
                    if state.low_notification_armed && state.output.len() <= state.low_watermark {
                        state.low_notification_armed = false;
                        state.write_callback_pending = false;
                        state.write_callback.clone()
                    } else {
                        None
                    }
                };
                if let Some(callback) = callback {
                    callback(Stream(id));
                    if !is_current(state, generation) {
                        return result;
                    }
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(_) => {
                result.error = true;
                break;
            }
        }
        if operation + 1 == STREAM_IO_BUDGET {
            result.budget_exhausted = true;
        }
    }
    result
}

fn take_pending_write_callback(state: &SharedStream, generation: u64) -> Option<StreamCb> {
    let mut state = state.borrow_mut();
    if state.closed
        || state.generation != generation
        || !state.write_enabled
        || !state.output.is_empty()
        || !state.write_callback_pending
    {
        return None;
    }
    state.write_callback_pending = false;
    state.write_callback.clone()
}

fn current_stream_flags(state: &SharedStream, generation: u64) -> Option<(bool, bool, bool)> {
    let state = state.borrow();
    if state.closed || state.generation != generation {
        return None;
    }
    Some((
        state.read_enabled,
        state.write_enabled,
        !state.output.is_empty(),
    ))
}

fn invoke_read(state: &SharedStream, id: usize, generation: u64, _bytes: usize) {
    let callback = {
        let state = state.borrow();
        if state.closed || state.generation != generation {
            return;
        }
        if state.input.len() < state.read_low {
            return;
        }
        state.read_callback.clone()
    };
    if let Some(callback) = callback {
        callback(Stream(id));
    }
}

fn report_error(id: usize, state: &SharedStream, generation: u64, what: c_short) {
    let callback = {
        let mut state = state.borrow_mut();
        if state.closed || state.generation != generation {
            return;
        }
        state.closed = true;
        state.generation = next_generation(state.generation);
        state.task = None;
        state.error_callback.clone()
    };
    if let Some(callback) = callback {
        callback(Stream(id), what);
    }
}

fn is_current(state: &SharedStream, generation: u64) -> bool {
    let state = state.borrow();
    !state.closed && state.generation == generation
}

fn set_interest(state: &mut StreamState, interest: Interest, value: bool) {
    match interest {
        Interest::Read => state.read_enabled = value,
        Interest::Write => {
            state.write_enabled = value;
            if !value {
                state.write_callback_pending = false;
            }
        }
        Interest::ReadWrite => {
            state.read_enabled = value;
            state.write_enabled = value;
            if !value {
                state.write_callback_pending = false;
            }
        }
    }
}

fn next_generation(generation: u64) -> u64 {
    generation.wrapping_add(1).max(1)
}

#[cfg(test)]
#[path = "stream_tests.rs"]
mod tests;
