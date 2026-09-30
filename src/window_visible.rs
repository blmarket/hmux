use crate::src::shared::abi::*;
use crate::src::shared::display::visible_range;
use crate::src::shared::layout::*;
use crate::src::shared::pane::{window_pane, PANE_SCROLLBARS_LEFT};
use crate::src::window::{
    window_pane_get_pane_lines, window_pane_is_floating, window_pane_is_visible,
    window_pane_scrollbar_reserve, window_pane_z_previous,
};
use crate::src::window::{PaneOrder, Window as _};

pub fn window_position_is_visible(ranges: &[visible_range], px: u_int) -> bool {
    ranges
        .iter()
        .any(|range| range.nx != 0 && px >= range.px && px < range.px.wrapping_add(range.nx))
}

/// Replace `ranges` with the unobstructed parts of a horizontal span in window
/// coordinates, retaining its allocation. Callers may reuse the buffer once its
/// previous result is consumed; nested queries need a separate buffer.
pub unsafe fn window_visible_ranges(
    base_wp: Option<&window_pane>,
    mut px: ::core::ffi::c_int,
    py: ::core::ffi::c_int,
    mut width: u_int,
    ranges: &mut Vec<visible_range>,
) {
    ranges.clear();
    if py < 0 || width == 0 {
        return;
    }
    if px < 0 {
        if -px as u_int >= width {
            return;
        }
        width = width.wrapping_sub(-px as u_int);
        px = 0;
    }
    let Some(base_wp) = base_wp else {
        ranges.push(visible_range {
            px: px as u_int,
            nx: width,
        });
        return;
    };

    if py as u_int
        >= ((base_wp.window_handle().as_ref()).expect("live window"))
            .size()
            .1
        || px as u_int
            >= ((base_wp.window_handle().as_ref()).expect("live window"))
                .size()
                .0
    {
        return;
    }
    if (px as u_int).wrapping_add(width)
        > ((base_wp.window_handle().as_ref()).expect("live window"))
            .size()
            .0
    {
        width = ((base_wp.window_handle().as_ref()).expect("live window"))
            .size()
            .0
            .wrapping_sub(px as u_int);
    }
    ranges.push(visible_range {
        px: px as u_int,
        nx: width,
    });
    let mut found_self = false;
    let mut cursor = base_wp
        .window_handle()
        .and_then(|window| window.step_pane(PaneOrder::Stacking, None, true));
    while let Some(pane_owner) = cursor {
        let wp = pane_owner.get();
        if std::ptr::eq(&*wp, base_wp) {
            found_self = true;
        } else {
            let floating = window_pane_is_floating(&*wp) != 0;
            let no_border =
                floating && window_pane_get_pane_lines(&*wp) == PANE_LINES_NONE as pane_lines;
            let (tb, bb) = if no_border {
                ((*wp).yoff, (*wp).yoff + (*wp).sy as ::core::ffi::c_int - 1)
            } else {
                (
                    if (*wp).yoff > 0 { (*wp).yoff - 1 } else { 0 },
                    (*wp).yoff + (*wp).sy as ::core::ffi::c_int,
                )
            };
            if found_self
                && window_pane_is_visible(&pane_owner) != 0
                && py >= tb
                && py <= bb
                && (floating || (py != tb && py != bb))
            {
                let (sb_w, sb_pos) = if window_pane_scrollbar_reserve(&*wp) != 0 {
                    (
                        (*wp).scrollbar_style.width + (*wp).scrollbar_style.pad,
                        ((base_wp.window_handle().as_ref()).expect("live window"))
                            .scrollbars()
                            .position,
                    )
                } else {
                    (0, 0)
                };
                let (mut lb, mut rb) = if no_border {
                    ((*wp).xoff, (*wp).xoff + (*wp).sx as ::core::ffi::c_int - 1)
                } else if sb_pos == PANE_SCROLLBARS_LEFT {
                    (
                        if (*wp).xoff > sb_w {
                            (*wp).xoff - 1 - sb_w
                        } else {
                            0
                        },
                        (*wp).xoff + (*wp).sx as ::core::ffi::c_int,
                    )
                } else {
                    (
                        if (*wp).xoff > 0 { (*wp).xoff - 1 } else { 0 },
                        (*wp).xoff + (*wp).sx as ::core::ffi::c_int + sb_w,
                    )
                };
                lb = lb.max(0);
                if rb >= 0 {
                    if (no_border
                        && rb
                            >= ((base_wp.window_handle().as_ref()).expect("live window"))
                                .size()
                                .0 as ::core::ffi::c_int)
                        || (!no_border
                            && rb
                                > ((base_wp.window_handle().as_ref()).expect("live window"))
                                    .size()
                                    .0 as ::core::ffi::c_int)
                    {
                        rb = ((base_wp.window_handle().as_ref()).expect("live window"))
                            .size()
                            .0
                            .wrapping_sub(1) as ::core::ffi::c_int;
                    }
                    if lb <= rb {
                        let mut i = 0;
                        while i < ranges.len() {
                            let range = ranges[i];
                            if range.nx != 0 {
                                let sx = range.px as ::core::ffi::c_int;
                                let ex = range.px.wrapping_add(range.nx).wrapping_sub(1)
                                    as ::core::ffi::c_int;
                                if lb > sx && lb <= ex && rb > ex {
                                    ranges[i].nx = (lb - sx) as u_int;
                                } else if rb >= sx && rb <= ex && lb <= sx {
                                    ranges[i].nx = (ex - rb) as u_int;
                                    ranges[i].px = (rb + 1) as u_int;
                                } else if lb > sx && rb <= ex {
                                    ranges.insert(
                                        i + 1,
                                        visible_range {
                                            px: (rb + 1) as u_int,
                                            nx: (ex - rb) as u_int,
                                        },
                                    );
                                    ranges[i].nx = (lb - sx) as u_int;
                                } else if lb <= sx && rb > ex {
                                    ranges[i].nx = 0;
                                }
                            }
                            i += 1;
                        }
                    }
                }
            }
        }
        cursor = window_pane_z_previous(wp.as_ref());
    }
}
