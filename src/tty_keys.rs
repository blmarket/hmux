use crate::src::bracketed_paste::{
    match_bracketed_paste_boundary, BracketedPasteBoundary, BracketedPasteBoundaryMatch,
};
use crate::src::style::colour::{colour_format, colour_parse_x11_logged};
use crate::src::events::events_fire_client;
use crate::src::ffi::libc::{
    __ctype_b_loc, memcpy, sscanf, strcspn, strlcpy, strlen, strncmp, strsep, strtol, strtoul,
};
use crate::src::ffi::resolv::__b64_pton;
use crate::src::input::{input_client_has_requests, input_request_reply};
use crate::src::key_string::key_string_format;
use crate::src::log::{log_debug, log_get_level};
use crate::src::options::{options_array_getv, options_get, options_get_number};
use crate::src::paste::paste_add_owned;
use crate::src::reactor::{
    evbuffer_drain, evbuffer_get_length, evbuffer_pullup, event_add, event_del, event_initialized,
    event_pending, event_set,
};
use crate::src::server_client::{
    server_client_handle_key, server_client_set_term_type, server_client_update_theme_colours,
};
use crate::src::session::session_theme_changed;
use crate::src::shared::abi::ssize_t;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::args;
use crate::src::shared::client::CLIENT_FOCUSED;
use crate::src::shared::client::*;
use crate::src::shared::client::{
    client, client_file, client_file_cb, client_file_entry, client_files, overlay_check_cb,
    overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb, overlay_resize_cb,
};
use crate::src::shared::colour::*;
use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds};
use crate::src::shared::control::control_state;
use crate::src::shared::control_character::{
    control_character_code, C0_ASC, C0_BEL, C0_BS, C0_CAN, C0_CR, C0_DC1, C0_DC2, C0_DC3, C0_DC4,
    C0_DLE, C0_EM, C0_ENQ, C0_EOT, C0_ESC, C0_ETB, C0_ETX, C0_FF, C0_FS, C0_GS, C0_HT, C0_LF,
    C0_NAK, C0_NUL, C0_RS, C0_SI, C0_SO, C0_SOH, C0_STX, C0_SUB, C0_SYN, C0_US, C0_VT,
};
use crate::src::shared::ctype::{
    _ISalnum, _ISalpha, _ISblank, _IScntrl, _ISdigit, _ISgraph, _ISlower, _ISprint, _ISpunct,
    _ISspace, _ISupper, _ISxdigit, ctype_code,
};
use crate::src::shared::display::*;
use crate::src::shared::display::{visible_range, visible_ranges};
use crate::src::shared::environment::environ;
use crate::src::shared::event::EV_TIMEOUT;
use crate::src::shared::format::{format_job_tree, format_tree};
use crate::src::shared::grid::*;
use crate::src::shared::hyperlinks::hyperlinks;
use crate::src::shared::input::input_request_type;
use crate::src::shared::input::{input_ctx, input_request, input_requests};
use crate::src::shared::input::{input_request_clipboard_data, input_request_palette_data};
use crate::src::shared::input::{
    INPUT_REQUEST_CLIPBOARD, INPUT_REQUEST_PALETTE, INPUT_REQUEST_QUEUE,
};
use crate::src::shared::key::*;
use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
use crate::src::shared::layout::layout_geometry;
use crate::src::shared::layout::*;
use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
use crate::src::shared::menu::menu_data;
use crate::src::shared::message::*;
use crate::src::shared::mouse::{
    mouse_event, MOUSE_MASK_BUTTONS, MOUSE_PARAM_BTN_OFF, MOUSE_PARAM_POS_OFF, MOUSE_WHEEL_DOWN,
    MOUSE_WHEEL_UP,
};
use crate::src::shared::options::{
    options, options_array, options_array_item, options_entry, options_value,
};
use crate::src::shared::pane::{
    window_pane, window_pane_modes, window_pane_prompt, window_pane_tree_entry, window_panes,
};
use crate::src::shared::pane::{window_pane_offset, window_pane_resize, window_pane_resizes};
use crate::src::shared::posix_terminal::VERASE;
use crate::src::shared::process::tmuxpeer;
use crate::src::shared::prompt::prompt;
use crate::src::shared::redraw::redraw_scene;
use crate::src::shared::screen::{screen, screen_sel, screen_titles};
use crate::src::shared::screen_write::screen_write_cline;
use crate::src::shared::session::{session, session_entry};
use crate::src::shared::spawn::spawn_editor_state;
use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
use crate::src::shared::tty::*;
use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
use crate::src::shared::tty::{
    TTY_ALL_REQUEST_FLAGS, TTY_BRACKETPASTE, TTY_HAVEDA, TTY_HAVEDA2, TTY_HAVESYNC, TTY_HAVEXDA,
    TTY_OSC52QUERY, TTY_TIMER, TTY_WAITBG, TTY_WAITFG, TTY_WINSIZEQUERY,
};
use crate::src::shared::utf8::wchar_t;
use crate::src::shared::utf8::*;
use crate::src::shared::window::{
    window, window_entry, window_mode, window_mode_entry, window_winlinks, winlink, winlink_entry,
    winlink_stack, winlinks,
};
use crate::src::tmux::global_options;
use crate::src::tty::{tty_invalidate, tty_set_size, tty_update_features};
use crate::src::tty_features::{tty_default_features, tty_parse_client_features};
use crate::src::tty_term::tty_term_string;
use crate::src::text::utf8::{utf8_append, utf8_from_data, utf8_fromwc, utf8_open};
use crate::src::window::window_update_focus;
use std::ffi::CStr;

use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_14;
use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_13;

use crate::src::shared::key::key_code_enum as C2RustUnnamed_37;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct tty_default_key_code {
    pub code: tty_code_code,
    pub key: key_code,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tty_default_key_raw {
    pub string: &'static ::std::ffi::CStr,
    pub key: key_code,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tty_default_key_xterm {
    pub template: &'static ::std::ffi::CStr,
    pub key: key_code,
}
pub const _POSIX_VDISABLE: ::core::ffi::c_int = '\0' as i32;

static mut tty_default_raw_keys: [tty_default_key_raw; 102] = [
    tty_default_key_raw {
        string: c"\x1BO[",
        key: '\u{1b}' as i32 as key_code,
    },
    tty_default_key_raw {
        string: c"\x1BOo",
        key: KEYC_KP_SLASH as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOj",
        key: KEYC_KP_STAR as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOm",
        key: KEYC_KP_MINUS as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOw",
        key: KEYC_KP_SEVEN as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOx",
        key: KEYC_KP_EIGHT as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOy",
        key: KEYC_KP_NINE as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOk",
        key: KEYC_KP_PLUS as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOt",
        key: KEYC_KP_FOUR as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOu",
        key: KEYC_KP_FIVE as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOv",
        key: KEYC_KP_SIX as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOq",
        key: KEYC_KP_ONE as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOr",
        key: KEYC_KP_TWO as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOs",
        key: KEYC_KP_THREE as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOM",
        key: KEYC_KP_ENTER as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOp",
        key: KEYC_KP_ZERO as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOn",
        key: KEYC_KP_PERIOD as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: c"\x1BOA",
        key: KEYC_UP as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_raw {
        string: c"\x1BOB",
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_raw {
        string: c"\x1BOC",
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_raw {
        string: c"\x1BOD",
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_raw {
        string: c"\x1B[A",
        key: KEYC_UP as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_raw {
        string: c"\x1B[B",
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_raw {
        string: c"\x1B[C",
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_raw {
        string: c"\x1B[D",
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_raw {
        string: c"\x1B\x1BOA",
        key: KEYC_UP as ::core::ffi::c_ulong as key_code | KEYC_CURSOR | KEYC_META,
    },
    tty_default_key_raw {
        string: c"\x1B\x1BOB",
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code | KEYC_CURSOR | KEYC_META,
    },
    tty_default_key_raw {
        string: c"\x1B\x1BOC",
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code | KEYC_CURSOR | KEYC_META,
    },
    tty_default_key_raw {
        string: c"\x1B\x1BOD",
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code | KEYC_CURSOR | KEYC_META,
    },
    tty_default_key_raw {
        string: c"\x1B\x1B[A",
        key: KEYC_UP as ::core::ffi::c_ulong as key_code | KEYC_CURSOR | KEYC_META,
    },
    tty_default_key_raw {
        string: c"\x1B\x1B[B",
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code | KEYC_CURSOR | KEYC_META,
    },
    tty_default_key_raw {
        string: c"\x1B\x1B[C",
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code | KEYC_CURSOR | KEYC_META,
    },
    tty_default_key_raw {
        string: c"\x1B\x1B[D",
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code | KEYC_CURSOR | KEYC_META,
    },
    tty_default_key_raw {
        string: c"\x1BOH",
        key: KEYC_HOME as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1BOF",
        key: KEYC_END as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1B\x1BOH",
        key: KEYC_HOME as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_raw {
        string: c"\x1B\x1BOF",
        key: KEYC_END as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_raw {
        string: c"\x1B[H",
        key: KEYC_HOME as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1B[F",
        key: KEYC_END as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1B\x1B[H",
        key: KEYC_HOME as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_raw {
        string: c"\x1B\x1B[F",
        key: KEYC_END as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_raw {
        string: c"\x1BOa",
        key: KEYC_UP as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1BOb",
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1BOc",
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1BOd",
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1B[a",
        key: KEYC_UP as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[b",
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[c",
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[d",
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[11~",
        key: KEYC_F1 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1B[12~",
        key: KEYC_F2 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1B[13~",
        key: KEYC_F3 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1B[14~",
        key: KEYC_F4 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1B[15~",
        key: KEYC_F5 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1B[17~",
        key: KEYC_F6 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1B[18~",
        key: KEYC_F7 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1B[19~",
        key: KEYC_F8 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1B[20~",
        key: KEYC_F9 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1B[21~",
        key: KEYC_F10 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1B[23~",
        key: KEYC_F1 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[24~",
        key: KEYC_F2 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[25~",
        key: KEYC_F3 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[26~",
        key: KEYC_F4 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[28~",
        key: KEYC_F5 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[29~",
        key: KEYC_F6 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[31~",
        key: KEYC_F7 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[32~",
        key: KEYC_F8 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[33~",
        key: KEYC_F9 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[34~",
        key: KEYC_F10 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[23$",
        key: KEYC_F11 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[24$",
        key: KEYC_F12 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[11^",
        key: KEYC_F1 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1B[12^",
        key: KEYC_F2 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1B[13^",
        key: KEYC_F3 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1B[14^",
        key: KEYC_F4 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1B[15^",
        key: KEYC_F5 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1B[17^",
        key: KEYC_F6 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1B[18^",
        key: KEYC_F7 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1B[19^",
        key: KEYC_F8 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1B[20^",
        key: KEYC_F9 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1B[21^",
        key: KEYC_F10 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1B[23^",
        key: KEYC_F11 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1B[24^",
        key: KEYC_F12 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: c"\x1B[11@",
        key: KEYC_F1 as ::core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[12@",
        key: KEYC_F2 as ::core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[13@",
        key: KEYC_F3 as ::core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[14@",
        key: KEYC_F4 as ::core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[15@",
        key: KEYC_F5 as ::core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[17@",
        key: KEYC_F6 as ::core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[18@",
        key: KEYC_F7 as ::core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[19@",
        key: KEYC_F8 as ::core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[20@",
        key: KEYC_F9 as ::core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[21@",
        key: KEYC_F10 as ::core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[23@",
        key: KEYC_F11 as ::core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[24@",
        key: KEYC_F12 as ::core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[I",
        key: KEYC_FOCUS_IN as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1B[O",
        key: KEYC_FOCUS_OUT as ::core::ffi::c_ulong as key_code,
    },
    // Keep paste boundaries in the key tree so terminal and user-key entries
    // can override them, with the same prefix/escape-time rules as other keys.
    tty_default_key_raw {
        string: c"\x1B[200~",
        key: KEYC_PASTE_START as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
    },
    tty_default_key_raw {
        string: c"\x1B[201~",
        key: KEYC_PASTE_END as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
    },
    tty_default_key_raw {
        string: c"\x1B[1;5Z",
        key: '\t' as i32 as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: c"\x1B[?997;1n",
        key: KEYC_REPORT_DARK_THEME as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: c"\x1B[?997;2n",
        key: KEYC_REPORT_LIGHT_THEME as ::core::ffi::c_ulong as key_code,
    },
];
static mut tty_default_xterm_keys: [tty_default_key_xterm; 30] = [
    tty_default_key_xterm {
        template: c"\x1B[1;_P",
        key: KEYC_F1 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1BO1;_P",
        key: KEYC_F1 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1BO_P",
        key: KEYC_F1 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[1;_Q",
        key: KEYC_F2 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1BO1;_Q",
        key: KEYC_F2 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1BO_Q",
        key: KEYC_F2 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[1;_R",
        key: KEYC_F3 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1BO1;_R",
        key: KEYC_F3 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1BO_R",
        key: KEYC_F3 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[1;_S",
        key: KEYC_F4 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1BO1;_S",
        key: KEYC_F4 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1BO_S",
        key: KEYC_F4 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[15;_~",
        key: KEYC_F5 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[17;_~",
        key: KEYC_F6 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[18;_~",
        key: KEYC_F7 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[19;_~",
        key: KEYC_F8 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[20;_~",
        key: KEYC_F9 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[21;_~",
        key: KEYC_F10 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[23;_~",
        key: KEYC_F11 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[24;_~",
        key: KEYC_F12 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[1;_A",
        key: KEYC_UP as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[1;_B",
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[1;_C",
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[1;_D",
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[1;_H",
        key: KEYC_HOME as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[1;_F",
        key: KEYC_END as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[5;_~",
        key: KEYC_PPAGE as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[6;_~",
        key: KEYC_NPAGE as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[2;_~",
        key: KEYC_IC as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: c"\x1B[3;_~",
        key: KEYC_DC as ::core::ffi::c_ulong as key_code,
    },
];
static mut tty_default_xterm_modifiers: [key_code; 10] = [
    0 as ::core::ffi::c_int as key_code,
    0 as ::core::ffi::c_int as key_code,
    KEYC_SHIFT,
    KEYC_META | KEYC_IMPLIED_META,
    KEYC_SHIFT | KEYC_META | KEYC_IMPLIED_META,
    KEYC_CTRL,
    KEYC_SHIFT | KEYC_CTRL,
    KEYC_META | KEYC_IMPLIED_META | KEYC_CTRL,
    KEYC_SHIFT | KEYC_META | KEYC_IMPLIED_META | KEYC_CTRL,
    KEYC_META | KEYC_IMPLIED_META,
];
static mut tty_default_code_keys: [tty_default_key_code; 136] = [
    tty_default_key_code {
        code: TTYC_KF1,
        key: KEYC_F1 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF2,
        key: KEYC_F2 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF3,
        key: KEYC_F3 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF4,
        key: KEYC_F4 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF5,
        key: KEYC_F5 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF6,
        key: KEYC_F6 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF7,
        key: KEYC_F7 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF8,
        key: KEYC_F8 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF9,
        key: KEYC_F9 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF10,
        key: KEYC_F10 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF11,
        key: KEYC_F11 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF12,
        key: KEYC_F12 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF13,
        key: KEYC_F1 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF14,
        key: KEYC_F2 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF15,
        key: KEYC_F3 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF16,
        key: KEYC_F4 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF17,
        key: KEYC_F5 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF18,
        key: KEYC_F6 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF19,
        key: KEYC_F7 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF20,
        key: KEYC_F8 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF21,
        key: KEYC_F9 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF22,
        key: KEYC_F10 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF23,
        key: KEYC_F11 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF24,
        key: KEYC_F12 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF25,
        key: KEYC_F1 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF26,
        key: KEYC_F2 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF27,
        key: KEYC_F3 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF28,
        key: KEYC_F4 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF29,
        key: KEYC_F5 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF30,
        key: KEYC_F6 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF31,
        key: KEYC_F7 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF32,
        key: KEYC_F8 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF33,
        key: KEYC_F9 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF34,
        key: KEYC_F10 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF35,
        key: KEYC_F11 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF36,
        key: KEYC_F12 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF37,
        key: KEYC_F1 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF38,
        key: KEYC_F2 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF39,
        key: KEYC_F3 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF40,
        key: KEYC_F4 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF41,
        key: KEYC_F5 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF42,
        key: KEYC_F6 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF43,
        key: KEYC_F7 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF44,
        key: KEYC_F8 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF45,
        key: KEYC_F9 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF46,
        key: KEYC_F10 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF47,
        key: KEYC_F11 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF48,
        key: KEYC_F12 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF49,
        key: KEYC_F1 as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF50,
        key: KEYC_F2 as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF51,
        key: KEYC_F3 as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF52,
        key: KEYC_F4 as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF53,
        key: KEYC_F5 as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF54,
        key: KEYC_F6 as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF55,
        key: KEYC_F7 as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF56,
        key: KEYC_F8 as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF57,
        key: KEYC_F9 as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF58,
        key: KEYC_F10 as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF59,
        key: KEYC_F11 as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF60,
        key: KEYC_F12 as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF61,
        key: KEYC_F1 as ::core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF62,
        key: KEYC_F2 as ::core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF63,
        key: KEYC_F3 as ::core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KICH1,
        key: KEYC_IC as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KDCH1,
        key: KEYC_DC as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KHOME,
        key: KEYC_HOME as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KEND,
        key: KEYC_END as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KNP,
        key: KEYC_NPAGE as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KPP,
        key: KEYC_PPAGE as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KCBT,
        key: KEYC_BTAB as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KCUU1,
        key: KEYC_UP as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_code {
        code: TTYC_KCUD1,
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_code {
        code: TTYC_KCUB1,
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_code {
        code: TTYC_KCUF1,
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_code {
        code: TTYC_KDC2,
        key: KEYC_DC as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KDC3,
        key: KEYC_DC as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KDC4,
        key: KEYC_DC as ::core::ffi::c_ulong as key_code
            | KEYC_SHIFT
            | KEYC_META
            | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KDC5,
        key: KEYC_DC as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KDC6,
        key: KEYC_DC as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KDC7,
        key: KEYC_DC as ::core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KIND,
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KDN2,
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KDN3,
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KDN4,
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code
            | KEYC_SHIFT
            | KEYC_META
            | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KDN5,
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KDN6,
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KDN7,
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KEND2,
        key: KEYC_END as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KEND3,
        key: KEYC_END as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KEND4,
        key: KEYC_END as ::core::ffi::c_ulong as key_code
            | KEYC_SHIFT
            | KEYC_META
            | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KEND5,
        key: KEYC_END as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KEND6,
        key: KEYC_END as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KEND7,
        key: KEYC_END as ::core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KHOM2,
        key: KEYC_HOME as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KHOM3,
        key: KEYC_HOME as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KHOM4,
        key: KEYC_HOME as ::core::ffi::c_ulong as key_code
            | KEYC_SHIFT
            | KEYC_META
            | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KHOM5,
        key: KEYC_HOME as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KHOM6,
        key: KEYC_HOME as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KHOM7,
        key: KEYC_HOME as ::core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KIC2,
        key: KEYC_IC as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KIC3,
        key: KEYC_IC as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KIC4,
        key: KEYC_IC as ::core::ffi::c_ulong as key_code
            | KEYC_SHIFT
            | KEYC_META
            | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KIC5,
        key: KEYC_IC as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KIC6,
        key: KEYC_IC as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KIC7,
        key: KEYC_IC as ::core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KLFT2,
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KLFT3,
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KLFT4,
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code
            | KEYC_SHIFT
            | KEYC_META
            | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KLFT5,
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KLFT6,
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KLFT7,
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KNXT2,
        key: KEYC_NPAGE as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KNXT3,
        key: KEYC_NPAGE as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KNXT4,
        key: KEYC_NPAGE as ::core::ffi::c_ulong as key_code
            | KEYC_SHIFT
            | KEYC_META
            | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KNXT5,
        key: KEYC_NPAGE as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KNXT6,
        key: KEYC_NPAGE as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KNXT7,
        key: KEYC_NPAGE as ::core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KPRV2,
        key: KEYC_PPAGE as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KPRV3,
        key: KEYC_PPAGE as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KPRV4,
        key: KEYC_PPAGE as ::core::ffi::c_ulong as key_code
            | KEYC_SHIFT
            | KEYC_META
            | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KPRV5,
        key: KEYC_PPAGE as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KPRV6,
        key: KEYC_PPAGE as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KPRV7,
        key: KEYC_PPAGE as ::core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KRIT2,
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KRIT3,
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KRIT4,
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code
            | KEYC_SHIFT
            | KEYC_META
            | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KRIT5,
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KRIT6,
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KRIT7,
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KRI,
        key: KEYC_UP as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KUP2,
        key: KEYC_UP as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KUP3,
        key: KEYC_UP as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KUP4,
        key: KEYC_UP as ::core::ffi::c_ulong as key_code
            | KEYC_SHIFT
            | KEYC_META
            | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KUP5,
        key: KEYC_UP as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KUP6,
        key: KEYC_UP as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KUP7,
        key: KEYC_UP as ::core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_CTRL,
    },
];
unsafe extern "C" fn tty_keys_add(
    mut tty: *mut tty,
    mut s: *const ::core::ffi::c_char,
    mut key: key_code,
) {
    let mut tk: *mut tty_key = ::core::ptr::null_mut::<tty_key>();
    let mut size: size_t = 0;
    let mut keystr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let key_string = key_string_format(key, true);
    keystr = key_string.as_ptr();
    tk = tty_keys_find(tty, s, strlen(s), &raw mut size);
    if tk.is_null() {
        log_debug(
            b"new key %s: 0x%llx (%s)\0" as *const u8 as *const ::core::ffi::c_char,
            s,
            key,
            keystr,
        );
        tty_keys_add1(&raw mut (*tty).key_tree, s, key);
    } else {
        log_debug(
            b"replacing key %s: 0x%llx (%s)\0" as *const u8 as *const ::core::ffi::c_char,
            s,
            key,
            keystr,
        );
        (*tk).key = key;
    };
}
unsafe extern "C" fn tty_keys_add1(
    mut tkp: *mut *mut tty_key,
    mut s: *const ::core::ffi::c_char,
    mut key: key_code,
) {
    let mut tk: *mut tty_key = ::core::ptr::null_mut::<tty_key>();
    tk = *tkp;
    if tk.is_null() {
        *tkp = Box::into_raw(Box::new(tty_key {
            ch: *s,
            key: KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code,
            left: ::core::ptr::null_mut(),
            right: ::core::ptr::null_mut(),
            next: ::core::ptr::null_mut(),
        }));
        tk = *tkp;
    }
    if *s as ::core::ffi::c_int == (*tk).ch as ::core::ffi::c_int {
        s = s.offset(1);
        if *s as ::core::ffi::c_int == '\0' as i32 {
            (*tk).key = key;
            return;
        }
        tkp = &raw mut (*tk).next;
    } else if (*s as ::core::ffi::c_int) < (*tk).ch as ::core::ffi::c_int {
        tkp = &raw mut (*tk).left;
    } else if *s as ::core::ffi::c_int > (*tk).ch as ::core::ffi::c_int {
        tkp = &raw mut (*tk).right;
    }
    tty_keys_add1(tkp, s, key);
}
#[no_mangle]
pub unsafe extern "C" fn tty_keys_build(mut tty: *mut tty) {
    let mut tdkr: *const tty_default_key_raw = ::core::ptr::null::<tty_default_key_raw>();
    let mut tdkx: *const tty_default_key_xterm = ::core::ptr::null::<tty_default_key_xterm>();
    let mut tdkc: *const tty_default_key_code = ::core::ptr::null::<tty_default_key_code>();
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut ov: *mut options_value = ::core::ptr::null_mut::<options_value>();
    let mut copy: [::core::ffi::c_char; 16] = [0; 16];
    let mut key: key_code = 0;
    if !(*tty).key_tree.is_null() {
        tty_keys_free(tty);
    }
    (*tty).key_tree = ::core::ptr::null_mut::<tty_key>();
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[tty_default_key_xterm; 30]>() as usize)
            .wrapping_div(::core::mem::size_of::<tty_default_key_xterm>() as usize)
    {
        tdkx = (&raw const tty_default_xterm_keys as *const tty_default_key_xterm)
            .offset(i as isize) as *const tty_default_key_xterm;
        j = 2 as u_int;
        while (j as usize)
            < (::core::mem::size_of::<[key_code; 10]>() as usize)
                .wrapping_div(::core::mem::size_of::<key_code>() as usize)
        {
            strlcpy(
                &raw mut copy as *mut ::core::ffi::c_char,
                (*tdkx).template.as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 16]>() as size_t,
            );
            copy[strcspn(
                &raw mut copy as *mut ::core::ffi::c_char,
                b"_\0" as *const u8 as *const ::core::ffi::c_char,
            ) as usize] = ('0' as i32 as u_int).wrapping_add(j) as ::core::ffi::c_char;
            key = (*tdkx).key | tty_default_xterm_modifiers[j as usize];
            tty_keys_add(tty, &raw mut copy as *mut ::core::ffi::c_char, key);
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[tty_default_key_raw; 102]>() as usize)
            .wrapping_div(::core::mem::size_of::<tty_default_key_raw>() as usize)
    {
        tdkr = (&raw const tty_default_raw_keys as *const tty_default_key_raw).offset(i as isize)
            as *const tty_default_key_raw;
        s = (*tdkr).string.as_ptr();
        if *s as ::core::ffi::c_int != '\0' as i32 {
            tty_keys_add(tty, s, (*tdkr).key);
        }
        i = i.wrapping_add(1);
    }
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[tty_default_key_code; 136]>() as usize)
            .wrapping_div(::core::mem::size_of::<tty_default_key_code>() as usize)
    {
        tdkc = (&raw const tty_default_code_keys as *const tty_default_key_code).offset(i as isize)
            as *const tty_default_key_code;
        s = tty_term_string((*tty).term, (*tdkc).code);
        if *s as ::core::ffi::c_int != '\0' as i32 {
            tty_keys_add(tty, s, (*tdkc).key);
        }
        i = i.wrapping_add(1);
    }
    o = options_get(
        global_options,
        b"user-keys\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if !o.is_null() {
        i = 0 as u_int;
        while i <= KEYC_NUSER as u_int {
            ov = options_array_getv(o, b"%u\0" as *const u8 as *const ::core::ffi::c_char, i);
            if !ov.is_null() {
                tty_keys_add(
                    tty,
                    (*ov).string_ptr(),
                    (KEYC_USER as ::core::ffi::c_ulong).wrapping_add(i as ::core::ffi::c_ulong)
                        as key_code,
                );
            }
            i = i.wrapping_add(1);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn tty_keys_free(mut tty: *mut tty) {
    tty_keys_free1((*tty).key_tree);
}
unsafe extern "C" fn tty_keys_free1(mut tk: *mut tty_key) {
    if !(*tk).next.is_null() {
        tty_keys_free1((*tk).next);
    }
    if !(*tk).left.is_null() {
        tty_keys_free1((*tk).left);
    }
    if !(*tk).right.is_null() {
        tty_keys_free1((*tk).right);
    }
    drop(Box::from_raw(tk));
}
unsafe extern "C" fn tty_keys_find(
    mut tty: *mut tty,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut size: *mut size_t,
) -> *mut tty_key {
    *size = 0 as size_t;
    return tty_keys_find1((*tty).key_tree, buf, len, size);
}
unsafe extern "C" fn tty_keys_find1(
    mut tk: *mut tty_key,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut size: *mut size_t,
) -> *mut tty_key {
    if len == 0 as size_t {
        return ::core::ptr::null_mut::<tty_key>();
    }
    if tk.is_null() {
        return ::core::ptr::null_mut::<tty_key>();
    }
    if (*tk).ch as ::core::ffi::c_int == *buf as ::core::ffi::c_int {
        buf = buf.offset(1);
        len = len.wrapping_sub(1);
        *size = (*size).wrapping_add(1);
        if len == 0 as size_t
            || (*tk).next.is_null() && (*tk).key != KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code
        {
            return tk;
        }
        tk = (*tk).next;
    } else if (*buf as ::core::ffi::c_int) < (*tk).ch as ::core::ffi::c_int {
        tk = (*tk).left;
    } else if *buf as ::core::ffi::c_int > (*tk).ch as ::core::ffi::c_int {
        tk = (*tk).right;
    }
    return tty_keys_find1(tk, buf, len, size);
}
unsafe extern "C" fn tty_keys_next1(
    mut tty: *mut tty,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut key: *mut key_code,
    mut size: *mut size_t,
    mut expired: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut c: *mut client = (*tty).client;
    let mut tk: *mut tty_key = ::core::ptr::null_mut::<tty_key>();
    let mut tk1: *mut tty_key = ::core::ptr::null_mut::<tty_key>();
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut more: utf8_state = UTF8_MORE;
    let mut uc: utf8_char = 0;
    let mut i: u_int = 0;
    log_debug(
        b"%s: next key is %zu (%.*s) (expired=%d)\0" as *const u8 as *const ::core::ffi::c_char,
        ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        len,
        len as ::core::ffi::c_int,
        buf,
        expired,
    );
    tk = tty_keys_find(tty, buf, len, size);
    if !tk.is_null() && (*tk).key != KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code {
        tk1 = tk;
        loop {
            log_debug(
                b"%s: keys in list: %#llx\0" as *const u8 as *const ::core::ffi::c_char,
                ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                (*tk1).key,
            );
            tk1 = (*tk1).next;
            if tk1.is_null() {
                break;
            }
        }
        if !(*tk).next.is_null() && expired == 0 {
            return 1 as ::core::ffi::c_int;
        }
        *key = (*tk).key;
        return 0 as ::core::ffi::c_int;
    }
    more = utf8_open(&raw mut ud, *buf as u_char);
    if more as ::core::ffi::c_uint == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint {
        *size = ud.size as size_t;
        if len < ud.size as size_t {
            if expired == 0 {
                return 1 as ::core::ffi::c_int;
            }
            return -(1 as ::core::ffi::c_int);
        }
        i = 1 as u_int;
        while i < ud.size as u_int {
            more = utf8_append(&raw mut ud, *buf.offset(i as isize) as u_char);
            i = i.wrapping_add(1);
        }
        if more as ::core::ffi::c_uint != UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint {
            return -(1 as ::core::ffi::c_int);
        }
        if utf8_from_data(&raw mut ud, &raw mut uc) as ::core::ffi::c_uint
            != UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return -(1 as ::core::ffi::c_int);
        }
        *key = uc as key_code;
        log_debug(
            b"%s: UTF-8 key %.*s %#llx\0" as *const u8 as *const ::core::ffi::c_char,
            ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            ud.size as ::core::ffi::c_int,
            &raw mut ud.data as *mut u_char,
            *key,
        );
        return 0 as ::core::ffi::c_int;
    }
    return -(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn tty_keys_winsz(
    mut tty: *mut tty,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut size: *mut size_t,
) -> ::core::ffi::c_int {
    let mut c: *mut client = (*tty).client;
    let mut end: size_t = 0;
    let mut tmp: [::core::ffi::c_char; 64] = [0; 64];
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut xpixel: u_int = 0;
    let mut ypixel: u_int = 0;
    let mut char_x: u_int = 0;
    let mut char_y: u_int = 0;
    *size = 0 as size_t;
    if (*tty).flags & TTY_WINSIZEQUERY == 0 {
        return -(1 as ::core::ffi::c_int);
    }
    if *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\u{1b}' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 1 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '[' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 2 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    end = 2 as size_t;
    while end < len && end != ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as usize {
        if *buf.offset(end as isize) as ::core::ffi::c_int == 't' as i32 {
            break;
        }
        if *(*__ctype_b_loc())
            .offset(*buf.offset(end as isize) as u_char as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            == 0
            && *buf.offset(end as isize) as ::core::ffi::c_int != ';' as i32
        {
            break;
        }
        end = end.wrapping_add(1);
    }
    if end == len {
        return 1 as ::core::ffi::c_int;
    }
    if end == ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as usize
        || *buf.offset(end as isize) as ::core::ffi::c_int != 't' as i32
    {
        return -(1 as ::core::ffi::c_int);
    }
    memcpy(
        &raw mut tmp as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        buf.offset(2 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        end.wrapping_sub(2 as size_t),
    );
    tmp[end.wrapping_sub(2 as size_t) as usize] = '\0' as i32 as ::core::ffi::c_char;
    if sscanf(
        &raw mut tmp as *mut ::core::ffi::c_char,
        b"8;%u;%u\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut sy,
        &raw mut sx,
    ) == 2 as ::core::ffi::c_int
    {
        tty_set_size(tty, sx, sy, (*tty).xpixel, (*tty).ypixel);
        *size = end.wrapping_add(1 as size_t);
        return 0 as ::core::ffi::c_int;
    } else if sscanf(
        &raw mut tmp as *mut ::core::ffi::c_char,
        b"4;%u;%u\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut ypixel,
        &raw mut xpixel,
    ) == 2 as ::core::ffi::c_int
    {
        char_x = if xpixel != 0 && (*tty).sx != 0 {
            xpixel.wrapping_div((*tty).sx)
        } else {
            0 as u_int
        };
        char_y = if ypixel != 0 && (*tty).sy != 0 {
            ypixel.wrapping_div((*tty).sy)
        } else {
            0 as u_int
        };
        tty_set_size(tty, (*tty).sx, (*tty).sy, char_x, char_y);
        tty_invalidate(tty);
        (*tty).flags &= !TTY_WINSIZEQUERY;
        *size = end.wrapping_add(1 as size_t);
        return 0 as ::core::ffi::c_int;
    }
    log_debug(
        b"%s: unrecognized window size sequence: %s\0" as *const u8 as *const ::core::ffi::c_char,
        ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        &raw mut tmp as *mut ::core::ffi::c_char,
    );
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn tty_keys_next(mut tty: *mut tty) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut c: *mut client = (*tty).client;
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut buf: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    let mut size: size_t = 0;
    let mut bspace: cc_t = 0;
    let mut delay: ::core::ffi::c_int = 0;
    let mut expired: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_int = 0;
    let mut bg: ::core::ffi::c_int = (*tty).bg;
    let mut key: key_code = 0;
    let mut onlykey: key_code = 0;
    let mut m: mouse_event = mouse_event {
        valid: 0 as ::core::ffi::c_int,
        ignore: 0,
        key: 0,
        statusat: 0,
        statuslines: 0,
        x: 0,
        y: 0,
        b: 0,
        lx: 0,
        ly: 0,
        lb: 0,
        ox: 0,
        oy: 0,
        s: 0,
        w: 0,
        wp: 0,
        sgr_type: 0,
        sgr_b: 0,
    };
    buf = evbuffer_pullup((*tty).in_0, -(1 as ::core::ffi::c_int) as ssize_t)
        as *const ::core::ffi::c_char;
    len = evbuffer_get_length(&*((*tty).in_0));
    if len == 0 as size_t {
        return 0 as ::core::ffi::c_int;
    }
    log_debug(
        b"%s: keys are %zu (%.*s)\0" as *const u8 as *const ::core::ffi::c_char,
        ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        len,
        len as ::core::ffi::c_int,
        buf,
    );
    match tty_keys_clipboard(tty, buf, len, &raw mut size) {
        0 => {
            key = KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
            current_block = 5025795842197473417;
        }
        -1 => {
            current_block = 1917311967535052937;
        }
        1 => {
            current_block = 16977559109335092698;
        }
        _ => {
            current_block = 1917311967535052937;
        }
    }
    match current_block {
        1917311967535052937 => {
            match tty_keys_sync(tty, buf, len, &raw mut size) {
                0 => {
                    key = KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
                    current_block = 5025795842197473417;
                }
                -1 => {
                    current_block = 4166486009154926805;
                }
                1 => {
                    current_block = 16977559109335092698;
                }
                _ => {
                    current_block = 4166486009154926805;
                }
            }
            match current_block {
                5025795842197473417 => {}
                16977559109335092698 => {}
                _ => {
                    match tty_keys_device_attributes(tty, buf, len, &raw mut size) {
                        0 => {
                            key = KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
                            current_block = 5025795842197473417;
                        }
                        -1 => {
                            current_block = 15652330335145281839;
                        }
                        1 => {
                            current_block = 16977559109335092698;
                        }
                        _ => {
                            current_block = 15652330335145281839;
                        }
                    }
                    match current_block {
                        5025795842197473417 => {}
                        16977559109335092698 => {}
                        _ => {
                            match tty_keys_device_attributes2(tty, buf, len, &raw mut size) {
                                0 => {
                                    key = KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
                                    current_block = 5025795842197473417;
                                }
                                -1 => {
                                    current_block = 224731115979188411;
                                }
                                1 => {
                                    current_block = 16977559109335092698;
                                }
                                _ => {
                                    current_block = 224731115979188411;
                                }
                            }
                            match current_block {
                                5025795842197473417 => {}
                                16977559109335092698 => {}
                                _ => {
                                    match tty_keys_extended_device_attributes(
                                        tty,
                                        buf,
                                        len,
                                        &raw mut size,
                                    ) {
                                        0 => {
                                            key = KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
                                            current_block = 5025795842197473417;
                                        }
                                        -1 => {
                                            current_block = 17478428563724192186;
                                        }
                                        1 => {
                                            current_block = 16977559109335092698;
                                        }
                                        _ => {
                                            current_block = 17478428563724192186;
                                        }
                                    }
                                    match current_block {
                                        5025795842197473417 => {}
                                        16977559109335092698 => {}
                                        _ => {
                                            match tty_keys_colours(
                                                tty,
                                                buf,
                                                len,
                                                &raw mut size,
                                                &raw mut (*tty).fg,
                                                &raw mut (*tty).bg,
                                            ) {
                                                0 => {
                                                    key = KEYC_UNKNOWN as ::core::ffi::c_ulong
                                                        as key_code;
                                                    if (*tty).bg != bg {
                                                        server_client_update_theme_colours(c);
                                                    }
                                                    session_theme_changed((*c).session);
                                                    current_block = 5025795842197473417;
                                                }
                                                -1 => {
                                                    current_block = 1538046216550696469;
                                                }
                                                1 => {
                                                    if (*tty).bg != bg {
                                                        server_client_update_theme_colours(c);
                                                    }
                                                    session_theme_changed((*c).session);
                                                    current_block = 16977559109335092698;
                                                }
                                                _ => {
                                                    current_block = 1538046216550696469;
                                                }
                                            }
                                            match current_block {
                                                16977559109335092698 => {}
                                                5025795842197473417 => {}
                                                _ => {
                                                    match tty_keys_palette(
                                                        tty,
                                                        buf,
                                                        len,
                                                        &raw mut size,
                                                    ) {
                                                        0 => {
                                                            key = KEYC_UNKNOWN
                                                                as ::core::ffi::c_ulong
                                                                as key_code;
                                                            current_block = 5025795842197473417;
                                                        }
                                                        -1 => {
                                                            current_block = 1836292691772056875;
                                                        }
                                                        1 => {
                                                            current_block = 16977559109335092698;
                                                        }
                                                        _ => {
                                                            current_block = 1836292691772056875;
                                                        }
                                                    }
                                                    match current_block {
                                                        5025795842197473417 => {}
                                                        16977559109335092698 => {}
                                                        _ => {
                                                            match tty_keys_mouse(
                                                                tty,
                                                                buf,
                                                                len,
                                                                &raw mut size,
                                                                &raw mut m,
                                                            ) {
                                                                0 => {
                                                                    key = KEYC_MOUSE
                                                                        as ::core::ffi::c_ulong
                                                                        as key_code;
                                                                    current_block =
                                                                        5025795842197473417;
                                                                }
                                                                -1 => {
                                                                    current_block =
                                                                        17784502470059252271;
                                                                }
                                                                -2 => {
                                                                    key = KEYC_MOUSE
                                                                        as ::core::ffi::c_ulong
                                                                        as key_code;
                                                                    log_debug(
                                                                        b"%s: discard key %.*s %#llx\0" as *const u8
                                                                            as *const ::core::ffi::c_char,
                                                                        ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                                                                        size as ::core::ffi::c_int,
                                                                        buf,
                                                                        key,
                                                                    );
                                                                    evbuffer_drain(
                                                                        (*tty).in_0,
                                                                        size,
                                                                    );
                                                                    return 1 as ::core::ffi::c_int;
                                                                }
                                                                1 => {
                                                                    current_block =
                                                                        16977559109335092698;
                                                                }
                                                                _ => {
                                                                    current_block =
                                                                        17784502470059252271;
                                                                }
                                                            }
                                                            match current_block {
                                                                5025795842197473417 => {}
                                                                16977559109335092698 => {}
                                                                _ => {
                                                                    match tty_keys_extended_key(
                                                                        tty,
                                                                        buf,
                                                                        len,
                                                                        &raw mut size,
                                                                        &raw mut key,
                                                                    ) {
                                                                        0 => {
                                                                            current_block =
                                                                                5025795842197473417;
                                                                        }
                                                                        -1 => {
                                                                            current_block =
                                                                                3938820862080741272;
                                                                        }
                                                                        1 => {
                                                                            current_block = 16977559109335092698;
                                                                        }
                                                                        _ => {
                                                                            current_block =
                                                                                3938820862080741272;
                                                                        }
                                                                    }
                                                                    match current_block {
                                                                        5025795842197473417 => {}
                                                                        16977559109335092698 => {}
                                                                        _ => {
                                                                            match tty_keys_winsz(
                                                                                tty,
                                                                                buf,
                                                                                len,
                                                                                &raw mut size,
                                                                            ) {
                                                                                0 => {
                                                                                    current_block = 1414802762261447502;
                                                                                    match current_block {
                                                                                        14661562966503102838 => {
                                                                                            current_block = 16977559109335092698;
                                                                                        }
                                                                                        _ => {
                                                                                            key = KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
                                                                                            current_block = 5025795842197473417;
                                                                                        }
                                                                                    }
                                                                                }
                                                                                1 => {
                                                                                    current_block = 14661562966503102838;
                                                                                    match current_block {
                                                                                        14661562966503102838 => {
                                                                                            current_block = 16977559109335092698;
                                                                                        }
                                                                                        _ => {
                                                                                            key = KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
                                                                                            current_block = 5025795842197473417;
                                                                                        }
                                                                                    }
                                                                                }
                                                                                -1 | _ => {
                                                                                    current_block = 12077302897653652224;
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    loop {
        match current_block {
            12077302897653652224 => {
                n = tty_keys_next1(tty, buf, len, &raw mut key, &raw mut size, expired);
                if n == 0 as ::core::ffi::c_int {
                    current_block = 5025795842197473417;
                    continue;
                }
                if n == 1 as ::core::ffi::c_int {
                    current_block = 16977559109335092698;
                    continue;
                }
                if *buf as ::core::ffi::c_int == '\u{1b}' as i32 && len > 1 as size_t {
                    n = tty_keys_next1(
                        tty,
                        buf.offset(1 as ::core::ffi::c_int as isize),
                        len.wrapping_sub(1 as size_t),
                        &raw mut key,
                        &raw mut size,
                        expired,
                    );
                    if n == 0 as ::core::ffi::c_int {
                        if key as ::core::ffi::c_ulonglong & KEYC_IMPLIED_META != 0 {
                            key = '\u{1b}' as i32 as key_code;
                            size = 1 as size_t;
                            current_block = 5025795842197473417;
                            continue;
                        } else {
                            key |= KEYC_META;
                            size = size.wrapping_add(1);
                            current_block = 5025795842197473417;
                            continue;
                        }
                    } else if n == 1 as ::core::ffi::c_int {
                        current_block = 16977559109335092698;
                        continue;
                    }
                }
                if *buf as ::core::ffi::c_int == '\u{1b}' as i32 && len >= 2 as size_t {
                    key = (*buf.offset(1 as ::core::ffi::c_int as isize) as u_char
                        as ::core::ffi::c_ulonglong
                        | KEYC_META) as key_code;
                    size = 2 as size_t;
                } else {
                    key = *buf.offset(0 as ::core::ffi::c_int as isize) as u_char as key_code;
                    size = 1 as size_t;
                }
                if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                    == C0_NUL as ::core::ffi::c_int as ::core::ffi::c_ulonglong
                {
                    key = (' ' as i32 as ::core::ffi::c_ulonglong
                        | KEYC_CTRL
                        | key as ::core::ffi::c_ulonglong & KEYC_META)
                        as key_code;
                }
                bspace = (*tty).tio.c_cc[VERASE as usize];
                if bspace as ::core::ffi::c_int != _POSIX_VDISABLE {
                    if key == bspace as key_code {
                        log_debug(
                            b"%s: key %#llx is BSpace\0" as *const u8 as *const ::core::ffi::c_char,
                            ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                            key,
                        );
                        key = KEYC_BSPACE as ::core::ffi::c_ulong as key_code;
                    }
                    if key == bspace as ::core::ffi::c_ulonglong | KEYC_META {
                        log_debug(
                            b"%s: key %#llx is M-BSpace\0" as *const u8
                                as *const ::core::ffi::c_char,
                            ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                            key,
                        );
                        key = (KEYC_BSPACE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                            | KEYC_META) as key_code;
                    }
                }
                onlykey = (key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY) as key_code;
                if onlykey < 0x20 as key_code
                    && onlykey != C0_HT as ::core::ffi::c_int as key_code
                    && onlykey != C0_CR as ::core::ffi::c_int as key_code
                    && onlykey != C0_ESC as ::core::ffi::c_int as key_code
                {
                    onlykey |= 0x40 as key_code;
                    if onlykey >= 'A' as i32 as key_code && onlykey <= 'Z' as i32 as key_code {
                        onlykey |= 0x20 as key_code;
                    }
                    key = (onlykey as ::core::ffi::c_ulonglong
                        | KEYC_CTRL
                        | key as ::core::ffi::c_ulonglong & KEYC_META)
                        as key_code;
                }
                current_block = 5025795842197473417;
            }
            5025795842197473417 => {
                log_debug(
                    b"%s: complete key %.*s %#llx\0" as *const u8 as *const ::core::ffi::c_char,
                    ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                    size as ::core::ffi::c_int,
                    buf,
                    key,
                );
                if event_initialized(&(*tty).key_timer) != 0 {
                    event_del(&raw mut (*tty).key_timer);
                }
                (*tty).flags &= !TTY_TIMER;
                if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                    == KEYC_PASTE_START as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                {
                    (*tty).flags |= TTY_BRACKETPASTE;
                } else if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                    == KEYC_PASTE_END as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                {
                    (*tty).flags &= !TTY_BRACKETPASTE;
                }
                if key == KEYC_FOCUS_OUT as ::core::ffi::c_ulong as key_code {
                    (*c).flags &= !CLIENT_FOCUSED as uint64_t;
                    window_update_focus((*(*(*c).session).curw).window);
                    events_fire_client(
                        b"client-focus-out\0" as *const u8 as *const ::core::ffi::c_char,
                        c,
                    );
                } else if key == KEYC_FOCUS_IN as ::core::ffi::c_ulong as key_code {
                    (*c).flags |= CLIENT_FOCUSED as uint64_t;
                    events_fire_client(
                        b"client-focus-in\0" as *const u8 as *const ::core::ffi::c_char,
                        c,
                    );
                    window_update_focus((*(*(*c).session).curw).window);
                }
                if key != KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code {
                    let bytes = if size == 0 {
                        Vec::new()
                    } else {
                        ::core::slice::from_raw_parts(buf.cast::<u8>(), size).to_vec()
                    };
                    let event = key_event::new(key, m, Some(bytes));
                    server_client_handle_key(c, event);
                }
                evbuffer_drain((*tty).in_0, size);
                return 1 as ::core::ffi::c_int;
            }
            _ => {
                log_debug(
                    b"%s: partial key %.*s\0" as *const u8 as *const ::core::ffi::c_char,
                    ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                    len as ::core::ffi::c_int,
                    buf,
                );
                if (*tty).flags & TTY_TIMER != 0 {
                    if event_initialized(&(*tty).key_timer) != 0
                        && event_pending(
                            &raw mut (*tty).key_timer,
                            EV_TIMEOUT as ::core::ffi::c_short,
                            ::core::ptr::null_mut::<timeval>(),
                        ) == 0
                    {
                        expired = 1 as ::core::ffi::c_int;
                        current_block = 12077302897653652224;
                    } else {
                        return 0 as ::core::ffi::c_int;
                    }
                } else {
                    delay = options_get_number(
                        global_options,
                        b"escape-time\0" as *const u8 as *const ::core::ffi::c_char,
                    ) as ::core::ffi::c_int;
                    if delay == 0 as ::core::ffi::c_int {
                        delay = 1 as ::core::ffi::c_int;
                    }
                    let partial_paste_end = match match_bracketed_paste_boundary(
                        ::core::slice::from_raw_parts(buf as *const u8, len),
                    ) {
                        BracketedPasteBoundaryMatch::Incomplete { boundary, consumed } => {
                            consumed != 0 && boundary != Some(BracketedPasteBoundary::Start)
                        }
                        BracketedPasteBoundaryMatch::Match { .. }
                        | BracketedPasteBoundaryMatch::NoMatch { .. } => false,
                    };
                    if (*tty).flags & TTY_BRACKETPASTE != 0 && partial_paste_end {
                        log_debug(
                            b"%s: increasing delay (partial paste end)\0" as *const u8
                                as *const ::core::ffi::c_char,
                            ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                        );
                        if delay < 500 as ::core::ffi::c_int {
                            delay = 500 as ::core::ffi::c_int;
                        }
                    }
                    if (*tty).flags & (TTY_WAITFG | TTY_WAITBG) != 0
                        || (*tty).flags & (TTY_OSC52QUERY | TTY_WINSIZEQUERY) != 0
                        || (*tty).flags & TTY_ALL_REQUEST_FLAGS != TTY_ALL_REQUEST_FLAGS
                        || input_client_has_requests(c)
                    {
                        log_debug(
                            b"%s: increasing delay (active query)\0" as *const u8
                                as *const ::core::ffi::c_char,
                            ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                        );
                        if delay < 500 as ::core::ffi::c_int {
                            delay = 500 as ::core::ffi::c_int;
                        }
                    }
                    tv.tv_sec = (delay / 1000 as ::core::ffi::c_int) as __time_t;
                    tv.tv_usec = ((delay % 1000 as ::core::ffi::c_int) as ::core::ffi::c_long
                        * 1000 as ::core::ffi::c_long)
                        as __suseconds_t;
                    if event_initialized(&(*tty).key_timer) != 0 {
                        event_del(&raw mut (*tty).key_timer);
                    }
                    event_set(
                        &raw mut (*tty).key_timer,
                        -(1 as ::core::ffi::c_int),
                        0 as ::core::ffi::c_short,
                        Some(
                            tty_keys_callback
                                as unsafe extern "C" fn(
                                    ::core::ffi::c_int,
                                    ::core::ffi::c_short,
                                    *mut ::core::ffi::c_void,
                                ) -> (),
                        ),
                        tty as *mut ::core::ffi::c_void,
                    );
                    event_add(&raw mut (*tty).key_timer, &raw mut tv);
                    (*tty).flags |= TTY_TIMER;
                    return 0 as ::core::ffi::c_int;
                }
            }
        }
    }
}
unsafe extern "C" fn tty_keys_callback(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut tty: *mut tty = data as *mut tty;
    if (*tty).flags & TTY_TIMER != 0 {
        while tty_keys_next(tty) != 0 {}
    }
}
unsafe extern "C" fn tty_keys_extended_key(
    mut tty: *mut tty,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut size: *mut size_t,
    mut key: *mut key_code,
) -> ::core::ffi::c_int {
    let mut c: *mut client = (*tty).client;
    let mut end: size_t = 0;
    let mut number: u_int = 0;
    let mut modifiers: u_int = 0;
    let mut tmp: [::core::ffi::c_char; 64] = [0; 64];
    let mut bspace: cc_t = 0;
    let mut nkey: key_code = 0;
    let mut onlykey: key_code = 0;
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut uc: utf8_char = 0;
    *size = 0 as size_t;
    if *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\u{1b}' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 1 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '[' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 2 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    end = 2 as size_t;
    while end < len && end != ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as usize {
        if *buf.offset(end as isize) as ::core::ffi::c_int == '~' as i32 {
            break;
        }
        if *(*__ctype_b_loc())
            .offset(*buf.offset(end as isize) as u_char as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            == 0
            && *buf.offset(end as isize) as ::core::ffi::c_int != ';' as i32
        {
            break;
        }
        end = end.wrapping_add(1);
    }
    if end == len {
        return 1 as ::core::ffi::c_int;
    }
    if end == ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as usize
        || *buf.offset(end as isize) as ::core::ffi::c_int != '~' as i32
            && *buf.offset(end as isize) as ::core::ffi::c_int != 'u' as i32
    {
        return -(1 as ::core::ffi::c_int);
    }
    memcpy(
        &raw mut tmp as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        buf.offset(2 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        end.wrapping_sub(2 as size_t),
    );
    tmp[end.wrapping_sub(2 as size_t) as usize] = '\0' as i32 as ::core::ffi::c_char;
    if *buf.offset(end as isize) as ::core::ffi::c_int == '~' as i32 {
        if sscanf(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"27;%u;%u\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut modifiers,
            &raw mut number,
        ) != 2 as ::core::ffi::c_int
        {
            return -(1 as ::core::ffi::c_int);
        }
    } else if sscanf(
        &raw mut tmp as *mut ::core::ffi::c_char,
        b"%u;%u\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut number,
        &raw mut modifiers,
    ) != 2 as ::core::ffi::c_int
    {
        return -(1 as ::core::ffi::c_int);
    }
    *size = end.wrapping_add(1 as size_t);
    bspace = (*tty).tio.c_cc[VERASE as usize];
    if bspace as ::core::ffi::c_int != _POSIX_VDISABLE && number == bspace as u_int {
        nkey = KEYC_BSPACE as ::core::ffi::c_ulong as key_code;
    } else {
        nkey = number as key_code;
    }
    if nkey != KEYC_BSPACE as ::core::ffi::c_ulong as key_code
        && nkey & !(0x7f as ::core::ffi::c_int) as key_code != 0
    {
        if utf8_fromwc(nkey as wchar_t, &raw mut ud) as ::core::ffi::c_uint
            == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
            && utf8_from_data(&raw mut ud, &raw mut uc) as ::core::ffi::c_uint
                == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            nkey = uc as key_code;
        } else {
            return -(1 as ::core::ffi::c_int);
        }
    }
    if modifiers > 0 as u_int {
        modifiers = modifiers.wrapping_sub(1);
        if modifiers & 1 as u_int != 0 {
            nkey |= KEYC_SHIFT;
        }
        if modifiers & 2 as u_int != 0 {
            nkey |= KEYC_META | KEYC_IMPLIED_META;
        }
        if modifiers & 4 as u_int != 0 {
            nkey |= KEYC_CTRL;
        }
        if modifiers & 8 as u_int != 0 {
            nkey |= KEYC_META | KEYC_IMPLIED_META;
        }
    }
    if nkey as ::core::ffi::c_ulonglong & KEYC_MASK_KEY == '\t' as i32 as ::core::ffi::c_ulonglong
        && nkey as ::core::ffi::c_ulonglong & KEYC_SHIFT != 0
    {
        nkey = (KEYC_BTAB as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
            | nkey as ::core::ffi::c_ulonglong & !KEYC_MASK_KEY & !KEYC_SHIFT)
            as key_code;
    }
    onlykey = (nkey as ::core::ffi::c_ulonglong & KEYC_MASK_KEY) as key_code;
    if (onlykey > 0x20 as key_code && onlykey < 0x7f as key_code
        || nkey as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            == (KEYC_TYPE_UNICODE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && nkey as ::core::ffi::c_ulonglong & KEYC_MASK_KEY > 0x7f as ::core::ffi::c_ulonglong)
        && nkey as ::core::ffi::c_ulonglong & KEYC_MASK_MODIFIERS == KEYC_SHIFT
    {
        nkey &= !KEYC_SHIFT;
    }
    if log_get_level() != 0 as ::core::ffi::c_int {
        let key_string = key_string_format(nkey, true);
        log_debug(
            b"%s: extended key %.*s is %llx (%s)\0" as *const u8 as *const ::core::ffi::c_char,
            ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            *size as ::core::ffi::c_int,
            buf,
            nkey,
            key_string.as_ptr(),
        );
    }
    *key = nkey;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn tty_keys_mouse(
    mut tty: *mut tty,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut size: *mut size_t,
    mut m: *mut mouse_event,
) -> ::core::ffi::c_int {
    let mut c: *mut client = (*tty).client;
    let mut i: u_int = 0;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut b: u_int = 0;
    let mut sgr_b: u_int = 0;
    let mut sgr_type: u_char = 0;
    let mut ch: u_char = 0;
    *size = 0 as size_t;
    sgr_b = 0 as u_int;
    b = sgr_b;
    y = b;
    x = y;
    sgr_type = ' ' as i32 as u_char;
    if *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\u{1b}' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 1 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '[' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 2 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'M' as i32 {
        *size = 3 as size_t;
        i = 0 as u_int;
        while i < 3 as u_int {
            if len <= *size {
                return 1 as ::core::ffi::c_int;
            }
            let fresh0 = *size;
            *size = (*size).wrapping_add(1);
            ch = *buf.offset(fresh0 as isize) as u_char;
            if i == 0 as u_int {
                b = ch as u_int;
            } else if i == 1 as u_int {
                x = ch as u_int;
            } else {
                y = ch as u_int;
            }
            i = i.wrapping_add(1);
        }
        log_debug(
            b"%s: mouse input: %.*s\0" as *const u8 as *const ::core::ffi::c_char,
            ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            *size as ::core::ffi::c_int,
            buf,
        );
        if b < MOUSE_PARAM_BTN_OFF as u_int
            || x < MOUSE_PARAM_POS_OFF as u_int
            || y < MOUSE_PARAM_POS_OFF as u_int
        {
            return -(2 as ::core::ffi::c_int);
        }
        b = b.wrapping_sub(MOUSE_PARAM_BTN_OFF as u_int);
        x = x.wrapping_sub(MOUSE_PARAM_POS_OFF as u_int);
        y = y.wrapping_sub(MOUSE_PARAM_POS_OFF as u_int);
    } else if *buf.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '<' as i32 {
        *size = 3 as size_t;
        loop {
            if len <= *size {
                return 1 as ::core::ffi::c_int;
            }
            let fresh1 = *size;
            *size = (*size).wrapping_add(1);
            ch = *buf.offset(fresh1 as isize) as u_char;
            if ch as ::core::ffi::c_int == ';' as i32 {
                break;
            }
            if (ch as ::core::ffi::c_int) < '0' as i32 || ch as ::core::ffi::c_int > '9' as i32 {
                return -(1 as ::core::ffi::c_int);
            }
            sgr_b = (10 as u_int)
                .wrapping_mul(sgr_b)
                .wrapping_add((ch as ::core::ffi::c_int - '0' as i32) as u_int);
        }
        loop {
            if len <= *size {
                return 1 as ::core::ffi::c_int;
            }
            let fresh2 = *size;
            *size = (*size).wrapping_add(1);
            ch = *buf.offset(fresh2 as isize) as u_char;
            if ch as ::core::ffi::c_int == ';' as i32 {
                break;
            }
            if (ch as ::core::ffi::c_int) < '0' as i32 || ch as ::core::ffi::c_int > '9' as i32 {
                return -(1 as ::core::ffi::c_int);
            }
            x = (10 as u_int)
                .wrapping_mul(x)
                .wrapping_add((ch as ::core::ffi::c_int - '0' as i32) as u_int);
        }
        loop {
            if len <= *size {
                return 1 as ::core::ffi::c_int;
            }
            let fresh3 = *size;
            *size = (*size).wrapping_add(1);
            ch = *buf.offset(fresh3 as isize) as u_char;
            if ch as ::core::ffi::c_int == 'M' as i32 || ch as ::core::ffi::c_int == 'm' as i32 {
                break;
            }
            if (ch as ::core::ffi::c_int) < '0' as i32 || ch as ::core::ffi::c_int > '9' as i32 {
                return -(1 as ::core::ffi::c_int);
            }
            y = (10 as u_int)
                .wrapping_mul(y)
                .wrapping_add((ch as ::core::ffi::c_int - '0' as i32) as u_int);
        }
        log_debug(
            b"%s: mouse input (SGR): %.*s\0" as *const u8 as *const ::core::ffi::c_char,
            ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            *size as ::core::ffi::c_int,
            buf,
        );
        if x < 1 as u_int || y < 1 as u_int {
            return -(2 as ::core::ffi::c_int);
        }
        x = x.wrapping_sub(1);
        y = y.wrapping_sub(1);
        b = sgr_b;
        sgr_type = ch;
        if sgr_type as ::core::ffi::c_int == 'm' as i32 {
            b = 3 as u_int;
        }
        if sgr_type as ::core::ffi::c_int == 'm' as i32
            && (sgr_b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_UP as u_int
                || sgr_b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_DOWN as u_int)
        {
            return -(2 as ::core::ffi::c_int);
        }
    } else {
        return -(1 as ::core::ffi::c_int);
    }
    (*m).lx = (*tty).mouse_last_x;
    (*m).x = x;
    (*m).ly = (*tty).mouse_last_y;
    (*m).y = y;
    (*m).lb = (*tty).mouse_last_b;
    (*m).b = b;
    (*m).sgr_type = sgr_type as u_int;
    (*m).sgr_b = sgr_b;
    (*tty).mouse_last_x = x;
    (*tty).mouse_last_y = y;
    (*tty).mouse_last_b = b;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn tty_keys_clipboard(
    mut tty: *mut tty,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut size: *mut size_t,
) -> ::core::ffi::c_int {
    let mut c: *mut client = (*tty).client;
    let mut end: size_t = 0;
    let mut terminator: size_t = 0 as size_t;
    let mut needed: size_t = 0;
    let mut clip: ::core::ffi::c_char = 0 as ::core::ffi::c_char;
    let mut outlen: ::core::ffi::c_int = 0;
    let mut cd: input_request_clipboard_data = input_request_clipboard_data {
        buf: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        len: 0,
        clip: 0,
    };
    *size = 0 as size_t;
    if *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\u{1b}' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 1 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != ']' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 2 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '5' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 3 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '2' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 4 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != ';' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 5 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    end = 5 as size_t;
    while end < len {
        if *buf.offset(end as isize) as ::core::ffi::c_int == '\u{7}' as i32 {
            terminator = 1 as size_t;
            break;
        } else if end > 5 as size_t
            && *buf.offset(end.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                == '\u{1b}' as i32
            && *buf.offset(end as isize) as ::core::ffi::c_int == '\\' as i32
        {
            terminator = 2 as size_t;
            break;
        } else {
            end = end.wrapping_add(1);
        }
    }
    if end == len {
        return 1 as ::core::ffi::c_int;
    }
    *size = end.wrapping_add(1 as size_t);
    buf = buf.offset(5 as ::core::ffi::c_int as isize);
    end = end.wrapping_sub(5 as size_t);
    end = end.wrapping_sub(terminator.wrapping_sub(1 as size_t));
    if end >= 2 as size_t
        && *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != ';' as i32
        && *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == ';' as i32
    {
        clip = *buf.offset(0 as ::core::ffi::c_int as isize);
    }
    while end != 0 as size_t && *buf as ::core::ffi::c_int != ';' as i32 {
        buf = buf.offset(1);
        end = end.wrapping_sub(1);
    }
    if end == 0 as size_t || end == 1 as size_t {
        return 0 as ::core::ffi::c_int;
    }
    buf = buf.offset(1);
    end = end.wrapping_sub(1);
    let mut copy = ::core::slice::from_raw_parts(buf.cast::<u8>(), end).to_vec();
    copy.push(0);
    needed = end
        .wrapping_add(3 as size_t)
        .wrapping_div(4 as size_t)
        .wrapping_mul(3 as size_t);
    if needed == 0 as size_t {
        return 0 as ::core::ffi::c_int;
    }
    let mut out = Vec::<u8>::with_capacity(needed);
    outlen = __b64_pton(
        copy.as_ptr().cast::<::core::ffi::c_char>(),
        out.as_mut_ptr(),
        needed,
    );
    if outlen == -(1 as ::core::ffi::c_int) {
        return 0 as ::core::ffi::c_int;
    }
    out.set_len(outlen as usize);
    drop(copy);
    log_debug(
        b"%s: %.*s\0" as *const u8 as *const ::core::ffi::c_char,
        b"tty_keys_clipboard\0" as *const u8 as *const ::core::ffi::c_char,
        outlen,
        out.as_ptr().cast::<::core::ffi::c_char>(),
    );
    cd.buf = out.as_mut_ptr().cast();
    cd.len = outlen as size_t;
    cd.clip = clip;
    input_request_reply(
        c,
        INPUT_REQUEST_CLIPBOARD,
        &raw mut cd as *mut ::core::ffi::c_void,
    );
    if (*tty).flags & TTY_OSC52QUERY != 0 {
        paste_add_owned(None, out.into_boxed_slice());
        event_del(&raw mut (*tty).clipboard_timer);
        (*tty).flags &= !TTY_OSC52QUERY;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn tty_keys_device_attributes(
    mut tty: *mut tty,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut size: *mut size_t,
) -> ::core::ffi::c_int {
    let mut c: *mut client = (*tty).client;
    let mut i: u_int = 0;
    let mut n: u_int = 0 as u_int;
    let mut tmp: [::core::ffi::c_char; 128] = [0; 128];
    let mut endptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: [::core::ffi::c_char; 32] = [
        0 as ::core::ffi::c_int as ::core::ffi::c_char,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ];
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    *size = 0 as size_t;
    if (*tty).flags & TTY_HAVEDA != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    if *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\u{1b}' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 1 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '[' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 2 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '?' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 3 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    i = 0 as u_int;
    while (i as usize) < ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as usize {
        if (3 as u_int).wrapping_add(i) as size_t == len {
            return 1 as ::core::ffi::c_int;
        }
        if *buf.offset((3 as u_int).wrapping_add(i) as isize) as ::core::ffi::c_int >= 'a' as i32
            && *buf.offset((3 as u_int).wrapping_add(i) as isize) as ::core::ffi::c_int
                <= 'z' as i32
        {
            break;
        }
        tmp[i as usize] = *buf.offset((3 as u_int).wrapping_add(i) as isize);
        i = i.wrapping_add(1);
    }
    if i as usize == ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as usize {
        return -(1 as ::core::ffi::c_int);
    }
    if *buf.offset((3 as u_int).wrapping_add(i) as isize) as ::core::ffi::c_int != 'c' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    tmp[i as usize] = '\0' as i32 as ::core::ffi::c_char;
    *size = (4 as u_int).wrapping_add(i) as size_t;
    cp = &raw mut tmp as *mut ::core::ffi::c_char;
    loop {
        next = strsep(
            &raw mut cp,
            b";\0" as *const u8 as *const ::core::ffi::c_char,
        );
        if next.is_null() {
            break;
        }
        p[n as usize] =
            strtoul(next, &raw mut endptr, 10 as ::core::ffi::c_int) as ::core::ffi::c_char;
        if *endptr as ::core::ffi::c_int != '\0' as i32 {
            p[n as usize] = 0 as ::core::ffi::c_char;
        }
        n = n.wrapping_add(1);
        if n as usize
            == (::core::mem::size_of::<[::core::ffi::c_char; 32]>() as usize)
                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>() as usize)
        {
            break;
        }
    }
    match p[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int {
        61..=65 => {
            i = 1 as u_int;
            while i < n {
                log_debug(
                    b"%s: DA feature: %d\0" as *const u8 as *const ::core::ffi::c_char,
                    ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                    p[i as usize] as ::core::ffi::c_int,
                );
                if p[i as usize] as ::core::ffi::c_int == 4 as ::core::ffi::c_int {
                    tty_parse_client_features(
                        c,
                        b"sixel\0" as *const u8 as *const ::core::ffi::c_char,
                        b",\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
                if p[i as usize] as ::core::ffi::c_int == 21 as ::core::ffi::c_int {
                    tty_parse_client_features(
                        c,
                        b"margins\0" as *const u8 as *const ::core::ffi::c_char,
                        b",\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
                if p[i as usize] as ::core::ffi::c_int == 28 as ::core::ffi::c_int {
                    tty_parse_client_features(
                        c,
                        b"rectfill\0" as *const u8 as *const ::core::ffi::c_char,
                        b",\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
                if p[i as usize] as ::core::ffi::c_int == 52 as ::core::ffi::c_int {
                    tty_parse_client_features(
                        c,
                        b"clipboard\0" as *const u8 as *const ::core::ffi::c_char,
                        b",\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
                i = i.wrapping_add(1);
            }
        }
        _ => {}
    }
    log_debug(
        b"%s: received primary DA %.*s\0" as *const u8 as *const ::core::ffi::c_char,
        ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        *size as ::core::ffi::c_int,
        buf,
    );
    tty_update_features(tty);
    (*tty).flags |= TTY_HAVEDA;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn tty_keys_sync(
    mut tty: *mut tty,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut size: *mut size_t,
) -> ::core::ffi::c_int {
    let mut c: *mut client = (*tty).client;
    static mut prefix: [::core::ffi::c_char; 9] =
        unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"\x1B[?2026;\0") };
    let mut i: size_t = 0;
    let mut status: ::core::ffi::c_int = 0;
    *size = 0 as size_t;
    if (*tty).flags & TTY_HAVESYNC != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    i = 0 as size_t;
    while i < (::core::mem::size_of::<[::core::ffi::c_char; 9]>() as usize).wrapping_sub(1 as usize)
    {
        if i == len {
            return 1 as ::core::ffi::c_int;
        }
        if *buf.offset(i as isize) as ::core::ffi::c_int != prefix[i as usize] as ::core::ffi::c_int
        {
            return -(1 as ::core::ffi::c_int);
        }
        i = i.wrapping_add(1);
    }
    if i == len {
        return 1 as ::core::ffi::c_int;
    }
    if (*buf.offset(i as isize) as ::core::ffi::c_int) < '0' as i32
        || *buf.offset(i as isize) as ::core::ffi::c_int > '4' as i32
    {
        return -(1 as ::core::ffi::c_int);
    }
    let fresh4 = i;
    i = i.wrapping_add(1);
    status = *buf.offset(fresh4 as isize) as ::core::ffi::c_int - '0' as i32;
    if i == len {
        return 1 as ::core::ffi::c_int;
    }
    let fresh5 = i;
    i = i.wrapping_add(1);
    if *buf.offset(fresh5 as isize) as ::core::ffi::c_int != '$' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if i == len {
        return 1 as ::core::ffi::c_int;
    }
    let fresh6 = i;
    i = i.wrapping_add(1);
    if *buf.offset(fresh6 as isize) as ::core::ffi::c_int != 'y' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    *size = i;
    if status == 1 as ::core::ffi::c_int
        || status == 2 as ::core::ffi::c_int
        || status == 3 as ::core::ffi::c_int
    {
        tty_parse_client_features(
            c,
            b"sync\0" as *const u8 as *const ::core::ffi::c_char,
            b",\0" as *const u8 as *const ::core::ffi::c_char,
        );
        tty_update_features(tty);
    }
    log_debug(
        b"%s: received DECRPM %.*s\0" as *const u8 as *const ::core::ffi::c_char,
        ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        *size as ::core::ffi::c_int,
        buf,
    );
    (*tty).flags |= TTY_HAVESYNC;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn tty_keys_device_attributes2(
    mut tty: *mut tty,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut size: *mut size_t,
) -> ::core::ffi::c_int {
    let mut c: *mut client = (*tty).client;
    let mut i: u_int = 0;
    let mut n: u_int = 0 as u_int;
    let mut tmp: [::core::ffi::c_char; 128] = [0; 128];
    let mut endptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: [::core::ffi::c_char; 32] = [
        0 as ::core::ffi::c_int as ::core::ffi::c_char,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ];
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    *size = 0 as size_t;
    if (*tty).flags & TTY_HAVEDA2 != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    if *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\u{1b}' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 1 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '[' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 2 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '>' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 3 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    i = 0 as u_int;
    while (i as usize) < ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as usize {
        if (3 as u_int).wrapping_add(i) as size_t == len {
            return 1 as ::core::ffi::c_int;
        }
        if *buf.offset((3 as u_int).wrapping_add(i) as isize) as ::core::ffi::c_int >= 'a' as i32
            && *buf.offset((3 as u_int).wrapping_add(i) as isize) as ::core::ffi::c_int
                <= 'z' as i32
        {
            break;
        }
        tmp[i as usize] = *buf.offset((3 as u_int).wrapping_add(i) as isize);
        i = i.wrapping_add(1);
    }
    if i as usize == ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as usize {
        return -(1 as ::core::ffi::c_int);
    }
    if *buf.offset((3 as u_int).wrapping_add(i) as isize) as ::core::ffi::c_int != 'c' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    tmp[i as usize] = '\0' as i32 as ::core::ffi::c_char;
    *size = (4 as u_int).wrapping_add(i) as size_t;
    cp = &raw mut tmp as *mut ::core::ffi::c_char;
    loop {
        next = strsep(
            &raw mut cp,
            b";\0" as *const u8 as *const ::core::ffi::c_char,
        );
        if next.is_null() {
            break;
        }
        p[n as usize] =
            strtoul(next, &raw mut endptr, 10 as ::core::ffi::c_int) as ::core::ffi::c_char;
        if *endptr as ::core::ffi::c_int != '\0' as i32 {
            p[n as usize] = 0 as ::core::ffi::c_char;
        }
        n = n.wrapping_add(1);
        if n as usize
            == (::core::mem::size_of::<[::core::ffi::c_char; 32]>() as usize)
                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>() as usize)
        {
            break;
        }
    }
    match p[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int {
        77 => {
            tty_default_features(
                c,
                b"mintty\0" as *const u8 as *const ::core::ffi::c_char,
                0 as u_int,
            );
        }
        84 => {
            tty_default_features(
                c,
                b"tmux\0" as *const u8 as *const ::core::ffi::c_char,
                0 as u_int,
            );
        }
        85 => {
            tty_default_features(
                c,
                b"rxvt-unicode\0" as *const u8 as *const ::core::ffi::c_char,
                0 as u_int,
            );
        }
        _ => {}
    }
    log_debug(
        b"%s: received secondary DA %.*s\0" as *const u8 as *const ::core::ffi::c_char,
        ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        *size as ::core::ffi::c_int,
        buf,
    );
    tty_update_features(tty);
    (*tty).flags |= TTY_HAVEDA2;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn tty_keys_extended_device_attributes(
    mut tty: *mut tty,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut size: *mut size_t,
) -> ::core::ffi::c_int {
    let mut c: *mut client = (*tty).client;
    let mut i: u_int = 0;
    let mut tmp: [::core::ffi::c_char; 128] = [0; 128];
    *size = 0 as size_t;
    if (*tty).flags & TTY_HAVEXDA != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    if *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\u{1b}' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 1 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != 'P' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 2 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '>' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 3 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '|' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 4 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[::core::ffi::c_char; 128]>() as usize).wrapping_sub(1 as usize)
    {
        if (4 as u_int).wrapping_add(i) as size_t == len {
            return 1 as ::core::ffi::c_int;
        }
        if *buf.offset((4 as u_int).wrapping_add(i).wrapping_sub(1 as u_int) as isize)
            as ::core::ffi::c_int
            == '\u{1b}' as i32
            && *buf.offset((4 as u_int).wrapping_add(i) as isize) as ::core::ffi::c_int
                == '\\' as i32
        {
            break;
        }
        tmp[i as usize] = *buf.offset((4 as u_int).wrapping_add(i) as isize);
        i = i.wrapping_add(1);
    }
    if i as usize
        == (::core::mem::size_of::<[::core::ffi::c_char; 128]>() as usize).wrapping_sub(1 as usize)
    {
        return -(1 as ::core::ffi::c_int);
    }
    *size = (5 as u_int).wrapping_add(i) as size_t;
    if i == 0 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    tmp[i.wrapping_sub(1 as u_int) as usize] = '\0' as i32 as ::core::ffi::c_char;
    if strncmp(
        &raw mut tmp as *mut ::core::ffi::c_char,
        b"iTerm2 \0" as *const u8 as *const ::core::ffi::c_char,
        7 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        tty_default_features(
            c,
            b"iTerm2\0" as *const u8 as *const ::core::ffi::c_char,
            0 as u_int,
        );
    } else if strncmp(
        &raw mut tmp as *mut ::core::ffi::c_char,
        b"tmux \0" as *const u8 as *const ::core::ffi::c_char,
        5 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        tty_default_features(
            c,
            b"tmux\0" as *const u8 as *const ::core::ffi::c_char,
            0 as u_int,
        );
    } else if strncmp(
        &raw mut tmp as *mut ::core::ffi::c_char,
        b"XTerm(\0" as *const u8 as *const ::core::ffi::c_char,
        6 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        tty_default_features(
            c,
            b"XTerm\0" as *const u8 as *const ::core::ffi::c_char,
            0 as u_int,
        );
    } else if strncmp(
        &raw mut tmp as *mut ::core::ffi::c_char,
        b"mintty \0" as *const u8 as *const ::core::ffi::c_char,
        7 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        tty_default_features(
            c,
            b"mintty\0" as *const u8 as *const ::core::ffi::c_char,
            0 as u_int,
        );
    } else if strncmp(
        &raw mut tmp as *mut ::core::ffi::c_char,
        b"foot(\0" as *const u8 as *const ::core::ffi::c_char,
        5 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        tty_default_features(
            c,
            b"foot\0" as *const u8 as *const ::core::ffi::c_char,
            0 as u_int,
        );
    } else if strncmp(
        &raw mut tmp as *mut ::core::ffi::c_char,
        b"WezTerm \0" as *const u8 as *const ::core::ffi::c_char,
        7 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        tty_default_features(
            c,
            b"WezTerm\0" as *const u8 as *const ::core::ffi::c_char,
            0 as u_int,
        );
    } else if strncmp(
        &raw mut tmp as *mut ::core::ffi::c_char,
        b"ghostty \0" as *const u8 as *const ::core::ffi::c_char,
        8 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        tty_default_features(
            c,
            b"ghostty\0" as *const u8 as *const ::core::ffi::c_char,
            0 as u_int,
        );
    } else if strncmp(
        &raw mut tmp as *mut ::core::ffi::c_char,
        b"Rio \0" as *const u8 as *const ::core::ffi::c_char,
        4 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        tty_default_features(
            c,
            b"Rio\0" as *const u8 as *const ::core::ffi::c_char,
            0 as u_int,
        );
    }
    log_debug(
        b"%s: received extended DA %.*s\0" as *const u8 as *const ::core::ffi::c_char,
        ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        *size as ::core::ffi::c_int,
        buf,
    );
    server_client_set_term_type(&mut *c, Some(CStr::from_ptr(tmp.as_ptr()).to_owned()));
    tty_update_features(tty);
    (*tty).flags |= TTY_HAVEXDA;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn tty_keys_colours(
    mut tty: *mut tty,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut size: *mut size_t,
    mut fg: *mut ::core::ffi::c_int,
    mut bg: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut c: *mut client = (*tty).client;
    let mut i: u_int = 0;
    let mut tmp: [::core::ffi::c_char; 128] = [0; 128];
    let mut n: ::core::ffi::c_int = 0;
    *size = 0 as size_t;
    if *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\u{1b}' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 1 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != ']' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 2 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '1' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 3 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '0' as i32
        && *buf.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '1' as i32
    {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 4 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != ';' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 5 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[::core::ffi::c_char; 128]>() as usize).wrapping_sub(1 as usize)
    {
        if (5 as u_int).wrapping_add(i) as size_t == len {
            return 1 as ::core::ffi::c_int;
        }
        if *buf.offset((5 as u_int).wrapping_add(i).wrapping_sub(1 as u_int) as isize)
            as ::core::ffi::c_int
            == '\u{1b}' as i32
            && *buf.offset((5 as u_int).wrapping_add(i) as isize) as ::core::ffi::c_int
                == '\\' as i32
        {
            break;
        }
        if *buf.offset((5 as u_int).wrapping_add(i) as isize) as ::core::ffi::c_int
            == '\u{7}' as i32
        {
            break;
        }
        tmp[i as usize] = *buf.offset((5 as u_int).wrapping_add(i) as isize);
        i = i.wrapping_add(1);
    }
    if i as usize
        == (::core::mem::size_of::<[::core::ffi::c_char; 128]>() as usize).wrapping_sub(1 as usize)
    {
        return -(1 as ::core::ffi::c_int);
    }
    *size = (6 as u_int).wrapping_add(i) as size_t;
    if i == 0 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    if tmp[i.wrapping_sub(1 as u_int) as usize] as ::core::ffi::c_int == '\u{1b}' as i32 {
        tmp[i.wrapping_sub(1 as u_int) as usize] = '\0' as i32 as ::core::ffi::c_char;
    } else {
        tmp[i as usize] = '\0' as i32 as ::core::ffi::c_char;
    }
    n = colour_parse_x11_logged(std::ffi::CStr::from_ptr(
        &raw mut tmp as *mut ::core::ffi::c_char,
    ))
    .unwrap_or(-1);
    if n != -(1 as ::core::ffi::c_int)
        && *buf.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '0' as i32
    {
        if !c.is_null() {
            log_debug(
                b"%s fg is %s\0" as *const u8 as *const ::core::ffi::c_char,
                ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                colour_format(n).as_ptr(),
            );
        } else {
            log_debug(
                b"fg is %s\0" as *const u8 as *const ::core::ffi::c_char,
                colour_format(n).as_ptr(),
            );
        }
        *fg = n;
        (*tty).flags &= !TTY_WAITFG;
    } else if n != -(1 as ::core::ffi::c_int) {
        if !c.is_null() {
            log_debug(
                b"%s bg is %s\0" as *const u8 as *const ::core::ffi::c_char,
                ((*c).name).as_ref().map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                colour_format(n).as_ptr(),
            );
        } else {
            log_debug(
                b"bg is %s\0" as *const u8 as *const ::core::ffi::c_char,
                colour_format(n).as_ptr(),
            );
        }
        *bg = n;
        (*tty).flags &= !TTY_WAITBG;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn tty_keys_palette(
    mut tty: *mut tty,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut size: *mut size_t,
) -> ::core::ffi::c_int {
    let mut c: *mut client = (*tty).client;
    let mut i: u_int = 0;
    let mut tmp: [::core::ffi::c_char; 128] = [0; 128];
    let mut endptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut idx: ::core::ffi::c_int = 0;
    let mut pd: input_request_palette_data = input_request_palette_data { idx: 0, c: 0 };
    *size = 0 as size_t;
    if *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\u{1b}' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 1 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != ']' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 2 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '4' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 3 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != ';' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 4 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[::core::ffi::c_char; 128]>() as usize).wrapping_sub(1 as usize)
    {
        if (4 as u_int).wrapping_add(i) as size_t == len {
            return 1 as ::core::ffi::c_int;
        }
        if *buf.offset((4 as u_int).wrapping_add(i).wrapping_sub(1 as u_int) as isize)
            as ::core::ffi::c_int
            == '\u{1b}' as i32
            && *buf.offset((4 as u_int).wrapping_add(i) as isize) as ::core::ffi::c_int
                == '\\' as i32
        {
            break;
        }
        if *buf.offset((4 as u_int).wrapping_add(i) as isize) as ::core::ffi::c_int
            == '\u{7}' as i32
        {
            break;
        }
        tmp[i as usize] = *buf.offset((4 as u_int).wrapping_add(i) as isize);
        i = i.wrapping_add(1);
    }
    if i as usize
        == (::core::mem::size_of::<[::core::ffi::c_char; 128]>() as usize).wrapping_sub(1 as usize)
    {
        return -(1 as ::core::ffi::c_int);
    }
    *size = (5 as u_int).wrapping_add(i) as size_t;
    if i == 0 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    if tmp[i.wrapping_sub(1 as u_int) as usize] as ::core::ffi::c_int == '\u{1b}' as i32 {
        tmp[i.wrapping_sub(1 as u_int) as usize] = '\0' as i32 as ::core::ffi::c_char;
    } else {
        tmp[i as usize] = '\0' as i32 as ::core::ffi::c_char;
    }
    idx = strtol(
        &raw mut tmp as *mut ::core::ffi::c_char,
        &raw mut endptr,
        10 as ::core::ffi::c_int,
    ) as ::core::ffi::c_int;
    if *endptr as ::core::ffi::c_int != ';' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if idx < 0 as ::core::ffi::c_int || idx > 255 as ::core::ffi::c_int {
        return -(1 as ::core::ffi::c_int);
    }
    pd.c = colour_parse_x11_logged(std::ffi::CStr::from_ptr(
        endptr.offset(1 as ::core::ffi::c_int as isize),
    ))
    .unwrap_or(-1);
    if pd.c == -(1 as ::core::ffi::c_int) {
        return 0 as ::core::ffi::c_int;
    }
    pd.idx = idx;
    input_request_reply(
        c,
        INPUT_REQUEST_PALETTE,
        &raw mut pd as *mut ::core::ffi::c_void,
    );
    return 0 as ::core::ffi::c_int;
}
