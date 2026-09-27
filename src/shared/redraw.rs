//! Authoritative redraw model declarations.

use super::abi::{u_int, uint64_t};
use super::client::client;
use super::layout::pane_lines;
use super::menu::MenuWeak;
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
    /// Preserve stable heap allocation while drawing helpers borrow spans.
    pub entries: Vec<Box<redraw_span>>,
}

impl redraw_spans {
    pub fn push(&mut self, span: redraw_span) {
        self.entries.push(Box::new(span));
    }

    pub fn iter(&self) -> impl Iterator<Item = &redraw_span> {
        self.entries.iter().map(Box::as_ref)
    }
}

#[repr(C)]
pub struct redraw_span {
    pub x: u_int,
    pub width: u_int,
    pub data: redraw_span_data,
}

/// The active payload owns any lifetime handles stored in a cached span.
#[derive(Clone)]
pub enum redraw_span_data {
    Pane(RedrawPaneSpan),
    Outside,
    Empty,
    Status(RedrawStatusSpan),
    Border(RedrawBorderSpan),
    Scrollbar(RedrawScrollbarSpan),
    Menu(RedrawMenuSpan),
}

impl Default for redraw_span_data {
    fn default() -> Self {
        Self::Pane(RedrawPaneSpan::default())
    }
}

impl redraw_span_data {
    pub fn kind(&self) -> redraw_span_type {
        match self {
            Self::Pane(_) => 0,
            Self::Outside => 1,
            Self::Empty => 2,
            Self::Status(_) => 3,
            Self::Border(_) => 4,
            Self::Scrollbar(_) => 5,
            Self::Menu(_) => 6,
        }
    }
    pub fn pane(&self) -> &RedrawPaneSpan {
        let Self::Pane(data) = self else {
            panic!("pane span expected")
        };
        data
    }
    pub fn pane_mut(&mut self) -> &mut RedrawPaneSpan {
        let Self::Pane(data) = self else {
            panic!("pane span expected")
        };
        data
    }
    pub fn border(&self) -> &RedrawBorderSpan {
        let Self::Border(data) = self else {
            panic!("border span expected")
        };
        data
    }
    pub fn border_mut(&mut self) -> &mut RedrawBorderSpan {
        let Self::Border(data) = self else {
            panic!("border span expected")
        };
        data
    }
    pub fn status(&self) -> &RedrawStatusSpan {
        let Self::Status(data) = self else {
            panic!("status span expected")
        };
        data
    }
    pub fn status_mut(&mut self) -> &mut RedrawStatusSpan {
        let Self::Status(data) = self else {
            panic!("status span expected")
        };
        data
    }
    pub fn scrollbar(&self) -> &RedrawScrollbarSpan {
        let Self::Scrollbar(data) = self else {
            panic!("scrollbar span expected")
        };
        data
    }
    pub fn scrollbar_mut(&mut self) -> &mut RedrawScrollbarSpan {
        let Self::Scrollbar(data) = self else {
            panic!("scrollbar span expected")
        };
        data
    }
    pub fn menu(&self) -> &RedrawMenuSpan {
        let Self::Menu(data) = self else {
            panic!("menu span expected")
        };
        data
    }
    pub fn menu_mut(&mut self) -> &mut RedrawMenuSpan {
        let Self::Menu(data) = self else {
            panic!("menu span expected")
        };
        data
    }
}

#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct RedrawPaneSpan {
    pub wp: *mut window_pane,
    pub px: u_int,
    pub py: u_int,
}

#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct RedrawBorderSpan {
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

#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct RedrawStatusSpan {
    pub wp: *mut window_pane,
    pub offset: u_int,
    pub cell_type: ::core::ffi::c_int,
}

#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct RedrawScrollbarSpan {
    pub wp: *mut window_pane,
    pub y: u_int,
    pub height: u_int,
    pub flags: ::core::ffi::c_int,
}

#[derive(Clone, Default)]
pub struct RedrawMenuSpan {
    pub md: MenuWeak,
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
        let xs = spans.iter().map(|span| span.x).collect::<Vec<_>>();
        assert_eq!(xs.len(), 118);
        assert_eq!(xs.first(), Some(&10));
        assert_eq!(xs.last(), Some(&127));
    }
}
