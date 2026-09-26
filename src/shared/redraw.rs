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
    /// Dropping the rows also drops their spans.
    pub lines: Box<[redraw_line]>,
    pub generation: uint64_t,
    pub sx: u_int,
    pub sy: u_int,
    pub ox: u_int,
    pub oy: u_int,
}

#[derive(Default)]
pub struct redraw_line {
    pub spans: [redraw_spans; 7],
}

#[derive(Default)]
pub struct redraw_spans {
    /// Spans stay individually boxed so pointers handed to drawing helpers stay
    /// valid even if appending another span grows this vector.
    pub entries: Vec<Box<redraw_span>>,
}

impl redraw_spans {
    pub fn push(&mut self, span: redraw_span) {
        self.entries.push(Box::new(span));
    }

    pub fn iter_mut_ptr(&mut self) -> impl Iterator<Item = *mut redraw_span> + '_ {
        self.entries
            .iter_mut()
            .map(|span| span.as_mut() as *mut redraw_span)
    }
}

#[repr(C)]
pub struct redraw_span {
    pub x: u_int,
    pub width: u_int,
    pub data: redraw_span_data,
}

#[derive(Copy, Clone, Default)]
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

impl Default for redraw_span_data_c2rust_unnamed {
    fn default() -> Self {
        Self {
            p: redraw_span_data_c2rust_unnamed_p::default(),
        }
    }
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

#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct redraw_span_data_c2rust_unnamed_p {
    pub wp: *mut window_pane,
    pub px: u_int,
    pub py: u_int,
}

pub type redraw_span_type = ::core::ffi::c_uint;

#[cfg(test)]
mod tests {
    use super::*;

    fn span(x: u_int) -> redraw_span {
        redraw_span {
            x,
            width: 1,
            data: Default::default(),
        }
    }

    #[test]
    fn span_addresses_and_order_survive_vector_growth() {
        let mut spans = redraw_spans::default();
        spans.push(span(10));
        let first = spans.entries[0].as_ref() as *const redraw_span;

        for x in 11..128 {
            spans.push(span(x));
        }

        assert_eq!(spans.entries[0].as_ref() as *const redraw_span, first);
        assert_eq!(unsafe { (*first).x }, 10);
        assert_eq!(spans.entries.first().unwrap().x, 10);
        assert_eq!(spans.entries.last().unwrap().x, 127);
        let xs = spans
            .iter_mut_ptr()
            .map(|span| unsafe { (*span).x })
            .collect::<Vec<_>>();
        assert_eq!(xs.len(), 118);
        assert_eq!(xs.first(), Some(&10));
        assert_eq!(xs.last(), Some(&127));
    }
}
