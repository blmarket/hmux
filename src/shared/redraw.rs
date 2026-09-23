//! Authoritative redraw model declarations.

use super::abi::{u_int, uint64_t};
use super::client::client;
use super::layout::pane_lines;
use super::menu::menu_data;
use super::pane::window_pane;
use super::window::window;

#[repr(C)]
pub struct redraw_scene {
    pub c: *mut client,
    pub w: *mut window,
    /// Stable boxed row slice owned by this scene, with length `sy`.
    /// `redraw_free_scene` drains spans before releasing these rows.
    pub lines: *mut redraw_line,
    pub generation: uint64_t,
    pub sx: u_int,
    pub sy: u_int,
    pub ox: u_int,
    pub oy: u_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct redraw_line {
    pub spans: [redraw_spans; 7],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct redraw_spans {
    pub tqh_first: *mut redraw_span,
    pub tqh_last: *mut *mut redraw_span,
}

#[repr(C)]
pub struct redraw_span {
    pub x: u_int,
    pub width: u_int,
    pub data: redraw_span_data,
    pub entry: redraw_span_entry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct redraw_span_entry {
    pub tqe_next: *mut redraw_span,
    pub tqe_prev: *mut *mut redraw_span,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct redraw_span_data {
    pub type_0: redraw_span_type,
    pub c2rust_unnamed: redraw_span_data_c2rust_unnamed,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union redraw_span_data_c2rust_unnamed {
    pub p: redraw_span_data_c2rust_unnamed_p,
    pub b: redraw_span_data_c2rust_unnamed_b,
    pub st: redraw_span_data_c2rust_unnamed_st,
    pub sb: redraw_span_data_c2rust_unnamed_sb,
    pub m: redraw_span_data_c2rust_unnamed_m,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct redraw_span_data_c2rust_unnamed_m {
    pub md: *mut menu_data,
    pub px: u_int,
    pub py: u_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct redraw_span_data_c2rust_unnamed_sb {
    pub wp: *mut window_pane,
    pub y: u_int,
    pub height: u_int,
    pub flags: ::core::ffi::c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct redraw_span_data_c2rust_unnamed_st {
    pub wp: *mut window_pane,
    pub offset: u_int,
    pub cell_type: ::core::ffi::c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct redraw_span_data_c2rust_unnamed_b {
    pub top_wp: *mut window_pane,
    pub bottom_wp: *mut window_pane,
    pub left_wp: *mut window_pane,
    pub right_wp: *mut window_pane,
    pub style_wp: *mut window_pane,
    pub cell_type: ::core::ffi::c_int,
    pub cell_mask: ::core::ffi::c_int,
    pub top_lines: pane_lines,
    pub bottom_lines: pane_lines,
    pub left_lines: pane_lines,
    pub right_lines: pane_lines,
    pub flags: ::core::ffi::c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct redraw_span_data_c2rust_unnamed_p {
    pub wp: *mut window_pane,
    pub px: u_int,
    pub py: u_int,
}

pub type redraw_span_type = ::core::ffi::c_uint;
