//! Authoritative mode_tree model declarations.

use super::abi::{size_t, u_int, uint64_t};
use super::client::client;
use super::key::key_code;
use super::menu::menu_item;
use super::pane::window_pane;
use super::prompt::{prompt, prompt_free_cb, prompt_key_result, prompt_result};
use super::screen::screen;
use super::screen_write::screen_write_ctx;
use super::sort::sort_criteria;

#[derive(Copy, Clone)]
pub struct mode_tree_help_info {
    pub width: u_int,
    pub item: &'static ::std::ffi::CStr,
    pub lines: &'static [&'static ::std::ffi::CStr],
}

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
}

impl Default for mode_tree_data {
    fn default() -> Self {
        Self {
            dead: 0,
            references: 0,
            zoomed: 0,
            wp: std::ptr::null_mut(),
            modedata: std::ptr::null_mut(),
            menu: std::ptr::null(),
            sort_crit: sort_criteria {
                order: 0,
                reversed: 0,
                order_seq: std::ptr::null_mut(),
            },
            view_name: None,
            buildcb: None,
            drawcb: None,
            searchcb: None,
            menucb: None,
            heightcb: None,
            keycb: None,
            swapcb: None,
            sortcb: None,
            helpcb: None,
            children: mode_tree_list::default(),
            saved: mode_tree_list::default(),
            lines: Vec::new(),
            depth: 0,
            maxdepth: 0,
            width: 0,
            height: 0,
            offset: 0,
            current: 0,
            screen: screen::empty(),
            prompt: std::ptr::null_mut(),
            prompt_data: std::ptr::null_mut(),
            prompt_cx: 0,
            prompt_top: 0,
            preview: 0,
            search: None,
            filter: None,
            no_matches: 0,
            search_dir: 0,
            search_icase: 0,
            help: 0,
        }
    }
}

pub type mode_tree_search_dir = ::core::ffi::c_uint;

pub struct mode_tree_prompt {
    pub mtd: *mut mode_tree_data,
    pub c: *mut client,
    pub inputcb: mode_tree_prompt_input_cb,
    pub freecb: prompt_free_cb,
}

pub type mode_tree_prompt_input_cb = Option<
    Box<dyn FnMut(*mut client, Option<&std::ffi::CStr>, prompt_key_result) -> prompt_result>,
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
    pub tag: uint64_t,
    pub name: std::ffi::CString,
    pub text: Option<std::ffi::CString>,
    pub expanded: ::core::ffi::c_int,
    pub tagged: ::core::ffi::c_int,
    pub draw_as_parent: ::core::ffi::c_int,
    pub no_tag: ::core::ffi::c_int,
    pub align: ::core::ffi::c_int,
    pub children: mode_tree_list,
}

impl mode_tree_item {
    pub fn empty() -> Self {
        Self {
            parent: Default::default(),
            itemdata: Default::default(),
            line: Default::default(),
            key: Default::default(),
            keystr: Default::default(),
            keylen: Default::default(),
            tag: 0,
            name: Default::default(),
            text: Default::default(),
            expanded: Default::default(),
            tagged: Default::default(),
            draw_as_parent: Default::default(),
            no_tag: Default::default(),
            align: Default::default(),
            children: Default::default(),
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

pub type mode_tree_help_cb = Option<fn() -> mode_tree_help_info>;

pub type mode_tree_sort_cb = Option<fn(&mut sort_criteria)>;

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

pub type mode_tree_height_cb = Option<Box<dyn FnMut(u_int) -> u_int>>;

pub type mode_tree_height_fn =
    Option<unsafe fn(*mut ::core::ffi::c_void, u_int) -> u_int>;

pub type mode_tree_menu_cb = Option<Box<dyn FnMut(*mut client, key_code)>>;

pub type mode_tree_menu_fn =
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
    Box<
        dyn FnMut(
            &mut sort_criteria,
            Option<uint64_t>,
            Option<&std::ffi::CStr>,
        ) -> Option<uint64_t>,
    >,
>;

pub type mode_tree_build_fn = Option<
    unsafe extern "C" fn(
        *mut ::core::ffi::c_void,
        *mut sort_criteria,
        *mut uint64_t,
        *const ::core::ffi::c_char,
    ) -> (),
>;
