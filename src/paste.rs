use crate::ffi::time;
use crate::fmt_args;
use crate::notify::notify_paste_buffer;
use crate::options::options_get_number;
use crate::tmux::clean_name;
use crate::tmux::global_options;
use crate::tree::GlobalTree;
pub use crate::types::*;
use crate::text::utf8_strvis;
use crate::xmalloc::xasprintf;
use ::core::cmp::Reverse;
use ::core::ffi::{CStr, c_char, c_int};
use ::core::ops::Bound;
use ::core::ptr::{null, null_mut};
use ::std::ffi::CString;

pub const RB_BLACK: c_int = 0 as c_int;
pub const RB_RED: c_int = 1 as c_int;
pub const RB_NEGINF: c_int = -(1 as c_int);
pub const RB_INF: c_int = 1 as c_int;
pub const VIS_OCTAL: c_int = 0x1 as c_int;
pub const VIS_CSTYLE: c_int = 0x2 as c_int;
pub const VIS_TAB: c_int = 0x8 as c_int;
pub const VIS_NL: c_int = 0x10 as c_int;
static mut paste_next_index: u_int = 0;
static mut paste_next_order: u_int = 0;
static mut paste_num_automatic: u_int = 0;
/// The name of every buffer by falling order, newest first. Each name is the
/// key the buffer itself is held under in [`paste_by_name`].
static paste_by_time: GlobalTree<Reverse<u_int>, CString> = GlobalTree::new();

/// One paste buffer: the bytes it holds, the name it is held under, when it
/// was made and where it sits in the falling order the store walks.
///
/// The fields are the buffer's own; outside this module a buffer is read
/// through the `paste_buffer_*` accessors and changed only by going back
/// through the store, which is what keeps the two trees below in step with
/// each other.
#[repr(C)]
pub struct paste_buffer {
    data: Vec<u8>,
    name: Option<CString>,
    created: time_t,
    automatic: c_int,
    order: u_int,
}

/// The buffers by name, which is what holds them.
static paste_by_name: GlobalTree<CString, Box<paste_buffer>> = GlobalTree::new();

/// The buffer named `name`, or null when the store has none.
fn buffer_of(name: &CStr) -> *mut paste_buffer {
    paste_by_name
        .map()
        .get(name)
        .map(|pb| &raw const **pb as *mut paste_buffer)
        .unwrap_or(null_mut::<paste_buffer>())
}

/// Every buffer the store holds, newest first.
fn newest_first() -> Vec<*mut paste_buffer> {
    paste_by_time
        .map()
        .values()
        .map(|name| buffer_of(name))
        .collect()
}

pub unsafe fn paste_buffer_name(pb: *mut paste_buffer) -> *const c_char {
    unsafe { cstr_ptr(&(*pb).name) }
}

pub unsafe fn paste_buffer_order(pb: *mut paste_buffer) -> u_int {
    unsafe { (*pb).order }
}

pub unsafe fn paste_buffer_created(pb: *mut paste_buffer) -> time_t {
    unsafe { (*pb).created }
}

/// The bytes the buffer holds, which are not necessarily a C string.
pub fn paste_buffer_data(pb: &paste_buffer) -> &[u8] {
    &pb.data
}

/// Whether the store named the buffer itself, which is what makes it one of
/// the ones `buffer-limit` counts and drops the oldest of.
pub unsafe fn paste_buffer_automatic(pb: *mut paste_buffer) -> c_int {
    unsafe { (*pb).automatic }
}

/// The next buffer by falling order, or the newest one when `pb` is null.
pub unsafe fn paste_walk(pb: *mut paste_buffer) -> *mut paste_buffer {
    unsafe {
        let tree = paste_by_time.map();
        let next = if pb.is_null() {
            tree.values().next()
        } else {
            tree.range((Bound::Excluded(Reverse((*pb).order)), Bound::Unbounded))
                .next()
                .map(|(_, name)| name)
        };
        next.map(|name| buffer_of(name))
            .unwrap_or(null_mut::<paste_buffer>())
    }
}

pub fn paste_is_empty() -> c_int {
    paste_by_time.map().is_empty() as c_int
}

/// The newest automatic buffer, and a copy of its name through `name` if the
/// caller wants one.
pub unsafe fn paste_get_top(name: *mut Option<CString>) -> *mut paste_buffer {
    unsafe {
        let found = newest_first().into_iter().find(|&pb| (*pb).automatic != 0);
        let Some(pb) = found else {
            return null_mut::<paste_buffer>();
        };
        if !name.is_null() {
            *name = Some(CStr::from_ptr(cstr_ptr(&(*pb).name)).to_owned());
        }
        pb
    }
}

pub unsafe fn paste_get_name(name: *const c_char) -> *mut paste_buffer {
    unsafe {
        if name.is_null() || *name == 0 {
            return null_mut::<paste_buffer>();
        }
        buffer_of(CStr::from_ptr(name))
    }
}

pub unsafe fn paste_free(pb: *mut paste_buffer) {
    unsafe {
        let name = CStr::from_ptr(cstr_ptr(&(*pb).name)).to_owned();
        let order = (*pb).order;
        let automatic = (*pb).automatic != 0;
        notify_paste_buffer(name.as_ptr(), 1);
        paste_by_time.map().remove(&Reverse(order));
        let _ = paste_by_name.map().remove(&name);
        if automatic {
            paste_num_automatic = paste_num_automatic.wrapping_sub(1);
        }
    }
}

/// Adds an automatic buffer named after `prefix`, freeing the oldest automatic
/// buffers first if the store is at `buffer-limit`. The buffer takes `data`
/// over.
pub unsafe fn paste_add(prefix: *const c_char, data: Vec<u8>) {
    unsafe {
        let prefix = if prefix.is_null() {
            c"buffer".as_ptr()
        } else {
            prefix
        };
        if data.is_empty() {
            return;
        }

        let limit = options_get_number(global_options, c"buffer-limit".as_ptr()) as u_int;
        let mut oldest_first = newest_first();
        oldest_first.reverse();
        for pb in oldest_first {
            if paste_num_automatic < limit {
                break;
            }
            if (*pb).automatic != 0 {
                paste_free(pb);
            }
        }

        let mut pb = Box::new(paste_buffer {
            data: Vec::new(),
            name: None,
            created: 0,
            automatic: 0,
            order: 0,
        });
        let pb_ptr = &raw mut *pb;
        loop {
            (*pb_ptr).name = Some(xasprintf(
                c"%s%u".as_ptr(),
                fmt_args![prefix, paste_next_index],
            ));
            paste_next_index = paste_next_index.wrapping_add(1);
            if paste_get_name(cstr_ptr(&(*pb_ptr).name)).is_null() {
                break;
            }
        }

        (*pb_ptr).data = data;
        (*pb_ptr).automatic = 1;
        paste_num_automatic = paste_num_automatic.wrapping_add(1);
        (*pb_ptr).created = time(null_mut::<time_t>());
        (*pb_ptr).order = paste_next_order;
        paste_next_order = paste_next_order.wrapping_add(1);
        let name = CStr::from_ptr(cstr_ptr(&(*pb_ptr).name)).to_owned();
        let order = (*pb_ptr).order;
        paste_by_name.map().insert(name.clone(), pb);
        paste_by_time.map().insert(Reverse(order), name.clone());
        notify_paste_buffer(name.as_ptr(), 0);
    }
}

/// Renames the buffer `oldname` to `newname`, which also makes it one the user
/// named rather than an automatic one. Renaming a buffer to the name it
/// already has stops at the "same buffer" check, so it stays automatic.
pub unsafe fn paste_rename(oldname: *const c_char, newname: *const c_char) -> Result<(), CString> {
    unsafe {
        if oldname.is_null() || *oldname == 0 {
            return Err(c"no buffer".to_owned());
        }
        if newname.is_null() || *newname == 0 {
            return Err(c"new name is empty".to_owned());
        }

        let Some(name) = clean_name(newname, 0) else {
            return Err(xasprintf(
                c"invalid buffer name: %s".as_ptr(),
                fmt_args![newname],
            ));
        };

        let pb = paste_get_name(oldname);
        if pb.is_null() {
            return Err(xasprintf(c"no buffer %s".as_ptr(), fmt_args![oldname]));
        }

        let pb_new = paste_get_name(name.as_ptr());
        if pb_new == pb {
            return Ok(());
        }
        if !pb_new.is_null() {
            paste_free(pb_new);
        }

        let old_name = CStr::from_ptr(cstr_ptr(&(*pb).name)).to_owned();
        let mut pb_box = paste_by_name
            .map()
            .remove(&old_name)
            .expect("paste buffer is indexed by its name");
        let pb = &raw mut *pb_box;
        (*pb).name = Some(name);
        if (*pb).automatic != 0 {
            paste_num_automatic = paste_num_automatic.wrapping_sub(1);
        }
        (*pb).automatic = 0;
        let new_name = CStr::from_ptr(cstr_ptr(&(*pb).name)).to_owned();
        paste_by_name.map().insert(new_name.clone(), pb_box);
        paste_by_time
            .map()
            .insert(Reverse((*pb).order), new_name.clone());

        notify_paste_buffer(oldname, 1);
        notify_paste_buffer(new_name.as_ptr(), 0);
        Ok(())
    }
}

/// Adds a buffer under `name`, replacing whatever was there, or an automatic
/// one when there is no name. The buffer takes `data` over.
pub unsafe fn paste_set(data: Vec<u8>, name: *const c_char) -> Result<(), CString> {
    unsafe {
        if data.is_empty() {
            return Ok(());
        }
        if name.is_null() {
            paste_add(null::<c_char>(), data);
            return Ok(());
        }
        if *name == 0 {
            return Err(c"empty buffer name".to_owned());
        }

        let Some(newname) = clean_name(name, 0) else {
            return Err(xasprintf(
                c"invalid buffer name: %s".as_ptr(),
                fmt_args![name],
            ));
        };

        let mut pb = Box::new(paste_buffer {
            data,
            name: Some(newname),
            created: 0,
            automatic: 0,
            order: 0,
        });
        let pb_ptr = &raw mut *pb;
        (*pb_ptr).order = paste_next_order;
        paste_next_order = paste_next_order.wrapping_add(1);
        (*pb_ptr).created = time(null_mut::<time_t>());

        let old = paste_get_name(cstr_ptr(&(*pb_ptr).name));
        if !old.is_null() {
            paste_free(old);
        }

        let name = CStr::from_ptr(cstr_ptr(&(*pb_ptr).name)).to_owned();
        let order = (*pb_ptr).order;
        paste_by_name.map().insert(name.clone(), pb);
        paste_by_time.map().insert(Reverse(order), name.clone());
        notify_paste_buffer(name.as_ptr(), 0);
        Ok(())
    }
}

/// Swaps a buffer's data for `data`.
pub unsafe fn paste_replace(pb: *mut paste_buffer, data: Vec<u8>) {
    unsafe {
        (*pb).data = data;
        notify_paste_buffer(cstr_ptr(&(*pb).name), 0);
    }
}

/// The first two hundred characters of a buffer, escaped for display, with
/// trailing dots when there was more. The dots are written *at* the two
/// hundredth character rather than after the escaping stopped, so an escaped
/// form that ran long keeps whatever it wrote past there.
pub unsafe fn paste_make_sample(pb: *mut paste_buffer) -> CString {
    unsafe {
        const FLAGS: c_int = VIS_OCTAL | VIS_CSTYLE | VIS_TAB | VIS_NL;
        const WIDTH: size_t = 200;

        let len = ::core::cmp::min((*pb).data.len() as size_t, WIDTH);
        let mut buf = vec![0u8; len * 4 + 4];
        let used = utf8_strvis(
            buf.as_mut_ptr() as *mut c_char,
            (*pb).data.as_ptr() as *const c_char,
            len,
            FLAGS,
        );
        if (*pb).data.len() as size_t > WIDTH || used > WIDTH {
            let dots = b"...";
            let start = WIDTH;
            buf[start..start + 3].copy_from_slice(dots);
            buf[start + 3] = 0;
        }
        CStr::from_ptr(buf.as_ptr() as *const c_char).to_owned()
    }
}

#[cfg(test)]
#[path = "tests/test_paste.rs"]
mod tests;
