use core::ffi::c_int;
use crate::WindowPane;
use crate::compat::strtonum;
use crate::ffi::{__b64_ntop, __b64_pton};
use crate::fmt_args;
use crate::fmt_engine::{FmtArg, format_alloc};
use crate::grid::{Grid, Hyperlinks, grid_cells_look_equal, grid_set_tab};
use crate::grid::{grid_default_cell};
use crate::log::{fatalx, log_debug, log_get_level};
use crate::notify::notify_pane;
use crate::options::{OptionsEngine, RustOptionsEngine};
use crate::pane_identity::PaneIdentity;
use crate::paste::{
    PasteBufferStore, paste_buffer_limit, with_paste_buffers, with_paste_buffers_mut,
};
use crate::reactor::Timer;
use crate::screen::Screen;
use crate::screen::screen_write_init_ctx;
use crate::screen::{RustScreenWriteCtx, ScreenWriteCtx, screen_write_ctx_on_pane};
use crate::server::client_walk;

pub use crate::consts::{
    ALL_MOUSE_MODES, CLIENT_UNATTACHEDFLAGS, COLOUR_FLAG_256, EXTENDED_KEY_MODES,
    GRID_ATTR_ALL_UNDERSCORE, GRID_ATTR_BLINK, GRID_ATTR_BRIGHT, GRID_ATTR_CHARSET, GRID_ATTR_DIM,
    GRID_ATTR_HIDDEN, GRID_ATTR_ITALICS, GRID_ATTR_OVERLINE, GRID_ATTR_REVERSE,
    GRID_ATTR_STRIKETHROUGH, GRID_ATTR_UNDERSCORE, GRID_ATTR_UNDERSCORE_2, GRID_ATTR_UNDERSCORE_3,
    GRID_ATTR_UNDERSCORE_4, GRID_ATTR_UNDERSCORE_5, GRID_LINE_START_OUTPUT, GRID_LINE_START_PROMPT,
    INPUT_BUF_DEFAULT_SIZE, INPUT_REQUEST_CLIPBOARD, INPUT_REQUEST_PALETTE, INT_MAX,
    MODE_BRACKETPASTE, MODE_CRLF, MODE_CURSOR, MODE_CURSOR_BLINKING, MODE_CURSOR_BLINKING_SET,
    MODE_CURSOR_VERY_VISIBLE, MODE_FOCUSON, MODE_INSERT, MODE_KCURSOR, MODE_KEYS_EXTENDED,
    MODE_KEYS_EXTENDED_2, MODE_KKEYPAD, MODE_MOUSE_ALL, MODE_MOUSE_BUTTON, MODE_MOUSE_SGR,
    MODE_MOUSE_STANDARD, MODE_MOUSE_UTF8, MODE_ORIGIN, MODE_SYNC, MODE_THEME_UPDATES, MODE_WRAP,
    PANE_CHANGED, PANE_REDRAW, PANE_STYLECHANGED, PANE_THEMECHANGED, PANE_UNSEENCHANGES,
    SCREEN_CURSOR_BAR, SCREEN_CURSOR_BLOCK, SCREEN_CURSOR_DEFAULT, SCREEN_CURSOR_UNDERLINE,
    THEME_DARK, THEME_LIGHT, THEME_UNKNOWN, TTY_STARTED, TTYC_MS, UTF8_ERROR, UTF8_MORE,
    WINDOW_BELL,
};
use crate::style::{ColourEngine, RustColourEngine};
use crate::text::{RustUtf8VisModel, Utf8VisModel, utf8_append, utf8_copy, utf8_open, utf8_set};
use crate::tmux::{get_timer, getversion};
use crate::tmux::{global_options, global_w_options};
use crate::tty::{tty_default_colours, tty_putcode_ss, tty_puts, tty_set_selection};
pub use crate::types::*;
use crate::window::{
    window_pane_get_bg, window_pane_get_fg,
    window_pane_get_fg_control_client, window_pane_get_theme,
};
use crate::xmalloc::xasprintf;
use ::core::cell::{Ref, RefCell, RefMut};
use ::std::ffi::{CStr, CString};
use ::std::rc::{Rc, Weak};
use bytes::Buf as _;

#[repr(C)]
pub struct input_ctx {
    /// What the parser is parsing output for.
    pub(crate) owner_of: InputOwner,
    pub event: Stream,
    pub cell: input_cell,
    pub old_cell: input_cell,
    pub old_cx: u_int,
    pub old_cy: u_int,
    pub old_mode: core::ffi::c_int,
    pub interm_buf: [u_char; 4],
    pub interm_len: size_t,
    pub param_buf: [u_char; 64],
    pub param_len: size_t,
    pub input_buf: Vec<u8>,
    pub input_end: input_end_type,
    pub param_list: [InputParam; 24],
    pub param_list_len: u_int,
    pub utf8data: utf8_data,
    pub utf8started: core::ffi::c_int,
    pub ch: core::ffi::c_int,
    pub last: utf8_data,
    /// The state the parser is in. A state is a table entry that lives for as
    /// long as the process does, so the parser always has one.
    pub state: &'static input_state,
    pub flags: core::ffi::c_int,
    pub requests: input_request_list,
    pub(crate) next_request_id: u64,
    pub request_count: u_int,
    pub request_timer: TimerHandle,
    pub since_ground: Option<Box<ByteBuffer>>,
    pub ground_timer: TimerHandle,
    /// The parser's observation of itself, which is what a request it made
    /// holds it by.
    pub(crate) owner: Option<InputCtxWeak>,
}
#[repr(C)]
pub struct input_request {
    /// The client the request went to, observed rather than held, so that a
    /// client which goes before the answer arrives leaves nothing behind.
    pub(crate) c: Option<ClientWeak>,
    pub(crate) ictx: InputCtxWeak,
    pub(crate) id: u64,
    pub type_0: input_request_type,
    pub t: uint64_t,
    pub end: input_end_type,
    pub idx: core::ffi::c_int,
    pub data: Option<CString>,
}
#[derive(Clone)]
pub struct input_request_handle {
    pub(crate) ictx: InputCtxWeak,
    pub(crate) id: u64,
}

impl input_request_handle {
    fn matches(&self, other: &Self) -> bool {
        self.id == other.id && self.ictx.0.ptr_eq(&other.ictx.0)
    }
}
pub type input_end_type = core::ffi::c_uint;
pub const INPUT_END_BEL: input_end_type = 1;
pub const INPUT_END_ST: input_end_type = 0;
pub const INPUT_REQUEST_QUEUE: input_request_type = 2;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct input_state {
    pub name: &'static CStr,
    pub enter: Option<InputStateAction>,
    pub exit: Option<InputStateAction>,
    pub transitions: &'static [input_transition],
}
/// The action performed when entering or leaving a parser state.
#[derive(Copy, Clone)]
pub enum InputStateAction {
    Ground,
    Clear,
    EnterDcs,
    EnterOsc,
    ExitOsc,
    EnterApc,
    ExitApc,
    EnterRename,
    ExitRename,
}

impl InputStateAction {
    unsafe fn call(self, ictx: &mut input_ctx, sctx: &mut RustScreenWriteCtx<'_>) {
        unsafe {
            match self {
                Self::Ground => input_ground(ictx),
                Self::Clear => input_clear(ictx),
                Self::EnterDcs => input_enter_dcs(ictx),
                Self::EnterOsc => input_enter_osc(ictx),
                Self::ExitOsc => input_exit_osc(ictx, sctx),
                Self::EnterApc => input_enter_apc(ictx),
                Self::ExitApc => input_exit_apc(ictx, sctx),
                Self::EnterRename => input_enter_rename(ictx),
                Self::ExitRename => input_exit_rename(ictx),
            }
        }
    }
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
    unsafe fn call(
        self,
        ictx: &mut input_ctx,
        sctx: &mut RustScreenWriteCtx<'_>,
    ) -> core::ffi::c_int {
        unsafe {
            match self {
                InputHandler::Print => input_print(&mut *ictx, sctx),
                InputHandler::C0Dispatch => input_c0_dispatch(&mut *ictx, sctx),
                InputHandler::EscDispatch => input_esc_dispatch(ictx, sctx),
                InputHandler::CsiDispatch => input_csi_dispatch(ictx, sctx),
                InputHandler::DcsDispatch => input_dcs_dispatch(ictx, sctx),
                InputHandler::Parameter => input_parameter(&mut *ictx),
                InputHandler::Intermediate => input_intermediate(&mut *ictx),
                InputHandler::Input => input_input(&mut *ictx),
                InputHandler::TopBitSet => input_top_bit_set(&mut *ictx, sctx),
                InputHandler::EndBel => input_end_bel(&mut *ictx),
            }
        }
    }
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct input_transition {
    pub first: core::ffi::c_int,
    pub last: core::ffi::c_int,
    pub handler: Option<InputHandler>,
    /// The state the transition moves to, or nothing for one that stays where
    /// it is.
    pub state: Option<&'static input_state>,
}

/// One parameter of a control sequence: left out, a number, or the
/// colon-joined string a caller takes apart for itself.
pub enum InputParam {
    Missing,
    Number(core::ffi::c_int),
    Str(CString),
}

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
pub type input_esc_type = core::ffi::c_uint;
pub type input_csi_type = core::ffi::c_uint;

pub const INPUT_REQUEST_TIMEOUT: core::ffi::c_int = 500 as core::ffi::c_int;
pub const INPUT_BUF_START: core::ffi::c_int = 32 as core::ffi::c_int;
pub const INPUT_DISCARD: core::ffi::c_int = 0x1 as core::ffi::c_int;
pub const INPUT_LAST: core::ffi::c_int = 0x2 as core::ffi::c_int;
static input_esc_table: [input_table_entry; 15] = [
    input_table_entry {
        ch: '0' as i32,
        interm: c"(",
        type_0: INPUT_ESC_SCSG0_ON as core::ffi::c_int,
    },
    input_table_entry {
        ch: '0' as i32,
        interm: c")",
        type_0: INPUT_ESC_SCSG1_ON as core::ffi::c_int,
    },
    input_table_entry {
        ch: '7' as i32,
        interm: c"",
        type_0: INPUT_ESC_DECSC as core::ffi::c_int,
    },
    input_table_entry {
        ch: '8' as i32,
        interm: c"",
        type_0: INPUT_ESC_DECRC as core::ffi::c_int,
    },
    input_table_entry {
        ch: '8' as i32,
        interm: c"#",
        type_0: INPUT_ESC_DECALN as core::ffi::c_int,
    },
    input_table_entry {
        ch: '=' as i32,
        interm: c"",
        type_0: INPUT_ESC_DECKPAM as core::ffi::c_int,
    },
    input_table_entry {
        ch: '>' as i32,
        interm: c"",
        type_0: INPUT_ESC_DECKPNM as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'B' as i32,
        interm: c"(",
        type_0: INPUT_ESC_SCSG0_OFF as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'B' as i32,
        interm: c")",
        type_0: INPUT_ESC_SCSG1_OFF as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'D' as i32,
        interm: c"",
        type_0: INPUT_ESC_IND as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'E' as i32,
        interm: c"",
        type_0: INPUT_ESC_NEL as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'H' as i32,
        interm: c"",
        type_0: INPUT_ESC_HTS as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'M' as i32,
        interm: c"",
        type_0: INPUT_ESC_RI as core::ffi::c_int,
    },
    input_table_entry {
        ch: '\\' as i32,
        interm: c"",
        type_0: INPUT_ESC_ST as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'c' as i32,
        interm: c"",
        type_0: INPUT_ESC_RIS as core::ffi::c_int,
    },
];
static input_csi_table: [input_table_entry; 43] = [
    input_table_entry {
        ch: '@' as i32,
        interm: c"",
        type_0: INPUT_CSI_ICH as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'A' as i32,
        interm: c"",
        type_0: INPUT_CSI_CUU as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'B' as i32,
        interm: c"",
        type_0: INPUT_CSI_CUD as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'C' as i32,
        interm: c"",
        type_0: INPUT_CSI_CUF as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'D' as i32,
        interm: c"",
        type_0: INPUT_CSI_CUB as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'E' as i32,
        interm: c"",
        type_0: INPUT_CSI_CNL as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'F' as i32,
        interm: c"",
        type_0: INPUT_CSI_CPL as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'G' as i32,
        interm: c"",
        type_0: INPUT_CSI_HPA as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'H' as i32,
        interm: c"",
        type_0: INPUT_CSI_CUP as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'J' as i32,
        interm: c"",
        type_0: INPUT_CSI_ED as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'K' as i32,
        interm: c"",
        type_0: INPUT_CSI_EL as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'L' as i32,
        interm: c"",
        type_0: INPUT_CSI_IL as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'M' as i32,
        interm: c"",
        type_0: INPUT_CSI_DL as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'P' as i32,
        interm: c"",
        type_0: INPUT_CSI_DCH as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'S' as i32,
        interm: c"",
        type_0: INPUT_CSI_SU as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'S' as i32,
        interm: c"?",
        type_0: INPUT_CSI_SM_GRAPHICS as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'T' as i32,
        interm: c"",
        type_0: INPUT_CSI_SD as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'X' as i32,
        interm: c"",
        type_0: INPUT_CSI_ECH as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'Z' as i32,
        interm: c"",
        type_0: INPUT_CSI_CBT as core::ffi::c_int,
    },
    input_table_entry {
        ch: '`' as i32,
        interm: c"",
        type_0: INPUT_CSI_HPA as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'b' as i32,
        interm: c"",
        type_0: INPUT_CSI_REP as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'c' as i32,
        interm: c"",
        type_0: INPUT_CSI_DA as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'c' as i32,
        interm: c">",
        type_0: INPUT_CSI_DA_TWO as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'd' as i32,
        interm: c"",
        type_0: INPUT_CSI_VPA as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'f' as i32,
        interm: c"",
        type_0: INPUT_CSI_CUP as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'g' as i32,
        interm: c"",
        type_0: INPUT_CSI_TBC as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'h' as i32,
        interm: c"",
        type_0: INPUT_CSI_SM as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'h' as i32,
        interm: c"?",
        type_0: INPUT_CSI_SM_PRIVATE as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'l' as i32,
        interm: c"",
        type_0: INPUT_CSI_RM as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'l' as i32,
        interm: c"?",
        type_0: INPUT_CSI_RM_PRIVATE as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'm' as i32,
        interm: c"",
        type_0: INPUT_CSI_SGR as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'm' as i32,
        interm: c">",
        type_0: INPUT_CSI_MODSET as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'n' as i32,
        interm: c"",
        type_0: INPUT_CSI_DSR as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'n' as i32,
        interm: c">",
        type_0: INPUT_CSI_MODOFF as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'n' as i32,
        interm: c"?",
        type_0: INPUT_CSI_DSR_PRIVATE as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'p' as i32,
        interm: c"$",
        type_0: INPUT_CSI_QUERY as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'p' as i32,
        interm: c"?$",
        type_0: INPUT_CSI_QUERY_PRIVATE as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'q' as i32,
        interm: c" ",
        type_0: INPUT_CSI_DECSCUSR as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'q' as i32,
        interm: c">",
        type_0: INPUT_CSI_XDA as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'r' as i32,
        interm: c"",
        type_0: INPUT_CSI_DECSTBM as core::ffi::c_int,
    },
    input_table_entry {
        ch: 's' as i32,
        interm: c"",
        type_0: INPUT_CSI_SCP as core::ffi::c_int,
    },
    input_table_entry {
        ch: 't' as i32,
        interm: c"",
        type_0: INPUT_CSI_WINOPS as core::ffi::c_int,
    },
    input_table_entry {
        ch: 'u' as i32,
        interm: c"",
        type_0: INPUT_CSI_RCP as core::ffi::c_int,
    },
];
pub(crate) static input_state_ground: input_state = {
    input_state {
        name: c"ground",
        enter: Some(InputStateAction::Ground),
        exit: None,
        transitions: &input_state_ground_table,
    }
};
static input_state_esc_enter: input_state = {
    input_state {
        name: c"esc_enter",
        enter: Some(InputStateAction::Clear),
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
        enter: Some(InputStateAction::Clear),
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
        enter: Some(InputStateAction::EnterDcs),
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
        enter: Some(InputStateAction::EnterOsc),
        exit: Some(InputStateAction::ExitOsc),
        transitions: &input_state_osc_string_table,
    }
};
static input_state_apc_string: input_state = {
    input_state {
        name: c"apc_string",
        enter: Some(InputStateAction::EnterApc),
        exit: Some(InputStateAction::ExitApc),
        transitions: &input_state_apc_string_table,
    }
};
static input_state_rename_string: input_state = {
    input_state {
        name: c"rename_string",
        enter: Some(InputStateAction::EnterRename),
        exit: Some(InputStateAction::ExitRename),
        transitions: &input_state_rename_string_table,
    }
};
static input_state_consume_st: input_state = {
    input_state {
        name: c"consume_st",
        enter: Some(InputStateAction::EnterRename),
        exit: None,
        transitions: &input_state_consume_st_table,
    }
};
static input_state_ground_table: [input_transition; 9] = {
    [
        input_transition {
            first: 0x18 as core::ffi::c_int,
            last: 0x18 as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1a as core::ffi::c_int,
            last: 0x1a as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1b as core::ffi::c_int,
            last: 0x1b as core::ffi::c_int,
            handler: None,
            state: Some(&input_state_esc_enter),
        },
        input_transition {
            first: 0 as core::ffi::c_int,
            last: 0x17 as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x19 as core::ffi::c_int,
            last: 0x19 as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x1c as core::ffi::c_int,
            last: 0x1f as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x20 as core::ffi::c_int,
            last: 0x7e as core::ffi::c_int,
            handler: Some(InputHandler::Print),
            state: None,
        },
        input_transition {
            first: 0x7f as core::ffi::c_int,
            last: 0x7f as core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x80 as core::ffi::c_int,
            last: 0xff as core::ffi::c_int,
            handler: Some(InputHandler::TopBitSet),
            state: None,
        },
    ]
};
static input_state_esc_enter_table: [input_transition; 22] = {
    [
        input_transition {
            first: 0x18 as core::ffi::c_int,
            last: 0x18 as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1a as core::ffi::c_int,
            last: 0x1a as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1b as core::ffi::c_int,
            last: 0x1b as core::ffi::c_int,
            handler: None,
            state: Some(&input_state_esc_enter),
        },
        input_transition {
            first: 0 as core::ffi::c_int,
            last: 0x17 as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x19 as core::ffi::c_int,
            last: 0x19 as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x1c as core::ffi::c_int,
            last: 0x1f as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x20 as core::ffi::c_int,
            last: 0x2f as core::ffi::c_int,
            handler: Some(InputHandler::Intermediate),
            state: Some(&input_state_esc_intermediate),
        },
        input_transition {
            first: 0x30 as core::ffi::c_int,
            last: 0x4f as core::ffi::c_int,
            handler: Some(InputHandler::EscDispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x50 as core::ffi::c_int,
            last: 0x50 as core::ffi::c_int,
            handler: None,
            state: Some(&input_state_dcs_enter),
        },
        input_transition {
            first: 0x51 as core::ffi::c_int,
            last: 0x57 as core::ffi::c_int,
            handler: Some(InputHandler::EscDispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x58 as core::ffi::c_int,
            last: 0x58 as core::ffi::c_int,
            handler: None,
            state: Some(&input_state_consume_st),
        },
        input_transition {
            first: 0x59 as core::ffi::c_int,
            last: 0x59 as core::ffi::c_int,
            handler: Some(InputHandler::EscDispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x5a as core::ffi::c_int,
            last: 0x5a as core::ffi::c_int,
            handler: Some(InputHandler::EscDispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x5b as core::ffi::c_int,
            last: 0x5b as core::ffi::c_int,
            handler: None,
            state: Some(&input_state_csi_enter),
        },
        input_transition {
            first: 0x5c as core::ffi::c_int,
            last: 0x5c as core::ffi::c_int,
            handler: Some(InputHandler::EscDispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x5d as core::ffi::c_int,
            last: 0x5d as core::ffi::c_int,
            handler: None,
            state: Some(&input_state_osc_string),
        },
        input_transition {
            first: 0x5e as core::ffi::c_int,
            last: 0x5e as core::ffi::c_int,
            handler: None,
            state: Some(&input_state_consume_st),
        },
        input_transition {
            first: 0x5f as core::ffi::c_int,
            last: 0x5f as core::ffi::c_int,
            handler: None,
            state: Some(&input_state_apc_string),
        },
        input_transition {
            first: 0x60 as core::ffi::c_int,
            last: 0x6a as core::ffi::c_int,
            handler: Some(InputHandler::EscDispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x6b as core::ffi::c_int,
            last: 0x6b as core::ffi::c_int,
            handler: None,
            state: Some(&input_state_rename_string),
        },
        input_transition {
            first: 0x6c as core::ffi::c_int,
            last: 0x7e as core::ffi::c_int,
            handler: Some(InputHandler::EscDispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x7f as core::ffi::c_int,
            last: 0xff as core::ffi::c_int,
            handler: None,
            state: None,
        },
    ]
};
static input_state_esc_intermediate_table: [input_transition; 9] = {
    [
        input_transition {
            first: 0x18 as core::ffi::c_int,
            last: 0x18 as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1a as core::ffi::c_int,
            last: 0x1a as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1b as core::ffi::c_int,
            last: 0x1b as core::ffi::c_int,
            handler: None,
            state: Some(&input_state_esc_enter),
        },
        input_transition {
            first: 0 as core::ffi::c_int,
            last: 0x17 as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x19 as core::ffi::c_int,
            last: 0x19 as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x1c as core::ffi::c_int,
            last: 0x1f as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x20 as core::ffi::c_int,
            last: 0x2f as core::ffi::c_int,
            handler: Some(InputHandler::Intermediate),
            state: None,
        },
        input_transition {
            first: 0x30 as core::ffi::c_int,
            last: 0x7e as core::ffi::c_int,
            handler: Some(InputHandler::EscDispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x7f as core::ffi::c_int,
            last: 0xff as core::ffi::c_int,
            handler: None,
            state: None,
        },
    ]
};
static input_state_csi_enter_table: [input_transition; 13] = {
    [
        input_transition {
            first: 0x18 as core::ffi::c_int,
            last: 0x18 as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1a as core::ffi::c_int,
            last: 0x1a as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1b as core::ffi::c_int,
            last: 0x1b as core::ffi::c_int,
            handler: None,
            state: Some(&input_state_esc_enter),
        },
        input_transition {
            first: 0 as core::ffi::c_int,
            last: 0x17 as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x19 as core::ffi::c_int,
            last: 0x19 as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x1c as core::ffi::c_int,
            last: 0x1f as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x20 as core::ffi::c_int,
            last: 0x2f as core::ffi::c_int,
            handler: Some(InputHandler::Intermediate),
            state: Some(&input_state_csi_intermediate),
        },
        input_transition {
            first: 0x30 as core::ffi::c_int,
            last: 0x39 as core::ffi::c_int,
            handler: Some(InputHandler::Parameter),
            state: Some(&input_state_csi_parameter),
        },
        input_transition {
            first: 0x3a as core::ffi::c_int,
            last: 0x3a as core::ffi::c_int,
            handler: Some(InputHandler::Parameter),
            state: Some(&input_state_csi_parameter),
        },
        input_transition {
            first: 0x3b as core::ffi::c_int,
            last: 0x3b as core::ffi::c_int,
            handler: Some(InputHandler::Parameter),
            state: Some(&input_state_csi_parameter),
        },
        input_transition {
            first: 0x3c as core::ffi::c_int,
            last: 0x3f as core::ffi::c_int,
            handler: Some(InputHandler::Intermediate),
            state: Some(&input_state_csi_parameter),
        },
        input_transition {
            first: 0x40 as core::ffi::c_int,
            last: 0x7e as core::ffi::c_int,
            handler: Some(InputHandler::CsiDispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x7f as core::ffi::c_int,
            last: 0xff as core::ffi::c_int,
            handler: None,
            state: None,
        },
    ]
};
static input_state_csi_parameter_table: [input_transition; 13] = {
    [
        input_transition {
            first: 0x18 as core::ffi::c_int,
            last: 0x18 as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1a as core::ffi::c_int,
            last: 0x1a as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1b as core::ffi::c_int,
            last: 0x1b as core::ffi::c_int,
            handler: None,
            state: Some(&input_state_esc_enter),
        },
        input_transition {
            first: 0 as core::ffi::c_int,
            last: 0x17 as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x19 as core::ffi::c_int,
            last: 0x19 as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x1c as core::ffi::c_int,
            last: 0x1f as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x20 as core::ffi::c_int,
            last: 0x2f as core::ffi::c_int,
            handler: Some(InputHandler::Intermediate),
            state: Some(&input_state_csi_intermediate),
        },
        input_transition {
            first: 0x30 as core::ffi::c_int,
            last: 0x39 as core::ffi::c_int,
            handler: Some(InputHandler::Parameter),
            state: None,
        },
        input_transition {
            first: 0x3a as core::ffi::c_int,
            last: 0x3a as core::ffi::c_int,
            handler: Some(InputHandler::Parameter),
            state: None,
        },
        input_transition {
            first: 0x3b as core::ffi::c_int,
            last: 0x3b as core::ffi::c_int,
            handler: Some(InputHandler::Parameter),
            state: None,
        },
        input_transition {
            first: 0x3c as core::ffi::c_int,
            last: 0x3f as core::ffi::c_int,
            handler: None,
            state: Some(&input_state_csi_ignore),
        },
        input_transition {
            first: 0x40 as core::ffi::c_int,
            last: 0x7e as core::ffi::c_int,
            handler: Some(InputHandler::CsiDispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x7f as core::ffi::c_int,
            last: 0xff as core::ffi::c_int,
            handler: None,
            state: None,
        },
    ]
};
static input_state_csi_intermediate_table: [input_transition; 10] = {
    [
        input_transition {
            first: 0x18 as core::ffi::c_int,
            last: 0x18 as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1a as core::ffi::c_int,
            last: 0x1a as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1b as core::ffi::c_int,
            last: 0x1b as core::ffi::c_int,
            handler: None,
            state: Some(&input_state_esc_enter),
        },
        input_transition {
            first: 0 as core::ffi::c_int,
            last: 0x17 as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x19 as core::ffi::c_int,
            last: 0x19 as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x1c as core::ffi::c_int,
            last: 0x1f as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x20 as core::ffi::c_int,
            last: 0x2f as core::ffi::c_int,
            handler: Some(InputHandler::Intermediate),
            state: None,
        },
        input_transition {
            first: 0x30 as core::ffi::c_int,
            last: 0x3f as core::ffi::c_int,
            handler: None,
            state: Some(&input_state_csi_ignore),
        },
        input_transition {
            first: 0x40 as core::ffi::c_int,
            last: 0x7e as core::ffi::c_int,
            handler: Some(InputHandler::CsiDispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x7f as core::ffi::c_int,
            last: 0xff as core::ffi::c_int,
            handler: None,
            state: None,
        },
    ]
};
static input_state_csi_ignore_table: [input_transition; 9] = {
    [
        input_transition {
            first: 0x18 as core::ffi::c_int,
            last: 0x18 as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1a as core::ffi::c_int,
            last: 0x1a as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1b as core::ffi::c_int,
            last: 0x1b as core::ffi::c_int,
            handler: None,
            state: Some(&input_state_esc_enter),
        },
        input_transition {
            first: 0 as core::ffi::c_int,
            last: 0x17 as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x19 as core::ffi::c_int,
            last: 0x19 as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x1c as core::ffi::c_int,
            last: 0x1f as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: None,
        },
        input_transition {
            first: 0x20 as core::ffi::c_int,
            last: 0x3f as core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x40 as core::ffi::c_int,
            last: 0x7e as core::ffi::c_int,
            handler: None,
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x7f as core::ffi::c_int,
            last: 0xff as core::ffi::c_int,
            handler: None,
            state: None,
        },
    ]
};
static input_state_dcs_enter_table: [input_transition; 13] = {
    [
        input_transition {
            first: 0x18 as core::ffi::c_int,
            last: 0x18 as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1a as core::ffi::c_int,
            last: 0x1a as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1b as core::ffi::c_int,
            last: 0x1b as core::ffi::c_int,
            handler: None,
            state: Some(&input_state_esc_enter),
        },
        input_transition {
            first: 0 as core::ffi::c_int,
            last: 0x17 as core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x19 as core::ffi::c_int,
            last: 0x19 as core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x1c as core::ffi::c_int,
            last: 0x1f as core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x20 as core::ffi::c_int,
            last: 0x2f as core::ffi::c_int,
            handler: Some(InputHandler::Intermediate),
            state: Some(&input_state_dcs_intermediate),
        },
        input_transition {
            first: 0x30 as core::ffi::c_int,
            last: 0x39 as core::ffi::c_int,
            handler: Some(InputHandler::Parameter),
            state: Some(&input_state_dcs_parameter),
        },
        input_transition {
            first: 0x3a as core::ffi::c_int,
            last: 0x3a as core::ffi::c_int,
            handler: None,
            state: Some(&input_state_dcs_ignore),
        },
        input_transition {
            first: 0x3b as core::ffi::c_int,
            last: 0x3b as core::ffi::c_int,
            handler: Some(InputHandler::Parameter),
            state: Some(&input_state_dcs_parameter),
        },
        input_transition {
            first: 0x3c as core::ffi::c_int,
            last: 0x3f as core::ffi::c_int,
            handler: Some(InputHandler::Intermediate),
            state: Some(&input_state_dcs_parameter),
        },
        input_transition {
            first: 0x40 as core::ffi::c_int,
            last: 0x7e as core::ffi::c_int,
            handler: Some(InputHandler::Input),
            state: Some(&input_state_dcs_handler),
        },
        input_transition {
            first: 0x7f as core::ffi::c_int,
            last: 0xff as core::ffi::c_int,
            handler: None,
            state: None,
        },
    ]
};
static input_state_dcs_parameter_table: [input_transition; 13] = {
    [
        input_transition {
            first: 0x18 as core::ffi::c_int,
            last: 0x18 as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1a as core::ffi::c_int,
            last: 0x1a as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1b as core::ffi::c_int,
            last: 0x1b as core::ffi::c_int,
            handler: None,
            state: Some(&input_state_esc_enter),
        },
        input_transition {
            first: 0 as core::ffi::c_int,
            last: 0x17 as core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x19 as core::ffi::c_int,
            last: 0x19 as core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x1c as core::ffi::c_int,
            last: 0x1f as core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x20 as core::ffi::c_int,
            last: 0x2f as core::ffi::c_int,
            handler: Some(InputHandler::Intermediate),
            state: Some(&input_state_dcs_intermediate),
        },
        input_transition {
            first: 0x30 as core::ffi::c_int,
            last: 0x39 as core::ffi::c_int,
            handler: Some(InputHandler::Parameter),
            state: None,
        },
        input_transition {
            first: 0x3a as core::ffi::c_int,
            last: 0x3a as core::ffi::c_int,
            handler: None,
            state: Some(&input_state_dcs_ignore),
        },
        input_transition {
            first: 0x3b as core::ffi::c_int,
            last: 0x3b as core::ffi::c_int,
            handler: Some(InputHandler::Parameter),
            state: None,
        },
        input_transition {
            first: 0x3c as core::ffi::c_int,
            last: 0x3f as core::ffi::c_int,
            handler: None,
            state: Some(&input_state_dcs_ignore),
        },
        input_transition {
            first: 0x40 as core::ffi::c_int,
            last: 0x7e as core::ffi::c_int,
            handler: Some(InputHandler::Input),
            state: Some(&input_state_dcs_handler),
        },
        input_transition {
            first: 0x7f as core::ffi::c_int,
            last: 0xff as core::ffi::c_int,
            handler: None,
            state: None,
        },
    ]
};
static input_state_dcs_intermediate_table: [input_transition; 10] = {
    [
        input_transition {
            first: 0x18 as core::ffi::c_int,
            last: 0x18 as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1a as core::ffi::c_int,
            last: 0x1a as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1b as core::ffi::c_int,
            last: 0x1b as core::ffi::c_int,
            handler: None,
            state: Some(&input_state_esc_enter),
        },
        input_transition {
            first: 0 as core::ffi::c_int,
            last: 0x17 as core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x19 as core::ffi::c_int,
            last: 0x19 as core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x1c as core::ffi::c_int,
            last: 0x1f as core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x20 as core::ffi::c_int,
            last: 0x2f as core::ffi::c_int,
            handler: Some(InputHandler::Intermediate),
            state: None,
        },
        input_transition {
            first: 0x30 as core::ffi::c_int,
            last: 0x3f as core::ffi::c_int,
            handler: None,
            state: Some(&input_state_dcs_ignore),
        },
        input_transition {
            first: 0x40 as core::ffi::c_int,
            last: 0x7e as core::ffi::c_int,
            handler: Some(InputHandler::Input),
            state: Some(&input_state_dcs_handler),
        },
        input_transition {
            first: 0x7f as core::ffi::c_int,
            last: 0xff as core::ffi::c_int,
            handler: None,
            state: None,
        },
    ]
};
static input_state_dcs_handler_table: [input_transition; 3] = {
    [
        input_transition {
            first: 0 as core::ffi::c_int,
            last: 0x1a as core::ffi::c_int,
            handler: Some(InputHandler::Input),
            state: None,
        },
        input_transition {
            first: 0x1b as core::ffi::c_int,
            last: 0x1b as core::ffi::c_int,
            handler: None,
            state: Some(&input_state_dcs_escape),
        },
        input_transition {
            first: 0x1c as core::ffi::c_int,
            last: 0xff as core::ffi::c_int,
            handler: Some(InputHandler::Input),
            state: None,
        },
    ]
};
static input_state_dcs_escape_table: [input_transition; 3] = {
    [
        input_transition {
            first: 0 as core::ffi::c_int,
            last: 0x5b as core::ffi::c_int,
            handler: Some(InputHandler::Input),
            state: Some(&input_state_dcs_handler),
        },
        input_transition {
            first: 0x5c as core::ffi::c_int,
            last: 0x5c as core::ffi::c_int,
            handler: Some(InputHandler::DcsDispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x5d as core::ffi::c_int,
            last: 0xff as core::ffi::c_int,
            handler: Some(InputHandler::Input),
            state: Some(&input_state_dcs_handler),
        },
    ]
};
static input_state_dcs_ignore_table: [input_transition; 7] = {
    [
        input_transition {
            first: 0x18 as core::ffi::c_int,
            last: 0x18 as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1a as core::ffi::c_int,
            last: 0x1a as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1b as core::ffi::c_int,
            last: 0x1b as core::ffi::c_int,
            handler: None,
            state: Some(&input_state_esc_enter),
        },
        input_transition {
            first: 0 as core::ffi::c_int,
            last: 0x17 as core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x19 as core::ffi::c_int,
            last: 0x19 as core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x1c as core::ffi::c_int,
            last: 0x1f as core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x20 as core::ffi::c_int,
            last: 0xff as core::ffi::c_int,
            handler: None,
            state: None,
        },
    ]
};
static input_state_osc_string_table: [input_transition; 9] = {
    [
        input_transition {
            first: 0x18 as core::ffi::c_int,
            last: 0x18 as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1a as core::ffi::c_int,
            last: 0x1a as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1b as core::ffi::c_int,
            last: 0x1b as core::ffi::c_int,
            handler: None,
            state: Some(&input_state_esc_enter),
        },
        input_transition {
            first: 0 as core::ffi::c_int,
            last: 0x6 as core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x7 as core::ffi::c_int,
            last: 0x7 as core::ffi::c_int,
            handler: Some(InputHandler::EndBel),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x8 as core::ffi::c_int,
            last: 0x17 as core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x19 as core::ffi::c_int,
            last: 0x19 as core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x1c as core::ffi::c_int,
            last: 0x1f as core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x20 as core::ffi::c_int,
            last: 0xff as core::ffi::c_int,
            handler: Some(InputHandler::Input),
            state: None,
        },
    ]
};
static input_state_apc_string_table: [input_transition; 7] = {
    [
        input_transition {
            first: 0x18 as core::ffi::c_int,
            last: 0x18 as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1a as core::ffi::c_int,
            last: 0x1a as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1b as core::ffi::c_int,
            last: 0x1b as core::ffi::c_int,
            handler: None,
            state: Some(&input_state_esc_enter),
        },
        input_transition {
            first: 0 as core::ffi::c_int,
            last: 0x17 as core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x19 as core::ffi::c_int,
            last: 0x19 as core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x1c as core::ffi::c_int,
            last: 0x1f as core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x20 as core::ffi::c_int,
            last: 0xff as core::ffi::c_int,
            handler: Some(InputHandler::Input),
            state: None,
        },
    ]
};
static input_state_rename_string_table: [input_transition; 7] = {
    [
        input_transition {
            first: 0x18 as core::ffi::c_int,
            last: 0x18 as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1a as core::ffi::c_int,
            last: 0x1a as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1b as core::ffi::c_int,
            last: 0x1b as core::ffi::c_int,
            handler: None,
            state: Some(&input_state_esc_enter),
        },
        input_transition {
            first: 0 as core::ffi::c_int,
            last: 0x17 as core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x19 as core::ffi::c_int,
            last: 0x19 as core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x1c as core::ffi::c_int,
            last: 0x1f as core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x20 as core::ffi::c_int,
            last: 0xff as core::ffi::c_int,
            handler: Some(InputHandler::Input),
            state: None,
        },
    ]
};
static input_state_consume_st_table: [input_transition; 7] = {
    [
        input_transition {
            first: 0x18 as core::ffi::c_int,
            last: 0x18 as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1a as core::ffi::c_int,
            last: 0x1a as core::ffi::c_int,
            handler: Some(InputHandler::C0Dispatch),
            state: Some(&input_state_ground),
        },
        input_transition {
            first: 0x1b as core::ffi::c_int,
            last: 0x1b as core::ffi::c_int,
            handler: None,
            state: Some(&input_state_esc_enter),
        },
        input_transition {
            first: 0 as core::ffi::c_int,
            last: 0x17 as core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x19 as core::ffi::c_int,
            last: 0x19 as core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x1c as core::ffi::c_int,
            last: 0x1f as core::ffi::c_int,
            handler: None,
            state: None,
        },
        input_transition {
            first: 0x20 as core::ffi::c_int,
            last: 0xff as core::ffi::c_int,
            handler: None,
            state: None,
        },
    ]
};
const input_buffer_size: crate::server_state::Value<size_t> =
    crate::server_state::Value::new(|state| &state.input_buffer_size);
/// The entry of `table` the parser's character and intermediates name.
fn input_table_find(
    ictx: &input_ctx,
    table: &'static [input_table_entry],
) -> Option<&'static input_table_entry> {
    table
        .binary_search_by(|entry| input_table_compare(ictx, entry).reverse())
        .ok()
        .map(|at| &table[at])
}
fn input_table_compare(key: &input_ctx, value: &input_table_entry) -> core::cmp::Ordering {
    key.ch.cmp(&value.ch).then_with(|| {
        CStr::from_bytes_until_nul(&key.interm_buf)
            .expect("input intermediates have a trailing NUL")
            .to_bytes()
            .cmp(value.interm.to_bytes())
    })
}

unsafe fn input_stop_utf8(ictx: &mut input_ctx, sctx: &mut RustScreenWriteCtx<'_>) {
    {
        const input_cell_buffer: crate::server_state::Value<utf8_data> =
            crate::server_state::Value::new(|state| &state.input_cell_buffer);
        if ictx.utf8started != 0 {
            utf8_copy(&mut ictx.cell.cell.data, &input_cell_buffer.get());
            sctx.collect_add(&ictx.cell.cell);
        }
        ictx.utf8started = 0 as core::ffi::c_int;
    }
}
unsafe fn input_ground_timer_callback(ictx: &mut input_ctx) {
    unsafe {
        log_debug(
            c"%s: %s expired",
            fmt_args![
                c"input_ground_timer_callback".as_ptr(),
                ictx.state.name.as_ptr()
            ],
        );
        ictx.reset(0 as core::ffi::c_int);
    }
}
fn input_start_ground_timer(ictx: &mut input_ctx) {
    let tv = timeval::from_secs(5 as __time_t);
    ictx.ground_timer.disarm();
    ictx.ground_timer.arm(tv);
}
fn input_reset_cell(ictx: &mut input_ctx) {
    ictx.cell.cell = grid_default_cell;
    ictx.cell.set = 0 as core::ffi::c_int;
    ictx.cell.g1set = 0 as core::ffi::c_int;
    ictx.cell.g0set = ictx.cell.g1set;
    ictx.old_cell = ictx.cell;
    ictx.old_cx = 0 as u_int;
    ictx.old_cy = 0 as u_int;
}
fn input_save_state(ictx: &mut input_ctx, sctx: &mut RustScreenWriteCtx<'_>) {
    {
        let s = &mut *sctx.screen_mut();
        ictx.old_cell = ictx.cell;
        (ictx.old_cx, ictx.old_cy) = s.cursor();
        ictx.old_mode = (*s).mode();
    }
}
fn input_restore_state(ictx: &mut input_ctx, sctx: &mut RustScreenWriteCtx<'_>) {
    ictx.cell = ictx.old_cell;
    if ictx.old_mode & MODE_ORIGIN != 0 {
        sctx.mode_set(MODE_ORIGIN);
    } else {
        sctx.mode_clear(MODE_ORIGIN);
    }
    sctx.cursormove(
        ictx.old_cx as core::ffi::c_int,
        ictx.old_cy as core::ffi::c_int,
        0 as core::ffi::c_int,
    );
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
    Popup(PopupDataWeak, Option<ClientWeak>),
}

impl input_ctx {
    /// Observes the registered pane whose output this parser handles.
    pub fn pane_ref(&self) -> Option<RustWindowPaneWeak> {
        match &self.owner_of {
            InputOwner::Pane(id) => crate::window::window_pane_find_by_id(*id),
            _ => None,
        }
    }

    /// Retains the window recorded by the pane, including while the pane is
    /// temporarily outside a window's pane list during a transfer.
    ///
    /// # Safety
    /// The pane and its recorded window must not be mutated during lookup.
    pub unsafe fn window_ref(&self) -> Option<WindowRef> {
        let pane = self.pane_ref()?;
        unsafe { pane.get()?.window_context() }
    }

    /// Resolves a colour using this input context's palette.
    /// # Safety
    /// Prevent conflicting access to the input owner while resolving its palette.
    pub unsafe fn palette_colour(&self, colour: c_int) -> c_int {
        unsafe {
            match &self.owner_of {
                InputOwner::Detached => -1,
                InputOwner::Pane(_) => self.pane_ref().and_then(|pane| pane.get().map(|pane| pane.palette_colour(colour))).unwrap_or(-1),
                InputOwner::Popup(held, _) => held.upgrade().map_or(-1, |held| RustColourEngine.get_palette(Some(&held.borrow().palette), colour)),
            }
        }
    }
    /// Updates an indexed colour through the input owner's palette engine.
    /// # Safety
    /// Prevent conflicting access to the input owner and its palette.
    pub unsafe fn set_palette_colour(&mut self, index: c_int, colour: c_int) -> c_int {
        unsafe {
            match &self.owner_of {
                InputOwner::Detached => 0,
                InputOwner::Pane(_) => self.pane_ref().and_then(|mut pane| pane.get_mut().map(|pane| pane.set_palette_colour(index, colour))).unwrap_or(0),
                InputOwner::Popup(held, _) => held.upgrade().map_or(0, |held| RustColourEngine.set_palette(Some(&mut held.borrow_mut().palette), index, colour)),
            }
        }
    }
    /// Clears application palette overrides in this input context.
    /// # Safety
    /// Prevent conflicting access to the input owner and its palette.
    pub unsafe fn clear_palette(&mut self) {
        unsafe {
            match &self.owner_of {
                InputOwner::Detached => {},
                InputOwner::Pane(_) => { if let Some(mut pane) = self.pane_ref() && let Some(pane) = pane.get_mut() { pane.clear_palette(); } }
                InputOwner::Popup(held, _) => { if let Some(held) = held.upgrade() { RustColourEngine.clear_palette(Some(&mut held.borrow_mut().palette)); } }
            }
        }
    }
    /// Updates a default colour, preserving the owner's context-specific invalidation.
    /// # Safety
    /// Prevent conflicting access to the input owner and its palette.
    pub unsafe fn set_palette_default(&mut self, foreground: bool, colour: c_int) -> bool {
        unsafe {
            match &self.owner_of {
                InputOwner::Detached => false,
                InputOwner::Pane(_) => self.pane_ref().and_then(|mut pane| pane.get_mut().map(|pane| pane.set_palette_default(foreground, colour))).is_some(),
                InputOwner::Popup(held, _) => {
                    let Some(held) = held.upgrade() else { return false };
                    let mut popup = held.borrow_mut();
                    if foreground { popup.palette.fg = colour; } else { popup.palette.bg = colour; }
                    true
                }
            }
        }
    }

    /// The client the parser answers to, held while the caller uses it.
    pub fn client(&self) -> Option<ClientRef> {
        match &self.owner_of {
            InputOwner::Popup(_, Some(held)) => held.upgrade(),
            _ => None,
        }
    }
}

/// Shared ownership of an input parser with checked access to its state.
#[derive(Clone)]
pub struct InputCtxRef(Rc<RefCell<input_ctx>>);

/// A parser observation that does not keep the parser allocation alive.
#[derive(Clone)]
pub struct InputCtxWeak(Weak<RefCell<input_ctx>>);

impl InputCtxRef {
    pub(crate) fn new(value: input_ctx) -> Self {
        let reference = Self(Rc::new(RefCell::new(value)));
        reference.borrow_mut().owner = Some(reference.downgrade());
        reference
    }

    pub(crate) fn borrow(&self) -> Ref<'_, input_ctx> {
        self.0.borrow()
    }

    pub(crate) fn borrow_mut(&self) -> RefMut<'_, input_ctx> {
        self.0.borrow_mut()
    }

    pub(crate) fn downgrade(&self) -> InputCtxWeak {
        InputCtxWeak(Rc::downgrade(&self.0))
    }
}

impl InputCtxWeak {
    pub(crate) fn upgrade(&self) -> Option<InputCtxRef> {
        self.0.upgrade().map(InputCtxRef)
    }
}

pub fn ictx_mut(value: &Option<InputCtxRef>) -> RefMut<'_, input_ctx> {
    value
        .as_ref()
        .expect("an input owner has a parser")
        .borrow_mut()
}

unsafe fn input_set_state(
    ictx: &mut input_ctx,
    sctx: &mut RustScreenWriteCtx<'_>,
    state: &'static input_state,
) {
    unsafe {
        if let Some(exit) = ictx.state.exit {
            exit.call(ictx, sctx);
        }
        ictx.state = state;
        if let Some(enter) = state.enter {
            enter.call(ictx, sctx);
        }
    }
}
pub(crate) unsafe fn input_parse(
    ictx: &mut input_ctx,
    sctx: &mut RustScreenWriteCtx<'_>,
    mut input: ByteBuffer,
) {
    unsafe {
        let mut state: Option<&'static input_state> = None;
        let mut itr: Option<&'static input_transition> = None;
        let mut pending = ByteBuffer::new();
        while input.has_remaining() {
            let chunk_len = input.chunk().len();
            let chunk = input.copy_to_bytes(chunk_len);
            let mut pending_start = None;
            for (off, &ch) in chunk.iter().enumerate() {
                ictx.ch = ch as core::ffi::c_int;
                let ch = ictx.ch;
                let holds = |itr: &input_transition| ch >= itr.first && ch <= itr.last;
                if !state.is_some_and(|previous| core::ptr::eq(ictx.state, previous))
                    || !itr.is_some_and(&holds)
                {
                    itr = ictx.state.transitions.iter().find(|itr| holds(itr));
                }
                let Some(itr) = itr else {
                    fatalx(c"no transition from state", fmt_args![]);
                };
                state = Some(ictx.state);
                if itr.handler != Some(InputHandler::Print) {
                    sctx.collect_end();
                }
                if let Some(handler) = itr.handler
                    && handler.call(ictx, sctx) != 0 as core::ffi::c_int
                {
                    if let Some(start) = pending_start.take() {
                        pending.append_bytes(chunk.slice(start..off));
                    }
                    continue;
                }
                if let Some(state) = itr.state {
                    input_set_state(ictx, sctx, state);
                }
                if core::ptr::eq(ictx.state, &input_state_ground) {
                    pending.clear();
                    pending_start = None;
                } else if pending_start.is_none() {
                    pending_start = Some(off);
                }
            }
            if let Some(start) = pending_start {
                pending.append_bytes(chunk.slice(start..));
            }
        }
        if let Some(since_ground) = ictx.since_ground.as_deref_mut() {
            since_ground.append_buf(&mut pending);
        }
    }
}
/// The parameters the parser collected for the sequence in hand.
fn input_params(ictx: &mut input_ctx) -> &mut [InputParam; 24] {
    &mut ictx.param_list
}
fn input_fields(buffer: &mut [u8], separator: u8) -> impl Iterator<Item = &CStr> {
    let len = CStr::from_bytes_until_nul(buffer)
        .expect("input parameters have a trailing NUL")
        .to_bytes_with_nul()
        .len();
    buffer[..len]
        .split_inclusive_mut(move |byte| *byte == separator)
        .map(move |field| {
            if let Some(last) = field.last_mut()
                && *last == separator
            {
                *last = 0;
            }
            CStr::from_bytes_with_nul(field).expect("each input field has a trailing NUL")
        })
}

unsafe fn input_split(ictx: &mut input_ctx) -> core::ffi::c_int {
    unsafe {
        for param in &mut ictx.param_list[..ictx.param_list_len as usize] {
            *param = InputParam::Missing;
        }
        ictx.param_list_len = 0;
        if ictx.param_len == 0 {
            return 0;
        }
        for field in input_fields(&mut ictx.param_buf, b';') {
            let param = if field.is_empty() {
                InputParam::Missing
            } else if field.to_bytes().contains(&b':') {
                InputParam::Str(field.to_owned())
            } else {
                let Ok(n) = strtonum(field, 0, INT_MAX as core::ffi::c_longlong) else {
                    return -1;
                };
                InputParam::Number(n as core::ffi::c_int)
            };
            ictx.param_list[ictx.param_list_len as usize] = param;
            ictx.param_list_len += 1;
            if ictx.param_list_len as usize == ictx.param_list.len() {
                return -1;
            }
        }
        for (i, param) in ictx.param_list[..ictx.param_list_len as usize]
            .iter()
            .enumerate()
        {
            match param {
                InputParam::Missing => {
                    log_debug(c"parameter %u: missing", fmt_args![i]);
                }
                InputParam::Str(value) => {
                    log_debug(c"parameter %u: string %s", fmt_args![i, value.as_ptr()]);
                }
                InputParam::Number(n) => {
                    log_debug(c"parameter %u: number %d", fmt_args![i, *n]);
                }
            }
        }
        0
    }
}

fn input_get(
    ictx: &mut input_ctx,
    validx: u_int,
    minval: core::ffi::c_int,
    defval: core::ffi::c_int,
) -> core::ffi::c_int {
    {
        if validx >= ictx.param_list_len {
            return defval;
        }
        let retval: core::ffi::c_int = match &input_params(ictx)[validx as usize] {
            InputParam::Missing => return defval,
            InputParam::Str(_) => return -(1 as core::ffi::c_int),
            InputParam::Number(n) => *n,
        };
        if retval < minval {
            return minval;
        }
        retval
    }
}
fn input_send_reply(ictx: &mut input_ctx, reply: &CStr) {
    {
        if !ictx.event.is_none() {
            log_debug(
                c"%s: %s",
                fmt_args![c"input_send_reply".as_ptr(), reply.as_ptr()],
            );
            ictx.event.write(reply.to_bytes());
        }
    }
}
fn input_reply(ictx: &mut input_ctx, add: core::ffi::c_int, fmt: &CStr, args: &[FmtArg]) {
    {
        let reply = format_alloc(fmt, args);
        if add != 0 && !ictx.requests.is_empty() {
            let ir = input_make_request(ictx, INPUT_REQUEST_QUEUE);
            input_request_mut_in(ictx, ir.id, |request| request.data = Some(reply));
        } else {
            input_send_reply(&mut *ictx, &reply);
        };
    }
}
fn input_clear(ictx: &mut input_ctx) {
    {
        ictx.ground_timer.disarm();
        ictx.interm_buf[0] = 0;
        ictx.interm_len = 0 as size_t;
        ictx.param_buf[0] = 0;
        ictx.param_len = 0 as size_t;
        ictx.input_buf.clear();
        ictx.input_buf.push(b'\0');
        ictx.input_end = INPUT_END_ST;
        ictx.flags &= !INPUT_DISCARD;
    }
}
fn input_ground(ictx: &mut input_ctx) {
    {
        ictx.ground_timer.disarm();
        if let Some(buf) = ictx.since_ground.as_deref_mut() {
            buf.clear();
        }
        ictx.input_buf.shrink_to(INPUT_BUF_START as usize);
    }
}
unsafe fn input_print(ictx: &mut input_ctx, sctx: &mut RustScreenWriteCtx<'_>) -> core::ffi::c_int {
    unsafe {
        input_stop_utf8(ictx, sctx);
        let set: core::ffi::c_int = if ictx.cell.set == 0 as core::ffi::c_int {
            ictx.cell.g0set
        } else {
            ictx.cell.g1set
        };
        if set == 1 as core::ffi::c_int {
            ictx.cell.cell.attr =
                (ictx.cell.cell.attr as core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
        } else {
            ictx.cell.cell.attr =
                (ictx.cell.cell.attr as core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
        }
        utf8_set(&mut ictx.cell.cell.data, ictx.ch as u_char);
        sctx.collect_add(&ictx.cell.cell);
        utf8_copy(&mut ictx.last, &ictx.cell.cell.data);
        ictx.flags |= INPUT_LAST;
        ictx.cell.cell.attr =
            (ictx.cell.cell.attr as core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
        0 as core::ffi::c_int
    }
}
fn input_intermediate(ictx: &mut input_ctx) -> core::ffi::c_int {
    {
        if ictx.interm_len == size_of::<[u_char; 4]>().wrapping_sub(1_usize) {
            ictx.flags |= INPUT_DISCARD;
        } else {
            let fresh15 = ictx.interm_len;
            ictx.interm_len = ictx.interm_len.wrapping_add(1);
            ictx.interm_buf[fresh15] = ictx.ch as u_char;
            ictx.interm_buf[ictx.interm_len] = '\0' as i32 as u_char;
        }
        0 as core::ffi::c_int
    }
}
fn input_parameter(ictx: &mut input_ctx) -> core::ffi::c_int {
    {
        if ictx.param_len == size_of::<[u_char; 64]>().wrapping_sub(1_usize) {
            ictx.flags |= INPUT_DISCARD;
        } else {
            let fresh14 = ictx.param_len;
            ictx.param_len = ictx.param_len.wrapping_add(1);
            ictx.param_buf[fresh14] = ictx.ch as u_char;
            ictx.param_buf[ictx.param_len] = '\0' as i32 as u_char;
        }
        0 as core::ffi::c_int
    }
}
/// The length of the collected string, which the buffer holds ahead of the
/// terminating NUL the appenders keep at its end.
fn input_length(ictx: &mut input_ctx) -> size_t {
    ictx.input_buf.len().wrapping_sub(1_usize) as size_t
}
unsafe fn input_input(ictx: &mut input_ctx) -> core::ffi::c_int {
    {
        let mut available: size_t = INPUT_BUF_START as size_t;
        while ictx.input_buf.len() as size_t >= available {
            available = available.wrapping_mul(2 as size_t);
            if available > input_buffer_size.get() {
                ictx.flags |= INPUT_DISCARD;
                return 0 as core::ffi::c_int;
            }
        }
        let ch = ictx.ch as u8;
        let buf = &mut ictx.input_buf;
        buf.reserve((available as usize).wrapping_sub(buf.len()));
        let end = buf.len().wrapping_sub(1_usize);
        buf[end] = ch;
        buf.push(b'\0');
        0 as core::ffi::c_int
    }
}
unsafe fn input_c0_dispatch(
    ictx: &mut input_ctx,
    sctx: &mut RustScreenWriteCtx<'_>,
) -> core::ffi::c_int {
    unsafe {
        let mut gc;
        let first_gc;
        let mut cx: u_int;
        let line: u_int;
        let width: u_int;
        let mut has_content: core::ffi::c_int = 0 as core::ffi::c_int;
        input_stop_utf8(ictx, sctx);
        log_debug(
            c"%s: '%c'",
            fmt_args![c"input_c0_dispatch".as_ptr(), ictx.ch],
        );
        match ictx.ch {
            0 => {}
            7 => {
                if let Some(window) = ictx.window_ref() {
                    window.raise_alerts(WINDOW_BELL);
                }
            }
            8 => {
                sctx.backspace();
            }
            9 => {
                let s = sctx.screen_mut();
                let (current_cx, current_cy) = s.cursor();
                cx = current_cx;
                if !(cx >= RustScreen::grid(s).width().wrapping_sub(1 as u_int)) {
                    line = current_cy.wrapping_add(RustScreen::grid(s).history_size());
                    first_gc = RustScreen::grid(&*s).cell(cx, line);
                    loop {
                        if has_content == 0 {
                            gc = RustScreen::grid(&*s).cell(cx, line);
                            if gc.data.size as core::ffi::c_int != 1 as core::ffi::c_int
                                || gc.data.data[0] != b' '
                                || grid_cells_look_equal(&gc, &first_gc) == 0
                            {
                                has_content = 1 as core::ffi::c_int;
                            }
                        }
                        cx = cx.wrapping_add(1);
                        if s.tab_is_set(cx) {
                            break;
                        }
                        if !(cx < RustScreen::grid(s).width().wrapping_sub(1 as u_int)) {
                            break;
                        }
                    }
                    width = cx.wrapping_sub(current_cx);
                    if has_content != 0 || width as usize > size_of::<[u_char; 32]>() {
                        s.set_cursor(cx, current_cy);
                    } else {
                        gc = RustScreen::grid(&*s).cell(current_cx, line);
                        grid_set_tab(&mut gc, width);
                        sctx.collect_add(&gc);
                    }
                }
            }
            10..=12 => {
                sctx.linefeed(0 as core::ffi::c_int, ictx.cell.cell.bg as u_int);
                if sctx.screen_mut().mode() & MODE_CRLF != 0 {
                    sctx.carriagereturn();
                }
            }
            13 => {
                sctx.carriagereturn();
            }
            14 => {
                ictx.cell.set = 1 as core::ffi::c_int;
            }
            15 => {
                ictx.cell.set = 0 as core::ffi::c_int;
            }
            _ => {
                log_debug(
                    c"%s: unknown '%c'",
                    fmt_args![c"input_c0_dispatch".as_ptr(), ictx.ch],
                );
            }
        }
        ictx.flags &= !INPUT_LAST;
        0 as core::ffi::c_int
    }
}
unsafe fn input_esc_dispatch(
    ictx: &mut input_ctx,
    sctx: &mut RustScreenWriteCtx<'_>,
) -> core::ffi::c_int {
    unsafe {
        if ictx.flags & INPUT_DISCARD != 0 {
            return 0 as core::ffi::c_int;
        }
        log_debug(
            c"%s: '%c', %s",
            fmt_args![
                c"input_esc_dispatch".as_ptr(),
                ictx.ch,
                CStr::from_bytes_until_nul(&ictx.interm_buf)
                    .expect("input intermediates have a trailing NUL")
            ],
        );
        let Some(entry) = input_table_find(ictx, &input_esc_table) else {
            log_debug(
                c"%s: unknown '%c'",
                fmt_args![c"input_esc_dispatch".as_ptr(), ictx.ch],
            );
            return 0 as core::ffi::c_int;
        };
        match entry.type_0 {
            9 => {
                ictx.clear_palette();
                input_reset_cell(ictx);
                sctx.reset();
                sctx.fullredraw();
            }
            6 => {
                sctx.linefeed(0 as core::ffi::c_int, ictx.cell.cell.bg as u_int);
            }
            7 => {
                sctx.carriagereturn();
                sctx.linefeed(0 as core::ffi::c_int, ictx.cell.cell.bg as u_int);
            }
            5 => {
                let s = sctx.screen_mut();
                let (cx, _) = s.cursor();
                if cx < RustScreen::grid(s).width() {
                    s.set_tab(cx);
                }
            }
            8 => {
                sctx.reverseindex(ictx.cell.cell.bg as u_int);
            }
            1 => {
                sctx.mode_set(MODE_KKEYPAD);
            }
            2 => {
                sctx.mode_clear(MODE_KKEYPAD);
            }
            4 => {
                input_save_state(ictx, sctx);
            }
            3 => {
                input_restore_state(ictx, sctx);
            }
            0 => {
                sctx.alignmenttest();
            }
            11 => {
                ictx.cell.g0set = 1 as core::ffi::c_int;
            }
            10 => {
                ictx.cell.g0set = 0 as core::ffi::c_int;
            }
            13 => {
                ictx.cell.g1set = 1 as core::ffi::c_int;
            }
            12 => {
                ictx.cell.g1set = 0 as core::ffi::c_int;
            }
            _ => {}
        }
        ictx.flags &= !INPUT_LAST;
        0 as core::ffi::c_int
    }
}
unsafe fn input_csi_dispatch(
    ictx: &mut input_ctx,
    sctx: &mut RustScreenWriteCtx<'_>,
) -> core::ffi::c_int {
    unsafe {
        let mut i: core::ffi::c_int;
        let mut n: core::ffi::c_int;
        let m: core::ffi::c_int;
        let ek: core::ffi::c_int;
        let set: core::ffi::c_int;
        let p: core::ffi::c_int;
        let mut cx: u_int;
        let bg: u_int = ictx.cell.cell.bg as u_int;
        if ictx.flags & INPUT_DISCARD != 0 {
            return 0 as core::ffi::c_int;
        }
        log_debug(
            c"%s: '%c' \"%s\" \"%s\"",
            fmt_args![
                c"input_csi_dispatch".as_ptr(),
                ictx.ch,
                CStr::from_bytes_until_nul(&ictx.interm_buf)
                    .expect("input parameters have a trailing NUL"),
                CStr::from_bytes_until_nul(&ictx.param_buf)
                    .expect("input parameters have a trailing NUL")
            ],
        );
        if input_split(ictx) != 0 as core::ffi::c_int {
            return 0 as core::ffi::c_int;
        }
        let Some(entry) = input_table_find(ictx, &input_csi_table) else {
            log_debug(
                c"%s: unknown '%c'",
                fmt_args![c"input_csi_dispatch".as_ptr(), ictx.ch],
            );
            return 0 as core::ffi::c_int;
        };
        match entry.type_0 {
            0 => {
                (cx, _) = sctx.cursor_position();
                if cx > sctx.size().0.wrapping_sub(1 as u_int) {
                    cx = sctx.size().0.wrapping_sub(1 as u_int);
                }
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as core::ffi::c_int,
                    1 as core::ffi::c_int,
                );
                if !(n == -(1 as core::ffi::c_int)) {
                    while cx > 0 as u_int && {
                        let fresh10 = n;
                        n -= 1;
                        fresh10 > 0 as core::ffi::c_int
                    } {
                        loop {
                            cx = cx.wrapping_sub(1);
                            if !(cx > 0 as u_int && !sctx.screen_mut().tab_is_set(cx)) {
                                break;
                            }
                        }
                    }
                    let (_, cy) = sctx.cursor_position();
                    sctx.screen_mut().set_cursor(cx, cy);
                }
            }
            3 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as core::ffi::c_int,
                    1 as core::ffi::c_int,
                );
                if n != -(1 as core::ffi::c_int) {
                    sctx.cursorleft(n as u_int);
                }
            }
            4 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as core::ffi::c_int,
                    1 as core::ffi::c_int,
                );
                if n != -(1 as core::ffi::c_int) {
                    sctx.cursordown(n as u_int);
                }
            }
            5 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as core::ffi::c_int,
                    1 as core::ffi::c_int,
                );
                if n != -(1 as core::ffi::c_int) {
                    sctx.cursorright(n as u_int);
                }
            }
            6 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as core::ffi::c_int,
                    1 as core::ffi::c_int,
                );
                m = input_get(
                    ictx,
                    1 as u_int,
                    1 as core::ffi::c_int,
                    1 as core::ffi::c_int,
                );
                if n != -(1 as core::ffi::c_int) && m != -(1 as core::ffi::c_int) {
                    sctx.cursormove(
                        m - 1 as core::ffi::c_int,
                        n - 1 as core::ffi::c_int,
                        1 as core::ffi::c_int,
                    );
                }
            }
            23 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    0 as core::ffi::c_int,
                    0 as core::ffi::c_int,
                );
                if !(n != 4 as core::ffi::c_int) {
                    m = input_get(
                        ictx,
                        1 as u_int,
                        0 as core::ffi::c_int,
                        0 as core::ffi::c_int,
                    );
                    ek = (global_options
                        .get()
                        .as_ref()
                        .expect("global options are initialized"))
                    .number(c"extended-keys") as core::ffi::c_int;
                    if !(ek == 0 as core::ffi::c_int) {
                        sctx.mode_clear(EXTENDED_KEY_MODES);
                        if m == 2 as core::ffi::c_int {
                            sctx.mode_set(MODE_KEYS_EXTENDED_2);
                        } else if m == 1 as core::ffi::c_int || ek == 2 as core::ffi::c_int {
                            sctx.mode_set(MODE_KEYS_EXTENDED);
                        }
                    }
                }
            }
            22 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    0 as core::ffi::c_int,
                    0 as core::ffi::c_int,
                );
                if !(n != 4 as core::ffi::c_int) {
                    sctx.mode_clear(MODE_KEYS_EXTENDED | MODE_KEYS_EXTENDED_2);
                    if (global_options
                        .get()
                        .as_ref()
                        .expect("global options are initialized"))
                    .number(c"extended-keys")
                        == 2 as core::ffi::c_longlong
                    {
                        sctx.mode_set(MODE_KEYS_EXTENDED);
                    }
                }
            }
            39 => {
                input_csi_dispatch_winops(ictx, sctx);
            }
            7 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as core::ffi::c_int,
                    1 as core::ffi::c_int,
                );
                if n != -(1 as core::ffi::c_int) {
                    sctx.cursorup(n as u_int);
                }
            }
            1 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as core::ffi::c_int,
                    1 as core::ffi::c_int,
                );
                if n != -(1 as core::ffi::c_int) {
                    sctx.carriagereturn();
                    sctx.cursordown(n as u_int);
                }
            }
            2 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as core::ffi::c_int,
                    1 as core::ffi::c_int,
                );
                if n != -(1 as core::ffi::c_int) {
                    sctx.carriagereturn();
                    sctx.cursorup(n as u_int);
                }
            }
            8 => {
                match input_get(
                    ictx,
                    0 as u_int,
                    0 as core::ffi::c_int,
                    0 as core::ffi::c_int,
                ) {
                    -1 => {}
                    0 => {
                        input_reply(ictx, 1 as core::ffi::c_int, c"\x1B[?1;2c", fmt_args![]);
                    }
                    _ => {
                        log_debug(
                            c"%s: unknown '%c'",
                            fmt_args![c"input_csi_dispatch".as_ptr(), ictx.ch],
                        );
                    }
                }
            }
            9 => {
                match input_get(
                    ictx,
                    0 as u_int,
                    0 as core::ffi::c_int,
                    0 as core::ffi::c_int,
                ) {
                    -1 => {}
                    0 => {
                        input_reply(ictx, 1 as core::ffi::c_int, c"\x1B[>84;0;0c", fmt_args![]);
                    }
                    _ => {
                        log_debug(
                            c"%s: unknown '%c'",
                            fmt_args![c"input_csi_dispatch".as_ptr(), ictx.ch],
                        );
                    }
                }
            }
            16 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as core::ffi::c_int,
                    1 as core::ffi::c_int,
                );
                if n != -(1 as core::ffi::c_int) {
                    sctx.clearcharacter(n as u_int, bg);
                }
            }
            10 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as core::ffi::c_int,
                    1 as core::ffi::c_int,
                );
                if n != -(1 as core::ffi::c_int) {
                    sctx.deletecharacter(n as u_int, bg);
                }
            }
            12 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as core::ffi::c_int,
                    1 as core::ffi::c_int,
                );
                m = input_get(
                    ictx,
                    1 as u_int,
                    1 as core::ffi::c_int,
                    sctx.size().1 as core::ffi::c_int,
                );
                if n != -(1 as core::ffi::c_int) && m != -(1 as core::ffi::c_int) {
                    sctx.scrollregion(
                        (n - 1 as core::ffi::c_int) as u_int,
                        (m - 1 as core::ffi::c_int) as u_int,
                    );
                }
            }
            13 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as core::ffi::c_int,
                    1 as core::ffi::c_int,
                );
                if n != -(1 as core::ffi::c_int) {
                    sctx.deleteline(n as u_int, bg);
                }
            }
            15 => {
                if input_get(
                    ictx,
                    0 as u_int,
                    0 as core::ffi::c_int,
                    0 as core::ffi::c_int,
                ) == 996
                {
                    input_report_current_theme(ictx);
                }
            }
            24 => {
                m = input_get(
                    ictx,
                    0 as u_int,
                    0 as core::ffi::c_int,
                    0 as core::ffi::c_int,
                );
                match m {
                    4 => {
                        n = if sctx.screen_mut().mode() & MODE_INSERT != 0 {
                            1 as core::ffi::c_int
                        } else {
                            2 as core::ffi::c_int
                        };
                    }
                    _ => {
                        n = 0 as core::ffi::c_int;
                    }
                }
                if m > 0 as core::ffi::c_int {
                    input_reply(
                        ictx,
                        1 as core::ffi::c_int,
                        c"\x1B[%d;%d$y",
                        fmt_args![m, n],
                    );
                }
            }
            25 => {
                m = input_get(
                    ictx,
                    0 as u_int,
                    0 as core::ffi::c_int,
                    0 as core::ffi::c_int,
                );
                match m {
                    1 => {
                        n = if sctx.screen_mut().mode() & MODE_KCURSOR != 0 {
                            1 as core::ffi::c_int
                        } else {
                            2 as core::ffi::c_int
                        };
                    }
                    3 => {
                        n = 4 as core::ffi::c_int;
                    }
                    6 => {
                        n = if sctx.screen_mut().mode() & MODE_ORIGIN != 0 {
                            1 as core::ffi::c_int
                        } else {
                            2 as core::ffi::c_int
                        };
                    }
                    7 => {
                        n = if sctx.screen_mut().mode() & MODE_WRAP != 0 {
                            1 as core::ffi::c_int
                        } else {
                            2 as core::ffi::c_int
                        };
                    }
                    12 => {
                        if sctx.screen_mut().cursor_style() as core::ffi::c_uint
                            != SCREEN_CURSOR_DEFAULT as core::ffi::c_int as core::ffi::c_uint
                            || sctx.screen_mut().mode() & MODE_CURSOR_BLINKING_SET != 0
                        {
                            n = if sctx.screen_mut().mode() & MODE_CURSOR_BLINKING != 0 {
                                1 as core::ffi::c_int
                            } else {
                                2 as core::ffi::c_int
                            };
                        } else {
                            let pane = ictx.pane_ref();
                            let options =
                                if let Some(pane) = pane.as_ref().and_then(|pane| pane.get()) {
                                    pane.options_ref().clone()
                                } else {
                                    global_w_options
                                        .get()
                                        .expect("global options are initialized")
                                };
                            p = options.number(c"cursor-style") as core::ffi::c_int;
                            n = if p == 1 as core::ffi::c_int
                                || p == 3 as core::ffi::c_int
                                || p == 5 as core::ffi::c_int
                            {
                                1 as core::ffi::c_int
                            } else {
                                2 as core::ffi::c_int
                            };
                        }
                    }
                    25 => {
                        n = if sctx.screen_mut().mode() & MODE_CURSOR != 0 {
                            1 as core::ffi::c_int
                        } else {
                            2 as core::ffi::c_int
                        };
                    }
                    47 | 1047 | 1049 => {
                        n = if sctx.screen_mut().is_alternate() {
                            1 as core::ffi::c_int
                        } else {
                            2 as core::ffi::c_int
                        };
                    }
                    1000 => {
                        n = if sctx.screen_mut().mode() & MODE_MOUSE_STANDARD != 0 {
                            1 as core::ffi::c_int
                        } else {
                            2 as core::ffi::c_int
                        };
                    }
                    1002 => {
                        n = if sctx.screen_mut().mode() & MODE_MOUSE_BUTTON != 0 {
                            1 as core::ffi::c_int
                        } else {
                            2 as core::ffi::c_int
                        };
                    }
                    1003 => {
                        n = if sctx.screen_mut().mode() & MODE_MOUSE_ALL != 0 {
                            1 as core::ffi::c_int
                        } else {
                            2 as core::ffi::c_int
                        };
                    }
                    1004 => {
                        n = if sctx.screen_mut().mode() & MODE_FOCUSON != 0 {
                            1 as core::ffi::c_int
                        } else {
                            2 as core::ffi::c_int
                        };
                    }
                    1005 => {
                        n = if sctx.screen_mut().mode() & MODE_MOUSE_UTF8 != 0 {
                            1 as core::ffi::c_int
                        } else {
                            2 as core::ffi::c_int
                        };
                    }
                    1006 => {
                        n = if sctx.screen_mut().mode() & MODE_MOUSE_SGR != 0 {
                            1 as core::ffi::c_int
                        } else {
                            2 as core::ffi::c_int
                        };
                    }
                    2004 => {
                        n = if sctx.screen_mut().mode() & MODE_BRACKETPASTE != 0 {
                            1 as core::ffi::c_int
                        } else {
                            2 as core::ffi::c_int
                        };
                    }
                    2026 => {
                        n = if sctx.screen_mut().mode() & MODE_SYNC != 0 {
                            1 as core::ffi::c_int
                        } else {
                            2 as core::ffi::c_int
                        };
                    }
                    2031 => {
                        n = if sctx.screen_mut().mode() & MODE_THEME_UPDATES != 0 {
                            1 as core::ffi::c_int
                        } else {
                            2 as core::ffi::c_int
                        };
                    }
                    _ => {
                        n = 0 as core::ffi::c_int;
                    }
                }
                if m > 0 as core::ffi::c_int {
                    input_reply(
                        ictx,
                        1 as core::ffi::c_int,
                        c"\x1B[?%d;%d$y",
                        fmt_args![m, n],
                    );
                }
            }
            14 => {
                match input_get(
                    ictx,
                    0 as u_int,
                    0 as core::ffi::c_int,
                    0 as core::ffi::c_int,
                ) {
                    -1 => {}
                    5 => {
                        input_reply(ictx, 1 as core::ffi::c_int, c"\x1B[0n", fmt_args![]);
                    }
                    6 => {
                        input_reply(
                            ictx,
                            1 as core::ffi::c_int,
                            c"\x1B[%u;%uR",
                            fmt_args![
                                sctx.cursor_position().1.wrapping_add(1 as u_int),
                                sctx.cursor_position().0.wrapping_add(1 as u_int)
                            ],
                        );
                    }
                    _ => {
                        log_debug(
                            c"%s: unknown '%c'",
                            fmt_args![c"input_csi_dispatch".as_ptr(), ictx.ch],
                        );
                    }
                }
            }
            17 => {
                match input_get(
                    ictx,
                    0 as u_int,
                    0 as core::ffi::c_int,
                    0 as core::ffi::c_int,
                ) {
                    -1 => {}
                    0 => {
                        sctx.clearendofscreen(bg);
                    }
                    1 => {
                        sctx.clearstartofscreen(bg);
                    }
                    2 => {
                        sctx.clearscreen(bg);
                    }
                    3 => {
                        if input_get(
                            ictx,
                            1 as u_int,
                            0 as core::ffi::c_int,
                            0 as core::ffi::c_int,
                        ) == 0 as core::ffi::c_int
                        {
                            sctx.clearhistory();
                        }
                    }
                    _ => {
                        log_debug(
                            c"%s: unknown '%c'",
                            fmt_args![c"input_csi_dispatch".as_ptr(), ictx.ch],
                        );
                    }
                }
            }
            18 => {
                match input_get(
                    ictx,
                    0 as u_int,
                    0 as core::ffi::c_int,
                    0 as core::ffi::c_int,
                ) {
                    -1 => {}
                    0 => {
                        sctx.clearendofline(bg);
                    }
                    1 => {
                        sctx.clearstartofline(bg);
                    }
                    2 => {
                        sctx.clearline(bg);
                    }
                    _ => {
                        log_debug(
                            c"%s: unknown '%c'",
                            fmt_args![c"input_csi_dispatch".as_ptr(), ictx.ch],
                        );
                    }
                }
            }
            19 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as core::ffi::c_int,
                    1 as core::ffi::c_int,
                );
                if n != -(1 as core::ffi::c_int) {
                    sctx.cursormove(
                        n - 1 as core::ffi::c_int,
                        -(1 as core::ffi::c_int),
                        1 as core::ffi::c_int,
                    );
                }
            }
            20 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as core::ffi::c_int,
                    1 as core::ffi::c_int,
                );
                if n != -(1 as core::ffi::c_int) {
                    sctx.insertcharacter(n as u_int, bg);
                }
            }
            21 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as core::ffi::c_int,
                    1 as core::ffi::c_int,
                );
                if n != -(1 as core::ffi::c_int) {
                    sctx.insertline(n as u_int, bg);
                }
            }
            27 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as core::ffi::c_int,
                    1 as core::ffi::c_int,
                );
                if !(n == -(1 as core::ffi::c_int)) {
                    m = sctx.size().0.wrapping_sub(sctx.cursor_position().0) as core::ffi::c_int;
                    if n > m {
                        n = m;
                    }
                    if !(!ictx.flags & INPUT_LAST != 0) {
                        set = if ictx.cell.set == 0 as core::ffi::c_int {
                            ictx.cell.g0set
                        } else {
                            ictx.cell.g1set
                        };
                        if set == 1 as core::ffi::c_int {
                            ictx.cell.cell.attr = (ictx.cell.cell.attr as core::ffi::c_int
                                | GRID_ATTR_CHARSET)
                                as u_short;
                        } else {
                            ictx.cell.cell.attr = (ictx.cell.cell.attr as core::ffi::c_int
                                & !GRID_ATTR_CHARSET)
                                as u_short;
                        }
                        utf8_copy(&mut ictx.cell.cell.data, &ictx.last);
                        i = 0 as core::ffi::c_int;
                        while i < n {
                            sctx.collect_add(&ictx.cell.cell);
                            i += 1;
                        }
                    }
                }
            }
            26 => {
                input_restore_state(ictx, sctx);
            }
            28 => {
                input_csi_dispatch_rm(ictx, sctx);
            }
            29 => {
                input_csi_dispatch_rm_private(ictx, sctx);
            }
            30 => {
                input_save_state(ictx, sctx);
            }
            32 => {
                input_csi_dispatch_sgr(ictx);
            }
            33 => {
                input_csi_dispatch_sm(ictx, sctx);
            }
            35 => {
                input_csi_dispatch_sm_private(ictx, sctx);
            }
            34 => {
                input_csi_dispatch_sm_graphics(ictx);
            }
            36 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as core::ffi::c_int,
                    1 as core::ffi::c_int,
                );
                if n != -(1 as core::ffi::c_int) {
                    sctx.scrollup(n as u_int, bg);
                }
            }
            31 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as core::ffi::c_int,
                    1 as core::ffi::c_int,
                );
                if n != -(1 as core::ffi::c_int) {
                    sctx.scrolldown(n as u_int, bg);
                }
            }
            37 => {
                match input_get(
                    ictx,
                    0 as u_int,
                    0 as core::ffi::c_int,
                    0 as core::ffi::c_int,
                ) {
                    -1 => {}
                    0 => {
                        let (cx, _) = sctx.cursor_position();
                        if cx < sctx.size().0 {
                            sctx.screen_mut().clear_tab(cx);
                        }
                    }
                    3 => {
                        sctx.screen_mut().clear_tabs();
                    }
                    _ => {
                        log_debug(
                            c"%s: unknown '%c'",
                            fmt_args![c"input_csi_dispatch".as_ptr(), ictx.ch],
                        );
                    }
                }
            }
            38 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    1 as core::ffi::c_int,
                    1 as core::ffi::c_int,
                );
                if n != -(1 as core::ffi::c_int) {
                    sctx.cursormove(
                        -(1 as core::ffi::c_int),
                        n - 1 as core::ffi::c_int,
                        1 as core::ffi::c_int,
                    );
                }
            }
            11 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    0 as core::ffi::c_int,
                    0 as core::ffi::c_int,
                );
                if !(n == -(1 as core::ffi::c_int)) {
                    sctx.screen_mut().set_cursor_style(n as u_int);
                    if n == 0 as core::ffi::c_int {
                        sctx.mode_clear(MODE_CURSOR_BLINKING_SET);
                    }
                }
            }
            40 => {
                n = input_get(
                    ictx,
                    0 as u_int,
                    0 as core::ffi::c_int,
                    0 as core::ffi::c_int,
                );
                if n == 0 as core::ffi::c_int {
                    input_reply(
                        ictx,
                        1 as core::ffi::c_int,
                        c"\x1BP>|tmux %s\x1B\\",
                        fmt_args![getversion()],
                    );
                }
            }
            _ => {}
        }
        ictx.flags &= !INPUT_LAST;
        0 as core::ffi::c_int
    }
}
fn input_csi_dispatch_rm(ictx: &mut input_ctx, sctx: &mut RustScreenWriteCtx<'_>) {
    {
        let mut i: u_int;
        i = 0 as u_int;
        while i < ictx.param_list_len {
            match input_get(ictx, i, 0 as core::ffi::c_int, -(1 as core::ffi::c_int)) {
                -1 => {}
                4 => {
                    sctx.mode_clear(MODE_INSERT);
                }
                34 => {
                    sctx.mode_set(MODE_CURSOR_VERY_VISIBLE);
                }
                _ => {
                    log_debug(
                        c"%s: unknown '%c'",
                        fmt_args![c"input_csi_dispatch_rm".as_ptr(), ictx.ch],
                    );
                }
            }
            i = i.wrapping_add(1);
        }
    }
}
unsafe fn input_csi_dispatch_rm_private(ictx: &mut input_ctx, sctx: &mut RustScreenWriteCtx<'_>) {
    unsafe {
        let mut i: u_int;
        i = 0 as u_int;
        while i < ictx.param_list_len {
            match input_get(ictx, i, 0 as core::ffi::c_int, -(1 as core::ffi::c_int)) {
                -1 => {}
                1 => {
                    sctx.mode_clear(MODE_KCURSOR);
                }
                3 => {
                    sctx.cursormove(
                        0 as core::ffi::c_int,
                        0 as core::ffi::c_int,
                        1 as core::ffi::c_int,
                    );
                    sctx.clearscreen(ictx.cell.cell.bg as u_int);
                }
                6 => {
                    sctx.mode_clear(MODE_ORIGIN);
                    sctx.cursormove(
                        0 as core::ffi::c_int,
                        0 as core::ffi::c_int,
                        1 as core::ffi::c_int,
                    );
                }
                7 => {
                    sctx.mode_clear(MODE_WRAP);
                }
                12 => {
                    sctx.mode_clear(MODE_CURSOR_BLINKING);
                    sctx.mode_set(MODE_CURSOR_BLINKING_SET);
                }
                25 => {
                    sctx.mode_clear(MODE_CURSOR);
                }
                1000..=1003 => {
                    sctx.mode_clear(ALL_MOUSE_MODES);
                }
                1004 => {
                    sctx.mode_clear(MODE_FOCUSON);
                }
                1005 => {
                    sctx.mode_clear(MODE_MOUSE_UTF8);
                }
                1006 => {
                    sctx.mode_clear(MODE_MOUSE_SGR);
                }
                47 | 1047 => {
                    sctx.alternateoff(&mut ictx.cell.cell, 0 as core::ffi::c_int);
                }
                1049 => {
                    sctx.alternateoff(&mut ictx.cell.cell, 1 as core::ffi::c_int);
                }
                2004 => {
                    sctx.mode_clear(MODE_BRACKETPASTE);
                }
                2026 => {
                    let mut pane = ictx.pane_ref();
                    if let Some(wp) = pane.as_mut().and_then(|pane| pane.get_mut()) { wp.stop_sync(); }
                    if let Some(wp) = pane.as_mut().and_then(|pane| pane.get_mut()) {
                        wp.request_redraw();
                    }
                }
                2031 => {
                    let mut pane = ictx.pane_ref();
                    if let Some(wp) = pane.as_mut().and_then(|pane| pane.get_mut()) {
                        wp.set_theme_updates(false);
                    } else { sctx.mode_clear(MODE_THEME_UPDATES); }
                }
                _ => {
                    log_debug(
                        c"%s: unknown '%c'",
                        fmt_args![c"input_csi_dispatch_rm_private".as_ptr(), ictx.ch],
                    );
                }
            }
            i = i.wrapping_add(1);
        }
    }
}
fn input_csi_dispatch_sm(ictx: &mut input_ctx, sctx: &mut RustScreenWriteCtx<'_>) {
    {
        let mut i: u_int;
        i = 0 as u_int;
        while i < ictx.param_list_len {
            match input_get(ictx, i, 0 as core::ffi::c_int, -(1 as core::ffi::c_int)) {
                -1 => {}
                4 => {
                    sctx.mode_set(MODE_INSERT);
                }
                34 => {
                    sctx.mode_clear(MODE_CURSOR_VERY_VISIBLE);
                }
                _ => {
                    log_debug(
                        c"%s: unknown '%c'",
                        fmt_args![c"input_csi_dispatch_sm".as_ptr(), ictx.ch],
                    );
                }
            }
            i = i.wrapping_add(1);
        }
    }
}
unsafe fn input_csi_dispatch_sm_private(ictx: &mut input_ctx, sctx: &mut RustScreenWriteCtx<'_>) {
    unsafe {
        let mut i: u_int;
        i = 0 as u_int;
        while i < ictx.param_list_len {
            match input_get(ictx, i, 0 as core::ffi::c_int, -(1 as core::ffi::c_int)) {
                -1 => {}
                1 => {
                    sctx.mode_set(MODE_KCURSOR);
                }
                3 => {
                    sctx.cursormove(
                        0 as core::ffi::c_int,
                        0 as core::ffi::c_int,
                        1 as core::ffi::c_int,
                    );
                    sctx.clearscreen(ictx.cell.cell.bg as u_int);
                }
                6 => {
                    sctx.mode_set(MODE_ORIGIN);
                    sctx.cursormove(
                        0 as core::ffi::c_int,
                        0 as core::ffi::c_int,
                        1 as core::ffi::c_int,
                    );
                }
                7 => {
                    sctx.mode_set(MODE_WRAP);
                }
                12 => {
                    sctx.mode_set(MODE_CURSOR_BLINKING);
                    sctx.mode_set(MODE_CURSOR_BLINKING_SET);
                }
                25 => {
                    sctx.mode_set(MODE_CURSOR);
                }
                1000 => {
                    sctx.mode_clear(ALL_MOUSE_MODES);
                    sctx.mode_set(MODE_MOUSE_STANDARD);
                }
                1002 => {
                    sctx.mode_clear(ALL_MOUSE_MODES);
                    sctx.mode_set(MODE_MOUSE_BUTTON);
                }
                1003 => {
                    sctx.mode_clear(ALL_MOUSE_MODES);
                    sctx.mode_set(MODE_MOUSE_ALL);
                }
                1004 => {
                    sctx.mode_set(MODE_FOCUSON);
                }
                1005 => {
                    sctx.mode_set(MODE_MOUSE_UTF8);
                }
                1006 => {
                    sctx.mode_set(MODE_MOUSE_SGR);
                }
                47 | 1047 => {
                    sctx.alternateon(&mut ictx.cell.cell, 0 as core::ffi::c_int);
                }
                1049 => {
                    sctx.alternateon(&mut ictx.cell.cell, 1 as core::ffi::c_int);
                }
                2004 => {
                    sctx.mode_set(MODE_BRACKETPASTE);
                }
                2031 => {
                    let mut pane = ictx.pane_ref();
                    if let Some(wp) = pane.as_mut().and_then(|pane| pane.get_mut()) {
                        wp.set_theme_updates(true);
                    } else { sctx.mode_set(MODE_THEME_UPDATES); }
                }
                2026 => {
                    let mut pane = ictx.pane_ref();
                    if let Some(wp) = pane.as_mut().and_then(|pane| pane.get_mut()) { wp.start_sync(); }
                }
                _ => {
                    log_debug(
                        c"%s: unknown '%c'",
                        fmt_args![c"input_csi_dispatch_sm_private".as_ptr(), ictx.ch],
                    );
                }
            }
            i = i.wrapping_add(1);
        }
    }
}
fn input_csi_dispatch_sm_graphics(_ictx: &mut input_ctx) {}
unsafe fn input_csi_dispatch_winops(ictx: &mut input_ctx, sctx: &mut RustScreenWriteCtx<'_>) {
    unsafe {
        let window = ictx.window_ref();
        let (x, y) = sctx.size();
        let mut at = 0;
        loop {
            let operation = input_get(ictx, at, 0, -1);
            if operation == -1 {
                break;
            }
            match operation {
                1 | 2 | 5 | 6 | 7 | 11 | 13 | 20 | 21 | 24 => {}
                3 | 4 | 8 | 9 | 10 => {
                    let count = if matches!(operation, 3 | 4 | 8) { 2 } else { 1 };
                    for _ in 0..count {
                        at += 1;
                        if input_get(ictx, at, 0, -1) == -1 {
                            return;
                        }
                    }
                }
                14..=16 => {
                    if let Some(window) = window.as_ref() {
                        let pixels = window.dimensions().pixels;
                        let (height, width) = if operation == 16 {
                            (pixels.height, pixels.width)
                        } else {
                            (y.wrapping_mul(pixels.height), x.wrapping_mul(pixels.width))
                        };
                        let format = match operation {
                            14 => c"\x1B[4;%u;%ut",
                            15 => c"\x1B[5;%u;%ut",
                            _ => c"\x1B[6;%u;%ut",
                        };
                        input_reply(ictx, 1, format, fmt_args![height, width]);
                    }
                }
                18 | 19 => {
                    let format = if operation == 18 {
                        c"\x1B[8;%u;%ut"
                    } else {
                        c"\x1B[9;%u;%ut"
                    };
                    input_reply(ictx, 1, format, fmt_args![y, x]);
                }
                22 | 23 => {
                    at += 1;
                    match input_get(ictx, at, 0, -1) {
                        -1 => return,
                        0 | 2 if operation == 22 => sctx.screen_mut().push_title(),
                        0 | 2 => {
                            sctx.screen_mut().pop_title();
                            if let Some(pane) = ictx.pane_ref() {
                                notify_pane(c"pane-title-changed", pane.get());
                            }
                            if let Some(window) = window.as_ref() {
                                window.redraw_borders();
                                window.redraw_status();
                            }
                        }
                        _ => {}
                    }
                }
                _ => log_debug(
                    c"%s: unknown '%c'",
                    fmt_args![c"input_csi_dispatch_winops", ictx.ch],
                ),
            }
            at += 1;
        }
    }
}

fn input_csi_dispatch_sgr_256_do(
    ictx: &mut input_ctx,
    fgbg: core::ffi::c_int,
    c: core::ffi::c_int,
) -> core::ffi::c_int {
    {
        let gc: &mut grid_cell = &mut ictx.cell.cell;
        if c == -(1 as core::ffi::c_int) || c > 255 as core::ffi::c_int {
            if fgbg == 38 as core::ffi::c_int {
                gc.fg = 8 as core::ffi::c_int;
            } else if fgbg == 48 as core::ffi::c_int {
                gc.bg = 8 as core::ffi::c_int;
            }
        } else if fgbg == 38 as core::ffi::c_int {
            gc.fg = c | COLOUR_FLAG_256;
        } else if fgbg == 48 as core::ffi::c_int {
            gc.bg = c | COLOUR_FLAG_256;
        } else if fgbg == 58 as core::ffi::c_int {
            gc.us = c | COLOUR_FLAG_256;
        }
        1 as core::ffi::c_int
    }
}
fn input_csi_dispatch_sgr_256(ictx: &mut input_ctx, fgbg: core::ffi::c_int, i: &mut u_int) {
    {
        let c: core::ffi::c_int = input_get(
            ictx,
            (*i).wrapping_add(1 as u_int),
            0 as core::ffi::c_int,
            -(1 as core::ffi::c_int),
        );
        if input_csi_dispatch_sgr_256_do(ictx, fgbg, c) != 0 {
            *i = (*i).wrapping_add(1);
        }
    }
}
fn input_csi_dispatch_sgr_rgb_do(
    ictx: &mut input_ctx,
    fgbg: core::ffi::c_int,
    r: core::ffi::c_int,
    g: core::ffi::c_int,
    b: core::ffi::c_int,
) -> core::ffi::c_int {
    {
        let gc: &mut grid_cell = &mut ictx.cell.cell;
        if r == -(1 as core::ffi::c_int) || r > 255 as core::ffi::c_int {
            return 0 as core::ffi::c_int;
        }
        if g == -(1 as core::ffi::c_int) || g > 255 as core::ffi::c_int {
            return 0 as core::ffi::c_int;
        }
        if b == -(1 as core::ffi::c_int) || b > 255 as core::ffi::c_int {
            return 0 as core::ffi::c_int;
        }
        if fgbg == 38 as core::ffi::c_int {
            gc.fg = RustColourEngine.join_rgb(r as u_char, g as u_char, b as u_char);
        } else if fgbg == 48 as core::ffi::c_int {
            gc.bg = RustColourEngine.join_rgb(r as u_char, g as u_char, b as u_char);
        } else if fgbg == 58 as core::ffi::c_int {
            gc.us = RustColourEngine.join_rgb(r as u_char, g as u_char, b as u_char);
        }
        1 as core::ffi::c_int
    }
}
fn input_csi_dispatch_sgr_rgb(ictx: &mut input_ctx, fgbg: core::ffi::c_int, i: &mut u_int) {
    {
        let r: core::ffi::c_int = input_get(
            ictx,
            (*i).wrapping_add(1 as u_int),
            0 as core::ffi::c_int,
            -(1 as core::ffi::c_int),
        );
        let g: core::ffi::c_int = input_get(
            ictx,
            (*i).wrapping_add(2 as u_int),
            0 as core::ffi::c_int,
            -(1 as core::ffi::c_int),
        );
        let b: core::ffi::c_int = input_get(
            ictx,
            (*i).wrapping_add(3 as u_int),
            0 as core::ffi::c_int,
            -(1 as core::ffi::c_int),
        );
        if input_csi_dispatch_sgr_rgb_do(ictx, fgbg, r, g, b) != 0 {
            *i = (*i).wrapping_add(3 as u_int);
        }
    }
}
unsafe fn input_csi_dispatch_sgr_colon(ictx: &mut input_ctx, mut i: u_int) {
    unsafe {
        let InputParam::Str(value) = &ictx.param_list[i as usize] else {
            return;
        };
        let mut copy = value.to_bytes_with_nul().to_vec();
        let mut p = [-1; 8];
        let mut n = 0;
        for field in input_fields(&mut copy, b':') {
            if !field.is_empty() {
                let Ok(value) = strtonum(field, 0, INT_MAX as core::ffi::c_longlong) else {
                    return;
                };
                p[n as usize] = value as core::ffi::c_int;
            }
            n += 1;
            if n as usize == p.len() {
                return;
            }
            log_debug(
                c"%s: %u = %d",
                fmt_args![
                    c"input_csi_dispatch_sgr_colon".as_ptr(),
                    n - 1,
                    p[n as usize - 1]
                ],
            );
        }
        if n == 0 as u_int {
            return;
        }
        if p[0 as core::ffi::c_int as usize] == 4 as core::ffi::c_int {
            if n != 2 as u_int {
                return;
            }
            let gc = &mut ictx.cell.cell;
            match p[1 as core::ffi::c_int as usize] {
                0 => {
                    gc.attr = (gc.attr as core::ffi::c_int & !GRID_ATTR_ALL_UNDERSCORE) as u_short;
                }
                1 => {
                    gc.attr = (gc.attr as core::ffi::c_int & !GRID_ATTR_ALL_UNDERSCORE) as u_short;
                    gc.attr = (gc.attr as core::ffi::c_int | GRID_ATTR_UNDERSCORE) as u_short;
                }
                2 => {
                    gc.attr = (gc.attr as core::ffi::c_int & !GRID_ATTR_ALL_UNDERSCORE) as u_short;
                    gc.attr = (gc.attr as core::ffi::c_int | GRID_ATTR_UNDERSCORE_2) as u_short;
                }
                3 => {
                    gc.attr = (gc.attr as core::ffi::c_int & !GRID_ATTR_ALL_UNDERSCORE) as u_short;
                    gc.attr = (gc.attr as core::ffi::c_int | GRID_ATTR_UNDERSCORE_3) as u_short;
                }
                4 => {
                    gc.attr = (gc.attr as core::ffi::c_int & !GRID_ATTR_ALL_UNDERSCORE) as u_short;
                    gc.attr = (gc.attr as core::ffi::c_int | GRID_ATTR_UNDERSCORE_4) as u_short;
                }
                5 => {
                    gc.attr = (gc.attr as core::ffi::c_int & !GRID_ATTR_ALL_UNDERSCORE) as u_short;
                    gc.attr = (gc.attr as core::ffi::c_int | GRID_ATTR_UNDERSCORE_5) as u_short;
                }
                _ => {}
            }
            return;
        }
        if n < 2 as u_int
            || p[0 as core::ffi::c_int as usize] != 38 as core::ffi::c_int
                && p[0 as core::ffi::c_int as usize] != 48 as core::ffi::c_int
                && p[0 as core::ffi::c_int as usize] != 58 as core::ffi::c_int
        {
            return;
        }
        match p[1 as core::ffi::c_int as usize] {
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
                            p[0 as core::ffi::c_int as usize],
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
                    p[0 as core::ffi::c_int as usize],
                    p[2 as core::ffi::c_int as usize],
                );
            }
            _ => {}
        };
    }
}
unsafe fn input_csi_dispatch_sgr(ictx: &mut input_ctx) {
    unsafe {
        let mut i: u_int;
        let mut link: u_int;
        let mut n: core::ffi::c_int;
        if ictx.param_list_len == 0 as u_int {
            ictx.cell.cell = grid_default_cell;
            return;
        }
        i = 0 as u_int;
        while i < ictx.param_list_len {
            if matches!(input_params(ictx)[i as usize], InputParam::Str(_)) {
                input_csi_dispatch_sgr_colon(ictx, i);
            } else {
                n = input_get(ictx, i, 0 as core::ffi::c_int, 0 as core::ffi::c_int);
                if !(n == -(1 as core::ffi::c_int)) {
                    if n == 38 as core::ffi::c_int
                        || n == 48 as core::ffi::c_int
                        || n == 58 as core::ffi::c_int
                    {
                        i = i.wrapping_add(1);
                        match input_get(ictx, i, 0 as core::ffi::c_int, -(1 as core::ffi::c_int)) {
                            2 => {
                                input_csi_dispatch_sgr_rgb(ictx, n, &mut i);
                            }
                            5 => {
                                input_csi_dispatch_sgr_256(ictx, n, &mut i);
                            }
                            _ => {}
                        }
                    } else {
                        let gc = &mut ictx.cell.cell;
                        match n {
                            0 => {
                                link = gc.link;
                                *gc = grid_default_cell;
                                gc.link = link;
                            }
                            1 => {
                                gc.attr =
                                    (gc.attr as core::ffi::c_int | GRID_ATTR_BRIGHT) as u_short;
                            }
                            2 => {
                                gc.attr = (gc.attr as core::ffi::c_int | GRID_ATTR_DIM) as u_short;
                            }
                            3 => {
                                gc.attr =
                                    (gc.attr as core::ffi::c_int | GRID_ATTR_ITALICS) as u_short;
                            }
                            4 => {
                                gc.attr = (gc.attr as core::ffi::c_int & !GRID_ATTR_ALL_UNDERSCORE)
                                    as u_short;
                                gc.attr =
                                    (gc.attr as core::ffi::c_int | GRID_ATTR_UNDERSCORE) as u_short;
                            }
                            5 | 6 => {
                                gc.attr =
                                    (gc.attr as core::ffi::c_int | GRID_ATTR_BLINK) as u_short;
                            }
                            7 => {
                                gc.attr =
                                    (gc.attr as core::ffi::c_int | GRID_ATTR_REVERSE) as u_short;
                            }
                            8 => {
                                gc.attr =
                                    (gc.attr as core::ffi::c_int | GRID_ATTR_HIDDEN) as u_short;
                            }
                            9 => {
                                gc.attr = (gc.attr as core::ffi::c_int | GRID_ATTR_STRIKETHROUGH)
                                    as u_short;
                            }
                            21 => {
                                gc.attr = (gc.attr as core::ffi::c_int & !GRID_ATTR_ALL_UNDERSCORE)
                                    as u_short;
                                gc.attr = (gc.attr as core::ffi::c_int | GRID_ATTR_UNDERSCORE_2)
                                    as u_short;
                            }
                            22 => {
                                gc.attr = (gc.attr as core::ffi::c_int
                                    & !(GRID_ATTR_BRIGHT | GRID_ATTR_DIM))
                                    as u_short;
                            }
                            23 => {
                                gc.attr =
                                    (gc.attr as core::ffi::c_int & !GRID_ATTR_ITALICS) as u_short;
                            }
                            24 => {
                                gc.attr = (gc.attr as core::ffi::c_int & !GRID_ATTR_ALL_UNDERSCORE)
                                    as u_short;
                            }
                            25 => {
                                gc.attr =
                                    (gc.attr as core::ffi::c_int & !GRID_ATTR_BLINK) as u_short;
                            }
                            27 => {
                                gc.attr =
                                    (gc.attr as core::ffi::c_int & !GRID_ATTR_REVERSE) as u_short;
                            }
                            28 => {
                                gc.attr =
                                    (gc.attr as core::ffi::c_int & !GRID_ATTR_HIDDEN) as u_short;
                            }
                            29 => {
                                gc.attr = (gc.attr as core::ffi::c_int & !GRID_ATTR_STRIKETHROUGH)
                                    as u_short;
                            }
                            30..=37 => {
                                gc.fg = n - 30 as core::ffi::c_int;
                            }
                            39 => {
                                gc.fg = 8 as core::ffi::c_int;
                            }
                            40..=47 => {
                                gc.bg = n - 40 as core::ffi::c_int;
                            }
                            49 => {
                                gc.bg = 8 as core::ffi::c_int;
                            }
                            53 => {
                                gc.attr =
                                    (gc.attr as core::ffi::c_int | GRID_ATTR_OVERLINE) as u_short;
                            }
                            55 => {
                                gc.attr =
                                    (gc.attr as core::ffi::c_int & !GRID_ATTR_OVERLINE) as u_short;
                            }
                            59 => {
                                gc.us = 8 as core::ffi::c_int;
                            }
                            90..=97 => {
                                gc.fg = n;
                            }
                            100..=107 => {
                                gc.bg = n - 10 as core::ffi::c_int;
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
fn input_end_bel(ictx: &mut input_ctx) -> core::ffi::c_int {
    {
        log_debug(c"%s", fmt_args![c"input_end_bel".as_ptr()]);
        ictx.input_end = INPUT_END_BEL;
        0 as core::ffi::c_int
    }
}
fn input_enter_dcs(ictx: &mut input_ctx) {
    {
        log_debug(c"%s", fmt_args![c"input_enter_dcs".as_ptr()]);
        input_clear(&mut *ictx);
        input_start_ground_timer(&mut *ictx);
        ictx.flags &= !INPUT_LAST;
    }
}
unsafe fn input_handle_decrqss(
    ictx: &mut input_ctx,
    sctx: &mut RustScreenWriteCtx<'_>,
) -> core::ffi::c_int {
    unsafe {
        if input_length(ictx) < 3 || ictx.input_buf.get(1..3) != Some(b" q") {
            input_reply(ictx, 1, c"\x1BP0$r\x1B\\", fmt_args![]);
            return 0;
        }
        let screen = sctx.screen_mut();
        let style = screen.cursor_style();
        let mode = screen.mode();
        let blinking = mode & MODE_CURSOR_BLINKING != 0;
        let ps = match style {
            SCREEN_CURSOR_BLOCK => {
                if blinking {
                    1
                } else {
                    2
                }
            }
            SCREEN_CURSOR_UNDERLINE => {
                if blinking {
                    3
                } else {
                    4
                }
            }
            SCREEN_CURSOR_BAR => {
                if blinking {
                    5
                } else {
                    6
                }
            }
            _ => {
                let pane = ictx.pane_ref();
                let options = if let Some(pane) = pane.as_ref().and_then(|pane| pane.get()) {
                    pane.options_ref().clone()
                } else {
                    global_w_options
                        .get()
                        .expect("global options are initialized")
                };
                let value = (options).number(c"cursor-style") as core::ffi::c_int;
                if (0..=6).contains(&value) { value } else { 0 }
            }
        };
        log_debug(
            c"%s: DECRQSS cursor -> Ps=%d (cstyle=%d mode=%#x)",
            fmt_args![
                c"input_handle_decrqss",
                ps,
                style as core::ffi::c_uint,
                mode
            ],
        );
        input_reply(ictx, 1, c"\x1BP1$r q%d q\x1B\\", fmt_args![ps]);
        0
    }
}
unsafe fn input_dcs_dispatch(
    ictx: &mut input_ctx,
    sctx: &mut RustScreenWriteCtx<'_>,
) -> core::ffi::c_int {
    unsafe {
        let len = input_length(ictx);
        if ictx.flags & INPUT_DISCARD != 0 {
            log_debug(
                c"%s: %zu bytes (discard)",
                fmt_args![c"input_dcs_dispatch", len],
            );
            return 0;
        }
        if ictx.interm_len == 1
            && ictx.interm_buf[0] == b'$'
            && ictx.input_buf[..len].starts_with(b"q")
        {
            return input_handle_decrqss(ictx, sctx);
        }
        let pane = ictx.pane_ref();
        let options = if let Some(pane) = pane.as_ref().and_then(|pane| pane.get()) {
            pane.options_ref().clone()
        } else {
            global_w_options
                .get()
                .expect("global options are initialized")
        };
        let allow_passthrough = (options).number(c"allow-passthrough");
        if allow_passthrough == 0 {
            return 0;
        }
        let input =
            CStr::from_bytes_until_nul(&ictx.input_buf).expect("DCS input has a trailing NUL");
        log_debug(c"%s: \"%s\"", fmt_args![c"input_dcs_dispatch", input]);
        if let Some(payload) = ictx.input_buf[..len].strip_prefix(b"tmux;") {
            sctx.rawstring(payload, (allow_passthrough == 2) as core::ffi::c_int);
        }
        0
    }
}

fn input_enter_osc(ictx: &mut input_ctx) {
    {
        log_debug(c"%s", fmt_args![c"input_enter_osc".as_ptr()]);
        input_clear(&mut *ictx);
        input_start_ground_timer(&mut *ictx);
        ictx.flags &= !INPUT_LAST;
    }
}
unsafe fn input_exit_osc(ictx: &mut input_ctx, sctx: &mut RustScreenWriteCtx<'_>) {
    unsafe {
        if ictx.flags & INPUT_DISCARD != 0 {
            return;
        }
        let input =
            CStr::from_bytes_until_nul(&ictx.input_buf).expect("OSC input has a trailing NUL");
        let bytes = input.to_bytes();
        if !bytes.first().is_some_and(u8::is_ascii_digit) {
            return;
        }
        log_debug(
            c"%s: \"%s\" (end %s)",
            fmt_args![
                c"input_exit_osc",
                input,
                if ictx.input_end == INPUT_END_ST {
                    c"ST"
                } else {
                    c"BEL"
                }
            ],
        );
        let mut option: u_int = 0;
        let mut offset = 0;
        while let Some(digit) = bytes.get(offset).filter(|byte| byte.is_ascii_digit()) {
            option = option
                .wrapping_mul(10)
                .wrapping_add((digit - b'0') as u_int);
            offset += 1;
        }
        match bytes.get(offset) {
            Some(b';') => offset += 1,
            None => {}
            _ => return,
        }
        let buffer = std::mem::take(&mut ictx.input_buf);
        let payload = CStr::from_bytes_until_nul(&buffer[offset..])
            .expect("the OSC payload retains the input's trailing NUL");
        match option {
            0 | 2 => {
                let pane = ictx.pane_ref();
                if let Some(pane) = pane.as_ref()
                    && pane
                        .get()
                        .is_some_and(|wp| wp.options_ref().number(c"allow-set-title") != 0)
                    && sctx.screen_mut().set_title(payload, 1) != 0
                {
                    notify_pane(c"pane-title-changed", pane.get());
                    if let Some(window) = ictx.window_ref() {
                        window.redraw_borders();
                        window.redraw_status();
                    }
                }
            }
            4 => input_osc_4(ictx, sctx, payload),
            7 => {
                let pane = ictx.pane_ref();
                if pane.is_some()
                    && sctx.screen_mut().set_path(payload, 1) != 0
                    && let Some(window) = ictx.window_ref()
                {
                    window.redraw_borders();
                    window.redraw_status();
                }
            }
            8 => input_osc_8(ictx, sctx, payload),
            9 => input_osc_9(ictx, sctx, payload),
            10 => input_osc_10(ictx, sctx, payload),
            11 => input_osc_11(ictx, sctx, payload),
            12 => input_osc_12(ictx, sctx, payload),
            52 => input_osc_52(ictx, payload),
            104 => input_osc_104(ictx, sctx, payload),
            110 => input_osc_110(ictx, sctx, payload),
            111 => input_osc_111(ictx, sctx, payload),
            112 => input_osc_112(ictx, sctx, payload),
            133 => input_osc_133(ictx, sctx, payload),
            _ => log_debug(c"%s: unknown '%u'", fmt_args![c"input_exit_osc", option]),
        }
        ictx.input_buf = buffer;
    }
}

fn input_enter_apc(ictx: &mut input_ctx) {
    {
        log_debug(c"%s", fmt_args![c"input_enter_apc".as_ptr()]);
        input_clear(&mut *ictx);
        input_start_ground_timer(&mut *ictx);
        ictx.flags &= !INPUT_LAST;
    }
}
unsafe fn input_exit_apc(ictx: &mut input_ctx, sctx: &mut RustScreenWriteCtx<'_>) {
    unsafe {
        if ictx.flags & INPUT_DISCARD != 0 {
            return;
        }
        let title =
            CStr::from_bytes_until_nul(&ictx.input_buf).expect("APC input has a trailing NUL");
        log_debug(c"%s: \"%s\"", fmt_args![c"input_exit_apc", title]);
        let pane = ictx.pane_ref();
        if let Some(pane) = pane.as_ref()
            && pane
                .get()
                .is_some_and(|wp| wp.options_ref().number(c"allow-set-title") != 0)
            && sctx.screen_mut().set_title(title, 1) != 0
        {
            notify_pane(c"pane-title-changed", pane.get());
            if let Some(window) = ictx.window_ref() {
                window.redraw_borders();
                window.redraw_status();
            }
        }
    }
}

fn input_enter_rename(ictx: &mut input_ctx) {
    {
        log_debug(c"%s", fmt_args![c"input_enter_rename".as_ptr()]);
        input_clear(&mut *ictx);
        input_start_ground_timer(&mut *ictx);
        ictx.flags &= !INPUT_LAST;
    }
}
unsafe fn input_exit_rename(ictx: &mut input_ctx) {
    unsafe {
        let Some(pane) = ictx.pane_ref() else {
            return;
        };
        if ictx.flags & INPUT_DISCARD != 0 {
            return;
        }
        if pane
            .get()
            .is_none_or(|wp| wp.options_ref().number(c"allow-rename") == 0)
        {
            return;
        }
        let name =
            CStr::from_bytes_until_nul(&ictx.input_buf).expect("rename input has a trailing NUL");
        log_debug(c"%s: \"%s\"", fmt_args![c"input_exit_rename", name]);
        if !RustUtf8VisModel.is_valid(name) {
            return;
        }
        let Some(window) = ictx.window_ref() else {
            return;
        };
        if ictx.input_buf.len() == 1 {
            RustOptionsEngine.remove_or_default(
                &window.options(),
                c"automatic-rename",
                -1,
                &mut None,
            );
            if (window.options()).number(c"automatic-rename") == 0 {
                window.set_name(c"", 1 as core::ffi::c_int);
            }
        } else {
            (window.options()).set_number(c"automatic-rename", 0 as core::ffi::c_longlong);
            window.set_name(name, 1 as core::ffi::c_int);
        }
        window.redraw_borders();
        window.redraw_status();
    }
}
unsafe fn input_top_bit_set(
    ictx: &mut input_ctx,
    sctx: &mut RustScreenWriteCtx<'_>,
) -> core::ffi::c_int {
    unsafe {
        let ud: &mut utf8_data = &mut ictx.utf8data;
        ictx.flags &= !INPUT_LAST;
        if ictx.utf8started == 0 {
            ictx.utf8started = 1 as core::ffi::c_int;
            if utf8_open(&mut *ud, ictx.ch as u_char) as core::ffi::c_uint
                != UTF8_MORE as core::ffi::c_int as core::ffi::c_uint
            {
                input_stop_utf8(ictx, sctx);
            }
            return 0 as core::ffi::c_int;
        }
        match utf8_append(&mut *ud, ictx.ch as u_char) {
            UTF8_MORE => return 0 as core::ffi::c_int,
            UTF8_ERROR => {
                input_stop_utf8(ictx, sctx);
                return 0 as core::ffi::c_int;
            }
            _ => {}
        }
        ictx.utf8started = 0 as core::ffi::c_int;
        log_debug(
            c"%s %hhu '%*s' (width %hhu)",
            fmt_args![
                c"input_top_bit_set",
                ud.size as core::ffi::c_int,
                ud.size as core::ffi::c_int,
                ud.data.as_slice(),
                ud.width as core::ffi::c_int
            ],
        );
        utf8_copy(&mut ictx.cell.cell.data, &*ud);
        sctx.collect_add(&ictx.cell.cell);
        utf8_copy(&mut ictx.last, &ictx.cell.cell.data);
        ictx.flags |= INPUT_LAST;
        0 as core::ffi::c_int
    }
}
fn input_osc_colour_reply(
    ictx: &mut input_ctx,
    add: core::ffi::c_int,
    n: u_int,
    idx: core::ffi::c_int,
    mut c: core::ffi::c_int,
    end_type: input_end_type,
) {
    {
        if c != -(1 as core::ffi::c_int) {
            c = RustColourEngine.force_rgb(c);
        }
        if c == -(1 as core::ffi::c_int) {
            return;
        }
        let (r, g, b) = RustColourEngine.split_rgb(c);
        let end = if end_type == INPUT_END_BEL {
            c"\x07"
        } else {
            c"\x1B\\"
        };
        if n == 4 as u_int {
            input_reply(
                ictx,
                add,
                c"\x1B]%u;%d;rgb:%02hhx%02hhx/%02hhx%02hhx/%02hhx%02hhx%s",
                fmt_args![
                    n,
                    idx,
                    r as core::ffi::c_int,
                    r as core::ffi::c_int,
                    g as core::ffi::c_int,
                    g as core::ffi::c_int,
                    b as core::ffi::c_int,
                    b as core::ffi::c_int,
                    end
                ],
            );
        } else {
            input_reply(
                ictx,
                add,
                c"\x1B]%u;rgb:%02hhx%02hhx/%02hhx%02hhx/%02hhx%02hhx%s",
                fmt_args![
                    n,
                    r as core::ffi::c_int,
                    r as core::ffi::c_int,
                    g as core::ffi::c_int,
                    g as core::ffi::c_int,
                    b as core::ffi::c_int,
                    b as core::ffi::c_int,
                    end
                ],
            );
        };
    }
}
unsafe fn input_osc_4(ictx: &mut input_ctx, sctx: &mut RustScreenWriteCtx<'_>, p: &CStr) {
    unsafe {
        let mut bad = false;
        let mut redraw = false;
        let mut copy = p.to_bytes_with_nul().to_vec();
        let mut fields = input_fields(&mut copy, b';');
        while let Some(index) = fields.next() {
            let Some(value) = fields.next() else {
                bad = !index.is_empty();
                break;
            };
            let idx = if index.is_empty() {
                0
            } else {
                let Ok(idx) = strtonum(index, 0, 255) else {
                    bad = true;
                    break;
                };
                idx as core::ffi::c_int
            };
            if value == c"?" {
                let colour = ictx.palette_colour(idx | COLOUR_FLAG_256);
                if colour != -1 {
                    input_osc_colour_reply(ictx, 1, 4, idx, colour, ictx.input_end);
                } else {
                    input_add_request(ictx, INPUT_REQUEST_PALETTE, idx);
                }
            } else {
                let colour = RustColourEngine.parse_x11(value);
                if colour != -1
                    && ictx.set_palette_colour(idx, colour) != 0
                {
                    redraw = true;
                }
            }
        }
        if bad {
            log_debug(c"bad OSC 4: %s", fmt_args![p]);
        }
        if redraw {
            sctx.fullredraw();
        }
    }
}

fn input_osc_8(ictx: &mut input_ctx, sctx: &mut RustScreenWriteCtx<'_>, p: &CStr) {
    {
        let bytes = p.to_bytes_with_nul();
        let Some(separator) = bytes.iter().position(|byte| *byte == b';') else {
            log_debug(c"bad OSC 8 %s", fmt_args![p]);
            return;
        };
        let mut id = None;
        for field in bytes[..separator].split(|byte| *byte == b':') {
            let Some(value) = field.strip_prefix(b"id=").filter(|value| !value.is_empty()) else {
                continue;
            };
            if id.is_some() {
                log_debug(c"bad OSC 8 %s", fmt_args![p]);
                return;
            }
            id = Some(CString::new(value).expect("hyperlink id has no NUL"));
        }
        let uri = CStr::from_bytes_with_nul(&bytes[separator + 1..])
            .expect("the hyperlink URI retains the input's trailing NUL");
        if uri.is_empty() {
            ictx.cell.cell.link = 0;
            return;
        }
        let link = sctx.screen_mut().hyperlinks().put(uri, id.as_deref());
        ictx.cell.cell.link = link;
        if let Some(id) = id.as_deref() {
            log_debug(c"hyperlink (id=%s) %s = %u", fmt_args![id, uri, link]);
        } else {
            log_debug(c"hyperlink (anonymous) %s = %u", fmt_args![uri, link]);
        }
    }
}

unsafe fn input_set_progress_bar(
    ictx: &mut input_ctx,
    sctx: &mut RustScreenWriteCtx<'_>,
    state: progress_bar_state,
    p: core::ffi::c_int,
) {
    unsafe {
        sctx.screen_mut().set_progress_bar(state, p);
        if let Some(window) = ictx.window_ref() {
            window.redraw_borders();
            window.redraw_status();
        }
    }
}
unsafe fn input_osc_9(ictx: &mut input_ctx, sctx: &mut RustScreenWriteCtx<'_>, p: &CStr) {
    unsafe {
        let Some((&state, rest)) = p
            .to_bytes()
            .strip_prefix(b"4;")
            .and_then(|s| s.split_first())
        else {
            return;
        };
        if !(b'0'..=b'4').contains(&state) {
            log_debug(c"bad OSC 9;4 %s", fmt_args![p]);
            return;
        }
        let progress = match rest {
            [] | [b';'] => Some(-1),
            [b';', digits @ ..] => digits.iter().try_fold(0, |value, digit| {
                if !digit.is_ascii_digit() {
                    return None;
                }
                let next = value * 10 + (digit - b'0') as core::ffi::c_int;
                (next <= 100).then_some(next)
            }),
            _ => None,
        };
        if let Some(progress) = progress {
            input_set_progress_bar(ictx, sctx, (state - b'0') as progress_bar_state, progress);
        } else {
            log_debug(c"bad OSC 9;4 %s", fmt_args![p]);
        }
    }
}

unsafe fn input_osc_10(ictx: &mut input_ctx, sctx: &mut RustScreenWriteCtx<'_>, p: &CStr) {
    unsafe {
        let mut pane = ictx.pane_ref();
        let mut defaults = grid_default_cell;
        let mut c: core::ffi::c_int;
        if p == c"?" {
            let Some(wp) = pane.as_mut().and_then(|pane| pane.get_mut()) else {
                return;
            };
            c = window_pane_get_fg_control_client(wp);
            if c == -(1 as core::ffi::c_int) {
                tty_default_colours(&mut defaults, wp);
                if defaults.fg == 8 as core::ffi::c_int || defaults.fg == 9 as core::ffi::c_int {
                    c = window_pane_get_fg(wp);
                } else {
                    c = defaults.fg;
                }
            }
            input_osc_colour_reply(
                ictx,
                1 as core::ffi::c_int,
                10 as u_int,
                0 as core::ffi::c_int,
                c,
                ictx.input_end,
            );
            return;
        }
        c = RustColourEngine.parse_x11(p);
        if c == -(1 as core::ffi::c_int) {
            log_debug(c"bad OSC 10: %s", fmt_args![p]);
            return;
        }
        if ictx.set_palette_default(true, c) {
            sctx.fullredraw();
        }
    }
}
unsafe fn input_osc_110(ictx: &mut input_ctx, sctx: &mut RustScreenWriteCtx<'_>, p: &CStr) {
    unsafe {
        let mut pane = ictx.pane_ref();
        if !p.is_empty() {
            return;
        }
        if ictx.set_palette_default(true, 8 as core::ffi::c_int) {
            sctx.fullredraw();
        }
    }
}
unsafe fn input_osc_11(ictx: &mut input_ctx, sctx: &mut RustScreenWriteCtx<'_>, p: &CStr) {
    unsafe {
        let mut pane = ictx.pane_ref();
        let c: core::ffi::c_int;
        if p == c"?" {
            let Some(wp) = pane.as_mut().and_then(|pane| pane.get_mut()) else {
                return;
            };
            c = window_pane_get_bg(wp);
            input_osc_colour_reply(
                ictx,
                1 as core::ffi::c_int,
                11 as u_int,
                0 as core::ffi::c_int,
                c,
                ictx.input_end,
            );
            return;
        }
        c = RustColourEngine.parse_x11(p);
        if c == -(1 as core::ffi::c_int) {
            log_debug(c"bad OSC 11: %s", fmt_args![p]);
            return;
        }
        if ictx.set_palette_default(false, c) {
            sctx.fullredraw();
        }
    }
}
unsafe fn input_osc_111(ictx: &mut input_ctx, sctx: &mut RustScreenWriteCtx<'_>, p: &CStr) {
    unsafe {
        let mut pane = ictx.pane_ref();
        if !p.is_empty() {
            return;
        }
        if ictx.set_palette_default(false, 8 as core::ffi::c_int) {
            sctx.fullredraw();
        }
    }
}
fn input_osc_12(ictx: &mut input_ctx, sctx: &mut RustScreenWriteCtx<'_>, p: &CStr) {
    {
        let pane = ictx.pane_ref();
        let c: core::ffi::c_int;
        if p == c"?" {
            if pane.is_some() {
                c = sctx.screen_mut().cursor_colour();
                input_osc_colour_reply(
                    ictx,
                    1 as core::ffi::c_int,
                    12 as u_int,
                    0 as core::ffi::c_int,
                    c,
                    ictx.input_end,
                );
            }
            return;
        }
        c = RustColourEngine.parse_x11(p);
        if c == -(1 as core::ffi::c_int) {
            log_debug(c"bad OSC 12: %s", fmt_args![p]);
            return;
        }
        sctx.screen_mut().set_cursor_colour(c);
    }
}
fn input_osc_112(_ictx: &mut input_ctx, sctx: &mut RustScreenWriteCtx<'_>, p: &CStr) {
    {
        if p.is_empty() {
            sctx.screen_mut()
                .set_cursor_colour(-(1 as core::ffi::c_int));
        }
    }
}
fn input_osc_133(_ictx: &mut input_ctx, sctx: &mut RustScreenWriteCtx<'_>, p: &CStr) {
    {
        let screen = sctx.screen_mut();
        let cy = screen.cursor().1;
        match p.to_bytes().first().copied() {
            Some(b'A') => RustScreen::grid_mut(screen).mark_prompt(cy, false),
            Some(b'C') => RustScreen::grid_mut(screen).mark_prompt(cy, true),
            _ => {},
        }
    }
}
unsafe fn input_osc_52_reply(ictx: &mut input_ctx, clip: core::ffi::c_char) {
    unsafe {
        let ev: Stream = ictx.event;

        let state: core::ffi::c_int = (global_options
            .get()
            .as_ref()
            .expect("global options are initialized"))
        .number(c"get-clipboard") as core::ffi::c_int;
        if state == 0 as core::ffi::c_int {
            return;
        }
        if state == 1 as core::ffi::c_int {
            let Some(bytes) =
                with_paste_buffers(|buffers| buffers.top().map(|buffer| buffer.data.to_vec()))
            else {
                return;
            };
            if ictx.input_end as core::ffi::c_uint
                == INPUT_END_BEL as core::ffi::c_int as core::ffi::c_uint
            {
                input_reply_clipboard(ev, &bytes, c"\x07", clip);
            } else {
                input_reply_clipboard(ev, &bytes, c"\x1B\\", clip);
            }
            return;
        }
        input_add_request(
            ictx,
            INPUT_REQUEST_CLIPBOARD,
            ictx.input_end as core::ffi::c_int,
        );
    }
}
unsafe fn input_osc_52_parse(ictx: &mut input_ctx, p: &CStr) -> Option<(CString, Vec<u8>)> {
    unsafe {
        if (global_options
            .get()
            .as_ref()
            .expect("global options are initialized"))
        .number(c"set-clipboard")
            != 2
        {
            return None;
        }
        let bytes = p.to_bytes_with_nul();
        let separator = bytes.iter().position(|byte| *byte == b';')?;
        let payload = CStr::from_bytes_with_nul(&bytes[separator + 1..])
            .expect("the clipboard payload retains the input's trailing NUL");
        if payload.is_empty() {
            return None;
        }
        log_debug(c"%s: %s", fmt_args![c"input_osc_52_parse", payload]);
        let mut clip = Vec::new();
        for &selector in &bytes[..separator] {
            if b"cpqs01234567".contains(&selector) && !clip.contains(&selector) {
                clip.push(selector);
            }
        }
        let clip = CString::new(clip).expect("clipboard selectors have no NUL");
        log_debug(
            c"%s: %.*s %s",
            fmt_args![
                c"input_osc_52_parse",
                separator as core::ffi::c_int,
                p,
                clip.as_c_str()
            ],
        );
        if payload == c"?" {
            input_osc_52_reply(ictx, clip.as_bytes_with_nul()[0] as core::ffi::c_char);
            return None;
        }
        let len = payload
            .count_bytes()
            .wrapping_add(3)
            .wrapping_div(4)
            .wrapping_mul(3);
        if len == 0 {
            return None;
        }
        let mut out = vec![0; len];
        let outlen = __b64_pton(payload.as_ptr(), out.as_mut_ptr(), len);
        if outlen == -1 {
            return None;
        }
        out.truncate(outlen as usize);
        Some((clip, out))
    }
}

unsafe fn input_osc_52(ictx: &mut input_ctx, p: &CStr) {
    unsafe {
        let mut pane = ictx.pane_ref();
        let Some((clip, out)) = input_osc_52_parse(ictx, p) else {
            return;
        };
        if pane.is_none() {
            let Some(mut client) = (*ictx).client() else {
                return;
            };
            tty_set_selection(client.as_tty_mut(), &clip, &out);
            let limit = paste_buffer_limit();
            with_paste_buffers_mut(|buffers| buffers.add_automatic(None, out, limit));
        } else {
            let Some(wp) = pane.as_mut().and_then(|pane| pane.get_mut()) else {
                return;
            };
            let mut ctx = screen_write_ctx_on_pane(wp);
            ctx.setselection(&clip, &out);
            ctx.finish();
            notify_pane(c"pane-set-clipboard", Some(wp));
            let limit = paste_buffer_limit();
            with_paste_buffers_mut(|buffers| buffers.add_automatic(None, out, limit));
        };
    }
}
unsafe fn input_osc_104(ictx: &mut input_ctx, sctx: &mut RustScreenWriteCtx<'_>, p: &CStr) {
    unsafe {
        if p.is_empty() {
            ictx.clear_palette();
            sctx.fullredraw();
            return;
        }
        let mut bad = false;
        let mut redraw = false;
        let mut copy = p.to_bytes_with_nul().to_vec();
        let mut fields = input_fields(&mut copy, b';').peekable();
        while let Some(field) = fields.next() {
            let idx = if field.is_empty() {
                if fields.peek().is_none() {
                    break;
                }
                0
            } else {
                let Ok(idx) = strtonum(field, 0, 255) else {
                    bad = true;
                    break;
                };
                idx as core::ffi::c_int
            };
            if ictx.set_palette_colour(idx, -1) != 0
            {
                redraw = true;
            }
        }
        if bad {
            log_debug(c"bad OSC 104: %s", fmt_args![p]);
        }
        if redraw {
            sctx.fullredraw();
        }
    }
}

pub unsafe fn input_reply_clipboard(bev: Stream, buf: &[u8], end: &CStr, clip: core::ffi::c_char) {
    unsafe {
        let len = buf.len() as size_t;
        let mut out: Vec<u8> = Vec::new();
        let mut outlen: core::ffi::c_int = 0 as core::ffi::c_int;
        if !buf.is_empty() {
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
                .wrapping_add(1 as size_t) as core::ffi::c_int;
            out = vec![0_u8; outlen as usize];
            outlen = __b64_ntop(
                buf.as_ptr(),
                len,
                out.as_mut_ptr() as *mut core::ffi::c_char,
                outlen as size_t,
            );
            if outlen == -(1 as core::ffi::c_int) {
                return;
            }
        }
        bev.write(b"\x1B]52;");
        if clip as core::ffi::c_int != 0 as core::ffi::c_int {
            bev.write(&[clip as u8]);
        }
        bev.write(b";");
        if outlen != 0 as core::ffi::c_int {
            bev.write(&out[..outlen as usize]);
        }
        bev.write(end.to_bytes());
    }
}
pub fn input_set_buffer_size(buffer_size: size_t) {
    {
        log_debug(
            c"%s: %lu -> %lu",
            fmt_args![
                c"input_set_buffer_size".as_ptr(),
                input_buffer_size.get(),
                buffer_size
            ],
        );
        input_buffer_size.set(buffer_size);
    }
}
unsafe fn input_request_timer_callback(ictx: &mut input_ctx) {
    unsafe {
        let t: uint64_t = get_timer();
        let expired = ictx
            .requests
            .iter()
            .filter(|request| request.t.wrapping_add(INPUT_REQUEST_TIMEOUT as uint64_t) < t)
            .map(|request| input_request_handle {
                ictx: request.ictx.clone(),
                id: request.id,
            })
            .collect::<Vec<_>>();
        for handle in expired {
            let queue = input_request_in(ictx, handle.id, |request| {
                request.type_0 as core::ffi::c_uint
                    == INPUT_REQUEST_QUEUE as core::ffi::c_int as core::ffi::c_uint
            });
            if queue == Some(true) {
                let data =
                    input_request_in(ictx, handle.id, |request| request.data.clone()).flatten();
                input_send_reply(ictx, data.as_deref().unwrap_or(c""));
            }
            input_free_request_in(ictx, handle);
        }
        if ictx.request_count != 0 as u_int {
            input_start_request_timer(&mut *ictx);
        }
    }
}
fn input_start_request_timer(ictx: &mut input_ctx) {
    {
        let tv = timeval::from_usecs(100000 as __suseconds_t);
        ictx.request_timer.disarm();
        ictx.request_timer.arm(tv);
    }
}
/// The parser a request belongs to, observed rather than held: a request
/// outlives its parser only when the parser's owner has given it up first.
fn ictx_weak(ictx: &input_ctx) -> InputCtxWeak {
    ictx.owner.clone().expect("a parser holds itself")
}

fn input_make_request(ictx: &mut input_ctx, type_0: input_request_type) -> input_request_handle {
    {
        let id = ictx.next_request_id;
        ictx.next_request_id = ictx.next_request_id.wrapping_add(1);
        let request = Box::new(input_request {
            c: None,
            ictx: ictx_weak(&*ictx),
            id,
            type_0: type_0,
            t: get_timer(),
            end: INPUT_END_ST,
            idx: 0,
            data: None,
        });
        let handle = input_request_handle {
            ictx: request.ictx.clone(),
            id,
        };
        ictx.request_count = ictx.request_count.wrapping_add(1);
        if ictx.request_count == 1 as u_int {
            input_start_request_timer(&mut *ictx);
        }
        ictx.requests.push(request);
        handle
    }
}
/// Takes `ir` off `list`, which is one of the two it sits on.
fn input_unlink_request(list: &mut input_requests, handle: &input_request_handle) {
    if let Some(at) = list.iter().position(|waiting| waiting.matches(handle)) {
        list.remove(at);
    }
}

fn input_request<R>(
    handle: &input_request_handle,
    f: impl FnOnce(&input_request) -> R,
) -> Option<R> {
    let ictx = handle.ictx.upgrade()?;
    let ictx = ictx.borrow();
    let request = ictx
        .requests
        .iter()
        .find(|request| request.id == handle.id)?;
    Some(f(request))
}

fn input_request_in<R>(
    ictx: &input_ctx,
    id: u64,
    f: impl FnOnce(&input_request) -> R,
) -> Option<R> {
    let request = ictx.requests.iter().find(|request| request.id == id)?;
    Some(f(request))
}

fn input_request_mut_in(ictx: &mut input_ctx, id: u64, f: impl FnOnce(&mut input_request)) {
    if let Some(request) = ictx.requests.iter_mut().find(|request| request.id == id) {
        f(request);
    }
}

unsafe fn input_free_request_in(ictx: &mut input_ctx, handle: input_request_handle) {
    unsafe {
        let Some(at) = ictx
            .requests
            .iter()
            .position(|request| request.id == handle.id)
        else {
            return;
        };
        let client = ictx.requests[at].c.clone();
        if let Some(mut client) = client.and_then(|client| client.upgrade()) {
            input_unlink_request(&mut client.as_client_mut().input_requests, &handle);
        }
        ictx.request_count = ictx.request_count.wrapping_sub(1);
        ictx.requests.remove(at);
    }
}

unsafe fn input_free_request_for_client(
    identity: *const client,
    requests: &mut input_requests,
    handle: input_request_handle,
) {
    unsafe {
        let Some(ictx) = handle.ictx.upgrade() else {
            return;
        };
        input_free_request_in_client(&mut ictx.borrow_mut(), handle, identity, requests);
    }
}

unsafe fn input_free_request_in_client(
    ictx: &mut input_ctx,
    handle: input_request_handle,
    identity: *const client,
    requests: &mut input_requests,
) {
    unsafe {
        let Some(at) = ictx
            .requests
            .iter()
            .position(|request| request.id == handle.id)
        else {
            return;
        };
        let client = ictx.requests[at].c.clone();
        if let Some(mut client_ref) = client.and_then(|client| client.upgrade()) {
            if core::ptr::eq(client_ref.as_ptr(), identity) {
                input_unlink_request(requests, &handle);
            } else {
                input_unlink_request(client_ref.input_requests_mut(), &handle);
            }
        }
        ictx.request_count = ictx.request_count.wrapping_sub(1);
        ictx.requests.remove(at);
    }
}
unsafe fn input_add_request(
    ictx: &mut input_ctx,
    type_0: input_request_type,
    idx: core::ffi::c_int,
) -> core::ffi::c_int {
    unsafe {
        let Some(window) = ictx.window_ref() else {
            return -1;
        };
        let mut c: Option<ClientRef> = None;
        for loop_0 in client_walk() {
            let attached = loop_0.attached_session();
            let view = loop_0.as_client();
            if !(view.flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0)
                && attached.is_some_and(|session| session.has(&window))
                && !(!view.tty.flags & TTY_STARTED != 0)
            {
                let newer = c.as_ref().is_none_or(|c| {
                    let previous = c.as_client().activity_time;
                    if view.activity_time.tv_sec == previous.tv_sec {
                        view.activity_time.tv_usec > previous.tv_usec
                    } else {
                        view.activity_time.tv_sec > previous.tv_sec
                    }
                });
                if newer {
                    c = Some(loop_0);
                }
            }
        }
        let Some(mut c) = c else {
            return -(1 as core::ffi::c_int);
        };
        let ir = input_make_request(ictx, type_0);
        let end = ictx.input_end;
        input_request_mut_in(ictx, ir.id, |request| {
            request.c = Some(c.downgrade());
            request.idx = idx;
            request.end = end;
        });
        c.as_client_mut().input_requests.push(ir);
        match type_0 {
            INPUT_REQUEST_PALETTE => {
                let s = xasprintf(c"\x1B]4;%d;?\x1B\\", fmt_args![idx]);
                tty_puts(c.as_tty_mut(), &s);
            }
            INPUT_REQUEST_CLIPBOARD => {
                tty_putcode_ss(c.as_tty_mut(), TTYC_MS, c"", c"?");
            }
            _ => {}
        }
        0 as core::ffi::c_int
    }
}
fn input_request_palette_reply(ictx: &mut input_ctx, end: input_end_type, data: &InputRequestData) {
    {
        let &InputRequestData::Palette { idx, c } = data else {
            return;
        };
        input_osc_colour_reply(ictx, 0 as core::ffi::c_int, 4 as u_int, idx, c, end);
    }
}
unsafe fn input_request_clipboard_reply(
    ictx: &mut input_ctx,
    idx: core::ffi::c_int,
    data: &InputRequestData,
) {
    unsafe {
        let ev: Stream = ictx.event;
        let InputRequestData::Clipboard { clip, buf: data } = data else {
            return;
        };

        let state: core::ffi::c_int = (global_options
            .get()
            .as_ref()
            .expect("global options are initialized"))
        .number(c"get-clipboard") as core::ffi::c_int;
        if state == 0 as core::ffi::c_int || state == 1 as core::ffi::c_int {
            return;
        }
        if state == 3 as core::ffi::c_int {
            let limit = paste_buffer_limit();
            with_paste_buffers_mut(|buffers| buffers.add_automatic(None, data.clone(), limit));
        }
        if idx == INPUT_END_BEL as core::ffi::c_int {
            input_reply_clipboard(ev, data, c"\x07", *clip);
        } else {
            input_reply_clipboard(ev, data, c"\x1B\\", *clip);
        };
    }
}

pub unsafe fn input_cancel_requests(c: &mut client) {
    unsafe {
        let identity = core::ptr::from_ref(c);
        let requests = c.input_requests.clone();
        for ir in requests {
            input_free_request_for_client(identity, &mut c.input_requests, ir);
        }
    }
}
unsafe fn input_report_current_theme(ictx: &mut input_ctx) {
    unsafe {
        let mut pane = ictx.pane_ref();
        if let Some(wp) = pane.as_mut().and_then(|pane| pane.get_mut()) {
            let theme = wp.acknowledge_theme();
            match theme {
                THEME_DARK => {
                    log_debug(
                        c"%s: %%%u dark theme",
                        fmt_args![c"input_report_current_theme".as_ptr(), wp.pane_id()],
                    );
                    input_reply(ictx, 0 as core::ffi::c_int, c"\x1B[?997;1n", fmt_args![]);
                }
                THEME_LIGHT => {
                    log_debug(
                        c"%s: %%%u light theme",
                        fmt_args![c"input_report_current_theme".as_ptr(), wp.pane_id()],
                    );
                    input_reply(ictx, 0 as core::ffi::c_int, c"\x1B[?997;2n", fmt_args![]);
                }
                THEME_UNKNOWN => {
                    log_debug(
                        c"%s: %%%u unknown theme",
                        fmt_args![c"input_report_current_theme".as_ptr(), wp.pane_id()],
                    );
                }
                _ => {}
            }
        }
    }
}

use crate::screen::RustScreen;

#[cfg(test)]
mod focused_tests {
    use super::*;
    use crate::style::{ColourEngine, RustColourEngine};
    use crate::tests::test_fixtures::{Pane, Window, ensure_reactor, globals};

    struct ParserCtx {
        _window: Window,
        pane: Pane,
        ictx: InputCtxRef,
        _guard: crate::tests::test_fixtures::GlobalsGuard,
    }

    impl ParserCtx {
        fn new() -> Self {
            let guard = globals();
            ensure_reactor();
            let mut window = Window::new(381, "parser", 80, 24);
            let mut pane = Pane::new(382, 80, 24, 100);
            window.add_pane(&mut pane);
            let wp = pane.ptr();
            let ictx = unsafe {
                (*wp).configure_test(crate::window_pane::PaneTestSetup::Palette(RustColourEngine.new_palette()));
                let context = InputCtxRef::create(
                    InputOwner::Pane((*wp).pane_id()),
                    Stream::NONE,
                );
            (*wp).configure_test(crate::window_pane::PaneTestSetup::Parser(Some(context.clone())));
            context
            };
            Self {
                _window: window,
                pane,
                ictx,
                _guard: guard,
            }
        }

        unsafe fn parse(&mut self, bytes: &'static [u8]) {
            unsafe {
                (&mut *self.pane.ptr()).parse_bytes(
                    ByteBuffer::from(bytes::Bytes::from_static(bytes)),
                );
            }
        }
    }

    impl Drop for ParserCtx {
        fn drop(&mut self) {
            unsafe {
                let wp = self.pane.ptr();
                (*wp).configure_test(crate::window_pane::PaneTestSetup::Parser(None));
                (*wp).configure_test(crate::window_pane::PaneTestSetup::Palette(Default::default()));
            }
        }
    }

    #[test]
    fn parameter_helpers_cover_missing_numbers_strings_limits_and_reset() {
        let ctx = ParserCtx::new();
        unsafe {
            let ictx = &mut ctx.ictx.borrow_mut();
            ictx.param_buf[..14].copy_from_slice(b";0;12;3:4;999\0");
            ictx.param_len = 13;
            assert_eq!(input_split(ictx), 0);
            assert_eq!(ictx.param_list_len, 5);
            assert_eq!(input_get(ictx, 0, 1, 7), 7);
            assert_eq!(input_get(ictx, 1, 1, 7), 1);
            assert_eq!(input_get(ictx, 2, 1, 7), 12);
            assert_eq!(input_get(ictx, 3, 1, 7), -1);
            assert_eq!(input_get(ictx, 20, 1, 7), 7);
            input_clear(ictx);
            for _ in 0..3 {
                ictx.ch = b'?' as i32;
                assert_eq!(input_intermediate(ictx), 0);
            }
            ictx.ch = b'!'.into();
            input_intermediate(ictx);
            assert_ne!(ictx.flags & INPUT_DISCARD, 0);
            input_clear(ictx);
            for _ in 0..63 {
                ictx.ch = b'1'.into();
                input_parameter(ictx);
            }
            ictx.ch = b'2'.into();
            input_parameter(ictx);
            assert_ne!(ictx.flags & INPUT_DISCARD, 0);
            input_clear(ictx);
            ictx.param_buf[..12].copy_from_slice(b"99999999999\0");
            ictx.param_len = 11;
            assert_eq!(input_split(ictx), -1);
            input_ground(ictx);
        }
    }

    #[test]
    fn parameter_fields_preserve_empty_values_and_reject_the_capacity_boundary() {
        let ctx = ParserCtx::new();
        let mut ictx = ctx.ictx.borrow_mut();
        for (input, result, count) in [
            (";12;;", 0, 4),
            ("2147483647", 0, 1),
            ("2147483648", -1, 0),
            ("12;-1", -1, 1),
            ("12;bad", -1, 1),
            (";;;;;;;;;;;;;;;;;;;;;;", 0, 23),
            (";;;;;;;;;;;;;;;;;;;;;;;", -1, 24),
            ("", 0, 0),
        ] {
            ictx.param_buf.fill(0);
            ictx.param_buf[..input.len()].copy_from_slice(input.as_bytes());
            ictx.param_len = input.len();
            unsafe { assert_eq!(input_split(&mut ictx), result, "{input:?}") };
            assert_eq!(ictx.param_list_len, count, "{input:?}");
            if input == ";12;;" {
                assert!(matches!(ictx.param_list[0], InputParam::Missing));
                assert!(matches!(ictx.param_list[1], InputParam::Number(12)));
                assert!(matches!(ictx.param_list[2], InputParam::Missing));
                assert!(matches!(ictx.param_list[3], InputParam::Missing));
            }
        }
        assert!(
            ictx.param_list
                .iter()
                .all(|param| matches!(param, InputParam::Missing))
        );
    }

    #[test]
    fn parser_state_machine_handles_fragmented_and_cancelled_sequences() {
        let mut ctx = ParserCtx::new();
        unsafe {
            ctx.parse(b"plain\x1b[");
            assert_eq!(ctx.ictx.borrow().state.name, c"csi_enter");
            assert_eq!(ctx.ictx.pending().unwrap().as_slice(), b"\x1b[");
            ctx.parse(b"2;3Hdone");
            assert_eq!(ctx.ictx.borrow().state.name, c"ground");
            assert!(ctx.ictx.pending().unwrap().is_empty());
            ctx.parse(b"\x1b[12\x18x\x1b[?12\x1ay\x1bP1;2$qignored\x1b\\z");
            assert_eq!(ctx.ictx.borrow().state.name, c"ground");
        }
    }

    #[test]
    fn c0_and_escape_dispatch_cover_cursor_charset_and_screen_operations() {
        let mut ctx = ParserCtx::new();
        unsafe {
            ctx.parse(b"abc\x08\t\r\n\x0e#\x0f\x1b7\x1b8\x1b=\x1b>\x1bH\x1bD\x1bE\x1bM");
            ctx.parse(b"\x1b(0q\x1b(Bx\x1b)0\x0eq\x0f\x1b)B\x1b#8");
            let s = ctx.pane.base();
            let (cx, cy) = (*s).cursor();
            assert!(cx < RustScreen::grid(&*s).width());
            assert!(cy < RustScreen::grid(&*s).height());
            assert_eq!(ctx.ictx.borrow().state.name, c"ground");
        }
    }

    #[test]
    fn csi_cursor_edit_and_scroll_dispatches_cover_defaults_and_bounds() {
        let mut ctx = ParserCtx::new();
        unsafe {
            ctx.parse(b"first line\r\nsecond line\r\nthird line");
            ctx.parse(b"\x1b[2A\x1b[1B\x1b[3C\x1b[2D\x1b[2E\x1b[F");
            ctx.parse(b"\x1b[4G\x1b[3d\x1b[2;6H\x1b[1;1f\x1b[Z");
            ctx.parse(b"\x1b[2@\x1b[3P\x1b[4X\x1b[L\x1b[2M\x1b[S\x1b[T");
            ctx.parse(b"\x1b[J\x1b[1J\x1b[2J\x1b[3J\x1b[K\x1b[1K\x1b[2K");
            ctx.parse(b"\x1b[2;20r\x1b[s\x1b[10;10H\x1b[u\x1b[3b\x1b[g\x1b[3g");
            assert_eq!(ctx.ictx.borrow().state.name, c"ground");
        }
    }

    #[test]
    fn cursor_backward_tab_skips_unset_columns_and_honours_the_count() {
        for (stream, expected_x) in [
            (&b"\x1b[19G\x1b[Z"[..], 16),
            (&b"\x1b[35G\x1b[2Z"[..], 24),
            (&b"\x1b[3g\x1b[13G\x1bH\x1b[21G\x1b[Z"[..], 12),
            (&b"\x1b[3g\x1b[21G\x1b[Z"[..], 0),
        ] {
            let mut ctx = ParserCtx::new();
            unsafe {
                ctx.parse(stream);
                assert_eq!(ctx.pane.base().cursor(), (expected_x, 0));
            }
        }
    }

    #[test]
    fn csi_modes_and_sgr_cover_standard_indexed_rgb_and_colon_forms() {
        let mut ctx = ParserCtx::new();
        unsafe {
            ctx.parse(b"\x1b[1;2;3;4;5;7;8;9mA\x1b[21;22;23;24;25;27;28;29m");
            ctx.parse(b"\x1b[30;47;90;107mB\x1b[38;5;123;48;5;231mC");
            ctx.parse(b"\x1b[38;2;1;2;3;48;2;250;128;64mD\x1b[39;49;0m");
            ctx.parse(b"\x1b[38:5:42mE\x1b[48:2::10:20:30mF\x1b[4:3;58:2::4:5:6;59m");
            ctx.parse(b"\x1b[2h\x1b[4h\x1b[12h\x1b[20h\x1b[2l\x1b[4l\x1b[12l\x1b[20l");
            ctx.parse(b"\x1b[?1h\x1b[?6h\x1b[?7h\x1b[?25l\x1b[?25h\x1b[?1l\x1b[?6l\x1b[?7l");
            ctx.parse(b"\x1b[3 q\x1b[5 q\x1b[0 q");
            assert_ne!(ctx.ictx.borrow().cell.cell.fg, 8);
            assert_ne!(ctx.ictx.borrow().cell.cell.bg, 8);
        }
    }

    #[test]
    fn colon_colour_fields_preserve_missing_values_and_capacity_limits() {
        let mut ctx = ParserCtx::new();
        for (sequence, expected) in [
            (&b"\x1b[38:5:42m"[..], COLOUR_FLAG_256 | 42),
            (&b"\x1b[38:5:42:0:0:0:0m"[..], COLOUR_FLAG_256 | 42),
            (&b"\x1b[38:5:42:0:0:0:0:0m"[..], 8),
            (&b"\x1b[38:5:m"[..], 8),
            (&b"\x1b[38:5:2147483648m"[..], 8),
            (&b"\x1b[38:2:1:2:3m"[..], RustColourEngine.join_rgb(1, 2, 3)),
            (
                &b"\x1b[38:2::1:2:3m"[..],
                RustColourEngine.join_rgb(1, 2, 3),
            ),
            (&b"\x1b[38:2::1:2:m"[..], 8),
        ] {
            unsafe {
                ctx.parse(b"\x1b[0m");
                ctx.parse(sequence);
            }
            assert_eq!(ctx.ictx.borrow().cell.cell.fg, expected, "{sequence:?}");
        }
    }

    #[test]
    fn palette_fields_preserve_empty_indices_and_partial_updates() {
        let mut ctx = ParserCtx::new();
        for (sequence, changed) in [
            (&b"\x1b]4;;#010203\x07"[..], &[(0, 0x010203)][..]),
            (&b"\x1b]4;-0;#010203\x07"[..], &[(0, 0x010203)][..]),
            (
                &b"\x1b]4; +1;#010203;255;#040506;\x07"[..],
                &[(1, 0x010203), (255, 0x040506)][..],
            ),
            (
                &b"\x1b]4;1;#010203;256;red;2;#040506\x07"[..],
                &[(1, 0x010203)][..],
            ),
            (&b"\x1b]4;1;invalid;2;#040506\x07"[..], &[(2, 0x040506)][..]),
            (&b"\x1b]4;1 ;red;2;#040506\x07"[..], &[][..]),
            (&b"\x1b]4;1;#010203;2\x07"[..], &[(1, 0x010203)][..]),
        ] {
            unsafe {
                ctx.parse(b"\x1b]104\x07");
                ctx.parse(sequence);
            }
            for index in [0, 1, 2, 255] {
                let actual = unsafe {
                    ctx.ictx.borrow_mut().palette_colour(index | COLOUR_FLAG_256)
                };
                let expected = changed
                    .iter()
                    .find(|(idx, _)| *idx == index)
                    .map_or(-1, |(_, rgb)| crate::style::COLOUR_FLAG_RGB | rgb);
                assert_eq!(actual, expected, "{sequence:?}: {index}");
            }
        }
    }

    #[test]
    fn hyperlink_fields_preserve_uri_delimiters_and_reject_duplicate_ids() {
        let mut ctx = ParserCtx::new();
        unsafe {
            ctx.parse(b"\x1b]8;id=:foo=x:id=one;https://example.invalid/a;b:c\x07");
            let link = ctx.ictx.borrow().cell.cell.link;
            assert_ne!(link, 0);
            let links = ctx.pane.base().hyperlinks();
            let (uri, id, _) = links.get(link).unwrap();
            assert_eq!(uri.as_c_str(), c"https://example.invalid/a;b:c");
            assert_eq!(id.as_c_str(), c"one");
            for invalid in [
                &b"\x1b]8;id=one:id=two;https://rejected.invalid\x07"[..],
                &b"\x1b]8;id=one:id=two;\x07"[..],
                &b"\x1b]8;id=one\x07"[..],
            ] {
                ctx.parse(invalid);
                assert_eq!(ctx.ictx.borrow().cell.cell.link, link);
            }
            ctx.parse(b"\x1b]8;id=:id=;https://anonymous.invalid\x07");
            let anonymous = ctx.ictx.borrow().cell.cell.link;
            let (uri, id, _) = links.get(anonymous).unwrap();
            assert_eq!(uri.as_c_str(), c"https://anonymous.invalid");
            assert!(id.is_empty());
            ctx.parse(b"\x1b]8;;\x07");
            assert_eq!(ctx.ictx.borrow().cell.cell.link, 0);
        }
    }

    #[test]
    fn progress_fields_preserve_omitted_values_and_reject_invalid_percentages() {
        let mut ctx = ParserCtx::new();
        for (sequence, state, progress) in [
            (&b"\x1b]9;4;1\x07"[..], PROGRESS_BAR_NORMAL, 37),
            (&b"\x1b]9;4;1;\x07"[..], PROGRESS_BAR_NORMAL, 37),
            (&b"\x1b]9;4;1;0\x07"[..], PROGRESS_BAR_NORMAL, 0),
            (&b"\x1b]9;4;1;100\x07"[..], PROGRESS_BAR_NORMAL, 100),
            (&b"\x1b]9;4;4;000100\x07"[..], PROGRESS_BAR_PAUSED, 100),
            (&b"\x1b]9;4;1;101\x07"[..], PROGRESS_BAR_ERROR, 37),
            (
                &b"\x1b]9;4;1;9999999999999999999999999\x07"[..],
                PROGRESS_BAR_ERROR,
                37,
            ),
            (&b"\x1b]9;4;1;-1\x07"[..], PROGRESS_BAR_ERROR, 37),
            (&b"\x1b]9;4;1; 1\x07"[..], PROGRESS_BAR_ERROR, 37),
            (&b"\x1b]9;4;1;1x\x07"[..], PROGRESS_BAR_ERROR, 37),
            (&b"\x1b]9;4;1;;\x07"[..], PROGRESS_BAR_ERROR, 37),
            (&b"\x1b]9;4;5;1\x07"[..], PROGRESS_BAR_ERROR, 37),
            (&b"\x1b]9;4;\x07"[..], PROGRESS_BAR_ERROR, 37),
        ] {
            unsafe {
                ctx.parse(b"\x1b]9;4;2;37\x07");
                ctx.parse(sequence);
                let actual = &ctx.pane.base().progress_bar();
                assert_eq!(
                    (actual.state, actual.progress),
                    (state, progress),
                    "{sequence:?}"
                );
            }
        }
    }

    #[test]
    fn palette_resets_distinguish_empty_indices_from_a_trailing_separator() {
        let mut ctx = ParserCtx::new();
        for (sequence, cleared) in [
            (&b"\x1b]104;1;2;\x07"[..], &[1, 2][..]),
            (&b"\x1b]104;;1\x07"[..], &[0, 1][..]),
            (&b"\x1b]104;1;;\x07"[..], &[0, 1][..]),
            (&b"\x1b]104;-0\x07"[..], &[0][..]),
            (&b"\x1b]104; +1;2\x07"[..], &[1, 2][..]),
            (&b"\x1b]104;1;256;2\x07"[..], &[1][..]),
            (&b"\x1b]104;1 ;2\x07"[..], &[][..]),
            (&b"\x1b]104\x07"[..], &[0, 1, 2][..]),
        ] {
            unsafe {
                ctx.parse(b"\x1b]4;0;#010203;1;#010203;2;#010203\x07");
                ctx.parse(sequence);
            }
            for index in [0, 1, 2] {
                let actual = unsafe {
                    ctx.ictx.borrow_mut().palette_colour(index | COLOUR_FLAG_256)
                };
                let expected = if cleared.contains(&index) {
                    -1
                } else {
                    RustColourEngine.join_rgb(1, 2, 3)
                };
                assert_eq!(actual, expected, "{sequence:?}: {index}");
            }
        }
    }

    #[test]
    fn osc_dispatch_preserves_wrapped_commands_and_recovers_after_invalid_headers() {
        let mut ctx = ParserCtx::new();
        unsafe {
            ctx.parse(b"\x1b]4294967305;4;1;25\x07");
            let screen = ctx.pane.base();
            assert_eq!(screen.progress_bar().state, PROGRESS_BAR_NORMAL);
            assert_eq!(screen.progress_bar().progress, 25);
            assert_eq!(ctx.ictx.borrow().input_buf, b"4294967305;4;1;25\0");
            for invalid in [
                &b"\x1b]9x;4;2;50\x07"[..],
                &b"\x1b];4;2;50\x07"[..],
                &b"\x1b]9\x07"[..],
                &b"\x1b]999;4;2;50\x07"[..],
            ] {
                ctx.parse(invalid);
                assert_eq!(ctx.pane.base().progress_bar().progress, 25);
            }
            ctx.parse(b"\x1b]9;4;2;50\x1b\\");
            let screen = ctx.pane.base();
            assert_eq!(screen.progress_bar().state, PROGRESS_BAR_ERROR);
            assert_eq!(screen.progress_bar().progress, 50);
            assert_eq!(ctx.ictx.borrow().input_buf, b"\0");
        }
    }

    #[test]
    fn clipboard_fields_filter_selectors_and_preserve_binary_decoding() {
        let ctx = ParserCtx::new();
        let mut ictx = ctx.ictx.borrow_mut();
        unsafe {
            let options = global_options
                .get()
                .expect("global options are initialized");
            (options).set_number(c"set-clipboard", 2);
            for (input, selectors, bytes) in [
                (c"ccp0p7xz;YQBi", c"cp07", &b"a\0b"[..]),
                (c";YQ==", c"", &b"a"[..]),
                (c"q; \tYQ==\n", c"q", &b"a"[..]),
                (c"c; \t", c"c", &b""[..]),
            ] {
                let (clip, decoded) = input_osc_52_parse(&mut ictx, input).unwrap();
                assert_eq!(clip.as_c_str(), selectors);
                assert_eq!(decoded, bytes);
            }
            for invalid in [c"c", c"c;", c"c;Y===", c"c;?"] {
                assert!(input_osc_52_parse(&mut ictx, invalid).is_none());
            }
            (options).set_number(c"set-clipboard", 1);
            assert!(input_osc_52_parse(&mut ictx, c"c;YQ==").is_none());
        }
    }

    #[test]
    fn ignored_window_operations_consume_their_arguments_before_restoring_titles() {
        let mut ctx = ParserCtx::new();
        unsafe {
            ctx.parse(b"\x1b]2;first\x07\x1b[22;0t\x1b]2;second\x07");
            ctx.parse(b"\x1b[3;99;22;23;0t");
            assert_eq!(ctx.pane.base().title(), Some(c"first"));
            ctx.parse(b"\x1b[22;0t\x1b]2;third\x07");
            ctx.parse(b"\x1b[9;22;23;0t");
            assert_eq!(ctx.pane.base().title(), Some(c"first"));
        }
    }

    #[test]
    fn a_parser_releases_each_screen_and_callback_between_fragments() {
        let _guard = globals();
        ensure_reactor();
        unsafe {
            let parser = InputCtxRef::create(InputOwner::Detached, Stream::NONE);
            let mut first = RustScreen::new_with_server_options(8, 2, 0);
            let capture = Rc::new(());
            let weak = Rc::downgrade(&capture);
            let callback = Rc::new(move |_: &mut tty_ctx| {
                let _ = &capture;
            });
            parser.parse_screen(
                &mut first,
                Some(callback),
                ByteBuffer::from(bytes::Bytes::from_static(b"\x1b[31")),
            );
            assert!(weak.upgrade().is_none());
            drop(first);
            let mut second = RustScreen::new_with_server_options(8, 2, 0);
            parser.parse_screen(
                &mut second,
                None,
                ByteBuffer::from(bytes::Bytes::from_static(b"mX")),
            );
            let cell = RustScreen::grid(&second).cell(0, 0);
            assert_eq!(cell.data.data[0], b'X');
            assert_eq!(cell.fg, 1);
            drop(second);
            parser.close();
        }
    }

    #[test]
    fn osc_and_string_states_cover_titles_colours_hyperlinks_and_terminators() {
        let mut ctx = ParserCtx::new();
        unsafe {
            ctx.parse(b"\x1b]0;title\x07\x1b]2;pane title\x1b\\");
            ctx.parse(b"\x1b]4;1;rgb:11/22/33\x07\x1b]104;1\x07");
            ctx.parse(b"\x1b]8;id=one;https://example.invalid/\x07link\x1b]8;;\x07");
            ctx.parse(b"\x1b]10;rgb:aa/bb/cc\x07\x1b]110\x07");
            ctx.parse(b"\x1b]11;rgb:01/02/03\x07\x1b]111\x07");
            ctx.parse(b"\x1b]12;rgb:10/20/30\x07\x1b]112\x07");
            ctx.parse(b"\x1b_ignored application string\x1b\\\x1bkignored rename\x1b\\");
            assert_eq!(ctx.ictx.borrow().state.name, c"ground");
        }
    }
}

#[cfg(test)]
#[path = "parser_deep_tests.rs"]
mod deep_tests;

impl InputCtxRef {
    pub unsafe fn create(owner_of: InputOwner, bev: Stream) -> InputCtxRef {
        unsafe {
            let mut input_buf = Vec::with_capacity(INPUT_BUF_START as usize);
            input_buf.push(b'\0');
            let since_ground = Box::new(ByteBuffer::new());
            let ictx_box = InputCtxRef::new(input_ctx {
                owner_of,
                event: bev,
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
                next_request_id: 0,
                request_count: 0,
                request_timer: TimerHandle(0),
                since_ground: Some(since_ground),
                ground_timer: TimerHandle(0),
                owner: None,
            });
            let mut ictx_guard = ictx_box.borrow_mut();
            let ictx = &mut *ictx_guard;
            let watching = ictx_box.downgrade();
            ictx.ground_timer.set_callback({
                let watching = watching.clone();
                move || {
                    if let Some(ictx) = watching.upgrade() {
                        input_ground_timer_callback(&mut ictx.borrow_mut());
                    }
                }
            });
            ictx.request_timer.set_callback(move || {
                if let Some(ictx) = watching.upgrade() {
                    input_request_timer_callback(&mut ictx.borrow_mut());
                }
            });
            (*ictx).reset(0 as core::ffi::c_int);
            drop(ictx_guard);
            ictx_box
        }
    }
    pub(crate) unsafe fn close(self) {
        let reference = self;

        unsafe {
            let mut ictx_guard = reference.borrow_mut();
            let ictx = &mut *ictx_guard;
            let requests = ictx
                .requests
                .iter()
                .map(|request| input_request_handle {
                    ictx: request.ictx.clone(),
                    id: request.id,
                })
                .collect::<Vec<_>>();
            for handle in requests {
                input_free_request_in(ictx, handle);
            }
            ictx.requests.clear();
            ictx.request_timer.disarm();
            ictx.ground_timer.disarm();
            let mut pane = ictx.pane_ref();
            if let Some(wp) = pane.as_mut().and_then(|pane| pane.get_mut()) { wp.stop_sync(); }
            drop(ictx_guard);
            drop(reference);
        }
    }
}

impl input_ctx {
    pub unsafe fn reset(&mut self, clear: core::ffi::c_int) {
        let ictx = self;

        unsafe {
            let mut pane = ictx.pane_ref();
            input_reset_cell(ictx);
            if clear != 0
                && let Some(wp) = pane.as_mut().and_then(|pane| pane.get_mut())
            {
                let mut sctx = if wp.active_mode().is_none() {
                    RustScreenWriteCtx::on_pane_base(wp)
                } else {
                    RustScreenWriteCtx::on_screen(wp.base_mut())
                };
                sctx.reset();
            }
            input_clear(ictx);
            ictx.state = &input_state_ground;
            ictx.flags = 0 as core::ffi::c_int;
        }
    }
    pub fn pending(&mut self) -> Option<&mut ByteBuffer> {
        let ictx = self;

        ictx.since_ground.as_deref_mut()
    }
    pub(crate) unsafe fn parse_screen(
        &mut self,
        s: &mut RustScreen,
        init_ctx: screen_write_init_ctx,
        input: ByteBuffer,
    ) {
        let ictx = self;

        unsafe {
            if input.is_empty() {
                return;
            }
            let mut sctx = RustScreenWriteCtx::on_callback_owned(s, init_ctx);
            input_parse(ictx, &mut sctx, input);
        }
    }
    pub(crate) unsafe fn parse_shared_screen(
        &mut self,
        screen: &ScreenRef,
        init_ctx: screen_write_init_ctx,
        input: ByteBuffer,
    ) {
        let ictx = self;

        if input.is_empty() {
            return;
        }
        let mut writer = RustScreenWriteCtx::on_shared_callback(screen, init_ctx);
        unsafe { input_parse(ictx, &mut writer, input) };
    }
}

impl InputCtxRef {
    pub unsafe fn reset(&self, clear: core::ffi::c_int) {
        unsafe { self.borrow_mut().reset(clear) }
    }
    pub fn pending(&self) -> Option<RefMut<'_, ByteBuffer>> {
        RefMut::filter_map(self.borrow_mut(), input_ctx::pending).ok()
    }
    pub(crate) unsafe fn parse_screen(
        &self,
        screen: &mut RustScreen,
        init_ctx: screen_write_init_ctx,
        input: ByteBuffer,
    ) {
        unsafe { self.borrow_mut().parse_screen(screen, init_ctx, input) }
    }
    pub(crate) unsafe fn parse_shared_screen(
        &self,
        screen: &ScreenRef,
        init_ctx: screen_write_init_ctx,
        input: ByteBuffer,
    ) {
        unsafe {
            self.borrow_mut()
                .parse_shared_screen(screen, init_ctx, input)
        }
    }
}

impl ClientRef {
    pub unsafe fn handle_input_reply(
        &mut self,
        type_0: input_request_type,
        data: &InputRequestData,
    ) {
        let c = self;

        unsafe {
            let identity = c.as_ptr();
            let requests = c.input_requests_mut().clone();
            let mut found: Option<input_request_handle> = None;
            let mut complete: core::ffi::c_int = 0 as core::ffi::c_int;
            for ir in requests {
                let Some((request_type, request_idx)) =
                    input_request(&ir, |request| (request.type_0, request.idx))
                else {
                    continue;
                };
                if request_type as core::ffi::c_uint != type_0 as core::ffi::c_uint {
                    input_free_request_for_client(identity, c.input_requests_mut(), ir);
                } else if type_0 as core::ffi::c_uint
                    == INPUT_REQUEST_PALETTE as core::ffi::c_int as core::ffi::c_uint
                {
                    let &InputRequestData::Palette { idx, .. } = data else {
                        return;
                    };
                    if idx != request_idx {
                        input_free_request_for_client(identity, c.input_requests_mut(), ir);
                    } else {
                        found = Some(ir);
                        break;
                    }
                } else if type_0 as core::ffi::c_uint
                    == INPUT_REQUEST_CLIPBOARD as core::ffi::c_int as core::ffi::c_uint
                {
                    found = Some(ir);
                    break;
                }
            }
            let Some(found) = found else {
                return;
            };
            let Some(owner) = found.ictx.upgrade() else {
                return;
            };
            let owner_requests = owner
                .borrow()
                .requests
                .iter()
                .map(|request| input_request_handle {
                    ictx: request.ictx.clone(),
                    id: request.id,
                })
                .collect::<Vec<_>>();
            for ir in owner_requests {
                let Some(request_type) =
                    input_request_in(&owner.borrow(), ir.id, |request| request.type_0)
                else {
                    continue;
                };
                if complete != 0
                    && request_type as core::ffi::c_uint
                        != INPUT_REQUEST_QUEUE as core::ffi::c_int as core::ffi::c_uint
                {
                    break;
                }
                if request_type as core::ffi::c_uint
                    == INPUT_REQUEST_QUEUE as core::ffi::c_int as core::ffi::c_uint
                {
                    {
                        let data = input_request_in(&owner.borrow(), ir.id, |request| {
                            request.data.clone()
                        })
                        .flatten();
                        input_send_reply(&mut owner.borrow_mut(), data.as_deref().unwrap_or(c""));
                    }
                } else if ir.matches(&found) {
                    if request_type as core::ffi::c_uint
                        == INPUT_REQUEST_PALETTE as core::ffi::c_int as core::ffi::c_uint
                    {
                        let end = input_request_in(&owner.borrow(), ir.id, |request| request.end);
                        if let Some(end) = end {
                            input_request_palette_reply(&mut owner.borrow_mut(), end, data);
                        }
                    } else if request_type as core::ffi::c_uint
                        == INPUT_REQUEST_CLIPBOARD as core::ffi::c_int as core::ffi::c_uint
                    {
                        let idx = input_request_in(&owner.borrow(), ir.id, |request| request.idx);
                        if let Some(idx) = idx {
                            input_request_clipboard_reply(&mut owner.borrow_mut(), idx, data);
                        }
                    }
                    complete = 1 as core::ffi::c_int;
                }
                input_free_request_in_client(
                    &mut owner.borrow_mut(),
                    ir,
                    identity,
                    c.input_requests_mut(),
                );
            }
        }
    }
}

#[cfg(test)]
pub use crate::consts::{PROGRESS_BAR_ERROR, PROGRESS_BAR_NORMAL, PROGRESS_BAR_PAUSED};
