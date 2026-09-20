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

#[derive(Copy, Clone)]
#[repr(C)]
pub struct mode_tree_data {
    pub dead: ::core::ffi::c_int,
    pub references: u_int,
    pub zoomed: ::core::ffi::c_int,
    pub wp: *mut window_pane,
    pub modedata: *mut ::core::ffi::c_void,
    pub menu: *const menu_item,
    pub sort_crit: sort_criteria,
    pub view_name: *const ::core::ffi::c_char,
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
    pub line_list: *mut mode_tree_line,
    pub line_size: u_int,
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
    pub search: *mut ::core::ffi::c_char,
    pub filter: *mut ::core::ffi::c_char,
    pub no_matches: ::core::ffi::c_int,
    pub search_dir: mode_tree_search_dir,
    pub search_icase: ::core::ffi::c_int,
    pub help: ::core::ffi::c_int,
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

#[derive(Copy, Clone)]
#[repr(C)]
pub struct mode_tree_item {
    pub parent: *mut mode_tree_item,
    pub itemdata: *mut ::core::ffi::c_void,
    pub line: u_int,
    pub key: key_code,
    pub keystr: *const ::core::ffi::c_char,
    pub keylen: size_t,
    pub tag: uint64_t,
    pub name: *const ::core::ffi::c_char,
    pub text: *const ::core::ffi::c_char,
    pub expanded: ::core::ffi::c_int,
    pub tagged: ::core::ffi::c_int,
    pub draw_as_parent: ::core::ffi::c_int,
    pub no_tag: ::core::ffi::c_int,
    pub align: ::core::ffi::c_int,
    pub children: mode_tree_list,
    pub entry: mode_tree_item_entry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct mode_tree_item_entry {
    pub tqe_next: *mut mode_tree_item,
    pub tqe_prev: *mut *mut mode_tree_item,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct mode_tree_list {
    pub tqh_first: *mut mode_tree_item,
    pub tqh_last: *mut *mut mode_tree_item,
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
