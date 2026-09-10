use super::RustScreen;
use super::write::{screen_write_free_list, screen_write_make_list};
use crate::fmt_args;
use crate::grid::{
    Grid, RustGrid, grid_adjust_lines, grid_create, grid_default_cell, grid_empty_line,
};
use crate::grid::{Hyperlinks, RustHyperlinks};
use crate::grid::{grid_view_clear, grid_view_delete_lines};
use crate::log::{fatalx, log_debug};

use crate::text::utf8_copy;
use crate::tmux::clean_name;
use crate::tmux::global_options;
pub use crate::types::*;
use ::core::ffi::{CStr, c_int, c_longlong};

/// A screen that can be exercised independently of its implementation.
pub trait Screen: Sized {
    /// The screen's hyperlink-store implementation.
    type Hyperlinks: Hyperlinks;

    /// Makes a screen with the given visible size and history limit.
    fn new(sx: u_int, sy: u_int, hlimit: u_int) -> Self;
    /// Returns the visible width and height.
    fn size(&self) -> (u_int, u_int);
    /// Returns the maximum number of history lines.
    fn history_limit(&self) -> u_int;
    /// Returns a handle to the screen's hyperlink store.
    fn hyperlinks(&self) -> Self::Hyperlinks;
    /// Removes every hyperlink from the screen.
    fn reset_hyperlinks(&mut self);
    /// Sets the cursor position.
    fn set_cursor(&mut self, cx: u_int, cy: u_int);
    /// Returns the cursor position.
    fn cursor(&self) -> (u_int, u_int);
    /// Sets the screen mode bits.
    fn set_mode(&mut self, mode: c_int);
    /// Returns the screen mode bits.
    fn mode(&self) -> c_int;
    /// Resets the screen while retaining its size and history.
    fn reinit(&mut self);
    /// Sets a selection on the screen.
    #[allow(clippy::too_many_arguments)]
    fn set_selection(
        &mut self,
        sx: u_int,
        sy: u_int,
        ex: u_int,
        ey: u_int,
        rectangle: bool,
        clipx: u_int,
        modekeys: c_int,
        gc: &grid_cell,
    );
    /// Clears any selection from the screen.
    fn clear_selection(&mut self);
    /// Hides the current selection without clearing its bounds.
    fn hide_selection(&mut self);
    /// Checks whether the cell at `(px, py)` is covered by the current selection.
    fn check_selection(&self, px: u_int, py: u_int) -> bool;
    /// Returns whether the screen currently has a selection.
    fn has_selection(&self) -> bool;
    /// Draws a cell the way the selection asks for, returning whether it changed.
    fn select_cell(&self, dst: &mut grid_cell, src: &grid_cell) -> bool;
}
pub use crate::consts::{
    ALL_MODES, EXTENDED_KEY_MODES, GRID_ATTR_CHARSET, GRID_ATTR_NOATTR, GRID_HISTORY,
    MODE_BRACKETPASTE, MODE_CRLF, MODE_CURSOR, MODE_CURSOR_BLINKING, MODE_CURSOR_BLINKING_SET,
    MODE_CURSOR_VERY_VISIBLE, MODE_FOCUSON, MODE_INSERT, MODE_KCURSOR, MODE_KEYS_EXTENDED,
    MODE_KEYS_EXTENDED_2, MODE_KKEYPAD, MODE_MOUSE_ALL, MODE_MOUSE_BUTTON, MODE_MOUSE_SGR,
    MODE_MOUSE_STANDARD, MODE_MOUSE_UTF8, MODE_ORIGIN, MODE_SYNC, MODE_THEME_UPDATES, MODE_WRAP,
    MODEKEY_EMACS, PROGRESS_BAR_HIDDEN, PROGRESS_BAR_INDETERMINATE, SCREEN_CURSOR_BAR,
    SCREEN_CURSOR_BLOCK, SCREEN_CURSOR_DEFAULT, SCREEN_CURSOR_UNDERLINE, UINT_MAX,
};
use ::std::collections::VecDeque;
use ::std::ffi::CString;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct screen_sel {
    pub hidden: c_int,
    pub rectangle: c_int,
    pub modekeys: c_int,
    pub sx: u_int,
    pub sy: u_int,
    pub ex: u_int,
    pub ey: u_int,
    pub clipx: u_int,
    pub cell: grid_cell,
}

/// The screen settings needed to update a terminal's mode and cursor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenModeState {
    pub mode: c_int,
    pub cursor: (u_int, u_int),
    pub cursor_style: screen_cursor_style,
    pub default_cursor_style: screen_cursor_style,
    pub cursor_colour: c_int,
    pub default_cursor_mode: c_int,
}

#[repr(C)]
pub(super) struct screen {
    pub title: Option<CString>,
    pub path: Option<CString>,
    pub titles: Option<Box<screen_titles>>,
    pub ntitles: u_int,
    pub(super) grid: Option<Box<RustGrid>>,
    pub(super) cx: u_int,
    pub(super) cy: u_int,
    pub cstyle: screen_cursor_style,
    pub default_cstyle: screen_cursor_style,
    pub ccolour: c_int,
    pub default_ccolour: c_int,
    pub rupper: u_int,
    pub rlower: u_int,
    pub(super) mode: c_int,
    pub default_mode: c_int,
    pub saved_cx: u_int,
    pub saved_cy: u_int,
    pub(super) saved_grid: Option<Box<RustGrid>>,
    pub saved_cell: grid_cell,
    pub saved_flags: c_int,
    pub tabs: Vec<u8>,
    pub(super) sel: Option<Box<screen_sel>>,
    pub write_list: Vec<screen_write_cline>,
    pub(super) hyperlinks: Option<RustHyperlinks>,
    pub progress_bar: progress_bar,
}

/// The titles a screen has pushed, most recently pushed first.
pub struct screen_titles {
    stack: VecDeque<CString>,
}

/// How many titles the stack keeps before the oldest are dropped.
const TITLE_LIMIT: u_int = 10;

/// The title stack of a screen, made on the first push.
fn screen_titles_of(s: &mut RustScreen) -> &mut screen_titles {
    s.0.titles
        .get_or_insert_with(|| {
            Box::new(screen_titles {
                stack: VecDeque::new(),
            })
        })
        .as_mut()
}

/// The title stack a screen carries, if it has one yet.
fn screen_titles_ptr(s: &mut RustScreen) -> Option<&mut screen_titles> {
    s.0.titles.as_deref_mut()
}

/// The grid a screen holds. A screen is only asked for it once `screen_init`
/// has given it one.
/// Throw the title stack away. The count of titles is deliberately left as
/// it was, which is what the C did.
fn screen_free_titles(s: &mut RustScreen) {
    s.0.titles = None;
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

impl RustScreen {
    pub(crate) fn grid(&self) -> &RustGrid {
        self.0.grid.as_deref().expect("a screen holds a grid")
    }

    pub(crate) fn grid_mut(&mut self) -> &mut RustGrid {
        self.0.grid.as_deref_mut().expect("a screen holds a grid")
    }

    /// A screen of `sx` by `sy` carrying `hlimit` lines of history. It starts
    /// as the empty screen and names only the fields a new screen differs on,
    /// then resets itself the way the C `screen_init` did.
    ///
    /// # Safety
    ///
    /// The global option set must be there, since resetting the screen reads
    /// `extended-keys` from it.
    pub(crate) fn new_with_server_options(sx: u_int, sy: u_int, hlimit: u_int) -> RustScreen {
        {
            let extended_keys = (global_options
                .get()
                .as_ref()
                .expect("global options are initialized"))
            .number(c"extended-keys");
            screen_new_with_extended_keys(sx, sy, hlimit, extended_keys)
        }
    }

    /// The screen's hyperlink set, if it owns one.
    fn hyperlinks_ref(&self) -> Option<&RustHyperlinks> {
        self.0.hyperlinks.as_ref()
    }

    /// Makes a live screen which shares the hyperlink set of `source`.
    pub(crate) fn new_sharing_hyperlinks(
        sx: u_int,
        sy: u_int,
        hlimit: u_int,
        source: &RustScreen,
    ) -> RustScreen {
        let mut screen = RustScreen::new_with_server_options(sx, sy, hlimit);
        screen.0.hyperlinks = Some(Screen::hyperlinks(source));
        screen
    }

    fn reset_hyperlinks_state(&mut self) {
        if let Some(hl) = self.0.hyperlinks.as_ref() {
            hl.reset();
        } else {
            self.0.hyperlinks = Some(RustHyperlinks::new());
        }
    }

    pub(crate) fn saved_cursor(&self) -> (u_int, u_int) {
        (self.0.saved_cx, self.0.saved_cy)
    }

    pub(crate) fn progress_bar(&self) -> progress_bar {
        self.0.progress_bar
    }

    pub fn set_progress_bar(&mut self, pbs: progress_bar_state, progress: c_int) {
        self.0.progress_bar.state = pbs;
        if progress >= 0 && pbs != PROGRESS_BAR_INDETERMINATE {
            self.0.progress_bar.progress = progress;
        }
    }

    pub fn set_title(&mut self, title: &CStr, untrusted: c_int) -> c_int {
        let Some(new_title) = (unsafe { clean_name(title, untrusted) }) else {
            return 0;
        };
        self.0.title = Some(new_title);
        1
    }

    pub fn set_path(&mut self, path: &CStr, untrusted: c_int) -> c_int {
        let Some(new_path) = (unsafe { clean_name(path, untrusted) }) else {
            return 0;
        };
        self.0.path = Some(new_path);
        1
    }

    pub(crate) fn title(&self) -> Option<&CStr> {
        self.0.title.as_deref()
    }

    pub(crate) fn path(&self) -> Option<&CStr> {
        self.0.path.as_deref()
    }

    pub fn push_title(&mut self) {
        log_debug(
            c"%s: %u",
            fmt_args![c"screen_push_title".as_ptr(), self.0.ntitles],
        );

        while self.0.ntitles >= TITLE_LIMIT {
            screen_titles_of(self).stack.pop_back();
            self.0.ntitles -= 1;
        }

        let title = self
            .0
            .title
            .clone()
            .expect("a screen always carries a title");
        screen_titles_of(self).stack.push_front(title);
        self.0.ntitles += 1;
    }

    pub fn pop_title(&mut self) {
        let Some(text) = screen_titles_ptr(self).and_then(|titles| titles.stack.pop_front()) else {
            return;
        };
        log_debug(
            c"%s: %u",
            fmt_args![c"screen_pop_title".as_ptr(), self.0.ntitles],
        );
        self.0.title = Some(text);
        self.0.ntitles -= 1;
    }
}

fn screen_new_with_extended_keys(
    sx: u_int,
    sy: u_int,
    hlimit: u_int,
    extended_keys: c_longlong,
) -> RustScreen {
    {
        let mut s = RustScreen::default();
        s.0.grid = Some(grid_create(sx, sy, hlimit));
        s.0.title = Some(c"".to_owned());
        s.0.mode = MODE_CURSOR;
        s.0.ccolour = -1;
        s.0.default_ccolour = -1;
        screen_reinit_with_extended_keys(&mut s, extended_keys);
        s
    }
}

/// Makes a standalone screen with tmux's default extended-key mode.
fn screen_new_standalone(sx: u_int, sy: u_int, hlimit: u_int) -> RustScreen {
    screen_new_with_extended_keys(sx, sy, hlimit, 0)
}

/// Reset a screen to what a new one is, keeping its size and its history.
pub unsafe fn screen_reinit(s: &mut RustScreen) {
    {
        let extended_keys = (global_options
            .get()
            .as_ref()
            .expect("global options are initialized"))
        .number(c"extended-keys");
        screen_reinit_with_extended_keys(s, extended_keys);
    }
}

fn screen_reinit_with_extended_keys(s: &mut RustScreen, extended_keys: c_longlong) {
    {
        s.0.cx = 0;
        s.0.cy = 0;
        s.0.rupper = 0;
        s.0.rlower = s.grid().sy.wrapping_sub(1);

        s.0.mode = MODE_CURSOR | MODE_WRAP | (s.0.mode & MODE_CRLF);

        if extended_keys == 2 {
            s.0.mode = (s.0.mode & !EXTENDED_KEY_MODES) | MODE_KEYS_EXTENDED;
        }

        if s.0.saved_grid.is_some() {
            screen_alternate_off(s, None, 0);
        }
        s.0.saved_cx = UINT_MAX;
        s.0.saved_cy = UINT_MAX;

        screen_reset_tabs(s);

        let gd = s.grid_mut();
        gd.clear_lines(gd.hsize, gd.sy, 8);

        s.clear_selection();
        screen_free_titles(s);
        s.set_progress_bar(PROGRESS_BAR_HIDDEN, 0);
        s.reset_hyperlinks();
    }
}

/// Resets a standalone screen with tmux's default extended-key mode.
fn screen_reinit_standalone(s: &mut RustScreen) {
    screen_reinit_with_extended_keys(s, 0);
}

#[cfg(test)]
#[path = "test_rust_screen.rs"]
mod tests;

fn screen_free(s: &mut RustScreen) {
    if !s.0.write_list.is_empty() {
        screen_write_free_list(s);
    }
}

impl Drop for RustScreen {
    fn drop(&mut self) {
        screen_free(self);
    }
}

/// Put a tab stop every eight columns.
pub fn screen_reset_tabs(s: &mut RustScreen) {
    let sx = RustScreen::grid(s).sx;
    s.0.tabs = vec![0; ((sx + 7) >> 3) as usize];
    let mut i = 8;
    while i < sx {
        let byte = &mut s.0.tabs[(i >> 3) as usize];
        *byte = (*byte as c_int | 1 << (i & 0x7)) as u8;
        i += 8;
    }
}

/// Turn a cursor style number into a shape and whether it blinks.
pub fn screen_set_cursor_style(style: u_int, cstyle: &mut screen_cursor_style, mode: &mut c_int) {
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

/// Set the progress bar, keeping the progress it had when there is none to
/// set.
/// Resize a screen, keeping the cell the cursor is on.
pub fn screen_resize_cursor(
    s: &mut RustScreen,
    sx: u_int,
    sy: u_int,
    reflow: c_int,
    eat_empty: c_int,
    cursor: c_int,
) {
    {
        let mut cx = s.0.cx;
        let mut cy = s.grid().hsize + s.0.cy;

        let collecting = !s.0.write_list.is_empty();
        if collecting {
            screen_write_free_list(s);
        }

        log_debug(
            c"%s: new size %ux%u, now %ux%u (cursor %u,%u = %u,%u)",
            fmt_args![
                c"screen_resize_cursor".as_ptr(),
                sx,
                sy,
                s.grid().sx,
                s.grid().sy,
                s.0.cx,
                s.0.cy,
                cx,
                cy
            ],
        );

        let sx = sx.max(1);
        let sy = sy.max(1);

        let mut reflow = reflow;
        if sx != s.grid().sx {
            s.grid_mut().sx = sx;
            screen_reset_tabs(s);
        } else {
            reflow = 0;
        }
        if sy != s.grid().sy {
            screen_resize_y(s, sy, eat_empty, &mut cy);
        }

        if reflow != 0 {
            screen_reflow(s, sx, &mut cx, &mut cy, cursor);
        }

        if cy >= s.grid().hsize {
            s.0.cx = cx;
            s.0.cy = cy - s.grid().hsize;
        } else {
            s.0.cx = 0;
            s.0.cy = 0;
        }

        log_debug(
            c"%s: cursor finished at %u,%u = %u,%u",
            fmt_args![c"screen_resize_cursor".as_ptr(), s.0.cx, s.0.cy, cx, cy],
        );

        if collecting {
            screen_write_make_list(s);
        }
    }
}

pub fn screen_resize(s: &mut RustScreen, sx: u_int, sy: u_int, reflow: c_int) {
    screen_resize_cursor(s, sx, sy, reflow, 1, 1)
}

/// Give the screen a new height, moving lines into and out of the history as
/// the new size needs.
fn screen_resize_y(s: &mut RustScreen, sy: u_int, eat_empty: c_int, cy: &mut u_int) {
    {
        let gd = s.0.grid.as_deref_mut().expect("a screen holds a grid");
        if sy == 0 {
            fatalx(c"zero size", fmt_args![]);
        }
        let oldy = gd.sy;

        /*
         * When getting smaller, nuke any empty lines at the bottom of the
         * screen, then move the rest into the history or delete them.
         */
        if sy < oldy {
            let mut needed = oldy - sy;

            if eat_empty != 0 {
                let mut available = oldy.wrapping_sub(1).wrapping_sub(s.0.cy);
                if available > 0 {
                    if available > needed {
                        available = needed;
                    }
                    grid_view_delete_lines(gd, oldy - available, available, 8);
                }
                needed -= available;
            }

            let mut available = s.0.cy;
            if gd.flags & GRID_HISTORY != 0 {
                gd.hscrolled += needed;
                gd.hsize += needed;
            } else if needed > 0 && available > 0 {
                if available > needed {
                    available = needed;
                }
                grid_view_delete_lines(gd, 0, available, 8);
                *cy = cy.wrapping_sub(available);
            }
        }

        /* Resize the historic data. */
        let line_count = gd.hsize + sy;
        grid_adjust_lines(gd, line_count);

        /* When getting larger, take lines from the history if there are any. */
        if sy > oldy {
            let mut needed = sy - oldy;
            let mut available = gd.hscrolled;
            if gd.flags & GRID_HISTORY != 0 && available > 0 {
                if available > needed {
                    available = needed;
                }
                gd.hscrolled -= available;
                gd.hsize -= available;
            } else {
                available = 0;
            }
            needed -= available;

            for i in gd.hsize + sy - needed..gd.hsize + sy {
                grid_empty_line(gd, i, 8);
            }
        }

        gd.sy = sy;
        s.0.rupper = 0;
        s.0.rlower = gd.sy.wrapping_sub(1);
    }
}

impl Screen for RustScreen {
    type Hyperlinks = RustHyperlinks;

    fn new(sx: u_int, sy: u_int, hlimit: u_int) -> Self {
        screen_new_standalone(sx, sy, hlimit)
    }

    fn size(&self) -> (u_int, u_int) {
        (self.grid().sx, self.grid().sy)
    }

    fn history_limit(&self) -> u_int {
        self.grid().hlimit
    }

    fn hyperlinks(&self) -> Self::Hyperlinks {
        self.hyperlinks_ref()
            .expect("a screen has hyperlinks")
            .clone()
    }

    fn reset_hyperlinks(&mut self) {
        self.reset_hyperlinks_state();
    }

    fn reinit(&mut self) {
        screen_reinit_standalone(self);
    }

    fn set_selection(
        &mut self,
        sx: u_int,
        sy: u_int,
        ex: u_int,
        ey: u_int,
        rectangle: bool,
        clipx: u_int,
        modekeys: c_int,
        gc: &grid_cell,
    ) {
        self.0.sel = Some(Box::new(screen_sel {
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

    fn clear_selection(&mut self) {
        self.0.sel = None;
    }

    fn hide_selection(&mut self) {
        if let Some(sel) = self.0.sel.as_mut() {
            sel.hidden = 1;
        }
    }

    fn check_selection(&self, px: u_int, py: u_int) -> bool {
        in_selection(self, px, py)
    }

    fn has_selection(&self) -> bool {
        self.0.sel.is_some()
    }

    fn select_cell(&self, dst: &mut grid_cell, src: &grid_cell) -> bool {
        let sel = match self.0.sel.as_ref() {
            None => return false,
            Some(sel) => sel,
        };
        if sel.hidden != 0 {
            return false;
        }

        *dst = sel.cell;

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
        true
    }

    fn mode(&self) -> c_int {
        self.0.mode
    }

    fn set_mode(&mut self, mode: c_int) {
        self.0.mode = mode;
    }

    fn cursor(&self) -> (u_int, u_int) {
        (self.0.cx, self.0.cy)
    }

    fn set_cursor(&mut self, cx: u_int, cy: u_int) {
        self.0.cx = cx;
        self.0.cy = cy;
    }
}

impl RustScreen {
    pub fn mode_state(&self) -> ScreenModeState {
        ScreenModeState {
            mode: self.mode(),
            cursor: self.cursor(),
            cursor_style: self.cursor_style(),
            default_cursor_style: self.default_cursor_style(),
            cursor_colour: self.cursor_colour(),
            default_cursor_mode: self.default_cursor_mode(),
        }
    }

    pub fn cursor_style(&self) -> screen_cursor_style {
        self.0.cstyle
    }

    pub fn default_cursor_style(&self) -> screen_cursor_style {
        self.0.default_cstyle
    }

    pub fn default_cursor_mode(&self) -> c_int {
        self.0.default_mode
    }

    pub fn cursor_colour(&self) -> c_int {
        if self.0.ccolour != -1 {
            self.0.ccolour
        } else {
            self.0.default_ccolour
        }
    }

    #[cfg(test)]
    pub(crate) fn default_cursor_colour(&self) -> c_int {
        self.0.default_ccolour
    }

    pub fn set_cursor_colour(&mut self, colour: c_int) {
        self.0.ccolour = colour;
    }

    pub fn set_cursor_style(&mut self, style: u_int) {
        screen_set_cursor_style(style, &mut self.0.cstyle, &mut self.0.mode);
    }

    pub fn set_default_cursor_colour(&mut self, colour: c_int) {
        self.0.default_ccolour = colour;
    }

    pub fn set_default_cursor_style(&mut self, style: u_int) {
        screen_set_cursor_style(style, &mut self.0.default_cstyle, &mut self.0.default_mode);
    }

    pub fn set_default_cursor(&mut self, oo: &RustOptionsRef) {
        {
            self.0.default_ccolour = (oo).number(c"cursor-colour") as c_int;
            self.0.default_mode = 0;
            self.set_default_cursor_style((oo).number(c"cursor-style") as u_int);
        }
    }

    pub fn region(&self) -> (u_int, u_int) {
        (self.0.rupper, self.0.rlower)
    }

    pub fn is_alternate(&self) -> bool {
        self.0.saved_grid.is_some()
    }

    pub fn is_initialized(&self) -> bool {
        self.0.grid.is_some()
    }

    #[cfg(test)]
    pub(crate) fn has_tabs(&self) -> bool {
        !self.0.tabs.is_empty()
    }

    #[cfg(test)]
    pub(crate) fn has_titles(&self) -> bool {
        self.0.titles.is_some()
    }

    #[cfg(test)]
    pub(crate) fn is_collecting(&self) -> bool {
        !self.0.write_list.is_empty()
    }

    pub fn tab_is_set(&self, x: u_int) -> bool {
        self.0.tabs[(x >> 3) as usize] as c_int & (1 << (x & 0x7)) != 0
    }

    pub fn set_tab(&mut self, x: u_int) {
        self.0.tabs[(x >> 3) as usize] |= 1 << (x & 0x7);
    }

    pub fn clear_tab(&mut self, x: u_int) {
        self.0.tabs[(x >> 3) as usize] &= !(1 << (x & 0x7));
    }

    pub fn clear_tabs(&mut self) {
        self.0.tabs.fill(0);
    }

    pub fn saved_grid(&self) -> Option<&RustGrid> {
        self.0.saved_grid.as_deref()
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
fn in_selection(s: &RustScreen, px: u_int, py: u_int) -> bool {
    let sel = match s.0.sel.as_ref() {
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

/// Reflow the grid to a new width, following the cell the cursor is on.
fn screen_reflow(s: &mut RustScreen, new_x: u_int, cx: &mut u_int, cy: &mut u_int, cursor: c_int) {
    let gd = s.grid_mut();
    let (mut wx, mut wy) = (0, 0);
    if cursor != 0 {
        (wx, wy) = gd.wrap_position(*cx, *cy);
        log_debug(
            c"%s: cursor %u,%u is %u,%u",
            fmt_args![c"screen_reflow".as_ptr(), *cx, *cy, wx, wy],
        );
    }

    gd.reflow(new_x);

    if cursor != 0 {
        (*cx, *cy) = gd.unwrap_position(wx, wy);
        log_debug(
            c"%s: new cursor is %u,%u",
            fmt_args![c"screen_reflow".as_ptr(), *cx, *cy],
        );
    } else {
        *cx = 0;
        *cy = gd.hsize;
    }
}

/// Put the screen aside and start on an empty one.
pub fn screen_alternate_on(s: &mut RustScreen, gc: &grid_cell, cursor: c_int) {
    {
        if s.0.saved_grid.is_some() {
            return;
        }
        let gd = s.0.grid.as_deref_mut().expect("a screen holds a grid");
        let sx = gd.sx;
        let sy = gd.sy;

        let mut saved_gd = grid_create(sx, sy, 0);
        saved_gd.duplicate_lines(0, gd, gd.hsize, sy);
        s.0.saved_grid = Some(saved_gd);
        if cursor != 0 {
            s.0.saved_cx = s.0.cx;
            s.0.saved_cy = s.0.cy;
        }
        s.0.saved_cell = *gc;

        grid_view_clear(gd, 0, 0, sx, sy, 8);

        s.0.saved_flags = gd.flags;
        gd.flags &= !GRID_HISTORY;
    }
}

/// Take the screen that was put aside back.
pub fn screen_alternate_off(s: &mut RustScreen, gc: Option<&mut grid_cell>, cursor: c_int) {
    {
        let (sx, sy) = (s.grid().sx, s.grid().sy);

        /*
         * If the current size is different, temporarily resize to the old
         * size before copying back.
         */
        if let Some((saved_sx, saved_sy)) = s.0.saved_grid.as_ref().map(|g| (g.sx, g.sy)) {
            screen_resize(s, saved_sx, saved_sy, 0);
        }

        /*
         * Restore the cursor position and cell. This happens even if not
         * currently in the alternate screen.
         */
        if cursor != 0 && s.0.saved_cx != UINT_MAX && s.0.saved_cy != UINT_MAX {
            s.0.cx = s.0.saved_cx;
            s.0.cy = s.0.saved_cy;
            if let Some(gc) = gc {
                *gc = s.0.saved_cell;
            }
        }

        /* If not in the alternate screen, do nothing more. */
        if s.0.saved_grid.is_none() {
            screen_clamp_cursor(s);
            return;
        }

        /* Restore the saved grid. */
        let saved_gd =
            s.0.saved_grid
                .as_deref()
                .expect("the saved grid is present");
        let gd = s.0.grid.as_deref_mut().expect("a screen holds a grid");
        gd.duplicate_lines(gd.hsize, saved_gd, 0, saved_gd.sy);

        /*
         * Turn history back on (so resize can use it) and then resize back to
         * the current size.
         */
        if s.0.saved_flags & GRID_HISTORY != 0 {
            gd.flags |= GRID_HISTORY;
        }
        screen_resize(s, sx, sy, 1);

        s.0.saved_grid.take();

        screen_clamp_cursor(s);
    }
}

/// Keep the cursor inside the screen.
fn screen_clamp_cursor(s: &mut RustScreen) {
    let gd = RustScreen::grid(s);
    let (sx, sy) = (gd.sx, gd.sy);
    if s.0.cx > sx - 1 {
        s.0.cx = sx - 1;
    }
    if s.0.cy > sy - 1 {
        s.0.cy = sy - 1;
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
pub fn screen_mode_to_string(mode: c_int) -> CString {
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
    CString::new(text).expect("a mode name has no interior NUL")
}

#[cfg(test)]
pub use crate::consts::{
    GRID_FLAG_EXTENDED, GRID_FLAG_PADDING, GRID_FLAG_TAB, PROGRESS_BAR_ERROR, PROGRESS_BAR_NORMAL,
};
