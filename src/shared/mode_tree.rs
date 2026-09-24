//! Authoritative mode_tree model declarations.

use super::abi::{size_t, u_int, uint64_t};
use super::client::client;
use super::key::key_code;
use super::menu::{menu, menu_item};
use super::pane::window_pane;
use super::prompt::{prompt, prompt_free_cb, prompt_key_result, prompt_result};
use super::screen::screen;
use super::screen_write::screen_write_ctx;
use super::sort::sort_criteria;

/// Identity retained by a mode tree across rebuilds. Legacy numeric tags are
/// kept for modes which have not yet migrated to semantic identities.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct ModeTreeIdentity {
    pub(crate) kind: u_int,
    pub(crate) first: u_int,
    pub(crate) second: u64,
    pub(crate) name: *const ::core::ffi::c_char,
    pub(crate) detail: *const ::core::ffi::c_char,
}

impl PartialEq for ModeTreeIdentity {
    fn eq(&self, other: &Self) -> bool {
        unsafe fn same_name(a: *const ::core::ffi::c_char, b: *const ::core::ffi::c_char) -> bool {
            match (a.is_null(), b.is_null()) {
                (true, true) => true,
                (false, false) => std::ffi::CStr::from_ptr(a) == std::ffi::CStr::from_ptr(b),
                _ => false,
            }
        }
        self.kind == other.kind
            && self.first == other.first
            && self.second == other.second
            && unsafe { same_name(self.name, other.name) && same_name(self.detail, other.detail) }
    }
}

impl Eq for ModeTreeIdentity {}

impl ModeTreeIdentity {
    pub const fn legacy(tag: u64) -> Self {
        Self {
            kind: 0,
            first: 0,
            second: tag,
            name: ::core::ptr::null(),
            detail: ::core::ptr::null(),
        }
    }

    pub const fn session(id: u_int) -> Self {
        Self {
            kind: 1,
            first: id,
            second: 0,
            name: ::core::ptr::null(),
            detail: ::core::ptr::null(),
        }
    }

    pub const fn winlink(session_id: u_int, index: ::core::ffi::c_int) -> Self {
        Self {
            kind: 2,
            first: session_id,
            second: index as u32 as u64,
            name: ::core::ptr::null(),
            detail: ::core::ptr::null(),
        }
    }

    pub const fn pane(id: u_int) -> Self {
        Self {
            kind: 3,
            first: id,
            second: 0,
            name: ::core::ptr::null(),
            detail: ::core::ptr::null(),
        }
    }

    /// The caller lends C strings until a mode-tree item copies them. Both
    /// pointers must remain valid whenever this identity is compared.
    pub const unsafe fn named(
        kind: u_int,
        first: u_int,
        second: u64,
        name: *const ::core::ffi::c_char,
        detail: *const ::core::ffi::c_char,
    ) -> Self {
        Self {
            kind,
            first,
            second,
            name,
            detail,
        }
    }
}

#[repr(C)]
pub struct mode_tree_data {
    pub dead: ::core::ffi::c_int,
    pub references: u_int,
    pub zoomed: ::core::ffi::c_int,
    pub wp: *mut window_pane,
    pub modedata: *mut ::core::ffi::c_void,
    pub menu: *const menu_item,
    pub sort_crit: sort_criteria,
    pub view_name: Option<&'static ::std::ffi::CStr>,
    pub buildcb: mode_tree_build_cb,
    pub drawcb: mode_tree_draw_cb,
    pub searchcb: mode_tree_search_cb,
    pub menucb: mode_tree_menu_cb,
    pub heightcb: mode_tree_height_cb,
    pub keycb: mode_tree_key_cb,
    pub swapcb: mode_tree_swap_cb,
    pub sortcb: mode_tree_sort_cb,
    pub helpcb: mode_tree_help_cb,
    pub children: mode_tree_list,
    pub saved: mode_tree_list,
    pub lines: Vec<mode_tree_line>,
    pub depth: u_int,
    pub maxdepth: u_int,
    pub width: u_int,
    pub height: u_int,
    pub offset: u_int,
    pub current: u_int,
    pub screen: screen,
    pub prompt: *mut prompt,
    pub prompt_data: *mut mode_tree_prompt,
    pub prompt_cx: u_int,
    pub prompt_top: ::core::ffi::c_int,
    pub preview: ::core::ffi::c_int,
    pub search: Option<std::ffi::CString>,
    pub filter: Option<std::ffi::CString>,
    pub no_matches: ::core::ffi::c_int,
    pub search_dir: mode_tree_search_dir,
    pub search_icase: ::core::ffi::c_int,
    pub help: ::core::ffi::c_int,
    pub build_identity: ModeTreeIdentity,
    pub has_build_identity: ::core::ffi::c_int,
}

pub type mode_tree_search_dir = ::core::ffi::c_uint;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct mode_tree_prompt {
    pub mtd: *mut mode_tree_data,
    pub c: *mut client,
    pub inputcb: mode_tree_prompt_input_cb,
    pub freecb: prompt_free_cb,
    pub data: *mut ::core::ffi::c_void,
}

pub type mode_tree_prompt_input_cb = Option<
    unsafe extern "C" fn(
        *mut client,
        *mut ::core::ffi::c_void,
        *const ::core::ffi::c_char,
        prompt_key_result,
    ) -> prompt_result,
>;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct mode_tree_line {
    pub item: *mut mode_tree_item,
    pub depth: u_int,
    pub last: ::core::ffi::c_int,
    pub flat: ::core::ffi::c_int,
}

#[repr(C)]
pub struct mode_tree_item {
    pub parent: *mut mode_tree_item,
    pub itemdata: *mut ::core::ffi::c_void,
    pub line: u_int,
    pub key: key_code,
    pub keystr: Option<std::ffi::CString>,
    pub keylen: size_t,
    pub identity: ModeTreeIdentity,
    pub name: std::ffi::CString,
    pub text: Option<std::ffi::CString>,
    pub expanded: ::core::ffi::c_int,
    pub tagged: ::core::ffi::c_int,
    pub draw_as_parent: ::core::ffi::c_int,
    pub no_tag: ::core::ffi::c_int,
    pub align: ::core::ffi::c_int,
    pub children: mode_tree_list,
    pub(crate) identity_name: Option<std::ffi::CString>,
    pub(crate) identity_detail: Option<std::ffi::CString>,
}

impl mode_tree_item {
    pub fn empty() -> Self {
        Self {
            parent: unsafe { ::core::mem::zeroed() },
            itemdata: unsafe { ::core::mem::zeroed() },
            line: unsafe { ::core::mem::zeroed() },
            key: unsafe { ::core::mem::zeroed() },
            keystr: Default::default(),
            keylen: unsafe { ::core::mem::zeroed() },
            identity: unsafe { ::core::mem::zeroed() },
            name: Default::default(),
            text: Default::default(),
            expanded: unsafe { ::core::mem::zeroed() },
            tagged: unsafe { ::core::mem::zeroed() },
            draw_as_parent: unsafe { ::core::mem::zeroed() },
            no_tag: unsafe { ::core::mem::zeroed() },
            align: unsafe { ::core::mem::zeroed() },
            children: Default::default(),
            identity_name: Default::default(),
            identity_detail: Default::default(),
        }
    }
}

/// Ordered child owners. Boxing keeps row pointers stable across vector growth
/// and moves between the live and saved trees during a rebuild.
#[derive(Default)]
pub struct mode_tree_list {
    pub(crate) items: Vec<Box<mode_tree_item>>,
}

impl mode_tree_list {
    pub(crate) fn pointers(&mut self) -> Vec<*mut mode_tree_item> {
        self.items
            .iter_mut()
            .map(|item| &mut **item as *mut _)
            .collect()
    }

    pub(crate) fn first(&mut self) -> *mut mode_tree_item {
        self.items
            .first_mut()
            .map_or(std::ptr::null_mut(), |item| &mut **item)
    }

    pub(crate) fn last(&mut self) -> *mut mode_tree_item {
        self.items
            .last_mut()
            .map_or(std::ptr::null_mut(), |item| &mut **item)
    }

    pub(crate) fn position(&self, item: *mut mode_tree_item) -> usize {
        self.items
            .iter()
            .position(|candidate| std::ptr::eq(&**candidate, item))
            .expect("mode tree item belongs to its parent list")
    }

    pub(crate) fn next(&mut self, item: *mut mode_tree_item) -> *mut mode_tree_item {
        let position = self.position(item);
        self.items
            .get_mut(position + 1)
            .map_or(std::ptr::null_mut(), |item| &mut **item)
    }

    pub(crate) fn previous(&mut self, item: *mut mode_tree_item) -> *mut mode_tree_item {
        self.position(item)
            .checked_sub(1)
            .and_then(|position| self.items.get_mut(position))
            .map_or(std::ptr::null_mut(), |item| &mut **item)
    }
}

pub type mode_tree_help_cb = Option<
    unsafe extern "C" fn(
        *mut u_int,
        *mut *const ::core::ffi::c_char,
    ) -> *mut *const ::core::ffi::c_char,
>;

pub type mode_tree_sort_cb = Option<unsafe extern "C" fn(*mut sort_criteria) -> ()>;

pub type mode_tree_swap_cb = Option<
    unsafe extern "C" fn(
        *mut ::core::ffi::c_void,
        *mut ::core::ffi::c_void,
        *mut sort_criteria,
    ) -> ::core::ffi::c_int,
>;

pub type mode_tree_key_cb = Option<
    unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut ::core::ffi::c_void, u_int) -> key_code,
>;

pub type mode_tree_height_cb =
    Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, u_int) -> u_int>;

pub type mode_tree_menu_cb =
    Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut client, key_code) -> ()>;

pub type mode_tree_search_cb = Option<
    unsafe extern "C" fn(
        *mut ::core::ffi::c_void,
        *mut ::core::ffi::c_void,
        *const ::core::ffi::c_char,
        ::core::ffi::c_int,
    ) -> ::core::ffi::c_int,
>;

pub type mode_tree_draw_cb = Option<
    unsafe extern "C" fn(
        *mut ::core::ffi::c_void,
        *mut ::core::ffi::c_void,
        *mut screen_write_ctx,
        u_int,
        u_int,
    ) -> (),
>;

pub type mode_tree_build_cb = Option<
    unsafe extern "C" fn(
        *mut ::core::ffi::c_void,
        *mut sort_criteria,
        *mut uint64_t,
        *const ::core::ffi::c_char,
    ) -> (),
>;

pub type mode_tree_each_cb = Option<
    unsafe extern "C" fn(
        *mut ::core::ffi::c_void,
        *mut ::core::ffi::c_void,
        *mut client,
        key_code,
    ) -> (),
>;
