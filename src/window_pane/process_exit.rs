use crate::grid::Grid as _;
use super::*;
use crate::consts::{PANE_STATUSREADY, PANE_STATUSDRAWN, PANE_REDRAW, MODE_CURSOR};
use crate::format::format_single;
use crate::notify::notify_pane;
use crate::screen::{ScreenWriteCtx, screen_write_ctx_on_pane_base};

pub(super) unsafe fn finish_process(wp: &mut window_pane, notify: bool) -> bool {
    unsafe {
        let gc;

        let sx = wp.base().grid().width();
        let sy = wp.base().grid().height();
        wp.close_process();
        let remain_on_exit: core::ffi::c_int =
            ((*wp).options_ref()).number(c"remain-on-exit") as core::ffi::c_int;
        if remain_on_exit != 0 as core::ffi::c_int && !*wp.flags() & PANE_STATUSREADY != 0 {
            return false;
        }
        let current_block_31: u64;
        match remain_on_exit {
            2 => {
                let status = wp.status;
                if status & 0x7f as core::ffi::c_int == 0 as core::ffi::c_int
                    && (status & 0xff00 as core::ffi::c_int) >> 8 as core::ffi::c_int
                        == 0 as core::ffi::c_int
                {
                    current_block_31 = 13550086250199790493;
                } else {
                    current_block_31 = 622960851218599991;
                }
            }
            1 | 3 => {
                current_block_31 = 622960851218599991;
            }
            _ => {
                current_block_31 = 13550086250199790493;
            }
        }
        match current_block_31 {
            13550086250199790493 => {}
            _ => {
                if *wp.flags() & PANE_STATUSDRAWN != 0 {
                    return false;
                }
                wp.flags |= PANE_STATUSDRAWN;
                wp.dead_time = timeval::now();
                if notify {
                    notify_pane(c"pane-died", Some(&*wp));
                }
                let s = (*wp).options_ref().string_ref(c"remain-on-exit-format");
                if !s.is_empty() {
                    let expanded = format_single(None, &s, None, None, None, Some(wp));
                    let mut writer = screen_write_ctx_on_pane_base(wp);
                    writer.scrollregion(0 as u_int, sy.wrapping_sub(1 as u_int));
                    writer.cursormove(
                        0 as core::ffi::c_int,
                        sy.wrapping_sub(1 as u_int) as core::ffi::c_int,
                        0 as core::ffi::c_int,
                    );
                    writer.linefeed(1 as core::ffi::c_int, 8 as u_int);
                    gc = grid_default_cell;
                    writer.format_draw(&gc, sx, expanded.as_bytes(), None, 0 as core::ffi::c_int);
                    writer.finish();
                }
                let pane_screen_mode = wp.base().mode() & !MODE_CURSOR;
                wp.base_mut().set_mode(pane_screen_mode);
                wp.flags |= PANE_REDRAW;
                return false;
            }
        }
        if notify {
            notify_pane(c"pane-exited", Some(&*wp));
        }
        true
    }
}
