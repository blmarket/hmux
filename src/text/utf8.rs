use crate::src::compat::strtonum::strtonum;
use crate::src::compat::utf8proc::{utf8proc_wctomb, utf8proc_wcwidth};
use crate::src::compat::vis::vis;
use crate::src::ffi::libc::{
    __ctype_b_loc, __errno_location, memcpy, memset, strchr, strlen, strncmp, strtoull, wctomb,
};
use crate::src::log::{fatalx, log_debug};
use crate::src::options::{
    options_array_first, options_array_item_value, options_array_next, options_get,
};
use crate::src::shared::abi::ssize_t;
use crate::src::shared::abi::*;
use crate::src::shared::command::{cmd_list, cmds};
use crate::src::shared::ctype::{
    _ISalnum, _ISalpha, _ISblank, _IScntrl, _ISdigit, _ISgraph, _ISlower, _ISprint, _ISpunct,
    _ISspace, _ISupper, _ISxdigit, ctype_code,
};
use crate::src::shared::errno::ERANGE;
use crate::src::shared::grid::*;
use crate::src::shared::limits::__LONG_LONG_MAX__;
use crate::src::shared::options::{
    options, options_array, options_array_item, options_entry, options_value,
};
use crate::src::shared::style::*;
use crate::src::shared::tree::{RB_BLACK, RB_NEGINF, RB_RED};
use crate::src::shared::utf8::*;
use crate::src::shared::utf8::{wchar_t, UTF8_SIZE};
use crate::src::shared::vis::VIS_DQ;
use crate::src::text::utf8_decode::{decode_utf8, DecodeResult};
use crate::src::tmux::global_options;
use std::ffi::{CStr, CString};

#[derive(Copy, Clone)]
#[repr(C)]
/// Dynamic entries are Box-owned while indexed; static defaults have
/// `allocated == 0` and must never be passed to `Box::from_raw`.
pub struct utf8_width_item {
    pub wc: wchar_t,
    pub width: u_int,
    pub allocated: ::core::ffi::c_int,
}
#[derive(Default)]
pub struct utf8_width_cache {
    entries: std::collections::BTreeMap<wchar_t, *mut utf8_width_item>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct utf8_item {
    pub index: u_int,
    pub data: [::core::ffi::c_char; 32],
    pub size: u_char,
}
#[derive(Default)]
pub struct utf8_data_tree {
    /// Borrowed aliases of items owned by the index tree for process lifetime.
    entries: std::collections::BTreeMap<(u_char, Vec<u8>), *mut utf8_item>,
}
#[derive(Default)]
pub struct utf8_index_tree {
    entries: std::collections::BTreeMap<u_int, Box<utf8_item>>,
}

pub const __WCHAR_MAX: ::core::ffi::c_int = __WCHAR_MAX__;
pub const ULLONG_MAX: ::core::ffi::c_ulonglong = (__LONG_LONG_MAX__ as ::core::ffi::c_ulonglong)
    .wrapping_mul(2 as ::core::ffi::c_ulonglong)
    .wrapping_add(1 as ::core::ffi::c_ulonglong);
pub const WCHAR_MAX: ::core::ffi::c_int = __WCHAR_MAX;

// The old comparator ordered entries only by their signed wchar_t value.
// BTreeMap has the same ordering, while its entry API preserves the tree's
// duplicate-insertion behavior of returning the existing item.
unsafe fn utf8_width_cache_find(head: *mut utf8_width_cache, wc: wchar_t) -> *mut utf8_width_item {
    (*head)
        .entries
        .get(&wc)
        .copied()
        .unwrap_or(::core::ptr::null_mut::<utf8_width_item>())
}

unsafe fn utf8_width_cache_insert(
    head: *mut utf8_width_cache,
    elm: *mut utf8_width_item,
) -> *mut utf8_width_item {
    match (*head).entries.entry((*elm).wc) {
        std::collections::btree_map::Entry::Occupied(entry) => *entry.get(),
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(elm);
            ::core::ptr::null_mut::<utf8_width_item>()
        }
    }
}

unsafe fn utf8_width_cache_minmax(
    head: *mut utf8_width_cache,
    val: ::core::ffi::c_int,
) -> *mut utf8_width_item {
    let entry = if val < 0 {
        (*head).entries.iter().next()
    } else {
        (*head).entries.iter().next_back()
    };
    entry
        .map(|(_, entry)| *entry)
        .unwrap_or(::core::ptr::null_mut::<utf8_width_item>())
}

unsafe fn utf8_width_cache_next(
    head: *mut utf8_width_cache,
    elm: *mut utf8_width_item,
) -> *mut utf8_width_item {
    (*head)
        .entries
        .range((
            std::ops::Bound::Excluded((*elm).wc),
            std::ops::Bound::Unbounded,
        ))
        .next()
        .map(|(_, entry)| *entry)
        .unwrap_or(::core::ptr::null_mut::<utf8_width_item>())
}

unsafe fn utf8_width_cache_remove(
    head: *mut utf8_width_cache,
    elm: *mut utf8_width_item,
) -> *mut utf8_width_item {
    (*head)
        .entries
        .remove(&(*elm).wc)
        .unwrap_or(::core::ptr::null_mut::<utf8_width_item>())
}

static mut utf8_width_cache: utf8_width_cache = utf8_width_cache {
    entries: std::collections::BTreeMap::new(),
};
static mut utf8_default_width_cache: [utf8_width_item; 162] = [
    utf8_width_item {
        wc: 0x261d as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x26f9 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x270a as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x270b as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x270c as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x270d as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f1e6 as wchar_t,
        width: 1 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f1e7 as wchar_t,
        width: 1 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f1e8 as wchar_t,
        width: 1 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f1e9 as wchar_t,
        width: 1 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f1ea as wchar_t,
        width: 1 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f1eb as wchar_t,
        width: 1 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f1ec as wchar_t,
        width: 1 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f1ed as wchar_t,
        width: 1 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f1ee as wchar_t,
        width: 1 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f1ef as wchar_t,
        width: 1 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f1f0 as wchar_t,
        width: 1 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f1f1 as wchar_t,
        width: 1 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f1f2 as wchar_t,
        width: 1 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f1f3 as wchar_t,
        width: 1 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f1f4 as wchar_t,
        width: 1 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f1f5 as wchar_t,
        width: 1 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f1f6 as wchar_t,
        width: 1 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f1f7 as wchar_t,
        width: 1 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f1f8 as wchar_t,
        width: 1 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f1f9 as wchar_t,
        width: 1 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f1fa as wchar_t,
        width: 1 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f1fb as wchar_t,
        width: 1 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f1fc as wchar_t,
        width: 1 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f1fd as wchar_t,
        width: 1 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f1fe as wchar_t,
        width: 1 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f1ff as wchar_t,
        width: 1 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f385 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f3c2 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f3c3 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f3c4 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f3c7 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f3ca as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f3cb as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f3cc as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f3fb as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f3fc as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f3fd as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f3fe as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f3ff as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f442 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f443 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f446 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f447 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f448 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f449 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f44a as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f44b as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f44c as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f44d as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f44e as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f44f as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f450 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f466 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f467 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f468 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f469 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f46b as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f46c as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f46d as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f46e as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f470 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f471 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f472 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f473 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f474 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f475 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f476 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f477 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f478 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f47c as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f481 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f482 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f483 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f485 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f486 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f487 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f48f as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f491 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f4aa as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f574 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f575 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f57a as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f590 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f595 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f596 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f645 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f646 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f647 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f64b as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f64c as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f64d as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f64e as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f64f as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f6a3 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f6b4 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f6b5 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f6b6 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f6c0 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f6cc as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f90c as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f90f as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f918 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f919 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f91a as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f91b as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f91c as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f91d as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f91e as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f91f as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f926 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f930 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f931 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f932 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f933 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f934 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f935 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f936 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f937 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f938 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f939 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f93d as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f93e as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f977 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f9b5 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f9b6 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f9b8 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f9b9 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f9bb as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f9cd as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f9ce as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f9cf as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f9d1 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f9d2 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f9d3 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f9d4 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f9d5 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f9d6 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f9d7 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f9d8 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f9d9 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f9da as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f9db as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f9dc as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1f9dd as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1fac3 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1fac4 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1fac5 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1faf0 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1faf1 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1faf2 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1faf3 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1faf4 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1faf5 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1faf6 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1faf7 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
    utf8_width_item {
        wc: 0x1faf8 as wchar_t,
        width: 2 as u_int,
        allocated: 0,
    },
];
// The old comparator ordered entries by size first and then by memcmp over exactly
// that many bytes. This key has the same ordering while the map preserves
// duplicate insertion behavior by retaining the first item for each key.
unsafe fn utf8_data_key(item: *const utf8_item) -> (u_char, Vec<u8>) {
    let size = (*item).size;
    let data = std::slice::from_raw_parts((*item).data.as_ptr().cast::<u8>(), size as usize);
    (size, data.to_vec())
}

unsafe fn utf8_data_tree_insert(head: *mut utf8_data_tree, elm: *mut utf8_item) -> *mut utf8_item {
    match (*head).entries.entry(utf8_data_key(elm)) {
        std::collections::btree_map::Entry::Occupied(entry) => *entry.get(),
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(elm);
            ::core::ptr::null_mut::<utf8_item>()
        }
    }
}

unsafe fn utf8_data_tree_find(head: *mut utf8_data_tree, item: *const utf8_item) -> *mut utf8_item {
    (*head)
        .entries
        .get(&utf8_data_key(item))
        .copied()
        .unwrap_or(::core::ptr::null_mut::<utf8_item>())
}

static mut utf8_data_tree: utf8_data_tree = utf8_data_tree {
    entries: std::collections::BTreeMap::new(),
};
// The old comparator ordered entries only by their unsigned index. BTreeMap's
// key ordering is identical, and its entry API retains RB_INSERT's behavior of
// returning the existing item without replacing it on duplicate keys.
unsafe fn utf8_index_tree_insert(
    head: *mut utf8_index_tree,
    elm: Box<utf8_item>,
) -> *mut utf8_item {
    match (*head).entries.entry(elm.index) {
        std::collections::btree_map::Entry::Occupied(mut entry) => {
            entry.get_mut().as_mut() as *mut utf8_item
        }
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(elm);
            ::core::ptr::null_mut::<utf8_item>()
        }
    }
}
unsafe fn utf8_index_tree_find(head: *mut utf8_index_tree, index: u_int) -> *mut utf8_item {
    (*head)
        .entries
        .get_mut(&index)
        .map(|item| item.as_mut() as *mut utf8_item)
        .unwrap_or(::core::ptr::null_mut::<utf8_item>())
}
static mut utf8_index_tree: utf8_index_tree = utf8_index_tree {
    entries: std::collections::BTreeMap::new(),
};
static mut utf8_no_width: ::core::ffi::c_int = 0;
static mut utf8_next_index: u_int = 0;
unsafe extern "C" fn utf8_item_by_data(
    mut data: *const u_char,
    mut size: size_t,
) -> *mut utf8_item {
    let mut ui: utf8_item = utf8_item {
        index: 0,
        data: [0; 32],
        size: 0,
    };
    memcpy(
        &raw mut ui.data as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        data as *const ::core::ffi::c_void,
        size,
    );
    ui.size = size as u_char;
    return utf8_data_tree_find(&raw mut utf8_data_tree, &ui);
}
unsafe extern "C" fn utf8_item_by_index(mut index: u_int) -> *mut utf8_item {
    let mut ui: utf8_item = utf8_item {
        index: 0,
        data: [0; 32],
        size: 0,
    };
    ui.index = index;
    return utf8_index_tree_find(&raw mut utf8_index_tree, ui.index);
}
unsafe extern "C" fn utf8_find_in_width_cache(mut wc: wchar_t) -> *mut utf8_width_item {
    return utf8_width_cache_find(&raw mut utf8_width_cache, wc);
}
unsafe extern "C" fn utf8_insert_width_cache(mut wc: wchar_t, mut width: u_int) {
    let mut uw: *mut utf8_width_item = ::core::ptr::null_mut::<utf8_width_item>();
    let mut old: *mut utf8_width_item = ::core::ptr::null_mut::<utf8_width_item>();
    log_debug(
        b"Unicode width cache: %08X=%u\0" as *const u8 as *const ::core::ffi::c_char,
        wc as u_int,
        width,
    );
    uw = Box::into_raw(Box::new(utf8_width_item {
        wc,
        width,
        allocated: 1,
    }));
    old = utf8_width_cache_insert(&raw mut utf8_width_cache, uw);
    if !old.is_null() {
        utf8_width_cache_remove(&raw mut utf8_width_cache, old);
        if (*old).allocated != 0 {
            drop(Box::from_raw(old));
        }
        utf8_width_cache_insert(&raw mut utf8_width_cache, uw);
    }
}
unsafe extern "C" fn utf8_add_to_width_cache(mut s: *const ::core::ffi::c_char) {
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut endptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut width: u_int = 0;
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut wc: wchar_t = 0;
    let mut wc_start: wchar_t = 0;
    let mut wc_end: wchar_t = 0;
    let mut n: ::core::ffi::c_ulonglong = 0;
    // The parser writes a NUL at '=', then only borrows the two parts during
    // this call. Keep the original C terminator and allocation stable.
    let mut copy_bytes = CStr::from_ptr(s).to_bytes_with_nul().to_vec();
    let copy = copy_bytes.as_mut_ptr().cast::<::core::ffi::c_char>();
    cp = strchr(copy, '=' as i32);
    if cp.is_null() {
        return;
    }
    let fresh0 = cp;
    cp = cp.offset(1);
    *fresh0 = '\0' as i32 as ::core::ffi::c_char;
    width = strtonum(
        cp,
        0 as ::core::ffi::c_longlong,
        2 as ::core::ffi::c_longlong,
        &raw mut errstr,
    ) as u_int;
    if !errstr.is_null() {
        return;
    }
    if strncmp(
        copy,
        b"U+\0" as *const u8 as *const ::core::ffi::c_char,
        2 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        *__errno_location() = 0 as ::core::ffi::c_int;
        n = strtoull(
            copy.offset(2 as ::core::ffi::c_int as isize),
            &raw mut endptr,
            16 as ::core::ffi::c_int,
        );
        if *copy.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '\0' as i32
            || n == 0 as ::core::ffi::c_ulonglong
            || n > WCHAR_MAX as ::core::ffi::c_ulonglong
            || *__errno_location() == ERANGE && n == ULLONG_MAX
        {
            return;
        }
        wc_start = n as wchar_t;
        if *endptr as ::core::ffi::c_int == '-' as i32 {
            endptr = endptr.offset(1);
            if strncmp(
                endptr,
                b"U+\0" as *const u8 as *const ::core::ffi::c_char,
                2 as size_t,
            ) != 0 as ::core::ffi::c_int
            {
                return;
            }
            *__errno_location() = 0 as ::core::ffi::c_int;
            n = strtoull(
                endptr.offset(2 as ::core::ffi::c_int as isize),
                &raw mut endptr,
                16 as ::core::ffi::c_int,
            );
            if *endptr as ::core::ffi::c_int != '\0' as i32
                || n == 0 as ::core::ffi::c_ulonglong
                || n > WCHAR_MAX as ::core::ffi::c_ulonglong
                || *__errno_location() == ERANGE && n == ULLONG_MAX
                || (n as wchar_t) < wc_start
            {
                return;
            }
            wc_end = n as wchar_t;
        } else {
            if *endptr as ::core::ffi::c_int != '\0' as i32 {
                return;
            }
            wc_end = wc_start;
        }
        wc = wc_start;
        while wc <= wc_end {
            utf8_insert_width_cache(wc, width);
            wc = wc.wrapping_add(1);
        }
    } else {
        let bytes = CStr::from_ptr(copy).to_bytes();
        let mut first = utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        };
        let mut offset = 0;
        utf8_no_width = 1 as ::core::ffi::c_int;
        if let Some(&byte) = bytes.first() {
            let mut more = utf8_open(&raw mut first, byte);
            if more == UTF8_MORE {
                offset = 1;
                while offset < bytes.len() && more == UTF8_MORE {
                    more = utf8_append(&raw mut first, bytes[offset]);
                    offset += 1;
                }
                if more != UTF8_DONE {
                    // utf8_fromcstr retries the first byte after an invalid
                    // or incomplete candidate, then emits it as one cell.
                    offset = 0;
                }
            }
            if more != UTF8_DONE {
                utf8_set(&raw mut first, bytes[offset]);
                offset += 1;
            }
        }
        utf8_no_width = 0 as ::core::ffi::c_int;
        if first.size == 0 || offset != bytes.len() {
            return;
        }
        let bytes = ::core::slice::from_raw_parts(first.data.as_ptr(), first.size as usize);
        let DecodeResult::Complete { codepoint, len } = decode_utf8(bytes) else {
            return;
        };
        if len != first.size as usize {
            return;
        }
        wc = codepoint as wchar_t;
        utf8_insert_width_cache(wc, width);
    }
}
#[no_mangle]
pub unsafe extern "C" fn utf8_update_width_cache() {
    let mut uw: *mut utf8_width_item = ::core::ptr::null_mut::<utf8_width_item>();
    let mut uw1: *mut utf8_width_item = ::core::ptr::null_mut::<utf8_width_item>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut i: u_int = 0;
    uw = utf8_width_cache_minmax(&raw mut utf8_width_cache, RB_NEGINF);
    while !uw.is_null() && {
        uw1 = utf8_width_cache_next(&raw mut utf8_width_cache, uw);
        1 as ::core::ffi::c_int != 0
    } {
        utf8_width_cache_remove(&raw mut utf8_width_cache, uw);
        if (*uw).allocated != 0 {
            drop(Box::from_raw(uw));
        }
        uw = uw1;
    }
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[utf8_width_item; 162]>() as usize)
            .wrapping_div(::core::mem::size_of::<utf8_width_item>() as usize)
    {
        utf8_width_cache_insert(
            &raw mut utf8_width_cache,
            (&raw mut utf8_default_width_cache as *mut utf8_width_item).offset(i as isize)
                as *mut utf8_width_item,
        );
        i = i.wrapping_add(1);
    }
    o = options_get(
        global_options,
        b"codepoint-widths\0" as *const u8 as *const ::core::ffi::c_char,
    );
    a = options_array_first(o);
    while !a.is_null() {
        utf8_add_to_width_cache((*options_array_item_value(a)).string_ptr());
        a = options_array_next(a);
    }
}
unsafe extern "C" fn utf8_put_item(
    mut data: *const u_char,
    mut size: size_t,
    mut index: *mut u_int,
) -> ::core::ffi::c_int {
    let mut ui: *mut utf8_item = ::core::ptr::null_mut::<utf8_item>();
    ui = utf8_item_by_data(data, size);
    if !ui.is_null() {
        *index = (*ui).index;
        log_debug(
            b"%s: found %.*s = %u\0" as *const u8 as *const ::core::ffi::c_char,
            b"utf8_put_item\0" as *const u8 as *const ::core::ffi::c_char,
            size as ::core::ffi::c_int,
            data,
            *index,
        );
        return 0 as ::core::ffi::c_int;
    }
    if utf8_next_index == (0xffffff as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as u_int {
        return -(1 as ::core::ffi::c_int);
    }
    let mut owned = Box::new(::core::mem::zeroed::<utf8_item>());
    let fresh2 = utf8_next_index;
    utf8_next_index = utf8_next_index.wrapping_add(1);
    owned.index = fresh2;
    memcpy(
        owned.data.as_mut_ptr().cast(),
        data as *const ::core::ffi::c_void,
        size,
    );
    owned.size = size as u_char;
    ui = owned.as_mut() as *mut utf8_item;
    assert!(
        utf8_index_tree_insert(&raw mut utf8_index_tree, owned).is_null(),
        "fresh UTF-8 index must be unique"
    );
    utf8_data_tree_insert(&raw mut utf8_data_tree, ui);
    *index = (*ui).index;
    log_debug(
        b"%s: added %.*s = %u\0" as *const u8 as *const ::core::ffi::c_char,
        b"utf8_put_item\0" as *const u8 as *const ::core::ffi::c_char,
        size as ::core::ffi::c_int,
        data,
        *index,
    );
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_from_data(
    mut ud: *const utf8_data,
    mut uc: *mut utf8_char,
) -> utf8_state {
    let mut current_block: u64;
    let mut index: u_int = 0;
    if (*ud).width as ::core::ffi::c_int > 2 as ::core::ffi::c_int {
        fatalx(
            b"invalid UTF-8 width: %u\0" as *const u8 as *const ::core::ffi::c_char,
            (*ud).width as ::core::ffi::c_int,
        );
    }
    if !((*ud).size as ::core::ffi::c_int > UTF8_SIZE) {
        if (*ud).size as ::core::ffi::c_int <= 3 as ::core::ffi::c_int {
            index = (((*ud).data[2 as ::core::ffi::c_int as usize] as utf8_char)
                << 16 as ::core::ffi::c_int
                | ((*ud).data[1 as ::core::ffi::c_int as usize] as utf8_char)
                    << 8 as ::core::ffi::c_int
                | (*ud).data[0 as ::core::ffi::c_int as usize] as utf8_char)
                as u_int;
            current_block = 11875828834189669668;
        } else if utf8_put_item(
            &raw const (*ud).data as *const u_char,
            (*ud).size as size_t,
            &raw mut index,
        ) != 0 as ::core::ffi::c_int
        {
            current_block = 801095099472899353;
        } else {
            current_block = 11875828834189669668;
        }
        match current_block {
            801095099472899353 => {}
            _ => {
                *uc = (((*ud).size as u_int) << 24 as ::core::ffi::c_int
                    | ((*ud).width as u_int).wrapping_add(1 as u_int) << 29 as ::core::ffi::c_int
                    | index) as utf8_char;
                log_debug(
                    b"%s: (%d %d %.*s) -> %08x\0" as *const u8 as *const ::core::ffi::c_char,
                    b"utf8_from_data\0" as *const u8 as *const ::core::ffi::c_char,
                    (*ud).width as ::core::ffi::c_int,
                    (*ud).size as ::core::ffi::c_int,
                    (*ud).size as ::core::ffi::c_int,
                    &raw const (*ud).data as *const u_char,
                    *uc,
                );
                return UTF8_DONE;
            }
        }
    }
    if (*ud).width as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        *uc = (0 as ::core::ffi::c_int as utf8_char) << 24 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int as utf8_char).wrapping_add(1 as utf8_char)
                << 29 as ::core::ffi::c_int;
    } else if (*ud).width as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        *uc = (1 as ::core::ffi::c_int as utf8_char) << 24 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int as utf8_char).wrapping_add(1 as utf8_char)
                << 29 as ::core::ffi::c_int
            | 0x20 as utf8_char;
    } else {
        *uc = (1 as ::core::ffi::c_int as utf8_char) << 24 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int as utf8_char).wrapping_add(1 as utf8_char)
                << 29 as ::core::ffi::c_int
            | 0x2020 as utf8_char;
    }
    return UTF8_ERROR;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_to_data(mut uc: utf8_char, mut ud: *mut utf8_data) {
    let mut ui: *mut utf8_item = ::core::ptr::null_mut::<utf8_item>();
    let mut index: u_int = 0;
    memset(
        ud as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<utf8_data>() as size_t,
    );
    (*ud).have = (uc >> 24 as ::core::ffi::c_int & 0x1f as utf8_char) as u_char;
    (*ud).size = (*ud).have;
    (*ud).width = (uc >> 29 as ::core::ffi::c_int).wrapping_sub(1 as utf8_char) as u_char;
    if (*ud).size as ::core::ffi::c_int <= 3 as ::core::ffi::c_int {
        (*ud).data[2 as ::core::ffi::c_int as usize] = (uc >> 16 as ::core::ffi::c_int) as u_char;
        (*ud).data[1 as ::core::ffi::c_int as usize] =
            (uc >> 8 as ::core::ffi::c_int & 0xff as utf8_char) as u_char;
        (*ud).data[0 as ::core::ffi::c_int as usize] = (uc & 0xff as utf8_char) as u_char;
    } else {
        index = (uc & 0xffffff as ::core::ffi::c_int as utf8_char) as u_int;
        ui = utf8_item_by_index(index);
        if ui.is_null() {
            memset(
                &raw mut (*ud).data as *mut u_char as *mut ::core::ffi::c_void,
                ' ' as i32,
                (*ud).size as size_t,
            );
        } else {
            memcpy(
                &raw mut (*ud).data as *mut u_char as *mut ::core::ffi::c_void,
                &raw mut (*ui).data as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                (*ud).size as size_t,
            );
        }
    }
    log_debug(
        b"%s: %08x -> (%d %d %.*s)\0" as *const u8 as *const ::core::ffi::c_char,
        b"utf8_to_data\0" as *const u8 as *const ::core::ffi::c_char,
        uc,
        (*ud).width as ::core::ffi::c_int,
        (*ud).size as ::core::ffi::c_int,
        (*ud).size as ::core::ffi::c_int,
        &raw mut (*ud).data as *mut u_char,
    );
}
#[no_mangle]
pub unsafe extern "C" fn utf8_build_one(mut ch: u_char) -> utf8_char {
    return (1 as ::core::ffi::c_int as utf8_char) << 24 as ::core::ffi::c_int
        | (1 as ::core::ffi::c_int as utf8_char).wrapping_add(1 as utf8_char)
            << 29 as ::core::ffi::c_int
        | ch as utf8_char;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_set(mut ud: *mut utf8_data, mut ch: u_char) {
    static mut empty: utf8_data = utf8_data {
        data: [
            0 as ::core::ffi::c_int as u_char,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
        have: 1 as u_char,
        size: 1 as u_char,
        width: 1 as u_char,
    };
    memcpy(
        ud as *mut ::core::ffi::c_void,
        &raw const empty as *const ::core::ffi::c_void,
        ::core::mem::size_of::<utf8_data>() as size_t,
    );
    *(&raw mut (*ud).data as *mut u_char) = ch;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_copy(mut to: *mut utf8_data, mut from: *const utf8_data) {
    let mut i: u_int = 0;
    memcpy(
        to as *mut ::core::ffi::c_void,
        from as *const ::core::ffi::c_void,
        ::core::mem::size_of::<utf8_data>() as size_t,
    );
    i = (*to).size as u_int;
    while (i as usize) < ::core::mem::size_of::<[u_char; 32]>() as usize {
        (*to).data[i as usize] = '\0' as i32 as u_char;
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn utf8_width(
    mut ud: *mut utf8_data,
    mut width: *mut ::core::ffi::c_int,
) -> utf8_state {
    let mut uw: *mut utf8_width_item = ::core::ptr::null_mut::<utf8_width_item>();
    let mut wc: wchar_t = 0;
    if utf8_towc(ud, &raw mut wc) as ::core::ffi::c_uint
        != UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return UTF8_ERROR;
    }
    uw = utf8_find_in_width_cache(wc);
    if !uw.is_null() {
        *width = (*uw).width as ::core::ffi::c_int;
        log_debug(
            b"cached width for %08X is %d\0" as *const u8 as *const ::core::ffi::c_char,
            wc as u_int,
            *width,
        );
        return UTF8_DONE;
    }
    *width = utf8proc_wcwidth(wc);
    log_debug(
        b"utf8proc_wcwidth(%05X) returned %d\0" as *const u8 as *const ::core::ffi::c_char,
        wc as u_int,
        *width,
    );
    if *width >= 0 as ::core::ffi::c_int && *width <= 0xff as ::core::ffi::c_int {
        return UTF8_DONE;
    }
    return UTF8_ERROR;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_towc(mut ud: *const utf8_data, mut wc: *mut wchar_t) -> utf8_state {
    let size = (*ud).size as usize;
    if size > ::core::mem::size_of::<[u_char; 32]>() {
        return UTF8_ERROR;
    }
    let bytes = ::core::slice::from_raw_parts((*ud).data.as_ptr(), size);
    match decode_utf8(bytes) {
        DecodeResult::Complete { codepoint, .. } => {
            *wc = codepoint as wchar_t;
            log_debug(
                b"UTF-8 %.*s is U+%06X\0" as *const u8 as *const ::core::ffi::c_char,
                (*ud).size as ::core::ffi::c_int,
                &raw const (*ud).data as *const u_char,
                *wc as u_int,
            );
            UTF8_DONE
        }
        _ => {
            log_debug(
                b"UTF-8 %.*s is invalid\0" as *const u8 as *const ::core::ffi::c_char,
                (*ud).size as ::core::ffi::c_int,
                &raw const (*ud).data as *const u_char,
            );
            UTF8_ERROR
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn utf8_has_whitespace(mut ud: *const utf8_data) -> ::core::ffi::c_int {
    let mut wc: wchar_t = 0;
    let mut offset: u_int = 0 as u_int;
    let mut size: u_int = 0;
    if (*ud).size as usize > ::core::mem::size_of::<[u_char; 32]>() {
        return 0 as ::core::ffi::c_int;
    }
    let bytes = ::core::slice::from_raw_parts((*ud).data.as_ptr(), (*ud).size as usize);
    while offset < (*ud).size as u_int {
        let remaining = &bytes[offset as usize..];
        let DecodeResult::Complete { codepoint, len } = decode_utf8(remaining) else {
            return 0 as ::core::ffi::c_int;
        };
        wc = codepoint as wchar_t;
        size = len as u_int;
        offset = offset.wrapping_add(size);
        match wc {
            9 | 10 | 11 | 12 | 13 | 32 | 133 | 160 | 5760 | 8192 | 8193 | 8194 | 8195 | 8196
            | 8197 | 8198 | 8199 | 8200 | 8201 | 8202 | 8232 | 8233 | 8239 | 8287 | 12288 => {
                return 1 as ::core::ffi::c_int
            }
            _ => {}
        }
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_fromwc(mut wc: wchar_t, mut ud: *mut utf8_data) -> utf8_state {
    let mut size: ::core::ffi::c_int = 0;
    let mut width: ::core::ffi::c_int = 0;
    size = utf8proc_wctomb(
        &raw mut (*ud).data as *mut u_char as *mut ::core::ffi::c_char,
        wc,
    );
    if size < 0 as ::core::ffi::c_int {
        log_debug(
            b"UTF-8 %d, wctomb() %d\0" as *const u8 as *const ::core::ffi::c_char,
            wc,
            *__errno_location(),
        );
        wctomb(::core::ptr::null_mut::<::core::ffi::c_char>(), 0 as wchar_t);
        return UTF8_ERROR;
    }
    if size == 0 as ::core::ffi::c_int {
        return UTF8_ERROR;
    }
    (*ud).have = size as u_char;
    (*ud).size = (*ud).have;
    if utf8_width(ud, &raw mut width) as ::core::ffi::c_uint
        == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*ud).width = width as u_char;
        return UTF8_DONE;
    }
    return UTF8_ERROR;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_open(mut ud: *mut utf8_data, mut ch: u_char) -> utf8_state {
    memset(
        ud as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<utf8_data>() as size_t,
    );
    let first = [ch as u8];
    let DecodeResult::Incomplete { expected } = decode_utf8(&first) else {
        return UTF8_ERROR;
    };
    if expected < 2 || expected > 4 {
        return UTF8_ERROR;
    }
    (*ud).size = expected as u_char;
    utf8_append(ud, ch);
    UTF8_MORE
}
#[no_mangle]
pub unsafe extern "C" fn utf8_append(mut ud: *mut utf8_data, mut ch: u_char) -> utf8_state {
    let mut width: ::core::ffi::c_int = 0;
    if (*ud).have as ::core::ffi::c_int >= (*ud).size as ::core::ffi::c_int {
        fatalx(b"UTF-8 character overflow\0" as *const u8 as *const ::core::ffi::c_char);
    }
    if (*ud).size as usize > ::core::mem::size_of::<[u_char; 32]>() as usize {
        fatalx(b"UTF-8 character size too large\0" as *const u8 as *const ::core::ffi::c_char);
    }
    if (*ud).have as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        && ch as ::core::ffi::c_int & 0xc0 as ::core::ffi::c_int != 0x80 as ::core::ffi::c_int
    {
        (*ud).width = 0xff as u_char;
    }
    let fresh1 = (*ud).have;
    (*ud).have = (*ud).have.wrapping_add(1);
    (*ud).data[fresh1 as usize] = ch;
    let bytes = ::core::slice::from_raw_parts((*ud).data.as_ptr(), (*ud).have as usize);
    match decode_utf8(bytes) {
        DecodeResult::Complete { len, .. }
            if (*ud).have as ::core::ffi::c_int == (*ud).size as ::core::ffi::c_int
                && len == (*ud).size as usize =>
        {
            if utf8_no_width == 0 {
                if (*ud).width as ::core::ffi::c_int == 0xff as ::core::ffi::c_int {
                    return UTF8_ERROR;
                }
                if utf8_width(ud, &raw mut width) as ::core::ffi::c_uint
                    != UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    return UTF8_ERROR;
                }
                (*ud).width = width as u_char;
            }
            UTF8_DONE
        }
        DecodeResult::Invalid { .. }
            if (*ud).have as ::core::ffi::c_int == (*ud).size as ::core::ffi::c_int =>
        {
            UTF8_ERROR
        }
        _ => UTF8_MORE,
    }
}
#[no_mangle]
pub unsafe extern "C" fn utf8_strvis(
    mut dst: *mut ::core::ffi::c_char,
    mut src: *const ::core::ffi::c_char,
    mut len: size_t,
    mut flag: ::core::ffi::c_int,
) -> size_t {
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut start: *const ::core::ffi::c_char = dst;
    let mut end: *const ::core::ffi::c_char = src.offset(len as isize);
    let mut more: utf8_state = UTF8_MORE;
    let mut i: size_t = 0;
    while src < end {
        more = utf8_open(&raw mut ud, *src as u_char);
        if more as ::core::ffi::c_uint == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint {
            loop {
                src = src.offset(1);
                if !(src < end
                    && more as ::core::ffi::c_uint
                        == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    break;
                }
                more = utf8_append(&raw mut ud, *src as u_char);
            }
            if more as ::core::ffi::c_uint == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                i = 0 as size_t;
                while i < ud.size as size_t {
                    let fresh3 = dst;
                    dst = dst.offset(1);
                    *fresh3 = ud.data[i as usize] as ::core::ffi::c_char;
                    i = i.wrapping_add(1);
                }
                continue;
            } else {
                src = src.offset(-(ud.have as ::core::ffi::c_int as isize));
            }
        }
        if flag & VIS_DQ != 0
            && *src.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '$' as i32
            && src < end.offset(-(1 as ::core::ffi::c_int as isize))
        {
            if *(*__ctype_b_loc()).offset(*src.offset(1 as ::core::ffi::c_int as isize) as u_char
                as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                & _ISalpha as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                != 0
                || *src.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '_' as i32
                || *src.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '{' as i32
            {
                let fresh4 = dst;
                dst = dst.offset(1);
                *fresh4 = '\\' as i32 as ::core::ffi::c_char;
            }
            let fresh5 = dst;
            dst = dst.offset(1);
            *fresh5 = '$' as i32 as ::core::ffi::c_char;
        } else if src < end.offset(-(1 as ::core::ffi::c_int as isize)) {
            dst = vis(
                dst,
                *src.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
                flag,
                *src.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
            );
        } else if src < end {
            dst = vis(
                dst,
                *src.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
                flag,
                '\0' as i32,
            );
        }
        src = src.offset(1);
    }
    *dst = '\0' as i32 as ::core::ffi::c_char;
    return dst.offset_from(start) as ::core::ffi::c_long as size_t;
}
/// Escape a C string using the same byte conversion as `utf8_strvis`, with
/// the result owned by Rust.
pub(crate) fn utf8_stravis_cstring(src: &CStr, flag: i32) -> CString {
    let source_len = src.to_bytes().len();
    // `utf8_strvis` writes at most four bytes for each source byte, followed
    // by one NUL. It initializes the buffer through that final NUL.
    let capacity = source_len
        .checked_mul(4)
        .and_then(|size| size.checked_add(1))
        .expect("escaped UTF-8 string is too large");
    let mut buffer = vec![0u8; capacity];
    let escaped_len = unsafe { utf8_strvis(buffer.as_mut_ptr().cast(), src.as_ptr(), source_len, flag) };
    buffer.truncate(escaped_len + 1);
    CString::from_vec_with_nul(buffer).expect("utf8_strvis output has no interior NUL")
}

/// Escape an explicit byte length without losing bytes after an input NUL.
pub(crate) fn utf8_stravisx_bytes(src: &[u8], flag: ::core::ffi::c_int) -> Vec<u8> {
    if src.is_empty() {
        return Vec::new();
    }
    let capacity = src
        .len()
        .checked_mul(4)
        .and_then(|size| size.checked_add(1))
        .expect("escaped UTF-8 bytes are too large");
    let mut buffer = vec![0u8; capacity];
    let escaped_len = unsafe {
        utf8_strvis(
            buffer.as_mut_ptr().cast(),
            src.as_ptr().cast(),
            src.len(),
            flag,
        )
    };
    buffer.truncate(escaped_len);
    buffer
}
#[no_mangle]
pub unsafe extern "C" fn utf8_isvalid(mut s: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut more: utf8_state = UTF8_MORE;
    end = s.offset(strlen(s) as isize);
    while s < end {
        more = utf8_open(&raw mut ud, *s as u_char);
        if more as ::core::ffi::c_uint == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint {
            loop {
                s = s.offset(1);
                if !(s < end
                    && more as ::core::ffi::c_uint
                        == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    break;
                }
                more = utf8_append(&raw mut ud, *s as u_char);
            }
            if more as ::core::ffi::c_uint == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                continue;
            }
            return 0 as ::core::ffi::c_int;
        } else {
            if (*s as ::core::ffi::c_int) < 0x20 as ::core::ffi::c_int
                || *s as ::core::ffi::c_int > 0x7e as ::core::ffi::c_int
            {
                return 0 as ::core::ffi::c_int;
            }
            s = s.offset(1);
        }
    }
    return 1 as ::core::ffi::c_int;
}
/// The input is a NUL-terminated C string. The result is ASCII and retains
/// the old sanitizer's first-NUL view and underscore width for UTF-8 cells.
pub(crate) fn utf8_sanitize_cstring(src: &CStr) -> CString {
    let mut dst = Vec::new();
    let mut more: utf8_state = UTF8_MORE;
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let source = src.to_bytes();
    let mut offset = 0;
    while offset < source.len() {
        let candidate_start = offset;
        more = unsafe { utf8_open(&raw mut ud, source[offset]) };
        if more as ::core::ffi::c_uint == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint {
            loop {
                offset += 1;
                if offset >= source.len() || more != UTF8_MORE {
                    break;
                }
                more = unsafe { utf8_append(&raw mut ud, source[offset]) };
            }
            if more as ::core::ffi::c_uint == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                // xreallocarray rejected the old zero-sized request when a
                // leading zero-width UTF-8 cell had produced no bytes yet.
                if dst.is_empty() && ud.width == 0 {
                    unsafe { fatalx(b"xreallocarray: zero size\0".as_ptr().cast()) };
                }
                dst.resize(dst.len() + ud.width as usize, b'_');
                continue;
            } else {
                // Retry each byte after an invalid or truncated candidate.
                // Rewinding by `have` could step before the input for a
                // complete-length invalid sequence.
                offset = candidate_start;
            }
        }
        if source[offset] > 0x1f && source[offset] < 0x7f {
            dst.push(source[offset]);
        } else {
            dst.push(b'_');
        }
        offset += 1;
    }
    CString::new(dst).expect("sanitized bytes contain no interior NUL")
}
#[no_mangle]
pub unsafe extern "C" fn utf8_strlen(mut s: *const utf8_data) -> size_t {
    let mut i: size_t = 0;
    i = 0 as size_t;
    while (*s.offset(i as isize)).size as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        i = i.wrapping_add(1);
    }
    return i;
}
#[no_mangle]
pub unsafe extern "C" fn utf8_strwidth(mut s: *const utf8_data, mut n: ssize_t) -> u_int {
    let mut i: ssize_t = 0;
    let mut width: u_int = 0 as u_int;
    i = 0 as ssize_t;
    while (*s.offset(i as isize)).size as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        if n != -(1 as ::core::ffi::c_int) as ssize_t && n == i {
            break;
        }
        width = width.wrapping_add((*s.offset(i as isize)).width as u_int);
        i += 1;
    }
    return width;
}
// Decode into Rust-owned cells while retaining the size-zero terminator used
// by the existing UTF-8 routines that borrow this array as a C-style view.
pub(crate) unsafe fn utf8_fromcstr_vec(mut src: *const ::core::ffi::c_char) -> Vec<utf8_data> {
    let mut cells = Vec::new();
    while *src != 0 {
        let mut cell = utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        };
        let mut more = utf8_open(&raw mut cell, *src as u_char);
        if more == UTF8_MORE {
            loop {
                src = src.offset(1);
                if *src == 0 || more != UTF8_MORE {
                    break;
                }
                more = utf8_append(&raw mut cell, *src as u_char);
            }
            if more == UTF8_DONE {
                cells.push(cell);
                continue;
            }
            src = src.offset(-(cell.have as isize));
        }
        utf8_set(&raw mut cell, *src as u_char);
        cells.push(cell);
        src = src.offset(1);
    }
    cells.push(utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    });
    cells
}

/// Copy the C-visible part of a sentinel-terminated cell stream into Rust-owned storage.
/// C consumers stop at the first NUL, even when it occurs inside a cell.
pub(crate) unsafe fn utf8_tocstr_cstring(mut src: *const utf8_data) -> CString {
    let mut bytes = Vec::new();
    while (*src).size != 0 {
        let data = &(&(*src).data)[..(*src).size as usize];
        if let Some(end) = data.iter().position(|&byte| byte == 0) {
            bytes.extend_from_slice(&data[..end]);
            break;
        }
        bytes.extend_from_slice(data);
        src = src.add(1);
    }
    CString::new(bytes).expect("the first NUL ends the copied string")
}
#[no_mangle]
pub unsafe extern "C" fn utf8_cstrwidth(mut s: *const ::core::ffi::c_char) -> u_int {
    let mut tmp: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut width: u_int = 0;
    let mut more: utf8_state = UTF8_MORE;
    width = 0 as u_int;
    while *s as ::core::ffi::c_int != '\0' as i32 {
        more = utf8_open(&raw mut tmp, *s as u_char);
        if more as ::core::ffi::c_uint == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint {
            loop {
                s = s.offset(1);
                if !(*s as ::core::ffi::c_int != '\0' as i32
                    && more as ::core::ffi::c_uint
                        == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    break;
                }
                more = utf8_append(&raw mut tmp, *s as u_char);
            }
            if more as ::core::ffi::c_uint == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                width = width.wrapping_add(tmp.width as u_int);
                continue;
            } else {
                s = s.offset(-(tmp.have as ::core::ffi::c_int as isize));
            }
        }
        if *s as ::core::ffi::c_int > 0x1f as ::core::ffi::c_int
            && *s as ::core::ffi::c_int != 0x7f as ::core::ffi::c_int
        {
            width = width.wrapping_add(1);
        }
        s = s.offset(1);
    }
    return width;
}
pub(crate) unsafe fn utf8_pad_cstring(
    s: *const ::core::ffi::c_char,
    width: u_int,
    left: bool,
) -> CString {
    let bytes = CStr::from_ptr(s).to_bytes();
    let padding = width.saturating_sub(utf8_cstrwidth(s)) as usize;
    let mut output = Vec::with_capacity(bytes.len() + padding);
    if left {
        output.resize(padding, b' ');
        output.extend_from_slice(bytes);
    } else {
        output.extend_from_slice(bytes);
        output.resize(bytes.len() + padding, b' ');
    }
    CString::new(output).expect("padded C string contains no NUL")
}
#[no_mangle]
pub unsafe extern "C" fn utf8_cstrhas(
    s: *const ::core::ffi::c_char,
    ud: *const utf8_data,
) -> ::core::ffi::c_int {
    utf8_cstrhas_impl(CStr::from_ptr(s), &*ud) as ::core::ffi::c_int
}
pub(crate) fn utf8_cstrhas_impl(s: &CStr, ud: &utf8_data) -> bool {
    let bytes = s.to_bytes();
    let mut offset = 0;
    let mut found = 0;
    while offset < bytes.len() {
        let mut cell = utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        };
        let mut more = unsafe { utf8_open(&raw mut cell, bytes[offset]) };
        if more == UTF8_MORE {
            let start = offset;
            offset += 1;
            while offset < bytes.len() && more == UTF8_MORE {
                more = unsafe { utf8_append(&raw mut cell, bytes[offset]) };
                offset += 1;
            }
            if more != UTF8_DONE {
                // utf8_fromcstr retries every byte after an incomplete or
                // invalid candidate, including bytes consumed by utf8_append.
                offset = start;
            }
        }
        if more != UTF8_DONE {
            unsafe { utf8_set(&raw mut cell, bytes[offset]) };
            offset += 1;
        }
        let matches = {
            cell.size == ud.size
                && cell.data[..cell.size as usize] == ud.data[..ud.size as usize]
        };
        if matches {
            found = 1;
        }
    }
    found != 0
}

pub const __WCHAR_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_owns_printable_ascii_and_preserves_legacy_widths() {
        unsafe {
            for (input, expected) in [
                (&b"\0"[..], &b""[..]),
                (&b"A\x01 \x7fB\0"[..], &b"A_ _B"[..]),
                (&b"caf\xc3\xa9\0"[..], &b"caf_"[..]),
                (&b"\xe4\xb8\xad\0"[..], &b"__"[..]),
                (&b"\xff\0"[..], &b"_"[..]),
                (&b"\xe2\x82\0"[..], &b"__"[..]),
                (&b"\xe2(\xa1\0"[..], &b"_(_"[..]),
                (&b"A\0B\0"[..], &b"A"[..]),
            ] {
                let nul = input.iter().position(|&byte| byte == 0).unwrap();
                let input = CStr::from_bytes_with_nul(&input[..=nul]).unwrap();
                assert_eq!(utf8_sanitize_cstring(input).as_bytes(), expected);
            }
        }
    }

    #[test]
    fn owned_cell_decoder_retries_each_byte_after_a_bad_sequence() {
        unsafe {
            let cells = utf8_fromcstr_vec(b"\xe2(\xa1\0".as_ptr().cast());
            assert_eq!(cells.len(), 4);
            for (cell, byte) in cells[..3].iter().zip([0xe2, b'(', 0xa1]) {
                assert_eq!(cell.size, 1);
                assert_eq!(cell.data[0], byte);
            }
            assert_eq!(cells[3].size, 0);
        }
    }

    #[test]
    fn utf8_tocstr_cstring_stops_at_the_first_nul_in_a_cell_or_between_cells() {
        unsafe {
            for parts in [
                vec![],
                vec![vec![b'a'], vec![0xe2, 0x82, 0xac], vec![0xff]],
                vec![vec![b'a', 0, b'b'], vec![b'c']],
                vec![vec![b'a'], vec![0], vec![b'b']],
            ] {
                let mut cells = Vec::new();
                for part in parts {
                    let mut cell: utf8_data = std::mem::zeroed();
                    cell.data[..part.len()].copy_from_slice(&part);
                    cell.size = part.len() as u_char;
                    cells.push(cell);
                }
                cells.push(std::mem::zeroed());

                let mut expected = Vec::new();
                for cell in &cells {
                    if cell.size == 0 {
                        break;
                    }
                    let bytes = &cell.data[..cell.size as usize];
                    if let Some(end) = bytes.iter().position(|&byte| byte == 0) {
                        expected.extend_from_slice(&bytes[..end]);
                        break;
                    }
                    expected.extend_from_slice(bytes);
                }
                assert_eq!(utf8_tocstr_cstring(cells.as_ptr()).as_bytes(), expected);
            }
        }
    }

    #[test]
    fn owned_vis_helpers_preserve_multibyte_and_explicit_length_bytes() {
        unsafe {
            let input = CString::new(&b"a\xc3\xa9\xff"[..]).unwrap();
            let escaped = utf8_stravis_cstring(input.as_c_str(), crate::src::shared::vis::VIS_OCTAL);
            assert_eq!(escaped.as_bytes(), b"a\xc3\xa9\\377");

            let bytes = utf8_stravisx_bytes(b"a\0b", crate::src::shared::vis::VIS_OCTAL);
            assert_eq!(bytes, b"a\\000b");
        }
    }

    #[test]
    fn utf8_index_tree_matches_index_comparator() {
        unsafe {
            let mut tree = utf8_index_tree::default();
            let mut first_item = Box::new(std::mem::zeroed::<utf8_item>());
            first_item.index = 7;
            let first = first_item.as_mut() as *mut utf8_item;
            assert!(utf8_index_tree_insert(&raw mut tree, first_item).is_null());

            let mut zero = Box::new(std::mem::zeroed::<utf8_item>());
            zero.index = 0;
            assert!(utf8_index_tree_insert(&raw mut tree, zero).is_null());
            let mut maximum = Box::new(std::mem::zeroed::<utf8_item>());
            maximum.index = u_int::MAX;
            assert!(utf8_index_tree_insert(&raw mut tree, maximum).is_null());
            let mut duplicate = Box::new(std::mem::zeroed::<utf8_item>());
            duplicate.index = 7;
            assert_eq!(
                utf8_index_tree_insert(&raw mut tree, duplicate),
                first,
                "duplicate indexes keep the original item"
            );

            assert_eq!(
                tree.entries.keys().copied().collect::<Vec<_>>(),
                vec![0, 7, u_int::MAX]
            );
            assert_eq!(utf8_index_tree_find(&raw mut tree, 7), first);
            assert!(utf8_index_tree_find(&raw mut tree, 8).is_null());
        }
    }

    #[test]
    fn utf8_data_tree_matches_data_comparator() {
        fn set_data(item: &mut utf8_item, data: &[u8]) {
            item.size = data.len() as u_char;
            for (index, byte) in data.iter().copied().enumerate() {
                item.data[index] = byte as ::core::ffi::c_char;
            }
        }

        unsafe {
            let mut tree = utf8_data_tree::default();
            let mut items: [utf8_item; 4] = [std::mem::zeroed(); 4];
            set_data(&mut items[0], &[0x80]);
            set_data(&mut items[1], &[0xff]);
            set_data(&mut items[2], &[0x00, 0x00]);
            set_data(&mut items[3], &[0x80]);
            items[3].data[1] = 0x7f as ::core::ffi::c_char;

            let first = &mut items[0] as *mut utf8_item;
            let duplicate = &mut items[3] as *mut utf8_item;
            assert!(utf8_data_tree_insert(&raw mut tree, first).is_null());
            assert!(utf8_data_tree_insert(&raw mut tree, &mut items[1]).is_null());
            assert!(utf8_data_tree_insert(&raw mut tree, &mut items[2]).is_null());
            assert_eq!(
                utf8_data_tree_insert(&raw mut tree, duplicate),
                first,
                "duplicate data keeps the original item"
            );

            assert_eq!(
                tree.entries.keys().cloned().collect::<Vec<_>>(),
                vec![(1, vec![0x80]), (1, vec![0xff]), (2, vec![0x00, 0x00])]
            );
            assert_eq!(utf8_data_tree_find(&raw mut tree, duplicate), first);
            let mut missing = items[0];
            set_data(&mut missing, &[0x81]);
            assert!(utf8_data_tree_find(&raw mut tree, &missing).is_null());
        }
    }

    #[test]
    fn utf8_width_cache_matches_width_comparator() {
        unsafe {
            let mut cache = utf8_width_cache::default();
            let mut items: [utf8_width_item; 4] = [std::mem::zeroed(); 4];
            items[0].wc = 7;
            items[1].wc = -1;
            items[2].wc = 0;
            items[3].wc = 7;

            let first = &mut items[0] as *mut utf8_width_item;
            let duplicate = &mut items[3] as *mut utf8_width_item;
            assert!(utf8_width_cache_insert(&raw mut cache, first).is_null());
            assert!(utf8_width_cache_insert(&raw mut cache, &mut items[1]).is_null());
            assert!(utf8_width_cache_insert(&raw mut cache, &mut items[2]).is_null());
            assert_eq!(
                utf8_width_cache_insert(&raw mut cache, duplicate),
                first,
                "duplicate codepoints keep the original item"
            );

            assert_eq!(
                cache.entries.keys().copied().collect::<Vec<_>>(),
                vec![-1, 0, 7]
            );
            assert_eq!(utf8_width_cache_find(&raw mut cache, 7), first);
            assert!(utf8_width_cache_find(&raw mut cache, 8).is_null());
            assert_eq!(
                utf8_width_cache_minmax(&raw mut cache, RB_NEGINF),
                &mut items[1] as *mut utf8_width_item
            );
            assert_eq!(
                utf8_width_cache_next(&raw mut cache, &mut items[1]),
                &mut items[2] as *mut utf8_width_item
            );
            assert_eq!(
                utf8_width_cache_minmax(&raw mut cache, 0),
                first,
                "non-negative minmax selects the maximum"
            );

            assert_eq!(utf8_width_cache_remove(&raw mut cache, first), first);
            assert!(utf8_width_cache_find(&raw mut cache, 7).is_null());
        }
    }

    #[test]
    fn utf8_width_cache_parses_entries_and_ignores_invalid_ones() {
        unsafe {
            for entry in [
                &b"U+E010=2\0"[..],
                &b"U+E011-U+E013=0\0"[..],
                &b"\xee\x80\xa0=1\0"[..], // U+E020, as a UTF-8 character.
                &b"U+E040=1\0U+E041=2\0"[..], // Stop at the first NUL.
                &b"z=2\0"[..],            // A single ASCII cell also uses this path.
            ] {
                utf8_add_to_width_cache(entry.as_ptr().cast());
            }

            for (codepoint, expected) in [
                (0xE010, 2),
                (0xE011, 0),
                (0xE012, 0),
                (0xE013, 0),
                (0xE020, 1),
                (0xE040, 1),
                ('z' as i32, 2),
            ] {
                let item = utf8_find_in_width_cache(codepoint);
                assert!(!item.is_null(), "missing U+{codepoint:04X}");
                assert_eq!((*item).width, expected, "wrong width for U+{codepoint:04X}");
            }

            for entry in [
                &b"U+E030\0"[..],          // No separator.
                &b"U+E030=3\0"[..],        // Width out of range.
                &b"U+E030-U+E02F=1\0"[..], // Reversed range.
                &b"ab=1\0"[..],            // More than one character.
                &b"=1\0"[..],              // No character.
                &b"\xee\x80\xa2z=1\0"[..], // Valid UTF-8 cell followed by ASCII.
                &b"\xc3(=1\0"[..],         // Invalid continuation retries the first byte.
                &b"\xe2\x82=1\0"[..],      // Incomplete UTF-8 retries the first byte.
                &b"\xff=1\0"[..],          // Invalid single byte.
                &b"\xc0\xaf=1\0"[..],      // Overlong sequence.
            ] {
                utf8_add_to_width_cache(entry.as_ptr().cast());
            }
            for codepoint in [0xE030, 0xE041, 0xE022, 'a' as i32, 'b' as i32, '(' as i32] {
                assert!(
                    utf8_find_in_width_cache(codepoint).is_null(),
                    "unexpected U+{codepoint:04X}"
                );
            }
            assert_eq!(utf8_no_width, 0);

            for codepoint in [0xE010, 0xE011, 0xE012, 0xE013, 0xE020, 0xE040, 'z' as i32] {
                let item = utf8_find_in_width_cache(codepoint);
                utf8_width_cache_remove(&raw mut utf8_width_cache, item);
                drop(Box::from_raw(item));
            }
        }
    }
}
