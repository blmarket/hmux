use crate::WindowPane;
use crate::pane_identity::PaneIdentity;
use crate::window_dimensions::WindowDimensionsState;

use super::RustScreen;
use super::Screen;
use super::draw::format_draw;

use super::redraw::screen_redraw_is_visible;
use super::state::screen_mode_to_string;
use super::state::{screen_alternate_off, screen_alternate_on, screen_reset_tabs};
use crate::fmt_args;
use crate::fmt_engine::{FmtArg, format_alloc};
use crate::grid::grid_cells_equal;
use crate::grid::{Grid, grid_default_cell, grid_get_line, grid_peek_line};
use crate::pane_geometry::PaneGeometryState;

pub(super) struct screen_write_state {
    pub(super) pane_ref: Option<RustWindowPaneWeak>,
    pub(super) flags: core::ffi::c_int,
    pub(super) init_ctx: screen_write_init_ctx,
    pub(super) item: CItem,
    pub(super) scrolled: u_int,
    pub(super) bg: u_int,
}

pub(super) struct screen_write_ctx<'a> {
    state: &'a mut screen_write_state,
    target: ScreenWriteTarget<'a>,
}

enum ScreenWriteTarget<'a> {
    Borrowed(&'a mut RustScreen),
    Shared(std::cell::RefMut<'a, RustScreen>),
}

pub(crate) type screen_write_init_ctx = Option<::std::rc::Rc<dyn Fn(&mut tty_ctx)>>;

impl Default for screen_write_state {
    fn default() -> Self {
        Self {
            pane_ref: None,
            flags: 0,
            init_ctx: None,
            item: CITEM_NONE,
            scrolled: 0,
            bg: 0,
        }
    }
}

impl core::ops::Deref for screen_write_ctx<'_> {
    type Target = screen_write_state;

    fn deref(&self) -> &Self::Target {
        self.state
    }
}

impl core::ops::DerefMut for screen_write_ctx<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.state
    }
}

impl<'a> screen_write_ctx<'a> {
    pub(super) fn new(state: &'a mut screen_write_state, target: &'a mut RustScreen) -> Self {
        Self {
            state,
            target: ScreenWriteTarget::Borrowed(target),
        }
    }

    pub(super) fn on_shared(
        state: &'a mut screen_write_state,
        target: std::cell::RefMut<'a, RustScreen>,
    ) -> Self {
        Self {
            state,
            target: ScreenWriteTarget::Shared(target),
        }
    }

    pub(super) fn pane_mut(&mut self) -> Option<&mut impl crate::WindowPane> {
        unsafe { self.state.pane_ref.as_mut()?.get_mut() }
    }

    pub(super) fn pane(&self) -> Option<&impl crate::WindowPane> {
        unsafe { self.state.pane_ref.as_ref()?.get() }
    }

    pub(super) fn screen(&self) -> &RustScreen {
        match &self.target {
            ScreenWriteTarget::Borrowed(screen) => screen,
            ScreenWriteTarget::Shared(screen) => screen,
        }
    }

    pub(super) fn screen_mut(&mut self) -> &mut RustScreen {
        match &mut self.target {
            ScreenWriteTarget::Borrowed(screen) => screen,
            ScreenWriteTarget::Shared(screen) => screen,
        }
    }
}

use crate::grid::{
    grid_view_clear, grid_view_clear_history, grid_view_delete_cells, grid_view_delete_lines,
    grid_view_delete_lines_region, grid_view_get_cell, grid_view_insert_cells,
    grid_view_insert_lines, grid_view_insert_lines_region, grid_view_scroll_region_down,
    grid_view_scroll_region_up, grid_view_set_cell, grid_view_set_cells, grid_view_set_padding,
};
use crate::layout::layout_cell_for_pane;
use crate::log::{fatalx, log_debug, log_get_level};

use crate::reactor::Timer;

use crate::status::{status_at_line, status_line_size};
use crate::terminfo::{AlternateCharacterSet, BorderCharacterSet, RustAlternateCharacterSet};
use crate::text::{
    HANGULJAMO_STATE_CHOSEONG, HANGULJAMO_STATE_NOT_COMPOSABLE, HANGULJAMO_STATE_NOT_HANGULJAMO,
    RustUtf8Compositor, Utf8Compositor,
};
use crate::text::{utf8_append, utf8_copy, utf8_fromcstr, utf8_open, utf8_set};
use crate::tmux::global_options;
use crate::tty::{
    tty_cmd_alignmenttest, tty_cmd_cell, tty_cmd_cells, tty_cmd_clearcharacter,
    tty_cmd_clearendofscreen, tty_cmd_clearscreen, tty_cmd_clearstartofscreen,
    tty_cmd_deletecharacter, tty_cmd_deleteline, tty_cmd_insertcharacter, tty_cmd_insertline,
    tty_cmd_rawstring, tty_cmd_redrawline, tty_cmd_reverseindex, tty_cmd_scrolldown,
    tty_cmd_scrollup, tty_cmd_setselection, tty_cmd_syncstart, tty_default_colours,
    tty_window_offset, tty_write,
};
pub use crate::types::*;
use crate::window::{window_pane_is_floating, window_ref_of};
use ::core::ffi::{CStr, c_int, c_uint};
#[repr(C)]
pub struct screen_write_cline {
    pub data: Option<Box<[u8]>>,
    /// What the line has collected, left to right. The items belong to the
    /// line until they go back to the free list.
    pub items: citems,
}

/// The collected items of one line, in the order they will be written.
pub type citems = Vec<CItem>;
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct screen_write_citem {
    pub x: u_int,
    pub wrapped: c_int,
    pub type_0: screen_write_citem_type,
    pub used: u_int,
    pub bg: u_int,
    pub gc: grid_cell,
}
pub type screen_write_citem_type = c_uint;
pub const CLEAR: screen_write_citem_type = 1;
pub use crate::consts::{
    BOX_LINES_DEFAULT, BOX_LINES_DOUBLE, BOX_LINES_HEAVY, BOX_LINES_PADDED, BOX_LINES_ROUNDED,
    BOX_LINES_SIMPLE, BOX_LINES_SINGLE, CELL_BORDERS, CELL_BOTTOMLEFT, CELL_BOTTOMRIGHT,
    CELL_LEFTJOIN, CELL_LEFTRIGHT, CELL_RIGHTJOIN, CELL_TOPBOTTOM, CELL_TOPLEFT, CELL_TOPRIGHT,
    CLIENT_REDRAWPANES, EXTENDED_KEY_MODES, GRID_ATTR_CHARSET, GRID_ATTR_DIM, GRID_ATTR_REVERSE,
    GRID_FLAG_CLEARED, GRID_FLAG_EXTENDED, GRID_FLAG_NOPALETTE, GRID_FLAG_PADDING,
    GRID_FLAG_SELECTED, GRID_FLAG_TAB, GRID_HISTORY, GRID_LINE_EXTENDED, GRID_LINE_WRAPPED,
    MODE_CURSOR, MODE_INSERT, MODE_KEYS_EXTENDED, MODE_ORIGIN, MODE_SYNC, MODE_WRAP, PANE_REDRAW,
    PANE_REDRAWSCROLLBAR, SIMPLE_BORDERS, TTY_CTX_CELL_INVALIDATE, TTY_CTX_INVISIBLE_PANES,
    TTY_CTX_OVERLAY_SYNC, TTY_CTX_PANE_OBSCURED, TTY_CTX_SYNC, TTY_CTX_WINDOW_BIGGER,
    TTY_CTX_WRAPPED, UINT_MAX, UTF8_DONE, UTF8_MORE,
};

pub const PADDED_BORDERS: [u8; 14] = *b"             \0";

pub const SCREEN_WRITE_SYNC: c_int = 0x1;
pub const SCREEN_WRITE_OBSCURED: c_int = 0x2;
pub const SCREEN_WRITE_CHECKED_IF_OBSCURED: c_int = 0x4;

pub const PANE_DROP: c_int = 0x2;

/// Which collected item, as an index into the pool below. Items are never
/// given back to the allocator, so an index stays good for the life of the
/// server — which is what lets `screen_write_collect_trim` read the wrapped
/// flag of an item it has just given up.
pub type CItem = u32;

/// The index no item has, which a context carries before it is started and
/// after it is stopped.
pub const CITEM_NONE: CItem = CItem::MAX;

struct CItemPool {
    items: std::collections::VecDeque<screen_write_citem>,
    free: std::collections::VecDeque<CItem>,
}

impl CItemPool {
    const fn new() -> Self {
        Self {
            items: std::collections::VecDeque::new(),
            free: std::collections::VecDeque::new(),
        }
    }

    fn allocate(&mut self) -> CItem {
        if let Some(ci) = self.free.pop_front() {
            self.items[ci as usize] = screen_write_citem::default();
            return ci;
        }
        let ci = next_citem_index(self.items.len());
        self.items.push_back(screen_write_citem::default());
        ci
    }
}

fn next_citem_index(len: usize) -> CItem {
    CItem::try_from(len)
        .ok()
        .filter(|&id| id != CITEM_NONE)
        .expect("collected item IDs exhausted")
}

static CITEM_POOL: std::sync::Mutex<CItemPool> = std::sync::Mutex::new(CItemPool::new());

/// The items of one line, as a snapshot that a walk may take from.
unsafe fn citem_list(head: &citems) -> Vec<CItem> {
    head.clone()
}

/// A snapshot of the item `ci` names.
pub(crate) fn citem_snapshot(ci: CItem) -> screen_write_citem {
    *CITEM_POOL
        .lock()
        .expect("collected item pool lock is not poisoned")
        .items
        .get(ci as usize)
        .expect("a collected item index names its pool entry")
}

/// Mutate the item `ci` names for the duration of `f`.
fn with_citem<R>(ci: CItem, f: impl FnOnce(&mut screen_write_citem) -> R) -> R {
    let mut pool = CITEM_POOL
        .lock()
        .expect("collected item pool lock is not poisoned");
    f(pool
        .items
        .get_mut(ci as usize)
        .expect("a collected item index names its pool entry"))
}

/// Where `ci` sits in `head`, which is wherever it was put.
unsafe fn citem_position(head: &citems, ci: CItem) -> Option<usize> {
    head.iter().position(|&item| item == ci)
}

/// Puts `ci` at the end of `head`.
unsafe fn citem_insert_tail(head: &mut citems, ci: CItem) {
    head.push(ci)
}

/// Puts `ci` in front of `before`, which is already in `head`.
unsafe fn citem_insert_before(head: &mut citems, before: CItem, ci: CItem) {
    unsafe {
        let at = citem_position(head, before).expect("the anchor is on this line");
        head.insert(at, ci);
    }
}

/// Puts `ci` behind `after`, which is already in `head`.
unsafe fn citem_insert_after(head: &mut citems, after: CItem, ci: CItem) {
    unsafe {
        let at = citem_position(head, after).expect("the anchor is on this line");
        head.insert(at + 1, ci);
    }
}

/// Takes `ci` out of the list of collected items it hangs in, which is
/// `head`.
unsafe fn citem_remove(head: &mut citems, ci: CItem) {
    unsafe {
        if let Some(at) = citem_position(head, ci) {
            head.remove(at);
        }
    }
}

/// Moves every item of `src` to the end of the free list, leaving `src`
/// empty.
fn citem_free_all(src: &mut citems) {
    CITEM_POOL
        .lock()
        .expect("collected item pool lock is not poisoned")
        .free
        .extend(src.drain(..))
}

/// The screen's collect lists, one for each line of its grid.
fn write_list(s: &mut RustScreen) -> &mut [screen_write_cline] {
    s.0.write_list.as_mut_slice()
}

/// A collected item to fill in: one off the free list, zeroed again, or a
/// fresh one when the free list is empty.
fn screen_write_get_citem() -> CItem {
    CITEM_POOL
        .lock()
        .expect("collected item pool lock is not poisoned")
        .allocate()
}

/// Hands `ci` back to the free list. Nothing it carried is written over, so
/// it can still be read back afterwards — which `screen_write_collect_trim`
/// relies on for the wrapped flag.
fn screen_write_free_citem(ci: CItem) {
    CITEM_POOL
        .lock()
        .expect("collected item pool lock is not poisoned")
        .free
        .push_back(ci)
}
unsafe fn screen_write_offset_timer(w_ref: WindowRef) {
    unsafe { w_ref.update_client_offsets() }
}

/// Moves the cursor, clamped to the screen, and arms the timer that works out
/// the window offsets again. A coordinate of -1 leaves that half alone.
unsafe fn screen_write_set_cursor(ctx: &mut screen_write_ctx, mut cx: c_int, mut cy: c_int) {
    unsafe {
        let s = ctx.screen_mut();
        let tv = timeval::from_usecs(10000 as __suseconds_t);
        if cx != -1 && cx as u_int == s.0.cx && cy != -1 && cy as u_int == s.0.cy {
            return;
        }
        if cx != -1 {
            if cx as u_int > RustScreen::grid(s).sx {
                cx = RustScreen::grid(s).sx.wrapping_sub(1) as c_int;
            }
            s.0.cx = cx as u_int;
        }
        if cy != -1 {
            if cy as u_int > RustScreen::grid(s).sy.wrapping_sub(1) {
                cy = RustScreen::grid(s).sy.wrapping_sub(1) as c_int;
            }
            s.0.cy = cy as u_int;
        }
        let Some(pane) = ctx.pane() else {
            return;
        };
        let window = pane
            .window_context()
            .expect("a writing pane has a window context");
        let mut w = window.as_window_mut();
        if !w.offset_timer.is_set() {
            let w_weak = window_ref_of(&w).map(|w_ref| w_ref.downgrade());
            w.offset_timer.set_callback(move || {
                let Some(w_ref) = w_weak.as_ref().and_then(WindowWeak::upgrade) else {
                    return;
                };
                screen_write_offset_timer(w_ref);
            });
        }
        if !w.offset_timer.is_armed() {
            w.offset_timer.arm(tv);
        }
    }
}
unsafe fn screen_write_redraw_cb(ttyctx: &tty_ctx) {
    unsafe {
        let TtyCtxArg::Pane(mut pane_ref) = ttyctx.arg.clone() else {
            return;
        };
        if let Some(pane) = pane_ref.get_mut() {
            *pane.flags_mut() |= PANE_REDRAW;
        }
    }
}
unsafe fn screen_write_set_client_cb(ttyctx: &mut tty_ctx, c: &mut client) -> c_int {
    unsafe {
        let TtyCtxArg::Pane(mut pane_ref) = ttyctx.arg.clone() else {
            return 0;
        };
        let id = pane_ref.id();
        let Some(window) = pane_ref.window() else {
            return 0;
        };
        let attached = c.attached_session();
        if ttyctx.flags & TTY_CTX_INVISIBLE_PANES != 0 {
            if attached
                .as_ref()
                .is_some_and(|session| session.has(&window))
            {
                return 1;
            }
            return 0;
        }
        let visible = attached
            .as_ref()
            .and_then(|session| session.curw())
            .and_then(|link| link.window())
            .is_some_and(|owner| owner.ptr_eq(&window));
        if !visible {
            return 0;
        }
        if layout_cell_for_pane(window.as_window().layout_root.as_deref(), &pane_ref).is_none() {
            return 0;
        }
        let pane = pane_ref
            .get_mut()
            .expect("the drawing pane still belongs to its window");
        if *pane.flags() & (PANE_REDRAW | PANE_DROP) != 0 {
            return -1;
        }
        if c.flags & CLIENT_REDRAWPANES as uint64_t != 0 {
            log_debug(
                c"%s: adding %%%u to deferred redraw",
                fmt_args![c"screen_write_set_client_cb", id],
            );
            *pane.flags_mut() |= PANE_REDRAW | PANE_REDRAWSCROLLBAR;
            return -1;
        }
        let (bigger, ox, oy, sx, sy) = tty_window_offset(&c.tty);
        (ttyctx.wox, ttyctx.woy, ttyctx.wsx, ttyctx.wsy) = (ox, oy, sx, sy);
        if bigger != 0 {
            ttyctx.flags |= TTY_CTX_WINDOW_BIGGER;
        } else {
            ttyctx.flags &= !TTY_CTX_WINDOW_BIGGER;
        }
        ttyctx.rxoff = pane.geometry().x;
        ttyctx.xoff = ttyctx.rxoff;
        ttyctx.ryoff = pane.geometry().y;
        ttyctx.yoff = ttyctx.ryoff;
        if status_at_line(&*c) == 0 {
            ttyctx.yoff = (ttyctx.yoff as u_int).wrapping_add(status_line_size(&*c)) as c_int;
        }
        1
    }
}

/// Whether anything is in front of the pane being written to: it hangs off
/// its window, or a floating pane in front of it overlaps it. The answer is
/// worked out once and kept in the context's flags.
unsafe fn screen_write_pane_is_obscured(ctx: &mut screen_write_ctx) -> c_int {
    unsafe {
        if ctx.pane().is_none() {
            return 0;
        }
        if ctx.flags & SCREEN_WRITE_CHECKED_IF_OBSCURED != 0 {
            if ctx.flags & SCREEN_WRITE_OBSCURED != 0 {
                return 1;
            }
            return 0;
        }
        ctx.flags |= SCREEN_WRITE_CHECKED_IF_OBSCURED;
        let base = ctx.pane().expect("the writer has a pane");
        let window = base
            .window_context()
            .expect("a writing pane has a window context");
        let w = window.as_window();
        let base_id = base.pane_id();
        let b_geometry = base.geometry();
        if b_geometry.x < 0
            || b_geometry.y < 0
            || (b_geometry.x as u_int).wrapping_add(b_geometry.width) > w.dimensions().size.width
            || (b_geometry.y as u_int).wrapping_add(b_geometry.height) > w.dimensions().size.height
        {
            ctx.flags |= SCREEN_WRITE_OBSCURED;
            return 1;
        }
        let Some(at) = w.z_index.iter().position(|pane| pane.id() == base_id) else {
            return 0;
        };
        for f in w.z_index[..at]
            .iter()
            .rev()
            .take_while(|pane| pane.is_alive())
        {
            let f = f.as_pane();
            let f_geometry = f.geometry();
            if window_pane_is_floating(
                &w,
                &{ crate::window::window_pane_ref_of(f) }.expect("the pane allocation exists"),
            ) != 0
                && (f_geometry.y >= b_geometry.y
                    && f_geometry.y <= b_geometry.y + b_geometry.height as c_int
                    || f_geometry.y + f_geometry.height as c_int >= b_geometry.y
                        && (f_geometry.y as u_int).wrapping_add(f_geometry.height)
                            <= (b_geometry.y as u_int).wrapping_add(b_geometry.height))
                && (f_geometry.x >= b_geometry.x
                    && f_geometry.x <= b_geometry.x + b_geometry.width as c_int
                    || f_geometry.x + f_geometry.width as c_int >= b_geometry.x
                        && (f_geometry.x as u_int).wrapping_add(f_geometry.width)
                            <= (b_geometry.x as u_int).wrapping_add(b_geometry.width))
            {
                ctx.flags |= SCREEN_WRITE_OBSCURED;
                return 1;
            }
        }
        0
    }
}

/// Fills in a terminal context from the screen being written to, and starts a
/// synchronised update if this is the first call on this context.
unsafe fn screen_write_initctx(
    ctx: &mut screen_write_ctx,
    ttyctx: &mut tty_ctx,
    is_sync: c_int,
    check_obscured: c_int,
) {
    unsafe {
        *ttyctx = tty_ctx::default();
        let s = ctx.screen();
        ttyctx.sx = RustScreen::grid(s).sx;
        ttyctx.sy = RustScreen::grid(s).sy;
        ttyctx.ocx = s.0.cx;
        ttyctx.ocy = s.0.cy;
        ttyctx.orlower = s.0.rlower;
        ttyctx.orupper = s.0.rupper;
        if check_obscured != 0 && screen_write_pane_is_obscured(ctx) != 0 {
            ttyctx.flags |= TTY_CTX_PANE_OBSCURED;
        }
        ttyctx.defaults = grid_default_cell;
        if let Some(cb) = ctx.init_ctx.clone() {
            cb(ttyctx);
            if let Some(palette) = &ttyctx.palette {
                if ttyctx.defaults.fg == 8 {
                    ttyctx.defaults.fg = palette.fg;
                }
                if ttyctx.defaults.bg == 8 {
                    ttyctx.defaults.bg = palette.bg;
                }
            }
        } else {
            ttyctx.redraw_cb = Some(screen_write_redraw_cb);
            if let Some(pane) = ctx.pane_mut() {
                tty_default_colours(&mut ttyctx.defaults, pane);
                ttyctx.palette = Some(pane.palette().clone());
                ttyctx.set_client_cb = Some(screen_write_set_client_cb);
                ttyctx.arg = TtyCtxArg::Pane(
                    ctx.pane_ref
                        .as_ref()
                        .expect("the writer has a pane")
                        .clone(),
                );
            }
        }
        if !ctx.flags & SCREEN_WRITE_SYNC != 0 {
            let pane = ctx.pane();
            let inactive = pane.is_some_and(|pane| {
                let window = pane
                    .window_context()
                    .expect("a writing pane has a window context");
                window.active_pane_id() != Some(pane.pane_id())
            });
            if inactive {
                ttyctx.flags |= TTY_CTX_SYNC;
            } else {
                if pane.is_none() {
                    ttyctx.flags |= TTY_CTX_OVERLAY_SYNC;
                }
                if is_sync != 0 {
                    ttyctx.flags |= TTY_CTX_SYNC;
                }
            }
            tty_write(|tty, ctx| tty_cmd_syncstart(tty, ctx), ttyctx);
            ctx.flags |= SCREEN_WRITE_SYNC;
        }
    }
}

/// Makes the screen's collect lists, one for each line of its grid.
pub(super) fn screen_write_make_list(s: &mut RustScreen) {
    let sy = RustScreen::grid(s).sy as usize;
    let mut list: Vec<screen_write_cline> = Vec::with_capacity(sy);
    for _ in 0..sy {
        list.push(screen_write_cline {
            data: None,
            items: citems::new(),
        });
    }
    s.0.write_list = list;
}

/// Gives up the screen's collect lists and the text they collected. What a
/// line still holds goes back to the free list: `screen_write_collect_flush`
/// leaves behind whatever a pane in front of this one hid, so a screen can be
/// freed with items still collected.
pub(super) fn screen_write_free_list(s: &mut RustScreen) {
    for cl in write_list(s) {
        citem_free_all(&mut cl.items);
    }
    s.0.write_list = Vec::new();
}
fn screen_write_init(s: &mut RustScreen) -> screen_write_state {
    if s.0.write_list.is_empty() {
        screen_write_make_list(s);
    }
    screen_write_state {
        item: screen_write_get_citem(),
        bg: 8,
        ..screen_write_state::default()
    }
}
pub(super) unsafe fn screen_write_start_pane_base(
    wp: &mut impl crate::WindowPane,
) -> screen_write_state {
    unsafe {
        let mut state = screen_write_init(wp.base_mut());
        state.pane_ref = crate::window::window_pane_ref_of(wp);
        if log_get_level() != 0 {
            log_debug(
                c"%s: size %ux%u, pane %%%u (at %u,%u)",
                fmt_args![
                    c"screen_write_start_pane_base",
                    RustScreen::grid(wp.base()).sx,
                    RustScreen::grid(wp.base()).sy,
                    wp.pane_id(),
                    wp.geometry().x,
                    wp.geometry().y
                ],
            );
        }
        state
    }
}
pub(super) fn screen_write_start_callback(
    s: &mut RustScreen,
    init_ctx: screen_write_init_ctx,
) -> screen_write_state {
    let mut state = screen_write_init(s);
    state.init_ctx = init_ctx;
    if log_get_level() != 0 {
        log_debug(
            c"%s: size %ux%u, with callback",
            fmt_args![
                c"screen_write_start_callback",
                RustScreen::grid(s).sx,
                RustScreen::grid(s).sy
            ],
        );
    }
    state
}
pub(super) fn screen_write_start(s: &mut RustScreen) -> screen_write_state {
    let state = screen_write_init(s);
    if log_get_level() != 0 {
        log_debug(
            c"%s: size %ux%u, no pane",
            fmt_args![
                c"screen_write_start",
                RustScreen::grid(s).sx,
                RustScreen::grid(s).sy
            ],
        );
    }
    state
}
pub(super) unsafe fn screen_write_stop(ctx: &mut screen_write_ctx) {
    unsafe {
        screen_write_collect_end(ctx);
        screen_write_collect_flush(ctx, 0, c"screen_write_stop");
        screen_write_free_citem(ctx.item);
    }
}
pub(super) unsafe fn screen_write_reset(ctx: &mut screen_write_ctx) {
    unsafe {
        let s = ctx.screen_mut();
        screen_reset_tabs(s);
        let lower = RustScreen::grid(s).sy.wrapping_sub(1);
        screen_write_scrollregion(ctx, 0, lower);
        let s = ctx.screen_mut();
        s.0.mode = MODE_CURSOR | MODE_WRAP;
        if (global_options
            .as_ref()
            .expect("global options are initialized"))
        .number(c"extended-keys")
            == 2 as core::ffi::c_longlong
        {
            s.0.mode = s.0.mode & !EXTENDED_KEY_MODES | MODE_KEYS_EXTENDED;
        }
        screen_write_clearscreen(ctx, 8);
        screen_write_set_cursor(ctx, 0, 0);
    }
}
pub(super) unsafe fn screen_write_putc(ctx: &mut screen_write_ctx, gcp: &grid_cell, ch: u_char) {
    unsafe {
        let mut gc = *gcp;
        utf8_set(&mut gc.data, ch);
        screen_write_cell(ctx, &gc);
    }
}

/// Whether a byte is one the writing calls put on the screen as itself.
fn is_printable(b: u8) -> bool {
    b == b'\t' || (0x20..0x7f).contains(&b)
}

/// How wide `text` will be on the screen.
pub(super) unsafe fn screen_write_strlen(fmt: &CStr, args: &[FmtArg]) -> size_t {
    unsafe {
        let msg = format_alloc(fmt, args);
        let bytes = msg.as_bytes();
        let mut size: size_t = 0;
        let mut i = 0;
        while i < bytes.len() {
            let mut ud = utf8_data::default();
            let b = bytes[i];
            if b > 0x7f && utf8_open(&mut ud, b) == UTF8_MORE {
                i += 1;
                if bytes.len() - i < (ud.size as usize).wrapping_sub(1) {
                    break;
                }
                let mut more;
                while {
                    more = utf8_append(&mut ud, bytes[i]);
                    more == UTF8_MORE
                } {
                    i += 1;
                }
                i += 1;
                if more == UTF8_DONE {
                    size = size.wrapping_add(ud.width as size_t);
                }
            } else {
                if is_printable(b) {
                    size = size.wrapping_add(1);
                }
                i += 1;
            }
        }
        size
    }
}
#[allow(clippy::too_many_arguments)]
pub(super) unsafe fn screen_write_text(
    ctx: &mut screen_write_ctx,
    cx: u_int,
    width: u_int,
    lines: u_int,
    more: c_int,
    gcp: &grid_cell,
    fmt: &CStr,
    args: &[FmtArg],
) -> c_int {
    unsafe {
        let cy: u_int = ctx.screen().0.cy;
        let mut idx: usize = 0;
        let mut gc = *gcp;
        let tmp = format_alloc(fmt, args);
        let mut owned = utf8_fromcstr(&tmp);
        owned.push(utf8_data::default());
        let text = &owned;
        let is = |i: usize, ch: u8| text[i].size == 1 && text[i].data[0] == ch;
        let mut left: u_int = cx.wrapping_add(width).wrapping_sub(ctx.screen().0.cx);
        loop {
            let mut at: u_int = 0;
            let mut end = idx;
            while text[end].size != 0 {
                if is(end, b'\n') {
                    break;
                }
                if at.wrapping_add(text[end].width as u_int) > left {
                    break;
                }
                at = at.wrapping_add(text[end].width as u_int);
                end += 1;
            }
            let next;
            if text[end].size == 0 {
                next = end;
            } else if is(end, b'\n') || is(end, b' ') {
                next = end + 1;
            } else {
                let mut i = end;
                while i > idx && !is(i, b' ') {
                    i -= 1;
                }
                if i != idx {
                    next = i + 1;
                    end = i;
                } else {
                    next = end;
                }
            }
            for cell in text[idx..end].iter() {
                utf8_copy(&mut gc.data, cell);
                screen_write_cell(ctx, &gc);
            }
            idx = next;
            if ctx.screen().0.cy == cy.wrapping_add(lines).wrapping_sub(1) || text[idx].size == 0 {
                break;
            }
            let next_y = ctx.screen().0.cy.wrapping_add(1);
            screen_write_cursormove(ctx, cx as c_int, next_y as c_int, 0);
            left = width;
        }
        let at_last_line = ctx.screen().0.cy == cy.wrapping_add(lines).wrapping_sub(1);
        let finished = more == 0 || ctx.screen().0.cx == cx.wrapping_add(width);
        let more_left = text[idx].size != 0;
        if at_last_line && finished || more_left {
            return 0;
        }
        if finished {
            let next_y = ctx.screen().0.cy.wrapping_add(1);
            screen_write_cursormove(ctx, cx as c_int, next_y as c_int, 0);
        }
        1
    }
}
pub(super) unsafe fn screen_write_puts(
    ctx: &mut screen_write_ctx,
    gcp: &grid_cell,
    fmt: &CStr,
    args: &[FmtArg],
) {
    unsafe { screen_write_vnputs(ctx, -1, gcp, fmt, args) }
}
pub(super) unsafe fn screen_write_nputs(
    ctx: &mut screen_write_ctx,
    maxlen: ssize_t,
    gcp: &grid_cell,
    fmt: &CStr,
    args: &[FmtArg],
) {
    unsafe { screen_write_vnputs(ctx, maxlen, gcp, fmt, args) }
}

/// Writes a formatted string, stopping at `maxlen` columns when that is not
/// negative; the columns a character that would not fit leaves behind are
/// filled with spaces.
pub(super) unsafe fn screen_write_vnputs(
    ctx: &mut screen_write_ctx,
    maxlen: ssize_t,
    gcp: &grid_cell,
    fmt: &CStr,
    args: &[FmtArg],
) {
    unsafe {
        let mut gc = *gcp;
        let msg = format_alloc(fmt, args);
        let bytes = msg.as_bytes();
        let mut size: size_t = 0;
        let mut i = 0;
        while i < bytes.len() {
            let b = bytes[i];
            if b > 0x7f && utf8_open(&mut gc.data, b) == UTF8_MORE {
                i += 1;
                if bytes.len() - i < (gc.data.size as usize).wrapping_sub(1) {
                    break;
                }
                let mut more;
                while {
                    more = utf8_append(&mut gc.data, bytes[i]);
                    more == UTF8_MORE
                } {
                    i += 1;
                }
                i += 1;
                if more != UTF8_DONE {
                    continue;
                }
                if maxlen > 0 && size.wrapping_add(gc.data.width as size_t) > maxlen as size_t {
                    while size < maxlen as size_t {
                        screen_write_putc(ctx, &gc, b' ');
                        size = size.wrapping_add(1);
                    }
                    break;
                }
                size = size.wrapping_add(gc.data.width as size_t);
                screen_write_cell(ctx, &gc);
            } else {
                if maxlen > 0 && size.wrapping_add(1) > maxlen as size_t {
                    break;
                }
                if b == 0x01 {
                    gc.attr = (gc.attr as c_int ^ GRID_ATTR_CHARSET) as u_short;
                } else if b == b'\n' {
                    screen_write_linefeed(ctx, 0, 8);
                    screen_write_carriagereturn(ctx);
                } else if is_printable(b) {
                    size = size.wrapping_add(1);
                    screen_write_putc(ctx, &gc, b);
                }
                i += 1;
            }
        }
    }
}
/// Copies a rectangle of another screen onto this one and writes it straight
/// out, without collecting it.
pub(super) unsafe fn screen_write_fast_copy(
    ctx: &mut screen_write_ctx,
    src: &RustScreen,
    px: u_int,
    py: u_int,
    nx: u_int,
    ny: u_int,
) {
    unsafe {
        let gd = RustScreen::grid(src);
        let mut ttyctx = tty_ctx::default();
        let mut gc;
        let (cx, cy) = ctx.screen().cursor();
        if nx == 0 || ny == 0 {
            return;
        }
        let (xoff, yoff) = ctx.pane().map_or((0, 0), |pane| {
            let geometry = pane.geometry();
            (geometry.x, geometry.y)
        });
        let mut yy = py;
        while yy < py.wrapping_add(ny) {
            if yy >= gd.hsize.wrapping_add(gd.sy) {
                break;
            }
            ctx.screen_mut().0.cx = cx;
            screen_write_initctx(ctx, &mut ttyctx, 0, 0);
            let s = ctx.screen();
            let mut ranges = visible_ranges::default();
            ranges.set_visible_ranges(
                ctx.pane(),
                (xoff as u_int).wrapping_add(s.0.cx) as c_int,
                s.0.cy.wrapping_add(yoff as u_int) as c_int,
                nx,
            );
            let mut xx = px;
            while xx < px.wrapping_add(nx) {
                let gl = grid_peek_line(gd, yy).expect("a copied line is in the grid");
                let s = ctx.screen_mut();
                let (sx, sy) = s.cursor();
                let sgl = grid_get_line(RustScreen::grid_mut(s), sy);
                if xx >= gl.cellsize() && sx >= sgl.cellsize() {
                    break;
                }
                gc = gd.cell(xx, yy);
                if xx.wrapping_add(gc.data.width as u_int) > px.wrapping_add(nx) {
                    break;
                }
                grid_view_set_cell(RustScreen::grid_mut(s), sx, sy, &gc);
                if !screen_redraw_is_visible(Some(&ranges), (xoff as u_int).wrapping_add(s.0.cx)) {
                    break;
                }
                ttyctx.cell = Some(gc);
                ttyctx.flags &= TTY_CTX_OVERLAY_SYNC | TTY_CTX_SYNC;
                tty_write(
                    |tty, ttyctx| tty_cmd_cell(tty, ttyctx, ctx.screen()),
                    &mut ttyctx,
                );
                ttyctx.ocx = ttyctx.ocx.wrapping_add(1);
                let s = ctx.screen_mut();
                s.0.cx = s.0.cx.wrapping_add(1);
                xx = xx.wrapping_add(1);
            }
            let s = ctx.screen_mut();
            s.0.cy = s.0.cy.wrapping_add(1);
            yy = yy.wrapping_add(1);
        }
        let s = ctx.screen_mut();
        s.0.cx = cx;
        s.0.cy = cy;
    }
}

/// Puts the character a border of `lines` draws for `cell_type` into `gc`.
fn screen_write_box_border_set(lines: box_lines, cell_type: c_int, gc: &mut grid_cell) {
    let family = match lines {
        BOX_LINES_DOUBLE => Some(BorderCharacterSet::Double),
        BOX_LINES_HEAVY => Some(BorderCharacterSet::Heavy),
        BOX_LINES_ROUNDED => Some(BorderCharacterSet::Rounded),
        _ => None,
    };
    if let Some(acs) = family
        .and_then(|family| RustAlternateCharacterSet.border(family, cell_type.try_into().ok()?))
    {
        gc.attr = (gc.attr as c_int & !GRID_ATTR_CHARSET) as u_short;
        utf8_copy(&mut gc.data, &acs);
        return;
    }
    let table = match lines {
        BOX_LINES_SIMPLE => &SIMPLE_BORDERS,
        BOX_LINES_PADDED => &PADDED_BORDERS,
        BOX_LINES_SINGLE | BOX_LINES_DEFAULT => {
            gc.attr = (gc.attr as c_int | GRID_ATTR_CHARSET) as u_short;
            utf8_set(&mut gc.data, CELL_BORDERS[cell_type as usize]);
            return;
        }
        _ => return,
    };
    gc.attr = (gc.attr as c_int & !GRID_ATTR_CHARSET) as u_short;
    utf8_set(&mut gc.data, table[cell_type as usize]);
}
/// Draws one edge of a box: a corner or join, the middle of the line and the
/// other corner or join.
unsafe fn box_edge(
    ctx: &mut screen_write_ctx,
    nx: u_int,
    lines: box_lines,
    gc: &mut grid_cell,
    left: c_int,
    right: c_int,
) {
    unsafe {
        screen_write_box_border_set(lines, left, gc);
        screen_write_cell(ctx, gc);
        screen_write_box_border_set(lines, CELL_LEFTRIGHT, gc);
        let mut i = 1;
        while i < nx.wrapping_sub(1) {
            screen_write_cell(ctx, gc);
            i = i.wrapping_add(1);
        }
        screen_write_box_border_set(lines, right, gc);
        screen_write_cell(ctx, gc);
    }
}

/// Draws a horizontal line of `nx` columns, joined at either end when asked.
pub(super) unsafe fn screen_write_hline(
    ctx: &mut screen_write_ctx,
    nx: u_int,
    left: c_int,
    right: c_int,
    lines: box_lines,
    border_gc: Option<&grid_cell>,
) {
    unsafe {
        let (cx, cy) = ctx.screen().cursor();
        let mut gc = border_gc.copied().unwrap_or(grid_default_cell);
        gc.attr = (gc.attr as c_int | GRID_ATTR_CHARSET) as u_short;
        let starts = if left != 0 {
            CELL_LEFTJOIN
        } else {
            CELL_LEFTRIGHT
        };
        let ends = if right != 0 {
            CELL_RIGHTJOIN
        } else {
            CELL_LEFTRIGHT
        };
        box_edge(ctx, nx, lines, &mut gc, starts, ends);
        screen_write_set_cursor(ctx, cx as c_int, cy as c_int);
    }
}

/// Draws a vertical line of `ny` lines, joined at either end when asked.
pub(super) unsafe fn screen_write_vline(
    ctx: &mut screen_write_ctx,
    ny: u_int,
    top: c_int,
    bottom: c_int,
) {
    unsafe {
        let (cx, cy) = ctx.screen().cursor();
        let mut gc = grid_default_cell;
        gc.attr = (gc.attr as c_int | GRID_ATTR_CHARSET) as u_short;
        screen_write_putc(ctx, &gc, if top != 0 { b'w' } else { b'x' });
        let mut i = 1;
        while i < ny.wrapping_sub(1) {
            screen_write_set_cursor(ctx, cx as c_int, cy.wrapping_add(i) as c_int);
            screen_write_putc(ctx, &gc, b'x');
            i = i.wrapping_add(1);
        }
        screen_write_set_cursor(
            ctx,
            cx as c_int,
            cy.wrapping_add(ny).wrapping_sub(1) as c_int,
        );
        screen_write_putc(ctx, &gc, if bottom != 0 { b'v' } else { b'x' });
        screen_write_set_cursor(ctx, cx as c_int, cy as c_int);
    }
}

/// Draws a menu: a box with one line per item, the chosen one in its own
/// style and a name starting with `-` drawn dimmed and without that byte.
pub(super) unsafe fn screen_write_menu(
    ctx: &mut screen_write_ctx,
    menu: &menu,
    choice: c_int,
    lines: box_lines,
    menu_gc: &grid_cell,
    border_gc: &grid_cell,
    choice_gc: &grid_cell,
) {
    unsafe {
        let (cx, cy) = ctx.screen().cursor();
        let width: u_int = menu.width;
        let default_gc = *menu_gc;
        screen_write_box(
            ctx,
            menu.width.wrapping_add(4),
            (menu.items.len() as u_int).wrapping_add(2),
            lines,
            Some(border_gc),
            menu.title.as_deref(),
        );
        let mut i = 0;
        while (i as usize) < menu.items.len() {
            let name = menu.items[i as usize].name.as_deref().map(CStr::to_bytes);
            let line = cy.wrapping_add(1).wrapping_add(i) as c_int;
            match name {
                None => {
                    screen_write_cursormove(ctx, cx as c_int, line, 0);
                    screen_write_hline(ctx, width.wrapping_add(4), 1, 1, lines, Some(border_gc));
                }
                Some(name) => {
                    let dim = name.first() == Some(&b'-');
                    let mut gc = if choice >= 0 && i == choice as u_int && !dim {
                        *choice_gc
                    } else {
                        default_gc
                    };
                    screen_write_cursormove(ctx, cx.wrapping_add(1) as c_int, line, 0);
                    let mut j = 0;
                    while j < width.wrapping_add(2) {
                        screen_write_putc(ctx, &gc, b' ');
                        j = j.wrapping_add(1);
                    }
                    screen_write_cursormove(ctx, cx.wrapping_add(2) as c_int, line, 0);
                    if dim {
                        gc.attr = (gc.attr as c_int | GRID_ATTR_DIM) as u_short;
                        format_draw(ctx, &gc, width, &name[1..], None, 0);
                    } else {
                        format_draw(ctx, &gc, width, name, None, 0);
                    }
                }
            }
            i = i.wrapping_add(1);
        }
        screen_write_set_cursor(ctx, cx as c_int, cy as c_int);
    }
}

/// Draws a box of `nx` by `ny` with an optional title along its top.
pub(super) unsafe fn screen_write_box(
    ctx: &mut screen_write_ctx,
    nx: u_int,
    ny: u_int,
    lines: box_lines,
    gcp: Option<&grid_cell>,
    title: Option<&CStr>,
) {
    unsafe {
        let (cx, cy) = ctx.screen().cursor();
        let mut gc = gcp.copied().unwrap_or(grid_default_cell);
        gc.attr = (gc.attr as c_int | GRID_ATTR_CHARSET) as u_short;
        gc.flags = (gc.flags as c_int | GRID_FLAG_NOPALETTE) as u_char;
        box_edge(ctx, nx, lines, &mut gc, CELL_TOPLEFT, CELL_TOPRIGHT);
        screen_write_set_cursor(
            ctx,
            cx as c_int,
            cy.wrapping_add(ny).wrapping_sub(1) as c_int,
        );
        box_edge(ctx, nx, lines, &mut gc, CELL_BOTTOMLEFT, CELL_BOTTOMRIGHT);
        screen_write_box_border_set(lines, CELL_TOPBOTTOM, &mut gc);
        let mut i = 1;
        while i < ny.wrapping_sub(1) {
            screen_write_set_cursor(ctx, cx as c_int, cy.wrapping_add(i) as c_int);
            screen_write_cell(ctx, &gc);
            screen_write_set_cursor(
                ctx,
                cx.wrapping_add(nx).wrapping_sub(1) as c_int,
                cy.wrapping_add(i) as c_int,
            );
            screen_write_cell(ctx, &gc);
            i = i.wrapping_add(1);
        }
        if let Some(title) = title {
            gc.attr = (gc.attr as c_int & !GRID_ATTR_CHARSET) as u_short;
            screen_write_cursormove(ctx, cx.wrapping_add(2) as c_int, cy as c_int, 0);
            format_draw(ctx, &gc, nx.wrapping_sub(4), title.to_bytes(), None, 0);
        }
        screen_write_set_cursor(ctx, cx as c_int, cy as c_int);
    }
}

/// Where a preview of `src` starts, so that the cursor is a third of the way
/// in and the preview stays inside the screen.
fn preview_offset(cursor: u_int, want: u_int, have: u_int) -> u_int {
    let mut at = if cursor < want.wrapping_div(3) {
        0
    } else {
        cursor.wrapping_sub(want.wrapping_div(3))
    };
    if at.wrapping_add(want) > have {
        at = if want > have {
            0
        } else {
            have.wrapping_sub(want)
        };
    }
    at
}

/// Draws a small copy of another screen, with its cursor shown in reverse.
pub(super) unsafe fn screen_write_preview(
    ctx: &mut screen_write_ctx,
    src: &RustScreen,
    nx: u_int,
    ny: u_int,
) {
    unsafe {
        let (cx, cy) = ctx.screen().cursor();
        let mut gc;
        let (px, py) = if src.0.mode & MODE_CURSOR != 0 {
            (
                preview_offset(src.0.cx, nx, RustScreen::grid(src).sx),
                preview_offset(src.0.cy, ny, RustScreen::grid(src).sy),
            )
        } else {
            (0, 0)
        };
        screen_write_fast_copy(
            ctx,
            src,
            px,
            RustScreen::grid(src).hsize.wrapping_add(py),
            nx,
            ny,
        );
        if src.0.mode & MODE_CURSOR != 0 {
            gc = grid_view_get_cell(RustScreen::grid(src), src.0.cx, src.0.cy);
            gc.attr = (gc.attr as c_int | GRID_ATTR_REVERSE) as u_short;
            screen_write_set_cursor(
                ctx,
                cx.wrapping_add(src.0.cx.wrapping_sub(px)) as c_int,
                cy.wrapping_add(src.0.cy.wrapping_sub(py)) as c_int,
            );
            screen_write_cell(ctx, &gc);
        }
    }
}
pub(super) fn screen_write_mode_set(ctx: &mut screen_write_ctx, mode: c_int) {
    let s = ctx.screen_mut();
    s.0.mode |= mode;
    if log_get_level() != 0 {
        log_debug(
            c"%s: %s",
            fmt_args![
                c"screen_write_mode_set",
                screen_mode_to_string(mode).as_c_str()
            ],
        );
    }
}

pub(super) fn screen_write_mode_clear(ctx: &mut screen_write_ctx, mode: c_int) {
    let s = ctx.screen_mut();
    s.0.mode &= !mode;
    if log_get_level() != 0 {
        log_debug(
            c"%s: %s",
            fmt_args![
                c"screen_write_mode_clear",
                screen_mode_to_string(mode).as_c_str()
            ],
        );
    }
}
pub(super) unsafe fn screen_write_cursorup(ctx: &mut screen_write_ctx, mut ny: u_int) {
    unsafe {
        let s = ctx.screen_mut();
        let mut cx: u_int = s.0.cx;
        let mut cy: u_int = s.0.cy;
        if ny == 0 {
            ny = 1;
        }
        if cy < s.0.rupper {
            if ny > cy {
                ny = cy;
            }
        } else if ny > cy.wrapping_sub(s.0.rupper) {
            ny = cy.wrapping_sub(s.0.rupper);
        }
        if cx == RustScreen::grid(&*s).sx {
            cx = cx.wrapping_sub(1);
        }
        cy = cy.wrapping_sub(ny);
        screen_write_set_cursor(ctx, cx as c_int, cy as c_int);
    }
}
pub(super) unsafe fn screen_write_cursordown(ctx: &mut screen_write_ctx, mut ny: u_int) {
    unsafe {
        let s = ctx.screen_mut();
        let mut cx: u_int = s.0.cx;
        let mut cy: u_int = s.0.cy;
        if ny == 0 {
            ny = 1;
        }
        if cy > s.0.rlower {
            if ny > RustScreen::grid(&*s).sy.wrapping_sub(1).wrapping_sub(cy) {
                ny = RustScreen::grid(&*s).sy.wrapping_sub(1).wrapping_sub(cy);
            }
        } else if ny > s.0.rlower.wrapping_sub(cy) {
            ny = s.0.rlower.wrapping_sub(cy);
        }
        if cx == RustScreen::grid(&*s).sx {
            cx = cx.wrapping_sub(1);
        } else if ny == 0 {
            return;
        }
        cy = cy.wrapping_add(ny);
        screen_write_set_cursor(ctx, cx as c_int, cy as c_int);
    }
}
pub(super) unsafe fn screen_write_cursorright(ctx: &mut screen_write_ctx, mut nx: u_int) {
    unsafe {
        let s = ctx.screen_mut();
        let mut cx: u_int = s.0.cx;
        let cy: u_int = s.0.cy;
        if nx == 0 {
            nx = 1;
        }
        if nx > RustScreen::grid(&*s).sx.wrapping_sub(1).wrapping_sub(cx) {
            nx = RustScreen::grid(&*s).sx.wrapping_sub(1).wrapping_sub(cx);
        }
        if nx == 0 {
            return;
        }
        cx = cx.wrapping_add(nx);
        screen_write_set_cursor(ctx, cx as c_int, cy as c_int);
    }
}
pub(super) unsafe fn screen_write_cursorleft(ctx: &mut screen_write_ctx, mut nx: u_int) {
    unsafe {
        let s = ctx.screen_mut();
        let mut cx: u_int = s.0.cx;
        let cy: u_int = s.0.cy;
        if nx == 0 {
            nx = 1;
        }
        if nx > cx {
            nx = cx;
        }
        if nx == 0 {
            return;
        }
        cx = cx.wrapping_sub(nx);
        screen_write_set_cursor(ctx, cx as c_int, cy as c_int);
    }
}
pub(super) unsafe fn screen_write_backspace(ctx: &mut screen_write_ctx) {
    unsafe {
        let (mut cx, mut cy) = ctx.screen().cursor();
        if cx == 0 {
            if cy == 0 {
                return;
            }
            let grid = RustScreen::grid_mut(ctx.screen_mut());
            let previous = grid.hsize.wrapping_add(cy).wrapping_sub(1);
            if grid_get_line(grid, previous).flags & GRID_LINE_WRAPPED != 0 {
                cy = cy.wrapping_sub(1);
                cx = grid.sx.wrapping_sub(1);
            }
        } else {
            cx = cx.wrapping_sub(1);
        }
        screen_write_set_cursor(ctx, cx as c_int, cy as c_int);
    }
}

/// Whether a cell holds one plain single-width character, which is the only
/// shape a redraw can write out as one cell rather than a whole line.
unsafe fn screen_write_cell_is_single(gc: &grid_cell) -> c_int {
    let single = gc.data.width == 1
        && gc.data.size == 1
        && gc.data.data[0] >= 0x20
        && gc.data.data[0] != 0x7f
        && gc.flags as c_int & (GRID_FLAG_CLEARED | GRID_FLAG_PADDING | GRID_FLAG_TAB) == 0;
    single as c_int
}

/// Redraws the visible parts of one line of the pane, a cell at a time where
/// only one column is visible and the whole line otherwise.
///
/// Two guards the C had are gone with the conversion, each unreachable:
/// a range that starts past the pane's right edge (every range starts at or
/// after the pane's `xoff`, and the one arrangement whose start runs past
/// `xoff + sx` — the range cut by a pane in front of it — has `nx` zero,
/// which the guard above skips), and a count of zero (`nx` is not zero, and
/// the clamp only ever replaces it with `sx - cx`, which is at least one
/// since `cx` is less than `sx`).
unsafe fn screen_write_redraw_line(ctx: &mut screen_write_ctx, ttyctx: &mut tty_ctx, yy: u_int) {
    unsafe {
        let mut gc;
        let mut ngc = grid_cell::default();
        let sx: u_int = RustScreen::grid(ctx.screen()).sx;
        let geometry = ctx
            .pane()
            .expect("a redrawn line belongs to a pane")
            .geometry();
        let (xoff, yoff) = (geometry.x, geometry.y);
        if ctx.screen().0.mode & MODE_SYNC != 0 {
            return;
        }
        let mut ranges = visible_ranges::default();
        ranges.set_visible_ranges(
            ctx.pane(),
            xoff,
            (yoff as u_int).wrapping_add(yy) as c_int,
            sx,
        );
        for ri in &ranges.ranges[..ranges.used as usize] {
            if ri.nx == 0 {
                continue;
            }
            let cx = ri.px.wrapping_sub(xoff as u_int);
            let n = if cx.wrapping_add(ri.nx) > sx {
                sx.wrapping_sub(cx)
            } else {
                ri.nx
            };
            ttyctx.value = TtyCtxValue::Num(n);
            ttyctx.ocx = cx;
            ttyctx.ocy = yy;
            if n != 1 {
                tty_write(
                    |tty, ttyctx| tty_cmd_redrawline(tty, ttyctx, ctx.screen()),
                    ttyctx,
                );
                continue;
            }
            gc = grid_view_get_cell(RustScreen::grid(ctx.screen()), cx, yy);
            if screen_write_cell_is_single(&gc) == 0 {
                tty_write(
                    |tty, ttyctx| tty_cmd_redrawline(tty, ttyctx, ctx.screen()),
                    ttyctx,
                );
                continue;
            }
            if !(gc.flags as c_int) & GRID_FLAG_SELECTED != 0 {
                ttyctx.cell = Some(gc);
            } else {
                ctx.screen().select_cell(&mut ngc, &gc);
                ttyctx.cell = Some(ngc);
            }
            tty_write(
                |tty, ttyctx| tty_cmd_cell(tty, ttyctx, ctx.screen()),
                ttyctx,
            );
        }
    }
}
unsafe fn screen_write_redraw_pane(ctx: &mut screen_write_ctx, ttyctx: &mut tty_ctx) {
    unsafe {
        let mut yy: u_int;
        yy = 0;
        while yy < RustScreen::grid(ctx.screen()).sy {
            screen_write_redraw_line(ctx, ttyctx, yy);
            yy = yy.wrapping_add(1);
        }
    }
}
/// Whether something is in front of the pane the context writes to, which
/// `screen_write_initctx` works out when it is asked to.
fn pane_obscured(ttyctx: &tty_ctx) -> bool {
    ttyctx.flags & TTY_CTX_PANE_OBSCURED != 0
}

/// Hands `cmd` to the terminal, or redraws the line under the cursor when a
/// pane in front of this one means the command cannot be written as it is.
unsafe fn write_or_redraw_line(
    ctx: &mut screen_write_ctx,
    ttyctx: &mut tty_ctx,
    mut cmd: impl FnMut(&mut tty, &tty_ctx, &RustScreen),
) {
    unsafe {
        if !pane_obscured(ttyctx) || ctx.pane().is_none() {
            tty_write(|tty, ttyctx| cmd(tty, ttyctx, ctx.screen()), ttyctx);
            return;
        }
        let cy = ctx.screen().0.cy;
        screen_write_redraw_line(ctx, ttyctx, cy);
    }
}

/// Hands `cmd` to the terminal, or redraws the whole pane when a pane in
/// front of this one means the command cannot be written as it is.
unsafe fn write_or_redraw_pane(
    ctx: &mut screen_write_ctx,
    ttyctx: &mut tty_ctx,
    mut cmd: impl FnMut(&mut tty, &tty_ctx, &RustScreen),
) {
    unsafe {
        if !pane_obscured(ttyctx) || ctx.pane().is_none() {
            tty_write(|tty, ttyctx| cmd(tty, ttyctx, ctx.screen()), ttyctx);
            return;
        }
        screen_write_redraw_pane(ctx, ttyctx);
    }
}

/// Fills the screen with `E`, which is what the alignment test draws.
pub(super) unsafe fn screen_write_alignmenttest(ctx: &mut screen_write_ctx) {
    unsafe {
        let mut ttyctx = tty_ctx::default();
        let mut gc = grid_default_cell;
        utf8_set(&mut gc.data, b'E');
        let grid = RustScreen::grid_mut(ctx.screen_mut());
        for yy in 0..grid.sy {
            for xx in 0..grid.sx {
                grid_view_set_cell(grid, xx, yy, &gc);
            }
        }
        let lower = grid.sy.wrapping_sub(1);
        screen_write_set_cursor(ctx, 0, 0);
        let s = ctx.screen_mut();
        s.0.rupper = 0;
        s.0.rlower = lower;
        screen_write_collect_clear(ctx, 0, lower);
        screen_write_initctx(ctx, &mut ttyctx, 1, 1);
        write_or_redraw_pane(ctx, &mut ttyctx, |tty, ctx, screen| {
            tty_cmd_alignmenttest(tty, ctx, screen)
        });
    }
}

/// The number of columns a call that takes one may work on: at least one,
/// and never past the right of the screen.
///
/// The guard the C had for a cursor past the last column is gone with the
/// conversion: `screen_write_set_cursor` clamps anything over `sx` back, and
/// at `sx` exactly this clamp is zero, which the caller has already returned
/// on.
fn columns_left(s: &RustScreen, nx: u_int) -> u_int {
    let nx = if nx == 0 { 1 } else { nx };
    let left = RustScreen::grid(s).sx.wrapping_sub(s.0.cx);
    if nx > left { left } else { nx }
}

pub(super) unsafe fn screen_write_insertcharacter(
    ctx: &mut screen_write_ctx,
    nx: u_int,
    bg: u_int,
) {
    unsafe {
        let mut ttyctx = tty_ctx::default();
        let nx = columns_left(ctx.screen(), nx);
        if nx == 0 {
            return;
        }
        screen_write_initctx(ctx, &mut ttyctx, 0, 1);
        ttyctx.bg = bg;
        let s = ctx.screen_mut();
        let (cx, cy) = s.cursor();
        grid_view_insert_cells(RustScreen::grid_mut(s), cx, cy, nx, bg);
        screen_write_collect_flush(ctx, 0, c"screen_write_insertcharacter");
        ttyctx.value = TtyCtxValue::Num(nx);
        write_or_redraw_line(ctx, &mut ttyctx, |tty, ctx, screen| {
            tty_cmd_insertcharacter(tty, ctx, screen)
        });
    }
}
pub(super) unsafe fn screen_write_deletecharacter(
    ctx: &mut screen_write_ctx,
    nx: u_int,
    bg: u_int,
) {
    unsafe {
        let mut ttyctx = tty_ctx::default();
        let nx = columns_left(ctx.screen(), nx);
        if nx == 0 {
            return;
        }
        screen_write_initctx(ctx, &mut ttyctx, 0, 1);
        ttyctx.bg = bg;
        let s = ctx.screen_mut();
        let (cx, cy) = s.cursor();
        grid_view_delete_cells(RustScreen::grid_mut(s), cx, cy, nx, bg);
        screen_write_collect_flush(ctx, 0, c"screen_write_deletecharacter");
        ttyctx.value = TtyCtxValue::Num(nx);
        write_or_redraw_line(ctx, &mut ttyctx, |tty, ctx, screen| {
            tty_cmd_deletecharacter(tty, ctx, screen)
        });
    }
}
pub(super) unsafe fn screen_write_clearcharacter(ctx: &mut screen_write_ctx, nx: u_int, bg: u_int) {
    unsafe {
        let mut ttyctx = tty_ctx::default();
        let nx = columns_left(ctx.screen(), nx);
        if nx == 0 {
            return;
        }
        screen_write_initctx(ctx, &mut ttyctx, 0, 1);
        ttyctx.bg = bg;
        let s = ctx.screen_mut();
        let (cx, cy) = s.cursor();
        grid_view_clear(RustScreen::grid_mut(s), cx, cy, nx, 1, bg);
        screen_write_collect_flush(ctx, 0, c"screen_write_clearcharacter");
        ttyctx.value = TtyCtxValue::Num(nx);
        write_or_redraw_line(ctx, &mut ttyctx, |tty, ctx, screen| {
            tty_cmd_clearcharacter(tty, ctx, screen)
        });
    }
}

/// The number of lines a call that takes one may work on when the cursor sits
/// outside the scroll region: at least one, and never past the bottom of the
/// screen.
///
/// The guard the C had for a count of zero after this clamp is gone with the
/// conversion: the cursor is never below the last line, so the clamp is at
/// least one.
fn lines_left(s: &RustScreen, ny: u_int) -> u_int {
    let ny = if ny == 0 { 1 } else { ny };
    let left = RustScreen::grid(s).sy.wrapping_sub(s.0.cy);
    if ny > left { left } else { ny }
}

/// The number of lines a call that takes one may work on inside the scroll
/// region: at least one, and never past the bottom of the region.
///
/// As with `lines_left`, the C's guard for a count of zero is unreachable:
/// the cursor is inside the region, so the clamp is at least one.
fn lines_left_in_region(s: &RustScreen, ny: u_int) -> u_int {
    let ny = if ny == 0 { 1 } else { ny };
    let left = s.0.rlower.wrapping_add(1).wrapping_sub(s.0.cy);
    if ny > left { left } else { ny }
}

/// Whether the cursor sits outside the scroll region.
fn outside_region(s: &RustScreen) -> bool {
    s.0.cy < s.0.rupper || s.0.cy > s.0.rlower
}

pub(super) unsafe fn screen_write_insertline(ctx: &mut screen_write_ctx, ny: u_int, bg: u_int) {
    unsafe {
        let mut ttyctx = tty_ctx::default();
        if outside_region(ctx.screen()) {
            let ny = lines_left(ctx.screen(), ny);
            screen_write_initctx(ctx, &mut ttyctx, 1, 1);
            ttyctx.bg = bg;
            let s = ctx.screen_mut();
            let cy = s.0.cy;
            grid_view_insert_lines(RustScreen::grid_mut(s), cy, ny, bg);
            screen_write_collect_flush(ctx, 0, c"screen_write_insertline");
            ttyctx.value = TtyCtxValue::Num(ny);
            write_or_redraw_pane(ctx, &mut ttyctx, |tty, ctx, screen| {
                tty_cmd_insertline(tty, ctx, screen)
            });
            return;
        }
        let ny = lines_left_in_region(ctx.screen(), ny);
        screen_write_initctx(ctx, &mut ttyctx, 1, 1);
        ttyctx.bg = bg;
        let s = ctx.screen_mut();
        let (cy, lower) = (s.0.cy, s.0.rlower);
        grid_view_insert_lines_region(RustScreen::grid_mut(s), lower, cy, ny, bg);
        screen_write_collect_flush(ctx, 0, c"screen_write_insertline");
        ttyctx.value = TtyCtxValue::Num(ny);
        write_or_redraw_pane(ctx, &mut ttyctx, |tty, ctx, screen| {
            tty_cmd_insertline(tty, ctx, screen)
        });
    }
}
pub(super) unsafe fn screen_write_deleteline(ctx: &mut screen_write_ctx, ny: u_int, bg: u_int) {
    unsafe {
        let mut ttyctx = tty_ctx::default();
        if outside_region(ctx.screen()) {
            let ny = lines_left(ctx.screen(), ny);
            screen_write_initctx(ctx, &mut ttyctx, 1, 1);
            ttyctx.bg = bg;
            let s = ctx.screen_mut();
            let cy = s.0.cy;
            grid_view_delete_lines(RustScreen::grid_mut(s), cy, ny, bg);
            screen_write_collect_flush(ctx, 0, c"screen_write_deleteline");
            ttyctx.value = TtyCtxValue::Num(ny);
            write_or_redraw_pane(ctx, &mut ttyctx, |tty, ctx, screen| {
                tty_cmd_deleteline(tty, ctx, screen)
            });
            return;
        }
        let ny = lines_left_in_region(ctx.screen(), ny);
        screen_write_initctx(ctx, &mut ttyctx, 1, 1);
        ttyctx.bg = bg;
        let s = ctx.screen_mut();
        let (cy, lower) = (s.0.cy, s.0.rlower);
        grid_view_delete_lines_region(RustScreen::grid_mut(s), lower, cy, ny, bg);
        screen_write_collect_flush(ctx, 0, c"screen_write_deleteline");
        ttyctx.value = TtyCtxValue::Num(ny);
        write_or_redraw_pane(ctx, &mut ttyctx, |tty, ctx, screen| {
            tty_cmd_deleteline(tty, ctx, screen)
        });
    }
}
pub(super) unsafe fn screen_write_clearline(ctx: &mut screen_write_ctx, bg: u_int) {
    unsafe {
        let ci = ctx.item;
        let cy = ctx.screen().0.cy;
        let grid = RustScreen::grid_mut(ctx.screen_mut());
        let sx = grid.sx;
        let line = grid.hsize.wrapping_add(cy);
        if grid_get_line(grid, line).cellsize() == 0 && (bg == 8 || bg == 9) {
            return;
        }
        grid_view_clear(grid, 0, cy, sx, 1, bg);
        screen_write_collect_clear(ctx, cy, 1);
        with_citem(ci, |item| {
            item.x = 0;
            item.used = sx;
            item.type_0 = CLEAR;
            item.bg = bg;
        });
        citem_insert_tail(&mut write_list(ctx.screen_mut())[cy as usize].items, ci);
        ctx.item = screen_write_get_citem();
    }
}
pub(super) unsafe fn screen_write_clearendofline(ctx: &mut screen_write_ctx, bg: u_int) {
    unsafe {
        let ci = ctx.item;
        let (cx, cy) = ctx.screen().cursor();
        if cx == 0 {
            screen_write_clearline(ctx, bg);
            return;
        }
        let grid = RustScreen::grid_mut(ctx.screen_mut());
        let sx = grid.sx;
        let line = grid.hsize.wrapping_add(cy);
        let used = grid_get_line(grid, line).cellsize();
        if cx > sx.wrapping_sub(1) || cx >= used && (bg == 8 || bg == 9) {
            return;
        }
        grid_view_clear(grid, cx, cy, sx.wrapping_sub(cx), 1, bg);
        with_citem(ci, |item| {
            item.x = cx;
            item.used = sx.wrapping_sub(cx);
            item.type_0 = CLEAR;
            item.bg = bg;
        });
        screen_write_collect_insert(ctx, ci);
    }
}

/// Clears from the start of the line to the cursor.
///
/// The C cleared the whole line instead when the cursor was past the last
/// column; that arm is gone with the conversion, since a cursor at or after
/// the last column has already gone to `screen_write_clearline` above.
pub(super) unsafe fn screen_write_clearstartofline(ctx: &mut screen_write_ctx, bg: u_int) {
    unsafe {
        let ci = ctx.item;
        let (cx, cy) = ctx.screen().cursor();
        let sx = RustScreen::grid(ctx.screen()).sx;
        if cx >= sx.wrapping_sub(1) {
            screen_write_clearline(ctx, bg);
            return;
        }
        grid_view_clear(
            RustScreen::grid_mut(ctx.screen_mut()),
            0,
            cy,
            cx.wrapping_add(1),
            1,
            bg,
        );
        with_citem(ci, |item| {
            item.x = 0;
            item.used = cx.wrapping_add(1);
            item.type_0 = CLEAR;
            item.bg = bg;
        });
        screen_write_collect_insert(ctx, ci);
    }
}

pub(super) unsafe fn screen_write_cursormove(
    ctx: &mut screen_write_ctx,
    mut px: c_int,
    mut py: c_int,
    origin: c_int,
) {
    unsafe {
        let s = ctx.screen();
        if origin != 0 && py != -1 && s.0.mode & MODE_ORIGIN != 0 {
            if py as u_int > s.0.rlower.wrapping_sub(s.0.rupper) {
                py = s.0.rlower as c_int;
            } else {
                py = (py as u_int).wrapping_add(s.0.rupper) as c_int;
            }
        }
        if px != -1 && px as u_int > RustScreen::grid(s).sx.wrapping_sub(1) {
            px = RustScreen::grid(s).sx.wrapping_sub(1) as c_int;
        }
        if py != -1 && py as u_int > RustScreen::grid(s).sy.wrapping_sub(1) {
            py = RustScreen::grid(s).sy.wrapping_sub(1) as c_int;
        }
        log_debug(
            c"%s: from %u,%u to %u,%u",
            fmt_args![c"screen_write_cursormove", s.0.cx, s.0.cy, px, py],
        );
        screen_write_set_cursor(ctx, px, py);
    }
}
pub(super) unsafe fn screen_write_reverseindex(ctx: &mut screen_write_ctx, bg: u_int) {
    unsafe {
        let s = ctx.screen_mut();
        let (cy, upper, lower) = (s.0.cy, s.0.rupper, s.0.rlower);
        if cy == upper {
            grid_view_scroll_region_down(RustScreen::grid_mut(s), upper, lower, bg);
            screen_write_collect_flush(ctx, 0, c"screen_write_reverseindex");
            let mut ttyctx = tty_ctx::default();
            screen_write_initctx(ctx, &mut ttyctx, 1, 1);
            ttyctx.bg = bg;
            write_or_redraw_pane(ctx, &mut ttyctx, |tty, ctx, screen| {
                tty_cmd_reverseindex(tty, ctx, screen)
            });
        } else if cy > 0 {
            screen_write_set_cursor(ctx, -1, cy.wrapping_sub(1) as c_int);
        }
    }
}
pub(super) unsafe fn screen_write_scrollregion(
    ctx: &mut screen_write_ctx,
    mut rupper: u_int,
    mut rlower: u_int,
) {
    unsafe {
        let bottom = RustScreen::grid(ctx.screen()).sy.wrapping_sub(1);
        if rupper > bottom {
            rupper = bottom;
        }
        if rlower > bottom {
            rlower = bottom;
        }
        if rupper >= rlower {
            return;
        }
        screen_write_collect_flush(ctx, 0, c"screen_write_scrollregion");
        screen_write_set_cursor(ctx, 0, 0);
        let s = ctx.screen_mut();
        s.0.rupper = rupper;
        s.0.rlower = rlower;
    }
}
pub(super) unsafe fn screen_write_linefeed(ctx: &mut screen_write_ctx, wrapped: c_int, bg: u_int) {
    unsafe {
        let s = ctx.screen_mut();
        let (cx, cy, upper, lower) = (s.0.cx, s.0.cy, s.0.rupper, s.0.rlower);
        let grid = RustScreen::grid_mut(s);
        let line = grid.hsize.wrapping_add(cy);
        let gl = grid_get_line(grid, line);
        if wrapped != 0 {
            gl.flags |= GRID_LINE_WRAPPED;
        }
        log_debug(
            c"%s: at %u,%u (region %u-%u)",
            fmt_args![c"screen_write_linefeed", cx, cy, upper, lower],
        );
        if bg != ctx.bg {
            screen_write_collect_flush(ctx, 1, c"screen_write_linefeed");
            ctx.bg = bg;
        }
        let s = ctx.screen_mut();
        let (cy, upper, lower) = (s.0.cy, s.0.rupper, s.0.rlower);
        if cy == lower {
            grid_view_scroll_region_up(RustScreen::grid_mut(s), upper, lower, bg);
            screen_write_collect_scroll(ctx, bg);
            ctx.scrolled = ctx.scrolled.wrapping_add(1);
        } else if cy < RustScreen::grid(s).sy.wrapping_sub(1) {
            screen_write_set_cursor(ctx, -1, cy.wrapping_add(1) as c_int);
        }
    }
}

/// The number of lines a scroll may move: at least one, and never more than
/// the scroll region holds.
fn lines_in_region(s: &RustScreen, lines: u_int) -> u_int {
    let region = s.0.rlower.wrapping_sub(s.0.rupper).wrapping_add(1);
    if lines == 0 {
        1
    } else if lines > region {
        region
    } else {
        lines
    }
}

pub(super) unsafe fn screen_write_scrollup(ctx: &mut screen_write_ctx, lines: u_int, bg: u_int) {
    unsafe {
        let lines = lines_in_region(ctx.screen(), lines);
        if bg != ctx.bg {
            screen_write_collect_flush(ctx, 1, c"screen_write_scrollup");
            ctx.bg = bg;
        }
        for _ in 0..lines {
            let s = ctx.screen_mut();
            let (upper, lower) = (s.0.rupper, s.0.rlower);
            grid_view_scroll_region_up(RustScreen::grid_mut(s), upper, lower, bg);
            screen_write_collect_scroll(ctx, bg);
        }
        ctx.scrolled = ctx.scrolled.wrapping_add(lines);
    }
}
pub(super) unsafe fn screen_write_scrolldown(ctx: &mut screen_write_ctx, lines: u_int, bg: u_int) {
    unsafe {
        let mut ttyctx = tty_ctx::default();
        screen_write_initctx(ctx, &mut ttyctx, 1, 1);
        ttyctx.bg = bg;
        let lines = lines_in_region(ctx.screen(), lines);
        for _ in 0..lines {
            let s = ctx.screen_mut();
            let (upper, lower) = (s.0.rupper, s.0.rlower);
            grid_view_scroll_region_down(RustScreen::grid_mut(s), upper, lower, bg);
        }
        screen_write_collect_flush(ctx, 0, c"screen_write_scrolldown");
        ttyctx.value = TtyCtxValue::Num(lines);
        write_or_redraw_pane(ctx, &mut ttyctx, |tty, ctx, screen| {
            tty_cmd_scrolldown(tty, ctx, screen)
        });
    }
}

pub(super) unsafe fn screen_write_carriagereturn(ctx: &mut screen_write_ctx) {
    unsafe { screen_write_set_cursor(ctx, 0, -1) }
}

/// Where the pane being written to sits in its window.
///
/// This is only asked for once a clear has found the pane obscured, which
/// `screen_write_pane_is_obscured` only answers for a context that has a
/// pane; the C's arm for a context without one is gone with the conversion,
/// and it only set offsets that were already zero.
fn pane_offset(ctx: &screen_write_ctx) -> (u_int, u_int) {
    let geometry = ctx
        .pane()
        .expect("an obscured writer has a pane")
        .geometry();
    (geometry.x as u_int, geometry.y as u_int)
}

/// Collects a clear of whatever is visible of `nx` columns from `px` on line
/// `y`, which is what a clear falls back to when a pane in front of this one
/// means the terminal cannot be told to clear the line itself.
unsafe fn collect_visible_clear(
    ctx: &mut screen_write_ctx,
    (xoff, yoff): (u_int, u_int),
    y: u_int,
    px: u_int,
    nx: u_int,
    bg: u_int,
) {
    unsafe {
        let mut ranges = visible_ranges::default();
        ranges.set_visible_ranges(
            ctx.pane(),
            xoff.wrapping_add(px) as c_int,
            yoff.wrapping_add(y) as c_int,
            nx,
        );
        for ri in &ranges.ranges[..ranges.used as usize] {
            if ri.nx != 0 {
                screen_write_collect_insert_clear(ctx, ri.px.wrapping_sub(xoff), ri.nx, bg);
            }
        }
    }
}

pub(super) unsafe fn screen_write_clearendofscreen(ctx: &mut screen_write_ctx, bg: u_int) {
    unsafe {
        let mut ttyctx = tty_ctx::default();
        let sx: u_int = RustScreen::grid(ctx.screen()).sx;
        let sy: u_int = RustScreen::grid(ctx.screen()).sy;
        screen_write_initctx(ctx, &mut ttyctx, 1, 1);
        ttyctx.bg = bg;
        let (cx, cy) = ctx.screen().cursor();
        if cx == 0
            && cy == 0
            && RustScreen::grid(ctx.screen()).flags & GRID_HISTORY != 0
            && ctx
                .pane()
                .is_some_and(|pane| (pane.options_ref()).number(c"scroll-on-clear") != 0)
        {
            grid_view_clear_history(RustScreen::grid_mut(ctx.screen_mut()), bg);
        } else {
            if cx <= sx.wrapping_sub(1) {
                grid_view_clear(
                    RustScreen::grid_mut(ctx.screen_mut()),
                    cx,
                    cy,
                    sx.wrapping_sub(cx),
                    1,
                    bg,
                );
            }
            grid_view_clear(
                RustScreen::grid_mut(ctx.screen_mut()),
                0,
                cy.wrapping_add(1),
                sx,
                sy.wrapping_sub(cy.wrapping_add(1)),
                bg,
            );
        }
        screen_write_collect_clear(ctx, cy.wrapping_add(1), sy.wrapping_sub(cy.wrapping_add(1)));
        screen_write_collect_flush(ctx, 0, c"screen_write_clearendofscreen");
        if !pane_obscured(&ttyctx) {
            tty_write(
                |tty, ttyctx| tty_cmd_clearendofscreen(tty, ttyctx, ctx.screen()),
                &mut ttyctx,
            );
            return;
        }
        let (ocx, ocy) = ctx.screen().cursor();
        let offset = pane_offset(ctx);
        if ocx <= sx.wrapping_sub(1) {
            collect_visible_clear(ctx, offset, ocy, ocx, sx.wrapping_sub(ocx), bg);
        }
        for y in ocy.wrapping_add(1)..sy {
            screen_write_set_cursor(ctx, 0, y as c_int);
            collect_visible_clear(ctx, offset, y, 0, sx, bg);
        }
        screen_write_set_cursor(ctx, ocx as c_int, ocy as c_int);
    }
}
/// Clears from the start of the screen to the cursor.
///
/// Two shapes of the C are kept as they are. The walk over the lines above
/// the cursor tests the cursor's own row, which the first step of the walk
/// has already moved to the top, so only the first line is collected; and the
/// clear of the cursor's row reads the cursor's column after that same move,
/// so it is one column wide whatever column the cursor was in.
pub(super) unsafe fn screen_write_clearstartofscreen(ctx: &mut screen_write_ctx, bg: u_int) {
    unsafe {
        let mut ttyctx = tty_ctx::default();
        let sx: u_int = RustScreen::grid(ctx.screen()).sx;
        screen_write_initctx(ctx, &mut ttyctx, 1, 1);
        ttyctx.bg = bg;
        let (cx, cy) = ctx.screen().cursor();
        if cy > 0 {
            grid_view_clear(RustScreen::grid_mut(ctx.screen_mut()), 0, 0, sx, cy, bg);
        }
        if cx > sx.wrapping_sub(1) {
            grid_view_clear(RustScreen::grid_mut(ctx.screen_mut()), 0, cy, sx, 1, bg);
        } else {
            grid_view_clear(
                RustScreen::grid_mut(ctx.screen_mut()),
                0,
                cy,
                cx.wrapping_add(1),
                1,
                bg,
            );
        }
        screen_write_collect_clear(ctx, 0, cy);
        screen_write_collect_flush(ctx, 0, c"screen_write_clearstartofscreen");
        if !pane_obscured(&ttyctx) {
            tty_write(
                |tty, ttyctx| tty_cmd_clearstartofscreen(tty, ttyctx, ctx.screen()),
                &mut ttyctx,
            );
            return;
        }
        let (ocx, ocy) = ctx.screen().cursor();
        let offset = pane_offset(ctx);
        let mut y = 0;
        while y < ctx.screen().0.cy {
            screen_write_set_cursor(ctx, 0, y as c_int);
            collect_visible_clear(ctx, offset, y, 0, sx, bg);
            y = y.wrapping_add(1);
        }
        screen_write_set_cursor(ctx, 0, ocy as c_int);
        let width = ctx.screen().0.cx.wrapping_add(1);
        collect_visible_clear(ctx, offset, ocy, 0, width, bg);
        screen_write_set_cursor(ctx, ocx as c_int, ocy as c_int);
    }
}
pub(super) unsafe fn screen_write_clearscreen(ctx: &mut screen_write_ctx, bg: u_int) {
    unsafe {
        let mut ttyctx = tty_ctx::default();
        let sx: u_int = RustScreen::grid(ctx.screen()).sx;
        let sy: u_int = RustScreen::grid(ctx.screen()).sy;
        screen_write_initctx(ctx, &mut ttyctx, 1, 1);
        ttyctx.bg = bg;
        if RustScreen::grid(ctx.screen()).flags & GRID_HISTORY != 0
            && ctx
                .pane()
                .is_some_and(|pane| (pane.options_ref()).number(c"scroll-on-clear") != 0)
        {
            grid_view_clear_history(RustScreen::grid_mut(ctx.screen_mut()), bg);
        } else {
            grid_view_clear(RustScreen::grid_mut(ctx.screen_mut()), 0, 0, sx, sy, bg);
        }
        screen_write_collect_clear(ctx, 0, sy);
        if !pane_obscured(&ttyctx) {
            tty_write(
                |tty, ttyctx| tty_cmd_clearscreen(tty, ttyctx, ctx.screen()),
                &mut ttyctx,
            );
            return;
        }
        let (ocx, ocy) = ctx.screen().cursor();
        let offset = pane_offset(ctx);
        for y in 0..sy {
            screen_write_set_cursor(ctx, 0, y as c_int);
            collect_visible_clear(ctx, offset, y, 0, sx, bg);
        }
        screen_write_set_cursor(ctx, ocx as c_int, ocy as c_int);
    }
}

pub(super) unsafe fn screen_write_clearhistory(ctx: &mut screen_write_ctx) {
    {
        RustScreen::grid_mut(ctx.screen_mut()).clear_history();
    }
}
pub(super) unsafe fn screen_write_fullredraw(ctx: &mut screen_write_ctx) {
    unsafe {
        let mut ttyctx = tty_ctx::default();
        screen_write_collect_flush(ctx, 0, c"screen_write_fullredraw");
        screen_write_initctx(ctx, &mut ttyctx, 1, 0);
        if let Some(redraw_cb) = ttyctx.redraw_cb {
            redraw_cb(&ttyctx);
        }
    }
}
/// The bytes collected for one line, which is as wide as the grid was when
/// the line first collected. Only a line that has collected text has them,
/// which is every line an item of type `TEXT` hangs on; the rest read empty.
fn line_text(cl: &screen_write_cline) -> &[u8] {
    cl.data.as_deref().unwrap_or_default()
}

/// Cuts what is already collected on line `y` out of the way of `used`
/// columns from `x`, and answers the item the new one is to go in front of,
/// or [`CITEM_NONE`] for the end of the line, along with whether any item
/// given up whole started the line wrapped.
///
/// An item given up whole is read back after it has gone to the free list:
/// the free only relinks it, so the wrapped flag it carried is still there
/// and is carried on to the item replacing it. That is what the C did and it
/// is pinned by a test.
unsafe fn screen_write_collect_trim(
    ctx: &mut screen_write_ctx,
    y: u_int,
    x: u_int,
    used: u_int,
) -> (CItem, bool) {
    unsafe {
        let mut wrapped = false;
        let items = &mut write_list(ctx.screen_mut())[y as usize].items;
        let sx = x;
        let ex = x.wrapping_add(used).wrapping_sub(1);
        let name = c"screen_write_collect_trim";
        if items.is_empty() {
            return (CITEM_NONE, wrapped);
        }
        for ci in citem_list(items) {
            let item = citem_snapshot(ci);
            let csx = item.x;
            let cex = item.x.wrapping_add(item.used).wrapping_sub(1);
            if cex < sx {
                log_debug(
                    c"%s: %p %u-%u before %u-%u",
                    fmt_args![name, ci, csx, cex, sx, ex],
                );
            } else if csx > ex {
                log_debug(
                    c"%s: %p %u-%u after %u-%u",
                    fmt_args![name, ci, csx, cex, sx, ex],
                );
                return (ci, wrapped);
            } else if csx >= sx && cex <= ex {
                log_debug(
                    c"%s: %p %u-%u inside %u-%u",
                    fmt_args![name, ci, csx, cex, sx, ex],
                );
                citem_remove(items, ci);
                screen_write_free_citem(ci);
                if csx == 0 && item.wrapped != 0 {
                    wrapped = true;
                }
            } else if csx < sx && cex >= sx && cex <= ex {
                log_debug(
                    c"%s: %p %u-%u start %u-%u",
                    fmt_args![name, ci, csx, cex, sx, ex],
                );
                with_citem(ci, |item| item.used = sx.wrapping_sub(csx));
                let item = citem_snapshot(ci);
                log_debug(
                    c"%s: %p now %u-%u",
                    fmt_args![
                        name,
                        ci,
                        item.x,
                        item.x.wrapping_add(item.used).wrapping_add(1)
                    ],
                );
            } else if cex > ex && csx >= sx && csx <= ex {
                log_debug(
                    c"%s: %p %u-%u end %u-%u",
                    fmt_args![name, ci, csx, cex, sx, ex],
                );
                with_citem(ci, |item| {
                    item.x = ex.wrapping_add(1);
                    item.used = cex.wrapping_sub(ex);
                });
                let item = citem_snapshot(ci);
                log_debug(
                    c"%s: %p now %u-%u",
                    fmt_args![
                        name,
                        ci,
                        item.x,
                        item.x.wrapping_add(item.used).wrapping_add(1)
                    ],
                );
                return (ci, wrapped);
            } else {
                log_debug(
                    c"%s: %p %u-%u under %u-%u",
                    fmt_args![name, ci, csx, cex, sx, ex],
                );
                let ci2 = screen_write_get_citem();
                with_citem(ci2, |second| {
                    second.type_0 = item.type_0;
                    second.bg = item.bg;
                    second.gc = item.gc;
                    second.x = ex.wrapping_add(1);
                    second.used = cex.wrapping_sub(ex);
                });
                citem_insert_after(items, ci, ci2);
                with_citem(ci, |first| first.used = sx.wrapping_sub(csx));
                let first = citem_snapshot(ci);
                let second = citem_snapshot(ci2);
                log_debug(
                    c"%s: %p now %u-%u (%p) and %u-%u (%p)",
                    fmt_args![
                        name,
                        ci,
                        first.x,
                        first.x.wrapping_add(first.used).wrapping_sub(1),
                        ci,
                        second.x,
                        second.x.wrapping_add(second.used).wrapping_sub(1),
                        ci2
                    ],
                );
                return (ci2, wrapped);
            }
        }
        (CITEM_NONE, wrapped)
    }
}

/// Gives up everything collected on `n` lines from `y`.
unsafe fn screen_write_collect_clear(ctx: &mut screen_write_ctx, y: u_int, n: u_int) {
    {
        let wl = write_list(ctx.screen_mut());
        for i in y..y.wrapping_add(n) {
            citem_free_all(&mut wl[i as usize].items);
        }
    }
}

/// Moves what is collected inside the scroll region up a line, taking the top
/// line's text buffer round to the bottom, and collects a clear of the line
/// that comes in at the bottom.
unsafe fn screen_write_collect_scroll(ctx: &mut screen_write_ctx, bg: u_int) {
    unsafe {
        let s = ctx.screen();
        log_debug(
            c"%s: at %u,%u (region %u-%u)",
            fmt_args![
                c"screen_write_collect_scroll",
                s.0.cx,
                s.0.cy,
                s.0.rupper,
                s.0.rlower
            ],
        );
        screen_write_collect_clear(ctx, s.0.rupper, 1);
        let s = ctx.screen();
        let (rupper, rlower, sx) = (s.0.rupper, s.0.rlower, RustScreen::grid(s).sx);
        let wl = write_list(ctx.screen_mut());
        let saved = wl[rupper as usize].data.take();
        let mut y = rupper;
        while y < rlower {
            let (above, below) = wl.split_at_mut(y.wrapping_add(1) as usize);
            let (into, from) = (&mut above[y as usize], &mut below[0]);
            into.items.append(&mut from.items);
            into.data = from.data.take();
            y = y.wrapping_add(1);
        }
        wl[rlower as usize].data = saved;
        let ci = screen_write_get_citem();
        with_citem(ci, |item| {
            item.x = 0;
            item.used = sx;
            item.type_0 = CLEAR;
            item.bg = bg;
        });
        citem_insert_tail(&mut wl[rlower as usize].items, ci);
    }
}

/// Tells the terminal about the lines that have been scrolled, or redraws the
/// pane when something in front of it means it cannot be told. Answers
/// whether the scroll was written out.
unsafe fn screen_write_collect_flush_scrolled(ctx: &mut screen_write_ctx) -> c_int {
    unsafe {
        let mut ttyctx = tty_ctx::default();
        screen_write_initctx(ctx, &mut ttyctx, 1, 1);
        if pane_obscured(&ttyctx) && ctx.pane().is_some() {
            screen_write_redraw_pane(ctx, &mut ttyctx);
            return 0;
        }
        let s = ctx.screen();
        log_debug(
            c"%s: scrolled %u (region %u-%u)",
            fmt_args![
                c"screen_write_collect_flush_scrolled",
                ctx.scrolled,
                s.0.rupper,
                s.0.rlower
            ],
        );
        let region = s.0.rlower.wrapping_sub(s.0.rupper).wrapping_add(1);
        if ctx.scrolled > region {
            ctx.scrolled = region;
        }
        if let Some(pane) = ctx.pane() {
            let geometry = pane.geometry();
            let bottom = (geometry.y as u_int).wrapping_add(geometry.height);
            let window = pane
                .window_context()
                .expect("a writing pane has a window context");
            let height = window.dimensions().size.height;
            if bottom > height {
                ttyctx.orlower = ttyctx.orlower.wrapping_sub(bottom.wrapping_sub(height));
            }
        }
        ttyctx.value = TtyCtxValue::Num(ctx.scrolled);
        ttyctx.bg = ctx.bg;
        tty_write(
            |tty, ttyctx| tty_cmd_scrollup(tty, ttyctx, ctx.screen()),
            &mut ttyctx,
        );
        if let Some(pane) = ctx.pane_mut() {
            *pane.flags_mut() |= PANE_REDRAWSCROLLBAR;
        }
        1
    }
}

/// Writes out whatever is collected on line `y` and is visible, and answers
/// how many items that was. What is written is given up; what a pane in front
/// of this one hides is left collected.
///
/// Two guards the C had are gone with the conversion. The overlap test above
/// them has already found that the item and the range meet, and both are at
/// least one column wide, so the start of the part to write is always before
/// its end and the length is never zero.
///
/// The `fatalx` arm for a list out of order stays, and is exempt from the
/// coverage requirement: its only effect is to stop the process.
unsafe fn screen_write_collect_flush_line(ctx: &mut screen_write_ctx, y: u_int) -> u_int {
    unsafe {
        let mut last: u_int = UINT_MAX;
        let mut items: u_int = 0;
        let (wsx, wsy, xoff, yoff) = match ctx.pane() {
            Some(pane) => {
                let window = pane
                    .window_context()
                    .expect("a writing pane has a window context");
                let size = window.dimensions().size;
                let geometry = pane.geometry();
                (size.width, size.height, geometry.x, geometry.y)
            }
            None => (
                RustScreen::grid(ctx.screen()).sx,
                RustScreen::grid(ctx.screen()).sy,
                0,
                0,
            ),
        };
        if y.wrapping_add(yoff as u_int) >= wsy {
            return 0;
        }
        let mut ranges = visible_ranges::default();
        ranges.set_visible_ranges(ctx.pane(), 0, y.wrapping_add(yoff as u_int) as c_int, wsx);
        for ci in citem_list(&ctx.screen().0.write_list[y as usize].items) {
            let item = citem_snapshot(ci);
            log_debug(
                c"collect list: x=%u (last %u), y=%u, used=%u",
                fmt_args![item.x, last, y, item.used],
            );
            if last != UINT_MAX && item.x <= last {
                fatalx(c"collect list bad order: %u <= %u", fmt_args![item.x, last]);
            }
            let mut written = false;
            for ri in &ranges.ranges[..ranges.used as usize] {
                if ri.nx == 0 {
                    continue;
                }
                let r_start = ri.px as c_int;
                let r_end = ri.px.wrapping_add(ri.nx) as c_int;
                let c_start = item.x as c_int;
                let c_end = item.x.wrapping_add(item.used) as c_int;
                if c_start + xoff >= r_end || c_end + xoff <= r_start {
                    continue;
                }
                let w_start = if r_start > c_start + xoff {
                    r_start - xoff
                } else {
                    c_start
                };
                let w_end = if c_end + xoff > r_end {
                    r_end - xoff
                } else {
                    c_end
                };
                let w_length = (w_end - w_start) as u_int;
                let mut ttyctx = tty_ctx::default();
                screen_write_set_cursor(ctx, w_start, y as c_int);
                if item.type_0 == CLEAR {
                    screen_write_initctx(ctx, &mut ttyctx, 1, 0);
                    ttyctx.bg = item.bg;
                    ttyctx.value = TtyCtxValue::Num(w_length);
                    tty_write(
                        |tty, ttyctx| tty_cmd_clearcharacter(tty, ttyctx, ctx.screen()),
                        &mut ttyctx,
                    );
                } else {
                    screen_write_initctx(ctx, &mut ttyctx, 0, 0);
                    ttyctx.cell = Some(item.gc);
                    if item.wrapped != 0 {
                        ttyctx.flags |= TTY_CTX_WRAPPED;
                    }
                    let text =
                        &line_text(&ctx.screen().0.write_list[y as usize])[w_start as usize..];
                    ttyctx.value = TtyCtxValue::Data(tty_ctx_data {
                        data: &text[..w_length as usize],
                    });
                    tty_write(
                        |tty, ttyctx| tty_cmd_cells(tty, ttyctx, ctx.screen()),
                        &mut ttyctx,
                    );
                }
                items = items.wrapping_add(1);
                written = true;
            }
            if written {
                last = item.x;
                citem_remove(&mut write_list(ctx.screen_mut())[y as usize].items, ci);
                screen_write_free_citem(ci);
            }
        }
        items
    }
}

/// Writes everything collected so far onto the grid.
///
/// `scroll_only` stops once the scrolled lines have been dealt with, leaving
/// the text where it is. A screen in a synchronised update, and one whose
/// scrolled lines could not be written, give up what they collected instead
/// of writing it; the C reached that second case by jumping past the flush,
/// and it is the tail of this function now.
unsafe fn screen_write_collect_flush(ctx: &mut screen_write_ctx, scroll_only: c_int, from: &CStr) {
    unsafe {
        let mut give_up = ctx.screen().0.mode & MODE_SYNC != 0;
        if !give_up {
            if ctx.scrolled != 0 && screen_write_collect_flush_scrolled(ctx) == 0 {
                give_up = true;
            } else {
                ctx.scrolled = 0;
            }
        }
        if !give_up {
            ctx.bg = 8;
            if scroll_only != 0 {
                return;
            }
            let (cx, cy) = ctx.screen().cursor();
            let mut items: u_int = 0;
            for y in 0..RustScreen::grid(ctx.screen()).sy {
                items = items.wrapping_add(screen_write_collect_flush_line(ctx, y));
            }
            let s = ctx.screen_mut();
            s.0.cx = cx;
            s.0.cy = cy;
            log_debug(
                c"%s: flushed %u items (%s)",
                fmt_args![c"screen_write_collect_flush", items, from],
            );
            return;
        }
        for cl in write_list(ctx.screen_mut()) {
            citem_free_all(&mut cl.items);
        }
        ctx.scrolled = 0;
        ctx.bg = 8;
    }
}

/// Puts `ci` on the line the cursor is on, in the order the writing calls
/// expect, and gives the context a fresh item to fill in.
unsafe fn screen_write_collect_insert(ctx: &mut screen_write_ctx, ci: CItem) {
    unsafe {
        let cy = ctx.screen().0.cy;
        let item = citem_snapshot(ci);
        let (before, wrapped) = screen_write_collect_trim(ctx, cy, item.x, item.used);
        if wrapped {
            with_citem(ci, |item| item.wrapped = 1);
        }
        let items = &mut write_list(ctx.screen_mut())[cy as usize].items;
        if before == CITEM_NONE {
            citem_insert_tail(items, ci);
        } else {
            citem_insert_before(items, before, ci);
        }
        ctx.item = screen_write_get_citem();
    }
}
unsafe fn screen_write_collect_insert_clear(
    ctx: &mut screen_write_ctx,
    px: u_int,
    nx: u_int,
    bg: u_int,
) {
    unsafe {
        let ci: CItem = ctx.item;
        if nx != 0 {
            with_citem(ci, |item| {
                item.x = px;
                item.used = nx;
                item.type_0 = CLEAR;
                item.bg = bg;
            });
            screen_write_collect_insert(ctx, ci);
        }
    }
}

/// Finishes the run of text being collected: it goes onto the grid, and the
/// padding cells of any wide character it wrote over are erased.
pub(super) unsafe fn screen_write_collect_end(ctx: &mut screen_write_ctx) {
    unsafe {
        let ci: CItem = ctx.item;
        let name = c"screen_write_collect_end";
        let mut bci: CItem = CITEM_NONE;
        let mut gc = grid_cell::default();
        let mut item = citem_snapshot(ci);
        if item.used == 0 {
            return;
        }
        let cx = ctx.screen().0.cx;
        with_citem(ci, |item| item.x = cx);
        item.x = cx;
        screen_write_collect_insert(ctx, ci);
        let next = ctx.item;
        let s = ctx.screen_mut();
        let cy = s.0.cy;
        log_debug(
            c"%s: %u %.*s (at %u,%u)",
            fmt_args![
                name,
                item.used,
                item.used as c_int,
                &line_text(&s.0.write_list[cy as usize])[item.x as usize..],
                s.0.cx,
                s.0.cy
            ],
        );
        if s.0.cx != 0 {
            let mut xx = s.0.cx;
            while xx > 0 {
                gc = grid_view_get_cell(RustScreen::grid(s), xx, s.0.cy);
                if !(gc.flags as c_int) & GRID_FLAG_PADDING != 0 {
                    break;
                }
                grid_view_set_cell(RustScreen::grid_mut(s), xx, cy, &grid_default_cell);
                log_debug(
                    c"%s: padding erased (before) at %u (cx %u)",
                    fmt_args![name, xx, s.0.cx],
                );
                xx = xx.wrapping_sub(1);
            }
            if xx != s.0.cx {
                if xx == 0 {
                    gc = grid_view_get_cell(RustScreen::grid(s), 0, s.0.cy);
                }
                if gc.data.width as c_int > 1 || gc.flags as c_int & GRID_FLAG_PADDING != 0 {
                    grid_view_set_cell(RustScreen::grid_mut(s), xx, cy, &grid_default_cell);
                    log_debug(
                        c"%s: padding erased (before) at %u (cx %u)",
                        fmt_args![name, xx, s.0.cx],
                    );
                }
                bci = next;
                with_citem(bci, |item| {
                    item.type_0 = CLEAR;
                    item.x = xx;
                    item.bg = 8;
                    item.used = s.0.cx.wrapping_sub(xx);
                });
                let before = citem_snapshot(bci);
                log_debug(
                    c"%s: padding erased (before): from %u, size %u",
                    fmt_args![name, before.x, before.used],
                );
            }
        }
        grid_view_set_cells(
            s.0.grid.as_deref_mut().expect("a screen holds a grid"),
            s.0.cx,
            s.0.cy,
            &item.gc,
            &line_text(&s.0.write_list[cy as usize])[item.x as usize..][..item.used as usize],
        );
        if bci != CITEM_NONE {
            screen_write_collect_insert(ctx, bci);
        }
        screen_write_set_cursor(ctx, ctx.screen().0.cx.wrapping_add(item.used) as c_int, -1);
        let aci = ctx.item;
        let s = ctx.screen_mut();
        let cy = s.0.cy;
        let mut xx = s.0.cx;
        while xx < RustScreen::grid(s).sx {
            gc = grid_view_get_cell(RustScreen::grid(s), xx, s.0.cy);
            if !(gc.flags as c_int) & GRID_FLAG_PADDING != 0 {
                break;
            }
            grid_view_set_cell(RustScreen::grid_mut(s), xx, cy, &grid_default_cell);
            log_debug(
                c"%s: padding erased (after) at %u (cx %u)",
                fmt_args![name, xx, s.0.cx],
            );
            xx = xx.wrapping_add(1);
        }
        if xx != s.0.cx {
            with_citem(aci, |item| {
                item.type_0 = CLEAR;
                item.x = s.0.cx;
                item.bg = 8;
                item.used = xx.wrapping_sub(s.0.cx);
            });
            let after = citem_snapshot(aci);
            log_debug(
                c"%s: padding erased (after): from %u, size %u",
                fmt_args![name, after.x, after.used],
            );
            screen_write_collect_insert(ctx, aci);
        }
    }
}

/// Collects one character to be written later, or writes it out now when it
/// is one the collecting cannot carry.
pub(super) unsafe fn screen_write_collect_add(ctx: &mut screen_write_ctx, gc: &grid_cell) {
    unsafe {
        let s = ctx.screen();
        let sx: u_int = RustScreen::grid(s).sx;
        let collect = gc.data.width == 1
            && gc.data.size == 1
            && gc.data.data[0] < 0x7f
            && gc.flags as c_int & GRID_FLAG_TAB == 0
            && gc.attr as c_int & GRID_ATTR_CHARSET == 0
            && s.0.mode & MODE_WRAP != 0
            && s.0.mode & MODE_INSERT == 0
            && s.0.sel.is_none();
        if !collect {
            screen_write_collect_end(ctx);
            screen_write_collect_flush(ctx, 0, c"screen_write_collect_add");
            screen_write_cell(ctx, gc);
            return;
        }
        if s.0.cx > sx.wrapping_sub(1)
            || citem_snapshot(ctx.item).used > sx.wrapping_sub(1).wrapping_sub(s.0.cx)
        {
            screen_write_collect_end(ctx);
        }
        let ci: CItem = ctx.item;
        let s = ctx.screen();
        if s.0.cx > sx.wrapping_sub(1) {
            log_debug(
                c"%s: wrapped at %u,%u",
                fmt_args![c"screen_write_collect_add", s.0.cx, s.0.cy],
            );
            with_citem(ci, |item| item.wrapped = 1);
            screen_write_linefeed(ctx, 1, 8);
            screen_write_set_cursor(ctx, 0, -1);
        }
        let item = citem_snapshot(ci);
        if item.used == 0 {
            with_citem(ci, |item| item.gc = *gc);
        }
        let (cx, cy) = ctx.screen().cursor();
        let cl = &mut write_list(ctx.screen_mut())[cy as usize];
        let text = cl
            .data
            .get_or_insert_with(|| std::vec::from_elem(0u8, sx as usize).into_boxed_slice());
        text[cx.wrapping_add(item.used) as usize] = gc.data.data[0];
        with_citem(ci, |item| item.used = item.used.wrapping_add(1));
    }
}

/// Whether the cell is already what the packed entry holds, which is what
/// lets a write be skipped.
fn cell_matches_entry(gc: &grid_cell, gce: &grid_cell_entry) -> bool {
    unsafe {
        let data = &gce.c2rust_unnamed.data;
        gce.flags as c_int & GRID_FLAG_EXTENDED == 0
            && gc.flags as c_int == gce.flags as c_int
            && gc.attr as c_int == data.attr as c_int
            && gc.fg == data.fg as c_int
            && gc.bg == data.bg as c_int
            && gc.data.width as c_int == 1
            && gc.data.size as c_int == 1
            && data.data == gc.data.data[0]
    }
}

/// Writes one character at the cursor, wrapping to the next line first when
/// it does not fit and putting padding cells behind a wide one.
pub(super) unsafe fn screen_write_cell(ctx: &mut screen_write_ctx, gc: &grid_cell) {
    unsafe {
        let ud: &utf8_data = &gc.data;
        let mut tmp_gc = grid_cell::default();
        let mut now_gc;
        let mut ttyctx = tty_ctx::default();
        let sx: u_int = RustScreen::grid(ctx.screen()).sx;
        let sy: u_int = RustScreen::grid(ctx.screen()).sy;
        let width: u_int = ud.width as u_int;
        let mut skip = true;
        let mut redraw = false;
        if gc.flags as c_int & GRID_FLAG_PADDING != 0 {
            return;
        }
        if screen_write_combine(ctx, gc) != 0 {
            return;
        }
        screen_write_collect_flush(ctx, 1, c"screen_write_cell");
        let s = ctx.screen_mut();
        if s.0.mode & MODE_WRAP == 0
            && width > 1
            && (width > sx || s.0.cx != sx && s.0.cx > sx.wrapping_sub(width))
        {
            return;
        }
        if s.0.mode & MODE_INSERT != 0 {
            let (cx, cy) = s.cursor();
            grid_view_insert_cells(RustScreen::grid_mut(s), cx, cy, width, 8);
            skip = false;
        }
        if s.0.mode & MODE_WRAP != 0 && s.0.cx > sx.wrapping_sub(width) {
            log_debug(
                c"%s: wrapped at %u,%u",
                fmt_args![c"screen_write_cell", s.0.cx, s.0.cy],
            );
            screen_write_linefeed(ctx, 1, 8);
            screen_write_set_cursor(ctx, 0, -1);
            screen_write_collect_flush(ctx, 0, c"screen_write_cell");
        }
        let s = ctx.screen();
        if s.0.cx > sx.wrapping_sub(width) || s.0.cy > sy.wrapping_sub(1) {
            return;
        }
        screen_write_initctx(ctx, &mut ttyctx, 0, 0);
        let s = ctx.screen_mut();
        let (cx, cy) = s.cursor();
        let gd = RustScreen::grid_mut(s);
        let line_y = gd.hsize.wrapping_add(cy);
        let extended = grid_get_line(gd, line_y).flags & GRID_LINE_EXTENDED != 0;
        if extended {
            now_gc = grid_view_get_cell(gd, cx, cy);
            if screen_write_overwrite(ctx, &mut now_gc, width) != 0 {
                redraw = true;
                skip = false;
            }
        }
        let s = ctx.screen_mut();
        let (cx, cy) = s.cursor();
        let gd = RustScreen::grid_mut(s);
        let mut xx = cx.wrapping_add(1);
        while xx < cx.wrapping_add(width) {
            log_debug(
                c"%s: new padding at %u,%u",
                fmt_args![c"screen_write_cell", xx, cy],
            );
            grid_view_set_padding(gd, xx, cy);
            skip = false;
            xx = xx.wrapping_add(1);
        }
        if skip {
            let gl = grid_get_line(gd, line_y);
            skip = if cx >= gl.cellsize() {
                grid_cells_equal(gc, &grid_default_cell) != 0
            } else {
                cell_matches_entry(gc, &gl.celldata()[cx as usize])
            };
        }
        let (cx, cy) = s.cursor();
        let selected = s.check_selection(cx, cy) as c_int;
        let gd = RustScreen::grid_mut(s);
        if selected != 0 && gc.flags as c_int & GRID_FLAG_SELECTED == 0 {
            tmp_gc = *gc;
            tmp_gc.flags = (tmp_gc.flags as c_int | GRID_FLAG_SELECTED) as u_char;
            grid_view_set_cell(gd, cx, cy, &tmp_gc);
        } else if selected == 0 && gc.flags as c_int & GRID_FLAG_SELECTED != 0 {
            tmp_gc = *gc;
            tmp_gc.flags = (tmp_gc.flags as c_int & !GRID_FLAG_SELECTED) as u_char;
            grid_view_set_cell(gd, cx, cy, &tmp_gc);
        } else if !skip {
            grid_view_set_cell(gd, cx, cy, gc);
        }
        if selected != 0 {
            skip = false;
        }
        let s = ctx.screen();
        let (xoff, yoff) = ctx.pane().map_or((0, 0), |pane| {
            let geometry = pane.geometry();
            (geometry.x, geometry.y)
        });
        let mut ranges = visible_ranges::default();
        ranges.set_visible_ranges(
            ctx.pane(),
            (xoff as u_int).wrapping_add(s.0.cx) as c_int,
            s.0.cy.wrapping_add(yoff as u_int) as c_int,
            width,
        );
        let not_wrap = (s.0.mode & MODE_WRAP == 0) as u_int;
        let next_x = if s.0.cx <= sx.wrapping_sub(not_wrap).wrapping_sub(width) {
            s.0.cx.wrapping_add(width)
        } else {
            sx.wrapping_sub(not_wrap)
        };
        screen_write_set_cursor(ctx, next_x as c_int, -1);
        if ctx.screen().0.mode & MODE_INSERT != 0 {
            screen_write_collect_flush(ctx, 0, c"screen_write_cell");
            ttyctx.value = TtyCtxValue::Num(width);
            tty_write(
                |tty, ttyctx| tty_cmd_insertcharacter(tty, ttyctx, ctx.screen()),
                &mut ttyctx,
            );
        }
        if skip || ctx.screen().0.mode & MODE_SYNC != 0 {
            return;
        }
        if redraw && ctx.pane().is_some() {
            screen_write_redraw_line(ctx, &mut ttyctx, ctx.screen().0.cy);
            return;
        }
        if selected != 0 {
            ctx.screen().select_cell(&mut tmp_gc, gc);
        } else {
            tmp_gc = *gc;
        }
        ttyctx.cell = Some(tmp_gc);
        let mut vis: u_int = 0;
        for range in &ranges.ranges[..ranges.used as usize] {
            vis = vis.wrapping_add(range.nx);
        }
        if vis >= width {
            tty_write(
                |tty, ttyctx| tty_cmd_cell(tty, ttyctx, ctx.screen()),
                &mut ttyctx,
            );
            return;
        }
        utf8_set(&mut tmp_gc.data, b' ');
        ttyctx.cell = Some(tmp_gc);
        for ri in &ranges.ranges[..ranges.used as usize] {
            let mut n = 0 as u_int;
            while n < ri.nx {
                ttyctx.ocx = (ri.px as c_int - xoff + n as c_int) as u_int;
                tty_write(
                    |tty, ttyctx| tty_cmd_cell(tty, ttyctx, ctx.screen()),
                    &mut ttyctx,
                );
                n = n.wrapping_add(1);
            }
        }
    }
}

/// Joins a character to the one in front of it when it is one that combines,
/// and answers whether it was taken that way rather than written on its own.
///
/// The guard on the size of the joined character is the C's, one byte too
/// lenient: a joined character of exactly 32 bytes still goes in, and
/// `utf8_from_data` cannot pack a size that large, so the cell reads back
/// with no size at all. It is kept as it is for parity.
unsafe fn screen_write_combine(ctx: &mut screen_write_ctx, gc: &grid_cell) -> c_int {
    unsafe {
        let ud: &utf8_data = &gc.data;
        let oo: &RustOptionsRef = global_options
            .as_ref()
            .expect("global options are initialized");
        let (mut cx, cy) = ctx.screen().cursor();
        let mut n: u_int;
        let mut last;
        let mut ttyctx = tty_ctx::default();
        let mut force_wide = false;
        let mut zero_width = false;
        if RustUtf8Compositor.is_hangul_filler(ud) {
            return 1;
        }
        if RustUtf8Compositor.is_zwj(ud) {
            zero_width = true;
        } else if RustUtf8Compositor.is_variation_selector(ud) {
            zero_width = true;
            if (oo).number(c"variation-selector-always-wide") != 0 {
                force_wide = true;
            }
        } else if ud.width as c_int == 0 {
            zero_width = true;
        }
        if (ud.size as c_int) < 2 || cx == 0 {
            return zero_width as c_int;
        }
        log_debug(
            c"%s: character %.*s at %u,%u (width %u)",
            fmt_args![
                c"screen_write_combine",
                ud.size as c_int,
                ud.data.as_slice(),
                cx,
                cy,
                ud.width as c_int
            ],
        );
        n = 1;
        let gd = RustScreen::grid(ctx.screen());
        last = grid_view_get_cell(gd, cx.wrapping_sub(n), cy);
        if cx != 1 && last.flags as c_int & GRID_FLAG_PADDING != 0 {
            n = 2;
            last = grid_view_get_cell(gd, cx.wrapping_sub(n), cy);
        }
        if n != last.data.width as u_int || last.flags as c_int & GRID_FLAG_PADDING != 0 {
            return zero_width as c_int;
        }
        if !zero_width {
            match RustUtf8Compositor.hangul_state(&last.data, ud) {
                HANGULJAMO_STATE_NOT_COMPOSABLE => return 1,
                HANGULJAMO_STATE_CHOSEONG => return 0,
                HANGULJAMO_STATE_NOT_HANGULJAMO => {
                    if RustUtf8Compositor.should_combine(&last.data, ud)
                        || RustUtf8Compositor.should_combine(ud, &last.data)
                    {
                        force_wide = true;
                    } else if !RustUtf8Compositor.has_zwj(&last.data) {
                        return 0;
                    }
                }
                _ => {}
            }
        }
        let size = last.data.size as usize;
        let more = ud.size as usize;
        if size + more > last.data.data.len() {
            return 0;
        }
        screen_write_collect_flush(ctx, 0, c"screen_write_combine");
        log_debug(
            c"%s: %.*s -> %.*s at %u,%u (offset %u, width %u)",
            fmt_args![
                c"screen_write_combine",
                ud.size as c_int,
                ud.data.as_slice(),
                last.data.size as c_int,
                last.data.data.as_slice(),
                cx.wrapping_sub(n),
                cy,
                n,
                last.data.width as c_int
            ],
        );
        let joining = &ud.data;
        last.data.data[size..size + more].copy_from_slice(&joining[..more]);
        last.data.size = (size + more) as u_char;
        if last.data.width as c_int == 1 && force_wide {
            last.data.width = 2;
            n = 2;
            cx = cx.wrapping_add(1);
        } else {
            force_wide = false;
        }
        let gd = RustScreen::grid_mut(ctx.screen_mut());
        grid_view_set_cell(gd, cx.wrapping_sub(n), cy, &last);
        if force_wide {
            grid_view_set_padding(gd, cx.wrapping_sub(1), cy);
        }
        let yoff = ctx.pane().map_or(0, |pane| pane.geometry().y as u_int);
        let mut ranges = visible_ranges::default();
        ranges.set_visible_ranges(
            ctx.pane(),
            cx.wrapping_sub(n) as c_int,
            cy.wrapping_add(yoff) as c_int,
            n,
        );
        let mut vis: u_int = 0;
        for range in &ranges.ranges[..ranges.used as usize] {
            vis = vis.wrapping_add(range.nx);
        }
        if vis < n {
            return 1;
        }
        screen_write_set_cursor(ctx, cx.wrapping_sub(n) as c_int, cy as c_int);
        screen_write_initctx(ctx, &mut ttyctx, 0, 0);
        ttyctx.cell = Some(last);
        if force_wide {
            ttyctx.flags |= TTY_CTX_CELL_INVALIDATE;
        }
        tty_write(
            |tty, ttyctx| tty_cmd_cell(tty, ttyctx, ctx.screen()),
            &mut ttyctx,
        );
        screen_write_set_cursor(ctx, cx as c_int, cy as c_int);
        1
    }
}

/// Erases the padding cells around the cursor that a character of `width`
/// columns is about to write over, and answers whether anything was erased.
fn screen_write_overwrite(ctx: &mut screen_write_ctx, gc: &mut grid_cell, width: u_int) -> c_int {
    let (cx, cy) = ctx.screen().cursor();
    let gd = RustScreen::grid_mut(ctx.screen_mut());
    let mut tmp_gc;
    let mut done = 0;
    if gc.flags as c_int & GRID_FLAG_PADDING != 0 {
        let mut xx = cx;
        while xx > 0 {
            tmp_gc = grid_view_get_cell(gd, xx, cy);
            if tmp_gc.flags as c_int & GRID_FLAG_PADDING == 0 {
                break;
            }
            log_debug(
                c"%s: padding at %u,%u",
                fmt_args![c"screen_write_overwrite", xx, cy],
            );
            grid_view_set_cell(gd, xx, cy, &grid_default_cell);
            xx = xx.wrapping_sub(1);
        }
        log_debug(
            c"%s: character at %u,%u",
            fmt_args![c"screen_write_overwrite", xx, cy],
        );
        grid_view_set_cell(gd, xx, cy, &grid_default_cell);
        done = 1;
    }
    if width != 1 || gc.data.width as c_int != 1 || gc.flags as c_int & GRID_FLAG_PADDING != 0 {
        let mut xx = cx.wrapping_add(width);
        while xx < gd.sx {
            tmp_gc = grid_view_get_cell(gd, xx, cy);
            if tmp_gc.flags as c_int & GRID_FLAG_PADDING == 0 {
                break;
            }
            log_debug(
                c"%s: overwrite at %u,%u",
                fmt_args![c"screen_write_overwrite", xx, cy],
            );
            if gc.flags as c_int & GRID_FLAG_TAB != 0 {
                tmp_gc = *gc;
                tmp_gc.data.data = [0; 32];
                tmp_gc.data.data[0] = b' ';
                tmp_gc.data.have = 1;
                tmp_gc.data.size = tmp_gc.data.have;
                tmp_gc.data.width = tmp_gc.data.size;
                grid_view_set_cell(gd, xx, cy, &tmp_gc);
            } else {
                grid_view_set_cell(gd, xx, cy, &grid_default_cell);
            }
            done = 1;
            xx = xx.wrapping_add(1);
        }
    }
    done
}
/// Tells the terminal what the selection is.
pub(super) unsafe fn screen_write_setselection(
    ctx: &mut screen_write_ctx,
    clip: &CStr,
    str: &[u_char],
) {
    unsafe {
        let mut ttyctx = tty_ctx::default();
        screen_write_initctx(ctx, &mut ttyctx, 0, 0);
        ttyctx.value = TtyCtxValue::Sel(tty_ctx_sel { clip, data: str });
        tty_write(|tty, ctx| tty_cmd_setselection(tty, ctx), &mut ttyctx);
    }
}

/// Hands bytes to the terminal as they are.
pub(super) unsafe fn screen_write_rawstring(
    ctx: &mut screen_write_ctx,
    str: &[u_char],
    allow_invisible_panes: c_int,
) {
    unsafe {
        let mut ttyctx = tty_ctx::default();
        screen_write_initctx(ctx, &mut ttyctx, 0, 0);
        if allow_invisible_panes != 0 {
            ttyctx.flags |= TTY_CTX_INVISIBLE_PANES;
        }
        ttyctx.value = TtyCtxValue::Data(tty_ctx_data { data: str });
        tty_write(|tty, ctx| tty_cmd_rawstring(tty, ctx), &mut ttyctx);
    }
}

/// Switches the pane to its alternate screen, or back, and asks for the
/// redraw the change needs.
unsafe fn screen_write_alternate(
    ctx: &mut screen_write_ctx,
    gc: &mut grid_cell,
    cursor: c_int,
    on: bool,
    from: &CStr,
) {
    unsafe {
        let mut ttyctx = tty_ctx::default();
        if ctx
            .pane()
            .is_some_and(|pane| (pane.options_ref()).number(c"alternate-screen") == 0)
        {
            return;
        }
        screen_write_collect_flush(ctx, 0, from);
        if on {
            screen_alternate_on(ctx.screen_mut(), gc, cursor);
        } else {
            screen_alternate_off(ctx.screen_mut(), Some(gc), cursor);
        }
        if let Some(pane) = ctx.pane() {
            let window = pane
                .window_context()
                .expect("a writing pane has a window context");
            window.fix_layout_panes(None);
            window.redraw_borders();
        }
        screen_write_initctx(ctx, &mut ttyctx, 1, 0);
        if let Some(cb) = ttyctx.redraw_cb {
            cb(&ttyctx);
        }
    }
}
pub(super) unsafe fn screen_write_alternateon(
    ctx: &mut screen_write_ctx,
    gc: &mut grid_cell,
    cursor: c_int,
) {
    unsafe { screen_write_alternate(ctx, gc, cursor, true, c"screen_write_alternateon") }
}
pub(super) unsafe fn screen_write_alternateoff(
    ctx: &mut screen_write_ctx,
    gc: &mut grid_cell,
    cursor: c_int,
) {
    unsafe { screen_write_alternate(ctx, gc, cursor, false, c"screen_write_alternateoff") }
}

#[cfg(test)]
#[path = "../tests/test_screen_write_hooks.rs"]
mod test_hooks;

#[cfg(test)]
#[path = "../tests/test_screen_write.rs"]
mod tests;

#[cfg(test)]
#[path = "../tests/test_coverage_screen_write.rs"]
mod coverage_tests;

#[cfg(test)]
pub use crate::consts::BOX_LINES_NONE;
