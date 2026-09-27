use crate::src::alerts::alerts_queue;
use crate::src::cmd::find::cmd_find_from_pane;
use crate::src::compat::strtonum::strtonum;
use crate::src::events::{events_fire, events_fire_pane};
use crate::src::events_payload::{
    event_payload_create, event_payload_set_int, event_payload_set_pane, event_payload_set_session,
    event_payload_set_string, event_payload_set_target, event_payload_set_time,
    event_payload_set_uint, event_payload_set_window,
};
use crate::src::ffi::libc::{
    memcpy, memset, strchr, strcmp, strlen, strncmp, strpbrk, strsep, strstr, strtol, time,
};
use crate::src::ffi::resolv::{__b64_ntop, __b64_pton};
use crate::src::format::bytes::format_message_with;
use crate::src::format::bytes::write_cstr;
use crate::src::format::bytes::xformat;
use crate::src::grid::{
    grid_cells_look_equal, grid_default_cell, grid_get_cell, grid_get_line_mut, grid_set_tab,
};
use crate::src::hyperlinks::hyperlinks_put;
use crate::src::log::{fatalx, log_byte, log_cstr, log_cstr_n, log_cstr_width, log_debug, log_hex};
use crate::src::options::{
    options_get_number, options_get_only, options_remove_or_default, options_set_number,
};
use crate::src::paste::{paste_add_owned, paste_buffer_data, paste_get_top};
use crate::src::reactor::{
    bufferevent_write, evbuffer_add, evbuffer_drain, evbuffer_get_length,
    evbuffer_new, event_add, event_del, event_set,
};
use crate::src::screen::{screen_clear_tabs, screen_has_tab, screen_set_tab};
use crate::src::screen::{
    screen_pop_title, screen_push_title, screen_set_cursor_colour, screen_set_cursor_style,
    screen_set_path, screen_set_progress_bar, screen_set_title,
};
use crate::src::screen_write::{
    screen_write_alignmenttest, screen_write_alternateoff, screen_write_alternateon,
    screen_write_backspace, screen_write_carriagereturn, screen_write_clearcharacter,
    screen_write_clearendofline, screen_write_clearendofscreen, screen_write_clearhistory,
    screen_write_clearline, screen_write_clearscreen, screen_write_clearstartofline,
    screen_write_clearstartofscreen, screen_write_collect_add, screen_write_collect_end,
    screen_write_cursordown, screen_write_cursorleft, screen_write_cursormove,
    screen_write_cursorright, screen_write_cursorup, screen_write_deletecharacter,
    screen_write_deleteline, screen_write_end_sync, screen_write_fullredraw,
    screen_write_insertcharacter, screen_write_insertline, screen_write_linefeed,
    screen_write_mode_clear, screen_write_mode_set, screen_write_rawstring, screen_write_reset,
    screen_write_reverseindex, screen_write_scrolldown, screen_write_scrollregion,
    screen_write_scrollup, screen_write_setselection, screen_write_start,
    screen_write_start_callback, screen_write_start_pane, screen_write_start_sync,
    screen_write_stop, screen_write_stop_sync,
};
use crate::src::server::clients;
use crate::src::server_fn::{server_redraw_window_borders, server_status_window};
use crate::src::session::session_has;
use crate::src::shared::events::event_payload;
use crate::src::shared::input::{input_request_clipboard_data, input_request_palette_data};
use crate::src::style::colour::{
    colour_force_rgb, colour_join_rgb, colour_palette_clear, colour_palette_get,
    colour_palette_set, colour_parse_x11_logged, colour_split_rgb,
};
use crate::src::text::utf8::{utf8_append, utf8_copy, utf8_isvalid, utf8_open, utf8_set};
use crate::src::tmux::{get_timer, getversion, global_options, global_w_options};
use crate::src::tty::{tty_default_colours, tty_putcode_ss, tty_puts, tty_set_selection};
use crate::src::window::{
    window_pane_get_bg, window_pane_get_fg, window_pane_get_fg_control_client,
    window_pane_get_new_data, window_pane_get_theme, window_pane_update_used_data, window_set_name,
    window_update_activity,
};
use std::collections::VecDeque;
use std::ffi::{CStr, CString};

use crate::src::shared::abi::*;
use crate::src::shared::client::client;
use crate::src::shared::client::CLIENT_UNATTACHEDFLAGS;
use crate::src::shared::colour::COLOUR_FLAG_256;
use crate::src::shared::colour::*;
use crate::src::shared::command::cmd_find_state;
use crate::src::shared::display::*;
use crate::src::shared::event::*;
use crate::src::shared::grid::*;
use crate::src::shared::hyperlinks::hyperlinks;
use crate::src::shared::input::input_request_type;
use crate::src::shared::input::{
    input_cell, input_ctx, input_end_type, input_param, input_request, input_state,
    input_transition,
};
use crate::src::shared::input::{
    INPUT_BUF_DEFAULT_SIZE, INPUT_REQUEST_CLIPBOARD, INPUT_REQUEST_PALETTE, INPUT_REQUEST_QUEUE,
};
use crate::src::shared::limits::INT_MAX;
use crate::src::shared::options::{options, options_entry};
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::{
    PANE_ACTIVITY, PANE_CHANGED, PANE_CMDRUNNING, PANE_STYLECHANGED, PANE_THEMECHANGED,
    PANE_UNSEENCHANGES,
};
use crate::src::shared::paste::paste_buffer;
use crate::src::shared::screen::{
    screen, ALL_MOUSE_MODES, EXTENDED_KEY_MODES, MODE_BRACKETPASTE, MODE_CRLF, MODE_CURSOR,
    MODE_CURSOR_BLINKING, MODE_CURSOR_BLINKING_SET, MODE_CURSOR_VERY_VISIBLE, MODE_FOCUSON,
    MODE_INSERT, MODE_KCURSOR, MODE_KEYS_EXTENDED, MODE_KEYS_EXTENDED_2, MODE_KKEYPAD,
    MODE_MOUSE_ALL, MODE_MOUSE_BUTTON, MODE_MOUSE_SGR, MODE_MOUSE_STANDARD, MODE_MOUSE_UTF8,
    MODE_ORIGIN, MODE_SYNC, MODE_THEME_UPDATES, MODE_WRAP,
};
use crate::src::shared::screen_write::{screen_write_ctx, screen_write_init_ctx_cb};
use crate::src::shared::session::session;
use crate::src::shared::tty::TTY_STARTED;
use crate::src::shared::tty::*;
use crate::src::shared::utf8::*;
use crate::src::shared::window::WINDOW_BELL;
use crate::src::shared::window::{window, winlink};

pub const INPUT_END_BEL: input_end_type = 1;
pub const INPUT_END_ST: input_end_type = 0;

impl input_ctx {
    // Only Rust constructs and accesses this record. Callbacks borrow its boxed,
    // stable address; no foreign consumer depends on the translated C layout.
    fn new() -> Self {
        Self {
            wp: ::core::ptr::null_mut(),
            event: ::core::ptr::null_mut(),
            ctx: screen_write_ctx {
                wp: ::core::ptr::null_mut(),
                s: ::core::ptr::null_mut(),
                flags: 0,
                init_ctx_cb: None,
                item: None,
                scrolled: 0,
                bg: 0,
            },
            palette: ::core::ptr::null_mut(),
            c: ::core::ptr::null_mut(),
            cell: Default::default(),
            old_cell: Default::default(),
            old_cx: 0,
            old_cy: 0,
            old_mode: 0,
            interm_buf: [0; 4],
            interm_len: 0,
            param_buf: [0; 64],
            param_len: 0,
            input_buf: vec![0; INPUT_BUF_START as usize],
            input_len: 0,
            input_end: INPUT_END_ST,
            param_list: std::array::from_fn(|_| input_param::Missing),
            param_list_len: 0,
            utf8data: Default::default(),
            utf8started: 0,
            ch: 0,
            last: Default::default(),
            state: ::core::ptr::null(),
            flags: 0,
            requests: VecDeque::new(),
            request_count: 0,
            request_timer: Default::default(),
            since_ground: evbuffer_new(),
            ground_timer: Default::default(),
        }
    }

    fn shrink_buffer(&mut self) {
        if self.input_buf.len() > INPUT_BUF_START as usize {
            self.input_buf.truncate(INPUT_BUF_START as usize);
            self.input_buf.shrink_to_fit();
        }
    }
}

unsafe fn input_clear_params(ictx: &mut input_ctx) {
    for param in &mut ictx.param_list[..ictx.param_list_len as usize] {
        *param = input_param::Missing;
    }
}

#[cfg(test)]
mod input_buffer_ownership_tests {
    use super::*;

    #[test]
    fn colon_parameter_survives_later_numeric_error_until_next_split() {
        unsafe {
            let ictx = Box::into_raw(Box::new(input_ctx::new()));
            let invalid = b"38:5:196;invalid\0";
            (&mut (*ictx).param_buf)[..invalid.len()].copy_from_slice(invalid);
            (*ictx).param_len = invalid.len() - 1;
            assert_eq!(input_split(ictx), -1);
            assert_eq!((*ictx).param_list_len, 1);
            assert_eq!(
                match &(*ictx).param_list[0] {
                    input_param::String(value) => value.as_c_str(),
                    _ => panic!("expected string parameter"),
                },
                c"38:5:196"
            );

            let next = b"48:5:25\0";
            (&mut (*ictx).param_buf)[..next.len()].copy_from_slice(next);
            (*ictx).param_len = next.len() - 1;
            assert_eq!(input_split(ictx), 0);
            assert_eq!((*ictx).param_list_len, 1);
            assert_eq!(
                match &(*ictx).param_list[0] {
                    input_param::String(value) => value.as_c_str(),
                    _ => panic!("expected string parameter"),
                },
                c"48:5:25"
            );
            input_clear_params(&mut *ictx);
            drop(Box::from_raw(ictx));
        }
    }

    #[test]
    fn parser_buffer_grows_preserves_bytes_and_shrinks() {
        unsafe {
            let ictx = Box::into_raw(Box::new(input_ctx::new()));
            assert_eq!((*ictx).input_buf.len(), INPUT_BUF_START as usize);
            for ch in (0..96).map(|i| if i == 17 { 0 } else { b'a' + (i % 26) }) {
                (*ictx).ch = ch as i32;
                input_input(ictx);
            }
            assert_eq!((*ictx).input_len, 96);
            assert_eq!((&(*ictx).input_buf)[17], 0);
            assert_eq!((&(*ictx).input_buf)[96], 0);
            assert_eq!((*ictx).input_buf.len(), 128);

            (*ictx).shrink_buffer();
            assert_eq!((*ictx).input_buf.len(), INPUT_BUF_START as usize);
            drop(Box::from_raw(ictx));
        }
    }
}

// input_ctx owns each boxed request. Boxes preserve addresses used by client
// observers as the deque grows; removing a request also drops its reply data.
impl input_request {
    fn new() -> Box<Self> {
        Box::new(Self {
            c: ::core::ptr::null_mut(),
            ictx: ::core::ptr::null_mut(),
            type_0: INPUT_REQUEST_PALETTE,
            t: 0,
            end: INPUT_END_ST,
            idx: 0,
            data: None,
        })
    }
}

unsafe fn input_ctx_requests<'a>(ictx: *mut input_ctx) -> &'a mut VecDeque<Box<input_request>> {
    &mut (*ictx).requests
}

unsafe fn input_client_requests<'a>(c: *mut client) -> &'a mut Vec<*mut input_request> {
    &mut (*c).input_requests
}

unsafe fn input_ctx_request_handles(ictx: *mut input_ctx) -> Vec<*mut input_request> {
    input_ctx_requests(ictx)
        .iter_mut()
        .map(|owner| &mut **owner as *mut input_request)
        .collect()
}

pub(crate) unsafe fn input_client_has_requests(c: *mut client) -> bool {
    !input_client_requests(c).is_empty()
}

#[cfg(test)]
mod input_request_ownership_tests {
    use super::*;

    #[test]
    fn queued_reply_survives_earlier_request_removal() {
        unsafe {
            let ictx = Box::into_raw(Box::new(input_ctx::new()));

            let mut c: client = client::empty();

            // Seed one pending nonqueue request without starting a timer.
            let mut pending_owner = input_request::new();
            let pending = &mut *pending_owner as *mut input_request;
            (*pending).ictx = ictx;
            (*pending).c = &mut c;
            (*pending).type_0 = INPUT_REQUEST_PALETTE;
            input_ctx_requests(ictx).push_back(pending_owner);
            input_client_requests(&mut c).push(pending);
            (*ictx).request_count = 1;

            input_reply(ictx, 1, |out| {
                out.write_all(b"reply:")?;
                write_cstr(out, b"\xff\xfe\0".as_ptr().cast::<::core::ffi::c_char>())
            });
            let queued = *input_ctx_request_handles(ictx).last().unwrap();
            assert_eq!((*queued).type_0, INPUT_REQUEST_QUEUE);
            assert_eq!(
                (*queued).data.as_ref().unwrap().to_bytes(),
                b"reply:\xff\xfe"
            );

            input_free_request(pending);
            assert_eq!(input_ctx_request_handles(ictx), vec![queued]);
            assert_eq!(input_client_requests(&mut c), &[]);
            assert_eq!(
                (*queued).data.as_ref().unwrap().to_bytes(),
                b"reply:\xff\xfe"
            );
            input_free_request(queued);
            assert!(input_ctx_requests(ictx).is_empty());
            assert_eq!((*ictx).request_count, 0);
            drop(Box::from_raw(ictx));
        }
    }

    #[test]
    fn matched_request_can_be_freed_before_later_queued_reply() {
        unsafe {
            let ictx = Box::into_raw(Box::new(input_ctx::new()));

            let mut c: client = client::empty();

            let mut pending_owner = input_request::new();
            let pending = &mut *pending_owner as *mut input_request;
            (*pending).ictx = ictx;
            (*pending).c = &mut c;
            (*pending).type_0 = INPUT_REQUEST_PALETTE;
            (*pending).idx = 7;
            input_ctx_requests(ictx).push_back(pending_owner);
            input_client_requests(&mut c).push(pending);
            (*ictx).request_count = 1;

            input_reply(ictx, 1, |out| out.write_all(b"queued"));
            let mut reply = input_request_palette_data { idx: 7, c: -1 };
            input_request_reply(&mut c, INPUT_REQUEST_PALETTE, (&raw mut reply).cast());

            assert!(input_ctx_requests(ictx).is_empty());
            assert!(input_client_requests(&mut c).is_empty());
            assert_eq!((*ictx).request_count, 0);
            drop(Box::from_raw(ictx));
        }
    }

    #[test]
    fn reply_without_pending_request_does_not_queue() {
        unsafe {
            let ictx = Box::into_raw(Box::new(input_ctx::new()));
            input_reply(ictx, 1, |out| out.write_all(b"\x1b[0n"));
            assert!(input_ctx_requests(ictx).is_empty());
            assert_eq!((*ictx).request_count, 0);
            drop(Box::from_raw(ictx));
        }
    }

    #[test]
    fn cancelling_client_drops_its_requests_from_the_input_owner() {
        unsafe {
            let ictx = Box::into_raw(Box::new(input_ctx::new()));

            let mut c: client = client::empty();

            for type_0 in [INPUT_REQUEST_PALETTE, INPUT_REQUEST_CLIPBOARD] {
                let mut owner = input_request::new();
                let ir = &mut *owner as *mut input_request;
                (*ir).ictx = ictx;
                (*ir).c = &mut c;
                (*ir).type_0 = type_0;
                input_ctx_requests(ictx).push_back(owner);
                input_client_requests(&mut c).push(ir);
                (*ictx).request_count += 1;
            }

            assert!(input_client_has_requests(&mut c));
            input_cancel_requests(&mut c);

            assert!(!input_client_has_requests(&mut c));
            assert!(input_ctx_requests(ictx).is_empty());
            assert_eq!((*ictx).request_count, 0);
            drop(Box::from_raw(ictx));
        }
    }
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct input_table_entry {
    pub ch: ::core::ffi::c_int,
    pub interm: &'static ::std::ffi::CStr,
    pub type_0: ::core::ffi::c_int,
}
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
unsafe fn input_table_find<'a>(
    table: &'a [input_table_entry],
    ictx: &input_ctx,
) -> Option<&'a input_table_entry> {
    let interm = CStr::from_ptr(ictx.interm_buf.as_ptr().cast());
    table
        .binary_search_by(|entry| {
            entry
                .ch
                .cmp(&ictx.ch)
                .then_with(|| entry.interm.to_bytes().cmp(interm.to_bytes()))
        })
        .ok()
        .map(|index| &table[index])
}
static mut input_state_ground: input_state = {
    input_state {
        name: c"ground",
        enter: Some(input_ground),
        exit: None,
        transitions: &raw const input_state_ground_table as *const input_transition,
    }
};
static mut input_state_esc_enter: input_state = {
    input_state {
        name: c"esc_enter",
        enter: Some(input_clear),
        exit: None,
        transitions: &raw const input_state_esc_enter_table as *const input_transition,
    }
};
static mut input_state_esc_intermediate: input_state = {
    input_state {
        name: c"esc_intermediate",
        enter: None,
        exit: None,
        transitions: &raw const input_state_esc_intermediate_table as *const input_transition,
    }
};
static mut input_state_csi_enter: input_state = {
    input_state {
        name: c"csi_enter",
        enter: Some(input_clear),
        exit: None,
        transitions: &raw const input_state_csi_enter_table as *const input_transition,
    }
};
static mut input_state_csi_parameter: input_state = {
    input_state {
        name: c"csi_parameter",
        enter: None,
        exit: None,
        transitions: &raw const input_state_csi_parameter_table as *const input_transition,
    }
};
static mut input_state_csi_intermediate: input_state = {
    input_state {
        name: c"csi_intermediate",
        enter: None,
        exit: None,
        transitions: &raw const input_state_csi_intermediate_table as *const input_transition,
    }
};
static mut input_state_csi_ignore: input_state = {
    input_state {
        name: c"csi_ignore",
        enter: None,
        exit: None,
        transitions: &raw const input_state_csi_ignore_table as *const input_transition,
    }
};
static mut input_state_dcs_enter: input_state = {
    input_state {
        name: c"dcs_enter",
        enter: Some(input_enter_dcs),
        exit: None,
        transitions: &raw const input_state_dcs_enter_table as *const input_transition,
    }
};
static mut input_state_dcs_parameter: input_state = {
    input_state {
        name: c"dcs_parameter",
        enter: None,
        exit: None,
        transitions: &raw const input_state_dcs_parameter_table as *const input_transition,
    }
};
static mut input_state_dcs_intermediate: input_state = {
    input_state {
        name: c"dcs_intermediate",
        enter: None,
        exit: None,
        transitions: &raw const input_state_dcs_intermediate_table as *const input_transition,
    }
};
static mut input_state_dcs_handler: input_state = {
    input_state {
        name: c"dcs_handler",
        enter: None,
        exit: None,
        transitions: &raw const input_state_dcs_handler_table as *const input_transition,
    }
};
static mut input_state_dcs_escape: input_state = {
    input_state {
        name: c"dcs_escape",
        enter: None,
        exit: None,
        transitions: &raw const input_state_dcs_escape_table as *const input_transition,
    }
};
static mut input_state_dcs_ignore: input_state = {
    input_state {
        name: c"dcs_ignore",
        enter: None,
        exit: None,
        transitions: &raw const input_state_dcs_ignore_table as *const input_transition,
    }
};
static mut input_state_osc_string: input_state = {
    input_state {
        name: c"osc_string",
        enter: Some(input_enter_osc),
        exit: Some(input_exit_osc),
        transitions: &raw const input_state_osc_string_table as *const input_transition,
    }
};
static mut input_state_apc_string: input_state = {
    input_state {
        name: c"apc_string",
        enter: Some(input_enter_apc),
        exit: Some(input_exit_apc),
        transitions: &raw const input_state_apc_string_table as *const input_transition,
    }
};
static mut input_state_rename_string: input_state = {
    input_state {
        name: c"rename_string",
        enter: Some(input_enter_rename),
        exit: Some(input_exit_rename),
        transitions: &raw const input_state_rename_string_table as *const input_transition,
    }
};
static mut input_state_consume_st: input_state = {
    input_state {
        name: c"consume_st",
        enter: Some(input_enter_rename),
        exit: None,
        transitions: &raw const input_state_consume_st_table as *const input_transition,
    }
};
static mut input_state_ground_table: [input_transition; 10] = {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_esc_enter,
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0x7e as ::core::ffi::c_int,
            handler: Some(input_print),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x7f as ::core::ffi::c_int,
            last: 0x7f as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x80 as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: Some(input_top_bit_set),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_esc_enter_table: [input_transition; 23] = {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_esc_enter,
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0x2f as ::core::ffi::c_int,
            handler: Some(input_intermediate),
            state: &raw const input_state_esc_intermediate,
        },
        input_transition {
            first: 0x30 as ::core::ffi::c_int,
            last: 0x4f as ::core::ffi::c_int,
            handler: Some(input_esc_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x50 as ::core::ffi::c_int,
            last: 0x50 as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_dcs_enter,
        },
        input_transition {
            first: 0x51 as ::core::ffi::c_int,
            last: 0x57 as ::core::ffi::c_int,
            handler: Some(input_esc_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x58 as ::core::ffi::c_int,
            last: 0x58 as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_consume_st,
        },
        input_transition {
            first: 0x59 as ::core::ffi::c_int,
            last: 0x59 as ::core::ffi::c_int,
            handler: Some(input_esc_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x5a as ::core::ffi::c_int,
            last: 0x5a as ::core::ffi::c_int,
            handler: Some(input_esc_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x5b as ::core::ffi::c_int,
            last: 0x5b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_csi_enter,
        },
        input_transition {
            first: 0x5c as ::core::ffi::c_int,
            last: 0x5c as ::core::ffi::c_int,
            handler: Some(input_esc_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x5d as ::core::ffi::c_int,
            last: 0x5d as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_osc_string,
        },
        input_transition {
            first: 0x5e as ::core::ffi::c_int,
            last: 0x5e as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_consume_st,
        },
        input_transition {
            first: 0x5f as ::core::ffi::c_int,
            last: 0x5f as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_apc_string,
        },
        input_transition {
            first: 0x60 as ::core::ffi::c_int,
            last: 0x6a as ::core::ffi::c_int,
            handler: Some(input_esc_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x6b as ::core::ffi::c_int,
            last: 0x6b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_rename_string,
        },
        input_transition {
            first: 0x6c as ::core::ffi::c_int,
            last: 0x7e as ::core::ffi::c_int,
            handler: Some(input_esc_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x7f as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_esc_intermediate_table: [input_transition; 10] = {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_esc_enter,
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0x2f as ::core::ffi::c_int,
            handler: Some(input_intermediate),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x30 as ::core::ffi::c_int,
            last: 0x7e as ::core::ffi::c_int,
            handler: Some(input_esc_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x7f as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_csi_enter_table: [input_transition; 14] = {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_esc_enter,
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0x2f as ::core::ffi::c_int,
            handler: Some(input_intermediate),
            state: &raw const input_state_csi_intermediate,
        },
        input_transition {
            first: 0x30 as ::core::ffi::c_int,
            last: 0x39 as ::core::ffi::c_int,
            handler: Some(input_parameter),
            state: &raw const input_state_csi_parameter,
        },
        input_transition {
            first: 0x3a as ::core::ffi::c_int,
            last: 0x3a as ::core::ffi::c_int,
            handler: Some(input_parameter),
            state: &raw const input_state_csi_parameter,
        },
        input_transition {
            first: 0x3b as ::core::ffi::c_int,
            last: 0x3b as ::core::ffi::c_int,
            handler: Some(input_parameter),
            state: &raw const input_state_csi_parameter,
        },
        input_transition {
            first: 0x3c as ::core::ffi::c_int,
            last: 0x3f as ::core::ffi::c_int,
            handler: Some(input_intermediate),
            state: &raw const input_state_csi_parameter,
        },
        input_transition {
            first: 0x40 as ::core::ffi::c_int,
            last: 0x7e as ::core::ffi::c_int,
            handler: Some(input_csi_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x7f as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_csi_parameter_table: [input_transition; 14] = {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_esc_enter,
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0x2f as ::core::ffi::c_int,
            handler: Some(input_intermediate),
            state: &raw const input_state_csi_intermediate,
        },
        input_transition {
            first: 0x30 as ::core::ffi::c_int,
            last: 0x39 as ::core::ffi::c_int,
            handler: Some(input_parameter),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x3a as ::core::ffi::c_int,
            last: 0x3a as ::core::ffi::c_int,
            handler: Some(input_parameter),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x3b as ::core::ffi::c_int,
            last: 0x3b as ::core::ffi::c_int,
            handler: Some(input_parameter),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x3c as ::core::ffi::c_int,
            last: 0x3f as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_csi_ignore,
        },
        input_transition {
            first: 0x40 as ::core::ffi::c_int,
            last: 0x7e as ::core::ffi::c_int,
            handler: Some(input_csi_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x7f as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_csi_intermediate_table: [input_transition; 11] = {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_esc_enter,
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0x2f as ::core::ffi::c_int,
            handler: Some(input_intermediate),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x30 as ::core::ffi::c_int,
            last: 0x3f as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_csi_ignore,
        },
        input_transition {
            first: 0x40 as ::core::ffi::c_int,
            last: 0x7e as ::core::ffi::c_int,
            handler: Some(input_csi_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x7f as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_csi_ignore_table: [input_transition; 10] = {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_esc_enter,
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0x3f as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x40 as ::core::ffi::c_int,
            last: 0x7e as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x7f as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_dcs_enter_table: [input_transition; 14] = {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_esc_enter,
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0x2f as ::core::ffi::c_int,
            handler: Some(input_intermediate),
            state: &raw const input_state_dcs_intermediate,
        },
        input_transition {
            first: 0x30 as ::core::ffi::c_int,
            last: 0x39 as ::core::ffi::c_int,
            handler: Some(input_parameter),
            state: &raw const input_state_dcs_parameter,
        },
        input_transition {
            first: 0x3a as ::core::ffi::c_int,
            last: 0x3a as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_dcs_ignore,
        },
        input_transition {
            first: 0x3b as ::core::ffi::c_int,
            last: 0x3b as ::core::ffi::c_int,
            handler: Some(input_parameter),
            state: &raw const input_state_dcs_parameter,
        },
        input_transition {
            first: 0x3c as ::core::ffi::c_int,
            last: 0x3f as ::core::ffi::c_int,
            handler: Some(input_intermediate),
            state: &raw const input_state_dcs_parameter,
        },
        input_transition {
            first: 0x40 as ::core::ffi::c_int,
            last: 0x7e as ::core::ffi::c_int,
            handler: Some(input_input),
            state: &raw const input_state_dcs_handler,
        },
        input_transition {
            first: 0x7f as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_dcs_parameter_table: [input_transition; 14] = {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_esc_enter,
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0x2f as ::core::ffi::c_int,
            handler: Some(input_intermediate),
            state: &raw const input_state_dcs_intermediate,
        },
        input_transition {
            first: 0x30 as ::core::ffi::c_int,
            last: 0x39 as ::core::ffi::c_int,
            handler: Some(input_parameter),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x3a as ::core::ffi::c_int,
            last: 0x3a as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_dcs_ignore,
        },
        input_transition {
            first: 0x3b as ::core::ffi::c_int,
            last: 0x3b as ::core::ffi::c_int,
            handler: Some(input_parameter),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x3c as ::core::ffi::c_int,
            last: 0x3f as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_dcs_ignore,
        },
        input_transition {
            first: 0x40 as ::core::ffi::c_int,
            last: 0x7e as ::core::ffi::c_int,
            handler: Some(input_input),
            state: &raw const input_state_dcs_handler,
        },
        input_transition {
            first: 0x7f as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_dcs_intermediate_table: [input_transition; 11] = {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_esc_enter,
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0x2f as ::core::ffi::c_int,
            handler: Some(input_intermediate),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x30 as ::core::ffi::c_int,
            last: 0x3f as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_dcs_ignore,
        },
        input_transition {
            first: 0x40 as ::core::ffi::c_int,
            last: 0x7e as ::core::ffi::c_int,
            handler: Some(input_input),
            state: &raw const input_state_dcs_handler,
        },
        input_transition {
            first: 0x7f as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_dcs_handler_table: [input_transition; 4] = {
    [
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(input_input),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_dcs_escape,
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: Some(input_input),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_dcs_escape_table: [input_transition; 4] = {
    [
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x5b as ::core::ffi::c_int,
            handler: Some(input_input),
            state: &raw const input_state_dcs_handler,
        },
        input_transition {
            first: 0x5c as ::core::ffi::c_int,
            last: 0x5c as ::core::ffi::c_int,
            handler: Some(input_dcs_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x5d as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: Some(input_input),
            state: &raw const input_state_dcs_handler,
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_dcs_ignore_table: [input_transition; 8] = {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_esc_enter,
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_osc_string_table: [input_transition; 10] = {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_esc_enter,
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x6 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x7 as ::core::ffi::c_int,
            last: 0x7 as ::core::ffi::c_int,
            handler: Some(input_end_bel),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x8 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: Some(input_input),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_apc_string_table: [input_transition; 8] = {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_esc_enter,
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: Some(input_input),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_rename_string_table: [input_transition; 8] = {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_esc_enter,
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: Some(input_input),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_consume_st_table: [input_transition; 8] = {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(input_c0_dispatch),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_esc_enter,
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_buffer_size: size_t = INPUT_BUF_DEFAULT_SIZE as size_t;
unsafe fn input_stop_utf8(mut ictx: *mut input_ctx) {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
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
        (*ictx).cell.cell.data = utf8_copy(&rc);
        screen_write_collect_add(sctx, &raw mut (*ictx).cell.cell);
    }
    (*ictx).utf8started = 0 as ::core::ffi::c_int;
}
unsafe fn input_fire_pane_title_changed(
    mut wp: *mut window_pane,
    mut title: *const ::core::ffi::c_char,
) {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    ep = event_payload_create();
    cmd_find_from_pane(&raw mut fs, wp, 0 as ::core::ffi::c_int);
    event_payload_set_target(ep, &raw mut fs);
    event_payload_set_pane(ep, b"pane\0" as *const u8 as *const ::core::ffi::c_char, wp);
    event_payload_set_window(
        ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).window as *mut window,
    );
    event_payload_set_string(
        ep,
        b"new_title\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write_cstr(out, title),
    );
    events_fire(
        b"pane-title-changed\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
}
unsafe fn input_ground_timer_callback(mut arg: *mut ::core::ffi::c_void) {
    let mut ictx: *mut input_ctx = arg as *mut input_ctx;
    log_debug(format_args!(
        "{}: {} expired",
        "input_ground_timer_callback",
        log_cstr((*(*ictx).state).name.as_ptr())
    ));
    input_reset(ictx, 0 as ::core::ffi::c_int);
}
unsafe fn input_start_ground_timer(mut ictx: *mut input_ctx) {
    let mut tv: timeval = timeval {
        tv_sec: 5 as __time_t,
        tv_usec: 0 as __suseconds_t,
    };
    event_del(&raw mut (*ictx).ground_timer);
    event_add(&raw mut (*ictx).ground_timer, &raw mut tv);
}
unsafe fn input_reset_cell(mut ictx: *mut input_ctx) {
    memcpy(
        &raw mut (*ictx).cell.cell as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    (*ictx).cell.set = 0 as ::core::ffi::c_int;
    (*ictx).cell.g1set = 0 as ::core::ffi::c_int;
    (*ictx).cell.g0set = (*ictx).cell.g1set;
    memcpy(
        &raw mut (*ictx).old_cell as *mut ::core::ffi::c_void,
        &raw mut (*ictx).cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<input_cell>() as size_t,
    );
    (*ictx).old_cx = 0 as u_int;
    (*ictx).old_cy = 0 as u_int;
}
unsafe fn input_save_state(mut ictx: *mut input_ctx) {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    let mut s: *mut screen = (*sctx).s;
    memcpy(
        &raw mut (*ictx).old_cell as *mut ::core::ffi::c_void,
        &raw mut (*ictx).cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<input_cell>() as size_t,
    );
    (*ictx).old_cx = (*s).cx;
    (*ictx).old_cy = (*s).cy;
    (*ictx).old_mode = (*s).mode;
}
unsafe fn input_restore_state(mut ictx: *mut input_ctx) {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    memcpy(
        &raw mut (*ictx).cell as *mut ::core::ffi::c_void,
        &raw mut (*ictx).old_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<input_cell>() as size_t,
    );
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
pub unsafe fn input_init(
    mut wp: *mut window_pane,
    mut bev: *mut bufferevent,
    mut palette: *mut colour_palette,
    mut c: *mut client,
) -> *mut input_ctx {
    let mut ictx: *mut input_ctx = ::core::ptr::null_mut::<input_ctx>();
    ictx = Box::into_raw(Box::new(input_ctx::new()));
    (*ictx).wp = wp;
    (*ictx).event = bev;
    (*ictx).palette = palette;
    (*ictx).c = c;
    event_set(
        &raw mut (*ictx).ground_timer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        move |_, _| unsafe { input_ground_timer_callback(ictx as *mut ::core::ffi::c_void) },
    );
    event_set(
        &raw mut (*ictx).request_timer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        move |_, _| unsafe { input_request_timer_callback(ictx as *mut ::core::ffi::c_void) },
    );
    input_reset(ictx, 0 as ::core::ffi::c_int);
    return ictx;
}
pub unsafe fn input_free(mut ictx: *mut input_ctx) {
    input_clear_params(&mut *ictx);
    loop {
        let ir = input_ctx_requests(ictx)
            .front_mut()
            .map(|owner| &mut **owner as *mut input_request);
        let Some(ir) = ir else {
            break;
        };
        input_free_request(ir);
    }
    event_del(&raw mut (*ictx).request_timer);
    event_del(&raw mut (*ictx).ground_timer);
    screen_write_stop_sync((*ictx).wp);
    drop(Box::from_raw(ictx));
}
pub unsafe fn input_reset(mut ictx: *mut input_ctx, mut clear: ::core::ffi::c_int) {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    let mut wp: *mut window_pane = (*ictx).wp;
    input_reset_cell(ictx);
    if clear != 0 && !wp.is_null() {
        if (*wp).modes.active.is_null() {
            screen_write_start_pane(sctx, wp, &raw mut (*wp).base);
        } else {
            screen_write_start(sctx, &raw mut (*wp).base);
        }
        screen_write_reset(sctx);
        screen_write_stop(sctx);
    }
    input_clear(ictx);
    (*ictx).state = &raw const input_state_ground as *const input_state;
    (*ictx).flags = 0 as ::core::ffi::c_int;
}
pub fn input_pending(ictx: &mut input_ctx) -> &mut evbuffer {
    &mut ictx.since_ground
}
unsafe fn input_set_state(mut ictx: *mut input_ctx, mut itr: *const input_transition) {
    if (*(*ictx).state).exit.is_some() {
        (*(*ictx).state).exit.expect("non-null function pointer")(ictx);
    }
    (*ictx).state = (*itr).state as *const input_state;
    if (*(*ictx).state).enter.is_some() {
        (*(*ictx).state).enter.expect("non-null function pointer")(ictx);
    }
}
unsafe fn input_parse(mut ictx: *mut input_ctx, mut buf: *const u_char, mut len: size_t) {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    let mut state: *const input_state = ::core::ptr::null::<input_state>();
    let mut itr: *const input_transition = ::core::ptr::null::<input_transition>();
    let mut off: size_t = 0 as size_t;
    while off < len {
        let fresh16 = off;
        off = off.wrapping_add(1);
        (*ictx).ch = *buf.offset(fresh16 as isize) as ::core::ffi::c_int;
        if (*ictx).state != state
            || itr.is_null()
            || (*ictx).ch < (*itr).first
            || (*ictx).ch > (*itr).last
        {
            itr = (*(*ictx).state).transitions;
            while (*itr).first != -(1 as ::core::ffi::c_int)
                && (*itr).last != -(1 as ::core::ffi::c_int)
            {
                if (*ictx).ch >= (*itr).first && (*ictx).ch <= (*itr).last {
                    break;
                }
                itr = itr.offset(1);
            }
            if (*itr).first == -(1 as ::core::ffi::c_int)
                || (*itr).last == -(1 as ::core::ffi::c_int)
            {
                fatalx(|out| out.write_all(b"no transition from state"));
            }
        }
        state = (*ictx).state as *const input_state;
        if (*itr).handler != Some(input_print) {
            screen_write_collect_end(sctx);
        }
        if (*itr).handler.is_some()
            && (*itr).handler.expect("non-null function pointer")(ictx) != 0 as ::core::ffi::c_int
        {
            continue;
        }
        if !(*itr).state.is_null() {
            input_set_state(ictx, itr);
        }
        if (*ictx).state != &raw const input_state_ground {
            evbuffer_add(
                &mut *(*ictx).since_ground,
                &raw mut (*ictx).ch as *const ::core::ffi::c_void,
                1 as size_t,
            );
        }
    }
}
pub unsafe fn input_parse_pane(mut wp: *mut window_pane) {
    let mut new_data: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut new_size: size_t = 0;
    new_data = window_pane_get_new_data(wp, &raw mut (*wp).offset, &raw mut new_size);
    if new_size != 0 as size_t {
        (*wp).last_output_time = time(::core::ptr::null_mut::<time_t>());
    }
    input_parse_buffer(wp, new_data as *const u_char, new_size);
    window_pane_update_used_data(wp, &raw mut (*wp).offset, new_size);
}
pub unsafe fn input_parse_buffer(
    mut wp: *mut window_pane,
    mut buf: *const u_char,
    mut len: size_t,
) {
    let mut ictx: *mut input_ctx = (*wp).ictx;
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    if len == 0 as size_t {
        return;
    }
    (*wp).output_generation = (*wp).output_generation.wrapping_add(1);
    window_update_activity((*wp).window as *mut window);
    if !(*wp).flags & PANE_ACTIVITY != 0 {
        (*wp).flags |= PANE_ACTIVITY;
        events_fire_pane(
            b"pane-activity\0" as *const u8 as *const ::core::ffi::c_char,
            wp,
        );
    }
    (*wp).flags |= PANE_CHANGED;
    if !(*wp).modes.active.is_null() {
        (*wp).flags |= PANE_UNSEENCHANGES;
    }
    if (*wp).modes.active.is_null() {
        screen_write_start_pane(sctx, wp, &raw mut (*wp).base);
    } else {
        screen_write_start(sctx, &raw mut (*wp).base);
    }
    log_debug(format_args!(
        "{}: %{} {}, {} bytes: {}",
        "input_parse_buffer",
        ((*wp).id) as u32,
        log_cstr((*(*ictx).state).name.as_ptr()),
        (len) as usize,
        log_cstr_n((buf) as *const _, len as ::core::ffi::c_int)
    ));
    input_parse(ictx, buf, len);
    screen_write_stop(sctx);
}
pub unsafe fn input_parse_screen(
    mut ictx: *mut input_ctx,
    mut s: *mut screen,
    mut cb: screen_write_init_ctx_cb,
    mut buf: *const u_char,
    mut len: size_t,
) {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    if len == 0 as size_t {
        return;
    }
    screen_write_start_callback(sctx, s, cb);
    input_parse(ictx, buf, len);
    screen_write_stop(sctx);
}
unsafe fn input_split(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ip: *mut input_param = ::core::ptr::null_mut::<input_param>();
    let mut i: u_int = 0;
    input_clear_params(&mut *ictx);
    (*ictx).param_list_len = 0 as u_int;
    if (*ictx).param_len == 0 as size_t {
        return 0 as ::core::ffi::c_int;
    }
    ip = (&raw mut (*ictx).param_list as *mut input_param).offset(0 as ::core::ffi::c_int as isize)
        as *mut input_param;
    ptr = &raw mut (*ictx).param_buf as *mut u_char as *mut ::core::ffi::c_char;
    loop {
        out = strsep(
            &raw mut ptr,
            b";\0" as *const u8 as *const ::core::ffi::c_char,
        );
        if out.is_null() {
            break;
        }
        if *out as ::core::ffi::c_int == '\0' as i32 {
            *ip = input_param::Missing;
        } else if !strchr(out, ':' as i32).is_null() {
            *ip = input_param::String(CStr::from_ptr(out).to_owned());
        } else {
            *ip = input_param::Number(strtonum(
                out,
                0 as ::core::ffi::c_longlong,
                INT_MAX as ::core::ffi::c_longlong,
                &raw mut errstr,
            ) as ::core::ffi::c_int);
            if !errstr.is_null() {
                return -(1 as ::core::ffi::c_int);
            }
        }
        (*ictx).param_list_len = (*ictx).param_list_len.wrapping_add(1);
        ip = (&raw mut (*ictx).param_list as *mut input_param)
            .offset((*ictx).param_list_len as isize) as *mut input_param;
        if (*ictx).param_list_len as usize
            == (::core::mem::size_of::<[input_param; 24]>() as usize)
                .wrapping_div(::core::mem::size_of::<input_param>() as usize)
        {
            return -(1 as ::core::ffi::c_int);
        }
    }
    i = 0 as u_int;
    while i < (*ictx).param_list_len {
        ip = (&raw mut (*ictx).param_list as *mut input_param).offset(i as isize)
            as *mut input_param;
        match &*ip {
            input_param::Missing => log_debug(format_args!("parameter {}: missing", (i) as u32)),
            input_param::String(value) => {
                log_debug(format_args!(
                    "parameter {}: string {}",
                    (i) as u32,
                    log_cstr((value.as_ptr()) as *const _)
                ));
            }
            input_param::Number(value) => log_debug(format_args!(
                "parameter {}: number {}",
                (i) as u32,
                (*value) as i32
            )),
        }
        i = i.wrapping_add(1);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe fn input_get(
    mut ictx: *mut input_ctx,
    mut validx: u_int,
    mut minval: ::core::ffi::c_int,
    mut defval: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut ip: *mut input_param = ::core::ptr::null_mut::<input_param>();
    let mut retval: ::core::ffi::c_int = 0;
    if validx >= (*ictx).param_list_len {
        return defval;
    }
    ip = (&raw mut (*ictx).param_list as *mut input_param).offset(validx as isize)
        as *mut input_param;
    retval = match &*ip {
        input_param::Missing => return defval,
        input_param::String(_) => return -1,
        input_param::Number(value) => *value,
    };
    if retval < minval {
        return minval;
    }
    return retval;
}
unsafe fn input_send_reply(mut ictx: *mut input_ctx, mut reply: *const ::core::ffi::c_char) {
    if !(*ictx).event.is_null() {
        log_debug(format_args!(
            "{}: {}",
            "input_send_reply",
            log_cstr((reply) as *const _)
        ));
        bufferevent_write(
            (*ictx).event,
            reply as *const ::core::ffi::c_void,
            strlen(reply),
        );
    }
}
unsafe fn input_reply(
    mut ictx: *mut input_ctx,
    mut add: ::core::ffi::c_int,
    write: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
) {
    let mut ir: *mut input_request = ::core::ptr::null_mut::<input_request>();
    let reply = format_message_with(write);
    if add != 0 && !input_ctx_requests(ictx).is_empty() {
        ir = input_make_request(ictx, INPUT_REQUEST_QUEUE);
        (*ir).data = Some(reply);
    } else {
        input_send_reply(ictx, reply.as_ptr());
    };
}
unsafe fn input_clear(mut ictx: *mut input_ctx) {
    event_del(&raw mut (*ictx).ground_timer);
    *(&raw mut (*ictx).interm_buf as *mut u_char) = '\0' as i32 as u_char;
    (*ictx).interm_len = 0 as size_t;
    *(&raw mut (*ictx).param_buf as *mut u_char) = '\0' as i32 as u_char;
    (*ictx).param_len = 0 as size_t;
    (&mut (*ictx).input_buf)[0] = 0;
    (*ictx).input_len = 0 as size_t;
    (*ictx).input_end = INPUT_END_ST;
    (*ictx).flags &= !INPUT_DISCARD;
}
unsafe fn input_ground(mut ictx: *mut input_ctx) {
    event_del(&raw mut (*ictx).ground_timer);
    evbuffer_drain(
        &mut *(*ictx).since_ground,
        evbuffer_get_length(&*(*ictx).since_ground),
    );
    (*ictx).shrink_buffer();
}
unsafe fn input_print(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
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
    screen_write_collect_add(sctx, &raw mut (*ictx).cell.cell);
    (*ictx).last = utf8_copy(&(*ictx).cell.cell.data);
    (*ictx).flags |= INPUT_LAST;
    (*ictx).cell.cell.attr =
        ((*ictx).cell.cell.attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
    return 0 as ::core::ffi::c_int;
}
unsafe fn input_intermediate(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    if (*ictx).interm_len
        == (::core::mem::size_of::<[u_char; 4]>() as usize).wrapping_sub(1 as usize)
    {
        (*ictx).flags |= INPUT_DISCARD;
    } else {
        let fresh15 = (*ictx).interm_len;
        (*ictx).interm_len = (*ictx).interm_len.wrapping_add(1);
        (*ictx).interm_buf[fresh15 as usize] = (*ictx).ch as u_char;
        (*ictx).interm_buf[(*ictx).interm_len as usize] = '\0' as i32 as u_char;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe fn input_parameter(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    if (*ictx).param_len
        == (::core::mem::size_of::<[u_char; 64]>() as usize).wrapping_sub(1 as usize)
    {
        (*ictx).flags |= INPUT_DISCARD;
    } else {
        let fresh14 = (*ictx).param_len;
        (*ictx).param_len = (*ictx).param_len.wrapping_add(1);
        (*ictx).param_buf[fresh14 as usize] = (*ictx).ch as u_char;
        (*ictx).param_buf[(*ictx).param_len as usize] = '\0' as i32 as u_char;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe fn input_input(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    let mut available: size_t = 0;
    available = (*ictx).input_buf.len();
    while (*ictx).input_len.wrapping_add(1 as size_t) >= available {
        available = available.wrapping_mul(2 as size_t);
        if available > input_buffer_size {
            (*ictx).flags |= INPUT_DISCARD;
            return 0 as ::core::ffi::c_int;
        }
        (*ictx).input_buf.resize(available, 0);
    }
    let fresh1 = (*ictx).input_len;
    (*ictx).input_len = (*ictx).input_len.wrapping_add(1);
    (&mut (*ictx).input_buf)[fresh1] = (*ictx).ch as u_char;
    (&mut (*ictx).input_buf)[(*ictx).input_len] = '\0' as i32 as u_char;
    return 0 as ::core::ffi::c_int;
}
unsafe fn input_c0_dispatch(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    let mut wp: *mut window_pane = (*ictx).wp;
    let mut s: *mut screen = (*sctx).s;
    let mut gc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let mut first_gc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let mut cx: u_int = 0;
    let mut line: u_int = 0;
    let mut width: u_int = 0;
    let mut has_content: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    input_stop_utf8(ictx);
    log_debug(format_args!(
        "{}: '{}'",
        "input_c0_dispatch",
        log_byte(((*ictx).ch) as u8)
    ));
    match (*ictx).ch {
        0 => {}
        7 => {
            if !wp.is_null() {
                events_fire_pane(
                    b"pane-bell\0" as *const u8 as *const ::core::ffi::c_char,
                    wp,
                );
                alerts_queue((*wp).window as *mut window, WINDOW_BELL);
            }
        }
        8 => {
            screen_write_backspace(sctx);
        }
        9 => {
            cx = (*s).cx;
            if !(cx >= (*s).grid().sx.wrapping_sub(1 as u_int)) {
                line = (*s).cy.wrapping_add((*s).grid().hsize);
                grid_get_cell((*s).grid(), cx, line, &mut first_gc);
                loop {
                    if has_content == 0 {
                        grid_get_cell((*s).grid(), cx, line, &mut gc);
                        if gc.data.size as ::core::ffi::c_int != 1 as ::core::ffi::c_int
                            || *(&raw mut gc.data.data as *mut u_char) as ::core::ffi::c_int
                                != ' ' as i32
                            || !grid_cells_look_equal(&gc, &first_gc)
                        {
                            has_content = 1 as ::core::ffi::c_int;
                        }
                    }
                    cx = cx.wrapping_add(1);
                    if screen_has_tab(&*s, cx) {
                        break;
                    }
                    if !(cx < (*s).grid().sx.wrapping_sub(1 as u_int)) {
                        break;
                    }
                }
                width = cx.wrapping_sub((*s).cx);
                if has_content != 0
                    || width as usize > ::core::mem::size_of::<[u_char; 32]>() as usize
                {
                    (*s).cx = cx;
                } else {
                    grid_get_cell((*s).grid(), (*s).cx, line, &mut gc);
                    grid_set_tab(&mut gc, width);
                    screen_write_collect_add(sctx, &raw mut gc);
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
            log_debug(format_args!(
                "{}: unknown '{}'",
                "input_c0_dispatch",
                log_byte(((*ictx).ch) as u8)
            ));
        }
    }
    (*ictx).flags &= !INPUT_LAST;
    return 0 as ::core::ffi::c_int;
}
unsafe fn input_esc_dispatch(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    let mut s: *mut screen = (*sctx).s;
    let mut entry: *const input_table_entry = ::core::ptr::null::<input_table_entry>();
    if (*ictx).flags & INPUT_DISCARD != 0 {
        return 0 as ::core::ffi::c_int;
    }
    log_debug(format_args!(
        "{}: '{}', {}",
        "input_esc_dispatch",
        log_byte(((*ictx).ch) as u8),
        log_cstr((&raw mut (*ictx).interm_buf as *mut u_char) as *const _)
    ));
    entry = input_table_find(&input_esc_table, &*ictx).map_or(::core::ptr::null(), |entry| {
        entry as *const input_table_entry
    });
    if entry.is_null() {
        log_debug(format_args!(
            "{}: unknown '{}'",
            "input_esc_dispatch",
            log_byte(((*ictx).ch) as u8)
        ));
        return 0 as ::core::ffi::c_int;
    }
    match (*entry).type_0 {
        9 => {
            colour_palette_clear((*ictx).palette);
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
            if (*s).cx < (*s).grid().sx {
                let column = (*s).cx;
                screen_set_tab(&mut *s, column, true);
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
        14 | _ => {}
    }
    (*ictx).flags &= !INPUT_LAST;
    return 0 as ::core::ffi::c_int;
}
unsafe fn input_csi_dispatch(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    let mut s: *mut screen = (*sctx).s;
    let mut entry: *const input_table_entry = ::core::ptr::null::<input_table_entry>();
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
    log_debug(format_args!(
        "{}: '{}' \"{}\" \"{}\"",
        "input_csi_dispatch",
        log_byte(((*ictx).ch) as u8),
        log_cstr((&raw mut (*ictx).interm_buf as *mut u_char) as *const _),
        log_cstr((&raw mut (*ictx).param_buf as *mut u_char) as *const _)
    ));
    if input_split(ictx) != 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    entry = input_table_find(&input_csi_table, &*ictx).map_or(::core::ptr::null(), |entry| {
        entry as *const input_table_entry
    });
    if entry.is_null() {
        log_debug(format_args!(
            "{}: unknown '{}'",
            "input_csi_dispatch",
            log_byte(((*ictx).ch) as u8)
        ));
        return 0 as ::core::ffi::c_int;
    }
    match (*entry).type_0 {
        0 => {
            cx = (*s).cx;
            if cx > (*s).grid().sx.wrapping_sub(1 as u_int) {
                cx = (*s).grid().sx.wrapping_sub(1 as u_int);
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
                    n = n - 1;
                    fresh10 > 0 as ::core::ffi::c_int
                } {
                    loop {
                        cx = cx.wrapping_sub(1);
                        if !(cx > 0 as u_int && !screen_has_tab(&*s, cx)) {
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
                ek = options_get_number(
                    global_options,
                    b"extended-keys\0" as *const u8 as *const ::core::ffi::c_char,
                ) as ::core::ffi::c_int;
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
                if options_get_number(
                    global_options,
                    b"extended-keys\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 2 as ::core::ffi::c_longlong
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
                    input_reply(ictx, 1 as ::core::ffi::c_int, |out| {
                        out.write_all(b"\x1B[?1;2c")
                    });
                }
                _ => {
                    log_debug(format_args!(
                        "{}: unknown '{}'",
                        "input_csi_dispatch",
                        log_byte(((*ictx).ch) as u8)
                    ));
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
                    input_reply(ictx, 1 as ::core::ffi::c_int, |out| {
                        out.write_all(b"\x1B[>84;0;0c")
                    });
                }
                _ => {
                    log_debug(format_args!(
                        "{}: unknown '{}'",
                        "input_csi_dispatch",
                        log_byte(((*ictx).ch) as u8)
                    ));
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
                (*s).grid().sy as ::core::ffi::c_int,
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
            match input_get(
                ictx,
                0 as u_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            ) {
                996 => {
                    input_report_current_theme(ictx);
                }
                _ => {}
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
                input_reply(ictx, 1 as ::core::ffi::c_int, |out| {
                    write!(out, "\x1B[{};{}$y", (m) as i32, (n) as i32)
                });
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
                        if !(*ictx).wp.is_null() {
                            oo = (*(*ictx).wp).options;
                        } else {
                            oo = global_w_options;
                        }
                        p = options_get_number(
                            oo,
                            b"cursor-style\0" as *const u8 as *const ::core::ffi::c_char,
                        ) as ::core::ffi::c_int;
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
                input_reply(ictx, 1 as ::core::ffi::c_int, |out| {
                    write!(out, "\x1B[?{};{}$y", (m) as i32, (n) as i32)
                });
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
                    input_reply(ictx, 1 as ::core::ffi::c_int, |out| {
                        out.write_all(b"\x1B[0n")
                    });
                }
                6 => {
                    input_reply(ictx, 1 as ::core::ffi::c_int, |out| {
                        write!(
                            out,
                            "\x1B[{};{}R",
                            ((*s).cy.wrapping_add(1 as u_int)) as u32,
                            ((*s).cx.wrapping_add(1 as u_int)) as u32
                        )
                    });
                }
                _ => {
                    log_debug(format_args!(
                        "{}: unknown '{}'",
                        "input_csi_dispatch",
                        log_byte(((*ictx).ch) as u8)
                    ));
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
                    log_debug(format_args!(
                        "{}: unknown '{}'",
                        "input_csi_dispatch",
                        log_byte(((*ictx).ch) as u8)
                    ));
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
                    log_debug(format_args!(
                        "{}: unknown '{}'",
                        "input_csi_dispatch",
                        log_byte(((*ictx).ch) as u8)
                    ));
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
                m = (*s).grid().sx.wrapping_sub((*s).cx) as ::core::ffi::c_int;
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
                    (*ictx).cell.cell.data = utf8_copy(&(*ictx).last);
                    i = 0 as ::core::ffi::c_int;
                    while i < n {
                        screen_write_collect_add(sctx, &raw mut (*ictx).cell.cell);
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
            input_csi_dispatch_sm_graphics();
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
                    if (*s).cx < (*s).grid().sx {
                        let column = (*s).cx;
                        screen_set_tab(&mut *s, column, false);
                    }
                }
                3 => {
                    screen_clear_tabs(&mut *s);
                }
                _ => {
                    log_debug(format_args!(
                        "{}: unknown '{}'",
                        "input_csi_dispatch",
                        log_byte(((*ictx).ch) as u8)
                    ));
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
                input_reply(ictx, 1 as ::core::ffi::c_int, |out| {
                    out.write_all(b"\x1BP>|tmux ")?;
                    write_cstr(out, getversion())?;
                    out.write_all(b"\x1B\\")
                });
            }
        }
        _ => {}
    }
    (*ictx).flags &= !INPUT_LAST;
    return 0 as ::core::ffi::c_int;
}
unsafe fn input_csi_dispatch_rm(mut ictx: *mut input_ctx) {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
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
                log_debug(format_args!(
                    "{}: unknown '{}'",
                    "input_csi_dispatch_rm",
                    log_byte(((*ictx).ch) as u8)
                ));
            }
        }
        i = i.wrapping_add(1);
    }
}
unsafe fn input_csi_dispatch_rm_private(mut ictx: *mut input_ctx) {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    let mut gc: *mut grid_cell = &raw mut (*ictx).cell.cell;
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
                screen_write_alternateoff(&mut *sctx, &mut *gc, 0 as ::core::ffi::c_int);
            }
            1049 => {
                screen_write_alternateoff(&mut *sctx, &mut *gc, 1 as ::core::ffi::c_int);
            }
            2004 => {
                screen_write_mode_clear(sctx, MODE_BRACKETPASTE);
            }
            2026 => {
                screen_write_end_sync(sctx);
            }
            2031 => {
                screen_write_mode_clear(sctx, MODE_THEME_UPDATES);
                if !(*ictx).wp.is_null() {
                    (*(*ictx).wp).flags &= !PANE_THEMECHANGED;
                }
            }
            _ => {
                log_debug(format_args!(
                    "{}: unknown '{}'",
                    "input_csi_dispatch_rm_private",
                    log_byte(((*ictx).ch) as u8)
                ));
            }
        }
        i = i.wrapping_add(1);
    }
}
unsafe fn input_csi_dispatch_sm(mut ictx: *mut input_ctx) {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
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
                log_debug(format_args!(
                    "{}: unknown '{}'",
                    "input_csi_dispatch_sm",
                    log_byte(((*ictx).ch) as u8)
                ));
            }
        }
        i = i.wrapping_add(1);
    }
}
unsafe fn input_csi_dispatch_sm_private(mut ictx: *mut input_ctx) {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    let mut gc: *mut grid_cell = &raw mut (*ictx).cell.cell;
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
                screen_write_alternateon(&mut *sctx, &*gc, 0 as ::core::ffi::c_int);
            }
            1049 => {
                screen_write_alternateon(&mut *sctx, &*gc, 1 as ::core::ffi::c_int);
            }
            2004 => {
                screen_write_mode_set(sctx, MODE_BRACKETPASTE);
            }
            2031 => {
                screen_write_mode_set(sctx, MODE_THEME_UPDATES);
                if !(*ictx).wp.is_null() {
                    (*(*ictx).wp).last_theme = window_pane_get_theme((*ictx).wp);
                    (*(*ictx).wp).flags &= !PANE_THEMECHANGED;
                }
            }
            2026 => {
                screen_write_start_sync((*ictx).wp);
            }
            _ => {
                log_debug(format_args!(
                    "{}: unknown '{}'",
                    "input_csi_dispatch_sm_private",
                    log_byte(((*ictx).ch) as u8)
                ));
            }
        }
        i = i.wrapping_add(1);
    }
}
unsafe fn input_csi_dispatch_sm_graphics() {}
unsafe fn input_csi_dispatch_winops(mut ictx: *mut input_ctx) {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    let mut s: *mut screen = (*sctx).s;
    let mut wp: *mut window_pane = (*ictx).wp;
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut x: u_int = (*s).grid().sx;
    let mut y: u_int = (*s).grid().sy;
    let mut n: ::core::ffi::c_int = 0;
    let mut m: ::core::ffi::c_int = 0;
    if !wp.is_null() {
        w = (*wp).window as *mut window;
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
                current_block_25 = 8019652857213515700;
            }
            9 | 10 => {
                current_block_25 = 8019652857213515700;
            }
            14 => {
                if w.is_null() {
                    current_block_25 = 980989089337379490;
                } else {
                    input_reply(ictx, 1 as ::core::ffi::c_int, |out| {
                        write!(
                            out,
                            "\x1B[4;{};{}t",
                            (y.wrapping_mul((*w).ypixel)) as u32,
                            (x.wrapping_mul((*w).xpixel)) as u32
                        )
                    });
                    current_block_25 = 980989089337379490;
                }
            }
            15 => {
                if w.is_null() {
                    current_block_25 = 980989089337379490;
                } else {
                    input_reply(ictx, 1 as ::core::ffi::c_int, |out| {
                        write!(
                            out,
                            "\x1B[5;{};{}t",
                            (y.wrapping_mul((*w).ypixel)) as u32,
                            (x.wrapping_mul((*w).xpixel)) as u32
                        )
                    });
                    current_block_25 = 980989089337379490;
                }
            }
            16 => {
                if w.is_null() {
                    current_block_25 = 980989089337379490;
                } else {
                    input_reply(ictx, 1 as ::core::ffi::c_int, |out| {
                        write!(
                            out,
                            "\x1B[6;{};{}t",
                            ((*w).ypixel) as u32,
                            ((*w).xpixel) as u32
                        )
                    });
                    current_block_25 = 980989089337379490;
                }
            }
            18 => {
                input_reply(ictx, 1 as ::core::ffi::c_int, |out| {
                    write!(out, "\x1B[8;{};{}t", (y) as u32, (x) as u32)
                });
                current_block_25 = 980989089337379490;
            }
            19 => {
                input_reply(ictx, 1 as ::core::ffi::c_int, |out| {
                    write!(out, "\x1B[9;{};{}t", (y) as u32, (x) as u32)
                });
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
                        screen_push_title(&mut *(*sctx).s);
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
                        screen_pop_title(&mut *(*sctx).s);
                        if !wp.is_null() {
                            input_fire_pane_title_changed(wp, (*(*sctx).s).title.as_ptr());
                            server_redraw_window_borders(w);
                            server_status_window(w);
                        }
                    }
                    _ => {}
                }
                current_block_25 = 980989089337379490;
            }
            _ => {
                log_debug(format_args!(
                    "{}: unknown '{}'",
                    "input_csi_dispatch_winops",
                    log_byte(((*ictx).ch) as u8)
                ));
                current_block_25 = 980989089337379490;
            }
        }
        match current_block_25 {
            8019652857213515700 => {
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
            _ => {}
        }
        m += 1;
    }
}
unsafe fn input_csi_dispatch_sgr_256_do(
    mut ictx: *mut input_ctx,
    mut fgbg: ::core::ffi::c_int,
    mut c: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut gc: *mut grid_cell = &raw mut (*ictx).cell.cell;
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
    return 1 as ::core::ffi::c_int;
}
unsafe fn input_csi_dispatch_sgr_256(
    mut ictx: *mut input_ctx,
    mut fgbg: ::core::ffi::c_int,
    mut i: *mut u_int,
) {
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
unsafe fn input_csi_dispatch_sgr_rgb_do(
    mut ictx: *mut input_ctx,
    mut fgbg: ::core::ffi::c_int,
    mut r: ::core::ffi::c_int,
    mut g: ::core::ffi::c_int,
    mut b: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut gc: *mut grid_cell = &raw mut (*ictx).cell.cell;
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
    return 1 as ::core::ffi::c_int;
}
unsafe fn input_csi_dispatch_sgr_rgb(
    mut ictx: *mut input_ctx,
    mut fgbg: ::core::ffi::c_int,
    mut i: *mut u_int,
) {
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
unsafe fn input_csi_dispatch_sgr_colon(mut ictx: *mut input_ctx, mut i: u_int) {
    let mut gc: *mut grid_cell = &raw mut (*ictx).cell.cell;
    let s = match &(*ictx).param_list[i as usize] {
        input_param::String(value) => value.as_ptr(),
        _ => panic!("SGR colon parser requires a string parameter"),
    };
    let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: [::core::ffi::c_int; 8] = [0; 8];
    let mut n: u_int = 0;
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    n = 0 as u_int;
    while (n as usize)
        < (::core::mem::size_of::<[::core::ffi::c_int; 8]>() as usize)
            .wrapping_div(::core::mem::size_of::<::core::ffi::c_int>() as usize)
    {
        p[n as usize] = -(1 as ::core::ffi::c_int);
        n = n.wrapping_add(1);
    }
    n = 0 as u_int;
    // strsep writes delimiters into this private C-string copy.
    let mut copy = std::ffi::CStr::from_ptr(s).to_bytes_with_nul().to_vec();
    ptr = copy.as_mut_ptr().cast();
    loop {
        out = strsep(
            &raw mut ptr,
            b":\0" as *const u8 as *const ::core::ffi::c_char,
        );
        if out.is_null() {
            break;
        }
        if *out as ::core::ffi::c_int != '\0' as i32 {
            let fresh13 = n;
            n = n.wrapping_add(1);
            p[fresh13 as usize] = strtonum(
                out,
                0 as ::core::ffi::c_longlong,
                INT_MAX as ::core::ffi::c_longlong,
                &raw mut errstr,
            ) as ::core::ffi::c_int;
            if !errstr.is_null()
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
        log_debug(format_args!(
            "{}: {} = {}",
            "input_csi_dispatch_sgr_colon",
            (n.wrapping_sub(1 as u_int)) as u32,
            (p[n.wrapping_sub(1 as u_int) as usize]) as i32
        ));
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
                (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_UNDERSCORE) as u_short;
            }
            2 => {
                (*gc).attr =
                    ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_ALL_UNDERSCORE) as u_short;
                (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_UNDERSCORE_2) as u_short;
            }
            3 => {
                (*gc).attr =
                    ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_ALL_UNDERSCORE) as u_short;
                (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_UNDERSCORE_3) as u_short;
            }
            4 => {
                (*gc).attr =
                    ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_ALL_UNDERSCORE) as u_short;
                (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_UNDERSCORE_4) as u_short;
            }
            5 => {
                (*gc).attr =
                    ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_ALL_UNDERSCORE) as u_short;
                (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_UNDERSCORE_5) as u_short;
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
        5 => {
            if !(n < 3 as u_int) {
                input_csi_dispatch_sgr_256_do(
                    ictx,
                    p[0 as ::core::ffi::c_int as usize],
                    p[2 as ::core::ffi::c_int as usize],
                );
            }
        }
        _ => {}
    };
}

#[cfg(test)]
mod sgr_colon_tests {
    use super::{
        colour_join_rgb, grid_cell, input_csi_dispatch_sgr_colon, input_ctx, input_param,
        COLOUR_FLAG_256, GRID_ATTR_UNDERSCORE_2, GRID_ATTR_UNDERSCORE_3,
    };

    fn parse(source: &[u8]) -> grid_cell {
        assert!(source.contains(&0));
        let mut ictx = input_ctx::new();
        let parameter = std::ffi::CStr::from_bytes_until_nul(source).unwrap();
        ictx.param_list[0] = input_param::String(parameter.to_owned());
        ictx.cell.cell.fg = 11;
        ictx.cell.cell.bg = 12;
        ictx.cell.cell.us = 13;
        ictx.cell.cell.attr = GRID_ATTR_UNDERSCORE_2 as u16;
        unsafe { input_csi_dispatch_sgr_colon(&raw mut ictx, 0) };
        assert_eq!(
            match &ictx.param_list[0] {
                input_param::String(value) => value.as_c_str(),
                _ => panic!("expected string parameter"),
            },
            parameter,
            "the input parameter must not be tokenized"
        );
        ictx.cell.cell
    }

    #[test]
    fn parses_rgb_indexed_and_underline_colon_parameters() {
        assert_eq!(parse(b"38:2:1:2:3\0").fg, unsafe {
            colour_join_rgb(1, 2, 3)
        });
        assert_eq!(parse(b"38:2::1:2:3\0").fg, unsafe {
            colour_join_rgb(1, 2, 3)
        });
        assert_eq!(parse(b"48:5:196\0").bg, 196 | COLOUR_FLAG_256);
        assert_eq!(parse(b"58:2:5:6:7\0").us, unsafe {
            colour_join_rgb(5, 6, 7)
        });
        assert_eq!(parse(b"4:3\0").attr as i32, GRID_ATTR_UNDERSCORE_3);
    }

    #[test]
    fn rejects_bad_and_excess_tokens_without_changing_colours() {
        for source in [
            b"38:2:999999999999999:2:3\0".as_slice(),
            b"38:2:abc:2:3\0",
            b"38:2:1:2:3:4:5:6\0", // eighth populated token
            b":::::::\0",          // eighth empty token
            b"38:2:300:2:3\0",     // parsed RGB outside the colour range
            b"38:2:1:2\0",         // incomplete RGB
        ] {
            let cell = parse(source);
            assert_eq!((cell.fg, cell.bg, cell.us), (11, 12, 13), "{source:?}");
        }
        // Both the owned CStr copy and the parser stop at the first NUL.
        assert_eq!(parse(b"38:5:196\0:2:1:2:3\0").fg, 196 | COLOUR_FLAG_256);
    }
}
unsafe fn input_csi_dispatch_sgr(mut ictx: *mut input_ctx) {
    let mut gc: *mut grid_cell = &raw mut (*ictx).cell.cell;
    let mut i: u_int = 0;
    let mut link: u_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    if (*ictx).param_list_len == 0 as u_int {
        memcpy(
            gc as *mut ::core::ffi::c_void,
            &raw const grid_default_cell as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
        return;
    }
    i = 0 as u_int;
    while i < (*ictx).param_list_len {
        if matches!(&(*ictx).param_list[i as usize], input_param::String(_)) {
            input_csi_dispatch_sgr_colon(ictx, i);
        } else {
            n = input_get(ictx, i, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
            if !(n == -(1 as ::core::ffi::c_int)) {
                if n == 38 as ::core::ffi::c_int
                    || n == 48 as ::core::ffi::c_int
                    || n == 58 as ::core::ffi::c_int
                {
                    i = i.wrapping_add(1);
                    match input_get(ictx, i, 0 as ::core::ffi::c_int, -(1 as ::core::ffi::c_int)) {
                        2 => {
                            input_csi_dispatch_sgr_rgb(ictx, n, &raw mut i);
                        }
                        5 => {
                            input_csi_dispatch_sgr_256(ictx, n, &raw mut i);
                        }
                        _ => {}
                    }
                } else {
                    match n {
                        0 => {
                            link = (*gc).link;
                            memcpy(
                                gc as *mut ::core::ffi::c_void,
                                &raw const grid_default_cell as *const ::core::ffi::c_void,
                                ::core::mem::size_of::<grid_cell>() as size_t,
                            );
                            (*gc).link = link;
                        }
                        1 => {
                            (*gc).attr =
                                ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_BRIGHT) as u_short;
                        }
                        2 => {
                            (*gc).attr =
                                ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_DIM) as u_short;
                        }
                        3 => {
                            (*gc).attr =
                                ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_ITALICS) as u_short;
                        }
                        4 => {
                            (*gc).attr = ((*gc).attr as ::core::ffi::c_int
                                & !GRID_ATTR_ALL_UNDERSCORE)
                                as u_short;
                            (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_UNDERSCORE)
                                as u_short;
                        }
                        5 | 6 => {
                            (*gc).attr =
                                ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_BLINK) as u_short;
                        }
                        7 => {
                            (*gc).attr =
                                ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_REVERSE) as u_short;
                        }
                        8 => {
                            (*gc).attr =
                                ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_HIDDEN) as u_short;
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
                            (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_UNDERSCORE_2)
                                as u_short;
                        }
                        22 => {
                            (*gc).attr = ((*gc).attr as ::core::ffi::c_int
                                & !(GRID_ATTR_BRIGHT | GRID_ATTR_DIM))
                                as u_short;
                        }
                        23 => {
                            (*gc).attr =
                                ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_ITALICS) as u_short;
                        }
                        24 => {
                            (*gc).attr = ((*gc).attr as ::core::ffi::c_int
                                & !GRID_ATTR_ALL_UNDERSCORE)
                                as u_short;
                        }
                        25 => {
                            (*gc).attr =
                                ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_BLINK) as u_short;
                        }
                        27 => {
                            (*gc).attr =
                                ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_REVERSE) as u_short;
                        }
                        28 => {
                            (*gc).attr =
                                ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_HIDDEN) as u_short;
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
                            (*gc).attr =
                                ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_OVERLINE) as u_short;
                        }
                        55 => {
                            (*gc).attr =
                                ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_OVERLINE) as u_short;
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
unsafe fn input_end_bel(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    log_debug(format_args!("{}", "input_end_bel"));
    (*ictx).input_end = INPUT_END_BEL;
    return 0 as ::core::ffi::c_int;
}
unsafe fn input_enter_dcs(mut ictx: *mut input_ctx) {
    log_debug(format_args!("{}", "input_enter_dcs"));
    input_clear(ictx);
    input_start_ground_timer(ictx);
    (*ictx).flags &= !INPUT_LAST;
}
unsafe fn input_handle_decrqss(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = (*ictx).wp;
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    let mut buf: *mut u_char = (*ictx).input_buf.as_mut_ptr();
    let mut len: size_t = (*ictx).input_len;
    let mut s: *mut screen = (*sctx).s;
    let mut ps: ::core::ffi::c_int = 0;
    let mut opt_ps: ::core::ffi::c_int = 0;
    let mut blinking: ::core::ffi::c_int = 0;
    if len < 3 as size_t
        || *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != ' ' as i32
        || *buf.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != 'q' as i32
    {
        input_reply(ictx, 1 as ::core::ffi::c_int, |out| {
            out.write_all(b"\x1BP0$r\x1B\\")
        });
        return 0 as ::core::ffi::c_int;
    } else {
        if (*s).cstyle as ::core::ffi::c_uint
            == SCREEN_CURSOR_BLOCK as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*s).cstyle as ::core::ffi::c_uint
                == SCREEN_CURSOR_UNDERLINE as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*s).cstyle as ::core::ffi::c_uint
                == SCREEN_CURSOR_BAR as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            blinking =
                ((*s).mode & MODE_CURSOR_BLINKING != 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
            match (*s).cstyle as ::core::ffi::c_uint {
                1 => {
                    ps = if blinking != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        2 as ::core::ffi::c_int
                    };
                }
                2 => {
                    ps = if blinking != 0 {
                        3 as ::core::ffi::c_int
                    } else {
                        4 as ::core::ffi::c_int
                    };
                }
                3 => {
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
                oo = (*wp).options;
            } else {
                oo = global_w_options;
            }
            opt_ps = options_get_number(
                oo,
                b"cursor-style\0" as *const u8 as *const ::core::ffi::c_char,
            ) as ::core::ffi::c_int;
            if opt_ps < 0 as ::core::ffi::c_int || opt_ps > 6 as ::core::ffi::c_int {
                opt_ps = 0 as ::core::ffi::c_int;
            }
            ps = opt_ps;
        }
        log_debug(format_args!(
            "{}: DECRQSS cursor -> Ps={} (cstyle={} mode={})",
            "input_handle_decrqss",
            (ps) as i32,
            ((*s).cstyle as ::core::ffi::c_uint) as i32,
            log_hex((((*s).mode) as u32) as u64)
        ));
        input_reply(ictx, 1 as ::core::ffi::c_int, |out| {
            write!(out, "\x1BP1$r q{} q\x1B\\", (ps) as i32)
        });
        return 0 as ::core::ffi::c_int;
    };
}
unsafe fn input_dcs_dispatch(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = (*ictx).wp;
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    let mut buf: *mut u_char = (*ictx).input_buf.as_mut_ptr();
    let mut len: size_t = (*ictx).input_len;
    let prefix: [::core::ffi::c_char; 6] =
        ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b"tmux;\0");
    let prefixlen: u_int = (::core::mem::size_of::<[::core::ffi::c_char; 6]>() as usize)
        .wrapping_sub(1 as usize) as u_int;
    let mut allow_passthrough: ::core::ffi::c_longlong = 0 as ::core::ffi::c_longlong;
    if wp.is_null() {
        oo = global_w_options;
    } else {
        oo = (*wp).options;
    }
    if (*ictx).flags & INPUT_DISCARD != 0 {
        log_debug(format_args!(
            "{}: {} bytes (discard)",
            "input_dcs_dispatch",
            (len) as usize
        ));
        return 0 as ::core::ffi::c_int;
    }
    if (*ictx).interm_len == 1 as size_t
        && (*ictx).interm_buf[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int == '$' as i32
    {
        if len >= 1 as size_t
            && *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'q' as i32
        {
            return input_handle_decrqss(ictx);
        }
    }
    allow_passthrough = options_get_number(
        oo,
        b"allow-passthrough\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if allow_passthrough == 0 {
        return 0 as ::core::ffi::c_int;
    }
    log_debug(format_args!(
        "{}: \"{}\"",
        "input_dcs_dispatch",
        log_cstr((buf) as *const _)
    ));
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
    return 0 as ::core::ffi::c_int;
}
unsafe fn input_enter_osc(mut ictx: *mut input_ctx) {
    log_debug(format_args!("{}", "input_enter_osc"));
    input_clear(ictx);
    input_start_ground_timer(ictx);
    (*ictx).flags &= !INPUT_LAST;
}
unsafe fn input_exit_osc(mut ictx: *mut input_ctx) {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    let mut wp: *mut window_pane = (*ictx).wp;
    let mut p: *mut u_char = (*ictx).input_buf.as_mut_ptr();
    let mut option: u_int = 0;
    if (*ictx).flags & INPUT_DISCARD != 0 {
        return;
    }
    if (*ictx).input_len < 1 as size_t
        || (*p as ::core::ffi::c_int) < '0' as i32
        || *p as ::core::ffi::c_int > '9' as i32
    {
        return;
    }
    log_debug(format_args!(
        "{}: \"{}\" (end {})",
        "input_exit_osc",
        log_cstr((p) as *const _),
        log_cstr(
            (if (*ictx).input_end as ::core::ffi::c_uint
                == INPUT_END_ST as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                b"ST\0" as *const u8 as *const ::core::ffi::c_char
            } else {
                b"BEL\0" as *const u8 as *const ::core::ffi::c_char
            }) as *const _
        )
    ));
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
                && options_get_number(
                    (*wp).options,
                    b"allow-set-title\0" as *const u8 as *const ::core::ffi::c_char,
                ) != 0
                && screen_set_title(
                    &mut *(*sctx).s,
                    CStr::from_ptr(p.cast()),
                    1 as ::core::ffi::c_int,
                ) != 0
            {
                input_fire_pane_title_changed(wp, p as *const ::core::ffi::c_char);
                server_redraw_window_borders((*wp).window as *mut window);
                server_status_window((*wp).window as *mut window);
            }
        }
        4 => {
            input_osc_4(ictx, p as *const ::core::ffi::c_char);
        }
        7 => {
            if !wp.is_null() && screen_set_path(&mut *(*sctx).s, CStr::from_ptr(p.cast())) != 0 {
                server_redraw_window_borders((*wp).window as *mut window);
                server_status_window((*wp).window as *mut window);
            }
        }
        8 => {
            input_osc_8(ictx, p as *const ::core::ffi::c_char);
        }
        9 => {
            input_osc_9(ictx, p as *const ::core::ffi::c_char);
        }
        10 => {
            input_osc_10(ictx, p as *const ::core::ffi::c_char);
        }
        11 => {
            input_osc_11(ictx, p as *const ::core::ffi::c_char);
        }
        12 => {
            input_osc_12(ictx, p as *const ::core::ffi::c_char);
        }
        52 => {
            input_osc_52(ictx, p as *const ::core::ffi::c_char);
        }
        104 => {
            input_osc_104(ictx, p as *const ::core::ffi::c_char);
        }
        110 => {
            input_osc_110(ictx, p as *const ::core::ffi::c_char);
        }
        111 => {
            input_osc_111(ictx, p as *const ::core::ffi::c_char);
        }
        112 => {
            input_osc_112(ictx, p as *const ::core::ffi::c_char);
        }
        133 => {
            input_osc_133(ictx, p as *const ::core::ffi::c_char);
        }
        _ => {
            log_debug(format_args!(
                "{}: unknown '{}'",
                "input_exit_osc",
                (option) as u32
            ));
        }
    };
}
unsafe fn input_enter_apc(mut ictx: *mut input_ctx) {
    log_debug(format_args!("{}", "input_enter_apc"));
    input_clear(ictx);
    input_start_ground_timer(ictx);
    (*ictx).flags &= !INPUT_LAST;
}
unsafe fn input_exit_apc(mut ictx: *mut input_ctx) {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    let mut wp: *mut window_pane = (*ictx).wp;
    if (*ictx).flags & INPUT_DISCARD != 0 {
        return;
    }
    log_debug(format_args!(
        "{}: \"{}\"",
        "input_exit_apc",
        log_cstr(((*ictx).input_buf.as_ptr()) as *const _)
    ));
    if !wp.is_null()
        && options_get_number(
            (*wp).options,
            b"allow-set-title\0" as *const u8 as *const ::core::ffi::c_char,
        ) != 0
        && screen_set_title(
            &mut *(*sctx).s,
            CStr::from_bytes_until_nul(&(*ictx).input_buf).expect("input buffer has a terminator"),
            1 as ::core::ffi::c_int,
        ) != 0
    {
        input_fire_pane_title_changed(wp, (*ictx).input_buf.as_ptr() as *const ::core::ffi::c_char);
        server_redraw_window_borders((*wp).window as *mut window);
        server_status_window((*wp).window as *mut window);
    }
}
unsafe fn input_enter_rename(mut ictx: *mut input_ctx) {
    log_debug(format_args!("{}", "input_enter_rename"));
    input_clear(ictx);
    input_start_ground_timer(ictx);
    (*ictx).flags &= !INPUT_LAST;
}
unsafe fn input_exit_rename(mut ictx: *mut input_ctx) {
    let mut wp: *mut window_pane = (*ictx).wp;
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    if wp.is_null() {
        return;
    }
    if (*ictx).flags & INPUT_DISCARD != 0 {
        return;
    }
    if options_get_number(
        (*(*ictx).wp).options,
        b"allow-rename\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0
    {
        return;
    }
    log_debug(format_args!(
        "{}: \"{}\"",
        "input_exit_rename",
        log_cstr(((*ictx).input_buf.as_ptr()) as *const _)
    ));
    if !utf8_isvalid(
        CStr::from_bytes_until_nul(&(*ictx).input_buf).expect("input buffer is terminated"),
    ) {
        return;
    }
    w = (*wp).window as *mut window;
    if (*ictx).input_len == 0 as size_t {
        o = options_get_only(
            (*w).options,
            b"automatic-rename\0" as *const u8 as *const ::core::ffi::c_char,
        );
        if !o.is_null() {
            options_remove_or_default(
                o,
                ::core::ptr::null::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<Option<std::ffi::CString>>(),
            );
        }
        if options_get_number(
            (*w).options,
            b"automatic-rename\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0
        {
            window_set_name(
                w,
                b"\0" as *const u8 as *const ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            );
        }
    } else {
        options_set_number(
            (*w).options,
            b"automatic-rename\0" as *const u8 as *const ::core::ffi::c_char,
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
unsafe fn input_top_bit_set(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
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
    match utf8_append(&mut *ud, (*ictx).ch as u_char) as ::core::ffi::c_uint {
        0 => return 0 as ::core::ffi::c_int,
        2 => {
            input_stop_utf8(ictx);
            return 0 as ::core::ffi::c_int;
        }
        1 | _ => {}
    }
    (*ictx).utf8started = 0 as ::core::ffi::c_int;
    log_debug(format_args!(
        "{} {} '{}' (width {})",
        "input_top_bit_set",
        ((*ud).size as ::core::ffi::c_int) as u8,
        log_cstr_width(
            (&raw mut (*ud).data as *mut u_char) as *const _,
            (*ud).size as ::core::ffi::c_int
        ),
        ((*ud).width as ::core::ffi::c_int) as u8
    ));
    (*ictx).cell.cell.data = utf8_copy(&*ud);
    screen_write_collect_add(sctx, &raw mut (*ictx).cell.cell);
    (*ictx).last = utf8_copy(&(*ictx).cell.cell.data);
    (*ictx).flags |= INPUT_LAST;
    return 0 as ::core::ffi::c_int;
}
unsafe fn input_osc_colour_reply(
    mut ictx: *mut input_ctx,
    mut add: ::core::ffi::c_int,
    mut n: u_int,
    mut idx: ::core::ffi::c_int,
    mut c: ::core::ffi::c_int,
    mut end_type: input_end_type,
) {
    let mut r: u_char = 0;
    let mut g: u_char = 0;
    let mut b: u_char = 0;
    let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if c != -(1 as ::core::ffi::c_int) {
        c = colour_force_rgb(c);
    }
    if c == -(1 as ::core::ffi::c_int) {
        return;
    }
    (r, g, b) = colour_split_rgb(c);
    if end_type as ::core::ffi::c_uint == INPUT_END_BEL as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        end = b"\x07\0" as *const u8 as *const ::core::ffi::c_char;
    } else {
        end = b"\x1B\\\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if n == 4 as u_int {
        input_reply(ictx, add, |out| {
            write!(
                out,
                "\x1B]{};{};rgb:{:02x}{:02x}/{:02x}{:02x}/{:02x}{:02x}",
                (n) as u32,
                (idx) as i32,
                (r as ::core::ffi::c_int) as u8,
                (r as ::core::ffi::c_int) as u8,
                (g as ::core::ffi::c_int) as u8,
                (g as ::core::ffi::c_int) as u8,
                (b as ::core::ffi::c_int) as u8,
                (b as ::core::ffi::c_int) as u8
            )?;
            write_cstr(out, end)
        });
    } else {
        input_reply(ictx, add, |out| {
            write!(
                out,
                "\x1B]{};rgb:{:02x}{:02x}/{:02x}{:02x}/{:02x}{:02x}",
                (n) as u32,
                (r as ::core::ffi::c_int) as u8,
                (r as ::core::ffi::c_int) as u8,
                (g as ::core::ffi::c_int) as u8,
                (g as ::core::ffi::c_int) as u8,
                (b as ::core::ffi::c_int) as u8,
                (b as ::core::ffi::c_int) as u8
            )?;
            write_cstr(out, end)
        });
    };
}
unsafe fn input_osc_4(mut ictx: *mut input_ctx, mut p: *const ::core::ffi::c_char) {
    // strsep writes NULs into this private copy; the original OSC input stays intact.
    let mut copy = std::ffi::CStr::from_ptr(p).to_bytes_with_nul().to_vec();
    let mut s: *mut ::core::ffi::c_char = copy.as_mut_ptr().cast();
    let mut next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut idx: ::core::ffi::c_long = 0;
    let mut c: ::core::ffi::c_int = 0;
    let mut bad: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut redraw: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut palette: *mut colour_palette = (*ictx).palette;
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
            s = strsep(
                &raw mut next,
                b";\0" as *const u8 as *const ::core::ffi::c_char,
            );
            if strcmp(s, b"?\0" as *const u8 as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                c = colour_palette_get(
                    palette,
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
                c = colour_parse_x11_logged(std::ffi::CStr::from_ptr(s)).unwrap_or(-1);
                if c == -(1 as ::core::ffi::c_int) {
                    s = next;
                } else {
                    if colour_palette_set(palette, idx as ::core::ffi::c_int, c) != 0 {
                        redraw = 1 as ::core::ffi::c_int;
                    }
                    s = next;
                }
            }
        }
    }
    if bad != 0 {
        log_debug(format_args!("bad OSC 4: {}", log_cstr((p) as *const _)));
    }
    if redraw != 0 {
        screen_write_fullredraw(&raw mut (*ictx).ctx);
    }
}
unsafe fn input_osc_8(mut ictx: *mut input_ctx, mut p: *const ::core::ffi::c_char) {
    let mut current_block: u64;
    let mut hl: *mut hyperlinks = (*(*ictx).ctx.s).hyperlinks;
    let mut gc: *mut grid_cell = &raw mut (*ictx).cell.cell;
    let mut start: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut uri: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut id: Option<std::ffi::CString> = None;
    start = p;
    loop {
        end = strpbrk(start, b":;\0" as *const u8 as *const ::core::ffi::c_char);
        if end.is_null() {
            current_block = 10886091980245723256;
            break;
        }
        if end.offset_from(start) as ::core::ffi::c_long >= 4 as ::core::ffi::c_long
            && strncmp(
                start,
                b"id=\0" as *const u8 as *const ::core::ffi::c_char,
                3 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            if id.is_some() {
                current_block = 9416799868769213755;
                break;
            }
            let id_start = start.add(3).cast::<u8>();
            let id_len = end.offset_from(start) as usize - 3;
            id = Some(
                std::ffi::CString::new(std::slice::from_raw_parts(id_start, id_len))
                    .expect("OSC 8 ID ends before the first NUL"),
            );
        }
        if *end as ::core::ffi::c_int == ';' as i32 {
            current_block = 10886091980245723256;
            break;
        }
        start = end.offset(1 as ::core::ffi::c_int as isize);
    }
    match current_block {
        10886091980245723256 => {
            if !(end.is_null() || *end as ::core::ffi::c_int != ';' as i32) {
                uri = end.offset(1 as ::core::ffi::c_int as isize);
                if *uri as ::core::ffi::c_int == '\0' as i32 {
                    (*gc).link = 0 as u_int;
                    return;
                }
                let id_ptr = id.as_ref().map_or(std::ptr::null(), |id| id.as_ptr());
                (*gc).link = hyperlinks_put(hl, uri, id_ptr);
                if id.is_none() {
                    log_debug(format_args!(
                        "hyperlink (anonymous) {} = {}",
                        log_cstr((uri) as *const _),
                        ((*gc).link) as u32
                    ));
                } else {
                    log_debug(format_args!(
                        "hyperlink (id={}) {} = {}",
                        log_cstr((id_ptr) as *const _),
                        log_cstr((uri) as *const _),
                        ((*gc).link) as u32
                    ));
                }
                return;
            }
        }
        _ => {}
    }
    log_debug(format_args!("bad OSC 8 {}", log_cstr((p) as *const _)));
}
unsafe fn input_set_progress_bar(
    mut ictx: *mut input_ctx,
    mut state: progress_bar_state,
    mut p: ::core::ffi::c_int,
) {
    screen_set_progress_bar(&mut *(*ictx).ctx.s, state, p);
    if !(*ictx).wp.is_null() {
        server_redraw_window_borders((*(*ictx).wp).window as *mut window);
        server_status_window((*(*ictx).wp).window as *mut window);
    }
}
unsafe fn input_osc_9(mut ictx: *mut input_ctx, mut p: *const ::core::ffi::c_char) {
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
                && *pb.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '\0' as i32
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
                    current_block = 12757115586032245927;
                    break;
                }
                let fresh8 = pb;
                pb = pb.offset(1);
                progress = progress * 10 as ::core::ffi::c_int + *fresh8 as ::core::ffi::c_int
                    - '0' as i32;
            }
            match current_block {
                12757115586032245927 => {}
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
    log_debug(format_args!("bad OSC 9;4 {}", log_cstr((p) as *const _)));
}
unsafe fn input_osc_10(mut ictx: *mut input_ctx, mut p: *const ::core::ffi::c_char) {
    let mut wp: *mut window_pane = (*ictx).wp;
    let mut defaults: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let mut c: ::core::ffi::c_int = 0;
    if strcmp(p, b"?\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int {
        if wp.is_null() {
            return;
        }
        c = window_pane_get_fg_control_client(wp);
        if c == -(1 as ::core::ffi::c_int) {
            tty_default_colours(&raw mut defaults, wp, ::core::ptr::null_mut::<u_int>());
            if defaults.fg == 8 as ::core::ffi::c_int || defaults.fg == 9 as ::core::ffi::c_int {
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
    c = colour_parse_x11_logged(std::ffi::CStr::from_ptr(p)).unwrap_or(-1);
    if c == -(1 as ::core::ffi::c_int) {
        log_debug(format_args!("bad OSC 10: {}", log_cstr((p) as *const _)));
        return;
    }
    if !(*ictx).palette.is_null() {
        (*(*ictx).palette).fg = c;
        if !wp.is_null() {
            (*wp).flags |= PANE_STYLECHANGED;
        }
        screen_write_fullredraw(&raw mut (*ictx).ctx);
    }
}
unsafe fn input_osc_110(mut ictx: *mut input_ctx, mut p: *const ::core::ffi::c_char) {
    let mut wp: *mut window_pane = (*ictx).wp;
    if *p as ::core::ffi::c_int != '\0' as i32 {
        return;
    }
    if !(*ictx).palette.is_null() {
        (*(*ictx).palette).fg = 8 as ::core::ffi::c_int;
        if !wp.is_null() {
            (*wp).flags |= PANE_STYLECHANGED;
        }
        screen_write_fullredraw(&raw mut (*ictx).ctx);
    }
}
unsafe fn input_osc_11(mut ictx: *mut input_ctx, mut p: *const ::core::ffi::c_char) {
    let mut wp: *mut window_pane = (*ictx).wp;
    let mut c: ::core::ffi::c_int = 0;
    if strcmp(p, b"?\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int {
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
    c = colour_parse_x11_logged(std::ffi::CStr::from_ptr(p)).unwrap_or(-1);
    if c == -(1 as ::core::ffi::c_int) {
        log_debug(format_args!("bad OSC 11: {}", log_cstr((p) as *const _)));
        return;
    }
    if !(*ictx).palette.is_null() {
        (*(*ictx).palette).bg = c;
        if !wp.is_null() {
            (*wp).flags |= PANE_STYLECHANGED | PANE_THEMECHANGED;
        }
        screen_write_fullredraw(&raw mut (*ictx).ctx);
    }
}
unsafe fn input_osc_111(mut ictx: *mut input_ctx, mut p: *const ::core::ffi::c_char) {
    let mut wp: *mut window_pane = (*ictx).wp;
    if *p as ::core::ffi::c_int != '\0' as i32 {
        return;
    }
    if !(*ictx).palette.is_null() {
        (*(*ictx).palette).bg = 8 as ::core::ffi::c_int;
        if !wp.is_null() {
            (*wp).flags |= PANE_STYLECHANGED | PANE_THEMECHANGED;
        }
        screen_write_fullredraw(&raw mut (*ictx).ctx);
    }
}
unsafe fn input_osc_12(mut ictx: *mut input_ctx, mut p: *const ::core::ffi::c_char) {
    let mut wp: *mut window_pane = (*ictx).wp;
    let mut c: ::core::ffi::c_int = 0;
    if strcmp(p, b"?\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int {
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
    c = colour_parse_x11_logged(std::ffi::CStr::from_ptr(p)).unwrap_or(-1);
    if c == -(1 as ::core::ffi::c_int) {
        log_debug(format_args!("bad OSC 12: {}", log_cstr((p) as *const _)));
        return;
    }
    screen_set_cursor_colour(&mut *(*ictx).ctx.s, c);
}
unsafe fn input_osc_112(mut ictx: *mut input_ctx, mut p: *const ::core::ffi::c_char) {
    if *p as ::core::ffi::c_int == '\0' as i32 {
        screen_set_cursor_colour(&mut *(*ictx).ctx.s, -(1 as ::core::ffi::c_int));
    }
}
unsafe fn input_osc_133_exit_status(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != ';' as i32
        || *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '\0' as i32
        || *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '=' as i32
    {
        return 0 as ::core::ffi::c_int;
    }

    let tail = std::ffi::CStr::from_ptr(p.add(2)).to_bytes();
    let end = tail
        .iter()
        .position(|&byte| byte == b';')
        .unwrap_or(tail.len());
    if end == 0 {
        return 0 as ::core::ffi::c_int;
    }
    let number = &tail[..end];
    if number.contains(&b'=') {
        return 0 as ::core::ffi::c_int;
    }

    // The token came from a C string, so it has no interior NUL. strtonum
    // borrows this terminated copy only for the duration of the call.
    let copy = CString::new(number).expect("OSC 133 status token contains no NUL");
    let status = strtonum(
        copy.as_ptr(),
        0 as ::core::ffi::c_longlong,
        255 as ::core::ffi::c_longlong,
        &raw mut errstr,
    );
    if !errstr.is_null() {
        return 255 as ::core::ffi::c_int;
    }
    return status as ::core::ffi::c_int;
}

#[cfg(test)]
mod osc_133_exit_status_tests {
    use super::input_osc_133_exit_status;

    #[test]
    fn preserves_numeric_and_parameter_syntax() {
        for (input, expected) in [
            (b"D\0".as_slice(), 0),
            (b"D;\0", 0),
            (b"D;;42\0", 0),
            (b"D;=42\0", 0),
            (b"D;42=ignored\0", 0),
            (b"D;42;k=v\0", 42),
            (b"D;0\0", 0),
            (b"D;255\0", 255),
            (b"D;+7\0", 7),
            (b"D; 7\0", 7),
            (b"D;256\0", 255),
            (b"D;-1\0", 255),
            (b"D;abc\0", 255),
            (b"D;4x\0", 255),
            (b"D;12\0;99\0", 12),
            (b"D;\xff\0", 255),
        ] {
            let actual = unsafe { input_osc_133_exit_status(input.as_ptr().cast()) };
            assert_eq!(actual, expected, "input {input:?}");
        }
    }
}
unsafe fn input_fire_command_event(mut wp: *mut window_pane, mut name: *const ::core::ffi::c_char) {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut tstart: time_t = (*wp).cmd_start_time;
    let mut end: time_t = 0;
    let mut tend: time_t = (*wp).cmd_end_time;
    ep = event_payload_create();
    cmd_find_from_pane(&raw mut fs, wp, 0 as ::core::ffi::c_int);
    event_payload_set_target(ep, &raw mut fs);
    if !fs.s.is_null() {
        event_payload_set_session(
            ep,
            b"session\0" as *const u8 as *const ::core::ffi::c_char,
            fs.s,
        );
    }
    if !fs.wl.is_null() {
        event_payload_set_int(
            ep,
            b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
            (*fs.wl).idx,
        );
    }
    event_payload_set_window(
        ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).window as *mut window,
    );
    event_payload_set_pane(ep, b"pane\0" as *const u8 as *const ::core::ffi::c_char, wp);
    if (*wp).cmd_status != -(1 as ::core::ffi::c_int) {
        event_payload_set_int(
            ep,
            b"command_status\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp).cmd_status,
        );
    }
    if tstart != 0 as time_t {
        event_payload_set_time(
            ep,
            b"command_start_time\0" as *const u8 as *const ::core::ffi::c_char,
            tstart,
        );
    }
    if tend != 0 as time_t {
        event_payload_set_time(
            ep,
            b"command_end_time\0" as *const u8 as *const ::core::ffi::c_char,
            tend,
        );
    }
    if tstart != 0 as time_t {
        if (*wp).flags & PANE_CMDRUNNING != 0 {
            end = time(::core::ptr::null_mut::<time_t>());
        } else {
            end = tend;
        }
        if end < tstart {
            end = tstart;
        }
        end -= tstart;
        event_payload_set_uint(
            ep,
            b"command_duration\0" as *const u8 as *const ::core::ffi::c_char,
            end as u_int,
        );
    }
    events_fire(name, ep);
}
unsafe fn input_osc_133(mut ictx: *mut input_ctx, mut p: *const ::core::ffi::c_char) {
    let mut wp: *mut window_pane = (*ictx).wp;
    let mut s: *mut screen = (*ictx).ctx.s;
    let mut gd: *mut grid = (*s).grid_mut();
    let mut line: u_int = (*s).cy.wrapping_add((*gd).hsize);
    let mut cp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut status: ::core::ffi::c_int = 0;
    let line = (line < (*gd).hsize.wrapping_add((*gd).sy)).then_some(line);
    match *p as ::core::ffi::c_int {
        65 | 78 => {
            if let Some(line) = line {
                let gl = grid_get_line_mut(&mut *gd, line);
                gl.osc133_data = osc133_data::default();
                gl.osc133_data.prompt_col = (*s).cx as u_short;
                gl.flags = (gl.flags as ::core::ffi::c_int | GRID_LINE_START_PROMPT) as u_short;
            }
            if !wp.is_null() {
                (*wp).last_prompt_time = time(::core::ptr::null_mut::<time_t>());
                events_fire_pane(
                    b"pane-shell-prompt\0" as *const u8 as *const ::core::ffi::c_char,
                    wp,
                );
            }
        }
        80 => {
            if let Some(line) = line {
                let gl = grid_get_line_mut(&mut *gd, line);
                cp = strstr(p, b";k=s\0" as *const u8 as *const ::core::ffi::c_char);
                if !cp.is_null()
                    && (*cp.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ';' as i32
                        || *cp.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '\0' as i32)
                {
                    gl.flags =
                        (gl.flags as ::core::ffi::c_int | GRID_LINE_SECOND_PROMPT) as u_short;
                } else {
                    gl.flags = (gl.flags as ::core::ffi::c_int | GRID_LINE_START_PROMPT) as u_short;
                }
                gl.osc133_data.prompt_col = (*s).cx as u_short;
            }
        }
        66 | 73 => {
            if let Some(line) = line {
                let gl = grid_get_line_mut(&mut *gd, line);
                gl.flags = (gl.flags as ::core::ffi::c_int | GRID_LINE_START_COMMAND) as u_short;
                gl.osc133_data.cmd_col = (*s).cx as u_short;
            }
        }
        67 => {
            if let Some(line) = line {
                let gl = grid_get_line_mut(&mut *gd, line);
                gl.flags = (gl.flags as ::core::ffi::c_int | GRID_LINE_START_OUTPUT) as u_short;
                gl.osc133_data.out_start_col = (*s).cx as u_short;
            }
            if !wp.is_null() {
                (*wp).cmd_start_time = time(::core::ptr::null_mut::<time_t>());
                (*wp).cmd_end_time = 0 as time_t;
                (*wp).flags |= PANE_CMDRUNNING;
                (*wp).cmd_status = -(1 as ::core::ffi::c_int);
                input_fire_command_event(
                    wp,
                    b"pane-command-started\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
        }
        68 => {
            status = input_osc_133_exit_status(p);
            if !wp.is_null() {
                (*wp).cmd_end_time = time(::core::ptr::null_mut::<time_t>());
                (*wp).flags &= !PANE_CMDRUNNING;
                (*wp).cmd_status = status;
                input_fire_command_event(
                    wp,
                    b"pane-command-finished\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            if let Some(line) = line {
                let gl = grid_get_line_mut(&mut *gd, line);
                gl.flags = (gl.flags as ::core::ffi::c_int | GRID_LINE_END_OUTPUT) as u_short;
                gl.osc133_data.out_end_col = (*s).cx as u_short;
                gl.osc133_data.exit_status = status as u_char;
            }
        }
        _ => {}
    };
}
unsafe fn input_osc_52_reply(mut ictx: *mut input_ctx, mut clip: ::core::ffi::c_char) {
    let mut ev: *mut bufferevent = (*ictx).event;
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut state: ::core::ffi::c_int = 0;
    let mut buf: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    state = options_get_number(
        global_options,
        b"get-clipboard\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    if state == 0 as ::core::ffi::c_int {
        return;
    }
    if state == 1 as ::core::ffi::c_int {
        pb = paste_get_top(None);
        if pb.is_null() {
            return;
        }
        buf = paste_buffer_data(pb, &raw mut len);
        if (*ictx).input_end as ::core::ffi::c_uint
            == INPUT_END_BEL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            input_reply_clipboard(
                ev,
                buf,
                len,
                b"\x07\0" as *const u8 as *const ::core::ffi::c_char,
                clip,
            );
        } else {
            input_reply_clipboard(
                ev,
                buf,
                len,
                b"\x1B\\\0" as *const u8 as *const ::core::ffi::c_char,
                clip,
            );
        }
        return;
    }
    input_add_request(
        ictx,
        INPUT_REQUEST_CLIPBOARD,
        (*ictx).input_end as ::core::ffi::c_int,
    );
}
unsafe fn input_osc_52_parse(
    mut ictx: *mut input_ctx,
    mut p: *const ::core::ffi::c_char,
    mut clip: *mut ::core::ffi::c_char,
) -> Option<Vec<u8>> {
    let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    let mut allow: *const ::core::ffi::c_char =
        b"cpqs01234567\0" as *const u8 as *const ::core::ffi::c_char;
    let mut i: u_int = 0;
    let mut j: u_int = 0 as u_int;
    if options_get_number(
        global_options,
        b"set-clipboard\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 2 as ::core::ffi::c_longlong
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
    log_debug(format_args!(
        "{}: {}",
        "input_osc_52_parse",
        log_cstr((end) as *const _)
    ));
    i = 0 as u_int;
    while p.offset(i as isize) != end {
        if !strchr(allow, *p.offset(i as isize) as ::core::ffi::c_int).is_null()
            && strchr(clip, *p.offset(i as isize) as ::core::ffi::c_int).is_null()
        {
            let fresh3 = j;
            j = j.wrapping_add(1);
            *clip.offset(fresh3 as isize) = *p.offset(i as isize);
        }
        i = i.wrapping_add(1);
    }
    log_debug(format_args!(
        "{}: {} {}",
        "input_osc_52_parse",
        log_cstr_n(
            (p) as *const _,
            (end.offset_from(p) as ::core::ffi::c_long - 1 as ::core::ffi::c_long)
                as ::core::ffi::c_int
        ),
        log_cstr((clip) as *const _)
    ));
    if strcmp(end, b"?\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int {
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
    let mut out = vec![0; len];
    let outlen = __b64_pton(end, out.as_mut_ptr(), len);
    if outlen == -(1 as ::core::ffi::c_int) {
        return None;
    }
    out.truncate(outlen as usize);
    Some(out)
}
unsafe fn input_osc_52(mut ictx: *mut input_ctx, mut p: *const ::core::ffi::c_char) {
    let mut wp: *mut window_pane = (*ictx).wp;
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: ::core::ptr::null_mut::<window_pane>(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
    };
    let mut clip: [::core::ffi::c_char; 13] = ::core::mem::transmute::<
        [u8; 13],
        [::core::ffi::c_char; 13],
    >(*b"\0\0\0\0\0\0\0\0\0\0\0\0\0");
    let Some(mut out) = input_osc_52_parse(ictx, p, clip.as_mut_ptr()) else {
        return;
    };
    if wp.is_null() {
        if (*ictx).c.is_null() {
            return;
        }
        tty_set_selection(
            &raw mut (*(*ictx).c).tty,
            &raw mut clip as *mut ::core::ffi::c_char,
            out.as_ptr().cast(),
            out.len(),
        );
        paste_add_owned(None, out.into_boxed_slice());
    } else {
        screen_write_start_pane(&raw mut ctx, wp, ::core::ptr::null_mut::<screen>());
        screen_write_setselection(
            &raw mut ctx,
            &raw mut clip as *mut ::core::ffi::c_char,
            out.as_mut_ptr(),
            out.len() as u_int,
        );
        screen_write_stop(&raw mut ctx);
        events_fire_pane(
            b"pane-set-clipboard\0" as *const u8 as *const ::core::ffi::c_char,
            wp,
        );
        paste_add_owned(None, out.into_boxed_slice());
    };
}
unsafe fn input_osc_104(mut ictx: *mut input_ctx, mut p: *const ::core::ffi::c_char) {
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut idx: ::core::ffi::c_long = 0;
    let mut bad: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut redraw: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if *p as ::core::ffi::c_int == '\0' as i32 {
        colour_palette_clear((*ictx).palette);
        screen_write_fullredraw(&raw mut (*ictx).ctx);
        return;
    }
    let mut copy = std::ffi::CStr::from_ptr(p).to_bytes_with_nul().to_vec();
    s = copy.as_mut_ptr().cast();
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
                (*ictx).palette,
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
        log_debug(format_args!("bad OSC 104: {}", log_cstr((p) as *const _)));
    }
    if redraw != 0 {
        screen_write_fullredraw(&raw mut (*ictx).ctx);
    }
}
pub unsafe fn input_reply_clipboard(
    mut bev: *mut bufferevent,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut end: *const ::core::ffi::c_char,
    mut clip: ::core::ffi::c_char,
) {
    let mut out = Vec::<u8>::new();
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
        out.resize(outlen as usize, 0);
        outlen = __b64_ntop(
            buf as *const ::core::ffi::c_uchar,
            len,
            out.as_mut_ptr().cast(),
            outlen as size_t,
        );
        if outlen == -(1 as ::core::ffi::c_int) {
            return;
        }
    }
    bufferevent_write(
        bev,
        b"\x1B]52;\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
        5 as size_t,
    );
    if clip as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        bufferevent_write(
            bev,
            &raw mut clip as *const ::core::ffi::c_void,
            1 as size_t,
        );
    }
    bufferevent_write(
        bev,
        b";\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
        1 as size_t,
    );
    if outlen != 0 as ::core::ffi::c_int {
        bufferevent_write(bev, out.as_ptr().cast(), outlen as size_t);
    }
    bufferevent_write(bev, end as *const ::core::ffi::c_void, strlen(end));
}
pub unsafe fn input_set_buffer_size(mut buffer_size: size_t) {
    log_debug(format_args!(
        "{}: {} -> {}",
        "input_set_buffer_size",
        (input_buffer_size) as ::core::ffi::c_ulong,
        (buffer_size) as ::core::ffi::c_ulong
    ));
    input_buffer_size = buffer_size;
}
unsafe fn input_request_timer_callback(mut arg: *mut ::core::ffi::c_void) {
    let mut ictx: *mut input_ctx = arg as *mut input_ctx;
    let mut t: uint64_t = get_timer();
    for ir in input_ctx_request_handles(ictx) {
        // Sending a queued reply can reenter input processing. Confirm that
        // this stable handle still belongs to the owner before dereferencing.
        if !input_ctx_requests(ictx)
            .iter()
            .any(|owner| std::ptr::eq(&**owner, ir))
        {
            continue;
        }
        if !((*ir).t >= t.wrapping_sub(INPUT_REQUEST_TIMEOUT as uint64_t)) {
            if (*ir).type_0 as ::core::ffi::c_uint
                == INPUT_REQUEST_QUEUE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                input_send_reply(
                    (*ir).ictx,
                    (*ir)
                        .data
                        .as_ref()
                        .expect("queued input request has no reply data")
                        .as_ptr(),
                );
            }
            input_free_request(ir);
        }
    }
    if (*ictx).request_count != 0 as u_int {
        input_start_request_timer(ictx);
    }
}
unsafe fn input_start_request_timer(mut ictx: *mut input_ctx) {
    let mut tv: timeval = timeval {
        tv_sec: 0 as __time_t,
        tv_usec: 100000 as __suseconds_t,
    };
    event_del(&raw mut (*ictx).request_timer);
    event_add(&raw mut (*ictx).request_timer, &raw mut tv);
}
unsafe fn input_make_request(
    mut ictx: *mut input_ctx,
    mut type_0: input_request_type,
) -> *mut input_request {
    let mut owner = input_request::new();
    let ir = &mut *owner as *mut input_request;
    (*ir).type_0 = type_0;
    (*ir).ictx = ictx;
    (*ir).t = get_timer();
    (*ictx).request_count = (*ictx).request_count.wrapping_add(1);
    if (*ictx).request_count == 1 as u_int {
        input_start_request_timer(ictx);
    }
    input_ctx_requests(ictx).push_back(owner);
    return ir;
}
unsafe fn input_free_request(mut ir: *mut input_request) {
    let mut ictx: *mut input_ctx = (*ir).ictx;
    if !(*ir).c.is_null() {
        let c_requests = input_client_requests((*ir).c);
        let index = c_requests
            .iter()
            .position(|request| *request == ir)
            .expect("request missing from its client handle collection");
        c_requests.remove(index);
    }
    (*ictx).request_count = (*ictx).request_count.wrapping_sub(1);
    let requests = input_ctx_requests(ictx);
    let index = requests
        .iter()
        .position(|owner| std::ptr::eq(&**owner, ir))
        .expect("request missing from its input context owner");
    drop(requests.remove(index).unwrap());
}
unsafe fn input_add_request(
    mut ictx: *mut input_ctx,
    mut type_0: input_request_type,
    mut idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = (*ictx).wp;
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    let mut ir: *mut input_request = ::core::ptr::null_mut::<input_request>();
    let mut s: [::core::ffi::c_char; 64] = [0; 64];
    if wp.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    w = (*wp).window as *mut window;
    loop_0 = clients.first();
    while !loop_0.is_null() {
        if !((*loop_0).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0) {
            if !((*loop_0).session.is_null() || session_has((*loop_0).session, w) == 0) {
                if !(!(*loop_0).tty.flags & TTY_STARTED != 0) {
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
        }
        loop_0 = clients.next(loop_0);
    }
    if c.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    ir = input_make_request(ictx, type_0);
    (*ir).c = c;
    (*ir).idx = idx;
    (*ir).end = (*ictx).input_end;
    input_client_requests(c).push(ir);
    match type_0 as ::core::ffi::c_uint {
        0 => {
            xformat(&mut s, format_args!("\x1B]4;{};?\x1B\\", idx as i32));
            tty_puts(&raw mut (*c).tty, &raw mut s as *mut ::core::ffi::c_char);
        }
        1 => {
            tty_putcode_ss(
                &raw mut (*c).tty,
                TTYC_MS,
                b"\0" as *const u8 as *const ::core::ffi::c_char,
                b"?\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        2 | _ => {}
    }
    return 0 as ::core::ffi::c_int;
}
unsafe fn input_request_palette_reply(
    mut ir: *mut input_request,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut pd: *mut input_request_palette_data = data as *mut input_request_palette_data;
    input_osc_colour_reply(
        (*ir).ictx,
        0 as ::core::ffi::c_int,
        4 as u_int,
        (*pd).idx,
        (*pd).c,
        (*ir).end,
    );
}
unsafe fn input_request_clipboard_reply(
    mut ir: *mut input_request,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut ictx: *mut input_ctx = (*ir).ictx;
    let mut ev: *mut bufferevent = (*ictx).event;
    let cd = &*data.cast::<input_request_clipboard_data>();
    let mut state: ::core::ffi::c_int = 0;
    state = options_get_number(
        global_options,
        b"get-clipboard\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    if state == 0 as ::core::ffi::c_int || state == 1 as ::core::ffi::c_int {
        return;
    }
    if state == 3 as ::core::ffi::c_int && !cd.data.is_empty() {
        let owned: Box<[u8]> = cd.data.as_slice().into();
        paste_add_owned(None, owned);
    }
    if (*ir).idx == INPUT_END_BEL as ::core::ffi::c_int {
        input_reply_clipboard(
            ev,
            cd.data.as_ptr().cast(),
            cd.data.len(),
            b"\x07\0" as *const u8 as *const ::core::ffi::c_char,
            cd.clip,
        );
    } else {
        input_reply_clipboard(
            ev,
            cd.data.as_ptr().cast(),
            cd.data.len(),
            b"\x1B\\\0" as *const u8 as *const ::core::ffi::c_char,
            cd.clip,
        );
    };
}
pub unsafe fn input_request_reply(
    mut c: *mut client,
    mut type_0: input_request_type,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut found: *mut input_request = ::core::ptr::null_mut::<input_request>();
    let mut pd: *mut input_request_palette_data = data as *mut input_request_palette_data;
    let mut complete: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    for ir in input_client_requests(c).clone() {
        if (*ir).type_0 as ::core::ffi::c_uint != type_0 as ::core::ffi::c_uint {
            input_free_request(ir);
        } else if type_0 as ::core::ffi::c_uint
            == INPUT_REQUEST_PALETTE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if (*pd).idx != (*ir).idx {
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
    // `found` is freed when its reply is handled. Keep the context separately
    // for later queued replies so the loop never reads through that freed box.
    let ictx = (*found).ictx;
    for ir in input_ctx_request_handles(ictx) {
        if !input_ctx_requests(ictx)
            .iter()
            .any(|owner| std::ptr::eq(&**owner, ir))
        {
            continue;
        }
        if complete != 0
            && (*ir).type_0 as ::core::ffi::c_uint
                != INPUT_REQUEST_QUEUE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            break;
        }
        if (*ir).type_0 as ::core::ffi::c_uint
            == INPUT_REQUEST_QUEUE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            input_send_reply(
                (*ir).ictx,
                (*ir)
                    .data
                    .as_ref()
                    .expect("queued input request has no reply data")
                    .as_ptr(),
            );
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
pub unsafe fn input_cancel_requests(mut c: *mut client) {
    for ir in input_client_requests(c).clone() {
        input_free_request(ir);
    }
}
unsafe fn input_report_current_theme(mut ictx: *mut input_ctx) {
    let mut wp: *mut window_pane = (*ictx).wp;
    if !wp.is_null() {
        (*wp).last_theme = window_pane_get_theme(wp);
        (*wp).flags &= !PANE_THEMECHANGED;
        match (*wp).last_theme as ::core::ffi::c_uint {
            2 => {
                log_debug(format_args!(
                    "{}: %{} dark theme",
                    "input_report_current_theme",
                    ((*wp).id) as u32
                ));
                input_reply(ictx, 0 as ::core::ffi::c_int, |out| {
                    out.write_all(b"\x1B[?997;1n")
                });
            }
            1 => {
                log_debug(format_args!(
                    "{}: %{} light theme",
                    "input_report_current_theme",
                    ((*wp).id) as u32
                ));
                input_reply(ictx, 0 as ::core::ffi::c_int, |out| {
                    out.write_all(b"\x1B[?997;2n")
                });
            }
            0 => {
                log_debug(format_args!(
                    "{}: %{} unknown theme",
                    "input_report_current_theme",
                    ((*wp).id) as u32
                ));
            }
            _ => {}
        }
    }
}
