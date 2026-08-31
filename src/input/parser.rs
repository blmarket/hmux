use crate::alerts::alerts_queue;
use crate::compat::strtonum;
use crate::ffi::{
    __b64_ntop, __b64_pton, strchr, strcmp, strlen, strncmp, strpbrk, strsep, strtol,
};
use crate::fmt_args;
use crate::fmt_engine::{FmtArg, format_alloc};
use crate::grid::hyperlinks_put;
use crate::grid::{grid_cells_look_equal, grid_set_tab};
use crate::grid::{grid_default_cell, grid_get_cell, grid_get_line};
use crate::list::{foreach_owned_safe, foreach_safe};
use crate::log::{fatalx, log_debug};
use crate::notify::notify_pane;
use crate::options::options_get_only_ptr;
use crate::options::{options_get_number, options_remove_or_default, options_set_number};
use crate::paste::paste_add;
use crate::paste::{paste_buffer_data, paste_get_top};
use crate::reactor::Timer;
use crate::screen::screen_set_cursor_style;
use crate::screen::{
    screen_grid, screen_grid_ptr, screen_pop_title, screen_push_title, screen_set_cursor_colour,
    screen_set_path, screen_set_progress_bar, screen_set_title,
};
use crate::screen::{
    screen_write_alignmenttest, screen_write_alternateoff, screen_write_alternateon,
    screen_write_backspace, screen_write_carriagereturn, screen_write_clearcharacter,
    screen_write_clearendofline, screen_write_clearendofscreen, screen_write_clearhistory,
    screen_write_clearline, screen_write_clearscreen, screen_write_clearstartofline,
    screen_write_clearstartofscreen, screen_write_collect_add, screen_write_collect_end,
    screen_write_cursordown, screen_write_cursorleft, screen_write_cursormove,
    screen_write_cursorright, screen_write_cursorup, screen_write_deletecharacter,
    screen_write_deleteline, screen_write_fullredraw, screen_write_insertcharacter,
    screen_write_insertline, screen_write_linefeed, screen_write_mode_clear, screen_write_mode_set,
    screen_write_rawstring, screen_write_reset, screen_write_reverseindex, screen_write_scrolldown,
    screen_write_scrollregion, screen_write_scrollup, screen_write_setselection,
    screen_write_start, screen_write_start_callback, screen_write_start_pane,
    screen_write_start_sync, screen_write_stop, screen_write_stop_sync,
};
use crate::server::client_ref_from_ptr;
use crate::server::client_walk;
use crate::server::{server_redraw_window_borders, server_status_window};
use crate::session::session_has;
use crate::style::{
    colour_force_rgb, colour_join_rgb, colour_palette_clear, colour_palette_get,
    colour_palette_set, colour_parseX11, colour_split_rgb,
};
use crate::text::{utf8_append, utf8_copy, utf8_isvalid, utf8_open, utf8_set};
use crate::tmux::{get_timer, getversion};
use crate::tmux::{global_options, global_w_options};
use crate::tty::{tty_default_colours, tty_putcode_ss, tty_puts, tty_set_selection};
pub use crate::types::*;
use crate::window::{
    window_pane_get_bg, window_pane_get_fg, window_pane_get_fg_control_client,
    window_pane_get_new_data, window_pane_get_theme, window_pane_update_used_data, window_set_name,
    window_update_activity,
};
use crate::xmalloc::xasprintf;
use ::core::cell::UnsafeCell;
use ::std::ffi::{CStr, CString};
use ::std::rc::{Rc, Weak};
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
#[repr(C)]
pub struct input_ctx {
    /// What the parser is parsing output for.
    pub(crate) owner_of: InputOwner,
    pub event: Stream,
    pub ctx: screen_write_ctx,
    pub cell: input_cell,
    pub old_cell: input_cell,
    pub old_cx: u_int,
    pub old_cy: u_int,
    pub old_mode: ::core::ffi::c_int,
    pub interm_buf: [u_char; 4],
    pub interm_len: size_t,
    pub param_buf: [u_char; 64],
    pub param_len: size_t,
    pub input_buf: Vec<u8>,
    pub input_end: input_end_type,
    pub param_list: [InputParam; 24],
    pub param_list_len: u_int,
    pub utf8data: utf8_data,
    pub utf8started: ::core::ffi::c_int,
    pub ch: ::core::ffi::c_int,
    pub last: utf8_data,
    /// The state the parser is in. A state is a table entry that lives for as
    /// long as the process does, so the parser always has one.
    pub state: &'static input_state,
    pub flags: ::core::ffi::c_int,
    pub requests: input_request_list,
    pub request_count: u_int,
    pub request_timer: TimerHandle,
    pub since_ground: Option<Box<Buf>>,
    pub ground_timer: TimerHandle,
    /// The parser's observation of itself, which is what a request it made
    /// holds it by.
    pub(crate) owner: Option<InputCtxWeak>,
}
#[repr(C)]
pub struct input_request {
    /// The client the request went to, observed rather than held, so that a
    /// client which goes before the answer arrives leaves nothing behind.
    pub(crate) client: Option<ClientWeak>,
    pub(crate) ictx: InputCtxWeak,
    pub type_0: input_request_type,
    pub t: uint64_t,
    pub end: input_end_type,
    pub idx: ::core::ffi::c_int,
    pub data: Option<::std::ffi::CString>,
}
pub type input_end_type = ::core::ffi::c_uint;
pub const INPUT_END_BEL: input_end_type = 1;
pub const INPUT_END_ST: input_end_type = 0;
pub const INPUT_REQUEST_QUEUE: input_request_type = 2;
pub const INPUT_REQUEST_CLIPBOARD: input_request_type = 1;
pub const INPUT_REQUEST_PALETTE: input_request_type = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct input_state {
    pub name: &'static ::core::ffi::CStr,
    pub enter: Option<unsafe fn(*mut input_ctx) -> ()>,
    pub exit: Option<unsafe fn(*mut input_ctx) -> ()>,
    pub transitions: &'static [input_transition],
}
/// The behaviour a transition runs for the byte it matched.
///
/// The parser tables name one of these instead of carrying a function
/// pointer, so a transition can be recognised by value: `input_parse` asks
/// whether the selected transition is [`InputHandler::Print`] to decide
/// whether the screen writer's pending text batch survives the byte.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum InputHandler {
    Print,
    C0Dispatch,
    EscDispatch,
    CsiDispatch,
    DcsDispatch,
    Parameter,
    Intermediate,
    Input,
    TopBitSet,
    EndBel,
}
impl InputHandler {
    /// Runs the handler; a non-zero result asks `input_parse` to stop
    /// processing the byte, exactly as the C callback's return did.
    unsafe fn call(self, ictx: *mut input_ctx) -> ::core::ffi::c_int {
        unsafe {
            match self {
                InputHandler::Print => input_print(ictx),
                InputHandler::C0Dispatch => input_c0_dispatch(ictx),
                InputHandler::EscDispatch => input_esc_dispatch(ictx),
                InputHandler::CsiDispatch => input_csi_dispatch(ictx),
                InputHandler::DcsDispatch => input_dcs_dispatch(ictx),
                InputHandler::Parameter => input_parameter(ictx),
                InputHandler::Intermediate => input_intermediate(ictx),
                InputHandler::Input => input_input(ictx),
                InputHandler::TopBitSet => input_top_bit_set(ictx),
                InputHandler::EndBel => input_end_bel(ictx),
            }
        }
    }
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct input_transition {
    pub first: ::core::ffi::c_int,
    pub last: ::core::ffi::c_int,
    pub handler: Option<InputHandler>,
    /// The state the transition moves to, or nothing for one that stays where
    /// it is.
    pub state: Option<&'static input_state>,
}

/// One parameter of a control sequence: left out, a number, or the
/// colon-joined string a caller takes apart for itself.
pub enum InputParam {
    Missing,
    Number(::core::ffi::c_int),
    Str(CString),
}
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
pub const TTYC_XT: tty_code_code = 232;
pub const TTYC_VPA: tty_code_code = 231;
pub const TTYC_U8: tty_code_code = 230;
pub const TTYC_TSL: tty_code_code = 229;
pub const TTYC_TC: tty_code_code = 228;
pub const TTYC_SYNC: tty_code_code = 227;
pub const TTYC_SWD: tty_code_code = 226;
pub const TTYC_SS: tty_code_code = 225;
pub const TTYC_SXL: tty_code_code = 224;
pub const TTYC_SPB: tty_code_code = 223;
pub const TTYC_SMXX: tty_code_code = 222;
pub const TTYC_SMULX: tty_code_code = 221;
pub const TTYC_SMUL: tty_code_code = 220;
pub const TTYC_SMSO: tty_code_code = 219;
pub const TTYC_SMOL: tty_code_code = 218;
pub const TTYC_SMKX: tty_code_code = 217;
pub const TTYC_SMCUP: tty_code_code = 216;
pub const TTYC_SMACS: tty_code_code = 215;
pub const TTYC_SITM: tty_code_code = 214;
pub const TTYC_SGR0: tty_code_code = 213;
pub const TTYC_SETULC1: tty_code_code = 212;
pub const TTYC_SETULC: tty_code_code = 211;
pub const TTYC_SETRGBF: tty_code_code = 210;
pub const TTYC_SETRGBB: tty_code_code = 209;
pub const TTYC_SETAL: tty_code_code = 208;
pub const TTYC_SETAF: tty_code_code = 207;
pub const TTYC_SETAB: tty_code_code = 206;
pub const TTYC_SE: tty_code_code = 205;
pub const TTYC_RMKX: tty_code_code = 204;
pub const TTYC_RMCUP: tty_code_code = 203;
pub const TTYC_RMACS: tty_code_code = 202;
pub const TTYC_RIN: tty_code_code = 201;
pub const TTYC_RI: tty_code_code = 200;
pub const TTYC_RGB: tty_code_code = 199;
pub const TTYC_REV: tty_code_code = 198;
pub const TTYC_RECT: tty_code_code = 197;
pub const TTYC_OP: tty_code_code = 196;
pub const TTYC_OL: tty_code_code = 195;
pub const TTYC_NOBR: tty_code_code = 194;
pub const TTYC_MS: tty_code_code = 193;
pub const TTYC_KUP7: tty_code_code = 192;
pub const TTYC_KUP6: tty_code_code = 191;
pub const TTYC_KUP5: tty_code_code = 190;
pub const TTYC_KUP4: tty_code_code = 189;
pub const TTYC_KUP3: tty_code_code = 188;
pub const TTYC_KUP2: tty_code_code = 187;
pub const TTYC_KRIT7: tty_code_code = 186;
pub const TTYC_KRIT6: tty_code_code = 185;
pub const TTYC_KRIT5: tty_code_code = 184;
pub const TTYC_KRIT4: tty_code_code = 183;
pub const TTYC_KRIT3: tty_code_code = 182;
pub const TTYC_KRIT2: tty_code_code = 181;
pub const TTYC_KRI: tty_code_code = 180;
pub const TTYC_KPRV7: tty_code_code = 179;
pub const TTYC_KPRV6: tty_code_code = 178;
pub const TTYC_KPRV5: tty_code_code = 177;
pub const TTYC_KPRV4: tty_code_code = 176;
pub const TTYC_KPRV3: tty_code_code = 175;
pub const TTYC_KPRV2: tty_code_code = 174;
pub const TTYC_KPP: tty_code_code = 173;
pub const TTYC_KNXT7: tty_code_code = 172;
pub const TTYC_KNXT6: tty_code_code = 171;
pub const TTYC_KNXT5: tty_code_code = 170;
pub const TTYC_KNXT4: tty_code_code = 169;
pub const TTYC_KNXT3: tty_code_code = 168;
pub const TTYC_KNXT2: tty_code_code = 167;
pub const TTYC_KNP: tty_code_code = 166;
pub const TTYC_KMOUS: tty_code_code = 165;
pub const TTYC_KLFT7: tty_code_code = 164;
pub const TTYC_KLFT6: tty_code_code = 163;
pub const TTYC_KLFT5: tty_code_code = 162;
pub const TTYC_KLFT4: tty_code_code = 161;
pub const TTYC_KLFT3: tty_code_code = 160;
pub const TTYC_KLFT2: tty_code_code = 159;
pub const TTYC_KIND: tty_code_code = 158;
pub const TTYC_KICH1: tty_code_code = 157;
pub const TTYC_KIC7: tty_code_code = 156;
pub const TTYC_KIC6: tty_code_code = 155;
pub const TTYC_KIC5: tty_code_code = 154;
pub const TTYC_KIC4: tty_code_code = 153;
pub const TTYC_KIC3: tty_code_code = 152;
pub const TTYC_KIC2: tty_code_code = 151;
pub const TTYC_KHOME: tty_code_code = 150;
pub const TTYC_KHOM7: tty_code_code = 149;
pub const TTYC_KHOM6: tty_code_code = 148;
pub const TTYC_KHOM5: tty_code_code = 147;
pub const TTYC_KHOM4: tty_code_code = 146;
pub const TTYC_KHOM3: tty_code_code = 145;
pub const TTYC_KHOM2: tty_code_code = 144;
pub const TTYC_KF9: tty_code_code = 143;
pub const TTYC_KF8: tty_code_code = 142;
pub const TTYC_KF7: tty_code_code = 141;
pub const TTYC_KF63: tty_code_code = 140;
pub const TTYC_KF62: tty_code_code = 139;
pub const TTYC_KF61: tty_code_code = 138;
pub const TTYC_KF60: tty_code_code = 137;
pub const TTYC_KF6: tty_code_code = 136;
pub const TTYC_KF59: tty_code_code = 135;
pub const TTYC_KF58: tty_code_code = 134;
pub const TTYC_KF57: tty_code_code = 133;
pub const TTYC_KF56: tty_code_code = 132;
pub const TTYC_KF55: tty_code_code = 131;
pub const TTYC_KF54: tty_code_code = 130;
pub const TTYC_KF53: tty_code_code = 129;
pub const TTYC_KF52: tty_code_code = 128;
pub const TTYC_KF51: tty_code_code = 127;
pub const TTYC_KF50: tty_code_code = 126;
pub const TTYC_KF5: tty_code_code = 125;
pub const TTYC_KF49: tty_code_code = 124;
pub const TTYC_KF48: tty_code_code = 123;
pub const TTYC_KF47: tty_code_code = 122;
pub const TTYC_KF46: tty_code_code = 121;
pub const TTYC_KF45: tty_code_code = 120;
pub const TTYC_KF44: tty_code_code = 119;
pub const TTYC_KF43: tty_code_code = 118;
pub const TTYC_KF42: tty_code_code = 117;
pub const TTYC_KF41: tty_code_code = 116;
pub const TTYC_KF40: tty_code_code = 115;
pub const TTYC_KF4: tty_code_code = 114;
pub const TTYC_KF39: tty_code_code = 113;
pub const TTYC_KF38: tty_code_code = 112;
pub const TTYC_KF37: tty_code_code = 111;
pub const TTYC_KF36: tty_code_code = 110;
pub const TTYC_KF35: tty_code_code = 109;
pub const TTYC_KF34: tty_code_code = 108;
pub const TTYC_KF33: tty_code_code = 107;
pub const TTYC_KF32: tty_code_code = 106;
pub const TTYC_KF31: tty_code_code = 105;
pub const TTYC_KF30: tty_code_code = 104;
pub const TTYC_KF3: tty_code_code = 103;
pub const TTYC_KF29: tty_code_code = 102;
pub const TTYC_KF28: tty_code_code = 101;
pub const TTYC_KF27: tty_code_code = 100;
pub const TTYC_KF26: tty_code_code = 99;
pub const TTYC_KF25: tty_code_code = 98;
pub const TTYC_KF24: tty_code_code = 97;
pub const TTYC_KF23: tty_code_code = 96;
pub const TTYC_KF22: tty_code_code = 95;
pub const TTYC_KF21: tty_code_code = 94;
pub const TTYC_KF20: tty_code_code = 93;
pub const TTYC_KF2: tty_code_code = 92;
pub const TTYC_KF19: tty_code_code = 91;
pub const TTYC_KF18: tty_code_code = 90;
pub const TTYC_KF17: tty_code_code = 89;
pub const TTYC_KF16: tty_code_code = 88;
pub const TTYC_KF15: tty_code_code = 87;
pub const TTYC_KF14: tty_code_code = 86;
pub const TTYC_KF13: tty_code_code = 85;
pub const TTYC_KF12: tty_code_code = 84;
pub const TTYC_KF11: tty_code_code = 83;
pub const TTYC_KF10: tty_code_code = 82;
pub const TTYC_KF1: tty_code_code = 81;
pub const TTYC_KEND7: tty_code_code = 80;
pub const TTYC_KEND6: tty_code_code = 79;
pub const TTYC_KEND5: tty_code_code = 78;
pub const TTYC_KEND4: tty_code_code = 77;
pub const TTYC_KEND3: tty_code_code = 76;
pub const TTYC_KEND2: tty_code_code = 75;
pub const TTYC_KEND: tty_code_code = 74;
pub const TTYC_KDN7: tty_code_code = 73;
pub const TTYC_KDN6: tty_code_code = 72;
pub const TTYC_KDN5: tty_code_code = 71;
pub const TTYC_KDN4: tty_code_code = 70;
pub const TTYC_KDN3: tty_code_code = 69;
pub const TTYC_KDN2: tty_code_code = 68;
pub const TTYC_KDCH1: tty_code_code = 67;
pub const TTYC_KDC7: tty_code_code = 66;
pub const TTYC_KDC6: tty_code_code = 65;
pub const TTYC_KDC5: tty_code_code = 64;
pub const TTYC_KDC4: tty_code_code = 63;
pub const TTYC_KDC3: tty_code_code = 62;
pub const TTYC_KDC2: tty_code_code = 61;
pub const TTYC_KCUU1: tty_code_code = 60;
pub const TTYC_KCUF1: tty_code_code = 59;
pub const TTYC_KCUD1: tty_code_code = 58;
pub const TTYC_KCUB1: tty_code_code = 57;
pub const TTYC_KCBT: tty_code_code = 56;
pub const TTYC_INVIS: tty_code_code = 55;
pub const TTYC_INDN: tty_code_code = 54;
pub const TTYC_IL1: tty_code_code = 53;
pub const TTYC_IL: tty_code_code = 52;
pub const TTYC_ICH1: tty_code_code = 51;
pub const TTYC_ICH: tty_code_code = 50;
pub const TTYC_HPA: tty_code_code = 49;
pub const TTYC_HOME: tty_code_code = 48;
pub const TTYC_HLS: tty_code_code = 47;
pub const TTYC_FSL: tty_code_code = 46;
pub const TTYC_ENMG: tty_code_code = 45;
pub const TTYC_ENFCS: tty_code_code = 44;
pub const TTYC_ENEKS: tty_code_code = 43;
pub const TTYC_ENBP: tty_code_code = 42;
pub const TTYC_ENACS: tty_code_code = 41;
pub const TTYC_EL1: tty_code_code = 40;
pub const TTYC_EL: tty_code_code = 39;
pub const TTYC_ED: tty_code_code = 38;
pub const TTYC_ECH: tty_code_code = 37;
pub const TTYC_E3: tty_code_code = 36;
pub const TTYC_DSMG: tty_code_code = 35;
pub const TTYC_DSFCS: tty_code_code = 34;
pub const TTYC_DSEKS: tty_code_code = 33;
pub const TTYC_DSBP: tty_code_code = 32;
pub const TTYC_DL1: tty_code_code = 31;
pub const TTYC_DL: tty_code_code = 30;
pub const TTYC_DIM: tty_code_code = 29;
pub const TTYC_DCH1: tty_code_code = 28;
pub const TTYC_DCH: tty_code_code = 27;
pub const TTYC_CVVIS: tty_code_code = 26;
pub const TTYC_CUU1: tty_code_code = 25;
pub const TTYC_CUU: tty_code_code = 24;
pub const TTYC_CUP: tty_code_code = 23;
pub const TTYC_CUF1: tty_code_code = 22;
pub const TTYC_CUF: tty_code_code = 21;
pub const TTYC_CUD1: tty_code_code = 20;
pub const TTYC_CUD: tty_code_code = 19;
pub const TTYC_CUB1: tty_code_code = 18;
pub const TTYC_CUB: tty_code_code = 17;
pub const TTYC_CSR: tty_code_code = 16;
pub const TTYC_CS: tty_code_code = 15;
pub const TTYC_CR: tty_code_code = 14;
pub const TTYC_COLORS: tty_code_code = 13;
pub const TTYC_CNORM: tty_code_code = 12;
pub const TTYC_CMG: tty_code_code = 11;
pub const TTYC_CLMG: tty_code_code = 10;
pub const TTYC_CLEAR: tty_code_code = 9;
pub const TTYC_CIVIS: tty_code_code = 8;
pub const TTYC_BOLD: tty_code_code = 7;
pub const TTYC_BLINK: tty_code_code = 6;
pub const TTYC_BIDI: tty_code_code = 5;
pub const TTYC_BEL: tty_code_code = 4;
pub const TTYC_BCE: tty_code_code = 3;
pub const TTYC_AX: tty_code_code = 2;
pub const TTYC_AM: tty_code_code = 1;
pub const TTYC_ACSC: tty_code_code = 0;
pub const UTF8_ERROR: utf8_state = 2;
pub const UTF8_DONE: utf8_state = 1;
pub const UTF8_MORE: utf8_state = 0;
pub const INPUT_ESC_ST: input_esc_type = 14;
pub const INPUT_ESC_SCSG1_OFF: input_esc_type = 12;
pub const INPUT_ESC_SCSG1_ON: input_esc_type = 13;
pub const INPUT_ESC_SCSG0_OFF: input_esc_type = 10;
pub const INPUT_ESC_SCSG0_ON: input_esc_type = 11;
pub const INPUT_ESC_DECALN: input_esc_type = 0;
pub const INPUT_ESC_DECRC: input_esc_type = 3;
pub const INPUT_ESC_DECSC: input_esc_type = 4;
pub const INPUT_ESC_DECKPNM: input_esc_type = 2;
pub const INPUT_ESC_DECKPAM: input_esc_type = 1;
pub const INPUT_ESC_RI: input_esc_type = 8;
pub const INPUT_ESC_HTS: input_esc_type = 5;
pub const INPUT_ESC_NEL: input_esc_type = 7;
pub const INPUT_ESC_IND: input_esc_type = 6;
pub const INPUT_ESC_RIS: input_esc_type = 9;
pub const INPUT_CSI_XDA: input_csi_type = 40;
pub const INPUT_CSI_DECSCUSR: input_csi_type = 11;
pub const INPUT_CSI_VPA: input_csi_type = 38;
pub const INPUT_CSI_TBC: input_csi_type = 37;
pub const INPUT_CSI_SD: input_csi_type = 31;
pub const INPUT_CSI_SU: input_csi_type = 36;
pub const INPUT_CSI_SM_GRAPHICS: input_csi_type = 34;
pub const INPUT_CSI_SM_PRIVATE: input_csi_type = 35;
pub const INPUT_CSI_SM: input_csi_type = 33;
pub const INPUT_CSI_SGR: input_csi_type = 32;
pub const INPUT_CSI_SCP: input_csi_type = 30;
pub const INPUT_CSI_RM_PRIVATE: input_csi_type = 29;
pub const INPUT_CSI_RM: input_csi_type = 28;
pub const INPUT_CSI_RCP: input_csi_type = 26;
pub const INPUT_CSI_REP: input_csi_type = 27;
pub const INPUT_CSI_IL: input_csi_type = 21;
pub const INPUT_CSI_ICH: input_csi_type = 20;
pub const INPUT_CSI_HPA: input_csi_type = 19;
pub const INPUT_CSI_EL: input_csi_type = 18;
pub const INPUT_CSI_ED: input_csi_type = 17;
pub const INPUT_CSI_DSR: input_csi_type = 14;
pub const INPUT_CSI_QUERY_PRIVATE: input_csi_type = 25;
pub const INPUT_CSI_QUERY: input_csi_type = 24;
pub const INPUT_CSI_DSR_PRIVATE: input_csi_type = 15;
pub const INPUT_CSI_DL: input_csi_type = 13;
pub const INPUT_CSI_DECSTBM: input_csi_type = 12;
pub const INPUT_CSI_DCH: input_csi_type = 10;
pub const INPUT_CSI_ECH: input_csi_type = 16;
pub const INPUT_CSI_DA_TWO: input_csi_type = 9;
pub const INPUT_CSI_DA: input_csi_type = 8;
pub const INPUT_CSI_CPL: input_csi_type = 2;
pub const INPUT_CSI_CNL: input_csi_type = 1;
pub const INPUT_CSI_CUU: input_csi_type = 7;
pub const INPUT_CSI_WINOPS: input_csi_type = 39;
pub const INPUT_CSI_MODOFF: input_csi_type = 22;
pub const INPUT_CSI_MODSET: input_csi_type = 23;
pub const INPUT_CSI_CUP: input_csi_type = 6;
pub const INPUT_CSI_CUF: input_csi_type = 5;
pub const INPUT_CSI_CUD: input_csi_type = 4;
pub const INPUT_CSI_CUB: input_csi_type = 3;
pub const INPUT_CSI_CBT: input_csi_type = 0;
pub type input_esc_type = ::core::ffi::c_uint;
pub type input_csi_type = ::core::ffi::c_uint;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
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
pub const ALL_MOUSE_MODES: ::core::ffi::c_int =
    MODE_MOUSE_STANDARD | MODE_MOUSE_BUTTON | MODE_MOUSE_ALL;
pub const EXTENDED_KEY_MODES: ::core::ffi::c_int = MODE_KEYS_EXTENDED | MODE_KEYS_EXTENDED_2;
pub const COLOUR_FLAG_256: ::core::ffi::c_int = 0x1000000 as ::core::ffi::c_int;
pub const GRID_ATTR_BRIGHT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const GRID_ATTR_DIM: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const GRID_ATTR_UNDERSCORE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const GRID_ATTR_BLINK: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const GRID_ATTR_REVERSE: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const GRID_ATTR_HIDDEN: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const GRID_ATTR_ITALICS: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const GRID_ATTR_CHARSET: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const GRID_ATTR_STRIKETHROUGH: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const GRID_ATTR_UNDERSCORE_2: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const GRID_ATTR_UNDERSCORE_3: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const GRID_ATTR_UNDERSCORE_4: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const GRID_ATTR_UNDERSCORE_5: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const GRID_ATTR_OVERLINE: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const GRID_ATTR_ALL_UNDERSCORE: ::core::ffi::c_int = GRID_ATTR_UNDERSCORE
    | GRID_ATTR_UNDERSCORE_2
    | GRID_ATTR_UNDERSCORE_3
    | GRID_ATTR_UNDERSCORE_4
    | GRID_ATTR_UNDERSCORE_5;
pub const GRID_LINE_START_PROMPT: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const GRID_LINE_START_OUTPUT: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const PANE_REDRAW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const PANE_CHANGED: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const PANE_STYLECHANGED: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const PANE_THEMECHANGED: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const PANE_UNSEENCHANGES: ::core::ffi::c_int = 0x4000 as ::core::ffi::c_int;
pub const WINDOW_BELL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const TTY_STARTED: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CLIENT_EXIT: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CLIENT_SUSPENDED: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const CLIENT_DEAD: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const CLIENT_UNATTACHEDFLAGS: ::core::ffi::c_int = CLIENT_DEAD | CLIENT_SUSPENDED | CLIENT_EXIT;
pub const INPUT_BUF_DEFAULT_SIZE: ::core::ffi::c_int = 1048576 as ::core::ffi::c_int;
pub const INPUT_REQUEST_TIMEOUT: ::core::ffi::c_int = 500 as ::core::ffi::c_int;
pub const INPUT_BUF_START: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const INPUT_DISCARD: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const INPUT_LAST: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
static input_esc_table: [input_table_entry; 15] = [
    input_table_entry {
        ch: '0' as i32,
        interm: c"(",
        type_0: INPUT_ESC_SCSG0_ON as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: '0' as i32,
        interm: c")",
        type_0: INPUT_ESC_SCSG1_ON as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: '7' as i32,
        interm: c"",
        type_0: INPUT_ESC_DECSC as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: '8' as i32,
        interm: c"",
        type_0: INPUT_ESC_DECRC as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: '8' as i32,
        interm: c"#",
        type_0: INPUT_ESC_DECALN as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: '=' as i32,
        interm: c"",
        type_0: INPUT_ESC_DECKPAM as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: '>' as i32,
        interm: c"",
        type_0: INPUT_ESC_DECKPNM as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'B' as i32,
        interm: c"(",
        type_0: INPUT_ESC_SCSG0_OFF as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'B' as i32,
        interm: c")",
        type_0: INPUT_ESC_SCSG1_OFF as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'D' as i32,
        interm: c"",
        type_0: INPUT_ESC_IND as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'E' as i32,
        interm: c"",
        type_0: INPUT_ESC_NEL as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'H' as i32,
        interm: c"",
        type_0: INPUT_ESC_HTS as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'M' as i32,
        interm: c"",
        type_0: INPUT_ESC_RI as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: '\\' as i32,
        interm: c"",
        type_0: INPUT_ESC_ST as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'c' as i32,
        interm: c"",
        type_0: INPUT_ESC_RIS as ::core::ffi::c_int,
    },
];
static input_csi_table: [input_table_entry; 43] = [
    input_table_entry {
        ch: '@' as i32,
        interm: c"",
        type_0: INPUT_CSI_ICH as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'A' as i32,
        interm: c"",
        type_0: INPUT_CSI_CUU as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'B' as i32,
        interm: c"",
        type_0: INPUT_CSI_CUD as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'C' as i32,
        interm: c"",
        type_0: INPUT_CSI_CUF as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'D' as i32,
        interm: c"",
        type_0: INPUT_CSI_CUB as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'E' as i32,
        interm: c"",
        type_0: INPUT_CSI_CNL as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'F' as i32,
        interm: c"",
        type_0: INPUT_CSI_CPL as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'G' as i32,
        interm: c"",
        type_0: INPUT_CSI_HPA as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'H' as i32,
        interm: c"",
        type_0: INPUT_CSI_CUP as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'J' as i32,
        interm: c"",
        type_0: INPUT_CSI_ED as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'K' as i32,
        interm: c"",
        type_0: INPUT_CSI_EL as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'L' as i32,
        interm: c"",
        type_0: INPUT_CSI_IL as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'M' as i32,
        interm: c"",
        type_0: INPUT_CSI_DL as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'P' as i32,
        interm: c"",
        type_0: INPUT_CSI_DCH as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'S' as i32,
        interm: c"",
        type_0: INPUT_CSI_SU as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'S' as i32,
        interm: c"?",
        type_0: INPUT_CSI_SM_GRAPHICS as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'T' as i32,
        interm: c"",
        type_0: INPUT_CSI_SD as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'X' as i32,
        interm: c"",
        type_0: INPUT_CSI_ECH as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'Z' as i32,
        interm: c"",
        type_0: INPUT_CSI_CBT as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: '`' as i32,
        interm: c"",
        type_0: INPUT_CSI_HPA as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'b' as i32,
        interm: c"",
        type_0: INPUT_CSI_REP as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'c' as i32,
        interm: c"",
        type_0: INPUT_CSI_DA as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'c' as i32,
        interm: c">",
        type_0: INPUT_CSI_DA_TWO as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'd' as i32,
        interm: c"",
        type_0: INPUT_CSI_VPA as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'f' as i32,
        interm: c"",
        type_0: INPUT_CSI_CUP as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'g' as i32,
        interm: c"",
        type_0: INPUT_CSI_TBC as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'h' as i32,
        interm: c"",
        type_0: INPUT_CSI_SM as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'h' as i32,
        interm: c"?",
        type_0: INPUT_CSI_SM_PRIVATE as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'l' as i32,
        interm: c"",
        type_0: INPUT_CSI_RM as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'l' as i32,
        interm: c"?",
        type_0: INPUT_CSI_RM_PRIVATE as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'm' as i32,
        interm: c"",
        type_0: INPUT_CSI_SGR as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'm' as i32,
        interm: c">",
        type_0: INPUT_CSI_MODSET as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'n' as i32,
        interm: c"",
        type_0: INPUT_CSI_DSR as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'n' as i32,
        interm: c">",
        type_0: INPUT_CSI_MODOFF as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'n' as i32,
        interm: c"?",
        type_0: INPUT_CSI_DSR_PRIVATE as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'p' as i32,
        interm: c"$",
        type_0: INPUT_CSI_QUERY as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'p' as i32,
        interm: c"?$",
        type_0: INPUT_CSI_QUERY_PRIVATE as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'q' as i32,
        interm: c" ",
        type_0: INPUT_CSI_DECSCUSR as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'q' as i32,
        interm: c">",
        type_0: INPUT_CSI_XDA as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'r' as i32,
        interm: c"",
        type_0: INPUT_CSI_DECSTBM as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 's' as i32,
        interm: c"",
        type_0: INPUT_CSI_SCP as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 't' as i32,
        interm: c"",
        type_0: INPUT_CSI_WINOPS as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'u' as i32,
        interm: c"",
        type_0: INPUT_CSI_RCP as ::core::ffi::c_int,
    },
];
pub(crate) static input_state_ground: input_state = {
    input_state {
        name: c"ground",
        enter: Some(input_ground),
        exit: None,
        transitions: &input_state_ground_table,
    }
};
static input_state_esc_enter: input_state = {
    input_state {
        name: c"esc_enter",
        enter: Some(input_clear),
        exit: None,
        transitions: &input_state_esc_enter_table,
    }
};
static input_state_esc_intermediate: input_state = {
    input_state {
        name: c"esc_intermediate",
        enter: None,
        exit: None,
        transitions: &input_state_esc_intermediate_table,
    }
};
static input_state_csi_enter: input_state = {
    input_state {
        name: c"csi_enter",
        enter: Some(input_clear),
        exit: None,
        transitions: &input_state_csi_enter_table,
    }
};
static input_state_csi_parameter: input_state = {
    input_state {
        name: c"csi_parameter",
        enter: None,
        exit: None,
        transitions: &input_state_csi_parameter_table,
    }
};
static input_state_csi_intermediate: input_state = {
    input_state {
        name: c"csi_intermediate",
        enter: None,
        exit: None,
        transitions: &input_state_csi_intermediate_table,
    }
};
static input_state_csi_ignore: input_state = {
    input_state {
        name: c"csi_ignore",
        enter: None,
        exit: None,
        transitions: &input_state_csi_ignore_table,
    }
};
static input_state_dcs_enter: input_state = {
    input_state {
        name: c"dcs_enter",
        enter: Some(input_enter_dcs),
        exit: None,
        transitions: &input_state_dcs_enter_table,
    }
};
static input_state_dcs_parameter: input_state = {
    input_state {
        name: c"dcs_parameter",
        enter: None,
        exit: None,
        transitions: &input_state_dcs_parameter_table,
    }
};
static input_state_dcs_intermediate: input_state = {
    input_state {
        name: c"dcs_intermediate",
        enter: None,
        exit: None,
        transitions: &input_state_dcs_intermediate_table,
    }
};
static input_state_dcs_handler: input_state = {
    input_state {
        name: c"dcs_handler",
        enter: None,
        exit: None,
        transitions: &input_state_dcs_handler_table,
    }
};
static input_state_dcs_escape: input_state = {
    input_state {
        name: c"dcs_escape",
        enter: None,
        exit: None,
        transitions: &input_state_dcs_escape_table,
    }
};
static input_state_dcs_ignore: input_state = {
    input_state {
        name: c"dcs_ignore",
        enter: None,
        exit: None,
        transitions: &input_state_dcs_ignore_table,
    }
};
static input_state_osc_string: input_state = {
    input_state {
        name: c"osc_string",
        enter: Some(input_enter_osc),
        exit: Some(input_exit_osc),
        transitions: &input_state_osc_string_table,
    }
};
static input_state_apc_string: input_state = {
    input_state {
        name: c"apc_string",
        enter: Some(input_enter_apc),
        exit: Some(input_exit_apc),
        transitions: &input_state_apc_string_table,
    }
};
static input_state_rename_string: input_state = {
    input_state {
        name: c"rename_string",
        enter: Some(input_enter_rename),
        exit: Some(input_exit_rename),
        transitions: &input_state_rename_string_table,
    }
};
static input_state_consume_st: input_state = {
    input_state {
        name: c"consume_st",
        enter: Some(input_enter_rename),
        exit: None,
        transitions: &input_state_consume_st_table,
    }
};
static input_state_ground_table: [input_transition; 9] = {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: Some(&input_state_esc_enter),
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0x7e as ::core::ffi::c_int,
            handler: Some(InputHandler::Print),
            state: None,
        },
        input_transition {
            first: 0x7f as ::core::ffi::c_int,
            last: 0x7f as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x80 as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: Some(InputHandler::TopBitSet),
            state: None,
        },
    ]
};
static input_state_esc_enter_table: [input_transition; 22] = {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: Some(&input_state_esc_enter),
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0x2f as ::core::ffi::c_int,
            handler: Some(InputHandler::Intermediate),
            state: Some(&input_state_esc_intermediate),
        },
        input_transition {
            first: 0x30 as ::core::ffi::c_int,
            last: 0x4f as ::core::ffi::c_int,
            handler: Some(InputHandler::EscDispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x50 as ::core::ffi::c_int,
            last: 0x50 as ::core::ffi::c_int,
            handler: None,
            state: Some(&input_state_dcs_enter),
        },
        input_transition {
            first: 0x51 as ::core::ffi::c_int,
            last: 0x57 as ::core::ffi::c_int,
            handler: Some(InputHandler::EscDispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x58 as ::core::ffi::c_int,
            last: 0x58 as ::core::ffi::c_int,
            handler: None,
            state: Some(&input_state_consume_st),
        },
        input_transition {
            first: 0x59 as ::core::ffi::c_int,
            last: 0x59 as ::core::ffi::c_int,
            handler: Some(InputHandler::EscDispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x5a as ::core::ffi::c_int,
            last: 0x5a as ::core::ffi::c_int,
            handler: Some(InputHandler::EscDispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x5b as ::core::ffi::c_int,
            last: 0x5b as ::core::ffi::c_int,
            handler: None,
            state: Some(&input_state_csi_enter),
        },
        input_transition {
            first: 0x5c as ::core::ffi::c_int,
            last: 0x5c as ::core::ffi::c_int,
            handler: Some(InputHandler::EscDispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x5d as ::core::ffi::c_int,
            last: 0x5d as ::core::ffi::c_int,
            handler: None,
            state: Some(&input_state_osc_string),
        },
        input_transition {
            first: 0x5e as ::core::ffi::c_int,
            last: 0x5e as ::core::ffi::c_int,
            handler: None,
            state: Some(&input_state_consume_st),
        },
        input_transition {
            first: 0x5f as ::core::ffi::c_int,
            last: 0x5f as ::core::ffi::c_int,
            handler: None,
            state: Some(&input_state_apc_string),
        },
        input_transition {
            first: 0x60 as ::core::ffi::c_int,
            last: 0x6a as ::core::ffi::c_int,
            handler: Some(InputHandler::EscDispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x6b as ::core::ffi::c_int,
            last: 0x6b as ::core::ffi::c_int,
            handler: None,
            state: Some(&input_state_rename_string),
        },
        input_transition {
            first: 0x6c as ::core::ffi::c_int,
            last: 0x7e as ::core::ffi::c_int,
            handler: Some(InputHandler::EscDispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x7f as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
    ]
};
static input_state_esc_intermediate_table: [input_transition; 9] = {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: Some(&input_state_esc_enter),
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0x2f as ::core::ffi::c_int,
            handler: Some(InputHandler::Intermediate),
            state: None,
        },
        input_transition {
            first: 0x30 as ::core::ffi::c_int,
            last: 0x7e as ::core::ffi::c_int,
            handler: Some(InputHandler::EscDispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x7f as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
    ]
};
static input_state_csi_enter_table: [input_transition; 13] = {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: Some(&input_state_esc_enter),
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0x2f as ::core::ffi::c_int,
            handler: Some(InputHandler::Intermediate),
            state: Some(&input_state_csi_intermediate),
        },
        input_transition {
            first: 0x30 as ::core::ffi::c_int,
            last: 0x39 as ::core::ffi::c_int,
            handler: Some(InputHandler::Parameter),
            state: Some(&input_state_csi_parameter),
        },
        input_transition {
            first: 0x3a as ::core::ffi::c_int,
            last: 0x3a as ::core::ffi::c_int,
            handler: Some(InputHandler::Parameter),
            state: Some(&input_state_csi_parameter),
        },
        input_transition {
            first: 0x3b as ::core::ffi::c_int,
            last: 0x3b as ::core::ffi::c_int,
            handler: Some(InputHandler::Parameter),
            state: Some(&input_state_csi_parameter),
        },
        input_transition {
            first: 0x3c as ::core::ffi::c_int,
            last: 0x3f as ::core::ffi::c_int,
            handler: Some(InputHandler::Intermediate),
            state: Some(&input_state_csi_parameter),
        },
        input_transition {
            first: 0x40 as ::core::ffi::c_int,
            last: 0x7e as ::core::ffi::c_int,
            handler: Some(InputHandler::CsiDispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x7f as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
    ]
};
static input_state_csi_parameter_table: [input_transition; 13] = {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: Some(&input_state_esc_enter),
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0x2f as ::core::ffi::c_int,
            handler: Some(InputHandler::Intermediate),
            state: Some(&input_state_csi_intermediate),
        },
        input_transition {
            first: 0x30 as ::core::ffi::c_int,
            last: 0x39 as ::core::ffi::c_int,
            handler: Some(InputHandler::Parameter),
            state: None,
        },
        input_transition {
            first: 0x3a as ::core::ffi::c_int,
            last: 0x3a as ::core::ffi::c_int,
            handler: Some(InputHandler::Parameter),
            state: None,
        },
        input_transition {
            first: 0x3b as ::core::ffi::c_int,
            last: 0x3b as ::core::ffi::c_int,
            handler: Some(InputHandler::Parameter),
            state: None,
        },
        input_transition {
            first: 0x3c as ::core::ffi::c_int,
            last: 0x3f as ::core::ffi::c_int,
            handler: None,
            state: Some(&input_state_csi_ignore),
        },
        input_transition {
            first: 0x40 as ::core::ffi::c_int,
            last: 0x7e as ::core::ffi::c_int,
            handler: Some(InputHandler::CsiDispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x7f as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
    ]
};
static input_state_csi_intermediate_table: [input_transition; 10] = {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: Some(&input_state_esc_enter),
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0x2f as ::core::ffi::c_int,
            handler: Some(InputHandler::Intermediate),
            state: None,
        },
        input_transition {
            first: 0x30 as ::core::ffi::c_int,
            last: 0x3f as ::core::ffi::c_int,
            handler: None,
            state: Some(&input_state_csi_ignore),
        },
        input_transition {
            first: 0x40 as ::core::ffi::c_int,
            last: 0x7e as ::core::ffi::c_int,
            handler: Some(InputHandler::CsiDispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x7f as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
    ]
};
static input_state_csi_ignore_table: [input_transition; 9] = {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: Some(&input_state_esc_enter),
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0x3f as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x40 as ::core::ffi::c_int,
            last: 0x7e as ::core::ffi::c_int,
            handler: None,
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x7f as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
    ]
};
static input_state_dcs_enter_table: [input_transition; 13] = {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: Some(&input_state_esc_enter),
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0x2f as ::core::ffi::c_int,
            handler: Some(InputHandler::Intermediate),
            state: Some(&input_state_dcs_intermediate),
        },
        input_transition {
            first: 0x30 as ::core::ffi::c_int,
            last: 0x39 as ::core::ffi::c_int,
            handler: Some(InputHandler::Parameter),
            state: Some(&input_state_dcs_parameter),
        },
        input_transition {
            first: 0x3a as ::core::ffi::c_int,
            last: 0x3a as ::core::ffi::c_int,
            handler: None,
            state: Some(&input_state_dcs_ignore),
        },
        input_transition {
            first: 0x3b as ::core::ffi::c_int,
            last: 0x3b as ::core::ffi::c_int,
            handler: Some(InputHandler::Parameter),
            state: Some(&input_state_dcs_parameter),
        },
        input_transition {
            first: 0x3c as ::core::ffi::c_int,
            last: 0x3f as ::core::ffi::c_int,
            handler: Some(InputHandler::Intermediate),
            state: Some(&input_state_dcs_parameter),
        },
        input_transition {
            first: 0x40 as ::core::ffi::c_int,
            last: 0x7e as ::core::ffi::c_int,
            handler: Some(InputHandler::Input),
            state: Some(&input_state_dcs_handler),
        },
        input_transition {
            first: 0x7f as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
    ]
};
static input_state_dcs_parameter_table: [input_transition; 13] = {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: Some(&input_state_esc_enter),
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0x2f as ::core::ffi::c_int,
            handler: Some(InputHandler::Intermediate),
            state: Some(&input_state_dcs_intermediate),
        },
        input_transition {
            first: 0x30 as ::core::ffi::c_int,
            last: 0x39 as ::core::ffi::c_int,
            handler: Some(InputHandler::Parameter),
            state: None,
        },
        input_transition {
            first: 0x3a as ::core::ffi::c_int,
            last: 0x3a as ::core::ffi::c_int,
            handler: None,
            state: Some(&input_state_dcs_ignore),
        },
        input_transition {
            first: 0x3b as ::core::ffi::c_int,
            last: 0x3b as ::core::ffi::c_int,
            handler: Some(InputHandler::Parameter),
            state: None,
        },
        input_transition {
            first: 0x3c as ::core::ffi::c_int,
            last: 0x3f as ::core::ffi::c_int,
            handler: None,
            state: Some(&input_state_dcs_ignore),
        },
        input_transition {
            first: 0x40 as ::core::ffi::c_int,
            last: 0x7e as ::core::ffi::c_int,
            handler: Some(InputHandler::Input),
            state: Some(&input_state_dcs_handler),
        },
        input_transition {
            first: 0x7f as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
    ]
};
static input_state_dcs_intermediate_table: [input_transition; 10] = {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: Some(&input_state_esc_enter),
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0x2f as ::core::ffi::c_int,
            handler: Some(InputHandler::Intermediate),
            state: None,
        },
        input_transition {
            first: 0x30 as ::core::ffi::c_int,
            last: 0x3f as ::core::ffi::c_int,
            handler: None,
            state: Some(&input_state_dcs_ignore),
        },
        input_transition {
            first: 0x40 as ::core::ffi::c_int,
            last: 0x7e as ::core::ffi::c_int,
            handler: Some(InputHandler::Input),
            state: Some(&input_state_dcs_handler),
        },
        input_transition {
            first: 0x7f as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
    ]
};
static input_state_dcs_handler_table: [input_transition; 3] = {
    [
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(InputHandler::Input),
            state: None,
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: Some(&input_state_dcs_escape),
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: Some(InputHandler::Input),
            state: None,
        },
    ]
};
static input_state_dcs_escape_table: [input_transition; 3] = {
    [
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x5b as ::core::ffi::c_int,
            handler: Some(InputHandler::Input),
            state: Some(&input_state_dcs_handler),
        },
        input_transition {
            first: 0x5c as ::core::ffi::c_int,
            last: 0x5c as ::core::ffi::c_int,
            handler: Some(InputHandler::DcsDispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x5d as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: Some(InputHandler::Input),
            state: Some(&input_state_dcs_handler),
        },
    ]
};
static input_state_dcs_ignore_table: [input_transition; 7] = {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: Some(&input_state_esc_enter),
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
    ]
};
static input_state_osc_string_table: [input_transition; 9] = {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: Some(&input_state_esc_enter),
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x6 as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x7 as ::core::ffi::c_int,
            last: 0x7 as ::core::ffi::c_int,
            handler: Some(InputHandler::EndBel),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x8 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: Some(InputHandler::Input),
            state: None,
        },
    ]
};
static input_state_apc_string_table: [input_transition; 7] = {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: Some(&input_state_esc_enter),
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: Some(InputHandler::Input),
            state: None,
        },
    ]
};
static input_state_rename_string_table: [input_transition; 7] = {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: Some(&input_state_esc_enter),
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: Some(InputHandler::Input),
            state: None,
        },
    ]
};
static input_state_consume_st_table: [input_transition; 7] = {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: Some(&input_state_esc_enter),
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: None,
        },
    ]
};
static mut input_buffer_size: size_t = INPUT_BUF_DEFAULT_SIZE as size_t;
/// The entry of `table` the parser's character and intermediates name.
unsafe fn input_table_find(
    ictx: *const input_ctx,
    table: &'static [input_table_entry],
) -> Option<&'static input_table_entry> {
    unsafe {
        table
            .binary_search_by(|entry| input_table_compare(ictx, entry).cmp(&0).reverse())
            .ok()
            .map(|at| &table[at])
    }
}
unsafe fn input_table_compare(
    mut key: *const input_ctx,
    mut value: *const input_table_entry,
) -> ::core::ffi::c_int {
    unsafe {
        let mut ictx: *const input_ctx = key;
        let mut entry: *const input_table_entry = value;
        if (*ictx).ch != (*entry).ch {
            return (*ictx).ch - (*entry).ch;
        }
        strcmp(
            &raw const (*ictx).interm_buf as *const u_char as *const ::core::ffi::c_char,
            (*entry).interm.as_ptr(),
        )
    }
}
unsafe fn input_stop_utf8(mut ictx: *mut input_ctx) {
    unsafe {
        let sctx: &mut screen_write_ctx = &mut (*ictx).ctx;
        static mut rc: utf8_data = unsafe {
            utf8_data {
                data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                    *b"\xEF\xBF\xBD\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                have: 3 as u_char,
                size: 3 as u_char,
                width: 1 as u_char,
            }
        };
        if (*ictx).utf8started != 0 {
            utf8_copy(&mut (*ictx).cell.cell.data, &rc);
            screen_write_collect_add(sctx, &mut (*ictx).cell.cell);
        }
        (*ictx).utf8started = 0 as ::core::ffi::c_int;
    }
}
unsafe fn input_ground_timer_callback(ictx: *mut input_ctx) {
    unsafe {
        log_debug(
            c"%s: %s expired".as_ptr(),
            fmt_args![
                c"input_ground_timer_callback".as_ptr(),
                (*ictx).state.name.as_ptr()
            ],
        );
        input_reset(&mut *ictx, 0 as ::core::ffi::c_int);
    }
}
unsafe fn input_start_ground_timer(mut ictx: *mut input_ctx) {
    unsafe {
        let mut tv = timeval::from_secs(5 as __time_t);
        (*ictx).ground_timer.disarm();
        (*ictx).ground_timer.arm(tv);
    }
}
unsafe fn input_reset_cell(mut ictx: *mut input_ctx) {
    unsafe {
        (*ictx).cell.cell = grid_default_cell;
        (*ictx).cell.set = 0 as ::core::ffi::c_int;
        (*ictx).cell.g1set = 0 as ::core::ffi::c_int;
        (*ictx).cell.g0set = (*ictx).cell.g1set;
        (*ictx).old_cell = (*ictx).cell;
        (*ictx).old_cx = 0 as u_int;
        (*ictx).old_cy = 0 as u_int;
    }
}
unsafe fn input_save_state(mut ictx: *mut input_ctx) {
    unsafe {
        let sctx: &mut screen_write_ctx = &mut (*ictx).ctx;
        let mut s: *mut screen = sctx.s;
        (*ictx).old_cell = (*ictx).cell;
        (*ictx).old_cx = (*s).cx;
        (*ictx).old_cy = (*s).cy;
        (*ictx).old_mode = (*s).mode;
    }
}
unsafe fn input_restore_state(mut ictx: *mut input_ctx) {
    unsafe {
        let sctx: &mut screen_write_ctx = &mut (*ictx).ctx;
        (*ictx).cell = (*ictx).old_cell;
        if (*ictx).old_mode & MODE_ORIGIN != 0 {
            screen_write_mode_set(sctx, MODE_ORIGIN);
        } else {
            screen_write_mode_clear(sctx, MODE_ORIGIN);
        }
        screen_write_cursormove(
            sctx,
            (*ictx).old_cx as ::core::ffi::c_int,
            (*ictx).old_cy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
    }
}
/// The parser an owner feeds its output through. One is made when the owner
/// is, so the code that writes through it always has one.
/// What a parser is parsing output for: a pane, a popup, or nothing at all —
/// the parser copy mode drives by hand has no owner.
#[derive(Clone, Default)]
pub enum InputOwner {
    #[default]
    Detached,
    /// The pane whose output it parses, named by the pane's id.
    Pane(u_int),
    /// The popup whose output it parses, and the client that popup is on.
    Popup(crate::overlay::PopupDataWeak, Option<ClientWeak>),
}

impl input_ctx {
    /// The pane the parser is parsing for, or null when it parses for none.
    pub fn pane(&self) -> *mut window_pane {
        match &self.owner_of {
            InputOwner::Pane(id) => crate::window::window_pane_find_by_id(*id),
            _ => ::core::ptr::null_mut(),
        }
    }

    /// The palette the parser reads its colours out of, or null when its
    /// owner has gone or it has none.
    pub fn palette(&self) -> *mut colour_palette {
        unsafe {
            match &self.owner_of {
                InputOwner::Detached => ::core::ptr::null_mut(),
                InputOwner::Pane(_) => match self.pane() {
                    wp if wp.is_null() => ::core::ptr::null_mut(),
                    wp => &raw mut (*wp).palette,
                },
                InputOwner::Popup(held, _) => match held.upgrade() {
                    Some(held) => &raw mut (*held.as_ptr()).palette,
                    None => ::core::ptr::null_mut(),
                },
            }
        }
    }

    /// The client the parser answers to, or null when it answers to none.
    pub fn client(&self) -> *mut client {
        match &self.owner_of {
            InputOwner::Popup(_, Some(held)) => held
                .upgrade()
                .map_or(::core::ptr::null_mut(), |c| c.as_ptr()),
            _ => ::core::ptr::null_mut(),
        }
    }
}

/// A strong owner of a parser. The raw pointer from [`InputCtxRef::as_ptr`]
/// is only a borrowed compatibility view; the handle must remain alive for
/// every use of that pointer.
#[derive(Clone)]
pub struct InputCtxRef(Rc<UnsafeCell<input_ctx>>);

/// A non-owning observation of a parser. The parser's own timers hold it this
/// way, so one that fires after its owner has given the parser up finds
/// nothing rather than a freed parser.
#[derive(Clone)]
pub struct InputCtxWeak(Weak<UnsafeCell<input_ctx>>);

impl InputCtxRef {
    pub(crate) fn new(value: input_ctx) -> Self {
        let reference = Self(Rc::new(UnsafeCell::new(value)));
        unsafe { (*reference.as_ptr()).owner = Some(reference.downgrade()) };
        reference
    }

    /// Returns a temporary raw view while this strong handle remains alive.
    pub(crate) fn as_ptr(&self) -> *mut input_ctx {
        self.0.get()
    }

    /// Makes a non-owning observation of this parser.
    pub(crate) fn downgrade(&self) -> InputCtxWeak {
        InputCtxWeak(Rc::downgrade(&self.0))
    }
}

impl InputCtxWeak {
    /// Upgrades the observation if the parser's owner still holds it.
    pub(crate) fn upgrade(&self) -> Option<InputCtxRef> {
        self.0.upgrade().map(InputCtxRef)
    }
}

pub fn ictx_mut(value: &mut Option<InputCtxRef>) -> &mut input_ctx {
    unsafe {
        &mut *value
            .as_ref()
            .expect("an input owner has a parser")
            .as_ptr()
    }
}

/// The parser an owner holds, if it has opened one.
pub fn ictx_opt(value: &Option<InputCtxRef>) -> Option<*mut input_ctx> {
    value.as_ref().map(InputCtxRef::as_ptr)
}

pub unsafe fn input_init(owner_of: InputOwner, mut bev: Stream) -> InputCtxRef {
    unsafe {
        let mut input_buf = Vec::with_capacity(INPUT_BUF_START as usize);
        input_buf.push(b'\0');
        let since_ground = Box::new(Buf::new());
        let ictx_box = InputCtxRef::new(input_ctx {
            owner_of,
            event: bev,
            ctx: screen_write_ctx::default(),
            cell: input_cell {
                cell: grid_default_cell,
                set: 0,
                g0set: 0,
                g1set: 0,
            },
            old_cell: input_cell {
                cell: grid_default_cell,
                set: 0,
                g0set: 0,
                g1set: 0,
            },
            old_cx: 0,
            old_cy: 0,
            old_mode: 0,
            interm_buf: [0; 4],
            interm_len: 0,
            param_buf: [0; 64],
            param_len: 0,
            input_buf,
            input_end: INPUT_END_ST,
            param_list: [const { InputParam::Missing }; 24],
            param_list_len: 0,
            utf8data: utf8_data::default(),
            utf8started: 0,
            ch: 0,
            last: utf8_data::default(),
            state: &input_state_ground,
            flags: 0,
            requests: input_request_list::new(),
            request_count: 0,
            request_timer: TimerHandle(0),
            since_ground: Some(since_ground),
            ground_timer: TimerHandle(0),
            owner: None,
        });
        let ictx = ictx_box.as_ptr();
        let watching = ictx_box.downgrade();
        (*ictx).ground_timer.set_callback({
            let watching = watching.clone();
            move || {
                if let Some(ictx) = watching.upgrade() {
                    input_ground_timer_callback(ictx.as_ptr());
                }
            }
        });
        (*ictx).request_timer.set_callback(move || {
            if let Some(ictx) = watching.upgrade() {
                input_request_timer_callback(ictx.as_ptr());
            }
        });
        input_reset(&mut *ictx, 0 as ::core::ffi::c_int);
        ictx_box
    }
}

pub(crate) unsafe fn input_free_box(reference: InputCtxRef) {
    unsafe {
        let ictx = &mut *reference.as_ptr();
        for ir in foreach_owned_safe(&raw mut ictx.requests) {
            input_free_request(ir);
        }
        ictx.requests.clear();
        ictx.request_timer.disarm();
        ictx.ground_timer.disarm();
        screen_write_stop_sync(ictx.pane());
        drop(reference);
    }
}

pub unsafe fn input_reset(ictx: &mut input_ctx, mut clear: ::core::ffi::c_int) {
    unsafe {
        let ictx: *mut input_ctx = ictx;
        let sctx: &mut screen_write_ctx = &mut (*ictx).ctx;
        let mut wp: *mut window_pane = (*ictx).pane();
        input_reset_cell(ictx);
        if clear != 0 && !wp.is_null() {
            if (*wp).modes.is_empty() {
                screen_write_start_pane(sctx, wp, Some(&mut (*wp).base));
            } else {
                screen_write_start(sctx, &mut (*wp).base);
            }
            screen_write_reset(sctx);
            screen_write_stop(sctx);
        }
        input_clear(ictx);
        (*ictx).state = &input_state_ground;
        (*ictx).flags = 0 as ::core::ffi::c_int;
    }
}
pub fn input_pending(ictx: &mut input_ctx) -> *mut Buf {
    ictx.since_ground
        .as_deref_mut()
        .map_or(::core::ptr::null_mut(), |buf| buf)
}
unsafe fn input_set_state(mut ictx: *mut input_ctx, state: &'static input_state) {
    unsafe {
        if let Some(exit) = (*ictx).state.exit {
            exit(ictx);
        }
        (*ictx).state = state;
        if let Some(enter) = state.enter {
            enter(ictx);
        }
    }
}
unsafe fn input_parse(mut ictx: *mut input_ctx, mut buf: *const u_char, mut len: size_t) {
    unsafe {
        let sctx: &mut screen_write_ctx = &mut (*ictx).ctx;
        let mut state: *const input_state = ::core::ptr::null::<input_state>();
        let mut itr: Option<&'static input_transition> = None;
        let mut off: size_t = 0 as size_t;
        while off < len {
            let fresh16 = off;
            off = off.wrapping_add(1);
            (*ictx).ch = *buf.add(fresh16) as ::core::ffi::c_int;
            let ch = (*ictx).ch;
            let holds = |itr: &input_transition| ch >= itr.first && ch <= itr.last;
            if !::core::ptr::eq((*ictx).state, state) || !itr.is_some_and(&holds) {
                itr = (*ictx).state.transitions.iter().find(|itr| holds(itr));
            }
            let Some(itr) = itr else {
                fatalx(c"no transition from state".as_ptr(), fmt_args![]);
            };
            state = (*ictx).state;
            if itr.handler != Some(InputHandler::Print) {
                screen_write_collect_end(sctx);
            }
            if let Some(handler) = itr.handler
                && handler.call(ictx) != 0 as ::core::ffi::c_int
            {
                continue;
            }
            if let Some(state) = itr.state {
                input_set_state(ictx, state);
            }
            if !::core::ptr::eq((*ictx).state, &input_state_ground)
                && let Some(buf) = (*ictx).since_ground.as_deref_mut()
            {
                buf.append(::core::slice::from_raw_parts(
                    &raw mut (*ictx).ch as *const u8,
                    1,
                ));
            }
        }
    }
}
pub unsafe fn input_parse_pane(mut wp: *mut window_pane) {
    unsafe {
        let mut new_data: *const u_char = ::core::ptr::null::<u_char>();
        let mut new_size: size_t = 0;
        (new_data, new_size) = window_pane_get_new_data(wp, &(*wp).offset);
        input_parse_buffer(wp, new_data, new_size);
        window_pane_update_used_data(wp, &mut (*wp).offset, new_size);
    }
}
pub unsafe fn input_parse_buffer(
    mut wp: *mut window_pane,
    mut buf: *const u_char,
    mut len: size_t,
) {
    unsafe {
        let ictx: *mut input_ctx = ictx_opt(&(*wp).ictx).expect("a pane being parsed has a parser");
        let sctx: &mut screen_write_ctx = &mut (*ictx).ctx;
        if len == 0 as size_t {
            return;
        }
        window_update_activity((*wp).window);
        crate::plugin::note_pane_output((*wp).id);
        (*wp).flags |= PANE_CHANGED;
        if !(*wp).modes.is_empty() {
            (*wp).flags |= PANE_UNSEENCHANGES;
        }
        if (*wp).modes.is_empty() {
            screen_write_start_pane(sctx, wp, Some(&mut (*wp).base));
        } else {
            screen_write_start(sctx, &mut (*wp).base);
        }
        log_debug(
            c"%s: %%%u %s, %zu bytes: %.*s".as_ptr(),
            fmt_args![
                c"input_parse_buffer".as_ptr(),
                (*wp).id,
                (*ictx).state.name.as_ptr(),
                len,
                len as ::core::ffi::c_int,
                buf
            ],
        );
        input_parse(ictx, buf, len);
        screen_write_stop(sctx);
    }
}
pub unsafe fn input_parse_screen(
    ictx: &mut input_ctx,
    mut s: *mut screen,
    mut cb: screen_write_init_ctx_cb,
    mut arg: *mut popup_data,
    mut buf: *const u_char,
    mut len: size_t,
) {
    unsafe {
        let ictx: *mut input_ctx = ictx;
        let sctx: &mut screen_write_ctx = &mut (*ictx).ctx;
        if len == 0 as size_t {
            return;
        }
        screen_write_start_callback(sctx, s, cb, arg);
        input_parse(ictx, buf, len);
        screen_write_stop(sctx);
    }
}
/// The parameters the parser collected for the sequence in hand.
unsafe fn input_params<'a>(ictx: *mut input_ctx) -> &'a mut [InputParam; 24] {
    unsafe { &mut (*ictx).param_list }
}
unsafe fn input_split(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    unsafe {
        let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut i: u_int = 0;
        i = 0 as u_int;
        while i < (*ictx).param_list_len {
            input_params(ictx)[i as usize] = InputParam::Missing;
            i = i.wrapping_add(1);
        }
        (*ictx).param_list_len = 0 as u_int;
        if (*ictx).param_len == 0 as size_t {
            return 0 as ::core::ffi::c_int;
        }
        ptr = &raw mut (*ictx).param_buf as *mut u_char as *mut ::core::ffi::c_char;
        loop {
            out = strsep(&raw mut ptr, c";".as_ptr());
            if out.is_null() {
                break;
            }
            let param = if *out as ::core::ffi::c_int == '\0' as i32 {
                InputParam::Missing
            } else if !strchr(out, ':' as i32).is_null() {
                InputParam::Str(CStr::from_ptr(out).to_owned())
            } else {
                let Ok(n) = strtonum(
                    out,
                    0 as ::core::ffi::c_longlong,
                    INT_MAX as ::core::ffi::c_longlong,
                ) else {
                    return -(1 as ::core::ffi::c_int);
                };
                InputParam::Number(n as ::core::ffi::c_int)
            };
            input_params(ictx)[(*ictx).param_list_len as usize] = param;
            (*ictx).param_list_len = (*ictx).param_list_len.wrapping_add(1);
            if (*ictx).param_list_len as usize == input_params(ictx).len() {
                return -(1 as ::core::ffi::c_int);
            }
        }
        i = 0 as u_int;
        while i < (*ictx).param_list_len {
            match &input_params(ictx)[i as usize] {
                InputParam::Missing => {
                    log_debug(c"parameter %u: missing".as_ptr(), fmt_args![i]);
                }
                InputParam::Str(value) => {
                    log_debug(
                        c"parameter %u: string %s".as_ptr(),
                        fmt_args![i, value.as_ptr()],
                    );
                }
                InputParam::Number(n) => {
                    log_debug(c"parameter %u: number %d".as_ptr(), fmt_args![i, *n]);
                }
            }
            i = i.wrapping_add(1);
        }
        0 as ::core::ffi::c_int
    }
}
unsafe fn input_get(
    mut ictx: *mut input_ctx,
    mut validx: u_int,
    mut minval: ::core::ffi::c_int,
    mut defval: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut retval: ::core::ffi::c_int = 0;
        if validx >= (*ictx).param_list_len {
            return defval;
        }
        match &input_params(ictx)[validx as usize] {
            InputParam::Missing => return defval,
            InputParam::Str(_) => return -(1 as ::core::ffi::c_int),
            InputParam::Number(n) => retval = *n,
        }
        if retval < minval {
            return minval;
        }
        retval
    }
}
unsafe fn input_send_reply(mut ictx: *mut input_ctx, reply: &CStr) {
    unsafe {
        if !(*ictx).event.is_none() {
            log_debug(
                c"%s: %s".as_ptr(),
                fmt_args![c"input_send_reply".as_ptr(), reply.as_ptr()],
            );
            (*ictx)
                .event
                .write(reply.as_ptr() as *const u8, reply.to_bytes().len());
        }
    }
}
unsafe fn input_reply(
    mut ictx: *mut input_ctx,
    mut add: ::core::ffi::c_int,
    mut fmt: *const ::core::ffi::c_char,
    args: &[FmtArg],
) {
    unsafe {
        let mut ir: *mut input_request = ::core::ptr::null_mut::<input_request>();
        let reply = format_alloc(fmt, args);
        if add != 0 && !(*ictx).requests.is_empty() {
            ir = input_make_request(ictx, INPUT_REQUEST_QUEUE);
            (*ir).data = Some(reply);
        } else {
            input_send_reply(ictx, &reply);
        };
    }
}
unsafe fn input_clear(mut ictx: *mut input_ctx) {
    unsafe {
        (*ictx).ground_timer.disarm();
        *(&raw mut (*ictx).interm_buf as *mut u_char) = '\0' as i32 as u_char;
        (*ictx).interm_len = 0 as size_t;
        *(&raw mut (*ictx).param_buf as *mut u_char) = '\0' as i32 as u_char;
        (*ictx).param_len = 0 as size_t;
        (*ictx).input_buf.clear();
        (*ictx).input_buf.push(b'\0');
        (*ictx).input_end = INPUT_END_ST;
        (*ictx).flags &= !INPUT_DISCARD;
    }
}
unsafe fn input_ground(mut ictx: *mut input_ctx) {
    unsafe {
        (*ictx).ground_timer.disarm();
        if let Some(buf) = (*ictx).since_ground.as_deref_mut() {
            buf.clear();
        }
        (*ictx).input_buf.shrink_to(INPUT_BUF_START as usize);
    }
}
unsafe fn input_print(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    unsafe {
        let sctx: &mut screen_write_ctx = &mut (*ictx).ctx;
        let mut set: ::core::ffi::c_int = 0;
        input_stop_utf8(ictx);
        set = if (*ictx).cell.set == 0 as ::core::ffi::c_int {
            (*ictx).cell.g0set
        } else {
            (*ictx).cell.g1set
        };
        if set == 1 as ::core::ffi::c_int {
            (*ictx).cell.cell.attr =
                ((*ictx).cell.cell.attr as ::core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
        } else {
            (*ictx).cell.cell.attr =
                ((*ictx).cell.cell.attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
        }
        utf8_set(&mut (*ictx).cell.cell.data, (*ictx).ch as u_char);
        screen_write_collect_add(sctx, &mut (*ictx).cell.cell);
        utf8_copy(&mut (*ictx).last, &(*ictx).cell.cell.data);
        (*ictx).flags |= INPUT_LAST;
        (*ictx).cell.cell.attr =
            ((*ictx).cell.cell.attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
        0 as ::core::ffi::c_int
    }
}
unsafe fn input_intermediate(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    unsafe {
        if (*ictx).interm_len
            == (::core::mem::size_of::<[u_char; 4]>() as usize).wrapping_sub(1_usize)
        {
            (*ictx).flags |= INPUT_DISCARD;
        } else {
            let fresh15 = (*ictx).interm_len;
            (*ictx).interm_len = (*ictx).interm_len.wrapping_add(1);
            (*ictx).interm_buf[fresh15] = (*ictx).ch as u_char;
            (*ictx).interm_buf[(*ictx).interm_len] = '\0' as i32 as u_char;
        }
        0 as ::core::ffi::c_int
    }
}
unsafe fn input_parameter(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    unsafe {
        if (*ictx).param_len
            == (::core::mem::size_of::<[u_char; 64]>() as usize).wrapping_sub(1_usize)
        {
            (*ictx).flags |= INPUT_DISCARD;
        } else {
            let fresh14 = (*ictx).param_len;
            (*ictx).param_len = (*ictx).param_len.wrapping_add(1);
            (*ictx).param_buf[fresh14] = (*ictx).ch as u_char;
            (*ictx).param_buf[(*ictx).param_len] = '\0' as i32 as u_char;
        }
        0 as ::core::ffi::c_int
    }
}
/// The length of the collected string, which the buffer holds ahead of the
/// terminating NUL the appenders keep at its end.
unsafe fn input_length(ictx: *mut input_ctx) -> size_t {
    unsafe { (*ictx).input_buf.len().wrapping_sub(1_usize) as size_t }
}
unsafe fn input_input(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    unsafe {
        let mut available: size_t = INPUT_BUF_START as size_t;
        while (*ictx).input_buf.len() as size_t >= available {
            available = available.wrapping_mul(2 as size_t);
            if available > input_buffer_size {
                (*ictx).flags |= INPUT_DISCARD;
                return 0 as ::core::ffi::c_int;
            }
        }
        let ch = (*ictx).ch as u8;
        let buf = &mut (*ictx).input_buf;
        buf.reserve((available as usize).wrapping_sub(buf.len()));
        let end = buf.len().wrapping_sub(1_usize);
        buf[end] = ch;
        buf.push(b'\0');
        0 as ::core::ffi::c_int
    }
}
unsafe fn input_c0_dispatch(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    unsafe {
        let sctx: &mut screen_write_ctx = &mut (*ictx).ctx;
        let mut wp: *mut window_pane = (*ictx).pane();
        let mut s: *mut screen = sctx.s;
        let mut gc = grid_default_cell;
        let mut first_gc = grid_default_cell;
        let mut cx: u_int = 0;
        let mut line: u_int = 0;
        let mut width: u_int = 0;
        let mut has_content: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        input_stop_utf8(ictx);
        log_debug(
            c"%s: '%c'".as_ptr(),
            fmt_args![c"input_c0_dispatch".as_ptr(), (*ictx).ch],
        );
        match (*ictx).ch {
            0 => {}
            7 => {
                if !wp.is_null() {
                    alerts_queue((*wp).window, WINDOW_BELL);
                }
            }
            8 => {
                screen_write_backspace(sctx);
            }
            9 => {
                cx = (*s).cx;
                if !(cx >= (*screen_grid_ptr(&mut *s)).sx.wrapping_sub(1 as u_int)) {
                    line = (*s).cy.wrapping_add((*screen_grid_ptr(&mut *s)).hsize);
                    first_gc = grid_get_cell(screen_grid(&*s), cx, line);
                    loop {
                        if has_content == 0 {
                            gc = grid_get_cell(screen_grid(&*s), cx, line);
                            if gc.data.size as ::core::ffi::c_int != 1 as ::core::ffi::c_int
                                || *(&raw mut gc.data.data as *mut u_char) as ::core::ffi::c_int
                                    != ' ' as i32
                                || grid_cells_look_equal(&mut gc, &mut first_gc) == 0
                            {
                                has_content = 1 as ::core::ffi::c_int;
                            }
                        }
                        cx = cx.wrapping_add(1);
                        if (&(*s).tabs)[(cx >> 3 as ::core::ffi::c_int) as usize]
                            as ::core::ffi::c_int
                            & (1 as ::core::ffi::c_int) << (cx & 0x7 as u_int)
                            != 0
                        {
                            break;
                        }
                        if !(cx < (*screen_grid_ptr(&mut *s)).sx.wrapping_sub(1 as u_int)) {
                            break;
                        }
                    }
                    width = cx.wrapping_sub((*s).cx);
                    if has_content != 0
                        || width as usize > ::core::mem::size_of::<[u_char; 32]>() as usize
                    {
                        (*s).cx = cx;
                    } else {
                        gc = grid_get_cell(screen_grid(&*s), (*s).cx, line);
                        grid_set_tab(&mut gc, width);
                        screen_write_collect_add(sctx, &mut gc);
                    }
                }
            }
            10..=12 => {
                screen_write_linefeed(sctx, 0 as ::core::ffi::c_int, (*ictx).cell.cell.bg as u_int);
                if (*s).mode & MODE_CRLF != 0 {
                    screen_write_carriagereturn(sctx);
                }
            }
            13 => {
                screen_write_carriagereturn(sctx);
            }
            14 => {
                (*ictx).cell.set = 1 as ::core::ffi::c_int;
            }
            15 => {
                (*ictx).cell.set = 0 as ::core::ffi::c_int;
            }
            _ => {
                log_debug(
                    c"%s: unknown '%c'".as_ptr(),
                    fmt_args![c"input_c0_dispatch".as_ptr(), (*ictx).ch],
                );
            }
        }
        (*ictx).flags &= !INPUT_LAST;
        0 as ::core::ffi::c_int
    }
}
unsafe fn input_esc_dispatch(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    unsafe {
        let sctx: &mut screen_write_ctx = &mut (*ictx).ctx;
        let mut s: *mut screen = sctx.s;
        if (*ictx).flags & INPUT_DISCARD != 0 {
            return 0 as ::core::ffi::c_int;
        }
        log_debug(
            c"%s: '%c', %s".as_ptr(),
            fmt_args![
                c"input_esc_dispatch".as_ptr(),
                (*ictx).ch,
                &raw mut (*ictx).interm_buf as *mut u_char
            ],
        );
        let Some(entry) = input_table_find(ictx, &input_esc_table) else {
            log_debug(
                c"%s: unknown '%c'".as_ptr(),
                fmt_args![c"input_esc_dispatch".as_ptr(), (*ictx).ch],
            );
            return 0 as ::core::ffi::c_int;
        };
        match entry.type_0 {
            9 => {
                colour_palette_clear((*ictx).palette().as_mut());
                input_reset_cell(ictx);
                screen_write_reset(sctx);
                screen_write_fullredraw(sctx);
            }
            6 => {
                screen_write_linefeed(sctx, 0 as ::core::ffi::c_int, (*ictx).cell.cell.bg as u_int);
            }
            7 => {
                screen_write_carriagereturn(sctx);
                screen_write_linefeed(sctx, 0 as ::core::ffi::c_int, (*ictx).cell.cell.bg as u_int);
            }
            5 => {
                if (*s).cx < (*screen_grid_ptr(&mut *s)).sx {
                    let fresh0 =
                        &mut (&mut (*s).tabs)[((*s).cx >> 3 as ::core::ffi::c_int) as usize];
                    *fresh0 = (*fresh0 as ::core::ffi::c_int
                        | (1 as ::core::ffi::c_int) << ((*s).cx & 0x7 as u_int))
                        as u8;
                }
            }
            8 => {
                screen_write_reverseindex(sctx, (*ictx).cell.cell.bg as u_int);
            }
            1 => {
                screen_write_mode_set(sctx, MODE_KKEYPAD);
            }
            2 => {
                screen_write_mode_clear(sctx, MODE_KKEYPAD);
            }
            4 => {
                input_save_state(ictx);
            }
            3 => {
                input_restore_state(ictx);
            }
            0 => {
                screen_write_alignmenttest(sctx);
            }
            11 => {
                (*ictx).cell.g0set = 1 as ::core::ffi::c_int;
            }
            10 => {
                (*ictx).cell.g0set = 0 as ::core::ffi::c_int;
            }
            13 => {
                (*ictx).cell.g1set = 1 as ::core::ffi::c_int;
            }
            12 => {
                (*ictx).cell.g1set = 0 as ::core::ffi::c_int;
            }
            _ => {}
        }
        (*ictx).flags &= !INPUT_LAST;
        0 as ::core::ffi::c_int
    }
}
unsafe fn input_csi_dispatch(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    unsafe {
        let sctx: &mut screen_write_ctx = &mut (*ictx).ctx;
        let mut s: *mut screen = sctx.s;
        let mut oo: *mut options = ::core::ptr::null_mut::<options>();
        let mut i: ::core::ffi::c_int = 0;
        let mut n: ::core::ffi::c_int = 0;
        let mut m: ::core::ffi::c_int = 0;
        let mut ek: ::core::ffi::c_int = 0;
        let mut set: ::core::ffi::c_int = 0;
        let mut p: ::core::ffi::c_int = 0;
        let mut cx: u_int = 0;
        let mut bg: u_int = (*ictx).cell.cell.bg as u_int;
        if (*ictx).flags & INPUT_DISCARD != 0 {
            return 0 as ::core::ffi::c_int;
        }
        log_debug(
            c"%s: '%c' \"%s\" \"%s\"".as_ptr(),
            fmt_args![
                c"input_csi_dispatch".as_ptr(),
                (*ictx).ch,
                &raw mut (*ictx).interm_buf as *mut u_char,
                &raw mut (*ictx).param_buf as *mut u_char
            ],
        );
        if input_split(ictx) != 0 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        let Some(entry) = input_table_find(ictx, &input_csi_table) else {
            log_debug(
                c"%s: unknown '%c'".as_ptr(),
                fmt_args![c"input_csi_dispatch".as_ptr(), (*ictx).ch],
            );
            return 0 as ::core::ffi::c_int;
        };
        match entry.type_0 {
            0 => {
                cx = (*s).cx;
                if cx > (*screen_grid_ptr(&mut *s)).sx.wrapping_sub(1 as u_int) {
                    cx = (*screen_grid_ptr(&mut *s)).sx.wrapping_sub(1 as u_int);
                }
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                );
                if !(n == -(1 as ::core::ffi::c_int)) {
                    while cx > 0 as u_int && {
                        let fresh10 = n;
                        n -= 1;
                        fresh10 > 0 as ::core::ffi::c_int
                    } {
                        loop {
                            cx = cx.wrapping_sub(1);
                            if !(cx > 0 as u_int
                                && (&(*s).tabs)[(cx >> 3 as ::core::ffi::c_int) as usize]
                                    as ::core::ffi::c_int
                                    & (1 as ::core::ffi::c_int) << (cx & 0x7 as u_int)
                                    == 0)
                            {
                                break;
                            }
                        }
                    }
                    (*s).cx = cx;
                }
            }
            3 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                );
                if n != -(1 as ::core::ffi::c_int) {
                    screen_write_cursorleft(sctx, n as u_int);
                }
            }
            4 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                );
                if n != -(1 as ::core::ffi::c_int) {
                    screen_write_cursordown(sctx, n as u_int);
                }
            }
            5 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                );
                if n != -(1 as ::core::ffi::c_int) {
                    screen_write_cursorright(sctx, n as u_int);
                }
            }
            6 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                );
                m = input_get(
                    ictx,
                    1 as u_int,
                    1 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                );
                if n != -(1 as ::core::ffi::c_int) && m != -(1 as ::core::ffi::c_int) {
                    screen_write_cursormove(
                        sctx,
                        m - 1 as ::core::ffi::c_int,
                        n - 1 as ::core::ffi::c_int,
                        1 as ::core::ffi::c_int,
                    );
                }
            }
            23 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                if !(n != 4 as ::core::ffi::c_int) {
                    m = input_get(
                        ictx,
                        1 as u_int,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                    ek = options_get_number(global_options, c"extended-keys".as_ptr())
                        as ::core::ffi::c_int;
                    if !(ek == 0 as ::core::ffi::c_int) {
                        screen_write_mode_clear(sctx, EXTENDED_KEY_MODES);
                        if m == 2 as ::core::ffi::c_int {
                            screen_write_mode_set(sctx, MODE_KEYS_EXTENDED_2);
                        } else if m == 1 as ::core::ffi::c_int || ek == 2 as ::core::ffi::c_int {
                            screen_write_mode_set(sctx, MODE_KEYS_EXTENDED);
                        }
                    }
                }
            }
            22 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                if !(n != 4 as ::core::ffi::c_int) {
                    screen_write_mode_clear(sctx, MODE_KEYS_EXTENDED | MODE_KEYS_EXTENDED_2);
                    if options_get_number(global_options, c"extended-keys".as_ptr())
                        == 2 as ::core::ffi::c_longlong
                    {
                        screen_write_mode_set(sctx, MODE_KEYS_EXTENDED);
                    }
                }
            }
            39 => {
                input_csi_dispatch_winops(ictx);
            }
            7 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                );
                if n != -(1 as ::core::ffi::c_int) {
                    screen_write_cursorup(sctx, n as u_int);
                }
            }
            1 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                );
                if n != -(1 as ::core::ffi::c_int) {
                    screen_write_carriagereturn(sctx);
                    screen_write_cursordown(sctx, n as u_int);
                }
            }
            2 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                );
                if n != -(1 as ::core::ffi::c_int) {
                    screen_write_carriagereturn(sctx);
                    screen_write_cursorup(sctx, n as u_int);
                }
            }
            8 => {
                match input_get(
                    ictx,
                    0 as u_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                ) {
                    -1 => {}
                    0 => {
                        input_reply(
                            ictx,
                            1 as ::core::ffi::c_int,
                            c"\x1B[?1;2c".as_ptr(),
                            fmt_args![],
                        );
                    }
                    _ => {
                        log_debug(
                            c"%s: unknown '%c'".as_ptr(),
                            fmt_args![c"input_csi_dispatch".as_ptr(), (*ictx).ch],
                        );
                    }
                }
            }
            9 => {
                match input_get(
                    ictx,
                    0 as u_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                ) {
                    -1 => {}
                    0 => {
                        input_reply(
                            ictx,
                            1 as ::core::ffi::c_int,
                            c"\x1B[>84;0;0c".as_ptr(),
                            fmt_args![],
                        );
                    }
                    _ => {
                        log_debug(
                            c"%s: unknown '%c'".as_ptr(),
                            fmt_args![c"input_csi_dispatch".as_ptr(), (*ictx).ch],
                        );
                    }
                }
            }
            16 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                );
                if n != -(1 as ::core::ffi::c_int) {
                    screen_write_clearcharacter(sctx, n as u_int, bg);
                }
            }
            10 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                );
                if n != -(1 as ::core::ffi::c_int) {
                    screen_write_deletecharacter(sctx, n as u_int, bg);
                }
            }
            12 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                );
                m = input_get(
                    ictx,
                    1 as u_int,
                    1 as ::core::ffi::c_int,
                    (*screen_grid_ptr(&mut *s)).sy as ::core::ffi::c_int,
                );
                if n != -(1 as ::core::ffi::c_int) && m != -(1 as ::core::ffi::c_int) {
                    screen_write_scrollregion(
                        sctx,
                        (n - 1 as ::core::ffi::c_int) as u_int,
                        (m - 1 as ::core::ffi::c_int) as u_int,
                    );
                }
            }
            13 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                );
                if n != -(1 as ::core::ffi::c_int) {
                    screen_write_deleteline(sctx, n as u_int, bg);
                }
            }
            15 => {
                if input_get(
                    ictx,
                    0 as u_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                ) == 996
                {
                    input_report_current_theme(ictx);
                }
            }
            24 => {
                m = input_get(
                    ictx,
                    0 as u_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                match m {
                    4 => {
                        n = if (*s).mode & MODE_INSERT != 0 {
                            1 as ::core::ffi::c_int
                        } else {
                            2 as ::core::ffi::c_int
                        };
                    }
                    _ => {
                        n = 0 as ::core::ffi::c_int;
                    }
                }
                if m > 0 as ::core::ffi::c_int {
                    input_reply(
                        ictx,
                        1 as ::core::ffi::c_int,
                        c"\x1B[%d;%d$y".as_ptr(),
                        fmt_args![m, n],
                    );
                }
            }
            25 => {
                m = input_get(
                    ictx,
                    0 as u_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                match m {
                    1 => {
                        n = if (*s).mode & MODE_KCURSOR != 0 {
                            1 as ::core::ffi::c_int
                        } else {
                            2 as ::core::ffi::c_int
                        };
                    }
                    3 => {
                        n = 4 as ::core::ffi::c_int;
                    }
                    6 => {
                        n = if (*s).mode & MODE_ORIGIN != 0 {
                            1 as ::core::ffi::c_int
                        } else {
                            2 as ::core::ffi::c_int
                        };
                    }
                    7 => {
                        n = if (*s).mode & MODE_WRAP != 0 {
                            1 as ::core::ffi::c_int
                        } else {
                            2 as ::core::ffi::c_int
                        };
                    }
                    12 => {
                        if (*s).cstyle as ::core::ffi::c_uint
                            != SCREEN_CURSOR_DEFAULT as ::core::ffi::c_int as ::core::ffi::c_uint
                            || (*s).mode & MODE_CURSOR_BLINKING_SET != 0
                        {
                            n = if (*s).mode & MODE_CURSOR_BLINKING != 0 {
                                1 as ::core::ffi::c_int
                            } else {
                                2 as ::core::ffi::c_int
                            };
                        } else {
                            if !(*ictx).pane().is_null() {
                                oo = (*(*ictx).pane()).options_ptr();
                            } else {
                                oo = global_w_options;
                            }
                            p = options_get_number(oo, c"cursor-style".as_ptr())
                                as ::core::ffi::c_int;
                            n = if p == 1 as ::core::ffi::c_int
                                || p == 3 as ::core::ffi::c_int
                                || p == 5 as ::core::ffi::c_int
                            {
                                1 as ::core::ffi::c_int
                            } else {
                                2 as ::core::ffi::c_int
                            };
                        }
                    }
                    25 => {
                        n = if (*s).mode & MODE_CURSOR != 0 {
                            1 as ::core::ffi::c_int
                        } else {
                            2 as ::core::ffi::c_int
                        };
                    }
                    47 | 1047 | 1049 => {
                        n = if (*s).saved_grid.is_some() {
                            1 as ::core::ffi::c_int
                        } else {
                            2 as ::core::ffi::c_int
                        };
                    }
                    1000 => {
                        n = if (*s).mode & MODE_MOUSE_STANDARD != 0 {
                            1 as ::core::ffi::c_int
                        } else {
                            2 as ::core::ffi::c_int
                        };
                    }
                    1002 => {
                        n = if (*s).mode & MODE_MOUSE_BUTTON != 0 {
                            1 as ::core::ffi::c_int
                        } else {
                            2 as ::core::ffi::c_int
                        };
                    }
                    1003 => {
                        n = if (*s).mode & MODE_MOUSE_ALL != 0 {
                            1 as ::core::ffi::c_int
                        } else {
                            2 as ::core::ffi::c_int
                        };
                    }
                    1004 => {
                        n = if (*s).mode & MODE_FOCUSON != 0 {
                            1 as ::core::ffi::c_int
                        } else {
                            2 as ::core::ffi::c_int
                        };
                    }
                    1005 => {
                        n = if (*s).mode & MODE_MOUSE_UTF8 != 0 {
                            1 as ::core::ffi::c_int
                        } else {
                            2 as ::core::ffi::c_int
                        };
                    }
                    1006 => {
                        n = if (*s).mode & MODE_MOUSE_SGR != 0 {
                            1 as ::core::ffi::c_int
                        } else {
                            2 as ::core::ffi::c_int
                        };
                    }
                    2004 => {
                        n = if (*s).mode & MODE_BRACKETPASTE != 0 {
                            1 as ::core::ffi::c_int
                        } else {
                            2 as ::core::ffi::c_int
                        };
                    }
                    2026 => {
                        n = if (*s).mode & MODE_SYNC != 0 {
                            1 as ::core::ffi::c_int
                        } else {
                            2 as ::core::ffi::c_int
                        };
                    }
                    2031 => {
                        n = if (*s).mode & MODE_THEME_UPDATES != 0 {
                            1 as ::core::ffi::c_int
                        } else {
                            2 as ::core::ffi::c_int
                        };
                    }
                    _ => {
                        n = 0 as ::core::ffi::c_int;
                    }
                }
                if m > 0 as ::core::ffi::c_int {
                    input_reply(
                        ictx,
                        1 as ::core::ffi::c_int,
                        c"\x1B[?%d;%d$y".as_ptr(),
                        fmt_args![m, n],
                    );
                }
            }
            14 => {
                match input_get(
                    ictx,
                    0 as u_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                ) {
                    -1 => {}
                    5 => {
                        input_reply(
                            ictx,
                            1 as ::core::ffi::c_int,
                            c"\x1B[0n".as_ptr(),
                            fmt_args![],
                        );
                    }
                    6 => {
                        input_reply(
                            ictx,
                            1 as ::core::ffi::c_int,
                            c"\x1B[%u;%uR".as_ptr(),
                            fmt_args![
                                (*s).cy.wrapping_add(1 as u_int),
                                (*s).cx.wrapping_add(1 as u_int)
                            ],
                        );
                    }
                    _ => {
                        log_debug(
                            c"%s: unknown '%c'".as_ptr(),
                            fmt_args![c"input_csi_dispatch".as_ptr(), (*ictx).ch],
                        );
                    }
                }
            }
            17 => {
                match input_get(
                    ictx,
                    0 as u_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                ) {
                    -1 => {}
                    0 => {
                        screen_write_clearendofscreen(sctx, bg);
                    }
                    1 => {
                        screen_write_clearstartofscreen(sctx, bg);
                    }
                    2 => {
                        screen_write_clearscreen(sctx, bg);
                    }
                    3 => {
                        if input_get(
                            ictx,
                            1 as u_int,
                            0 as ::core::ffi::c_int,
                            0 as ::core::ffi::c_int,
                        ) == 0 as ::core::ffi::c_int
                        {
                            screen_write_clearhistory(sctx);
                        }
                    }
                    _ => {
                        log_debug(
                            c"%s: unknown '%c'".as_ptr(),
                            fmt_args![c"input_csi_dispatch".as_ptr(), (*ictx).ch],
                        );
                    }
                }
            }
            18 => {
                match input_get(
                    ictx,
                    0 as u_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                ) {
                    -1 => {}
                    0 => {
                        screen_write_clearendofline(sctx, bg);
                    }
                    1 => {
                        screen_write_clearstartofline(sctx, bg);
                    }
                    2 => {
                        screen_write_clearline(sctx, bg);
                    }
                    _ => {
                        log_debug(
                            c"%s: unknown '%c'".as_ptr(),
                            fmt_args![c"input_csi_dispatch".as_ptr(), (*ictx).ch],
                        );
                    }
                }
            }
            19 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                );
                if n != -(1 as ::core::ffi::c_int) {
                    screen_write_cursormove(
                        sctx,
                        n - 1 as ::core::ffi::c_int,
                        -(1 as ::core::ffi::c_int),
                        1 as ::core::ffi::c_int,
                    );
                }
            }
            20 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                );
                if n != -(1 as ::core::ffi::c_int) {
                    screen_write_insertcharacter(sctx, n as u_int, bg);
                }
            }
            21 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                );
                if n != -(1 as ::core::ffi::c_int) {
                    screen_write_insertline(sctx, n as u_int, bg);
                }
            }
            27 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                );
                if !(n == -(1 as ::core::ffi::c_int)) {
                    m = (*screen_grid_ptr(&mut *s)).sx.wrapping_sub((*s).cx) as ::core::ffi::c_int;
                    if n > m {
                        n = m;
                    }
                    if !(!(*ictx).flags & INPUT_LAST != 0) {
                        set = if (*ictx).cell.set == 0 as ::core::ffi::c_int {
                            (*ictx).cell.g0set
                        } else {
                            (*ictx).cell.g1set
                        };
                        if set == 1 as ::core::ffi::c_int {
                            (*ictx).cell.cell.attr = ((*ictx).cell.cell.attr as ::core::ffi::c_int
                                | GRID_ATTR_CHARSET)
                                as u_short;
                        } else {
                            (*ictx).cell.cell.attr = ((*ictx).cell.cell.attr as ::core::ffi::c_int
                                & !GRID_ATTR_CHARSET)
                                as u_short;
                        }
                        utf8_copy(&mut (*ictx).cell.cell.data, &(*ictx).last);
                        i = 0 as ::core::ffi::c_int;
                        while i < n {
                            screen_write_collect_add(sctx, &mut (*ictx).cell.cell);
                            i += 1;
                        }
                    }
                }
            }
            26 => {
                input_restore_state(ictx);
            }
            28 => {
                input_csi_dispatch_rm(ictx);
            }
            29 => {
                input_csi_dispatch_rm_private(ictx);
            }
            30 => {
                input_save_state(ictx);
            }
            32 => {
                input_csi_dispatch_sgr(ictx);
            }
            33 => {
                input_csi_dispatch_sm(ictx);
            }
            35 => {
                input_csi_dispatch_sm_private(ictx);
            }
            34 => {
                input_csi_dispatch_sm_graphics(ictx);
            }
            36 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                );
                if n != -(1 as ::core::ffi::c_int) {
                    screen_write_scrollup(sctx, n as u_int, bg);
                }
            }
            31 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                );
                if n != -(1 as ::core::ffi::c_int) {
                    screen_write_scrolldown(sctx, n as u_int, bg);
                }
            }
            37 => {
                match input_get(
                    ictx,
                    0 as u_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                ) {
                    -1 => {}
                    0 => {
                        if (*s).cx < (*screen_grid_ptr(&mut *s)).sx {
                            let fresh11 = &mut (&mut (*s).tabs)
                                [((*s).cx >> 3 as ::core::ffi::c_int) as usize];
                            *fresh11 = (*fresh11 as ::core::ffi::c_int
                                & !((1 as ::core::ffi::c_int) << ((*s).cx & 0x7 as u_int)))
                                as u8;
                        }
                    }
                    3 => {
                        (*s).tabs.fill(0);
                    }
                    _ => {
                        log_debug(
                            c"%s: unknown '%c'".as_ptr(),
                            fmt_args![c"input_csi_dispatch".as_ptr(), (*ictx).ch],
                        );
                    }
                }
            }
            38 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                );
                if n != -(1 as ::core::ffi::c_int) {
                    screen_write_cursormove(
                        sctx,
                        -(1 as ::core::ffi::c_int),
                        n - 1 as ::core::ffi::c_int,
                        1 as ::core::ffi::c_int,
                    );
                }
            }
            11 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                if !(n == -(1 as ::core::ffi::c_int)) {
                    screen_set_cursor_style(n as u_int, &mut (*s).cstyle, &mut (*s).mode);
                    if n == 0 as ::core::ffi::c_int {
                        screen_write_mode_clear(sctx, MODE_CURSOR_BLINKING_SET);
                    }
                }
            }
            40 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                if n == 0 as ::core::ffi::c_int {
                    input_reply(
                        ictx,
                        1 as ::core::ffi::c_int,
                        c"\x1BP>|tmux %s\x1B\\".as_ptr(),
                        fmt_args![getversion()],
                    );
                }
            }
            _ => {}
        }
        (*ictx).flags &= !INPUT_LAST;
        0 as ::core::ffi::c_int
    }
}
unsafe fn input_csi_dispatch_rm(mut ictx: *mut input_ctx) {
    unsafe {
        let sctx: &mut screen_write_ctx = &mut (*ictx).ctx;
        let mut i: u_int = 0;
        i = 0 as u_int;
        while i < (*ictx).param_list_len {
            match input_get(ictx, i, 0 as ::core::ffi::c_int, -(1 as ::core::ffi::c_int)) {
                -1 => {}
                4 => {
                    screen_write_mode_clear(sctx, MODE_INSERT);
                }
                34 => {
                    screen_write_mode_set(sctx, MODE_CURSOR_VERY_VISIBLE);
                }
                _ => {
                    log_debug(
                        c"%s: unknown '%c'".as_ptr(),
                        fmt_args![c"input_csi_dispatch_rm".as_ptr(), (*ictx).ch],
                    );
                }
            }
            i = i.wrapping_add(1);
        }
    }
}
unsafe fn input_csi_dispatch_rm_private(mut ictx: *mut input_ctx) {
    unsafe {
        let sctx: &mut screen_write_ctx = &mut (*ictx).ctx;
        let mut gc: *mut grid_cell = &mut (*ictx).cell.cell;
        let mut i: u_int = 0;
        i = 0 as u_int;
        while i < (*ictx).param_list_len {
            match input_get(ictx, i, 0 as ::core::ffi::c_int, -(1 as ::core::ffi::c_int)) {
                -1 => {}
                1 => {
                    screen_write_mode_clear(sctx, MODE_KCURSOR);
                }
                3 => {
                    screen_write_cursormove(
                        sctx,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        1 as ::core::ffi::c_int,
                    );
                    screen_write_clearscreen(sctx, (*gc).bg as u_int);
                }
                6 => {
                    screen_write_mode_clear(sctx, MODE_ORIGIN);
                    screen_write_cursormove(
                        sctx,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        1 as ::core::ffi::c_int,
                    );
                }
                7 => {
                    screen_write_mode_clear(sctx, MODE_WRAP);
                }
                12 => {
                    screen_write_mode_clear(sctx, MODE_CURSOR_BLINKING);
                    screen_write_mode_set(sctx, MODE_CURSOR_BLINKING_SET);
                }
                25 => {
                    screen_write_mode_clear(sctx, MODE_CURSOR);
                }
                1000..=1003 => {
                    screen_write_mode_clear(sctx, ALL_MOUSE_MODES);
                }
                1004 => {
                    screen_write_mode_clear(sctx, MODE_FOCUSON);
                }
                1005 => {
                    screen_write_mode_clear(sctx, MODE_MOUSE_UTF8);
                }
                1006 => {
                    screen_write_mode_clear(sctx, MODE_MOUSE_SGR);
                }
                47 | 1047 => {
                    screen_write_alternateoff(sctx, &mut *gc, 0 as ::core::ffi::c_int);
                }
                1049 => {
                    screen_write_alternateoff(sctx, &mut *gc, 1 as ::core::ffi::c_int);
                }
                2004 => {
                    screen_write_mode_clear(sctx, MODE_BRACKETPASTE);
                }
                2026 => {
                    screen_write_stop_sync((*ictx).pane());
                    if !(*ictx).pane().is_null() {
                        (*(*ictx).pane()).flags |= PANE_REDRAW;
                    }
                }
                2031 => {
                    screen_write_mode_clear(sctx, MODE_THEME_UPDATES);
                    if !(*ictx).pane().is_null() {
                        (*(*ictx).pane()).flags &= !PANE_THEMECHANGED;
                    }
                }
                _ => {
                    log_debug(
                        c"%s: unknown '%c'".as_ptr(),
                        fmt_args![c"input_csi_dispatch_rm_private".as_ptr(), (*ictx).ch],
                    );
                }
            }
            i = i.wrapping_add(1);
        }
    }
}
unsafe fn input_csi_dispatch_sm(mut ictx: *mut input_ctx) {
    unsafe {
        let sctx: &mut screen_write_ctx = &mut (*ictx).ctx;
        let mut i: u_int = 0;
        i = 0 as u_int;
        while i < (*ictx).param_list_len {
            match input_get(ictx, i, 0 as ::core::ffi::c_int, -(1 as ::core::ffi::c_int)) {
                -1 => {}
                4 => {
                    screen_write_mode_set(sctx, MODE_INSERT);
                }
                34 => {
                    screen_write_mode_clear(sctx, MODE_CURSOR_VERY_VISIBLE);
                }
                _ => {
                    log_debug(
                        c"%s: unknown '%c'".as_ptr(),
                        fmt_args![c"input_csi_dispatch_sm".as_ptr(), (*ictx).ch],
                    );
                }
            }
            i = i.wrapping_add(1);
        }
    }
}
unsafe fn input_csi_dispatch_sm_private(mut ictx: *mut input_ctx) {
    unsafe {
        let sctx: &mut screen_write_ctx = &mut (*ictx).ctx;
        let mut gc: *mut grid_cell = &mut (*ictx).cell.cell;
        let mut i: u_int = 0;
        i = 0 as u_int;
        while i < (*ictx).param_list_len {
            match input_get(ictx, i, 0 as ::core::ffi::c_int, -(1 as ::core::ffi::c_int)) {
                -1 => {}
                1 => {
                    screen_write_mode_set(sctx, MODE_KCURSOR);
                }
                3 => {
                    screen_write_cursormove(
                        sctx,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        1 as ::core::ffi::c_int,
                    );
                    screen_write_clearscreen(sctx, (*ictx).cell.cell.bg as u_int);
                }
                6 => {
                    screen_write_mode_set(sctx, MODE_ORIGIN);
                    screen_write_cursormove(
                        sctx,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        1 as ::core::ffi::c_int,
                    );
                }
                7 => {
                    screen_write_mode_set(sctx, MODE_WRAP);
                }
                12 => {
                    screen_write_mode_set(sctx, MODE_CURSOR_BLINKING);
                    screen_write_mode_set(sctx, MODE_CURSOR_BLINKING_SET);
                }
                25 => {
                    screen_write_mode_set(sctx, MODE_CURSOR);
                }
                1000 => {
                    screen_write_mode_clear(sctx, ALL_MOUSE_MODES);
                    screen_write_mode_set(sctx, MODE_MOUSE_STANDARD);
                }
                1002 => {
                    screen_write_mode_clear(sctx, ALL_MOUSE_MODES);
                    screen_write_mode_set(sctx, MODE_MOUSE_BUTTON);
                }
                1003 => {
                    screen_write_mode_clear(sctx, ALL_MOUSE_MODES);
                    screen_write_mode_set(sctx, MODE_MOUSE_ALL);
                }
                1004 => {
                    screen_write_mode_set(sctx, MODE_FOCUSON);
                }
                1005 => {
                    screen_write_mode_set(sctx, MODE_MOUSE_UTF8);
                }
                1006 => {
                    screen_write_mode_set(sctx, MODE_MOUSE_SGR);
                }
                47 | 1047 => {
                    screen_write_alternateon(sctx, &mut *gc, 0 as ::core::ffi::c_int);
                }
                1049 => {
                    screen_write_alternateon(sctx, &mut *gc, 1 as ::core::ffi::c_int);
                }
                2004 => {
                    screen_write_mode_set(sctx, MODE_BRACKETPASTE);
                }
                2031 => {
                    screen_write_mode_set(sctx, MODE_THEME_UPDATES);
                    if !(*ictx).pane().is_null() {
                        (*(*ictx).pane()).last_theme = window_pane_get_theme((*ictx).pane());
                        (*(*ictx).pane()).flags &= !PANE_THEMECHANGED;
                    }
                }
                2026 => {
                    screen_write_start_sync((*ictx).pane());
                }
                _ => {
                    log_debug(
                        c"%s: unknown '%c'".as_ptr(),
                        fmt_args![c"input_csi_dispatch_sm_private".as_ptr(), (*ictx).ch],
                    );
                }
            }
            i = i.wrapping_add(1);
        }
    }
}
unsafe fn input_csi_dispatch_sm_graphics(_ictx: *mut input_ctx) {}
unsafe fn input_csi_dispatch_winops(mut ictx: *mut input_ctx) {
    unsafe {
        let sctx: &mut screen_write_ctx = &mut (*ictx).ctx;
        let mut s: *mut screen = sctx.s;
        let mut wp: *mut window_pane = (*ictx).pane();
        let mut w: *mut window = ::core::ptr::null_mut::<window>();
        let mut x: u_int = (*screen_grid_ptr(&mut *s)).sx;
        let mut y: u_int = (*screen_grid_ptr(&mut *s)).sy;
        let mut n: ::core::ffi::c_int = 0;
        let mut m: ::core::ffi::c_int = 0;
        if !wp.is_null() {
            w = (*wp).window;
        }
        m = 0 as ::core::ffi::c_int;
        loop {
            n = input_get(
                ictx,
                m as u_int,
                0 as ::core::ffi::c_int,
                -(1 as ::core::ffi::c_int),
            );
            if !(n != -(1 as ::core::ffi::c_int)) {
                break;
            }
            let mut current_block_25: u64;
            match n {
                1 | 2 | 5 | 6 | 7 | 11 | 13 | 20 | 21 | 24 => {
                    current_block_25 = 980989089337379490;
                }
                3 | 4 | 8 => {
                    m += 1;
                    if input_get(
                        ictx,
                        m as u_int,
                        0 as ::core::ffi::c_int,
                        -(1 as ::core::ffi::c_int),
                    ) == -(1 as ::core::ffi::c_int)
                    {
                        return;
                    }
                    current_block_25 = 16600302173379129607;
                }
                9 | 10 => {
                    current_block_25 = 16600302173379129607;
                }
                14 => {
                    if w.is_null() {
                        current_block_25 = 980989089337379490;
                    } else {
                        input_reply(
                            ictx,
                            1 as ::core::ffi::c_int,
                            c"\x1B[4;%u;%ut".as_ptr(),
                            fmt_args![y.wrapping_mul((*w).ypixel), x.wrapping_mul((*w).xpixel)],
                        );
                        current_block_25 = 980989089337379490;
                    }
                }
                15 => {
                    if w.is_null() {
                        current_block_25 = 980989089337379490;
                    } else {
                        input_reply(
                            ictx,
                            1 as ::core::ffi::c_int,
                            c"\x1B[5;%u;%ut".as_ptr(),
                            fmt_args![y.wrapping_mul((*w).ypixel), x.wrapping_mul((*w).xpixel)],
                        );
                        current_block_25 = 980989089337379490;
                    }
                }
                16 => {
                    if w.is_null() {
                        current_block_25 = 980989089337379490;
                    } else {
                        input_reply(
                            ictx,
                            1 as ::core::ffi::c_int,
                            c"\x1B[6;%u;%ut".as_ptr(),
                            fmt_args![(*w).ypixel, (*w).xpixel],
                        );
                        current_block_25 = 980989089337379490;
                    }
                }
                18 => {
                    input_reply(
                        ictx,
                        1 as ::core::ffi::c_int,
                        c"\x1B[8;%u;%ut".as_ptr(),
                        fmt_args![y, x],
                    );
                    current_block_25 = 980989089337379490;
                }
                19 => {
                    input_reply(
                        ictx,
                        1 as ::core::ffi::c_int,
                        c"\x1B[9;%u;%ut".as_ptr(),
                        fmt_args![y, x],
                    );
                    current_block_25 = 980989089337379490;
                }
                22 => {
                    m += 1;
                    match input_get(
                        ictx,
                        m as u_int,
                        0 as ::core::ffi::c_int,
                        -(1 as ::core::ffi::c_int),
                    ) {
                        -1 => return,
                        0 | 2 => {
                            screen_push_title(sctx.s);
                        }
                        _ => {}
                    }
                    current_block_25 = 980989089337379490;
                }
                23 => {
                    m += 1;
                    match input_get(
                        ictx,
                        m as u_int,
                        0 as ::core::ffi::c_int,
                        -(1 as ::core::ffi::c_int),
                    ) {
                        -1 => return,
                        0 | 2 => {
                            screen_pop_title(sctx.s);
                            if !wp.is_null() {
                                notify_pane(c"pane-title-changed".as_ptr(), wp);
                                server_redraw_window_borders(w);
                                server_status_window(w);
                            }
                        }
                        _ => {}
                    }
                    current_block_25 = 980989089337379490;
                }
                _ => {
                    log_debug(
                        c"%s: unknown '%c'".as_ptr(),
                        fmt_args![c"input_csi_dispatch_winops".as_ptr(), (*ictx).ch],
                    );
                    current_block_25 = 980989089337379490;
                }
            }
            if current_block_25 == 16600302173379129607 {
                m += 1;
                if input_get(
                    ictx,
                    m as u_int,
                    0 as ::core::ffi::c_int,
                    -(1 as ::core::ffi::c_int),
                ) == -(1 as ::core::ffi::c_int)
                {
                    return;
                }
            }
            m += 1;
        }
    }
}
unsafe fn input_csi_dispatch_sgr_256_do(
    mut ictx: *mut input_ctx,
    mut fgbg: ::core::ffi::c_int,
    mut c: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut gc: *mut grid_cell = &mut (*ictx).cell.cell;
        if c == -(1 as ::core::ffi::c_int) || c > 255 as ::core::ffi::c_int {
            if fgbg == 38 as ::core::ffi::c_int {
                (*gc).fg = 8 as ::core::ffi::c_int;
            } else if fgbg == 48 as ::core::ffi::c_int {
                (*gc).bg = 8 as ::core::ffi::c_int;
            }
        } else if fgbg == 38 as ::core::ffi::c_int {
            (*gc).fg = c | COLOUR_FLAG_256;
        } else if fgbg == 48 as ::core::ffi::c_int {
            (*gc).bg = c | COLOUR_FLAG_256;
        } else if fgbg == 58 as ::core::ffi::c_int {
            (*gc).us = c | COLOUR_FLAG_256;
        }
        1 as ::core::ffi::c_int
    }
}
unsafe fn input_csi_dispatch_sgr_256(
    mut ictx: *mut input_ctx,
    mut fgbg: ::core::ffi::c_int,
    i: &mut u_int,
) {
    unsafe {
        let mut c: ::core::ffi::c_int = 0;
        c = input_get(
            ictx,
            (*i).wrapping_add(1 as u_int),
            0 as ::core::ffi::c_int,
            -(1 as ::core::ffi::c_int),
        );
        if input_csi_dispatch_sgr_256_do(ictx, fgbg, c) != 0 {
            *i = (*i).wrapping_add(1);
        }
    }
}
unsafe fn input_csi_dispatch_sgr_rgb_do(
    mut ictx: *mut input_ctx,
    mut fgbg: ::core::ffi::c_int,
    mut r: ::core::ffi::c_int,
    mut g: ::core::ffi::c_int,
    mut b: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut gc: *mut grid_cell = &mut (*ictx).cell.cell;
        if r == -(1 as ::core::ffi::c_int) || r > 255 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        if g == -(1 as ::core::ffi::c_int) || g > 255 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        if b == -(1 as ::core::ffi::c_int) || b > 255 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        if fgbg == 38 as ::core::ffi::c_int {
            (*gc).fg = colour_join_rgb(r as u_char, g as u_char, b as u_char);
        } else if fgbg == 48 as ::core::ffi::c_int {
            (*gc).bg = colour_join_rgb(r as u_char, g as u_char, b as u_char);
        } else if fgbg == 58 as ::core::ffi::c_int {
            (*gc).us = colour_join_rgb(r as u_char, g as u_char, b as u_char);
        }
        1 as ::core::ffi::c_int
    }
}
unsafe fn input_csi_dispatch_sgr_rgb(
    mut ictx: *mut input_ctx,
    mut fgbg: ::core::ffi::c_int,
    i: &mut u_int,
) {
    unsafe {
        let mut r: ::core::ffi::c_int = 0;
        let mut g: ::core::ffi::c_int = 0;
        let mut b: ::core::ffi::c_int = 0;
        r = input_get(
            ictx,
            (*i).wrapping_add(1 as u_int),
            0 as ::core::ffi::c_int,
            -(1 as ::core::ffi::c_int),
        );
        g = input_get(
            ictx,
            (*i).wrapping_add(2 as u_int),
            0 as ::core::ffi::c_int,
            -(1 as ::core::ffi::c_int),
        );
        b = input_get(
            ictx,
            (*i).wrapping_add(3 as u_int),
            0 as ::core::ffi::c_int,
            -(1 as ::core::ffi::c_int),
        );
        if input_csi_dispatch_sgr_rgb_do(ictx, fgbg, r, g, b) != 0 {
            *i = (*i).wrapping_add(3 as u_int);
        }
    }
}
unsafe fn input_csi_dispatch_sgr_colon(mut ictx: *mut input_ctx, mut i: u_int) {
    unsafe {
        let mut gc: *mut grid_cell = &mut (*ictx).cell.cell;
        let InputParam::Str(value) = &input_params(ictx)[i as usize] else {
            return;
        };
        let mut s: *const ::core::ffi::c_char = value.as_ptr();
        let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut p: [::core::ffi::c_int; 8] = [0; 8];
        let mut n: u_int = 0;
        n = 0 as u_int;
        while (n as usize)
            < (::core::mem::size_of::<[::core::ffi::c_int; 8]>() as usize)
                .wrapping_div(::core::mem::size_of::<::core::ffi::c_int>() as usize)
        {
            p[n as usize] = -(1 as ::core::ffi::c_int);
            n = n.wrapping_add(1);
        }
        n = 0 as u_int;
        let mut copy = CStr::from_ptr(s).to_bytes_with_nul().to_vec();
        ptr = copy.as_mut_ptr() as *mut ::core::ffi::c_char;
        loop {
            out = strsep(&raw mut ptr, c":".as_ptr());
            if out.is_null() {
                break;
            }
            if *out as ::core::ffi::c_int != '\0' as i32 {
                let fresh13 = n;
                n = n.wrapping_add(1);
                let parsed = strtonum(
                    out,
                    0 as ::core::ffi::c_longlong,
                    INT_MAX as ::core::ffi::c_longlong,
                );
                p[fresh13 as usize] = parsed.unwrap_or(0) as ::core::ffi::c_int;
                if parsed.is_err()
                    || n as usize
                        == (::core::mem::size_of::<[::core::ffi::c_int; 8]>() as usize)
                            .wrapping_div(::core::mem::size_of::<::core::ffi::c_int>() as usize)
                {
                    return;
                }
            } else {
                n = n.wrapping_add(1);
                if n as usize
                    == (::core::mem::size_of::<[::core::ffi::c_int; 8]>() as usize)
                        .wrapping_div(::core::mem::size_of::<::core::ffi::c_int>() as usize)
                {
                    return;
                }
            }
            log_debug(
                c"%s: %u = %d".as_ptr(),
                fmt_args![
                    c"input_csi_dispatch_sgr_colon".as_ptr(),
                    n.wrapping_sub(1 as u_int),
                    p[n.wrapping_sub(1 as u_int) as usize]
                ],
            );
        }
        if n == 0 as u_int {
            return;
        }
        if p[0 as ::core::ffi::c_int as usize] == 4 as ::core::ffi::c_int {
            if n != 2 as u_int {
                return;
            }
            match p[1 as ::core::ffi::c_int as usize] {
                0 => {
                    (*gc).attr =
                        ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_ALL_UNDERSCORE) as u_short;
                }
                1 => {
                    (*gc).attr =
                        ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_ALL_UNDERSCORE) as u_short;
                    (*gc).attr =
                        ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_UNDERSCORE) as u_short;
                }
                2 => {
                    (*gc).attr =
                        ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_ALL_UNDERSCORE) as u_short;
                    (*gc).attr =
                        ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_UNDERSCORE_2) as u_short;
                }
                3 => {
                    (*gc).attr =
                        ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_ALL_UNDERSCORE) as u_short;
                    (*gc).attr =
                        ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_UNDERSCORE_3) as u_short;
                }
                4 => {
                    (*gc).attr =
                        ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_ALL_UNDERSCORE) as u_short;
                    (*gc).attr =
                        ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_UNDERSCORE_4) as u_short;
                }
                5 => {
                    (*gc).attr =
                        ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_ALL_UNDERSCORE) as u_short;
                    (*gc).attr =
                        ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_UNDERSCORE_5) as u_short;
                }
                _ => {}
            }
            return;
        }
        if n < 2 as u_int
            || p[0 as ::core::ffi::c_int as usize] != 38 as ::core::ffi::c_int
                && p[0 as ::core::ffi::c_int as usize] != 48 as ::core::ffi::c_int
                && p[0 as ::core::ffi::c_int as usize] != 58 as ::core::ffi::c_int
        {
            return;
        }
        match p[1 as ::core::ffi::c_int as usize] {
            2 => {
                if !(n < 3 as u_int) {
                    if n == 5 as u_int {
                        i = 2 as u_int;
                    } else {
                        i = 3 as u_int;
                    }
                    if !(n < i.wrapping_add(3 as u_int)) {
                        input_csi_dispatch_sgr_rgb_do(
                            ictx,
                            p[0 as ::core::ffi::c_int as usize],
                            p[i as usize],
                            p[i.wrapping_add(1 as u_int) as usize],
                            p[i.wrapping_add(2 as u_int) as usize],
                        );
                    }
                }
            }
            5 if !(n < 3 as u_int) => {
                input_csi_dispatch_sgr_256_do(
                    ictx,
                    p[0 as ::core::ffi::c_int as usize],
                    p[2 as ::core::ffi::c_int as usize],
                );
            }
            _ => {}
        };
    }
}
unsafe fn input_csi_dispatch_sgr(mut ictx: *mut input_ctx) {
    unsafe {
        let mut gc: *mut grid_cell = &mut (*ictx).cell.cell;
        let mut i: u_int = 0;
        let mut link: u_int = 0;
        let mut n: ::core::ffi::c_int = 0;
        if (*ictx).param_list_len == 0 as u_int {
            *gc = grid_default_cell;
            return;
        }
        i = 0 as u_int;
        while i < (*ictx).param_list_len {
            if matches!(input_params(ictx)[i as usize], InputParam::Str(_)) {
                input_csi_dispatch_sgr_colon(ictx, i);
            } else {
                n = input_get(ictx, i, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
                if !(n == -(1 as ::core::ffi::c_int)) {
                    if n == 38 as ::core::ffi::c_int
                        || n == 48 as ::core::ffi::c_int
                        || n == 58 as ::core::ffi::c_int
                    {
                        i = i.wrapping_add(1);
                        match input_get(
                            ictx,
                            i,
                            0 as ::core::ffi::c_int,
                            -(1 as ::core::ffi::c_int),
                        ) {
                            2 => {
                                input_csi_dispatch_sgr_rgb(ictx, n, &mut i);
                            }
                            5 => {
                                input_csi_dispatch_sgr_256(ictx, n, &mut i);
                            }
                            _ => {}
                        }
                    } else {
                        match n {
                            0 => {
                                link = (*gc).link;
                                *gc = grid_default_cell;
                                (*gc).link = link;
                            }
                            1 => {
                                (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_BRIGHT)
                                    as u_short;
                            }
                            2 => {
                                (*gc).attr =
                                    ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_DIM) as u_short;
                            }
                            3 => {
                                (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_ITALICS)
                                    as u_short;
                            }
                            4 => {
                                (*gc).attr = ((*gc).attr as ::core::ffi::c_int
                                    & !GRID_ATTR_ALL_UNDERSCORE)
                                    as u_short;
                                (*gc).attr = ((*gc).attr as ::core::ffi::c_int
                                    | GRID_ATTR_UNDERSCORE)
                                    as u_short;
                            }
                            5 | 6 => {
                                (*gc).attr =
                                    ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_BLINK) as u_short;
                            }
                            7 => {
                                (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_REVERSE)
                                    as u_short;
                            }
                            8 => {
                                (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_HIDDEN)
                                    as u_short;
                            }
                            9 => {
                                (*gc).attr = ((*gc).attr as ::core::ffi::c_int
                                    | GRID_ATTR_STRIKETHROUGH)
                                    as u_short;
                            }
                            21 => {
                                (*gc).attr = ((*gc).attr as ::core::ffi::c_int
                                    & !GRID_ATTR_ALL_UNDERSCORE)
                                    as u_short;
                                (*gc).attr = ((*gc).attr as ::core::ffi::c_int
                                    | GRID_ATTR_UNDERSCORE_2)
                                    as u_short;
                            }
                            22 => {
                                (*gc).attr = ((*gc).attr as ::core::ffi::c_int
                                    & !(GRID_ATTR_BRIGHT | GRID_ATTR_DIM))
                                    as u_short;
                            }
                            23 => {
                                (*gc).attr = ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_ITALICS)
                                    as u_short;
                            }
                            24 => {
                                (*gc).attr = ((*gc).attr as ::core::ffi::c_int
                                    & !GRID_ATTR_ALL_UNDERSCORE)
                                    as u_short;
                            }
                            25 => {
                                (*gc).attr = ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_BLINK)
                                    as u_short;
                            }
                            27 => {
                                (*gc).attr = ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_REVERSE)
                                    as u_short;
                            }
                            28 => {
                                (*gc).attr = ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_HIDDEN)
                                    as u_short;
                            }
                            29 => {
                                (*gc).attr = ((*gc).attr as ::core::ffi::c_int
                                    & !GRID_ATTR_STRIKETHROUGH)
                                    as u_short;
                            }
                            30..=37 => {
                                (*gc).fg = n - 30 as ::core::ffi::c_int;
                            }
                            39 => {
                                (*gc).fg = 8 as ::core::ffi::c_int;
                            }
                            40..=47 => {
                                (*gc).bg = n - 40 as ::core::ffi::c_int;
                            }
                            49 => {
                                (*gc).bg = 8 as ::core::ffi::c_int;
                            }
                            53 => {
                                (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_OVERLINE)
                                    as u_short;
                            }
                            55 => {
                                (*gc).attr = ((*gc).attr as ::core::ffi::c_int
                                    & !GRID_ATTR_OVERLINE)
                                    as u_short;
                            }
                            59 => {
                                (*gc).us = 8 as ::core::ffi::c_int;
                            }
                            90..=97 => {
                                (*gc).fg = n;
                            }
                            100..=107 => {
                                (*gc).bg = n - 10 as ::core::ffi::c_int;
                            }
                            _ => {}
                        }
                    }
                }
            }
            i = i.wrapping_add(1);
        }
    }
}
unsafe fn input_end_bel(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    unsafe {
        log_debug(c"%s".as_ptr(), fmt_args![c"input_end_bel".as_ptr()]);
        (*ictx).input_end = INPUT_END_BEL;
        0 as ::core::ffi::c_int
    }
}
unsafe fn input_enter_dcs(mut ictx: *mut input_ctx) {
    unsafe {
        log_debug(c"%s".as_ptr(), fmt_args![c"input_enter_dcs".as_ptr()]);
        input_clear(ictx);
        input_start_ground_timer(ictx);
        (*ictx).flags &= !INPUT_LAST;
    }
}
unsafe fn input_handle_decrqss(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    unsafe {
        let mut wp: *mut window_pane = (*ictx).pane();
        let mut oo: *mut options = ::core::ptr::null_mut::<options>();
        let sctx: &mut screen_write_ctx = &mut (*ictx).ctx;
        let mut buf: *mut u_char = (*ictx).input_buf.as_mut_ptr();
        let mut len: size_t = input_length(ictx);
        let mut s: *mut screen = sctx.s;
        let mut ps: ::core::ffi::c_int = 0;
        let mut opt_ps: ::core::ffi::c_int = 0;
        let mut blinking: ::core::ffi::c_int = 0;
        if len < 3 as size_t
            || *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != ' ' as i32
            || *buf.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != 'q' as i32
        {
            input_reply(
                ictx,
                1 as ::core::ffi::c_int,
                c"\x1BP0$r\x1B\\".as_ptr(),
                fmt_args![],
            );
            0 as ::core::ffi::c_int
        } else {
            if (*s).cstyle as ::core::ffi::c_uint
                == SCREEN_CURSOR_BLOCK as ::core::ffi::c_int as ::core::ffi::c_uint
                || (*s).cstyle as ::core::ffi::c_uint
                    == SCREEN_CURSOR_UNDERLINE as ::core::ffi::c_int as ::core::ffi::c_uint
                || (*s).cstyle as ::core::ffi::c_uint
                    == SCREEN_CURSOR_BAR as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                blinking = ((*s).mode & MODE_CURSOR_BLINKING != 0 as ::core::ffi::c_int)
                    as ::core::ffi::c_int;
                match (*s).cstyle {
                    SCREEN_CURSOR_BLOCK => {
                        ps = if blinking != 0 {
                            1 as ::core::ffi::c_int
                        } else {
                            2 as ::core::ffi::c_int
                        };
                    }
                    SCREEN_CURSOR_UNDERLINE => {
                        ps = if blinking != 0 {
                            3 as ::core::ffi::c_int
                        } else {
                            4 as ::core::ffi::c_int
                        };
                    }
                    SCREEN_CURSOR_BAR => {
                        ps = if blinking != 0 {
                            5 as ::core::ffi::c_int
                        } else {
                            6 as ::core::ffi::c_int
                        };
                    }
                    _ => {
                        ps = 0 as ::core::ffi::c_int;
                    }
                }
            } else {
                if !wp.is_null() {
                    oo = (*wp).options_ptr();
                } else {
                    oo = global_w_options;
                }
                opt_ps = options_get_number(oo, c"cursor-style".as_ptr()) as ::core::ffi::c_int;
                if opt_ps < 0 as ::core::ffi::c_int || opt_ps > 6 as ::core::ffi::c_int {
                    opt_ps = 0 as ::core::ffi::c_int;
                }
                ps = opt_ps;
            }
            log_debug(
                c"%s: DECRQSS cursor -> Ps=%d (cstyle=%d mode=%#x)".as_ptr(),
                fmt_args![
                    c"input_handle_decrqss".as_ptr(),
                    ps,
                    (*s).cstyle as ::core::ffi::c_uint,
                    (*s).mode
                ],
            );
            input_reply(
                ictx,
                1 as ::core::ffi::c_int,
                c"\x1BP1$r q%d q\x1B\\".as_ptr(),
                fmt_args![ps],
            );
            0 as ::core::ffi::c_int
        }
    }
}
unsafe fn input_dcs_dispatch(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    unsafe {
        let mut wp: *mut window_pane = (*ictx).pane();
        let mut oo: *mut options = ::core::ptr::null_mut::<options>();
        let sctx: &mut screen_write_ctx = &mut (*ictx).ctx;
        let mut buf: *mut u_char = (*ictx).input_buf.as_mut_ptr();
        let mut len: size_t = input_length(ictx);
        let prefix: [::core::ffi::c_char; 6] =
            ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b"tmux;\0");
        let prefixlen: u_int = (::core::mem::size_of::<[::core::ffi::c_char; 6]>() as usize)
            .wrapping_sub(1_usize) as u_int;
        let mut allow_passthrough: ::core::ffi::c_longlong = 0 as ::core::ffi::c_longlong;
        if wp.is_null() {
            oo = global_w_options;
        } else {
            oo = (*wp).options_ptr();
        }
        if (*ictx).flags & INPUT_DISCARD != 0 {
            log_debug(
                c"%s: %zu bytes (discard)".as_ptr(),
                fmt_args![c"input_dcs_dispatch".as_ptr(), len],
            );
            return 0 as ::core::ffi::c_int;
        }
        if (*ictx).interm_len == 1 as size_t
            && (*ictx).interm_buf[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                == '$' as i32
            && len >= 1 as size_t
            && *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'q' as i32
        {
            return input_handle_decrqss(ictx);
        }
        allow_passthrough = options_get_number(oo, c"allow-passthrough".as_ptr());
        if allow_passthrough == 0 {
            return 0 as ::core::ffi::c_int;
        }
        log_debug(
            c"%s: \"%s\"".as_ptr(),
            fmt_args![c"input_dcs_dispatch".as_ptr(), buf],
        );
        if len >= prefixlen as size_t
            && strncmp(
                buf as *const ::core::ffi::c_char,
                &raw const prefix as *const ::core::ffi::c_char,
                prefixlen as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            screen_write_rawstring(
                sctx,
                buf.offset(prefixlen as isize),
                len.wrapping_sub(prefixlen as size_t) as u_int,
                (allow_passthrough == 2 as ::core::ffi::c_longlong) as ::core::ffi::c_int,
            );
        }
        0 as ::core::ffi::c_int
    }
}
unsafe fn input_enter_osc(mut ictx: *mut input_ctx) {
    unsafe {
        log_debug(c"%s".as_ptr(), fmt_args![c"input_enter_osc".as_ptr()]);
        input_clear(ictx);
        input_start_ground_timer(ictx);
        (*ictx).flags &= !INPUT_LAST;
    }
}
unsafe fn input_exit_osc(mut ictx: *mut input_ctx) {
    unsafe {
        let sctx: &mut screen_write_ctx = &mut (*ictx).ctx;
        let mut wp: *mut window_pane = (*ictx).pane();
        let mut p: *mut u_char = (*ictx).input_buf.as_mut_ptr();
        let mut option: u_int = 0;
        if (*ictx).flags & INPUT_DISCARD != 0 {
            return;
        }
        if input_length(ictx) < 1 as size_t
            || (*p as ::core::ffi::c_int) < '0' as i32
            || *p as ::core::ffi::c_int > '9' as i32
        {
            return;
        }
        log_debug(
            c"%s: \"%s\" (end %s)".as_ptr(),
            fmt_args![
                c"input_exit_osc".as_ptr(),
                p,
                if (*ictx).input_end as ::core::ffi::c_uint
                    == INPUT_END_ST as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    c"ST".as_ptr()
                } else {
                    c"BEL".as_ptr()
                }
            ],
        );
        option = 0 as u_int;
        while *p as ::core::ffi::c_int >= '0' as i32 && *p as ::core::ffi::c_int <= '9' as i32 {
            let fresh2 = p;
            p = p.offset(1);
            option = option
                .wrapping_mul(10 as u_int)
                .wrapping_add(*fresh2 as u_int)
                .wrapping_sub('0' as i32 as u_int);
        }
        if *p as ::core::ffi::c_int != ';' as i32 && *p as ::core::ffi::c_int != '\0' as i32 {
            return;
        }
        if *p as ::core::ffi::c_int == ';' as i32 {
            p = p.offset(1);
        }
        match option {
            0 | 2 => {
                if !wp.is_null()
                    && options_get_number((*wp).options_ptr(), c"allow-set-title".as_ptr()) != 0
                    && screen_set_title(
                        &mut *sctx.s,
                        p as *const ::core::ffi::c_char,
                        1 as ::core::ffi::c_int,
                    ) != 0
                {
                    notify_pane(c"pane-title-changed".as_ptr(), wp);
                    server_redraw_window_borders((*wp).window);
                    server_status_window((*wp).window);
                }
            }
            4 => {
                input_osc_4(ictx, CStr::from_ptr(p as *const ::core::ffi::c_char));
            }
            7 => {
                if !wp.is_null()
                    && screen_set_path(
                        sctx.s,
                        p as *const ::core::ffi::c_char,
                        1 as ::core::ffi::c_int,
                    ) != 0
                {
                    server_redraw_window_borders((*wp).window);
                    server_status_window((*wp).window);
                }
            }
            8 => {
                input_osc_8(ictx, CStr::from_ptr(p as *const ::core::ffi::c_char));
            }
            9 => {
                input_osc_9(ictx, CStr::from_ptr(p as *const ::core::ffi::c_char));
            }
            10 => {
                input_osc_10(ictx, CStr::from_ptr(p as *const ::core::ffi::c_char));
            }
            11 => {
                input_osc_11(ictx, CStr::from_ptr(p as *const ::core::ffi::c_char));
            }
            12 => {
                input_osc_12(ictx, CStr::from_ptr(p as *const ::core::ffi::c_char));
            }
            52 => {
                input_osc_52(ictx, CStr::from_ptr(p as *const ::core::ffi::c_char));
            }
            104 => {
                input_osc_104(ictx, CStr::from_ptr(p as *const ::core::ffi::c_char));
            }
            110 => {
                input_osc_110(ictx, CStr::from_ptr(p as *const ::core::ffi::c_char));
            }
            111 => {
                input_osc_111(ictx, CStr::from_ptr(p as *const ::core::ffi::c_char));
            }
            112 => {
                input_osc_112(ictx, CStr::from_ptr(p as *const ::core::ffi::c_char));
            }
            133 => {
                input_osc_133(ictx, CStr::from_ptr(p as *const ::core::ffi::c_char));
            }
            _ => {
                log_debug(
                    c"%s: unknown '%u'".as_ptr(),
                    fmt_args![c"input_exit_osc".as_ptr(), option],
                );
            }
        };
    }
}
unsafe fn input_enter_apc(mut ictx: *mut input_ctx) {
    unsafe {
        log_debug(c"%s".as_ptr(), fmt_args![c"input_enter_apc".as_ptr()]);
        input_clear(ictx);
        input_start_ground_timer(ictx);
        (*ictx).flags &= !INPUT_LAST;
    }
}
unsafe fn input_exit_apc(mut ictx: *mut input_ctx) {
    unsafe {
        let sctx: &mut screen_write_ctx = &mut (*ictx).ctx;
        let mut wp: *mut window_pane = (*ictx).pane();
        if (*ictx).flags & INPUT_DISCARD != 0 {
            return;
        }
        log_debug(
            c"%s: \"%s\"".as_ptr(),
            fmt_args![c"input_exit_apc".as_ptr(), (*ictx).input_buf.as_ptr()],
        );
        if !wp.is_null()
            && options_get_number((*wp).options_ptr(), c"allow-set-title".as_ptr()) != 0
            && screen_set_title(
                &mut *sctx.s,
                (*ictx).input_buf.as_ptr() as *const ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            ) != 0
        {
            notify_pane(c"pane-title-changed".as_ptr(), wp);
            server_redraw_window_borders((*wp).window);
            server_status_window((*wp).window);
        }
    }
}
unsafe fn input_enter_rename(mut ictx: *mut input_ctx) {
    unsafe {
        log_debug(c"%s".as_ptr(), fmt_args![c"input_enter_rename".as_ptr()]);
        input_clear(ictx);
        input_start_ground_timer(ictx);
        (*ictx).flags &= !INPUT_LAST;
    }
}
unsafe fn input_exit_rename(mut ictx: *mut input_ctx) {
    unsafe {
        let mut wp: *mut window_pane = (*ictx).pane();
        let mut w: *mut window = ::core::ptr::null_mut::<window>();
        let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
        if wp.is_null() {
            return;
        }
        if (*ictx).flags & INPUT_DISCARD != 0 {
            return;
        }
        if options_get_number((*(*ictx).pane()).options_ptr(), c"allow-rename".as_ptr()) == 0 {
            return;
        }
        log_debug(
            c"%s: \"%s\"".as_ptr(),
            fmt_args![c"input_exit_rename".as_ptr(), (*ictx).input_buf.as_ptr()],
        );
        if utf8_isvalid((*ictx).input_buf.as_ptr() as *const ::core::ffi::c_char) == 0 {
            return;
        }
        w = (*wp).window;
        if input_length(ictx) == 0 as size_t {
            o = options_get_only_ptr((*w).options_ptr(), c"automatic-rename".as_ptr());
            if !o.is_null() {
                options_remove_or_default(o, -(1 as ::core::ffi::c_int), &mut None);
            }
            if options_get_number((*w).options_ptr(), c"automatic-rename".as_ptr()) == 0 {
                window_set_name(w, c"".as_ptr(), 1 as ::core::ffi::c_int);
            }
        } else {
            options_set_number(
                (*w).options_ptr(),
                c"automatic-rename".as_ptr(),
                0 as ::core::ffi::c_longlong,
            );
            window_set_name(
                w,
                (*ictx).input_buf.as_ptr() as *const ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            );
        }
        server_redraw_window_borders(w);
        server_status_window(w);
    }
}
unsafe fn input_top_bit_set(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    unsafe {
        let sctx: &mut screen_write_ctx = &mut (*ictx).ctx;
        let mut ud: *mut utf8_data = &raw mut (*ictx).utf8data;
        (*ictx).flags &= !INPUT_LAST;
        if (*ictx).utf8started == 0 {
            (*ictx).utf8started = 1 as ::core::ffi::c_int;
            if utf8_open(&mut *ud, (*ictx).ch as u_char) as ::core::ffi::c_uint
                != UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                input_stop_utf8(ictx);
            }
            return 0 as ::core::ffi::c_int;
        }
        match utf8_append(&mut *ud, (*ictx).ch as u_char) {
            UTF8_MORE => return 0 as ::core::ffi::c_int,
            UTF8_ERROR => {
                input_stop_utf8(ictx);
                return 0 as ::core::ffi::c_int;
            }
            _ => {}
        }
        (*ictx).utf8started = 0 as ::core::ffi::c_int;
        log_debug(
            c"%s %hhu '%*s' (width %hhu)".as_ptr(),
            fmt_args![
                c"input_top_bit_set".as_ptr(),
                (*ud).size as ::core::ffi::c_int,
                (*ud).size as ::core::ffi::c_int,
                &raw mut (*ud).data as *mut u_char,
                (*ud).width as ::core::ffi::c_int
            ],
        );
        utf8_copy(&mut (*ictx).cell.cell.data, &*ud);
        screen_write_collect_add(sctx, &mut (*ictx).cell.cell);
        utf8_copy(&mut (*ictx).last, &(*ictx).cell.cell.data);
        (*ictx).flags |= INPUT_LAST;
        0 as ::core::ffi::c_int
    }
}
unsafe fn input_osc_colour_reply(
    mut ictx: *mut input_ctx,
    mut add: ::core::ffi::c_int,
    mut n: u_int,
    mut idx: ::core::ffi::c_int,
    mut c: ::core::ffi::c_int,
    mut end_type: input_end_type,
) {
    unsafe {
        let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        if c != -(1 as ::core::ffi::c_int) {
            c = colour_force_rgb(c);
        }
        if c == -(1 as ::core::ffi::c_int) {
            return;
        }
        let (r, g, b) = colour_split_rgb(c);
        if end_type as ::core::ffi::c_uint
            == INPUT_END_BEL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            end = c"\x07".as_ptr();
        } else {
            end = c"\x1B\\".as_ptr();
        }
        if n == 4 as u_int {
            input_reply(
                ictx,
                add,
                c"\x1B]%u;%d;rgb:%02hhx%02hhx/%02hhx%02hhx/%02hhx%02hhx%s".as_ptr(),
                fmt_args![
                    n,
                    idx,
                    r as ::core::ffi::c_int,
                    r as ::core::ffi::c_int,
                    g as ::core::ffi::c_int,
                    g as ::core::ffi::c_int,
                    b as ::core::ffi::c_int,
                    b as ::core::ffi::c_int,
                    end
                ],
            );
        } else {
            input_reply(
                ictx,
                add,
                c"\x1B]%u;rgb:%02hhx%02hhx/%02hhx%02hhx/%02hhx%02hhx%s".as_ptr(),
                fmt_args![
                    n,
                    r as ::core::ffi::c_int,
                    r as ::core::ffi::c_int,
                    g as ::core::ffi::c_int,
                    g as ::core::ffi::c_int,
                    b as ::core::ffi::c_int,
                    b as ::core::ffi::c_int,
                    end
                ],
            );
        };
    }
}
unsafe fn input_osc_4(mut ictx: *mut input_ctx, p: &CStr) {
    unsafe {
        let mut p = p.as_ptr();
        let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut idx: ::core::ffi::c_long = 0;
        let mut c: ::core::ffi::c_int = 0;
        let mut bad: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut redraw: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let palette: *mut colour_palette = (*ictx).palette();
        let mut copy = CStr::from_ptr(p).to_bytes_with_nul().to_vec();
        s = copy.as_mut_ptr() as *mut ::core::ffi::c_char;
        while !s.is_null() && *s as ::core::ffi::c_int != '\0' as i32 {
            idx = strtol(s, &raw mut next, 10 as ::core::ffi::c_int);
            let fresh9 = next;
            next = next.offset(1);
            if *fresh9 as ::core::ffi::c_int != ';' as i32 {
                bad = 1 as ::core::ffi::c_int;
                break;
            } else if idx < 0 as ::core::ffi::c_long || idx >= 256 as ::core::ffi::c_long {
                bad = 1 as ::core::ffi::c_int;
                break;
            } else {
                s = strsep(&raw mut next, c";".as_ptr());
                if strcmp(s, c"?".as_ptr()) == 0 as ::core::ffi::c_int {
                    c = colour_palette_get(
                        palette.as_ref(),
                        (idx | COLOUR_FLAG_256 as ::core::ffi::c_long) as ::core::ffi::c_int,
                    );
                    if c != -(1 as ::core::ffi::c_int) {
                        input_osc_colour_reply(
                            ictx,
                            1 as ::core::ffi::c_int,
                            4 as u_int,
                            idx as ::core::ffi::c_int,
                            c,
                            (*ictx).input_end,
                        );
                        s = next;
                    } else {
                        input_add_request(ictx, INPUT_REQUEST_PALETTE, idx as ::core::ffi::c_int);
                        s = next;
                    }
                } else {
                    c = colour_parseX11(s);
                    if c == -(1 as ::core::ffi::c_int) {
                        s = next;
                    } else {
                        if colour_palette_set(palette.as_mut(), idx as ::core::ffi::c_int, c) != 0 {
                            redraw = 1 as ::core::ffi::c_int;
                        }
                        s = next;
                    }
                }
            }
        }
        if bad != 0 {
            log_debug(c"bad OSC 4: %s".as_ptr(), fmt_args![p]);
        }
        if redraw != 0 {
            screen_write_fullredraw(&mut (*ictx).ctx);
        }
    }
}
unsafe fn input_osc_8(mut ictx: *mut input_ctx, p: &CStr) {
    unsafe {
        let mut p = p.as_ptr();
        let mut current_block: u64;
        let hl = (*(*ictx).ctx.s).hyperlinks_ref();
        let mut gc: *mut grid_cell = &mut (*ictx).cell.cell;
        let mut start: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut uri: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut id: Option<CString> = None;
        start = p;
        loop {
            end = strpbrk(start, c":;".as_ptr());
            if end.is_null() {
                current_block = 10886091980245723256;
                break;
            }
            if end.offset_from(start) as ::core::ffi::c_long >= 4 as ::core::ffi::c_long
                && strncmp(start, c"id=".as_ptr(), 3 as size_t) == 0 as ::core::ffi::c_int
            {
                if id.is_some() {
                    current_block = 11927621148471894391;
                    break;
                }
                id = Some(
                    CString::new(::core::slice::from_raw_parts(
                        start.offset(3 as ::core::ffi::c_int as isize) as *const u8,
                        (end.offset_from(start) as usize).saturating_sub(3),
                    ))
                    .expect("hyperlink id has no NUL"),
                );
            }
            if *end as ::core::ffi::c_int == ';' as i32 {
                current_block = 10886091980245723256;
                break;
            }
            start = end.offset(1 as ::core::ffi::c_int as isize);
        }
        if current_block == 10886091980245723256
            && !(end.is_null() || *end as ::core::ffi::c_int != ';' as i32)
        {
            uri = end.offset(1 as ::core::ffi::c_int as isize);
            if *uri as ::core::ffi::c_int == '\0' as i32 {
                (*gc).link = 0 as u_int;
                return;
            }
            let Some(hl) = hl else {
                return;
            };
            (*gc).link = hyperlinks_put(hl, CStr::from_ptr(uri), id.as_deref());
            if id.is_none() {
                log_debug(
                    c"hyperlink (anonymous) %s = %u".as_ptr(),
                    fmt_args![uri, (*gc).link],
                );
            } else {
                log_debug(
                    c"hyperlink (id=%s) %s = %u".as_ptr(),
                    fmt_args![id.as_ref().unwrap().as_ptr(), uri, (*gc).link],
                );
            }
            return;
        }
        log_debug(c"bad OSC 8 %s".as_ptr(), fmt_args![p]);
    }
}
unsafe fn input_set_progress_bar(
    mut ictx: *mut input_ctx,
    mut state: progress_bar_state,
    mut p: ::core::ffi::c_int,
) {
    unsafe {
        screen_set_progress_bar((*ictx).ctx.s, state, p);
        if !(*ictx).pane().is_null() {
            server_redraw_window_borders((*(*ictx).pane()).window);
            server_status_window((*(*ictx).pane()).window);
        }
    }
}
unsafe fn input_osc_9(mut ictx: *mut input_ctx, p: &CStr) {
    unsafe {
        let mut p = p.as_ptr();
        let mut current_block: u64;
        let mut pb: *const ::core::ffi::c_char = p;
        let mut state: progress_bar_state = PROGRESS_BAR_HIDDEN;
        let mut progress: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let fresh4 = pb;
        pb = pb.offset(1);
        if *fresh4 as ::core::ffi::c_int != '4' as i32 {
            return;
        }
        if *pb as ::core::ffi::c_int == '\0' as i32
            || *pb as ::core::ffi::c_int == ';' as i32
                && *pb.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '\0' as i32
        {
            return;
        }
        let fresh5 = pb;
        pb = pb.offset(1);
        if *fresh5 as ::core::ffi::c_int != ';' as i32 {
            return;
        }
        if !((*pb as ::core::ffi::c_int) < '0' as i32 || *pb as ::core::ffi::c_int > '4' as i32) {
            let fresh6 = pb;
            pb = pb.offset(1);
            state = (*fresh6 as ::core::ffi::c_int - '0' as i32) as progress_bar_state;
            if *pb as ::core::ffi::c_int == '\0' as i32
                || *pb as ::core::ffi::c_int == ';' as i32
                    && *pb.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\0' as i32
            {
                input_set_progress_bar(ictx, state, -(1 as ::core::ffi::c_int));
                return;
            }
            let fresh7 = pb;
            pb = pb.offset(1);
            if !(*fresh7 as ::core::ffi::c_int != ';' as i32) {
                loop {
                    if !(*pb as ::core::ffi::c_int >= '0' as i32
                        && *pb as ::core::ffi::c_int <= '9' as i32)
                    {
                        current_block = 10599921512955367680;
                        break;
                    }
                    if progress > 100 as ::core::ffi::c_int {
                        current_block = 5988496509038859168;
                        break;
                    }
                    let fresh8 = pb;
                    pb = pb.offset(1);
                    progress = progress * 10 as ::core::ffi::c_int + *fresh8 as ::core::ffi::c_int
                        - '0' as i32;
                }
                match current_block {
                    5988496509038859168 => {}
                    _ => {
                        if !(*pb as ::core::ffi::c_int != '\0' as i32
                            || progress < 0 as ::core::ffi::c_int
                            || progress > 100 as ::core::ffi::c_int)
                        {
                            input_set_progress_bar(ictx, state, progress);
                            return;
                        }
                    }
                }
            }
        }
        log_debug(c"bad OSC 9;4 %s".as_ptr(), fmt_args![p]);
    }
}
unsafe fn input_osc_10(mut ictx: *mut input_ctx, p: &CStr) {
    unsafe {
        let mut p = p.as_ptr();
        let mut wp: *mut window_pane = (*ictx).pane();
        let mut defaults = grid_default_cell;
        let mut c: ::core::ffi::c_int = 0;
        if strcmp(p, c"?".as_ptr()) == 0 as ::core::ffi::c_int {
            if wp.is_null() {
                return;
            }
            c = window_pane_get_fg_control_client(wp);
            if c == -(1 as ::core::ffi::c_int) {
                tty_default_colours(&mut defaults, wp);
                if defaults.fg == 8 as ::core::ffi::c_int || defaults.fg == 9 as ::core::ffi::c_int
                {
                    c = window_pane_get_fg(wp);
                } else {
                    c = defaults.fg;
                }
            }
            input_osc_colour_reply(
                ictx,
                1 as ::core::ffi::c_int,
                10 as u_int,
                0 as ::core::ffi::c_int,
                c,
                (*ictx).input_end,
            );
            return;
        }
        c = colour_parseX11(p);
        if c == -(1 as ::core::ffi::c_int) {
            log_debug(c"bad OSC 10: %s".as_ptr(), fmt_args![p]);
            return;
        }
        if !(*ictx).palette().is_null() {
            (*(*ictx).palette()).fg = c;
            if !wp.is_null() {
                (*wp).flags |= PANE_STYLECHANGED;
            }
            screen_write_fullredraw(&mut (*ictx).ctx);
        }
    }
}
unsafe fn input_osc_110(mut ictx: *mut input_ctx, p: &CStr) {
    unsafe {
        let mut p = p.as_ptr();
        let mut wp: *mut window_pane = (*ictx).pane();
        if *p as ::core::ffi::c_int != '\0' as i32 {
            return;
        }
        if !(*ictx).palette().is_null() {
            (*(*ictx).palette()).fg = 8 as ::core::ffi::c_int;
            if !wp.is_null() {
                (*wp).flags |= PANE_STYLECHANGED;
            }
            screen_write_fullredraw(&mut (*ictx).ctx);
        }
    }
}
unsafe fn input_osc_11(mut ictx: *mut input_ctx, p: &CStr) {
    unsafe {
        let mut p = p.as_ptr();
        let mut wp: *mut window_pane = (*ictx).pane();
        let mut c: ::core::ffi::c_int = 0;
        if strcmp(p, c"?".as_ptr()) == 0 as ::core::ffi::c_int {
            if wp.is_null() {
                return;
            }
            c = window_pane_get_bg(wp);
            input_osc_colour_reply(
                ictx,
                1 as ::core::ffi::c_int,
                11 as u_int,
                0 as ::core::ffi::c_int,
                c,
                (*ictx).input_end,
            );
            return;
        }
        c = colour_parseX11(p);
        if c == -(1 as ::core::ffi::c_int) {
            log_debug(c"bad OSC 11: %s".as_ptr(), fmt_args![p]);
            return;
        }
        if !(*ictx).palette().is_null() {
            (*(*ictx).palette()).bg = c;
            if !wp.is_null() {
                (*wp).flags |= PANE_STYLECHANGED | PANE_THEMECHANGED;
            }
            screen_write_fullredraw(&mut (*ictx).ctx);
        }
    }
}
unsafe fn input_osc_111(mut ictx: *mut input_ctx, p: &CStr) {
    unsafe {
        let mut p = p.as_ptr();
        let mut wp: *mut window_pane = (*ictx).pane();
        if *p as ::core::ffi::c_int != '\0' as i32 {
            return;
        }
        if !(*ictx).palette().is_null() {
            (*(*ictx).palette()).bg = 8 as ::core::ffi::c_int;
            if !wp.is_null() {
                (*wp).flags |= PANE_STYLECHANGED | PANE_THEMECHANGED;
            }
            screen_write_fullredraw(&mut (*ictx).ctx);
        }
    }
}
unsafe fn input_osc_12(mut ictx: *mut input_ctx, p: &CStr) {
    unsafe {
        let mut p = p.as_ptr();
        let mut wp: *mut window_pane = (*ictx).pane();
        let mut c: ::core::ffi::c_int = 0;
        if strcmp(p, c"?".as_ptr()) == 0 as ::core::ffi::c_int {
            if !wp.is_null() {
                c = (*(*ictx).ctx.s).ccolour;
                if c == -(1 as ::core::ffi::c_int) {
                    c = (*(*ictx).ctx.s).default_ccolour;
                }
                input_osc_colour_reply(
                    ictx,
                    1 as ::core::ffi::c_int,
                    12 as u_int,
                    0 as ::core::ffi::c_int,
                    c,
                    (*ictx).input_end,
                );
            }
            return;
        }
        c = colour_parseX11(p);
        if c == -(1 as ::core::ffi::c_int) {
            log_debug(c"bad OSC 12: %s".as_ptr(), fmt_args![p]);
            return;
        }
        screen_set_cursor_colour((*ictx).ctx.s, c);
    }
}
unsafe fn input_osc_112(mut ictx: *mut input_ctx, p: &CStr) {
    unsafe {
        let mut p = p.as_ptr();
        if *p as ::core::ffi::c_int == '\0' as i32 {
            screen_set_cursor_colour((*ictx).ctx.s, -(1 as ::core::ffi::c_int));
        }
    }
}
unsafe fn input_osc_133(mut ictx: *mut input_ctx, p: &CStr) {
    unsafe {
        let mut p = p.as_ptr();
        let mut gd: *mut grid = screen_grid_ptr(&mut *(*ictx).ctx.s);
        let mut line: u_int = (*(*ictx).ctx.s).cy.wrapping_add((*gd).hsize);
        let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
        if line > (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int) {
            return;
        }
        gl = grid_get_line(&mut *gd, line);
        match *p as ::core::ffi::c_int {
            65 => {
                (*gl).flags |= GRID_LINE_START_PROMPT;
            }
            67 => {
                (*gl).flags |= GRID_LINE_START_OUTPUT;
            }
            _ => {}
        };
    }
}
unsafe fn input_osc_52_reply(mut ictx: *mut input_ctx, mut clip: ::core::ffi::c_char) {
    unsafe {
        let mut ev: Stream = (*ictx).event;
        let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
        let mut state: ::core::ffi::c_int = 0;
        let mut buf: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut len: size_t = 0;
        state = options_get_number(global_options, c"get-clipboard".as_ptr()) as ::core::ffi::c_int;
        if state == 0 as ::core::ffi::c_int {
            return;
        }
        if state == 1 as ::core::ffi::c_int {
            pb = paste_get_top(None);
            if pb.is_null() {
                return;
            }
            let bytes = paste_buffer_data(&*pb);
            buf = bytes.as_ptr() as *const ::core::ffi::c_char;
            len = bytes.len() as size_t;
            if (*ictx).input_end as ::core::ffi::c_uint
                == INPUT_END_BEL as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                input_reply_clipboard(ev, buf, len, c"\x07", clip);
            } else {
                input_reply_clipboard(ev, buf, len, c"\x1B\\", clip);
            }
            return;
        }
        input_add_request(
            ictx,
            INPUT_REQUEST_CLIPBOARD,
            (*ictx).input_end as ::core::ffi::c_int,
        );
    }
}
unsafe fn input_osc_52_parse(
    mut ictx: *mut input_ctx,
    p: &CStr,
    mut clip: *mut ::core::ffi::c_char,
) -> Option<Vec<u8>> {
    unsafe {
        let p = p.as_ptr();
        let mut end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut len: size_t = 0;
        let mut allow: *const ::core::ffi::c_char = c"cpqs01234567".as_ptr();
        let mut i: u_int = 0;
        let mut j: u_int = 0 as u_int;
        if options_get_number(global_options, c"set-clipboard".as_ptr())
            != 2 as ::core::ffi::c_longlong
        {
            return None;
        }
        end = strchr(p, ';' as i32);
        if end.is_null() {
            return None;
        }
        end = end.offset(1);
        if *end as ::core::ffi::c_int == '\0' as i32 {
            return None;
        }
        log_debug(
            c"%s: %s".as_ptr(),
            fmt_args![c"input_osc_52_parse".as_ptr(), end],
        );
        i = 0 as u_int;
        while !std::ptr::eq(p.offset(i as isize), end) {
            if !strchr(allow, *p.offset(i as isize) as ::core::ffi::c_int).is_null()
                && strchr(clip, *p.offset(i as isize) as ::core::ffi::c_int).is_null()
            {
                let fresh3 = j;
                j = j.wrapping_add(1);
                *clip.offset(fresh3 as isize) = *p.offset(i as isize);
            }
            i = i.wrapping_add(1);
        }
        log_debug(
            c"%s: %.*s %s".as_ptr(),
            fmt_args![
                c"input_osc_52_parse".as_ptr(),
                (end.offset_from(p) as ::core::ffi::c_long - 1 as ::core::ffi::c_long)
                    as ::core::ffi::c_int,
                p,
                clip
            ],
        );
        if strcmp(end, c"?".as_ptr()) == 0 as ::core::ffi::c_int {
            input_osc_52_reply(ictx, *clip);
            return None;
        }
        len = strlen(end)
            .wrapping_add(3 as size_t)
            .wrapping_div(4 as size_t)
            .wrapping_mul(3 as size_t);
        if len == 0 as size_t {
            return None;
        }
        let mut out: Vec<u8> = vec![0_u8; len as usize];
        let outlen = __b64_pton(end, out.as_mut_ptr(), len);
        if outlen == -(1 as ::core::ffi::c_int) {
            return None;
        }
        out.truncate(outlen as usize);
        Some(out)
    }
}
unsafe fn input_osc_52(mut ictx: *mut input_ctx, p: &CStr) {
    unsafe {
        let mut p = p.as_ptr();
        let mut wp: *mut window_pane = (*ictx).pane();
        let mut ctx = screen_write_ctx::default();
        let mut clip: [::core::ffi::c_char; 13] = ::core::mem::transmute::<
            [u8; 13],
            [::core::ffi::c_char; 13],
        >(*b"\0\0\0\0\0\0\0\0\0\0\0\0\0");
        let Some(mut out) = input_osc_52_parse(
            ictx,
            CStr::from_ptr(p),
            &raw mut clip as *mut ::core::ffi::c_char,
        ) else {
            return;
        };
        if wp.is_null() {
            if (*ictx).client().is_null() {
                return;
            }
            tty_set_selection(
                &mut (*(*ictx).client()).tty,
                &raw mut clip as *mut ::core::ffi::c_char,
                out.as_ptr() as *const ::core::ffi::c_char,
                out.len() as size_t,
            );
            paste_add(::core::ptr::null::<::core::ffi::c_char>(), out);
        } else {
            screen_write_start_pane(&mut ctx, wp, None);
            screen_write_setselection(
                &mut ctx,
                &raw mut clip as *mut ::core::ffi::c_char,
                out.as_mut_ptr(),
                out.len() as u_int,
            );
            screen_write_stop(&mut ctx);
            notify_pane(c"pane-set-clipboard".as_ptr(), wp);
            paste_add(::core::ptr::null::<::core::ffi::c_char>(), out);
        };
    }
}
unsafe fn input_osc_104(mut ictx: *mut input_ctx, p: &CStr) {
    unsafe {
        let mut p = p.as_ptr();
        let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut idx: ::core::ffi::c_long = 0;
        let mut bad: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut redraw: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if *p as ::core::ffi::c_int == '\0' as i32 {
            colour_palette_clear((*ictx).palette().as_mut());
            screen_write_fullredraw(&mut (*ictx).ctx);
            return;
        }
        let mut copy = CStr::from_ptr(p).to_bytes_with_nul().to_vec();
        s = copy.as_mut_ptr() as *mut ::core::ffi::c_char;
        while *s as ::core::ffi::c_int != '\0' as i32 {
            idx = strtol(s, &raw mut s, 10 as ::core::ffi::c_int);
            if *s as ::core::ffi::c_int != '\0' as i32 && *s as ::core::ffi::c_int != ';' as i32 {
                bad = 1 as ::core::ffi::c_int;
                break;
            } else if idx < 0 as ::core::ffi::c_long || idx >= 256 as ::core::ffi::c_long {
                bad = 1 as ::core::ffi::c_int;
                break;
            } else {
                if colour_palette_set(
                    (*ictx).palette().as_mut(),
                    idx as ::core::ffi::c_int,
                    -(1 as ::core::ffi::c_int),
                ) != 0
                {
                    redraw = 1 as ::core::ffi::c_int;
                }
                if *s as ::core::ffi::c_int == ';' as i32 {
                    s = s.offset(1);
                }
            }
        }
        if bad != 0 {
            log_debug(c"bad OSC 104: %s".as_ptr(), fmt_args![p]);
        }
        if redraw != 0 {
            screen_write_fullredraw(&mut (*ictx).ctx);
        }
    }
}
pub unsafe fn input_reply_clipboard(
    mut bev: Stream,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    end: &CStr,
    mut clip: ::core::ffi::c_char,
) {
    unsafe {
        let mut out: Vec<u8> = Vec::new();
        let mut outlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if !buf.is_null() && len != 0 as size_t {
            if len
                >= (INT_MAX as size_t)
                    .wrapping_mul(3 as size_t)
                    .wrapping_div(4 as size_t)
                    .wrapping_sub(1 as size_t)
            {
                return;
            }
            outlen = (4 as size_t)
                .wrapping_mul(len.wrapping_add(2 as size_t).wrapping_div(3 as size_t))
                .wrapping_add(1 as size_t) as ::core::ffi::c_int;
            out = vec![0_u8; outlen as usize];
            outlen = __b64_ntop(
                buf as *const ::core::ffi::c_uchar,
                len,
                out.as_mut_ptr() as *mut ::core::ffi::c_char,
                outlen as size_t,
            );
            if outlen == -(1 as ::core::ffi::c_int) {
                return;
            }
        }
        bev.write(b"\x1B]52;\0".as_ptr(), 5 as size_t);
        if clip as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            bev.write(&raw mut clip as *const u8, 1 as size_t);
        }
        bev.write(b";\0".as_ptr(), 1 as size_t);
        if outlen != 0 as ::core::ffi::c_int {
            bev.write(out.as_ptr(), outlen as size_t);
        }
        bev.write(end.as_ptr() as *const u8, end.to_bytes().len());
    }
}
pub fn input_set_buffer_size(mut buffer_size: size_t) {
    unsafe {
        log_debug(
            c"%s: %lu -> %lu".as_ptr(),
            fmt_args![
                c"input_set_buffer_size".as_ptr(),
                input_buffer_size,
                buffer_size
            ],
        );
        input_buffer_size = buffer_size;
    }
}
unsafe fn input_request_timer_callback(ictx: *mut input_ctx) {
    unsafe {
        let t: uint64_t = get_timer();
        for ir in foreach_owned_safe(&raw mut (*ictx).requests) {
            if (*ir).t.wrapping_add(INPUT_REQUEST_TIMEOUT as uint64_t) < t {
                if (*ir).type_0 as ::core::ffi::c_uint
                    == INPUT_REQUEST_QUEUE as ::core::ffi::c_int as ::core::ffi::c_uint
                    && let Some(ictx) = (*ir).ictx.upgrade()
                {
                    input_send_reply(ictx.as_ptr(), (*ir).data.as_deref().unwrap_or(c""));
                }
                input_free_request(ir);
            }
        }
        if (*ictx).request_count != 0 as u_int {
            input_start_request_timer(ictx);
        }
    }
}
unsafe fn input_start_request_timer(mut ictx: *mut input_ctx) {
    unsafe {
        let mut tv = timeval::from_usecs(100000 as __suseconds_t);
        (*ictx).request_timer.disarm();
        (*ictx).request_timer.arm(tv);
    }
}
/// The parser a request belongs to, observed rather than held: a request
/// outlives its parser only when the parser's owner has given it up first.
unsafe fn ictx_weak(ictx: *mut input_ctx) -> InputCtxWeak {
    unsafe { (*ictx).owner.clone().expect("a parser holds itself") }
}

unsafe fn input_make_request(
    mut ictx: *mut input_ctx,
    mut type_0: input_request_type,
) -> *mut input_request {
    unsafe {
        let mut request = Box::new(input_request {
            client: None,
            ictx: ictx_weak(ictx),
            type_0,
            t: get_timer(),
            end: INPUT_END_ST,
            idx: 0,
            data: None,
        });
        let ir = &raw mut *request;
        (*ictx).request_count = (*ictx).request_count.wrapping_add(1);
        if (*ictx).request_count == 1 as u_int {
            input_start_request_timer(ictx);
        }
        (*ictx).requests.push(request);
        ir
    }
}
/// Takes `ir` off `list`, which is one of the two it sits on.
unsafe fn input_unlink_request(list: *mut input_requests, ir: *mut input_request) {
    unsafe {
        if let Some(at) = (*list).iter().position(|&waiting| waiting == ir) {
            (*list).remove(at);
        }
    }
}

unsafe fn input_free_request(ir: *mut input_request) {
    unsafe {
        if let Some(client) = (*ir).client.as_ref().and_then(ClientWeak::upgrade) {
            input_unlink_request(&raw mut (*client.as_ptr()).input_requests, ir);
        }
        let Some(ictx) = (*ir).ictx.upgrade() else {
            return;
        };
        let ictx = ictx.as_ptr();
        (*ictx).request_count = (*ictx).request_count.wrapping_sub(1);
        if let Some(at) = (*ictx)
            .requests
            .iter()
            .position(|waiting| std::ptr::eq(&raw const **waiting, ir))
        {
            (*ictx).requests.remove(at);
        }
    }
}
unsafe fn input_add_request(
    mut ictx: *mut input_ctx,
    mut type_0: input_request_type,
    mut idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut wp: *mut window_pane = (*ictx).pane();
        let mut w: *mut window = ::core::ptr::null_mut::<window>();
        let mut c: *mut client = ::core::ptr::null_mut::<client>();
        let mut ir: *mut input_request = ::core::ptr::null_mut::<input_request>();
        if wp.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        w = (*wp).window;
        for loop_0 in client_walk() {
            if !((*loop_0).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0)
                && !((*loop_0).session.is_null() || session_has((*loop_0).session, w) == 0)
                && !(!(*loop_0).tty.flags & TTY_STARTED != 0)
            {
                if c.is_null() {
                    c = loop_0;
                } else if if (*loop_0).activity_time.tv_sec == (*c).activity_time.tv_sec {
                    ((*loop_0).activity_time.tv_usec > (*c).activity_time.tv_usec)
                        as ::core::ffi::c_int
                } else {
                    ((*loop_0).activity_time.tv_sec > (*c).activity_time.tv_sec)
                        as ::core::ffi::c_int
                } != 0
                {
                    c = loop_0;
                }
            }
        }
        if c.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        ir = input_make_request(ictx, type_0);
        (*ir).client = client_ref_from_ptr(c).map(|c| c.downgrade());
        (*ir).idx = idx;
        (*ir).end = (*ictx).input_end;
        (*c).input_requests.push(ir);
        match type_0 {
            INPUT_REQUEST_PALETTE => {
                let s = xasprintf(c"\x1B]4;%d;?\x1B\\".as_ptr(), fmt_args![idx]);
                tty_puts(&mut (*c).tty, s.as_ptr());
            }
            INPUT_REQUEST_CLIPBOARD => {
                tty_putcode_ss(&mut (*c).tty, TTYC_MS, c"".as_ptr(), c"?".as_ptr());
            }
            _ => {}
        }
        0 as ::core::ffi::c_int
    }
}
unsafe fn input_request_palette_reply(mut ir: *mut input_request, data: &InputRequestData) {
    unsafe {
        let &InputRequestData::Palette { idx, c } = data else {
            return;
        };
        let Some(ictx) = (*ir).ictx.upgrade() else {
            return;
        };
        input_osc_colour_reply(
            ictx.as_ptr(),
            0 as ::core::ffi::c_int,
            4 as u_int,
            idx,
            c,
            (*ir).end,
        );
    }
}
unsafe fn input_request_clipboard_reply(mut ir: *mut input_request, data: &InputRequestData) {
    unsafe {
        let Some(ictx) = (*ir).ictx.upgrade() else {
            return;
        };
        let mut ev: Stream = (*ictx.as_ptr()).event;
        let InputRequestData::Clipboard { clip, data } = data else {
            return;
        };
        let mut state: ::core::ffi::c_int = 0;
        state = options_get_number(global_options, c"get-clipboard".as_ptr()) as ::core::ffi::c_int;
        if state == 0 as ::core::ffi::c_int || state == 1 as ::core::ffi::c_int {
            return;
        }
        if state == 3 as ::core::ffi::c_int {
            paste_add(::core::ptr::null::<::core::ffi::c_char>(), data.clone());
        }
        let buf = data.as_ptr() as *const ::core::ffi::c_char;
        let len = data.len() as size_t;
        if (*ir).idx == INPUT_END_BEL as ::core::ffi::c_int {
            input_reply_clipboard(ev, buf, len, c"\x07", *clip);
        } else {
            input_reply_clipboard(ev, buf, len, c"\x1B\\", *clip);
        };
    }
}
pub unsafe fn input_request_reply(
    mut c: *mut client,
    mut type_0: input_request_type,
    data: &InputRequestData,
) {
    unsafe {
        let mut found: *mut input_request = ::core::ptr::null_mut::<input_request>();
        let mut complete: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        for ir in foreach_safe(&raw mut (*c).input_requests) {
            if (*ir).type_0 as ::core::ffi::c_uint != type_0 as ::core::ffi::c_uint {
                input_free_request(ir);
            } else if type_0 as ::core::ffi::c_uint
                == INPUT_REQUEST_PALETTE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let &InputRequestData::Palette { idx, .. } = data else {
                    return;
                };
                if idx != (*ir).idx {
                    input_free_request(ir);
                } else {
                    found = ir;
                    break;
                }
            } else if type_0 as ::core::ffi::c_uint
                == INPUT_REQUEST_CLIPBOARD as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                found = ir;
                break;
            }
        }
        if found.is_null() {
            return;
        }
        let Some(owner) = (*found).ictx.upgrade() else {
            return;
        };
        for ir in foreach_owned_safe(&raw mut (*owner.as_ptr()).requests) {
            if complete != 0
                && (*ir).type_0 as ::core::ffi::c_uint
                    != INPUT_REQUEST_QUEUE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                break;
            }
            if (*ir).type_0 as ::core::ffi::c_uint
                == INPUT_REQUEST_QUEUE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if let Some(ictx) = (*ir).ictx.upgrade() {
                    input_send_reply(ictx.as_ptr(), (*ir).data.as_deref().unwrap_or(c""));
                }
            } else if ir == found {
                if (*ir).type_0 as ::core::ffi::c_uint
                    == INPUT_REQUEST_PALETTE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    input_request_palette_reply(ir, data);
                } else if (*ir).type_0 as ::core::ffi::c_uint
                    == INPUT_REQUEST_CLIPBOARD as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    input_request_clipboard_reply(ir, data);
                }
                complete = 1 as ::core::ffi::c_int;
            }
            input_free_request(ir);
        }
    }
}
pub unsafe fn input_cancel_requests(c: *mut client) {
    unsafe {
        for ir in foreach_safe(&raw mut (*c).input_requests) {
            input_free_request(ir);
        }
    }
}
unsafe fn input_report_current_theme(mut ictx: *mut input_ctx) {
    unsafe {
        let mut wp: *mut window_pane = (*ictx).pane();
        if !wp.is_null() {
            (*wp).last_theme = window_pane_get_theme(wp);
            (*wp).flags &= !PANE_THEMECHANGED;
            match (*wp).last_theme {
                THEME_DARK => {
                    log_debug(
                        c"%s: %%%u dark theme".as_ptr(),
                        fmt_args![c"input_report_current_theme".as_ptr(), (*wp).id],
                    );
                    input_reply(
                        ictx,
                        0 as ::core::ffi::c_int,
                        c"\x1B[?997;1n".as_ptr(),
                        fmt_args![],
                    );
                }
                THEME_LIGHT => {
                    log_debug(
                        c"%s: %%%u light theme".as_ptr(),
                        fmt_args![c"input_report_current_theme".as_ptr(), (*wp).id],
                    );
                    input_reply(
                        ictx,
                        0 as ::core::ffi::c_int,
                        c"\x1B[?997;2n".as_ptr(),
                        fmt_args![],
                    );
                }
                THEME_UNKNOWN => {
                    log_debug(
                        c"%s: %%%u unknown theme".as_ptr(),
                        fmt_args![c"input_report_current_theme".as_ptr(), (*wp).id],
                    );
                }
                _ => {}
            }
        }
    }
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
