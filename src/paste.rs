use crate::ffi::time;
use crate::fmt_args;
use crate::notify::notify_paste_buffer;
use crate::options::OptionsRef;

use crate::text::{RustUtf8VisModel, Utf8VisModel};
use crate::tmux::{clean_name, global_options};
use crate::types::{time_t, u_int};
use crate::xmalloc::xasprintf;
use core::cell::RefCell;
use core::cmp::Reverse;
use core::ffi::{CStr, c_int};
use core::ptr::null_mut;
use std::collections::BTreeMap;
use std::ffi::CString;

const VIS_OCTAL: c_int = 0x1;
const VIS_CSTYLE: c_int = 0x2;
const VIS_TAB: c_int = 0x8;
const VIS_NL: c_int = 0x10;

/// An immutable observation of one paste buffer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PasteBufferRef<'a> {
    /// The name under which the buffer is stored.
    pub name: &'a CStr,
    /// The buffer bytes, which may contain NUL and need not be UTF-8.
    pub data: &'a [u8],
    /// The wall-clock time at which the buffer was created.
    pub created: time_t,
    /// The buffer's monotonically increasing creation order.
    pub order: u_int,
}

/// A store of named and automatically named tmux paste buffers.
pub trait PasteBufferStore {
    /// Walks all buffers newest first.
    fn buffers(&self) -> impl Iterator<Item = PasteBufferRef<'_>>;

    /// Returns whether the store has no buffers.
    fn is_empty(&self) -> bool;

    /// Finds the newest automatic buffer.
    fn top(&self) -> Option<PasteBufferRef<'_>>;

    /// Finds the buffer filed under a nonempty name.
    fn get(&self, name: &CStr) -> Option<PasteBufferRef<'_>>;

    /// Adds an automatically named buffer and enforces `automatic_limit`.
    fn add_automatic(&mut self, prefix: Option<&CStr>, data: Vec<u8>, automatic_limit: u_int);

    /// Adds or replaces a named buffer.
    fn set_named(&mut self, name: &CStr, data: Vec<u8>) -> Result<(), CString>;

    /// Renames a buffer and makes it a named rather than automatic buffer.
    fn rename(&mut self, old_name: &CStr, new_name: &CStr) -> Result<(), CString>;

    /// Removes a named buffer, returning whether it existed.
    fn remove(&mut self, name: &CStr) -> bool;

    /// Replaces only a named buffer's data, returning whether it existed.
    fn replace(&mut self, name: &CStr, data: Vec<u8>) -> bool;

    /// Returns the escaped display sample for a named buffer.
    fn sample(&self, name: &CStr) -> Option<CString>;
}

#[derive(Default)]
struct RustPasteBuffer {
    data: Vec<u8>,
    name: CString,
    created: time_t,
    automatic: bool,
    order: u_int,
}

enum PasteBufferEvent {
    Changed(CString),
    Deleted(CString),
}

/// The Rust paste-buffer store used by hmux.
pub struct RustPasteBufferStore {
    next_index: u_int,
    next_order: u_int,
    num_automatic: u_int,
    by_time: BTreeMap<Reverse<u_int>, CString>,
    by_name: BTreeMap<CString, RustPasteBuffer>,
    record_events: bool,
    events: Vec<PasteBufferEvent>,
}

impl RustPasteBufferStore {
    /// Makes an independent empty store with no server notifications.
    pub fn new() -> Self {
        Self {
            next_index: 0,
            next_order: 0,
            num_automatic: 0,
            by_time: BTreeMap::new(),
            by_name: BTreeMap::new(),
            record_events: false,
            events: Vec::new(),
        }
    }

    pub(crate) const fn server() -> Self {
        Self {
            next_index: 0,
            next_order: 0,
            num_automatic: 0,
            by_time: BTreeMap::new(),
            by_name: BTreeMap::new(),
            record_events: true,
            events: Vec::new(),
        }
    }

    fn observe(buffer: &RustPasteBuffer) -> PasteBufferRef<'_> {
        PasteBufferRef {
            name: buffer.name.as_c_str(),
            data: &buffer.data,
            created: buffer.created,
            order: buffer.order,
        }
    }

    fn changed(&mut self, name: CString) {
        if self.record_events {
            self.events.push(PasteBufferEvent::Changed(name));
        }
    }

    fn deleted(&mut self, name: CString) {
        if self.record_events {
            self.events.push(PasteBufferEvent::Deleted(name));
        }
    }

    fn remove_existing(&mut self, name: &CStr) -> bool {
        let Some(buffer) = self.by_name.remove(name) else {
            return false;
        };
        self.by_time.remove(&Reverse(buffer.order));
        if buffer.automatic {
            self.num_automatic = self.num_automatic.wrapping_sub(1);
        }
        self.deleted(buffer.name);
        true
    }

    fn next_automatic_name(&mut self, prefix: &CStr) -> CString {
        loop {
            let mut name = prefix.to_bytes().to_vec();
            name.extend_from_slice(self.next_index.to_string().as_bytes());
            self.next_index = self.next_index.wrapping_add(1);
            let name = CString::new(name).expect("a C string prefix has no NUL");
            if !self.by_name.contains_key(name.as_c_str()) {
                return name;
            }
        }
    }

    fn insert(&mut self, name: CString, data: Vec<u8>, automatic: bool) {
        let order = self.next_order;
        self.next_order = self.next_order.wrapping_add(1);
        let buffer = RustPasteBuffer {
            data,
            name: name.clone(),
            created: unsafe { time(null_mut::<time_t>()) },
            automatic,
            order,
        };
        if automatic {
            self.num_automatic = self.num_automatic.wrapping_add(1);
        }
        self.by_time.insert(Reverse(order), name.clone());
        self.by_name.insert(name.clone(), buffer);
        self.changed(name);
    }

    fn take_events(&mut self) -> Vec<PasteBufferEvent> {
        core::mem::take(&mut self.events)
    }
}

impl Default for RustPasteBufferStore {
    fn default() -> Self {
        Self::new()
    }
}

impl PasteBufferStore for RustPasteBufferStore {
    fn buffers(&self) -> impl Iterator<Item = PasteBufferRef<'_>> {
        self.by_time
            .values()
            .filter_map(|name| self.by_name.get(name))
            .map(Self::observe)
    }

    fn is_empty(&self) -> bool {
        self.by_time.is_empty()
    }

    fn top(&self) -> Option<PasteBufferRef<'_>> {
        self.buffers().find(|buffer| {
            self.by_name
                .get(buffer.name)
                .is_some_and(|buffer| buffer.automatic)
        })
    }

    fn get(&self, name: &CStr) -> Option<PasteBufferRef<'_>> {
        if name.is_empty() {
            return None;
        }
        self.by_name.get(name).map(Self::observe)
    }

    fn add_automatic(&mut self, prefix: Option<&CStr>, data: Vec<u8>, automatic_limit: u_int) {
        if data.is_empty() {
            return;
        }
        let oldest = self.by_time.values().rev().cloned().collect::<Vec<_>>();
        for name in oldest {
            if self.num_automatic < automatic_limit {
                break;
            }
            if self
                .by_name
                .get(name.as_c_str())
                .is_some_and(|buffer| buffer.automatic)
            {
                self.remove_existing(name.as_c_str());
            }
        }
        let name = self.next_automatic_name(prefix.unwrap_or(c"buffer"));
        self.insert(name, data, true);
    }

    fn set_named(&mut self, name: &CStr, data: Vec<u8>) -> Result<(), CString> {
        if data.is_empty() {
            return Ok(());
        }
        if name.is_empty() {
            return Err(c"empty buffer name".to_owned());
        }
        let Some(name) = (unsafe { clean_name(name, 0) }) else {
            return Err(xasprintf(
                c"invalid buffer name: %s",
                fmt_args![name.as_ptr()],
            ));
        };
        self.remove_existing(name.as_c_str());
        self.insert(name, data, false);
        Ok(())
    }

    fn rename(&mut self, old_name: &CStr, new_name: &CStr) -> Result<(), CString> {
        if old_name.is_empty() {
            return Err(c"no buffer".to_owned());
        }
        if new_name.is_empty() {
            return Err(c"new name is empty".to_owned());
        }
        let Some(new_name) = (unsafe { clean_name(new_name, 0) }) else {
            return Err(xasprintf(
                c"invalid buffer name: %s",
                fmt_args![new_name.as_ptr()],
            ));
        };
        if !self.by_name.contains_key(old_name) {
            return Err(xasprintf(c"no buffer %s", fmt_args![old_name.as_ptr()]));
        }
        if old_name == new_name.as_c_str() {
            return Ok(());
        }
        self.remove_existing(new_name.as_c_str());
        let mut buffer = self
            .by_name
            .remove(old_name)
            .expect("the old paste buffer was just found");
        if buffer.automatic {
            self.num_automatic = self.num_automatic.wrapping_sub(1);
        }
        buffer.automatic = false;
        buffer.name = new_name.clone();
        self.by_time.insert(Reverse(buffer.order), new_name.clone());
        self.by_name.insert(new_name.clone(), buffer);
        self.deleted(old_name.to_owned());
        self.changed(new_name);
        Ok(())
    }

    fn remove(&mut self, name: &CStr) -> bool {
        self.remove_existing(name)
    }

    fn replace(&mut self, name: &CStr, data: Vec<u8>) -> bool {
        let Some(buffer) = self.by_name.get_mut(name) else {
            return false;
        };
        buffer.data = data;
        let name = buffer.name.clone();
        self.changed(name);
        true
    }

    fn sample(&self, name: &CStr) -> Option<CString> {
        self.by_name
            .get(name)
            .map(|buffer| make_sample(&buffer.data))
    }
}

const PASTE_BUFFERS: crate::server_state::LocalField<RefCell<RustPasteBufferStore>> =
    crate::server_state::LocalField::new(|state| &state.paste_buffers);

/// Runs `read` with shared access to this thread's server paste-buffer store.
pub(crate) fn with_paste_buffers<R>(read: impl FnOnce(&RustPasteBufferStore) -> R) -> R {
    PASTE_BUFFERS.with_borrow(read)
}

/// Runs `mutate` exclusively, then dispatches notifications after releasing
/// this thread's store borrow so observers may revisit the committed state.
pub(crate) fn with_paste_buffers_mut<R>(mutate: impl FnOnce(&mut RustPasteBufferStore) -> R) -> R {
    let (result, events) = PASTE_BUFFERS.with_borrow_mut(|store| {
        let result = mutate(store);
        let events = store.take_events();
        (result, events)
    });
    for event in events {
        unsafe {
            match event {
                PasteBufferEvent::Changed(name) => notify_paste_buffer(name.as_c_str(), 0),
                PasteBufferEvent::Deleted(name) => notify_paste_buffer(name.as_c_str(), 1),
            }
        }
    }
    result
}

/// Returns the live automatic-buffer limit.
pub(crate) unsafe fn paste_buffer_limit() -> u_int {
    {
        (global_options
            .get()
            .as_ref()
            .expect("global options are initialized"))
        .number(c"buffer-limit") as u_int
    }
}

fn make_sample(data: &[u8]) -> CString {
    const FLAGS: c_int = VIS_OCTAL | VIS_CSTYLE | VIS_TAB | VIS_NL;
    const WIDTH: usize = 200;

    let len = core::cmp::min(data.len(), WIDTH);
    let mut sample = RustUtf8VisModel
        .encode_utf8(&data[..len], FLAGS)
        .into_bytes();
    if data.len() > WIDTH || sample.len() > WIDTH {
        sample.truncate(WIDTH);
        sample.extend_from_slice(b"...");
    }
    unsafe { CString::from_vec_unchecked(sample) }
}

#[cfg(test)]
#[path = "tests/test_paste_adapter.rs"]
mod tests;
