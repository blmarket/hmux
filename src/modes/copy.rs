use crate::arguments::{args_count, args_has, args_parse, args_string};
use crate::cmd::{cmd_mouse_at, cmd_mouse_pane};
use crate::compat::strtonum;
use crate::environ::environ_t;
use crate::ffi::{
    __ctype_tolower_loc, abs, regcomp, regexec, regfree, strcasecmp, strchr, strcmp, strcspn,
    strncmp,
};
use crate::fmt_args;
use crate::fmt_engine::{FmtArg, format_alloc};
use crate::format::format_draw;
use crate::format::{
    format_add, format_add_cb, format_create_defaults, format_expand, format_get_pane,
    format_grid_hyperlink, format_grid_line, format_grid_word, format_single,
};
use crate::grid::{
    grid_default_cell, grid_duplicate_lines, grid_get_cell, grid_get_line, grid_in_set,
    grid_line_length, grid_peek_line, grid_unwrap_position, grid_wrap_position,
};
use crate::grid::{
    grid_reader_cursor_back_to_indentation, grid_reader_cursor_end_of_line,
    grid_reader_cursor_jump, grid_reader_cursor_jump_back, grid_reader_cursor_left,
    grid_reader_cursor_next_word, grid_reader_cursor_next_word_end,
    grid_reader_cursor_previous_word, grid_reader_cursor_right, grid_reader_cursor_start_of_line,
    grid_reader_get_cursor, grid_reader_in_set, grid_reader_start,
};
use crate::input::InputOwner;
use crate::input::{ictx_mut, input_free_box, input_init, input_parse_screen};
use crate::job::{job_get_event, job_run};
use crate::log::{fatalx, log_debug};
use crate::notify::notify_pane;
use crate::options::{options_get_number, options_get_string};
use crate::paste::{paste_add, paste_set};
use crate::paste::{paste_buffer_data, paste_get_top};
use crate::reactor::Timer;
use crate::screen::screen_write_strlen;
use crate::screen::{
    screen_clear_selection, screen_free, screen_grid, screen_grid_mut, screen_grid_ptr,
    screen_hide_selection, screen_init, screen_resize, screen_resize_cursor,
    screen_set_default_cursor, screen_set_selection,
};
use crate::screen::{
    screen_write_carriagereturn, screen_write_cell, screen_write_cursormove,
    screen_write_deleteline, screen_write_insertline, screen_write_linefeed, screen_write_nputs,
    screen_write_putc, screen_write_setselection, screen_write_start, screen_write_start_pane,
    screen_write_stop, screen_write_vnputs,
};
use crate::session::session_options;
use crate::status::status_message_set;
use crate::style::style_apply;
use crate::terminfo::tty_acs_get;
use crate::text::{utf8_copy, utf8_set, utf8_to_data, utf8_vec_fromcstr};
use crate::tmux::get_timer;
use crate::tmux::{global_options, global_w_options};
use crate::tty::tty_window_offset;
pub use crate::types::*;
use crate::window::window_pane_current_mode;
use crate::window::window_pane_find_by_id;
use crate::window::{window_pane_reset_mode, window_set_active_pane};
use ::core::ffi::CStr;
use ::std::borrow::Cow;
use ::std::ffi::CString;

pub const MSG_READ_CANCEL: msgtype = 307;
pub const MSG_WRITE_CLOSE: msgtype = 306;
pub const MSG_WRITE_READY: msgtype = 305;
pub const MSG_WRITE: msgtype = 304;
pub const MSG_WRITE_OPEN: msgtype = 303;
pub const MSG_READ_DONE: msgtype = 302;
pub const MSG_READ: msgtype = 301;
pub const MSG_READ_OPEN: msgtype = 300;
pub const MSG_FLAGS: msgtype = 218;
pub const MSG_EXEC: msgtype = 217;
pub const MSG_WAKEUP: msgtype = 216;
pub const MSG_UNLOCK: msgtype = 215;
pub const MSG_SUSPEND: msgtype = 214;
pub const MSG_OLDSTDOUT: msgtype = 213;
pub const MSG_OLDSTDIN: msgtype = 212;
pub const MSG_OLDSTDERR: msgtype = 211;
pub const MSG_SHUTDOWN: msgtype = 210;
pub const MSG_SHELL: msgtype = 209;
pub const MSG_RESIZE: msgtype = 208;
pub const MSG_READY: msgtype = 207;
pub const MSG_LOCK: msgtype = 206;
pub const MSG_EXITING: msgtype = 205;
pub const MSG_EXITED: msgtype = 204;
pub const MSG_EXIT: msgtype = 203;
pub const MSG_DETACHKILL: msgtype = 202;
pub const MSG_DETACH: msgtype = 201;
pub const MSG_COMMAND: msgtype = 200;
pub const MSG_IDENTIFY_TERMINFO: msgtype = 112;
pub const MSG_IDENTIFY_LONGFLAGS: msgtype = 111;
pub const MSG_IDENTIFY_STDOUT: msgtype = 110;
pub const MSG_IDENTIFY_FEATURES: msgtype = 109;
pub const MSG_IDENTIFY_CWD: msgtype = 108;
pub const MSG_IDENTIFY_CLIENTPID: msgtype = 107;
pub const MSG_IDENTIFY_DONE: msgtype = 106;
pub const MSG_IDENTIFY_ENVIRON: msgtype = 105;
pub const MSG_IDENTIFY_STDIN: msgtype = 104;
pub const MSG_IDENTIFY_OLDCWD: msgtype = 103;
pub const MSG_IDENTIFY_TTYNAME: msgtype = 102;
pub const MSG_IDENTIFY_TERM: msgtype = 101;
pub const MSG_IDENTIFY_FLAGS: msgtype = 100;
pub const MSG_VERSION: msgtype = 12;
pub const PANE_LINES_SPACES: pane_lines = 5;
pub const PANE_LINES_NUMBER: pane_lines = 4;
pub const PANE_LINES_SIMPLE: pane_lines = 3;
pub const PANE_LINES_HEAVY: pane_lines = 2;
pub const PANE_LINES_DOUBLE: pane_lines = 1;
pub const PANE_LINES_SINGLE: pane_lines = 0;
pub const PROGRESS_BAR_PAUSED: progress_bar_state = 4;
pub const PROGRESS_BAR_INDETERMINATE: progress_bar_state = 3;
pub const PROGRESS_BAR_ERROR: progress_bar_state = 2;
pub const PROGRESS_BAR_NORMAL: progress_bar_state = 1;
pub const PROGRESS_BAR_HIDDEN: progress_bar_state = 0;
pub const SCREEN_CURSOR_BAR: screen_cursor_style = 3;
pub const SCREEN_CURSOR_UNDERLINE: screen_cursor_style = 2;
pub const SCREEN_CURSOR_BLOCK: screen_cursor_style = 1;
pub const SCREEN_CURSOR_DEFAULT: screen_cursor_style = 0;
pub const STYLE_DEFAULT_SET: style_default_type = 3;
pub const STYLE_DEFAULT_POP: style_default_type = 2;
pub const STYLE_DEFAULT_PUSH: style_default_type = 1;
pub const STYLE_DEFAULT_BASE: style_default_type = 0;
pub const STYLE_RANGE_CONTROL: style_range_type = 7;
pub const STYLE_RANGE_USER: style_range_type = 6;
pub const STYLE_RANGE_SESSION: style_range_type = 5;
pub const STYLE_RANGE_WINDOW: style_range_type = 4;
pub const STYLE_RANGE_PANE: style_range_type = 3;
pub const STYLE_RANGE_RIGHT: style_range_type = 2;
pub const STYLE_RANGE_LEFT: style_range_type = 1;
pub const STYLE_RANGE_NONE: style_range_type = 0;
pub const STYLE_LIST_RIGHT_MARKER: style_list = 4;
pub const STYLE_LIST_LEFT_MARKER: style_list = 3;
pub const STYLE_LIST_FOCUS: style_list = 2;
pub const STYLE_LIST_ON: style_list = 1;
pub const STYLE_LIST_OFF: style_list = 0;
pub const STYLE_ALIGN_ABSOLUTE_CENTRE: style_align = 4;
pub const STYLE_ALIGN_RIGHT: style_align = 3;
pub const STYLE_ALIGN_CENTRE: style_align = 2;
pub const STYLE_ALIGN_LEFT: style_align = 1;
pub const STYLE_ALIGN_DEFAULT: style_align = 0;
pub const THEME_DARK: client_theme = 2;
pub const THEME_LIGHT: client_theme = 1;
pub const THEME_UNKNOWN: client_theme = 0;
pub const LAYOUT_WINDOWPANE: layout_type = 2;
pub const LAYOUT_TOPBOTTOM: layout_type = 1;
pub const LAYOUT_LEFTRIGHT: layout_type = 0;
pub const PROMPT_TYPE_INVALID: prompt_type = 255;
pub const PROMPT_TYPE_WINDOW_TARGET: prompt_type = 3;
pub const PROMPT_TYPE_TARGET: prompt_type = 2;
pub const PROMPT_TYPE_SEARCH: prompt_type = 1;
pub const PROMPT_TYPE_COMMAND: prompt_type = 0;
pub const PROMPT_COMMAND: client_prompt_mode = 1;
pub const PROMPT_ENTRY: client_prompt_mode = 0;
pub const CLIENT_EXIT_DETACH: client_exit_type = 2;
pub const CLIENT_EXIT_SHUTDOWN: client_exit_type = 1;
pub const CLIENT_EXIT_RETURN: client_exit_type = 0;
pub const ARGS_PARSE_COMMANDS: args_parse_type = 3;
pub const ARGS_PARSE_COMMANDS_OR_STRING: args_parse_type = 2;
pub const ARGS_PARSE_STRING: args_parse_type = 1;
pub const ARGS_PARSE_INVALID: args_parse_type = 0;
#[repr(C)]
#[derive(Default)]
pub struct window_copy_mode_data {
    pub screen: screen,
    pub(crate) backing: Option<Box<screen>>,
    pub backing_written: ::core::ffi::c_int,
    pub ictx: Option<InputCtxRef>,
    pub viewmode: ::core::ffi::c_int,
    pub oy: u_int,
    pub selx: u_int,
    pub sely: u_int,
    pub endselx: u_int,
    pub endsely: u_int,
    pub cursordrag: window_copy_mode_data_cursordrag,
    pub modekeys: ::core::ffi::c_int,
    pub lineflag: window_copy_mode_data_lineflag,
    pub rectflag: ::core::ffi::c_int,
    pub scroll_exit: ::core::ffi::c_int,
    pub hide_position: ::core::ffi::c_int,
    pub line_numbers: ::core::ffi::c_int,
    pub selflag: window_copy_mode_data_selflag,
    pub recentre_state: window_copy_mode_data_recentre_state,
    pub recentre_line: u_int,
    pub separators: Option<::std::ffi::CString>,
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
    pub showmark: ::core::ffi::c_int,
    pub searchtype: ::core::ffi::c_int,
    pub searchdirection: ::core::ffi::c_int,
    pub searchregex: ::core::ffi::c_int,
    pub searchstr: Option<::std::ffi::CString>,
    pub searchmark: Vec<u8>,
    pub searchcount: ::core::ffi::c_int,
    pub searchmore: ::core::ffi::c_int,
    pub searchall: ::core::ffi::c_int,
    pub searchx: ::core::ffi::c_int,
    pub searchy: ::core::ffi::c_int,
    pub searcho: ::core::ffi::c_int,
    pub searchgen: u_char,
    pub timeout: ::core::ffi::c_int,
    pub jumptype: ::core::ffi::c_int,
    pub jumpchar: Option<utf8_data>,
    pub dragtimer: TimerHandle,
}
impl window_copy_mode_data {
    /// The string copy mode is searching for, or null when nothing has been
    /// searched for.
    pub(crate) fn searchstr_ptr(&self) -> *mut ::core::ffi::c_char {
        cstr_ptr(&self.searchstr)
    }
}
pub type window_copy_mode_data_recentre_state = ::core::ffi::c_uint;
pub const RECENTRE_BOTTOM: window_copy_mode_data_recentre_state = 2;
pub const RECENTRE_MIDDLE: window_copy_mode_data_recentre_state = 1;
pub const RECENTRE_TOP: window_copy_mode_data_recentre_state = 0;
pub type window_copy_mode_data_selflag = ::core::ffi::c_uint;
pub const SEL_LINE: window_copy_mode_data_selflag = 2;
pub const SEL_WORD: window_copy_mode_data_selflag = 1;
pub const SEL_CHAR: window_copy_mode_data_selflag = 0;
pub type window_copy_mode_data_lineflag = ::core::ffi::c_uint;
pub const LINE_SEL_RIGHT_LEFT: window_copy_mode_data_lineflag = 2;
pub const LINE_SEL_LEFT_RIGHT: window_copy_mode_data_lineflag = 1;
pub const LINE_SEL_NONE: window_copy_mode_data_lineflag = 0;
pub type window_copy_mode_data_cursordrag = ::core::ffi::c_uint;
pub const CURSORDRAG_SEL: window_copy_mode_data_cursordrag = 2;
pub const CURSORDRAG_ENDSEL: window_copy_mode_data_cursordrag = 1;
pub const CURSORDRAG_NONE: window_copy_mode_data_cursordrag = 0;
pub const WINDOW_COPY_LINE_NUMBERS_DEFAULT: window_copy_line_numbers = 1;
pub const WINDOW_COPY_LINE_NUMBERS_OFF: window_copy_line_numbers = 0;
pub const WINDOW_COPY_LINE_NUMBERS_HYBRID: window_copy_line_numbers = 4;
pub const WINDOW_COPY_LINE_NUMBERS_RELATIVE: window_copy_line_numbers = 3;
pub const WINDOW_COPY_LINE_NUMBERS_ABSOLUTE: window_copy_line_numbers = 2;
pub const WINDOW_COPY_CMD_NOTHING: window_copy_cmd_action = 0;
pub type window_copy_cmd_action = ::core::ffi::c_uint;
pub const WINDOW_COPY_CMD_CANCEL: window_copy_cmd_action = 2;
pub const WINDOW_COPY_CMD_REDRAW: window_copy_cmd_action = 1;
pub const WINDOW_COPY_CMD_CLEAR_NEVER: window_copy_cmd_clear = 1;
pub type window_copy_cmd_clear = ::core::ffi::c_uint;
pub const WINDOW_COPY_CMD_CLEAR_EMACS_ONLY: window_copy_cmd_clear = 2;
pub const WINDOW_COPY_CMD_CLEAR_ALWAYS: window_copy_cmd_clear = 0;
#[repr(C)]
pub struct window_copy_cmd_state<'a> {
    /// The mode entry the command is running against, borrowed for as long
    /// as the command runs.
    pub wme: &'a mut window_mode_entry,
    pub args: &'a args,
    pub wargs: *mut args,
    pub m: *mut mouse_event,
    pub c: *mut client,
    pub s: *mut session,
    pub wl: *mut winlink,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_copy_cmd_entry {
    pub command: &'static ::core::ffi::CStr,
    pub minargs: u_int,
    pub maxargs: u_int,
    pub args: args_parse_t,
    pub flags: ::core::ffi::c_int,
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
pub type window_copy_line_position = ::core::ffi::c_uint;
pub const WINDOW_COPY_JUMPTOFORWARD: window_copy_search_type = 5;
pub const WINDOW_COPY_JUMPTOBACKWARD: window_copy_search_type = 6;
pub const WINDOW_COPY_JUMPBACKWARD: window_copy_search_type = 4;
pub const WINDOW_COPY_JUMPFORWARD: window_copy_search_type = 3;
pub const WINDOW_COPY_OFF: window_copy_search_type = 0;
pub type window_copy_search_type = ::core::ffi::c_uint;
pub type window_copy_rel_pos = ::core::ffi::c_uint;
pub type window_copy_line_numbers = ::core::ffi::c_uint;
#[inline]
fn tolower(mut __c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        if __c >= -(128 as ::core::ffi::c_int) && __c < 256 as ::core::ffi::c_int {
            *(*__ctype_tolower_loc()).offset(__c as isize) as ::core::ffi::c_int
        } else {
            __c
        }
    }
}
pub const REG_EXTENDED: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const REG_ICASE: ::core::ffi::c_int = (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int;
pub const REG_NOTBOL: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const UCHAR_MAX: ::core::ffi::c_int =
    __SCHAR_MAX__ * 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const UINT_MAX: ::core::ffi::c_uint = (__INT_MAX__ as ::core::ffi::c_uint)
    .wrapping_mul(2 as ::core::ffi::c_uint)
    .wrapping_add(1 as ::core::ffi::c_uint);
pub const WHITESPACE: &::core::ffi::CStr = c"\t ";
pub const MODEKEY_EMACS: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const MODEKEY_VI: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const GRID_ATTR_CHARSET: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const GRID_FLAG_PADDING: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const GRID_FLAG_EXTENDED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const GRID_FLAG_NOPALETTE: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const GRID_FLAG_TAB: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const GRID_LINE_WRAPPED: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const GRID_LINE_START_PROMPT: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const GRID_LINE_START_OUTPUT: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const GRID_HISTORY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const PANE_REDRAW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const PANE_REDRAWSCROLLBAR: ::core::ffi::c_int = 0x8000 as ::core::ffi::c_int;
pub const MOUSE_MASK_BUTTONS: ::core::ffi::c_int = 195 as ::core::ffi::c_int;
pub const MOUSE_WHEEL_UP: ::core::ffi::c_int = 64 as ::core::ffi::c_int;
pub const MOUSE_WHEEL_DOWN: ::core::ffi::c_int = 65 as ::core::ffi::c_int;
pub const CLIENT_READONLY: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const JOB_NOWAIT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const WINDOW_COPY_SEARCH_TIMEOUT: ::core::ffi::c_int = 10000 as ::core::ffi::c_int;
pub const WINDOW_COPY_SEARCH_ALL_TIMEOUT: ::core::ffi::c_int = 200 as ::core::ffi::c_int;
pub const WINDOW_COPY_SEARCH_MAX_LINE: ::core::ffi::c_int = 2000 as ::core::ffi::c_int;
pub const WINDOW_COPY_DRAG_REPEAT_TIME: ::core::ffi::c_int = 50000 as ::core::ffi::c_int;
unsafe fn window_copy_scroll_timer(wme: &mut window_mode_entry) {
    unsafe {
        let mut wp: *mut window_pane = wme.wp;
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut tv = timeval::from_usecs(WINDOW_COPY_DRAG_REPEAT_TIME as __suseconds_t);
        (*data).dragtimer.disarm();
        if window_pane_current_mode(wp) != wme {
            return;
        }
        if (*data).cy == 0 as u_int {
            (*data).dragtimer.arm(tv);
            window_copy_cursor_up(wme, 1 as ::core::ffi::c_int);
        } else if (*data).cy
            == (*screen_grid_ptr(&mut (*data).screen))
                .sy
                .wrapping_sub(1 as u_int)
        {
            (*data).dragtimer.arm(tv);
            window_copy_cursor_down(wme, 1 as ::core::ffi::c_int);
        }
    }
}
pub(crate) fn window_copy_backing(data: &mut window_copy_mode_data) -> *mut screen {
    data.backing
        .as_deref_mut()
        .map_or(::core::ptr::null_mut::<screen>(), |s| &raw mut *s)
}
unsafe fn window_copy_free_backing(data: &mut window_copy_mode_data) {
    unsafe {
        if let Some(mut backing) = data.backing.take() {
            screen_free(&mut *backing);
        }
    }
}
unsafe fn window_copy_clone_screen(
    mut src: *mut screen,
    mut hint: *mut screen,
    mut want_cursor: bool,
    mut trim: ::core::ffi::c_int,
) -> (Box<screen>, u_int, u_int) {
    unsafe {
        let mut dst: Box<screen>;
        let mut gl: Option<&grid_line>;
        let mut sy: u_int = 0;
        let mut wx: u_int = 0;
        let mut wy: u_int = 0;
        let mut reflow: ::core::ffi::c_int = 0;
        sy = (*screen_grid_ptr(&mut *src))
            .hsize
            .wrapping_add((*screen_grid_ptr(&mut *src)).sy);
        if trim != 0 {
            while sy > (*screen_grid_ptr(&mut *src)).hsize {
                gl = grid_peek_line(screen_grid(&*src), sy.wrapping_sub(1 as u_int));
                if !gl.is_some_and(|gl| gl.cellused == 0 as u_int) {
                    break;
                }
                sy = sy.wrapping_sub(1);
            }
        }
        log_debug(
            c"%s: target screen is %ux%u, source %ux%u".as_ptr(),
            fmt_args![
                c"window_copy_clone_screen".as_ptr(),
                (*screen_grid_ptr(&mut *src)).sx,
                sy,
                (*screen_grid_ptr(&mut *hint)).sx,
                (*screen_grid_ptr(&mut *src))
                    .hsize
                    .wrapping_add((*screen_grid_ptr(&mut *src)).sy)
            ],
        );
        dst = Box::new(screen::new(
            (*screen_grid_ptr(&mut *src)).sx,
            sy,
            (*screen_grid_ptr(&mut *src)).hlimit,
        ));
        (*screen_grid_ptr(&mut *dst)).flags |= GRID_HISTORY;
        grid_duplicate_lines(
            screen_grid_mut(&mut dst),
            0 as u_int,
            screen_grid(&*src),
            0 as u_int,
            sy,
        );
        (*screen_grid_ptr(&mut *dst)).sy = sy.wrapping_sub((*screen_grid_ptr(&mut *src)).hsize);
        (*screen_grid_ptr(&mut *dst)).hsize = (*screen_grid_ptr(&mut *src)).hsize;
        (*screen_grid_ptr(&mut *dst)).hscrolled = (*screen_grid_ptr(&mut *src)).hscrolled;
        if (*src).cy
            > (*screen_grid_ptr(&mut *dst))
                .sy
                .wrapping_sub(1 as u_int)
        {
            dst.cx = 0 as u_int;
            dst.cy = (*screen_grid_ptr(&mut *dst))
                .sy
                .wrapping_sub(1 as u_int);
        } else {
            dst.cx = (*src).cx;
            dst.cy = (*src).cy;
        }
        let mut cx = 0 as u_int;
        let mut cy = 0 as u_int;
        if want_cursor {
            cx = dst.cx;
            cy = (*screen_grid_ptr(&mut *dst)).hsize.wrapping_add(dst.cy);
            reflow = ((*screen_grid_ptr(&mut *hint)).sx != (*screen_grid_ptr(&mut *dst)).sx)
                as ::core::ffi::c_int;
        } else {
            reflow = 0 as ::core::ffi::c_int;
        }
        if reflow != 0 {
            (wx, wy) = grid_wrap_position(screen_grid(&dst), cx, cy);
        }
        screen_resize_cursor(
            &raw mut *dst,
            (*screen_grid_ptr(&mut *hint)).sx,
            (*screen_grid_ptr(&mut *hint)).sy,
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        if reflow != 0 {
            (cx, cy) = grid_unwrap_position(screen_grid(&dst), wx, wy);
        }
        (dst, cx, cy)
    }
}
unsafe fn window_copy_common_init(
    wme: &mut window_mode_entry,
    mode: WindowMode,
) -> *mut window_copy_mode_data {
    unsafe {
        let mut wp: *mut window_pane = wme.wp;
        let mut base: *mut screen = &raw mut (*wp).base;
        let mut data = Box::new(window_copy_mode_data::default());
        let data_ptr: *mut window_copy_mode_data = &mut *data;
        wme.state = match mode {
            WindowMode::Copy => WindowModeState::Copy(data),
            WindowMode::View => WindowModeState::View(data),
            _ => panic!("not a copy-mode state"),
        };
        let data = data_ptr;
        (*data).cursordrag = CURSORDRAG_NONE;
        (*data).lineflag = LINE_SEL_NONE;
        (*data).selflag = SEL_CHAR;
        if (*wp).searchstr.is_some() {
            (*data).searchtype = WINDOW_COPY_SEARCHUP as ::core::ffi::c_int;
            (*data).searchregex = (*wp).searchregex;
            (*data).searchstr = (*wp).searchstr.clone();
        } else {
            (*data).searchtype = WINDOW_COPY_OFF as ::core::ffi::c_int;
            (*data).searchregex = 0 as ::core::ffi::c_int;
            (*data).searchstr = None;
        }
        (*data).searcho = -(1 as ::core::ffi::c_int);
        (*data).searchy = (*data).searcho;
        (*data).searchx = (*data).searchy;
        (*data).searchall = 1 as ::core::ffi::c_int;
        (*data).jumptype = WINDOW_COPY_OFF as ::core::ffi::c_int;
        ::core::ptr::write(&raw mut (*data).jumpchar, None);
        (*data).line_numbers = 1 as ::core::ffi::c_int;
        screen_init(
            &mut (*data).screen,
            (*screen_grid_ptr(&mut *base)).sx,
            (*screen_grid_ptr(&mut *base)).sy,
            0 as u_int,
        );
        screen_set_default_cursor(&raw mut (*data).screen, global_w_options);
        (*data).modekeys = options_get_number((*(*wp).window).options_ptr(), c"mode-keys".as_ptr())
            as ::core::ffi::c_int;
        // The timer outlives the mode it was armed for, so it finds the
        // pane again by id and only fires while copy mode is still on it.
        let id = (*wp).id;
        (*data).dragtimer.set_callback(move || {
            let wp = window_pane_find_by_id(id);
            if wp.is_null() {
                return;
            }
            let wme = window_pane_current_mode(wp);
            if !wme.is_null() {
                window_copy_scroll_timer(&mut *wme);
            }
        });
        data
    }
}
pub(crate) unsafe fn window_copy_init(
    wme: &mut window_mode_entry,
    _fs: *mut cmd_find_state,
    args: Option<&args>,
) -> *mut screen {
    unsafe {
        let mut wp: *mut window_pane = wme.swp;
        let mut data: *mut window_copy_mode_data = ::core::ptr::null_mut::<window_copy_mode_data>();
        let mut base: *mut screen = &raw mut (*wp).base;
        let mut ctx = screen_write_ctx::default();
        let mut i: u_int = 0;
        let mut cx: u_int = 0;
        let mut cy: u_int = 0;
        data = window_copy_common_init(wme, WindowMode::Copy);
        let cloned = window_copy_clone_screen(
            base,
            &raw mut (*data).screen,
            true,
            (wme.swp != wme.wp) as ::core::ffi::c_int,
        );
        (*data).backing = Some(cloned.0);
        cx = cloned.1;
        cy = cloned.2;
        (*data).cx = cx;
        if cy < (*screen_grid_ptr(&mut *window_copy_backing(&mut *data))).hsize {
            (*data).cy = 0 as u_int;
            (*data).oy = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
                .hsize
                .wrapping_sub(cy);
        } else {
            (*data).cy = cy.wrapping_sub((*screen_grid_ptr(&mut *window_copy_backing(&mut *data))).hsize);
            (*data).oy = 0 as u_int;
        }
        (*data).scroll_exit = args.map_or(0, |args| args_has(args, b'e'));
        (*data).hide_position = args.map_or(0, |args| args_has(args, b'H'));
        if (*base).hyperlinks.is_some() {
            (*data).screen.hyperlinks = (*base).hyperlinks.clone();
        }
        (*data).screen.cx = window_copy_cursor_offset(
            wme,
            (*data).cx,
            (*screen_grid_ptr(&mut (*data).screen)).sx,
        );
        (*data).screen.cy = (*data).cy;
        (*data).mx = (*data).cx;
        (*data).my = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        (*data).showmark = 0 as ::core::ffi::c_int;
        screen_write_start(&mut ctx, &mut (*data).screen);
        i = 0 as u_int;
        while i < (*screen_grid_ptr(&mut (*data).screen)).sy {
            window_copy_write_line(wme, &mut ctx, i);
            i = i.wrapping_add(1);
        }
        screen_write_cursormove(
            &mut ctx,
            window_copy_cursor_offset(
                wme,
                (*data).cx,
                (*screen_grid_ptr(&mut (*data).screen)).sx,
            ) as ::core::ffi::c_int,
            (*data).cy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_stop(&mut ctx);
        (*data).recentre_state = RECENTRE_MIDDLE;
        (*data).recentre_line = 0 as u_int;
        &raw mut (*data).screen
    }
}
pub(crate) unsafe fn window_copy_view_init(
    wme: &mut window_mode_entry,
    _fs: *mut cmd_find_state,
    _args: Option<&args>,
) -> *mut screen {
    unsafe {
        let mut wp: *mut window_pane = wme.wp;
        let mut data: *mut window_copy_mode_data = ::core::ptr::null_mut::<window_copy_mode_data>();
        let mut base: *mut screen = &raw mut (*wp).base;
        let mut sx: u_int = (*screen_grid_ptr(&mut *base)).sx;
        data = window_copy_common_init(wme, WindowMode::View);
        (*data).viewmode = 1 as ::core::ffi::c_int;
        (*data).line_numbers = 0 as ::core::ffi::c_int;
        (*data).backing = Some(Box::new(screen::new(
            sx,
            (*screen_grid_ptr(&mut *base)).sy,
            UINT_MAX,
        )));
        (*data).ictx = Some(input_init(InputOwner::Detached, Stream::NONE));
        (*data).mx = (*data).cx;
        (*data).my = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        (*data).showmark = 0 as ::core::ffi::c_int;
        &raw mut (*data).screen
    }
}
pub(crate) unsafe fn window_copy_free(wme: &mut window_mode_entry) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        (*data).dragtimer.disarm();
        drop(::core::mem::take(&mut (*data).searchmark));
        (*data).searchstr = None;
        (*data).separators = None;
        if let Some(ictx) = (*data).ictx.take() {
            input_free_box(ictx);
        }
        window_copy_free_backing(&mut *data);
        screen_free(&mut (*data).screen);
    }
}
pub unsafe fn window_copy_add(
    mut wp: *mut window_pane,
    mut parse: ::core::ffi::c_int,
    mut fmt: *const ::core::ffi::c_char,
    args: &[FmtArg],
) {
    unsafe {
        window_copy_vadd(wp, parse, fmt, args);
    }
}
fn window_copy_init_ctx_cb(_ctx: &mut screen_write_ctx, ttyctx: &mut tty_ctx) {
    ttyctx.defaults = grid_default_cell;
    ttyctx.palette = ::core::ptr::null_mut::<colour_palette>();
    ttyctx.redraw_cb = None;
    ttyctx.set_client_cb = None;
    ttyctx.arg = TtyCtxArg::None;
}
pub unsafe fn window_copy_vadd(
    mut wp: *mut window_pane,
    mut parse: ::core::ffi::c_int,
    mut fmt: *const ::core::ffi::c_char,
    args: &[FmtArg],
) {
    unsafe {
        let mut wme: *mut window_mode_entry = window_pane_current_mode(wp);
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut backing: *mut screen = window_copy_backing(&mut *data);
        let mut backing_ctx = screen_write_ctx::default();
        let mut ctx = screen_write_ctx::default();
        let mut gc = grid_default_cell;
        let mut old_hsize: u_int = 0;
        let mut old_cy: u_int = 0;
        old_hsize = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data))).hsize;
        screen_write_start(&mut backing_ctx, &mut *backing);
        if (*data).backing_written != 0 {
            screen_write_carriagereturn(&mut backing_ctx);
            screen_write_linefeed(&mut backing_ctx, 0 as ::core::ffi::c_int, 8 as u_int);
        } else {
            (*data).backing_written = 1 as ::core::ffi::c_int;
        }
        old_cy = (*backing).cy;
        if parse != 0 {
            let text = format_alloc(fmt, args);
            input_parse_screen(
                ictx_mut(&mut (*data).ictx),
                backing,
                Some(window_copy_init_ctx_cb),
                ::core::ptr::null_mut::<popup_data>(),
                text.as_ptr() as *const u_char,
                text.as_bytes().len() as size_t,
            );
        } else {
            gc = grid_default_cell;
            screen_write_vnputs(&mut backing_ctx, 0 as ssize_t, &mut gc, fmt, args);
        }
        screen_write_stop(&mut backing_ctx);
        (*data).oy = (*data).oy.wrapping_add(
            (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
                .hsize
                .wrapping_sub(old_hsize),
        );
        screen_write_start_pane(&mut ctx, wp, Some(&mut (*data).screen));
        if (*screen_grid_ptr(&mut *window_copy_backing(&mut *data))).hsize != 0 {
            window_copy_redraw_lines(&mut *wme, 0 as u_int, 1 as u_int);
        }
        window_copy_redraw_lines(
            &mut *wme,
            old_cy,
            (*backing).cy.wrapping_sub(old_cy).wrapping_add(1 as u_int),
        );
        screen_write_stop(&mut ctx);
    }
}
pub unsafe fn window_copy_scroll(
    mut wp: *mut window_pane,
    mut sl_mpos: ::core::ffi::c_int,
    mut my: u_int,
    mut tty_oy: u_int,
    mut scroll_exit: ::core::ffi::c_int,
) {
    unsafe {
        let mut wme: *mut window_mode_entry = window_pane_current_mode(wp);
        if !wme.is_null() {
            window_set_active_pane((*wp).window, wp, 0 as ::core::ffi::c_int);
            window_copy_scroll1(&mut *wme, wp, sl_mpos, my, tty_oy, scroll_exit);
        }
    }
}
unsafe fn window_copy_scroll1(
    wme: &mut window_mode_entry,
    mut wp: *mut window_pane,
    mut sl_mpos: ::core::ffi::c_int,
    mut my: u_int,
    mut tty_oy: u_int,
    mut scroll_exit: ::core::ffi::c_int,
) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut ox: u_int = 0;
        let mut oy: u_int = 0;
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut n: u_int = 0;
        let mut offset: u_int = 0;
        let mut size: u_int = 0;
        let mut new_offset: u_int = 0;
        let mut slider_height: u_int = (*wp).sb_slider_h;
        let mut sb_height: u_int = (*wp).sy;
        let mut sb_top: u_int = (*wp).yoff as u_int;
        let mut sy: u_int = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data))).sy;
        let mut my_w: u_int = 0;
        let mut new_slider_y: ::core::ffi::c_int = 0;
        let mut delta: ::core::ffi::c_int = 0;
        my_w = my.wrapping_add(tty_oy);
        if my_w <= sb_top.wrapping_add(sl_mpos as u_int) {
            new_slider_y = sb_top.wrapping_sub((*wp).yoff as u_int) as ::core::ffi::c_int;
        } else if my_w.wrapping_sub(sl_mpos as u_int)
            > sb_top.wrapping_add(sb_height).wrapping_sub(slider_height)
        {
            new_slider_y = sb_top
                .wrapping_sub((*wp).yoff as u_int)
                .wrapping_add(sb_height.wrapping_sub(slider_height))
                as ::core::ffi::c_int;
        } else {
            new_slider_y = my_w
                .wrapping_sub((*wp).yoff as u_int)
                .wrapping_sub(sl_mpos as u_int) as ::core::ffi::c_int;
        }
        if (*wp).modes.is_empty() {
            return;
        }
        let Some((current_offset, current_size)) = window_copy_get_current_offset(wp) else {
            return;
        };
        (offset, size) = (current_offset, current_size);
        new_offset = (new_slider_y as ::core::ffi::c_float
            * (size.wrapping_add(sb_height) as ::core::ffi::c_float
                / sb_height as ::core::ffi::c_float)) as u_int;
        delta =
            (offset as ::core::ffi::c_int as u_int).wrapping_sub(new_offset) as ::core::ffi::c_int;
        oy = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        ox = window_copy_find_length(wme, oy);
        if (*data).cx != ox {
            (*data).lastcx = (*data).cx;
            (*data).lastsx = ox;
        }
        (*data).cx = (*data).lastcx;
        if delta >= 0 as ::core::ffi::c_int {
            n = delta as u_int;
            if (*data).oy.wrapping_add(n)
                > (*screen_grid_ptr(&mut *window_copy_backing(&mut *data))).hsize
            {
                (*data).oy = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data))).hsize;
                if (*data).cy < n {
                    (*data).cy = 0 as u_int;
                } else {
                    (*data).cy = (*data).cy.wrapping_sub(n);
                }
            } else {
                (*data).oy = (*data).oy.wrapping_add(n);
            }
        } else {
            n = -delta as u_int;
            if (*data).oy < n {
                (*data).oy = 0 as u_int;
                if (*data).cy.wrapping_add(n.wrapping_sub((*data).oy)) >= sy {
                    (*data).cy = sy.wrapping_sub(1 as u_int);
                } else {
                    (*data).cy = (*data).cy.wrapping_add(n.wrapping_sub((*data).oy));
                }
            } else {
                (*data).oy = (*data).oy.wrapping_sub(n);
            }
        }
        (*data).cursordrag = CURSORDRAG_NONE;
        if (*data).screen.sel.is_none() || (*data).rectflag == 0 {
            py = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
                .hsize
                .wrapping_add((*data).cy)
                .wrapping_sub((*data).oy);
            px = window_copy_find_length(wme, py);
            if (*data).cx >= (*data).lastsx && (*data).cx != px || (*data).cx > px {
                window_copy_cursor_end_of_line(wme);
            }
        }
        if scroll_exit != 0 && (*data).oy == 0 as u_int {
            window_pane_reset_mode(wp);
            return;
        }
        if !(*data).searchmark.is_empty() && (*data).timeout == 0 {
            window_copy_search_marks(
                wme,
                ::core::ptr::null_mut::<screen>(),
                (*data).searchregex,
                1 as ::core::ffi::c_int,
            );
        }
        window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        window_copy_redraw_screen(wme);
    }
}
pub unsafe fn window_copy_pageup(mut wp: *mut window_pane, mut half_page: ::core::ffi::c_int) {
    unsafe {
        window_copy_pageup1(&mut *window_pane_current_mode(wp), half_page);
    }
}
unsafe fn window_copy_pageup1(wme: &mut window_mode_entry, mut half_page: ::core::ffi::c_int) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut s: *mut screen = &raw mut (*data).screen;
        let mut n: u_int = 0;
        let mut ox: u_int = 0;
        let mut oy: u_int = 0;
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        oy = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        ox = window_copy_find_length(wme, oy);
        if (*data).cx != ox {
            (*data).lastcx = (*data).cx;
            (*data).lastsx = ox;
        }
        (*data).cx = (*data).lastcx;
        n = 1 as u_int;
        if (*screen_grid_ptr(&mut *s)).sy > 2 as u_int {
            if half_page != 0 {
                n = (*screen_grid_ptr(&mut *s)).sy.wrapping_div(2 as u_int);
            } else {
                n = (*screen_grid_ptr(&mut *s)).sy.wrapping_sub(2 as u_int);
            }
        }
        if (*data).oy.wrapping_add(n) > (*screen_grid_ptr(&mut *window_copy_backing(&mut *data))).hsize {
            (*data).oy = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data))).hsize;
            if (*data).cy < n {
                (*data).cy = 0 as u_int;
            } else {
                (*data).cy = (*data).cy.wrapping_sub(n);
            }
        } else {
            (*data).oy = (*data).oy.wrapping_add(n);
        }
        if (*data).screen.sel.is_none() || (*data).rectflag == 0 {
            py = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
                .hsize
                .wrapping_add((*data).cy)
                .wrapping_sub((*data).oy);
            px = window_copy_find_length(wme, py);
            if (*data).cx >= (*data).lastsx && (*data).cx != px || (*data).cx > px {
                window_copy_cursor_end_of_line(wme);
            }
        }
        if !(*data).searchmark.is_empty() && (*data).timeout == 0 {
            window_copy_search_marks(
                wme,
                ::core::ptr::null_mut::<screen>(),
                (*data).searchregex,
                1 as ::core::ffi::c_int,
            );
        }
        window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        window_copy_redraw_screen(wme);
    }
}
pub unsafe fn window_copy_pagedown(
    mut wp: *mut window_pane,
    mut half_page: ::core::ffi::c_int,
    mut scroll_exit: ::core::ffi::c_int,
) {
    unsafe {
        if window_copy_pagedown1(&mut *window_pane_current_mode(wp), half_page, scroll_exit) != 0 {
            window_pane_reset_mode(wp);
        }
    }
}
unsafe fn window_copy_pagedown1(
    wme: &mut window_mode_entry,
    mut half_page: ::core::ffi::c_int,
    mut scroll_exit: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut s: *mut screen = &raw mut (*data).screen;
        let mut n: u_int = 0;
        let mut ox: u_int = 0;
        let mut oy: u_int = 0;
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        oy = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        ox = window_copy_find_length(wme, oy);
        if (*data).cx != ox {
            (*data).lastcx = (*data).cx;
            (*data).lastsx = ox;
        }
        (*data).cx = (*data).lastcx;
        n = 1 as u_int;
        if (*screen_grid_ptr(&mut *s)).sy > 2 as u_int {
            if half_page != 0 {
                n = (*screen_grid_ptr(&mut *s)).sy.wrapping_div(2 as u_int);
            } else {
                n = (*screen_grid_ptr(&mut *s)).sy.wrapping_sub(2 as u_int);
            }
        }
        if (*data).oy < n {
            (*data).oy = 0 as u_int;
            if (*data).cy.wrapping_add(n.wrapping_sub((*data).oy))
                >= (*screen_grid_ptr(&mut *window_copy_backing(&mut *data))).sy
            {
                (*data).cy = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
                    .sy
                    .wrapping_sub(1 as u_int);
            } else {
                (*data).cy = (*data).cy.wrapping_add(n.wrapping_sub((*data).oy));
            }
        } else {
            (*data).oy = (*data).oy.wrapping_sub(n);
        }
        if (*data).screen.sel.is_none() || (*data).rectflag == 0 {
            py = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
                .hsize
                .wrapping_add((*data).cy)
                .wrapping_sub((*data).oy);
            px = window_copy_find_length(wme, py);
            if (*data).cx >= (*data).lastsx && (*data).cx != px || (*data).cx > px {
                window_copy_cursor_end_of_line(wme);
            }
        }
        if scroll_exit != 0 && (*data).oy == 0 as u_int {
            return 1 as ::core::ffi::c_int;
        }
        if !(*data).searchmark.is_empty() && (*data).timeout == 0 {
            window_copy_search_marks(
                wme,
                ::core::ptr::null_mut::<screen>(),
                (*data).searchregex,
                1 as ::core::ffi::c_int,
            );
        }
        window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        window_copy_redraw_screen(wme);
        0 as ::core::ffi::c_int
    }
}
unsafe fn window_copy_previous_paragraph(wme: &mut window_mode_entry) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut oy: u_int = 0;
        oy = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        while oy > 0 as u_int && window_copy_find_length(wme, oy) == 0 as u_int {
            oy = oy.wrapping_sub(1);
        }
        while oy > 0 as u_int && window_copy_find_length(wme, oy) > 0 as u_int {
            oy = oy.wrapping_sub(1);
        }
        window_copy_scroll_to(wme, 0 as u_int, oy, 0 as ::core::ffi::c_int);
    }
}
unsafe fn window_copy_next_paragraph(wme: &mut window_mode_entry) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut s: *mut screen = &raw mut (*data).screen;
        let mut maxy: u_int = 0;
        let mut ox: u_int = 0;
        let mut oy: u_int = 0;
        oy = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        maxy = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_add((*screen_grid_ptr(&mut *s)).sy)
            .wrapping_sub(1 as u_int);
        while oy < maxy && window_copy_find_length(wme, oy) == 0 as u_int {
            oy = oy.wrapping_add(1);
        }
        while oy < maxy && window_copy_find_length(wme, oy) > 0 as u_int {
            oy = oy.wrapping_add(1);
        }
        ox = window_copy_find_length(wme, oy);
        window_copy_scroll_to(wme, ox, oy, 0 as ::core::ffi::c_int);
    }
}
pub unsafe fn window_copy_get_word(
    mut wp: *mut window_pane,
    mut x: u_int,
    mut y: u_int,
) -> Option<CString> {
    unsafe {
        let mut wme: *mut window_mode_entry = window_pane_current_mode(wp);
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut gd: *mut grid = screen_grid_ptr(&mut *window_copy_backing(&mut *data));
        format_grid_word(gd, x, (*gd).hsize.wrapping_add(y).wrapping_sub((*data).oy))
    }
}
pub unsafe fn window_copy_get_line(mut wp: *mut window_pane, mut y: u_int) -> CString {
    unsafe {
        let mut wme: *mut window_mode_entry = window_pane_current_mode(wp);
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut gd: *mut grid = screen_grid_ptr(&mut *window_copy_backing(&mut *data));
        format_grid_line(gd, (*gd).hsize.wrapping_add(y).wrapping_sub((*data).oy))
    }
}
pub unsafe fn window_copy_get_hyperlink(
    mut wp: *mut window_pane,
    mut x: u_int,
    mut y: u_int,
) -> Option<CString> {
    unsafe {
        let mut wme: *mut window_mode_entry = window_pane_current_mode(wp);
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut gd: *mut grid = screen_grid_ptr(&mut (*data).screen);
        format_grid_hyperlink(gd, x, (*gd).hsize.wrapping_add(y), (*wp).screen())
    }
}
unsafe fn window_copy_cursor_hyperlink_cb(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wp: *mut window_pane = format_get_pane(ft);
        let mut wme: *mut window_mode_entry = window_pane_current_mode(wp);
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut gd: *mut grid = screen_grid_ptr(&mut (*data).screen);
        format_grid_hyperlink(
            gd,
            (*data).cx,
            (*gd).hsize.wrapping_add((*data).cy),
            &raw mut (*data).screen,
        )
    }
}
unsafe fn window_copy_cursor_word_cb(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wp: *mut window_pane = format_get_pane(ft);
        let mut wme: *mut window_mode_entry = window_pane_current_mode(wp);
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        window_copy_get_word(wp, (*data).cx, (*data).cy)
    }
}
unsafe fn window_copy_cursor_line_cb(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wp: *mut window_pane = format_get_pane(ft);
        let mut wme: *mut window_mode_entry = window_pane_current_mode(wp);
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        Some(window_copy_get_line(wp, (*data).cy))
    }
}
unsafe fn window_copy_search_match_cb(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wp: *mut window_pane = format_get_pane(ft);
        let mut wme: *mut window_mode_entry = window_pane_current_mode(wp);
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        window_copy_match_at_cursor(data)
    }
}
pub(crate) unsafe fn window_copy_formats(wme: &mut window_mode_entry, ft: &mut format_tree) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut hsize: u_int = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data))).hsize;
        let mut position: u_int = 0;
        let mut limit: u_int = 0;
        let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
        gl = grid_get_line(
            screen_grid_mut(&mut *window_copy_backing(&mut *data)),
            hsize.wrapping_sub((*data).oy),
        );
        format_add(
            ft,
            c"top_line_time",
            c"%llu".as_ptr(),
            fmt_args![(*gl).time as ::core::ffi::c_ulonglong],
        );
        format_add(
            ft,
            c"scroll_position",
            c"%d".as_ptr(),
            fmt_args![(*data).oy],
        );
        if window_copy_line_number_is_absolute(wme) != 0 {
            position = hsize.wrapping_sub((*data).oy).wrapping_add(1 as u_int);
            limit = hsize.wrapping_add((*screen_grid_ptr(&mut *window_copy_backing(&mut *data))).sy);
        } else {
            position = (*data).oy;
            limit = hsize;
        }
        format_add(ft, c"copy_position", c"%u".as_ptr(), fmt_args![position]);
        format_add(ft, c"copy_position_limit", c"%u".as_ptr(), fmt_args![limit]);
        format_add(
            ft,
            c"rectangle_toggle",
            c"%d".as_ptr(),
            fmt_args![(*data).rectflag],
        );
        format_add(ft, c"copy_cursor_x", c"%d".as_ptr(), fmt_args![(*data).cx]);
        format_add(ft, c"copy_cursor_y", c"%d".as_ptr(), fmt_args![(*data).cy]);
        if (*data).screen.sel.is_some() {
            format_add(
                ft,
                c"selection_start_x",
                c"%d".as_ptr(),
                fmt_args![(*data).selx],
            );
            format_add(
                ft,
                c"selection_start_y",
                c"%d".as_ptr(),
                fmt_args![(*data).sely],
            );
            format_add(
                ft,
                c"selection_end_x",
                c"%d".as_ptr(),
                fmt_args![(*data).endselx],
            );
            format_add(
                ft,
                c"selection_end_y",
                c"%d".as_ptr(),
                fmt_args![(*data).endsely],
            );
            if (*data).cursordrag as ::core::ffi::c_uint
                != CURSORDRAG_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                format_add(ft, c"selection_active", c"1".as_ptr(), fmt_args![]);
            } else {
                format_add(ft, c"selection_active", c"0".as_ptr(), fmt_args![]);
            }
            if (*data).endselx != (*data).selx || (*data).endsely != (*data).sely {
                format_add(ft, c"selection_present", c"1".as_ptr(), fmt_args![]);
            } else {
                format_add(ft, c"selection_present", c"0".as_ptr(), fmt_args![]);
            }
        } else {
            format_add(ft, c"selection_active", c"0".as_ptr(), fmt_args![]);
            format_add(ft, c"selection_present", c"0".as_ptr(), fmt_args![]);
        }
        match (*data).selflag {
            SEL_CHAR => {
                format_add(ft, c"selection_mode", c"char".as_ptr(), fmt_args![]);
            }
            SEL_WORD => {
                format_add(ft, c"selection_mode", c"word".as_ptr(), fmt_args![]);
            }
            SEL_LINE => {
                format_add(ft, c"selection_mode", c"line".as_ptr(), fmt_args![]);
            }
            _ => {}
        }
        format_add(
            ft,
            c"search_present",
            c"%d".as_ptr(),
            fmt_args![(!(*data).searchmark.is_empty()) as ::core::ffi::c_int],
        );
        format_add(
            ft,
            c"search_timed_out",
            c"%d".as_ptr(),
            fmt_args![(*data).timeout],
        );
        if (*data).searchcount != -(1 as ::core::ffi::c_int) {
            format_add(
                ft,
                c"search_count",
                c"%d".as_ptr(),
                fmt_args![(*data).searchcount],
            );
            format_add(
                ft,
                c"search_count_partial",
                c"%d".as_ptr(),
                fmt_args![(*data).searchmore],
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
pub(crate) unsafe fn window_copy_get_screen(wme: &mut window_mode_entry) -> *mut screen {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        window_copy_backing(&mut *data)
    }
}
unsafe fn window_copy_size_changed(wme: &mut window_mode_entry) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut s: *mut screen = &raw mut (*data).screen;
        let mut ctx = screen_write_ctx::default();
        let mut search: ::core::ffi::c_int = (!(*data).searchmark.is_empty()) as ::core::ffi::c_int;
        window_copy_clear_selection(wme);
        window_copy_clear_marks(wme);
        screen_write_start(&mut ctx, &mut *s);
        window_copy_write_lines(wme, &mut ctx, 0 as u_int, (*screen_grid_ptr(&mut *s)).sy);
        screen_write_stop(&mut ctx);
        if search != 0 && (*data).timeout == 0 {
            window_copy_search_marks(
                wme,
                ::core::ptr::null_mut::<screen>(),
                (*data).searchregex,
                0 as ::core::ffi::c_int,
            );
        }
        (*data).searchx = (*data).cx as ::core::ffi::c_int;
        (*data).searchy = (*data).cy as ::core::ffi::c_int;
        (*data).searcho = (*data).oy as ::core::ffi::c_int;
    }
}
pub(crate) unsafe fn window_copy_resize(wme: &mut window_mode_entry, mut sx: u_int, mut sy: u_int) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut s: *mut screen = &raw mut (*data).screen;
        let mut gd: *mut grid = screen_grid_ptr(&mut *window_copy_backing(&mut *data));
        let mut cx: u_int = 0;
        let mut cy: u_int = 0;
        let mut wx: u_int = 0;
        let mut wy: u_int = 0;
        let mut reflow: ::core::ffi::c_int = 0;
        screen_resize(&mut *s, sx, sy, 0 as ::core::ffi::c_int);
        cx = (*data).cx;
        if (*data).oy > (*gd).hsize.wrapping_add((*data).cy) {
            (*data).oy = (*gd).hsize.wrapping_add((*data).cy);
        }
        cy = (*gd)
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        reflow = ((*gd).sx != sx) as ::core::ffi::c_int;
        if reflow != 0 {
            (wx, wy) = grid_wrap_position(&*gd, cx, cy);
        }
        screen_resize_cursor(
            window_copy_backing(&mut *data),
            sx,
            sy,
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        if reflow != 0 {
            (cx, cy) = grid_unwrap_position(&*gd, wx, wy);
        }
        (*data).cx = cx;
        if cy < (*gd).hsize {
            (*data).cy = 0 as u_int;
            (*data).oy = (*gd).hsize.wrapping_sub(cy);
        } else {
            (*data).cy = cy.wrapping_sub((*gd).hsize);
            (*data).oy = 0 as u_int;
        }
        window_copy_size_changed(wme);
        window_copy_redraw_screen(wme);
    }
}
pub(crate) unsafe fn window_copy_key_table(
    wme: &mut window_mode_entry,
) -> &'static ::core::ffi::CStr {
    unsafe {
        let mut wp: *mut window_pane = wme.wp;
        if options_get_number((*(*wp).window).options_ptr(), c"mode-keys".as_ptr())
            == MODEKEY_VI as ::core::ffi::c_longlong
        {
            return c"copy-mode-vi";
        }
        c"copy-mode"
    }
}
unsafe fn window_copy_expand_search_string(
    cs: &mut window_copy_cmd_state<'_>,
) -> ::core::ffi::c_int {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut ss: *const ::core::ffi::c_char = args_string(&*cs.wargs, 0 as u_int);
        if ss.is_null() || *ss as ::core::ffi::c_int == '\0' as i32 {
            return 0 as ::core::ffi::c_int;
        }
        if args_has(cs.args, b'F') != 0 {
            let expanded = format_single(
                ::core::ptr::null_mut::<cmdq_item>(),
                CStr::from_ptr(ss),
                ::core::ptr::null_mut::<client>(),
                ::core::ptr::null_mut::<session>(),
                ::core::ptr::null_mut::<winlink>(),
                (*wme).wp,
            );
            if expanded.as_bytes().is_empty() {
                return 0 as ::core::ffi::c_int;
            }
            (*data).searchstr = Some(expanded);
        } else {
            (*data).searchstr = Some(CStr::from_ptr(ss).to_owned());
        }
        1 as ::core::ffi::c_int
    }
}
unsafe fn window_copy_cmd_append_selection(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut s: *mut session = cs.s;
        if !s.is_null() {
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
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut s: *mut session = cs.s;
        if !s.is_null() {
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
        let mut wme: *mut window_mode_entry = cs.wme;
        window_copy_cursor_back_to_indentation(&mut *wme);
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_begin_selection(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut c: *mut client = cs.c;
        let mut m: *mut mouse_event = cs.m;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        if !m.is_null() {
            window_copy_start_drag(c, &*m);
            return WINDOW_COPY_CMD_NOTHING;
        }
        (*data).lineflag = LINE_SEL_NONE;
        (*data).selflag = SEL_CHAR;
        window_copy_start_selection(&mut *wme);
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_stop_selection(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        (*data).cursordrag = CURSORDRAG_NONE;
        (*data).lineflag = LINE_SEL_NONE;
        (*data).selflag = SEL_CHAR;
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_bottom_line(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        (*data).cx = 0 as u_int;
        (*data).cy = (*screen_grid_ptr(&mut (*data).screen))
            .sy
            .wrapping_sub(1 as u_int);
        window_copy_update_selection(&mut *wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
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
        let mut wme: *mut window_mode_entry = cs.wme;
        window_copy_clear_selection(&mut *wme);
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_do_copy_end_of_line(
    cs: &mut window_copy_cmd_state<'_>,
    mut pipe: ::core::ffi::c_int,
    mut cancel: ::core::ffi::c_int,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut c: *mut client = cs.c;
        let mut s: *mut session = cs.s;
        let mut wl: *mut winlink = cs.wl;
        let mut wp: *mut window_pane = (*wme).wp;
        let mut count: u_int = args_count(&*cs.wargs);
        let mut np: u_int = (*wme).prefix;
        let mut ocx: u_int = 0;
        let mut ocy: u_int = 0;
        let mut ooy: u_int = 0;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut prefix: Option<::std::ffi::CString> = None;
        let mut command: Option<::std::ffi::CString> = None;
        let mut arg0: *const ::core::ffi::c_char = args_string(&*cs.wargs, 0 as u_int);
        let mut arg1: *const ::core::ffi::c_char = args_string(&*cs.wargs, 1 as u_int);
        let mut set_paste: ::core::ffi::c_int =
            (args_has(&*cs.wargs, 'P' as i32 as u_char) == 0) as ::core::ffi::c_int;
        let mut set_clip: ::core::ffi::c_int =
            (args_has(&*cs.wargs, 'C' as i32 as u_char) == 0) as ::core::ffi::c_int;
        if pipe != 0 {
            if count == 2 as u_int {
                prefix = Some(format_single(
                    ::core::ptr::null_mut::<cmdq_item>(),
                    CStr::from_ptr(arg1),
                    c,
                    s,
                    wl,
                    wp,
                ));
            }
            if !s.is_null() && count > 0 as u_int && *arg0 as ::core::ffi::c_int != '\0' as i32 {
                command = Some(format_single(
                    ::core::ptr::null_mut::<cmdq_item>(),
                    CStr::from_ptr(arg0),
                    c,
                    s,
                    wl,
                    wp,
                ));
            }
        } else if count == 1 as u_int {
            prefix = Some(format_single(
                ::core::ptr::null_mut::<cmdq_item>(),
                CStr::from_ptr(arg0),
                c,
                s,
                wl,
                wp,
            ));
        }
        ocx = (*data).cx;
        ocy = (*data).cy;
        ooy = (*data).oy;
        window_copy_start_selection(&mut *wme);
        while np > 1 as u_int {
            window_copy_cursor_down(&mut *wme, 0 as ::core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
        window_copy_cursor_end_of_line(&mut *wme);
        if !s.is_null() {
            if pipe != 0 {
                window_copy_copy_pipe(
                    &mut *wme,
                    s,
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
        (*data).cx = ocx;
        (*data).cy = ocy;
        (*data).oy = ooy;
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_copy_end_of_line(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe { window_copy_do_copy_end_of_line(cs, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) }
}
unsafe fn window_copy_cmd_copy_end_of_line_and_cancel(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe { window_copy_do_copy_end_of_line(cs, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int) }
}
unsafe fn window_copy_cmd_copy_pipe_end_of_line(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe { window_copy_do_copy_end_of_line(cs, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) }
}
unsafe fn window_copy_cmd_copy_pipe_end_of_line_and_cancel(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe { window_copy_do_copy_end_of_line(cs, 1 as ::core::ffi::c_int, 1 as ::core::ffi::c_int) }
}
unsafe fn window_copy_do_copy_line(
    cs: &mut window_copy_cmd_state<'_>,
    mut pipe: ::core::ffi::c_int,
    mut cancel: ::core::ffi::c_int,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut c: *mut client = cs.c;
        let mut s: *mut session = cs.s;
        let mut wl: *mut winlink = cs.wl;
        let mut wp: *mut window_pane = (*wme).wp;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut count: u_int = args_count(&*cs.wargs);
        let mut np: u_int = (*wme).prefix;
        let mut ocx: u_int = 0;
        let mut ocy: u_int = 0;
        let mut ooy: u_int = 0;
        let mut prefix: Option<::std::ffi::CString> = None;
        let mut command: Option<::std::ffi::CString> = None;
        let mut arg0: *const ::core::ffi::c_char = args_string(&*cs.wargs, 0 as u_int);
        let mut arg1: *const ::core::ffi::c_char = args_string(&*cs.wargs, 1 as u_int);
        let mut set_paste: ::core::ffi::c_int =
            (args_has(&*cs.wargs, 'P' as i32 as u_char) == 0) as ::core::ffi::c_int;
        let mut set_clip: ::core::ffi::c_int =
            (args_has(&*cs.wargs, 'C' as i32 as u_char) == 0) as ::core::ffi::c_int;
        if pipe != 0 {
            if count == 2 as u_int {
                prefix = Some(format_single(
                    ::core::ptr::null_mut::<cmdq_item>(),
                    CStr::from_ptr(arg1),
                    c,
                    s,
                    wl,
                    wp,
                ));
            }
            if !s.is_null() && count > 0 as u_int && *arg0 as ::core::ffi::c_int != '\0' as i32 {
                command = Some(format_single(
                    ::core::ptr::null_mut::<cmdq_item>(),
                    CStr::from_ptr(arg0),
                    c,
                    s,
                    wl,
                    wp,
                ));
            }
        } else if count == 1 as u_int {
            prefix = Some(format_single(
                ::core::ptr::null_mut::<cmdq_item>(),
                CStr::from_ptr(arg0),
                c,
                s,
                wl,
                wp,
            ));
        }
        ocx = (*data).cx;
        ocy = (*data).cy;
        ooy = (*data).oy;
        (*data).selflag = SEL_CHAR;
        window_copy_cursor_start_of_line(&mut *wme);
        window_copy_start_selection(&mut *wme);
        while np > 1 as u_int {
            window_copy_cursor_down(&mut *wme, 0 as ::core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
        window_copy_cursor_end_of_line(&mut *wme);
        if !s.is_null() {
            if pipe != 0 {
                window_copy_copy_pipe(
                    &mut *wme,
                    s,
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
        (*data).cx = ocx;
        (*data).cy = ocy;
        (*data).oy = ooy;
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_copy_line(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    unsafe { window_copy_do_copy_line(cs, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) }
}
unsafe fn window_copy_cmd_copy_line_and_cancel(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe { window_copy_do_copy_line(cs, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int) }
}
unsafe fn window_copy_cmd_copy_pipe_line(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe { window_copy_do_copy_line(cs, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) }
}
unsafe fn window_copy_cmd_copy_pipe_line_and_cancel(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe { window_copy_do_copy_line(cs, 1 as ::core::ffi::c_int, 1 as ::core::ffi::c_int) }
}
unsafe fn window_copy_cmd_copy_selection_no_clear(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut c: *mut client = cs.c;
        let mut s: *mut session = cs.s;
        let mut wl: *mut winlink = cs.wl;
        let mut wp: *mut window_pane = (*wme).wp;
        let mut prefix: Option<::std::ffi::CString> = None;
        let mut arg0: *const ::core::ffi::c_char = args_string(&*cs.wargs, 0 as u_int);
        let mut set_paste: ::core::ffi::c_int =
            (args_has(&*cs.wargs, 'P' as i32 as u_char) == 0) as ::core::ffi::c_int;
        let mut set_clip: ::core::ffi::c_int =
            (args_has(&*cs.wargs, 'C' as i32 as u_char) == 0) as ::core::ffi::c_int;
        if !arg0.is_null() {
            prefix = Some(format_single(
                ::core::ptr::null_mut::<cmdq_item>(),
                CStr::from_ptr(arg0),
                c,
                s,
                wl,
                wp,
            ));
        }
        if !s.is_null() {
            window_copy_copy_selection(&mut *wme, prefix.as_deref(), set_paste, set_clip);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_copy_selection(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        window_copy_cmd_copy_selection_no_clear(cs);
        window_copy_clear_selection(&mut *wme);
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_copy_selection_and_cancel(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        window_copy_cmd_copy_selection_no_clear(cs);
        window_copy_clear_selection(&mut *wme);
        WINDOW_COPY_CMD_CANCEL
    }
}
unsafe fn window_copy_cmd_cursor_down(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut np: u_int = (*wme).prefix;
        while np != 0 as u_int {
            window_copy_cursor_down(&mut *wme, 0 as ::core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_cursor_down_and_cancel(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut np: u_int = (*wme).prefix;
        let mut cy: u_int = 0;
        cy = (*data).cy;
        while np != 0 as u_int {
            window_copy_cursor_down(&mut *wme, 0 as ::core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
        if cy == (*data).cy && (*data).oy == 0 as u_int {
            return WINDOW_COPY_CMD_CANCEL;
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_cursor_left(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut np: u_int = (*wme).prefix;
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
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut np: u_int = (*wme).prefix;
        while np != 0 as u_int {
            window_copy_cursor_right(
                &mut *wme,
                ((*data).screen.sel.is_some() && (*data).rectflag != 0) as ::core::ffi::c_int,
            );
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_scroll_to(
    cs: &mut window_copy_cmd_state<'_>,
    mut to: u_int,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut oy: u_int = 0;
        let mut delta: u_int = 0;
        let mut scroll_up: ::core::ffi::c_int = 0;
        scroll_up = (*data).cy.wrapping_sub(to) as ::core::ffi::c_int;
        delta = abs(scroll_up) as u_int;
        oy = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_sub((*data).oy);
        if scroll_up > 0 as ::core::ffi::c_int && (*data).oy >= delta {
            window_copy_scroll_up(&mut *wme, delta);
            (*data).cy = (*data).cy.wrapping_sub(delta);
        } else if scroll_up < 0 as ::core::ffi::c_int && oy >= delta {
            window_copy_scroll_down(&mut *wme, delta);
            (*data).cy = (*data).cy.wrapping_add(delta);
        }
        window_copy_update_selection(&mut *wme, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_scroll_bottom(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut data: *mut window_copy_mode_data = cs.wme.state.copy();
        let mut bottom: u_int = 0;
        bottom = (*screen_grid_ptr(&mut (*data).screen))
            .sy
            .wrapping_sub(1 as u_int);
        window_copy_cmd_scroll_to(cs, bottom)
    }
}
unsafe fn window_copy_cmd_scroll_middle(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut data: *mut window_copy_mode_data = cs.wme.state.copy();
        let mut mid_value: u_int = 0;
        mid_value = (*screen_grid_ptr(&mut (*data).screen))
            .sy
            .wrapping_sub(1 as u_int)
            .wrapping_div(2 as u_int);
        window_copy_cmd_scroll_to(cs, mid_value)
    }
}
unsafe fn window_copy_cmd_scroll_to_mouse(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut wp: *mut window_pane = (*wme).wp;
        let mut c: *mut client = cs.c;
        let mut m: *mut mouse_event = cs.m;
        let mut scroll_exit: ::core::ffi::c_int = args_has(&*cs.wargs, 'e' as i32 as u_char);
        let (_bigger, _tty_ox, tty_oy, _tty_sx, _tty_sy) = tty_window_offset(&(*c).tty);
        window_copy_scroll(wp, (*c).tty.mouse_slider_mpos, (*m).y, tty_oy, scroll_exit);
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_scroll_top(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    unsafe { window_copy_cmd_scroll_to(cs, 0 as u_int) }
}
unsafe fn window_copy_cmd_cursor_up(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut np: u_int = (*wme).prefix;
        while np != 0 as u_int {
            window_copy_cursor_up(&mut *wme, 0 as ::core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_centre_vertical(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        window_copy_update_cursor(
            &mut *wme,
            (*data).cx,
            (*(*wme).wp).sy.wrapping_div(2 as u_int),
        );
        window_copy_update_selection(&mut *wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_centre_horizontal(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        window_copy_update_cursor(
            &mut *wme,
            (*(*wme).wp).sx.wrapping_div(2 as u_int),
            (*data).cy,
        );
        window_copy_update_selection(&mut *wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_end_of_line(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        window_copy_cursor_end_of_line(&mut *wme);
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_halfpage_down(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut np: u_int = (*wme).prefix;
        while np != 0 as u_int {
            if window_copy_pagedown1(&mut *wme, 1 as ::core::ffi::c_int, (*data).scroll_exit) != 0 {
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
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut np: u_int = (*wme).prefix;
        while np != 0 as u_int {
            if window_copy_pagedown1(&mut *wme, 1 as ::core::ffi::c_int, 1 as ::core::ffi::c_int)
                != 0
            {
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
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut np: u_int = (*wme).prefix;
        while np != 0 as u_int {
            window_copy_pageup1(&mut *wme, 1 as ::core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_toggle_position(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        (*data).hide_position = ((*data).hide_position == 0) as ::core::ffi::c_int;
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_history_bottom(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut s: *mut screen = window_copy_backing(&mut *data);
        let mut oy: u_int = 0;
        oy = (*screen_grid_ptr(&mut *s))
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        if (*data).lineflag as ::core::ffi::c_uint
            == LINE_SEL_RIGHT_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
            && oy == (*data).endsely
        {
            window_copy_other_end(&mut *wme);
        }
        (*data).cy = (*screen_grid_ptr(&mut (*data).screen))
            .sy
            .wrapping_sub(1 as u_int);
        (*data).cx = window_copy_cursor_limit(
            &mut *wme,
            (*screen_grid_ptr(&mut *s)).hsize.wrapping_add((*data).cy),
            0 as ::core::ffi::c_int,
        );
        (*data).oy = 0 as u_int;
        if !(*data).searchmark.is_empty() && (*data).timeout == 0 {
            window_copy_search_marks(
                &mut *wme,
                ::core::ptr::null_mut::<screen>(),
                (*data).searchregex,
                1 as ::core::ffi::c_int,
            );
        }
        window_copy_update_selection(&mut *wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_history_top(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut oy: u_int = 0;
        oy = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        if (*data).lineflag as ::core::ffi::c_uint
            == LINE_SEL_LEFT_RIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
            && oy == (*data).sely
        {
            window_copy_other_end(&mut *wme);
        }
        (*data).cy = 0 as u_int;
        (*data).cx = 0 as u_int;
        (*data).oy = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data))).hsize;
        if !(*data).searchmark.is_empty() && (*data).timeout == 0 {
            window_copy_search_marks(
                &mut *wme,
                ::core::ptr::null_mut::<screen>(),
                (*data).searchregex,
                1 as ::core::ffi::c_int,
            );
        }
        window_copy_update_selection(&mut *wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_jump_again(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut np: u_int = (*wme).prefix;
        match (*data).jumptype {
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
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut np: u_int = (*wme).prefix;
        match (*data).jumptype {
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
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        (*data).cx = 0 as u_int;
        (*data).cy = (*screen_grid_ptr(&mut (*data).screen))
            .sy
            .wrapping_sub(1 as u_int)
            .wrapping_div(2 as u_int);
        window_copy_update_selection(&mut *wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_previous_matching_bracket(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut np: u_int = (*wme).prefix;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut s: *mut screen = window_copy_backing(&mut *data);
        let mut open: [::core::ffi::c_char; 4] =
            ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"{[(\0");
        let mut close: [::core::ffi::c_char; 4] =
            ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"}])\0");
        let mut tried: ::core::ffi::c_char = 0;
        let mut found: ::core::ffi::c_char = 0;
        let mut start: ::core::ffi::c_char = 0;
        let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut xx: u_int = 0;
        let mut n: u_int = 0;
        let mut gc = grid_default_cell;
        let mut failed: ::core::ffi::c_int = 0;
        while np != 0 as u_int {
            px = (*data).cx;
            py = (*screen_grid_ptr(&mut *s))
                .hsize
                .wrapping_add((*data).cy)
                .wrapping_sub((*data).oy);
            xx = window_copy_find_length(&mut *wme, py);
            if xx == 0 as u_int {
                break;
            }
            tried = 0 as ::core::ffi::c_char;
            loop {
                gc = grid_get_cell(screen_grid(&*s), px, py);
                if gc.data.size as ::core::ffi::c_int != 1 as ::core::ffi::c_int
                    || gc.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0
                {
                    cp = ::core::ptr::null_mut::<::core::ffi::c_char>();
                } else {
                    found = *(&raw mut gc.data.data as *mut u_char) as ::core::ffi::c_char;
                    cp = strchr(
                        &raw const close as *const ::core::ffi::c_char,
                        found as ::core::ffi::c_int,
                    );
                }
                if cp.is_null() {
                    if !((*data).modekeys == MODEKEY_EMACS) {
                        break;
                    }
                    if tried == 0 && px > 0 as u_int {
                        px = px.wrapping_sub(1);
                        tried = 1 as ::core::ffi::c_char;
                    } else {
                        window_copy_cursor_previous_word(
                            &mut *wme,
                            CStr::from_ptr(&raw const close as *const ::core::ffi::c_char),
                            1 as ::core::ffi::c_int,
                        );
                        break;
                    }
                } else {
                    start = open[cp.offset_from(&raw mut close as *mut ::core::ffi::c_char)
                        as ::core::ffi::c_long as usize];
                    n = 1 as u_int;
                    failed = 0 as ::core::ffi::c_int;
                    loop {
                        if px == 0 as u_int {
                            if py == 0 as u_int {
                                failed = 1 as ::core::ffi::c_int;
                                break;
                            } else {
                                loop {
                                    py = py.wrapping_sub(1);
                                    xx = window_copy_find_length(&mut *wme, py);
                                    if !(xx == 0 as u_int && py > 0 as u_int) {
                                        break;
                                    }
                                }
                                if xx == 0 as u_int && py == 0 as u_int {
                                    failed = 1 as ::core::ffi::c_int;
                                    break;
                                } else {
                                    px = xx.wrapping_sub(1 as u_int);
                                }
                            }
                        } else {
                            px = px.wrapping_sub(1);
                        }
                        gc = grid_get_cell(screen_grid(&*s), px, py);
                        if gc.data.size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
                            && !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0
                        {
                            if *(&raw mut gc.data.data as *mut u_char) as ::core::ffi::c_int
                                == found as ::core::ffi::c_int
                            {
                                n = n.wrapping_add(1);
                            } else if *(&raw mut gc.data.data as *mut u_char) as ::core::ffi::c_int
                                == start as ::core::ffi::c_int
                            {
                                n = n.wrapping_sub(1);
                            }
                        }
                        if !(n != 0 as u_int) {
                            break;
                        }
                    }
                    if failed == 0 {
                        window_copy_scroll_to(&mut *wme, px, py, 0 as ::core::ffi::c_int);
                    }
                    break;
                }
            }
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_next_matching_bracket(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut np: u_int = (*wme).prefix;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut s: *mut screen = window_copy_backing(&mut *data);
        let mut open: [::core::ffi::c_char; 4] =
            ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"{[(\0");
        let mut close: [::core::ffi::c_char; 4] =
            ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"}])\0");
        let mut tried: ::core::ffi::c_char = 0;
        let mut found: ::core::ffi::c_char = 0;
        let mut end: ::core::ffi::c_char = 0;
        let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut xx: u_int = 0;
        let mut yy: u_int = 0;
        let mut sx: u_int = 0;
        let mut sy: u_int = 0;
        let mut n: u_int = 0;
        let mut gc = grid_default_cell;
        let mut failed: ::core::ffi::c_int = 0;
        let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
        's_22: while np != 0 as u_int {
            px = (*data).cx;
            py = (*screen_grid_ptr(&mut *s))
                .hsize
                .wrapping_add((*data).cy)
                .wrapping_sub((*data).oy);
            xx = window_copy_find_length(&mut *wme, py);
            yy = (*screen_grid_ptr(&mut *s))
                .hsize
                .wrapping_add((*screen_grid_ptr(&mut *s)).sy)
                .wrapping_sub(1 as u_int);
            if xx == 0 as u_int {
                break;
            }
            tried = 0 as ::core::ffi::c_char;
            loop {
                gc = grid_get_cell(screen_grid(&*s), px, py);
                if gc.data.size as ::core::ffi::c_int != 1 as ::core::ffi::c_int
                    || gc.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0
                {
                    cp = ::core::ptr::null_mut::<::core::ffi::c_char>();
                } else {
                    found = *(&raw mut gc.data.data as *mut u_char) as ::core::ffi::c_char;
                    cp = strchr(
                        &raw const close as *const ::core::ffi::c_char,
                        found as ::core::ffi::c_int,
                    );
                    if !cp.is_null() && (*data).modekeys == MODEKEY_VI {
                        sx = (*data).cx;
                        sy = (*screen_grid_ptr(&mut *s))
                            .hsize
                            .wrapping_add((*data).cy)
                            .wrapping_sub((*data).oy);
                        window_copy_scroll_to(&mut *wme, px, py, 0 as ::core::ffi::c_int);
                        window_copy_cmd_previous_matching_bracket(cs);
                        px = (*data).cx;
                        py = (*screen_grid_ptr(&mut *s))
                            .hsize
                            .wrapping_add((*data).cy)
                            .wrapping_sub((*data).oy);
                        gc = grid_get_cell(screen_grid(&*s), px, py);
                        if gc.data.size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
                            && !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0
                            && !strchr(
                                &raw const close as *const ::core::ffi::c_char,
                                *(&raw mut gc.data.data as *mut u_char) as ::core::ffi::c_int,
                            )
                            .is_null()
                        {
                            window_copy_scroll_to(&mut *wme, sx, sy, 0 as ::core::ffi::c_int);
                        }
                        break 's_22;
                    } else {
                        cp = strchr(
                            &raw const open as *const ::core::ffi::c_char,
                            found as ::core::ffi::c_int,
                        );
                    }
                }
                if cp.is_null() {
                    if (*data).modekeys == MODEKEY_EMACS {
                        if tried == 0 && px <= xx {
                            px = px.wrapping_add(1);
                            tried = 1 as ::core::ffi::c_char;
                        } else {
                            window_copy_cursor_next_word_end(
                                &mut *wme,
                                CStr::from_ptr(&raw const open as *const ::core::ffi::c_char),
                                0 as ::core::ffi::c_int,
                            );
                            break;
                        }
                    } else if px > xx {
                        if py == yy {
                            break;
                        }
                        gl = grid_get_line(screen_grid_mut(&mut *s), py);
                        if !(*gl).flags & GRID_LINE_WRAPPED != 0 {
                            break;
                        }
                        if (*gl).cellsize() > (*screen_grid_ptr(&mut *s)).sx {
                            break;
                        }
                        px = 0 as u_int;
                        py = py.wrapping_add(1);
                        xx = window_copy_find_length(&mut *wme, py);
                    } else {
                        px = px.wrapping_add(1);
                    }
                } else {
                    end = close[cp.offset_from(&raw mut open as *mut ::core::ffi::c_char)
                        as ::core::ffi::c_long as usize];
                    n = 1 as u_int;
                    failed = 0 as ::core::ffi::c_int;
                    loop {
                        if px > xx {
                            if py == yy {
                                failed = 1 as ::core::ffi::c_int;
                                break;
                            } else {
                                px = 0 as u_int;
                                py = py.wrapping_add(1);
                                xx = window_copy_find_length(&mut *wme, py);
                            }
                        } else {
                            px = px.wrapping_add(1);
                        }
                        gc = grid_get_cell(screen_grid(&*s), px, py);
                        if gc.data.size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
                            && !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0
                        {
                            if *(&raw mut gc.data.data as *mut u_char) as ::core::ffi::c_int
                                == found as ::core::ffi::c_int
                            {
                                n = n.wrapping_add(1);
                            } else if *(&raw mut gc.data.data as *mut u_char) as ::core::ffi::c_int
                                == end as ::core::ffi::c_int
                            {
                                n = n.wrapping_sub(1);
                            }
                        }
                        if !(n != 0 as u_int) {
                            break;
                        }
                    }
                    if failed == 0 {
                        window_copy_scroll_to(&mut *wme, px, py, 0 as ::core::ffi::c_int);
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
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut np: u_int = (*wme).prefix;
        while np != 0 as u_int {
            window_copy_next_paragraph(&mut *wme);
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_next_space(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut np: u_int = (*wme).prefix;
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
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut np: u_int = (*wme).prefix;
        while np != 0 as u_int {
            window_copy_cursor_next_word_end(&mut *wme, c"", 0 as ::core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_next_word(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut np: u_int = (*wme).prefix;
        let mut separators: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        separators = options_get_string(session_options(cs.s), c"word-separators".as_ptr());
        let separators = CStr::from_ptr(separators);
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
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut np: u_int = (*wme).prefix;
        let mut separators: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        separators = options_get_string(session_options(cs.s), c"word-separators".as_ptr());
        let separators = CStr::from_ptr(separators);
        while np != 0 as u_int {
            window_copy_cursor_next_word_end(&mut *wme, separators, 0 as ::core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_other_end(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut np: u_int = (*wme).prefix;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        (*data).selflag = SEL_CHAR;
        if np.wrapping_rem(2 as u_int) != 0 as u_int {
            window_copy_other_end(&mut *wme);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_selection_mode(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut so: *mut options = session_options(cs.s);
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut s: *const ::core::ffi::c_char = args_string(&*cs.wargs, 0 as u_int);
        if s.is_null()
            || strcasecmp(s, c"char".as_ptr()) == 0 as ::core::ffi::c_int
            || strcasecmp(s, c"c".as_ptr()) == 0 as ::core::ffi::c_int
        {
            (*data).selflag = SEL_CHAR;
        } else if strcasecmp(s, c"word".as_ptr()) == 0 as ::core::ffi::c_int
            || strcasecmp(s, c"w".as_ptr()) == 0 as ::core::ffi::c_int
        {
            (*data).separators = Some(
                CStr::from_ptr(options_get_string(so, c"word-separators".as_ptr())).to_owned(),
            );
            (*data).selflag = SEL_WORD;
        } else if strcasecmp(s, c"line".as_ptr()) == 0 as ::core::ffi::c_int
            || strcasecmp(s, c"l".as_ptr()) == 0 as ::core::ffi::c_int
        {
            (*data).selflag = SEL_LINE;
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_page_down(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut np: u_int = (*wme).prefix;
        while np != 0 as u_int {
            if window_copy_pagedown1(&mut *wme, 0 as ::core::ffi::c_int, (*data).scroll_exit) != 0 {
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
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut np: u_int = (*wme).prefix;
        while np != 0 as u_int {
            if window_copy_pagedown1(&mut *wme, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int)
                != 0
            {
                return WINDOW_COPY_CMD_CANCEL;
            }
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_page_up(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut np: u_int = (*wme).prefix;
        while np != 0 as u_int {
            window_copy_pageup1(&mut *wme, 0 as ::core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_previous_paragraph(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut np: u_int = (*wme).prefix;
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
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut np: u_int = (*wme).prefix;
        while np != 0 as u_int {
            window_copy_cursor_previous_word(&mut *wme, c"", 1 as ::core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_previous_word(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut np: u_int = (*wme).prefix;
        let mut separators: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        separators = options_get_string(session_options(cs.s), c"word-separators".as_ptr());
        let separators = CStr::from_ptr(separators);
        while np != 0 as u_int {
            window_copy_cursor_previous_word(&mut *wme, separators, 1 as ::core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_rectangle_on(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        (*data).lineflag = LINE_SEL_NONE;
        window_copy_rectangle_set(&mut *wme, 1 as ::core::ffi::c_int);
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_rectangle_off(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        (*data).lineflag = LINE_SEL_NONE;
        window_copy_rectangle_set(&mut *wme, 0 as ::core::ffi::c_int);
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_rectangle_toggle(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        (*data).lineflag = LINE_SEL_NONE;
        window_copy_rectangle_set(&mut *wme, ((*data).rectflag == 0) as ::core::ffi::c_int);
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_scroll_exit_on(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut data: *mut window_copy_mode_data = cs.wme.state.copy();
        (*data).scroll_exit = 1 as ::core::ffi::c_int;
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_scroll_exit_off(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut data: *mut window_copy_mode_data = cs.wme.state.copy();
        (*data).scroll_exit = 0 as ::core::ffi::c_int;
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_scroll_exit_toggle(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut data: *mut window_copy_mode_data = cs.wme.state.copy();
        (*data).scroll_exit = ((*data).scroll_exit == 0) as ::core::ffi::c_int;
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_scroll_down(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut np: u_int = (*wme).prefix;
        while np != 0 as u_int {
            window_copy_cursor_down(&mut *wme, 1 as ::core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
        if (*data).scroll_exit != 0 && (*data).oy == 0 as u_int {
            return WINDOW_COPY_CMD_CANCEL;
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_scroll_down_and_cancel(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut np: u_int = (*wme).prefix;
        while np != 0 as u_int {
            window_copy_cursor_down(&mut *wme, 1 as ::core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
        if (*data).oy == 0 as u_int {
            return WINDOW_COPY_CMD_CANCEL;
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_scroll_up(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut np: u_int = (*wme).prefix;
        while np != 0 as u_int {
            window_copy_cursor_up(&mut *wme, 1 as ::core::ffi::c_int);
            np = np.wrapping_sub(1);
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_search_again(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut np: u_int = (*wme).prefix;
        if (*data).searchtype == WINDOW_COPY_SEARCHUP as ::core::ffi::c_int {
            while np != 0 as u_int {
                window_copy_search_up(&mut *wme, (*data).searchregex);
                np = np.wrapping_sub(1);
            }
        } else if (*data).searchtype == WINDOW_COPY_SEARCHDOWN as ::core::ffi::c_int {
            while np != 0 as u_int {
                window_copy_search_down(&mut *wme, (*data).searchregex);
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
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut np: u_int = (*wme).prefix;
        if (*data).searchtype == WINDOW_COPY_SEARCHUP as ::core::ffi::c_int {
            while np != 0 as u_int {
                window_copy_search_down(&mut *wme, (*data).searchregex);
                np = np.wrapping_sub(1);
            }
        } else if (*data).searchtype == WINDOW_COPY_SEARCHDOWN as ::core::ffi::c_int {
            while np != 0 as u_int {
                window_copy_search_up(&mut *wme, (*data).searchregex);
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
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut np: u_int = (*wme).prefix;
        (*data).lineflag = LINE_SEL_LEFT_RIGHT;
        (*data).rectflag = 0 as ::core::ffi::c_int;
        (*data).selflag = SEL_LINE;
        (*data).dx = (*data).cx;
        (*data).dy = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        window_copy_cursor_start_of_line(&mut *wme);
        (*data).selrx = (*data).cx;
        (*data).selry = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        (*data).endselry = (*data).selry;
        window_copy_start_selection(&mut *wme);
        window_copy_cursor_end_of_line(&mut *wme);
        (*data).endselry = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        (*data).endselrx = window_copy_find_length(&mut *wme, (*data).endselry);
        while np > 1 as u_int {
            window_copy_cursor_down(&mut *wme, 0 as ::core::ffi::c_int);
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
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut so: *mut options = session_options(cs.s);
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut nextx: u_int = 0;
        let mut nexty: u_int = 0;
        (*data).lineflag = LINE_SEL_LEFT_RIGHT;
        (*data).rectflag = 0 as ::core::ffi::c_int;
        (*data).selflag = SEL_WORD;
        (*data).dx = (*data).cx;
        (*data).dy = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        (*data).separators =
            Some(CStr::from_ptr(options_get_string(so, c"word-separators".as_ptr())).to_owned());
        window_copy_cursor_previous_word(
            &mut *wme,
            (*data).separators.as_deref().unwrap_or(c""),
            0 as ::core::ffi::c_int,
        );
        px = (*data).cx;
        py = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        (*data).selrx = px;
        (*data).selry = py;
        window_copy_start_selection(&mut *wme);
        nextx = px.wrapping_add(1 as u_int);
        nexty = py;
        if grid_get_line(
            screen_grid_mut(&mut *window_copy_backing(&mut *data)),
            nexty,
        )
        .flags
            & GRID_LINE_WRAPPED
            != 0
            && nextx
                > (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
                    .sx
                    .wrapping_sub(1 as u_int)
        {
            nextx = 0 as u_int;
            nexty = nexty.wrapping_add(1);
        }
        if px >= window_copy_find_length(&mut *wme, py)
            || window_copy_in_set(&mut *wme, nextx, nexty, WHITESPACE) == 0
        {
            window_copy_cursor_next_word_end(
                &mut *wme,
                (*data).separators.as_deref().unwrap_or(c""),
                1 as ::core::ffi::c_int,
            );
        } else {
            window_copy_update_cursor(&mut *wme, px, (*data).cy);
            if window_copy_update_selection(
                &mut *wme,
                1 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            ) != 0
            {
                window_copy_redraw_lines(&mut *wme, (*data).cy, 1 as u_int);
            }
        }
        (*data).endselrx = (*data).cx;
        (*data).endselry = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        if (*data).dy > (*data).endselry {
            (*data).dy = (*data).endselry;
            (*data).dx = (*data).endselrx;
        } else if (*data).dx > (*data).endselrx {
            (*data).dx = (*data).endselrx;
        }
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_set_mark(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    unsafe {
        let mut data: *mut window_copy_mode_data = cs.wme.state.copy();
        (*data).mx = (*data).cx;
        (*data).my = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        (*data).showmark = 1 as ::core::ffi::c_int;
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_start_of_line(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        window_copy_cursor_start_of_line(&mut *wme);
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_top_line(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        (*data).cx = 0 as u_int;
        (*data).cy = 0 as u_int;
        window_copy_update_selection(&mut *wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_copy_pipe_no_clear(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut c: *mut client = cs.c;
        let mut s: *mut session = cs.s;
        let mut wl: *mut winlink = cs.wl;
        let mut wp: *mut window_pane = (*wme).wp;
        let mut command: Option<::std::ffi::CString> = None;
        let mut prefix: Option<::std::ffi::CString> = None;
        let mut arg0: *const ::core::ffi::c_char = args_string(&*cs.wargs, 0 as u_int);
        let mut arg1: *const ::core::ffi::c_char = args_string(&*cs.wargs, 1 as u_int);
        let mut set_paste: ::core::ffi::c_int =
            (args_has(&*cs.wargs, 'P' as i32 as u_char) == 0) as ::core::ffi::c_int;
        let mut set_clip: ::core::ffi::c_int =
            (args_has(&*cs.wargs, 'C' as i32 as u_char) == 0) as ::core::ffi::c_int;
        if !arg1.is_null() {
            prefix = Some(format_single(
                ::core::ptr::null_mut::<cmdq_item>(),
                CStr::from_ptr(arg1),
                c,
                s,
                wl,
                wp,
            ));
        }
        if !s.is_null() && !arg0.is_null() && *arg0 as ::core::ffi::c_int != '\0' as i32 {
            command = Some(format_single(
                ::core::ptr::null_mut::<cmdq_item>(),
                CStr::from_ptr(arg0),
                c,
                s,
                wl,
                wp,
            ));
        }
        window_copy_copy_pipe(
            &mut *wme,
            s,
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
        let mut wme: *mut window_mode_entry = cs.wme;
        window_copy_cmd_copy_pipe_no_clear(cs);
        window_copy_clear_selection(&mut *wme);
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_copy_pipe_and_cancel(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        window_copy_cmd_copy_pipe_no_clear(cs);
        window_copy_clear_selection(&mut *wme);
        WINDOW_COPY_CMD_CANCEL
    }
}
unsafe fn window_copy_cmd_pipe_no_clear(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut c: *mut client = cs.c;
        let mut s: *mut session = cs.s;
        let mut wl: *mut winlink = cs.wl;
        let mut wp: *mut window_pane = (*wme).wp;
        let mut command: Option<::std::ffi::CString> = None;
        let mut arg0: *const ::core::ffi::c_char = args_string(&*cs.wargs, 0 as u_int);
        if !s.is_null() && !arg0.is_null() && *arg0 as ::core::ffi::c_int != '\0' as i32 {
            command = Some(format_single(
                ::core::ptr::null_mut::<cmdq_item>(),
                CStr::from_ptr(arg0),
                c,
                s,
                wl,
                wp,
            ));
        }
        window_copy_pipe(&mut *wme, s, command.as_deref());
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_pipe(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        window_copy_cmd_pipe_no_clear(cs);
        window_copy_clear_selection(&mut *wme);
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_pipe_and_cancel(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        window_copy_cmd_pipe_no_clear(cs);
        window_copy_clear_selection(&mut *wme);
        WINDOW_COPY_CMD_CANCEL
    }
}
unsafe fn window_copy_cmd_goto_line(cs: &mut window_copy_cmd_state<'_>) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut arg0: *const ::core::ffi::c_char = args_string(&*cs.wargs, 0 as u_int);
        if *arg0 as ::core::ffi::c_int != '\0' as i32 {
            window_copy_goto_line(&mut *wme, CStr::from_ptr(arg0));
        }
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_jump_backward(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut np: u_int = (*wme).prefix;
        let mut arg0: *const ::core::ffi::c_char = args_string(&*cs.wargs, 0 as u_int);
        if *arg0 as ::core::ffi::c_int != '\0' as i32 {
            (*data).jumptype = WINDOW_COPY_JUMPBACKWARD as ::core::ffi::c_int;
            (*data).jumpchar = utf8_vec_fromcstr(arg0).into_iter().next();
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
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut np: u_int = (*wme).prefix;
        let mut arg0: *const ::core::ffi::c_char = args_string(&*cs.wargs, 0 as u_int);
        if *arg0 as ::core::ffi::c_int != '\0' as i32 {
            (*data).jumptype = WINDOW_COPY_JUMPFORWARD as ::core::ffi::c_int;
            (*data).jumpchar = utf8_vec_fromcstr(arg0).into_iter().next();
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
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut np: u_int = (*wme).prefix;
        let mut arg0: *const ::core::ffi::c_char = args_string(&*cs.wargs, 0 as u_int);
        if *arg0 as ::core::ffi::c_int != '\0' as i32 {
            (*data).jumptype = WINDOW_COPY_JUMPTOBACKWARD as ::core::ffi::c_int;
            (*data).jumpchar = utf8_vec_fromcstr(arg0).into_iter().next();
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
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut np: u_int = (*wme).prefix;
        let mut arg0: *const ::core::ffi::c_char = args_string(&*cs.wargs, 0 as u_int);
        if *arg0 as ::core::ffi::c_int != '\0' as i32 {
            (*data).jumptype = WINDOW_COPY_JUMPTOFORWARD as ::core::ffi::c_int;
            (*data).jumpchar = utf8_vec_fromcstr(arg0).into_iter().next();
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
        let mut wme: *mut window_mode_entry = cs.wme;
        window_copy_jump_to_mark(&mut *wme);
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_next_prompt(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        window_copy_cursor_prompt(
            &mut *wme,
            1 as ::core::ffi::c_int,
            args_has(&*cs.wargs, 'o' as i32 as u_char),
        );
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_previous_prompt(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        window_copy_cursor_prompt(
            &mut *wme,
            0 as ::core::ffi::c_int,
            args_has(&*cs.wargs, 'o' as i32 as u_char),
        );
        WINDOW_COPY_CMD_NOTHING
    }
}
unsafe fn window_copy_cmd_search_backward(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut np: u_int = (*wme).prefix;
        if window_copy_expand_search_string(cs) == 0 {
            return WINDOW_COPY_CMD_NOTHING;
        }
        if (*data).searchstr.is_some() {
            (*data).searchtype = WINDOW_COPY_SEARCHUP as ::core::ffi::c_int;
            (*data).searchregex = 1 as ::core::ffi::c_int;
            (*data).timeout = 0 as ::core::ffi::c_int;
            while np != 0 as u_int {
                window_copy_search_up(&mut *wme, 1 as ::core::ffi::c_int);
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
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut np: u_int = (*wme).prefix;
        if window_copy_expand_search_string(cs) == 0 {
            return WINDOW_COPY_CMD_NOTHING;
        }
        if (*data).searchstr.is_some() {
            (*data).searchtype = WINDOW_COPY_SEARCHUP as ::core::ffi::c_int;
            (*data).searchregex = 0 as ::core::ffi::c_int;
            (*data).timeout = 0 as ::core::ffi::c_int;
            while np != 0 as u_int {
                window_copy_search_up(&mut *wme, 0 as ::core::ffi::c_int);
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
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut np: u_int = (*wme).prefix;
        if window_copy_expand_search_string(cs) == 0 {
            return WINDOW_COPY_CMD_NOTHING;
        }
        if (*data).searchstr.is_some() {
            (*data).searchtype = WINDOW_COPY_SEARCHDOWN as ::core::ffi::c_int;
            (*data).searchregex = 1 as ::core::ffi::c_int;
            (*data).timeout = 0 as ::core::ffi::c_int;
            while np != 0 as u_int {
                window_copy_search_down(&mut *wme, 1 as ::core::ffi::c_int);
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
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut np: u_int = (*wme).prefix;
        if window_copy_expand_search_string(cs) == 0 {
            return WINDOW_COPY_CMD_NOTHING;
        }
        if (*data).searchstr.is_some() {
            (*data).searchtype = WINDOW_COPY_SEARCHDOWN as ::core::ffi::c_int;
            (*data).searchregex = 0 as ::core::ffi::c_int;
            (*data).timeout = 0 as ::core::ffi::c_int;
            while np != 0 as u_int {
                window_copy_search_down(&mut *wme, 0 as ::core::ffi::c_int);
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
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut arg0: *const ::core::ffi::c_char = args_string(&*cs.wargs, 0 as u_int);
        let mut ss: *const ::core::ffi::c_char = (*data).searchstr_ptr();
        let mut prefix: ::core::ffi::c_char = 0;
        let mut action: window_copy_cmd_action = WINDOW_COPY_CMD_NOTHING;
        (*data).timeout = 0 as ::core::ffi::c_int;
        log_debug(
            c"%s: %s".as_ptr(),
            fmt_args![
                c"window_copy_cmd_search_backward_incremental".as_ptr(),
                arg0
            ],
        );
        let fresh3 = arg0;
        arg0 = arg0.offset(1);
        prefix = *fresh3;
        if (*data).searchx == -(1 as ::core::ffi::c_int)
            || (*data).searchy == -(1 as ::core::ffi::c_int)
        {
            (*data).searchx = (*data).cx as ::core::ffi::c_int;
            (*data).searchy = (*data).cy as ::core::ffi::c_int;
            (*data).searcho = (*data).oy as ::core::ffi::c_int;
        } else if !ss.is_null() && strcmp(arg0, ss) != 0 as ::core::ffi::c_int {
            (*data).cx = (*data).searchx as u_int;
            (*data).cy = (*data).searchy as u_int;
            (*data).oy = (*data).searcho as u_int;
            (*data).cx = window_copy_cursor_limit(
                &mut *wme,
                (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
                    .hsize
                    .wrapping_add((*data).cy)
                    .wrapping_sub((*data).oy),
                0 as ::core::ffi::c_int,
            );
            action = WINDOW_COPY_CMD_REDRAW;
        }
        if *arg0 as ::core::ffi::c_int == '\0' as i32 {
            window_copy_clear_marks(&mut *wme);
            return WINDOW_COPY_CMD_REDRAW;
        }
        match prefix as ::core::ffi::c_int {
            61 | 45 => {
                (*data).searchtype = WINDOW_COPY_SEARCHUP as ::core::ffi::c_int;
                (*data).searchregex = 0 as ::core::ffi::c_int;
                (*data).searchstr = Some(CStr::from_ptr(arg0).to_owned());
                if window_copy_search_up(&mut *wme, 0 as ::core::ffi::c_int) == 0 {
                    window_copy_clear_marks(&mut *wme);
                    return WINDOW_COPY_CMD_REDRAW;
                }
            }
            43 => {
                (*data).searchtype = WINDOW_COPY_SEARCHDOWN as ::core::ffi::c_int;
                (*data).searchregex = 0 as ::core::ffi::c_int;
                (*data).searchstr = Some(CStr::from_ptr(arg0).to_owned());
                if window_copy_search_down(&mut *wme, 0 as ::core::ffi::c_int) == 0 {
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
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut arg0: *const ::core::ffi::c_char = args_string(&*cs.wargs, 0 as u_int);
        let mut ss: *const ::core::ffi::c_char = (*data).searchstr_ptr();
        let mut prefix: ::core::ffi::c_char = 0;
        let mut action: window_copy_cmd_action = WINDOW_COPY_CMD_NOTHING;
        (*data).timeout = 0 as ::core::ffi::c_int;
        log_debug(
            c"%s: %s".as_ptr(),
            fmt_args![c"window_copy_cmd_search_forward_incremental".as_ptr(), arg0],
        );
        let fresh2 = arg0;
        arg0 = arg0.offset(1);
        prefix = *fresh2;
        if (*data).searchx == -(1 as ::core::ffi::c_int)
            || (*data).searchy == -(1 as ::core::ffi::c_int)
        {
            (*data).searchx = (*data).cx as ::core::ffi::c_int;
            (*data).searchy = (*data).cy as ::core::ffi::c_int;
            (*data).searcho = (*data).oy as ::core::ffi::c_int;
        } else if !ss.is_null() && strcmp(arg0, ss) != 0 as ::core::ffi::c_int {
            (*data).cx = (*data).searchx as u_int;
            (*data).cy = (*data).searchy as u_int;
            (*data).oy = (*data).searcho as u_int;
            (*data).cx = window_copy_cursor_limit(
                &mut *wme,
                (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
                    .hsize
                    .wrapping_add((*data).cy)
                    .wrapping_sub((*data).oy),
                0 as ::core::ffi::c_int,
            );
            action = WINDOW_COPY_CMD_REDRAW;
        }
        if *arg0 as ::core::ffi::c_int == '\0' as i32 {
            window_copy_clear_marks(&mut *wme);
            return WINDOW_COPY_CMD_REDRAW;
        }
        match prefix as ::core::ffi::c_int {
            61 | 43 => {
                (*data).searchtype = WINDOW_COPY_SEARCHDOWN as ::core::ffi::c_int;
                (*data).searchregex = 0 as ::core::ffi::c_int;
                (*data).searchstr = Some(CStr::from_ptr(arg0).to_owned());
                if window_copy_search_down(&mut *wme, 0 as ::core::ffi::c_int) == 0 {
                    window_copy_clear_marks(&mut *wme);
                    return WINDOW_COPY_CMD_REDRAW;
                }
            }
            45 => {
                (*data).searchtype = WINDOW_COPY_SEARCHUP as ::core::ffi::c_int;
                (*data).searchregex = 0 as ::core::ffi::c_int;
                (*data).searchstr = Some(CStr::from_ptr(arg0).to_owned());
                if window_copy_search_up(&mut *wme, 0 as ::core::ffi::c_int) == 0 {
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
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut wp: *mut window_pane = (*wme).swp;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut oy_from_top: u_int = 0;
        if (*data).viewmode != 0 {
            return WINDOW_COPY_CMD_NOTHING;
        }
        if (*data).oy > (*screen_grid_ptr(&mut *window_copy_backing(&mut *data))).hsize {
            (*data).oy = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data))).hsize;
        }
        oy_from_top = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_sub((*data).oy);
        window_copy_free_backing(&mut *data);
        (*data).backing = Some(
            window_copy_clone_screen(
                &raw mut (*wp).base,
                &raw mut (*data).screen,
                false,
                ((*wme).swp != (*wme).wp) as ::core::ffi::c_int,
            )
            .0,
        );
        if oy_from_top <= (*screen_grid_ptr(&mut *window_copy_backing(&mut *data))).hsize {
            (*data).oy = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
                .hsize
                .wrapping_sub(oy_from_top);
        } else {
            (*data).cy = 0 as u_int;
            (*data).oy = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data))).hsize;
        }
        window_copy_size_changed(&mut *wme);
        WINDOW_COPY_CMD_REDRAW
    }
}
unsafe fn window_copy_cmd_recentre_top_bottom(
    cs: &mut window_copy_cmd_state<'_>,
) -> window_copy_cmd_action {
    unsafe {
        let mut wme: *mut window_mode_entry = cs.wme;
        let mut data: *mut window_copy_mode_data = (*wme).state.copy();
        let mut cy: u_int = (*data).cy;
        let mut oy: u_int = (*data).oy;
        let mut sy: u_int = (*screen_grid_ptr(&mut (*data).screen))
            .sy
            .wrapping_sub(1 as u_int);
        let mut sm: u_int = sy.wrapping_div(2 as u_int);
        let mut backing_row: u_int = 0;
        let mut target: window_copy_line_position = MIDDLE;
        backing_row = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_add(cy)
            .wrapping_sub((*data).oy);
        if (*data).recentre_line != backing_row {
            (*data).recentre_state = RECENTRE_MIDDLE;
            (*data).recentre_line = backing_row;
        }
        match (*data).recentre_state {
            RECENTRE_MIDDLE => {
                (*data).recentre_state = RECENTRE_TOP;
                target = MIDDLE;
            }
            RECENTRE_TOP => {
                (*data).recentre_state = RECENTRE_BOTTOM;
                target = TOP;
            }
            _ => {
                (*data).recentre_state = RECENTRE_MIDDLE;
                target = BOTTOM;
            }
        }
        oy = (*data).oy;
        match target {
            MIDDLE => {
                if cy < sm {
                    window_copy_scroll_down(&mut *wme, sm.wrapping_sub(cy));
                } else if cy > sm {
                    window_copy_scroll_up(&mut *wme, cy.wrapping_sub(sm));
                }
                if (*data).oy != oy {
                    (*data).cy = cy.wrapping_add((*data).oy.wrapping_sub(oy));
                }
            }
            TOP => {
                window_copy_scroll_up(&mut *wme, cy);
                (*data).cy = cy.wrapping_sub(oy.wrapping_sub((*data).oy));
            }
            BOTTOM => {
                window_copy_scroll_down(&mut *wme, sy.wrapping_sub(cy));
                (*data).cy = cy.wrapping_add((*data).oy.wrapping_sub(oy));
            }
            _ => {}
        }
        window_copy_update_selection(&mut *wme, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_append_selection),
        },
        window_copy_cmd_entry {
            command: c"append-selection-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_append_selection_and_cancel),
        },
        window_copy_cmd_entry {
            command: c"back-to-indentation",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_begin_selection),
        },
        window_copy_cmd_entry {
            command: c"bottom-line",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_clear_selection),
        },
        window_copy_cmd_entry {
            command: c"copy-end-of-line",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"CP",
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_end_of_line),
        },
        window_copy_cmd_entry {
            command: c"copy-end-of-line-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"CP",
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_end_of_line_and_cancel),
        },
        window_copy_cmd_entry {
            command: c"copy-pipe-end-of-line",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"CP",
                lower: 0 as ::core::ffi::c_int,
                upper: 2 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_pipe_end_of_line),
        },
        window_copy_cmd_entry {
            command: c"copy-pipe-end-of-line-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"CP",
                lower: 0 as ::core::ffi::c_int,
                upper: 2 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_pipe_end_of_line_and_cancel),
        },
        window_copy_cmd_entry {
            command: c"copy-line",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"CP",
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_line),
        },
        window_copy_cmd_entry {
            command: c"copy-line-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"CP",
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_line_and_cancel),
        },
        window_copy_cmd_entry {
            command: c"copy-pipe-line",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"CP",
                lower: 0 as ::core::ffi::c_int,
                upper: 2 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_pipe_line),
        },
        window_copy_cmd_entry {
            command: c"copy-pipe-line-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"CP",
                lower: 0 as ::core::ffi::c_int,
                upper: 2 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_pipe_line_and_cancel),
        },
        window_copy_cmd_entry {
            command: c"copy-pipe-no-clear",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"CP",
                lower: 0 as ::core::ffi::c_int,
                upper: 2 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_NEVER,
            f: Some(window_copy_cmd_copy_pipe_no_clear),
        },
        window_copy_cmd_entry {
            command: c"copy-pipe",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"CP",
                lower: 0 as ::core::ffi::c_int,
                upper: 2 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_pipe),
        },
        window_copy_cmd_entry {
            command: c"copy-pipe-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"CP",
                lower: 0 as ::core::ffi::c_int,
                upper: 2 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_pipe_and_cancel),
        },
        window_copy_cmd_entry {
            command: c"copy-selection-no-clear",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"CP",
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_NEVER,
            f: Some(window_copy_cmd_copy_selection_no_clear),
        },
        window_copy_cmd_entry {
            command: c"copy-selection",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"CP",
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_selection),
        },
        window_copy_cmd_entry {
            command: c"copy-selection-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"CP",
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_copy_selection_and_cancel),
        },
        window_copy_cmd_entry {
            command: c"cursor-down",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 1 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_jump_again),
        },
        window_copy_cmd_entry {
            command: c"jump-backward",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 1 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_jump_backward),
        },
        window_copy_cmd_entry {
            command: c"jump-forward",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 1 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_jump_forward),
        },
        window_copy_cmd_entry {
            command: c"jump-reverse",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_jump_reverse),
        },
        window_copy_cmd_entry {
            command: c"jump-to-backward",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 1 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_jump_to_backward),
        },
        window_copy_cmd_entry {
            command: c"jump-to-forward",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 1 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_jump_to_forward),
        },
        window_copy_cmd_entry {
            command: c"jump-to-mark",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_other_end),
        },
        window_copy_cmd_entry {
            command: c"page-down",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_NEVER,
            f: Some(window_copy_cmd_pipe_no_clear),
        },
        window_copy_cmd_entry {
            command: c"pipe",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_pipe),
        },
        window_copy_cmd_entry {
            command: c"pipe-and-cancel",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_pipe_and_cancel),
        },
        window_copy_cmd_entry {
            command: c"previous-matching-bracket",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_rectangle_on),
        },
        window_copy_cmd_entry {
            command: c"rectangle-off",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_rectangle_off),
        },
        window_copy_cmd_entry {
            command: c"rectangle-toggle",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_rectangle_toggle),
        },
        window_copy_cmd_entry {
            command: c"refresh-from-pane",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_search_again),
        },
        window_copy_cmd_entry {
            command: c"search-backward",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_search_backward),
        },
        window_copy_cmd_entry {
            command: c"search-backward-text",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_search_backward_text),
        },
        window_copy_cmd_entry {
            command: c"search-backward-incremental",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 1 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_search_backward_incremental),
        },
        window_copy_cmd_entry {
            command: c"search-forward",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_search_forward),
        },
        window_copy_cmd_entry {
            command: c"search-forward-text",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_search_forward_text),
        },
        window_copy_cmd_entry {
            command: c"search-forward-incremental",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 1 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_search_forward_incremental),
        },
        window_copy_cmd_entry {
            command: c"search-reverse",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_search_reverse),
        },
        window_copy_cmd_entry {
            command: c"select-line",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_select_line),
        },
        window_copy_cmd_entry {
            command: c"select-word",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_select_word),
        },
        window_copy_cmd_entry {
            command: c"selection-mode",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 1 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_selection_mode),
        },
        window_copy_cmd_entry {
            command: c"set-mark",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: 0 as ::core::ffi::c_int,
            clear: WINDOW_COPY_CMD_CLEAR_ALWAYS,
            f: Some(window_copy_cmd_stop_selection),
        },
        window_copy_cmd_entry {
            command: c"toggle-position",
            minargs: 0,
            maxargs: 0,
            args: args_parse_t {
                template: c"",
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
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
                lower: 0 as ::core::ffi::c_int,
                upper: 0 as ::core::ffi::c_int,
                cb: None,
            },
            flags: WINDOW_COPY_CMD_FLAG_READONLY,
            clear: WINDOW_COPY_CMD_CLEAR_EMACS_ONLY,
            f: Some(window_copy_cmd_top_line),
        },
    ]
};
pub const WINDOW_COPY_CMD_FLAG_READONLY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub(crate) unsafe fn window_copy_command(
    wme: &mut window_mode_entry,
    mut c: *mut client,
    mut s: *mut session,
    mut wl: *mut winlink,
    args: &args,
    m: Option<&mut mouse_event>,
) {
    unsafe {
        let m = m.map_or(::core::ptr::null_mut(), |m| m);
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut wp: *mut window_pane = wme.wp;

        let mut action: window_copy_cmd_action = WINDOW_COPY_CMD_NOTHING;
        let mut clear: window_copy_cmd_clear = WINDOW_COPY_CMD_CLEAR_NEVER;
        let mut command: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut i: u_int = 0;
        let mut count: u_int = args_count(args);
        let mut keys: ::core::ffi::c_int = 0;
        let mut flags: ::core::ffi::c_int = 0;
        let mut error: Option<::std::ffi::CString> = None;
        if count == 0 as u_int {
            return;
        }
        command = args_string(args, 0 as u_int);
        if !m.is_null()
            && (*m).valid != 0
            && !((*m).b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_UP as u_int
                || (*m).b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_DOWN as u_int)
        {
            window_copy_move_mouse(&*m);
        }
        let mut cs = window_copy_cmd_state {
            wme,
            args,
            wargs: ::core::ptr::null_mut::<args>(),
            m,
            c,
            s,
            wl,
        };
        action = WINDOW_COPY_CMD_NOTHING;
        i = 0 as u_int;
        while (i as usize)
            < (::core::mem::size_of::<[window_copy_cmd_entry; 93]>() as usize)
                .wrapping_div(::core::mem::size_of::<window_copy_cmd_entry>() as usize)
        {
            if strcmp(window_copy_cmd_table[i as usize].command.as_ptr(), command)
                == 0 as ::core::ffi::c_int
            {
                flags = window_copy_cmd_table[i as usize].flags;
                if !c.is_null()
                    && (*c).flags & CLIENT_READONLY as uint64_t != 0
                    && !flags & WINDOW_COPY_CMD_FLAG_READONLY != 0
                {
                    status_message_set(
                        c,
                        -(1 as ::core::ffi::c_int),
                        1 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        c"client is read-only".as_ptr(),
                        fmt_args![],
                    );
                    return;
                }
                let values = if args.values.is_empty() {
                    ::core::ptr::null_mut()
                } else {
                    args.values.as_ptr().cast_mut()
                };
                let Some(mut wargs) = args_parse(
                    &raw const (*(&raw const window_copy_cmd_table
                        as *const window_copy_cmd_entry)
                        .offset(i as isize))
                    .args,
                    values,
                    count,
                    &mut error,
                ) else {
                    break;
                };
                cs.wargs = &raw mut *wargs;
                clear = window_copy_cmd_table[i as usize].clear;
                action = window_copy_cmd_table[i as usize]
                    .f
                    .expect("non-null function pointer")(&mut cs);
                drop(wargs);
                cs.wargs = ::core::ptr::null_mut::<args>();
                break;
            } else {
                i = i.wrapping_add(1);
            }
        }
        if strncmp(command, c"search-".as_ptr(), 7 as size_t) != 0 as ::core::ffi::c_int
            && !(*data).searchmark.is_empty()
        {
            keys = options_get_number((*(*wp).window).options_ptr(), c"mode-keys".as_ptr())
                as ::core::ffi::c_int;
            if clear as ::core::ffi::c_uint
                == WINDOW_COPY_CMD_CLEAR_EMACS_ONLY as ::core::ffi::c_int as ::core::ffi::c_uint
                && keys == MODEKEY_VI
            {
                clear = WINDOW_COPY_CMD_CLEAR_NEVER;
            }
            if clear as ::core::ffi::c_uint
                != WINDOW_COPY_CMD_CLEAR_NEVER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                window_copy_clear_marks(wme);
                (*data).searchy = -(1 as ::core::ffi::c_int);
                (*data).searchx = (*data).searchy;
            }
            if action as ::core::ffi::c_uint
                == WINDOW_COPY_CMD_NOTHING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                action = WINDOW_COPY_CMD_REDRAW;
            }
        }
        wme.prefix = 1 as u_int;
        if action as ::core::ffi::c_uint
            == WINDOW_COPY_CMD_CANCEL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            window_pane_reset_mode(wp);
        } else if action as ::core::ffi::c_uint
            == WINDOW_COPY_CMD_REDRAW as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            window_copy_redraw_screen(wme);
        } else if action as ::core::ffi::c_uint
            == WINDOW_COPY_CMD_NOTHING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            window_copy_redraw_lines(wme, 0 as u_int, 1 as u_int);
        }
    }
}
unsafe fn window_copy_scroll_to(
    wme: &mut window_mode_entry,
    mut px: u_int,
    mut py: u_int,
    mut no_redraw: ::core::ffi::c_int,
) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut gd: *mut grid = screen_grid_ptr(&mut *window_copy_backing(&mut *data));
        let mut offset: u_int = 0;
        let mut gap: u_int = 0;
        (*data).cx = px;
        if py >= (*gd).hsize.wrapping_sub((*data).oy)
            && py < (*gd).hsize.wrapping_sub((*data).oy).wrapping_add((*gd).sy)
        {
            (*data).cy = py.wrapping_sub((*gd).hsize.wrapping_sub((*data).oy));
        } else {
            gap = (*gd).sy.wrapping_div(4 as u_int);
            if py < (*gd).sy {
                offset = 0 as u_int;
                (*data).cy = py;
            } else if py > (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(gap) {
                offset = (*gd).hsize;
                (*data).cy = py.wrapping_sub((*gd).hsize);
            } else {
                offset = py.wrapping_add(gap).wrapping_sub((*gd).sy);
                (*data).cy = py.wrapping_sub(offset);
            }
            (*data).oy = (*gd).hsize.wrapping_sub(offset);
        }
        if no_redraw == 0 && !(*data).searchmark.is_empty() && (*data).timeout == 0 {
            window_copy_search_marks(
                wme,
                ::core::ptr::null_mut::<screen>(),
                (*data).searchregex,
                1 as ::core::ffi::c_int,
            );
        }
        window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
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
    cis: ::core::ffi::c_int,
) -> bool {
    let gc = grid_get_cell(gd, px, py);
    let sgc = grid_get_cell(sgd, spx, 0 as u_int);
    let ud = &gc.data;
    let sud = &sgc.data;
    if sud.data[0] == b'\t' && sud.size == 1 && gc.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0
    {
        return true;
    }
    if ud.size != sud.size || ud.width != sud.width {
        return false;
    }
    if cis != 0 && ud.size == 1 {
        return tolower(ud.data[0] as ::core::ffi::c_int) == sud.data[0] as ::core::ffi::c_int;
    }
    ud.data[..ud.size as usize] == sud.data[..sud.size as usize]
}
unsafe fn window_copy_search_lr(
    mut gd: *mut grid,
    mut sgd: *mut grid,
    mut py: u_int,
    mut first: u_int,
    mut last: u_int,
    mut cis: ::core::ffi::c_int,
) -> Option<u_int> {
    unsafe {
        let mut ax: u_int = 0;
        let mut bx: u_int = 0;
        let mut px: u_int = 0;
        let mut pywrap: u_int = 0;
        let mut endline: u_int = 0;
        let mut padding: u_int = 0;
        let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
        let mut gc = grid_default_cell;
        endline = (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int);
        ax = first;
        while ax < last {
            padding = 0 as u_int;
            bx = 0 as u_int;
            while bx < (*sgd).sx {
                px = ax.wrapping_add(bx).wrapping_add(padding);
                pywrap = py;
                while px >= (*gd).sx && pywrap < endline {
                    gl = grid_get_line(&mut *gd, pywrap);
                    if !(*gl).flags & GRID_LINE_WRAPPED != 0 {
                        break;
                    }
                    px = px.wrapping_sub((*gd).sx);
                    pywrap = pywrap.wrapping_add(1);
                }
                if px.wrapping_sub(padding) >= (*gd).sx {
                    break;
                }
                gc = grid_get_cell(&*gd, px, pywrap);
                if gc.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
                    padding = padding.wrapping_add(
                        (gc.data.width as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as u_int,
                    );
                }
                if !window_copy_search_compare(&*gd, px, pywrap, &*sgd, bx, cis) {
                    break;
                }
                bx = bx.wrapping_add(1);
            }
            if bx == (*sgd).sx {
                return Some(ax);
            }
            ax = ax.wrapping_add(1);
        }
        None
    }
}
unsafe fn window_copy_search_rl(
    mut gd: *mut grid,
    mut sgd: *mut grid,
    mut py: u_int,
    mut first: u_int,
    mut last: u_int,
    mut cis: ::core::ffi::c_int,
) -> Option<u_int> {
    unsafe {
        let mut ax: u_int = 0;
        let mut bx: u_int = 0;
        let mut px: u_int = 0;
        let mut pywrap: u_int = 0;
        let mut endline: u_int = 0;
        let mut padding: u_int = 0;
        let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
        let mut gc = grid_default_cell;
        endline = (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int);
        ax = last;
        while ax > first {
            padding = 0 as u_int;
            bx = 0 as u_int;
            while bx < (*sgd).sx {
                px = ax
                    .wrapping_sub(1 as u_int)
                    .wrapping_add(bx)
                    .wrapping_add(padding);
                pywrap = py;
                while px >= (*gd).sx && pywrap < endline {
                    gl = grid_get_line(&mut *gd, pywrap);
                    if !(*gl).flags & GRID_LINE_WRAPPED != 0 {
                        break;
                    }
                    px = px.wrapping_sub((*gd).sx);
                    pywrap = pywrap.wrapping_add(1);
                }
                if px.wrapping_sub(padding) >= (*gd).sx {
                    break;
                }
                gc = grid_get_cell(&*gd, px, pywrap);
                if gc.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
                    padding = padding.wrapping_add(
                        (gc.data.width as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as u_int,
                    );
                }
                if !window_copy_search_compare(&*gd, px, pywrap, &*sgd, bx, cis) {
                    break;
                }
                bx = bx.wrapping_add(1);
            }
            if bx == (*sgd).sx {
                return Some(ax.wrapping_sub(1 as u_int));
            }
            ax = ax.wrapping_sub(1);
        }
        None
    }
}
unsafe fn window_copy_search_lr_regex(
    mut gd: *mut grid,
    mut py: u_int,
    mut first: u_int,
    mut last: u_int,
    mut reg: *mut regex_t,
) -> Option<(u_int, u_int)> {
    unsafe {
        let mut ppx: u_int = 0;
        let mut psx: u_int = 0;
        let mut eflags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut endline: u_int = 0;
        let mut foundx: u_int = 0;
        let mut foundy: u_int = 0;
        let mut len: u_int = 0;
        let mut pywrap: u_int = 0;
        let mut regmatch: regmatch_t = regmatch_t { rm_so: 0, rm_eo: 0 };
        let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
        if first >= last {
            return None;
        }
        if first != 0 as u_int {
            eflags |= REG_NOTBOL;
        }
        let mut buf: Vec<u8> = vec![b'\0'];
        window_copy_stringify(gd, py, first, (*gd).sx, &mut buf);
        len = (*gd).sx.wrapping_sub(first);
        endline = (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int);
        pywrap = py;
        while pywrap < endline && len < WINDOW_COPY_SEARCH_MAX_LINE as u_int {
            gl = grid_get_line(&mut *gd, pywrap);
            if !(*gl).flags & GRID_LINE_WRAPPED != 0 {
                break;
            }
            pywrap = pywrap.wrapping_add(1);
            window_copy_stringify(gd, pywrap, 0 as u_int, (*gd).sx, &mut buf);
            len = len.wrapping_add((*gd).sx);
        }
        if regexec(
            reg,
            buf.as_ptr() as *const ::core::ffi::c_char,
            1 as size_t,
            &raw mut regmatch,
            eflags,
        ) == 0 as ::core::ffi::c_int
            && regmatch.rm_so != regmatch.rm_eo
        {
            foundx = first;
            foundy = py;
            window_copy_cstrtocellpos(
                gd,
                len,
                &mut foundx,
                &mut foundy,
                CStr::from_ptr(
                    buf.as_ptr().add(regmatch.rm_so as usize) as *const ::core::ffi::c_char
                ),
            );
            if foundy == py && foundx < last {
                ppx = foundx;
                len = len.wrapping_sub(foundx.wrapping_sub(first));
                window_copy_cstrtocellpos(
                    gd,
                    len,
                    &mut foundx,
                    &mut foundy,
                    CStr::from_ptr(
                        buf.as_ptr().add(regmatch.rm_eo as usize) as *const ::core::ffi::c_char
                    ),
                );
                psx = foundx;
                while foundy > py {
                    psx = psx.wrapping_add((*gd).sx);
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
    mut gd: *mut grid,
    mut py: u_int,
    mut first: u_int,
    mut last: u_int,
    mut reg: *mut regex_t,
) -> Option<(u_int, u_int)> {
    unsafe {
        let mut eflags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut endline: u_int = 0;
        let mut len: u_int = 0;
        let mut pywrap: u_int = 0;
        let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
        if first != 0 as u_int {
            eflags |= REG_NOTBOL;
        }
        let mut buf: Vec<u8> = vec![b'\0'];
        window_copy_stringify(gd, py, first, (*gd).sx, &mut buf);
        len = (*gd).sx.wrapping_sub(first);
        endline = (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int);
        pywrap = py;
        while pywrap < endline && len < WINDOW_COPY_SEARCH_MAX_LINE as u_int {
            gl = grid_get_line(&mut *gd, pywrap);
            if !(*gl).flags & GRID_LINE_WRAPPED != 0 {
                break;
            }
            pywrap = pywrap.wrapping_add(1);
            window_copy_stringify(gd, pywrap, 0 as u_int, (*gd).sx, &mut buf);
            len = len.wrapping_add((*gd).sx);
        }
        window_copy_last_regex(
            gd,
            py,
            first,
            last,
            len,
            CStr::from_ptr(buf.as_ptr() as *const ::core::ffi::c_char),
            reg,
            eflags,
        )
    }
}
unsafe fn window_copy_cellstring<'a>(gl: &grid_line, mut px: u_int) -> Cow<'a, [u8]> {
    unsafe {
        let mut ud = utf8_data::default();
        if px >= gl.cellsize() {
            return Cow::Borrowed(b" ");
        }
        let gce = &(*gl).celldata()[px as usize];
        if gce.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0 {
            return Cow::Borrowed(&[]);
        }
        if !(gce.flags as ::core::ffi::c_int) & GRID_FLAG_EXTENDED != 0 {
            return Cow::Borrowed(::core::slice::from_raw_parts(
                &raw const gce.c2rust_unnamed.data.data,
                1,
            ));
        }
        if gce.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
            return Cow::Borrowed(b"\t");
        }
        utf8_to_data(
            (*gl).extddata()[gce.c2rust_unnamed.offset as usize].data,
            &mut ud,
        );
        if ud.size as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            return Cow::Borrowed(&[]);
        }
        Cow::Owned(ud.data[..ud.size as usize].to_vec())
    }
}
unsafe fn window_copy_last_regex(
    mut gd: *mut grid,
    mut py: u_int,
    mut first: u_int,
    mut last: u_int,
    mut len: u_int,
    buf: &CStr,
    mut preg: *const regex_t,
    mut eflags: ::core::ffi::c_int,
) -> Option<(u_int, u_int)> {
    unsafe {
        let ppx: u_int;
        let mut psx: u_int;
        let mut foundx: u_int = 0;
        let mut foundy: u_int = 0;
        let mut oldx: u_int = 0;
        let mut px: u_int = 0 as u_int;
        let mut savepx: u_int = 0;
        let mut savesx: u_int = 0 as u_int;
        let mut regmatch: regmatch_t = regmatch_t { rm_so: 0, rm_eo: 0 };
        foundx = first;
        foundy = py;
        oldx = first;
        let buf = buf.as_ptr();
        while regexec(
            preg,
            buf.offset(px as isize),
            1 as size_t,
            &raw mut regmatch,
            eflags,
        ) == 0 as ::core::ffi::c_int
        {
            if regmatch.rm_so == regmatch.rm_eo {
                break;
            }
            window_copy_cstrtocellpos(
                gd,
                len,
                &mut foundx,
                &mut foundy,
                CStr::from_ptr(buf.offset(px as isize).offset(regmatch.rm_so as isize)),
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
                CStr::from_ptr(buf.offset(px as isize).offset(regmatch.rm_eo as isize)),
            );
            if foundy > py || foundx >= last {
                ppx = savepx;
                psx = foundx;
                while foundy > py {
                    psx = psx.wrapping_add((*gd).sx);
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
    mut gd: *mut grid,
    mut py: u_int,
    mut first: u_int,
    mut last: u_int,
    buf: &mut Vec<u8>,
) {
    unsafe {
        let mut ax: u_int = 0;
        let mut gl: Option<&grid_line>;
        gl = grid_peek_line(&*gd, py);
        let Some(gl) = gl else {
            return;
        };
        buf.pop();
        ax = first;
        while ax < last {
            buf.extend_from_slice(&window_copy_cellstring(gl, ax));
            ax = ax.wrapping_add(1);
        }
        buf.push(b'\0');
    }
}
unsafe fn window_copy_cstrtocellpos<'a>(
    mut gd: *mut grid,
    mut ncells: u_int,
    ppx: &mut u_int,
    ppy: &mut u_int,
    str: &CStr,
) {
    unsafe {
        let mut cell: u_int = 0;
        let mut ccell: u_int = 0;
        let mut px: u_int = 0;
        let mut pywrap: u_int = 0;
        let mut pos: u_int = 0;
        let mut match_0: ::core::ffi::c_int = 0;
        let mut gl: Option<&grid_line>;
        let mut cells: Vec<window_copy_search_cell<'a>> = Vec::with_capacity(ncells as usize);
        cell = 0 as u_int;
        px = *ppx;
        pywrap = *ppy;
        gl = grid_peek_line(&*gd, pywrap);
        let Some(mut line) = gl else {
            return;
        };
        while cell < ncells {
            cells.push(window_copy_search_cell {
                d: window_copy_cellstring(line, px),
            });
            cell = cell.wrapping_add(1);
            px = px.wrapping_add(1);
            if !(px == (*gd).sx) {
                continue;
            }
            px = 0 as u_int;
            pywrap = pywrap.wrapping_add(1);
            gl = grid_peek_line(&*gd, pywrap);
            match gl {
                Some(next) => line = next,
                None => break,
            }
        }
        ncells = cells.len() as u_int;
        cell = 0 as u_int;
        let str = str.to_bytes();
        while cell < ncells {
            ccell = cell;
            pos = 0 as u_int;
            match_0 = 1 as ::core::ffi::c_int;
            while ccell < ncells {
                let Some(&ch) = str.get(pos as usize) else {
                    match_0 = 0 as ::core::ffi::c_int;
                    break;
                };
                let d = &cells[ccell as usize].d;
                if d.len() == 1 {
                    if ch != d[0] {
                        match_0 = 0 as ::core::ffi::c_int;
                        break;
                    }
                    pos = pos.wrapping_add(1);
                } else {
                    let dlen = d.len().min(str.len() - pos as usize);
                    if str[pos as usize..pos as usize + dlen] != d[..dlen] {
                        match_0 = 0 as ::core::ffi::c_int;
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
        while px >= (*gd).sx {
            px = px.wrapping_sub((*gd).sx);
            pywrap = pywrap.wrapping_add(1);
        }
        *ppx = px;
        *ppy = pywrap;
    }
}
unsafe fn window_copy_move_left(
    mut s: *mut screen,
    fx: &mut u_int,
    fy: &mut u_int,
    mut wrapflag: ::core::ffi::c_int,
) {
    unsafe {
        if *fx == 0 as u_int {
            if *fy == 0 as u_int {
                if wrapflag != 0 {
                    *fx = (*screen_grid_ptr(&mut *s)).sx.wrapping_sub(1 as u_int);
                    *fy = (*screen_grid_ptr(&mut *s))
                        .hsize
                        .wrapping_add((*screen_grid_ptr(&mut *s)).sy)
                        .wrapping_sub(1 as u_int);
                }
                return;
            }
            *fx = (*screen_grid_ptr(&mut *s)).sx.wrapping_sub(1 as u_int);
            *fy = (*fy).wrapping_sub(1 as u_int);
        } else {
            *fx = (*fx).wrapping_sub(1 as u_int);
        };
    }
}
unsafe fn window_copy_move_right(
    mut s: *mut screen,
    fx: &mut u_int,
    fy: &mut u_int,
    mut wrapflag: ::core::ffi::c_int,
) {
    unsafe {
        if *fx == (*screen_grid_ptr(&mut *s)).sx.wrapping_sub(1 as u_int) {
            if *fy
                == (*screen_grid_ptr(&mut *s))
                    .hsize
                    .wrapping_add((*screen_grid_ptr(&mut *s)).sy)
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
fn window_copy_is_lowercase(s: &CStr) -> ::core::ffi::c_int {
    for &byte in s.to_bytes() {
        if byte as ::core::ffi::c_int != tolower(byte as ::core::ffi::c_int) {
            return 0 as ::core::ffi::c_int;
        }
    }
    1 as ::core::ffi::c_int
}
unsafe fn window_copy_search_back_overlap(
    mut gd: *mut grid,
    mut preg: *mut regex_t,
    ppx: &mut u_int,
    psx: &mut u_int,
    ppy: &mut u_int,
    mut endline: u_int,
) {
    unsafe {
        let mut endx: u_int = 0;
        let mut endy: u_int = 0;
        let mut oldendx: u_int = 0;
        let mut oldendy: u_int = 0;
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut sx: u_int = 0;
        let mut found: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        oldendx = (*ppx).wrapping_add(*psx);
        oldendy = (*ppy).wrapping_sub(1 as u_int);
        while oldendx > (*gd).sx.wrapping_sub(1 as u_int) {
            oldendx = oldendx.wrapping_sub((*gd).sx);
            oldendy = oldendy.wrapping_add(1);
        }
        endx = oldendx;
        endy = oldendy;
        px = *ppx;
        py = *ppy;
        while found != 0
            && px == 0 as u_int
            && py.wrapping_sub(1 as u_int) > endline
            && grid_get_line(&mut *gd, py.wrapping_sub(2 as u_int)).flags & GRID_LINE_WRAPPED != 0
            && endx == oldendx
            && endy == oldendy
        {
            py = py.wrapping_sub(1);
            (px, sx, found) = match window_copy_search_rl_regex(
                gd,
                py.wrapping_sub(1 as u_int),
                0 as u_int,
                (*gd).sx,
                preg,
            ) {
                Some((found_px, found_sx)) => (found_px, found_sx, 1),
                None => (0 as u_int, 0 as u_int, 0),
            };
            if found != 0 {
                endx = px.wrapping_add(sx);
                endy = py.wrapping_sub(1 as u_int);
                while endx > (*gd).sx.wrapping_sub(1 as u_int) {
                    endx = endx.wrapping_sub((*gd).sx);
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
unsafe fn window_copy_search_jump(
    wme: &mut window_mode_entry,
    mut gd: *mut grid,
    mut sgd: *mut grid,
    mut fx: u_int,
    mut fy: u_int,
    mut endline: u_int,
    mut cis: ::core::ffi::c_int,
    mut wrap: ::core::ffi::c_int,
    mut direction: ::core::ffi::c_int,
    mut regex: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut i: u_int = 0;
        let mut px: u_int = 0;
        let mut sx: u_int = 0;
        let mut found: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut cflags: ::core::ffi::c_int = REG_EXTENDED;
        let mut reg: regex_t = regex_t::default();
        if regex != 0 {
            let mut sbuf: Vec<u8> = vec![b'\0'];
            window_copy_stringify(sgd, 0 as u_int, 0 as u_int, (*sgd).sx, &mut sbuf);
            if cis != 0 {
                cflags |= REG_ICASE;
            }
            if regcomp(
                &raw mut reg,
                sbuf.as_ptr() as *const ::core::ffi::c_char,
                cflags,
            ) != 0 as ::core::ffi::c_int
            {
                return 0 as ::core::ffi::c_int;
            }
        }
        if direction != 0 {
            i = fy;
            while i <= endline {
                if regex != 0 {
                    (px, sx, found) =
                        match window_copy_search_lr_regex(gd, i, fx, (*gd).sx, &raw mut reg) {
                            Some((found_px, found_sx)) => (found_px, found_sx, 1),
                            None => (0 as u_int, 0 as u_int, 0),
                        };
                } else {
                    (px, found) = match window_copy_search_lr(gd, sgd, i, fx, (*gd).sx, cis) {
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
                        &raw mut reg,
                    ) {
                        Some((found_px, found_sx)) => (found_px, found_sx, 1),
                        None => (0 as u_int, 0 as u_int, 0),
                    };
                    if found != 0 {
                        window_copy_search_back_overlap(
                            gd, &mut reg, &mut px, &mut sx, &mut i, endline,
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
                    fx = (*gd).sx.wrapping_sub(1 as u_int);
                    i = i.wrapping_sub(1);
                }
            }
        }
        if regex != 0 {
            regfree(&raw mut reg);
        }
        if found != 0 {
            window_copy_scroll_to(wme, px, i, 1 as ::core::ffi::c_int);
            return 1 as ::core::ffi::c_int;
        }
        if wrap != 0 {
            return window_copy_search_jump(
                wme,
                gd,
                sgd,
                if direction != 0 {
                    0 as u_int
                } else {
                    (*gd).sx.wrapping_sub(1 as u_int)
                },
                if direction != 0 {
                    0 as u_int
                } else {
                    (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int)
                },
                fy,
                cis,
                0 as ::core::ffi::c_int,
                direction,
                regex,
            );
        }
        0 as ::core::ffi::c_int
    }
}
unsafe fn window_copy_move_after_search_mark(
    data: &mut window_copy_mode_data,
    fx: &mut u_int,
    fy: &mut u_int,
    mut wrapflag: ::core::ffi::c_int,
) {
    unsafe {
        let mut s: *mut screen = window_copy_backing(data);
        let start = window_copy_search_mark_at(data, *fx, *fy);
        if start.is_some_and(|start| {
            (&data.searchmark)[start as usize] as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        }) {
            let start = start.expect("the mark just looked at");
            while let Some(at) = window_copy_search_mark_at(data, *fx, *fy) {
                if (&data.searchmark)[at as usize] as ::core::ffi::c_int
                    != (&data.searchmark)[start as usize] as ::core::ffi::c_int
                {
                    break;
                }
                if wrapflag == 0
                    && *fx == (*screen_grid_ptr(&mut *s)).sx.wrapping_sub(1 as u_int)
                    && *fy
                        == (*screen_grid_ptr(&mut *s))
                            .hsize
                            .wrapping_add((*screen_grid_ptr(&mut *s)).sy)
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
    mut direction: ::core::ffi::c_int,
    mut regex: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut wp: *mut window_pane = wme.wp;
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut s: *mut screen = window_copy_backing(&mut *data);
        let mut ctx = screen_write_ctx::default();
        let mut gd: *mut grid = screen_grid_ptr(&mut *s);
        let mut str: *const ::core::ffi::c_char = (*data).searchstr_ptr();
        let mut at: u_int = 0;
        let mut endline: u_int = 0;
        let mut fx: u_int = 0;
        let mut fy: u_int = 0;
        let mut ssx: u_int = 0;
        let mut cis: ::core::ffi::c_int = 0;
        let mut found: ::core::ffi::c_int = 0;
        let mut keys: ::core::ffi::c_int = 0;
        let mut visible_only: ::core::ffi::c_int = 0;
        let mut wrapflag: ::core::ffi::c_int = 0;
        if regex != 0
            && *str.offset(strcspn(str, c"^$*+()?[].\\".as_ptr()) as isize) as ::core::ffi::c_int
                == '\0' as i32
        {
            regex = 0 as ::core::ffi::c_int;
        }
        (*data).searchdirection = direction;
        if (*data).timeout != 0 {
            return 0 as ::core::ffi::c_int;
        }
        if (*data).searchall != 0 || (*wp).searchstr.is_none() || (*wp).searchregex != regex {
            visible_only = 0 as ::core::ffi::c_int;
            (*data).searchall = 0 as ::core::ffi::c_int;
        } else {
            visible_only =
                ((*wp).searchstr.as_deref() == Some(CStr::from_ptr(str))) as ::core::ffi::c_int;
        }
        if visible_only == 0 as ::core::ffi::c_int && !(*data).searchmark.is_empty() {
            window_copy_clear_marks(wme);
        }
        (*wp).searchstr = Some(CStr::from_ptr(str).to_owned());
        (*wp).searchregex = regex;
        fx = (*data).cx;
        fy = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_sub((*data).oy)
            .wrapping_add((*data).cy);
        ssx = screen_write_strlen(c"%s".as_ptr(), fmt_args![str]) as u_int;
        if ssx == 0 as u_int {
            return 0 as ::core::ffi::c_int;
        }
        let mut ss = screen::new(ssx, 1 as u_int, 0 as u_int);
        screen_write_start(&mut ctx, &mut ss);
        screen_write_nputs(
            &mut ctx,
            -(1 as ::core::ffi::c_int) as ssize_t,
            &grid_default_cell,
            c"%s".as_ptr(),
            fmt_args![str],
        );
        screen_write_stop(&mut ctx);
        wrapflag = options_get_number((*(*wp).window).options_ptr(), c"wrap-search".as_ptr())
            as ::core::ffi::c_int;
        cis = window_copy_is_lowercase(CStr::from_ptr(str));
        keys = options_get_number((*(*wp).window).options_ptr(), c"mode-keys".as_ptr())
            as ::core::ffi::c_int;
        if direction != 0 {
            if keys == MODEKEY_VI {
                if !(*data).searchmark.is_empty() {
                    window_copy_move_after_search_mark(&mut *data, &mut fx, &mut fy, wrapflag);
                } else {
                    window_copy_move_right(s, &mut fx, &mut fy, wrapflag);
                }
            }
            endline = (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int);
        } else {
            window_copy_move_left(s, &mut fx, &mut fy, wrapflag);
            endline = 0 as u_int;
        }
        found = window_copy_search_jump(
            wme,
            gd,
            screen_grid_ptr(&mut ss),
            fx,
            fy,
            endline,
            cis,
            wrapflag,
            direction,
            regex,
        );
        if found != 0 {
            window_copy_search_marks(wme, &raw mut ss, regex, visible_only);
            fx = (*data).cx;
            fy = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
                .hsize
                .wrapping_sub((*data).oy)
                .wrapping_add((*data).cy);
            if direction != 0
                && window_copy_search_mark_at(&mut *data, fx, fy).is_some_and(|at| {
                    at > 0 as u_int
                        && !(*data).searchmark.is_empty()
                        && (&(*data).searchmark)[at as usize] as ::core::ffi::c_int
                            == (&(*data).searchmark)[at.wrapping_sub(1 as u_int) as usize]
                                as ::core::ffi::c_int
                })
            {
                window_copy_move_after_search_mark(&mut *data, &mut fx, &mut fy, wrapflag);
                window_copy_search_jump(
                    wme,
                    gd,
                    screen_grid_ptr(&mut ss),
                    fx,
                    fy,
                    endline,
                    cis,
                    wrapflag,
                    direction,
                    regex,
                );
                fx = (*data).cx;
                fy = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
                    .hsize
                    .wrapping_sub((*data).oy)
                    .wrapping_add((*data).cy);
            }
            if direction != 0 {
                if keys == MODEKEY_EMACS {
                    window_copy_move_after_search_mark(&mut *data, &mut fx, &mut fy, wrapflag);
                    (*data).cx = fx;
                    (*data).cy = fy
                        .wrapping_sub((*screen_grid_ptr(&mut *window_copy_backing(&mut *data))).hsize)
                        .wrapping_add((*data).oy);
                }
            } else if let Some(start) = window_copy_search_mark_at(&mut *data, fx, fy) {
                while window_copy_search_mark_at(&mut *data, fx, fy).is_some_and(|at| {
                    !(*data).searchmark.is_empty()
                        && (&(*data).searchmark)[at as usize] as ::core::ffi::c_int
                            == (&(*data).searchmark)[start as usize] as ::core::ffi::c_int
                }) {
                    (*data).cx = fx;
                    (*data).cy = fy
                        .wrapping_sub((*screen_grid_ptr(&mut *window_copy_backing(&mut *data))).hsize)
                        .wrapping_add((*data).oy);
                    if at == 0 as u_int {
                        break;
                    }
                    window_copy_move_left(s, &mut fx, &mut fy, 0 as ::core::ffi::c_int);
                }
            }
        }
        window_copy_redraw_screen(wme);
        screen_free(&mut ss);
        found
    }
}
/// The first and last line of what the pane shows, the first walked back
/// over the wrapped lines it continues.
unsafe fn window_copy_visible_lines(data: &mut window_copy_mode_data) -> (u_int, u_int) {
    unsafe {
        let mut gd: *mut grid = screen_grid_ptr(&mut *window_copy_backing(data));
        let mut gl: Option<&grid_line>;
        let mut start = (*gd).hsize.wrapping_sub(data.oy);
        while start > 0 as u_int {
            gl = grid_peek_line(&*gd, start.wrapping_sub(1 as u_int));
            if !gl.is_some_and(|gl| gl.flags & GRID_LINE_WRAPPED != 0) {
                break;
            }
            start = start.wrapping_sub(1);
        }
        let end = (*gd).hsize.wrapping_sub(data.oy).wrapping_add((*gd).sy);
        (start, end)
    }
}
/// Where `px`,`py` falls in the search mark, or nothing when it is not on
/// the part of the history the pane shows.
unsafe fn window_copy_search_mark_at(
    data: &mut window_copy_mode_data,
    mut px: u_int,
    mut py: u_int,
) -> Option<u_int> {
    unsafe {
        let mut s: *mut screen = window_copy_backing(data);
        let mut gd: *mut grid = screen_grid_ptr(&mut *s);
        if py < (*gd).hsize.wrapping_sub(data.oy) {
            return None;
        }
        if py
            > (*gd)
                .hsize
                .wrapping_sub(data.oy)
                .wrapping_add((*gd).sy)
                .wrapping_sub(1 as u_int)
        {
            return None;
        }
        Some(
            py.wrapping_sub((*gd).hsize.wrapping_sub(data.oy))
                .wrapping_mul((*gd).sx)
                .wrapping_add(px),
        )
    }
}
fn window_copy_clip_width(mut width: u_int, mut b: u_int, mut sx: u_int, mut sy: u_int) -> u_int {
    if b.wrapping_add(width) > sx.wrapping_mul(sy) {
        sx.wrapping_mul(sy).wrapping_sub(b)
    } else {
        width
    }
}
unsafe fn window_copy_search_mark_match(
    data: &mut window_copy_mode_data,
    mut px: u_int,
    mut py: u_int,
    mut width: u_int,
    mut regex: ::core::ffi::c_int,
) -> u_int {
    unsafe {
        let mut gd: *mut grid = screen_grid_ptr(&mut *window_copy_backing(data));
        let mut gc = grid_default_cell;
        let mut i: u_int = 0;
        let mut w: u_int = width;
        let mut sx: u_int = (*gd).sx;
        let mut sy: u_int = (*gd).sy;
        if let Some(b) = window_copy_search_mark_at(data, px, py) {
            width = window_copy_clip_width(width, b, sx, sy);
            w = width;
            i = b;
            while i < b.wrapping_add(w) {
                if regex == 0 {
                    gc = grid_get_cell(&*gd, px.wrapping_add(i.wrapping_sub(b)), py);
                    if gc.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
                        w = w.wrapping_add(
                            (gc.data.width as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                                as u_int,
                        );
                    }
                    w = window_copy_clip_width(w, b, sx, sy);
                }
                if (&data.searchmark)[i as usize] as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                    (&mut data.searchmark)[i as usize] = data.searchgen;
                }
                i = i.wrapping_add(1);
            }
            if data.searchgen as ::core::ffi::c_int == UCHAR_MAX {
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
    mut ssp: *mut screen,
    mut regex: ::core::ffi::c_int,
    mut visible_only: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut s: *mut screen = window_copy_backing(&mut *data);
        let mut ss = screen::default();
        let mut ctx = screen_write_ctx::default();
        let mut gd: *mut grid = screen_grid_ptr(&mut *s);
        let mut gc = grid_default_cell;
        let mut found: ::core::ffi::c_int = 0;
        let mut cis: ::core::ffi::c_int = 0;
        let mut stopped: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut cflags: ::core::ffi::c_int = REG_EXTENDED;
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut nfound: u_int = 0 as u_int;
        let mut width: u_int = 0;
        let mut start: u_int = 0;
        let mut end: u_int = 0;
        let mut sx: u_int = (*gd).sx;
        let mut sy: u_int = (*gd).sy;
        let mut reg: regex_t = regex_t::default();
        let mut stop: uint64_t = 0 as uint64_t;
        let mut tstart: uint64_t = 0;
        let mut t: uint64_t = 0;
        if ssp.is_null() {
            width = screen_write_strlen(c"%s".as_ptr(), fmt_args![(*data).searchstr.as_deref()])
                as u_int;
            screen_init(&mut ss, width, 1 as u_int, 0 as u_int);
            screen_write_start(&mut ctx, &mut ss);
            screen_write_nputs(
                &mut ctx,
                -(1 as ::core::ffi::c_int) as ssize_t,
                &grid_default_cell,
                c"%s".as_ptr(),
                fmt_args![(*data).searchstr.as_deref()],
            );
            screen_write_stop(&mut ctx);
            ssp = &raw mut ss;
        } else {
            width = (*screen_grid_ptr(&mut *ssp)).sx;
        }
        cis = window_copy_is_lowercase((*data).searchstr.as_deref().unwrap_or(c""));
        if regex != 0 {
            let mut sbuf: Vec<u8> = vec![b'\0'];
            window_copy_stringify(
                screen_grid_ptr(&mut *ssp),
                0 as u_int,
                0 as u_int,
                (*screen_grid_ptr(&mut *ssp)).sx,
                &mut sbuf,
            );
            if cis != 0 {
                cflags |= REG_ICASE;
            }
            if regcomp(
                &raw mut reg,
                sbuf.as_ptr() as *const ::core::ffi::c_char,
                cflags,
            ) != 0 as ::core::ffi::c_int
            {
                return 0 as ::core::ffi::c_int;
            }
        }
        tstart = get_timer();
        if visible_only != 0 {
            (start, end) = window_copy_visible_lines(&mut *data);
        } else {
            start = 0 as u_int;
            end = (*gd).hsize.wrapping_add(sy);
            stop = get_timer().wrapping_add(WINDOW_COPY_SEARCH_ALL_TIMEOUT as uint64_t);
        }
        loop {
            (*data).searchmark.clear();
            (*data)
                .searchmark
                .resize((sx as usize).saturating_mul(sy as usize), 0);
            (*data).searchgen = 1 as u_char;
            py = start;
            while py < end {
                px = 0 as u_int;
                loop {
                    if regex != 0 {
                        (px, width, found) =
                            match window_copy_search_lr_regex(gd, py, px, sx, &raw mut reg) {
                                Some((found_px, found_width)) => (found_px, found_width, 1),
                                None => (0 as u_int, 0 as u_int, 0),
                            };
                        gc = grid_get_cell(
                            &*gd,
                            px.wrapping_add(width).wrapping_sub(1 as u_int),
                            py,
                        );
                        if gc.data.width as ::core::ffi::c_int > 2 as ::core::ffi::c_int {
                            width = width.wrapping_add(
                                (gc.data.width as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                                    as u_int,
                            );
                        }
                        if found == 0 {
                            break;
                        }
                    } else {
                        (px, found) = match window_copy_search_lr(
                            gd,
                            screen_grid_ptr(&mut *ssp),
                            py,
                            px,
                            sx,
                            cis,
                        ) {
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
                    (*data).timeout = 1 as ::core::ffi::c_int;
                    break;
                } else if stop != 0 as uint64_t && t > stop {
                    stopped = 1 as ::core::ffi::c_int;
                    break;
                } else {
                    py = py.wrapping_add(1);
                }
            }
            if (*data).timeout != 0 {
                window_copy_clear_marks(wme);
                break;
            } else if stopped != 0 && stop != 0 as uint64_t {
                (start, end) = window_copy_visible_lines(&mut *data);
                stop = 0 as uint64_t;
            } else {
                if visible_only == 0 {
                    if stopped != 0 {
                        if nfound > 1000 as u_int {
                            (*data).searchcount = 1000 as ::core::ffi::c_int;
                        } else if nfound > 100 as u_int {
                            (*data).searchcount = 100 as ::core::ffi::c_int;
                        } else if nfound > 10 as u_int {
                            (*data).searchcount = 10 as ::core::ffi::c_int;
                        } else {
                            (*data).searchcount = -(1 as ::core::ffi::c_int);
                        }
                        (*data).searchmore = 1 as ::core::ffi::c_int;
                    } else {
                        (*data).searchcount = nfound as ::core::ffi::c_int;
                        (*data).searchmore = 0 as ::core::ffi::c_int;
                    }
                }
                break;
            }
        }
        if ssp == &raw mut ss {
            screen_free(&mut ss);
        }
        if regex != 0 {
            regfree(&raw mut reg);
        }
        1 as ::core::ffi::c_int
    }
}
unsafe fn window_copy_clear_marks(wme: &mut window_mode_entry) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        (*data).searchcount = -(1 as ::core::ffi::c_int);
        (*data).searchmore = 0 as ::core::ffi::c_int;
        (*data).searchmark.clear();
    }
}
unsafe fn window_copy_search_up(
    wme: &mut window_mode_entry,
    mut regex: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe { window_copy_search(wme, 0 as ::core::ffi::c_int, regex) }
}
unsafe fn window_copy_search_down(
    wme: &mut window_mode_entry,
    mut regex: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe { window_copy_search(wme, 1 as ::core::ffi::c_int, regex) }
}
unsafe fn window_copy_goto_line(wme: &mut window_mode_entry, linestr: &CStr) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut hsize: u_int = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data))).hsize;
        let mut line: u_int = 0;
        let Ok(lineno) = strtonum(
            linestr.as_ptr(),
            -(1 as ::core::ffi::c_int) as ::core::ffi::c_longlong,
            INT_MAX as ::core::ffi::c_longlong,
        ) else {
            return;
        };
        let mut lineno = lineno as ::core::ffi::c_int;
        if window_copy_line_number_is_absolute(wme) != 0 {
            if lineno <= 0 as ::core::ffi::c_int {
                line = 1 as u_int;
            } else if lineno as u_int > hsize.wrapping_add(1 as u_int) {
                line = hsize.wrapping_add(1 as u_int);
            } else {
                line = lineno as u_int;
            }
            (*data).oy = hsize.wrapping_sub(line.wrapping_sub(1 as u_int));
        } else {
            if lineno < 0 as ::core::ffi::c_int || lineno as u_int > hsize {
                lineno = hsize as ::core::ffi::c_int;
            }
            (*data).oy = lineno as u_int;
        }
        window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        window_copy_redraw_screen(wme);
    }
}
/// The first and last cell of the run of marks `at` stands in.
unsafe fn window_copy_match_start_end(
    data: &mut window_copy_mode_data,
    mut at: u_int,
) -> (u_int, u_int) {
    unsafe {
        let mut gd: *mut grid = screen_grid_ptr(&mut *window_copy_backing(data));
        let mut last: u_int = (*gd).sy.wrapping_mul((*gd).sx).wrapping_sub(1 as u_int);
        let mut mark: u_char = (&data.searchmark)[at as usize];
        let mut end = at;
        let mut start = end;
        while start != 0 as u_int
            && (&data.searchmark)[start as usize] as ::core::ffi::c_int
                == mark as ::core::ffi::c_int
        {
            start = start.wrapping_sub(1);
        }
        if (&data.searchmark)[start as usize] as ::core::ffi::c_int != mark as ::core::ffi::c_int {
            start = start.wrapping_add(1);
        }
        while end != last
            && (&data.searchmark)[end as usize] as ::core::ffi::c_int == mark as ::core::ffi::c_int
        {
            end = end.wrapping_add(1);
        }
        if (&data.searchmark)[end as usize] as ::core::ffi::c_int != mark as ::core::ffi::c_int {
            end = end.wrapping_sub(1);
        }
        (start, end)
    }
}
/// The text of the search match the cursor stands in, or nothing when the
/// screen carries no marks or the cursor is not on one.
unsafe fn window_copy_match_at_cursor(data: *mut window_copy_mode_data) -> Option<CString> {
    unsafe {
        let gd: *mut grid = screen_grid_ptr(&mut *window_copy_backing(&mut *data));
        let mut gc = grid_default_cell;
        let mut at: u_int = 0;
        let mut start: u_int = 0;
        let mut end: u_int = 0;
        let sx: u_int = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data))).sx;
        if (*data).searchmark.is_empty() {
            return None;
        }
        let cy = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_sub((*data).oy)
            .wrapping_add((*data).cy);
        let Some(found_at) = window_copy_search_mark_at(&mut *data, (*data).cx, cy) else {
            return None;
        };
        at = found_at;
        if (&(*data).searchmark)[at as usize] as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            && (at == 0 as u_int || {
                at = at.wrapping_sub(1);
                (&(*data).searchmark)[at as usize] as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            })
        {
            return None;
        }
        (start, end) = window_copy_match_start_end(&mut *data, at);
        let mut buf: Vec<u8> = Vec::new();
        at = start;
        while at <= end {
            let py = at.wrapping_div(sx);
            let px = at.wrapping_sub(py.wrapping_mul(sx));
            gc = grid_get_cell(
                &*gd,
                px,
                (*gd).hsize.wrapping_add(py).wrapping_sub((*data).oy),
            );
            if gc.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
                buf.push(b'\t');
            } else if !(gc.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0) {
                buf.extend_from_slice(::core::slice::from_raw_parts(
                    &raw const gc.data.data as *const u8,
                    gc.data.size as usize,
                ));
            }
            at = at.wrapping_add(1);
        }
        if buf.is_empty() {
            return None;
        }
        Some(CString::from_vec_unchecked(buf))
    }
}
unsafe fn window_copy_update_style(
    wme: &mut window_mode_entry,
    mut fx: u_int,
    mut fy: u_int,
    gc: &mut grid_cell,
    mgc: &grid_cell,
    cgc: &grid_cell,
    mkgc: &grid_cell,
) {
    unsafe {
        let mut wp: *mut window_pane = wme.wp;
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut mark: u_int = 0;
        let mut start: u_int = 0;
        let mut end: u_int = 0;
        let mut cy: u_int = 0;
        let mut cursor: u_int = 0;
        let mut current: u_int = 0;
        let mut inv: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut found: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut keys: ::core::ffi::c_int = 0;
        if (*data).showmark != 0 && fy == (*data).my {
            (*gc).attr = (*mkgc).attr;
            if fx == (*data).mx {
                inv = 1 as ::core::ffi::c_int;
            }
            if inv != 0 {
                (*gc).fg = (*mkgc).bg;
                (*gc).bg = (*mkgc).fg;
            } else {
                (*gc).fg = (*mkgc).fg;
                (*gc).bg = (*mkgc).bg;
            }
        }
        if (*data).searchmark.is_empty() {
            return;
        }
        let Some(found_at) = window_copy_search_mark_at(&mut *data, fx, fy) else {
            return;
        };
        current = found_at;
        mark = (&(*data).searchmark)[current as usize] as u_int;
        if mark == 0 as u_int {
            return;
        }
        cy = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_sub((*data).oy)
            .wrapping_add((*data).cy);
        if let Some(found_at) = window_copy_search_mark_at(&mut *data, (*data).cx, cy) {
            cursor = found_at;
            keys = options_get_number((*(*wp).window).options_ptr(), c"mode-keys".as_ptr())
                as ::core::ffi::c_int;
            if cursor != 0 as u_int && keys == MODEKEY_EMACS && (*data).searchdirection != 0 {
                if (&(*data).searchmark)[cursor.wrapping_sub(1 as u_int) as usize] as u_int == mark
                {
                    cursor = cursor.wrapping_sub(1);
                    found = 1 as ::core::ffi::c_int;
                }
            } else if (&(*data).searchmark)[cursor as usize] as u_int == mark {
                found = 1 as ::core::ffi::c_int;
            }
            if found != 0 {
                (start, end) = window_copy_match_start_end(&mut *data, cursor);
                if current >= start && current <= end {
                    (*gc).attr = (*cgc).attr;
                    if inv != 0 {
                        (*gc).fg = (*cgc).bg;
                        (*gc).bg = (*cgc).fg;
                    } else {
                        (*gc).fg = (*cgc).fg;
                        (*gc).bg = (*cgc).bg;
                    }
                    return;
                }
            }
        }
        (*gc).attr = (*mgc).attr;
        if inv != 0 {
            (*gc).fg = (*mgc).bg;
            (*gc).bg = (*mgc).fg;
        } else {
            (*gc).fg = (*mgc).fg;
            (*gc).bg = (*mgc).bg;
        };
    }
}
unsafe fn window_copy_write_one(
    wme: &mut window_mode_entry,
    ctx: &mut screen_write_ctx,
    mut px: u_int,
    mut py: u_int,
    mut fy: u_int,
    mut nx: u_int,
    mgc: &grid_cell,
    cgc: &grid_cell,
    mkgc: &grid_cell,
) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut gd: *mut grid = screen_grid_ptr(&mut *window_copy_backing(&mut *data));
        let mut gc = grid_default_cell;
        let mut fx: u_int = 0;
        screen_write_cursormove(
            ctx,
            px as ::core::ffi::c_int,
            py as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        fx = 0 as u_int;
        while fx < nx {
            gc = grid_get_cell(&*gd, fx, fy);
            if fx.wrapping_add(gc.data.width as u_int) <= nx {
                window_copy_update_style(wme, fx, fy, &mut gc, mgc, cgc, mkgc);
                screen_write_cell(ctx, &mut gc);
            }
            fx = fx.wrapping_add(1);
        }
    }
}
unsafe fn window_copy_line_number_mode(wme: &mut window_mode_entry) -> ::core::ffi::c_int {
    unsafe {
        let mut wp: *mut window_pane = wme.wp;
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut oo: *mut options = (*(*wp).window).options_ptr();
        if (*data).line_numbers == 0 {
            return WINDOW_COPY_LINE_NUMBERS_OFF as ::core::ffi::c_int;
        }
        options_get_number(oo, c"copy-mode-line-numbers".as_ptr()) as ::core::ffi::c_int
    }
}
unsafe fn window_copy_line_number_is_absolute(wme: &mut window_mode_entry) -> ::core::ffi::c_int {
    unsafe {
        match window_copy_line_number_mode(wme) {
            2..=4 => return 1 as ::core::ffi::c_int,
            0 | 1 => return 0 as ::core::ffi::c_int,
            _ => {}
        }
        fatalx(c"bad line number mode".as_ptr(), fmt_args![]);
    }
}
unsafe fn window_copy_line_numbers_active(wme: &mut window_mode_entry) -> ::core::ffi::c_int {
    unsafe {
        (window_copy_line_number_mode(wme) != WINDOW_COPY_LINE_NUMBERS_OFF as ::core::ffi::c_int)
            as ::core::ffi::c_int
    }
}
unsafe fn window_copy_line_number_width(wme: &mut window_mode_entry) -> u_int {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut lines: u_int = 0;
        let mut digits: u_int = 0;
        if window_copy_line_numbers_active(wme) == 0 {
            return 0 as u_int;
        }
        lines = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_add((*screen_grid_ptr(&mut *window_copy_backing(&mut *data))).sy)
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
unsafe fn window_copy_cursor_offset(
    wme: &mut window_mode_entry,
    mut cx: u_int,
    mut sx: u_int,
) -> u_int {
    unsafe {
        let mut width: u_int = window_copy_line_number_width(wme);
        let mut content: u_int = 0;
        if width == 0 as u_int {
            return cx;
        }
        if width >= sx {
            content = 1 as u_int;
        } else {
            content = sx.wrapping_sub(width);
        }
        if cx >= content {
            return sx.wrapping_sub(1 as u_int);
        }
        width.wrapping_add(cx)
    }
}
unsafe fn window_copy_cursor_unoffset(
    wme: &mut window_mode_entry,
    mut vx: u_int,
    mut sx: u_int,
) -> u_int {
    unsafe {
        let mut width: u_int = window_copy_line_number_width(wme);
        let mut content: u_int = 0;
        if width == 0 as u_int {
            return vx;
        }
        if width >= sx {
            content = 1 as u_int;
        } else {
            content = sx.wrapping_sub(width);
        }
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
    mut wp: *mut window_pane,
    mut enabled: ::core::ffi::c_int,
) {
    unsafe {
        let mut wme: *mut window_mode_entry = window_pane_current_mode(wp);
        let mut data: *mut window_copy_mode_data = ::core::ptr::null_mut::<window_copy_mode_data>();
        if wme.is_null() {
            return;
        }
        if (*wme).mode() != WindowMode::Copy {
            return;
        }
        data = (*wme).state.copy();
        if data.is_null() {
            return;
        }
        if (*data).line_numbers == enabled {
            return;
        }
        (*data).line_numbers = enabled;
        window_copy_redraw_screen(&mut *wme);
    }
}
/// How far back the pane is scrolled and how much history it has, or
/// nothing when the pane is not in a copy or view mode.
pub unsafe fn window_copy_get_current_offset(mut wp: *mut window_pane) -> Option<(u_int, u_int)> {
    unsafe {
        let mut wme: *mut window_mode_entry = window_pane_current_mode(wp);
        if !matches!(
            (*wme).state,
            WindowModeState::Copy(_) | WindowModeState::View(_)
        ) {
            return None;
        }
        let data = (*wme).state.copy();
        let hsize = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data))).hsize;
        Some((hsize.wrapping_sub((*data).oy), hsize))
    }
}
unsafe fn window_copy_write_line(
    wme: &mut window_mode_entry,
    ctx: &mut screen_write_ctx,
    mut py: u_int,
) {
    unsafe {
        let mut wp: *mut window_pane = wme.wp;
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut s: *mut screen = &raw mut (*data).screen;
        let mut oo: *mut options = (*(*wp).window).options_ptr();
        let mut gc = grid_default_cell;
        let mut mgc = grid_default_cell;
        let mut cgc = grid_default_cell;
        let mut mkgc = grid_default_cell;
        let mut ln_gc = grid_default_cell;
        let mut cur_ln_gc = grid_default_cell;
        let mut sx: u_int = (*screen_grid_ptr(&mut *s)).sx;
        let mut hsize: u_int = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data))).hsize;
        let mut width: u_int = 0;
        let mut absolute: u_int = 0;
        let mut line_number: u_int = 0;
        let mut content_sx: u_int = 0;
        let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut current: ::core::ffi::c_int = 0;
        let mut mode: ::core::ffi::c_int = 0;
        width = window_copy_line_number_width(wme);
        if width >= sx {
            content_sx = 1 as u_int;
        } else if width != 0 as u_int {
            content_sx = sx.wrapping_sub(width);
        } else {
            content_sx = sx;
        }
        let mut ft = format_create_defaults(
            ::core::ptr::null_mut::<cmdq_item>(),
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null_mut::<session>(),
            ::core::ptr::null_mut::<winlink>(),
            wp,
        );
        style_apply(
            &mut gc,
            oo,
            c"copy-mode-position-style".as_ptr(),
            Some(&mut ft),
        );
        gc.flags = (gc.flags as ::core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
        style_apply(
            &mut mgc,
            oo,
            c"copy-mode-match-style".as_ptr(),
            Some(&mut ft),
        );
        mgc.flags = (mgc.flags as ::core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
        style_apply(
            &mut cgc,
            oo,
            c"copy-mode-current-match-style".as_ptr(),
            Some(&mut ft),
        );
        cgc.flags = (cgc.flags as ::core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
        style_apply(
            &mut mkgc,
            oo,
            c"copy-mode-mark-style".as_ptr(),
            Some(&mut ft),
        );
        mkgc.flags = (mkgc.flags as ::core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
        if width != 0 as u_int {
            style_apply(
                &mut ln_gc,
                oo,
                c"copy-mode-line-number-style".as_ptr(),
                Some(&mut ft),
            );
            ln_gc.flags = (ln_gc.flags as ::core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
            style_apply(
                &mut cur_ln_gc,
                oo,
                c"copy-mode-current-line-number-style".as_ptr(),
                Some(&mut ft),
            );
            cur_ln_gc.flags =
                (cur_ln_gc.flags as ::core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
            current = (py == (*data).cy) as ::core::ffi::c_int;
            absolute = hsize
                .wrapping_sub((*data).oy)
                .wrapping_add(py)
                .wrapping_add(1 as u_int);
            mode = window_copy_line_number_mode(wme);
            if mode == WINDOW_COPY_LINE_NUMBERS_DEFAULT as ::core::ffi::c_int {
                if py < (*data).oy {
                    line_number = (*data).oy.wrapping_sub(py);
                } else {
                    line_number = py.wrapping_sub((*data).oy);
                }
            } else if mode == WINDOW_COPY_LINE_NUMBERS_ABSOLUTE as ::core::ffi::c_int {
                line_number = absolute;
            } else if mode == WINDOW_COPY_LINE_NUMBERS_HYBRID as ::core::ffi::c_int && current != 0
            {
                line_number = absolute;
            } else if py > (*data).cy {
                line_number = py.wrapping_sub((*data).cy);
            } else {
                line_number = (*data).cy.wrapping_sub(py);
            }
            screen_write_cursormove(
                ctx,
                0 as ::core::ffi::c_int,
                py as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            screen_write_nputs(
                ctx,
                width as ssize_t,
                if current != 0 { &cur_ln_gc } else { &ln_gc },
                c"%*u ".as_ptr(),
                fmt_args![
                    width as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                    line_number
                ],
            );
        }
        window_copy_write_one(
            wme,
            ctx,
            width,
            py,
            hsize.wrapping_sub((*data).oy).wrapping_add(py),
            content_sx,
            &mut mgc,
            &mut cgc,
            &mut mkgc,
        );
        if py == 0 as u_int && (*s).rupper < (*s).rlower && (*data).hide_position == 0 {
            value = options_get_string(oo, c"copy-mode-position-format".as_ptr());
            if *value as ::core::ffi::c_int != '\0' as i32 {
                let expanded = format_expand(&mut ft, CStr::from_ptr(value));
                if !expanded.as_bytes().is_empty() {
                    screen_write_cursormove(
                        ctx,
                        width as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                    format_draw(
                        ctx,
                        &gc,
                        content_sx,
                        expanded.as_bytes(),
                        None,
                        0 as ::core::ffi::c_int,
                    );
                }
            }
        }
        if py == (*data).cy && (*data).cx >= content_sx {
            screen_write_cursormove(
                ctx,
                window_copy_cursor_offset(wme, (*data).cx, (*screen_grid_ptr(&mut *s)).sx)
                    as ::core::ffi::c_int,
                py as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            screen_write_putc(ctx, &grid_default_cell, '$' as i32 as u_char);
        }
    }
}
unsafe fn window_copy_write_lines(
    wme: &mut window_mode_entry,
    ctx: &mut screen_write_ctx,
    mut py: u_int,
    mut ny: u_int,
) {
    unsafe {
        let mut yy: u_int = 0;
        yy = py;
        while yy < py.wrapping_add(ny) {
            window_copy_write_line(wme, ctx, yy);
            yy = yy.wrapping_add(1);
        }
    }
}
unsafe fn window_copy_redraw_selection(wme: &mut window_mode_entry, mut old_y: u_int) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut gd: *mut grid = screen_grid_ptr(&mut *window_copy_backing(&mut *data));
        let mut new_y: u_int = 0;
        let mut start: u_int = 0;
        let mut end: u_int = 0;
        new_y = (*data).cy;
        if old_y <= new_y {
            start = old_y;
            end = new_y;
        } else {
            start = new_y;
            end = old_y;
        }
        if (*data).selflag as ::core::ffi::c_uint
            == SEL_WORD as ::core::ffi::c_int as ::core::ffi::c_uint
            && end < (*gd).sy.wrapping_add((*data).oy).wrapping_sub(1 as u_int)
        {
            end = end.wrapping_add(1);
        }
        window_copy_redraw_lines(wme, start, end.wrapping_sub(start).wrapping_add(1 as u_int));
    }
}
unsafe fn window_copy_redraw_lines(wme: &mut window_mode_entry, mut py: u_int, mut ny: u_int) {
    unsafe {
        let mut wp: *mut window_pane = wme.wp;
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut s: *mut screen = &raw mut (*data).screen;
        let mut ctx = screen_write_ctx::default();
        let mut i: u_int = 0;
        if window_copy_line_number_width(wme) != 0 as u_int {
            screen_write_start(&mut ctx, &mut (*data).screen);
            i = py;
            while i < py.wrapping_add(ny) {
                window_copy_write_line(wme, &mut ctx, i);
                i = i.wrapping_add(1);
            }
            screen_write_cursormove(
                &mut ctx,
                window_copy_cursor_offset(wme, (*data).cx, (*screen_grid_ptr(&mut *s)).sx)
                    as ::core::ffi::c_int,
                (*data).cy as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            screen_write_stop(&mut ctx);
            (*wp).flags |= PANE_REDRAW | PANE_REDRAWSCROLLBAR;
            return;
        }
        screen_write_start_pane(&mut ctx, wp, None);
        i = py;
        while i < py.wrapping_add(ny) {
            window_copy_write_line(wme, &mut ctx, i);
            i = i.wrapping_add(1);
        }
        screen_write_cursormove(
            &mut ctx,
            window_copy_cursor_offset(wme, (*data).cx, (*screen_grid_ptr(&mut *s)).sx)
                as ::core::ffi::c_int,
            (*data).cy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_stop(&mut ctx);
        (*wp).flags |= PANE_REDRAWSCROLLBAR;
    }
}
unsafe fn window_copy_redraw_screen(wme: &mut window_mode_entry) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        window_copy_redraw_lines(
            wme,
            0 as u_int,
            (*screen_grid_ptr(&mut (*data).screen)).sy,
        );
    }
}
pub(crate) unsafe fn window_copy_style_changed(wme: &mut window_mode_entry) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        if (*data).screen.sel.is_some() {
            window_copy_set_selection(wme, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
        }
        window_copy_redraw_screen(wme);
    }
}
unsafe fn window_copy_synchronize_cursor_end(
    wme: &mut window_mode_entry,
    mut begin: ::core::ffi::c_int,
    mut no_reset: ::core::ffi::c_int,
) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut xx: u_int = 0;
        let mut yy: u_int = 0;
        xx = (*data).cx;
        yy = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        match (*data).selflag {
            SEL_WORD => {
                if !(no_reset != 0) {
                    begin = 0 as ::core::ffi::c_int;
                    if (*data).dy > yy || (*data).dy == yy && (*data).dx > xx {
                        (xx, yy) = window_copy_cursor_previous_word_pos(
                            wme,
                            (*data).separators.as_deref().unwrap_or(c""),
                        );
                        begin = 1 as ::core::ffi::c_int;
                        (*data).endselx = (*data).endselrx;
                        (*data).endsely = (*data).endselry;
                    } else {
                        if xx >= window_copy_find_length(wme, yy)
                            || window_copy_in_set(wme, xx.wrapping_add(1 as u_int), yy, WHITESPACE)
                                == 0
                        {
                            (xx, yy) = window_copy_cursor_next_word_end_pos(
                                wme,
                                (*data).separators.as_deref().unwrap_or(c""),
                            );
                        }
                        (*data).selx = (*data).selrx;
                        (*data).sely = (*data).selry;
                    }
                }
            }
            SEL_LINE if !(no_reset != 0) => {
                begin = 0 as ::core::ffi::c_int;
                if (*data).dy > yy {
                    xx = 0 as u_int;
                    begin = 1 as ::core::ffi::c_int;
                    (*data).endselx = (*data).endselrx;
                    (*data).endsely = (*data).endselry;
                } else {
                    if yy < (*data).endselry {
                        yy = (*data).endselry;
                    }
                    xx = window_copy_find_length(wme, yy);
                    (*data).selx = (*data).selrx;
                    (*data).sely = (*data).selry;
                }
            }
            _ => {}
        }
        if begin != 0 {
            (*data).selx = xx;
            (*data).sely = yy;
        } else {
            (*data).endselx = xx;
            (*data).endsely = yy;
        };
    }
}
unsafe fn window_copy_synchronize_cursor(
    wme: &mut window_mode_entry,
    mut no_reset: ::core::ffi::c_int,
) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        match (*data).cursordrag {
            CURSORDRAG_ENDSEL => {
                window_copy_synchronize_cursor_end(wme, 0 as ::core::ffi::c_int, no_reset);
            }
            CURSORDRAG_SEL => {
                window_copy_synchronize_cursor_end(wme, 1 as ::core::ffi::c_int, no_reset);
            }
            _ => {}
        };
    }
}
unsafe fn window_copy_update_cursor(wme: &mut window_mode_entry, mut cx: u_int, mut cy: u_int) {
    unsafe {
        let mut wp: *mut window_pane = wme.wp;
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut s: *mut screen = &raw mut (*data).screen;
        let mut ctx = screen_write_ctx::default();
        let mut old_cx: u_int = 0;
        let mut old_cy: u_int = 0;
        let mut py: u_int = 0;
        let mut width: u_int = 0;
        let mut content_sx: u_int = 0;
        let mut maxx: u_int = 0;
        let mut allow_onemore: ::core::ffi::c_int = 0;
        if (*data).rectflag == 0 && cy < (*screen_grid_ptr(&mut *s)).sy {
            allow_onemore =
                ((*data).screen.sel.is_some() && (*data).rectflag != 0) as ::core::ffi::c_int;
            py = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
                .hsize
                .wrapping_add(cy)
                .wrapping_sub((*data).oy);
            maxx = window_copy_cursor_limit(wme, py, allow_onemore);
            if cx > maxx {
                cx = maxx;
            }
        }
        old_cx = (*data).cx;
        old_cy = (*data).cy;
        (*data).cx = cx;
        (*data).cy = cy;
        if window_copy_line_numbers_active(wme) != 0 {
            width = window_copy_line_number_width(wme);
            if (*s).sel.is_some()
                || (*data).lineflag as ::core::ffi::c_uint
                    != LINE_SEL_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                || old_cy != (*data).cy
            {
                window_copy_redraw_screen(wme);
                return;
            }
            if width >= (*screen_grid_ptr(&mut *s)).sx {
                content_sx = 1 as u_int;
            } else {
                content_sx = (*screen_grid_ptr(&mut *s)).sx.wrapping_sub(width);
            }
            if old_cx >= content_sx || (*data).cx >= content_sx {
                window_copy_redraw_screen(wme);
                return;
            }
            screen_write_start_pane(&mut ctx, wp, None);
            screen_write_cursormove(
                &mut ctx,
                window_copy_cursor_offset(wme, (*data).cx, (*screen_grid_ptr(&mut *s)).sx)
                    as ::core::ffi::c_int,
                (*data).cy as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            screen_write_stop(&mut ctx);
            return;
        }
        if old_cx == (*screen_grid_ptr(&mut *s)).sx {
            window_copy_redraw_lines(wme, old_cy, 1 as u_int);
        }
        if (*data).cx == (*screen_grid_ptr(&mut *s)).sx {
            window_copy_redraw_lines(wme, (*data).cy, 1 as u_int);
        } else {
            screen_write_start_pane(&mut ctx, wp, None);
            screen_write_cursormove(
                &mut ctx,
                window_copy_cursor_offset(wme, (*data).cx, (*screen_grid_ptr(&mut *s)).sx)
                    as ::core::ffi::c_int,
                (*data).cy as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            screen_write_stop(&mut ctx);
        };
    }
}
unsafe fn window_copy_start_selection(wme: &mut window_mode_entry) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        (*data).selx = (*data).cx;
        (*data).sely = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        (*data).endselx = (*data).selx;
        (*data).endsely = (*data).sely;
        (*data).cursordrag = CURSORDRAG_ENDSEL;
        window_copy_set_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    }
}
unsafe fn window_copy_adjust_selection(
    wme: &mut window_mode_entry,
    selx: &mut u_int,
    sely: &mut u_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut s: *mut screen = &raw mut (*data).screen;
        let mut sx: u_int = 0;
        let mut sy: u_int = 0;
        let mut ty: u_int = 0;
        let mut relpos: ::core::ffi::c_int = 0;
        sx = *selx;
        sy = *sely;
        ty = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_sub((*data).oy);
        if sy < ty {
            relpos = WINDOW_COPY_REL_POS_ABOVE as ::core::ffi::c_int;
            if (*data).rectflag == 0 {
                sx = 0 as u_int;
            }
            sy = 0 as u_int;
        } else if sy
            > ty.wrapping_add((*screen_grid_ptr(&mut *s)).sy)
                .wrapping_sub(1 as u_int)
        {
            relpos = WINDOW_COPY_REL_POS_BELOW as ::core::ffi::c_int;
            if (*data).rectflag == 0 {
                sx = (*screen_grid_ptr(&mut *s)).sx.wrapping_sub(1 as u_int);
            }
            sy = (*screen_grid_ptr(&mut *s)).sy.wrapping_sub(1 as u_int);
        } else {
            relpos = WINDOW_COPY_REL_POS_ON_SCREEN as ::core::ffi::c_int;
            sy = sy.wrapping_sub(ty);
        }
        *selx = sx;
        *sely = sy;
        relpos
    }
}
unsafe fn window_copy_update_selection(
    wme: &mut window_mode_entry,
    mut may_redraw: ::core::ffi::c_int,
    mut no_reset: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut s: *mut screen = &raw mut (*data).screen;
        if (*s).sel.is_none()
            && (*data).lineflag as ::core::ffi::c_uint
                == LINE_SEL_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return 0 as ::core::ffi::c_int;
        }
        window_copy_set_selection(wme, may_redraw, no_reset)
    }
}
unsafe fn window_copy_set_selection(
    wme: &mut window_mode_entry,
    mut may_redraw: ::core::ffi::c_int,
    mut no_reset: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut wp: *mut window_pane = wme.wp;
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut s: *mut screen = &raw mut (*data).screen;
        let mut oo: *mut options = (*(*wp).window).options_ptr();
        let mut gc = grid_default_cell;
        let mut sx: u_int = 0;
        let mut sy: u_int = 0;
        let mut cy: u_int = 0;
        let mut endsx: u_int = 0;
        let mut endsy: u_int = 0;
        let mut clipx: u_int = 0;
        let mut startrelpos: ::core::ffi::c_int = 0;
        let mut endrelpos: ::core::ffi::c_int = 0;
        window_copy_synchronize_cursor(wme, no_reset);
        sx = (*data).selx;
        sy = (*data).sely;
        startrelpos = window_copy_adjust_selection(wme, &mut sx, &mut sy);
        endsx = (*data).endselx;
        endsy = (*data).endsely;
        endrelpos = window_copy_adjust_selection(wme, &mut endsx, &mut endsy);
        if startrelpos == endrelpos
            && startrelpos != WINDOW_COPY_REL_POS_ON_SCREEN as ::core::ffi::c_int
        {
            screen_hide_selection(s);
            return 0 as ::core::ffi::c_int;
        }
        let mut ft = format_create_defaults(
            ::core::ptr::null_mut::<cmdq_item>(),
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null_mut::<session>(),
            ::core::ptr::null_mut::<winlink>(),
            wp,
        );
        style_apply(
            &mut gc,
            oo,
            c"copy-mode-selection-style".as_ptr(),
            Some(&mut ft),
        );
        gc.flags = (gc.flags as ::core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
        clipx = window_copy_line_number_width(wme);
        if clipx >= (*screen_grid_ptr(&mut *s)).sx {
            clipx = (*screen_grid_ptr(&mut *s)).sx.wrapping_sub(1 as u_int);
        }
        if window_copy_line_numbers_active(wme) != 0 {
            sx = window_copy_cursor_offset(wme, sx, (*screen_grid_ptr(&mut *s)).sx);
            endsx = window_copy_cursor_offset(wme, endsx, (*screen_grid_ptr(&mut *s)).sx);
        }
        screen_set_selection(
            s,
            sx,
            sy,
            endsx,
            endsy,
            (*data).rectflag as u_int,
            clipx,
            (*data).modekeys,
            &mut gc,
        );
        if (*data).rectflag != 0 && may_redraw != 0 {
            cy = (*data).cy;
            if (*data).cursordrag as ::core::ffi::c_uint
                == CURSORDRAG_ENDSEL as ::core::ffi::c_int as ::core::ffi::c_uint
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
        1 as ::core::ffi::c_int
    }
}
/// The text the selection covers, or the match under the cursor when there is
/// no selection at all, or nothing when neither holds any text.
unsafe fn window_copy_get_selection(wme: &mut window_mode_entry) -> Option<Vec<u8>> {
    unsafe {
        let mut wp: *mut window_pane = wme.wp;
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut s: *mut screen = &raw mut (*data).screen;
        let mut i: u_int = 0;
        let mut xx: u_int = 0;
        let mut yy: u_int = 0;
        let mut sx: u_int = 0;
        let mut sy: u_int = 0;
        let mut ex: u_int = 0;
        let mut ey: u_int = 0;
        let mut ey_last: u_int = 0;
        let mut firstsx: u_int = 0;
        let mut lastex: u_int = 0;
        let mut restex: u_int = 0;
        let mut restsx: u_int = 0;
        let mut selx: u_int = 0;
        let mut keys: ::core::ffi::c_int = 0;
        if (*data).screen.sel.is_none()
            && (*data).lineflag as ::core::ffi::c_uint
                == LINE_SEL_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return window_copy_match_at_cursor(data).map(CString::into_bytes);
        }
        let mut buf: Vec<u8> = Vec::new();
        xx = (*data).endselx;
        yy = (*data).endsely;
        if yy < (*data).sely || yy == (*data).sely && xx < (*data).selx {
            sx = xx;
            sy = yy;
            ex = (*data).selx;
            ey = (*data).sely;
        } else {
            sx = (*data).selx;
            sy = (*data).sely;
            ex = xx;
            ey = yy;
        }
        ey_last = window_copy_find_length(wme, ey);
        if ex > ey_last {
            ex = ey_last;
        }
        xx = (*screen_grid_ptr(&mut *s)).sx;
        keys = options_get_number((*(*wp).window).options_ptr(), c"mode-keys".as_ptr())
            as ::core::ffi::c_int;
        if (*data).rectflag != 0 {
            if (*data).cursordrag as ::core::ffi::c_uint
                == CURSORDRAG_ENDSEL as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                selx = (*data).selx;
            } else {
                selx = (*data).endselx;
            }
            if selx < (*data).cx {
                if keys == MODEKEY_EMACS {
                    lastex = (*data).cx;
                    restex = (*data).cx;
                } else {
                    lastex = (*data).cx.wrapping_add(1 as u_int);
                    restex = (*data).cx.wrapping_add(1 as u_int);
                }
                firstsx = selx;
                restsx = selx;
            } else {
                lastex = selx.wrapping_add(1 as u_int);
                restex = selx.wrapping_add(1 as u_int);
                firstsx = (*data).cx;
                restsx = (*data).cx;
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
            && (!grid_get_line(screen_grid_mut(&mut *window_copy_backing(&mut *data)), ey).flags
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
    mut set_paste: ::core::ffi::c_int,
    mut set_clip: ::core::ffi::c_int,
) {
    unsafe {
        let mut wp: *mut window_pane = wme.wp;
        let mut ctx = screen_write_ctx::default();
        let mut redraw: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if set_clip != 0
            && options_get_number(global_options, c"set-clipboard".as_ptr())
                != 0 as ::core::ffi::c_longlong
        {
            if window_copy_line_numbers_active(wme) != 0 && (*wp).flags & PANE_REDRAW != 0 {
                redraw = PANE_REDRAW;
                (*wp).flags &= !PANE_REDRAW;
            }
            screen_write_start_pane(&mut ctx, wp, None);
            screen_write_setselection(
                &mut ctx,
                c"".as_ptr(),
                buf.as_ptr() as *mut u_char,
                buf.len() as u_int,
            );
            screen_write_stop(&mut ctx);
            (*wp).flags |= redraw;
            notify_pane(c"pane-set-clipboard".as_ptr(), wp);
        }
        if set_paste != 0 {
            paste_add(prefix.map_or(::core::ptr::null(), CStr::as_ptr), buf);
        }
    }
}
unsafe fn window_copy_pipe_run(
    wme: &mut window_mode_entry,
    mut s: *mut session,
    cmd: Option<&CStr>,
) -> Option<Vec<u8>> {
    unsafe {
        let mut job: *mut job = ::core::ptr::null_mut::<job>();
        let buf = window_copy_get_selection(wme);
        let cmd = match cmd.filter(|cmd| !cmd.is_empty()) {
            Some(cmd) => cmd.as_ptr(),
            None => options_get_string(global_options, c"copy-command".as_ptr()),
        };
        if !cmd.is_null() && *cmd as ::core::ffi::c_int != '\0' as i32 {
            job = job_run(
                cmd,
                &[],
                ::core::ptr::null_mut::<environ_t>(),
                s,
                ::core::ptr::null::<::core::ffi::c_char>(),
                None,
                None,
                None,
                JobData::None,
                JOB_NOWAIT,
                -(1 as ::core::ffi::c_int),
                -(1 as ::core::ffi::c_int),
            );
            if !job.is_null() {
                let written = buf.as_deref().unwrap_or(&[]);
                job_get_event(job).write(written.as_ptr(), written.len() as size_t);
            }
        }
        buf
    }
}
unsafe fn window_copy_pipe(wme: &mut window_mode_entry, mut s: *mut session, cmd: Option<&CStr>) {
    unsafe {
        window_copy_pipe_run(wme, s, cmd);
    }
}
unsafe fn window_copy_copy_pipe(
    wme: &mut window_mode_entry,
    mut s: *mut session,
    prefix: Option<&CStr>,
    cmd: Option<&CStr>,
    mut set_paste: ::core::ffi::c_int,
    mut set_clip: ::core::ffi::c_int,
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
    mut set_paste: ::core::ffi::c_int,
    mut set_clip: ::core::ffi::c_int,
) {
    unsafe {
        if let Some(buf) = window_copy_get_selection(wme) {
            window_copy_copy_buffer(wme, prefix, buf, set_paste, set_clip);
        }
    }
}
unsafe fn window_copy_append_selection(wme: &mut window_mode_entry) {
    unsafe {
        let mut wp: *mut window_pane = wme.wp;
        let mut bufname: Option<::std::ffi::CString> = None;
        let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
        let mut ctx = screen_write_ctx::default();
        let Some(buf) = window_copy_get_selection(wme) else {
            return;
        };
        if options_get_number(global_options, c"set-clipboard".as_ptr())
            != 0 as ::core::ffi::c_longlong
        {
            screen_write_start_pane(&mut ctx, wp, None);
            screen_write_setselection(
                &mut ctx,
                c"".as_ptr(),
                buf.as_ptr() as *mut u_char,
                buf.len() as u_int,
            );
            screen_write_stop(&mut ctx);
            notify_pane(c"pane-set-clipboard".as_ptr(), wp);
        }
        pb = paste_get_top(Some(&mut bufname));
        let mut data: Vec<u8> = Vec::new();
        if !pb.is_null() {
            data.extend_from_slice(paste_buffer_data(&*pb));
        }
        data.extend_from_slice(&buf);
        let _ = paste_set(
            data,
            bufname
                .as_ref()
                .map_or(::core::ptr::null(), |value| value.as_ptr()),
        );
    }
}
unsafe fn window_copy_copy_line(
    wme: &mut window_mode_entry,
    buf: &mut Vec<u8>,
    mut sy: u_int,
    mut sx: u_int,
    mut ex: u_int,
) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut gd: *mut grid = screen_grid_ptr(&mut *window_copy_backing(&mut *data));
        let mut gc = grid_default_cell;
        let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
        let mut ud = utf8_data::default();
        let mut i: u_int = 0;
        let mut xx: u_int = 0;
        let mut wrapped: u_int = 0 as u_int;
        if sx > ex {
            return;
        }
        gl = grid_get_line(&mut *gd, sy);
        if (*gl).flags & GRID_LINE_WRAPPED != 0 && (*gl).cellsize() <= (*gd).sx {
            wrapped = 1 as u_int;
        }
        if wrapped != 0 {
            xx = (*gl).cellsize();
        } else {
            xx = window_copy_find_length(wme, sy);
        }
        if ex > xx {
            ex = xx;
        }
        if sx > xx {
            sx = xx;
        }
        if sx < ex {
            i = sx;
            while i < ex {
                gc = grid_get_cell(&*gd, i, sy);
                if !(gc.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0) {
                    if gc.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
                        utf8_set(&mut ud, '\t' as i32 as u_char);
                    } else {
                        utf8_copy(&mut ud, &gc.data);
                    }
                    if ud.size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
                        && gc.attr as ::core::ffi::c_int & GRID_ATTR_CHARSET != 0
                        && let Some(acs) =
                            tty_acs_get(None, ud.data[0 as ::core::ffi::c_int as usize])
                                .map(|s| s.to_bytes())
                        && acs.len() <= ud.data.len()
                    {
                        ud.size = acs.len() as u_char;
                        ud.data[..acs.len()].copy_from_slice(acs);
                    }
                    buf.extend_from_slice(::core::slice::from_raw_parts(
                        &raw const ud.data as *const u8,
                        ud.size as usize,
                    ));
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
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        screen_clear_selection(&mut (*data).screen);
        (*data).cursordrag = CURSORDRAG_NONE;
        (*data).lineflag = LINE_SEL_NONE;
        (*data).selflag = SEL_CHAR;
        py = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        px = window_copy_cursor_limit(wme, py, (*data).rectflag);
        if (*data).cx > px {
            window_copy_update_cursor(wme, px, (*data).cy);
        }
    }
}
unsafe fn window_copy_in_set(
    wme: &mut window_mode_entry,
    mut px: u_int,
    mut py: u_int,
    set: &CStr,
) -> ::core::ffi::c_int {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        grid_in_set(screen_grid(&*window_copy_backing(&mut *data)), px, py, set)
    }
}
unsafe fn window_copy_find_length(wme: &mut window_mode_entry, mut py: u_int) -> u_int {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        grid_line_length(screen_grid(&*window_copy_backing(&mut *data)), py)
    }
}
unsafe fn window_copy_cursor_limit(
    wme: &mut window_mode_entry,
    mut py: u_int,
    mut allow_onemore: ::core::ffi::c_int,
) -> u_int {
    unsafe {
        let mut oo: *mut options = (*(*wme.wp).window).options_ptr();
        let mut len: u_int = 0;
        len = window_copy_find_length(wme, py);
        if allow_onemore != 0
            || options_get_number(oo, c"mode-keys".as_ptr())
                != MODEKEY_VI as ::core::ffi::c_longlong
        {
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
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut back_s: *mut screen = window_copy_backing(&mut *data);
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut oldy: u_int = 0;
        let mut hsize: u_int = 0;
        px = (*data).cx;
        hsize = (*screen_grid_ptr(&mut *back_s)).hsize;
        py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
        oldy = (*data).cy;
        let mut gr = grid_reader_start(screen_grid(&*back_s), px, py);
        grid_reader_cursor_start_of_line(&mut gr, 1 as ::core::ffi::c_int);
        (px, py) = grid_reader_get_cursor(&gr);
        window_copy_acquire_cursor_up(wme, hsize, (*data).oy, oldy, px, py);
    }
}
unsafe fn window_copy_cursor_back_to_indentation(wme: &mut window_mode_entry) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut back_s: *mut screen = window_copy_backing(&mut *data);
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut oldy: u_int = 0;
        let mut hsize: u_int = 0;
        px = (*data).cx;
        hsize = (*screen_grid_ptr(&mut *back_s)).hsize;
        py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
        oldy = (*data).cy;
        let mut gr = grid_reader_start(screen_grid(&*back_s), px, py);
        grid_reader_cursor_back_to_indentation(&mut gr);
        (px, py) = grid_reader_get_cursor(&gr);
        window_copy_acquire_cursor_up(wme, hsize, (*data).oy, oldy, px, py);
    }
}
unsafe fn window_copy_cursor_end_of_line(wme: &mut window_mode_entry) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut back_s: *mut screen = window_copy_backing(&mut *data);
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut oldy: u_int = 0;
        let mut hsize: u_int = 0;
        px = (*data).cx;
        hsize = (*screen_grid_ptr(&mut *back_s)).hsize;
        py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
        oldy = (*data).cy;
        let mut gr = grid_reader_start(screen_grid(&*back_s), px, py);
        if (*data).screen.sel.is_some() && (*data).rectflag != 0 {
            grid_reader_cursor_end_of_line(
                &mut gr,
                1 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
        } else {
            grid_reader_cursor_end_of_line(
                &mut gr,
                1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
        }
        (px, py) = grid_reader_get_cursor(&gr);
        if (*data).screen.sel.is_none() || (*data).rectflag == 0 {
            px = window_copy_cursor_limit(wme, py, 0 as ::core::ffi::c_int);
        }
        window_copy_acquire_cursor_down(
            wme,
            hsize,
            (*screen_grid_ptr(&mut *back_s)).sy,
            (*data).oy,
            oldy,
            px,
            py,
            0 as ::core::ffi::c_int,
        );
    }
}
unsafe fn window_copy_other_end(wme: &mut window_mode_entry) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut s: *mut screen = &raw mut (*data).screen;
        let mut selx: u_int = 0;
        let mut sely: u_int = 0;
        let mut cy: u_int = 0;
        let mut yy: u_int = 0;
        let mut hsize: u_int = 0;
        if (*s).sel.is_none()
            && (*data).lineflag as ::core::ffi::c_uint
                == LINE_SEL_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return;
        }
        if (*data).lineflag as ::core::ffi::c_uint
            == LINE_SEL_LEFT_RIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*data).lineflag = LINE_SEL_RIGHT_LEFT;
        } else if (*data).lineflag as ::core::ffi::c_uint
            == LINE_SEL_RIGHT_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*data).lineflag = LINE_SEL_LEFT_RIGHT;
        }
        match (*data).cursordrag {
            CURSORDRAG_NONE | CURSORDRAG_SEL => {
                (*data).cursordrag = CURSORDRAG_ENDSEL;
            }
            CURSORDRAG_ENDSEL => {
                (*data).cursordrag = CURSORDRAG_SEL;
            }
            _ => {}
        }
        selx = (*data).endselx;
        sely = (*data).endsely;
        if (*data).cursordrag as ::core::ffi::c_uint
            == CURSORDRAG_SEL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            selx = (*data).selx;
            sely = (*data).sely;
        }
        cy = (*data).cy;
        yy = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        (*data).cx = selx;
        hsize = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data))).hsize;
        if sely < hsize.wrapping_sub((*data).oy) {
            (*data).oy = hsize.wrapping_sub(sely);
            (*data).cy = 0 as u_int;
        } else if sely
            > hsize
                .wrapping_sub((*data).oy)
                .wrapping_add((*screen_grid_ptr(&mut *s)).sy)
        {
            (*data).oy = hsize
                .wrapping_sub(sely)
                .wrapping_add((*screen_grid_ptr(&mut *s)).sy)
                .wrapping_sub(1 as u_int);
            (*data).cy = (*screen_grid_ptr(&mut *s)).sy.wrapping_sub(1 as u_int);
        } else {
            (*data).cy = cy.wrapping_add(sely).wrapping_sub(yy);
        }
        yy = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        hsize = window_copy_cursor_limit(wme, yy, (*data).rectflag);
        if (*data).cx > hsize {
            (*data).cx = hsize;
        }
        window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
        window_copy_redraw_screen(wme);
    }
}
unsafe fn window_copy_cursor_left(wme: &mut window_mode_entry) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut back_s: *mut screen = window_copy_backing(&mut *data);
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut oldy: u_int = 0;
        let mut hsize: u_int = 0;
        px = (*data).cx;
        hsize = (*screen_grid_ptr(&mut *back_s)).hsize;
        py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
        oldy = (*data).cy;
        let mut gr = grid_reader_start(screen_grid(&*back_s), px, py);
        grid_reader_cursor_left(&mut gr, 1 as ::core::ffi::c_int);
        (px, py) = grid_reader_get_cursor(&gr);
        window_copy_acquire_cursor_up(wme, hsize, (*data).oy, oldy, px, py);
    }
}
unsafe fn window_copy_cursor_right(wme: &mut window_mode_entry, mut all: ::core::ffi::c_int) {
    unsafe {
        let mut wp: *mut window_pane = wme.wp;
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut oo: *mut options = (*(*wp).window).options_ptr();
        let mut back_s: *mut screen = window_copy_backing(&mut *data);
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut oldy: u_int = 0;
        let mut hsize: u_int = 0;
        let mut onemore: ::core::ffi::c_int = 0;
        px = (*data).cx;
        hsize = (*screen_grid_ptr(&mut *back_s)).hsize;
        py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
        oldy = (*data).cy;
        onemore = (options_get_number(oo, c"mode-keys".as_ptr())
            != MODEKEY_VI as ::core::ffi::c_longlong) as ::core::ffi::c_int;
        let mut gr = grid_reader_start(screen_grid(&*back_s), px, py);
        grid_reader_cursor_right(&mut gr, 1 as ::core::ffi::c_int, all, onemore);
        (px, py) = grid_reader_get_cursor(&gr);
        window_copy_acquire_cursor_down(
            wme,
            hsize,
            (*screen_grid_ptr(&mut *back_s)).sy,
            (*data).oy,
            oldy,
            px,
            py,
            0 as ::core::ffi::c_int,
        );
    }
}
unsafe fn window_copy_cursor_up(wme: &mut window_mode_entry, mut scroll_only: ::core::ffi::c_int) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut s: *mut screen = &raw mut (*data).screen;
        let mut ox: u_int = 0;
        let mut oy: u_int = 0;
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut norectsel: ::core::ffi::c_int = 0;
        norectsel = ((*data).screen.sel.is_none() || (*data).rectflag == 0) as ::core::ffi::c_int;
        oy = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        ox = window_copy_find_length(wme, oy);
        if norectsel != 0 && (*data).cx != ox {
            (*data).lastcx = (*data).cx;
            (*data).lastsx = ox;
        }
        if (*data).lineflag as ::core::ffi::c_uint
            == LINE_SEL_LEFT_RIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
            && oy == (*data).sely
        {
            window_copy_other_end(wme);
        }
        if scroll_only != 0 || (*data).cy == 0 as u_int {
            if norectsel != 0 {
                (*data).cx = (*data).lastcx;
            }
            window_copy_scroll_down(wme, 1 as u_int);
            if scroll_only != 0 {
                if (*data).cy == (*screen_grid_ptr(&mut *s)).sy.wrapping_sub(1 as u_int) {
                    window_copy_redraw_lines(wme, (*data).cy, 1 as u_int);
                } else {
                    window_copy_redraw_lines(wme, (*data).cy, 2 as u_int);
                }
            }
        } else {
            if norectsel != 0 {
                window_copy_update_cursor(wme, (*data).lastcx, (*data).cy.wrapping_sub(1 as u_int));
            } else {
                window_copy_update_cursor(wme, (*data).cx, (*data).cy.wrapping_sub(1 as u_int));
            }
            if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int)
                != 0
            {
                if (*data).cy == (*screen_grid_ptr(&mut *s)).sy.wrapping_sub(1 as u_int) {
                    window_copy_redraw_lines(wme, (*data).cy, 1 as u_int);
                } else {
                    window_copy_redraw_lines(wme, (*data).cy, 2 as u_int);
                }
            }
        }
        if norectsel != 0 {
            py = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
                .hsize
                .wrapping_add((*data).cy)
                .wrapping_sub((*data).oy);
            px = window_copy_find_length(wme, py);
            if (*data).cx >= (*data).lastsx && (*data).cx != px || (*data).cx > px {
                window_copy_update_cursor(wme, px, (*data).cy);
                if window_copy_update_selection(
                    wme,
                    1 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                ) != 0
                {
                    window_copy_redraw_lines(wme, (*data).cy, 1 as u_int);
                }
            }
        }
        if (*data).lineflag as ::core::ffi::c_uint
            == LINE_SEL_LEFT_RIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            py = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
                .hsize
                .wrapping_add((*data).cy)
                .wrapping_sub((*data).oy);
            if (*data).rectflag != 0 {
                px = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data))).sx;
            } else {
                px = window_copy_find_length(wme, py);
            }
            window_copy_update_cursor(wme, px, (*data).cy);
            if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int)
                != 0
            {
                window_copy_redraw_lines(wme, (*data).cy, 1 as u_int);
            }
        } else if (*data).lineflag as ::core::ffi::c_uint
            == LINE_SEL_RIGHT_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            window_copy_update_cursor(wme, 0 as u_int, (*data).cy);
            if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int)
                != 0
            {
                window_copy_redraw_lines(wme, (*data).cy, 1 as u_int);
            }
        }
    }
}
unsafe fn window_copy_cursor_down(
    wme: &mut window_mode_entry,
    mut scroll_only: ::core::ffi::c_int,
) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut s: *mut screen = &raw mut (*data).screen;
        let mut ox: u_int = 0;
        let mut oy: u_int = 0;
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut norectsel: ::core::ffi::c_int = 0;
        norectsel = ((*data).screen.sel.is_none() || (*data).rectflag == 0) as ::core::ffi::c_int;
        oy = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        ox = window_copy_find_length(wme, oy);
        if norectsel != 0 && (*data).cx != ox {
            (*data).lastcx = (*data).cx;
            (*data).lastsx = ox;
        }
        if (*data).lineflag as ::core::ffi::c_uint
            == LINE_SEL_RIGHT_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
            && oy == (*data).endsely
        {
            window_copy_other_end(wme);
        }
        if scroll_only != 0 || (*data).cy == (*screen_grid_ptr(&mut *s)).sy.wrapping_sub(1 as u_int) {
            if norectsel != 0 {
                (*data).cx = (*data).lastcx;
            }
            window_copy_scroll_up(wme, 1 as u_int);
            if scroll_only != 0 && (*data).cy > 0 as u_int {
                window_copy_redraw_lines(wme, (*data).cy.wrapping_sub(1 as u_int), 2 as u_int);
            }
        } else {
            if norectsel != 0 {
                window_copy_update_cursor(wme, (*data).lastcx, (*data).cy.wrapping_add(1 as u_int));
            } else {
                window_copy_update_cursor(wme, (*data).cx, (*data).cy.wrapping_add(1 as u_int));
            }
            if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int)
                != 0
            {
                window_copy_redraw_lines(wme, (*data).cy.wrapping_sub(1 as u_int), 2 as u_int);
            }
        }
        if norectsel != 0 {
            py = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
                .hsize
                .wrapping_add((*data).cy)
                .wrapping_sub((*data).oy);
            px = window_copy_find_length(wme, py);
            if (*data).cx >= (*data).lastsx && (*data).cx != px || (*data).cx > px {
                window_copy_update_cursor(wme, px, (*data).cy);
                if window_copy_update_selection(
                    wme,
                    1 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                ) != 0
                {
                    window_copy_redraw_lines(wme, (*data).cy, 1 as u_int);
                }
            }
        }
        if (*data).lineflag as ::core::ffi::c_uint
            == LINE_SEL_LEFT_RIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            py = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
                .hsize
                .wrapping_add((*data).cy)
                .wrapping_sub((*data).oy);
            if (*data).rectflag != 0 {
                px = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data))).sx;
            } else {
                px = window_copy_find_length(wme, py);
            }
            window_copy_update_cursor(wme, px, (*data).cy);
            if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int)
                != 0
            {
                window_copy_redraw_lines(wme, (*data).cy, 1 as u_int);
            }
        } else if (*data).lineflag as ::core::ffi::c_uint
            == LINE_SEL_RIGHT_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            window_copy_update_cursor(wme, 0 as u_int, (*data).cy);
            if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int)
                != 0
            {
                window_copy_redraw_lines(wme, (*data).cy, 1 as u_int);
            }
        }
    }
}
unsafe fn window_copy_cursor_jump(wme: &mut window_mode_entry) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let Some(jc) = (*data).jumpchar.as_ref() else {
            return;
        };
        let mut back_s: *mut screen = window_copy_backing(&mut *data);
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut oldy: u_int = 0;
        let mut hsize: u_int = 0;
        px = (*data).cx.wrapping_add(1 as u_int);
        hsize = (*screen_grid_ptr(&mut *back_s)).hsize;
        py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
        oldy = (*data).cy;
        let mut gr = grid_reader_start(screen_grid(&*back_s), px, py);
        if grid_reader_cursor_jump(&mut gr, jc) != 0 {
            (px, py) = grid_reader_get_cursor(&gr);
            window_copy_acquire_cursor_down(
                wme,
                hsize,
                (*screen_grid_ptr(&mut *back_s)).sy,
                (*data).oy,
                oldy,
                px,
                py,
                0 as ::core::ffi::c_int,
            );
        }
    }
}
unsafe fn window_copy_cursor_jump_back(wme: &mut window_mode_entry) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let Some(jc) = (*data).jumpchar.as_ref() else {
            return;
        };
        let mut back_s: *mut screen = window_copy_backing(&mut *data);
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut oldy: u_int = 0;
        let mut hsize: u_int = 0;
        px = (*data).cx;
        hsize = (*screen_grid_ptr(&mut *back_s)).hsize;
        py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
        oldy = (*data).cy;
        let mut gr = grid_reader_start(screen_grid(&*back_s), px, py);
        grid_reader_cursor_left(&mut gr, 0 as ::core::ffi::c_int);
        if grid_reader_cursor_jump_back(&mut gr, jc) != 0 {
            (px, py) = grid_reader_get_cursor(&gr);
            window_copy_acquire_cursor_up(wme, hsize, (*data).oy, oldy, px, py);
        }
    }
}
unsafe fn window_copy_cursor_jump_to(wme: &mut window_mode_entry) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let Some(jc) = (*data).jumpchar.as_ref() else {
            return;
        };
        let mut back_s: *mut screen = window_copy_backing(&mut *data);
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut oldy: u_int = 0;
        let mut hsize: u_int = 0;
        px = (*data).cx.wrapping_add(2 as u_int);
        hsize = (*screen_grid_ptr(&mut *back_s)).hsize;
        py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
        oldy = (*data).cy;
        let mut gr = grid_reader_start(screen_grid(&*back_s), px, py);
        if grid_reader_cursor_jump(&mut gr, jc) != 0 {
            grid_reader_cursor_left(&mut gr, 1 as ::core::ffi::c_int);
            (px, py) = grid_reader_get_cursor(&gr);
            window_copy_acquire_cursor_down(
                wme,
                hsize,
                (*screen_grid_ptr(&mut *back_s)).sy,
                (*data).oy,
                oldy,
                px,
                py,
                0 as ::core::ffi::c_int,
            );
        }
    }
}
unsafe fn window_copy_cursor_jump_to_back(wme: &mut window_mode_entry) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let Some(jc) = (*data).jumpchar.as_ref() else {
            return;
        };
        let mut oo: *mut options = (*(*wme.wp).window).options_ptr();
        let mut back_s: *mut screen = window_copy_backing(&mut *data);
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut oldy: u_int = 0;
        let mut hsize: u_int = 0;
        let mut onemore: ::core::ffi::c_int = 0;
        px = (*data).cx;
        hsize = (*screen_grid_ptr(&mut *back_s)).hsize;
        py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
        oldy = (*data).cy;
        onemore = (options_get_number(oo, c"mode-keys".as_ptr())
            != MODEKEY_VI as ::core::ffi::c_longlong) as ::core::ffi::c_int;
        let mut gr = grid_reader_start(screen_grid(&*back_s), px, py);
        grid_reader_cursor_left(&mut gr, 0 as ::core::ffi::c_int);
        grid_reader_cursor_left(&mut gr, 0 as ::core::ffi::c_int);
        if grid_reader_cursor_jump_back(&mut gr, jc) != 0 {
            grid_reader_cursor_right(
                &mut gr,
                1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                onemore,
            );
            (px, py) = grid_reader_get_cursor(&gr);
            window_copy_acquire_cursor_up(wme, hsize, (*data).oy, oldy, px, py);
        }
    }
}
unsafe fn window_copy_cursor_next_word(wme: &mut window_mode_entry, separators: &CStr) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut back_s: *mut screen = window_copy_backing(&mut *data);
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut oldy: u_int = 0;
        let mut hsize: u_int = 0;
        px = (*data).cx;
        hsize = (*screen_grid_ptr(&mut *back_s)).hsize;
        py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
        oldy = (*data).cy;
        let mut gr = grid_reader_start(screen_grid(&*back_s), px, py);
        grid_reader_cursor_next_word(&mut gr, separators);
        (px, py) = grid_reader_get_cursor(&gr);
        window_copy_acquire_cursor_down(
            wme,
            hsize,
            (*screen_grid_ptr(&mut *back_s)).sy,
            (*data).oy,
            oldy,
            px,
            py,
            0 as ::core::ffi::c_int,
        );
    }
}
unsafe fn window_copy_cursor_next_word_end_pos(
    wme: &mut window_mode_entry,
    separators: &CStr,
) -> (u_int, u_int) {
    unsafe {
        let mut wp: *mut window_pane = wme.wp;
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut oo: *mut options = (*(*wp).window).options_ptr();
        let mut back_s: *mut screen = window_copy_backing(&mut *data);
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut hsize: u_int = 0;
        px = (*data).cx;
        hsize = (*screen_grid_ptr(&mut *back_s)).hsize;
        py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
        let mut gr = grid_reader_start(screen_grid(&*back_s), px, py);
        if options_get_number(oo, c"mode-keys".as_ptr()) == MODEKEY_VI as ::core::ffi::c_longlong {
            if grid_reader_in_set(&mut gr, WHITESPACE) == 0 {
                grid_reader_cursor_right(
                    &mut gr,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
            }
            grid_reader_cursor_next_word_end(&mut gr, separators);
            grid_reader_cursor_left(&mut gr, 1 as ::core::ffi::c_int);
        } else {
            grid_reader_cursor_next_word_end(&mut gr, separators);
        }
        (px, py) = grid_reader_get_cursor(&gr);
        (px, py)
    }
}
unsafe fn window_copy_cursor_next_word_end(
    wme: &mut window_mode_entry,
    separators: &CStr,
    mut no_reset: ::core::ffi::c_int,
) {
    unsafe {
        let mut wp: *mut window_pane = wme.wp;
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut oo: *mut options = (*(*wp).window).options_ptr();
        let mut back_s: *mut screen = window_copy_backing(&mut *data);
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut oldy: u_int = 0;
        let mut hsize: u_int = 0;
        px = (*data).cx;
        hsize = (*screen_grid_ptr(&mut *back_s)).hsize;
        py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
        oldy = (*data).cy;
        let mut gr = grid_reader_start(screen_grid(&*back_s), px, py);
        if options_get_number(oo, c"mode-keys".as_ptr()) == MODEKEY_VI as ::core::ffi::c_longlong {
            if grid_reader_in_set(&mut gr, WHITESPACE) == 0 {
                grid_reader_cursor_right(
                    &mut gr,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
            }
            grid_reader_cursor_next_word_end(&mut gr, separators);
            grid_reader_cursor_left(&mut gr, 1 as ::core::ffi::c_int);
        } else {
            grid_reader_cursor_next_word_end(&mut gr, separators);
        }
        (px, py) = grid_reader_get_cursor(&gr);
        window_copy_acquire_cursor_down(
            wme,
            hsize,
            (*screen_grid_ptr(&mut *back_s)).sy,
            (*data).oy,
            oldy,
            px,
            py,
            no_reset,
        );
    }
}
unsafe fn window_copy_cursor_previous_word_pos(
    wme: &mut window_mode_entry,
    separators: &CStr,
) -> (u_int, u_int) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut back_s: *mut screen = window_copy_backing(&mut *data);
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut hsize: u_int = 0;
        px = (*data).cx;
        hsize = (*screen_grid_ptr(&mut *back_s)).hsize;
        py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
        let mut gr = grid_reader_start(screen_grid(&*back_s), px, py);
        grid_reader_cursor_previous_word(
            &mut gr,
            separators,
            0 as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
        );
        (px, py) = grid_reader_get_cursor(&gr);
        (px, py)
    }
}
unsafe fn window_copy_cursor_previous_word(
    wme: &mut window_mode_entry,
    separators: &CStr,
    mut already: ::core::ffi::c_int,
) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut w: *mut window = (*wme.wp).window;
        let mut back_s: *mut screen = window_copy_backing(&mut *data);
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut oldy: u_int = 0;
        let mut hsize: u_int = 0;
        let mut stop_at_eol: ::core::ffi::c_int = 0;
        if options_get_number((*w).options_ptr(), c"mode-keys".as_ptr())
            == MODEKEY_EMACS as ::core::ffi::c_longlong
        {
            stop_at_eol = 1 as ::core::ffi::c_int;
        } else {
            stop_at_eol = 0 as ::core::ffi::c_int;
        }
        px = (*data).cx;
        hsize = (*screen_grid_ptr(&mut *back_s)).hsize;
        py = hsize.wrapping_add((*data).cy).wrapping_sub((*data).oy);
        oldy = (*data).cy;
        let mut gr = grid_reader_start(screen_grid(&*back_s), px, py);
        grid_reader_cursor_previous_word(&mut gr, separators, already, stop_at_eol);
        (px, py) = grid_reader_get_cursor(&gr);
        window_copy_acquire_cursor_up(wme, hsize, (*data).oy, oldy, px, py);
    }
}
unsafe fn window_copy_cursor_prompt(
    wme: &mut window_mode_entry,
    mut direction: ::core::ffi::c_int,
    mut start_output: ::core::ffi::c_int,
) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut s: *mut screen = window_copy_backing(&mut *data);
        let mut gd: *mut grid = screen_grid_ptr(&mut *s);
        let mut end_line: u_int = 0;
        let mut line: u_int = (*gd)
            .hsize
            .wrapping_sub((*data).oy)
            .wrapping_add((*data).cy);
        let mut add: ::core::ffi::c_int = 0;
        let mut line_flag: ::core::ffi::c_int = 0;
        if start_output != 0 {
            line_flag = GRID_LINE_START_OUTPUT;
        } else {
            line_flag = GRID_LINE_START_PROMPT;
        }
        if direction == 0 as ::core::ffi::c_int {
            add = -(1 as ::core::ffi::c_int);
            end_line = 0 as u_int;
        } else {
            add = 1 as ::core::ffi::c_int;
            end_line = (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int);
        }
        if line == end_line {
            return;
        }
        loop {
            if line == end_line {
                return;
            }
            line = line.wrapping_add(add as u_int);
            if grid_get_line(&mut *gd, line).flags & line_flag != 0 {
                break;
            }
        }
        (*data).cx = 0 as u_int;
        if line > (*gd).hsize {
            (*data).cy = line.wrapping_sub((*gd).hsize);
            (*data).oy = 0 as u_int;
        } else {
            (*data).cy = 0 as u_int;
            (*data).oy = (*gd).hsize.wrapping_sub(line);
        }
        window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        window_copy_redraw_screen(wme);
    }
}
unsafe fn window_copy_scroll_up(wme: &mut window_mode_entry, mut ny: u_int) {
    unsafe {
        let mut wp: *mut window_pane = wme.wp;
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut s: *mut screen = &raw mut (*data).screen;
        let mut ctx = screen_write_ctx::default();
        if (*data).oy < ny {
            ny = (*data).oy;
        }
        if ny == 0 as u_int {
            return;
        }
        (*data).oy = (*data).oy.wrapping_sub(ny);
        if !(*data).searchmark.is_empty() && (*data).timeout == 0 {
            window_copy_search_marks(
                wme,
                ::core::ptr::null_mut::<screen>(),
                (*data).searchregex,
                1 as ::core::ffi::c_int,
            );
        }
        window_copy_update_selection(wme, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        if window_copy_line_numbers_active(wme) != 0 {
            if window_copy_line_number_mode(wme)
                != WINDOW_COPY_LINE_NUMBERS_ABSOLUTE as ::core::ffi::c_int
            {
                window_copy_redraw_screen(wme);
                return;
            }
            screen_write_start(&mut ctx, &mut (*data).screen);
            screen_write_cursormove(
                &mut ctx,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            screen_write_deleteline(&mut ctx, ny, 8 as u_int);
            window_copy_write_lines(wme, &mut ctx, (*screen_grid_ptr(&mut *s)).sy.wrapping_sub(ny), ny);
            window_copy_write_line(wme, &mut ctx, 0 as u_int);
            if (*screen_grid_ptr(&mut *s)).sy > 1 as u_int {
                window_copy_write_line(wme, &mut ctx, 1 as u_int);
            }
            if (*screen_grid_ptr(&mut *s)).sy > 3 as u_int {
                window_copy_write_line(
                    wme,
                    &mut ctx,
                    (*screen_grid_ptr(&mut *s)).sy.wrapping_sub(2 as u_int),
                );
            }
            if (*s).sel.is_some() && (*screen_grid_ptr(&mut *s)).sy > ny {
                window_copy_write_line(
                    wme,
                    &mut ctx,
                    (*screen_grid_ptr(&mut *s))
                        .sy
                        .wrapping_sub(ny)
                        .wrapping_sub(1 as u_int),
                );
            }
            screen_write_cursormove(
                &mut ctx,
                window_copy_cursor_offset(wme, (*data).cx, (*screen_grid_ptr(&mut *s)).sx)
                    as ::core::ffi::c_int,
                (*data).cy as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            screen_write_stop(&mut ctx);
            (*wp).flags |= PANE_REDRAW | PANE_REDRAWSCROLLBAR;
            return;
        }
        screen_write_start_pane(&mut ctx, wp, None);
        screen_write_cursormove(
            &mut ctx,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_deleteline(&mut ctx, ny, 8 as u_int);
        window_copy_write_lines(wme, &mut ctx, (*screen_grid_ptr(&mut *s)).sy.wrapping_sub(ny), ny);
        window_copy_write_line(wme, &mut ctx, 0 as u_int);
        if (*screen_grid_ptr(&mut *s)).sy > 1 as u_int {
            window_copy_write_line(wme, &mut ctx, 1 as u_int);
        }
        if (*screen_grid_ptr(&mut *s)).sy > 3 as u_int {
            window_copy_write_line(
                wme,
                &mut ctx,
                (*screen_grid_ptr(&mut *s)).sy.wrapping_sub(2 as u_int),
            );
        }
        if (*s).sel.is_some() && (*screen_grid_ptr(&mut *s)).sy > ny {
            window_copy_write_line(
                wme,
                &mut ctx,
                (*screen_grid_ptr(&mut *s))
                    .sy
                    .wrapping_sub(ny)
                    .wrapping_sub(1 as u_int),
            );
        }
        screen_write_cursormove(
            &mut ctx,
            window_copy_cursor_offset(wme, (*data).cx, (*screen_grid_ptr(&mut *s)).sx)
                as ::core::ffi::c_int,
            (*data).cy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_stop(&mut ctx);
        (*wp).flags |= PANE_REDRAWSCROLLBAR;
    }
}
unsafe fn window_copy_scroll_down(wme: &mut window_mode_entry, mut ny: u_int) {
    unsafe {
        let mut wp: *mut window_pane = wme.wp;
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut s: *mut screen = &raw mut (*data).screen;
        let mut ctx = screen_write_ctx::default();
        if ny > (*screen_grid_ptr(&mut *window_copy_backing(&mut *data))).hsize {
            return;
        }
        if (*data).oy
            > (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
                .hsize
                .wrapping_sub(ny)
        {
            ny = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
                .hsize
                .wrapping_sub((*data).oy);
        }
        if ny == 0 as u_int {
            return;
        }
        (*data).oy = (*data).oy.wrapping_add(ny);
        if !(*data).searchmark.is_empty() && (*data).timeout == 0 {
            window_copy_search_marks(
                wme,
                ::core::ptr::null_mut::<screen>(),
                (*data).searchregex,
                1 as ::core::ffi::c_int,
            );
        }
        window_copy_update_selection(wme, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        if window_copy_line_numbers_active(wme) != 0 {
            if window_copy_line_number_mode(wme)
                != WINDOW_COPY_LINE_NUMBERS_ABSOLUTE as ::core::ffi::c_int
            {
                window_copy_redraw_screen(wme);
                return;
            }
            screen_write_start(&mut ctx, &mut (*data).screen);
            screen_write_cursormove(
                &mut ctx,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            screen_write_insertline(&mut ctx, ny, 8 as u_int);
            window_copy_write_lines(wme, &mut ctx, 0 as u_int, ny);
            if (*s).sel.is_some() && (*screen_grid_ptr(&mut *s)).sy > ny {
                window_copy_write_line(wme, &mut ctx, ny);
            } else if ny == 1 as u_int {
                window_copy_write_line(wme, &mut ctx, 1 as u_int);
            }
            screen_write_cursormove(
                &mut ctx,
                window_copy_cursor_offset(wme, (*data).cx, (*screen_grid_ptr(&mut *s)).sx)
                    as ::core::ffi::c_int,
                (*data).cy as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            screen_write_stop(&mut ctx);
            (*wp).flags |= PANE_REDRAW | PANE_REDRAWSCROLLBAR;
            return;
        }
        screen_write_start_pane(&mut ctx, wp, None);
        screen_write_cursormove(
            &mut ctx,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_insertline(&mut ctx, ny, 8 as u_int);
        window_copy_write_lines(wme, &mut ctx, 0 as u_int, ny);
        if (*s).sel.is_some() && (*screen_grid_ptr(&mut *s)).sy > ny {
            window_copy_write_line(wme, &mut ctx, ny);
        } else if ny == 1 as u_int {
            window_copy_write_line(wme, &mut ctx, 1 as u_int);
        }
        screen_write_cursormove(
            &mut ctx,
            window_copy_cursor_offset(wme, (*data).cx, (*screen_grid_ptr(&mut *s)).sx)
                as ::core::ffi::c_int,
            (*data).cy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_stop(&mut ctx);
        (*wp).flags |= PANE_REDRAWSCROLLBAR;
    }
}
unsafe fn window_copy_rectangle_set(wme: &mut window_mode_entry, mut rectflag: ::core::ffi::c_int) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        (*data).rectflag = rectflag;
        py = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        px = window_copy_cursor_limit(wme, py, (*data).rectflag);
        if (*data).cx > px {
            window_copy_update_cursor(wme, px, (*data).cy);
        }
        window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        window_copy_redraw_screen(wme);
    }
}
unsafe fn window_copy_move_mouse(m: &mouse_event) {
    unsafe {
        let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
        let mut x: u_int = 0;
        let mut y: u_int = 0;
        let mut data: *mut window_copy_mode_data = ::core::ptr::null_mut::<window_copy_mode_data>();
        let Some((_, _, wp)) = cmd_mouse_pane(m) else {
            return;
        };
        wme = window_pane_current_mode(wp);
        if wme.is_null() {
            return;
        }
        if (*wme).mode() != WindowMode::Copy && (*wme).mode() != WindowMode::View {
            return;
        }
        if match cmd_mouse_at(wp, m, 0 as ::core::ffi::c_int) {
            Some((at_x, at_y)) => {
                (x, y) = (at_x, at_y);
                false
            }
            None => true,
        } {
            return;
        }
        data = (*wme).state.copy();
        x = window_copy_cursor_unoffset(
            &mut *wme,
            x,
            (*screen_grid_ptr(&mut (*data).screen)).sx,
        );
        window_copy_update_cursor(&mut *wme, x, y);
    }
}
pub unsafe fn window_copy_start_drag(mut c: *mut client, m: &mouse_event) {
    unsafe {
        let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
        let mut data: *mut window_copy_mode_data = ::core::ptr::null_mut::<window_copy_mode_data>();
        let mut x: u_int = 0;
        let mut y: u_int = 0;
        let mut yg: u_int = 0;
        if c.is_null() {
            return;
        }
        let Some((_, _, wp)) = cmd_mouse_pane(m) else {
            return;
        };
        wme = window_pane_current_mode(wp);
        if wme.is_null() {
            return;
        }
        if (*wme).mode() != WindowMode::Copy && (*wme).mode() != WindowMode::View {
            return;
        }
        if match cmd_mouse_at(wp, m, 1 as ::core::ffi::c_int) {
            Some((at_x, at_y)) => {
                (x, y) = (at_x, at_y);
                false
            }
            None => true,
        } {
            return;
        }
        (*c).tty.mouse_drag_update = Some(window_copy_drag_update);
        (*c).tty.mouse_drag_release = Some(window_copy_drag_release);
        data = (*wme).state.copy();
        x = window_copy_cursor_unoffset(
            &mut *wme,
            x,
            (*screen_grid_ptr(&mut (*data).screen)).sx,
        );
        yg = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_add(y)
            .wrapping_sub((*data).oy);
        if x < (*data).selrx || x > (*data).endselrx || yg != (*data).selry {
            (*data).selflag = SEL_CHAR;
        }
        match (*data).selflag {
            SEL_WORD => {
                if (*data).separators.is_some() {
                    window_copy_update_cursor(&mut *wme, x, y);
                    (x, y) = window_copy_cursor_previous_word_pos(
                        &mut *wme,
                        (*data).separators.as_deref().unwrap_or(c""),
                    );
                    y = y.wrapping_sub(
                        (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
                            .hsize
                            .wrapping_sub((*data).oy),
                    );
                }
                window_copy_update_cursor(&mut *wme, x, y);
            }
            SEL_LINE => {
                window_copy_update_cursor(&mut *wme, 0 as u_int, y);
            }
            SEL_CHAR => {
                window_copy_update_cursor(&mut *wme, x, y);
                window_copy_start_selection(&mut *wme);
            }
            _ => {}
        }
        window_copy_redraw_screen(&mut *wme);
        window_copy_drag_update(c, m);
    }
}
unsafe fn window_copy_drag_update(mut c: *mut client, m: &mouse_event) {
    unsafe {
        let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
        let mut data: *mut window_copy_mode_data = ::core::ptr::null_mut::<window_copy_mode_data>();
        let mut x: u_int = 0;
        let mut y: u_int = 0;
        let mut old_cx: u_int = 0;
        let mut old_cy: u_int = 0;
        let mut tv = timeval::from_usecs(WINDOW_COPY_DRAG_REPEAT_TIME as __suseconds_t);
        if c.is_null() {
            return;
        }
        let Some((_, _, wp)) = cmd_mouse_pane(m) else {
            return;
        };
        wme = window_pane_current_mode(wp);
        if wme.is_null() {
            return;
        }
        if (*wme).mode() != WindowMode::Copy && (*wme).mode() != WindowMode::View {
            return;
        }
        data = (*wme).state.copy();
        (*data).dragtimer.disarm();
        if match cmd_mouse_at(wp, m, 0 as ::core::ffi::c_int) {
            Some((at_x, at_y)) => {
                (x, y) = (at_x, at_y);
                false
            }
            None => true,
        } {
            return;
        }
        x = window_copy_cursor_unoffset(
            &mut *wme,
            x,
            (*screen_grid_ptr(&mut (*data).screen)).sx,
        );
        old_cx = (*data).cx;
        old_cy = (*data).cy;
        window_copy_update_cursor(&mut *wme, x, y);
        if window_copy_update_selection(&mut *wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int)
            != 0
        {
            window_copy_redraw_selection(&mut *wme, old_cy);
        }
        if old_cy != (*data).cy || old_cx == (*data).cx {
            if y == 0 as u_int {
                (*data).dragtimer.arm(tv);
                window_copy_cursor_up(&mut *wme, 1 as ::core::ffi::c_int);
            } else if y
                == (*screen_grid_ptr(&mut (*data).screen))
                    .sy
                    .wrapping_sub(1 as u_int)
            {
                (*data).dragtimer.arm(tv);
                window_copy_cursor_down(&mut *wme, 1 as ::core::ffi::c_int);
            }
        }
    }
}
unsafe fn window_copy_drag_release(mut c: *mut client, m: &mouse_event) {
    unsafe {
        let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
        let mut data: *mut window_copy_mode_data = ::core::ptr::null_mut::<window_copy_mode_data>();
        if c.is_null() {
            return;
        }
        let Some((_, _, wp)) = cmd_mouse_pane(m) else {
            return;
        };
        wme = window_pane_current_mode(wp);
        if wme.is_null() {
            return;
        }
        if (*wme).mode() != WindowMode::Copy && (*wme).mode() != WindowMode::View {
            return;
        }
        data = (*wme).state.copy();
        if window_copy_line_numbers_active(&mut *wme) != 0 {
            window_copy_drag_update(c, m);
        }
        (*data).dragtimer.disarm();
    }
}
unsafe fn window_copy_jump_to_mark(wme: &mut window_mode_entry) {
    unsafe {
        let mut data: *mut window_copy_mode_data = wme.state.copy();
        let mut tmx: u_int = 0;
        let mut tmy: u_int = 0;
        tmx = (*data).cx;
        tmy = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
            .hsize
            .wrapping_add((*data).cy)
            .wrapping_sub((*data).oy);
        (*data).cx = (*data).mx;
        if (*data).my < (*screen_grid_ptr(&mut *window_copy_backing(&mut *data))).hsize {
            (*data).cy = 0 as u_int;
            (*data).oy = (*screen_grid_ptr(&mut *window_copy_backing(&mut *data)))
                .hsize
                .wrapping_sub((*data).my);
        } else {
            (*data).cy = (*data)
                .my
                .wrapping_sub((*screen_grid_ptr(&mut *window_copy_backing(&mut *data))).hsize);
            (*data).oy = 0 as u_int;
        }
        (*data).mx = tmx;
        (*data).my = tmy;
        (*data).showmark = 1 as ::core::ffi::c_int;
        window_copy_update_selection(wme, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        window_copy_redraw_screen(wme);
    }
}
unsafe fn window_copy_acquire_cursor_up(
    wme: &mut window_mode_entry,
    mut hsize: u_int,
    mut oy: u_int,
    mut oldy: u_int,
    mut px: u_int,
    mut py: u_int,
) {
    unsafe {
        let mut cy: u_int = 0;
        let mut yy: u_int = 0;
        let mut ny: u_int = 0;
        let mut nd: u_int = 0;
        yy = hsize.wrapping_sub(oy);
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
            window_copy_cursor_up(wme, 1 as ::core::ffi::c_int);
            ny = ny.wrapping_sub(1);
        }
        window_copy_update_cursor(wme, px, cy);
        if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) != 0
        {
            window_copy_redraw_lines(wme, cy, nd);
        }
    }
}
unsafe fn window_copy_acquire_cursor_down(
    wme: &mut window_mode_entry,
    mut hsize: u_int,
    mut sy: u_int,
    mut oy: u_int,
    mut oldy: u_int,
    mut px: u_int,
    mut py: u_int,
    mut no_reset: ::core::ffi::c_int,
) {
    unsafe {
        let mut cy: u_int = 0;
        let mut yy: u_int = 0;
        let mut ny: u_int = 0;
        let mut nd: u_int = 0;
        cy = py.wrapping_sub(hsize).wrapping_add(oy);
        yy = sy.wrapping_sub(1 as u_int);
        if cy > yy {
            ny = cy.wrapping_sub(yy);
            oldy = yy;
            nd = 1 as u_int;
        } else {
            ny = 0 as u_int;
            nd = cy.wrapping_sub(oldy).wrapping_add(1 as u_int);
        }
        while ny > 0 as u_int {
            window_copy_cursor_down(wme, 1 as ::core::ffi::c_int);
            ny = ny.wrapping_sub(1);
        }
        if cy > yy {
            window_copy_update_cursor(wme, px, yy);
        } else {
            window_copy_update_cursor(wme, px, cy);
        }
        if window_copy_update_selection(wme, 1 as ::core::ffi::c_int, no_reset) != 0 {
            window_copy_redraw_lines(wme, oldy, nd);
        }
    }
}
pub const __SCHAR_MAX__: ::core::ffi::c_int = 127 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
