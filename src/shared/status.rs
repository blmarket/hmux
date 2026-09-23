//! Authoritative status model declarations.

use super::abi::{timeval, u_int};
use super::client::client;
use super::event::event;
use super::grid::grid_cell;
use super::prompt::{prompt_key_result, prompt_result};
use super::screen::screen;
use super::style::{style, style_line_entry};

#[repr(C)]
pub struct status_line {
    pub timer: event,
    pub screen: screen,
    pub active: *mut screen,
    pub references: ::core::ffi::c_int,
    pub prompt_cx: u_int,
    pub style: grid_cell,
    pub entries: [style_line_entry; 5],
}

pub type status_prompt_input_cb = Option<
    unsafe extern "C" fn(
        *mut client,
        *mut ::core::ffi::c_void,
        *const ::core::ffi::c_char,
        prompt_key_result,
    ) -> prompt_result,
>;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct message_entry {
    pub msg: *mut ::core::ffi::c_char,
    pub msg_num: u_int,
    pub msg_time: timeval,
    pub entry: message_entry_entry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct message_entry_entry {
    pub tqe_next: *mut message_entry,
    pub tqe_prev: *mut *mut message_entry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct message_list {
    pub tqh_first: *mut message_entry,
    pub tqh_last: *mut *mut message_entry,
}
