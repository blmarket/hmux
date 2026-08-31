use super::write::{screen_write_free_list, screen_write_make_list};
use crate::fmt_args;
use crate::grid::HyperlinksRef;
use crate::grid::{
    grid_adjust_lines, grid_clear_lines, grid_create, grid_default_cell, grid_duplicate_lines,
    grid_empty_line, grid_reflow, grid_unwrap_position, grid_wrap_position,
};
use crate::grid::{grid_view_clear, grid_view_delete_lines};
use crate::log::{fatalx, log_debug};
use crate::options::options_get_number;
use crate::text::{utf8_copy, utf8_to_data};
use crate::tmux::clean_name;
use crate::tmux::global_options;
pub use crate::types::*;
use ::core::ffi::{c_char, c_int};
use ::core::ptr::null_mut;
use ::std::collections::VecDeque;
use ::std::ffi::CString;
pub const PROGRESS_BAR_PAUSED: progress_bar_state = 4;
pub const PROGRESS_BAR_INDETERMINATE: progress_bar_state = 3;
pub const PROGRESS_BAR_ERROR: progress_bar_state = 2;
pub const PROGRESS_BAR_NORMAL: progress_bar_state = 1;
pub const PROGRESS_BAR_HIDDEN: progress_bar_state = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct screen_sel {
    pub hidden: ::core::ffi::c_int,
    pub rectangle: ::core::ffi::c_int,
    pub modekeys: ::core::ffi::c_int,
    pub sx: u_int,
    pub sy: u_int,
    pub ex: u_int,
    pub ey: u_int,
    pub clipx: u_int,
    pub cell: grid_cell,
}
pub const SCREEN_CURSOR_BAR: screen_cursor_style = 3;
pub const SCREEN_CURSOR_UNDERLINE: screen_cursor_style = 2;
pub const SCREEN_CURSOR_BLOCK: screen_cursor_style = 1;
pub const SCREEN_CURSOR_DEFAULT: screen_cursor_style = 0;
pub const UINT_MAX: ::core::ffi::c_uint = u_int::MAX;
pub const MODEKEY_EMACS: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const MODE_CURSOR: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const MODE_INSERT: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MODE_KCURSOR: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const MODE_KKEYPAD: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const MODE_WRAP: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const MODE_MOUSE_STANDARD: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const MODE_MOUSE_BUTTON: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const MODE_CURSOR_BLINKING: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const MODE_MOUSE_UTF8: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const MODE_MOUSE_SGR: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const MODE_BRACKETPASTE: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const MODE_FOCUSON: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const MODE_MOUSE_ALL: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const MODE_ORIGIN: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const MODE_CRLF: ::core::ffi::c_int = 0x4000 as ::core::ffi::c_int;
pub const MODE_KEYS_EXTENDED: ::core::ffi::c_int = 0x8000 as ::core::ffi::c_int;
pub const MODE_CURSOR_VERY_VISIBLE: ::core::ffi::c_int = 0x10000 as ::core::ffi::c_int;
pub const MODE_CURSOR_BLINKING_SET: ::core::ffi::c_int = 0x20000 as ::core::ffi::c_int;
pub const MODE_KEYS_EXTENDED_2: ::core::ffi::c_int = 0x40000 as ::core::ffi::c_int;
pub const MODE_THEME_UPDATES: ::core::ffi::c_int = 0x80000 as ::core::ffi::c_int;
pub const MODE_SYNC: ::core::ffi::c_int = 0x100000 as ::core::ffi::c_int;
pub const ALL_MODES: ::core::ffi::c_int = 0xffffff as ::core::ffi::c_int;
pub const EXTENDED_KEY_MODES: ::core::ffi::c_int = MODE_KEYS_EXTENDED | MODE_KEYS_EXTENDED_2;
pub const GRID_ATTR_CHARSET: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const GRID_ATTR_NOATTR: ::core::ffi::c_int = 0x4000 as ::core::ffi::c_int;
pub const GRID_FLAG_PADDING: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const GRID_FLAG_EXTENDED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const GRID_FLAG_TAB: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const GRID_HISTORY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;

/// The titles a screen has pushed, most recently pushed first.
pub struct screen_titles {
    stack: VecDeque<CString>,
}

/// How many titles the stack keeps before the oldest are dropped.
const TITLE_LIMIT: u_int = 10;

/// The title stack of a screen, made on the first push.
unsafe fn screen_titles_of<'a>(s: *mut screen) -> &'a mut screen_titles {
    unsafe {
        &mut *screen_titles_ptr(s).unwrap_or_else(|| {
            &raw mut **(*s).titles.insert(Box::new(screen_titles {
                stack: VecDeque::new(),
            }))
        })
    }
}

/// The title stack a screen carries, if it has one yet.
unsafe fn screen_titles_ptr(s: *mut screen) -> Option<*mut screen_titles> {
    unsafe { (*s).titles.as_deref_mut().map(|titles| &raw mut *titles) }
}

/// The grid a screen holds. A screen is only asked for it once `screen_init`
/// has given it one.
pub(crate) fn screen_grid(s: &screen) -> &grid {
    s.grid.as_deref().expect("a screen holds a grid")
}

/// The same grid, to write to.
pub(crate) fn screen_grid_mut(s: &mut screen) -> &mut grid {
    s.grid.as_deref_mut().expect("a screen holds a grid")
}

pub(crate) unsafe fn screen_grid_ptr(s: *mut screen) -> *mut grid {
    unsafe {
        (*s).grid
            .as_deref_mut()
            .map(|grid| &raw mut *grid)
            .unwrap_or(null_mut::<grid>())
    }
}

pub(crate) unsafe fn screen_saved_grid_ptr(s: *mut screen) -> *mut grid {
    unsafe {
        (*s).saved_grid
            .as_deref_mut()
            .map(|grid| &raw mut *grid)
            .unwrap_or(null_mut::<grid>())
    }
}

/// Throw the title stack away. The count of titles is deliberately left as
/// it was, which is what the C did.
unsafe fn screen_free_titles(s: *mut screen) {
    unsafe {
        (*s).titles = None;
    }
}

impl Default for screen {
    /// A screen holding nothing: no grid, no title and no tab stops. This is
    /// what the C left on the stack for `screen_init` to fill in, and since it
    /// owns nothing, overwriting it frees nothing.
    fn default() -> screen {
        screen {
            title: None,
            path: None,
            titles: None,
            ntitles: 0,
            grid: None,
            cx: 0,
            cy: 0,
            cstyle: SCREEN_CURSOR_DEFAULT,
            default_cstyle: SCREEN_CURSOR_DEFAULT,
            ccolour: 0,
            default_ccolour: 0,
            rupper: 0,
            rlower: 0,
            mode: 0,
            default_mode: 0,
            saved_cx: 0,
            saved_cy: 0,
            saved_grid: None,
            saved_cell: grid_default_cell,
            saved_flags: 0,
            tabs: Vec::new(),
            sel: None,
            write_list: Vec::new(),
            hyperlinks: None,
            progress_bar: progress_bar {
                state: PROGRESS_BAR_HIDDEN,
                progress: 0,
            },
        }
    }
}

impl screen {
    /// A screen of `sx` by `sy` carrying `hlimit` lines of history. It starts
    /// as the empty screen and names only the fields a new screen differs on,
    /// then resets itself the way the C `screen_init` did.
    ///
    /// # Safety
    ///
    /// The global option set must be there, since resetting the screen reads
    /// `extended-keys` from it.
    pub fn new(sx: u_int, sy: u_int, hlimit: u_int) -> screen {
        unsafe {
            let mut s = screen {
                grid: Some(grid_create(sx, sy, hlimit)),
                title: Some(c"".to_owned()),
                mode: MODE_CURSOR,
                ccolour: -1,
                default_ccolour: -1,
                ..screen::default()
            };
            screen_reinit(&raw mut s);
            s
        }
    }

    /// Returns a temporary raw view of the hyperlink set, if this screen owns
    /// one. A caller must keep this screen, or another cloned handle, alive
    /// until the raw pointer is no longer used.
    pub(crate) fn hyperlinks_ptr(&self) -> *mut hyperlinks {
        self.hyperlinks
            .as_ref()
            .map_or(null_mut(), HyperlinksRef::as_ptr)
    }

    /// The screen's hyperlink set, if it owns one.
    pub(crate) fn hyperlinks_ref(&self) -> Option<&HyperlinksRef> {
        self.hyperlinks.as_ref()
    }
}

/// Puts a new screen where `s` points. The memory it points at must be
/// uninitialised, or have been freed or moved out of already: whatever was
/// there is written over without being given up first.
pub unsafe fn screen_init(s: *mut screen, sx: u_int, sy: u_int, hlimit: u_int) {
    unsafe { ::core::ptr::write(s, screen::new(sx, sy, hlimit)) }
}

/// Reset a screen to what a new one is, keeping its size and its history.
pub unsafe fn screen_reinit(s: *mut screen) {
    unsafe {
        let gd = screen_grid_ptr(s);
        (*s).cx = 0;
        (*s).cy = 0;
        (*s).rupper = 0;
        (*s).rlower = (*gd).sy.wrapping_sub(1);

        (*s).mode = MODE_CURSOR | MODE_WRAP | ((*s).mode & MODE_CRLF);

        if options_get_number(global_options, c"extended-keys".as_ptr()) == 2 {
            (*s).mode = ((*s).mode & !EXTENDED_KEY_MODES) | MODE_KEYS_EXTENDED;
        }

        if (*s).saved_grid.is_some() {
            screen_alternate_off(s, None, 0);
        }
        (*s).saved_cx = UINT_MAX;
        (*s).saved_cy = UINT_MAX;

        screen_reset_tabs(s);

        grid_clear_lines(&mut *gd, (*gd).hsize, (*gd).sy, 8);

        screen_clear_selection(s);
        screen_free_titles(s);
        screen_set_progress_bar(s, PROGRESS_BAR_HIDDEN, 0);
        screen_reset_hyperlinks(s);
    }
}

pub unsafe fn screen_reset_hyperlinks(s: *mut screen) {
    unsafe {
        if let Some(hl) = (*s).hyperlinks.as_ref() {
            hl.reset();
        } else {
            (*s).hyperlinks = Some(HyperlinksRef::new());
        }
    }
}

pub unsafe fn screen_free(s: *mut screen) {
    unsafe {
        (*s).sel = None;
        drop(::core::mem::take(&mut (*s).tabs));
        (*s).path = None;
        (*s).title = None;

        if !(*s).write_list.is_empty() {
            screen_write_free_list(s);
        }

        (*s).saved_grid.take();
        (*s).grid.take();

        let _ = (*s).hyperlinks.take();
        screen_free_titles(s);
    }
}

/// Put a tab stop every eight columns.
pub unsafe fn screen_reset_tabs(s: *mut screen) {
    unsafe {
        let sx = (*screen_grid_ptr(s)).sx;
        (*s).tabs = vec![0; ((sx + 7) >> 3) as usize];
        let mut i = 8;
        while i < sx {
            let byte = &mut (&mut (*s).tabs)[(i >> 3) as usize];
            *byte = (*byte as c_int | 1 << (i & 0x7)) as u8;
            i += 8;
        }
    }
}

pub unsafe fn screen_set_default_cursor(s: *mut screen, oo: *mut options) {
    unsafe {
        (*s).default_ccolour = options_get_number(oo, c"cursor-colour".as_ptr()) as c_int;

        let style = options_get_number(oo, c"cursor-style".as_ptr()) as u_int;
        (*s).default_mode = 0;
        screen_set_cursor_style(
            style,
            &mut (*s).default_cstyle,
            &mut (*s).default_mode,
        );
    }
}

/// Turn a cursor style number into a shape and whether it blinks.
pub fn screen_set_cursor_style(
    style: u_int,
    cstyle: &mut screen_cursor_style,
    mode: &mut c_int,
) {
    let (shape, blinking) = match style {
        0 => (SCREEN_CURSOR_DEFAULT, None),
        1 => (SCREEN_CURSOR_BLOCK, Some(true)),
        2 => (SCREEN_CURSOR_BLOCK, Some(false)),
        3 => (SCREEN_CURSOR_UNDERLINE, Some(true)),
        4 => (SCREEN_CURSOR_UNDERLINE, Some(false)),
        5 => (SCREEN_CURSOR_BAR, Some(true)),
        6 => (SCREEN_CURSOR_BAR, Some(false)),
        _ => return,
    };
    *cstyle = shape;
    match blinking {
        Some(true) => *mode |= MODE_CURSOR_BLINKING,
        Some(false) => *mode &= !MODE_CURSOR_BLINKING,
        None => {}
    }
}

pub unsafe fn screen_set_cursor_colour(s: *mut screen, colour: c_int) {
    unsafe { (*s).ccolour = colour }
}

/// Set the title, unless the name it is given cannot be cleaned up.
pub unsafe fn screen_set_title(s: *mut screen, title: *const c_char, untrusted: c_int) -> c_int {
    unsafe {
        let Some(new_title) = clean_name(title, untrusted) else {
            return 0;
        };
        (*s).title = Some(new_title);
        1
    }
}

/// Set the path, unless the name it is given cannot be cleaned up.
pub unsafe fn screen_set_path(s: *mut screen, path: *const c_char, untrusted: c_int) -> c_int {
    unsafe {
        let Some(new_path) = clean_name(path, untrusted) else {
            return 0;
        };
        (*s).path = Some(new_path);
        1
    }
}

/// Push the current title onto the stack, dropping the oldest once the stack
/// is full.
pub unsafe fn screen_push_title(s: *mut screen) {
    unsafe {
        log_debug(
            c"%s: %u".as_ptr(),
            fmt_args![c"screen_push_title".as_ptr(), (*s).ntitles],
        );

        while (*s).ntitles >= TITLE_LIMIT {
            /*
             * The C reached for the stack before making sure there was one,
             * so a screen that was reset while its stack was full follows a
             * null pointer here, which is kept as it was.
             */
            let titles = &mut *screen_titles_ptr(s).unwrap_or(null_mut::<screen_titles>());
            titles.stack.pop_back();
            (*s).ntitles -= 1;
        }

        let title = (*s).title.clone().expect("a screen always carries a title");
        screen_titles_of(s).stack.push_front(title);
        (*s).ntitles += 1;
    }
}

/// Take the title back off the stack.
pub unsafe fn screen_pop_title(s: *mut screen) {
    unsafe {
        let Some(titles) = screen_titles_ptr(s) else {
            return;
        };
        log_debug(
            c"%s: %u".as_ptr(),
            fmt_args![c"screen_pop_title".as_ptr(), (*s).ntitles],
        );

        if let Some(text) = (*titles).stack.pop_front() {
            (*s).title = Some(text);
            (*s).ntitles -= 1;
        }
    }
}

/// Set the progress bar, keeping the progress it had when there is none to
/// set.
pub unsafe fn screen_set_progress_bar(s: *mut screen, pbs: progress_bar_state, p: c_int) {
    unsafe {
        (*s).progress_bar.state = pbs;
        if p >= 0 && pbs != PROGRESS_BAR_INDETERMINATE {
            (*s).progress_bar.progress = p;
        }
    }
}

/// Resize a screen, keeping the cell the cursor is on.
pub unsafe fn screen_resize_cursor(
    s: *mut screen,
    sx: u_int,
    sy: u_int,
    reflow: c_int,
    eat_empty: c_int,
    cursor: c_int,
) {
    unsafe {
        let gd = screen_grid_ptr(s);
        let mut cx = (*s).cx;
        let mut cy = (*gd).hsize + (*s).cy;

        let collecting = !(*s).write_list.is_empty();
        if collecting {
            screen_write_free_list(s);
        }

        log_debug(
            c"%s: new size %ux%u, now %ux%u (cursor %u,%u = %u,%u)".as_ptr(),
            fmt_args![
                c"screen_resize_cursor".as_ptr(),
                sx,
                sy,
                (*gd).sx,
                (*gd).sy,
                (*s).cx,
                (*s).cy,
                cx,
                cy
            ],
        );

        let sx = sx.max(1);
        let sy = sy.max(1);

        let mut reflow = reflow;
        if sx != (*gd).sx {
            (*gd).sx = sx;
            screen_reset_tabs(s);
        } else {
            reflow = 0;
        }
        if sy != (*gd).sy {
            screen_resize_y(s, sy, eat_empty, &mut cy);
        }

        if reflow != 0 {
            screen_reflow(s, sx, &mut cx, &mut cy, cursor);
        }

        if cy >= (*gd).hsize {
            (*s).cx = cx;
            (*s).cy = cy - (*gd).hsize;
        } else {
            (*s).cx = 0;
            (*s).cy = 0;
        }

        log_debug(
            c"%s: cursor finished at %u,%u = %u,%u".as_ptr(),
            fmt_args![c"screen_resize_cursor".as_ptr(), (*s).cx, (*s).cy, cx, cy],
        );

        if collecting {
            screen_write_make_list(s);
        }
    }
}

pub unsafe fn screen_resize(s: *mut screen, sx: u_int, sy: u_int, reflow: c_int) {
    unsafe { screen_resize_cursor(s, sx, sy, reflow, 1, 1) }
}

/// Give the screen a new height, moving lines into and out of the history as
/// the new size needs.
unsafe fn screen_resize_y(s: *mut screen, sy: u_int, eat_empty: c_int, cy: &mut u_int) {
    unsafe {
        let gd = screen_grid_ptr(s);
        if sy == 0 {
            fatalx(c"zero size".as_ptr(), fmt_args![]);
        }
        let oldy = (*gd).sy;

        /*
         * When getting smaller, nuke any empty lines at the bottom of the
         * screen, then move the rest into the history or delete them.
         */
        if sy < oldy {
            let mut needed = oldy - sy;

            if eat_empty != 0 {
                let mut available = oldy.wrapping_sub(1).wrapping_sub((*s).cy);
                if available > 0 {
                    if available > needed {
                        available = needed;
                    }
                    grid_view_delete_lines(&mut *gd, oldy - available, available, 8);
                }
                needed -= available;
            }

            let mut available = (*s).cy;
            if (*gd).flags & GRID_HISTORY != 0 {
                (*gd).hscrolled += needed;
                (*gd).hsize += needed;
            } else if needed > 0 && available > 0 {
                if available > needed {
                    available = needed;
                }
                grid_view_delete_lines(&mut *gd, 0, available, 8);
                *cy = cy.wrapping_sub(available);
            }
        }

        /* Resize the historic data. */
        grid_adjust_lines(&mut *gd, (*gd).hsize + sy);

        /* When getting larger, take lines from the history if there are any. */
        if sy > oldy {
            let mut needed = sy - oldy;
            let mut available = (*gd).hscrolled;
            if (*gd).flags & GRID_HISTORY != 0 && available > 0 {
                if available > needed {
                    available = needed;
                }
                (*gd).hscrolled -= available;
                (*gd).hsize -= available;
            } else {
                available = 0;
            }
            needed -= available;

            for i in (*gd).hsize + sy - needed..(*gd).hsize + sy {
                grid_empty_line(&mut *gd, i, 8);
            }
        }

        (*gd).sy = sy;
        (*s).rupper = 0;
        (*s).rlower = (*gd).sy.wrapping_sub(1);
    }
}

pub unsafe fn screen_set_selection(
    s: *mut screen,
    sx: u_int,
    sy: u_int,
    ex: u_int,
    ey: u_int,
    rectangle: u_int,
    clipx: u_int,
    modekeys: c_int,
    gc: *mut grid_cell,
) {
    unsafe {
        (*s).sel = Some(Box::new(screen_sel {
            cell: *gc,
            hidden: 0,
            rectangle: rectangle as c_int,
            modekeys,
            sx,
            sy,
            ex,
            ey,
            clipx,
        }));
    }
}

pub unsafe fn screen_clear_selection(s: *mut screen) {
    unsafe {
        (*s).sel = None;
    }
}

pub unsafe fn screen_hide_selection(s: *mut screen) {
    unsafe {
        if let Some(sel) = (*s).sel.as_mut() {
            sel.hidden = 1;
        }
    }
}

/// Where a selection ends: with emacs keys the cell the cursor is on is not
/// part of it.
fn selection_end(sel: &screen_sel, x: u_int) -> u_int {
    if sel.modekeys == MODEKEY_EMACS && x != 0 {
        x - 1
    } else if sel.modekeys == MODEKEY_EMACS {
        0
    } else {
        x
    }
}

/// The same for a selection that was made upwards, where the caller has
/// already turned down a start in the first column.
fn selection_start(sel: &screen_sel) -> u_int {
    if sel.modekeys == MODEKEY_EMACS {
        sel.sx.wrapping_sub(1)
    } else {
        sel.sx
    }
}

/// Whether a cell is inside the selection.
unsafe fn in_selection(s: *mut screen, px: u_int, py: u_int) -> bool {
    unsafe {
        let sel = match (*s).sel.as_ref() {
            None => return false,
            Some(sel) => sel,
        };
        if sel.hidden != 0 {
            return false;
        }
        if px < sel.clipx {
            return false;
        }

        if sel.rectangle != 0 {
            if sel.sy < sel.ey {
                /* start line < end line -- downward selection. */
                if py < sel.sy || py > sel.ey {
                    return false;
                }
            } else if sel.sy > sel.ey {
                /* start line > end line -- upward selection. */
                if py > sel.sy || py < sel.ey {
                    return false;
                }
            } else if py != sel.sy {
                /* starting line == ending line. */
                return false;
            }

            /*
             * Need to include the selection start row, but not the cursor
             * row, which means the selection changes depending on which way
             * it is drawn.
             */
            if sel.ex < sel.sx {
                return px >= sel.ex && px <= sel.sx;
            }
            return px >= sel.sx && px <= sel.ex;
        }

        if sel.sy < sel.ey {
            /* starting line < ending line -- downward selection. */
            if py < sel.sy || py > sel.ey {
                return false;
            }
            if py == sel.sy && px < sel.sx {
                return false;
            }
            return !(py == sel.ey && px > selection_end(sel, sel.ex));
        }
        if sel.sy > sel.ey {
            /* starting line > ending line -- upward selection. */
            if py > sel.sy || py < sel.ey {
                return false;
            }
            if py == sel.ey && px < sel.ex {
                return false;
            }
            return !(py == sel.sy && (sel.sx == 0 || px > selection_start(sel)));
        }

        /* starting line == ending line. */
        if py != sel.sy {
            return false;
        }
        if sel.ex < sel.sx {
            /* cursor (ex) is on the left. */
            return px <= selection_start(sel) && px >= sel.ex;
        }
        /* selection start (sx) is on the left. */
        px >= sel.sx && px <= selection_end(sel, sel.ex)
    }
}

pub unsafe fn screen_check_selection(s: *mut screen, px: u_int, py: u_int) -> c_int {
    unsafe { in_selection(s, px, py) as c_int }
}

/// Draw a cell the way the selection asks for, keeping what the selection
/// leaves to the cell itself.
pub unsafe fn screen_select_cell(
    s: *mut screen,
    dst: *mut grid_cell,
    src: *const grid_cell,
) -> c_int {
    unsafe {
        let sel = match (*s).sel.as_ref() {
            None => return 0,
            Some(sel) => sel,
        };
        if sel.hidden != 0 {
            return 0;
        }

        *dst = sel.cell;

        let src = &*src;
        let dst = &mut *dst;
        if dst.fg == 8 || dst.fg == 9 {
            dst.fg = src.fg;
        }
        if dst.bg == 8 || dst.bg == 9 {
            dst.bg = src.bg;
        }

        utf8_copy(&mut dst.data, &src.data);
        dst.flags = src.flags;
        let keep = if dst.attr as c_int & GRID_ATTR_NOATTR != 0 {
            src.attr as c_int & GRID_ATTR_CHARSET
        } else {
            src.attr as c_int
        };
        dst.attr = (dst.attr as c_int | keep) as u_short;
        1
    }
}

/// Reflow the grid to a new width, following the cell the cursor is on.
unsafe fn screen_reflow(
    s: *mut screen,
    new_x: u_int,
    cx: &mut u_int,
    cy: &mut u_int,
    cursor: c_int,
) {
    unsafe {
        let gd = screen_grid_ptr(s);
        let (mut wx, mut wy) = (0, 0);
        if cursor != 0 {
            (wx, wy) = grid_wrap_position(&*gd, *cx, *cy);
            log_debug(
                c"%s: cursor %u,%u is %u,%u".as_ptr(),
                fmt_args![c"screen_reflow".as_ptr(), *cx, *cy, wx, wy],
            );
        }

        grid_reflow(&mut *gd, new_x);

        if cursor != 0 {
            (*cx, *cy) = grid_unwrap_position(&*gd, wx, wy);
            log_debug(
                c"%s: new cursor is %u,%u".as_ptr(),
                fmt_args![c"screen_reflow".as_ptr(), *cx, *cy],
            );
        } else {
            *cx = 0;
            *cy = (*gd).hsize;
        }
    }
}

/// Put the screen aside and start on an empty one.
pub unsafe fn screen_alternate_on(s: *mut screen, gc: &grid_cell, cursor: c_int) {
    unsafe {
        if (*s).saved_grid.is_some() {
            return;
        }
        let gd = screen_grid_ptr(s);
        let sx = (*gd).sx;
        let sy = (*gd).sy;

        (*s).saved_grid = Some(grid_create(sx, sy, 0));
        let saved_gd = screen_saved_grid_ptr(s);
        grid_duplicate_lines(&mut *saved_gd, 0, &*gd, (*gd).hsize, sy);
        if cursor != 0 {
            (*s).saved_cx = (*s).cx;
            (*s).saved_cy = (*s).cy;
        }
        (*s).saved_cell = *gc;

        grid_view_clear(&mut *gd, 0, 0, sx, sy, 8);

        (*s).saved_flags = (*gd).flags;
        (*gd).flags &= !GRID_HISTORY;
    }
}

/// Take the screen that was put aside back.
pub unsafe fn screen_alternate_off(s: *mut screen, gc: Option<&mut grid_cell>, cursor: c_int) {
    unsafe {
        let gd = screen_grid_ptr(s);
        let sx = (*gd).sx;
        let sy = (*gd).sy;

        /*
         * If the current size is different, temporarily resize to the old
         * size before copying back.
         */
        if let Some(saved_grid) = (*s).saved_grid.as_ref() {
            screen_resize(s, saved_grid.sx, saved_grid.sy, 0);
        }

        /*
         * Restore the cursor position and cell. This happens even if not
         * currently in the alternate screen.
         */
        if cursor != 0 && (*s).saved_cx != UINT_MAX && (*s).saved_cy != UINT_MAX {
            (*s).cx = (*s).saved_cx;
            (*s).cy = (*s).saved_cy;
            if let Some(gc) = gc {
                *gc = (*s).saved_cell;
            }
        }

        /* If not in the alternate screen, do nothing more. */
        if (*s).saved_grid.is_none() {
            screen_clamp_cursor(s);
            return;
        }

        /* Restore the saved grid. */
        let saved_gd = screen_saved_grid_ptr(s);
        grid_duplicate_lines(&mut *gd, (*gd).hsize, &*saved_gd, 0, (*saved_gd).sy);

        /*
         * Turn history back on (so resize can use it) and then resize back to
         * the current size.
         */
        if (*s).saved_flags & GRID_HISTORY != 0 {
            (*gd).flags |= GRID_HISTORY;
        }
        screen_resize(s, sx, sy, 1);

        (*s).saved_grid.take();

        screen_clamp_cursor(s);
    }
}

/// Keep the cursor inside the screen.
unsafe fn screen_clamp_cursor(s: *mut screen) {
    unsafe {
        let gd = screen_grid_ptr(s);
        if (*s).cx > (*gd).sx - 1 {
            (*s).cx = (*gd).sx - 1;
        }
        if (*s).cy > (*gd).sy - 1 {
            (*s).cy = (*gd).sy - 1;
        }
    }
}

/// The modes, in the order they are named.
const MODES: [(c_int, &str); 21] = [
    (MODE_CURSOR, "CURSOR"),
    (MODE_INSERT, "INSERT"),
    (MODE_KCURSOR, "KCURSOR"),
    (MODE_KKEYPAD, "KKEYPAD"),
    (MODE_WRAP, "WRAP"),
    (MODE_MOUSE_STANDARD, "MOUSE_STANDARD"),
    (MODE_MOUSE_BUTTON, "MOUSE_BUTTON"),
    (MODE_CURSOR_BLINKING, "CURSOR_BLINKING"),
    (MODE_CURSOR_VERY_VISIBLE, "CURSOR_VERY_VISIBLE"),
    (MODE_CURSOR_BLINKING_SET, "CURSOR_BLINKING_SET"),
    (MODE_MOUSE_UTF8, "MOUSE_UTF8"),
    (MODE_MOUSE_SGR, "MOUSE_SGR"),
    (MODE_BRACKETPASTE, "BRACKETPASTE"),
    (MODE_FOCUSON, "FOCUSON"),
    (MODE_MOUSE_ALL, "MOUSE_ALL"),
    (MODE_ORIGIN, "ORIGIN"),
    (MODE_CRLF, "CRLF"),
    (MODE_KEYS_EXTENDED, "KEYS_EXTENDED"),
    (MODE_KEYS_EXTENDED_2, "KEYS_EXTENDED_2"),
    (MODE_THEME_UPDATES, "THEME_UPDATES"),
    (MODE_SYNC, "SYNC"),
];

/// The names of the modes that are set, in a buffer that lasts until the next
/// call.
/// The modes `mode` carries, comma-separated, as the caller's own string.
pub fn screen_mode_to_string(mode: c_int) -> ::std::ffi::CString {
    if mode == 0 {
        return c"NONE".to_owned();
    }
    if mode == ALL_MODES {
        return c"ALL".to_owned();
    }
    let text = MODES
        .iter()
        .filter(|(bit, _)| mode & bit != 0)
        .map(|(_, name)| *name)
        .collect::<Vec<_>>()
        .join(",");
    ::std::ffi::CString::new(text).expect("a mode name has no interior NUL")
}

/// How much room the printed lines get.
const PRINT_SIZE: usize = 16384;

/// The lines of a screen written out one per line, as the caller's own
/// string, stopping at whatever [`PRINT_SIZE`] holds.
pub unsafe fn screen_print(s: *mut screen, line: c_int) -> ::std::ffi::CString {
    unsafe {
        let mut buffer = [0u8; PRINT_SIZE];
        let buf = &mut buffer;

        let mut last = 0;
        let gd = screen_grid_ptr(s);
        'out: for y in 0..(*gd).hsize + (*gd).sy {
            if line >= 0 && y != line as u_int {
                continue;
            }
            let header = format!("{y:04} \"");
            if header.len() >= PRINT_SIZE - last {
                break;
            }
            buf[last..last + header.len()].copy_from_slice(header.as_bytes());
            last += header.len();

            let gl = &(*gd).linedata[y as usize];
            for x in 0..gl.cellused {
                let gce = gl.celldata()[x as usize];
                if gce.flags as c_int & GRID_FLAG_PADDING != 0 {
                    continue;
                }

                if gce.flags as c_int & GRID_FLAG_EXTENDED == 0 {
                    if last + 2 >= PRINT_SIZE {
                        break 'out;
                    }
                    buf[last] = gce.c2rust_unnamed.data.data;
                    last += 1;
                } else if gce.flags as c_int & GRID_FLAG_TAB != 0 {
                    /*
                     * The arm for the alternate character set that came next
                     * in the C is gone: it tested the same bit as the tab
                     * above it, so it was never reached.
                     */
                    if last + 2 >= PRINT_SIZE {
                        break 'out;
                    }
                    buf[last] = b'\t';
                    last += 1;
                } else {
                    let mut ud = utf8_data::default();
                    utf8_to_data(
                        gl.extddata()[gce.c2rust_unnamed.offset as usize].data,
                        &mut ud,
                    );
                    let size = ud.size as usize;
                    if size > 0 {
                        if last + size + 1 >= PRINT_SIZE {
                            break 'out;
                        }
                        buf[last..last + size].copy_from_slice(&ud.data[..size]);
                        last += size;
                    }
                }
            }

            if last + 3 >= PRINT_SIZE {
                break;
            }
            buf[last] = b'"';
            buf[last + 1] = b'\n';
            last += 2;
        }
        // A cell holding a zero byte ended the C's answer where it stood, so
        // the printed lines stop there too.
        let printed = &buf[..last];
        let end = printed.iter().position(|&byte| byte == 0).unwrap_or(last);
        ::std::ffi::CString::new(&printed[..end]).expect("the bytes stop at the first zero")
    }
}

#[cfg(test)]
#[path = "../tests/test_screen.rs"]
mod tests;
