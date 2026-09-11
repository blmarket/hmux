use crate::CompiledRegex;
use crate::WindowPane;
use crate::args::RustArguments;
use crate::args::args_parse;
use crate::args::args_parse_t;
use crate::cmd::{cmd_mouse_at, cmd_mouse_pane};
use crate::compat::strtonum;
use crate::compat::{cstr_eq_ignore_case, tolower};
use crate::ffi::abs;
use crate::fmt_args;
use crate::fmt_engine::{FmtArg, format_alloc};
use crate::format::{
    format_add, format_add_cb, format_create_defaults, format_expand, format_grid_hyperlink,
    format_grid_line, format_grid_word, format_single,
};
use crate::grid::{
    Grid, GridReader, RustGridReader, grid_default_cell, };
use crate::input::InputOwner;
use crate::job::job_run;
use crate::log::{fatalx, log_debug};
use crate::notify::notify_pane;
#[cfg(test)]
use crate::window::window_pane_find_by_id;

use crate::pane_geometry::PaneGeometryState;
use crate::pane_search::PaneSearchState;
use crate::paste::{
    PasteBufferStore, paste_buffer_limit, with_paste_buffers, with_paste_buffers_mut,
};
use crate::reactor::Timer;
use crate::screen::Screen;
use crate::screen::{
    RustScreenWriteCtx, ScreenWriteCtx, screen_write_ctx_on_screen, screen_write_strlen,
};
use crate::screen::{screen_resize, screen_resize_cursor};

use crate::status::status_message_set;
use crate::style::style_apply;
use crate::terminfo::{AlternateCharacterSet, RustAlternateCharacterSet};
use crate::text::{utf8_copy, utf8_fromcstr, utf8_set, utf8_to_data};
use crate::tmux::get_timer;
use crate::tmux::{global_options, global_w_options};
use crate::tty::tty_window_offset;
pub use crate::types::*;
use ::core::ffi::CStr;
use ::std::borrow::Cow;
use ::std::ffi::CString;

#[repr(C)]
#[derive(Default)]
pub struct window_copy_mode_data {
    pub(crate) screen: ScreenRef,
    pub(crate) backing: Option<Box<RustScreen>>,
    pub backing_written: core::ffi::c_int,
    pub ictx: Option<InputCtxRef>,
    pub viewmode: core::ffi::c_int,
    pub oy: u_int,
    pub selx: u_int,
    pub sely: u_int,
    pub endselx: u_int,
    pub endsely: u_int,
    pub cursordrag: window_copy_mode_data_cursordrag,
    pub modekeys: core::ffi::c_int,
    pub lineflag: window_copy_mode_data_lineflag,
    pub rectflag: core::ffi::c_int,
    pub scroll_exit: core::ffi::c_int,
    pub hide_position: core::ffi::c_int,
    pub line_numbers: core::ffi::c_int,
    pub selflag: window_copy_mode_data_selflag,
    pub recentre_state: window_copy_mode_data_recentre_state,
    pub recentre_line: u_int,
    pub separators: Option<std::rc::Rc<CStr>>,
    pub dx: u_int,
    pub dy: u_int,
    pub selrx: u_int,
    pub selry: u_int,
    pub endselrx: u_int,
    pub endselry: u_int,
    pub cx: u_int,
    pub cy: u_int,
    pub lastcx: u_int,
    pub lastsx: u_int,
    pub mx: u_int,
    pub my: u_int,
    pub showmark: core::ffi::c_int,
    pub searchtype: core::ffi::c_int,
    pub searchdirection: core::ffi::c_int,
    pub searchregex: core::ffi::c_int,
    pub searchstr: Option<CString>,
    pub searchmark: Vec<u8>,
    pub searchcount: core::ffi::c_int,
    pub searchmore: core::ffi::c_int,
    pub searchall: core::ffi::c_int,
    pub searchx: core::ffi::c_int,
    pub searchy: core::ffi::c_int,
    pub searcho: core::ffi::c_int,
    pub searchgen: u_char,
    pub timeout: core::ffi::c_int,
    pub jumptype: core::ffi::c_int,
    pub jumpchar: Option<utf8_data>,
    pub dragtimer: TimerHandle,
}

pub type window_copy_mode_data_recentre_state = core::ffi::c_uint;
pub const RECENTRE_BOTTOM: window_copy_mode_data_recentre_state = 2;
pub const RECENTRE_MIDDLE: window_copy_mode_data_recentre_state = 1;
pub const RECENTRE_TOP: window_copy_mode_data_recentre_state = 0;
pub type window_copy_mode_data_selflag = core::ffi::c_uint;
pub const SEL_LINE: window_copy_mode_data_selflag = 2;
pub const SEL_WORD: window_copy_mode_data_selflag = 1;
pub const SEL_CHAR: window_copy_mode_data_selflag = 0;
pub type window_copy_mode_data_lineflag = core::ffi::c_uint;
pub const LINE_SEL_RIGHT_LEFT: window_copy_mode_data_lineflag = 2;
pub const LINE_SEL_LEFT_RIGHT: window_copy_mode_data_lineflag = 1;
pub const LINE_SEL_NONE: window_copy_mode_data_lineflag = 0;
pub type window_copy_mode_data_cursordrag = core::ffi::c_uint;
pub const CURSORDRAG_SEL: window_copy_mode_data_cursordrag = 2;
pub const CURSORDRAG_ENDSEL: window_copy_mode_data_cursordrag = 1;
pub const CURSORDRAG_NONE: window_copy_mode_data_cursordrag = 0;
pub const WINDOW_COPY_LINE_NUMBERS_DEFAULT: window_copy_line_numbers = 1;
pub const WINDOW_COPY_LINE_NUMBERS_OFF: window_copy_line_numbers = 0;
pub const WINDOW_COPY_LINE_NUMBERS_HYBRID: window_copy_line_numbers = 4;
pub const WINDOW_COPY_LINE_NUMBERS_ABSOLUTE: window_copy_line_numbers = 2;
pub const WINDOW_COPY_CMD_NOTHING: window_copy_cmd_action = 0;
pub type window_copy_cmd_action = core::ffi::c_uint;
pub const WINDOW_COPY_CMD_CANCEL: window_copy_cmd_action = 2;
pub const WINDOW_COPY_CMD_REDRAW: window_copy_cmd_action = 1;
pub const WINDOW_COPY_CMD_CLEAR_NEVER: window_copy_cmd_clear = 1;
pub type window_copy_cmd_clear = core::ffi::c_uint;
pub const WINDOW_COPY_CMD_CLEAR_EMACS_ONLY: window_copy_cmd_clear = 2;
pub const WINDOW_COPY_CMD_CLEAR_ALWAYS: window_copy_cmd_clear = 0;
#[repr(C)]
pub struct window_copy_cmd_state<'a> {
    /// The mode entry the command is running against, borrowed for as long
    /// as the command runs.
    pub wme: &'a mut window_mode_entry,
    pub args: &'a RustArguments,
    pub wargs: &'a RustArguments,
    pub m: Option<&'a mouse_event>,
    pub c: Option<&'a mut client>,
    pub s: Option<&'a session>,
    pub wl: Option<&'a winlink>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_copy_cmd_entry {
    pub command: &'static CStr,
    pub minargs: u_int,
    pub maxargs: u_int,
    pub args: args_parse_t,
    pub flags: core::ffi::c_int,
    pub clear: window_copy_cmd_clear,
    pub f: Option<unsafe fn(&mut window_copy_cmd_state<'_>) -> window_copy_cmd_action>,
}

pub const WINDOW_COPY_REL_POS_ON_SCREEN: window_copy_rel_pos = 1;
pub const WINDOW_COPY_REL_POS_BELOW: window_copy_rel_pos = 2;
pub const WINDOW_COPY_REL_POS_ABOVE: window_copy_rel_pos = 0;
pub struct window_copy_search_cell<'a> {
    pub d: Cow<'a, [u8]>,
}
pub const WINDOW_COPY_SEARCHDOWN: window_copy_search_type = 2;
pub const WINDOW_COPY_SEARCHUP: window_copy_search_type = 1;
pub const BOTTOM: window_copy_line_position = 2;
pub const TOP: window_copy_line_position = 1;
pub const MIDDLE: window_copy_line_position = 0;
pub type window_copy_line_position = core::ffi::c_uint;
pub const WINDOW_COPY_JUMPTOFORWARD: window_copy_search_type = 5;
pub const WINDOW_COPY_JUMPTOBACKWARD: window_copy_search_type = 6;
pub const WINDOW_COPY_JUMPBACKWARD: window_copy_search_type = 4;
pub const WINDOW_COPY_JUMPFORWARD: window_copy_search_type = 3;
pub const WINDOW_COPY_OFF: window_copy_search_type = 0;
pub type window_copy_search_type = core::ffi::c_uint;
pub type window_copy_rel_pos = core::ffi::c_uint;
pub type window_copy_line_numbers = core::ffi::c_uint;
pub use crate::consts::{
    CLIENT_READONLY, GRID_ATTR_CHARSET, GRID_FLAG_EXTENDED, GRID_FLAG_NOPALETTE, GRID_FLAG_PADDING,
    GRID_FLAG_TAB, GRID_HISTORY, GRID_LINE_START_OUTPUT, GRID_LINE_START_PROMPT, GRID_LINE_WRAPPED,
    INT_MAX, JOB_NOWAIT, MODEKEY_EMACS, MODEKEY_VI, MOUSE_MASK_BUTTONS, MOUSE_WHEEL_DOWN,
    MOUSE_WHEEL_UP, PANE_REDRAW, PANE_REDRAWSCROLLBAR, REG_EXTENDED, REG_ICASE, UCHAR_MAX,
    UINT_MAX,
};

pub const REG_NOTBOL: core::ffi::c_int = 1 as core::ffi::c_int;

pub const WHITESPACE: &CStr = c"\t ";

pub const WINDOW_COPY_SEARCH_TIMEOUT: core::ffi::c_int = 10000 as core::ffi::c_int;
pub const WINDOW_COPY_SEARCH_ALL_TIMEOUT: core::ffi::c_int = 200 as core::ffi::c_int;
pub const WINDOW_COPY_SEARCH_MAX_LINE: core::ffi::c_int = 2000 as core::ffi::c_int;
pub const WINDOW_COPY_DRAG_REPEAT_TIME: core::ffi::c_int = 50000 as core::ffi::c_int;
unsafe fn window_copy_scroll_timer(mut pane: RustWindowPaneWeak, mode: WindowMode) {
    unsafe {
        let Some(wme) = pane
            .get_mut()
            .and_then(|pane| (pane).active_mode_mut().map(|mode| mode.into_entry()))
        else {
            return;
        };
        if wme.mode() != mode {
            return;
        }
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        let tv = timeval::from_usecs(WINDOW_COPY_DRAG_REPEAT_TIME as __suseconds_t);
        data.dragtimer.disarm();
        if data.cy == 0 as u_int {
            data.dragtimer.arm(tv);
            window_copy_cursor_up(wme, 1 as core::ffi::c_int);
        } else if data.cy
            == RustScreen::grid(&data.screen.borrow())
                .height()
                .wrapping_sub(1 as u_int)
        {
            data.dragtimer.arm(tv);
            window_copy_cursor_down(wme, 1 as core::ffi::c_int);
        }
    }
}
fn window_copy_free_backing(data: &mut window_copy_mode_data) {
    data.backing.take();
}
fn window_copy_clone_screen(
    src: &RustScreen,
    hint: &RustScreen,
    want_cursor: bool,
    trim: core::ffi::c_int,
) -> (Box<RustScreen>, u_int, u_int) {
    {
        let mut dst: Box<RustScreen>;
        let mut gl: Option<crate::grid::GridLineInfo>;
        let mut sy: u_int;
        let mut wx: u_int = 0;
        let mut wy: u_int = 0;
        let reflow: core::ffi::c_int;
        sy = RustScreen::grid(src)
            .history_size()
            .wrapping_add(RustScreen::grid(src).height());
        if trim != 0 {
            while sy > RustScreen::grid(src).history_size() {
                gl = (RustScreen::grid(src)).peek_line(sy.wrapping_sub(1 as u_int));
                if !gl.is_some_and(|gl| gl.cellused == 0 as u_int) {
                    break;
                }
                sy = sy.wrapping_sub(1);
            }
        }
        log_debug(
            c"%s: target screen is %ux%u, source %ux%u",
            fmt_args![
                c"window_copy_clone_screen",
                RustScreen::grid(src).width(),
                sy,
                RustScreen::grid(hint).width(),
                RustScreen::grid(src)
                    .history_size()
                    .wrapping_add(RustScreen::grid(src).height())
            ],
        );
        dst = Box::new(RustScreen::new_with_server_options(
            RustScreen::grid(src).width(),
            sy,
            src.history_limit(),
        ));
        dst.copy_history_from(src, sy);
        let mut cx = 0 as u_int;
        let mut cy = 0 as u_int;
        if want_cursor {
            let (dst_cx, dst_cy) = dst.cursor();
            cx = dst_cx;
            cy = RustScreen::grid(&dst).history_size().wrapping_add(dst_cy);
            reflow = (RustScreen::grid(hint).width() != RustScreen::grid(&dst).width()) as core::ffi::c_int;
        } else {
            reflow = 0 as core::ffi::c_int;
        }
        if reflow != 0 {
            (wx, wy) = RustScreen::grid(&dst).wrap_position(cx, cy);
        }
        screen_resize_cursor(
            &mut dst,
            RustScreen::grid(hint).width(),
            RustScreen::grid(hint).height(),
            1 as core::ffi::c_int,
            0 as core::ffi::c_int,
            0 as core::ffi::c_int,
        );
        if reflow != 0 {
            (cx, cy) = RustScreen::grid(&dst).unwrap_position(wx, wy);
        }
        (dst, cx, cy)
    }
}
unsafe fn window_copy_common_init(
    pane: &crate::window::RustWindowPaneWeak,
    mode: WindowMode,
) -> Box<window_copy_mode_data> {
    unsafe {
        let wp = pane.get().expect("the initializing pane still exists");
        let base = wp.base();
        let mut data = Box::new(window_copy_mode_data::default());
        data.cursordrag = CURSORDRAG_NONE;
        data.lineflag = LINE_SEL_NONE;
        data.selflag = SEL_CHAR;
        if let Some(query) = wp.query() {
            data.searchtype = WINDOW_COPY_SEARCHUP as core::ffi::c_int;
            data.searchregex = core::ffi::c_int::from(wp.is_regex());
            data.searchstr = Some(query.to_owned());
        } else {
            data.searchtype = WINDOW_COPY_OFF as core::ffi::c_int;
            data.searchregex = 0 as core::ffi::c_int;
            data.searchstr = None;
        }
        data.searcho = -(1 as core::ffi::c_int);
        data.searchy = data.searcho;
        data.searchx = data.searchy;
        data.searchall = 1 as core::ffi::c_int;
        data.jumptype = WINDOW_COPY_OFF as core::ffi::c_int;
        data.jumpchar = None;
        data.line_numbers = 1 as core::ffi::c_int;
        let (sx, sy) = base.size();
        data.screen = ScreenRef::new(RustScreen::new_sharing_hyperlinks(sx, sy, 0 as u_int, base));
        data.screen.borrow_mut().set_default_cursor(
            global_w_options
                .get()
                .as_ref()
                .expect("global options are initialized"),
        );
        data.modekeys = pane
            .window()
            .expect("the initializing pane has a window")
            .options()
            .number(c"mode-keys") as core::ffi::c_int;
        let pane = pane.clone();
        data.dragtimer
            .set_callback(move || window_copy_scroll_timer(pane.clone(), mode));
        data
    }
}
pub(crate) unsafe fn window_copy_init(
    wme: &mut window_mode_entry,
    pane: crate::window::RustWindowPaneWeak,
    _fs: Option<&cmd_find_state>,
    args: Option<&RustArguments>,
) {
    unsafe {
        let source = wme.source_pane_ref().expect("copy mode has a source pane");
        let mut i: u_int;

        let mut data = window_copy_common_init(&pane, WindowMode::Copy);
        let cloned = window_copy_clone_screen(
            source.get().expect("copy mode source still exists").base(),
            &data.screen.borrow(),
            true,
            (source.id() != pane.id()) as core::ffi::c_int,
        );
        data.backing = Some(cloned.0);
        let cx: u_int = cloned.1;
        let cy: u_int = cloned.2;
        data.cx = cx;
        if cy
            < RustScreen::grid(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
            )
            .history_size()
        {
            data.cy = 0 as u_int;
            data.oy = RustScreen::grid(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
            )
            .history_size()
            .wrapping_sub(cy);
        } else {
            data.cy = cy.wrapping_sub(
                RustScreen::grid(
                    data.backing
                        .as_deref()
                        .expect("copy mode has a backing screen"),
                )
                .history_size(),
            );
            data.oy = 0 as u_int;
        }
        data.scroll_exit = args.map_or(0, |args| args.argument_flag_count(b'e'));
        data.hide_position = args.map_or(0, |args| args.argument_flag_count(b'H'));
        wme.state = WindowModeState::Copy(data);
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let cx_offset =
            window_copy_cursor_offset(wme, data.cx, RustScreen::grid(&data.screen.borrow()).width());
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        data.screen.borrow_mut().set_cursor(cx_offset, data.cy);
        data.mx = data.cx;
        data.my = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_add(data.cy)
        .wrapping_sub(data.oy);
        data.showmark = 0 as core::ffi::c_int;
        let sy = RustScreen::grid(&data.screen.borrow()).height();
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let x = window_copy_cursor_offset(wme, data.cx, RustScreen::grid(&data.screen.borrow()).width());
        let display = data.screen.clone();
        let mut writer = RustScreenWriteCtx::on_shared_screen(&display);
        i = 0 as u_int;
        while i < sy {
            window_copy_write_line(wme, &mut writer, i);
            i = i.wrapping_add(1);
        }
        let cy = wme
            .state
            .copy_mode_data_ref()
            .expect("copy mode has state")
            .cy;
        writer.cursormove(
            x as core::ffi::c_int,
            cy as core::ffi::c_int,
            0 as core::ffi::c_int,
        );
        writer.finish();
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        data.recentre_state = RECENTRE_MIDDLE;
        data.recentre_line = 0 as u_int;
    }
}
pub(crate) unsafe fn window_copy_view_init(
    wme: &mut window_mode_entry,
    pane: crate::window::RustWindowPaneWeak,
    _fs: Option<&cmd_find_state>,
    _args: Option<&RustArguments>,
) {
    unsafe {
        let (sx, sy) = pane
            .get()
            .expect("the initializing pane still exists")
            .base()
            .size();
        let mut data = window_copy_common_init(&pane, WindowMode::View);
        data.viewmode = 1 as core::ffi::c_int;
        data.line_numbers = 0 as core::ffi::c_int;
        data.backing = Some(Box::new(RustScreen::new_with_server_options(
            sx, sy, UINT_MAX,
        )));
        data.ictx = Some(InputCtxRef::create(InputOwner::Detached, Stream::NONE));
        data.mx = data.cx;
        data.my = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_add(data.cy)
        .wrapping_sub(data.oy);
        data.showmark = 0 as core::ffi::c_int;
        wme.state = WindowModeState::View(data);
    }
}
pub(crate) unsafe fn window_copy_free(wme: &mut window_mode_entry) {
    unsafe {
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        data.dragtimer.disarm();
        drop(core::mem::take(&mut data.searchmark));
        data.searchstr = None;
        data.separators = None;
        if let Some(ictx) = data.ictx.take() {
            ictx.close();
        }
        window_copy_free_backing(&mut *data);
        data.screen = ScreenRef::default();
    }
}
pub unsafe fn window_copy_add(
    wp: &mut (impl crate::WindowPane + ?Sized),
    parse: core::ffi::c_int,
    fmt: &CStr,
    args: &[FmtArg],
) {
    unsafe {
        window_copy_vadd(wp, parse, fmt, args);
    }
}
fn window_copy_init_ctx_cb(ttyctx: &mut tty_ctx) {
    ttyctx.defaults = grid_default_cell;
    ttyctx.palette = None;
    ttyctx.redraw_cb = None;
    ttyctx.set_client_cb = None;
    ttyctx.arg = TtyCtxArg::None;
}
pub unsafe fn window_copy_vadd(
    wp: &mut (impl crate::WindowPane + ?Sized),
    parse: core::ffi::c_int,
    fmt: &CStr,
    args: &[FmtArg],
) {
    unsafe {
        let wme = (&mut *wp).active_mode_mut().map(|mode| mode.into_entry()).expect("pane is in a mode");
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        let backing = &mut *data
            .backing
            .as_deref_mut()
            .expect("copy mode has a backing screen");
        let gc;

        let old_cy: u_int;
        let old_hsize: u_int = RustScreen::grid(backing).history_size();
        let append = data.backing_written != 0;
        if !append {
            data.backing_written = 1 as core::ffi::c_int;
        }
        if parse != 0 {
            if append {
                let mut backing_ctx = screen_write_ctx_on_screen(backing);
                backing_ctx.carriagereturn();
                backing_ctx.linefeed(0 as core::ffi::c_int, 8 as u_int);
                old_cy = backing_ctx.cursor_position().1;
                backing_ctx.finish();
            } else {
                old_cy = backing.cursor().1;
            }
            let text = format_alloc(fmt, args);
            data.ictx
                .as_ref()
                .expect("an input owner has a parser")
                .parse_screen(
                    &mut *backing,
                    Some(std::rc::Rc::new(window_copy_init_ctx_cb)),
                    ByteBuffer::from(bytes::Bytes::from(text.into_bytes())),
                );
        } else {
            let mut backing_ctx = screen_write_ctx_on_screen(backing);
            if append {
                backing_ctx.carriagereturn();
                backing_ctx.linefeed(0 as core::ffi::c_int, 8 as u_int);
            }
            old_cy = backing_ctx.cursor_position().1;
            gc = grid_default_cell;
            backing_ctx.vnputs(0 as ssize_t, &gc, fmt.as_ref(), args);
            backing_ctx.finish();
        }
        data.oy = data
            .oy
            .wrapping_add(RustScreen::grid(backing).history_size().wrapping_sub(old_hsize));
        if RustScreen::grid(backing).history_size() != 0 {
            window_copy_redraw_lines(wme, 0 as u_int, 1 as u_int);
        }
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let ny = data
            .backing
            .as_deref()
            .expect("copy mode has a backing screen")
            .cursor()
            .1
            .wrapping_sub(old_cy)
            .wrapping_add(1);
        window_copy_redraw_lines(wme, old_cy, ny);
    }
}
pub unsafe fn window_copy_scroll(
    wp: &mut (impl crate::WindowPane + ?Sized),
    sl_mpos: core::ffi::c_int,
    my: u_int,
    tty_oy: u_int,
    scroll_exit: core::ffi::c_int,
) {
    unsafe {
        if (wp).active_mode().is_none() {
            return;
        }
        let selected_id = wp.pane_id();
        let window = wp
            .window_context()
            .expect("a scrolling pane has a window context");
        window.set_active_pane(
            &crate::window::window_pane_find_by_id(selected_id).expect("the selected pane exists"),
            0,
        );
        let slider_height = wp.slider().sb_slider_h;
        let geometry = wp.geometry();
        let Some((offset, size)) = window_copy_get_current_offset(wp) else {
            return;
        };
        let wme = (wp).active_mode_mut().map(|mode| mode.into_entry()).expect("pane is in a copy or view mode");
        if window_copy_scroll1(
            wme,
            slider_height,
            geometry.sy,
            geometry.yoff as u_int,
            offset,
            size,
            sl_mpos,
            my,
            tty_oy,
            scroll_exit,
        ) {
            (wp).reset_mode();
        }
    }
}
unsafe fn window_copy_scroll1(
    wme: &mut window_mode_entry,
    slider_height: u_int,
    sb_height: u_int,
    sb_top: u_int,
    offset: u_int,
    size: u_int,
    sl_mpos: core::ffi::c_int,
    my: u_int,
    tty_oy: u_int,
    scroll_exit: core::ffi::c_int,
) -> bool {
    unsafe {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");

        let px: u_int;
        let py: u_int;
        let n: u_int;

        let sy: u_int = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .height();

        let my_w: u_int = my.wrapping_add(tty_oy);
        let new_slider_y: core::ffi::c_int = if my_w <= sb_top.wrapping_add(sl_mpos as u_int) {
            0
        } else if my_w.wrapping_sub(sl_mpos as u_int)
            > sb_top.wrapping_add(sb_height).wrapping_sub(slider_height)
        {
            sb_height.wrapping_sub(slider_height) as core::ffi::c_int
        } else {
            my_w.wrapping_sub(sb_top).wrapping_sub(sl_mpos as u_int) as core::ffi::c_int
        };
        let new_offset: u_int = (new_slider_y as core::ffi::c_float
            * (size.wrapping_add(sb_height) as core::ffi::c_float
                / sb_height as core::ffi::c_float)) as u_int;
        let delta: core::ffi::c_int =
            (offset as core::ffi::c_int as u_int).wrapping_sub(new_offset) as core::ffi::c_int;
        let oy: u_int = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_add(data.cy)
        .wrapping_sub(data.oy);
        let ox: u_int = window_copy_find_length(wme, oy);
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        if data.cx != ox {
            data.lastcx = data.cx;
            data.lastsx = ox;
        }
        data.cx = data.lastcx;
        if delta >= 0 as core::ffi::c_int {
            n = delta as u_int;
            if data.oy.wrapping_add(n)
                > RustScreen::grid(
                    data.backing
                        .as_deref()
                        .expect("copy mode has a backing screen"),
                )
                .history_size()
            {
                data.oy = RustScreen::grid(
                    data.backing
                        .as_deref()
                        .expect("copy mode has a backing screen"),
                )
                .history_size();
                if data.cy < n {
                    data.cy = 0 as u_int;
                } else {
                    data.cy = data.cy.wrapping_sub(n);
                }
            } else {
                data.oy = data.oy.wrapping_add(n);
            }
        } else {
            n = -delta as u_int;
            if data.oy < n {
                data.oy = 0 as u_int;
                if data.cy.wrapping_add(n.wrapping_sub(data.oy)) >= sy {
                    data.cy = sy.wrapping_sub(1 as u_int);
                } else {
                    data.cy = data.cy.wrapping_add(n.wrapping_sub(data.oy));
                }
            } else {
                data.oy = data.oy.wrapping_sub(n);
            }
        }
        data.cursordrag = CURSORDRAG_NONE;
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        if !data.screen.borrow().has_selection() || data.rectflag == 0 {
            py = RustScreen::grid(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
            )
            .history_size()
            .wrapping_add(data.cy)
            .wrapping_sub(data.oy);
            px = window_copy_find_length(wme, py);
            if data.cx >= data.lastsx && data.cx != px || data.cx > px {
                window_copy_cursor_end_of_line(wme);
            }
        }
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        if scroll_exit != 0 && data.oy == 0 as u_int {
            return true;
        }
        if !data.searchmark.is_empty() && data.timeout == 0 {
            let regex = data.searchregex;
            window_copy_search_marks(wme, None, regex, 1 as core::ffi::c_int);
        }
        window_copy_update_selection(wme, 1 as core::ffi::c_int, 0 as core::ffi::c_int);
        window_copy_redraw_screen(wme);
        false
    }
}
pub unsafe fn window_copy_pageup(wp: &mut (impl crate::WindowPane + ?Sized), half_page: core::ffi::c_int) {
    unsafe {
        window_copy_pageup1(
            (&mut *wp).active_mode_mut().map(|mode| mode.into_entry()).expect("pane is in a mode"),
            half_page,
        );
    }
}
unsafe fn window_copy_pageup1(wme: &mut window_mode_entry, half_page: core::ffi::c_int) {
    unsafe {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let mut n: u_int;

        let px: u_int;
        let py: u_int;
        let oy: u_int = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_add(data.cy)
        .wrapping_sub(data.oy);
        let ox: u_int = window_copy_find_length(wme, oy);
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        let s = &data.screen;
        if data.cx != ox {
            data.lastcx = data.cx;
            data.lastsx = ox;
        }
        data.cx = data.lastcx;
        n = 1 as u_int;
        if RustScreen::grid(&s.borrow()).height() > 2 as u_int {
            if half_page != 0 {
                n = RustScreen::grid(&s.borrow()).height().wrapping_div(2 as u_int);
            } else {
                n = RustScreen::grid(&s.borrow()).height().wrapping_sub(2 as u_int);
            }
        }
        if data.oy.wrapping_add(n)
            > RustScreen::grid(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
            )
            .history_size()
        {
            data.oy = RustScreen::grid(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
            )
            .history_size();
            if data.cy < n {
                data.cy = 0 as u_int;
            } else {
                data.cy = data.cy.wrapping_sub(n);
            }
        } else {
            data.oy = data.oy.wrapping_add(n);
        }
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        if !data.screen.borrow().has_selection() || data.rectflag == 0 {
            py = RustScreen::grid(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
            )
            .history_size()
            .wrapping_add(data.cy)
            .wrapping_sub(data.oy);
            px = window_copy_find_length(wme, py);
            if data.cx >= data.lastsx && data.cx != px || data.cx > px {
                window_copy_cursor_end_of_line(wme);
            }
        }
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        if !data.searchmark.is_empty() && data.timeout == 0 {
            let regex = data.searchregex;
            window_copy_search_marks(wme, None, regex, 1 as core::ffi::c_int);
        }
        window_copy_update_selection(wme, 1 as core::ffi::c_int, 0 as core::ffi::c_int);
        window_copy_redraw_screen(wme);
    }
}
pub unsafe fn window_copy_pagedown(
    wp: &mut (impl crate::WindowPane + ?Sized),
    half_page: core::ffi::c_int,
    scroll_exit: core::ffi::c_int,
) {
    unsafe {
        if window_copy_pagedown1(
            (wp).active_mode_mut().map(|mode| mode.into_entry()).expect("pane is in a mode"),
            half_page,
            scroll_exit,
        ) != 0
        {
            (wp).reset_mode();
        }
    }
}
unsafe fn window_copy_pagedown1(
    wme: &mut window_mode_entry,
    half_page: core::ffi::c_int,
    scroll_exit: core::ffi::c_int,
) -> core::ffi::c_int {
    unsafe {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let mut n: u_int;

        let px: u_int;
        let py: u_int;
        let oy: u_int = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_add(data.cy)
        .wrapping_sub(data.oy);
        let ox: u_int = window_copy_find_length(wme, oy);
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        let s = &data.screen;
        if data.cx != ox {
            data.lastcx = data.cx;
            data.lastsx = ox;
        }
        data.cx = data.lastcx;
        n = 1 as u_int;
        if RustScreen::grid(&s.borrow()).height() > 2 as u_int {
            if half_page != 0 {
                n = RustScreen::grid(&s.borrow()).height().wrapping_div(2 as u_int);
            } else {
                n = RustScreen::grid(&s.borrow()).height().wrapping_sub(2 as u_int);
            }
        }
        if data.oy < n {
            data.oy = 0 as u_int;
            if data.cy.wrapping_add(n.wrapping_sub(data.oy))
                >= RustScreen::grid(
                    data.backing
                        .as_deref()
                        .expect("copy mode has a backing screen"),
                )
                .height()
            {
                data.cy = RustScreen::grid(
                    data.backing
                        .as_deref()
                        .expect("copy mode has a backing screen"),
                )
                .height()
                .wrapping_sub(1 as u_int);
            } else {
                data.cy = data.cy.wrapping_add(n.wrapping_sub(data.oy));
            }
        } else {
            data.oy = data.oy.wrapping_sub(n);
        }
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        if !data.screen.borrow().has_selection() || data.rectflag == 0 {
            py = RustScreen::grid(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
            )
            .history_size()
            .wrapping_add(data.cy)
            .wrapping_sub(data.oy);
            px = window_copy_find_length(wme, py);
            if data.cx >= data.lastsx && data.cx != px || data.cx > px {
                window_copy_cursor_end_of_line(wme);
            }
        }
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        if scroll_exit != 0 && data.oy == 0 as u_int {
            return 1 as core::ffi::c_int;
        }
        if !data.searchmark.is_empty() && data.timeout == 0 {
            let regex = data.searchregex;
            window_copy_search_marks(wme, None, regex, 1 as core::ffi::c_int);
        }
        window_copy_update_selection(wme, 1 as core::ffi::c_int, 0 as core::ffi::c_int);
        window_copy_redraw_screen(wme);
        0 as core::ffi::c_int
    }
}
unsafe fn window_copy_previous_paragraph(wme: &mut window_mode_entry) {
    unsafe {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let mut oy: u_int;
        oy = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_add(data.cy)
        .wrapping_sub(data.oy);
        while oy > 0 as u_int && window_copy_find_length(wme, oy) == 0 as u_int {
            oy = oy.wrapping_sub(1);
        }
        while oy > 0 as u_int && window_copy_find_length(wme, oy) > 0 as u_int {
            oy = oy.wrapping_sub(1);
        }
        window_copy_scroll_to(wme, 0 as u_int, oy, 0 as core::ffi::c_int);
    }
}
unsafe fn window_copy_next_paragraph(wme: &mut window_mode_entry) {
    unsafe {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let s = &data.screen;

        let mut oy: u_int;
        oy = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_add(data.cy)
        .wrapping_sub(data.oy);
        let maxy: u_int = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_add(RustScreen::grid(&s.borrow()).height())
        .wrapping_sub(1 as u_int);
        while oy < maxy && window_copy_find_length(wme, oy) == 0 as u_int {
            oy = oy.wrapping_add(1);
        }
        while oy < maxy && window_copy_find_length(wme, oy) > 0 as u_int {
            oy = oy.wrapping_add(1);
        }
        let ox: u_int = window_copy_find_length(wme, oy);
        window_copy_scroll_to(wme, ox, oy, 0 as core::ffi::c_int);
    }
}
pub unsafe fn window_copy_get_word(
    wp: &(impl crate::WindowPane + ?Sized),
    x: u_int,
    y: u_int,
) -> Option<CString> {
    unsafe {
        let wme = (wp).active_mode().expect("pane is in a mode");
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let gd = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        );
        format_grid_word(gd, x, gd.history_size().wrapping_add(y).wrapping_sub(data.oy))
    }
}
pub fn window_copy_get_line(wp: &(impl crate::WindowPane + ?Sized), y: u_int) -> CString {
    {
        let wme = (wp).active_mode().expect("pane is in a mode");
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let gd = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        );
        format_grid_line(gd, gd.history_size().wrapping_add(y).wrapping_sub(data.oy))
    }
}
pub fn window_copy_get_hyperlink(
    wp: &(impl crate::WindowPane + ?Sized),
    x: u_int,
    y: u_int,
) -> Option<CString> {
    {
        let wme = (wp).active_mode().expect("pane is in a mode");
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let display = data.screen.borrow();
        let gd = RustScreen::grid(&display);
        format_grid_hyperlink(gd, x, gd.history_size().wrapping_add(y), &wp.screen_ref())
    }
}
unsafe fn window_copy_cursor_hyperlink_cb(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        let wme = (wp).active_mode().expect("pane is in a mode");
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let display = data.screen.borrow();
        let gd = RustScreen::grid(&display);
        format_grid_hyperlink(gd, data.cx, gd.history_size().wrapping_add(data.cy), &display)
    }
}
unsafe fn window_copy_cursor_word_cb(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        let wme = (wp).active_mode().expect("pane is in a mode");
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        window_copy_get_word(wp, data.cx, data.cy)
    }
}
unsafe fn window_copy_cursor_line_cb(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        let wme = (wp).active_mode().expect("pane is in a mode");
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        Some(window_copy_get_line(wp, data.cy))
    }
}
unsafe fn window_copy_search_match_cb(ft: &format_tree) -> Option<CString> {
    unsafe {
        let pane = ft.pane_handle()?;
        let wp = pane.get()?;
        let wme = (wp).active_mode().expect("pane is in a mode");
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        window_copy_match_at_cursor(data)
    }
}
pub(crate) fn window_copy_formats(wme: &window_mode_entry, ft: &mut format_tree) {
    {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let hsize: u_int = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size();
        let position: u_int;
        let limit: u_int;
        let gl = (RustScreen::grid(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
            )).line_info(hsize.wrapping_sub(data.oy),);
        format_add(
            ft,
            c"top_line_time",
            c"%llu",
            fmt_args![gl.time as core::ffi::c_ulonglong],
        );
        format_add(ft, c"scroll_position", c"%d", fmt_args![data.oy]);
        if window_copy_line_number_is_absolute(wme) != 0 {
            position = hsize.wrapping_sub(data.oy).wrapping_add(1 as u_int);
            limit = hsize.wrapping_add(
                RustScreen::grid(
                    data.backing
                        .as_deref()
                        .expect("copy mode has a backing screen"),
                )
                .height(),
            );
        } else {
            position = data.oy;
            limit = hsize;
        }
        format_add(ft, c"copy_position", c"%u", fmt_args![position]);
        format_add(ft, c"copy_position_limit", c"%u", fmt_args![limit]);
        format_add(ft, c"rectangle_toggle", c"%d", fmt_args![data.rectflag]);
        format_add(ft, c"copy_cursor_x", c"%d", fmt_args![data.cx]);
        format_add(ft, c"copy_cursor_y", c"%d", fmt_args![data.cy]);
        if data.screen.borrow().has_selection() {
            format_add(ft, c"selection_start_x", c"%d", fmt_args![data.selx]);
            format_add(ft, c"selection_start_y", c"%d", fmt_args![data.sely]);
            format_add(ft, c"selection_end_x", c"%d", fmt_args![data.endselx]);
            format_add(ft, c"selection_end_y", c"%d", fmt_args![data.endsely]);
            if data.cursordrag as core::ffi::c_uint
                != CURSORDRAG_NONE as core::ffi::c_int as core::ffi::c_uint
            {
                format_add(ft, c"selection_active", c"1", fmt_args![]);
            } else {
                format_add(ft, c"selection_active", c"0", fmt_args![]);
            }
            if data.endselx != data.selx || data.endsely != data.sely {
                format_add(ft, c"selection_present", c"1", fmt_args![]);
            } else {
                format_add(ft, c"selection_present", c"0", fmt_args![]);
            }
        } else {
            format_add(ft, c"selection_active", c"0", fmt_args![]);
            format_add(ft, c"selection_present", c"0", fmt_args![]);
        }
        match data.selflag {
            SEL_CHAR => {
                format_add(ft, c"selection_mode", c"char", fmt_args![]);
            }
            SEL_WORD => {
                format_add(ft, c"selection_mode", c"word", fmt_args![]);
            }
            SEL_LINE => {
                format_add(ft, c"selection_mode", c"line", fmt_args![]);
            }
            _ => {}
        }
        format_add(
            ft,
            c"search_present",
            c"%d",
            fmt_args![(!data.searchmark.is_empty()) as core::ffi::c_int],
        );
        format_add(ft, c"search_timed_out", c"%d", fmt_args![data.timeout]);
        if data.searchcount != -(1 as core::ffi::c_int) {
            format_add(ft, c"search_count", c"%d", fmt_args![data.searchcount]);
            format_add(
                ft,
                c"search_count_partial",
                c"%d",
                fmt_args![data.searchmore],
            );
        }
        format_add_cb(ft, c"search_match", Some(window_copy_search_match_cb));
        format_add_cb(ft, c"copy_cursor_word", Some(window_copy_cursor_word_cb));
        format_add_cb(ft, c"copy_cursor_line", Some(window_copy_cursor_line_cb));
        format_add_cb(
            ft,
            c"copy_cursor_hyperlink",
            Some(window_copy_cursor_hyperlink_cb),
        );
    }
}
pub(crate) fn window_copy_get_screen(wme: &window_mode_entry) -> Option<&RustScreen> {
    wme.state.copy_mode_data_ref()?.backing.as_deref()
}
unsafe fn window_copy_size_changed(wme: &mut window_mode_entry) {
    unsafe {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let search: core::ffi::c_int = (!data.searchmark.is_empty()) as core::ffi::c_int;
        window_copy_clear_selection(wme);
        window_copy_clear_marks(wme);
        let display = wme
            .state
            .copy_mode_data_ref()
            .expect("copy mode has state")
            .screen
            .clone();
        let sy = display.borrow().size().1;
        let mut writer = RustScreenWriteCtx::on_shared_screen(&display);
        window_copy_write_lines(wme, &mut writer, 0 as u_int, sy);
        writer.finish();
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        if search != 0 && data.timeout == 0 {
            let regex = data.searchregex;
            window_copy_search_marks(wme, None, regex, 0 as core::ffi::c_int);
        }
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        data.searchx = data.cx as core::ffi::c_int;
        data.searchy = data.cy as core::ffi::c_int;
        data.searcho = data.oy as core::ffi::c_int;
    }
}
pub(crate) unsafe fn window_copy_resize(wme: &mut window_mode_entry, sx: u_int, sy: u_int) {
    unsafe {
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        let mut s = data.screen.borrow_mut();
        let gd = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        );
        let mut cx: u_int;
        let mut cy: u_int;
        let mut wx: u_int = 0;
        let mut wy: u_int = 0;

        screen_resize(&mut s, sx, sy, 0 as core::ffi::c_int);
        drop(s);
        cx = data.cx;
        if data.oy > gd.history_size().wrapping_add(data.cy) {
            data.oy = gd.history_size().wrapping_add(data.cy);
        }
        cy = gd.history_size().wrapping_add(data.cy).wrapping_sub(data.oy);
        let reflow: core::ffi::c_int = (gd.width() != sx) as core::ffi::c_int;
        if reflow != 0 {
            (wx, wy) = (*gd).wrap_position(cx, cy);
        }
        screen_resize_cursor(
            &mut *data
                .backing
                .as_deref_mut()
                .expect("copy mode has a backing screen"),
            sx,
            sy,
            1 as core::ffi::c_int,
            0 as core::ffi::c_int,
            0 as core::ffi::c_int,
        );
        let gd = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        );
        if reflow != 0 {
            (cx, cy) = gd.unwrap_position(wx, wy);
        }
        data.cx = cx;
        if cy < gd.history_size() {
            data.cy = 0 as u_int;
            data.oy = gd.history_size().wrapping_sub(cy);
        } else {
            data.cy = cy.wrapping_sub(gd.history_size());
            data.oy = 0 as u_int;
        }
        window_copy_size_changed(wme);
        window_copy_redraw_screen(wme);
    }
}
pub(crate) fn window_copy_key_table(wme: &window_mode_entry) -> &'static CStr {
    {
        let pane = wme.pane_ref().expect("mode has a pane");
        let window = pane.window().expect("mode pane has a window");
        if (window.options()).number(c"mode-keys") == MODEKEY_VI as core::ffi::c_longlong {
            return c"copy-mode-vi";
        }
        c"copy-mode"
    }
}
unsafe fn window_copy_expand_search_string(cs: &mut window_copy_cmd_state<'_>) -> core::ffi::c_int {
    unsafe {
        let wme = &mut *cs.wme;
        let ss = match {
            let args: &RustArguments = cs.wargs;
            let idx = 0 as u_int;
            args.argument_string(idx)
        } {
            Some(ss) if !ss.to_bytes().is_empty() => ss,
            _ => return 0 as core::ffi::c_int,
        };
        let search = if ({
            let args: &RustArguments = cs.args;
            args.argument_flag_count(b'F')
        }) != 0
        {
            let pane = wme.pane_ref().expect("mode has a pane");
            let expanded = format_single(None, ss, None, None, None, pane.get());
            if expanded.as_bytes().is_empty() {
                return 0 as core::ffi::c_int;
            }
            expanded
        } else {
            ss.to_owned()
        };
        wme.state
            .copy_mode_data_mut()
            .expect("copy mode has state")
            .searchstr = Some(search);
        1 as core::ffi::c_int
    }
}
unsafe fn window_copy_cmd_append_selection(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let s = cs.s;
        if s.is_some() {
            window_copy_append_selection(&mut *wme);
        }
        window_copy_clear_selection(&mut *wme);
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_append_selection_and_cancel(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let s = cs.s;
        if s.is_some() {
            window_copy_append_selection(&mut *wme);
        }
        window_copy_clear_selection(&mut *wme);
        WINDOW_COPY_CMD_CANCEL
    }
}
unsafe fn window_copy_cmd_back_to_indentation(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        window_copy_cursor_back_to_indentation(&mut *wme);
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_begin_selection(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let c = cs.c.as_deref_mut();
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        if let Some(m) = cs.m {
            window_copy_start_drag(c, m);
            return WINDOW_COPY_CMD_NOTHING;
        }
        data.lineflag = LINE_SEL_NONE;
        data.selflag = SEL_CHAR;
        window_copy_start_selection(&mut *wme);
        WINDOW_COPY_CMD_REDRAW
    }
}
fn window_copy_cmd_stop_selection(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    {
        let wme = &mut *cs.wme;
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        data.cursordrag = CURSORDRAG_NONE;
        data.lineflag = LINE_SEL_NONE;
        data.selflag = SEL_CHAR;
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_bottom_line(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        data.cx = 0 as u_int;
        data.cy = RustScreen::grid(&data.screen.borrow())
            .height()
            .wrapping_sub(1 as u_int);
        window_copy_update_selection(&mut *wme, 1 as core::ffi::c_int, 0 as core::ffi::c_int);
        WINDOW_COPY_CMD_REDRAW
    }
}
fn window_copy_cmd_cancel(_cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    WINDOW_COPY_CMD_CANCEL
}
unsafe fn window_copy_cmd_clear_selection(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        window_copy_clear_selection(&mut *wme);
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_do_copy_end_of_line(
    cs: &mut window_copy_cmd_state<'_>,
    pipe: core::ffi::c_int,
    cancel: core::ffi::c_int,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let c = cs.c.as_deref_mut();
        let s = cs.s;
        let wl = cs.wl;
        let pane = wme.pane_ref().expect("mode has a pane");
        let count: u_int = {
            let args: &RustArguments = cs.wargs;
            args.argument_count()
        };
        let mut np: u_int = wme.prefix;

        let mut prefix: Option<CString> = None;
        let mut command: Option<CString> = None;
        let arg0 = {
            let args: &RustArguments = cs.wargs;
            let idx = 0 as u_int;
            args.argument_string(idx)
        };
        let arg1 = {
            let args: &RustArguments = cs.wargs;
            let idx = 1 as u_int;
            args.argument_string(idx)
        };
        let set_paste: core::ffi::c_int = (({
            let args: &RustArguments = cs.wargs;
            let flag = 'P' as i32 as u_char;
            args.argument_flag_count(flag)
        }) == 0) as core::ffi::c_int;
        let set_clip: core::ffi::c_int = (({
            let args: &RustArguments = cs.wargs;
            let flag = 'C' as i32 as u_char;
            args.argument_flag_count(flag)
        }) == 0) as core::ffi::c_int;
        if pipe != 0 {
            if let Some(arg1) = arg1.filter(|_| count == 2 as u_int) {
                prefix = Some(format_single(None, arg1, c.as_deref(), s, wl, pane.get()));
            }
            if s.is_some()
                && let Some(arg0) =
                    arg0.filter(|arg0| count > 0 as u_int && !arg0.to_bytes().is_empty())
            {
                command = Some(format_single(None, arg0, c.as_deref(), s, wl, pane.get()));
            }
        } else if let Some(arg0) = arg0.filter(|_| count == 1 as u_int) {
            prefix = Some(format_single(None, arg0, c.as_deref(), s, wl, pane.get()));
        }
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let ocx: u_int = data.cx;
        let ocy: u_int = data.cy;
        let ooy: u_int = data.oy;
        window_copy_start_selection(&mut *wme);
        while np > 1 as u_int {
            window_copy_cursor_down(&mut *wme, 0 as core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
        window_copy_cursor_end_of_line(&mut *wme);
        if s.is_some() {
            if pipe != 0 {
                window_copy_copy_pipe(
                    &mut *wme,
                    cs.s,
                    prefix.as_deref(),
                    command.as_deref(),
                    set_paste,
                    set_clip,
                );
            } else {
                window_copy_copy_selection(&mut *wme, prefix.as_deref(), set_paste, set_clip);
            }
            if cancel != 0 {
                return WINDOW_COPY_CMD_CANCEL;
            }
        }
        window_copy_clear_selection(&mut *wme);
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        data.cx = ocx;
        data.cy = ocy;
        data.oy = ooy;
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_copy_end_of_line(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe { window_copy_do_copy_end_of_line(cs, 0 as core::ffi::c_int, 0 as core::ffi::c_int) }
}
unsafe fn window_copy_cmd_copy_end_of_line_and_cancel(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe { window_copy_do_copy_end_of_line(cs, 0 as core::ffi::c_int, 1 as core::ffi::c_int) }
}
unsafe fn window_copy_cmd_copy_pipe_end_of_line(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe { window_copy_do_copy_end_of_line(cs, 1 as core::ffi::c_int, 0 as core::ffi::c_int) }
}
unsafe fn window_copy_cmd_copy_pipe_end_of_line_and_cancel(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe { window_copy_do_copy_end_of_line(cs, 1 as core::ffi::c_int, 1 as core::ffi::c_int) }
}
unsafe fn window_copy_do_copy_line(
    cs: &mut window_copy_cmd_state<'_>,
    pipe: core::ffi::c_int,
    cancel: core::ffi::c_int,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let c = cs.c.as_deref_mut();
        let s = cs.s;
        let wl = cs.wl;
        let pane = wme.pane_ref().expect("mode has a pane");
        let count: u_int = {
            let args: &RustArguments = cs.wargs;
            args.argument_count()
        };
        let mut np: u_int = wme.prefix;

        let mut prefix: Option<CString> = None;
        let mut command: Option<CString> = None;
        let arg0 = {
            let args: &RustArguments = cs.wargs;
            let idx = 0 as u_int;
            args.argument_string(idx)
        };
        let arg1 = {
            let args: &RustArguments = cs.wargs;
            let idx = 1 as u_int;
            args.argument_string(idx)
        };
        let set_paste: core::ffi::c_int = (({
            let args: &RustArguments = cs.wargs;
            let flag = 'P' as i32 as u_char;
            args.argument_flag_count(flag)
        }) == 0) as core::ffi::c_int;
        let set_clip: core::ffi::c_int = (({
            let args: &RustArguments = cs.wargs;
            let flag = 'C' as i32 as u_char;
            args.argument_flag_count(flag)
        }) == 0) as core::ffi::c_int;
        if pipe != 0 {
            if let Some(arg1) = arg1.filter(|_| count == 2 as u_int) {
                prefix = Some(format_single(None, arg1, c.as_deref(), s, wl, pane.get()));
            }
            if s.is_some()
                && let Some(arg0) =
                    arg0.filter(|arg0| count > 0 as u_int && !arg0.to_bytes().is_empty())
            {
                command = Some(format_single(None, arg0, c.as_deref(), s, wl, pane.get()));
            }
        } else if let Some(arg0) = arg0.filter(|_| count == 1 as u_int) {
            prefix = Some(format_single(None, arg0, c.as_deref(), s, wl, pane.get()));
        }
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        let ocx: u_int = data.cx;
        let ocy: u_int = data.cy;
        let ooy: u_int = data.oy;
        data.selflag = SEL_CHAR;
        window_copy_cursor_start_of_line(&mut *wme);
        window_copy_start_selection(&mut *wme);
        while np > 1 as u_int {
            window_copy_cursor_down(&mut *wme, 0 as core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
        window_copy_cursor_end_of_line(&mut *wme);
        if s.is_some() {
            if pipe != 0 {
                window_copy_copy_pipe(
                    &mut *wme,
                    cs.s,
                    prefix.as_deref(),
                    command.as_deref(),
                    set_paste,
                    set_clip,
                );
            } else {
                window_copy_copy_selection(&mut *wme, prefix.as_deref(), set_paste, set_clip);
            }
            if cancel != 0 {
                return WINDOW_COPY_CMD_CANCEL;
            }
        }
        window_copy_clear_selection(&mut *wme);
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        data.cx = ocx;
        data.cy = ocy;
        data.oy = ooy;
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_copy_line(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    unsafe { window_copy_do_copy_line(cs, 0 as core::ffi::c_int, 0 as core::ffi::c_int) }
}
unsafe fn window_copy_cmd_copy_line_and_cancel(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe { window_copy_do_copy_line(cs, 0 as core::ffi::c_int, 1 as core::ffi::c_int) }
}
unsafe fn window_copy_cmd_copy_pipe_line(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe { window_copy_do_copy_line(cs, 1 as core::ffi::c_int, 0 as core::ffi::c_int) }
}
unsafe fn window_copy_cmd_copy_pipe_line_and_cancel(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe { window_copy_do_copy_line(cs, 1 as core::ffi::c_int, 1 as core::ffi::c_int) }
}
unsafe fn window_copy_cmd_copy_selection_no_clear(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let c = cs.c.as_deref_mut();
        let s = cs.s;
        let wl = cs.wl;
        let pane = wme.pane_ref().expect("mode has a pane");
        let mut prefix: Option<CString> = None;
        let arg0 = {
            let args: &RustArguments = cs.wargs;
            let idx = 0 as u_int;
            args.argument_string(idx)
        };
        let set_paste: core::ffi::c_int = (({
            let args: &RustArguments = cs.wargs;
            let flag = 'P' as i32 as u_char;
            args.argument_flag_count(flag)
        }) == 0) as core::ffi::c_int;
        let set_clip: core::ffi::c_int = (({
            let args: &RustArguments = cs.wargs;
            let flag = 'C' as i32 as u_char;
            args.argument_flag_count(flag)
        }) == 0) as core::ffi::c_int;
        if let Some(arg0) = arg0 {
            prefix = Some(format_single(None, arg0, c.as_deref(), s, wl, pane.get()));
        }
        if s.is_some() {
            window_copy_copy_selection(&mut *wme, prefix.as_deref(), set_paste, set_clip);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_copy_selection(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        window_copy_cmd_copy_selection_no_clear(cs);
        window_copy_clear_selection(cs.wme);
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_copy_selection_and_cancel(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        window_copy_cmd_copy_selection_no_clear(cs);
        window_copy_clear_selection(cs.wme);
        WINDOW_COPY_CMD_CANCEL
    }
}
unsafe fn window_copy_cmd_cursor_down(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let mut np: u_int = wme.prefix;
        while np != 0 as u_int {
            window_copy_cursor_down(&mut *wme, 0 as core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_cursor_down_and_cancel(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let mut np: u_int = wme.prefix;

        let cy: u_int = wme
            .state
            .copy_mode_data_ref()
            .expect("copy mode has state")
            .cy;
        while np != 0 as u_int {
            window_copy_cursor_down(&mut *wme, 0 as core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        if cy == data.cy && data.oy == 0 as u_int {
            return WINDOW_COPY_CMD_CANCEL;
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_cursor_left(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let mut np: u_int = wme.prefix;
        while np != 0 as u_int {
            window_copy_cursor_left(&mut *wme);
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_cursor_right(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let mut np: u_int = wme.prefix;
        while np != 0 as u_int {
            let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
            let rectangle =
                (data.screen.borrow().has_selection() && data.rectflag != 0) as core::ffi::c_int;
            window_copy_cursor_right(wme, rectangle);
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_scroll_to(
    cs: &mut window_copy_cmd_state<'_>,
    to: u_int,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");

        let scroll_up: core::ffi::c_int = data.cy.wrapping_sub(to) as core::ffi::c_int;
        let delta: u_int = abs(scroll_up) as u_int;
        let oy: u_int = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_sub(data.oy);
        if scroll_up > 0 as core::ffi::c_int && data.oy >= delta {
            window_copy_scroll_up(&mut *wme, delta);
            let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
            data.cy = data.cy.wrapping_sub(delta);
        } else if scroll_up < 0 as core::ffi::c_int && oy >= delta {
            window_copy_scroll_down(&mut *wme, delta);
            let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
            data.cy = data.cy.wrapping_add(delta);
        }
        window_copy_update_selection(&mut *wme, 0 as core::ffi::c_int, 0 as core::ffi::c_int);
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_scroll_bottom(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let data = cs
            .wme
            .state
            .copy_mode_data_ref()
            .expect("copy mode has state");

        let bottom: u_int = RustScreen::grid(&data.screen.borrow())
            .height()
            .wrapping_sub(1 as u_int);
        window_copy_cmd_scroll_to(cs, bottom)
    }
}
unsafe fn window_copy_cmd_scroll_middle(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let data = cs
            .wme
            .state
            .copy_mode_data_ref()
            .expect("copy mode has state");

        let mid_value: u_int = RustScreen::grid(&data.screen.borrow())
            .height()
            .wrapping_sub(1 as u_int)
            .wrapping_div(2 as u_int);
        window_copy_cmd_scroll_to(cs, mid_value)
    }
}
unsafe fn window_copy_cmd_scroll_to_mouse(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let mut pane = wme.pane_ref().expect("mode has a pane");
        let wp = pane.get_mut().expect("mode pane is live");
        let c =
            cs.c.as_deref_mut()
                .expect("the scroll command has a client");
        let m = cs.m.expect("the scroll command has a mouse event");
        let scroll_exit: core::ffi::c_int = {
            let args: &RustArguments = cs.wargs;
            let flag = 'e' as i32 as u_char;
            args.argument_flag_count(flag)
        };
        let (_bigger, _tty_ox, tty_oy, _tty_sx, _tty_sy) = tty_window_offset(&c.tty);
        window_copy_scroll(&mut *wp, c.tty.mouse_slider_mpos, m.y, tty_oy, scroll_exit);
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_scroll_top(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    unsafe { window_copy_cmd_scroll_to(cs, 0 as u_int) }
}
unsafe fn window_copy_cmd_cursor_up(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let mut np: u_int = wme.prefix;
        while np != 0 as u_int {
            window_copy_cursor_up(&mut *wme, 0 as core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_centre_vertical(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let pane = wme.pane_ref().expect("mode has a pane");
        let geometry = pane.get().expect("mode pane is live").geometry();
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let x = data.cx;
        let y = geometry.sy.wrapping_div(2);
        window_copy_update_cursor(wme, x, y);
        window_copy_update_selection(&mut *wme, 1 as core::ffi::c_int, 0 as core::ffi::c_int);
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_centre_horizontal(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let pane = wme.pane_ref().expect("mode has a pane");
        let geometry = pane.get().expect("mode pane is live").geometry();
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let x = geometry.sx.wrapping_div(2);
        let y = data.cy;
        window_copy_update_cursor(wme, x, y);
        window_copy_update_selection(&mut *wme, 1 as core::ffi::c_int, 0 as core::ffi::c_int);
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_end_of_line(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        window_copy_cursor_end_of_line(&mut *wme);
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_halfpage_down(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let mut np: u_int = wme.prefix;
        while np != 0 as u_int {
            let scroll_exit = wme
                .state
                .copy_mode_data_ref()
                .expect("copy mode has state")
                .scroll_exit;
            if window_copy_pagedown1(&mut *wme, 1 as core::ffi::c_int, scroll_exit) != 0 {
                return WINDOW_COPY_CMD_CANCEL;
            }
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_halfpage_down_and_cancel(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let mut np: u_int = wme.prefix;
        while np != 0 as u_int {
            if window_copy_pagedown1(&mut *wme, 1 as core::ffi::c_int, 1 as core::ffi::c_int) != 0 {
                return WINDOW_COPY_CMD_CANCEL;
            }
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_halfpage_up(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let mut np: u_int = wme.prefix;
        while np != 0 as u_int {
            window_copy_pageup1(&mut *wme, 1 as core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
fn window_copy_cmd_toggle_position(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    {
        let wme = &mut *cs.wme;
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        data.hide_position = (data.hide_position == 0) as core::ffi::c_int;
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_history_bottom(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let s = data
            .backing
            .as_deref()
            .expect("copy mode has a backing screen");

        let oy: u_int = RustScreen::grid(s)
            .history_size()
            .wrapping_add(data.cy)
            .wrapping_sub(data.oy);
        if data.lineflag as core::ffi::c_uint
            == LINE_SEL_RIGHT_LEFT as core::ffi::c_int as core::ffi::c_uint
            && oy == data.endsely
        {
            window_copy_other_end(&mut *wme);
        }
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        data.cy = RustScreen::grid(&data.screen.borrow())
            .height()
            .wrapping_sub(1 as u_int);
        let row = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_add(data.cy);
        let cx = window_copy_cursor_limit(wme, row, 0);
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        data.cx = cx;
        data.oy = 0;
        if !data.searchmark.is_empty() && data.timeout == 0 {
            let regex = data.searchregex;
            window_copy_search_marks(wme, None, regex, 1);
        }
        window_copy_update_selection(&mut *wme, 1 as core::ffi::c_int, 0 as core::ffi::c_int);
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_history_top(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");

        let oy: u_int = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_add(data.cy)
        .wrapping_sub(data.oy);
        if data.lineflag as core::ffi::c_uint
            == LINE_SEL_LEFT_RIGHT as core::ffi::c_int as core::ffi::c_uint
            && oy == data.sely
        {
            window_copy_other_end(&mut *wme);
        }
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        data.cy = 0 as u_int;
        data.cx = 0 as u_int;
        data.oy = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size();
        if !data.searchmark.is_empty() && data.timeout == 0 {
            let regex = data.searchregex;
            window_copy_search_marks(wme, None, regex, 1);
        }
        window_copy_update_selection(&mut *wme, 1 as core::ffi::c_int, 0 as core::ffi::c_int);
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_jump_again(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let mut np: u_int = wme.prefix;
        match data.jumptype {
            3 => {
                while np != 0 as u_int {
                    window_copy_cursor_jump(&mut *wme);
                    np = np.wrapping_sub(1);
                }
            }
            4 => {
                while np != 0 as u_int {
                    window_copy_cursor_jump_back(&mut *wme);
                    np = np.wrapping_sub(1);
                }
            }
            5 => {
                while np != 0 as u_int {
                    window_copy_cursor_jump_to(&mut *wme);
                    np = np.wrapping_sub(1);
                }
            }
            6 => {
                while np != 0 as u_int {
                    window_copy_cursor_jump_to_back(&mut *wme);
                    np = np.wrapping_sub(1);
                }
            }
            _ => {}
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_jump_reverse(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let mut np: u_int = wme.prefix;
        match data.jumptype {
            3 => {
                while np != 0 as u_int {
                    window_copy_cursor_jump_back(&mut *wme);
                    np = np.wrapping_sub(1);
                }
            }
            4 => {
                while np != 0 as u_int {
                    window_copy_cursor_jump(&mut *wme);
                    np = np.wrapping_sub(1);
                }
            }
            5 => {
                while np != 0 as u_int {
                    window_copy_cursor_jump_to_back(&mut *wme);
                    np = np.wrapping_sub(1);
                }
            }
            6 => {
                while np != 0 as u_int {
                    window_copy_cursor_jump_to(&mut *wme);
                    np = np.wrapping_sub(1);
                }
            }
            _ => {}
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_middle_line(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        data.cx = 0 as u_int;
        data.cy = RustScreen::grid(&data.screen.borrow())
            .height()
            .wrapping_sub(1 as u_int)
            .wrapping_div(2 as u_int);
        window_copy_update_selection(&mut *wme, 1 as core::ffi::c_int, 0 as core::ffi::c_int);
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_previous_matching_bracket(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe { window_copy_previous_matching_bracket(cs.wme) };
    WINDOW_COPY_CMD_NOTHING
}
unsafe fn window_copy_previous_matching_bracket(wme: &mut window_mode_entry) {
    unsafe {
        let mut np: u_int = wme.prefix;
        let open = c"{[(";
        let close = c"}])";
        let mut tried: core::ffi::c_char;
        let mut found: u8 = 0;
        let mut start: u8;
        let mut cp: Option<usize>;
        let mut px: u_int;
        let mut py: u_int;
        let mut xx: u_int;
        let mut n: u_int;
        let mut gc;
        let mut failed: core::ffi::c_int;
        while np != 0 as u_int {
            let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
            let s = data
                .backing
                .as_deref()
                .expect("copy mode has a backing screen");
            px = data.cx;
            py = RustScreen::grid(s)
                .history_size()
                .wrapping_add(data.cy)
                .wrapping_sub(data.oy);
            xx = window_copy_find_length(wme, py);
            if xx == 0 as u_int {
                break;
            }
            tried = 0 as core::ffi::c_char;
            loop {
                gc = RustScreen::grid(s).cell(px, py);
                if gc.data.size as core::ffi::c_int != 1 as core::ffi::c_int
                    || gc.flags as core::ffi::c_int & GRID_FLAG_PADDING != 0
                {
                    cp = None;
                } else {
                    found = gc.data.data[0];
                    cp = close.to_bytes_with_nul().iter().position(|&ch| ch == found);
                }
                if cp.is_none() {
                    if !(data.modekeys == MODEKEY_EMACS) {
                        break;
                    }
                    if tried == 0 && px > 0 as u_int {
                        px = px.wrapping_sub(1);
                        tried = 1 as core::ffi::c_char;
                    } else {
                        window_copy_cursor_previous_word(&mut *wme, close, 1 as core::ffi::c_int);
                        break;
                    }
                } else if let Some(cp) = cp {
                    start = open.to_bytes_with_nul()[cp];
                    n = 1 as u_int;
                    failed = 0 as core::ffi::c_int;
                    loop {
                        if px == 0 as u_int {
                            if py == 0 as u_int {
                                failed = 1 as core::ffi::c_int;
                                break;
                            } else {
                                loop {
                                    py = py.wrapping_sub(1);
                                    xx = window_copy_find_length(wme, py);
                                    if !(xx == 0 as u_int && py > 0 as u_int) {
                                        break;
                                    }
                                }
                                if xx == 0 as u_int && py == 0 as u_int {
                                    failed = 1 as core::ffi::c_int;
                                    break;
                                } else {
                                    px = xx.wrapping_sub(1 as u_int);
                                }
                            }
                        } else {
                            px = px.wrapping_sub(1);
                        }
                        gc = RustScreen::grid(s).cell(px, py);
                        if gc.data.size as core::ffi::c_int == 1 as core::ffi::c_int
                            && !(gc.flags as core::ffi::c_int) & GRID_FLAG_PADDING != 0
                        {
                            if gc.data.data[0] as core::ffi::c_int == found as core::ffi::c_int {
                                n = n.wrapping_add(1);
                            } else if gc.data.data[0] as core::ffi::c_int
                                == start as core::ffi::c_int
                            {
                                n = n.wrapping_sub(1);
                            }
                        }
                        if !(n != 0 as u_int) {
                            break;
                        }
                    }
                    if failed == 0 {
                        window_copy_scroll_to(&mut *wme, px, py, 0 as core::ffi::c_int);
                    }
                    break;
                }
            }
            np = np.wrapping_sub(1);
        }
    }
}
unsafe fn window_copy_cmd_next_matching_bracket(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let mut np = wme.prefix;
        let open = c"{[(";
        let close = c"}])";
        let mut tried: core::ffi::c_char;
        let mut found: u8 = 0;
        let mut end: u8;
        let mut cp: Option<usize>;
        let mut px: u_int;
        let mut py: u_int;
        let mut xx: u_int;
        let mut yy: u_int;
        let sx: u_int;
        let sy: u_int;
        let mut n: u_int;
        let mut gc;
        let mut failed: core::ffi::c_int;
        's_22: while np != 0 as u_int {
            let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
            let s = data
                .backing
                .as_deref()
                .expect("copy mode has a backing screen");
            px = data.cx;
            py = RustScreen::grid(s)
                .history_size()
                .wrapping_add(data.cy)
                .wrapping_sub(data.oy);
            xx = window_copy_find_length(wme, py);
            yy = RustScreen::grid(s)
                .history_size()
                .wrapping_add(RustScreen::grid(s).height())
                .wrapping_sub(1 as u_int);
            if xx == 0 as u_int {
                break;
            }
            tried = 0 as core::ffi::c_char;
            loop {
                gc = RustScreen::grid(s).cell(px, py);
                if gc.data.size as core::ffi::c_int != 1 as core::ffi::c_int
                    || gc.flags as core::ffi::c_int & GRID_FLAG_PADDING != 0
                {
                    cp = None;
                } else {
                    found = gc.data.data[0];
                    cp = close.to_bytes_with_nul().iter().position(|&ch| ch == found);
                    if cp.is_some() && data.modekeys == MODEKEY_VI {
                        sx = data.cx;
                        sy = RustScreen::grid(s)
                            .history_size()
                            .wrapping_add(data.cy)
                            .wrapping_sub(data.oy);
                        window_copy_scroll_to(&mut *wme, px, py, 0 as core::ffi::c_int);
                        window_copy_previous_matching_bracket(wme);
                        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
                        let s = data
                            .backing
                            .as_deref()
                            .expect("copy mode has a backing screen");
                        px = data.cx;
                        py = RustScreen::grid(s)
                            .history_size()
                            .wrapping_add(data.cy)
                            .wrapping_sub(data.oy);
                        gc = RustScreen::grid(s).cell(px, py);
                        if gc.data.size as core::ffi::c_int == 1 as core::ffi::c_int
                            && !(gc.flags as core::ffi::c_int) & GRID_FLAG_PADDING != 0
                            && close.to_bytes_with_nul().contains(&gc.data.data[0])
                        {
                            window_copy_scroll_to(&mut *wme, sx, sy, 0 as core::ffi::c_int);
                        }
                        break 's_22;
                    } else {
                        cp = open.to_bytes_with_nul().iter().position(|&ch| ch == found);
                    }
                }
                if cp.is_none() {
                    if data.modekeys == MODEKEY_EMACS {
                        if tried == 0 && px <= xx {
                            px = px.wrapping_add(1);
                            tried = 1 as core::ffi::c_char;
                        } else {
                            window_copy_cursor_next_word_end(
                                &mut *wme,
                                open,
                                0 as core::ffi::c_int,
                            );
                            break;
                        }
                    } else if px > xx {
                        if py == yy {
                            break;
                        }
                        let gl = (RustScreen::grid(s)).line_info(py);
                        if !gl.flags & GRID_LINE_WRAPPED != 0 {
                            break;
                        }
                        if gl.cells > RustScreen::grid(s).width() {
                            break;
                        }
                        px = 0 as u_int;
                        py = py.wrapping_add(1);
                        xx = window_copy_find_length(wme, py);
                    } else {
                        px = px.wrapping_add(1);
                    }
                } else if let Some(cp) = cp {
                    end = close.to_bytes_with_nul()[cp];
                    n = 1 as u_int;
                    failed = 0 as core::ffi::c_int;
                    loop {
                        if px > xx {
                            if py == yy {
                                failed = 1 as core::ffi::c_int;
                                break;
                            } else {
                                px = 0 as u_int;
                                py = py.wrapping_add(1);
                                xx = window_copy_find_length(wme, py);
                            }
                        } else {
                            px = px.wrapping_add(1);
                        }
                        gc = RustScreen::grid(s).cell(px, py);
                        if gc.data.size as core::ffi::c_int == 1 as core::ffi::c_int
                            && !(gc.flags as core::ffi::c_int) & GRID_FLAG_PADDING != 0
                        {
                            if gc.data.data[0] as core::ffi::c_int == found as core::ffi::c_int {
                                n = n.wrapping_add(1);
                            } else if gc.data.data[0] as core::ffi::c_int == end as core::ffi::c_int
                            {
                                n = n.wrapping_sub(1);
                            }
                        }
                        if !(n != 0 as u_int) {
                            break;
                        }
                    }
                    if failed == 0 {
                        window_copy_scroll_to(&mut *wme, px, py, 0 as core::ffi::c_int);
                    }
                    break;
                }
            }
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_next_paragraph(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let mut np: u_int = wme.prefix;
        while np != 0 as u_int {
            window_copy_next_paragraph(&mut *wme);
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_next_space(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let mut np: u_int = wme.prefix;
        while np != 0 as u_int {
            window_copy_cursor_next_word(&mut *wme, c"");
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_next_space_end(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let mut np: u_int = wme.prefix;
        while np != 0 as u_int {
            window_copy_cursor_next_word_end(&mut *wme, c"", 0 as core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_next_word(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let mut np: u_int = wme.prefix;
        let separators = ((cs.s.expect("the copy command has a session"))
            .options_ref()
            .clone())
        .string_ref(c"word-separators");
        let separators = separators.as_ref();
        while np != 0 as u_int {
            window_copy_cursor_next_word(&mut *wme, separators);
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_next_word_end(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let mut np: u_int = wme.prefix;
        let separators = ((cs.s.expect("the copy command has a session"))
            .options_ref()
            .clone())
        .string_ref(c"word-separators");
        let separators = separators.as_ref();
        while np != 0 as u_int {
            window_copy_cursor_next_word_end(&mut *wme, separators, 0 as core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_other_end(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let np: u_int = wme.prefix;
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        data.selflag = SEL_CHAR;
        if np.wrapping_rem(2 as u_int) != 0 as u_int {
            window_copy_other_end(&mut *wme);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
fn window_copy_cmd_selection_mode(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    {
        let wme = &mut *cs.wme;
        let so = (cs.s.expect("the copy command has a session"))
            .options_ref()
            .clone();
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        let s = {
            let args: &RustArguments = cs.wargs;
            let idx = 0 as u_int;
            args.argument_string(idx)
        };
        let named = |name: &CStr| s.is_some_and(|s| cstr_eq_ignore_case(s, name));
        if s.is_none() || named(c"char") || named(c"c") {
            data.selflag = SEL_CHAR;
        } else if named(c"word") || named(c"w") {
            data.separators = Some(so.string_ref(c"word-separators"));
            data.selflag = SEL_WORD;
        } else if named(c"line") || named(c"l") {
            data.selflag = SEL_LINE;
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_page_down(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let mut np: u_int = wme.prefix;
        while np != 0 as u_int {
            let scroll_exit = wme
                .state
                .copy_mode_data_ref()
                .expect("copy mode has state")
                .scroll_exit;
            if window_copy_pagedown1(&mut *wme, 0 as core::ffi::c_int, scroll_exit) != 0 {
                return WINDOW_COPY_CMD_CANCEL;
            }
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_page_down_and_cancel(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let mut np: u_int = wme.prefix;
        while np != 0 as u_int {
            if window_copy_pagedown1(&mut *wme, 0 as core::ffi::c_int, 1 as core::ffi::c_int) != 0 {
                return WINDOW_COPY_CMD_CANCEL;
            }
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_page_up(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let mut np: u_int = wme.prefix;
        while np != 0 as u_int {
            window_copy_pageup1(&mut *wme, 0 as core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_previous_paragraph(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let mut np: u_int = wme.prefix;
        while np != 0 as u_int {
            window_copy_previous_paragraph(&mut *wme);
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_previous_space(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let mut np: u_int = wme.prefix;
        while np != 0 as u_int {
            window_copy_cursor_previous_word(&mut *wme, c"", 1 as core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_previous_word(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let mut np: u_int = wme.prefix;
        let separators = ((cs.s.expect("the copy command has a session"))
            .options_ref()
            .clone())
        .string_ref(c"word-separators");
        let separators = separators.as_ref();
        while np != 0 as u_int {
            window_copy_cursor_previous_word(&mut *wme, separators, 1 as core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_rectangle_on(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        data.lineflag = LINE_SEL_NONE;
        window_copy_rectangle_set(&mut *wme, 1 as core::ffi::c_int);
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_rectangle_off(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        data.lineflag = LINE_SEL_NONE;
        window_copy_rectangle_set(&mut *wme, 0 as core::ffi::c_int);
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_rectangle_toggle(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        data.lineflag = LINE_SEL_NONE;
        let rectangle = (data.rectflag == 0) as core::ffi::c_int;
        window_copy_rectangle_set(wme, rectangle);
        WINDOW_COPY_CMD_NOTHING
    }
}
fn window_copy_cmd_scroll_exit_on(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    {
        let data = cs
            .wme
            .state
            .copy_mode_data_mut()
            .expect("copy mode has state");
        data.scroll_exit = 1 as core::ffi::c_int;
        WINDOW_COPY_CMD_NOTHING
    }
}
fn window_copy_cmd_scroll_exit_off(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    {
        let data = cs
            .wme
            .state
            .copy_mode_data_mut()
            .expect("copy mode has state");
        data.scroll_exit = 0 as core::ffi::c_int;
        WINDOW_COPY_CMD_NOTHING
    }
}
fn window_copy_cmd_scroll_exit_toggle(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    {
        let data = cs
            .wme
            .state
            .copy_mode_data_mut()
            .expect("copy mode has state");
        data.scroll_exit = (data.scroll_exit == 0) as core::ffi::c_int;
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_scroll_down(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let mut np: u_int = wme.prefix;
        while np != 0 as u_int {
            window_copy_cursor_down(&mut *wme, 1 as core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        if data.scroll_exit != 0 && data.oy == 0 as u_int {
            return WINDOW_COPY_CMD_CANCEL;
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_scroll_down_and_cancel(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let mut np: u_int = wme.prefix;
        while np != 0 as u_int {
            window_copy_cursor_down(&mut *wme, 1 as core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        if data.oy == 0 as u_int {
            return WINDOW_COPY_CMD_CANCEL;
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_scroll_up(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let mut np: u_int = wme.prefix;
        while np != 0 as u_int {
            window_copy_cursor_up(&mut *wme, 1 as core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_search_again(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let mut np: u_int = wme.prefix;
        let search_type = wme
            .state
            .copy_mode_data_ref()
            .expect("copy mode has state")
            .searchtype;
        if search_type == WINDOW_COPY_SEARCHUP as core::ffi::c_int {
            while np != 0 as u_int {
                let regex = wme
                    .state
                    .copy_mode_data_ref()
                    .expect("copy mode has state")
                    .searchregex;
                window_copy_search_up(&mut *wme, regex);
                np = np.wrapping_sub(1);
            }
        } else if search_type == WINDOW_COPY_SEARCHDOWN as core::ffi::c_int {
            while np != 0 as u_int {
                let regex = wme
                    .state
                    .copy_mode_data_ref()
                    .expect("copy mode has state")
                    .searchregex;
                window_copy_search_down(&mut *wme, regex);
                np = np.wrapping_sub(1);
            }
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_search_reverse(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let mut np: u_int = wme.prefix;
        let search_type = wme
            .state
            .copy_mode_data_ref()
            .expect("copy mode has state")
            .searchtype;
        if search_type == WINDOW_COPY_SEARCHUP as core::ffi::c_int {
            while np != 0 as u_int {
                let regex = wme
                    .state
                    .copy_mode_data_ref()
                    .expect("copy mode has state")
                    .searchregex;
                window_copy_search_down(&mut *wme, regex);
                np = np.wrapping_sub(1);
            }
        } else if search_type == WINDOW_COPY_SEARCHDOWN as core::ffi::c_int {
            while np != 0 as u_int {
                let regex = wme
                    .state
                    .copy_mode_data_ref()
                    .expect("copy mode has state")
                    .searchregex;
                window_copy_search_up(&mut *wme, regex);
                np = np.wrapping_sub(1);
            }
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_select_line(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        let mut np: u_int = wme.prefix;
        data.lineflag = LINE_SEL_LEFT_RIGHT;
        data.rectflag = 0 as core::ffi::c_int;
        data.selflag = SEL_LINE;
        data.dx = data.cx;
        data.dy = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_add(data.cy)
        .wrapping_sub(data.oy);
        window_copy_cursor_start_of_line(wme);
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        data.selrx = data.cx;
        data.selry = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_add(data.cy)
        .wrapping_sub(data.oy);
        data.endselry = data.selry;
        window_copy_start_selection(&mut *wme);
        window_copy_cursor_end_of_line(wme);
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        data.endselry = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_add(data.cy)
        .wrapping_sub(data.oy);
        let row = data.endselry;
        let end = window_copy_find_length(wme, row);
        wme.state
            .copy_mode_data_mut()
            .expect("copy mode has state")
            .endselrx = end;
        while np > 1 as u_int {
            window_copy_cursor_down(&mut *wme, 0 as core::ffi::c_int);
            window_copy_cursor_end_of_line(&mut *wme);
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_select_word(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let so = (cs.s.expect("the copy command has a session"))
            .options_ref()
            .clone();
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");

        let mut nextx: u_int;
        let mut nexty: u_int;
        data.lineflag = LINE_SEL_LEFT_RIGHT;
        data.rectflag = 0 as core::ffi::c_int;
        data.selflag = SEL_WORD;
        data.dx = data.cx;
        data.dy = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_add(data.cy)
        .wrapping_sub(data.oy);
        let separators = so.string_ref(c"word-separators");
        data.separators = Some(separators.clone());
        window_copy_cursor_previous_word(wme, &separators, 0);
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        let px: u_int = data.cx;
        let py: u_int = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_add(data.cy)
        .wrapping_sub(data.oy);
        data.selrx = px;
        data.selry = py;
        window_copy_start_selection(&mut *wme);
        nextx = px.wrapping_add(1 as u_int);
        nexty = py;
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        if (RustScreen::grid(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
            )).line_info(nexty,)
        .flags
            & GRID_LINE_WRAPPED
            != 0
            && nextx
                > RustScreen::grid(
                    data.backing
                        .as_deref()
                        .expect("copy mode has a backing screen"),
                )
                .width()
                .wrapping_sub(1 as u_int)
        {
            nextx = 0 as u_int;
            nexty = nexty.wrapping_add(1);
        }
        if px >= window_copy_find_length(wme, py)
            || window_copy_in_set(wme, nextx, nexty, WHITESPACE) == 0
        {
            let separators = data.separators.clone();
            window_copy_cursor_next_word_end(
                wme,
                separators.as_deref().unwrap_or(c""),
                1 as core::ffi::c_int,
            );
        } else {
            let cy = data.cy;
            window_copy_update_cursor(wme, px, cy);
            if window_copy_update_selection(&mut *wme, 1 as core::ffi::c_int, 1 as core::ffi::c_int)
                != 0
            {
                let cy = wme
                    .state
                    .copy_mode_data_ref()
                    .expect("copy mode has state")
                    .cy;
                window_copy_redraw_lines(wme, cy, 1);
            }
        }
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        data.endselrx = data.cx;
        data.endselry = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_add(data.cy)
        .wrapping_sub(data.oy);
        if data.dy > data.endselry {
            data.dy = data.endselry;
            data.dx = data.endselrx;
        } else if data.dx > data.endselrx {
            data.dx = data.endselrx;
        }
        WINDOW_COPY_CMD_REDRAW
    }
}
fn window_copy_cmd_set_mark(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    {
        let data = cs
            .wme
            .state
            .copy_mode_data_mut()
            .expect("copy mode has state");
        data.mx = data.cx;
        data.my = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_add(data.cy)
        .wrapping_sub(data.oy);
        data.showmark = 1 as core::ffi::c_int;
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_start_of_line(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        window_copy_cursor_start_of_line(&mut *wme);
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_top_line(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        data.cx = 0 as u_int;
        data.cy = 0 as u_int;
        window_copy_update_selection(&mut *wme, 1 as core::ffi::c_int, 0 as core::ffi::c_int);
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_copy_pipe_no_clear(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let c = cs.c.as_deref_mut();
        let s = cs.s;
        let wl = cs.wl;
        let pane = wme.pane_ref().expect("mode has a pane");
        let mut command: Option<CString> = None;
        let mut prefix: Option<CString> = None;
        let arg0 = {
            let args: &RustArguments = cs.wargs;
            let idx = 0 as u_int;
            args.argument_string(idx)
        };
        let arg1 = {
            let args: &RustArguments = cs.wargs;
            let idx = 1 as u_int;
            args.argument_string(idx)
        };
        let set_paste: core::ffi::c_int = (({
            let args: &RustArguments = cs.wargs;
            let flag = 'P' as i32 as u_char;
            args.argument_flag_count(flag)
        }) == 0) as core::ffi::c_int;
        let set_clip: core::ffi::c_int = (({
            let args: &RustArguments = cs.wargs;
            let flag = 'C' as i32 as u_char;
            args.argument_flag_count(flag)
        }) == 0) as core::ffi::c_int;
        if let Some(arg1) = arg1 {
            prefix = Some(format_single(None, arg1, c.as_deref(), s, wl, pane.get()));
        }
        if s.is_some()
            && let Some(arg0) = arg0.filter(|arg0| !arg0.to_bytes().is_empty())
        {
            command = Some(format_single(None, arg0, c.as_deref(), s, wl, pane.get()));
        }
        window_copy_copy_pipe(
            &mut *wme,
            cs.s,
            prefix.as_deref(),
            command.as_deref(),
            set_paste,
            set_clip,
        );
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_copy_pipe(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    unsafe {
        window_copy_cmd_copy_pipe_no_clear(cs);
        window_copy_clear_selection(cs.wme);
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_copy_pipe_and_cancel(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        window_copy_cmd_copy_pipe_no_clear(cs);
        window_copy_clear_selection(cs.wme);
        WINDOW_COPY_CMD_CANCEL
    }
}
unsafe fn window_copy_cmd_pipe_no_clear(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let c = cs.c.as_deref_mut();
        let s = cs.s;
        let wl = cs.wl;
        let pane = wme.pane_ref().expect("mode has a pane");
        let mut command: Option<CString> = None;
        if s.is_some()
            && let Some(arg0) = {
                let args: &RustArguments = cs.wargs;
                let idx = 0 as u_int;
                args.argument_string(idx)
            }
            .filter(|arg0| !arg0.to_bytes().is_empty())
        {
            command = Some(format_single(None, arg0, c.as_deref(), s, wl, pane.get()));
        }
        window_copy_pipe(&mut *wme, cs.s, command.as_deref());
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_pipe(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    unsafe {
        window_copy_cmd_pipe_no_clear(cs);
        window_copy_clear_selection(cs.wme);
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_pipe_and_cancel(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        window_copy_cmd_pipe_no_clear(cs);
        window_copy_clear_selection(cs.wme);
        WINDOW_COPY_CMD_CANCEL
    }
}
unsafe fn window_copy_cmd_goto_line(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        if let Some(arg0) = {
            let args: &RustArguments = cs.wargs;
            let idx = 0 as u_int;
            args.argument_string(idx)
        }
        .filter(|arg0| !arg0.to_bytes().is_empty())
        {
            window_copy_goto_line(&mut *wme, arg0);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_jump_backward(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        let mut np: u_int = wme.prefix;
        if let Some(arg0) = {
            let args: &RustArguments = cs.wargs;
            let idx = 0 as u_int;
            args.argument_string(idx)
        }
        .filter(|arg0| !arg0.to_bytes().is_empty())
        {
            data.jumptype = WINDOW_COPY_JUMPBACKWARD as core::ffi::c_int;
            data.jumpchar = utf8_fromcstr(arg0).into_iter().next();
            while np != 0 as u_int {
                window_copy_cursor_jump_back(&mut *wme);
                np = np.wrapping_sub(1);
            }
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_jump_forward(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        let mut np: u_int = wme.prefix;
        if let Some(arg0) = {
            let args: &RustArguments = cs.wargs;
            let idx = 0 as u_int;
            args.argument_string(idx)
        }
        .filter(|arg0| !arg0.to_bytes().is_empty())
        {
            data.jumptype = WINDOW_COPY_JUMPFORWARD as core::ffi::c_int;
            data.jumpchar = utf8_fromcstr(arg0).into_iter().next();
            while np != 0 as u_int {
                window_copy_cursor_jump(&mut *wme);
                np = np.wrapping_sub(1);
            }
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_jump_to_backward(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        let mut np: u_int = wme.prefix;
        if let Some(arg0) = {
            let args: &RustArguments = cs.wargs;
            let idx = 0 as u_int;
            args.argument_string(idx)
        }
        .filter(|arg0| !arg0.to_bytes().is_empty())
        {
            data.jumptype = WINDOW_COPY_JUMPTOBACKWARD as core::ffi::c_int;
            data.jumpchar = utf8_fromcstr(arg0).into_iter().next();
            while np != 0 as u_int {
                window_copy_cursor_jump_to_back(&mut *wme);
                np = np.wrapping_sub(1);
            }
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_jump_to_forward(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        let mut np: u_int = wme.prefix;
        if let Some(arg0) = {
            let args: &RustArguments = cs.wargs;
            let idx = 0 as u_int;
            args.argument_string(idx)
        }
        .filter(|arg0| !arg0.to_bytes().is_empty())
        {
            data.jumptype = WINDOW_COPY_JUMPTOFORWARD as core::ffi::c_int;
            data.jumpchar = utf8_fromcstr(arg0).into_iter().next();
            while np != 0 as u_int {
                window_copy_cursor_jump_to(&mut *wme);
                np = np.wrapping_sub(1);
            }
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_jump_to_mark(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        window_copy_jump_to_mark(&mut *wme);
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_next_prompt(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        window_copy_cursor_prompt(&mut *wme, 1 as core::ffi::c_int, {
            let args: &RustArguments = cs.wargs;
            let flag = 'o' as i32 as u_char;
            args.argument_flag_count(flag)
        });
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_previous_prompt(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        window_copy_cursor_prompt(&mut *wme, 0 as core::ffi::c_int, {
            let args: &RustArguments = cs.wargs;
            let flag = 'o' as i32 as u_char;
            args.argument_flag_count(flag)
        });
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_search_backward(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut np = cs.wme.prefix;
        if window_copy_expand_search_string(cs) == 0 {
            return WINDOW_COPY_CMD_NOTHING;
        }
        let wme = &mut *cs.wme;
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        if data.searchstr.is_some() {
            data.searchtype = WINDOW_COPY_SEARCHUP as core::ffi::c_int;
            data.searchregex = 1 as core::ffi::c_int;
            data.timeout = 0 as core::ffi::c_int;
            while np != 0 as u_int {
                window_copy_search_up(&mut *wme, 1 as core::ffi::c_int);
                np = np.wrapping_sub(1);
            }
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_search_backward_text(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut np = cs.wme.prefix;
        if window_copy_expand_search_string(cs) == 0 {
            return WINDOW_COPY_CMD_NOTHING;
        }
        let wme = &mut *cs.wme;
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        if data.searchstr.is_some() {
            data.searchtype = WINDOW_COPY_SEARCHUP as core::ffi::c_int;
            data.searchregex = 0 as core::ffi::c_int;
            data.timeout = 0 as core::ffi::c_int;
            while np != 0 as u_int {
                window_copy_search_up(&mut *wme, 0 as core::ffi::c_int);
                np = np.wrapping_sub(1);
            }
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_search_forward(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut np = cs.wme.prefix;
        if window_copy_expand_search_string(cs) == 0 {
            return WINDOW_COPY_CMD_NOTHING;
        }
        let wme = &mut *cs.wme;
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        if data.searchstr.is_some() {
            data.searchtype = WINDOW_COPY_SEARCHDOWN as core::ffi::c_int;
            data.searchregex = 1 as core::ffi::c_int;
            data.timeout = 0 as core::ffi::c_int;
            while np != 0 as u_int {
                window_copy_search_down(&mut *wme, 1 as core::ffi::c_int);
                np = np.wrapping_sub(1);
            }
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_search_forward_text(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut np = cs.wme.prefix;
        if window_copy_expand_search_string(cs) == 0 {
            return WINDOW_COPY_CMD_NOTHING;
        }
        let wme = &mut *cs.wme;
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        if data.searchstr.is_some() {
            data.searchtype = WINDOW_COPY_SEARCHDOWN as core::ffi::c_int;
            data.searchregex = 0 as core::ffi::c_int;
            data.timeout = 0 as core::ffi::c_int;
            while np != 0 as u_int {
                window_copy_search_down(&mut *wme, 0 as core::ffi::c_int);
                np = np.wrapping_sub(1);
            }
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_search_backward_incremental(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        let arg0 = {
            let args: &RustArguments = cs.wargs;
            let idx = 0 as u_int;
            args.argument_string(idx)
        }
        .unwrap_or(c"");
        let mut action: window_copy_cmd_action = WINDOW_COPY_CMD_NOTHING;
        data.timeout = 0 as core::ffi::c_int;
        log_debug(
            c"%s: %s",
            fmt_args![c"window_copy_cmd_search_backward_incremental", arg0],
        );
        let (prefix, arg0) = match arg0.to_bytes_with_nul().split_first() {
            Some((&prefix, pattern)) if prefix != 0 => (
                prefix as core::ffi::c_char,
                CStr::from_bytes_with_nul(pattern)
                    .expect("the tail of a C string ends at the same NUL"),
            ),
            _ => (0 as core::ffi::c_char, arg0),
        };
        if data.searchx == -(1 as core::ffi::c_int) || data.searchy == -(1 as core::ffi::c_int) {
            data.searchx = data.cx as core::ffi::c_int;
            data.searchy = data.cy as core::ffi::c_int;
            data.searcho = data.oy as core::ffi::c_int;
        } else if data.searchstr.as_deref().is_some_and(|ss| ss != arg0) {
            data.cx = data.searchx as u_int;
            data.cy = data.searchy as u_int;
            data.oy = data.searcho as u_int;
            let row = RustScreen::grid(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
            )
            .history_size()
            .wrapping_add(data.cy)
            .wrapping_sub(data.oy);
            let cx = window_copy_cursor_limit(wme, row, 0);
            wme.state
                .copy_mode_data_mut()
                .expect("copy mode has state")
                .cx = cx;
            action = WINDOW_COPY_CMD_REDRAW;
        }
        if arg0.to_bytes().is_empty() {
            window_copy_clear_marks(&mut *wme);
            return WINDOW_COPY_CMD_REDRAW;
        }
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        match prefix as core::ffi::c_int {
            61 | 45 => {
                data.searchtype = WINDOW_COPY_SEARCHUP as core::ffi::c_int;
                data.searchregex = 0 as core::ffi::c_int;
                data.searchstr = Some(arg0.to_owned());
                if window_copy_search_up(&mut *wme, 0 as core::ffi::c_int) == 0 {
                    window_copy_clear_marks(&mut *wme);
                    return WINDOW_COPY_CMD_REDRAW;
                }
            }
            43 => {
                data.searchtype = WINDOW_COPY_SEARCHDOWN as core::ffi::c_int;
                data.searchregex = 0 as core::ffi::c_int;
                data.searchstr = Some(arg0.to_owned());
                if window_copy_search_down(&mut *wme, 0 as core::ffi::c_int) == 0 {
                    window_copy_clear_marks(&mut *wme);
                    return WINDOW_COPY_CMD_REDRAW;
                }
            }
            _ => {}
        }
        action
    }
}
unsafe fn window_copy_cmd_search_forward_incremental(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        let arg0 = {
            let args: &RustArguments = cs.wargs;
            let idx = 0 as u_int;
            args.argument_string(idx)
        }
        .unwrap_or(c"");
        let mut action: window_copy_cmd_action = WINDOW_COPY_CMD_NOTHING;
        data.timeout = 0 as core::ffi::c_int;
        log_debug(
            c"%s: %s",
            fmt_args![c"window_copy_cmd_search_forward_incremental", arg0],
        );
        let (prefix, arg0) = match arg0.to_bytes_with_nul().split_first() {
            Some((&prefix, pattern)) if prefix != 0 => (
                prefix as core::ffi::c_char,
                CStr::from_bytes_with_nul(pattern)
                    .expect("the tail of a C string ends at the same NUL"),
            ),
            _ => (0 as core::ffi::c_char, arg0),
        };
        if data.searchx == -(1 as core::ffi::c_int) || data.searchy == -(1 as core::ffi::c_int) {
            data.searchx = data.cx as core::ffi::c_int;
            data.searchy = data.cy as core::ffi::c_int;
            data.searcho = data.oy as core::ffi::c_int;
        } else if data.searchstr.as_deref().is_some_and(|ss| ss != arg0) {
            data.cx = data.searchx as u_int;
            data.cy = data.searchy as u_int;
            data.oy = data.searcho as u_int;
            let row = RustScreen::grid(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
            )
            .history_size()
            .wrapping_add(data.cy)
            .wrapping_sub(data.oy);
            let cx = window_copy_cursor_limit(wme, row, 0);
            wme.state
                .copy_mode_data_mut()
                .expect("copy mode has state")
                .cx = cx;
            action = WINDOW_COPY_CMD_REDRAW;
        }
        if arg0.to_bytes().is_empty() {
            window_copy_clear_marks(&mut *wme);
            return WINDOW_COPY_CMD_REDRAW;
        }
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        match prefix as core::ffi::c_int {
            61 | 43 => {
                data.searchtype = WINDOW_COPY_SEARCHDOWN as core::ffi::c_int;
                data.searchregex = 0 as core::ffi::c_int;
                data.searchstr = Some(arg0.to_owned());
                if window_copy_search_down(&mut *wme, 0 as core::ffi::c_int) == 0 {
                    window_copy_clear_marks(&mut *wme);
                    return WINDOW_COPY_CMD_REDRAW;
                }
            }
            45 => {
                data.searchtype = WINDOW_COPY_SEARCHUP as core::ffi::c_int;
                data.searchregex = 0 as core::ffi::c_int;
                data.searchstr = Some(arg0.to_owned());
                if window_copy_search_up(&mut *wme, 0 as core::ffi::c_int) == 0 {
                    window_copy_clear_marks(&mut *wme);
                    return WINDOW_COPY_CMD_REDRAW;
                }
            }
            _ => {}
        }
        action
    }
}
unsafe fn window_copy_cmd_refresh_from_pane(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");

        if data.viewmode != 0 {
            return WINDOW_COPY_CMD_NOTHING;
        }
        let Some(source) = wme.source_pane_ref() else {
            return WINDOW_COPY_CMD_NOTHING;
        };
        let Some(wp) = source.get() else {
            return WINDOW_COPY_CMD_NOTHING;
        };
        let pane = wme.pane_ref().expect("mode has a pane");
        let trim = (source.id() != pane.id()) as core::ffi::c_int;
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        if data.oy
            > RustScreen::grid(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
            )
            .history_size()
        {
            data.oy = RustScreen::grid(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
            )
            .history_size();
        }
        let oy_from_top: u_int = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_sub(data.oy);
        window_copy_free_backing(&mut *data);
        data.backing =
            Some(window_copy_clone_screen(wp.base(), &data.screen.borrow(), false, trim).0);
        if oy_from_top
            <= RustScreen::grid(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
            )
            .history_size()
        {
            data.oy = RustScreen::grid(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
            )
            .history_size()
            .wrapping_sub(oy_from_top);
        } else {
            data.cy = 0 as u_int;
            data.oy = RustScreen::grid(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
            )
            .history_size();
        }
        window_copy_size_changed(&mut *wme);
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_recentre_top_bottom(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let wme = &mut *cs.wme;
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        let cy: u_int = data.cy;

        let sy: u_int = RustScreen::grid(&data.screen.borrow())
            .height()
            .wrapping_sub(1 as u_int);
        let sm: u_int = sy.wrapping_div(2 as u_int);

        let backing_row: u_int = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_add(cy)
        .wrapping_sub(data.oy);
        if data.recentre_line != backing_row {
            data.recentre_state = RECENTRE_MIDDLE;
            data.recentre_line = backing_row;
        }
        let target: window_copy_line_position = match data.recentre_state {
            RECENTRE_MIDDLE => {
                data.recentre_state = RECENTRE_TOP;
                MIDDLE
            }
            RECENTRE_TOP => {
                data.recentre_state = RECENTRE_BOTTOM;
                TOP
            }
            _ => {
                data.recentre_state = RECENTRE_MIDDLE;
                BOTTOM
            }
        };
        let oy: u_int = data.oy;
        match target {
            MIDDLE => {
                if cy < sm {
                    window_copy_scroll_down(&mut *wme, sm.wrapping_sub(cy));
                } else if cy > sm {
                    window_copy_scroll_up(&mut *wme, cy.wrapping_sub(sm));
                }
                let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
                if data.oy != oy {
                    data.cy = cy.wrapping_add(data.oy.wrapping_sub(oy));
                }
            }
            TOP => {
                window_copy_scroll_up(&mut *wme, cy);
                let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
                data.cy = cy.wrapping_sub(oy.wrapping_sub(data.oy));
            }
            BOTTOM => {
                window_copy_scroll_down(&mut *wme, sy.wrapping_sub(cy));
                let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
                data.cy = cy.wrapping_add(data.oy.wrapping_sub(oy));
            }
            _ => {}
        }
        window_copy_update_selection(&mut *wme, 0 as core::ffi::c_int, 0 as core::ffi::c_int);
        WINDOW_COPY_CMD_REDRAW
    }
}
static window_copy_cmd_table: [window_copy_cmd_entry; 93] = {
    [
        window_copy_cmd_entry {
            command: c"append-selection",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_append_selection),
        },
        window_copy_cmd_entry {
            command: c"append-selection-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_append_selection_and_cancel),
        },
        window_copy_cmd_entry {
            command: c"back-to-indentation",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_back_to_indentation),
        },
        window_copy_cmd_entry {
            command: c"begin-selection",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_begin_selection),
        },
        window_copy_cmd_entry {
            command: c"bottom-line",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_bottom_line),
        },
        window_copy_cmd_entry {
            command: c"cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_cancel),
        },
        window_copy_cmd_entry {
            command: c"clear-selection",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_clear_selection),
        },
        window_copy_cmd_entry {
            command: c"copy-end-of-line",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"CP",
                lower: 0 as core::ffi::c_int,
                upper: 1 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_end_of_line),
        },
        window_copy_cmd_entry {
            command: c"copy-end-of-line-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"CP",
                lower: 0 as core::ffi::c_int,
                upper: 1 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_end_of_line_and_cancel),
        },
        window_copy_cmd_entry {
            command: c"copy-pipe-end-of-line",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"CP",
                lower: 0 as core::ffi::c_int,
                upper: 2 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_pipe_end_of_line),
        },
        window_copy_cmd_entry {
            command: c"copy-pipe-end-of-line-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"CP",
                lower: 0 as core::ffi::c_int,
                upper: 2 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_pipe_end_of_line_and_cancel),
        },
        window_copy_cmd_entry {
            command: c"copy-line",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"CP",
                lower: 0 as core::ffi::c_int,
                upper: 1 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_line),
        },
        window_copy_cmd_entry {
            command: c"copy-line-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"CP",
                lower: 0 as core::ffi::c_int,
                upper: 1 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_line_and_cancel),
        },
        window_copy_cmd_entry {
            command: c"copy-pipe-line",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"CP",
                lower: 0 as core::ffi::c_int,
                upper: 2 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_pipe_line),
        },
        window_copy_cmd_entry {
            command: c"copy-pipe-line-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"CP",
                lower: 0 as core::ffi::c_int,
                upper: 2 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_pipe_line_and_cancel),
        },
        window_copy_cmd_entry {
            command: c"copy-pipe-no-clear",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"CP",
                lower: 0 as core::ffi::c_int,
                upper: 2 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_NEVER,
            f: Some(window_copy_cmd_copy_pipe_no_clear),
        },
        window_copy_cmd_entry {
            command: c"copy-pipe",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"CP",
                lower: 0 as core::ffi::c_int,
                upper: 2 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_pipe),
        },
        window_copy_cmd_entry {
            command: c"copy-pipe-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"CP",
                lower: 0 as core::ffi::c_int,
                upper: 2 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_pipe_and_cancel),
        },
        window_copy_cmd_entry {
            command: c"copy-selection-no-clear",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"CP",
                lower: 0 as core::ffi::c_int,
                upper: 1 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_NEVER,
            f: Some(window_copy_cmd_copy_selection_no_clear),
        },
        window_copy_cmd_entry {
            command: c"copy-selection",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"CP",
                lower: 0 as core::ffi::c_int,
                upper: 1 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_selection),
        },
        window_copy_cmd_entry {
            command: c"copy-selection-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"CP",
                lower: 0 as core::ffi::c_int,
                upper: 1 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_selection_and_cancel),
        },
        window_copy_cmd_entry {
            command: c"cursor-down",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_cursor_down),
        },
        window_copy_cmd_entry {
            command: c"cursor-down-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_cursor_down_and_cancel),
        },
        window_copy_cmd_entry {
            command: c"cursor-left",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_cursor_left),
        },
        window_copy_cmd_entry {
            command: c"cursor-right",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_cursor_right),
        },
        window_copy_cmd_entry {
            command: c"cursor-up",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_cursor_up),
        },
        window_copy_cmd_entry {
            command: c"cursor-centre-vertical",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_centre_vertical),
        },
        window_copy_cmd_entry {
            command: c"cursor-centre-horizontal",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_centre_horizontal),
        },
        window_copy_cmd_entry {
            command: c"end-of-line",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_end_of_line),
        },
        window_copy_cmd_entry {
            command: c"goto-line",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 1 as core::ffi::c_int,
                upper: 1 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_goto_line),
        },
        window_copy_cmd_entry {
            command: c"halfpage-down",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_halfpage_down),
        },
        window_copy_cmd_entry {
            command: c"halfpage-down-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_halfpage_down_and_cancel),
        },
        window_copy_cmd_entry {
            command: c"halfpage-up",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_halfpage_up),
        },
        window_copy_cmd_entry {
            command: c"history-bottom",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_history_bottom),
        },
        window_copy_cmd_entry {
            command: c"history-top",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_history_top),
        },
        window_copy_cmd_entry {
            command: c"jump-again",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_jump_again),
        },
        window_copy_cmd_entry {
            command: c"jump-backward",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 1 as core::ffi::c_int,
                upper: 1 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_jump_backward),
        },
        window_copy_cmd_entry {
            command: c"jump-forward",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 1 as core::ffi::c_int,
                upper: 1 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_jump_forward),
        },
        window_copy_cmd_entry {
            command: c"jump-reverse",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_jump_reverse),
        },
        window_copy_cmd_entry {
            command: c"jump-to-backward",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 1 as core::ffi::c_int,
                upper: 1 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_jump_to_backward),
        },
        window_copy_cmd_entry {
            command: c"jump-to-forward",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 1 as core::ffi::c_int,
                upper: 1 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_jump_to_forward),
        },
        window_copy_cmd_entry {
            command: c"jump-to-mark",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_jump_to_mark),
        },
        window_copy_cmd_entry {
            command: c"next-prompt",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"o",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_next_prompt),
        },
        window_copy_cmd_entry {
            command: c"previous-prompt",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"o",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_previous_prompt),
        },
        window_copy_cmd_entry {
            command: c"middle-line",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_middle_line),
        },
        window_copy_cmd_entry {
            command: c"next-matching-bracket",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_next_matching_bracket),
        },
        window_copy_cmd_entry {
            command: c"next-paragraph",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_next_paragraph),
        },
        window_copy_cmd_entry {
            command: c"next-space",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_next_space),
        },
        window_copy_cmd_entry {
            command: c"next-space-end",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_next_space_end),
        },
        window_copy_cmd_entry {
            command: c"next-word",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_next_word),
        },
        window_copy_cmd_entry {
            command: c"next-word-end",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_next_word_end),
        },
        window_copy_cmd_entry {
            command: c"other-end",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_other_end),
        },
        window_copy_cmd_entry {
            command: c"page-down",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_page_down),
        },
        window_copy_cmd_entry {
            command: c"page-down-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_page_down_and_cancel),
        },
        window_copy_cmd_entry {
            command: c"page-up",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_page_up),
        },
        window_copy_cmd_entry {
            command: c"pipe-no-clear",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 1 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_NEVER,
            f: Some(window_copy_cmd_pipe_no_clear),
        },
        window_copy_cmd_entry {
            command: c"pipe",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 1 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_pipe),
        },
        window_copy_cmd_entry {
            command: c"pipe-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 1 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_pipe_and_cancel),
        },
        window_copy_cmd_entry {
            command: c"previous-matching-bracket",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_previous_matching_bracket),
        },
        window_copy_cmd_entry {
            command: c"previous-paragraph",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_previous_paragraph),
        },
        window_copy_cmd_entry {
            command: c"previous-space",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_previous_space),
        },
        window_copy_cmd_entry {
            command: c"previous-word",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_previous_word),
        },
        window_copy_cmd_entry {
            command: c"recentre-top-bottom",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_recentre_top_bottom),
        },
        window_copy_cmd_entry {
            command: c"rectangle-on",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_rectangle_on),
        },
        window_copy_cmd_entry {
            command: c"rectangle-off",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_rectangle_off),
        },
        window_copy_cmd_entry {
            command: c"rectangle-toggle",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_rectangle_toggle),
        },
        window_copy_cmd_entry {
            command: c"refresh-from-pane",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_refresh_from_pane),
        },
        window_copy_cmd_entry {
            command: c"scroll-bottom",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_scroll_bottom),
        },
        window_copy_cmd_entry {
            command: c"scroll-down",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_scroll_down),
        },
        window_copy_cmd_entry {
            command: c"scroll-down-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_scroll_down_and_cancel),
        },
        window_copy_cmd_entry {
            command: c"scroll-exit-on",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: 0,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_scroll_exit_on),
        },
        window_copy_cmd_entry {
            command: c"scroll-exit-off",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: 0,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_scroll_exit_off),
        },
        window_copy_cmd_entry {
            command: c"scroll-exit-toggle",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: 0,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_scroll_exit_toggle),
        },
        window_copy_cmd_entry {
            command: c"scroll-middle",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_scroll_middle),
        },
        window_copy_cmd_entry {
            command: c"scroll-to-mouse",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"e",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_scroll_to_mouse),
        },
        window_copy_cmd_entry {
            command: c"scroll-top",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_scroll_top),
        },
        window_copy_cmd_entry {
            command: c"scroll-up",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_scroll_up),
        },
        window_copy_cmd_entry {
            command: c"search-again",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_search_again),
        },
        window_copy_cmd_entry {
            command: c"search-backward",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 1 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_search_backward),
        },
        window_copy_cmd_entry {
            command: c"search-backward-text",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 1 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_search_backward_text),
        },
        window_copy_cmd_entry {
            command: c"search-backward-incremental",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 1 as core::ffi::c_int,
                upper: 1 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_search_backward_incremental),
        },
        window_copy_cmd_entry {
            command: c"search-forward",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 1 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_search_forward),
        },
        window_copy_cmd_entry {
            command: c"search-forward-text",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 1 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_search_forward_text),
        },
        window_copy_cmd_entry {
            command: c"search-forward-incremental",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 1 as core::ffi::c_int,
                upper: 1 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_search_forward_incremental),
        },
        window_copy_cmd_entry {
            command: c"search-reverse",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_search_reverse),
        },
        window_copy_cmd_entry {
            command: c"select-line",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_select_line),
        },
        window_copy_cmd_entry {
            command: c"select-word",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_select_word),
        },
        window_copy_cmd_entry {
            command: c"selection-mode",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 1 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_selection_mode),
        },
        window_copy_cmd_entry {
            command: c"set-mark",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_set_mark),
        },
        window_copy_cmd_entry {
            command: c"start-of-line",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_start_of_line),
        },
        window_copy_cmd_entry {
            command: c"stop-selection",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: 0 as core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_stop_selection),
        },
        window_copy_cmd_entry {
            command: c"toggle-position",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_NEVER,
            f: Some(window_copy_cmd_toggle_position),
        },
        window_copy_cmd_entry {
            command: c"top-line",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as core::ffi::c_int,
                upper: 0 as core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_top_line),
        },
    ]
};
pub const WINDOW_COPY_CMD_FLAG_READONLY: core::ffi::c_int = 0x1 as core::ffi::c_int;
pub(crate) unsafe fn window_copy_command(
    wme: &mut window_mode_entry,
    mut c: Option<&mut client>,
    s: Option<&session>,
    wl: Option<&winlink>,
    args: &RustArguments,
    m: Option<&mut mouse_event>,
) {
    unsafe {
        let m = m.as_deref();

        let mut action: window_copy_cmd_action;
        let mut clear: window_copy_cmd_clear = WINDOW_COPY_CMD_CLEAR_NEVER;
        let mut i: u_int;
        let count: u_int = args.argument_count();
        let keys: core::ffi::c_int;
        let flags: core::ffi::c_int;
        let mut error: Option<CString> = None;
        if count == 0 as u_int {
            return;
        }
        let command = {
            let idx = 0 as u_int;
            args.argument_string(idx)
        }
        .expect("the count is not zero");
        if let Some(m) = m
            && m.valid != 0
            && !(m.b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_UP as u_int
                || m.b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_DOWN as u_int)
        {
            window_copy_move_mouse(m);
        }
        action = WINDOW_COPY_CMD_NOTHING;
        i = 0 as u_int;
        while (i as usize)
            < size_of::<[window_copy_cmd_entry; 93]>()
                .wrapping_div(size_of::<window_copy_cmd_entry>())
        {
            if window_copy_cmd_table[i as usize].command == command {
                flags = window_copy_cmd_table[i as usize].flags;
                if c.as_deref()
                    .is_some_and(|c| c.flags & CLIENT_READONLY as uint64_t != 0)
                    && !flags & WINDOW_COPY_CMD_FLAG_READONLY != 0
                {
                    status_message_set(
                        c.as_deref_mut(),
                        -(1 as core::ffi::c_int),
                        1 as core::ffi::c_int,
                        0 as core::ffi::c_int,
                        0 as core::ffi::c_int,
                        c"client is read-only",
                        fmt_args![],
                    );
                    return;
                }
                let Some(wargs) = args_parse(
                    &window_copy_cmd_table[i as usize].args,
                    args.argument_values(),
                    &mut error,
                ) else {
                    break;
                };
                let mut cs = window_copy_cmd_state {
                    wme,
                    args,
                    wargs: &wargs,
                    m,
                    c,
                    s,
                    wl,
                };
                clear = window_copy_cmd_table[i as usize].clear;
                action = window_copy_cmd_table[i as usize]
                    .f
                    .expect("non-null function pointer")(&mut cs);
                break;
            } else {
                i = i.wrapping_add(1);
            }
        }
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        if !command.to_bytes().starts_with(b"search-") && !data.searchmark.is_empty() {
            let pane = wme.pane_ref().expect("mode has a pane");
            let window = pane.window().expect("mode pane has a window");
            keys = window.options().number(c"mode-keys") as core::ffi::c_int;
            if clear as core::ffi::c_uint
                == WINDOW_COPY_CMD_CLEAR_EMACS_ONLY as core::ffi::c_int as core::ffi::c_uint
                && keys == MODEKEY_VI
            {
                clear = WINDOW_COPY_CMD_CLEAR_NEVER;
            }
            if clear as core::ffi::c_uint
                != WINDOW_COPY_CMD_CLEAR_NEVER as core::ffi::c_int as core::ffi::c_uint
            {
                window_copy_clear_marks(wme);
                let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
                data.searchy = -(1 as core::ffi::c_int);
                data.searchx = data.searchy;
            }
            if action as core::ffi::c_uint
                == WINDOW_COPY_CMD_NOTHING as core::ffi::c_int as core::ffi::c_uint
            {
                action = WINDOW_COPY_CMD_REDRAW;
            }
        }
        wme.prefix = 1 as u_int;
        if action as core::ffi::c_uint
            == WINDOW_COPY_CMD_CANCEL as core::ffi::c_int as core::ffi::c_uint
        {
            let mut pane = wme.pane_ref().expect("mode has a pane");
            (pane.get_mut().expect("mode pane is live")).reset_mode();
        } else if action as core::ffi::c_uint
            == WINDOW_COPY_CMD_REDRAW as core::ffi::c_int as core::ffi::c_uint
        {
            window_copy_redraw_screen(wme);
        } else if action as core::ffi::c_uint
            == WINDOW_COPY_CMD_NOTHING as core::ffi::c_int as core::ffi::c_uint
        {
            window_copy_redraw_lines(wme, 0 as u_int, 1 as u_int);
        }
    }
}
unsafe fn window_copy_scroll_to(
    wme: &mut window_mode_entry,
    px: u_int,
    py: u_int,
    no_redraw: core::ffi::c_int,
) {
    unsafe {
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        let gd = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        );
        let offset: u_int;
        let gap: u_int;
        data.cx = px;
        if py >= gd.history_size().wrapping_sub(data.oy)
            && py < gd.history_size().wrapping_sub(data.oy).wrapping_add(gd.height())
        {
            data.cy = py.wrapping_sub(gd.history_size().wrapping_sub(data.oy));
        } else {
            gap = gd.height().wrapping_div(4 as u_int);
            if py < gd.height() {
                offset = 0 as u_int;
                data.cy = py;
            } else if py > gd.history_size().wrapping_add(gd.height()).wrapping_sub(gap) {
                offset = gd.history_size();
                data.cy = py.wrapping_sub(gd.history_size());
            } else {
                offset = py.wrapping_add(gap).wrapping_sub(gd.height());
                data.cy = py.wrapping_sub(offset);
            }
            data.oy = gd.history_size().wrapping_sub(offset);
        }
        if no_redraw == 0 && !data.searchmark.is_empty() && data.timeout == 0 {
            let regex = data.searchregex;
            window_copy_search_marks(wme, None, regex, 1 as core::ffi::c_int);
        }
        window_copy_update_selection(wme, 1 as core::ffi::c_int, 0 as core::ffi::c_int);
        if no_redraw == 0 {
            window_copy_redraw_screen(wme);
        }
    }
}
/// Whether the cell at `px`, `py` in `gd` is the one at `spx` on the first
/// line of `sgd`, with `cis` asking for a case-insensitive match of a
/// single-byte character. A tab in the search string stands for a cell the
/// grid padded out to the next tab stop, however wide that padding made it.
fn window_copy_search_compare(
    gd: &grid,
    px: u_int,
    py: u_int,
    sgd: &grid,
    spx: u_int,
    cis: core::ffi::c_int,
) -> bool {
    let gc = gd.cell(px, py);
    let sgc = sgd.cell(spx, 0);
    let ud = &gc.data;
    let sud = &sgc.data;
    if sud.data[0] == b'\t' && sud.size == 1 && gc.flags as core::ffi::c_int & GRID_FLAG_TAB != 0 {
        return true;
    }
    if ud.size != sud.size || ud.width != sud.width {
        return false;
    }
    if cis != 0 && ud.size == 1 {
        return tolower(ud.data[0]) == sud.data[0];
    }
    ud.data[..ud.size as usize] == sud.data[..sud.size as usize]
}
fn window_copy_search_lr(
    gd: &grid,
    sgd: &grid,
    py: u_int,
    first: u_int,
    last: u_int,
    cis: core::ffi::c_int,
) -> Option<u_int> {
    {
        let mut ax: u_int;
        let mut bx: u_int;
        let mut px: u_int;
        let mut pywrap: u_int;

        let mut padding: u_int;
        let mut gc;
        let endline: u_int = gd.history_size().wrapping_add(gd.height()).wrapping_sub(1 as u_int);
        ax = first;
        while ax < last {
            padding = 0 as u_int;
            bx = 0 as u_int;
            while bx < sgd.width() {
                px = ax.wrapping_add(bx).wrapping_add(padding);
                pywrap = py;
                while px >= gd.width() && pywrap < endline {
                    let gl = (gd).peek_line(pywrap).expect("a line inside the grid");
                    if !gl.flags & GRID_LINE_WRAPPED != 0 {
                        break;
                    }
                    px = px.wrapping_sub(gd.width());
                    pywrap = pywrap.wrapping_add(1);
                }
                if px.wrapping_sub(padding) >= gd.width() {
                    break;
                }
                gc = gd.cell(px, pywrap);
                if gc.flags as core::ffi::c_int & GRID_FLAG_TAB != 0 {
                    padding = padding.wrapping_add(
                        (gc.data.width as core::ffi::c_int - 1 as core::ffi::c_int) as u_int,
                    );
                }
                if !window_copy_search_compare(gd, px, pywrap, sgd, bx, cis) {
                    break;
                }
                bx = bx.wrapping_add(1);
            }
            if bx == sgd.width() {
                return Some(ax);
            }
            ax = ax.wrapping_add(1);
        }
        None
    }
}
fn window_copy_search_rl(
    gd: &grid,
    sgd: &grid,
    py: u_int,
    first: u_int,
    last: u_int,
    cis: core::ffi::c_int,
) -> Option<u_int> {
    let mut ax: u_int;
    let mut bx: u_int;
    let mut px: u_int;
    let mut pywrap: u_int;

    let mut padding: u_int;
    let mut gc;
    let endline: u_int = gd.history_size().wrapping_add(gd.height()).wrapping_sub(1 as u_int);
    ax = last;
    while ax > first {
        padding = 0 as u_int;
        bx = 0 as u_int;
        while bx < sgd.width() {
            px = ax
                .wrapping_sub(1 as u_int)
                .wrapping_add(bx)
                .wrapping_add(padding);
            pywrap = py;
            while px >= gd.width() && pywrap < endline {
                let gl = (gd).peek_line(pywrap).expect("a line inside the grid");
                if !gl.flags & GRID_LINE_WRAPPED != 0 {
                    break;
                }
                px = px.wrapping_sub(gd.width());
                pywrap = pywrap.wrapping_add(1);
            }
            if px.wrapping_sub(padding) >= gd.width() {
                break;
            }
            gc = gd.cell(px, pywrap);
            if gc.flags as core::ffi::c_int & GRID_FLAG_TAB != 0 {
                padding = padding.wrapping_add(
                    (gc.data.width as core::ffi::c_int - 1 as core::ffi::c_int) as u_int,
                );
            }
            if !window_copy_search_compare(gd, px, pywrap, sgd, bx, cis) {
                break;
            }
            bx = bx.wrapping_add(1);
        }
        if bx == sgd.width() {
            return Some(ax.wrapping_sub(1 as u_int));
        }
        ax = ax.wrapping_sub(1);
    }
    None
}
unsafe fn window_copy_search_lr_regex(
    gd: &grid,
    py: u_int,
    first: u_int,
    last: u_int,
    reg: &CompiledRegex,
) -> Option<(u_int, u_int)> {
    unsafe {
        let ppx: u_int;
        let mut psx: u_int;
        let mut eflags: core::ffi::c_int = 0 as core::ffi::c_int;

        let mut foundx: u_int;
        let mut foundy: u_int;
        let mut len: u_int;
        let mut pywrap: u_int;
        if first >= last {
            return None;
        }
        if first != 0 as u_int {
            eflags |= REG_NOTBOL;
        }
        let mut buf: Vec<u8> = vec![b'\0'];
        window_copy_stringify(gd, py, first, gd.width(), &mut buf);
        len = gd.width().wrapping_sub(first);
        let endline: u_int = gd.history_size().wrapping_add(gd.height()).wrapping_sub(1 as u_int);
        pywrap = py;
        while pywrap < endline && len < WINDOW_COPY_SEARCH_MAX_LINE as u_int {
            let gl = (gd).line_info(pywrap);
            if !gl.flags & GRID_LINE_WRAPPED != 0 {
                break;
            }
            pywrap = pywrap.wrapping_add(1);
            window_copy_stringify(gd, pywrap, 0 as u_int, gd.width(), &mut buf);
            len = len.wrapping_add(gd.width());
        }
        let text = CStr::from_bytes_until_nul(&buf).expect("a terminated search line");
        if let Some([regmatch]) = reg
            .captures::<1>(text, 0, eflags)
            .filter(|[matched]| matched.rm_so != matched.rm_eo)
        {
            foundx = first;
            foundy = py;
            window_copy_cstrtocellpos(
                gd,
                len,
                &mut foundx,
                &mut foundy,
                &text[regmatch.rm_so as usize..],
            );
            if foundy == py && foundx < last {
                ppx = foundx;
                len = len.wrapping_sub(foundx.wrapping_sub(first));
                window_copy_cstrtocellpos(
                    gd,
                    len,
                    &mut foundx,
                    &mut foundy,
                    &text[regmatch.rm_eo as usize..],
                );
                psx = foundx;
                while foundy > py {
                    psx = psx.wrapping_add(gd.width());
                    foundy = foundy.wrapping_sub(1);
                }
                psx = psx.wrapping_sub(ppx);
                return Some((ppx, psx));
            }
        }
        None
    }
}
unsafe fn window_copy_search_rl_regex(
    gd: &grid,
    py: u_int,
    first: u_int,
    last: u_int,
    reg: &CompiledRegex,
) -> Option<(u_int, u_int)> {
    unsafe {
        let mut eflags: core::ffi::c_int = 0 as core::ffi::c_int;

        let mut len: u_int;
        let mut pywrap: u_int;
        if first != 0 as u_int {
            eflags |= REG_NOTBOL;
        }
        let mut buf: Vec<u8> = vec![b'\0'];
        window_copy_stringify(gd, py, first, gd.width(), &mut buf);
        len = gd.width().wrapping_sub(first);
        let endline: u_int = gd.history_size().wrapping_add(gd.height()).wrapping_sub(1 as u_int);
        pywrap = py;
        while pywrap < endline && len < WINDOW_COPY_SEARCH_MAX_LINE as u_int {
            let gl = (gd).line_info(pywrap);
            if !gl.flags & GRID_LINE_WRAPPED != 0 {
                break;
            }
            pywrap = pywrap.wrapping_add(1);
            window_copy_stringify(gd, pywrap, 0 as u_int, gd.width(), &mut buf);
            len = len.wrapping_add(gd.width());
        }
        window_copy_last_regex(
            gd,
            py,
            first,
            last,
            len,
            CStr::from_bytes_until_nul(&buf).expect("a terminated search line"),
            (reg, eflags),
        )
    }
}
unsafe fn window_copy_last_regex(
    gd: &grid,
    py: u_int,
    first: u_int,
    last: u_int,
    mut len: u_int,
    buf: &CStr,
    regex: (&CompiledRegex, core::ffi::c_int),
) -> Option<(u_int, u_int)> {
    unsafe {
        let (preg, eflags) = regex;
        let ppx: u_int;
        let mut psx: u_int;
        let mut foundx: u_int;
        let mut foundy: u_int;
        let mut oldx: u_int;
        let mut px: u_int = 0 as u_int;
        let mut savepx: u_int = 0;
        let mut savesx: u_int = 0 as u_int;
        foundx = first;
        foundy = py;
        oldx = first;
        while let Some([regmatch]) = preg.captures::<1>(buf, px as usize, eflags) {
            if regmatch.rm_so == regmatch.rm_eo {
                break;
            }
            window_copy_cstrtocellpos(
                gd,
                len,
                &mut foundx,
                &mut foundy,
                &buf[px as usize + regmatch.rm_so as usize..],
            );
            if foundy > py || foundx >= last {
                break;
            }
            len = len.wrapping_sub(foundx.wrapping_sub(oldx));
            savepx = foundx;
            window_copy_cstrtocellpos(
                gd,
                len,
                &mut foundx,
                &mut foundy,
                &buf[px as usize + regmatch.rm_eo as usize..],
            );
            if foundy > py || foundx >= last {
                ppx = savepx;
                psx = foundx;
                while foundy > py {
                    psx = psx.wrapping_add(gd.width());
                    foundy = foundy.wrapping_sub(1);
                }
                psx = psx.wrapping_sub(ppx);
                return Some((ppx, psx));
            } else {
                savesx = foundx.wrapping_sub(savepx);
                len = len.wrapping_sub(savesx);
                oldx = foundx;
            }
            px = px.wrapping_add(regmatch.rm_eo as u_int);
        }
        if savesx > 0 as u_int {
            Some((savepx, savesx))
        } else {
            None
        }
    }
}
unsafe fn window_copy_stringify(
    gd: &grid,
    py: u_int,
    first: u_int,
    last: u_int,
    buf: &mut Vec<u8>,
) {
    unsafe {
        let mut ax: u_int;

        let gl: Option<crate::grid::GridLineInfo> = (gd).peek_line(py);
        if gl.is_none() {
            return;
        }
        buf.pop();
        ax = first;
        while ax < last {
            buf.extend_from_slice(&gd.cell_bytes(ax, py));
            ax = ax.wrapping_add(1);
        }
        buf.push(b'\0');
    }
}
unsafe fn window_copy_cstrtocellpos(
    gd: &grid,
    mut ncells: u_int,
    ppx: &mut u_int,
    ppy: &mut u_int,
    str: &CStr,
) {
    unsafe {
        let mut cell: u_int;
        let mut ccell: u_int;
        let mut px: u_int;
        let mut pywrap: u_int;
        let mut pos: u_int;
        let mut match_0: core::ffi::c_int;
        let mut gl: Option<crate::grid::GridLineInfo>;
        let mut cells: Vec<window_copy_search_cell> = Vec::with_capacity(ncells as usize);
        cell = 0 as u_int;
        px = *ppx;
        pywrap = *ppy;
        gl = (gd).peek_line(pywrap);
        if gl.is_none() {
            return;
        }
        while cell < ncells {
            cells.push(window_copy_search_cell {
                d: gd.cell_bytes(px, pywrap),
            });
            cell = cell.wrapping_add(1);
            px = px.wrapping_add(1);
            if !(px == gd.width()) {
                continue;
            }
            px = 0 as u_int;
            pywrap = pywrap.wrapping_add(1);
            gl = (gd).peek_line(pywrap);
            if gl.is_none() {
                break;
            }
        }
        ncells = cells.len() as u_int;
        cell = 0 as u_int;
        let str = str.to_bytes();
        while cell < ncells {
            ccell = cell;
            pos = 0 as u_int;
            match_0 = 1 as core::ffi::c_int;
            while ccell < ncells {
                let Some(&ch) = str.get(pos as usize) else {
                    match_0 = 0 as core::ffi::c_int;
                    break;
                };
                let d = &cells[ccell as usize].d;
                if d.len() == 1 {
                    if ch != d[0] {
                        match_0 = 0 as core::ffi::c_int;
                        break;
                    }
                    pos = pos.wrapping_add(1);
                } else {
                    let dlen = d.len().min(str.len() - pos as usize);
                    if str[pos as usize..pos as usize + dlen] != d[..dlen] {
                        match_0 = 0 as core::ffi::c_int;
                        break;
                    }
                    pos = pos.wrapping_add(dlen as u_int);
                }
                ccell = ccell.wrapping_add(1);
            }
            if match_0 != 0 {
                break;
            }
            cell = cell.wrapping_add(1);
        }
        px = (*ppx).wrapping_add(cell);
        pywrap = *ppy;
        while px >= gd.width() {
            px = px.wrapping_sub(gd.width());
            pywrap = pywrap.wrapping_add(1);
        }
        *ppx = px;
        *ppy = pywrap;
    }
}
fn window_copy_move_left(
    s: &RustScreen,
    fx: &mut u_int,
    fy: &mut u_int,
    wrapflag: core::ffi::c_int,
) {
    {
        if *fx == 0 as u_int {
            if *fy == 0 as u_int {
                if wrapflag != 0 {
                    *fx = RustScreen::grid(s).width().wrapping_sub(1 as u_int);
                    *fy = RustScreen::grid(s)
                        .history_size()
                        .wrapping_add(RustScreen::grid(s).height())
                        .wrapping_sub(1 as u_int);
                }
                return;
            }
            *fx = RustScreen::grid(s).width().wrapping_sub(1 as u_int);
            *fy = (*fy).wrapping_sub(1 as u_int);
        } else {
            *fx = (*fx).wrapping_sub(1 as u_int);
        };
    }
}
fn window_copy_move_right(
    s: &RustScreen,
    fx: &mut u_int,
    fy: &mut u_int,
    wrapflag: core::ffi::c_int,
) {
    {
        if *fx == RustScreen::grid(s).width().wrapping_sub(1 as u_int) {
            if *fy
                == RustScreen::grid(s)
                    .history_size()
                    .wrapping_add(RustScreen::grid(s).height())
                    .wrapping_sub(1 as u_int)
            {
                if wrapflag != 0 {
                    *fx = 0 as u_int;
                    *fy = 0 as u_int;
                }
                return;
            }
            *fx = 0 as u_int;
            *fy = (*fy).wrapping_add(1 as u_int);
        } else {
            *fx = (*fx).wrapping_add(1 as u_int);
        };
    }
}
fn window_copy_is_lowercase(s: &CStr) -> core::ffi::c_int {
    for &byte in s.to_bytes() {
        if byte != tolower(byte) {
            return 0 as core::ffi::c_int;
        }
    }
    1 as core::ffi::c_int
}
unsafe fn window_copy_search_back_overlap(
    gd: &grid,
    preg: &CompiledRegex,
    ppx: &mut u_int,
    psx: &mut u_int,
    ppy: &mut u_int,
    endline: u_int,
) {
    unsafe {
        let mut endx: u_int;
        let mut endy: u_int;
        let mut oldendx: u_int;
        let mut oldendy: u_int;
        let mut px: u_int;
        let mut py: u_int;
        let mut sx: u_int;
        let mut found: core::ffi::c_int = 1 as core::ffi::c_int;
        oldendx = (*ppx).wrapping_add(*psx);
        oldendy = (*ppy).wrapping_sub(1 as u_int);
        while oldendx > gd.width().wrapping_sub(1 as u_int) {
            oldendx = oldendx.wrapping_sub(gd.width());
            oldendy = oldendy.wrapping_add(1);
        }
        endx = oldendx;
        endy = oldendy;
        px = *ppx;
        py = *ppy;
        while found != 0
            && px == 0 as u_int
            && py.wrapping_sub(1 as u_int) > endline
            && (gd).line_info(py.wrapping_sub(2 as u_int)).flags & GRID_LINE_WRAPPED != 0
            && endx == oldendx
            && endy == oldendy
        {
            py = py.wrapping_sub(1);
            let width = gd.width();
            (px, sx, found) = match window_copy_search_rl_regex(
                gd,
                py.wrapping_sub(1 as u_int),
                0 as u_int,
                width,
                preg,
            ) {
                Some((found_px, found_sx)) => (found_px, found_sx, 1),
                None => (0 as u_int, 0 as u_int, 0),
            };
            if found != 0 {
                endx = px.wrapping_add(sx);
                endy = py.wrapping_sub(1 as u_int);
                while endx > gd.width().wrapping_sub(1 as u_int) {
                    endx = endx.wrapping_sub(gd.width());
                    endy = endy.wrapping_add(1);
                }
                if endx == oldendx && endy == oldendy {
                    *ppx = px;
                    *ppy = py;
                }
            }
        }
    }
}
unsafe fn window_copy_search_position(
    gd: &grid,
    sgd: &grid,
    mut fx: u_int,
    fy: u_int,
    endline: u_int,
    search: (
        core::ffi::c_int,
        core::ffi::c_int,
        core::ffi::c_int,
        core::ffi::c_int,
    ),
) -> Option<(u_int, u_int)> {
    unsafe {
        let (cis, wrap, direction, regex) = search;
        let gd_sx = gd.width();
        let mut i: u_int;
        let mut px: u_int = 0;
        let mut sx: u_int;
        let mut found: core::ffi::c_int = 0 as core::ffi::c_int;
        let mut cflags: core::ffi::c_int = REG_EXTENDED;
        let mut reg: Option<CompiledRegex> = None;
        if regex != 0 {
            let mut sbuf: Vec<u8> = vec![b'\0'];
            window_copy_stringify(sgd, 0 as u_int, 0 as u_int, sgd.width(), &mut sbuf);
            if cis != 0 {
                cflags |= REG_ICASE;
            }
            let pattern = CStr::from_bytes_until_nul(&sbuf).expect("a terminated search pattern");
            let compiled = CompiledRegex::compile(pattern, cflags)?;
            reg = Some(compiled);
        }
        if direction != 0 {
            i = fy;
            while i <= endline {
                if regex != 0 {
                    (px, _, found) = match window_copy_search_lr_regex(
                        gd,
                        i,
                        fx,
                        gd_sx,
                        reg.as_ref().expect("a compiled search pattern"),
                    ) {
                        Some((found_px, found_sx)) => (found_px, found_sx, 1),
                        None => (0 as u_int, 0 as u_int, 0),
                    };
                } else {
                    (px, found) = match window_copy_search_lr(gd, sgd, i, fx, gd.width(), cis) {
                        Some(found_px) => (found_px, 1),
                        None => (px, 0),
                    };
                }
                if found != 0 {
                    break;
                }
                fx = 0 as u_int;
                i = i.wrapping_add(1);
            }
        } else {
            i = fy.wrapping_add(1 as u_int);
            while endline < i {
                if regex != 0 {
                    (px, sx, found) = match window_copy_search_rl_regex(
                        gd,
                        i.wrapping_sub(1 as u_int),
                        0 as u_int,
                        fx.wrapping_add(1 as u_int),
                        reg.as_ref().expect("a compiled search pattern"),
                    ) {
                        Some((found_px, found_sx)) => (found_px, found_sx, 1),
                        None => (0 as u_int, 0 as u_int, 0),
                    };
                    if found != 0 {
                        window_copy_search_back_overlap(
                            gd,
                            reg.as_ref().expect("a compiled search pattern"),
                            &mut px,
                            &mut sx,
                            &mut i,
                            endline,
                        );
                    }
                } else {
                    (px, found) = match window_copy_search_rl(
                        gd,
                        sgd,
                        i.wrapping_sub(1 as u_int),
                        0 as u_int,
                        fx.wrapping_add(1 as u_int),
                        cis,
                    ) {
                        Some(found_px) => (found_px, 1),
                        None => (px, 0),
                    };
                }
                if found != 0 {
                    i = i.wrapping_sub(1);
                    break;
                } else {
                    fx = gd.width().wrapping_sub(1 as u_int);
                    i = i.wrapping_sub(1);
                }
            }
        }
        drop(reg);
        if found != 0 {
            return Some((px, i));
        }
        if wrap != 0 {
            return window_copy_search_position(
                gd,
                sgd,
                if direction != 0 {
                    0 as u_int
                } else {
                    gd.width().wrapping_sub(1 as u_int)
                },
                if direction != 0 {
                    0 as u_int
                } else {
                    gd.history_size().wrapping_add(gd.height()).wrapping_sub(1 as u_int)
                },
                fy,
                (cis, 0 as core::ffi::c_int, direction, regex),
            );
        }
        None
    }
}
fn window_copy_move_after_search_mark(
    data: &window_copy_mode_data,
    fx: &mut u_int,
    fy: &mut u_int,
    wrapflag: core::ffi::c_int,
) {
    {
        let s = data
            .backing
            .as_deref()
            .expect("copy mode has a backing screen");
        let start = window_copy_search_mark_at(data, *fx, *fy);
        if start.is_some_and(|start| {
            (&data.searchmark)[start as usize] as core::ffi::c_int != 0 as core::ffi::c_int
        }) {
            let start = start.expect("the mark just looked at");
            while let Some(at) = window_copy_search_mark_at(data, *fx, *fy) {
                if (&data.searchmark)[at as usize] as core::ffi::c_int
                    != (&data.searchmark)[start as usize] as core::ffi::c_int
                {
                    break;
                }
                if wrapflag == 0
                    && *fx == RustScreen::grid(s).width().wrapping_sub(1 as u_int)
                    && *fy
                        == RustScreen::grid(s)
                            .history_size()
                            .wrapping_add(RustScreen::grid(s).height())
                            .wrapping_sub(1 as u_int)
                {
                    break;
                }
                window_copy_move_right(s, fx, fy, wrapflag);
            }
        }
    }
}
unsafe fn window_copy_search(
    wme: &mut window_mode_entry,
    direction: core::ffi::c_int,
    mut regex: core::ffi::c_int,
) -> core::ffi::c_int {
    unsafe {
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        let search = data
            .searchstr
            .as_deref()
            .expect("copy mode has a search string")
            .to_owned();
        let at: u_int = 0;

        let mut fx: u_int;
        let mut fy: u_int;

        let visible_only: core::ffi::c_int;

        if regex != 0
            && !search
                .to_bytes()
                .iter()
                .any(|byte| b"^$*+()?[].\\".contains(byte))
        {
            regex = 0 as core::ffi::c_int;
        }
        data.searchdirection = direction;
        if data.timeout != 0 {
            return 0 as core::ffi::c_int;
        }
        let search_all = data.searchall != 0;
        let mut pane = wme.pane_ref().expect("mode has a pane");
        let wp = pane.get().expect("mode pane is live");
        if search_all || wp.query().is_none() || wp.is_regex() != (regex != 0) {
            visible_only = 0 as core::ffi::c_int;
            wme.state
                .copy_mode_data_mut()
                .expect("copy mode has state")
                .searchall = 0;
        } else {
            visible_only = wp.matches(&search, regex != 0) as core::ffi::c_int;
        }
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        if visible_only == 0 as core::ffi::c_int && !data.searchmark.is_empty() {
            window_copy_clear_marks(wme);
        }
        pane.get_mut()
            .expect("mode pane is live")
            .set(&search, regex != 0);
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        fx = data.cx;
        fy = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_sub(data.oy)
        .wrapping_add(data.cy);
        let ssx: u_int = screen_write_strlen(c"%s", fmt_args![search.as_c_str()]) as u_int;
        if ssx == 0 as u_int {
            return 0 as core::ffi::c_int;
        }
        let mut ss = RustScreen::new_with_server_options(ssx, 1 as u_int, 0 as u_int);
        let mut ctx = screen_write_ctx_on_screen(&mut ss);
        ctx.nputs(
            -(1 as core::ffi::c_int) as ssize_t,
            &grid_default_cell,
            c"%s",
            fmt_args![search.as_c_str()],
        );
        ctx.finish();
        let window = pane.window().expect("mode pane has a window");
        let wrapflag: core::ffi::c_int =
            window.options().number(c"wrap-search") as core::ffi::c_int;
        let cis: core::ffi::c_int = window_copy_is_lowercase(&search);
        let keys: core::ffi::c_int = window.options().number(c"mode-keys") as core::ffi::c_int;
        let endline: u_int = if direction != 0 {
            if keys == MODEKEY_VI {
                if !data.searchmark.is_empty() {
                    window_copy_move_after_search_mark(data, &mut fx, &mut fy, wrapflag);
                } else {
                    window_copy_move_right(
                        data.backing
                            .as_deref()
                            .expect("copy mode has a backing screen"),
                        &mut fx,
                        &mut fy,
                        wrapflag,
                    );
                }
            }
            let gd = RustScreen::grid(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
            );
            gd.history_size().wrapping_add(gd.height()).wrapping_sub(1 as u_int)
        } else {
            window_copy_move_left(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
                &mut fx,
                &mut fy,
                wrapflag,
            );
            0 as u_int
        };
        let found: core::ffi::c_int = if let Some((x, y)) = window_copy_search_position(
            RustScreen::grid(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
            ),
            RustScreen::grid(&ss),
            fx,
            fy,
            endline,
            (cis, wrapflag, direction, regex),
        ) {
            window_copy_scroll_to(wme, x, y, 1);
            1
        } else {
            0
        };
        if found != 0 {
            window_copy_search_marks(wme, Some(&ss), regex, visible_only);
            let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
            fx = data.cx;
            fy = RustScreen::grid(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
            )
            .history_size()
            .wrapping_sub(data.oy)
            .wrapping_add(data.cy);
            if direction != 0
                && window_copy_search_mark_at(data, fx, fy).is_some_and(|at| {
                    at > 0 as u_int
                        && !data.searchmark.is_empty()
                        && (&data.searchmark)[at as usize] as core::ffi::c_int
                            == (&data.searchmark)[at.wrapping_sub(1 as u_int) as usize]
                                as core::ffi::c_int
                })
            {
                window_copy_move_after_search_mark(data, &mut fx, &mut fy, wrapflag);
                if let Some((x, y)) = window_copy_search_position(
                    RustScreen::grid(
                        data.backing
                            .as_deref()
                            .expect("copy mode has a backing screen"),
                    ),
                    RustScreen::grid(&ss),
                    fx,
                    fy,
                    endline,
                    (cis, wrapflag, direction, regex),
                ) {
                    window_copy_scroll_to(wme, x, y, 1);
                }
                let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
                fx = data.cx;
                fy = RustScreen::grid(
                    data.backing
                        .as_deref()
                        .expect("copy mode has a backing screen"),
                )
                .history_size()
                .wrapping_sub(data.oy)
                .wrapping_add(data.cy);
            }
            let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
            if direction != 0 {
                if keys == MODEKEY_EMACS {
                    window_copy_move_after_search_mark(&*data, &mut fx, &mut fy, wrapflag);
                    data.cx = fx;
                    data.cy = fy
                        .wrapping_sub(
                            RustScreen::grid(
                                data.backing
                                    .as_deref()
                                    .expect("copy mode has a backing screen"),
                            )
                            .history_size(),
                        )
                        .wrapping_add(data.oy);
                }
            } else if let Some(start) = window_copy_search_mark_at(&*data, fx, fy) {
                while window_copy_search_mark_at(&*data, fx, fy).is_some_and(|at| {
                    !data.searchmark.is_empty()
                        && (&data.searchmark)[at as usize] as core::ffi::c_int
                            == (&data.searchmark)[start as usize] as core::ffi::c_int
                }) {
                    data.cx = fx;
                    data.cy = fy
                        .wrapping_sub(
                            RustScreen::grid(
                                data.backing
                                    .as_deref()
                                    .expect("copy mode has a backing screen"),
                            )
                            .history_size(),
                        )
                        .wrapping_add(data.oy);
                    if at == 0 as u_int {
                        break;
                    }
                    window_copy_move_left(
                        data.backing
                            .as_deref()
                            .expect("copy mode has a backing screen"),
                        &mut fx,
                        &mut fy,
                        0 as core::ffi::c_int,
                    );
                }
            }
        }
        window_copy_redraw_screen(wme);
        found
    }
}
/// The first and last line of what the pane shows, the first walked back
/// over the wrapped lines it continues.
fn window_copy_visible_lines(data: &window_copy_mode_data) -> (u_int, u_int) {
    {
        let gd = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        );
        let mut gl: Option<crate::grid::GridLineInfo>;
        let mut start = gd.history_size().wrapping_sub(data.oy);
        while start > 0 as u_int {
            gl = (gd).peek_line(start.wrapping_sub(1 as u_int));
            if !gl.is_some_and(|gl| gl.flags & GRID_LINE_WRAPPED != 0) {
                break;
            }
            start = start.wrapping_sub(1);
        }
        let end = gd.history_size().wrapping_sub(data.oy).wrapping_add(gd.height());
        (start, end)
    }
}
/// Where `px`,`py` falls in the search mark, or nothing when it is not on
/// the part of the history the pane shows.
fn window_copy_search_mark_at(data: &window_copy_mode_data, px: u_int, py: u_int) -> Option<u_int> {
    {
        let gd = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        );
        if py < gd.history_size().wrapping_sub(data.oy) {
            return None;
        }
        if py
            > gd.history_size()
                .wrapping_sub(data.oy)
                .wrapping_add(gd.height())
                .wrapping_sub(1 as u_int)
        {
            return None;
        }
        Some(
            py.wrapping_sub(gd.history_size().wrapping_sub(data.oy))
                .wrapping_mul(gd.width())
                .wrapping_add(px),
        )
    }
}
fn window_copy_clip_width(width: u_int, b: u_int, sx: u_int, sy: u_int) -> u_int {
    if b.wrapping_add(width) > sx.wrapping_mul(sy) {
        sx.wrapping_mul(sy).wrapping_sub(b)
    } else {
        width
    }
}
fn window_copy_search_mark_match(
    data: &mut window_copy_mode_data,
    px: u_int,
    py: u_int,
    mut width: u_int,
    regex: core::ffi::c_int,
) -> u_int {
    {
        let gd = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        );
        let mut gc;
        let mut i: u_int;
        let mut w: u_int = width;
        let sx: u_int = gd.width();
        let sy: u_int = gd.height();
        if let Some(b) = window_copy_search_mark_at(data, px, py) {
            width = window_copy_clip_width(width, b, sx, sy);
            w = width;
            i = b;
            while i < b.wrapping_add(w) {
                if regex == 0 {
                    gc = (*gd).cell(px.wrapping_add(i.wrapping_sub(b)), py);
                    if gc.flags as core::ffi::c_int & GRID_FLAG_TAB != 0 {
                        w = w.wrapping_add(
                            (gc.data.width as core::ffi::c_int - 1 as core::ffi::c_int) as u_int,
                        );
                    }
                    w = window_copy_clip_width(w, b, sx, sy);
                }
                if (&data.searchmark)[i as usize] as core::ffi::c_int == 0 as core::ffi::c_int {
                    (&mut data.searchmark)[i as usize] = data.searchgen;
                }
                i = i.wrapping_add(1);
            }
            if data.searchgen as core::ffi::c_int == UCHAR_MAX {
                data.searchgen = 1 as u_char;
            } else {
                data.searchgen = data.searchgen.wrapping_add(1);
            }
        }
        w
    }
}
unsafe fn window_copy_search_marks(
    wme: &mut window_mode_entry,
    ssp: Option<&RustScreen>,
    regex: core::ffi::c_int,
    visible_only: core::ffi::c_int,
) -> core::ffi::c_int {
    unsafe {
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        let s = data
            .backing
            .as_deref()
            .expect("copy mode has a backing screen");
        let mut ss;
        let gd = RustScreen::grid(s);
        let mut gc;
        let mut found: core::ffi::c_int;

        let mut stopped: core::ffi::c_int = 0 as core::ffi::c_int;
        let mut cflags: core::ffi::c_int = REG_EXTENDED;
        let mut px: u_int;
        let mut py: u_int;
        let mut nfound: u_int = 0 as u_int;
        let mut width: u_int;
        let mut start: u_int;
        let mut end: u_int;
        let sx: u_int = gd.width();
        let sy: u_int = gd.height();
        let mut reg: Option<CompiledRegex> = None;
        let mut stop: uint64_t = 0 as uint64_t;

        let mut t: uint64_t;
        // A caller with no screen of its own gets one drawn here holding the
        // search string, which the rest of the body then reads through the
        // same handle.
        let ssp = match ssp {
            None => {
                width = screen_write_strlen(c"%s", fmt_args![data.searchstr.as_deref()]) as u_int;
                ss = RustScreen::new_with_server_options(width, 1 as u_int, 0 as u_int);
                let mut ctx = screen_write_ctx_on_screen(&mut ss);
                ctx.nputs(
                    -(1 as core::ffi::c_int) as ssize_t,
                    &grid_default_cell,
                    c"%s",
                    fmt_args![data.searchstr.as_deref()],
                );
                ctx.finish();
                &ss
            }
            Some(ssp) => {
                width = RustScreen::grid(ssp).width();
                ssp
            }
        };
        let cis: core::ffi::c_int =
            window_copy_is_lowercase(data.searchstr.as_deref().unwrap_or(c""));
        if regex != 0 {
            let mut sbuf: Vec<u8> = vec![b'\0'];
            window_copy_stringify(
                RustScreen::grid(ssp),
                0 as u_int,
                0 as u_int,
                RustScreen::grid(ssp).width(),
                &mut sbuf,
            );
            if cis != 0 {
                cflags |= REG_ICASE;
            }
            let pattern = CStr::from_bytes_until_nul(&sbuf).expect("a terminated search pattern");
            let Some(compiled) = CompiledRegex::compile(pattern, cflags) else {
                return 0;
            };
            reg = Some(compiled);
        }
        let tstart: uint64_t = get_timer();
        if visible_only != 0 {
            (start, end) = window_copy_visible_lines(&*data);
        } else {
            start = 0 as u_int;
            end = gd.history_size().wrapping_add(sy);
            stop = get_timer().wrapping_add(WINDOW_COPY_SEARCH_ALL_TIMEOUT as uint64_t);
        }
        loop {
            data.searchmark.clear();
            data.searchmark
                .resize((sx as usize).saturating_mul(sy as usize), 0);
            data.searchgen = 1 as u_char;
            py = start;
            while py < end {
                px = 0 as u_int;
                loop {
                    let gd = RustScreen::grid(
                        data.backing
                            .as_deref()
                            .expect("copy mode has a backing screen"),
                    );
                    if regex != 0 {
                        (px, width, found) = match window_copy_search_lr_regex(
                            gd,
                            py,
                            px,
                            sx,
                            reg.as_ref().expect("a compiled search pattern"),
                        ) {
                            Some((found_px, found_width)) => (found_px, found_width, 1),
                            None => (0 as u_int, 0 as u_int, 0),
                        };
                        gc = (*gd).cell(px.wrapping_add(width).wrapping_sub(1), py);
                        if gc.data.width as core::ffi::c_int > 2 as core::ffi::c_int {
                            width = width.wrapping_add(
                                (gc.data.width as core::ffi::c_int - 1 as core::ffi::c_int)
                                    as u_int,
                            );
                        }
                        if found == 0 {
                            break;
                        }
                    } else {
                        (px, found) =
                            match window_copy_search_lr(gd, RustScreen::grid(ssp), py, px, sx, cis)
                            {
                                Some(found_px) => (found_px, 1),
                                None => (px, 0),
                            };
                        if found == 0 {
                            break;
                        }
                    }
                    nfound = nfound.wrapping_add(1);
                    px = px.wrapping_add(window_copy_search_mark_match(
                        &mut *data, px, py, width, regex,
                    ));
                }
                t = get_timer();
                if t.wrapping_sub(tstart) > WINDOW_COPY_SEARCH_TIMEOUT as uint64_t {
                    data.timeout = 1 as core::ffi::c_int;
                    break;
                } else if stop != 0 as uint64_t && t > stop {
                    stopped = 1 as core::ffi::c_int;
                    break;
                } else {
                    py = py.wrapping_add(1);
                }
            }
            if data.timeout != 0 {
                window_copy_clear_marks(wme);
                break;
            } else if stopped != 0 && stop != 0 as uint64_t {
                (start, end) = window_copy_visible_lines(&*data);
                stop = 0 as uint64_t;
            } else {
                if visible_only == 0 {
                    if stopped != 0 {
                        if nfound > 1000 as u_int {
                            data.searchcount = 1000 as core::ffi::c_int;
                        } else if nfound > 100 as u_int {
                            data.searchcount = 100 as core::ffi::c_int;
                        } else if nfound > 10 as u_int {
                            data.searchcount = 10 as core::ffi::c_int;
                        } else {
                            data.searchcount = -(1 as core::ffi::c_int);
                        }
                        data.searchmore = 1 as core::ffi::c_int;
                    } else {
                        data.searchcount = nfound as core::ffi::c_int;
                        data.searchmore = 0 as core::ffi::c_int;
                    }
                }
                break;
            }
        }
        drop(reg);
        1 as core::ffi::c_int
    }
}
fn window_copy_clear_marks(wme: &mut window_mode_entry) {
    {
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        data.searchcount = -(1 as core::ffi::c_int);
        data.searchmore = 0 as core::ffi::c_int;
        data.searchmark.clear();
    }
}
unsafe fn window_copy_search_up(
    wme: &mut window_mode_entry,
    regex: core::ffi::c_int,
) -> core::ffi::c_int {
    unsafe { window_copy_search(wme, 0 as core::ffi::c_int, regex) }
}
unsafe fn window_copy_search_down(
    wme: &mut window_mode_entry,
    regex: core::ffi::c_int,
) -> core::ffi::c_int {
    unsafe { window_copy_search(wme, 1 as core::ffi::c_int, regex) }
}
unsafe fn window_copy_goto_line(wme: &mut window_mode_entry, linestr: &CStr) {
    unsafe {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let hsize: u_int = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size();
        let line: u_int;
        let Ok(lineno) = strtonum(
            linestr,
            -(1 as core::ffi::c_int) as core::ffi::c_longlong,
            INT_MAX as core::ffi::c_longlong,
        ) else {
            return;
        };
        let mut lineno = lineno as core::ffi::c_int;
        let absolute = window_copy_line_number_is_absolute(wme) != 0;
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        if absolute {
            if lineno <= 0 as core::ffi::c_int {
                line = 1 as u_int;
            } else if lineno as u_int > hsize.wrapping_add(1 as u_int) {
                line = hsize.wrapping_add(1 as u_int);
            } else {
                line = lineno as u_int;
            }
            data.oy = hsize.wrapping_sub(line.wrapping_sub(1 as u_int));
        } else {
            if lineno < 0 as core::ffi::c_int || lineno as u_int > hsize {
                lineno = hsize as core::ffi::c_int;
            }
            data.oy = lineno as u_int;
        }
        window_copy_update_selection(wme, 1 as core::ffi::c_int, 0 as core::ffi::c_int);
        window_copy_redraw_screen(wme);
    }
}
/// The first and last cell of the run of marks `at` stands in.
fn window_copy_match_start_end(data: &window_copy_mode_data, at: u_int) -> (u_int, u_int) {
    {
        let gd = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        );
        let last: u_int = gd.height().wrapping_mul(gd.width()).wrapping_sub(1 as u_int);
        let mark: u_char = (&data.searchmark)[at as usize];
        let mut end = at;
        let mut start = end;
        while start != 0 as u_int
            && (&data.searchmark)[start as usize] as core::ffi::c_int == mark as core::ffi::c_int
        {
            start = start.wrapping_sub(1);
        }
        if (&data.searchmark)[start as usize] as core::ffi::c_int != mark as core::ffi::c_int {
            start = start.wrapping_add(1);
        }
        while end != last
            && (&data.searchmark)[end as usize] as core::ffi::c_int == mark as core::ffi::c_int
        {
            end = end.wrapping_add(1);
        }
        if (&data.searchmark)[end as usize] as core::ffi::c_int != mark as core::ffi::c_int {
            end = end.wrapping_sub(1);
        }
        (start, end)
    }
}
/// The text of the search match the cursor stands in, or nothing when the
/// screen carries no marks or the cursor is not on one.
unsafe fn window_copy_match_at_cursor(data: &window_copy_mode_data) -> Option<CString> {
    unsafe {
        let gd = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        );
        let mut gc;
        let mut at: u_int;
        let start: u_int;
        let end: u_int;
        let sx: u_int = gd.width();
        if data.searchmark.is_empty() {
            return None;
        }
        // The grid's own numbers are read off it before the walk that hands
        // the mode data on, which is the same values read once.
        let (hsize, oy, cx) = (gd.history_size(), data.oy, data.cx);
        let cy = hsize.wrapping_sub(oy).wrapping_add(data.cy);
        let found_at = window_copy_search_mark_at(data, cx, cy)?;
        at = found_at;
        if data.searchmark[at as usize] as core::ffi::c_int == 0 as core::ffi::c_int
            && (at == 0 as u_int || {
                at = at.wrapping_sub(1);
                data.searchmark[at as usize] as core::ffi::c_int == 0 as core::ffi::c_int
            })
        {
            return None;
        }
        (start, end) = window_copy_match_start_end(data, at);
        let mut buf: Vec<u8> = Vec::new();
        at = start;
        while at <= end {
            let py = at.wrapping_div(sx);
            let px = at.wrapping_sub(py.wrapping_mul(sx));
            gc = gd.cell(px, hsize.wrapping_add(py).wrapping_sub(oy));
            if gc.flags as core::ffi::c_int & GRID_FLAG_TAB != 0 {
                buf.push(b'\t');
            } else if !(gc.flags as core::ffi::c_int & GRID_FLAG_PADDING != 0) {
                buf.extend_from_slice(&gc.data.data[..gc.data.size as usize]);
            }
            at = at.wrapping_add(1);
        }
        if buf.is_empty() {
            return None;
        }
        Some(CString::from_vec_unchecked(buf))
    }
}
fn window_copy_window_options(wme: &window_mode_entry) -> RustOptionsRef {
    let pane = wme.pane_ref().expect("copy mode has a pane");
    if let Some(window) = pane.window() {
        return window.options();
    }
    unsafe {
        pane.get()
            .expect("the pane exists during mode teardown")
            .options_ref()
            .parent()
            .expect("pane options inherit window options")
    }
}

unsafe fn window_copy_format_context(wme: &window_mode_entry) -> Box<format_tree> {
    unsafe {
        let pane = wme.pane_ref().expect("copy mode has a pane");
        format_create_defaults(None, None, None, None, pane.get())
    }
}

fn window_copy_update_style(
    wme: &window_mode_entry,
    fx: u_int,
    fy: u_int,
    gc: &mut grid_cell,
    mgc: &grid_cell,
    cgc: &grid_cell,
    mkgc: &grid_cell,
) {
    {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");

        let start: u_int;
        let end: u_int;

        let mut cursor: u_int;

        let mut inv: core::ffi::c_int = 0 as core::ffi::c_int;
        let mut found: core::ffi::c_int = 0 as core::ffi::c_int;
        let keys: core::ffi::c_int;
        if data.showmark != 0 && fy == data.my {
            gc.attr = mkgc.attr;
            if fx == data.mx {
                inv = 1 as core::ffi::c_int;
            }
            if inv != 0 {
                gc.fg = mkgc.bg;
                gc.bg = mkgc.fg;
            } else {
                gc.fg = mkgc.fg;
                gc.bg = mkgc.bg;
            }
        }
        if data.searchmark.is_empty() {
            return;
        }
        let Some(found_at) = window_copy_search_mark_at(data, fx, fy) else {
            return;
        };
        let current: u_int = found_at;
        let mark: u_int = (&data.searchmark)[current as usize] as u_int;
        if mark == 0 as u_int {
            return;
        }
        let cy: u_int = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_sub(data.oy)
        .wrapping_add(data.cy);
        if let Some(found_at) = window_copy_search_mark_at(data, data.cx, cy) {
            cursor = found_at;
            keys = window_copy_window_options(wme).number(c"mode-keys") as core::ffi::c_int;
            if cursor != 0 as u_int && keys == MODEKEY_EMACS && data.searchdirection != 0 {
                if (&data.searchmark)[cursor.wrapping_sub(1 as u_int) as usize] as u_int == mark {
                    cursor = cursor.wrapping_sub(1);
                    found = 1 as core::ffi::c_int;
                }
            } else if (&data.searchmark)[cursor as usize] as u_int == mark {
                found = 1 as core::ffi::c_int;
            }
            if found != 0 {
                (start, end) = window_copy_match_start_end(data, cursor);
                if current >= start && current <= end {
                    gc.attr = cgc.attr;
                    if inv != 0 {
                        gc.fg = cgc.bg;
                        gc.bg = cgc.fg;
                    } else {
                        gc.fg = cgc.fg;
                        gc.bg = cgc.bg;
                    }
                    return;
                }
            }
        }
        gc.attr = mgc.attr;
        if inv != 0 {
            gc.fg = mgc.bg;
            gc.bg = mgc.fg;
        } else {
            gc.fg = mgc.fg;
            gc.bg = mgc.bg;
        };
    }
}
fn window_copy_write_one(
    wme: &window_mode_entry,
    writer: &mut impl ScreenWriteCtx,
    coords: (u_int, u_int, u_int, u_int),
    mgc: &grid_cell,
    cgc: &grid_cell,
    mkgc: &grid_cell,
) {
    {
        let (px, py, fy, nx) = coords;
        let mut gc;
        let mut fx: u_int;
        writer.cursormove(
            px as core::ffi::c_int,
            py as core::ffi::c_int,
            0 as core::ffi::c_int,
        );
        fx = 0 as u_int;
        while fx < nx {
            let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
            let gd = RustScreen::grid(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
            );
            gc = gd.cell(fx, fy);
            if fx.wrapping_add(gc.data.width as u_int) <= nx {
                window_copy_update_style(wme, fx, fy, &mut gc, mgc, cgc, mkgc);
                writer.cell(&gc);
            }
            fx = fx.wrapping_add(1);
        }
    }
}
fn window_copy_line_number_mode(wme: &window_mode_entry) -> core::ffi::c_int {
    {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        if data.line_numbers == 0 {
            return WINDOW_COPY_LINE_NUMBERS_OFF as core::ffi::c_int;
        }
        window_copy_window_options(wme).number(c"copy-mode-line-numbers") as core::ffi::c_int
    }
}
fn window_copy_line_number_is_absolute(wme: &window_mode_entry) -> core::ffi::c_int {
    {
        match window_copy_line_number_mode(wme) {
            2..=4 => return 1 as core::ffi::c_int,
            0 | 1 => return 0 as core::ffi::c_int,
            _ => {}
        }
        fatalx(c"bad line number mode", fmt_args![]);
    }
}
fn window_copy_line_numbers_active(wme: &window_mode_entry) -> core::ffi::c_int {
    {
        (window_copy_line_number_mode(wme) != WINDOW_COPY_LINE_NUMBERS_OFF as core::ffi::c_int)
            as core::ffi::c_int
    }
}
fn window_copy_line_number_width(wme: &window_mode_entry) -> u_int {
    {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let mut lines: u_int;
        let mut digits: u_int;
        if window_copy_line_numbers_active(wme) == 0 {
            return 0 as u_int;
        }
        lines = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_add(
            RustScreen::grid(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
            )
            .height(),
        )
        .wrapping_add(1 as u_int);
        digits = 1 as u_int;
        while lines >= 10 as u_int {
            lines = lines.wrapping_div(10 as u_int);
            digits = digits.wrapping_add(1);
        }
        if digits < 3 as u_int {
            digits = 3 as u_int;
        }
        digits.wrapping_add(1 as u_int)
    }
}
fn window_copy_cursor_offset(wme: &window_mode_entry, cx: u_int, sx: u_int) -> u_int {
    {
        let width: u_int = window_copy_line_number_width(wme);

        if width == 0 as u_int {
            return cx;
        }
        let content: u_int = if width >= sx {
            1 as u_int
        } else {
            sx.wrapping_sub(width)
        };
        if cx >= content {
            return sx.wrapping_sub(1 as u_int);
        }
        width.wrapping_add(cx)
    }
}
fn window_copy_cursor_unoffset(wme: &window_mode_entry, mut vx: u_int, sx: u_int) -> u_int {
    {
        let width: u_int = window_copy_line_number_width(wme);

        if width == 0 as u_int {
            return vx;
        }
        let content: u_int = if width >= sx {
            1 as u_int
        } else {
            sx.wrapping_sub(width)
        };
        if vx < width {
            return 0 as u_int;
        }
        vx = vx.wrapping_sub(width);
        if vx >= content {
            return content.wrapping_sub(1 as u_int);
        }
        vx
    }
}
pub unsafe fn window_copy_set_line_numbers(
    wp: &mut (impl crate::WindowPane + ?Sized),
    enabled: core::ffi::c_int,
) {
    unsafe {
        let Some(wme) = (&mut *wp).active_mode_mut().map(|mode| mode.into_entry()) else {
            return;
        };
        if wme.mode() != WindowMode::Copy {
            return;
        }
        let Some(data) = wme.state.copy_mode_data_mut() else {
            return;
        };
        if data.line_numbers == enabled {
            return;
        }
        data.line_numbers = enabled;
        window_copy_redraw_screen(wme);
    }
}
/// How far back the pane is scrolled and how much history it has, or
/// nothing when the pane is not in a copy or view mode.
pub fn window_copy_get_current_offset(wp: &(impl crate::WindowPane + ?Sized)) -> Option<(u_int, u_int)> {
    let wme = (wp).active_mode()?;
    let (WindowModeState::Copy(data) | WindowModeState::View(data)) = &wme.state else {
        return None;
    };
    let hsize = RustScreen::grid(
        data.backing
            .as_deref()
            .expect("copy mode has a backing screen"),
    )
    .history_size();
    Some((hsize.wrapping_sub(data.oy), hsize))
}

unsafe fn window_copy_write_line(
    wme: &mut window_mode_entry,
    writer: &mut impl ScreenWriteCtx,
    py: u_int,
) {
    unsafe {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let oo = window_copy_window_options(wme);
        let mut gc = grid_default_cell;
        let mut mgc = grid_default_cell;
        let mut cgc = grid_default_cell;
        let mut mkgc = grid_default_cell;
        let mut ln_gc = grid_default_cell;
        let mut cur_ln_gc = grid_default_cell;
        let sx: u_int = RustScreen::grid(&data.screen.borrow()).width();
        let hsize: u_int = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size();

        let absolute: u_int;
        let line_number: u_int;

        let current: core::ffi::c_int;
        let mode: core::ffi::c_int;
        let width: u_int = window_copy_line_number_width(wme);
        let content_sx: u_int = if width >= sx {
            1 as u_int
        } else if width != 0 as u_int {
            sx.wrapping_sub(width)
        } else {
            sx
        };
        let mut ft = window_copy_format_context(wme);
        style_apply(&mut gc, &oo, c"copy-mode-position-style", Some(&mut ft));
        gc.flags = (gc.flags as core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
        style_apply(&mut mgc, &oo, c"copy-mode-match-style", Some(&mut ft));
        mgc.flags = (mgc.flags as core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
        style_apply(
            &mut cgc,
            &oo,
            c"copy-mode-current-match-style",
            Some(&mut ft),
        );
        cgc.flags = (cgc.flags as core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
        style_apply(&mut mkgc, &oo, c"copy-mode-mark-style", Some(&mut ft));
        mkgc.flags = (mkgc.flags as core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
        if width != 0 as u_int {
            style_apply(
                &mut ln_gc,
                &oo,
                c"copy-mode-line-number-style",
                Some(&mut ft),
            );
            ln_gc.flags = (ln_gc.flags as core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
            style_apply(
                &mut cur_ln_gc,
                &oo,
                c"copy-mode-current-line-number-style",
                Some(&mut ft),
            );
            cur_ln_gc.flags = (cur_ln_gc.flags as core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
            let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
            current = (py == data.cy) as core::ffi::c_int;
            absolute = hsize
                .wrapping_sub(data.oy)
                .wrapping_add(py)
                .wrapping_add(1 as u_int);
            mode = window_copy_line_number_mode(wme);
            if mode == WINDOW_COPY_LINE_NUMBERS_DEFAULT as core::ffi::c_int {
                if py < data.oy {
                    line_number = data.oy.wrapping_sub(py);
                } else {
                    line_number = py.wrapping_sub(data.oy);
                }
            } else if mode == WINDOW_COPY_LINE_NUMBERS_ABSOLUTE as core::ffi::c_int
                || mode == WINDOW_COPY_LINE_NUMBERS_HYBRID as core::ffi::c_int && current != 0
            {
                line_number = absolute;
            } else if py > data.cy {
                line_number = py.wrapping_sub(data.cy);
            } else {
                line_number = data.cy.wrapping_sub(py);
            }
            writer.cursormove(
                0 as core::ffi::c_int,
                py as core::ffi::c_int,
                0 as core::ffi::c_int,
            );
            writer.nputs(
                width as ssize_t,
                if current != 0 { &cur_ln_gc } else { &ln_gc },
                c"%*u ",
                fmt_args![
                    width as core::ffi::c_int - 1 as core::ffi::c_int,
                    line_number
                ],
            );
        }
        let oy = wme
            .state
            .copy_mode_data_ref()
            .expect("copy mode has state")
            .oy;
        window_copy_write_one(
            wme,
            writer,
            (
                width,
                py,
                hsize.wrapping_sub(oy).wrapping_add(py),
                content_sx,
            ),
            &mgc,
            &cgc,
            &mkgc,
        );
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        if py == 0 as u_int
            && data.screen.borrow().region().0 < data.screen.borrow().region().1
            && data.hide_position == 0
        {
            let value = oo.string_ref(c"copy-mode-position-format");
            if !value.is_empty() {
                let expanded = format_expand(&mut ft, &value);
                if !expanded.as_bytes().is_empty() {
                    writer.cursormove(
                        width as core::ffi::c_int,
                        0 as core::ffi::c_int,
                        0 as core::ffi::c_int,
                    );
                    writer.format_draw(
                        &gc,
                        content_sx,
                        expanded.as_bytes(),
                        None,
                        0 as core::ffi::c_int,
                    );
                }
            }
        }
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        if py == data.cy && data.cx >= content_sx {
            let x =
                window_copy_cursor_offset(wme, data.cx, RustScreen::grid(&data.screen.borrow()).width())
                    as core::ffi::c_int;
            writer.cursormove(x, py as core::ffi::c_int, 0 as core::ffi::c_int);
            writer.putc(&grid_default_cell, '$' as i32 as u_char);
        }
    }
}
unsafe fn window_copy_write_lines(
    wme: &mut window_mode_entry,
    writer: &mut impl ScreenWriteCtx,
    py: u_int,
    ny: u_int,
) {
    unsafe {
        let mut yy: u_int;
        yy = py;
        while yy < py.wrapping_add(ny) {
            window_copy_write_line(wme, writer, yy);
            yy = yy.wrapping_add(1);
        }
    }
}
unsafe fn window_copy_redraw_selection(wme: &mut window_mode_entry, old_y: u_int) {
    unsafe {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let gd = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        );

        let start: u_int;
        let mut end: u_int;
        let new_y: u_int = data.cy;
        if old_y <= new_y {
            start = old_y;
            end = new_y;
        } else {
            start = new_y;
            end = old_y;
        }
        if data.selflag as core::ffi::c_uint == SEL_WORD as core::ffi::c_int as core::ffi::c_uint
            && end < gd.height().wrapping_add(data.oy).wrapping_sub(1 as u_int)
        {
            end = end.wrapping_add(1);
        }
        window_copy_redraw_lines(wme, start, end.wrapping_sub(start).wrapping_add(1 as u_int));
    }
}
unsafe fn window_copy_redraw_lines(wme: &mut window_mode_entry, py: u_int, ny: u_int) {
    unsafe {
        let Some(mut pane) = wme.pane_ref() else {
            return;
        };
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        if pane.get().is_none() {
            return;
        }
        let _window = pane.window();
        let s = data.screen.clone();
        let mut i: u_int;
        if window_copy_line_number_width(wme) != 0 as u_int {
            let x = window_copy_cursor_offset(wme, data.cx, RustScreen::grid(&s.borrow()).width());
            let mut writer = RustScreenWriteCtx::on_shared_screen(&s);
            i = py;
            while i < py.wrapping_add(ny) {
                window_copy_write_line(wme, &mut writer, i);
                i = i.wrapping_add(1);
            }
            let cy = wme
                .state
                .copy_mode_data_ref()
                .expect("copy mode has state")
                .cy;
            writer.cursormove(
                x as core::ffi::c_int,
                cy as core::ffi::c_int,
                0 as core::ffi::c_int,
            );
            writer.finish();
            if let Some(pane) = pane.get_mut() {
                pane.request_full_redraw();
            }
            return;
        }
        let x = window_copy_cursor_offset(wme, data.cx, RustScreen::grid(&s.borrow()).width());
        let mut writer = RustScreenWriteCtx::on_shared_pane_screen(&s, Some(pane.clone()));
        i = py;
        while i < py.wrapping_add(ny) {
            window_copy_write_line(wme, &mut writer, i);
            i = i.wrapping_add(1);
        }
        let cy = wme
            .state
            .copy_mode_data_ref()
            .expect("copy mode has state")
            .cy;
        writer.cursormove(
            x as core::ffi::c_int,
            cy as core::ffi::c_int,
            0 as core::ffi::c_int,
        );
        writer.finish();
        if let Some(pane) = pane.get_mut() {
            pane.request_scrollbar_redraw();
        }
    }
}
unsafe fn window_copy_redraw_screen(wme: &mut window_mode_entry) {
    unsafe {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let sy = RustScreen::grid(&data.screen.borrow()).height();
        window_copy_redraw_lines(wme, 0, sy);
    }
}
pub(crate) unsafe fn window_copy_style_changed(wme: &mut window_mode_entry) {
    unsafe {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        if data.screen.borrow().has_selection() {
            window_copy_set_selection(wme, 0 as core::ffi::c_int, 1 as core::ffi::c_int);
        }
        window_copy_redraw_screen(wme);
    }
}
fn window_copy_synchronize_cursor_end(
    wme: &mut window_mode_entry,
    mut begin: core::ffi::c_int,
    no_reset: core::ffi::c_int,
) {
    {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let mut xx: u_int;
        let mut yy: u_int;
        xx = data.cx;
        yy = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_add(data.cy)
        .wrapping_sub(data.oy);
        match data.selflag {
            SEL_WORD => {
                if !(no_reset != 0) {
                    begin = 0 as core::ffi::c_int;
                    if data.dy > yy || data.dy == yy && data.dx > xx {
                        (xx, yy) = window_copy_cursor_previous_word_pos(
                            wme,
                            data.separators.as_deref().unwrap_or(c""),
                        );
                        begin = 1 as core::ffi::c_int;
                        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
                        data.endselx = data.endselrx;
                        data.endsely = data.endselry;
                    } else {
                        if xx >= window_copy_find_length(wme, yy)
                            || window_copy_in_set(wme, xx.wrapping_add(1 as u_int), yy, WHITESPACE)
                                == 0
                        {
                            (xx, yy) = window_copy_cursor_next_word_end_pos(
                                wme,
                                data.separators.as_deref().unwrap_or(c""),
                            );
                        }
                        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
                        data.selx = data.selrx;
                        data.sely = data.selry;
                    }
                }
            }
            SEL_LINE if !(no_reset != 0) => {
                begin = 0 as core::ffi::c_int;
                if data.dy > yy {
                    xx = 0 as u_int;
                    begin = 1 as core::ffi::c_int;
                    let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
                    data.endselx = data.endselrx;
                    data.endsely = data.endselry;
                } else {
                    if yy < data.endselry {
                        yy = data.endselry;
                    }
                    xx = window_copy_find_length(wme, yy);
                    let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
                    data.selx = data.selrx;
                    data.sely = data.selry;
                }
            }
            _ => {}
        }
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        if begin != 0 {
            data.selx = xx;
            data.sely = yy;
        } else {
            data.endselx = xx;
            data.endsely = yy;
        };
    }
}
fn window_copy_synchronize_cursor(wme: &mut window_mode_entry, no_reset: core::ffi::c_int) {
    {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        match data.cursordrag {
            CURSORDRAG_ENDSEL => {
                window_copy_synchronize_cursor_end(wme, 0 as core::ffi::c_int, no_reset);
            }
            CURSORDRAG_SEL => {
                window_copy_synchronize_cursor_end(wme, 1 as core::ffi::c_int, no_reset);
            }
            _ => {}
        };
    }
}
unsafe fn window_copy_update_cursor(wme: &mut window_mode_entry, mut cx: u_int, cy: u_int) {
    unsafe {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let s = &data.screen;

        let py: u_int;
        let width: u_int;
        let content_sx: u_int;
        let maxx: u_int;
        let allow_onemore: core::ffi::c_int;
        if data.rectflag == 0 && cy < RustScreen::grid(&s.borrow()).height() {
            allow_onemore =
                (data.screen.borrow().has_selection() && data.rectflag != 0) as core::ffi::c_int;
            py = RustScreen::grid(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
            )
            .history_size()
            .wrapping_add(cy)
            .wrapping_sub(data.oy);
            maxx = window_copy_cursor_limit(wme, py, allow_onemore);
            if cx > maxx {
                cx = maxx;
            }
        }
        let old_cx: u_int = data.cx;
        let old_cy: u_int = data.cy;
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        data.cx = cx;
        data.cy = cy;
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let s = &data.screen;
        if window_copy_line_numbers_active(wme) != 0 {
            width = window_copy_line_number_width(wme);
            if s.borrow().has_selection()
                || data.lineflag as core::ffi::c_uint
                    != LINE_SEL_NONE as core::ffi::c_int as core::ffi::c_uint
                || old_cy != data.cy
            {
                window_copy_redraw_screen(wme);
                return;
            }
            if width >= RustScreen::grid(&s.borrow()).width() {
                content_sx = 1 as u_int;
            } else {
                content_sx = RustScreen::grid(&s.borrow()).width().wrapping_sub(width);
            }
            if old_cx >= content_sx || data.cx >= content_sx {
                window_copy_redraw_screen(wme);
                return;
            }
            let x = window_copy_cursor_offset(wme, data.cx, RustScreen::grid(&s.borrow()).width())
                as core::ffi::c_int;
            let y = data.cy as core::ffi::c_int;
            let display = wme
                .state
                .copy_mode_data_ref()
                .expect("copy mode has state")
                .screen
                .clone();
            let pane = wme.pane_ref().filter(|pane| pane.get().is_some());
            let mut writer = RustScreenWriteCtx::on_shared_pane_screen(&display, pane);
            writer.cursormove(x, y, 0 as core::ffi::c_int);
            writer.finish();
            return;
        }
        if old_cx == RustScreen::grid(&s.borrow()).width() {
            window_copy_redraw_lines(wme, old_cy, 1 as u_int);
        }
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let s = &data.screen;
        if data.cx == RustScreen::grid(&s.borrow()).width() {
            let cy = data.cy;
            window_copy_redraw_lines(wme, cy, 1 as u_int);
        } else {
            let x = window_copy_cursor_offset(wme, data.cx, RustScreen::grid(&s.borrow()).width())
                as core::ffi::c_int;
            let y = data.cy as core::ffi::c_int;
            let display = wme
                .state
                .copy_mode_data_ref()
                .expect("copy mode has state")
                .screen
                .clone();
            let pane = wme.pane_ref().filter(|pane| pane.get().is_some());
            let mut writer = RustScreenWriteCtx::on_shared_pane_screen(&display, pane);
            writer.cursormove(x, y, 0 as core::ffi::c_int);
            writer.finish();
        };
    }
}
unsafe fn window_copy_start_selection(wme: &mut window_mode_entry) {
    unsafe {
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        data.selx = data.cx;
        data.sely = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_add(data.cy)
        .wrapping_sub(data.oy);
        data.endselx = data.selx;
        data.endsely = data.sely;
        data.cursordrag = CURSORDRAG_ENDSEL;
        window_copy_set_selection(wme, 1 as core::ffi::c_int, 0 as core::ffi::c_int);
    }
}
fn window_copy_adjust_selection(
    wme: &window_mode_entry,
    selx: &mut u_int,
    sely: &mut u_int,
) -> core::ffi::c_int {
    {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let s = &data.screen;
        let mut sx: u_int;
        let mut sy: u_int;

        let relpos: core::ffi::c_int;
        sx = *selx;
        sy = *sely;
        let ty: u_int = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_sub(data.oy);
        if sy < ty {
            relpos = WINDOW_COPY_REL_POS_ABOVE as core::ffi::c_int;
            if data.rectflag == 0 {
                sx = 0 as u_int;
            }
            sy = 0 as u_int;
        } else if sy
            > ty.wrapping_add(RustScreen::grid(&s.borrow()).height())
                .wrapping_sub(1 as u_int)
        {
            relpos = WINDOW_COPY_REL_POS_BELOW as core::ffi::c_int;
            if data.rectflag == 0 {
                sx = RustScreen::grid(&s.borrow()).width().wrapping_sub(1 as u_int);
            }
            sy = RustScreen::grid(&s.borrow()).height().wrapping_sub(1 as u_int);
        } else {
            relpos = WINDOW_COPY_REL_POS_ON_SCREEN as core::ffi::c_int;
            sy = sy.wrapping_sub(ty);
        }
        *selx = sx;
        *sely = sy;
        relpos
    }
}
unsafe fn window_copy_update_selection(
    wme: &mut window_mode_entry,
    may_redraw: core::ffi::c_int,
    no_reset: core::ffi::c_int,
) -> core::ffi::c_int {
    unsafe {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let s = &data.screen;
        if !s.borrow().has_selection()
            && data.lineflag as core::ffi::c_uint
                == LINE_SEL_NONE as core::ffi::c_int as core::ffi::c_uint
        {
            return 0 as core::ffi::c_int;
        }
        window_copy_set_selection(wme, may_redraw, no_reset)
    }
}
unsafe fn window_copy_set_selection(
    wme: &mut window_mode_entry,
    may_redraw: core::ffi::c_int,
    no_reset: core::ffi::c_int,
) -> core::ffi::c_int {
    unsafe {
        let oo = window_copy_window_options(wme);
        let mut gc = grid_default_cell;
        let mut sx: u_int;
        let mut sy: u_int;
        let cy: u_int;
        let mut endsx: u_int;
        let mut endsy: u_int;
        let mut clipx: u_int;

        window_copy_synchronize_cursor(wme, no_reset);
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        sx = data.selx;
        sy = data.sely;
        let startrelpos: core::ffi::c_int = window_copy_adjust_selection(wme, &mut sx, &mut sy);
        endsx = data.endselx;
        endsy = data.endsely;
        let endrelpos: core::ffi::c_int = window_copy_adjust_selection(wme, &mut endsx, &mut endsy);
        if startrelpos == endrelpos
            && startrelpos != WINDOW_COPY_REL_POS_ON_SCREEN as core::ffi::c_int
        {
            wme.state
                .copy_mode_data_mut()
                .expect("copy mode has state")
                .screen
                .borrow_mut()
                .hide_selection();
            return 0 as core::ffi::c_int;
        }
        let mut ft = window_copy_format_context(wme);
        style_apply(&mut gc, &oo, c"copy-mode-selection-style", Some(&mut ft));
        gc.flags = (gc.flags as core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let s = &data.screen;
        clipx = window_copy_line_number_width(wme);
        if clipx >= RustScreen::grid(&s.borrow()).width() {
            clipx = RustScreen::grid(&s.borrow()).width().wrapping_sub(1 as u_int);
        }
        if window_copy_line_numbers_active(wme) != 0 {
            sx = window_copy_cursor_offset(wme, sx, RustScreen::grid(&s.borrow()).width());
            endsx = window_copy_cursor_offset(wme, endsx, RustScreen::grid(&s.borrow()).width());
        }
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        data.screen.borrow_mut().set_selection(
            sx,
            sy,
            endsx,
            endsy,
            data.rectflag != 0,
            clipx,
            data.modekeys,
            &gc,
        );
        if data.rectflag != 0 && may_redraw != 0 {
            cy = data.cy;
            if data.cursordrag as core::ffi::c_uint
                == CURSORDRAG_ENDSEL as core::ffi::c_int as core::ffi::c_uint
            {
                if sy < cy {
                    window_copy_redraw_lines(wme, sy, cy.wrapping_sub(sy).wrapping_add(1 as u_int));
                } else {
                    window_copy_redraw_lines(wme, cy, sy.wrapping_sub(cy).wrapping_add(1 as u_int));
                }
            } else if endsy < cy {
                window_copy_redraw_lines(
                    wme,
                    endsy,
                    cy.wrapping_sub(endsy).wrapping_add(1 as u_int),
                );
            } else {
                window_copy_redraw_lines(wme, cy, endsy.wrapping_sub(cy).wrapping_add(1 as u_int));
            }
        }
        1 as core::ffi::c_int
    }
}
/// The text the selection covers, or the match under the cursor when there is
/// no selection at all, or nothing when neither holds any text.
unsafe fn window_copy_get_selection(wme: &window_mode_entry) -> Option<Vec<u8>> {
    unsafe {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let s = &data.screen;
        let mut i: u_int;
        let mut xx: u_int;

        let sx: u_int;
        let sy: u_int;
        let mut ex: u_int;
        let ey: u_int;

        let firstsx: u_int;
        let lastex: u_int;
        let restex: u_int;
        let restsx: u_int;
        let selx: u_int;

        if !data.screen.borrow().has_selection()
            && data.lineflag as core::ffi::c_uint
                == LINE_SEL_NONE as core::ffi::c_int as core::ffi::c_uint
        {
            return window_copy_match_at_cursor(data).map(CString::into_bytes);
        }
        let mut buf: Vec<u8> = Vec::new();
        xx = data.endselx;
        let yy: u_int = data.endsely;
        if yy < data.sely || yy == data.sely && xx < data.selx {
            sx = xx;
            sy = yy;
            ex = data.selx;
            ey = data.sely;
        } else {
            sx = data.selx;
            sy = data.sely;
            ex = xx;
            ey = yy;
        }
        let ey_last: u_int = window_copy_find_length(wme, ey);
        if ex > ey_last {
            ex = ey_last;
        }
        xx = RustScreen::grid(&s.borrow()).width();
        let pane = wme.pane_ref().expect("mode has a pane");
        let window = pane.window().expect("mode pane has a window");
        let keys: core::ffi::c_int = window.options().number(c"mode-keys") as core::ffi::c_int;
        if data.rectflag != 0 {
            if data.cursordrag as core::ffi::c_uint
                == CURSORDRAG_ENDSEL as core::ffi::c_int as core::ffi::c_uint
            {
                selx = data.selx;
            } else {
                selx = data.endselx;
            }
            if selx < data.cx {
                if keys == MODEKEY_EMACS {
                    lastex = data.cx;
                    restex = data.cx;
                } else {
                    lastex = data.cx.wrapping_add(1 as u_int);
                    restex = data.cx.wrapping_add(1 as u_int);
                }
                firstsx = selx;
                restsx = selx;
            } else {
                lastex = selx.wrapping_add(1 as u_int);
                restex = selx.wrapping_add(1 as u_int);
                firstsx = data.cx;
                restsx = data.cx;
            }
        } else {
            if keys == MODEKEY_EMACS {
                lastex = ex;
            } else {
                lastex = ex.wrapping_add(1 as u_int);
            }
            restex = xx;
            firstsx = sx;
            restsx = 0 as u_int;
        }
        i = sy;
        while i <= ey {
            window_copy_copy_line(
                wme,
                &mut buf,
                i,
                if i == sy { firstsx } else { restsx },
                if i == ey { lastex } else { restex },
            );
            i = i.wrapping_add(1);
        }
        if buf.is_empty() {
            return None;
        }
        if (keys == MODEKEY_EMACS || lastex <= ey_last)
            && (!(RustScreen::grid(
                    data.backing
                        .as_deref()
                        .expect("copy mode has a backing screen"),
                )).line_info(ey,)
            .flags
                & GRID_LINE_WRAPPED
                != 0
                || lastex != ey_last)
        {
            buf.pop();
        }
        Some(buf)
    }
}
unsafe fn window_copy_copy_buffer(
    wme: &mut window_mode_entry,
    prefix: Option<&CStr>,
    buf: Vec<u8>,
    set_paste: core::ffi::c_int,
    set_clip: core::ffi::c_int,
) {
    unsafe {
        let mut pane = wme.pane_ref().expect("mode has a pane");
        if set_clip != 0
            && (global_options
                .get()
                .as_ref()
                .expect("global options are initialized"))
            .number(c"set-clipboard")
                != 0 as core::ffi::c_longlong
        {
            let display = wme
                .state
                .copy_mode_data_ref()
                .expect("copy mode has state")
                .screen
                .clone();
            pane.write_clipboard_selection(&display, &buf, window_copy_line_numbers_active(wme) != 0);
        }
        if set_paste != 0 {
            let limit = paste_buffer_limit();
            with_paste_buffers_mut(|buffers| buffers.add_automatic(prefix, buf, limit));
        }
    }
}
unsafe fn window_copy_pipe_run(
    wme: &mut window_mode_entry,
    s: Option<&session>,
    cmd: Option<&CStr>,
) -> Option<Vec<u8>> {
    unsafe {
        let buf = window_copy_get_selection(wme);
        let fallback;
        let cmd = match cmd.filter(|cmd| !cmd.is_empty()) {
            Some(cmd) => cmd,
            None => {
                fallback = global_options
                    .get()
                    .as_ref()
                    .expect("global options are initialized")
                    .string_ref(c"copy-command");
                &fallback
            }
        };
        if !cmd.is_empty() {
            let job = job_run(
                Some(cmd),
                &[],
                None,
                s,
                None,
                None,
                None,
                JOB_NOWAIT,
                -(1 as core::ffi::c_int),
                -(1 as core::ffi::c_int),
            );
            if let Some(event) = job.and_then(crate::job::job_event_by_id) {
                let written = buf.as_deref().unwrap_or(&[]);
                event.write(written);
            }
        }
        buf
    }
}
unsafe fn window_copy_pipe(wme: &mut window_mode_entry, s: Option<&session>, cmd: Option<&CStr>) {
    unsafe {
        window_copy_pipe_run(wme, s, cmd);
    }
}
unsafe fn window_copy_copy_pipe(
    wme: &mut window_mode_entry,
    s: Option<&session>,
    prefix: Option<&CStr>,
    cmd: Option<&CStr>,
    set_paste: core::ffi::c_int,
    set_clip: core::ffi::c_int,
) {
    unsafe {
        if let Some(buf) = window_copy_pipe_run(wme, s, cmd) {
            window_copy_copy_buffer(wme, prefix, buf, set_paste, set_clip);
        }
    }
}
unsafe fn window_copy_copy_selection(
    wme: &mut window_mode_entry,
    prefix: Option<&CStr>,
    set_paste: core::ffi::c_int,
    set_clip: core::ffi::c_int,
) {
    unsafe {
        if let Some(buf) = window_copy_get_selection(wme) {
            window_copy_copy_buffer(wme, prefix, buf, set_paste, set_clip);
        }
    }
}
unsafe fn window_copy_append_selection(wme: &mut window_mode_entry) {
    unsafe {
        let pane = wme.pane_ref().expect("mode has a pane");
        let Some(buf) = window_copy_get_selection(wme) else {
            return;
        };
        if (global_options
            .get()
            .as_ref()
            .expect("global options are initialized"))
        .number(c"set-clipboard")
            != 0 as core::ffi::c_longlong
        {
            let display = wme
                .state
                .copy_mode_data_ref()
                .expect("copy mode has state")
                .screen
                .clone();
            let mut writer =
                RustScreenWriteCtx::on_shared_pane_screen(&display, Some(pane.clone()));
            writer.setselection(c"", &buf);
            writer.finish();
            notify_pane(c"pane-set-clipboard", pane.get());
        }
        let top = with_paste_buffers(|buffers| {
            buffers
                .top()
                .map(|buffer| (buffer.name.to_owned(), buffer.data.to_vec()))
        });
        let mut data = top.as_ref().map_or_else(Vec::new, |(_, data)| data.clone());
        data.extend_from_slice(&buf);
        if let Some((name, _)) = top {
            let _ = with_paste_buffers_mut(|buffers| buffers.set_named(name.as_c_str(), data));
        } else {
            let limit = paste_buffer_limit();
            with_paste_buffers_mut(|buffers| buffers.add_automatic(None, data, limit));
        }
    }
}
fn window_copy_copy_line(
    wme: &window_mode_entry,
    buf: &mut Vec<u8>,
    sy: u_int,
    mut sx: u_int,
    mut ex: u_int,
) {
    {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let gd = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        );
        let mut gc;
        let mut ud = utf8_data::default();
        let mut i: u_int;

        let mut wrapped: u_int = 0 as u_int;
        if sx > ex {
            return;
        }
        let gl = (gd).line_info(sy);
        if gl.flags & GRID_LINE_WRAPPED != 0 && gl.cells <= gd.width() {
            wrapped = 1 as u_int;
        }
        let xx: u_int = if wrapped != 0 {
            gl.cells
        } else {
            window_copy_find_length(wme, sy)
        };
        if ex > xx {
            ex = xx;
        }
        if sx > xx {
            sx = xx;
        }
        if sx < ex {
            i = sx;
            while i < ex {
                gc = gd.cell(i, sy);
                if !(gc.flags as core::ffi::c_int & GRID_FLAG_PADDING != 0) {
                    if gc.flags as core::ffi::c_int & GRID_FLAG_TAB != 0 {
                        utf8_set(&mut ud, '\t' as i32 as u_char);
                    } else {
                        utf8_copy(&mut ud, &gc.data);
                    }
                    if ud.size as core::ffi::c_int == 1 as core::ffi::c_int
                        && gc.attr as core::ffi::c_int & GRID_ATTR_CHARSET != 0
                        && let Some(acs) = RustAlternateCharacterSet
                            .unicode_for_key(ud.data[0 as core::ffi::c_int as usize])
                            .map(|value| value.into_bytes())
                        && acs.len() <= ud.data.len()
                    {
                        ud.size = acs.len() as u_char;
                        ud.data[..acs.len()].copy_from_slice(&acs);
                    }
                    buf.extend_from_slice(&ud.data[..ud.size as usize]);
                }
                i = i.wrapping_add(1);
            }
        }
        if wrapped == 0 || ex != xx {
            buf.push(b'\n');
        }
    }
}
unsafe fn window_copy_clear_selection(wme: &mut window_mode_entry) {
    unsafe {
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");

        data.screen.borrow_mut().clear_selection();
        data.cursordrag = CURSORDRAG_NONE;
        data.lineflag = LINE_SEL_NONE;
        data.selflag = SEL_CHAR;
        let py: u_int = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_add(data.cy)
        .wrapping_sub(data.oy);
        let (cx, cy, rectangle) = (data.cx, data.cy, data.rectflag);
        let px: u_int = window_copy_cursor_limit(wme, py, rectangle);
        if cx > px {
            window_copy_update_cursor(wme, px, cy);
        }
    }
}
fn window_copy_in_set(
    wme: &window_mode_entry,
    px: u_int,
    py: u_int,
    set: &CStr,
) -> core::ffi::c_int {
    {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .in_set(px, py, set)
    }
}
fn window_copy_find_length(wme: &window_mode_entry, py: u_int) -> u_int {
    {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .line_length(py)
    }
}
fn window_copy_cursor_limit(
    wme: &window_mode_entry,
    py: u_int,
    allow_onemore: core::ffi::c_int,
) -> u_int {
    {
        let oo = window_copy_window_options(wme);

        let len: u_int = window_copy_find_length(wme, py);
        if allow_onemore != 0 || (oo).number(c"mode-keys") != MODEKEY_VI as core::ffi::c_longlong {
            return len;
        }
        if len == 0 as u_int {
            return 0 as u_int;
        }
        len.wrapping_sub(1 as u_int)
    }
}
unsafe fn window_copy_cursor_start_of_line(wme: &mut window_mode_entry) {
    unsafe {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let back_s = data
            .backing
            .as_deref()
            .expect("copy mode has a backing screen");
        let mut px: u_int;
        let mut py: u_int;

        px = data.cx;
        let hsize: u_int = RustScreen::grid(back_s).history_size();
        py = hsize.wrapping_add(data.cy).wrapping_sub(data.oy);
        let oldy: u_int = data.cy;
        let mut gr = RustGridReader::start(RustScreen::grid(back_s), px, py);
        gr.start_of_line(true);
        (px, py) = gr.cursor();
        let oy = data.oy;
        window_copy_acquire_cursor_up(wme, hsize, oy, oldy, px, py);
    }
}
unsafe fn window_copy_cursor_back_to_indentation(wme: &mut window_mode_entry) {
    unsafe {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let back_s = data
            .backing
            .as_deref()
            .expect("copy mode has a backing screen");
        let mut px: u_int;
        let mut py: u_int;

        px = data.cx;
        let hsize: u_int = RustScreen::grid(back_s).history_size();
        py = hsize.wrapping_add(data.cy).wrapping_sub(data.oy);
        let oldy: u_int = data.cy;
        let mut gr = RustGridReader::start(RustScreen::grid(back_s), px, py);
        gr.back_to_indentation();
        (px, py) = gr.cursor();
        let oy = data.oy;
        window_copy_acquire_cursor_up(wme, hsize, oy, oldy, px, py);
    }
}
unsafe fn window_copy_cursor_end_of_line(wme: &mut window_mode_entry) {
    unsafe {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let back_s = data
            .backing
            .as_deref()
            .expect("copy mode has a backing screen");
        let mut px: u_int;
        let mut py: u_int;

        px = data.cx;
        let hsize: u_int = RustScreen::grid(back_s).history_size();
        py = hsize.wrapping_add(data.cy).wrapping_sub(data.oy);
        let oldy: u_int = data.cy;
        let mut gr = RustGridReader::start(RustScreen::grid(back_s), px, py);
        if data.screen.borrow().has_selection() && data.rectflag != 0 {
            gr.end_of_line(true, true);
        } else {
            gr.end_of_line(true, false);
        }
        (px, py) = gr.cursor();
        if !data.screen.borrow().has_selection() || data.rectflag == 0 {
            px = window_copy_cursor_limit(wme, py, 0 as core::ffi::c_int);
        }
        let oy = data.oy;
        let sy = RustScreen::grid(back_s).height();
        window_copy_acquire_cursor_down(wme, hsize, sy, oy, oldy, px, py, 0 as core::ffi::c_int);
    }
}
unsafe fn window_copy_other_end(wme: &mut window_mode_entry) {
    unsafe {
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        let s = &data.screen;
        let mut selx: u_int;
        let mut sely: u_int;

        let mut yy: u_int;
        let mut hsize: u_int;
        if !s.borrow().has_selection()
            && data.lineflag as core::ffi::c_uint
                == LINE_SEL_NONE as core::ffi::c_int as core::ffi::c_uint
        {
            return;
        }
        if data.lineflag as core::ffi::c_uint
            == LINE_SEL_LEFT_RIGHT as core::ffi::c_int as core::ffi::c_uint
        {
            data.lineflag = LINE_SEL_RIGHT_LEFT;
        } else if data.lineflag as core::ffi::c_uint
            == LINE_SEL_RIGHT_LEFT as core::ffi::c_int as core::ffi::c_uint
        {
            data.lineflag = LINE_SEL_LEFT_RIGHT;
        }
        match data.cursordrag {
            CURSORDRAG_NONE | CURSORDRAG_SEL => {
                data.cursordrag = CURSORDRAG_ENDSEL;
            }
            CURSORDRAG_ENDSEL => {
                data.cursordrag = CURSORDRAG_SEL;
            }
            _ => {}
        }
        selx = data.endselx;
        sely = data.endsely;
        if data.cursordrag as core::ffi::c_uint
            == CURSORDRAG_SEL as core::ffi::c_int as core::ffi::c_uint
        {
            selx = data.selx;
            sely = data.sely;
        }
        let cy: u_int = data.cy;
        yy = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_add(data.cy)
        .wrapping_sub(data.oy);
        data.cx = selx;
        hsize = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size();
        if sely < hsize.wrapping_sub(data.oy) {
            data.oy = hsize.wrapping_sub(sely);
            data.cy = 0 as u_int;
        } else if sely
            > hsize
                .wrapping_sub(data.oy)
                .wrapping_add(RustScreen::grid(&s.borrow()).height())
        {
            data.oy = hsize
                .wrapping_sub(sely)
                .wrapping_add(RustScreen::grid(&s.borrow()).height())
                .wrapping_sub(1 as u_int);
            data.cy = RustScreen::grid(&s.borrow()).height().wrapping_sub(1 as u_int);
        } else {
            data.cy = cy.wrapping_add(sely).wrapping_sub(yy);
        }
        yy = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_add(data.cy)
        .wrapping_sub(data.oy);
        let rectangle = data.rectflag;
        hsize = window_copy_cursor_limit(wme, yy, rectangle);
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        if data.cx > hsize {
            data.cx = hsize;
        }
        window_copy_update_selection(wme, 1 as core::ffi::c_int, 1 as core::ffi::c_int);
        window_copy_redraw_screen(wme);
    }
}
unsafe fn window_copy_cursor_left(wme: &mut window_mode_entry) {
    unsafe {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let back_s = data
            .backing
            .as_deref()
            .expect("copy mode has a backing screen");
        let mut px: u_int;
        let mut py: u_int;

        px = data.cx;
        let hsize: u_int = RustScreen::grid(back_s).history_size();
        py = hsize.wrapping_add(data.cy).wrapping_sub(data.oy);
        let oldy: u_int = data.cy;
        let mut gr = RustGridReader::start(RustScreen::grid(back_s), px, py);
        gr.left(true);
        (px, py) = gr.cursor();
        let oy = data.oy;
        window_copy_acquire_cursor_up(wme, hsize, oy, oldy, px, py);
    }
}
unsafe fn window_copy_cursor_right(wme: &mut window_mode_entry, all: core::ffi::c_int) {
    unsafe {
        let pane = wme.pane_ref().expect("mode has a pane");
        let _window = pane.window().expect("mode pane has a window");
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let oo = window_copy_window_options(wme);
        let back_s = data
            .backing
            .as_deref()
            .expect("copy mode has a backing screen");
        let mut px: u_int;
        let mut py: u_int;

        px = data.cx;
        let hsize: u_int = RustScreen::grid(back_s).history_size();
        py = hsize.wrapping_add(data.cy).wrapping_sub(data.oy);
        let oldy: u_int = data.cy;
        let onemore: core::ffi::c_int =
            ((oo).number(c"mode-keys") != MODEKEY_VI as core::ffi::c_longlong) as core::ffi::c_int;
        let mut gr = RustGridReader::start(RustScreen::grid(back_s), px, py);
        gr.right(true, all != 0, onemore != 0);
        (px, py) = gr.cursor();
        let oy = data.oy;
        let sy = RustScreen::grid(back_s).height();
        window_copy_acquire_cursor_down(wme, hsize, sy, oy, oldy, px, py, 0 as core::ffi::c_int);
    }
}
unsafe fn window_copy_cursor_up(wme: &mut window_mode_entry, scroll_only: core::ffi::c_int) {
    unsafe {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");

        let mut px: u_int;
        let mut py: u_int;

        let norectsel: core::ffi::c_int =
            (!data.screen.borrow().has_selection() || data.rectflag == 0) as core::ffi::c_int;
        let oy: u_int = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_add(data.cy)
        .wrapping_sub(data.oy);
        let ox: u_int = window_copy_find_length(wme, oy);
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        if norectsel != 0 && data.cx != ox {
            data.lastcx = data.cx;
            data.lastsx = ox;
        }
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        if data.lineflag as core::ffi::c_uint
            == LINE_SEL_LEFT_RIGHT as core::ffi::c_int as core::ffi::c_uint
            && oy == data.sely
        {
            window_copy_other_end(wme);
        }
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        if scroll_only != 0 || data.cy == 0 as u_int {
            if norectsel != 0 {
                data.cx = data.lastcx;
            }
            window_copy_scroll_down(wme, 1 as u_int);
            let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
            if scroll_only != 0 {
                if data.cy
                    == RustScreen::grid(&data.screen.borrow())
                        .height()
                        .wrapping_sub(1 as u_int)
                {
                    let cy = data.cy;
                    window_copy_redraw_lines(wme, cy, 1 as u_int);
                } else {
                    let cy = data.cy;
                    window_copy_redraw_lines(wme, cy, 2 as u_int);
                }
            }
        } else {
            if norectsel != 0 {
                let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
                let cy = data.cy;
                let cx = data.lastcx;
                window_copy_update_cursor(wme, cx, cy.wrapping_sub(1 as u_int));
            } else {
                let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
                let cy = data.cy;
                let cx = data.cx;
                window_copy_update_cursor(wme, cx, cy.wrapping_sub(1 as u_int));
            }
            if window_copy_update_selection(wme, 1 as core::ffi::c_int, 0 as core::ffi::c_int) != 0
            {
                let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
                if data.cy
                    == RustScreen::grid(&data.screen.borrow())
                        .height()
                        .wrapping_sub(1 as u_int)
                {
                    let cy = data.cy;
                    window_copy_redraw_lines(wme, cy, 1 as u_int);
                } else {
                    let cy = data.cy;
                    window_copy_redraw_lines(wme, cy, 2 as u_int);
                }
            }
        }
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        if norectsel != 0 {
            py = RustScreen::grid(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
            )
            .history_size()
            .wrapping_add(data.cy)
            .wrapping_sub(data.oy);
            px = window_copy_find_length(wme, py);
            if data.cx >= data.lastsx && data.cx != px || data.cx > px {
                let cy = data.cy;
                window_copy_update_cursor(wme, px, cy);
                if window_copy_update_selection(wme, 1 as core::ffi::c_int, 0 as core::ffi::c_int)
                    != 0
                {
                    let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
                    let cy = data.cy;
                    window_copy_redraw_lines(wme, cy, 1 as u_int);
                }
            }
        }
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        if data.lineflag as core::ffi::c_uint
            == LINE_SEL_LEFT_RIGHT as core::ffi::c_int as core::ffi::c_uint
        {
            py = RustScreen::grid(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
            )
            .history_size()
            .wrapping_add(data.cy)
            .wrapping_sub(data.oy);
            if data.rectflag != 0 {
                px = RustScreen::grid(
                    data.backing
                        .as_deref()
                        .expect("copy mode has a backing screen"),
                )
                .width();
            } else {
                px = window_copy_find_length(wme, py);
            }
            let cy = data.cy;
            window_copy_update_cursor(wme, px, cy);
            if window_copy_update_selection(wme, 1 as core::ffi::c_int, 0 as core::ffi::c_int) != 0
            {
                let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
                let cy = data.cy;
                window_copy_redraw_lines(wme, cy, 1 as u_int);
            }
        } else if data.lineflag as core::ffi::c_uint
            == LINE_SEL_RIGHT_LEFT as core::ffi::c_int as core::ffi::c_uint
        {
            let cy = data.cy;
            window_copy_update_cursor(wme, 0 as u_int, cy);
            if window_copy_update_selection(wme, 1 as core::ffi::c_int, 0 as core::ffi::c_int) != 0
            {
                let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
                let cy = data.cy;
                window_copy_redraw_lines(wme, cy, 1 as u_int);
            }
        }
    }
}
unsafe fn window_copy_cursor_down(wme: &mut window_mode_entry, scroll_only: core::ffi::c_int) {
    unsafe {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");

        let mut px: u_int;
        let mut py: u_int;

        let norectsel: core::ffi::c_int =
            (!data.screen.borrow().has_selection() || data.rectflag == 0) as core::ffi::c_int;
        let oy: u_int = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_add(data.cy)
        .wrapping_sub(data.oy);
        let ox: u_int = window_copy_find_length(wme, oy);
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        if norectsel != 0 && data.cx != ox {
            data.lastcx = data.cx;
            data.lastsx = ox;
        }
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        if data.lineflag as core::ffi::c_uint
            == LINE_SEL_RIGHT_LEFT as core::ffi::c_int as core::ffi::c_uint
            && oy == data.endsely
        {
            window_copy_other_end(wme);
        }
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        if scroll_only != 0
            || data.cy
                == RustScreen::grid(&data.screen.borrow())
                    .height()
                    .wrapping_sub(1 as u_int)
        {
            if norectsel != 0 {
                data.cx = data.lastcx;
            }
            window_copy_scroll_up(wme, 1 as u_int);
            let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
            if scroll_only != 0 && data.cy > 0 as u_int {
                let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
                let cy = data.cy;
                window_copy_redraw_lines(wme, cy.wrapping_sub(1 as u_int), 2 as u_int);
            }
        } else {
            if norectsel != 0 {
                let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
                let cy = data.cy;
                let cx = data.lastcx;
                window_copy_update_cursor(wme, cx, cy.wrapping_add(1 as u_int));
            } else {
                let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
                let cy = data.cy;
                let cx = data.cx;
                window_copy_update_cursor(wme, cx, cy.wrapping_add(1 as u_int));
            }
            if window_copy_update_selection(wme, 1 as core::ffi::c_int, 0 as core::ffi::c_int) != 0
            {
                let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
                let cy = data.cy;
                window_copy_redraw_lines(wme, cy.wrapping_sub(1 as u_int), 2 as u_int);
            }
        }
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        if norectsel != 0 {
            py = RustScreen::grid(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
            )
            .history_size()
            .wrapping_add(data.cy)
            .wrapping_sub(data.oy);
            px = window_copy_find_length(wme, py);
            if data.cx >= data.lastsx && data.cx != px || data.cx > px {
                let cy = data.cy;
                window_copy_update_cursor(wme, px, cy);
                if window_copy_update_selection(wme, 1 as core::ffi::c_int, 0 as core::ffi::c_int)
                    != 0
                {
                    let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
                    let cy = data.cy;
                    window_copy_redraw_lines(wme, cy, 1 as u_int);
                }
            }
        }
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        if data.lineflag as core::ffi::c_uint
            == LINE_SEL_LEFT_RIGHT as core::ffi::c_int as core::ffi::c_uint
        {
            py = RustScreen::grid(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
            )
            .history_size()
            .wrapping_add(data.cy)
            .wrapping_sub(data.oy);
            if data.rectflag != 0 {
                px = RustScreen::grid(
                    data.backing
                        .as_deref()
                        .expect("copy mode has a backing screen"),
                )
                .width();
            } else {
                px = window_copy_find_length(wme, py);
            }
            let cy = data.cy;
            window_copy_update_cursor(wme, px, cy);
            if window_copy_update_selection(wme, 1 as core::ffi::c_int, 0 as core::ffi::c_int) != 0
            {
                let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
                let cy = data.cy;
                window_copy_redraw_lines(wme, cy, 1 as u_int);
            }
        } else if data.lineflag as core::ffi::c_uint
            == LINE_SEL_RIGHT_LEFT as core::ffi::c_int as core::ffi::c_uint
        {
            let cy = data.cy;
            window_copy_update_cursor(wme, 0 as u_int, cy);
            if window_copy_update_selection(wme, 1 as core::ffi::c_int, 0 as core::ffi::c_int) != 0
            {
                let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
                let cy = data.cy;
                window_copy_redraw_lines(wme, cy, 1 as u_int);
            }
        }
    }
}
unsafe fn window_copy_cursor_jump(wme: &mut window_mode_entry) {
    unsafe {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let Some(jc) = data.jumpchar.as_ref() else {
            return;
        };
        let back_s = data
            .backing
            .as_deref()
            .expect("copy mode has a backing screen");
        let mut px: u_int;
        let mut py: u_int;

        px = data.cx.wrapping_add(1 as u_int);
        let hsize: u_int = RustScreen::grid(back_s).history_size();
        py = hsize.wrapping_add(data.cy).wrapping_sub(data.oy);
        let oldy: u_int = data.cy;
        let mut gr = RustGridReader::start(RustScreen::grid(back_s), px, py);
        if gr.jump(&jc.data[..jc.size as usize]) {
            (px, py) = gr.cursor();
            let oy = data.oy;
            let sy = RustScreen::grid(back_s).height();
            window_copy_acquire_cursor_down(
                wme,
                hsize,
                sy,
                oy,
                oldy,
                px,
                py,
                0 as core::ffi::c_int,
            );
        }
    }
}
unsafe fn window_copy_cursor_jump_back(wme: &mut window_mode_entry) {
    unsafe {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let Some(jc) = data.jumpchar.as_ref() else {
            return;
        };
        let back_s = data
            .backing
            .as_deref()
            .expect("copy mode has a backing screen");
        let mut px: u_int;
        let mut py: u_int;

        px = data.cx;
        let hsize: u_int = RustScreen::grid(back_s).history_size();
        py = hsize.wrapping_add(data.cy).wrapping_sub(data.oy);
        let oldy: u_int = data.cy;
        let mut gr = RustGridReader::start(RustScreen::grid(back_s), px, py);
        gr.left(false);
        if gr.jump_back(&jc.data[..jc.size as usize]) {
            (px, py) = gr.cursor();
            let oy = data.oy;
            window_copy_acquire_cursor_up(wme, hsize, oy, oldy, px, py);
        }
    }
}
unsafe fn window_copy_cursor_jump_to(wme: &mut window_mode_entry) {
    unsafe {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let Some(jc) = data.jumpchar.as_ref() else {
            return;
        };
        let back_s = data
            .backing
            .as_deref()
            .expect("copy mode has a backing screen");
        let mut px: u_int;
        let mut py: u_int;

        px = data.cx.wrapping_add(2 as u_int);
        let hsize: u_int = RustScreen::grid(back_s).history_size();
        py = hsize.wrapping_add(data.cy).wrapping_sub(data.oy);
        let oldy: u_int = data.cy;
        let mut gr = RustGridReader::start(RustScreen::grid(back_s), px, py);
        if gr.jump(&jc.data[..jc.size as usize]) {
            gr.left(true);
            (px, py) = gr.cursor();
            let oy = data.oy;
            let sy = RustScreen::grid(back_s).height();
            window_copy_acquire_cursor_down(
                wme,
                hsize,
                sy,
                oy,
                oldy,
                px,
                py,
                0 as core::ffi::c_int,
            );
        }
    }
}
unsafe fn window_copy_cursor_jump_to_back(wme: &mut window_mode_entry) {
    unsafe {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let Some(jc) = data.jumpchar.as_ref() else {
            return;
        };
        let pane = wme.pane_ref().expect("mode has a pane");
        let _window = pane.window().expect("mode pane has a window");
        let oo = window_copy_window_options(wme);
        let back_s = data
            .backing
            .as_deref()
            .expect("copy mode has a backing screen");
        let mut px: u_int;
        let mut py: u_int;

        px = data.cx;
        let hsize: u_int = RustScreen::grid(back_s).history_size();
        py = hsize.wrapping_add(data.cy).wrapping_sub(data.oy);
        let oldy: u_int = data.cy;
        let onemore: core::ffi::c_int =
            ((oo).number(c"mode-keys") != MODEKEY_VI as core::ffi::c_longlong) as core::ffi::c_int;
        let mut gr = RustGridReader::start(RustScreen::grid(back_s), px, py);
        gr.left(false);
        gr.left(false);
        if gr.jump_back(&jc.data[..jc.size as usize]) {
            gr.right(true, false, onemore != 0);
            (px, py) = gr.cursor();
            let oy = data.oy;
            window_copy_acquire_cursor_up(wme, hsize, oy, oldy, px, py);
        }
    }
}
unsafe fn window_copy_cursor_next_word(wme: &mut window_mode_entry, separators: &CStr) {
    unsafe {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let back_s = data
            .backing
            .as_deref()
            .expect("copy mode has a backing screen");
        let mut px: u_int;
        let mut py: u_int;

        px = data.cx;
        let hsize: u_int = RustScreen::grid(back_s).history_size();
        py = hsize.wrapping_add(data.cy).wrapping_sub(data.oy);
        let oldy: u_int = data.cy;
        let mut gr = RustGridReader::start(RustScreen::grid(back_s), px, py);
        gr.next_word(separators);
        (px, py) = gr.cursor();
        let oy = data.oy;
        let sy = RustScreen::grid(back_s).height();
        window_copy_acquire_cursor_down(wme, hsize, sy, oy, oldy, px, py, 0 as core::ffi::c_int);
    }
}
fn window_copy_cursor_next_word_end_pos(
    wme: &window_mode_entry,
    separators: &CStr,
) -> (u_int, u_int) {
    {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let oo = window_copy_window_options(wme);
        let back_s = data
            .backing
            .as_deref()
            .expect("copy mode has a backing screen");
        let mut px: u_int;
        let mut py: u_int;

        px = data.cx;
        let hsize: u_int = RustScreen::grid(back_s).history_size();
        py = hsize.wrapping_add(data.cy).wrapping_sub(data.oy);
        let mut gr = RustGridReader::start(RustScreen::grid(back_s), px, py);
        if (oo).number(c"mode-keys") == MODEKEY_VI as core::ffi::c_longlong {
            if !gr.in_set(WHITESPACE) {
                gr.right(false, false, false);
            }
            gr.next_word_end(separators);
            gr.left(true);
        } else {
            gr.next_word_end(separators);
        }
        (px, py) = gr.cursor();
        (px, py)
    }
}
unsafe fn window_copy_cursor_next_word_end(
    wme: &mut window_mode_entry,
    separators: &CStr,
    no_reset: core::ffi::c_int,
) {
    unsafe {
        let pane = wme.pane_ref().expect("mode has a pane");
        let _window = pane.window().expect("mode pane has a window");
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let oo = window_copy_window_options(wme);
        let back_s = data
            .backing
            .as_deref()
            .expect("copy mode has a backing screen");
        let mut px: u_int;
        let mut py: u_int;

        px = data.cx;
        let hsize: u_int = RustScreen::grid(back_s).history_size();
        py = hsize.wrapping_add(data.cy).wrapping_sub(data.oy);
        let oldy: u_int = data.cy;
        let mut gr = RustGridReader::start(RustScreen::grid(back_s), px, py);
        if (oo).number(c"mode-keys") == MODEKEY_VI as core::ffi::c_longlong {
            if !gr.in_set(WHITESPACE) {
                gr.right(false, false, false);
            }
            gr.next_word_end(separators);
            gr.left(true);
        } else {
            gr.next_word_end(separators);
        }
        (px, py) = gr.cursor();
        let oy = data.oy;
        let sy = RustScreen::grid(back_s).height();
        window_copy_acquire_cursor_down(wme, hsize, sy, oy, oldy, px, py, no_reset);
    }
}
fn window_copy_cursor_previous_word_pos(
    wme: &window_mode_entry,
    separators: &CStr,
) -> (u_int, u_int) {
    {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let back_s = data
            .backing
            .as_deref()
            .expect("copy mode has a backing screen");
        let mut px: u_int;
        let mut py: u_int;

        px = data.cx;
        let hsize: u_int = RustScreen::grid(back_s).history_size();
        py = hsize.wrapping_add(data.cy).wrapping_sub(data.oy);
        let mut gr = RustGridReader::start(RustScreen::grid(back_s), px, py);
        gr.previous_word(separators, false, true);
        (px, py) = gr.cursor();
        (px, py)
    }
}
unsafe fn window_copy_cursor_previous_word(
    wme: &mut window_mode_entry,
    separators: &CStr,
    already: core::ffi::c_int,
) {
    unsafe {
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let pane = wme.pane_ref().expect("mode has a pane");
        let window = pane.window().expect("mode pane has a window");
        let back_s = data
            .backing
            .as_deref()
            .expect("copy mode has a backing screen");
        let mut px: u_int;
        let mut py: u_int;

        let stop_at_eol: core::ffi::c_int =
            if window.options().number(c"mode-keys") == MODEKEY_EMACS as core::ffi::c_longlong {
                1 as core::ffi::c_int
            } else {
                0 as core::ffi::c_int
            };
        px = data.cx;
        let hsize: u_int = RustScreen::grid(back_s).history_size();
        py = hsize.wrapping_add(data.cy).wrapping_sub(data.oy);
        let oldy: u_int = data.cy;
        let mut gr = RustGridReader::start(RustScreen::grid(back_s), px, py);
        gr.previous_word(separators, already != 0, stop_at_eol != 0);
        (px, py) = gr.cursor();
        let oy = data.oy;
        window_copy_acquire_cursor_up(wme, hsize, oy, oldy, px, py);
    }
}
unsafe fn window_copy_cursor_prompt(
    wme: &mut window_mode_entry,
    direction: core::ffi::c_int,
    start_output: core::ffi::c_int,
) {
    unsafe {
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        let s = data
            .backing
            .as_deref()
            .expect("copy mode has a backing screen");
        let gd = RustScreen::grid(s);
        let end_line: u_int;
        let mut line: u_int = gd.history_size().wrapping_sub(data.oy).wrapping_add(data.cy);
        let add: core::ffi::c_int;

        let line_flag: core::ffi::c_int = if start_output != 0 {
            GRID_LINE_START_OUTPUT
        } else {
            GRID_LINE_START_PROMPT
        };
        if direction == 0 as core::ffi::c_int {
            add = -(1 as core::ffi::c_int);
            end_line = 0 as u_int;
        } else {
            add = 1 as core::ffi::c_int;
            end_line = gd.history_size().wrapping_add(gd.height()).wrapping_sub(1 as u_int);
        }
        if line == end_line {
            return;
        }
        loop {
            if line == end_line {
                return;
            }
            line = line.wrapping_add(add as u_int);
            if (gd).line_info(line).flags & line_flag != 0 {
                break;
            }
        }
        data.cx = 0 as u_int;
        if line > gd.history_size() {
            data.cy = line.wrapping_sub(gd.history_size());
            data.oy = 0 as u_int;
        } else {
            data.cy = 0 as u_int;
            data.oy = gd.history_size().wrapping_sub(line);
        }
        window_copy_update_selection(wme, 1 as core::ffi::c_int, 0 as core::ffi::c_int);
        window_copy_redraw_screen(wme);
    }
}
unsafe fn window_copy_scroll_up(wme: &mut window_mode_entry, mut ny: u_int) {
    unsafe {
        let mut pane = wme.pane_ref().expect("mode has a pane");
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        if data.oy < ny {
            ny = data.oy;
        }
        if ny == 0 as u_int {
            return;
        }
        data.oy = data.oy.wrapping_sub(ny);
        if !data.searchmark.is_empty() && data.timeout == 0 {
            let regex = data.searchregex;
            window_copy_search_marks(wme, None, regex, 1 as core::ffi::c_int);
        }
        window_copy_update_selection(wme, 0 as core::ffi::c_int, 0 as core::ffi::c_int);
        let s = wme
            .state
            .copy_mode_data_ref()
            .expect("copy mode has state")
            .screen
            .clone();
        let (sx, sy) = s.borrow().size();
        if window_copy_line_numbers_active(wme) != 0 {
            if window_copy_line_number_mode(wme)
                != WINDOW_COPY_LINE_NUMBERS_ABSOLUTE as core::ffi::c_int
            {
                window_copy_redraw_screen(wme);
                return;
            }
            let mut writer = RustScreenWriteCtx::on_shared_screen(&s);
            writer.cursormove(
                0 as core::ffi::c_int,
                0 as core::ffi::c_int,
                0 as core::ffi::c_int,
            );
            writer.deleteline(ny, 8 as u_int);
            window_copy_write_lines(wme, &mut writer, sy.wrapping_sub(ny), ny);
            window_copy_write_line(wme, &mut writer, 0 as u_int);
            if sy > 1 as u_int {
                window_copy_write_line(wme, &mut writer, 1 as u_int);
            }
            if sy > 3 as u_int {
                window_copy_write_line(wme, &mut writer, sy.wrapping_sub(2 as u_int));
            }
            if s.borrow().has_selection() && sy > ny {
                window_copy_write_line(
                    wme,
                    &mut writer,
                    sy.wrapping_sub(ny).wrapping_sub(1 as u_int),
                );
            }
            let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
            let x = window_copy_cursor_offset(wme, data.cx, sx) as core::ffi::c_int;
            let y = data.cy as core::ffi::c_int;
            writer.cursormove(x, y, 0 as core::ffi::c_int);
            writer.finish();
            pane.get_mut().expect("mode pane is live").request_full_redraw();
            return;
        }
        let mut writer = RustScreenWriteCtx::on_shared_pane_screen(&s, Some(pane.clone()));
        writer.cursormove(
            0 as core::ffi::c_int,
            0 as core::ffi::c_int,
            0 as core::ffi::c_int,
        );
        writer.deleteline(ny, 8 as u_int);
        window_copy_write_lines(wme, &mut writer, sy.wrapping_sub(ny), ny);
        window_copy_write_line(wme, &mut writer, 0 as u_int);
        if sy > 1 as u_int {
            window_copy_write_line(wme, &mut writer, 1 as u_int);
        }
        if sy > 3 as u_int {
            window_copy_write_line(wme, &mut writer, sy.wrapping_sub(2 as u_int));
        }
        if s.borrow().has_selection() && sy > ny {
            window_copy_write_line(
                wme,
                &mut writer,
                sy.wrapping_sub(ny).wrapping_sub(1 as u_int),
            );
        }
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let x = window_copy_cursor_offset(wme, data.cx, sx) as core::ffi::c_int;
        let y = data.cy as core::ffi::c_int;
        writer.cursormove(x, y, 0 as core::ffi::c_int);
        writer.finish();
        pane.get_mut().expect("mode pane is live").request_scrollbar_redraw();
    }
}
unsafe fn window_copy_scroll_down(wme: &mut window_mode_entry, mut ny: u_int) {
    unsafe {
        let mut pane = wme.pane_ref().expect("mode has a pane");
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        if ny
            > RustScreen::grid(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
            )
            .history_size()
        {
            return;
        }
        if data.oy
            > RustScreen::grid(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
            )
            .history_size()
            .wrapping_sub(ny)
        {
            ny = RustScreen::grid(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
            )
            .history_size()
            .wrapping_sub(data.oy);
        }
        if ny == 0 as u_int {
            return;
        }
        data.oy = data.oy.wrapping_add(ny);
        if !data.searchmark.is_empty() && data.timeout == 0 {
            let regex = data.searchregex;
            window_copy_search_marks(wme, None, regex, 1 as core::ffi::c_int);
        }
        window_copy_update_selection(wme, 0 as core::ffi::c_int, 0 as core::ffi::c_int);
        let s = wme
            .state
            .copy_mode_data_ref()
            .expect("copy mode has state")
            .screen
            .clone();
        let (sx, sy) = s.borrow().size();
        if window_copy_line_numbers_active(wme) != 0 {
            if window_copy_line_number_mode(wme)
                != WINDOW_COPY_LINE_NUMBERS_ABSOLUTE as core::ffi::c_int
            {
                window_copy_redraw_screen(wme);
                return;
            }
            let mut writer = RustScreenWriteCtx::on_shared_screen(&s);
            writer.cursormove(
                0 as core::ffi::c_int,
                0 as core::ffi::c_int,
                0 as core::ffi::c_int,
            );
            writer.insertline(ny, 8 as u_int);
            window_copy_write_lines(wme, &mut writer, 0 as u_int, ny);
            if s.borrow().has_selection() && sy > ny {
                window_copy_write_line(wme, &mut writer, ny);
            } else if ny == 1 as u_int {
                window_copy_write_line(wme, &mut writer, 1 as u_int);
            }
            let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
            let x = window_copy_cursor_offset(wme, data.cx, sx) as core::ffi::c_int;
            let y = data.cy as core::ffi::c_int;
            writer.cursormove(x, y, 0 as core::ffi::c_int);
            writer.finish();
            pane.get_mut().expect("mode pane is live").request_full_redraw();
            return;
        }
        let mut writer = RustScreenWriteCtx::on_shared_pane_screen(&s, Some(pane.clone()));
        writer.cursormove(
            0 as core::ffi::c_int,
            0 as core::ffi::c_int,
            0 as core::ffi::c_int,
        );
        writer.insertline(ny, 8 as u_int);
        window_copy_write_lines(wme, &mut writer, 0 as u_int, ny);
        if s.borrow().has_selection() && sy > ny {
            window_copy_write_line(wme, &mut writer, ny);
        } else if ny == 1 as u_int {
            window_copy_write_line(wme, &mut writer, 1 as u_int);
        }
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        let x = window_copy_cursor_offset(wme, data.cx, sx) as core::ffi::c_int;
        let y = data.cy as core::ffi::c_int;
        writer.cursormove(x, y, 0 as core::ffi::c_int);
        writer.finish();
        pane.get_mut().expect("mode pane is live").request_scrollbar_redraw();
    }
}
unsafe fn window_copy_rectangle_set(wme: &mut window_mode_entry, rectflag: core::ffi::c_int) {
    unsafe {
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");

        data.rectflag = rectflag;
        let py: u_int = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_add(data.cy)
        .wrapping_sub(data.oy);
        let (cx, cy) = (data.cx, data.cy);
        let px: u_int = window_copy_cursor_limit(wme, py, rectflag);
        if cx > px {
            window_copy_update_cursor(wme, px, cy);
        }
        window_copy_update_selection(wme, 1 as core::ffi::c_int, 0 as core::ffi::c_int);
        window_copy_redraw_screen(wme);
    }
}
unsafe fn window_copy_move_mouse(m: &mouse_event) {
    unsafe {
        let mut x: u_int = 0;
        let mut y: u_int = 0;
        let Some((_, _, mut pane)) = cmd_mouse_pane(m) else {
            return;
        };
        let Some(wp) = pane.get_mut() else {
            return;
        };
        let mouse_at = cmd_mouse_at(wp, m, 0);
        let Some(wme) = (&mut *wp).active_mode_mut().map(|mode| mode.into_entry()) else {
            return;
        };
        if wme.mode() != WindowMode::Copy && wme.mode() != WindowMode::View {
            return;
        }
        if match mouse_at {
            Some((at_x, at_y)) => {
                (x, y) = (at_x, at_y);
                false
            }
            None => true,
        } {
            return;
        }
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        x = window_copy_cursor_unoffset(wme, x, RustScreen::grid(&data.screen.borrow()).width());
        window_copy_update_cursor(wme, x, y);
    }
}
pub unsafe fn window_copy_start_drag(c: Option<&mut client>, m: &mouse_event) {
    unsafe {
        let Some(c) = c else {
            return;
        };
        let mut x: u_int = 0;
        let mut y: u_int = 0;

        let Some((_, _, mut pane)) = cmd_mouse_pane(m) else {
            return;
        };
        let Some(wp) = pane.get_mut() else {
            return;
        };
        let mouse_at = cmd_mouse_at(wp, m, 1);
        let Some(wme) = (&mut *wp).active_mode_mut().map(|mode| mode.into_entry()) else {
            return;
        };
        if wme.mode() != WindowMode::Copy && wme.mode() != WindowMode::View {
            return;
        }
        if match mouse_at {
            Some((at_x, at_y)) => {
                (x, y) = (at_x, at_y);
                false
            }
            None => true,
        } {
            return;
        }
        c.tty.mouse_drag_update = Some(std::rc::Rc::new(|c, m| window_copy_drag_update(c, m)));
        c.tty.mouse_drag_release = Some(std::rc::Rc::new(|c, m| window_copy_drag_release(c, m)));
        let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
        x = window_copy_cursor_unoffset(wme, x, RustScreen::grid(&data.screen.borrow()).width());
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        let yg: u_int = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_add(y)
        .wrapping_sub(data.oy);
        if x < data.selrx || x > data.endselrx || yg != data.selry {
            data.selflag = SEL_CHAR;
        }
        match data.selflag {
            SEL_WORD => {
                if data.separators.is_some() {
                    window_copy_update_cursor(wme, x, y);
                    let data = wme.state.copy_mode_data_ref().expect("copy mode has state");
                    (x, y) = window_copy_cursor_previous_word_pos(
                        wme,
                        data.separators.as_deref().unwrap_or(c""),
                    );
                    y = y.wrapping_sub(
                        RustScreen::grid(
                            data.backing
                                .as_deref()
                                .expect("copy mode has a backing screen"),
                        )
                        .history_size()
                        .wrapping_sub(data.oy),
                    );
                }
                window_copy_update_cursor(wme, x, y);
            }
            SEL_LINE => {
                window_copy_update_cursor(wme, 0 as u_int, y);
            }
            SEL_CHAR => {
                window_copy_update_cursor(wme, x, y);
                window_copy_start_selection(wme);
            }
            _ => {}
        }
        window_copy_redraw_screen(wme);
        window_copy_drag_update(c, m);
    }
}
unsafe fn window_copy_drag_update(_c: &mut client, m: &mouse_event) {
    unsafe {
        let mut x: u_int = 0;
        let mut y: u_int = 0;

        let tv = timeval::from_usecs(WINDOW_COPY_DRAG_REPEAT_TIME as __suseconds_t);
        let Some((_, _, mut pane)) = cmd_mouse_pane(m) else {
            return;
        };
        let Some(wp) = pane.get_mut() else {
            return;
        };
        let mouse_at = cmd_mouse_at(wp, m, 0);
        let Some(wme) = (&mut *wp).active_mode_mut().map(|mode| mode.into_entry()) else {
            return;
        };
        if wme.mode() != WindowMode::Copy && wme.mode() != WindowMode::View {
            return;
        }
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        data.dragtimer.disarm();
        if match mouse_at {
            Some((at_x, at_y)) => {
                (x, y) = (at_x, at_y);
                false
            }
            None => true,
        } {
            return;
        }
        let sx = RustScreen::grid(&data.screen.borrow()).width();
        let old_cx: u_int = data.cx;
        let old_cy: u_int = data.cy;
        x = window_copy_cursor_unoffset(wme, x, sx);
        window_copy_update_cursor(wme, x, y);
        if window_copy_update_selection(wme, 1 as core::ffi::c_int, 0 as core::ffi::c_int) != 0 {
            window_copy_redraw_selection(wme, old_cy);
        }
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");
        if old_cy != data.cy || old_cx == data.cx {
            if y == 0 as u_int {
                data.dragtimer.arm(tv);
                window_copy_cursor_up(wme, 1 as core::ffi::c_int);
            } else if y
                == RustScreen::grid(&data.screen.borrow())
                    .height()
                    .wrapping_sub(1 as u_int)
            {
                data.dragtimer.arm(tv);
                window_copy_cursor_down(wme, 1 as core::ffi::c_int);
            }
        }
    }
}
unsafe fn window_copy_drag_release(c: &mut client, m: &mouse_event) {
    unsafe {
        let Some((_, _, mut pane)) = cmd_mouse_pane(m) else {
            return;
        };
        let update = {
            let Some(wme) = pane.get().and_then(|pane| (pane).active_mode()) else {
                return;
            };
            if !matches!(wme.mode(), WindowMode::Copy | WindowMode::View) {
                return;
            }
            window_copy_line_numbers_active(wme) != 0
        };
        if update {
            window_copy_drag_update(c, m);
        }
        if let Some(wme) = pane
            .get_mut()
            .and_then(|pane| (pane).active_mode_mut().map(|mode| mode.into_entry()))
            && matches!(wme.mode(), WindowMode::Copy | WindowMode::View)
        {
            wme.state
                .copy_mode_data_mut()
                .expect("copy mode has state")
                .dragtimer
                .disarm();
        }
    }
}
unsafe fn window_copy_jump_to_mark(wme: &mut window_mode_entry) {
    unsafe {
        let data = wme.state.copy_mode_data_mut().expect("copy mode has state");

        let tmx: u_int = data.cx;
        let tmy: u_int = RustScreen::grid(
            data.backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size()
        .wrapping_add(data.cy)
        .wrapping_sub(data.oy);
        data.cx = data.mx;
        if data.my
            < RustScreen::grid(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
            )
            .history_size()
        {
            data.cy = 0 as u_int;
            data.oy = RustScreen::grid(
                data.backing
                    .as_deref()
                    .expect("copy mode has a backing screen"),
            )
            .history_size()
            .wrapping_sub(data.my);
        } else {
            data.cy = data.my.wrapping_sub(
                RustScreen::grid(
                    data.backing
                        .as_deref()
                        .expect("copy mode has a backing screen"),
                )
                .history_size(),
            );
            data.oy = 0 as u_int;
        }
        data.mx = tmx;
        data.my = tmy;
        data.showmark = 1 as core::ffi::c_int;
        window_copy_update_selection(wme, 0 as core::ffi::c_int, 0 as core::ffi::c_int);
        window_copy_redraw_screen(wme);
    }
}
unsafe fn window_copy_acquire_cursor_up(
    wme: &mut window_mode_entry,
    hsize: u_int,
    oy: u_int,
    oldy: u_int,
    px: u_int,
    py: u_int,
) {
    unsafe {
        let cy: u_int;

        let mut ny: u_int;
        let nd: u_int;
        let yy: u_int = hsize.wrapping_sub(oy);
        if py < yy {
            ny = yy.wrapping_sub(py);
            cy = 0 as u_int;
            nd = 1 as u_int;
        } else {
            ny = 0 as u_int;
            cy = py.wrapping_sub(yy);
            nd = oldy.wrapping_sub(cy).wrapping_add(1 as u_int);
        }
        while ny > 0 as u_int {
            window_copy_cursor_up(wme, 1 as core::ffi::c_int);
            ny = ny.wrapping_sub(1);
        }
        window_copy_update_cursor(wme, px, cy);
        if window_copy_update_selection(wme, 1 as core::ffi::c_int, 0 as core::ffi::c_int) != 0 {
            window_copy_redraw_lines(wme, cy, nd);
        }
    }
}
#[allow(clippy::too_many_arguments)]
unsafe fn window_copy_acquire_cursor_down(
    wme: &mut window_mode_entry,
    hsize: u_int,
    sy: u_int,
    oy: u_int,
    mut oldy: u_int,
    px: u_int,
    py: u_int,
    no_reset: core::ffi::c_int,
) {
    unsafe {
        let mut ny: u_int;
        let nd: u_int;
        let cy: u_int = py.wrapping_sub(hsize).wrapping_add(oy);
        let yy: u_int = sy.wrapping_sub(1 as u_int);
        if cy > yy {
            ny = cy.wrapping_sub(yy);
            oldy = yy;
            nd = 1 as u_int;
        } else {
            ny = 0 as u_int;
            nd = cy.wrapping_sub(oldy).wrapping_add(1 as u_int);
        }
        while ny > 0 as u_int {
            window_copy_cursor_down(wme, 1 as core::ffi::c_int);
            ny = ny.wrapping_sub(1);
        }
        if cy > yy {
            window_copy_update_cursor(wme, px, yy);
        } else {
            window_copy_update_cursor(wme, px, cy);
        }
        if window_copy_update_selection(wme, 1 as core::ffi::c_int, no_reset) != 0 {
            window_copy_redraw_lines(wme, oldy, nd);
        }
    }
}

use crate::screen::RustScreen;

#[cfg(test)]
#[path = "copy_tests.rs"]
mod tests;

impl RustWindowPaneWeak {
    /// # Safety
    /// Exclude conflicting pane and copy-mode access during this operation.
    pub(crate) unsafe fn copy_line_numbers(&self, enabled: core::ffi::c_int) -> bool {
        let Some(mut owner) = self.upgrade() else {
            return false;
        };
        unsafe { window_copy_set_line_numbers(owner.as_pane_mut(), enabled) };
        true
    }
    /// # Safety
    /// Exclude conflicting pane and copy-mode access during this operation.
    pub(crate) unsafe fn copy_page_up(&self, half_page: core::ffi::c_int) -> bool {
        let Some(mut owner) = self.upgrade() else {
            return false;
        };
        unsafe { window_copy_pageup(owner.as_pane_mut(), half_page) };
        true
    }
    /// # Safety
    /// Exclude conflicting pane and copy-mode access during this operation.
    pub(crate) unsafe fn copy_page_down(
        &self,
        half_page: core::ffi::c_int,
        exit: core::ffi::c_int,
    ) -> bool {
        let Some(mut owner) = self.upgrade() else {
            return false;
        };
        unsafe { window_copy_pagedown(owner.as_pane_mut(), half_page, exit) };
        true
    }
    /// # Safety
    /// Exclude conflicting pane and copy-mode access during this operation.
    pub(crate) unsafe fn copy_scroll(
        &self,
        position: core::ffi::c_int,
        mouse_y: u_int,
        offset_y: u_int,
        exit: core::ffi::c_int,
    ) -> bool {
        let Some(mut owner) = self.upgrade() else {
            return false;
        };
        unsafe { window_copy_scroll(owner.as_pane_mut(), position, mouse_y, offset_y, exit) };
        true
    }
}

impl ClientRef {
    /// # Safety
    /// Exclude conflicting client access, including reentrant callbacks, during this operation.
    pub(crate) unsafe fn start_copy_drag(c: Option<&mut Self>, m: &mouse_event) {
        unsafe { window_copy_start_drag(c.map(|c| c.as_client_mut()), m) }
    }
}

#[cfg(test)]
pub const WINDOW_COPY_LINE_NUMBERS_RELATIVE: window_copy_line_numbers = 3;
