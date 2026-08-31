use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{c_int, c_short};
use std::os::fd::BorrowedFd;
use std::rc::Rc;

use hmux_rt::{AsyncFd, Interest as RtInterest, Readiness, TaskHandle, TaskId};

use super::notify::{Notify, Select2, SelectResult, yield_now};
use super::{Interest, StreamCb, StreamErrorCb};
use crate::reactor::buffer::Buf;
use crate::types::size_t;

pub const STREAM_EVENT_READING: c_short = 0x01;
pub const STREAM_EVENT_WRITING: c_short = 0x02;
pub const STREAM_EVENT_EOF: c_short = 0x10;
pub const STREAM_EVENT_ERROR: c_short = 0x20;

const STREAM_IO_BUDGET: usize = 64;
const READ_CHUNK: usize = 64 * 1024;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
#[repr(transparent)]
pub struct Stream(pub(crate) usize);

impl Stream {
    pub const NONE: Self = Self(0);

    pub fn is_none(&self) -> bool {
        self.0 == 0
    }

    pub fn new(
        fd: c_int,
        read: Option<StreamCb>,
        write: Option<StreamCb>,
        error: Option<StreamErrorCb>,
    ) -> Self {
        Self(current_stream_registry().allocate(fd, read, write, error))
    }

    pub fn free(self) {
        if self.0 != 0 {
            current_stream_registry().free(self.0);
        }
    }

    pub fn input_len(&self) -> usize {
        if self.0 == 0 {
            0
        } else {
            current_stream_registry().input_len(self.0)
        }
    }

    pub fn output_len(&self) -> usize {
        if self.0 == 0 {
            0
        } else {
            current_stream_registry().output_len(self.0)
        }
    }

    pub fn with_input<R>(&self, callback: impl FnOnce(&mut Buf) -> R) -> Option<R> {
        if self.0 == 0 {
            None
        } else {
            current_stream_registry().with_input(self.0, callback)
        }
    }

    pub fn with_output<R>(&self, callback: impl FnOnce(&mut Buf) -> R) -> Option<R> {
        if self.0 == 0 {
            None
        } else {
            current_stream_registry().with_output(self.0, callback)
        }
    }

    pub unsafe fn write(&self, data: *const u8, len: size_t) -> c_int {
        if self.0 == 0 || (data.is_null() && len != 0) {
            return -1;
        }
        if len == 0 {
            return 0;
        }
        let bytes = unsafe { std::slice::from_raw_parts(data, len) };
        if current_stream_registry().write(self.0, bytes) {
            0
        } else {
            -1
        }
    }

    pub fn write_buffer(&self, buffer: &mut Buf) -> c_int {
        if self.0 != 0 && current_stream_registry().write_buffer(self.0, buffer) {
            0
        } else {
            -1
        }
    }

    pub fn enable(&self, interest: Interest) {
        if self.0 != 0 {
            current_stream_registry().enable(self.0, interest);
        }
    }

    pub fn disable(&self, interest: Interest) {
        if self.0 != 0 {
            current_stream_registry().disable(self.0, interest);
        }
    }

    pub fn set_write_watermark(&self, low: size_t, high: size_t) {
        if self.0 != 0 {
            current_stream_registry().set_watermark(self.0, low, high);
        }
    }
}

struct StreamState {
    generation: u64,
    fd: c_int,
    read_callback: Option<StreamCb>,
    write_callback: Option<StreamCb>,
    error_callback: Option<StreamErrorCb>,
    read_enabled: bool,
    write_enabled: bool,
    input: Buf,
    output: Buf,
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
pub(crate) struct StreamRegistry {
    inner: Rc<RefCell<StreamRegistryInner>>,
}

impl StreamRegistry {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Drops every stream the registry is holding, for the same reason
    /// [`RuntimeControl::shutdown`] drops every callback.
    pub(crate) fn shutdown(&self) {
        loop {
            let streams = std::mem::take(&mut self.inner.borrow_mut().streams);
            if streams.is_empty() {
                break;
            }
            drop(streams);
        }
    }

    fn allocate(
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
            input: Buf::new(),
            output: Buf::new(),
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

    fn free(&self, id: usize) {
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

    fn input_len(&self, id: usize) -> usize {
        self.lookup(id)
            .map(|state| state.borrow().input.len())
            .unwrap_or(0)
    }

    fn output_len(&self, id: usize) -> usize {
        self.lookup(id)
            .map(|state| state.borrow().output.len())
            .unwrap_or(0)
    }

    fn with_input<R>(&self, id: usize, callback: impl FnOnce(&mut Buf) -> R) -> Option<R> {
        let state = self.lookup(id)?;
        Some(callback(&mut state.borrow_mut().input))
    }

    fn with_output<R>(&self, id: usize, callback: impl FnOnce(&mut Buf) -> R) -> Option<R> {
        let state = self.lookup(id)?;
        Some(callback(&mut state.borrow_mut().output))
    }

    fn write(&self, id: usize, bytes: &[u8]) -> bool {
        let Some(state) = self.lookup(id) else {
            return false;
        };
        let notify = {
            let mut state = state.borrow_mut();
            if state.closed {
                return false;
            }
            let before = state.output.len();
            state.output.append(bytes);
            if before <= state.low_watermark && state.output.len() > state.low_watermark {
                state.low_notification_armed = true;
            }
            state.notify.clone()
        };
        notify.notify();
        true
    }

    fn write_buffer(&self, id: usize, buffer: &mut Buf) -> bool {
        let Some(state) = self.lookup(id) else {
            return false;
        };
        let notify = {
            let mut state = state.borrow_mut();
            if state.closed {
                return false;
            }
            let before = state.output.len();
            state.output.append_buf(buffer);
            if before <= state.low_watermark && state.output.len() > state.low_watermark {
                state.low_notification_armed = true;
            }
            state.notify.clone()
        };
        notify.notify();
        true
    }

    fn enable(&self, id: usize, interest: Interest) {
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

    fn disable(&self, id: usize, interest: Interest) {
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

    fn set_watermark(&self, id: usize, low: usize, high: usize) {
        let Some(state) = self.lookup(id) else {
            return;
        };
        let mut state = state.borrow_mut();
        state.low_watermark = low;
        state.high_watermark = high;
        state.low_notification_armed = state.output.len() > low;
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

    pub(crate) fn respawn_active(&self, handle: &TaskHandle) {
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
                state.read_enabled,
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
            descriptor = None;
            let raw = unsafe { BorrowedFd::borrow_raw(fd) };
            match AsyncFd::new(&task_handle, raw, wanted) {
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
        let (mut read_enabled, mut write_enabled, mut has_output) = {
            let state = state.borrow();
            if state.closed || state.generation != generation {
                return;
            }
            (
                state.read_enabled,
                state.write_enabled,
                !state.output.is_empty(),
            )
        };
        if let Some(callback) = take_pending_write_callback(&state, generation) {
            callback(Stream(id));
            if !is_current(&state, generation) {
                return;
            }
        }
        let Some((new_read_enabled, new_write_enabled, new_has_output)) =
            current_stream_flags(&state, generation)
        else {
            return;
        };
        read_enabled = new_read_enabled;
        write_enabled = new_write_enabled;
        has_output = new_has_output;
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
            let Some((new_read_enabled, new_write_enabled, _)) =
                current_stream_flags(&state, generation)
            else {
                return;
            };
            read_enabled = new_read_enabled;
            write_enabled = new_write_enabled;
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
            state.input.read_from_fd(fd, READ_CHUNK)
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
            state.output.write_to_fd(fd)
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

fn current_stream_registry() -> StreamRegistry {
    super::runtime::stream_registry()
}

#[cfg(test)]
#[path = "../tests/test_reactor_stream.rs"]
mod tests;
