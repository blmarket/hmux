use super::*;
use crate::format::{format_create, format_defaults};
use crate::consts::{FORMAT_PANE, FORMAT_NOJOBS};
use crate::style::style_add;

pub(super) unsafe fn refresh_styles(pane: &mut window_pane) {
    unsafe {
        let options = pane.options_ref().clone();
        pane.flags &= !PANE_STYLECHANGED;
        let mut format = format_create(None, None, (FORMAT_PANE | pane.id) as c_int, FORMAT_NOJOBS);
        format_defaults(&mut format, None, None, None, Some(pane));
        let mut active = grid_default_cell;
        active.fg = pane.palette.fg;
        active.bg = pane.palette.bg;
        style_add(&mut active, &options, c"window-active-style", Some(&mut format));
        let mut normal = grid_default_cell;
        normal.fg = pane.palette.fg;
        normal.bg = pane.palette.bg;
        style_add(&mut normal, &options, c"window-style", Some(&mut format));
        pane.cached_gc = normal;
        pane.cached_active_gc = active;
    }
}
