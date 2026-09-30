//! Pane mouse hit testing uses copied geometry across Window operations.
use super::*;
use crate::src::shared::pane::PANE_SCROLLBARS_RIGHT;

pub(super) unsafe fn in_scrollbar_area(
    pane: &Rc<UnsafeCell<window_pane>>,
    mut px: ::core::ffi::c_int,
    mut py: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let (sx, sy, xoff, yoff) = pane.geometry();
    let scrollbar = &pane;
    let mut width: u_int = 0;
    let mut pad: u_int = 0;
    let mut total: u_int = 0;
    let mut start: ::core::ffi::c_int = 0;
    let mut end: ::core::ffi::c_int = 0;
    if !scrollbar.scrollbar_overlay() {
        return 0 as ::core::ffi::c_int;
    }
    if py < yoff || py >= yoff + sy as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    width = scrollbar.scrollbar_width() as u_int;
    pad = scrollbar.scrollbar_pad() as u_int;
    total = width.wrapping_add(pad);
    if total == 0 as u_int || total > sx {
        total = sx;
    }
    let window = pane
        .window_observer()
        .upgrade()
        .expect("scrollbar pane window");
    let position = window.scrollbar_position();
    window.release(c"scrollbar pane window");
    if position == PANE_SCROLLBARS_LEFT {
        start = xoff;
        end = xoff + total as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
    } else {
        end = xoff + sx as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
        start = end - total as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    }
    return (px >= start && px <= end) as ::core::ffi::c_int;
}

pub(super) unsafe fn mouse_location(
    pane: &Rc<UnsafeCell<window_pane>>,
    x: i32,
    y: i32,
    slider: &mut u32,
) -> key_code_mouse_location {
    let window = pane.window_observer().upgrade().expect("mouse pane window");
    let location = mouse_location_in(pane, &window, x, y, slider);
    window.release(c"mouse pane window");
    location
}

unsafe fn mouse_location_in(
    pane_owner: &Rc<UnsafeCell<window_pane>>,
    window_owner: &WindowRef,
    mut px: ::core::ffi::c_int,
    mut py: ::core::ffi::c_int,
    sl_mpos: &mut u_int,
) -> key_code_mouse_location {
    let (sx, sy, xoff, yoff) = pane_owner.geometry();
    let scrollbar = &pane_owner;
    let position = window_owner.scrollbar_position();
    let mut examined = None;
    let mut pane_status: ::core::ffi::c_int = 0;
    let mut sb_w: ::core::ffi::c_int = 0;
    let mut sb_pad: ::core::ffi::c_int = 0;
    let mut pane_status_line: ::core::ffi::c_int = 0;
    let mut sl_top: ::core::ffi::c_int = 0;
    let mut sl_bottom: ::core::ffi::c_int = 0;
    let mut bdr_bottom: ::core::ffi::c_int = 0;
    let mut bdr_top: ::core::ffi::c_int = 0;
    let mut bdr_left: ::core::ffi::c_int = 0;
    let mut bdr_right: ::core::ffi::c_int = 0;
    let mut sb_start: ::core::ffi::c_int = 0;
    let mut sb_end: ::core::ffi::c_int = 0;
    let mut sb_overlay: ::core::ffi::c_int = 0;
    pane_status = pane_owner.border_status();
    sb_overlay = scrollbar.scrollbar_overlay() as i32;
    if scrollbar.scrollbar_visible() {
        sb_w = scrollbar.scrollbar_width();
        sb_pad = scrollbar.scrollbar_pad();
        if sb_overlay != 0 && sb_w > sx as ::core::ffi::c_int {
            sb_w = sx as ::core::ffi::c_int;
        }
    } else {
        sb_w = 0 as ::core::ffi::c_int;
        sb_pad = 0 as ::core::ffi::c_int;
    }
    if pane_status == PANE_STATUS_TOP {
        pane_status_line = yoff - 1 as ::core::ffi::c_int;
    } else if pane_status == PANE_STATUS_BOTTOM {
        pane_status_line = (yoff as u_int).wrapping_add(sy) as ::core::ffi::c_int;
    } else {
        pane_status_line = -(1 as ::core::ffi::c_int);
    }
    bdr_left = xoff - 1 as ::core::ffi::c_int;
    if sb_overlay == 0 && position == PANE_SCROLLBARS_LEFT {
        bdr_left -= sb_pad + sb_w;
    }
    if sb_overlay != 0
        && sb_w != 0 as ::core::ffi::c_int
        && py >= yoff
        && py < yoff + sy as ::core::ffi::c_int
        && px >= xoff
        && px < xoff + sx as ::core::ffi::c_int
    {
        if position == PANE_SCROLLBARS_LEFT {
            sb_start = xoff;
            sb_end = sb_start + sb_w - 1 as ::core::ffi::c_int;
        } else {
            sb_end = xoff + sx as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
            sb_start = sb_end - sb_w + 1 as ::core::ffi::c_int;
        }
        if px >= sb_start && px <= sb_end {
            sl_top =
                (yoff as u_int).wrapping_add(scrollbar.scrollbar_slider_y()) as ::core::ffi::c_int;
            sl_bottom = (yoff as u_int)
                .wrapping_add(scrollbar.scrollbar_slider_y())
                .wrapping_add(scrollbar.scrollbar_slider_height())
                .wrapping_sub(1 as u_int) as ::core::ffi::c_int;
            if py < sl_top {
                return KEYC_MOUSE_LOCATION_SCROLLBAR_UP;
            } else if py >= sl_top && py <= sl_bottom {
                *sl_mpos = (py as u_int)
                    .wrapping_sub(scrollbar.scrollbar_slider_y())
                    .wrapping_sub(yoff as u_int);
                return KEYC_MOUSE_LOCATION_SCROLLBAR_SLIDER;
            } else {
                return KEYC_MOUSE_LOCATION_SCROLLBAR_DOWN;
            }
        }
        return KEYC_MOUSE_LOCATION_PANE;
    }
    if (pane_status != PANE_STATUS_OFF
        && py != pane_status_line
        && py != yoff + sy as ::core::ffi::c_int
        || yoff == 0 as ::core::ffi::c_int && py < sy as ::core::ffi::c_int
        || py >= yoff && py < yoff + sy as ::core::ffi::c_int)
        && (position == PANE_SCROLLBARS_RIGHT
            && px < xoff + sx as ::core::ffi::c_int + sb_pad + sb_w
            || position == PANE_SCROLLBARS_LEFT
                && px < xoff + sx as ::core::ffi::c_int - sb_pad - sb_w)
    {
        if position == PANE_SCROLLBARS_RIGHT
            && (px >= xoff + sx as ::core::ffi::c_int + sb_pad
                && px < xoff + sx as ::core::ffi::c_int + sb_pad + sb_w)
            || position == PANE_SCROLLBARS_LEFT
                && (px >= xoff - sb_pad - sb_w && px < xoff - sb_pad)
        {
            sl_top =
                (yoff as u_int).wrapping_add(scrollbar.scrollbar_slider_y()) as ::core::ffi::c_int;
            sl_bottom = (yoff as u_int)
                .wrapping_add(scrollbar.scrollbar_slider_y())
                .wrapping_add(scrollbar.scrollbar_slider_height())
                .wrapping_sub(1 as u_int) as ::core::ffi::c_int;
            if py < sl_top {
                return KEYC_MOUSE_LOCATION_SCROLLBAR_UP;
            } else if py >= sl_top && py <= sl_bottom {
                *sl_mpos = (py as u_int)
                    .wrapping_sub(scrollbar.scrollbar_slider_y())
                    .wrapping_sub(yoff as u_int);
                return KEYC_MOUSE_LOCATION_SCROLLBAR_SLIDER;
            } else {
                return KEYC_MOUSE_LOCATION_SCROLLBAR_DOWN;
            }
        } else if pane_owner.is_floating()
            && pane_owner.pane_lines() as ::core::ffi::c_uint
                != PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
            && (px == bdr_left
                || py == yoff - 1 as ::core::ffi::c_int
                || py == yoff + sy as ::core::ffi::c_int)
        {
            return KEYC_MOUSE_LOCATION_BORDER;
        } else {
            return KEYC_MOUSE_LOCATION_PANE;
        }
    } else {
        let mut cursor = window_owner.next_pane(None);
        while let Some(border_pane) = cursor {
            let (border_sx, border_sy, border_xoff, border_yoff) = border_pane.geometry();
            let border_scrollbar = &border_pane;
            examined = Some(border_pane.clone());
            if !(!border_pane.is_visible()) {
                if !(border_pane.is_floating()
                    && border_pane.pane_lines() as ::core::ffi::c_uint
                        == PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    if border_scrollbar.scrollbar_reserved() {
                        sb_w = border_scrollbar.scrollbar_width();
                        sb_pad = border_scrollbar.scrollbar_pad();
                    } else {
                        sb_w = 0 as ::core::ffi::c_int;
                        sb_pad = 0 as ::core::ffi::c_int;
                    }
                    bdr_top = border_yoff - 1 as ::core::ffi::c_int;
                    bdr_bottom =
                        (border_yoff as u_int).wrapping_add(border_sy) as ::core::ffi::c_int;
                    bdr_left = border_xoff - 1 as ::core::ffi::c_int;
                    if position == PANE_SCROLLBARS_LEFT {
                        bdr_left -= sb_pad + sb_w;
                        bdr_right =
                            (border_xoff as u_int).wrapping_add(border_sx) as ::core::ffi::c_int;
                    } else {
                        bdr_right = (border_xoff as u_int)
                            .wrapping_add(border_sx)
                            .wrapping_add(sb_pad as u_int)
                            .wrapping_add(sb_w as u_int)
                            as ::core::ffi::c_int;
                    }
                    if py >= border_yoff - 1 as ::core::ffi::c_int
                        && py <= border_yoff + border_sy as ::core::ffi::c_int
                    {
                        if px == bdr_right {
                            break;
                        }
                        if pane_owner.is_floating() {
                            if px == bdr_left {
                                break;
                            }
                        }
                    }
                    if px >= bdr_left && px <= border_xoff + border_sx as ::core::ffi::c_int {
                        bdr_bottom =
                            (border_yoff as u_int).wrapping_add(border_sy) as ::core::ffi::c_int;
                        if py == bdr_bottom {
                            break;
                        }
                        if py == bdr_top {
                            break;
                        }
                    }
                }
            }
            cursor = window_owner.next_pane(Some(&border_pane));
        }
        if examined.is_some() {
            return KEYC_MOUSE_LOCATION_BORDER;
        }
    }
    return KEYC_MOUSE_LOCATION_NOWHERE;
}
