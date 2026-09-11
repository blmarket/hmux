use crate::window::{window_pane_exited, window_pane_get_theme};
use super::*;
use crate::format::{format_create, format_defaults};
use crate::consts::{FORMAT_PANE, FORMAT_NOJOBS, MODE_THEME_UPDATES};
use crate::style::{style_add, THEME_LIGHT, THEME_DARK, THEME_UNKNOWN};

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

pub(super) unsafe fn send_theme_update(wp: &mut window_pane) {
    unsafe {
        if window_pane_exited(wp) != 0 {
            return;
        }
        if *wp.flags() & PANE_THEMECHANGED == 0 {
            return;
        }
        if wp.screen_ref().mode() & MODE_THEME_UPDATES == 0 {
            return;
        }
        let theme = window_pane_get_theme(Some(wp));
        if wp.last_theme == theme {
            return;
        }
        wp.last_theme = theme;
        wp.flags &= !PANE_THEMECHANGED;
        match theme {
            THEME_LIGHT => {
                log_debug(
                    c"%s: %%%u light theme",
                    fmt_args![c"window_pane_send_theme_update", wp.pane_id()],
                );
                wp.write_terminal(b"\x1B[?997;2n");
            }
            THEME_DARK => {
                log_debug(
                    c"%s: %%%u dark theme",
                    fmt_args![c"window_pane_send_theme_update", wp.pane_id()],
                );
                wp.write_terminal(b"\x1B[?997;1n");
            }
            THEME_UNKNOWN => {
                log_debug(
                    c"%s: %%%u unknown theme",
                    fmt_args![c"window_pane_send_theme_update", wp.pane_id()],
                );
            }
            _ => {}
        };
    }
}
