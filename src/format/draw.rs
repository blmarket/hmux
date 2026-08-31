use crate::fmt_args;
use crate::grid::grid_default_cell;
use crate::log::log_debug;
use crate::screen::screen_free;
use crate::screen::{
    screen_write_cell, screen_write_clearendofline, screen_write_cursormove,
    screen_write_fast_copy, screen_write_putc, screen_write_start, screen_write_stop,
};
use crate::style::{style_copy, style_parse, style_set, style_tostring};
use crate::text::{utf8_append, utf8_open, utf8_set};
pub use crate::types::*;
use ::core::ffi::{CStr, c_char, c_int};
use ::std::ffi::CString;

pub const UTF8_DONE: utf8_state = 1;
pub const UTF8_MORE: utf8_state = 0;
pub const STYLE_ALIGN_DEFAULT: style_align = 0;
pub const STYLE_ALIGN_LEFT: style_align = 1;
pub const STYLE_ALIGN_CENTRE: style_align = 2;
pub const STYLE_ALIGN_RIGHT: style_align = 3;
pub const STYLE_DEFAULT_BASE: style_default_type = 0;
pub const STYLE_DEFAULT_PUSH: style_default_type = 1;
pub const STYLE_DEFAULT_POP: style_default_type = 2;
pub const STYLE_DEFAULT_SET: style_default_type = 3;
pub const STYLE_LIST_OFF: style_list = 0;
pub const STYLE_LIST_ON: style_list = 1;
pub const STYLE_LIST_FOCUS: style_list = 2;
pub const STYLE_LIST_LEFT_MARKER: style_list = 3;
pub const STYLE_LIST_RIGHT_MARKER: style_list = 4;
pub const STYLE_RANGE_NONE: style_range_type = 0;
pub const STYLE_RANGE_LEFT: style_range_type = 1;
pub const STYLE_RANGE_RIGHT: style_range_type = 2;
pub const STYLE_RANGE_PANE: style_range_type = 3;
pub const STYLE_RANGE_WINDOW: style_range_type = 4;
pub const STYLE_RANGE_SESSION: style_range_type = 5;
pub const STYLE_RANGE_USER: style_range_type = 6;

/// The eight screens a format is drawn into before any of them reaches the
/// caller's: three for the alignments, one for the absolute centre, one for the
/// list, one each for the list's two markers, and one for whatever follows the
/// list. A screen is named by its index into those arrays.
const LEFT: usize = 0;
const CENTRE: usize = 1;
const RIGHT: usize = 2;
const ABSOLUTE_CENTRE: usize = 3;
const LIST: usize = 4;
const LIST_LEFT: usize = 5;
const LIST_RIGHT: usize = 6;
const AFTER: usize = 7;
const TOTAL: usize = 8;

/// The name of each screen, for the log.
static names: [&CStr; TOTAL] = [
    c"LEFT",
    c"CENTRE",
    c"RIGHT",
    c"ABSOLUTE_CENTRE",
    c"LIST",
    c"LIST_LEFT",
    c"LIST_RIGHT",
    c"AFTER",
];

/// How long a range's name can be, the same buffer `style` carries one in.
const RANGE_STRING: usize = 16;

/// One range of columns a style asked to be marked, while it is still being
/// drawn: which screen it is on and where it starts and ends there. The columns
/// are moved to where that screen lands on the caller's line once the widths
/// are known.
struct format_range {
    index: usize,
    start: u_int,
    end: u_int,
    type_0: style_range_type,
    argument: u_int,
    string: [c_char; RANGE_STRING],
}

/// The name in one of the fixed-size buffers a range carries, up to its
/// terminating NUL, which is what the C compared with `strcmp`.
fn range_name(buffer: &[c_char; RANGE_STRING]) -> &[c_char] {
    let end = buffer
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(buffer.len());
    &buffer[..end]
}

/// Whether an open range is the same one `sy` asks for, so that it carries on
/// rather than ending here. Which parts have to match depends on the kind:
/// the left, right, control and no-range kinds carry nothing else, the pane,
/// window and session kinds carry an id and the user kind a name.
fn format_is_type(fr: &format_range, sy: &style) -> bool {
    if fr.type_0 != sy.range_type {
        return false;
    }
    match fr.type_0 {
        STYLE_RANGE_PANE | STYLE_RANGE_WINDOW | STYLE_RANGE_SESSION => {
            fr.argument == sy.range_argument
        }
        STYLE_RANGE_USER => range_name(&fr.string) == range_name(&sy.range_string),
        _ => true,
    }
}

/// Moves the ranges on the screen named by `which` to where that screen was
/// drawn, and drops the ones no part of which was: `start` and `width` are the
/// columns of it that were copied and `offset` is where they landed.
fn format_update_ranges(
    frs: &mut Vec<format_range>,
    which: usize,
    offset: u_int,
    start: u_int,
    width: u_int,
) {
    let end = start.wrapping_add(width);
    frs.retain_mut(|fr| {
        if fr.index != which {
            return true;
        }
        if fr.end <= start || fr.start >= end {
            return false;
        }
        fr.start = fr.start.max(start);
        fr.end = fr.end.min(end);
        if fr.start == fr.end {
            return false;
        }
        fr.start = fr.start.wrapping_sub(start).wrapping_add(offset);
        fr.end = fr.end.wrapping_sub(start).wrapping_add(offset);
        true
    });
}

/// Copies `width` columns of the screen named by `which` from `start` into the
/// caller's line at `offset`, taking the ranges on that screen with it.
unsafe fn format_draw_put(
    octx: &mut screen_write_ctx,
    ocx: u_int,
    ocy: u_int,
    screens: &mut [screen; TOTAL],
    which: usize,
    frs: &mut Vec<format_range>,
    offset: u_int,
    start: u_int,
    width: u_int,
) {
    unsafe {
        screen_write_cursormove(octx, ocx.wrapping_add(offset) as c_int, ocy as c_int, 0);
        screen_write_fast_copy(octx, &screens[which], start, 0, width, 1);
        format_update_ranges(frs, which, offset, start, width);
    }
}

/// Copies the list into `width` columns at `offset`, scrolled so that the
/// focused part of it is in the middle, with the markers drawn at whichever
/// end the list runs past.
unsafe fn format_draw_put_list(
    octx: &mut screen_write_ctx,
    ocx: u_int,
    ocy: u_int,
    mut offset: u_int,
    mut width: u_int,
    screens: &mut [screen; TOTAL],
    focus_start: c_int,
    focus_end: c_int,
    frs: &mut Vec<format_range>,
) {
    unsafe {
        if width >= screens[LIST].cx {
            format_draw_put(octx, ocx, ocy, screens, LIST, frs, offset, 0, width);
            return;
        }

        let focus_centre = (focus_start + (focus_end - focus_start) / 2) as u_int;
        let mut start = if focus_centre < width / 2 {
            0
        } else {
            focus_centre.wrapping_sub(width / 2)
        };
        if start.wrapping_add(width) > screens[LIST].cx {
            start = screens[LIST].cx.wrapping_sub(width);
        }

        let marker_left = screens[LIST_LEFT].cx;
        let marker_right = screens[LIST_RIGHT].cx;
        if start != 0 && width > marker_left {
            screen_write_cursormove(octx, ocx.wrapping_add(offset) as c_int, ocy as c_int, 0);
            screen_write_fast_copy(octx, &screens[LIST_LEFT], 0, 0, marker_left, 1);
            offset = offset.wrapping_add(marker_left);
            start = start.wrapping_add(marker_left);
            width = width.wrapping_sub(marker_left);
        }
        if start.wrapping_add(width) < screens[LIST].cx && width > marker_right {
            screen_write_cursormove(
                octx,
                ocx.wrapping_add(offset)
                    .wrapping_add(width)
                    .wrapping_sub(marker_right) as c_int,
                ocy as c_int,
                0,
            );
            screen_write_fast_copy(octx, &screens[LIST_RIGHT], 0, 0, marker_right, 1);
            width = width.wrapping_sub(marker_right);
        }

        format_draw_put(octx, ocx, ocy, screens, LIST, frs, offset, start, width);
    }
}

/// Draws a format with no list in it: the centre is given up first, then the
/// right, then the left, and the absolute centre goes over all of them.
unsafe fn format_draw_none(
    octx: &mut screen_write_ctx,
    available: u_int,
    ocx: u_int,
    ocy: u_int,
    screens: &mut [screen; TOTAL],
    frs: &mut Vec<format_range>,
) {
    unsafe {
        let mut width_left = screens[LEFT].cx;
        let mut width_centre = screens[CENTRE].cx;
        let mut width_right = screens[RIGHT].cx;
        let mut width_abs_centre = screens[ABSOLUTE_CENTRE].cx;

        while width_left + width_centre + width_right > available {
            if width_centre > 0 {
                width_centre -= 1;
            } else if width_right > 0 {
                width_right -= 1;
            } else {
                width_left -= 1;
            }
        }

        format_draw_put(octx, ocx, ocy, screens, LEFT, frs, 0, 0, width_left);
        format_draw_put(
            octx,
            ocx,
            ocy,
            screens,
            RIGHT,
            frs,
            available.wrapping_sub(width_right),
            screens[RIGHT].cx.wrapping_sub(width_right),
            width_right,
        );
        format_draw_put(
            octx,
            ocx,
            ocy,
            screens,
            CENTRE,
            frs,
            width_left
                .wrapping_add(available.wrapping_sub(width_right).wrapping_sub(width_left) / 2)
                .wrapping_sub(width_centre / 2),
            (screens[CENTRE].cx / 2).wrapping_sub(width_centre / 2),
            width_centre,
        );

        if width_abs_centre > available {
            width_abs_centre = available;
        }
        format_draw_put(
            octx,
            ocx,
            ocy,
            screens,
            ABSOLUTE_CENTRE,
            frs,
            available.wrapping_sub(width_abs_centre) / 2,
            0,
            width_abs_centre,
        );
    }
}

/// Draws a format whose list is left-aligned: the list sits just after the
/// left, and what follows the list just after that.
unsafe fn format_draw_left(
    octx: &mut screen_write_ctx,
    available: u_int,
    ocx: u_int,
    ocy: u_int,
    screens: &mut [screen; TOTAL],
    mut focus_start: c_int,
    mut focus_end: c_int,
    frs: &mut Vec<format_range>,
) {
    unsafe {
        let mut width_left = screens[LEFT].cx;
        let mut width_centre = screens[CENTRE].cx;
        let mut width_right = screens[RIGHT].cx;
        let mut width_abs_centre = screens[ABSOLUTE_CENTRE].cx;
        let mut width_list = screens[LIST].cx;
        let mut width_after = screens[AFTER].cx;

        while width_left + width_centre + width_right + width_list + width_after > available {
            if width_centre > 0 {
                width_centre -= 1;
            } else if width_list > 0 {
                width_list -= 1;
            } else if width_right > 0 {
                width_right -= 1;
            } else if width_after > 0 {
                width_after -= 1;
            } else {
                width_left -= 1;
            }
        }

        if width_list == 0 {
            let mut ctx = screen_write_ctx::default();
            screen_write_start(&mut ctx, &mut screens[LEFT]);
            screen_write_fast_copy(&mut ctx, &screens[AFTER], 0, 0, width_after, 1);
            screen_write_stop(&mut ctx);
            format_draw_none(octx, available, ocx, ocy, screens, frs);
            return;
        }

        format_draw_put(octx, ocx, ocy, screens, LEFT, frs, 0, 0, width_left);
        format_draw_put(
            octx,
            ocx,
            ocy,
            screens,
            RIGHT,
            frs,
            available.wrapping_sub(width_right),
            screens[RIGHT].cx.wrapping_sub(width_right),
            width_right,
        );
        format_draw_put(
            octx,
            ocx,
            ocy,
            screens,
            AFTER,
            frs,
            width_left.wrapping_add(width_list),
            0,
            width_after,
        );

        let before = width_left
            .wrapping_add(width_list)
            .wrapping_add(width_after);
        format_draw_put(
            octx,
            ocx,
            ocy,
            screens,
            CENTRE,
            frs,
            before
                .wrapping_add(available.wrapping_sub(width_right).wrapping_sub(before) / 2)
                .wrapping_sub(width_centre / 2),
            (screens[CENTRE].cx / 2).wrapping_sub(width_centre / 2),
            width_centre,
        );

        if focus_start == -1 || focus_end == -1 {
            focus_end = 0;
            focus_start = 0;
        }
        format_draw_put_list(
            octx,
            ocx,
            ocy,
            width_left,
            width_list,
            screens,
            focus_start,
            focus_end,
            frs,
        );

        if width_abs_centre > available {
            width_abs_centre = available;
        }
        format_draw_put(
            octx,
            ocx,
            ocy,
            screens,
            ABSOLUTE_CENTRE,
            frs,
            available.wrapping_sub(width_abs_centre) / 2,
            0,
            width_abs_centre,
        );
    }
}

/// Draws a format whose list is centred: the list, the centre and what follows
/// the list are all placed around the middle of the room left over.
unsafe fn format_draw_centre(
    octx: &mut screen_write_ctx,
    available: u_int,
    ocx: u_int,
    ocy: u_int,
    screens: &mut [screen; TOTAL],
    mut focus_start: c_int,
    mut focus_end: c_int,
    frs: &mut Vec<format_range>,
) {
    unsafe {
        let mut width_left = screens[LEFT].cx;
        let mut width_centre = screens[CENTRE].cx;
        let mut width_right = screens[RIGHT].cx;
        let mut width_abs_centre = screens[ABSOLUTE_CENTRE].cx;
        let mut width_list = screens[LIST].cx;
        let mut width_after = screens[AFTER].cx;

        while width_left + width_centre + width_right + width_list + width_after > available {
            if width_list > 0 {
                width_list -= 1;
            } else if width_after > 0 {
                width_after -= 1;
            } else if width_centre > 0 {
                width_centre -= 1;
            } else if width_right > 0 {
                width_right -= 1;
            } else {
                width_left -= 1;
            }
        }

        if width_list == 0 {
            let mut ctx = screen_write_ctx::default();
            screen_write_start(&mut ctx, &mut screens[CENTRE]);
            screen_write_fast_copy(&mut ctx, &screens[AFTER], 0, 0, width_after, 1);
            screen_write_stop(&mut ctx);
            format_draw_none(octx, available, ocx, ocy, screens, frs);
            return;
        }

        format_draw_put(octx, ocx, ocy, screens, LEFT, frs, 0, 0, width_left);
        format_draw_put(
            octx,
            ocx,
            ocy,
            screens,
            RIGHT,
            frs,
            available.wrapping_sub(width_right),
            screens[RIGHT].cx.wrapping_sub(width_right),
            width_right,
        );

        let middle = width_left
            .wrapping_add(available.wrapping_sub(width_right).wrapping_sub(width_left) / 2);
        format_draw_put(
            octx,
            ocx,
            ocy,
            screens,
            CENTRE,
            frs,
            middle
                .wrapping_sub(width_list / 2)
                .wrapping_sub(width_centre),
            0,
            width_centre,
        );
        format_draw_put(
            octx,
            ocx,
            ocy,
            screens,
            AFTER,
            frs,
            middle.wrapping_sub(width_list / 2).wrapping_add(width_list),
            0,
            width_after,
        );

        if focus_start == -1 || focus_end == -1 {
            focus_end = (screens[LIST].cx / 2) as c_int;
            focus_start = focus_end;
        }
        format_draw_put_list(
            octx,
            ocx,
            ocy,
            middle.wrapping_sub(width_list / 2),
            width_list,
            screens,
            focus_start,
            focus_end,
            frs,
        );

        if width_abs_centre > available {
            width_abs_centre = available;
        }
        format_draw_put(
            octx,
            ocx,
            ocy,
            screens,
            ABSOLUTE_CENTRE,
            frs,
            available.wrapping_sub(width_abs_centre) / 2,
            0,
            width_abs_centre,
        );
    }
}

/// Draws a format whose list is right-aligned: what follows the list is at the
/// right-hand end, with the list just before it.
unsafe fn format_draw_right(
    octx: &mut screen_write_ctx,
    available: u_int,
    ocx: u_int,
    ocy: u_int,
    screens: &mut [screen; TOTAL],
    mut focus_start: c_int,
    mut focus_end: c_int,
    frs: &mut Vec<format_range>,
) {
    unsafe {
        let mut width_left = screens[LEFT].cx;
        let mut width_centre = screens[CENTRE].cx;
        let mut width_right = screens[RIGHT].cx;
        let mut width_abs_centre = screens[ABSOLUTE_CENTRE].cx;
        let mut width_list = screens[LIST].cx;
        let mut width_after = screens[AFTER].cx;

        while width_left + width_centre + width_right + width_list + width_after > available {
            if width_centre > 0 {
                width_centre -= 1;
            } else if width_list > 0 {
                width_list -= 1;
            } else if width_right > 0 {
                width_right -= 1;
            } else if width_after > 0 {
                width_after -= 1;
            } else {
                width_left -= 1;
            }
        }

        if width_list == 0 {
            let mut ctx = screen_write_ctx::default();
            screen_write_start(&mut ctx, &mut screens[RIGHT]);
            screen_write_fast_copy(&mut ctx, &screens[AFTER], 0, 0, width_after, 1);
            screen_write_stop(&mut ctx);
            format_draw_none(octx, available, ocx, ocy, screens, frs);
            return;
        }

        format_draw_put(octx, ocx, ocy, screens, LEFT, frs, 0, 0, width_left);
        format_draw_put(
            octx,
            ocx,
            ocy,
            screens,
            AFTER,
            frs,
            available.wrapping_sub(width_after),
            screens[AFTER].cx.wrapping_sub(width_after),
            width_after,
        );

        let before = available
            .wrapping_sub(width_right)
            .wrapping_sub(width_list)
            .wrapping_sub(width_after);
        format_draw_put(octx, ocx, ocy, screens, RIGHT, frs, before, 0, width_right);
        format_draw_put(
            octx,
            ocx,
            ocy,
            screens,
            CENTRE,
            frs,
            width_left
                .wrapping_add(before.wrapping_sub(width_left) / 2)
                .wrapping_sub(width_centre / 2),
            (screens[CENTRE].cx / 2).wrapping_sub(width_centre / 2),
            width_centre,
        );

        if focus_start == -1 || focus_end == -1 {
            focus_end = 0;
            focus_start = 0;
        }
        format_draw_put_list(
            octx,
            ocx,
            ocy,
            available.wrapping_sub(width_list).wrapping_sub(width_after),
            width_list,
            screens,
            focus_start,
            focus_end,
            frs,
        );

        if width_abs_centre > available {
            width_abs_centre = available;
        }
        format_draw_put(
            octx,
            ocx,
            ocy,
            screens,
            ABSOLUTE_CENTRE,
            frs,
            available.wrapping_sub(width_abs_centre) / 2,
            0,
            width_abs_centre,
        );
    }
}

/// Draws a format whose list is in the absolute centre: the list and the
/// absolute centre are placed together in the middle of the whole line, over
/// the rest, and are given up separately from it.
unsafe fn format_draw_absolute_centre(
    octx: &mut screen_write_ctx,
    available: u_int,
    ocx: u_int,
    ocy: u_int,
    screens: &mut [screen; TOTAL],
    mut focus_start: c_int,
    mut focus_end: c_int,
    frs: &mut Vec<format_range>,
) {
    unsafe {
        let mut width_left = screens[LEFT].cx;
        let mut width_centre = screens[CENTRE].cx;
        let mut width_right = screens[RIGHT].cx;
        let mut width_abs_centre = screens[ABSOLUTE_CENTRE].cx;
        let mut width_list = screens[LIST].cx;
        let mut width_after = screens[AFTER].cx;

        while width_left + width_centre + width_right > available {
            if width_centre > 0 {
                width_centre -= 1;
            } else if width_right > 0 {
                width_right -= 1;
            } else {
                width_left -= 1;
            }
        }
        while width_list + width_after + width_abs_centre > available {
            if width_list > 0 {
                width_list -= 1;
            } else if width_after > 0 {
                width_after -= 1;
            } else {
                width_abs_centre -= 1;
            }
        }

        format_draw_put(octx, ocx, ocy, screens, LEFT, frs, 0, 0, width_left);
        format_draw_put(
            octx,
            ocx,
            ocy,
            screens,
            RIGHT,
            frs,
            available.wrapping_sub(width_right),
            screens[RIGHT].cx.wrapping_sub(width_right),
            width_right,
        );

        let middle = width_left
            .wrapping_add(available.wrapping_sub(width_right).wrapping_sub(width_left) / 2);
        format_draw_put(
            octx,
            ocx,
            ocy,
            screens,
            CENTRE,
            frs,
            middle.wrapping_sub(width_centre),
            0,
            width_centre,
        );

        if focus_start == -1 || focus_end == -1 {
            focus_end = (screens[LIST].cx / 2) as c_int;
            focus_start = focus_end;
        }

        let mut offset = available
            .wrapping_sub(width_list)
            .wrapping_sub(width_abs_centre)
            / 2;
        format_draw_put(
            octx,
            ocx,
            ocy,
            screens,
            ABSOLUTE_CENTRE,
            frs,
            offset,
            0,
            width_abs_centre,
        );
        offset = offset.wrapping_add(width_abs_centre);
        format_draw_put_list(
            octx,
            ocx,
            ocy,
            offset,
            width_list,
            screens,
            focus_start,
            focus_end,
            frs,
        );
        offset = offset.wrapping_add(width_list);
        format_draw_put(octx, ocx, ocy, screens, AFTER, frs, offset, 0, width_after);
    }
}

/// The run of `#` at the front of `s`, which is written twice for each one it
/// stands for. Answers how far past them to carry on reading and how many
/// columns they take.
///
/// A run in front of a `[` is an escaped style: an even run leaves the reader
/// on the `[`, which is then read as text, and an odd one leaves it on the
/// last `#`, which is then read as the start of a real style. Callers only
/// reach here on a `#`, so the C's answer for a run of none is gone.
fn format_leading_hashes(s: &[u8]) -> (usize, u_int, u_int) {
    let n = s.iter().take_while(|&&byte| byte == b'#').count();
    if s.get(n) != Some(&b'[') {
        return (n, n as u_int, (n as u_int).div_ceil(2));
    }
    let width = n as u_int / 2;
    if n % 2 == 0 {
        (n, n as u_int, width)
    } else {
        (n - 1, n as u_int, width)
    }
}

/// The rest of a character whose first byte `utf8_open` took: appends bytes
/// until the character is whole or one of them is not part of it, leaving `at`
/// past everything it read, the way the C's loop did.
unsafe fn utf8_more(ud: &mut utf8_data, bytes: &[u8], at: &mut usize) -> utf8_state {
    unsafe {
        let mut more = UTF8_MORE;
        loop {
            *at += 1;
            if *at >= bytes.len() || more != UTF8_MORE {
                return more;
            }
            more = utf8_append(ud, bytes[*at]);
        }
    }
}

/// Draws `n` copies of `ch` in the current style.
unsafe fn format_draw_many(ctx: &mut screen_write_ctx, sy: &mut style, ch: u8, n: u_int) {
    unsafe {
        utf8_set(&mut sy.gc.data, ch);
        for _ in 0..n {
            screen_write_cell(ctx, &mut sy.gc);
        }
    }
}

/// An owned copy of `text`.
fn copy_of(text: &[u8]) -> CString {
    unsafe { CString::from_vec_unchecked(text.to_vec()) }
}

/// Draws an expanded format into `octx`, taking `available` columns of the
/// caller's line, and hands back the ranges of it a style asked to be marked.
///
/// The format is drawn into eight screens of its own first — one per alignment,
/// one for the list and its markers, and one for whatever follows the list —
/// and only then are the widths known and the screens copied onto the caller's
/// line. Which of the five drawing functions places them is decided by the
/// alignment in force when the list opened, so a list opened under no
/// alignment at all is not drawn.
pub unsafe fn format_draw(
    octx: &mut screen_write_ctx,
    base: &grid_cell,
    available: u_int,
    expanded: &[u8],
    srs: Option<&mut style_ranges>,
    default_colours: c_int,
) {
    unsafe {
        let mut map: [usize; 5] = [LEFT, LEFT, CENTRE, RIGHT, ABSOLUTE_CENTRE];
        let mut current = LEFT;
        let mut last = LEFT;
        let mut focus_start: c_int = -1;
        let mut focus_end: c_int = -1;
        let mut list_state: c_int = -1;
        let mut fill: c_int = -1;
        let mut list_align = STYLE_ALIGN_DEFAULT;

        let bytes = expanded;
        let os = octx.s;
        let (ocx, ocy) = ((*os).cx, (*os).cy);

        let mut base_default: grid_cell = *base;
        let mut current_default: grid_cell = *base;
        let mut sy = style::default();
        let mut saved_sy = style::default();
        style_set(&mut sy, &current_default);

        let mut frs = Vec::<format_range>::new();
        let mut fr: Option<format_range> = None;

        log_debug(
            c"%s: %.*s".as_ptr(),
            fmt_args![
                c"format_draw".as_ptr(),
                bytes.len() as c_int,
                bytes.as_ptr()
            ],
        );

        /*
         * Three screens for the left, right and centre alignments, one for the
         * absolute centre, one for the list, one for anything after the list
         * and two for the list's left and right markers.
         */
        let mut s: [screen; TOTAL] =
            ::core::array::from_fn(|_| screen::new(bytes.len() as u_int, 1, 0));
        let mut ctx = [screen_write_ctx::default(); TOTAL];
        let mut width: [u_int; TOTAL] = [0; TOTAL];
        for i in 0..TOTAL {
            screen_write_start(&mut ctx[i], &mut s[i]);
            screen_write_clearendofline(&mut ctx[i], current_default.bg as u_int);
        }

        let mut unterminated = false;
        let mut at = 0;
        while at < bytes.len() {
            /* A run of hashes, escaped or not. */
            if bytes[at] == b'#' && bytes.get(at + 1) != Some(&b'[') && at + 1 < bytes.len() {
                let n = bytes[at..].iter().take_while(|&&byte| byte == b'#').count() as u_int;
                let even = n.is_multiple_of(2);
                if bytes.get(at + n as usize) != Some(&b'[') {
                    at += n as usize;
                    let n = n.div_ceil(2);
                    width[current] += n;
                    format_draw_many(&mut ctx[current], &mut sy, b'#', n);
                    continue;
                }
                at += if even { n as usize + 1 } else { n as usize - 1 };
                if sy.ignore != 0 {
                    continue;
                }
                format_draw_many(&mut ctx[current], &mut sy, b'#', n / 2);
                width[current] += n / 2;
                if even {
                    utf8_set(&mut sy.gc.data, b'[');
                    screen_write_cell(&mut ctx[current], &mut sy.gc);
                    width[current] += 1;
                }
                continue;
            }

            /* Anything that is not the start of a style is a character. */
            if bytes[at] != b'#' || bytes.get(at + 1) != Some(&b'[') || sy.ignore != 0 {
                let mut more = utf8_open(&mut sy.gc.data, bytes[at]);
                if more == UTF8_MORE {
                    more = utf8_more(&mut sy.gc.data, bytes, &mut at);
                    if more != UTF8_DONE {
                        at -= sy.gc.data.have as usize;
                    }
                }
                if more != UTF8_DONE {
                    if bytes[at] < 0x20 || bytes[at] > 0x7e {
                        at += 1;
                        continue;
                    }
                    utf8_set(&mut sy.gc.data, bytes[at]);
                    at += 1;
                }
                screen_write_cell(&mut ctx[current], &mut sy.gc);
                width[current] += sy.gc.data.width as u_int;
                continue;
            }

            /* A style: find where it ends and read it. */
            let Some(after_style) = skip_style(bytes, at) else {
                let from = &bytes[at + 2..];
                log_debug(
                    c"%s: no terminating ] at '%.*s'".as_ptr(),
                    fmt_args![c"format_draw".as_ptr(), from.len() as c_int, from.as_ptr()],
                );
                frs.clear();
                for i in 0..TOTAL {
                    screen_write_stop(&mut ctx[i]);
                }
                unterminated = true;
                break;
            };

            let text = &bytes[at + 2..after_style - 1];
            style_copy(&mut saved_sy, &sy);
            if style_parse(&mut sy, &current_default, text) != 0 {
                log_debug(
                    c"%s: invalid style '%.*s'".as_ptr(),
                    fmt_args![c"format_draw".as_ptr(), text.len() as c_int, text.as_ptr()],
                );
                at = after_style;
                continue;
            }
            log_debug(
                c"%s: style '%.*s' -> '%s'".as_ptr(),
                fmt_args![
                    c"format_draw".as_ptr(),
                    text.len() as c_int,
                    text.as_ptr(),
                    style_tostring(&sy).as_c_str()
                ],
            );

            if default_colours != 0 {
                sy.gc.bg = base_default.bg;
                sy.gc.fg = base_default.fg;
            }
            if sy.fill != 8 {
                fill = sy.fill;
            }
            match sy.default_type {
                STYLE_DEFAULT_PUSH => {
                    current_default = saved_sy.gc;
                    sy.default_type = STYLE_DEFAULT_BASE;
                }
                STYLE_DEFAULT_POP => {
                    current_default = base_default;
                    sy.default_type = STYLE_DEFAULT_BASE;
                }
                STYLE_DEFAULT_SET => {
                    base_default = saved_sy.gc;
                    current_default = saved_sy.gc;
                    sy.default_type = STYLE_DEFAULT_BASE;
                }
                _ => {}
            }

            match sy.list {
                /* Entering the list, or leaving a marker or the focus. */
                STYLE_LIST_ON => {
                    if list_state != 0 {
                        fr = None;
                        list_state = 0;
                        list_align = sy.align;
                    }
                    if focus_start != -1 && focus_end == -1 {
                        focus_end = s[LIST].cx as c_int;
                    }
                    current = LIST;
                }
                /* Entering the focus. */
                STYLE_LIST_FOCUS => {
                    if list_state == 0 && focus_start == -1 {
                        focus_start = s[LIST].cx as c_int;
                    }
                }
                /* Leaving the list, or outside it. */
                STYLE_LIST_OFF => {
                    if list_state == 0 {
                        fr = None;
                        if focus_start != -1 && focus_end == -1 {
                            focus_end = s[LIST].cx as c_int;
                        }
                        map[list_align as usize] = AFTER;
                        if list_align == STYLE_ALIGN_LEFT {
                            map[STYLE_ALIGN_DEFAULT as usize] = AFTER;
                        }
                        list_state = 1;
                    }
                    current = map[sy.align as usize];
                }
                /*
                 * Entering one of the two markers, which is every value of
                 * the list style that is left. A marker is only taken while
                 * inside the list and only if it has nothing in it yet.
                 */
                _ => {
                    let marker = if sy.list == STYLE_LIST_LEFT_MARKER {
                        LIST_LEFT
                    } else {
                        LIST_RIGHT
                    };
                    if list_state == 0 && s[marker].cx == 0 {
                        fr = None;
                        if focus_start != -1 && focus_end == -1 {
                            focus_end = -1;
                            focus_start = -1;
                        }
                        current = marker;
                    }
                }
            }
            if current != last {
                log_debug(
                    c"%s: change %s -> %s".as_ptr(),
                    fmt_args![
                        c"format_draw".as_ptr(),
                        names[last].as_ptr(),
                        names[current].as_ptr()
                    ],
                );
                last = current;
            }

            /*
             * End the open range if the style is no longer the same one, and
             * open a new one if this style asks for it.
             */
            if srs.is_some() {
                if let Some(open) = &fr
                    && !format_is_type(open, &sy)
                {
                    let mut open = fr.take().expect("just seen");
                    if s[current].cx != open.start {
                        open.end = s[current].cx.wrapping_add(1);
                        frs.push(open);
                    }
                }
                if fr.is_none() && sy.range_type != STYLE_RANGE_NONE {
                    fr = Some(format_range {
                        index: current,
                        start: s[current].cx,
                        end: 0,
                        type_0: sy.range_type,
                        argument: sy.range_argument,
                        string: sy.range_string,
                    });
                }
            }

            at = after_style;
        }

        if !unterminated {
            for i in 0..TOTAL {
                screen_write_stop(&mut ctx[i]);
                log_debug(
                    c"%s: width %s is %u".as_ptr(),
                    fmt_args![c"format_draw".as_ptr(), names[i].as_ptr(), width[i]],
                );
            }
            if focus_start != -1 && focus_end != -1 {
                log_debug(
                    c"%s: focus %d-%d".as_ptr(),
                    fmt_args![c"format_draw".as_ptr(), focus_start, focus_end],
                );
            }
            for fr in &frs {
                log_debug(
                    c"%s: range %d|%u is %s %u-%u".as_ptr(),
                    fmt_args![
                        c"format_draw".as_ptr(),
                        fr.type_0,
                        fr.argument,
                        names[fr.index].as_ptr(),
                        fr.start,
                        fr.end
                    ],
                );
            }

            /* Clear the area the format is drawn into. */
            if fill != -1 {
                let mut gc = grid_default_cell;
                gc.bg = fill;
                for _ in 0..available {
                    screen_write_putc(octx, &mut gc, b' ');
                }
            }

            match list_align {
                STYLE_ALIGN_DEFAULT => {
                    format_draw_none(octx, available, ocx, ocy, &mut s, &mut frs)
                }
                STYLE_ALIGN_LEFT => format_draw_left(
                    octx,
                    available,
                    ocx,
                    ocy,
                    &mut s,
                    focus_start,
                    focus_end,
                    &mut frs,
                ),
                STYLE_ALIGN_CENTRE => format_draw_centre(
                    octx,
                    available,
                    ocx,
                    ocy,
                    &mut s,
                    focus_start,
                    focus_end,
                    &mut frs,
                ),
                STYLE_ALIGN_RIGHT => format_draw_right(
                    octx,
                    available,
                    ocx,
                    ocy,
                    &mut s,
                    focus_start,
                    focus_end,
                    &mut frs,
                ),
                _ => format_draw_absolute_centre(
                    octx,
                    available,
                    ocx,
                    ocy,
                    &mut s,
                    focus_start,
                    focus_end,
                    &mut frs,
                ),
            }

            if let Some(srs) = srs {
                for fr in frs.drain(..) {
                    let sr = style_range {
                        type_0: fr.type_0,
                        argument: fr.argument,
                        string: fr.string,
                        start: fr.start,
                        end: fr.end,
                    };
                    log_range(&sr);
                    srs.push(sr);
                }
            }
        }

        for i in 0..TOTAL {
            screen_free(&mut s[i]);
        }
        screen_write_cursormove(octx, ocx as c_int, ocy as c_int, 0);
    }
}

/// Says in the log what one finished range covers.
fn log_range(sr: &style_range) {
    unsafe {
        let kind = match sr.type_0 {
            STYLE_RANGE_LEFT => c"%s: range left at %u-%u",
            STYLE_RANGE_RIGHT => c"%s: range right at %u-%u",
            STYLE_RANGE_PANE => c"%s: range pane|%%%u at %u-%u",
            STYLE_RANGE_WINDOW => c"%s: range window|%u at %u-%u",
            STYLE_RANGE_SESSION => c"%s: range session|$%u at %u-%u",
            STYLE_RANGE_USER => c"%s: range user|%u at %u-%u",
            _ => c"%s: range control|%u at %u-%u",
        };
        if sr.type_0 == STYLE_RANGE_LEFT || sr.type_0 == STYLE_RANGE_RIGHT {
            log_debug(
                kind.as_ptr(),
                fmt_args![c"format_draw".as_ptr(), sr.start, sr.end],
            );
        } else {
            log_debug(
                kind.as_ptr(),
                fmt_args![c"format_draw".as_ptr(), sr.argument, sr.start, sr.end],
            );
        }
    }
}

/// How many columns an expanded format takes once its styles are left out.
/// A style that is never closed makes the whole of it nothing.
pub fn format_width(expanded: &[u8]) -> u_int {
    unsafe {
        let bytes = expanded;
        let mut ud = utf8_data::default();
        let mut width = 0;
        let mut at = 0;
        while at < bytes.len() {
            if bytes[at] == b'#' {
                let (step, _, leading_width) = format_leading_hashes(&bytes[at..]);
                width += leading_width;
                at += step;
                if bytes.get(at) == Some(&b'#') {
                    match skip_style(bytes, at) {
                        Some(next) => at = next,
                        None => return 0,
                    }
                }
                continue;
            }
            let more = utf8_open(&mut ud, bytes[at]);
            if more == UTF8_MORE {
                if utf8_more(&mut ud, bytes, &mut at) == UTF8_DONE {
                    width += ud.width as u_int;
                }
            } else if bytes[at] > 0x1f && bytes[at] < 0x7f {
                width += 1;
                at += 1;
            } else {
                at += 1;
            }
        }
        width
    }
}

/// Where the style starting at `at` ends, one past its `]`, or `None` if it
/// has none. A `#{...}` inside the style keeps its own `]` from ending it.
fn skip_style(bytes: &[u8], at: usize) -> Option<usize> {
    let mut brackets = 0;
    let mut i = at + 2;
    while i < bytes.len() {
        let next = bytes.get(i + 1).copied();
        if bytes[i] == b'#' && next == Some(b'{') {
            brackets += 1;
        }
        if bytes[i] == b'#' && next.is_some_and(|byte| b",#{}:".contains(&byte)) {
            i += 1;
        } else {
            if bytes[i] == b'}' {
                brackets -= 1;
            }
            if bytes[i] == b']' && brackets == 0 {
                return Some(i + 1);
            }
        }
        i += 1;
    }
    None
}

/// The first `limit` columns of an expanded format, styles and all.
pub fn format_trim_left(expanded: &[u8], limit: u_int) -> CString {
    unsafe {
        let bytes = expanded;
        let mut ud = utf8_data::default();
        let mut out = Vec::<u8>::new();
        let mut width = 0;
        let mut at = 0;
        while at < bytes.len() && width < limit {
            if bytes[at] == b'#' {
                let (step, n, mut leading_width) = format_leading_hashes(&bytes[at..]);
                leading_width = leading_width.min(limit - width);
                if leading_width != 0 {
                    if n == 1 {
                        out.push(b'#');
                    } else {
                        out.extend(::core::iter::repeat_n(b'#', 2 * leading_width as usize));
                    }
                    width += leading_width;
                }
                at += step;
                if bytes.get(at) != Some(&b'#') {
                    continue;
                }
                let Some(next) = skip_style(bytes, at) else {
                    break;
                };
                out.extend_from_slice(&bytes[at..next]);
                at = next;
                continue;
            }
            let more = utf8_open(&mut ud, bytes[at]);
            if more == UTF8_MORE {
                if utf8_more(&mut ud, bytes, &mut at) == UTF8_DONE {
                    if width + ud.width as u_int <= limit {
                        out.extend_from_slice(&ud.data[..ud.size as usize]);
                    }
                    width += ud.width as u_int;
                } else {
                    at -= ud.have as usize;
                    at += 1;
                }
            } else if bytes[at] > 0x1f && bytes[at] < 0x7f {
                if width < limit {
                    out.push(bytes[at]);
                }
                width += 1;
                at += 1;
            } else {
                at += 1;
            }
        }
        copy_of(&out)
    }
}

/// The last `limit` columns of an expanded format, styles and all. A format
/// that is no wider than the limit is handed back whole, which includes one
/// carrying a style that is never closed, since that has no width at all.
pub fn format_trim_right(expanded: &[u8], limit: u_int) -> CString {
    unsafe {
        let total_width = format_width(expanded);
        if total_width <= limit {
            return copy_of(expanded);
        }
        let skip = total_width - limit;

        let bytes = expanded;
        let mut ud = utf8_data::default();
        let mut out = Vec::<u8>::new();
        let mut width = 0;
        let mut at = 0;
        while at < bytes.len() {
            if bytes[at] == b'#' {
                let (step, n, leading_width) = format_leading_hashes(&bytes[at..]);
                let mut copy_width = leading_width;
                if width <= skip {
                    copy_width = copy_width.saturating_sub(skip - width);
                }
                if copy_width != 0 {
                    if n == 1 {
                        out.push(b'#');
                    } else {
                        out.extend(::core::iter::repeat_n(b'#', 2 * copy_width as usize));
                    }
                }
                width += leading_width;
                at += step;
                if bytes.get(at) != Some(&b'#') {
                    continue;
                }
                /*
                 * A style that never ends has already made the whole format
                 * nothing wide, which was handed back above, so `skip_style`
                 * always finds the end here.
                 */
                let next = skip_style(bytes, at).expect("the format has a width");
                out.extend_from_slice(&bytes[at..next]);
                at = next;
                continue;
            }
            let more = utf8_open(&mut ud, bytes[at]);
            if more == UTF8_MORE {
                if utf8_more(&mut ud, bytes, &mut at) == UTF8_DONE {
                    if width >= skip {
                        out.extend_from_slice(&ud.data[..ud.size as usize]);
                    }
                    width += ud.width as u_int;
                } else {
                    at -= ud.have as usize;
                    at += 1;
                }
            } else if bytes[at] > 0x1f && bytes[at] < 0x7f {
                if width >= skip {
                    out.push(bytes[at]);
                }
                width += 1;
                at += 1;
            } else {
                at += 1;
            }
        }
        copy_of(&out)
    }
}
#[cfg(test)]
#[path = "../tests/test_format_draw.rs"]
mod tests;
