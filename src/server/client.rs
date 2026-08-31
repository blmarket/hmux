use super::message::server_check_unattached;
use super::message::{
    server_destroy_pane, server_redraw_client, server_redraw_window_borders, server_status_client,
    server_status_window,
};
use super::run::client_walk;
use super::run::{clients, current_time, server_proc};
use super::run::{server_add_accept, server_update_socket};
use crate::alerts::alerts_check_session;
use crate::arguments::args_from_vector;
use crate::cfg::start_cfg;
use crate::cfg::{cfg_client, cfg_finished};
use crate::cmd::cmd_parse_from_arguments;
use crate::cmd::{cmd_display_panes_draw, cmd_display_panes_free_box, cmd_display_panes_key};
use crate::cmd::{cmd_find_from_client, cmd_find_from_mouse};
use crate::cmd::{cmd_list_all_have, cmd_unpack_argv};
use crate::cmd::{
    cmdq_append, cmdq_error, cmdq_get_callback1, cmdq_get_client, cmdq_get_command, cmdq_get_error,
    cmdq_insert_after, cmdq_new,
};
use crate::compat::imsg_get_fd;
use crate::control::{
    control_all_done, control_discard, control_pane_offset, control_ready, control_reset_offsets,
    control_start, control_stop, control_write,
};
use crate::environ::{environ_entry_value, environ_find, environ_ptr, environ_put, environ_t};
use crate::ffi::{access, close, gettimeofday, isatty, sscanf, strchr, strcmp, strlen, ttyname};
use crate::file::{file_fire_done, file_print, file_read_data, file_read_done, file_write_ready};
use crate::fmt_args;
use crate::fmt_engine::format_alloc;
use crate::format::{format_create, format_defaults, format_expand_time, format_lost_client};
use crate::input::input_cancel_requests;
use crate::key_bindings::{
    key_binding_flags, key_binding_key, key_bindings_dispatch, key_bindings_get,
    key_bindings_get_table_ref, key_table_activity_time, key_table_name,
    key_table_set_activity_time,
};
use crate::log::{fatal, log_debug, log_get_level};
use crate::modes::window_copy_add;
use crate::names::check_window_name;
use crate::notify::notify_client;
use crate::options::{
    options_get_command, options_get_number, options_get_string, options_set_number,
};
use crate::overlay::{
    menu_check_cb, menu_draw_cb, menu_free_box, menu_key_cb, menu_mode_cb, menu_resize_cb,
};
use crate::overlay::{
    popup_check_cb, popup_draw_cb, popup_free_box, popup_key_cb, popup_mode_cb, popup_resize_cb,
};
use crate::proc::{peer_ptr, proc_add_peer, proc_kill_peer, proc_remove_peer, proc_send};
use crate::reactor;
use crate::reactor::{Interest, IoWatch, Reactor, Timer};
use crate::resize::recalculate_sizes;
use crate::resize::{recalculate_size, resize_window};
use crate::screen::screen_mode_to_string;
use crate::screen::screen_redraw_is_visible;
use crate::screen::{screen_redraw_get_visible_ranges, screen_redraw_pane, screen_redraw_screen};
use crate::session::session_ref_from_ptr;
use crate::session::{
    session_attached, session_cwd, session_get_curw, session_id, session_name_owned,
    session_options,
};
use crate::session::{session_find_by_id, session_theme_changed, session_update_activity};
use crate::status::{
    status_at_line, status_free, status_get_range, status_init, status_line_size,
    status_message_clear, status_prompt_clear, status_prompt_key, status_prompt_line_at,
    status_timer_start,
};
use crate::terminfo::tty_get_features;
use crate::terminfo::{tty_term_has, tty_term_of};
use crate::text::key_string_lookup_key;
use crate::text::{utf8_sanitize, utf8_stravisx};
use crate::tmux::{checkshell, find_home, setblocking};
use crate::tmux::{global_options, global_s_options};
use crate::tree::GlobalTree;
use crate::tty::{
    TTY_OPENED, tty_close, tty_cursor, tty_free, tty_init, tty_margin_off, tty_open,
    tty_region_off, tty_repeat_requests, tty_reset, tty_resize, tty_send_requests, tty_set_path,
    tty_set_progress_bar, tty_set_title, tty_start_tty, tty_stop_tty, tty_sync_end,
    tty_update_client_offset, tty_update_mode, tty_window_offset,
};
pub use crate::types::*;
use crate::window::pane_walk;
use crate::window::window_get_active;
use crate::window::window_pane_current_mode;
use crate::window::window_ref_from_ptr;
use crate::window::winlinks_into;
use crate::window::{
    window_get_active_at, window_pane_border_status_get_range, window_pane_find_by_id,
    window_pane_get_new_data, window_pane_is_floating, window_pane_key, window_pane_paste,
    window_pane_send_resize, window_pane_send_theme_update, window_pane_set_mode,
    window_pane_show_scrollbar, window_panes_first, window_panes_next, window_redraw_active_switch,
    window_refs, window_set_active_pane, window_update_focus, winlink_find_by_index,
};
use crate::window::{window_get_latest, window_set_latest};
use crate::xmalloc::xasprintf;
use ::core::ffi::CStr;
use ::std::ffi::CString;
pub const BUFFER_EOL_NUL: ::core::ffi::c_uint = 4;
pub const BUFFER_EOL_LF: ::core::ffi::c_uint = 3;
pub const BUFFER_EOL_CRLF_STRICT: ::core::ffi::c_uint = 2;
pub const BUFFER_EOL_CRLF: ::core::ffi::c_uint = 1;
pub const BUFFER_EOL_ANY: ::core::ffi::c_uint = 0;
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
pub const KEYC_TYPE_NOTYPE: key_code_type = 13;
pub const KEYC_TYPE_TRIPLECLICK: key_code_type = 12;
pub const KEYC_TYPE_DOUBLECLICK: key_code_type = 11;
pub const KEYC_TYPE_SECONDCLICK: key_code_type = 10;
pub const KEYC_TYPE_WHEELUP: key_code_type = 9;
pub const KEYC_TYPE_WHEELDOWN: key_code_type = 8;
pub const KEYC_TYPE_MOUSEDRAGEND: key_code_type = 7;
pub const KEYC_TYPE_MOUSEDRAG: key_code_type = 6;
pub const KEYC_TYPE_MOUSEUP: key_code_type = 5;
pub const KEYC_TYPE_MOUSEDOWN: key_code_type = 4;
pub const KEYC_TYPE_MOUSEMOVE: key_code_type = 3;
pub const KEYC_TYPE_FUNCTION: key_code_type = 2;
pub const KEYC_TYPE_USER: key_code_type = 1;
pub const KEYC_TYPE_UNICODE: key_code_type = 0;
pub type key_code_mouse_location = ::core::ffi::c_uint;
pub const KEYC_MOUSE_LOCATION_NOWHERE: key_code_mouse_location = 19;
pub const KEYC_MOUSE_LOCATION_CONTROL9: key_code_mouse_location = 18;
pub const KEYC_MOUSE_LOCATION_CONTROL8: key_code_mouse_location = 17;
pub const KEYC_MOUSE_LOCATION_CONTROL7: key_code_mouse_location = 16;
pub const KEYC_MOUSE_LOCATION_CONTROL6: key_code_mouse_location = 15;
pub const KEYC_MOUSE_LOCATION_CONTROL5: key_code_mouse_location = 14;
pub const KEYC_MOUSE_LOCATION_CONTROL4: key_code_mouse_location = 13;
pub const KEYC_MOUSE_LOCATION_CONTROL3: key_code_mouse_location = 12;
pub const KEYC_MOUSE_LOCATION_CONTROL2: key_code_mouse_location = 11;
pub const KEYC_MOUSE_LOCATION_CONTROL1: key_code_mouse_location = 10;
pub const KEYC_MOUSE_LOCATION_CONTROL0: key_code_mouse_location = 9;
pub const KEYC_MOUSE_LOCATION_SCROLLBAR_DOWN: key_code_mouse_location = 8;
pub const KEYC_MOUSE_LOCATION_SCROLLBAR_SLIDER: key_code_mouse_location = 7;
pub const KEYC_MOUSE_LOCATION_SCROLLBAR_UP: key_code_mouse_location = 6;
pub const KEYC_MOUSE_LOCATION_BORDER: key_code_mouse_location = 5;
pub const KEYC_MOUSE_LOCATION_STATUS_DEFAULT: key_code_mouse_location = 4;
pub const KEYC_MOUSE_LOCATION_STATUS_RIGHT: key_code_mouse_location = 3;
pub const KEYC_MOUSE_LOCATION_STATUS_LEFT: key_code_mouse_location = 2;
pub const KEYC_MOUSE_LOCATION_STATUS: key_code_mouse_location = 1;
pub const KEYC_MOUSE_LOCATION_PANE: key_code_mouse_location = 0;
pub type keyc = ::core::ffi::c_ulong;
pub const KEYC_TRIPLECLICK11_CONTROL9: keyc = 51539610386;
pub const KEYC_TRIPLECLICK10_CONTROL9: keyc = 51539610130;
pub const KEYC_TRIPLECLICK9_CONTROL9: keyc = 51539609874;
pub const KEYC_TRIPLECLICK8_CONTROL9: keyc = 51539609618;
pub const KEYC_TRIPLECLICK7_CONTROL9: keyc = 51539609362;
pub const KEYC_TRIPLECLICK6_CONTROL9: keyc = 51539609106;
pub const KEYC_TRIPLECLICK3_CONTROL9: keyc = 51539608338;
pub const KEYC_TRIPLECLICK2_CONTROL9: keyc = 51539608082;
pub const KEYC_TRIPLECLICK1_CONTROL9: keyc = 51539607826;
pub const KEYC_TRIPLECLICK_CONTROL9: keyc = 51539607570;
pub const KEYC_TRIPLECLICK11_CONTROL8: keyc = 51539610385;
pub const KEYC_TRIPLECLICK10_CONTROL8: keyc = 51539610129;
pub const KEYC_TRIPLECLICK9_CONTROL8: keyc = 51539609873;
pub const KEYC_TRIPLECLICK8_CONTROL8: keyc = 51539609617;
pub const KEYC_TRIPLECLICK7_CONTROL8: keyc = 51539609361;
pub const KEYC_TRIPLECLICK6_CONTROL8: keyc = 51539609105;
pub const KEYC_TRIPLECLICK3_CONTROL8: keyc = 51539608337;
pub const KEYC_TRIPLECLICK2_CONTROL8: keyc = 51539608081;
pub const KEYC_TRIPLECLICK1_CONTROL8: keyc = 51539607825;
pub const KEYC_TRIPLECLICK_CONTROL8: keyc = 51539607569;
pub const KEYC_TRIPLECLICK11_CONTROL7: keyc = 51539610384;
pub const KEYC_TRIPLECLICK10_CONTROL7: keyc = 51539610128;
pub const KEYC_TRIPLECLICK9_CONTROL7: keyc = 51539609872;
pub const KEYC_TRIPLECLICK8_CONTROL7: keyc = 51539609616;
pub const KEYC_TRIPLECLICK7_CONTROL7: keyc = 51539609360;
pub const KEYC_TRIPLECLICK6_CONTROL7: keyc = 51539609104;
pub const KEYC_TRIPLECLICK3_CONTROL7: keyc = 51539608336;
pub const KEYC_TRIPLECLICK2_CONTROL7: keyc = 51539608080;
pub const KEYC_TRIPLECLICK1_CONTROL7: keyc = 51539607824;
pub const KEYC_TRIPLECLICK_CONTROL7: keyc = 51539607568;
pub const KEYC_TRIPLECLICK11_CONTROL6: keyc = 51539610383;
pub const KEYC_TRIPLECLICK10_CONTROL6: keyc = 51539610127;
pub const KEYC_TRIPLECLICK9_CONTROL6: keyc = 51539609871;
pub const KEYC_TRIPLECLICK8_CONTROL6: keyc = 51539609615;
pub const KEYC_TRIPLECLICK7_CONTROL6: keyc = 51539609359;
pub const KEYC_TRIPLECLICK6_CONTROL6: keyc = 51539609103;
pub const KEYC_TRIPLECLICK3_CONTROL6: keyc = 51539608335;
pub const KEYC_TRIPLECLICK2_CONTROL6: keyc = 51539608079;
pub const KEYC_TRIPLECLICK1_CONTROL6: keyc = 51539607823;
pub const KEYC_TRIPLECLICK_CONTROL6: keyc = 51539607567;
pub const KEYC_TRIPLECLICK11_CONTROL5: keyc = 51539610382;
pub const KEYC_TRIPLECLICK10_CONTROL5: keyc = 51539610126;
pub const KEYC_TRIPLECLICK9_CONTROL5: keyc = 51539609870;
pub const KEYC_TRIPLECLICK8_CONTROL5: keyc = 51539609614;
pub const KEYC_TRIPLECLICK7_CONTROL5: keyc = 51539609358;
pub const KEYC_TRIPLECLICK6_CONTROL5: keyc = 51539609102;
pub const KEYC_TRIPLECLICK3_CONTROL5: keyc = 51539608334;
pub const KEYC_TRIPLECLICK2_CONTROL5: keyc = 51539608078;
pub const KEYC_TRIPLECLICK1_CONTROL5: keyc = 51539607822;
pub const KEYC_TRIPLECLICK_CONTROL5: keyc = 51539607566;
pub const KEYC_TRIPLECLICK11_CONTROL4: keyc = 51539610381;
pub const KEYC_TRIPLECLICK10_CONTROL4: keyc = 51539610125;
pub const KEYC_TRIPLECLICK9_CONTROL4: keyc = 51539609869;
pub const KEYC_TRIPLECLICK8_CONTROL4: keyc = 51539609613;
pub const KEYC_TRIPLECLICK7_CONTROL4: keyc = 51539609357;
pub const KEYC_TRIPLECLICK6_CONTROL4: keyc = 51539609101;
pub const KEYC_TRIPLECLICK3_CONTROL4: keyc = 51539608333;
pub const KEYC_TRIPLECLICK2_CONTROL4: keyc = 51539608077;
pub const KEYC_TRIPLECLICK1_CONTROL4: keyc = 51539607821;
pub const KEYC_TRIPLECLICK_CONTROL4: keyc = 51539607565;
pub const KEYC_TRIPLECLICK11_CONTROL3: keyc = 51539610380;
pub const KEYC_TRIPLECLICK10_CONTROL3: keyc = 51539610124;
pub const KEYC_TRIPLECLICK9_CONTROL3: keyc = 51539609868;
pub const KEYC_TRIPLECLICK8_CONTROL3: keyc = 51539609612;
pub const KEYC_TRIPLECLICK7_CONTROL3: keyc = 51539609356;
pub const KEYC_TRIPLECLICK6_CONTROL3: keyc = 51539609100;
pub const KEYC_TRIPLECLICK3_CONTROL3: keyc = 51539608332;
pub const KEYC_TRIPLECLICK2_CONTROL3: keyc = 51539608076;
pub const KEYC_TRIPLECLICK1_CONTROL3: keyc = 51539607820;
pub const KEYC_TRIPLECLICK_CONTROL3: keyc = 51539607564;
pub const KEYC_TRIPLECLICK11_CONTROL2: keyc = 51539610379;
pub const KEYC_TRIPLECLICK10_CONTROL2: keyc = 51539610123;
pub const KEYC_TRIPLECLICK9_CONTROL2: keyc = 51539609867;
pub const KEYC_TRIPLECLICK8_CONTROL2: keyc = 51539609611;
pub const KEYC_TRIPLECLICK7_CONTROL2: keyc = 51539609355;
pub const KEYC_TRIPLECLICK6_CONTROL2: keyc = 51539609099;
pub const KEYC_TRIPLECLICK3_CONTROL2: keyc = 51539608331;
pub const KEYC_TRIPLECLICK2_CONTROL2: keyc = 51539608075;
pub const KEYC_TRIPLECLICK1_CONTROL2: keyc = 51539607819;
pub const KEYC_TRIPLECLICK_CONTROL2: keyc = 51539607563;
pub const KEYC_TRIPLECLICK11_CONTROL1: keyc = 51539610378;
pub const KEYC_TRIPLECLICK10_CONTROL1: keyc = 51539610122;
pub const KEYC_TRIPLECLICK9_CONTROL1: keyc = 51539609866;
pub const KEYC_TRIPLECLICK8_CONTROL1: keyc = 51539609610;
pub const KEYC_TRIPLECLICK7_CONTROL1: keyc = 51539609354;
pub const KEYC_TRIPLECLICK6_CONTROL1: keyc = 51539609098;
pub const KEYC_TRIPLECLICK3_CONTROL1: keyc = 51539608330;
pub const KEYC_TRIPLECLICK2_CONTROL1: keyc = 51539608074;
pub const KEYC_TRIPLECLICK1_CONTROL1: keyc = 51539607818;
pub const KEYC_TRIPLECLICK_CONTROL1: keyc = 51539607562;
pub const KEYC_TRIPLECLICK11_CONTROL0: keyc = 51539610377;
pub const KEYC_TRIPLECLICK10_CONTROL0: keyc = 51539610121;
pub const KEYC_TRIPLECLICK9_CONTROL0: keyc = 51539609865;
pub const KEYC_TRIPLECLICK8_CONTROL0: keyc = 51539609609;
pub const KEYC_TRIPLECLICK7_CONTROL0: keyc = 51539609353;
pub const KEYC_TRIPLECLICK6_CONTROL0: keyc = 51539609097;
pub const KEYC_TRIPLECLICK3_CONTROL0: keyc = 51539608329;
pub const KEYC_TRIPLECLICK2_CONTROL0: keyc = 51539608073;
pub const KEYC_TRIPLECLICK1_CONTROL0: keyc = 51539607817;
pub const KEYC_TRIPLECLICK_CONTROL0: keyc = 51539607561;
pub const KEYC_TRIPLECLICK11_SCROLLBAR_DOWN: keyc = 51539610376;
pub const KEYC_TRIPLECLICK10_SCROLLBAR_DOWN: keyc = 51539610120;
pub const KEYC_TRIPLECLICK9_SCROLLBAR_DOWN: keyc = 51539609864;
pub const KEYC_TRIPLECLICK8_SCROLLBAR_DOWN: keyc = 51539609608;
pub const KEYC_TRIPLECLICK7_SCROLLBAR_DOWN: keyc = 51539609352;
pub const KEYC_TRIPLECLICK6_SCROLLBAR_DOWN: keyc = 51539609096;
pub const KEYC_TRIPLECLICK3_SCROLLBAR_DOWN: keyc = 51539608328;
pub const KEYC_TRIPLECLICK2_SCROLLBAR_DOWN: keyc = 51539608072;
pub const KEYC_TRIPLECLICK1_SCROLLBAR_DOWN: keyc = 51539607816;
pub const KEYC_TRIPLECLICK_SCROLLBAR_DOWN: keyc = 51539607560;
pub const KEYC_TRIPLECLICK11_SCROLLBAR_SLIDER: keyc = 51539610375;
pub const KEYC_TRIPLECLICK10_SCROLLBAR_SLIDER: keyc = 51539610119;
pub const KEYC_TRIPLECLICK9_SCROLLBAR_SLIDER: keyc = 51539609863;
pub const KEYC_TRIPLECLICK8_SCROLLBAR_SLIDER: keyc = 51539609607;
pub const KEYC_TRIPLECLICK7_SCROLLBAR_SLIDER: keyc = 51539609351;
pub const KEYC_TRIPLECLICK6_SCROLLBAR_SLIDER: keyc = 51539609095;
pub const KEYC_TRIPLECLICK3_SCROLLBAR_SLIDER: keyc = 51539608327;
pub const KEYC_TRIPLECLICK2_SCROLLBAR_SLIDER: keyc = 51539608071;
pub const KEYC_TRIPLECLICK1_SCROLLBAR_SLIDER: keyc = 51539607815;
pub const KEYC_TRIPLECLICK_SCROLLBAR_SLIDER: keyc = 51539607559;
pub const KEYC_TRIPLECLICK11_SCROLLBAR_UP: keyc = 51539610374;
pub const KEYC_TRIPLECLICK10_SCROLLBAR_UP: keyc = 51539610118;
pub const KEYC_TRIPLECLICK9_SCROLLBAR_UP: keyc = 51539609862;
pub const KEYC_TRIPLECLICK8_SCROLLBAR_UP: keyc = 51539609606;
pub const KEYC_TRIPLECLICK7_SCROLLBAR_UP: keyc = 51539609350;
pub const KEYC_TRIPLECLICK6_SCROLLBAR_UP: keyc = 51539609094;
pub const KEYC_TRIPLECLICK3_SCROLLBAR_UP: keyc = 51539608326;
pub const KEYC_TRIPLECLICK2_SCROLLBAR_UP: keyc = 51539608070;
pub const KEYC_TRIPLECLICK1_SCROLLBAR_UP: keyc = 51539607814;
pub const KEYC_TRIPLECLICK_SCROLLBAR_UP: keyc = 51539607558;
pub const KEYC_TRIPLECLICK11_BORDER: keyc = 51539610373;
pub const KEYC_TRIPLECLICK10_BORDER: keyc = 51539610117;
pub const KEYC_TRIPLECLICK9_BORDER: keyc = 51539609861;
pub const KEYC_TRIPLECLICK8_BORDER: keyc = 51539609605;
pub const KEYC_TRIPLECLICK7_BORDER: keyc = 51539609349;
pub const KEYC_TRIPLECLICK6_BORDER: keyc = 51539609093;
pub const KEYC_TRIPLECLICK3_BORDER: keyc = 51539608325;
pub const KEYC_TRIPLECLICK2_BORDER: keyc = 51539608069;
pub const KEYC_TRIPLECLICK1_BORDER: keyc = 51539607813;
pub const KEYC_TRIPLECLICK_BORDER: keyc = 51539607557;
pub const KEYC_TRIPLECLICK11_STATUS_DEFAULT: keyc = 51539610372;
pub const KEYC_TRIPLECLICK10_STATUS_DEFAULT: keyc = 51539610116;
pub const KEYC_TRIPLECLICK9_STATUS_DEFAULT: keyc = 51539609860;
pub const KEYC_TRIPLECLICK8_STATUS_DEFAULT: keyc = 51539609604;
pub const KEYC_TRIPLECLICK7_STATUS_DEFAULT: keyc = 51539609348;
pub const KEYC_TRIPLECLICK6_STATUS_DEFAULT: keyc = 51539609092;
pub const KEYC_TRIPLECLICK3_STATUS_DEFAULT: keyc = 51539608324;
pub const KEYC_TRIPLECLICK2_STATUS_DEFAULT: keyc = 51539608068;
pub const KEYC_TRIPLECLICK1_STATUS_DEFAULT: keyc = 51539607812;
pub const KEYC_TRIPLECLICK_STATUS_DEFAULT: keyc = 51539607556;
pub const KEYC_TRIPLECLICK11_STATUS_RIGHT: keyc = 51539610371;
pub const KEYC_TRIPLECLICK10_STATUS_RIGHT: keyc = 51539610115;
pub const KEYC_TRIPLECLICK9_STATUS_RIGHT: keyc = 51539609859;
pub const KEYC_TRIPLECLICK8_STATUS_RIGHT: keyc = 51539609603;
pub const KEYC_TRIPLECLICK7_STATUS_RIGHT: keyc = 51539609347;
pub const KEYC_TRIPLECLICK6_STATUS_RIGHT: keyc = 51539609091;
pub const KEYC_TRIPLECLICK3_STATUS_RIGHT: keyc = 51539608323;
pub const KEYC_TRIPLECLICK2_STATUS_RIGHT: keyc = 51539608067;
pub const KEYC_TRIPLECLICK1_STATUS_RIGHT: keyc = 51539607811;
pub const KEYC_TRIPLECLICK_STATUS_RIGHT: keyc = 51539607555;
pub const KEYC_TRIPLECLICK11_STATUS_LEFT: keyc = 51539610370;
pub const KEYC_TRIPLECLICK10_STATUS_LEFT: keyc = 51539610114;
pub const KEYC_TRIPLECLICK9_STATUS_LEFT: keyc = 51539609858;
pub const KEYC_TRIPLECLICK8_STATUS_LEFT: keyc = 51539609602;
pub const KEYC_TRIPLECLICK7_STATUS_LEFT: keyc = 51539609346;
pub const KEYC_TRIPLECLICK6_STATUS_LEFT: keyc = 51539609090;
pub const KEYC_TRIPLECLICK3_STATUS_LEFT: keyc = 51539608322;
pub const KEYC_TRIPLECLICK2_STATUS_LEFT: keyc = 51539608066;
pub const KEYC_TRIPLECLICK1_STATUS_LEFT: keyc = 51539607810;
pub const KEYC_TRIPLECLICK_STATUS_LEFT: keyc = 51539607554;
pub const KEYC_TRIPLECLICK11_STATUS: keyc = 51539610369;
pub const KEYC_TRIPLECLICK10_STATUS: keyc = 51539610113;
pub const KEYC_TRIPLECLICK9_STATUS: keyc = 51539609857;
pub const KEYC_TRIPLECLICK8_STATUS: keyc = 51539609601;
pub const KEYC_TRIPLECLICK7_STATUS: keyc = 51539609345;
pub const KEYC_TRIPLECLICK6_STATUS: keyc = 51539609089;
pub const KEYC_TRIPLECLICK3_STATUS: keyc = 51539608321;
pub const KEYC_TRIPLECLICK2_STATUS: keyc = 51539608065;
pub const KEYC_TRIPLECLICK1_STATUS: keyc = 51539607809;
pub const KEYC_TRIPLECLICK_STATUS: keyc = 51539607553;
pub const KEYC_TRIPLECLICK11_PANE: keyc = 51539610368;
pub const KEYC_TRIPLECLICK10_PANE: keyc = 51539610112;
pub const KEYC_TRIPLECLICK9_PANE: keyc = 51539609856;
pub const KEYC_TRIPLECLICK8_PANE: keyc = 51539609600;
pub const KEYC_TRIPLECLICK7_PANE: keyc = 51539609344;
pub const KEYC_TRIPLECLICK6_PANE: keyc = 51539609088;
pub const KEYC_TRIPLECLICK3_PANE: keyc = 51539608320;
pub const KEYC_TRIPLECLICK2_PANE: keyc = 51539608064;
pub const KEYC_TRIPLECLICK1_PANE: keyc = 51539607808;
pub const KEYC_TRIPLECLICK_PANE: keyc = 51539607552;
pub const KEYC_DOUBLECLICK11_CONTROL9: keyc = 47244643090;
pub const KEYC_DOUBLECLICK10_CONTROL9: keyc = 47244642834;
pub const KEYC_DOUBLECLICK9_CONTROL9: keyc = 47244642578;
pub const KEYC_DOUBLECLICK8_CONTROL9: keyc = 47244642322;
pub const KEYC_DOUBLECLICK7_CONTROL9: keyc = 47244642066;
pub const KEYC_DOUBLECLICK6_CONTROL9: keyc = 47244641810;
pub const KEYC_DOUBLECLICK3_CONTROL9: keyc = 47244641042;
pub const KEYC_DOUBLECLICK2_CONTROL9: keyc = 47244640786;
pub const KEYC_DOUBLECLICK1_CONTROL9: keyc = 47244640530;
pub const KEYC_DOUBLECLICK_CONTROL9: keyc = 47244640274;
pub const KEYC_DOUBLECLICK11_CONTROL8: keyc = 47244643089;
pub const KEYC_DOUBLECLICK10_CONTROL8: keyc = 47244642833;
pub const KEYC_DOUBLECLICK9_CONTROL8: keyc = 47244642577;
pub const KEYC_DOUBLECLICK8_CONTROL8: keyc = 47244642321;
pub const KEYC_DOUBLECLICK7_CONTROL8: keyc = 47244642065;
pub const KEYC_DOUBLECLICK6_CONTROL8: keyc = 47244641809;
pub const KEYC_DOUBLECLICK3_CONTROL8: keyc = 47244641041;
pub const KEYC_DOUBLECLICK2_CONTROL8: keyc = 47244640785;
pub const KEYC_DOUBLECLICK1_CONTROL8: keyc = 47244640529;
pub const KEYC_DOUBLECLICK_CONTROL8: keyc = 47244640273;
pub const KEYC_DOUBLECLICK11_CONTROL7: keyc = 47244643088;
pub const KEYC_DOUBLECLICK10_CONTROL7: keyc = 47244642832;
pub const KEYC_DOUBLECLICK9_CONTROL7: keyc = 47244642576;
pub const KEYC_DOUBLECLICK8_CONTROL7: keyc = 47244642320;
pub const KEYC_DOUBLECLICK7_CONTROL7: keyc = 47244642064;
pub const KEYC_DOUBLECLICK6_CONTROL7: keyc = 47244641808;
pub const KEYC_DOUBLECLICK3_CONTROL7: keyc = 47244641040;
pub const KEYC_DOUBLECLICK2_CONTROL7: keyc = 47244640784;
pub const KEYC_DOUBLECLICK1_CONTROL7: keyc = 47244640528;
pub const KEYC_DOUBLECLICK_CONTROL7: keyc = 47244640272;
pub const KEYC_DOUBLECLICK11_CONTROL6: keyc = 47244643087;
pub const KEYC_DOUBLECLICK10_CONTROL6: keyc = 47244642831;
pub const KEYC_DOUBLECLICK9_CONTROL6: keyc = 47244642575;
pub const KEYC_DOUBLECLICK8_CONTROL6: keyc = 47244642319;
pub const KEYC_DOUBLECLICK7_CONTROL6: keyc = 47244642063;
pub const KEYC_DOUBLECLICK6_CONTROL6: keyc = 47244641807;
pub const KEYC_DOUBLECLICK3_CONTROL6: keyc = 47244641039;
pub const KEYC_DOUBLECLICK2_CONTROL6: keyc = 47244640783;
pub const KEYC_DOUBLECLICK1_CONTROL6: keyc = 47244640527;
pub const KEYC_DOUBLECLICK_CONTROL6: keyc = 47244640271;
pub const KEYC_DOUBLECLICK11_CONTROL5: keyc = 47244643086;
pub const KEYC_DOUBLECLICK10_CONTROL5: keyc = 47244642830;
pub const KEYC_DOUBLECLICK9_CONTROL5: keyc = 47244642574;
pub const KEYC_DOUBLECLICK8_CONTROL5: keyc = 47244642318;
pub const KEYC_DOUBLECLICK7_CONTROL5: keyc = 47244642062;
pub const KEYC_DOUBLECLICK6_CONTROL5: keyc = 47244641806;
pub const KEYC_DOUBLECLICK3_CONTROL5: keyc = 47244641038;
pub const KEYC_DOUBLECLICK2_CONTROL5: keyc = 47244640782;
pub const KEYC_DOUBLECLICK1_CONTROL5: keyc = 47244640526;
pub const KEYC_DOUBLECLICK_CONTROL5: keyc = 47244640270;
pub const KEYC_DOUBLECLICK11_CONTROL4: keyc = 47244643085;
pub const KEYC_DOUBLECLICK10_CONTROL4: keyc = 47244642829;
pub const KEYC_DOUBLECLICK9_CONTROL4: keyc = 47244642573;
pub const KEYC_DOUBLECLICK8_CONTROL4: keyc = 47244642317;
pub const KEYC_DOUBLECLICK7_CONTROL4: keyc = 47244642061;
pub const KEYC_DOUBLECLICK6_CONTROL4: keyc = 47244641805;
pub const KEYC_DOUBLECLICK3_CONTROL4: keyc = 47244641037;
pub const KEYC_DOUBLECLICK2_CONTROL4: keyc = 47244640781;
pub const KEYC_DOUBLECLICK1_CONTROL4: keyc = 47244640525;
pub const KEYC_DOUBLECLICK_CONTROL4: keyc = 47244640269;
pub const KEYC_DOUBLECLICK11_CONTROL3: keyc = 47244643084;
pub const KEYC_DOUBLECLICK10_CONTROL3: keyc = 47244642828;
pub const KEYC_DOUBLECLICK9_CONTROL3: keyc = 47244642572;
pub const KEYC_DOUBLECLICK8_CONTROL3: keyc = 47244642316;
pub const KEYC_DOUBLECLICK7_CONTROL3: keyc = 47244642060;
pub const KEYC_DOUBLECLICK6_CONTROL3: keyc = 47244641804;
pub const KEYC_DOUBLECLICK3_CONTROL3: keyc = 47244641036;
pub const KEYC_DOUBLECLICK2_CONTROL3: keyc = 47244640780;
pub const KEYC_DOUBLECLICK1_CONTROL3: keyc = 47244640524;
pub const KEYC_DOUBLECLICK_CONTROL3: keyc = 47244640268;
pub const KEYC_DOUBLECLICK11_CONTROL2: keyc = 47244643083;
pub const KEYC_DOUBLECLICK10_CONTROL2: keyc = 47244642827;
pub const KEYC_DOUBLECLICK9_CONTROL2: keyc = 47244642571;
pub const KEYC_DOUBLECLICK8_CONTROL2: keyc = 47244642315;
pub const KEYC_DOUBLECLICK7_CONTROL2: keyc = 47244642059;
pub const KEYC_DOUBLECLICK6_CONTROL2: keyc = 47244641803;
pub const KEYC_DOUBLECLICK3_CONTROL2: keyc = 47244641035;
pub const KEYC_DOUBLECLICK2_CONTROL2: keyc = 47244640779;
pub const KEYC_DOUBLECLICK1_CONTROL2: keyc = 47244640523;
pub const KEYC_DOUBLECLICK_CONTROL2: keyc = 47244640267;
pub const KEYC_DOUBLECLICK11_CONTROL1: keyc = 47244643082;
pub const KEYC_DOUBLECLICK10_CONTROL1: keyc = 47244642826;
pub const KEYC_DOUBLECLICK9_CONTROL1: keyc = 47244642570;
pub const KEYC_DOUBLECLICK8_CONTROL1: keyc = 47244642314;
pub const KEYC_DOUBLECLICK7_CONTROL1: keyc = 47244642058;
pub const KEYC_DOUBLECLICK6_CONTROL1: keyc = 47244641802;
pub const KEYC_DOUBLECLICK3_CONTROL1: keyc = 47244641034;
pub const KEYC_DOUBLECLICK2_CONTROL1: keyc = 47244640778;
pub const KEYC_DOUBLECLICK1_CONTROL1: keyc = 47244640522;
pub const KEYC_DOUBLECLICK_CONTROL1: keyc = 47244640266;
pub const KEYC_DOUBLECLICK11_CONTROL0: keyc = 47244643081;
pub const KEYC_DOUBLECLICK10_CONTROL0: keyc = 47244642825;
pub const KEYC_DOUBLECLICK9_CONTROL0: keyc = 47244642569;
pub const KEYC_DOUBLECLICK8_CONTROL0: keyc = 47244642313;
pub const KEYC_DOUBLECLICK7_CONTROL0: keyc = 47244642057;
pub const KEYC_DOUBLECLICK6_CONTROL0: keyc = 47244641801;
pub const KEYC_DOUBLECLICK3_CONTROL0: keyc = 47244641033;
pub const KEYC_DOUBLECLICK2_CONTROL0: keyc = 47244640777;
pub const KEYC_DOUBLECLICK1_CONTROL0: keyc = 47244640521;
pub const KEYC_DOUBLECLICK_CONTROL0: keyc = 47244640265;
pub const KEYC_DOUBLECLICK11_SCROLLBAR_DOWN: keyc = 47244643080;
pub const KEYC_DOUBLECLICK10_SCROLLBAR_DOWN: keyc = 47244642824;
pub const KEYC_DOUBLECLICK9_SCROLLBAR_DOWN: keyc = 47244642568;
pub const KEYC_DOUBLECLICK8_SCROLLBAR_DOWN: keyc = 47244642312;
pub const KEYC_DOUBLECLICK7_SCROLLBAR_DOWN: keyc = 47244642056;
pub const KEYC_DOUBLECLICK6_SCROLLBAR_DOWN: keyc = 47244641800;
pub const KEYC_DOUBLECLICK3_SCROLLBAR_DOWN: keyc = 47244641032;
pub const KEYC_DOUBLECLICK2_SCROLLBAR_DOWN: keyc = 47244640776;
pub const KEYC_DOUBLECLICK1_SCROLLBAR_DOWN: keyc = 47244640520;
pub const KEYC_DOUBLECLICK_SCROLLBAR_DOWN: keyc = 47244640264;
pub const KEYC_DOUBLECLICK11_SCROLLBAR_SLIDER: keyc = 47244643079;
pub const KEYC_DOUBLECLICK10_SCROLLBAR_SLIDER: keyc = 47244642823;
pub const KEYC_DOUBLECLICK9_SCROLLBAR_SLIDER: keyc = 47244642567;
pub const KEYC_DOUBLECLICK8_SCROLLBAR_SLIDER: keyc = 47244642311;
pub const KEYC_DOUBLECLICK7_SCROLLBAR_SLIDER: keyc = 47244642055;
pub const KEYC_DOUBLECLICK6_SCROLLBAR_SLIDER: keyc = 47244641799;
pub const KEYC_DOUBLECLICK3_SCROLLBAR_SLIDER: keyc = 47244641031;
pub const KEYC_DOUBLECLICK2_SCROLLBAR_SLIDER: keyc = 47244640775;
pub const KEYC_DOUBLECLICK1_SCROLLBAR_SLIDER: keyc = 47244640519;
pub const KEYC_DOUBLECLICK_SCROLLBAR_SLIDER: keyc = 47244640263;
pub const KEYC_DOUBLECLICK11_SCROLLBAR_UP: keyc = 47244643078;
pub const KEYC_DOUBLECLICK10_SCROLLBAR_UP: keyc = 47244642822;
pub const KEYC_DOUBLECLICK9_SCROLLBAR_UP: keyc = 47244642566;
pub const KEYC_DOUBLECLICK8_SCROLLBAR_UP: keyc = 47244642310;
pub const KEYC_DOUBLECLICK7_SCROLLBAR_UP: keyc = 47244642054;
pub const KEYC_DOUBLECLICK6_SCROLLBAR_UP: keyc = 47244641798;
pub const KEYC_DOUBLECLICK3_SCROLLBAR_UP: keyc = 47244641030;
pub const KEYC_DOUBLECLICK2_SCROLLBAR_UP: keyc = 47244640774;
pub const KEYC_DOUBLECLICK1_SCROLLBAR_UP: keyc = 47244640518;
pub const KEYC_DOUBLECLICK_SCROLLBAR_UP: keyc = 47244640262;
pub const KEYC_DOUBLECLICK11_BORDER: keyc = 47244643077;
pub const KEYC_DOUBLECLICK10_BORDER: keyc = 47244642821;
pub const KEYC_DOUBLECLICK9_BORDER: keyc = 47244642565;
pub const KEYC_DOUBLECLICK8_BORDER: keyc = 47244642309;
pub const KEYC_DOUBLECLICK7_BORDER: keyc = 47244642053;
pub const KEYC_DOUBLECLICK6_BORDER: keyc = 47244641797;
pub const KEYC_DOUBLECLICK3_BORDER: keyc = 47244641029;
pub const KEYC_DOUBLECLICK2_BORDER: keyc = 47244640773;
pub const KEYC_DOUBLECLICK1_BORDER: keyc = 47244640517;
pub const KEYC_DOUBLECLICK_BORDER: keyc = 47244640261;
pub const KEYC_DOUBLECLICK11_STATUS_DEFAULT: keyc = 47244643076;
pub const KEYC_DOUBLECLICK10_STATUS_DEFAULT: keyc = 47244642820;
pub const KEYC_DOUBLECLICK9_STATUS_DEFAULT: keyc = 47244642564;
pub const KEYC_DOUBLECLICK8_STATUS_DEFAULT: keyc = 47244642308;
pub const KEYC_DOUBLECLICK7_STATUS_DEFAULT: keyc = 47244642052;
pub const KEYC_DOUBLECLICK6_STATUS_DEFAULT: keyc = 47244641796;
pub const KEYC_DOUBLECLICK3_STATUS_DEFAULT: keyc = 47244641028;
pub const KEYC_DOUBLECLICK2_STATUS_DEFAULT: keyc = 47244640772;
pub const KEYC_DOUBLECLICK1_STATUS_DEFAULT: keyc = 47244640516;
pub const KEYC_DOUBLECLICK_STATUS_DEFAULT: keyc = 47244640260;
pub const KEYC_DOUBLECLICK11_STATUS_RIGHT: keyc = 47244643075;
pub const KEYC_DOUBLECLICK10_STATUS_RIGHT: keyc = 47244642819;
pub const KEYC_DOUBLECLICK9_STATUS_RIGHT: keyc = 47244642563;
pub const KEYC_DOUBLECLICK8_STATUS_RIGHT: keyc = 47244642307;
pub const KEYC_DOUBLECLICK7_STATUS_RIGHT: keyc = 47244642051;
pub const KEYC_DOUBLECLICK6_STATUS_RIGHT: keyc = 47244641795;
pub const KEYC_DOUBLECLICK3_STATUS_RIGHT: keyc = 47244641027;
pub const KEYC_DOUBLECLICK2_STATUS_RIGHT: keyc = 47244640771;
pub const KEYC_DOUBLECLICK1_STATUS_RIGHT: keyc = 47244640515;
pub const KEYC_DOUBLECLICK_STATUS_RIGHT: keyc = 47244640259;
pub const KEYC_DOUBLECLICK11_STATUS_LEFT: keyc = 47244643074;
pub const KEYC_DOUBLECLICK10_STATUS_LEFT: keyc = 47244642818;
pub const KEYC_DOUBLECLICK9_STATUS_LEFT: keyc = 47244642562;
pub const KEYC_DOUBLECLICK8_STATUS_LEFT: keyc = 47244642306;
pub const KEYC_DOUBLECLICK7_STATUS_LEFT: keyc = 47244642050;
pub const KEYC_DOUBLECLICK6_STATUS_LEFT: keyc = 47244641794;
pub const KEYC_DOUBLECLICK3_STATUS_LEFT: keyc = 47244641026;
pub const KEYC_DOUBLECLICK2_STATUS_LEFT: keyc = 47244640770;
pub const KEYC_DOUBLECLICK1_STATUS_LEFT: keyc = 47244640514;
pub const KEYC_DOUBLECLICK_STATUS_LEFT: keyc = 47244640258;
pub const KEYC_DOUBLECLICK11_STATUS: keyc = 47244643073;
pub const KEYC_DOUBLECLICK10_STATUS: keyc = 47244642817;
pub const KEYC_DOUBLECLICK9_STATUS: keyc = 47244642561;
pub const KEYC_DOUBLECLICK8_STATUS: keyc = 47244642305;
pub const KEYC_DOUBLECLICK7_STATUS: keyc = 47244642049;
pub const KEYC_DOUBLECLICK6_STATUS: keyc = 47244641793;
pub const KEYC_DOUBLECLICK3_STATUS: keyc = 47244641025;
pub const KEYC_DOUBLECLICK2_STATUS: keyc = 47244640769;
pub const KEYC_DOUBLECLICK1_STATUS: keyc = 47244640513;
pub const KEYC_DOUBLECLICK_STATUS: keyc = 47244640257;
pub const KEYC_DOUBLECLICK11_PANE: keyc = 47244643072;
pub const KEYC_DOUBLECLICK10_PANE: keyc = 47244642816;
pub const KEYC_DOUBLECLICK9_PANE: keyc = 47244642560;
pub const KEYC_DOUBLECLICK8_PANE: keyc = 47244642304;
pub const KEYC_DOUBLECLICK7_PANE: keyc = 47244642048;
pub const KEYC_DOUBLECLICK6_PANE: keyc = 47244641792;
pub const KEYC_DOUBLECLICK3_PANE: keyc = 47244641024;
pub const KEYC_DOUBLECLICK2_PANE: keyc = 47244640768;
pub const KEYC_DOUBLECLICK1_PANE: keyc = 47244640512;
pub const KEYC_DOUBLECLICK_PANE: keyc = 47244640256;
pub const KEYC_SECONDCLICK11_CONTROL9: keyc = 42949675794;
pub const KEYC_SECONDCLICK10_CONTROL9: keyc = 42949675538;
pub const KEYC_SECONDCLICK9_CONTROL9: keyc = 42949675282;
pub const KEYC_SECONDCLICK8_CONTROL9: keyc = 42949675026;
pub const KEYC_SECONDCLICK7_CONTROL9: keyc = 42949674770;
pub const KEYC_SECONDCLICK6_CONTROL9: keyc = 42949674514;
pub const KEYC_SECONDCLICK3_CONTROL9: keyc = 42949673746;
pub const KEYC_SECONDCLICK2_CONTROL9: keyc = 42949673490;
pub const KEYC_SECONDCLICK1_CONTROL9: keyc = 42949673234;
pub const KEYC_SECONDCLICK_CONTROL9: keyc = 42949672978;
pub const KEYC_SECONDCLICK11_CONTROL8: keyc = 42949675793;
pub const KEYC_SECONDCLICK10_CONTROL8: keyc = 42949675537;
pub const KEYC_SECONDCLICK9_CONTROL8: keyc = 42949675281;
pub const KEYC_SECONDCLICK8_CONTROL8: keyc = 42949675025;
pub const KEYC_SECONDCLICK7_CONTROL8: keyc = 42949674769;
pub const KEYC_SECONDCLICK6_CONTROL8: keyc = 42949674513;
pub const KEYC_SECONDCLICK3_CONTROL8: keyc = 42949673745;
pub const KEYC_SECONDCLICK2_CONTROL8: keyc = 42949673489;
pub const KEYC_SECONDCLICK1_CONTROL8: keyc = 42949673233;
pub const KEYC_SECONDCLICK_CONTROL8: keyc = 42949672977;
pub const KEYC_SECONDCLICK11_CONTROL7: keyc = 42949675792;
pub const KEYC_SECONDCLICK10_CONTROL7: keyc = 42949675536;
pub const KEYC_SECONDCLICK9_CONTROL7: keyc = 42949675280;
pub const KEYC_SECONDCLICK8_CONTROL7: keyc = 42949675024;
pub const KEYC_SECONDCLICK7_CONTROL7: keyc = 42949674768;
pub const KEYC_SECONDCLICK6_CONTROL7: keyc = 42949674512;
pub const KEYC_SECONDCLICK3_CONTROL7: keyc = 42949673744;
pub const KEYC_SECONDCLICK2_CONTROL7: keyc = 42949673488;
pub const KEYC_SECONDCLICK1_CONTROL7: keyc = 42949673232;
pub const KEYC_SECONDCLICK_CONTROL7: keyc = 42949672976;
pub const KEYC_SECONDCLICK11_CONTROL6: keyc = 42949675791;
pub const KEYC_SECONDCLICK10_CONTROL6: keyc = 42949675535;
pub const KEYC_SECONDCLICK9_CONTROL6: keyc = 42949675279;
pub const KEYC_SECONDCLICK8_CONTROL6: keyc = 42949675023;
pub const KEYC_SECONDCLICK7_CONTROL6: keyc = 42949674767;
pub const KEYC_SECONDCLICK6_CONTROL6: keyc = 42949674511;
pub const KEYC_SECONDCLICK3_CONTROL6: keyc = 42949673743;
pub const KEYC_SECONDCLICK2_CONTROL6: keyc = 42949673487;
pub const KEYC_SECONDCLICK1_CONTROL6: keyc = 42949673231;
pub const KEYC_SECONDCLICK_CONTROL6: keyc = 42949672975;
pub const KEYC_SECONDCLICK11_CONTROL5: keyc = 42949675790;
pub const KEYC_SECONDCLICK10_CONTROL5: keyc = 42949675534;
pub const KEYC_SECONDCLICK9_CONTROL5: keyc = 42949675278;
pub const KEYC_SECONDCLICK8_CONTROL5: keyc = 42949675022;
pub const KEYC_SECONDCLICK7_CONTROL5: keyc = 42949674766;
pub const KEYC_SECONDCLICK6_CONTROL5: keyc = 42949674510;
pub const KEYC_SECONDCLICK3_CONTROL5: keyc = 42949673742;
pub const KEYC_SECONDCLICK2_CONTROL5: keyc = 42949673486;
pub const KEYC_SECONDCLICK1_CONTROL5: keyc = 42949673230;
pub const KEYC_SECONDCLICK_CONTROL5: keyc = 42949672974;
pub const KEYC_SECONDCLICK11_CONTROL4: keyc = 42949675789;
pub const KEYC_SECONDCLICK10_CONTROL4: keyc = 42949675533;
pub const KEYC_SECONDCLICK9_CONTROL4: keyc = 42949675277;
pub const KEYC_SECONDCLICK8_CONTROL4: keyc = 42949675021;
pub const KEYC_SECONDCLICK7_CONTROL4: keyc = 42949674765;
pub const KEYC_SECONDCLICK6_CONTROL4: keyc = 42949674509;
pub const KEYC_SECONDCLICK3_CONTROL4: keyc = 42949673741;
pub const KEYC_SECONDCLICK2_CONTROL4: keyc = 42949673485;
pub const KEYC_SECONDCLICK1_CONTROL4: keyc = 42949673229;
pub const KEYC_SECONDCLICK_CONTROL4: keyc = 42949672973;
pub const KEYC_SECONDCLICK11_CONTROL3: keyc = 42949675788;
pub const KEYC_SECONDCLICK10_CONTROL3: keyc = 42949675532;
pub const KEYC_SECONDCLICK9_CONTROL3: keyc = 42949675276;
pub const KEYC_SECONDCLICK8_CONTROL3: keyc = 42949675020;
pub const KEYC_SECONDCLICK7_CONTROL3: keyc = 42949674764;
pub const KEYC_SECONDCLICK6_CONTROL3: keyc = 42949674508;
pub const KEYC_SECONDCLICK3_CONTROL3: keyc = 42949673740;
pub const KEYC_SECONDCLICK2_CONTROL3: keyc = 42949673484;
pub const KEYC_SECONDCLICK1_CONTROL3: keyc = 42949673228;
pub const KEYC_SECONDCLICK_CONTROL3: keyc = 42949672972;
pub const KEYC_SECONDCLICK11_CONTROL2: keyc = 42949675787;
pub const KEYC_SECONDCLICK10_CONTROL2: keyc = 42949675531;
pub const KEYC_SECONDCLICK9_CONTROL2: keyc = 42949675275;
pub const KEYC_SECONDCLICK8_CONTROL2: keyc = 42949675019;
pub const KEYC_SECONDCLICK7_CONTROL2: keyc = 42949674763;
pub const KEYC_SECONDCLICK6_CONTROL2: keyc = 42949674507;
pub const KEYC_SECONDCLICK3_CONTROL2: keyc = 42949673739;
pub const KEYC_SECONDCLICK2_CONTROL2: keyc = 42949673483;
pub const KEYC_SECONDCLICK1_CONTROL2: keyc = 42949673227;
pub const KEYC_SECONDCLICK_CONTROL2: keyc = 42949672971;
pub const KEYC_SECONDCLICK11_CONTROL1: keyc = 42949675786;
pub const KEYC_SECONDCLICK10_CONTROL1: keyc = 42949675530;
pub const KEYC_SECONDCLICK9_CONTROL1: keyc = 42949675274;
pub const KEYC_SECONDCLICK8_CONTROL1: keyc = 42949675018;
pub const KEYC_SECONDCLICK7_CONTROL1: keyc = 42949674762;
pub const KEYC_SECONDCLICK6_CONTROL1: keyc = 42949674506;
pub const KEYC_SECONDCLICK3_CONTROL1: keyc = 42949673738;
pub const KEYC_SECONDCLICK2_CONTROL1: keyc = 42949673482;
pub const KEYC_SECONDCLICK1_CONTROL1: keyc = 42949673226;
pub const KEYC_SECONDCLICK_CONTROL1: keyc = 42949672970;
pub const KEYC_SECONDCLICK11_CONTROL0: keyc = 42949675785;
pub const KEYC_SECONDCLICK10_CONTROL0: keyc = 42949675529;
pub const KEYC_SECONDCLICK9_CONTROL0: keyc = 42949675273;
pub const KEYC_SECONDCLICK8_CONTROL0: keyc = 42949675017;
pub const KEYC_SECONDCLICK7_CONTROL0: keyc = 42949674761;
pub const KEYC_SECONDCLICK6_CONTROL0: keyc = 42949674505;
pub const KEYC_SECONDCLICK3_CONTROL0: keyc = 42949673737;
pub const KEYC_SECONDCLICK2_CONTROL0: keyc = 42949673481;
pub const KEYC_SECONDCLICK1_CONTROL0: keyc = 42949673225;
pub const KEYC_SECONDCLICK_CONTROL0: keyc = 42949672969;
pub const KEYC_SECONDCLICK11_SCROLLBAR_DOWN: keyc = 42949675784;
pub const KEYC_SECONDCLICK10_SCROLLBAR_DOWN: keyc = 42949675528;
pub const KEYC_SECONDCLICK9_SCROLLBAR_DOWN: keyc = 42949675272;
pub const KEYC_SECONDCLICK8_SCROLLBAR_DOWN: keyc = 42949675016;
pub const KEYC_SECONDCLICK7_SCROLLBAR_DOWN: keyc = 42949674760;
pub const KEYC_SECONDCLICK6_SCROLLBAR_DOWN: keyc = 42949674504;
pub const KEYC_SECONDCLICK3_SCROLLBAR_DOWN: keyc = 42949673736;
pub const KEYC_SECONDCLICK2_SCROLLBAR_DOWN: keyc = 42949673480;
pub const KEYC_SECONDCLICK1_SCROLLBAR_DOWN: keyc = 42949673224;
pub const KEYC_SECONDCLICK_SCROLLBAR_DOWN: keyc = 42949672968;
pub const KEYC_SECONDCLICK11_SCROLLBAR_SLIDER: keyc = 42949675783;
pub const KEYC_SECONDCLICK10_SCROLLBAR_SLIDER: keyc = 42949675527;
pub const KEYC_SECONDCLICK9_SCROLLBAR_SLIDER: keyc = 42949675271;
pub const KEYC_SECONDCLICK8_SCROLLBAR_SLIDER: keyc = 42949675015;
pub const KEYC_SECONDCLICK7_SCROLLBAR_SLIDER: keyc = 42949674759;
pub const KEYC_SECONDCLICK6_SCROLLBAR_SLIDER: keyc = 42949674503;
pub const KEYC_SECONDCLICK3_SCROLLBAR_SLIDER: keyc = 42949673735;
pub const KEYC_SECONDCLICK2_SCROLLBAR_SLIDER: keyc = 42949673479;
pub const KEYC_SECONDCLICK1_SCROLLBAR_SLIDER: keyc = 42949673223;
pub const KEYC_SECONDCLICK_SCROLLBAR_SLIDER: keyc = 42949672967;
pub const KEYC_SECONDCLICK11_SCROLLBAR_UP: keyc = 42949675782;
pub const KEYC_SECONDCLICK10_SCROLLBAR_UP: keyc = 42949675526;
pub const KEYC_SECONDCLICK9_SCROLLBAR_UP: keyc = 42949675270;
pub const KEYC_SECONDCLICK8_SCROLLBAR_UP: keyc = 42949675014;
pub const KEYC_SECONDCLICK7_SCROLLBAR_UP: keyc = 42949674758;
pub const KEYC_SECONDCLICK6_SCROLLBAR_UP: keyc = 42949674502;
pub const KEYC_SECONDCLICK3_SCROLLBAR_UP: keyc = 42949673734;
pub const KEYC_SECONDCLICK2_SCROLLBAR_UP: keyc = 42949673478;
pub const KEYC_SECONDCLICK1_SCROLLBAR_UP: keyc = 42949673222;
pub const KEYC_SECONDCLICK_SCROLLBAR_UP: keyc = 42949672966;
pub const KEYC_SECONDCLICK11_BORDER: keyc = 42949675781;
pub const KEYC_SECONDCLICK10_BORDER: keyc = 42949675525;
pub const KEYC_SECONDCLICK9_BORDER: keyc = 42949675269;
pub const KEYC_SECONDCLICK8_BORDER: keyc = 42949675013;
pub const KEYC_SECONDCLICK7_BORDER: keyc = 42949674757;
pub const KEYC_SECONDCLICK6_BORDER: keyc = 42949674501;
pub const KEYC_SECONDCLICK3_BORDER: keyc = 42949673733;
pub const KEYC_SECONDCLICK2_BORDER: keyc = 42949673477;
pub const KEYC_SECONDCLICK1_BORDER: keyc = 42949673221;
pub const KEYC_SECONDCLICK_BORDER: keyc = 42949672965;
pub const KEYC_SECONDCLICK11_STATUS_DEFAULT: keyc = 42949675780;
pub const KEYC_SECONDCLICK10_STATUS_DEFAULT: keyc = 42949675524;
pub const KEYC_SECONDCLICK9_STATUS_DEFAULT: keyc = 42949675268;
pub const KEYC_SECONDCLICK8_STATUS_DEFAULT: keyc = 42949675012;
pub const KEYC_SECONDCLICK7_STATUS_DEFAULT: keyc = 42949674756;
pub const KEYC_SECONDCLICK6_STATUS_DEFAULT: keyc = 42949674500;
pub const KEYC_SECONDCLICK3_STATUS_DEFAULT: keyc = 42949673732;
pub const KEYC_SECONDCLICK2_STATUS_DEFAULT: keyc = 42949673476;
pub const KEYC_SECONDCLICK1_STATUS_DEFAULT: keyc = 42949673220;
pub const KEYC_SECONDCLICK_STATUS_DEFAULT: keyc = 42949672964;
pub const KEYC_SECONDCLICK11_STATUS_RIGHT: keyc = 42949675779;
pub const KEYC_SECONDCLICK10_STATUS_RIGHT: keyc = 42949675523;
pub const KEYC_SECONDCLICK9_STATUS_RIGHT: keyc = 42949675267;
pub const KEYC_SECONDCLICK8_STATUS_RIGHT: keyc = 42949675011;
pub const KEYC_SECONDCLICK7_STATUS_RIGHT: keyc = 42949674755;
pub const KEYC_SECONDCLICK6_STATUS_RIGHT: keyc = 42949674499;
pub const KEYC_SECONDCLICK3_STATUS_RIGHT: keyc = 42949673731;
pub const KEYC_SECONDCLICK2_STATUS_RIGHT: keyc = 42949673475;
pub const KEYC_SECONDCLICK1_STATUS_RIGHT: keyc = 42949673219;
pub const KEYC_SECONDCLICK_STATUS_RIGHT: keyc = 42949672963;
pub const KEYC_SECONDCLICK11_STATUS_LEFT: keyc = 42949675778;
pub const KEYC_SECONDCLICK10_STATUS_LEFT: keyc = 42949675522;
pub const KEYC_SECONDCLICK9_STATUS_LEFT: keyc = 42949675266;
pub const KEYC_SECONDCLICK8_STATUS_LEFT: keyc = 42949675010;
pub const KEYC_SECONDCLICK7_STATUS_LEFT: keyc = 42949674754;
pub const KEYC_SECONDCLICK6_STATUS_LEFT: keyc = 42949674498;
pub const KEYC_SECONDCLICK3_STATUS_LEFT: keyc = 42949673730;
pub const KEYC_SECONDCLICK2_STATUS_LEFT: keyc = 42949673474;
pub const KEYC_SECONDCLICK1_STATUS_LEFT: keyc = 42949673218;
pub const KEYC_SECONDCLICK_STATUS_LEFT: keyc = 42949672962;
pub const KEYC_SECONDCLICK11_STATUS: keyc = 42949675777;
pub const KEYC_SECONDCLICK10_STATUS: keyc = 42949675521;
pub const KEYC_SECONDCLICK9_STATUS: keyc = 42949675265;
pub const KEYC_SECONDCLICK8_STATUS: keyc = 42949675009;
pub const KEYC_SECONDCLICK7_STATUS: keyc = 42949674753;
pub const KEYC_SECONDCLICK6_STATUS: keyc = 42949674497;
pub const KEYC_SECONDCLICK3_STATUS: keyc = 42949673729;
pub const KEYC_SECONDCLICK2_STATUS: keyc = 42949673473;
pub const KEYC_SECONDCLICK1_STATUS: keyc = 42949673217;
pub const KEYC_SECONDCLICK_STATUS: keyc = 42949672961;
pub const KEYC_SECONDCLICK11_PANE: keyc = 42949675776;
pub const KEYC_SECONDCLICK10_PANE: keyc = 42949675520;
pub const KEYC_SECONDCLICK9_PANE: keyc = 42949675264;
pub const KEYC_SECONDCLICK8_PANE: keyc = 42949675008;
pub const KEYC_SECONDCLICK7_PANE: keyc = 42949674752;
pub const KEYC_SECONDCLICK6_PANE: keyc = 42949674496;
pub const KEYC_SECONDCLICK3_PANE: keyc = 42949673728;
pub const KEYC_SECONDCLICK2_PANE: keyc = 42949673472;
pub const KEYC_SECONDCLICK1_PANE: keyc = 42949673216;
pub const KEYC_SECONDCLICK_PANE: keyc = 42949672960;
pub const KEYC_MOUSEDRAGEND11_CONTROL9: keyc = 30064773906;
pub const KEYC_MOUSEDRAGEND10_CONTROL9: keyc = 30064773650;
pub const KEYC_MOUSEDRAGEND9_CONTROL9: keyc = 30064773394;
pub const KEYC_MOUSEDRAGEND8_CONTROL9: keyc = 30064773138;
pub const KEYC_MOUSEDRAGEND7_CONTROL9: keyc = 30064772882;
pub const KEYC_MOUSEDRAGEND6_CONTROL9: keyc = 30064772626;
pub const KEYC_MOUSEDRAGEND3_CONTROL9: keyc = 30064771858;
pub const KEYC_MOUSEDRAGEND2_CONTROL9: keyc = 30064771602;
pub const KEYC_MOUSEDRAGEND1_CONTROL9: keyc = 30064771346;
pub const KEYC_MOUSEDRAGEND_CONTROL9: keyc = 30064771090;
pub const KEYC_MOUSEDRAGEND11_CONTROL8: keyc = 30064773905;
pub const KEYC_MOUSEDRAGEND10_CONTROL8: keyc = 30064773649;
pub const KEYC_MOUSEDRAGEND9_CONTROL8: keyc = 30064773393;
pub const KEYC_MOUSEDRAGEND8_CONTROL8: keyc = 30064773137;
pub const KEYC_MOUSEDRAGEND7_CONTROL8: keyc = 30064772881;
pub const KEYC_MOUSEDRAGEND6_CONTROL8: keyc = 30064772625;
pub const KEYC_MOUSEDRAGEND3_CONTROL8: keyc = 30064771857;
pub const KEYC_MOUSEDRAGEND2_CONTROL8: keyc = 30064771601;
pub const KEYC_MOUSEDRAGEND1_CONTROL8: keyc = 30064771345;
pub const KEYC_MOUSEDRAGEND_CONTROL8: keyc = 30064771089;
pub const KEYC_MOUSEDRAGEND11_CONTROL7: keyc = 30064773904;
pub const KEYC_MOUSEDRAGEND10_CONTROL7: keyc = 30064773648;
pub const KEYC_MOUSEDRAGEND9_CONTROL7: keyc = 30064773392;
pub const KEYC_MOUSEDRAGEND8_CONTROL7: keyc = 30064773136;
pub const KEYC_MOUSEDRAGEND7_CONTROL7: keyc = 30064772880;
pub const KEYC_MOUSEDRAGEND6_CONTROL7: keyc = 30064772624;
pub const KEYC_MOUSEDRAGEND3_CONTROL7: keyc = 30064771856;
pub const KEYC_MOUSEDRAGEND2_CONTROL7: keyc = 30064771600;
pub const KEYC_MOUSEDRAGEND1_CONTROL7: keyc = 30064771344;
pub const KEYC_MOUSEDRAGEND_CONTROL7: keyc = 30064771088;
pub const KEYC_MOUSEDRAGEND11_CONTROL6: keyc = 30064773903;
pub const KEYC_MOUSEDRAGEND10_CONTROL6: keyc = 30064773647;
pub const KEYC_MOUSEDRAGEND9_CONTROL6: keyc = 30064773391;
pub const KEYC_MOUSEDRAGEND8_CONTROL6: keyc = 30064773135;
pub const KEYC_MOUSEDRAGEND7_CONTROL6: keyc = 30064772879;
pub const KEYC_MOUSEDRAGEND6_CONTROL6: keyc = 30064772623;
pub const KEYC_MOUSEDRAGEND3_CONTROL6: keyc = 30064771855;
pub const KEYC_MOUSEDRAGEND2_CONTROL6: keyc = 30064771599;
pub const KEYC_MOUSEDRAGEND1_CONTROL6: keyc = 30064771343;
pub const KEYC_MOUSEDRAGEND_CONTROL6: keyc = 30064771087;
pub const KEYC_MOUSEDRAGEND11_CONTROL5: keyc = 30064773902;
pub const KEYC_MOUSEDRAGEND10_CONTROL5: keyc = 30064773646;
pub const KEYC_MOUSEDRAGEND9_CONTROL5: keyc = 30064773390;
pub const KEYC_MOUSEDRAGEND8_CONTROL5: keyc = 30064773134;
pub const KEYC_MOUSEDRAGEND7_CONTROL5: keyc = 30064772878;
pub const KEYC_MOUSEDRAGEND6_CONTROL5: keyc = 30064772622;
pub const KEYC_MOUSEDRAGEND3_CONTROL5: keyc = 30064771854;
pub const KEYC_MOUSEDRAGEND2_CONTROL5: keyc = 30064771598;
pub const KEYC_MOUSEDRAGEND1_CONTROL5: keyc = 30064771342;
pub const KEYC_MOUSEDRAGEND_CONTROL5: keyc = 30064771086;
pub const KEYC_MOUSEDRAGEND11_CONTROL4: keyc = 30064773901;
pub const KEYC_MOUSEDRAGEND10_CONTROL4: keyc = 30064773645;
pub const KEYC_MOUSEDRAGEND9_CONTROL4: keyc = 30064773389;
pub const KEYC_MOUSEDRAGEND8_CONTROL4: keyc = 30064773133;
pub const KEYC_MOUSEDRAGEND7_CONTROL4: keyc = 30064772877;
pub const KEYC_MOUSEDRAGEND6_CONTROL4: keyc = 30064772621;
pub const KEYC_MOUSEDRAGEND3_CONTROL4: keyc = 30064771853;
pub const KEYC_MOUSEDRAGEND2_CONTROL4: keyc = 30064771597;
pub const KEYC_MOUSEDRAGEND1_CONTROL4: keyc = 30064771341;
pub const KEYC_MOUSEDRAGEND_CONTROL4: keyc = 30064771085;
pub const KEYC_MOUSEDRAGEND11_CONTROL3: keyc = 30064773900;
pub const KEYC_MOUSEDRAGEND10_CONTROL3: keyc = 30064773644;
pub const KEYC_MOUSEDRAGEND9_CONTROL3: keyc = 30064773388;
pub const KEYC_MOUSEDRAGEND8_CONTROL3: keyc = 30064773132;
pub const KEYC_MOUSEDRAGEND7_CONTROL3: keyc = 30064772876;
pub const KEYC_MOUSEDRAGEND6_CONTROL3: keyc = 30064772620;
pub const KEYC_MOUSEDRAGEND3_CONTROL3: keyc = 30064771852;
pub const KEYC_MOUSEDRAGEND2_CONTROL3: keyc = 30064771596;
pub const KEYC_MOUSEDRAGEND1_CONTROL3: keyc = 30064771340;
pub const KEYC_MOUSEDRAGEND_CONTROL3: keyc = 30064771084;
pub const KEYC_MOUSEDRAGEND11_CONTROL2: keyc = 30064773899;
pub const KEYC_MOUSEDRAGEND10_CONTROL2: keyc = 30064773643;
pub const KEYC_MOUSEDRAGEND9_CONTROL2: keyc = 30064773387;
pub const KEYC_MOUSEDRAGEND8_CONTROL2: keyc = 30064773131;
pub const KEYC_MOUSEDRAGEND7_CONTROL2: keyc = 30064772875;
pub const KEYC_MOUSEDRAGEND6_CONTROL2: keyc = 30064772619;
pub const KEYC_MOUSEDRAGEND3_CONTROL2: keyc = 30064771851;
pub const KEYC_MOUSEDRAGEND2_CONTROL2: keyc = 30064771595;
pub const KEYC_MOUSEDRAGEND1_CONTROL2: keyc = 30064771339;
pub const KEYC_MOUSEDRAGEND_CONTROL2: keyc = 30064771083;
pub const KEYC_MOUSEDRAGEND11_CONTROL1: keyc = 30064773898;
pub const KEYC_MOUSEDRAGEND10_CONTROL1: keyc = 30064773642;
pub const KEYC_MOUSEDRAGEND9_CONTROL1: keyc = 30064773386;
pub const KEYC_MOUSEDRAGEND8_CONTROL1: keyc = 30064773130;
pub const KEYC_MOUSEDRAGEND7_CONTROL1: keyc = 30064772874;
pub const KEYC_MOUSEDRAGEND6_CONTROL1: keyc = 30064772618;
pub const KEYC_MOUSEDRAGEND3_CONTROL1: keyc = 30064771850;
pub const KEYC_MOUSEDRAGEND2_CONTROL1: keyc = 30064771594;
pub const KEYC_MOUSEDRAGEND1_CONTROL1: keyc = 30064771338;
pub const KEYC_MOUSEDRAGEND_CONTROL1: keyc = 30064771082;
pub const KEYC_MOUSEDRAGEND11_CONTROL0: keyc = 30064773897;
pub const KEYC_MOUSEDRAGEND10_CONTROL0: keyc = 30064773641;
pub const KEYC_MOUSEDRAGEND9_CONTROL0: keyc = 30064773385;
pub const KEYC_MOUSEDRAGEND8_CONTROL0: keyc = 30064773129;
pub const KEYC_MOUSEDRAGEND7_CONTROL0: keyc = 30064772873;
pub const KEYC_MOUSEDRAGEND6_CONTROL0: keyc = 30064772617;
pub const KEYC_MOUSEDRAGEND3_CONTROL0: keyc = 30064771849;
pub const KEYC_MOUSEDRAGEND2_CONTROL0: keyc = 30064771593;
pub const KEYC_MOUSEDRAGEND1_CONTROL0: keyc = 30064771337;
pub const KEYC_MOUSEDRAGEND_CONTROL0: keyc = 30064771081;
pub const KEYC_MOUSEDRAGEND11_SCROLLBAR_DOWN: keyc = 30064773896;
pub const KEYC_MOUSEDRAGEND10_SCROLLBAR_DOWN: keyc = 30064773640;
pub const KEYC_MOUSEDRAGEND9_SCROLLBAR_DOWN: keyc = 30064773384;
pub const KEYC_MOUSEDRAGEND8_SCROLLBAR_DOWN: keyc = 30064773128;
pub const KEYC_MOUSEDRAGEND7_SCROLLBAR_DOWN: keyc = 30064772872;
pub const KEYC_MOUSEDRAGEND6_SCROLLBAR_DOWN: keyc = 30064772616;
pub const KEYC_MOUSEDRAGEND3_SCROLLBAR_DOWN: keyc = 30064771848;
pub const KEYC_MOUSEDRAGEND2_SCROLLBAR_DOWN: keyc = 30064771592;
pub const KEYC_MOUSEDRAGEND1_SCROLLBAR_DOWN: keyc = 30064771336;
pub const KEYC_MOUSEDRAGEND_SCROLLBAR_DOWN: keyc = 30064771080;
pub const KEYC_MOUSEDRAGEND11_SCROLLBAR_SLIDER: keyc = 30064773895;
pub const KEYC_MOUSEDRAGEND10_SCROLLBAR_SLIDER: keyc = 30064773639;
pub const KEYC_MOUSEDRAGEND9_SCROLLBAR_SLIDER: keyc = 30064773383;
pub const KEYC_MOUSEDRAGEND8_SCROLLBAR_SLIDER: keyc = 30064773127;
pub const KEYC_MOUSEDRAGEND7_SCROLLBAR_SLIDER: keyc = 30064772871;
pub const KEYC_MOUSEDRAGEND6_SCROLLBAR_SLIDER: keyc = 30064772615;
pub const KEYC_MOUSEDRAGEND3_SCROLLBAR_SLIDER: keyc = 30064771847;
pub const KEYC_MOUSEDRAGEND2_SCROLLBAR_SLIDER: keyc = 30064771591;
pub const KEYC_MOUSEDRAGEND1_SCROLLBAR_SLIDER: keyc = 30064771335;
pub const KEYC_MOUSEDRAGEND_SCROLLBAR_SLIDER: keyc = 30064771079;
pub const KEYC_MOUSEDRAGEND11_SCROLLBAR_UP: keyc = 30064773894;
pub const KEYC_MOUSEDRAGEND10_SCROLLBAR_UP: keyc = 30064773638;
pub const KEYC_MOUSEDRAGEND9_SCROLLBAR_UP: keyc = 30064773382;
pub const KEYC_MOUSEDRAGEND8_SCROLLBAR_UP: keyc = 30064773126;
pub const KEYC_MOUSEDRAGEND7_SCROLLBAR_UP: keyc = 30064772870;
pub const KEYC_MOUSEDRAGEND6_SCROLLBAR_UP: keyc = 30064772614;
pub const KEYC_MOUSEDRAGEND3_SCROLLBAR_UP: keyc = 30064771846;
pub const KEYC_MOUSEDRAGEND2_SCROLLBAR_UP: keyc = 30064771590;
pub const KEYC_MOUSEDRAGEND1_SCROLLBAR_UP: keyc = 30064771334;
pub const KEYC_MOUSEDRAGEND_SCROLLBAR_UP: keyc = 30064771078;
pub const KEYC_MOUSEDRAGEND11_BORDER: keyc = 30064773893;
pub const KEYC_MOUSEDRAGEND10_BORDER: keyc = 30064773637;
pub const KEYC_MOUSEDRAGEND9_BORDER: keyc = 30064773381;
pub const KEYC_MOUSEDRAGEND8_BORDER: keyc = 30064773125;
pub const KEYC_MOUSEDRAGEND7_BORDER: keyc = 30064772869;
pub const KEYC_MOUSEDRAGEND6_BORDER: keyc = 30064772613;
pub const KEYC_MOUSEDRAGEND3_BORDER: keyc = 30064771845;
pub const KEYC_MOUSEDRAGEND2_BORDER: keyc = 30064771589;
pub const KEYC_MOUSEDRAGEND1_BORDER: keyc = 30064771333;
pub const KEYC_MOUSEDRAGEND_BORDER: keyc = 30064771077;
pub const KEYC_MOUSEDRAGEND11_STATUS_DEFAULT: keyc = 30064773892;
pub const KEYC_MOUSEDRAGEND10_STATUS_DEFAULT: keyc = 30064773636;
pub const KEYC_MOUSEDRAGEND9_STATUS_DEFAULT: keyc = 30064773380;
pub const KEYC_MOUSEDRAGEND8_STATUS_DEFAULT: keyc = 30064773124;
pub const KEYC_MOUSEDRAGEND7_STATUS_DEFAULT: keyc = 30064772868;
pub const KEYC_MOUSEDRAGEND6_STATUS_DEFAULT: keyc = 30064772612;
pub const KEYC_MOUSEDRAGEND3_STATUS_DEFAULT: keyc = 30064771844;
pub const KEYC_MOUSEDRAGEND2_STATUS_DEFAULT: keyc = 30064771588;
pub const KEYC_MOUSEDRAGEND1_STATUS_DEFAULT: keyc = 30064771332;
pub const KEYC_MOUSEDRAGEND_STATUS_DEFAULT: keyc = 30064771076;
pub const KEYC_MOUSEDRAGEND11_STATUS_RIGHT: keyc = 30064773891;
pub const KEYC_MOUSEDRAGEND10_STATUS_RIGHT: keyc = 30064773635;
pub const KEYC_MOUSEDRAGEND9_STATUS_RIGHT: keyc = 30064773379;
pub const KEYC_MOUSEDRAGEND8_STATUS_RIGHT: keyc = 30064773123;
pub const KEYC_MOUSEDRAGEND7_STATUS_RIGHT: keyc = 30064772867;
pub const KEYC_MOUSEDRAGEND6_STATUS_RIGHT: keyc = 30064772611;
pub const KEYC_MOUSEDRAGEND3_STATUS_RIGHT: keyc = 30064771843;
pub const KEYC_MOUSEDRAGEND2_STATUS_RIGHT: keyc = 30064771587;
pub const KEYC_MOUSEDRAGEND1_STATUS_RIGHT: keyc = 30064771331;
pub const KEYC_MOUSEDRAGEND_STATUS_RIGHT: keyc = 30064771075;
pub const KEYC_MOUSEDRAGEND11_STATUS_LEFT: keyc = 30064773890;
pub const KEYC_MOUSEDRAGEND10_STATUS_LEFT: keyc = 30064773634;
pub const KEYC_MOUSEDRAGEND9_STATUS_LEFT: keyc = 30064773378;
pub const KEYC_MOUSEDRAGEND8_STATUS_LEFT: keyc = 30064773122;
pub const KEYC_MOUSEDRAGEND7_STATUS_LEFT: keyc = 30064772866;
pub const KEYC_MOUSEDRAGEND6_STATUS_LEFT: keyc = 30064772610;
pub const KEYC_MOUSEDRAGEND3_STATUS_LEFT: keyc = 30064771842;
pub const KEYC_MOUSEDRAGEND2_STATUS_LEFT: keyc = 30064771586;
pub const KEYC_MOUSEDRAGEND1_STATUS_LEFT: keyc = 30064771330;
pub const KEYC_MOUSEDRAGEND_STATUS_LEFT: keyc = 30064771074;
pub const KEYC_MOUSEDRAGEND11_STATUS: keyc = 30064773889;
pub const KEYC_MOUSEDRAGEND10_STATUS: keyc = 30064773633;
pub const KEYC_MOUSEDRAGEND9_STATUS: keyc = 30064773377;
pub const KEYC_MOUSEDRAGEND8_STATUS: keyc = 30064773121;
pub const KEYC_MOUSEDRAGEND7_STATUS: keyc = 30064772865;
pub const KEYC_MOUSEDRAGEND6_STATUS: keyc = 30064772609;
pub const KEYC_MOUSEDRAGEND3_STATUS: keyc = 30064771841;
pub const KEYC_MOUSEDRAGEND2_STATUS: keyc = 30064771585;
pub const KEYC_MOUSEDRAGEND1_STATUS: keyc = 30064771329;
pub const KEYC_MOUSEDRAGEND_STATUS: keyc = 30064771073;
pub const KEYC_MOUSEDRAGEND11_PANE: keyc = 30064773888;
pub const KEYC_MOUSEDRAGEND10_PANE: keyc = 30064773632;
pub const KEYC_MOUSEDRAGEND9_PANE: keyc = 30064773376;
pub const KEYC_MOUSEDRAGEND8_PANE: keyc = 30064773120;
pub const KEYC_MOUSEDRAGEND7_PANE: keyc = 30064772864;
pub const KEYC_MOUSEDRAGEND6_PANE: keyc = 30064772608;
pub const KEYC_MOUSEDRAGEND3_PANE: keyc = 30064771840;
pub const KEYC_MOUSEDRAGEND2_PANE: keyc = 30064771584;
pub const KEYC_MOUSEDRAGEND1_PANE: keyc = 30064771328;
pub const KEYC_MOUSEDRAGEND_PANE: keyc = 30064771072;
pub const KEYC_MOUSEDRAG11_CONTROL9: keyc = 25769806610;
pub const KEYC_MOUSEDRAG10_CONTROL9: keyc = 25769806354;
pub const KEYC_MOUSEDRAG9_CONTROL9: keyc = 25769806098;
pub const KEYC_MOUSEDRAG8_CONTROL9: keyc = 25769805842;
pub const KEYC_MOUSEDRAG7_CONTROL9: keyc = 25769805586;
pub const KEYC_MOUSEDRAG6_CONTROL9: keyc = 25769805330;
pub const KEYC_MOUSEDRAG3_CONTROL9: keyc = 25769804562;
pub const KEYC_MOUSEDRAG2_CONTROL9: keyc = 25769804306;
pub const KEYC_MOUSEDRAG1_CONTROL9: keyc = 25769804050;
pub const KEYC_MOUSEDRAG_CONTROL9: keyc = 25769803794;
pub const KEYC_MOUSEDRAG11_CONTROL8: keyc = 25769806609;
pub const KEYC_MOUSEDRAG10_CONTROL8: keyc = 25769806353;
pub const KEYC_MOUSEDRAG9_CONTROL8: keyc = 25769806097;
pub const KEYC_MOUSEDRAG8_CONTROL8: keyc = 25769805841;
pub const KEYC_MOUSEDRAG7_CONTROL8: keyc = 25769805585;
pub const KEYC_MOUSEDRAG6_CONTROL8: keyc = 25769805329;
pub const KEYC_MOUSEDRAG3_CONTROL8: keyc = 25769804561;
pub const KEYC_MOUSEDRAG2_CONTROL8: keyc = 25769804305;
pub const KEYC_MOUSEDRAG1_CONTROL8: keyc = 25769804049;
pub const KEYC_MOUSEDRAG_CONTROL8: keyc = 25769803793;
pub const KEYC_MOUSEDRAG11_CONTROL7: keyc = 25769806608;
pub const KEYC_MOUSEDRAG10_CONTROL7: keyc = 25769806352;
pub const KEYC_MOUSEDRAG9_CONTROL7: keyc = 25769806096;
pub const KEYC_MOUSEDRAG8_CONTROL7: keyc = 25769805840;
pub const KEYC_MOUSEDRAG7_CONTROL7: keyc = 25769805584;
pub const KEYC_MOUSEDRAG6_CONTROL7: keyc = 25769805328;
pub const KEYC_MOUSEDRAG3_CONTROL7: keyc = 25769804560;
pub const KEYC_MOUSEDRAG2_CONTROL7: keyc = 25769804304;
pub const KEYC_MOUSEDRAG1_CONTROL7: keyc = 25769804048;
pub const KEYC_MOUSEDRAG_CONTROL7: keyc = 25769803792;
pub const KEYC_MOUSEDRAG11_CONTROL6: keyc = 25769806607;
pub const KEYC_MOUSEDRAG10_CONTROL6: keyc = 25769806351;
pub const KEYC_MOUSEDRAG9_CONTROL6: keyc = 25769806095;
pub const KEYC_MOUSEDRAG8_CONTROL6: keyc = 25769805839;
pub const KEYC_MOUSEDRAG7_CONTROL6: keyc = 25769805583;
pub const KEYC_MOUSEDRAG6_CONTROL6: keyc = 25769805327;
pub const KEYC_MOUSEDRAG3_CONTROL6: keyc = 25769804559;
pub const KEYC_MOUSEDRAG2_CONTROL6: keyc = 25769804303;
pub const KEYC_MOUSEDRAG1_CONTROL6: keyc = 25769804047;
pub const KEYC_MOUSEDRAG_CONTROL6: keyc = 25769803791;
pub const KEYC_MOUSEDRAG11_CONTROL5: keyc = 25769806606;
pub const KEYC_MOUSEDRAG10_CONTROL5: keyc = 25769806350;
pub const KEYC_MOUSEDRAG9_CONTROL5: keyc = 25769806094;
pub const KEYC_MOUSEDRAG8_CONTROL5: keyc = 25769805838;
pub const KEYC_MOUSEDRAG7_CONTROL5: keyc = 25769805582;
pub const KEYC_MOUSEDRAG6_CONTROL5: keyc = 25769805326;
pub const KEYC_MOUSEDRAG3_CONTROL5: keyc = 25769804558;
pub const KEYC_MOUSEDRAG2_CONTROL5: keyc = 25769804302;
pub const KEYC_MOUSEDRAG1_CONTROL5: keyc = 25769804046;
pub const KEYC_MOUSEDRAG_CONTROL5: keyc = 25769803790;
pub const KEYC_MOUSEDRAG11_CONTROL4: keyc = 25769806605;
pub const KEYC_MOUSEDRAG10_CONTROL4: keyc = 25769806349;
pub const KEYC_MOUSEDRAG9_CONTROL4: keyc = 25769806093;
pub const KEYC_MOUSEDRAG8_CONTROL4: keyc = 25769805837;
pub const KEYC_MOUSEDRAG7_CONTROL4: keyc = 25769805581;
pub const KEYC_MOUSEDRAG6_CONTROL4: keyc = 25769805325;
pub const KEYC_MOUSEDRAG3_CONTROL4: keyc = 25769804557;
pub const KEYC_MOUSEDRAG2_CONTROL4: keyc = 25769804301;
pub const KEYC_MOUSEDRAG1_CONTROL4: keyc = 25769804045;
pub const KEYC_MOUSEDRAG_CONTROL4: keyc = 25769803789;
pub const KEYC_MOUSEDRAG11_CONTROL3: keyc = 25769806604;
pub const KEYC_MOUSEDRAG10_CONTROL3: keyc = 25769806348;
pub const KEYC_MOUSEDRAG9_CONTROL3: keyc = 25769806092;
pub const KEYC_MOUSEDRAG8_CONTROL3: keyc = 25769805836;
pub const KEYC_MOUSEDRAG7_CONTROL3: keyc = 25769805580;
pub const KEYC_MOUSEDRAG6_CONTROL3: keyc = 25769805324;
pub const KEYC_MOUSEDRAG3_CONTROL3: keyc = 25769804556;
pub const KEYC_MOUSEDRAG2_CONTROL3: keyc = 25769804300;
pub const KEYC_MOUSEDRAG1_CONTROL3: keyc = 25769804044;
pub const KEYC_MOUSEDRAG_CONTROL3: keyc = 25769803788;
pub const KEYC_MOUSEDRAG11_CONTROL2: keyc = 25769806603;
pub const KEYC_MOUSEDRAG10_CONTROL2: keyc = 25769806347;
pub const KEYC_MOUSEDRAG9_CONTROL2: keyc = 25769806091;
pub const KEYC_MOUSEDRAG8_CONTROL2: keyc = 25769805835;
pub const KEYC_MOUSEDRAG7_CONTROL2: keyc = 25769805579;
pub const KEYC_MOUSEDRAG6_CONTROL2: keyc = 25769805323;
pub const KEYC_MOUSEDRAG3_CONTROL2: keyc = 25769804555;
pub const KEYC_MOUSEDRAG2_CONTROL2: keyc = 25769804299;
pub const KEYC_MOUSEDRAG1_CONTROL2: keyc = 25769804043;
pub const KEYC_MOUSEDRAG_CONTROL2: keyc = 25769803787;
pub const KEYC_MOUSEDRAG11_CONTROL1: keyc = 25769806602;
pub const KEYC_MOUSEDRAG10_CONTROL1: keyc = 25769806346;
pub const KEYC_MOUSEDRAG9_CONTROL1: keyc = 25769806090;
pub const KEYC_MOUSEDRAG8_CONTROL1: keyc = 25769805834;
pub const KEYC_MOUSEDRAG7_CONTROL1: keyc = 25769805578;
pub const KEYC_MOUSEDRAG6_CONTROL1: keyc = 25769805322;
pub const KEYC_MOUSEDRAG3_CONTROL1: keyc = 25769804554;
pub const KEYC_MOUSEDRAG2_CONTROL1: keyc = 25769804298;
pub const KEYC_MOUSEDRAG1_CONTROL1: keyc = 25769804042;
pub const KEYC_MOUSEDRAG_CONTROL1: keyc = 25769803786;
pub const KEYC_MOUSEDRAG11_CONTROL0: keyc = 25769806601;
pub const KEYC_MOUSEDRAG10_CONTROL0: keyc = 25769806345;
pub const KEYC_MOUSEDRAG9_CONTROL0: keyc = 25769806089;
pub const KEYC_MOUSEDRAG8_CONTROL0: keyc = 25769805833;
pub const KEYC_MOUSEDRAG7_CONTROL0: keyc = 25769805577;
pub const KEYC_MOUSEDRAG6_CONTROL0: keyc = 25769805321;
pub const KEYC_MOUSEDRAG3_CONTROL0: keyc = 25769804553;
pub const KEYC_MOUSEDRAG2_CONTROL0: keyc = 25769804297;
pub const KEYC_MOUSEDRAG1_CONTROL0: keyc = 25769804041;
pub const KEYC_MOUSEDRAG_CONTROL0: keyc = 25769803785;
pub const KEYC_MOUSEDRAG11_SCROLLBAR_DOWN: keyc = 25769806600;
pub const KEYC_MOUSEDRAG10_SCROLLBAR_DOWN: keyc = 25769806344;
pub const KEYC_MOUSEDRAG9_SCROLLBAR_DOWN: keyc = 25769806088;
pub const KEYC_MOUSEDRAG8_SCROLLBAR_DOWN: keyc = 25769805832;
pub const KEYC_MOUSEDRAG7_SCROLLBAR_DOWN: keyc = 25769805576;
pub const KEYC_MOUSEDRAG6_SCROLLBAR_DOWN: keyc = 25769805320;
pub const KEYC_MOUSEDRAG3_SCROLLBAR_DOWN: keyc = 25769804552;
pub const KEYC_MOUSEDRAG2_SCROLLBAR_DOWN: keyc = 25769804296;
pub const KEYC_MOUSEDRAG1_SCROLLBAR_DOWN: keyc = 25769804040;
pub const KEYC_MOUSEDRAG_SCROLLBAR_DOWN: keyc = 25769803784;
pub const KEYC_MOUSEDRAG11_SCROLLBAR_SLIDER: keyc = 25769806599;
pub const KEYC_MOUSEDRAG10_SCROLLBAR_SLIDER: keyc = 25769806343;
pub const KEYC_MOUSEDRAG9_SCROLLBAR_SLIDER: keyc = 25769806087;
pub const KEYC_MOUSEDRAG8_SCROLLBAR_SLIDER: keyc = 25769805831;
pub const KEYC_MOUSEDRAG7_SCROLLBAR_SLIDER: keyc = 25769805575;
pub const KEYC_MOUSEDRAG6_SCROLLBAR_SLIDER: keyc = 25769805319;
pub const KEYC_MOUSEDRAG3_SCROLLBAR_SLIDER: keyc = 25769804551;
pub const KEYC_MOUSEDRAG2_SCROLLBAR_SLIDER: keyc = 25769804295;
pub const KEYC_MOUSEDRAG1_SCROLLBAR_SLIDER: keyc = 25769804039;
pub const KEYC_MOUSEDRAG_SCROLLBAR_SLIDER: keyc = 25769803783;
pub const KEYC_MOUSEDRAG11_SCROLLBAR_UP: keyc = 25769806598;
pub const KEYC_MOUSEDRAG10_SCROLLBAR_UP: keyc = 25769806342;
pub const KEYC_MOUSEDRAG9_SCROLLBAR_UP: keyc = 25769806086;
pub const KEYC_MOUSEDRAG8_SCROLLBAR_UP: keyc = 25769805830;
pub const KEYC_MOUSEDRAG7_SCROLLBAR_UP: keyc = 25769805574;
pub const KEYC_MOUSEDRAG6_SCROLLBAR_UP: keyc = 25769805318;
pub const KEYC_MOUSEDRAG3_SCROLLBAR_UP: keyc = 25769804550;
pub const KEYC_MOUSEDRAG2_SCROLLBAR_UP: keyc = 25769804294;
pub const KEYC_MOUSEDRAG1_SCROLLBAR_UP: keyc = 25769804038;
pub const KEYC_MOUSEDRAG_SCROLLBAR_UP: keyc = 25769803782;
pub const KEYC_MOUSEDRAG11_BORDER: keyc = 25769806597;
pub const KEYC_MOUSEDRAG10_BORDER: keyc = 25769806341;
pub const KEYC_MOUSEDRAG9_BORDER: keyc = 25769806085;
pub const KEYC_MOUSEDRAG8_BORDER: keyc = 25769805829;
pub const KEYC_MOUSEDRAG7_BORDER: keyc = 25769805573;
pub const KEYC_MOUSEDRAG6_BORDER: keyc = 25769805317;
pub const KEYC_MOUSEDRAG3_BORDER: keyc = 25769804549;
pub const KEYC_MOUSEDRAG2_BORDER: keyc = 25769804293;
pub const KEYC_MOUSEDRAG1_BORDER: keyc = 25769804037;
pub const KEYC_MOUSEDRAG_BORDER: keyc = 25769803781;
pub const KEYC_MOUSEDRAG11_STATUS_DEFAULT: keyc = 25769806596;
pub const KEYC_MOUSEDRAG10_STATUS_DEFAULT: keyc = 25769806340;
pub const KEYC_MOUSEDRAG9_STATUS_DEFAULT: keyc = 25769806084;
pub const KEYC_MOUSEDRAG8_STATUS_DEFAULT: keyc = 25769805828;
pub const KEYC_MOUSEDRAG7_STATUS_DEFAULT: keyc = 25769805572;
pub const KEYC_MOUSEDRAG6_STATUS_DEFAULT: keyc = 25769805316;
pub const KEYC_MOUSEDRAG3_STATUS_DEFAULT: keyc = 25769804548;
pub const KEYC_MOUSEDRAG2_STATUS_DEFAULT: keyc = 25769804292;
pub const KEYC_MOUSEDRAG1_STATUS_DEFAULT: keyc = 25769804036;
pub const KEYC_MOUSEDRAG_STATUS_DEFAULT: keyc = 25769803780;
pub const KEYC_MOUSEDRAG11_STATUS_RIGHT: keyc = 25769806595;
pub const KEYC_MOUSEDRAG10_STATUS_RIGHT: keyc = 25769806339;
pub const KEYC_MOUSEDRAG9_STATUS_RIGHT: keyc = 25769806083;
pub const KEYC_MOUSEDRAG8_STATUS_RIGHT: keyc = 25769805827;
pub const KEYC_MOUSEDRAG7_STATUS_RIGHT: keyc = 25769805571;
pub const KEYC_MOUSEDRAG6_STATUS_RIGHT: keyc = 25769805315;
pub const KEYC_MOUSEDRAG3_STATUS_RIGHT: keyc = 25769804547;
pub const KEYC_MOUSEDRAG2_STATUS_RIGHT: keyc = 25769804291;
pub const KEYC_MOUSEDRAG1_STATUS_RIGHT: keyc = 25769804035;
pub const KEYC_MOUSEDRAG_STATUS_RIGHT: keyc = 25769803779;
pub const KEYC_MOUSEDRAG11_STATUS_LEFT: keyc = 25769806594;
pub const KEYC_MOUSEDRAG10_STATUS_LEFT: keyc = 25769806338;
pub const KEYC_MOUSEDRAG9_STATUS_LEFT: keyc = 25769806082;
pub const KEYC_MOUSEDRAG8_STATUS_LEFT: keyc = 25769805826;
pub const KEYC_MOUSEDRAG7_STATUS_LEFT: keyc = 25769805570;
pub const KEYC_MOUSEDRAG6_STATUS_LEFT: keyc = 25769805314;
pub const KEYC_MOUSEDRAG3_STATUS_LEFT: keyc = 25769804546;
pub const KEYC_MOUSEDRAG2_STATUS_LEFT: keyc = 25769804290;
pub const KEYC_MOUSEDRAG1_STATUS_LEFT: keyc = 25769804034;
pub const KEYC_MOUSEDRAG_STATUS_LEFT: keyc = 25769803778;
pub const KEYC_MOUSEDRAG11_STATUS: keyc = 25769806593;
pub const KEYC_MOUSEDRAG10_STATUS: keyc = 25769806337;
pub const KEYC_MOUSEDRAG9_STATUS: keyc = 25769806081;
pub const KEYC_MOUSEDRAG8_STATUS: keyc = 25769805825;
pub const KEYC_MOUSEDRAG7_STATUS: keyc = 25769805569;
pub const KEYC_MOUSEDRAG6_STATUS: keyc = 25769805313;
pub const KEYC_MOUSEDRAG3_STATUS: keyc = 25769804545;
pub const KEYC_MOUSEDRAG2_STATUS: keyc = 25769804289;
pub const KEYC_MOUSEDRAG1_STATUS: keyc = 25769804033;
pub const KEYC_MOUSEDRAG_STATUS: keyc = 25769803777;
pub const KEYC_MOUSEDRAG11_PANE: keyc = 25769806592;
pub const KEYC_MOUSEDRAG10_PANE: keyc = 25769806336;
pub const KEYC_MOUSEDRAG9_PANE: keyc = 25769806080;
pub const KEYC_MOUSEDRAG8_PANE: keyc = 25769805824;
pub const KEYC_MOUSEDRAG7_PANE: keyc = 25769805568;
pub const KEYC_MOUSEDRAG6_PANE: keyc = 25769805312;
pub const KEYC_MOUSEDRAG3_PANE: keyc = 25769804544;
pub const KEYC_MOUSEDRAG2_PANE: keyc = 25769804288;
pub const KEYC_MOUSEDRAG1_PANE: keyc = 25769804032;
pub const KEYC_MOUSEDRAG_PANE: keyc = 25769803776;
pub const KEYC_MOUSEUP11_CONTROL9: keyc = 21474839314;
pub const KEYC_MOUSEUP10_CONTROL9: keyc = 21474839058;
pub const KEYC_MOUSEUP9_CONTROL9: keyc = 21474838802;
pub const KEYC_MOUSEUP8_CONTROL9: keyc = 21474838546;
pub const KEYC_MOUSEUP7_CONTROL9: keyc = 21474838290;
pub const KEYC_MOUSEUP6_CONTROL9: keyc = 21474838034;
pub const KEYC_MOUSEUP3_CONTROL9: keyc = 21474837266;
pub const KEYC_MOUSEUP2_CONTROL9: keyc = 21474837010;
pub const KEYC_MOUSEUP1_CONTROL9: keyc = 21474836754;
pub const KEYC_MOUSEUP_CONTROL9: keyc = 21474836498;
pub const KEYC_MOUSEUP11_CONTROL8: keyc = 21474839313;
pub const KEYC_MOUSEUP10_CONTROL8: keyc = 21474839057;
pub const KEYC_MOUSEUP9_CONTROL8: keyc = 21474838801;
pub const KEYC_MOUSEUP8_CONTROL8: keyc = 21474838545;
pub const KEYC_MOUSEUP7_CONTROL8: keyc = 21474838289;
pub const KEYC_MOUSEUP6_CONTROL8: keyc = 21474838033;
pub const KEYC_MOUSEUP3_CONTROL8: keyc = 21474837265;
pub const KEYC_MOUSEUP2_CONTROL8: keyc = 21474837009;
pub const KEYC_MOUSEUP1_CONTROL8: keyc = 21474836753;
pub const KEYC_MOUSEUP_CONTROL8: keyc = 21474836497;
pub const KEYC_MOUSEUP11_CONTROL7: keyc = 21474839312;
pub const KEYC_MOUSEUP10_CONTROL7: keyc = 21474839056;
pub const KEYC_MOUSEUP9_CONTROL7: keyc = 21474838800;
pub const KEYC_MOUSEUP8_CONTROL7: keyc = 21474838544;
pub const KEYC_MOUSEUP7_CONTROL7: keyc = 21474838288;
pub const KEYC_MOUSEUP6_CONTROL7: keyc = 21474838032;
pub const KEYC_MOUSEUP3_CONTROL7: keyc = 21474837264;
pub const KEYC_MOUSEUP2_CONTROL7: keyc = 21474837008;
pub const KEYC_MOUSEUP1_CONTROL7: keyc = 21474836752;
pub const KEYC_MOUSEUP_CONTROL7: keyc = 21474836496;
pub const KEYC_MOUSEUP11_CONTROL6: keyc = 21474839311;
pub const KEYC_MOUSEUP10_CONTROL6: keyc = 21474839055;
pub const KEYC_MOUSEUP9_CONTROL6: keyc = 21474838799;
pub const KEYC_MOUSEUP8_CONTROL6: keyc = 21474838543;
pub const KEYC_MOUSEUP7_CONTROL6: keyc = 21474838287;
pub const KEYC_MOUSEUP6_CONTROL6: keyc = 21474838031;
pub const KEYC_MOUSEUP3_CONTROL6: keyc = 21474837263;
pub const KEYC_MOUSEUP2_CONTROL6: keyc = 21474837007;
pub const KEYC_MOUSEUP1_CONTROL6: keyc = 21474836751;
pub const KEYC_MOUSEUP_CONTROL6: keyc = 21474836495;
pub const KEYC_MOUSEUP11_CONTROL5: keyc = 21474839310;
pub const KEYC_MOUSEUP10_CONTROL5: keyc = 21474839054;
pub const KEYC_MOUSEUP9_CONTROL5: keyc = 21474838798;
pub const KEYC_MOUSEUP8_CONTROL5: keyc = 21474838542;
pub const KEYC_MOUSEUP7_CONTROL5: keyc = 21474838286;
pub const KEYC_MOUSEUP6_CONTROL5: keyc = 21474838030;
pub const KEYC_MOUSEUP3_CONTROL5: keyc = 21474837262;
pub const KEYC_MOUSEUP2_CONTROL5: keyc = 21474837006;
pub const KEYC_MOUSEUP1_CONTROL5: keyc = 21474836750;
pub const KEYC_MOUSEUP_CONTROL5: keyc = 21474836494;
pub const KEYC_MOUSEUP11_CONTROL4: keyc = 21474839309;
pub const KEYC_MOUSEUP10_CONTROL4: keyc = 21474839053;
pub const KEYC_MOUSEUP9_CONTROL4: keyc = 21474838797;
pub const KEYC_MOUSEUP8_CONTROL4: keyc = 21474838541;
pub const KEYC_MOUSEUP7_CONTROL4: keyc = 21474838285;
pub const KEYC_MOUSEUP6_CONTROL4: keyc = 21474838029;
pub const KEYC_MOUSEUP3_CONTROL4: keyc = 21474837261;
pub const KEYC_MOUSEUP2_CONTROL4: keyc = 21474837005;
pub const KEYC_MOUSEUP1_CONTROL4: keyc = 21474836749;
pub const KEYC_MOUSEUP_CONTROL4: keyc = 21474836493;
pub const KEYC_MOUSEUP11_CONTROL3: keyc = 21474839308;
pub const KEYC_MOUSEUP10_CONTROL3: keyc = 21474839052;
pub const KEYC_MOUSEUP9_CONTROL3: keyc = 21474838796;
pub const KEYC_MOUSEUP8_CONTROL3: keyc = 21474838540;
pub const KEYC_MOUSEUP7_CONTROL3: keyc = 21474838284;
pub const KEYC_MOUSEUP6_CONTROL3: keyc = 21474838028;
pub const KEYC_MOUSEUP3_CONTROL3: keyc = 21474837260;
pub const KEYC_MOUSEUP2_CONTROL3: keyc = 21474837004;
pub const KEYC_MOUSEUP1_CONTROL3: keyc = 21474836748;
pub const KEYC_MOUSEUP_CONTROL3: keyc = 21474836492;
pub const KEYC_MOUSEUP11_CONTROL2: keyc = 21474839307;
pub const KEYC_MOUSEUP10_CONTROL2: keyc = 21474839051;
pub const KEYC_MOUSEUP9_CONTROL2: keyc = 21474838795;
pub const KEYC_MOUSEUP8_CONTROL2: keyc = 21474838539;
pub const KEYC_MOUSEUP7_CONTROL2: keyc = 21474838283;
pub const KEYC_MOUSEUP6_CONTROL2: keyc = 21474838027;
pub const KEYC_MOUSEUP3_CONTROL2: keyc = 21474837259;
pub const KEYC_MOUSEUP2_CONTROL2: keyc = 21474837003;
pub const KEYC_MOUSEUP1_CONTROL2: keyc = 21474836747;
pub const KEYC_MOUSEUP_CONTROL2: keyc = 21474836491;
pub const KEYC_MOUSEUP11_CONTROL1: keyc = 21474839306;
pub const KEYC_MOUSEUP10_CONTROL1: keyc = 21474839050;
pub const KEYC_MOUSEUP9_CONTROL1: keyc = 21474838794;
pub const KEYC_MOUSEUP8_CONTROL1: keyc = 21474838538;
pub const KEYC_MOUSEUP7_CONTROL1: keyc = 21474838282;
pub const KEYC_MOUSEUP6_CONTROL1: keyc = 21474838026;
pub const KEYC_MOUSEUP3_CONTROL1: keyc = 21474837258;
pub const KEYC_MOUSEUP2_CONTROL1: keyc = 21474837002;
pub const KEYC_MOUSEUP1_CONTROL1: keyc = 21474836746;
pub const KEYC_MOUSEUP_CONTROL1: keyc = 21474836490;
pub const KEYC_MOUSEUP11_CONTROL0: keyc = 21474839305;
pub const KEYC_MOUSEUP10_CONTROL0: keyc = 21474839049;
pub const KEYC_MOUSEUP9_CONTROL0: keyc = 21474838793;
pub const KEYC_MOUSEUP8_CONTROL0: keyc = 21474838537;
pub const KEYC_MOUSEUP7_CONTROL0: keyc = 21474838281;
pub const KEYC_MOUSEUP6_CONTROL0: keyc = 21474838025;
pub const KEYC_MOUSEUP3_CONTROL0: keyc = 21474837257;
pub const KEYC_MOUSEUP2_CONTROL0: keyc = 21474837001;
pub const KEYC_MOUSEUP1_CONTROL0: keyc = 21474836745;
pub const KEYC_MOUSEUP_CONTROL0: keyc = 21474836489;
pub const KEYC_MOUSEUP11_SCROLLBAR_DOWN: keyc = 21474839304;
pub const KEYC_MOUSEUP10_SCROLLBAR_DOWN: keyc = 21474839048;
pub const KEYC_MOUSEUP9_SCROLLBAR_DOWN: keyc = 21474838792;
pub const KEYC_MOUSEUP8_SCROLLBAR_DOWN: keyc = 21474838536;
pub const KEYC_MOUSEUP7_SCROLLBAR_DOWN: keyc = 21474838280;
pub const KEYC_MOUSEUP6_SCROLLBAR_DOWN: keyc = 21474838024;
pub const KEYC_MOUSEUP3_SCROLLBAR_DOWN: keyc = 21474837256;
pub const KEYC_MOUSEUP2_SCROLLBAR_DOWN: keyc = 21474837000;
pub const KEYC_MOUSEUP1_SCROLLBAR_DOWN: keyc = 21474836744;
pub const KEYC_MOUSEUP_SCROLLBAR_DOWN: keyc = 21474836488;
pub const KEYC_MOUSEUP11_SCROLLBAR_SLIDER: keyc = 21474839303;
pub const KEYC_MOUSEUP10_SCROLLBAR_SLIDER: keyc = 21474839047;
pub const KEYC_MOUSEUP9_SCROLLBAR_SLIDER: keyc = 21474838791;
pub const KEYC_MOUSEUP8_SCROLLBAR_SLIDER: keyc = 21474838535;
pub const KEYC_MOUSEUP7_SCROLLBAR_SLIDER: keyc = 21474838279;
pub const KEYC_MOUSEUP6_SCROLLBAR_SLIDER: keyc = 21474838023;
pub const KEYC_MOUSEUP3_SCROLLBAR_SLIDER: keyc = 21474837255;
pub const KEYC_MOUSEUP2_SCROLLBAR_SLIDER: keyc = 21474836999;
pub const KEYC_MOUSEUP1_SCROLLBAR_SLIDER: keyc = 21474836743;
pub const KEYC_MOUSEUP_SCROLLBAR_SLIDER: keyc = 21474836487;
pub const KEYC_MOUSEUP11_SCROLLBAR_UP: keyc = 21474839302;
pub const KEYC_MOUSEUP10_SCROLLBAR_UP: keyc = 21474839046;
pub const KEYC_MOUSEUP9_SCROLLBAR_UP: keyc = 21474838790;
pub const KEYC_MOUSEUP8_SCROLLBAR_UP: keyc = 21474838534;
pub const KEYC_MOUSEUP7_SCROLLBAR_UP: keyc = 21474838278;
pub const KEYC_MOUSEUP6_SCROLLBAR_UP: keyc = 21474838022;
pub const KEYC_MOUSEUP3_SCROLLBAR_UP: keyc = 21474837254;
pub const KEYC_MOUSEUP2_SCROLLBAR_UP: keyc = 21474836998;
pub const KEYC_MOUSEUP1_SCROLLBAR_UP: keyc = 21474836742;
pub const KEYC_MOUSEUP_SCROLLBAR_UP: keyc = 21474836486;
pub const KEYC_MOUSEUP11_BORDER: keyc = 21474839301;
pub const KEYC_MOUSEUP10_BORDER: keyc = 21474839045;
pub const KEYC_MOUSEUP9_BORDER: keyc = 21474838789;
pub const KEYC_MOUSEUP8_BORDER: keyc = 21474838533;
pub const KEYC_MOUSEUP7_BORDER: keyc = 21474838277;
pub const KEYC_MOUSEUP6_BORDER: keyc = 21474838021;
pub const KEYC_MOUSEUP3_BORDER: keyc = 21474837253;
pub const KEYC_MOUSEUP2_BORDER: keyc = 21474836997;
pub const KEYC_MOUSEUP1_BORDER: keyc = 21474836741;
pub const KEYC_MOUSEUP_BORDER: keyc = 21474836485;
pub const KEYC_MOUSEUP11_STATUS_DEFAULT: keyc = 21474839300;
pub const KEYC_MOUSEUP10_STATUS_DEFAULT: keyc = 21474839044;
pub const KEYC_MOUSEUP9_STATUS_DEFAULT: keyc = 21474838788;
pub const KEYC_MOUSEUP8_STATUS_DEFAULT: keyc = 21474838532;
pub const KEYC_MOUSEUP7_STATUS_DEFAULT: keyc = 21474838276;
pub const KEYC_MOUSEUP6_STATUS_DEFAULT: keyc = 21474838020;
pub const KEYC_MOUSEUP3_STATUS_DEFAULT: keyc = 21474837252;
pub const KEYC_MOUSEUP2_STATUS_DEFAULT: keyc = 21474836996;
pub const KEYC_MOUSEUP1_STATUS_DEFAULT: keyc = 21474836740;
pub const KEYC_MOUSEUP_STATUS_DEFAULT: keyc = 21474836484;
pub const KEYC_MOUSEUP11_STATUS_RIGHT: keyc = 21474839299;
pub const KEYC_MOUSEUP10_STATUS_RIGHT: keyc = 21474839043;
pub const KEYC_MOUSEUP9_STATUS_RIGHT: keyc = 21474838787;
pub const KEYC_MOUSEUP8_STATUS_RIGHT: keyc = 21474838531;
pub const KEYC_MOUSEUP7_STATUS_RIGHT: keyc = 21474838275;
pub const KEYC_MOUSEUP6_STATUS_RIGHT: keyc = 21474838019;
pub const KEYC_MOUSEUP3_STATUS_RIGHT: keyc = 21474837251;
pub const KEYC_MOUSEUP2_STATUS_RIGHT: keyc = 21474836995;
pub const KEYC_MOUSEUP1_STATUS_RIGHT: keyc = 21474836739;
pub const KEYC_MOUSEUP_STATUS_RIGHT: keyc = 21474836483;
pub const KEYC_MOUSEUP11_STATUS_LEFT: keyc = 21474839298;
pub const KEYC_MOUSEUP10_STATUS_LEFT: keyc = 21474839042;
pub const KEYC_MOUSEUP9_STATUS_LEFT: keyc = 21474838786;
pub const KEYC_MOUSEUP8_STATUS_LEFT: keyc = 21474838530;
pub const KEYC_MOUSEUP7_STATUS_LEFT: keyc = 21474838274;
pub const KEYC_MOUSEUP6_STATUS_LEFT: keyc = 21474838018;
pub const KEYC_MOUSEUP3_STATUS_LEFT: keyc = 21474837250;
pub const KEYC_MOUSEUP2_STATUS_LEFT: keyc = 21474836994;
pub const KEYC_MOUSEUP1_STATUS_LEFT: keyc = 21474836738;
pub const KEYC_MOUSEUP_STATUS_LEFT: keyc = 21474836482;
pub const KEYC_MOUSEUP11_STATUS: keyc = 21474839297;
pub const KEYC_MOUSEUP10_STATUS: keyc = 21474839041;
pub const KEYC_MOUSEUP9_STATUS: keyc = 21474838785;
pub const KEYC_MOUSEUP8_STATUS: keyc = 21474838529;
pub const KEYC_MOUSEUP7_STATUS: keyc = 21474838273;
pub const KEYC_MOUSEUP6_STATUS: keyc = 21474838017;
pub const KEYC_MOUSEUP3_STATUS: keyc = 21474837249;
pub const KEYC_MOUSEUP2_STATUS: keyc = 21474836993;
pub const KEYC_MOUSEUP1_STATUS: keyc = 21474836737;
pub const KEYC_MOUSEUP_STATUS: keyc = 21474836481;
pub const KEYC_MOUSEUP11_PANE: keyc = 21474839296;
pub const KEYC_MOUSEUP10_PANE: keyc = 21474839040;
pub const KEYC_MOUSEUP9_PANE: keyc = 21474838784;
pub const KEYC_MOUSEUP8_PANE: keyc = 21474838528;
pub const KEYC_MOUSEUP7_PANE: keyc = 21474838272;
pub const KEYC_MOUSEUP6_PANE: keyc = 21474838016;
pub const KEYC_MOUSEUP3_PANE: keyc = 21474837248;
pub const KEYC_MOUSEUP2_PANE: keyc = 21474836992;
pub const KEYC_MOUSEUP1_PANE: keyc = 21474836736;
pub const KEYC_MOUSEUP_PANE: keyc = 21474836480;
pub const KEYC_MOUSEDOWN11_CONTROL9: keyc = 17179872018;
pub const KEYC_MOUSEDOWN10_CONTROL9: keyc = 17179871762;
pub const KEYC_MOUSEDOWN9_CONTROL9: keyc = 17179871506;
pub const KEYC_MOUSEDOWN8_CONTROL9: keyc = 17179871250;
pub const KEYC_MOUSEDOWN7_CONTROL9: keyc = 17179870994;
pub const KEYC_MOUSEDOWN6_CONTROL9: keyc = 17179870738;
pub const KEYC_MOUSEDOWN3_CONTROL9: keyc = 17179869970;
pub const KEYC_MOUSEDOWN2_CONTROL9: keyc = 17179869714;
pub const KEYC_MOUSEDOWN1_CONTROL9: keyc = 17179869458;
pub const KEYC_MOUSEDOWN_CONTROL9: keyc = 17179869202;
pub const KEYC_MOUSEDOWN11_CONTROL8: keyc = 17179872017;
pub const KEYC_MOUSEDOWN10_CONTROL8: keyc = 17179871761;
pub const KEYC_MOUSEDOWN9_CONTROL8: keyc = 17179871505;
pub const KEYC_MOUSEDOWN8_CONTROL8: keyc = 17179871249;
pub const KEYC_MOUSEDOWN7_CONTROL8: keyc = 17179870993;
pub const KEYC_MOUSEDOWN6_CONTROL8: keyc = 17179870737;
pub const KEYC_MOUSEDOWN3_CONTROL8: keyc = 17179869969;
pub const KEYC_MOUSEDOWN2_CONTROL8: keyc = 17179869713;
pub const KEYC_MOUSEDOWN1_CONTROL8: keyc = 17179869457;
pub const KEYC_MOUSEDOWN_CONTROL8: keyc = 17179869201;
pub const KEYC_MOUSEDOWN11_CONTROL7: keyc = 17179872016;
pub const KEYC_MOUSEDOWN10_CONTROL7: keyc = 17179871760;
pub const KEYC_MOUSEDOWN9_CONTROL7: keyc = 17179871504;
pub const KEYC_MOUSEDOWN8_CONTROL7: keyc = 17179871248;
pub const KEYC_MOUSEDOWN7_CONTROL7: keyc = 17179870992;
pub const KEYC_MOUSEDOWN6_CONTROL7: keyc = 17179870736;
pub const KEYC_MOUSEDOWN3_CONTROL7: keyc = 17179869968;
pub const KEYC_MOUSEDOWN2_CONTROL7: keyc = 17179869712;
pub const KEYC_MOUSEDOWN1_CONTROL7: keyc = 17179869456;
pub const KEYC_MOUSEDOWN_CONTROL7: keyc = 17179869200;
pub const KEYC_MOUSEDOWN11_CONTROL6: keyc = 17179872015;
pub const KEYC_MOUSEDOWN10_CONTROL6: keyc = 17179871759;
pub const KEYC_MOUSEDOWN9_CONTROL6: keyc = 17179871503;
pub const KEYC_MOUSEDOWN8_CONTROL6: keyc = 17179871247;
pub const KEYC_MOUSEDOWN7_CONTROL6: keyc = 17179870991;
pub const KEYC_MOUSEDOWN6_CONTROL6: keyc = 17179870735;
pub const KEYC_MOUSEDOWN3_CONTROL6: keyc = 17179869967;
pub const KEYC_MOUSEDOWN2_CONTROL6: keyc = 17179869711;
pub const KEYC_MOUSEDOWN1_CONTROL6: keyc = 17179869455;
pub const KEYC_MOUSEDOWN_CONTROL6: keyc = 17179869199;
pub const KEYC_MOUSEDOWN11_CONTROL5: keyc = 17179872014;
pub const KEYC_MOUSEDOWN10_CONTROL5: keyc = 17179871758;
pub const KEYC_MOUSEDOWN9_CONTROL5: keyc = 17179871502;
pub const KEYC_MOUSEDOWN8_CONTROL5: keyc = 17179871246;
pub const KEYC_MOUSEDOWN7_CONTROL5: keyc = 17179870990;
pub const KEYC_MOUSEDOWN6_CONTROL5: keyc = 17179870734;
pub const KEYC_MOUSEDOWN3_CONTROL5: keyc = 17179869966;
pub const KEYC_MOUSEDOWN2_CONTROL5: keyc = 17179869710;
pub const KEYC_MOUSEDOWN1_CONTROL5: keyc = 17179869454;
pub const KEYC_MOUSEDOWN_CONTROL5: keyc = 17179869198;
pub const KEYC_MOUSEDOWN11_CONTROL4: keyc = 17179872013;
pub const KEYC_MOUSEDOWN10_CONTROL4: keyc = 17179871757;
pub const KEYC_MOUSEDOWN9_CONTROL4: keyc = 17179871501;
pub const KEYC_MOUSEDOWN8_CONTROL4: keyc = 17179871245;
pub const KEYC_MOUSEDOWN7_CONTROL4: keyc = 17179870989;
pub const KEYC_MOUSEDOWN6_CONTROL4: keyc = 17179870733;
pub const KEYC_MOUSEDOWN3_CONTROL4: keyc = 17179869965;
pub const KEYC_MOUSEDOWN2_CONTROL4: keyc = 17179869709;
pub const KEYC_MOUSEDOWN1_CONTROL4: keyc = 17179869453;
pub const KEYC_MOUSEDOWN_CONTROL4: keyc = 17179869197;
pub const KEYC_MOUSEDOWN11_CONTROL3: keyc = 17179872012;
pub const KEYC_MOUSEDOWN10_CONTROL3: keyc = 17179871756;
pub const KEYC_MOUSEDOWN9_CONTROL3: keyc = 17179871500;
pub const KEYC_MOUSEDOWN8_CONTROL3: keyc = 17179871244;
pub const KEYC_MOUSEDOWN7_CONTROL3: keyc = 17179870988;
pub const KEYC_MOUSEDOWN6_CONTROL3: keyc = 17179870732;
pub const KEYC_MOUSEDOWN3_CONTROL3: keyc = 17179869964;
pub const KEYC_MOUSEDOWN2_CONTROL3: keyc = 17179869708;
pub const KEYC_MOUSEDOWN1_CONTROL3: keyc = 17179869452;
pub const KEYC_MOUSEDOWN_CONTROL3: keyc = 17179869196;
pub const KEYC_MOUSEDOWN11_CONTROL2: keyc = 17179872011;
pub const KEYC_MOUSEDOWN10_CONTROL2: keyc = 17179871755;
pub const KEYC_MOUSEDOWN9_CONTROL2: keyc = 17179871499;
pub const KEYC_MOUSEDOWN8_CONTROL2: keyc = 17179871243;
pub const KEYC_MOUSEDOWN7_CONTROL2: keyc = 17179870987;
pub const KEYC_MOUSEDOWN6_CONTROL2: keyc = 17179870731;
pub const KEYC_MOUSEDOWN3_CONTROL2: keyc = 17179869963;
pub const KEYC_MOUSEDOWN2_CONTROL2: keyc = 17179869707;
pub const KEYC_MOUSEDOWN1_CONTROL2: keyc = 17179869451;
pub const KEYC_MOUSEDOWN_CONTROL2: keyc = 17179869195;
pub const KEYC_MOUSEDOWN11_CONTROL1: keyc = 17179872010;
pub const KEYC_MOUSEDOWN10_CONTROL1: keyc = 17179871754;
pub const KEYC_MOUSEDOWN9_CONTROL1: keyc = 17179871498;
pub const KEYC_MOUSEDOWN8_CONTROL1: keyc = 17179871242;
pub const KEYC_MOUSEDOWN7_CONTROL1: keyc = 17179870986;
pub const KEYC_MOUSEDOWN6_CONTROL1: keyc = 17179870730;
pub const KEYC_MOUSEDOWN3_CONTROL1: keyc = 17179869962;
pub const KEYC_MOUSEDOWN2_CONTROL1: keyc = 17179869706;
pub const KEYC_MOUSEDOWN1_CONTROL1: keyc = 17179869450;
pub const KEYC_MOUSEDOWN_CONTROL1: keyc = 17179869194;
pub const KEYC_MOUSEDOWN11_CONTROL0: keyc = 17179872009;
pub const KEYC_MOUSEDOWN10_CONTROL0: keyc = 17179871753;
pub const KEYC_MOUSEDOWN9_CONTROL0: keyc = 17179871497;
pub const KEYC_MOUSEDOWN8_CONTROL0: keyc = 17179871241;
pub const KEYC_MOUSEDOWN7_CONTROL0: keyc = 17179870985;
pub const KEYC_MOUSEDOWN6_CONTROL0: keyc = 17179870729;
pub const KEYC_MOUSEDOWN3_CONTROL0: keyc = 17179869961;
pub const KEYC_MOUSEDOWN2_CONTROL0: keyc = 17179869705;
pub const KEYC_MOUSEDOWN1_CONTROL0: keyc = 17179869449;
pub const KEYC_MOUSEDOWN_CONTROL0: keyc = 17179869193;
pub const KEYC_MOUSEDOWN11_SCROLLBAR_DOWN: keyc = 17179872008;
pub const KEYC_MOUSEDOWN10_SCROLLBAR_DOWN: keyc = 17179871752;
pub const KEYC_MOUSEDOWN9_SCROLLBAR_DOWN: keyc = 17179871496;
pub const KEYC_MOUSEDOWN8_SCROLLBAR_DOWN: keyc = 17179871240;
pub const KEYC_MOUSEDOWN7_SCROLLBAR_DOWN: keyc = 17179870984;
pub const KEYC_MOUSEDOWN6_SCROLLBAR_DOWN: keyc = 17179870728;
pub const KEYC_MOUSEDOWN3_SCROLLBAR_DOWN: keyc = 17179869960;
pub const KEYC_MOUSEDOWN2_SCROLLBAR_DOWN: keyc = 17179869704;
pub const KEYC_MOUSEDOWN1_SCROLLBAR_DOWN: keyc = 17179869448;
pub const KEYC_MOUSEDOWN_SCROLLBAR_DOWN: keyc = 17179869192;
pub const KEYC_MOUSEDOWN11_SCROLLBAR_SLIDER: keyc = 17179872007;
pub const KEYC_MOUSEDOWN10_SCROLLBAR_SLIDER: keyc = 17179871751;
pub const KEYC_MOUSEDOWN9_SCROLLBAR_SLIDER: keyc = 17179871495;
pub const KEYC_MOUSEDOWN8_SCROLLBAR_SLIDER: keyc = 17179871239;
pub const KEYC_MOUSEDOWN7_SCROLLBAR_SLIDER: keyc = 17179870983;
pub const KEYC_MOUSEDOWN6_SCROLLBAR_SLIDER: keyc = 17179870727;
pub const KEYC_MOUSEDOWN3_SCROLLBAR_SLIDER: keyc = 17179869959;
pub const KEYC_MOUSEDOWN2_SCROLLBAR_SLIDER: keyc = 17179869703;
pub const KEYC_MOUSEDOWN1_SCROLLBAR_SLIDER: keyc = 17179869447;
pub const KEYC_MOUSEDOWN_SCROLLBAR_SLIDER: keyc = 17179869191;
pub const KEYC_MOUSEDOWN11_SCROLLBAR_UP: keyc = 17179872006;
pub const KEYC_MOUSEDOWN10_SCROLLBAR_UP: keyc = 17179871750;
pub const KEYC_MOUSEDOWN9_SCROLLBAR_UP: keyc = 17179871494;
pub const KEYC_MOUSEDOWN8_SCROLLBAR_UP: keyc = 17179871238;
pub const KEYC_MOUSEDOWN7_SCROLLBAR_UP: keyc = 17179870982;
pub const KEYC_MOUSEDOWN6_SCROLLBAR_UP: keyc = 17179870726;
pub const KEYC_MOUSEDOWN3_SCROLLBAR_UP: keyc = 17179869958;
pub const KEYC_MOUSEDOWN2_SCROLLBAR_UP: keyc = 17179869702;
pub const KEYC_MOUSEDOWN1_SCROLLBAR_UP: keyc = 17179869446;
pub const KEYC_MOUSEDOWN_SCROLLBAR_UP: keyc = 17179869190;
pub const KEYC_MOUSEDOWN11_BORDER: keyc = 17179872005;
pub const KEYC_MOUSEDOWN10_BORDER: keyc = 17179871749;
pub const KEYC_MOUSEDOWN9_BORDER: keyc = 17179871493;
pub const KEYC_MOUSEDOWN8_BORDER: keyc = 17179871237;
pub const KEYC_MOUSEDOWN7_BORDER: keyc = 17179870981;
pub const KEYC_MOUSEDOWN6_BORDER: keyc = 17179870725;
pub const KEYC_MOUSEDOWN3_BORDER: keyc = 17179869957;
pub const KEYC_MOUSEDOWN2_BORDER: keyc = 17179869701;
pub const KEYC_MOUSEDOWN1_BORDER: keyc = 17179869445;
pub const KEYC_MOUSEDOWN_BORDER: keyc = 17179869189;
pub const KEYC_MOUSEDOWN11_STATUS_DEFAULT: keyc = 17179872004;
pub const KEYC_MOUSEDOWN10_STATUS_DEFAULT: keyc = 17179871748;
pub const KEYC_MOUSEDOWN9_STATUS_DEFAULT: keyc = 17179871492;
pub const KEYC_MOUSEDOWN8_STATUS_DEFAULT: keyc = 17179871236;
pub const KEYC_MOUSEDOWN7_STATUS_DEFAULT: keyc = 17179870980;
pub const KEYC_MOUSEDOWN6_STATUS_DEFAULT: keyc = 17179870724;
pub const KEYC_MOUSEDOWN3_STATUS_DEFAULT: keyc = 17179869956;
pub const KEYC_MOUSEDOWN2_STATUS_DEFAULT: keyc = 17179869700;
pub const KEYC_MOUSEDOWN1_STATUS_DEFAULT: keyc = 17179869444;
pub const KEYC_MOUSEDOWN_STATUS_DEFAULT: keyc = 17179869188;
pub const KEYC_MOUSEDOWN11_STATUS_RIGHT: keyc = 17179872003;
pub const KEYC_MOUSEDOWN10_STATUS_RIGHT: keyc = 17179871747;
pub const KEYC_MOUSEDOWN9_STATUS_RIGHT: keyc = 17179871491;
pub const KEYC_MOUSEDOWN8_STATUS_RIGHT: keyc = 17179871235;
pub const KEYC_MOUSEDOWN7_STATUS_RIGHT: keyc = 17179870979;
pub const KEYC_MOUSEDOWN6_STATUS_RIGHT: keyc = 17179870723;
pub const KEYC_MOUSEDOWN3_STATUS_RIGHT: keyc = 17179869955;
pub const KEYC_MOUSEDOWN2_STATUS_RIGHT: keyc = 17179869699;
pub const KEYC_MOUSEDOWN1_STATUS_RIGHT: keyc = 17179869443;
pub const KEYC_MOUSEDOWN_STATUS_RIGHT: keyc = 17179869187;
pub const KEYC_MOUSEDOWN11_STATUS_LEFT: keyc = 17179872002;
pub const KEYC_MOUSEDOWN10_STATUS_LEFT: keyc = 17179871746;
pub const KEYC_MOUSEDOWN9_STATUS_LEFT: keyc = 17179871490;
pub const KEYC_MOUSEDOWN8_STATUS_LEFT: keyc = 17179871234;
pub const KEYC_MOUSEDOWN7_STATUS_LEFT: keyc = 17179870978;
pub const KEYC_MOUSEDOWN6_STATUS_LEFT: keyc = 17179870722;
pub const KEYC_MOUSEDOWN3_STATUS_LEFT: keyc = 17179869954;
pub const KEYC_MOUSEDOWN2_STATUS_LEFT: keyc = 17179869698;
pub const KEYC_MOUSEDOWN1_STATUS_LEFT: keyc = 17179869442;
pub const KEYC_MOUSEDOWN_STATUS_LEFT: keyc = 17179869186;
pub const KEYC_MOUSEDOWN11_STATUS: keyc = 17179872001;
pub const KEYC_MOUSEDOWN10_STATUS: keyc = 17179871745;
pub const KEYC_MOUSEDOWN9_STATUS: keyc = 17179871489;
pub const KEYC_MOUSEDOWN8_STATUS: keyc = 17179871233;
pub const KEYC_MOUSEDOWN7_STATUS: keyc = 17179870977;
pub const KEYC_MOUSEDOWN6_STATUS: keyc = 17179870721;
pub const KEYC_MOUSEDOWN3_STATUS: keyc = 17179869953;
pub const KEYC_MOUSEDOWN2_STATUS: keyc = 17179869697;
pub const KEYC_MOUSEDOWN1_STATUS: keyc = 17179869441;
pub const KEYC_MOUSEDOWN_STATUS: keyc = 17179869185;
pub const KEYC_MOUSEDOWN11_PANE: keyc = 17179872000;
pub const KEYC_MOUSEDOWN10_PANE: keyc = 17179871744;
pub const KEYC_MOUSEDOWN9_PANE: keyc = 17179871488;
pub const KEYC_MOUSEDOWN8_PANE: keyc = 17179871232;
pub const KEYC_MOUSEDOWN7_PANE: keyc = 17179870976;
pub const KEYC_MOUSEDOWN6_PANE: keyc = 17179870720;
pub const KEYC_MOUSEDOWN3_PANE: keyc = 17179869952;
pub const KEYC_MOUSEDOWN2_PANE: keyc = 17179869696;
pub const KEYC_MOUSEDOWN1_PANE: keyc = 17179869440;
pub const KEYC_MOUSEDOWN_PANE: keyc = 17179869184;
pub const KEYC_WHEELUP11_CONTROL9: keyc = 38654708498;
pub const KEYC_WHEELUP10_CONTROL9: keyc = 38654708242;
pub const KEYC_WHEELUP9_CONTROL9: keyc = 38654707986;
pub const KEYC_WHEELUP8_CONTROL9: keyc = 38654707730;
pub const KEYC_WHEELUP7_CONTROL9: keyc = 38654707474;
pub const KEYC_WHEELUP6_CONTROL9: keyc = 38654707218;
pub const KEYC_WHEELUP3_CONTROL9: keyc = 38654706450;
pub const KEYC_WHEELUP2_CONTROL9: keyc = 38654706194;
pub const KEYC_WHEELUP1_CONTROL9: keyc = 38654705938;
pub const KEYC_WHEELUP_CONTROL9: keyc = 38654705682;
pub const KEYC_WHEELUP11_CONTROL8: keyc = 38654708497;
pub const KEYC_WHEELUP10_CONTROL8: keyc = 38654708241;
pub const KEYC_WHEELUP9_CONTROL8: keyc = 38654707985;
pub const KEYC_WHEELUP8_CONTROL8: keyc = 38654707729;
pub const KEYC_WHEELUP7_CONTROL8: keyc = 38654707473;
pub const KEYC_WHEELUP6_CONTROL8: keyc = 38654707217;
pub const KEYC_WHEELUP3_CONTROL8: keyc = 38654706449;
pub const KEYC_WHEELUP2_CONTROL8: keyc = 38654706193;
pub const KEYC_WHEELUP1_CONTROL8: keyc = 38654705937;
pub const KEYC_WHEELUP_CONTROL8: keyc = 38654705681;
pub const KEYC_WHEELUP11_CONTROL7: keyc = 38654708496;
pub const KEYC_WHEELUP10_CONTROL7: keyc = 38654708240;
pub const KEYC_WHEELUP9_CONTROL7: keyc = 38654707984;
pub const KEYC_WHEELUP8_CONTROL7: keyc = 38654707728;
pub const KEYC_WHEELUP7_CONTROL7: keyc = 38654707472;
pub const KEYC_WHEELUP6_CONTROL7: keyc = 38654707216;
pub const KEYC_WHEELUP3_CONTROL7: keyc = 38654706448;
pub const KEYC_WHEELUP2_CONTROL7: keyc = 38654706192;
pub const KEYC_WHEELUP1_CONTROL7: keyc = 38654705936;
pub const KEYC_WHEELUP_CONTROL7: keyc = 38654705680;
pub const KEYC_WHEELUP11_CONTROL6: keyc = 38654708495;
pub const KEYC_WHEELUP10_CONTROL6: keyc = 38654708239;
pub const KEYC_WHEELUP9_CONTROL6: keyc = 38654707983;
pub const KEYC_WHEELUP8_CONTROL6: keyc = 38654707727;
pub const KEYC_WHEELUP7_CONTROL6: keyc = 38654707471;
pub const KEYC_WHEELUP6_CONTROL6: keyc = 38654707215;
pub const KEYC_WHEELUP3_CONTROL6: keyc = 38654706447;
pub const KEYC_WHEELUP2_CONTROL6: keyc = 38654706191;
pub const KEYC_WHEELUP1_CONTROL6: keyc = 38654705935;
pub const KEYC_WHEELUP_CONTROL6: keyc = 38654705679;
pub const KEYC_WHEELUP11_CONTROL5: keyc = 38654708494;
pub const KEYC_WHEELUP10_CONTROL5: keyc = 38654708238;
pub const KEYC_WHEELUP9_CONTROL5: keyc = 38654707982;
pub const KEYC_WHEELUP8_CONTROL5: keyc = 38654707726;
pub const KEYC_WHEELUP7_CONTROL5: keyc = 38654707470;
pub const KEYC_WHEELUP6_CONTROL5: keyc = 38654707214;
pub const KEYC_WHEELUP3_CONTROL5: keyc = 38654706446;
pub const KEYC_WHEELUP2_CONTROL5: keyc = 38654706190;
pub const KEYC_WHEELUP1_CONTROL5: keyc = 38654705934;
pub const KEYC_WHEELUP_CONTROL5: keyc = 38654705678;
pub const KEYC_WHEELUP11_CONTROL4: keyc = 38654708493;
pub const KEYC_WHEELUP10_CONTROL4: keyc = 38654708237;
pub const KEYC_WHEELUP9_CONTROL4: keyc = 38654707981;
pub const KEYC_WHEELUP8_CONTROL4: keyc = 38654707725;
pub const KEYC_WHEELUP7_CONTROL4: keyc = 38654707469;
pub const KEYC_WHEELUP6_CONTROL4: keyc = 38654707213;
pub const KEYC_WHEELUP3_CONTROL4: keyc = 38654706445;
pub const KEYC_WHEELUP2_CONTROL4: keyc = 38654706189;
pub const KEYC_WHEELUP1_CONTROL4: keyc = 38654705933;
pub const KEYC_WHEELUP_CONTROL4: keyc = 38654705677;
pub const KEYC_WHEELUP11_CONTROL3: keyc = 38654708492;
pub const KEYC_WHEELUP10_CONTROL3: keyc = 38654708236;
pub const KEYC_WHEELUP9_CONTROL3: keyc = 38654707980;
pub const KEYC_WHEELUP8_CONTROL3: keyc = 38654707724;
pub const KEYC_WHEELUP7_CONTROL3: keyc = 38654707468;
pub const KEYC_WHEELUP6_CONTROL3: keyc = 38654707212;
pub const KEYC_WHEELUP3_CONTROL3: keyc = 38654706444;
pub const KEYC_WHEELUP2_CONTROL3: keyc = 38654706188;
pub const KEYC_WHEELUP1_CONTROL3: keyc = 38654705932;
pub const KEYC_WHEELUP_CONTROL3: keyc = 38654705676;
pub const KEYC_WHEELUP11_CONTROL2: keyc = 38654708491;
pub const KEYC_WHEELUP10_CONTROL2: keyc = 38654708235;
pub const KEYC_WHEELUP9_CONTROL2: keyc = 38654707979;
pub const KEYC_WHEELUP8_CONTROL2: keyc = 38654707723;
pub const KEYC_WHEELUP7_CONTROL2: keyc = 38654707467;
pub const KEYC_WHEELUP6_CONTROL2: keyc = 38654707211;
pub const KEYC_WHEELUP3_CONTROL2: keyc = 38654706443;
pub const KEYC_WHEELUP2_CONTROL2: keyc = 38654706187;
pub const KEYC_WHEELUP1_CONTROL2: keyc = 38654705931;
pub const KEYC_WHEELUP_CONTROL2: keyc = 38654705675;
pub const KEYC_WHEELUP11_CONTROL1: keyc = 38654708490;
pub const KEYC_WHEELUP10_CONTROL1: keyc = 38654708234;
pub const KEYC_WHEELUP9_CONTROL1: keyc = 38654707978;
pub const KEYC_WHEELUP8_CONTROL1: keyc = 38654707722;
pub const KEYC_WHEELUP7_CONTROL1: keyc = 38654707466;
pub const KEYC_WHEELUP6_CONTROL1: keyc = 38654707210;
pub const KEYC_WHEELUP3_CONTROL1: keyc = 38654706442;
pub const KEYC_WHEELUP2_CONTROL1: keyc = 38654706186;
pub const KEYC_WHEELUP1_CONTROL1: keyc = 38654705930;
pub const KEYC_WHEELUP_CONTROL1: keyc = 38654705674;
pub const KEYC_WHEELUP11_CONTROL0: keyc = 38654708489;
pub const KEYC_WHEELUP10_CONTROL0: keyc = 38654708233;
pub const KEYC_WHEELUP9_CONTROL0: keyc = 38654707977;
pub const KEYC_WHEELUP8_CONTROL0: keyc = 38654707721;
pub const KEYC_WHEELUP7_CONTROL0: keyc = 38654707465;
pub const KEYC_WHEELUP6_CONTROL0: keyc = 38654707209;
pub const KEYC_WHEELUP3_CONTROL0: keyc = 38654706441;
pub const KEYC_WHEELUP2_CONTROL0: keyc = 38654706185;
pub const KEYC_WHEELUP1_CONTROL0: keyc = 38654705929;
pub const KEYC_WHEELUP_CONTROL0: keyc = 38654705673;
pub const KEYC_WHEELUP11_SCROLLBAR_DOWN: keyc = 38654708488;
pub const KEYC_WHEELUP10_SCROLLBAR_DOWN: keyc = 38654708232;
pub const KEYC_WHEELUP9_SCROLLBAR_DOWN: keyc = 38654707976;
pub const KEYC_WHEELUP8_SCROLLBAR_DOWN: keyc = 38654707720;
pub const KEYC_WHEELUP7_SCROLLBAR_DOWN: keyc = 38654707464;
pub const KEYC_WHEELUP6_SCROLLBAR_DOWN: keyc = 38654707208;
pub const KEYC_WHEELUP3_SCROLLBAR_DOWN: keyc = 38654706440;
pub const KEYC_WHEELUP2_SCROLLBAR_DOWN: keyc = 38654706184;
pub const KEYC_WHEELUP1_SCROLLBAR_DOWN: keyc = 38654705928;
pub const KEYC_WHEELUP_SCROLLBAR_DOWN: keyc = 38654705672;
pub const KEYC_WHEELUP11_SCROLLBAR_SLIDER: keyc = 38654708487;
pub const KEYC_WHEELUP10_SCROLLBAR_SLIDER: keyc = 38654708231;
pub const KEYC_WHEELUP9_SCROLLBAR_SLIDER: keyc = 38654707975;
pub const KEYC_WHEELUP8_SCROLLBAR_SLIDER: keyc = 38654707719;
pub const KEYC_WHEELUP7_SCROLLBAR_SLIDER: keyc = 38654707463;
pub const KEYC_WHEELUP6_SCROLLBAR_SLIDER: keyc = 38654707207;
pub const KEYC_WHEELUP3_SCROLLBAR_SLIDER: keyc = 38654706439;
pub const KEYC_WHEELUP2_SCROLLBAR_SLIDER: keyc = 38654706183;
pub const KEYC_WHEELUP1_SCROLLBAR_SLIDER: keyc = 38654705927;
pub const KEYC_WHEELUP_SCROLLBAR_SLIDER: keyc = 38654705671;
pub const KEYC_WHEELUP11_SCROLLBAR_UP: keyc = 38654708486;
pub const KEYC_WHEELUP10_SCROLLBAR_UP: keyc = 38654708230;
pub const KEYC_WHEELUP9_SCROLLBAR_UP: keyc = 38654707974;
pub const KEYC_WHEELUP8_SCROLLBAR_UP: keyc = 38654707718;
pub const KEYC_WHEELUP7_SCROLLBAR_UP: keyc = 38654707462;
pub const KEYC_WHEELUP6_SCROLLBAR_UP: keyc = 38654707206;
pub const KEYC_WHEELUP3_SCROLLBAR_UP: keyc = 38654706438;
pub const KEYC_WHEELUP2_SCROLLBAR_UP: keyc = 38654706182;
pub const KEYC_WHEELUP1_SCROLLBAR_UP: keyc = 38654705926;
pub const KEYC_WHEELUP_SCROLLBAR_UP: keyc = 38654705670;
pub const KEYC_WHEELUP11_BORDER: keyc = 38654708485;
pub const KEYC_WHEELUP10_BORDER: keyc = 38654708229;
pub const KEYC_WHEELUP9_BORDER: keyc = 38654707973;
pub const KEYC_WHEELUP8_BORDER: keyc = 38654707717;
pub const KEYC_WHEELUP7_BORDER: keyc = 38654707461;
pub const KEYC_WHEELUP6_BORDER: keyc = 38654707205;
pub const KEYC_WHEELUP3_BORDER: keyc = 38654706437;
pub const KEYC_WHEELUP2_BORDER: keyc = 38654706181;
pub const KEYC_WHEELUP1_BORDER: keyc = 38654705925;
pub const KEYC_WHEELUP_BORDER: keyc = 38654705669;
pub const KEYC_WHEELUP11_STATUS_DEFAULT: keyc = 38654708484;
pub const KEYC_WHEELUP10_STATUS_DEFAULT: keyc = 38654708228;
pub const KEYC_WHEELUP9_STATUS_DEFAULT: keyc = 38654707972;
pub const KEYC_WHEELUP8_STATUS_DEFAULT: keyc = 38654707716;
pub const KEYC_WHEELUP7_STATUS_DEFAULT: keyc = 38654707460;
pub const KEYC_WHEELUP6_STATUS_DEFAULT: keyc = 38654707204;
pub const KEYC_WHEELUP3_STATUS_DEFAULT: keyc = 38654706436;
pub const KEYC_WHEELUP2_STATUS_DEFAULT: keyc = 38654706180;
pub const KEYC_WHEELUP1_STATUS_DEFAULT: keyc = 38654705924;
pub const KEYC_WHEELUP_STATUS_DEFAULT: keyc = 38654705668;
pub const KEYC_WHEELUP11_STATUS_RIGHT: keyc = 38654708483;
pub const KEYC_WHEELUP10_STATUS_RIGHT: keyc = 38654708227;
pub const KEYC_WHEELUP9_STATUS_RIGHT: keyc = 38654707971;
pub const KEYC_WHEELUP8_STATUS_RIGHT: keyc = 38654707715;
pub const KEYC_WHEELUP7_STATUS_RIGHT: keyc = 38654707459;
pub const KEYC_WHEELUP6_STATUS_RIGHT: keyc = 38654707203;
pub const KEYC_WHEELUP3_STATUS_RIGHT: keyc = 38654706435;
pub const KEYC_WHEELUP2_STATUS_RIGHT: keyc = 38654706179;
pub const KEYC_WHEELUP1_STATUS_RIGHT: keyc = 38654705923;
pub const KEYC_WHEELUP_STATUS_RIGHT: keyc = 38654705667;
pub const KEYC_WHEELUP11_STATUS_LEFT: keyc = 38654708482;
pub const KEYC_WHEELUP10_STATUS_LEFT: keyc = 38654708226;
pub const KEYC_WHEELUP9_STATUS_LEFT: keyc = 38654707970;
pub const KEYC_WHEELUP8_STATUS_LEFT: keyc = 38654707714;
pub const KEYC_WHEELUP7_STATUS_LEFT: keyc = 38654707458;
pub const KEYC_WHEELUP6_STATUS_LEFT: keyc = 38654707202;
pub const KEYC_WHEELUP3_STATUS_LEFT: keyc = 38654706434;
pub const KEYC_WHEELUP2_STATUS_LEFT: keyc = 38654706178;
pub const KEYC_WHEELUP1_STATUS_LEFT: keyc = 38654705922;
pub const KEYC_WHEELUP_STATUS_LEFT: keyc = 38654705666;
pub const KEYC_WHEELUP11_STATUS: keyc = 38654708481;
pub const KEYC_WHEELUP10_STATUS: keyc = 38654708225;
pub const KEYC_WHEELUP9_STATUS: keyc = 38654707969;
pub const KEYC_WHEELUP8_STATUS: keyc = 38654707713;
pub const KEYC_WHEELUP7_STATUS: keyc = 38654707457;
pub const KEYC_WHEELUP6_STATUS: keyc = 38654707201;
pub const KEYC_WHEELUP3_STATUS: keyc = 38654706433;
pub const KEYC_WHEELUP2_STATUS: keyc = 38654706177;
pub const KEYC_WHEELUP1_STATUS: keyc = 38654705921;
pub const KEYC_WHEELUP_STATUS: keyc = 38654705665;
pub const KEYC_WHEELUP11_PANE: keyc = 38654708480;
pub const KEYC_WHEELUP10_PANE: keyc = 38654708224;
pub const KEYC_WHEELUP9_PANE: keyc = 38654707968;
pub const KEYC_WHEELUP8_PANE: keyc = 38654707712;
pub const KEYC_WHEELUP7_PANE: keyc = 38654707456;
pub const KEYC_WHEELUP6_PANE: keyc = 38654707200;
pub const KEYC_WHEELUP3_PANE: keyc = 38654706432;
pub const KEYC_WHEELUP2_PANE: keyc = 38654706176;
pub const KEYC_WHEELUP1_PANE: keyc = 38654705920;
pub const KEYC_WHEELUP_PANE: keyc = 38654705664;
pub const KEYC_WHEELDOWN11_CONTROL9: keyc = 34359741202;
pub const KEYC_WHEELDOWN10_CONTROL9: keyc = 34359740946;
pub const KEYC_WHEELDOWN9_CONTROL9: keyc = 34359740690;
pub const KEYC_WHEELDOWN8_CONTROL9: keyc = 34359740434;
pub const KEYC_WHEELDOWN7_CONTROL9: keyc = 34359740178;
pub const KEYC_WHEELDOWN6_CONTROL9: keyc = 34359739922;
pub const KEYC_WHEELDOWN3_CONTROL9: keyc = 34359739154;
pub const KEYC_WHEELDOWN2_CONTROL9: keyc = 34359738898;
pub const KEYC_WHEELDOWN1_CONTROL9: keyc = 34359738642;
pub const KEYC_WHEELDOWN_CONTROL9: keyc = 34359738386;
pub const KEYC_WHEELDOWN11_CONTROL8: keyc = 34359741201;
pub const KEYC_WHEELDOWN10_CONTROL8: keyc = 34359740945;
pub const KEYC_WHEELDOWN9_CONTROL8: keyc = 34359740689;
pub const KEYC_WHEELDOWN8_CONTROL8: keyc = 34359740433;
pub const KEYC_WHEELDOWN7_CONTROL8: keyc = 34359740177;
pub const KEYC_WHEELDOWN6_CONTROL8: keyc = 34359739921;
pub const KEYC_WHEELDOWN3_CONTROL8: keyc = 34359739153;
pub const KEYC_WHEELDOWN2_CONTROL8: keyc = 34359738897;
pub const KEYC_WHEELDOWN1_CONTROL8: keyc = 34359738641;
pub const KEYC_WHEELDOWN_CONTROL8: keyc = 34359738385;
pub const KEYC_WHEELDOWN11_CONTROL7: keyc = 34359741200;
pub const KEYC_WHEELDOWN10_CONTROL7: keyc = 34359740944;
pub const KEYC_WHEELDOWN9_CONTROL7: keyc = 34359740688;
pub const KEYC_WHEELDOWN8_CONTROL7: keyc = 34359740432;
pub const KEYC_WHEELDOWN7_CONTROL7: keyc = 34359740176;
pub const KEYC_WHEELDOWN6_CONTROL7: keyc = 34359739920;
pub const KEYC_WHEELDOWN3_CONTROL7: keyc = 34359739152;
pub const KEYC_WHEELDOWN2_CONTROL7: keyc = 34359738896;
pub const KEYC_WHEELDOWN1_CONTROL7: keyc = 34359738640;
pub const KEYC_WHEELDOWN_CONTROL7: keyc = 34359738384;
pub const KEYC_WHEELDOWN11_CONTROL6: keyc = 34359741199;
pub const KEYC_WHEELDOWN10_CONTROL6: keyc = 34359740943;
pub const KEYC_WHEELDOWN9_CONTROL6: keyc = 34359740687;
pub const KEYC_WHEELDOWN8_CONTROL6: keyc = 34359740431;
pub const KEYC_WHEELDOWN7_CONTROL6: keyc = 34359740175;
pub const KEYC_WHEELDOWN6_CONTROL6: keyc = 34359739919;
pub const KEYC_WHEELDOWN3_CONTROL6: keyc = 34359739151;
pub const KEYC_WHEELDOWN2_CONTROL6: keyc = 34359738895;
pub const KEYC_WHEELDOWN1_CONTROL6: keyc = 34359738639;
pub const KEYC_WHEELDOWN_CONTROL6: keyc = 34359738383;
pub const KEYC_WHEELDOWN11_CONTROL5: keyc = 34359741198;
pub const KEYC_WHEELDOWN10_CONTROL5: keyc = 34359740942;
pub const KEYC_WHEELDOWN9_CONTROL5: keyc = 34359740686;
pub const KEYC_WHEELDOWN8_CONTROL5: keyc = 34359740430;
pub const KEYC_WHEELDOWN7_CONTROL5: keyc = 34359740174;
pub const KEYC_WHEELDOWN6_CONTROL5: keyc = 34359739918;
pub const KEYC_WHEELDOWN3_CONTROL5: keyc = 34359739150;
pub const KEYC_WHEELDOWN2_CONTROL5: keyc = 34359738894;
pub const KEYC_WHEELDOWN1_CONTROL5: keyc = 34359738638;
pub const KEYC_WHEELDOWN_CONTROL5: keyc = 34359738382;
pub const KEYC_WHEELDOWN11_CONTROL4: keyc = 34359741197;
pub const KEYC_WHEELDOWN10_CONTROL4: keyc = 34359740941;
pub const KEYC_WHEELDOWN9_CONTROL4: keyc = 34359740685;
pub const KEYC_WHEELDOWN8_CONTROL4: keyc = 34359740429;
pub const KEYC_WHEELDOWN7_CONTROL4: keyc = 34359740173;
pub const KEYC_WHEELDOWN6_CONTROL4: keyc = 34359739917;
pub const KEYC_WHEELDOWN3_CONTROL4: keyc = 34359739149;
pub const KEYC_WHEELDOWN2_CONTROL4: keyc = 34359738893;
pub const KEYC_WHEELDOWN1_CONTROL4: keyc = 34359738637;
pub const KEYC_WHEELDOWN_CONTROL4: keyc = 34359738381;
pub const KEYC_WHEELDOWN11_CONTROL3: keyc = 34359741196;
pub const KEYC_WHEELDOWN10_CONTROL3: keyc = 34359740940;
pub const KEYC_WHEELDOWN9_CONTROL3: keyc = 34359740684;
pub const KEYC_WHEELDOWN8_CONTROL3: keyc = 34359740428;
pub const KEYC_WHEELDOWN7_CONTROL3: keyc = 34359740172;
pub const KEYC_WHEELDOWN6_CONTROL3: keyc = 34359739916;
pub const KEYC_WHEELDOWN3_CONTROL3: keyc = 34359739148;
pub const KEYC_WHEELDOWN2_CONTROL3: keyc = 34359738892;
pub const KEYC_WHEELDOWN1_CONTROL3: keyc = 34359738636;
pub const KEYC_WHEELDOWN_CONTROL3: keyc = 34359738380;
pub const KEYC_WHEELDOWN11_CONTROL2: keyc = 34359741195;
pub const KEYC_WHEELDOWN10_CONTROL2: keyc = 34359740939;
pub const KEYC_WHEELDOWN9_CONTROL2: keyc = 34359740683;
pub const KEYC_WHEELDOWN8_CONTROL2: keyc = 34359740427;
pub const KEYC_WHEELDOWN7_CONTROL2: keyc = 34359740171;
pub const KEYC_WHEELDOWN6_CONTROL2: keyc = 34359739915;
pub const KEYC_WHEELDOWN3_CONTROL2: keyc = 34359739147;
pub const KEYC_WHEELDOWN2_CONTROL2: keyc = 34359738891;
pub const KEYC_WHEELDOWN1_CONTROL2: keyc = 34359738635;
pub const KEYC_WHEELDOWN_CONTROL2: keyc = 34359738379;
pub const KEYC_WHEELDOWN11_CONTROL1: keyc = 34359741194;
pub const KEYC_WHEELDOWN10_CONTROL1: keyc = 34359740938;
pub const KEYC_WHEELDOWN9_CONTROL1: keyc = 34359740682;
pub const KEYC_WHEELDOWN8_CONTROL1: keyc = 34359740426;
pub const KEYC_WHEELDOWN7_CONTROL1: keyc = 34359740170;
pub const KEYC_WHEELDOWN6_CONTROL1: keyc = 34359739914;
pub const KEYC_WHEELDOWN3_CONTROL1: keyc = 34359739146;
pub const KEYC_WHEELDOWN2_CONTROL1: keyc = 34359738890;
pub const KEYC_WHEELDOWN1_CONTROL1: keyc = 34359738634;
pub const KEYC_WHEELDOWN_CONTROL1: keyc = 34359738378;
pub const KEYC_WHEELDOWN11_CONTROL0: keyc = 34359741193;
pub const KEYC_WHEELDOWN10_CONTROL0: keyc = 34359740937;
pub const KEYC_WHEELDOWN9_CONTROL0: keyc = 34359740681;
pub const KEYC_WHEELDOWN8_CONTROL0: keyc = 34359740425;
pub const KEYC_WHEELDOWN7_CONTROL0: keyc = 34359740169;
pub const KEYC_WHEELDOWN6_CONTROL0: keyc = 34359739913;
pub const KEYC_WHEELDOWN3_CONTROL0: keyc = 34359739145;
pub const KEYC_WHEELDOWN2_CONTROL0: keyc = 34359738889;
pub const KEYC_WHEELDOWN1_CONTROL0: keyc = 34359738633;
pub const KEYC_WHEELDOWN_CONTROL0: keyc = 34359738377;
pub const KEYC_WHEELDOWN11_SCROLLBAR_DOWN: keyc = 34359741192;
pub const KEYC_WHEELDOWN10_SCROLLBAR_DOWN: keyc = 34359740936;
pub const KEYC_WHEELDOWN9_SCROLLBAR_DOWN: keyc = 34359740680;
pub const KEYC_WHEELDOWN8_SCROLLBAR_DOWN: keyc = 34359740424;
pub const KEYC_WHEELDOWN7_SCROLLBAR_DOWN: keyc = 34359740168;
pub const KEYC_WHEELDOWN6_SCROLLBAR_DOWN: keyc = 34359739912;
pub const KEYC_WHEELDOWN3_SCROLLBAR_DOWN: keyc = 34359739144;
pub const KEYC_WHEELDOWN2_SCROLLBAR_DOWN: keyc = 34359738888;
pub const KEYC_WHEELDOWN1_SCROLLBAR_DOWN: keyc = 34359738632;
pub const KEYC_WHEELDOWN_SCROLLBAR_DOWN: keyc = 34359738376;
pub const KEYC_WHEELDOWN11_SCROLLBAR_SLIDER: keyc = 34359741191;
pub const KEYC_WHEELDOWN10_SCROLLBAR_SLIDER: keyc = 34359740935;
pub const KEYC_WHEELDOWN9_SCROLLBAR_SLIDER: keyc = 34359740679;
pub const KEYC_WHEELDOWN8_SCROLLBAR_SLIDER: keyc = 34359740423;
pub const KEYC_WHEELDOWN7_SCROLLBAR_SLIDER: keyc = 34359740167;
pub const KEYC_WHEELDOWN6_SCROLLBAR_SLIDER: keyc = 34359739911;
pub const KEYC_WHEELDOWN3_SCROLLBAR_SLIDER: keyc = 34359739143;
pub const KEYC_WHEELDOWN2_SCROLLBAR_SLIDER: keyc = 34359738887;
pub const KEYC_WHEELDOWN1_SCROLLBAR_SLIDER: keyc = 34359738631;
pub const KEYC_WHEELDOWN_SCROLLBAR_SLIDER: keyc = 34359738375;
pub const KEYC_WHEELDOWN11_SCROLLBAR_UP: keyc = 34359741190;
pub const KEYC_WHEELDOWN10_SCROLLBAR_UP: keyc = 34359740934;
pub const KEYC_WHEELDOWN9_SCROLLBAR_UP: keyc = 34359740678;
pub const KEYC_WHEELDOWN8_SCROLLBAR_UP: keyc = 34359740422;
pub const KEYC_WHEELDOWN7_SCROLLBAR_UP: keyc = 34359740166;
pub const KEYC_WHEELDOWN6_SCROLLBAR_UP: keyc = 34359739910;
pub const KEYC_WHEELDOWN3_SCROLLBAR_UP: keyc = 34359739142;
pub const KEYC_WHEELDOWN2_SCROLLBAR_UP: keyc = 34359738886;
pub const KEYC_WHEELDOWN1_SCROLLBAR_UP: keyc = 34359738630;
pub const KEYC_WHEELDOWN_SCROLLBAR_UP: keyc = 34359738374;
pub const KEYC_WHEELDOWN11_BORDER: keyc = 34359741189;
pub const KEYC_WHEELDOWN10_BORDER: keyc = 34359740933;
pub const KEYC_WHEELDOWN9_BORDER: keyc = 34359740677;
pub const KEYC_WHEELDOWN8_BORDER: keyc = 34359740421;
pub const KEYC_WHEELDOWN7_BORDER: keyc = 34359740165;
pub const KEYC_WHEELDOWN6_BORDER: keyc = 34359739909;
pub const KEYC_WHEELDOWN3_BORDER: keyc = 34359739141;
pub const KEYC_WHEELDOWN2_BORDER: keyc = 34359738885;
pub const KEYC_WHEELDOWN1_BORDER: keyc = 34359738629;
pub const KEYC_WHEELDOWN_BORDER: keyc = 34359738373;
pub const KEYC_WHEELDOWN11_STATUS_DEFAULT: keyc = 34359741188;
pub const KEYC_WHEELDOWN10_STATUS_DEFAULT: keyc = 34359740932;
pub const KEYC_WHEELDOWN9_STATUS_DEFAULT: keyc = 34359740676;
pub const KEYC_WHEELDOWN8_STATUS_DEFAULT: keyc = 34359740420;
pub const KEYC_WHEELDOWN7_STATUS_DEFAULT: keyc = 34359740164;
pub const KEYC_WHEELDOWN6_STATUS_DEFAULT: keyc = 34359739908;
pub const KEYC_WHEELDOWN3_STATUS_DEFAULT: keyc = 34359739140;
pub const KEYC_WHEELDOWN2_STATUS_DEFAULT: keyc = 34359738884;
pub const KEYC_WHEELDOWN1_STATUS_DEFAULT: keyc = 34359738628;
pub const KEYC_WHEELDOWN_STATUS_DEFAULT: keyc = 34359738372;
pub const KEYC_WHEELDOWN11_STATUS_RIGHT: keyc = 34359741187;
pub const KEYC_WHEELDOWN10_STATUS_RIGHT: keyc = 34359740931;
pub const KEYC_WHEELDOWN9_STATUS_RIGHT: keyc = 34359740675;
pub const KEYC_WHEELDOWN8_STATUS_RIGHT: keyc = 34359740419;
pub const KEYC_WHEELDOWN7_STATUS_RIGHT: keyc = 34359740163;
pub const KEYC_WHEELDOWN6_STATUS_RIGHT: keyc = 34359739907;
pub const KEYC_WHEELDOWN3_STATUS_RIGHT: keyc = 34359739139;
pub const KEYC_WHEELDOWN2_STATUS_RIGHT: keyc = 34359738883;
pub const KEYC_WHEELDOWN1_STATUS_RIGHT: keyc = 34359738627;
pub const KEYC_WHEELDOWN_STATUS_RIGHT: keyc = 34359738371;
pub const KEYC_WHEELDOWN11_STATUS_LEFT: keyc = 34359741186;
pub const KEYC_WHEELDOWN10_STATUS_LEFT: keyc = 34359740930;
pub const KEYC_WHEELDOWN9_STATUS_LEFT: keyc = 34359740674;
pub const KEYC_WHEELDOWN8_STATUS_LEFT: keyc = 34359740418;
pub const KEYC_WHEELDOWN7_STATUS_LEFT: keyc = 34359740162;
pub const KEYC_WHEELDOWN6_STATUS_LEFT: keyc = 34359739906;
pub const KEYC_WHEELDOWN3_STATUS_LEFT: keyc = 34359739138;
pub const KEYC_WHEELDOWN2_STATUS_LEFT: keyc = 34359738882;
pub const KEYC_WHEELDOWN1_STATUS_LEFT: keyc = 34359738626;
pub const KEYC_WHEELDOWN_STATUS_LEFT: keyc = 34359738370;
pub const KEYC_WHEELDOWN11_STATUS: keyc = 34359741185;
pub const KEYC_WHEELDOWN10_STATUS: keyc = 34359740929;
pub const KEYC_WHEELDOWN9_STATUS: keyc = 34359740673;
pub const KEYC_WHEELDOWN8_STATUS: keyc = 34359740417;
pub const KEYC_WHEELDOWN7_STATUS: keyc = 34359740161;
pub const KEYC_WHEELDOWN6_STATUS: keyc = 34359739905;
pub const KEYC_WHEELDOWN3_STATUS: keyc = 34359739137;
pub const KEYC_WHEELDOWN2_STATUS: keyc = 34359738881;
pub const KEYC_WHEELDOWN1_STATUS: keyc = 34359738625;
pub const KEYC_WHEELDOWN_STATUS: keyc = 34359738369;
pub const KEYC_WHEELDOWN11_PANE: keyc = 34359741184;
pub const KEYC_WHEELDOWN10_PANE: keyc = 34359740928;
pub const KEYC_WHEELDOWN9_PANE: keyc = 34359740672;
pub const KEYC_WHEELDOWN8_PANE: keyc = 34359740416;
pub const KEYC_WHEELDOWN7_PANE: keyc = 34359740160;
pub const KEYC_WHEELDOWN6_PANE: keyc = 34359739904;
pub const KEYC_WHEELDOWN3_PANE: keyc = 34359739136;
pub const KEYC_WHEELDOWN2_PANE: keyc = 34359738880;
pub const KEYC_WHEELDOWN1_PANE: keyc = 34359738624;
pub const KEYC_WHEELDOWN_PANE: keyc = 34359738368;
pub const KEYC_MOUSEMOVE11_CONTROL9: keyc = 12884904722;
pub const KEYC_MOUSEMOVE10_CONTROL9: keyc = 12884904466;
pub const KEYC_MOUSEMOVE9_CONTROL9: keyc = 12884904210;
pub const KEYC_MOUSEMOVE8_CONTROL9: keyc = 12884903954;
pub const KEYC_MOUSEMOVE7_CONTROL9: keyc = 12884903698;
pub const KEYC_MOUSEMOVE6_CONTROL9: keyc = 12884903442;
pub const KEYC_MOUSEMOVE3_CONTROL9: keyc = 12884902674;
pub const KEYC_MOUSEMOVE2_CONTROL9: keyc = 12884902418;
pub const KEYC_MOUSEMOVE1_CONTROL9: keyc = 12884902162;
pub const KEYC_MOUSEMOVE_CONTROL9: keyc = 12884901906;
pub const KEYC_MOUSEMOVE11_CONTROL8: keyc = 12884904721;
pub const KEYC_MOUSEMOVE10_CONTROL8: keyc = 12884904465;
pub const KEYC_MOUSEMOVE9_CONTROL8: keyc = 12884904209;
pub const KEYC_MOUSEMOVE8_CONTROL8: keyc = 12884903953;
pub const KEYC_MOUSEMOVE7_CONTROL8: keyc = 12884903697;
pub const KEYC_MOUSEMOVE6_CONTROL8: keyc = 12884903441;
pub const KEYC_MOUSEMOVE3_CONTROL8: keyc = 12884902673;
pub const KEYC_MOUSEMOVE2_CONTROL8: keyc = 12884902417;
pub const KEYC_MOUSEMOVE1_CONTROL8: keyc = 12884902161;
pub const KEYC_MOUSEMOVE_CONTROL8: keyc = 12884901905;
pub const KEYC_MOUSEMOVE11_CONTROL7: keyc = 12884904720;
pub const KEYC_MOUSEMOVE10_CONTROL7: keyc = 12884904464;
pub const KEYC_MOUSEMOVE9_CONTROL7: keyc = 12884904208;
pub const KEYC_MOUSEMOVE8_CONTROL7: keyc = 12884903952;
pub const KEYC_MOUSEMOVE7_CONTROL7: keyc = 12884903696;
pub const KEYC_MOUSEMOVE6_CONTROL7: keyc = 12884903440;
pub const KEYC_MOUSEMOVE3_CONTROL7: keyc = 12884902672;
pub const KEYC_MOUSEMOVE2_CONTROL7: keyc = 12884902416;
pub const KEYC_MOUSEMOVE1_CONTROL7: keyc = 12884902160;
pub const KEYC_MOUSEMOVE_CONTROL7: keyc = 12884901904;
pub const KEYC_MOUSEMOVE11_CONTROL6: keyc = 12884904719;
pub const KEYC_MOUSEMOVE10_CONTROL6: keyc = 12884904463;
pub const KEYC_MOUSEMOVE9_CONTROL6: keyc = 12884904207;
pub const KEYC_MOUSEMOVE8_CONTROL6: keyc = 12884903951;
pub const KEYC_MOUSEMOVE7_CONTROL6: keyc = 12884903695;
pub const KEYC_MOUSEMOVE6_CONTROL6: keyc = 12884903439;
pub const KEYC_MOUSEMOVE3_CONTROL6: keyc = 12884902671;
pub const KEYC_MOUSEMOVE2_CONTROL6: keyc = 12884902415;
pub const KEYC_MOUSEMOVE1_CONTROL6: keyc = 12884902159;
pub const KEYC_MOUSEMOVE_CONTROL6: keyc = 12884901903;
pub const KEYC_MOUSEMOVE11_CONTROL5: keyc = 12884904718;
pub const KEYC_MOUSEMOVE10_CONTROL5: keyc = 12884904462;
pub const KEYC_MOUSEMOVE9_CONTROL5: keyc = 12884904206;
pub const KEYC_MOUSEMOVE8_CONTROL5: keyc = 12884903950;
pub const KEYC_MOUSEMOVE7_CONTROL5: keyc = 12884903694;
pub const KEYC_MOUSEMOVE6_CONTROL5: keyc = 12884903438;
pub const KEYC_MOUSEMOVE3_CONTROL5: keyc = 12884902670;
pub const KEYC_MOUSEMOVE2_CONTROL5: keyc = 12884902414;
pub const KEYC_MOUSEMOVE1_CONTROL5: keyc = 12884902158;
pub const KEYC_MOUSEMOVE_CONTROL5: keyc = 12884901902;
pub const KEYC_MOUSEMOVE11_CONTROL4: keyc = 12884904717;
pub const KEYC_MOUSEMOVE10_CONTROL4: keyc = 12884904461;
pub const KEYC_MOUSEMOVE9_CONTROL4: keyc = 12884904205;
pub const KEYC_MOUSEMOVE8_CONTROL4: keyc = 12884903949;
pub const KEYC_MOUSEMOVE7_CONTROL4: keyc = 12884903693;
pub const KEYC_MOUSEMOVE6_CONTROL4: keyc = 12884903437;
pub const KEYC_MOUSEMOVE3_CONTROL4: keyc = 12884902669;
pub const KEYC_MOUSEMOVE2_CONTROL4: keyc = 12884902413;
pub const KEYC_MOUSEMOVE1_CONTROL4: keyc = 12884902157;
pub const KEYC_MOUSEMOVE_CONTROL4: keyc = 12884901901;
pub const KEYC_MOUSEMOVE11_CONTROL3: keyc = 12884904716;
pub const KEYC_MOUSEMOVE10_CONTROL3: keyc = 12884904460;
pub const KEYC_MOUSEMOVE9_CONTROL3: keyc = 12884904204;
pub const KEYC_MOUSEMOVE8_CONTROL3: keyc = 12884903948;
pub const KEYC_MOUSEMOVE7_CONTROL3: keyc = 12884903692;
pub const KEYC_MOUSEMOVE6_CONTROL3: keyc = 12884903436;
pub const KEYC_MOUSEMOVE3_CONTROL3: keyc = 12884902668;
pub const KEYC_MOUSEMOVE2_CONTROL3: keyc = 12884902412;
pub const KEYC_MOUSEMOVE1_CONTROL3: keyc = 12884902156;
pub const KEYC_MOUSEMOVE_CONTROL3: keyc = 12884901900;
pub const KEYC_MOUSEMOVE11_CONTROL2: keyc = 12884904715;
pub const KEYC_MOUSEMOVE10_CONTROL2: keyc = 12884904459;
pub const KEYC_MOUSEMOVE9_CONTROL2: keyc = 12884904203;
pub const KEYC_MOUSEMOVE8_CONTROL2: keyc = 12884903947;
pub const KEYC_MOUSEMOVE7_CONTROL2: keyc = 12884903691;
pub const KEYC_MOUSEMOVE6_CONTROL2: keyc = 12884903435;
pub const KEYC_MOUSEMOVE3_CONTROL2: keyc = 12884902667;
pub const KEYC_MOUSEMOVE2_CONTROL2: keyc = 12884902411;
pub const KEYC_MOUSEMOVE1_CONTROL2: keyc = 12884902155;
pub const KEYC_MOUSEMOVE_CONTROL2: keyc = 12884901899;
pub const KEYC_MOUSEMOVE11_CONTROL1: keyc = 12884904714;
pub const KEYC_MOUSEMOVE10_CONTROL1: keyc = 12884904458;
pub const KEYC_MOUSEMOVE9_CONTROL1: keyc = 12884904202;
pub const KEYC_MOUSEMOVE8_CONTROL1: keyc = 12884903946;
pub const KEYC_MOUSEMOVE7_CONTROL1: keyc = 12884903690;
pub const KEYC_MOUSEMOVE6_CONTROL1: keyc = 12884903434;
pub const KEYC_MOUSEMOVE3_CONTROL1: keyc = 12884902666;
pub const KEYC_MOUSEMOVE2_CONTROL1: keyc = 12884902410;
pub const KEYC_MOUSEMOVE1_CONTROL1: keyc = 12884902154;
pub const KEYC_MOUSEMOVE_CONTROL1: keyc = 12884901898;
pub const KEYC_MOUSEMOVE11_CONTROL0: keyc = 12884904713;
pub const KEYC_MOUSEMOVE10_CONTROL0: keyc = 12884904457;
pub const KEYC_MOUSEMOVE9_CONTROL0: keyc = 12884904201;
pub const KEYC_MOUSEMOVE8_CONTROL0: keyc = 12884903945;
pub const KEYC_MOUSEMOVE7_CONTROL0: keyc = 12884903689;
pub const KEYC_MOUSEMOVE6_CONTROL0: keyc = 12884903433;
pub const KEYC_MOUSEMOVE3_CONTROL0: keyc = 12884902665;
pub const KEYC_MOUSEMOVE2_CONTROL0: keyc = 12884902409;
pub const KEYC_MOUSEMOVE1_CONTROL0: keyc = 12884902153;
pub const KEYC_MOUSEMOVE_CONTROL0: keyc = 12884901897;
pub const KEYC_MOUSEMOVE11_SCROLLBAR_DOWN: keyc = 12884904712;
pub const KEYC_MOUSEMOVE10_SCROLLBAR_DOWN: keyc = 12884904456;
pub const KEYC_MOUSEMOVE9_SCROLLBAR_DOWN: keyc = 12884904200;
pub const KEYC_MOUSEMOVE8_SCROLLBAR_DOWN: keyc = 12884903944;
pub const KEYC_MOUSEMOVE7_SCROLLBAR_DOWN: keyc = 12884903688;
pub const KEYC_MOUSEMOVE6_SCROLLBAR_DOWN: keyc = 12884903432;
pub const KEYC_MOUSEMOVE3_SCROLLBAR_DOWN: keyc = 12884902664;
pub const KEYC_MOUSEMOVE2_SCROLLBAR_DOWN: keyc = 12884902408;
pub const KEYC_MOUSEMOVE1_SCROLLBAR_DOWN: keyc = 12884902152;
pub const KEYC_MOUSEMOVE_SCROLLBAR_DOWN: keyc = 12884901896;
pub const KEYC_MOUSEMOVE11_SCROLLBAR_SLIDER: keyc = 12884904711;
pub const KEYC_MOUSEMOVE10_SCROLLBAR_SLIDER: keyc = 12884904455;
pub const KEYC_MOUSEMOVE9_SCROLLBAR_SLIDER: keyc = 12884904199;
pub const KEYC_MOUSEMOVE8_SCROLLBAR_SLIDER: keyc = 12884903943;
pub const KEYC_MOUSEMOVE7_SCROLLBAR_SLIDER: keyc = 12884903687;
pub const KEYC_MOUSEMOVE6_SCROLLBAR_SLIDER: keyc = 12884903431;
pub const KEYC_MOUSEMOVE3_SCROLLBAR_SLIDER: keyc = 12884902663;
pub const KEYC_MOUSEMOVE2_SCROLLBAR_SLIDER: keyc = 12884902407;
pub const KEYC_MOUSEMOVE1_SCROLLBAR_SLIDER: keyc = 12884902151;
pub const KEYC_MOUSEMOVE_SCROLLBAR_SLIDER: keyc = 12884901895;
pub const KEYC_MOUSEMOVE11_SCROLLBAR_UP: keyc = 12884904710;
pub const KEYC_MOUSEMOVE10_SCROLLBAR_UP: keyc = 12884904454;
pub const KEYC_MOUSEMOVE9_SCROLLBAR_UP: keyc = 12884904198;
pub const KEYC_MOUSEMOVE8_SCROLLBAR_UP: keyc = 12884903942;
pub const KEYC_MOUSEMOVE7_SCROLLBAR_UP: keyc = 12884903686;
pub const KEYC_MOUSEMOVE6_SCROLLBAR_UP: keyc = 12884903430;
pub const KEYC_MOUSEMOVE3_SCROLLBAR_UP: keyc = 12884902662;
pub const KEYC_MOUSEMOVE2_SCROLLBAR_UP: keyc = 12884902406;
pub const KEYC_MOUSEMOVE1_SCROLLBAR_UP: keyc = 12884902150;
pub const KEYC_MOUSEMOVE_SCROLLBAR_UP: keyc = 12884901894;
pub const KEYC_MOUSEMOVE11_BORDER: keyc = 12884904709;
pub const KEYC_MOUSEMOVE10_BORDER: keyc = 12884904453;
pub const KEYC_MOUSEMOVE9_BORDER: keyc = 12884904197;
pub const KEYC_MOUSEMOVE8_BORDER: keyc = 12884903941;
pub const KEYC_MOUSEMOVE7_BORDER: keyc = 12884903685;
pub const KEYC_MOUSEMOVE6_BORDER: keyc = 12884903429;
pub const KEYC_MOUSEMOVE3_BORDER: keyc = 12884902661;
pub const KEYC_MOUSEMOVE2_BORDER: keyc = 12884902405;
pub const KEYC_MOUSEMOVE1_BORDER: keyc = 12884902149;
pub const KEYC_MOUSEMOVE_BORDER: keyc = 12884901893;
pub const KEYC_MOUSEMOVE11_STATUS_DEFAULT: keyc = 12884904708;
pub const KEYC_MOUSEMOVE10_STATUS_DEFAULT: keyc = 12884904452;
pub const KEYC_MOUSEMOVE9_STATUS_DEFAULT: keyc = 12884904196;
pub const KEYC_MOUSEMOVE8_STATUS_DEFAULT: keyc = 12884903940;
pub const KEYC_MOUSEMOVE7_STATUS_DEFAULT: keyc = 12884903684;
pub const KEYC_MOUSEMOVE6_STATUS_DEFAULT: keyc = 12884903428;
pub const KEYC_MOUSEMOVE3_STATUS_DEFAULT: keyc = 12884902660;
pub const KEYC_MOUSEMOVE2_STATUS_DEFAULT: keyc = 12884902404;
pub const KEYC_MOUSEMOVE1_STATUS_DEFAULT: keyc = 12884902148;
pub const KEYC_MOUSEMOVE_STATUS_DEFAULT: keyc = 12884901892;
pub const KEYC_MOUSEMOVE11_STATUS_RIGHT: keyc = 12884904707;
pub const KEYC_MOUSEMOVE10_STATUS_RIGHT: keyc = 12884904451;
pub const KEYC_MOUSEMOVE9_STATUS_RIGHT: keyc = 12884904195;
pub const KEYC_MOUSEMOVE8_STATUS_RIGHT: keyc = 12884903939;
pub const KEYC_MOUSEMOVE7_STATUS_RIGHT: keyc = 12884903683;
pub const KEYC_MOUSEMOVE6_STATUS_RIGHT: keyc = 12884903427;
pub const KEYC_MOUSEMOVE3_STATUS_RIGHT: keyc = 12884902659;
pub const KEYC_MOUSEMOVE2_STATUS_RIGHT: keyc = 12884902403;
pub const KEYC_MOUSEMOVE1_STATUS_RIGHT: keyc = 12884902147;
pub const KEYC_MOUSEMOVE_STATUS_RIGHT: keyc = 12884901891;
pub const KEYC_MOUSEMOVE11_STATUS_LEFT: keyc = 12884904706;
pub const KEYC_MOUSEMOVE10_STATUS_LEFT: keyc = 12884904450;
pub const KEYC_MOUSEMOVE9_STATUS_LEFT: keyc = 12884904194;
pub const KEYC_MOUSEMOVE8_STATUS_LEFT: keyc = 12884903938;
pub const KEYC_MOUSEMOVE7_STATUS_LEFT: keyc = 12884903682;
pub const KEYC_MOUSEMOVE6_STATUS_LEFT: keyc = 12884903426;
pub const KEYC_MOUSEMOVE3_STATUS_LEFT: keyc = 12884902658;
pub const KEYC_MOUSEMOVE2_STATUS_LEFT: keyc = 12884902402;
pub const KEYC_MOUSEMOVE1_STATUS_LEFT: keyc = 12884902146;
pub const KEYC_MOUSEMOVE_STATUS_LEFT: keyc = 12884901890;
pub const KEYC_MOUSEMOVE11_STATUS: keyc = 12884904705;
pub const KEYC_MOUSEMOVE10_STATUS: keyc = 12884904449;
pub const KEYC_MOUSEMOVE9_STATUS: keyc = 12884904193;
pub const KEYC_MOUSEMOVE8_STATUS: keyc = 12884903937;
pub const KEYC_MOUSEMOVE7_STATUS: keyc = 12884903681;
pub const KEYC_MOUSEMOVE6_STATUS: keyc = 12884903425;
pub const KEYC_MOUSEMOVE3_STATUS: keyc = 12884902657;
pub const KEYC_MOUSEMOVE2_STATUS: keyc = 12884902401;
pub const KEYC_MOUSEMOVE1_STATUS: keyc = 12884902145;
pub const KEYC_MOUSEMOVE_STATUS: keyc = 12884901889;
pub const KEYC_MOUSEMOVE11_PANE: keyc = 12884904704;
pub const KEYC_MOUSEMOVE10_PANE: keyc = 12884904448;
pub const KEYC_MOUSEMOVE9_PANE: keyc = 12884904192;
pub const KEYC_MOUSEMOVE8_PANE: keyc = 12884903936;
pub const KEYC_MOUSEMOVE7_PANE: keyc = 12884903680;
pub const KEYC_MOUSEMOVE6_PANE: keyc = 12884903424;
pub const KEYC_MOUSEMOVE3_PANE: keyc = 12884902656;
pub const KEYC_MOUSEMOVE2_PANE: keyc = 12884902400;
pub const KEYC_MOUSEMOVE1_PANE: keyc = 12884902144;
pub const KEYC_MOUSEMOVE_PANE: keyc = 12884901888;
pub const KEYC_DOUBLECLICK: keyc = 8589934643;
pub const KEYC_DRAGGING: keyc = 8589934642;
pub const KEYC_MOUSE: keyc = 8589934641;
pub const KEYC_REPORT_LIGHT_THEME: keyc = 8589934640;
pub const KEYC_REPORT_DARK_THEME: keyc = 8589934639;
pub const KEYC_KP_PERIOD: keyc = 8589934638;
pub const KEYC_KP_ZERO: keyc = 8589934637;
pub const KEYC_KP_ENTER: keyc = 8589934636;
pub const KEYC_KP_THREE: keyc = 8589934635;
pub const KEYC_KP_TWO: keyc = 8589934634;
pub const KEYC_KP_ONE: keyc = 8589934633;
pub const KEYC_KP_SIX: keyc = 8589934632;
pub const KEYC_KP_FIVE: keyc = 8589934631;
pub const KEYC_KP_FOUR: keyc = 8589934630;
pub const KEYC_KP_PLUS: keyc = 8589934629;
pub const KEYC_KP_NINE: keyc = 8589934628;
pub const KEYC_KP_EIGHT: keyc = 8589934627;
pub const KEYC_KP_SEVEN: keyc = 8589934626;
pub const KEYC_KP_MINUS: keyc = 8589934625;
pub const KEYC_KP_STAR: keyc = 8589934624;
pub const KEYC_KP_SLASH: keyc = 8589934623;
pub const KEYC_RIGHT: keyc = 8589934622;
pub const KEYC_LEFT: keyc = 8589934621;
pub const KEYC_DOWN: keyc = 8589934620;
pub const KEYC_UP: keyc = 8589934619;
pub const KEYC_BTAB: keyc = 8589934618;
pub const KEYC_PPAGE: keyc = 8589934617;
pub const KEYC_NPAGE: keyc = 8589934616;
pub const KEYC_END: keyc = 8589934615;
pub const KEYC_HOME: keyc = 8589934614;
pub const KEYC_DC: keyc = 8589934613;
pub const KEYC_IC: keyc = 8589934612;
pub const KEYC_F12: keyc = 8589934611;
pub const KEYC_F11: keyc = 8589934610;
pub const KEYC_F10: keyc = 8589934609;
pub const KEYC_F9: keyc = 8589934608;
pub const KEYC_F8: keyc = 8589934607;
pub const KEYC_F7: keyc = 8589934606;
pub const KEYC_F6: keyc = 8589934605;
pub const KEYC_F5: keyc = 8589934604;
pub const KEYC_F4: keyc = 8589934603;
pub const KEYC_F3: keyc = 8589934602;
pub const KEYC_F2: keyc = 8589934601;
pub const KEYC_F1: keyc = 8589934600;
pub const KEYC_BSPACE: keyc = 8589934599;
pub const KEYC_PASTE_END: keyc = 8589934598;
pub const KEYC_PASTE_START: keyc = 8589934597;
pub const KEYC_ANY: keyc = 8589934596;
pub const KEYC_FOCUS_OUT: keyc = 8589934595;
pub const KEYC_FOCUS_IN: keyc = 8589934594;
pub const KEYC_UNKNOWN: keyc = 8589934593;
pub const KEYC_NONE: keyc = 8589934592;
pub const KEYC_USER: keyc = 4294967296;
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
pub const CMD_RETURN_STOP: cmd_retval = 2;
pub const CMD_RETURN_WAIT: cmd_retval = 1;
pub const CMD_RETURN_NORMAL: cmd_retval = 0;
pub const CMD_RETURN_ERROR: cmd_retval = -1;
pub const CMD_PARSE_SUCCESS: cmd_parse_status = 1;
pub const CMD_PARSE_ERROR: cmd_parse_status = 0;
pub const EINTR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const STDIN_FILENO: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const STDOUT_FILENO: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const STDERR_FILENO: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const X_OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const _PATH_BSHELL: &CStr = c"/bin/sh";
pub const _PATH_TTY: &CStr = c"/dev/tty";
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const RB_BLACK: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RB_RED: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RB_NEGINF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const EV_TIMEOUT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const EV_READ: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const VIS_OCTAL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const VIS_CSTYLE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const VIS_NOSLASH: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const IMSG_HEADER_SIZE: usize = ::core::mem::size_of::<imsg_hdr>();
pub const KEYC_META: ::core::ffi::c_ulonglong = 0x100000000000 as ::core::ffi::c_ulonglong;
pub const KEYC_CTRL: ::core::ffi::c_ulonglong = 0x200000000000 as ::core::ffi::c_ulonglong;
pub const KEYC_SHIFT: ::core::ffi::c_ulonglong = 0x400000000000 as ::core::ffi::c_ulonglong;
pub const KEYC_SENT: ::core::ffi::c_ulonglong = 0x40000000000000 as ::core::ffi::c_ulonglong;
pub const KEYC_MASK_TYPE: ::core::ffi::c_ulonglong = 0xff00000000 as ::core::ffi::c_ulonglong;
pub const KEYC_MASK_MODIFIERS: ::core::ffi::c_ulonglong =
    0xff0000000000 as ::core::ffi::c_ulonglong;
pub const KEYC_MASK_KEY: ::core::ffi::c_ulonglong = 0xffffffffff as ::core::ffi::c_ulonglong;
pub const KEYC_CLICK_TIMEOUT: ::core::ffi::c_int = 300 as ::core::ffi::c_int;
pub const KEYC_MOUSE_LOCATION_SHIFT: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const KEYC_MOUSE_BUTTON_SHIFT: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const MODE_CURSOR: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const MODE_MOUSE_STANDARD: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const MODE_MOUSE_BUTTON: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const MODE_BRACKETPASTE: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const MODE_MOUSE_ALL: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const ALL_MOUSE_MODES: ::core::ffi::c_int =
    MODE_MOUSE_STANDARD | MODE_MOUSE_BUTTON | MODE_MOUSE_ALL;
pub const PANE_REDRAW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const PANE_ZOOMED: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const PANE_EXITED: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const PANE_STYLECHANGED: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const PANE_REDRAWSCROLLBAR: ::core::ffi::c_int = 0x8000 as ::core::ffi::c_int;
pub const WINDOW_ZOOMED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const WINDOW_RESIZE: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const WINLINK_BELL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const WINLINK_ACTIVITY: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const WINLINK_SILENCE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const WINLINK_ALERTFLAGS: ::core::ffi::c_int =
    WINLINK_BELL | WINLINK_ACTIVITY | WINLINK_SILENCE;
pub const WINDOW_SIZE_LATEST: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const PANE_STATUS_OFF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PANE_STATUS_TOP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PANE_STATUS_BOTTOM: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_RIGHT: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_LEFT: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const MOUSE_MASK_BUTTONS: ::core::ffi::c_int = 195 as ::core::ffi::c_int;
pub const MOUSE_MASK_SHIFT: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const MOUSE_MASK_META: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const MOUSE_MASK_CTRL: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const MOUSE_MASK_DRAG: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const MOUSE_WHEEL_UP: ::core::ffi::c_int = 64 as ::core::ffi::c_int;
pub const MOUSE_WHEEL_DOWN: ::core::ffi::c_int = 65 as ::core::ffi::c_int;
pub const MOUSE_BUTTON_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const MOUSE_BUTTON_2: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const MOUSE_BUTTON_3: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const MOUSE_BUTTON_6: ::core::ffi::c_int = 66 as ::core::ffi::c_int;
pub const MOUSE_BUTTON_7: ::core::ffi::c_int = 67 as ::core::ffi::c_int;
pub const MOUSE_BUTTON_8: ::core::ffi::c_int = 128 as ::core::ffi::c_int;
pub const MOUSE_BUTTON_9: ::core::ffi::c_int = 129 as ::core::ffi::c_int;
pub const MOUSE_BUTTON_10: ::core::ffi::c_int = 130 as ::core::ffi::c_int;
pub const MOUSE_BUTTON_11: ::core::ffi::c_int = 131 as ::core::ffi::c_int;
pub const TTY_NOCURSOR: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const TTY_FREEZE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const TTY_BLOCK: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const CMD_READONLY: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CLIENT_PASTE_TIME_LIMIT: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const CLIENT_TERMINAL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CLIENT_EXIT: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CLIENT_REDRAWWINDOW: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const CLIENT_REDRAWSTATUS: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CLIENT_REPEAT: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const CLIENT_SUSPENDED: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const CLIENT_ATTACHED: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const CLIENT_EXITED: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const CLIENT_DEAD: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const CLIENT_REDRAWBORDERS: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const CLIENT_READONLY: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const CLIENT_CONTROL: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const CLIENT_FOCUSED: ::core::ffi::c_int = 0x8000 as ::core::ffi::c_int;
pub const CLIENT_UTF8: ::core::ffi::c_int = 0x10000 as ::core::ffi::c_int;
pub const CLIENT_IGNORESIZE: ::core::ffi::c_int = 0x20000 as ::core::ffi::c_int;
pub const CLIENT_IDENTIFIED: ::core::ffi::c_int = 0x40000 as ::core::ffi::c_int;
pub const CLIENT_STATUSFORCE: ::core::ffi::c_int = 0x80000 as ::core::ffi::c_int;
pub const CLIENT_DOUBLECLICK: ::core::ffi::c_int = 0x100000 as ::core::ffi::c_int;
pub const CLIENT_TRIPLECLICK: ::core::ffi::c_int = 0x200000 as ::core::ffi::c_int;
pub const CLIENT_REDRAWSTATUSALWAYS: ::core::ffi::c_int = 0x1000000 as ::core::ffi::c_int;
pub const CLIENT_REDRAWOVERLAY: ::core::ffi::c_int = 0x2000000 as ::core::ffi::c_int;
pub const CLIENT_CONTROL_NOOUTPUT: ::core::ffi::c_int = 0x4000000 as ::core::ffi::c_int;
pub const CLIENT_REDRAWPANES: ::core::ffi::c_int = 0x20000000 as ::core::ffi::c_int;
pub const CLIENT_ACTIVEPANE: ::core::ffi::c_ulonglong = 0x80000000 as ::core::ffi::c_ulonglong;
pub const CLIENT_CONTROL_PAUSEAFTER: ::core::ffi::c_ulonglong =
    0x100000000 as ::core::ffi::c_ulonglong;
pub const CLIENT_CONTROL_WAITEXIT: ::core::ffi::c_ulonglong =
    0x200000000 as ::core::ffi::c_ulonglong;
pub const CLIENT_BRACKETPASTING: ::core::ffi::c_ulonglong =
    0x1000000000 as ::core::ffi::c_ulonglong;
pub const CLIENT_ASSUMEPASTING: ::core::ffi::c_ulonglong = 0x2000000000 as ::core::ffi::c_ulonglong;
pub const CLIENT_REDRAWSCROLLBARS: ::core::ffi::c_ulonglong =
    0x4000000000 as ::core::ffi::c_ulonglong;
pub const CLIENT_NO_DETACH_ON_DESTROY: ::core::ffi::c_ulonglong =
    0x8000000000 as ::core::ffi::c_ulonglong;
pub const CLIENT_ALLREDRAWFLAGS: ::core::ffi::c_ulonglong = (CLIENT_REDRAWWINDOW
    | CLIENT_REDRAWSTATUS
    | CLIENT_REDRAWSTATUSALWAYS
    | CLIENT_REDRAWBORDERS
    | CLIENT_REDRAWOVERLAY
    | CLIENT_REDRAWPANES)
    as ::core::ffi::c_ulonglong
    | CLIENT_REDRAWSCROLLBARS;
pub const CLIENT_UNATTACHEDFLAGS: ::core::ffi::c_int = CLIENT_DEAD | CLIENT_SUSPENDED | CLIENT_EXIT;
pub const CLIENT_NODETACHFLAGS: ::core::ffi::c_int = CLIENT_DEAD | CLIENT_EXIT;
pub const KEY_BINDING_REPEAT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const FORMAT_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub fn server_client_how_many() -> u_int {
    unsafe {
        let mut n: u_int = 0;
        n = 0 as u_int;
        for c in client_walk() {
            if !(*c).session.is_null() && !(*c).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0 {
                n = n.wrapping_add(1);
            }
        }
        n
    }
}
unsafe fn server_client_overlay_timer(c: *mut client) {
    unsafe {
        server_client_clear_overlay(c);
    }
}
impl OverlayData {
    pub fn is_none(self) -> bool {
        matches!(self, OverlayData::None)
    }
    pub fn menu(self) -> *mut menu_data {
        match self {
            OverlayData::Menu(data) => data,
            _ => panic!("overlay data is not a menu"),
        }
    }
    pub fn popup(self) -> *mut popup_data {
        match self {
            OverlayData::Popup(data) => data,
            _ => panic!("overlay data is not a popup"),
        }
    }
    pub fn display_panes(self) -> *mut cmd_display_panes_data {
        match self {
            OverlayData::DisplayPanes(data) => data,
            _ => panic!("overlay data is not display-panes"),
        }
    }
}
impl Overlay {
    /// The check the overlay installs with itself. `display-panes` covers
    /// nothing: it draws over the panes and lets everything through.
    pub fn check(self) -> OverlayCheck {
        match self {
            Overlay::None => OverlayCheck::None,
            Overlay::Menu => OverlayCheck::Menu,
            Overlay::Popup => OverlayCheck::Popup,
            Overlay::DisplayPanes { .. } => OverlayCheck::None,
        }
    }
    pub fn has_mode(self) -> bool {
        matches!(self, Overlay::Menu | Overlay::Popup)
    }
    pub fn has_key(self) -> bool {
        match self {
            Overlay::None => false,
            Overlay::Menu | Overlay::Popup => true,
            Overlay::DisplayPanes { keys } => keys,
        }
    }
    pub fn has_resize(self) -> bool {
        matches!(self, Overlay::Menu | Overlay::Popup)
    }
    pub unsafe fn mode(self, c: *mut client, data: OverlayData) -> (*mut screen, u_int, u_int) {
        unsafe {
            match self {
                Overlay::Menu => menu_mode_cb(c, data.menu()),
                Overlay::Popup => popup_mode_cb(c, data.popup()),
                _ => panic!("overlay has no mode"),
            }
        }
    }
    pub unsafe fn draw(self, c: *mut client, data: OverlayData, ctx: &mut screen_redraw_ctx) {
        unsafe {
            match self {
                Overlay::Menu => menu_draw_cb(c, data.menu(), ctx),
                Overlay::Popup => popup_draw_cb(c, data.popup(), ctx),
                Overlay::DisplayPanes { .. } => {
                    cmd_display_panes_draw(c, data.display_panes(), ctx)
                }
                Overlay::None => panic!("overlay is not set"),
            }
        }
    }
    pub unsafe fn key(
        self,
        c: *mut client,
        data: OverlayData,
        event: *mut key_event,
    ) -> ::core::ffi::c_int {
        unsafe {
            match self {
                Overlay::Menu => menu_key_cb(c, data.menu(), event),
                Overlay::Popup => popup_key_cb(c, data.popup(), event),
                Overlay::DisplayPanes { keys: true } => {
                    cmd_display_panes_key(c, data.display_panes(), event)
                }
                _ => panic!("overlay has no key"),
            }
        }
    }
    pub unsafe fn free(self, c: *mut client, data: OverlayState) {
        unsafe {
            match (self, data) {
                (Overlay::Menu, OverlayState::Menu(data)) => menu_free_box(c, data),
                (Overlay::Popup, OverlayState::Popup(data)) => popup_free_box(c, data),
                (Overlay::DisplayPanes { .. }, OverlayState::DisplayPanes(data)) => {
                    cmd_display_panes_free_box(c, data)
                }
                (Overlay::None, _) => panic!("overlay is not set"),
                _ => panic!("overlay data does not match overlay"),
            }
        }
    }
    pub unsafe fn resize(self, c: *mut client, data: OverlayData) {
        unsafe {
            match self {
                Overlay::Menu => menu_resize_cb(c, data.menu()),
                Overlay::Popup => popup_resize_cb(c, data.popup()),
                _ => panic!("overlay has no resize"),
            }
        }
    }
}

impl OverlayCheck {
    pub unsafe fn call(
        self,
        c: *mut client,
        data: OverlayData,
        px: u_int,
        py: u_int,
        nx: u_int,
    ) -> *mut visible_ranges {
        unsafe {
            match self {
                OverlayCheck::Menu => menu_check_cb(c, data.menu(), px, py, nx),
                OverlayCheck::Popup => popup_check_cb(c, data.popup(), px, py, nx),
                OverlayCheck::None => panic!("overlay check is not set"),
            }
        }
    }
}

pub unsafe fn server_client_set_overlay(
    mut c: *mut client,
    mut delay: u_int,
    mut overlay: Overlay,
    data: OverlayState,
) {
    unsafe {
        let mut tv = timeval::default();
        if (*c).overlay().is_some() {
            server_client_clear_overlay(c);
        }
        tv.tv_sec = delay.wrapping_div(1000 as u_int) as __time_t;
        tv.tv_usec = (delay.wrapping_rem(1000 as u_int) as ::core::ffi::c_long
            * 1000 as ::core::ffi::c_long) as __suseconds_t;
        (*c).overlay_timer
            .set_callback(move || server_client_overlay_timer(c));
        if delay != 0 as u_int {
            (*c).overlay_timer.arm(tv);
        }
        (*c).set_overlay(overlay, data);
        if (*c).overlay_check().is_none() {
            (*c).tty.flags |= TTY_FREEZE;
        }
        if !overlay.has_mode() {
            (*c).tty.flags |= TTY_NOCURSOR;
        }
        window_update_focus((*session_get_curw((*c).session)).window());
        server_redraw_client(c);
    }
}
pub unsafe fn server_client_clear_overlay(mut c: *mut client) {
    unsafe {
        if (*c).overlay().is_none() {
            return;
        }
        (*c).overlay_timer.disarm();
        let (overlay, data) = (*c).take_overlay();
        if !data.is_none() {
            overlay.free(c, data);
        }
        if (*c).overlay().is_none() {
            (*c).tty.flags &= !(TTY_FREEZE | TTY_NOCURSOR);
            if !(*c).session.is_null() {
                window_update_focus((*session_get_curw((*c).session)).window());
            }
        }
        server_redraw_client(c);
    }
}
pub unsafe fn server_client_ranges_is_empty(mut r: *mut visible_ranges) -> ::core::ffi::c_int {
    unsafe {
        let mut i: u_int = 0;
        i = 0 as u_int;
        while i < (*r).used {
            if (*(*r).ranges.as_ptr().add(i as usize)).nx != 0 as u_int {
                return 0 as ::core::ffi::c_int;
            }
            i = i.wrapping_add(1);
        }
        1 as ::core::ffi::c_int
    }
}
pub unsafe fn server_client_ensure_ranges(mut r: *mut visible_ranges, mut n: u_int) {
    unsafe {
        if (*r).ranges.len() >= n as usize {
            return;
        }
        (*r).ranges
            .resize(n as usize, visible_range { px: 0, nx: 0 });
    }
}
pub unsafe fn server_client_overlay_range(
    mut x: u_int,
    mut y: u_int,
    mut sx: u_int,
    mut sy: u_int,
    mut px: u_int,
    mut py: u_int,
    mut nx: u_int,
    mut r: *mut visible_ranges,
) {
    unsafe {
        let mut ox: u_int = 0;
        let mut onx: u_int = 0;
        if py < y || py > y.wrapping_add(sy).wrapping_sub(1 as u_int) {
            server_client_ensure_ranges(r, 1 as u_int);
            (*(*r)
                .ranges
                .as_mut_ptr()
                .offset(0 as ::core::ffi::c_int as isize))
            .px = px;
            (*(*r)
                .ranges
                .as_mut_ptr()
                .offset(0 as ::core::ffi::c_int as isize))
            .nx = nx;
            (*r).used = 1 as u_int;
            return;
        }
        server_client_ensure_ranges(r, 2 as u_int);
        if px < x {
            (*(*r)
                .ranges
                .as_mut_ptr()
                .offset(0 as ::core::ffi::c_int as isize))
            .px = px;
            (*(*r)
                .ranges
                .as_mut_ptr()
                .offset(0 as ::core::ffi::c_int as isize))
            .nx = x.wrapping_sub(px);
            if (*(*r)
                .ranges
                .as_mut_ptr()
                .offset(0 as ::core::ffi::c_int as isize))
            .nx > nx
            {
                (*(*r)
                    .ranges
                    .as_mut_ptr()
                    .offset(0 as ::core::ffi::c_int as isize))
                .nx = nx;
            }
        } else {
            (*(*r)
                .ranges
                .as_mut_ptr()
                .offset(0 as ::core::ffi::c_int as isize))
            .px = 0 as u_int;
            (*(*r)
                .ranges
                .as_mut_ptr()
                .offset(0 as ::core::ffi::c_int as isize))
            .nx = 0 as u_int;
        }
        ox = x.wrapping_add(sx);
        if px > ox {
            ox = px;
        }
        onx = px.wrapping_add(nx);
        if onx > ox {
            (*(*r)
                .ranges
                .as_mut_ptr()
                .offset(1 as ::core::ffi::c_int as isize))
            .px = ox;
            (*(*r)
                .ranges
                .as_mut_ptr()
                .offset(1 as ::core::ffi::c_int as isize))
            .nx = onx.wrapping_sub(ox);
        } else {
            (*(*r)
                .ranges
                .as_mut_ptr()
                .offset(1 as ::core::ffi::c_int as isize))
            .px = 0 as u_int;
            (*(*r)
                .ranges
                .as_mut_ptr()
                .offset(1 as ::core::ffi::c_int as isize))
            .nx = 0 as u_int;
        }
        (*r).used = 2 as u_int;
    }
}
pub unsafe fn server_client_check_nested(mut c: *mut client) -> ::core::ffi::c_int {
    unsafe {
        let envent = environ_find(&*environ_ptr(&(*c).environ), c"TMUX".as_ptr());
        if !envent
            .and_then(environ_entry_value)
            .is_some_and(|value| !value.is_empty())
        {
            return 0 as ::core::ffi::c_int;
        }
        for wp in pane_walk() {
            if strcmp(
                &raw mut (*wp).tty as *mut ::core::ffi::c_char,
                cstr_ptr(&(*c).ttyname),
            ) == 0 as ::core::ffi::c_int
            {
                return 1 as ::core::ffi::c_int;
            }
        }
        0 as ::core::ffi::c_int
    }
}
pub unsafe fn server_client_set_key_table(
    mut c: *mut client,
    mut name: *const ::core::ffi::c_char,
) {
    unsafe {
        if name.is_null() {
            name = server_client_get_key_table(c);
        }
        let table_ref = key_bindings_get_table_ref(name, 1 as ::core::ffi::c_int)
            .expect("key table creation requested");
        (*c).keytable_ref = Some(table_ref);
        let mut now = timeval::default();
        if gettimeofday(&raw mut now, ::core::ptr::null_mut()) != 0 as ::core::ffi::c_int {
            fatal(c"gettimeofday failed".as_ptr(), fmt_args![]);
        }
        key_table_set_activity_time((*c).keytable(), now);
    }
}
unsafe fn server_client_key_table_activity_diff(mut c: *mut client) -> uint64_t {
    unsafe {
        let mut diff = timeval::default();
        let since = key_table_activity_time((*c).keytable());
        diff.tv_sec = (*c).activity_time.tv_sec - since.tv_sec;
        diff.tv_usec = (*c).activity_time.tv_usec - since.tv_usec;
        if diff.tv_usec < 0 as __suseconds_t {
            diff.tv_sec -= 1;
            diff.tv_usec += 1000000 as __suseconds_t;
        }
        (diff.tv_sec as ::core::ffi::c_ulonglong)
            .wrapping_mul(1000 as ::core::ffi::c_ulonglong)
            .wrapping_add(
                (diff.tv_usec as ::core::ffi::c_ulonglong)
                    .wrapping_div(1000 as ::core::ffi::c_ulonglong),
            ) as uint64_t
    }
}
pub unsafe fn server_client_get_key_table(mut c: *mut client) -> *const ::core::ffi::c_char {
    unsafe {
        let mut s: *mut session = (*c).session;
        let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        if s.is_null() {
            return c"root".as_ptr();
        }
        name = options_get_string(session_options(s), c"key-table".as_ptr());
        if *name as ::core::ffi::c_int == '\0' as i32 {
            return c"root".as_ptr();
        }
        name
    }
}
unsafe fn server_client_is_default_key_table(
    mut c: *mut client,
    mut table: *mut key_table,
) -> ::core::ffi::c_int {
    unsafe {
        (key_table_name(table) == CStr::from_ptr(server_client_get_key_table(c)))
            as ::core::ffi::c_int
    }
}

static CLIENT_HANDLES: GlobalTree<usize, ClientWeak> = GlobalTree::new();

pub(crate) fn register_client_handle(reference: &ClientRef) {
    let key = reference.with(|client| client as *const client as usize);
    CLIENT_HANDLES.map().insert(key, reference.downgrade());
}

fn unregister_client_handle(c: *mut client) {
    CLIENT_HANDLES.map().remove(&(c as usize));
}

/// The session the client was attached to before this one, while it lives.
pub unsafe fn client_get_last_session(c: *mut client) -> *mut session {
    unsafe {
        (*c).last_session
            .as_ref()
            .and_then(SessionWeak::upgrade)
            .map_or(::core::ptr::null_mut(), |s| s.as_ptr())
    }
}

/// Records `s` as the session the client was attached to before this one.
pub unsafe fn client_set_last_session(c: *mut client, s: *mut session) {
    unsafe {
        (*c).last_session = session_ref_from_ptr(s).map(|s| s.downgrade());
    }
}

/// The window the client has anchored its pan offset to, while it lives.
pub unsafe fn client_get_pan_window(c: *mut client) -> *mut window {
    unsafe {
        (*c).pan_window
            .as_ref()
            .and_then(WindowWeak::upgrade)
            .map_or(::core::ptr::null_mut(), |w| w.as_ptr())
    }
}

/// Anchors the client's pan offset to `w`.
pub unsafe fn client_set_pan_window(c: *mut client, w: *mut window) {
    unsafe {
        (*c).pan_window = window_ref_from_ptr(w).map(|w| w.downgrade());
    }
}

/// The same client, observed rather than held.
pub(crate) fn client_weak_from_ptr(c: *mut client) -> Option<ClientWeak> {
    client_ref_from_ptr(c).map(|c| c.downgrade())
}

pub(crate) fn client_ref_from_ptr(c: *mut client) -> Option<ClientRef> {
    if c.is_null() {
        return None;
    }
    let key = c as usize;
    let reference = CLIENT_HANDLES.map().get(&key).cloned()?.upgrade();
    if reference
        .as_ref()
        .is_some_and(|reference| reference.with(|client| std::ptr::eq(client, c)))
    {
        return reference;
    }
    CLIENT_HANDLES.map().remove(&key);
    None
}

pub(crate) fn client_registry_clear() {
    CLIENT_HANDLES.map().clear();
}

impl Drop for ClientStorage {
    fn drop(&mut self) {
        unsafe {
            let c = &mut self.value;
            c.event.disable();
            c.repeat_timer.disarm();
            c.click_timer.disarm();
            c.message_timer.disarm();
            c.overlay_timer.disarm();
            c.status.timer.disarm();
            c.tty.start_timer.disarm();
            c.tty.clipboard_timer.disarm();
            if c.overlay().is_some() {
                server_client_clear_overlay(c);
            }
            input_cancel_requests(c);
            status_free(c);
            c.status.screen = screen::default();
            if c.tty.flags & TTY_OPENED != 0 {
                tty_free(&raw mut c.tty);
            }
            format_lost_client(c);
            c.queue = None;
            c.files.clear();
            c.windows.clear();
            c.environ = None;
            c.control_state = None;
            c.name = None;
            c.user = None;
            c.title = None;
            c.path = None;
            c.cwd = None;
            c.term_name = None;
            c.term_type = None;
            c.ttyname = None;
            c.term_caps.clear();
            c.prompt_buffer.clear();
            c.prompt_saved = None;
            c.exit_session = None;
            c.exit_message = None;
            c.message_string = None;
            c.prompt_string = None;
            c.prompt_last = None;
            unregister_client_handle(c);
        }
    }
}

pub fn server_client_create(mut fd: ::core::ffi::c_int) -> *mut client {
    unsafe {
        setblocking(fd, 0 as ::core::ffi::c_int);
        let mut fresh = client::default();
        fresh.fd = -(1 as ::core::ffi::c_int);
        fresh.out_fd = -(1 as ::core::ffi::c_int);
        fresh.click_wp = -(1 as ::core::ffi::c_int);
        fresh.theme = THEME_UNKNOWN;
        fresh.environ = Some(Box::new(environ_t::new()));
        fresh.queue = Some(cmdq_new());
        fresh.tty = tty {
            sx: 80 as u_int,
            sy: 24 as u_int,
            ..tty::default()
        };
        let reference = ClientRef::new(fresh);
        let c = reference.as_ptr();
        (*c).peer = Some(proc_add_peer(
            server_proc,
            fd,
            Some(server_client_dispatch),
            Some(reference.downgrade()),
        ));
        if gettimeofday(&raw mut (*c).creation_time, ::core::ptr::null_mut())
            != 0 as ::core::ffi::c_int
        {
            fatal(c"gettimeofday failed".as_ptr(), fmt_args![]);
        }
        (*c).activity_time = (*c).creation_time;
        status_init(c);
        (*c).flags |= CLIENT_FOCUSED as uint64_t;
        let table_ref = key_bindings_get_table_ref(c"root".as_ptr(), 1 as ::core::ffi::c_int)
            .expect("root key table creation requested");
        (*c).keytable_ref = Some(table_ref);
        (*c).repeat_timer
            .set_callback(move || server_client_repeat_timer(c));
        (*c).click_timer
            .set_callback(move || server_client_click_timer(c));
        clients.push(reference);
        log_debug(c"new client %p".as_ptr(), fmt_args![c]);
        c
    }
}
pub unsafe fn server_client_open(
    mut c: *mut client,
    cause: &mut Option<CString>,
) -> ::core::ffi::c_int {
    unsafe {
        let mut ttynam: *const ::core::ffi::c_char = _PATH_TTY.as_ptr();
        if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            return 0 as ::core::ffi::c_int;
        }
        if strcmp(cstr_ptr(&(*c).ttyname), ttynam) == 0 as ::core::ffi::c_int
            || (isatty(STDIN_FILENO) != 0
                && {
                    ttynam = ttyname(STDIN_FILENO);
                    !ttynam.is_null()
                }
                && strcmp(cstr_ptr(&(*c).ttyname), ttynam) == 0 as ::core::ffi::c_int
                || isatty(STDOUT_FILENO) != 0
                    && {
                        ttynam = ttyname(STDOUT_FILENO);
                        !ttynam.is_null()
                    }
                    && strcmp(cstr_ptr(&(*c).ttyname), ttynam) == 0 as ::core::ffi::c_int
                || isatty(STDERR_FILENO) != 0
                    && {
                        ttynam = ttyname(STDERR_FILENO);
                        !ttynam.is_null()
                    }
                    && strcmp(cstr_ptr(&(*c).ttyname), ttynam) == 0 as ::core::ffi::c_int)
        {
            *cause = Some(xasprintf(
                c"can't use %s".as_ptr(),
                fmt_args![(*c).ttyname.as_deref()],
            ));
            return -(1 as ::core::ffi::c_int);
        }
        if (*c).flags & CLIENT_TERMINAL as uint64_t == 0 {
            *cause = Some(c"not a terminal".to_owned());
            return -(1 as ::core::ffi::c_int);
        }
        if tty_open(&raw mut (*c).tty, cause) != 0 as ::core::ffi::c_int {
            return -(1 as ::core::ffi::c_int);
        }
        0 as ::core::ffi::c_int
    }
}
unsafe fn server_client_attached_lost(mut c: *mut client) {
    unsafe {
        let mut s: *mut session = ::core::ptr::null_mut::<session>();
        let mut found: *mut client = ::core::ptr::null_mut::<client>();
        log_debug(c"lost attached client %p".as_ptr(), fmt_args![c]);
        for w_ref in window_refs() {
            let w = w_ref.as_ptr();
            if window_get_latest(w) == c {
                found = ::core::ptr::null_mut::<client>();
                for loop_0 in client_walk() {
                    s = (*loop_0).session;
                    if !(loop_0 == c || s.is_null() || (*session_get_curw(s)).window() != w)
                        && (found.is_null()
                            || (if (*loop_0).activity_time.tv_sec == (*found).activity_time.tv_sec {
                                ((*loop_0).activity_time.tv_usec > (*found).activity_time.tv_usec)
                                    as ::core::ffi::c_int
                            } else {
                                ((*loop_0).activity_time.tv_sec > (*found).activity_time.tv_sec)
                                    as ::core::ffi::c_int
                            }) != 0)
                    {
                        found = loop_0;
                    }
                }
                if !found.is_null() {
                    server_client_update_latest(found);
                }
            }
        }
    }
}
pub unsafe fn server_client_set_session(mut c: *mut client, mut s: *mut session) {
    unsafe {
        let mut old: *mut session = (*c).session;
        if !s.is_null() && !(*c).session.is_null() && (*c).session != s {
            client_set_last_session(c, (*c).session);
        } else if s.is_null() {
            (*c).last_session = None;
        }
        (*c).session = s;
        (*c).flags |= CLIENT_FOCUSED as uint64_t;
        if !old.is_null() && !session_get_curw(old).is_null() {
            window_update_focus((*session_get_curw(old)).window());
        }
        if !s.is_null() {
            window_set_latest((*session_get_curw(s)).window(), c);
            recalculate_sizes();
            window_update_focus((*session_get_curw(s)).window());
            session_update_activity(s, ::core::ptr::null_mut::<timeval>());
            session_theme_changed(s);
            gettimeofday(&raw mut (*s).last_attached_time, ::core::ptr::null_mut());
            (*session_get_curw(s)).flags &= !WINLINK_ALERTFLAGS;
            alerts_check_session(s);
            tty_update_client_offset(c);
            status_timer_start(c);
            notify_client(c"client-session-changed".as_ptr(), c);
            server_redraw_client(c);
        }
        server_check_unattached();
        server_update_socket();
    }
}
pub unsafe fn server_client_lost(mut c: *mut client) {
    unsafe {
        if (*c).flags & CLIENT_DEAD as uint64_t != 0 {
            return;
        }
        let client_ref = client_ref_from_ptr(c);
        (*c).flags |= CLIENT_DEAD as uint64_t;
        server_client_clear_overlay(c);
        status_prompt_clear(c);
        status_message_clear(c);
        for cf in (*c).files.values().cloned().collect::<Vec<_>>() {
            (*cf.as_ptr()).error = EINTR;
            file_fire_done(cf);
        }
        drop(::core::mem::take(&mut (*c).windows));
        clients.retain(|listed| listed.as_ptr() != c);
        log_debug(c"lost client %p".as_ptr(), fmt_args![c]);
        if (*c).flags & CLIENT_ATTACHED as uint64_t != 0 {
            server_client_attached_lost(c);
            notify_client(c"client-detached".as_ptr(), c);
        }
        if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            control_stop(c);
        }
        if (*c).flags & CLIENT_TERMINAL as uint64_t != 0 {
            tty_free(&raw mut (*c).tty);
        }
        (*c).ttyname = None;
        (*c).term_name = None;
        (*c).term_type = None;
        drop(::core::mem::take(&mut (*c).term_caps));
        status_free(c);
        (*c).status.screen = screen::default();
        input_cancel_requests(c);
        (*c).title = None;
        (*c).path = None;
        (*c).cwd = None;
        (*c).exit_session = None;
        (*c).exit_message = None;
        (*c).repeat_timer.disarm();
        (*c).click_timer.disarm();
        (*c).keytable_ref = None;
        (*c).message_string = None;
        (*c).message_timer.disarm();
        (*c).prompt_saved = None;
        (*c).prompt_string = None;
        (*c).prompt_last = None;
        (*c).prompt_buffer = Vec::new();
        format_lost_client(c);
        (*c).environ = None;
        if let Some(peer) = (*c).peer.take() {
            proc_remove_peer(peer);
        }
        if (*c).out_fd != -(1 as ::core::ffi::c_int) {
            close((*c).out_fd);
        }
        if (*c).fd != -(1 as ::core::ffi::c_int) {
            close((*c).fd);
            (*c).fd = -(1 as ::core::ffi::c_int);
        }
        if let Some(client_ref) = client_ref {
            reactor::current().defer(move || drop(client_ref));
        }
        server_add_accept(0 as ::core::ffi::c_int);
        recalculate_sizes();
        server_check_unattached();
        server_update_socket();
    }
}
pub unsafe fn server_client_suspend(mut c: *mut client) {
    unsafe {
        let mut s: *mut session = (*c).session;
        if s.is_null() || (*c).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0 {
            return;
        }
        tty_stop_tty(&raw mut (*c).tty);
        (*c).flags |= CLIENT_SUSPENDED as uint64_t;
        proc_send(
            peer_ptr(&(*c).peer),
            MSG_SUSPEND,
            -(1 as ::core::ffi::c_int),
            ::core::ptr::null::<u8>(),
            0 as size_t,
        );
    }
}
pub unsafe fn server_client_detach(mut c: *mut client, mut msgtype: msgtype) {
    unsafe {
        let mut s: *mut session = (*c).session;
        if s.is_null() || (*c).flags & CLIENT_NODETACHFLAGS as uint64_t != 0 {
            return;
        }
        (*c).flags |= CLIENT_EXIT as uint64_t;
        (*c).exit_type = CLIENT_EXIT_DETACH;
        (*c).exit_msgtype = msgtype;
        (*c).exit_session = session_name_owned(s);
    }
}
pub unsafe fn server_client_exec(mut c: *mut client, mut cmd: *const ::core::ffi::c_char) {
    unsafe {
        let mut s: *mut session = (*c).session;
        let mut shell: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut cmdsize: size_t = 0;
        let mut shellsize: size_t = 0;
        if *cmd as ::core::ffi::c_int == '\0' as i32 {
            return;
        }
        cmdsize = strlen(cmd).wrapping_add(1 as size_t);
        if !s.is_null() {
            shell = options_get_string(session_options(s), c"default-shell".as_ptr());
        } else {
            shell = options_get_string(global_s_options, c"default-shell".as_ptr());
        }
        if checkshell(shell) == 0 {
            shell = _PATH_BSHELL.as_ptr();
        }
        shellsize = strlen(shell).wrapping_add(1 as size_t);
        let mut msg: Vec<u8> = Vec::with_capacity(cmdsize.wrapping_add(shellsize) as usize);
        msg.extend_from_slice(::core::slice::from_raw_parts(
            cmd as *const u8,
            cmdsize as usize,
        ));
        msg.extend_from_slice(::core::slice::from_raw_parts(
            shell as *const u8,
            shellsize as usize,
        ));
        proc_send(
            peer_ptr(&(*c).peer),
            MSG_EXEC,
            -(1 as ::core::ffi::c_int),
            msg.as_ptr(),
            cmdsize.wrapping_add(shellsize),
        );
    }
}
unsafe fn server_client_check_mouse_in_pane(
    mut wp: *mut window_pane,
    mut px: ::core::ffi::c_int,
    mut py: ::core::ffi::c_int,
    sl_mpos: &mut u_int,
) -> key_code_mouse_location {
    unsafe {
        let mut w: *mut window = (*wp).window;
        let mut fwp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut pane_status: ::core::ffi::c_int = 0;
        let mut sb_w: ::core::ffi::c_int = 0;
        let mut sb_pad: ::core::ffi::c_int = 0;
        let mut pane_status_line: ::core::ffi::c_int = 0;
        let mut sl_top: ::core::ffi::c_int = 0;
        let mut sl_bottom: ::core::ffi::c_int = 0;
        let mut bdr_bottom: ::core::ffi::c_int = 0;
        let mut bdr_top: ::core::ffi::c_int = 0;
        let mut bdr_left: ::core::ffi::c_int = 0;
        let mut bdr_right: ::core::ffi::c_int = 0;
        pane_status = options_get_number((*w).options_ptr(), c"pane-border-status".as_ptr())
            as ::core::ffi::c_int;
        if window_pane_show_scrollbar(wp) != 0 {
            sb_w = (*wp).scrollbar_style.width;
            sb_pad = (*wp).scrollbar_style.pad;
        } else {
            sb_w = 0 as ::core::ffi::c_int;
            sb_pad = 0 as ::core::ffi::c_int;
        }
        if pane_status == PANE_STATUS_TOP {
            pane_status_line = (*wp).yoff - 1 as ::core::ffi::c_int;
        } else if pane_status == PANE_STATUS_BOTTOM {
            pane_status_line = ((*wp).yoff as u_int).wrapping_add((*wp).sy) as ::core::ffi::c_int;
        } else {
            pane_status_line = -(1 as ::core::ffi::c_int);
        }
        bdr_left = (*wp).xoff - 1 as ::core::ffi::c_int;
        if (*w).sb_pos == PANE_SCROLLBARS_LEFT {
            bdr_left -= sb_pad + sb_w;
        }
        if (pane_status != PANE_STATUS_OFF
            && py != pane_status_line
            && py != (*wp).yoff + (*wp).sy as ::core::ffi::c_int
            || (*wp).yoff == 0 as ::core::ffi::c_int && py < (*wp).sy as ::core::ffi::c_int
            || py >= (*wp).yoff && py < (*wp).yoff + (*wp).sy as ::core::ffi::c_int)
            && ((*w).sb_pos == PANE_SCROLLBARS_RIGHT
                && px < (*wp).xoff + (*wp).sx as ::core::ffi::c_int + sb_pad + sb_w
                || (*w).sb_pos == PANE_SCROLLBARS_LEFT
                    && px < (*wp).xoff + (*wp).sx as ::core::ffi::c_int - sb_pad - sb_w)
        {
            if (*w).sb_pos == PANE_SCROLLBARS_RIGHT
                && (px >= (*wp).xoff + (*wp).sx as ::core::ffi::c_int + sb_pad
                    && px < (*wp).xoff + (*wp).sx as ::core::ffi::c_int + sb_pad + sb_w)
                || (*w).sb_pos == PANE_SCROLLBARS_LEFT
                    && (px >= (*wp).xoff - sb_pad - sb_w && px < (*wp).xoff - sb_pad)
            {
                sl_top =
                    ((*wp).yoff as u_int).wrapping_add((*wp).sb_slider_y) as ::core::ffi::c_int;
                sl_bottom = ((*wp).yoff as u_int)
                    .wrapping_add((*wp).sb_slider_y)
                    .wrapping_add((*wp).sb_slider_h)
                    .wrapping_sub(1 as u_int) as ::core::ffi::c_int;
                if py < sl_top {
                    return KEYC_MOUSE_LOCATION_SCROLLBAR_UP;
                } else if py >= sl_top && py <= sl_bottom {
                    *sl_mpos = (py as u_int)
                        .wrapping_sub((*wp).sb_slider_y)
                        .wrapping_sub((*wp).yoff as u_int);
                    return KEYC_MOUSE_LOCATION_SCROLLBAR_SLIDER;
                } else {
                    return KEYC_MOUSE_LOCATION_SCROLLBAR_DOWN;
                }
            } else if window_pane_is_floating(wp) != 0
                && (px == bdr_left
                    || py == (*wp).yoff - 1 as ::core::ffi::c_int
                    || py == (*wp).yoff + (*wp).sy as ::core::ffi::c_int)
            {
                return KEYC_MOUSE_LOCATION_BORDER;
            } else {
                return KEYC_MOUSE_LOCATION_PANE;
            }
        } else {
            fwp = window_panes_first(w);
            while !fwp.is_null() {
                if !((*w).flags & WINDOW_ZOOMED != 0 && !(*fwp).flags & PANE_ZOOMED != 0) {
                    if window_pane_show_scrollbar(fwp) != 0 {
                        sb_w = (*fwp).scrollbar_style.width;
                        sb_pad = (*fwp).scrollbar_style.pad;
                    } else {
                        sb_w = 0 as ::core::ffi::c_int;
                        sb_pad = 0 as ::core::ffi::c_int;
                    }
                    bdr_top = (*fwp).yoff - 1 as ::core::ffi::c_int;
                    bdr_bottom =
                        ((*fwp).yoff as u_int).wrapping_add((*fwp).sy) as ::core::ffi::c_int;
                    bdr_left = (*fwp).xoff - 1 as ::core::ffi::c_int;
                    if (*w).sb_pos == PANE_SCROLLBARS_LEFT {
                        bdr_left -= sb_pad + sb_w;
                        bdr_right =
                            ((*fwp).xoff as u_int).wrapping_add((*fwp).sx) as ::core::ffi::c_int;
                    } else {
                        bdr_right = ((*fwp).xoff as u_int)
                            .wrapping_add((*fwp).sx)
                            .wrapping_add(sb_pad as u_int)
                            .wrapping_add(sb_w as u_int)
                            as ::core::ffi::c_int;
                    }
                    if py >= (*fwp).yoff - 1 as ::core::ffi::c_int
                        && py <= (*fwp).yoff + (*fwp).sy as ::core::ffi::c_int
                    {
                        if px == bdr_right {
                            break;
                        }
                        if window_pane_is_floating(wp) != 0 && px == bdr_left {
                            break;
                        }
                    }
                    if px >= bdr_left && px <= (*fwp).xoff + (*fwp).sx as ::core::ffi::c_int {
                        bdr_bottom =
                            ((*fwp).yoff as u_int).wrapping_add((*fwp).sy) as ::core::ffi::c_int;
                        if py == bdr_bottom {
                            break;
                        }
                        if py == bdr_top {
                            break;
                        }
                    }
                }
                fwp = window_panes_next(w, fwp);
            }
            if !fwp.is_null() {
                return KEYC_MOUSE_LOCATION_BORDER;
            }
        }
        KEYC_MOUSE_LOCATION_NOWHERE
    }
}
unsafe fn server_client_check_mouse(mut c: *mut client, mut event: *mut key_event) -> key_code {
    unsafe {
        let mut current_block: u64;
        let mut m: *mut mouse_event = &raw mut (*event).m;
        let mut s: *mut session = (*c).session;
        let mut fs: *mut session = ::core::ptr::null_mut::<session>();
        let mut w: *mut window = (*session_get_curw(s)).window();
        let mut fwl: *mut winlink = ::core::ptr::null_mut::<winlink>();
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut fwp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut lwp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut x: u_int = 0;
        let mut y: u_int = 0;
        let mut sx: u_int = 0;
        let mut sy: u_int = 0;
        let mut px: u_int = 0;
        let mut py: u_int = 0;
        let mut n: u_int = 0;
        let mut sl_mpos: u_int = 0 as u_int;
        let mut b: u_int = 0;
        let mut bn: u_int = 0;
        let mut ignore: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut key: key_code = 0;
        let mut tv = timeval::default();
        let mut sr: *mut style_range = ::core::ptr::null_mut::<style_range>();
        let mut type_0: key_code_type = KEYC_TYPE_NOTYPE;
        let mut loc: key_code_mouse_location = KEYC_MOUSE_LOCATION_NOWHERE;
        log_debug(
            c"%s mouse %02x at %u,%u (last %u,%u) (%d)".as_ptr(),
            fmt_args![
                (*c).name.as_deref(),
                (*m).b,
                (*m).x,
                (*m).y,
                (*m).lx,
                (*m).ly,
                (*c).tty.mouse_drag_flag
            ],
        );
        if (*c).tty.mouse_last_pane != -(1 as ::core::ffi::c_int) {
            lwp = window_pane_find_by_id((*c).tty.mouse_last_pane as u_int);
            if !lwp.is_null() {
                log_debug(
                    c"%s mouse last pane %%%u".as_ptr(),
                    fmt_args![(*c).name.as_deref(), (*lwp).id],
                );
            }
        }
        if (*event).key == KEYC_DOUBLECLICK as ::core::ffi::c_ulong as key_code {
            type_0 = KEYC_TYPE_DOUBLECLICK;
            x = (*m).x;
            y = (*m).y;
            b = (*m).b;
            ignore = 1 as ::core::ffi::c_int;
            log_debug(c"double-click at %u,%u".as_ptr(), fmt_args![x, y]);
        } else if (*m).sgr_type != ' ' as i32 as u_int
            && (*m).sgr_b & MOUSE_MASK_DRAG as u_int != 0
            && (*m).sgr_b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int
            || (*m).sgr_type == ' ' as i32 as u_int
                && (*m).b & MOUSE_MASK_DRAG as u_int != 0
                && (*m).b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int
                && (*m).lb & MOUSE_MASK_BUTTONS as u_int == 3 as u_int
        {
            type_0 = KEYC_TYPE_MOUSEMOVE;
            x = (*m).x;
            y = (*m).y;
            b = 0 as u_int;
            log_debug(c"move at %u,%u".as_ptr(), fmt_args![x, y]);
        } else if (*m).b & MOUSE_MASK_DRAG as u_int != 0 {
            type_0 = KEYC_TYPE_MOUSEDRAG;
            if (*c).tty.mouse_drag_flag != 0 {
                x = (*m).x;
                y = (*m).y;
                b = (*m).b;
                if x == (*m).lx && y == (*m).ly {
                    return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
                }
                log_debug(c"drag update at %u,%u".as_ptr(), fmt_args![x, y]);
            } else {
                x = (*m).lx;
                y = (*m).ly;
                b = (*m).lb;
                log_debug(c"drag start at %u,%u".as_ptr(), fmt_args![x, y]);
            }
        } else if (*m).b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_UP as u_int
            || (*m).b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_DOWN as u_int
        {
            if (*m).b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_UP as u_int {
                type_0 = KEYC_TYPE_WHEELUP;
            } else {
                type_0 = KEYC_TYPE_WHEELDOWN;
            }
            x = (*m).x;
            y = (*m).y;
            b = (*m).b;
            log_debug(c"wheel at %u,%u".as_ptr(), fmt_args![x, y]);
        } else if (*m).b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int {
            type_0 = KEYC_TYPE_MOUSEUP;
            x = (*m).x;
            y = (*m).y;
            b = (*m).lb;
            if (*m).sgr_type == 'm' as i32 as u_int {
                b = (*m).sgr_b;
            }
            log_debug(c"up at %u,%u".as_ptr(), fmt_args![x, y]);
        } else {
            if (*c).flags & CLIENT_DOUBLECLICK as uint64_t != 0 {
                (*c).click_timer.disarm();
                (*c).flags &= !CLIENT_DOUBLECLICK as uint64_t;
                type_0 = KEYC_TYPE_SECONDCLICK;
                x = (*m).x;
                y = (*m).y;
                b = (*m).b;
                log_debug(c"second-click at %u,%u".as_ptr(), fmt_args![x, y]);
                (*c).flags |= CLIENT_TRIPLECLICK as uint64_t;
                current_block = 9441801433784995173;
            } else if (*c).flags & CLIENT_TRIPLECLICK as uint64_t != 0 {
                (*c).click_timer.disarm();
                (*c).flags &= !CLIENT_TRIPLECLICK as uint64_t;
                type_0 = KEYC_TYPE_TRIPLECLICK;
                x = (*m).x;
                y = (*m).y;
                b = (*m).b;
                log_debug(c"triple-click at %u,%u".as_ptr(), fmt_args![x, y]);
                current_block = 12711162995783632332;
            } else {
                current_block = 9441801433784995173;
            }
            match current_block {
                12711162995783632332 => {}
                _ => {
                    if type_0 as ::core::ffi::c_uint
                        == KEYC_TYPE_NOTYPE as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        type_0 = KEYC_TYPE_MOUSEDOWN;
                        x = (*m).x;
                        y = (*m).y;
                        b = (*m).b;
                        log_debug(c"down at %u,%u".as_ptr(), fmt_args![x, y]);
                        (*c).flags |= CLIENT_DOUBLECLICK as uint64_t;
                    }
                }
            }
        }
        if type_0 as ::core::ffi::c_uint
            == KEYC_TYPE_NOTYPE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
        }
        (*m).s = session_id(s) as ::core::ffi::c_int;
        (*m).w = -(1 as ::core::ffi::c_int);
        (*m).wp = -(1 as ::core::ffi::c_int);
        (*m).ignore = ignore;
        (*m).statusat = status_at_line(c);
        (*m).statuslines = status_line_size(c);
        if (*m).statusat != -(1 as ::core::ffi::c_int)
            && y >= (*m).statusat as u_int
            && y < ((*m).statusat as u_int).wrapping_add((*m).statuslines)
        {
            sr = status_get_range(c, x, y.wrapping_sub((*m).statusat as u_int));
            if sr.is_null() {
                loc = KEYC_MOUSE_LOCATION_STATUS_DEFAULT;
            } else {
                match (*sr).type_0 {
                    STYLE_RANGE_NONE => return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code,
                    STYLE_RANGE_LEFT => {
                        log_debug(c"mouse range: left".as_ptr(), fmt_args![]);
                        loc = KEYC_MOUSE_LOCATION_STATUS_LEFT;
                    }
                    STYLE_RANGE_RIGHT => {
                        log_debug(c"mouse range: right".as_ptr(), fmt_args![]);
                        loc = KEYC_MOUSE_LOCATION_STATUS_RIGHT;
                    }
                    STYLE_RANGE_PANE => {
                        fwp = window_pane_find_by_id((*sr).argument);
                        if fwp.is_null() {
                            return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
                        }
                        (*m).wp = (*sr).argument as ::core::ffi::c_int;
                        log_debug(c"mouse range: pane %%%u".as_ptr(), fmt_args![(*m).wp]);
                        loc = KEYC_MOUSE_LOCATION_STATUS;
                    }
                    STYLE_RANGE_WINDOW => {
                        fwl = winlink_find_by_index(
                            &raw mut (*s).windows,
                            (*sr).argument as ::core::ffi::c_int,
                        );
                        if fwl.is_null() {
                            return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
                        }
                        (*m).w = (*(*fwl).window()).id as ::core::ffi::c_int;
                        log_debug(c"mouse range: window @%u".as_ptr(), fmt_args![(*m).w]);
                        loc = KEYC_MOUSE_LOCATION_STATUS;
                    }
                    STYLE_RANGE_SESSION => {
                        fs = session_find_by_id((*sr).argument);
                        if fs.is_null() {
                            return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
                        }
                        (*m).s = (*sr).argument as ::core::ffi::c_int;
                        log_debug(c"mouse range: session $%u".as_ptr(), fmt_args![(*m).s]);
                        loc = KEYC_MOUSE_LOCATION_STATUS;
                    }
                    STYLE_RANGE_USER => {
                        log_debug(c"mouse range: user".as_ptr(), fmt_args![]);
                        loc = KEYC_MOUSE_LOCATION_STATUS;
                    }
                    STYLE_RANGE_CONTROL => {
                        n = (*sr).argument;
                        log_debug(c"mouse range: control %u".as_ptr(), fmt_args![n]);
                        loc = (KEYC_MOUSE_LOCATION_CONTROL0 as ::core::ffi::c_int as u_int)
                            .wrapping_add(n)
                            as key_code_mouse_location;
                    }
                    _ => {}
                }
            }
        }
        if loc as ::core::ffi::c_uint
            == KEYC_MOUSE_LOCATION_NOWHERE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if (*c).tty.mouse_scrolling_flag != 0 {
                if !lwp.is_null() {
                    loc = KEYC_MOUSE_LOCATION_SCROLLBAR_SLIDER;
                    (*m).wp = (*lwp).id as ::core::ffi::c_int;
                    (*m).w = (*(*lwp).window).id as ::core::ffi::c_int;
                }
            } else {
                px = x;
                if (*m).statusat == 0 as ::core::ffi::c_int && y >= (*m).statuslines {
                    py = y.wrapping_sub((*m).statuslines);
                } else if (*m).statusat > 0 as ::core::ffi::c_int && y >= (*m).statusat as u_int {
                    py = ((*m).statusat - 1 as ::core::ffi::c_int) as u_int;
                } else {
                    py = y;
                }
                {
                    let (window_bigger, off_x, off_y, off_sx, off_sy) =
                        tty_window_offset(&raw mut (*c).tty);
                    ((*m).ox, (*m).oy, sx, sy) = (off_x, off_y, off_sx, off_sy);
                    window_bigger
                };
                log_debug(
                    c"mouse window @%u at %u,%u (%ux%u)".as_ptr(),
                    fmt_args![(*w).id, (*m).ox, (*m).oy, sx, sy],
                );
                if px > sx || py > sy {
                    return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
                }
                px = px.wrapping_add((*m).ox);
                py = py.wrapping_add((*m).oy);
                if type_0 as ::core::ffi::c_uint
                    == KEYC_TYPE_MOUSEDRAG as ::core::ffi::c_int as ::core::ffi::c_uint
                    && !lwp.is_null()
                {
                    wp = lwp;
                } else {
                    wp = window_get_active_at(w, px, py);
                }
                if wp.is_null() {
                    return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
                }
                loc = server_client_check_mouse_in_pane(
                    wp,
                    px as ::core::ffi::c_int,
                    py as ::core::ffi::c_int,
                    &mut sl_mpos,
                );
                if loc as ::core::ffi::c_uint
                    == KEYC_MOUSE_LOCATION_PANE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    log_debug(
                        c"mouse %u,%u on pane %%%u".as_ptr(),
                        fmt_args![x, y, (*wp).id],
                    );
                } else if loc as ::core::ffi::c_uint
                    == KEYC_MOUSE_LOCATION_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    sr = window_pane_border_status_get_range(wp, px, py);
                    if !sr.is_null() {
                        n = (*sr).argument;
                        loc = (KEYC_MOUSE_LOCATION_CONTROL0 as ::core::ffi::c_int as u_int)
                            .wrapping_add(n)
                            as key_code_mouse_location;
                    }
                    log_debug(c"mouse on pane %%%u border".as_ptr(), fmt_args![(*wp).id]);
                } else if loc as ::core::ffi::c_uint
                    == KEYC_MOUSE_LOCATION_SCROLLBAR_UP as ::core::ffi::c_int as ::core::ffi::c_uint
                    || loc as ::core::ffi::c_uint
                        == KEYC_MOUSE_LOCATION_SCROLLBAR_SLIDER as ::core::ffi::c_int
                            as ::core::ffi::c_uint
                    || loc as ::core::ffi::c_uint
                        == KEYC_MOUSE_LOCATION_SCROLLBAR_DOWN as ::core::ffi::c_int
                            as ::core::ffi::c_uint
                {
                    log_debug(
                        c"mouse on pane %%%u scrollbar".as_ptr(),
                        fmt_args![(*wp).id],
                    );
                }
                (*m).wp = (*wp).id as ::core::ffi::c_int;
                (*m).w = (*(*wp).window).id as ::core::ffi::c_int;
            }
        }
        if type_0 as ::core::ffi::c_uint
            == KEYC_TYPE_MOUSEDOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            || type_0 as ::core::ffi::c_uint
                == KEYC_TYPE_SECONDCLICK as ::core::ffi::c_int as ::core::ffi::c_uint
            || type_0 as ::core::ffi::c_uint
                == KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if type_0 as ::core::ffi::c_uint
                != KEYC_TYPE_MOUSEDOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                && ((*m).b != (*c).click_button
                    || loc as ::core::ffi::c_uint
                        != (*c).click_loc as key_code_mouse_location as ::core::ffi::c_uint
                    || (*m).wp != (*c).click_wp)
            {
                type_0 = KEYC_TYPE_MOUSEDOWN;
                log_debug(c"click sequence reset at %u,%u".as_ptr(), fmt_args![x, y]);
                (*c).flags &= !CLIENT_TRIPLECLICK as uint64_t;
                (*c).flags |= CLIENT_DOUBLECLICK as uint64_t;
            }
            if type_0 as ::core::ffi::c_uint
                != KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_uint
                && KEYC_CLICK_TIMEOUT != 0 as ::core::ffi::c_int
            {
                (*c).click_event = *m;
                (*c).click_button = (*m).b;
                (*c).click_loc = loc as ::core::ffi::c_int;
                (*c).click_wp = (*m).wp;
                log_debug(c"click timer started".as_ptr(), fmt_args![]);
                tv.tv_sec = (KEYC_CLICK_TIMEOUT / 1000 as ::core::ffi::c_int) as __time_t;
                tv.tv_usec = ((KEYC_CLICK_TIMEOUT % 1000 as ::core::ffi::c_int)
                    as ::core::ffi::c_long
                    * 1000 as ::core::ffi::c_long) as __suseconds_t;
                (*c).click_timer.disarm();
                (*c).click_timer.arm(tv);
            }
        }
        key = KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
        if type_0 as ::core::ffi::c_uint
            != KEYC_TYPE_MOUSEDRAG as ::core::ffi::c_int as ::core::ffi::c_uint
            && type_0 as ::core::ffi::c_uint
                != KEYC_TYPE_WHEELUP as ::core::ffi::c_int as ::core::ffi::c_uint
            && type_0 as ::core::ffi::c_uint
                != KEYC_TYPE_WHEELDOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            && type_0 as ::core::ffi::c_uint
                != KEYC_TYPE_DOUBLECLICK as ::core::ffi::c_int as ::core::ffi::c_uint
            && type_0 as ::core::ffi::c_uint
                != KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*c).tty.mouse_drag_flag != 0 as ::core::ffi::c_int
        {
            if (*c).tty.mouse_drag_release.is_some() {
                (*c).tty
                    .mouse_drag_release
                    .expect("non-null function pointer")(c, m);
            }
            (*c).tty.mouse_drag_update = None;
            (*c).tty.mouse_drag_release = None;
            (*c).tty.mouse_scrolling_flag = 0 as ::core::ffi::c_int;
            type_0 = KEYC_TYPE_MOUSEDRAGEND;
            (*c).tty.mouse_drag_flag = 0 as ::core::ffi::c_int;
            (*c).tty.mouse_slider_mpos = -(1 as ::core::ffi::c_int);
            (*c).tty.mouse_last_pane = -(1 as ::core::ffi::c_int);
        }
        if type_0 as ::core::ffi::c_uint
            == KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_uint
            && loc as ::core::ffi::c_uint
                == KEYC_MOUSE_LOCATION_PANE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            key = KEYC_MOUSEMOVE_PANE as ::core::ffi::c_ulong as key_code;
            if !wp.is_null()
                && wp != window_get_active(w)
                && options_get_number(session_options(s), c"focus-follows-mouse".as_ptr()) != 0
            {
                window_redraw_active_switch(w, wp);
                window_set_active_pane(w, wp, 1 as ::core::ffi::c_int);
                server_redraw_window_borders(w);
                server_status_window(w);
            }
        }
        if type_0 as ::core::ffi::c_uint
            == KEYC_TYPE_MOUSEDRAG as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if (*c).tty.mouse_drag_update.is_some() {
                key = KEYC_DRAGGING as ::core::ffi::c_ulong as key_code;
            }
            (*c).tty.mouse_drag_flag =
                (b & MOUSE_MASK_BUTTONS as u_int).wrapping_add(1 as u_int) as ::core::ffi::c_int;
            if lwp.is_null() {
                wp = window_get_active_at(w, px, py);
                lwp = wp;
                if !wp.is_null() {
                    (*c).tty.mouse_last_pane = (*wp).id as ::core::ffi::c_int;
                }
            }
            if (*c).tty.mouse_scrolling_flag == 0 as ::core::ffi::c_int
                && loc as ::core::ffi::c_uint
                    == KEYC_MOUSE_LOCATION_SCROLLBAR_SLIDER as ::core::ffi::c_int
                        as ::core::ffi::c_uint
            {
                (*c).tty.mouse_scrolling_flag = 1 as ::core::ffi::c_int;
                if (*m).statusat == 0 as ::core::ffi::c_int {
                    (*c).tty.mouse_slider_mpos =
                        sl_mpos.wrapping_add((*m).statuslines) as ::core::ffi::c_int;
                } else {
                    (*c).tty.mouse_slider_mpos = sl_mpos as ::core::ffi::c_int;
                }
            }
        }
        if key == KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code {
            if b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_1 as u_int {
                bn = 1 as u_int;
            } else if b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_2 as u_int {
                bn = 2 as u_int;
            } else if b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_3 as u_int {
                bn = 3 as u_int;
            } else if b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_6 as u_int {
                bn = 6 as u_int;
            } else if b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_7 as u_int {
                bn = 7 as u_int;
            } else if b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_8 as u_int {
                bn = 8 as u_int;
            } else if b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_9 as u_int {
                bn = 9 as u_int;
            } else if b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_10 as u_int {
                bn = 10 as u_int;
            } else if b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_11 as u_int {
                bn = 11 as u_int;
            } else {
                bn = 0 as u_int;
            }
            key = ((type_0 as ::core::ffi::c_ulonglong) << 32 as ::core::ffi::c_int
                | (bn as ::core::ffi::c_ulonglong) << KEYC_MOUSE_BUTTON_SHIFT
                | (loc as ::core::ffi::c_ulonglong) << KEYC_MOUSE_LOCATION_SHIFT)
                as key_code;
        }
        if b & MOUSE_MASK_META as u_int != 0 {
            key |= KEYC_META;
        }
        if b & MOUSE_MASK_CTRL as u_int != 0 {
            key |= KEYC_CTRL;
        }
        if b & MOUSE_MASK_SHIFT as u_int != 0 {
            key |= KEYC_SHIFT;
        }
        if log_get_level() != 0 as ::core::ffi::c_int {
            log_debug(
                c"mouse key is %s".as_ptr(),
                fmt_args![key_string_lookup_key(key, 1 as ::core::ffi::c_int)],
            );
        }
        key
    }
}
unsafe fn server_client_is_bracket_paste(
    mut c: *mut client,
    mut key: key_code,
) -> ::core::ffi::c_int {
    unsafe {
        if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
            == KEYC_PASTE_START as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
        {
            (*c).flags =
                ((*c).flags as ::core::ffi::c_ulonglong | CLIENT_BRACKETPASTING) as uint64_t;
            (*c).paste_time = current_time;
            log_debug(
                c"%s: bracket paste on".as_ptr(),
                fmt_args![(*c).name.as_deref()],
            );
            return 0 as ::core::ffi::c_int;
        }
        if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
            == KEYC_PASTE_END as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
        {
            (*c).flags =
                ((*c).flags as ::core::ffi::c_ulonglong & !CLIENT_BRACKETPASTING) as uint64_t;
            log_debug(
                c"%s: bracket paste off".as_ptr(),
                fmt_args![(*c).name.as_deref()],
            );
            return 0 as ::core::ffi::c_int;
        }
        ((*c).flags as ::core::ffi::c_ulonglong & CLIENT_BRACKETPASTING != 0) as ::core::ffi::c_int
    }
}
unsafe fn server_client_is_assume_paste(mut c: *mut client) -> ::core::ffi::c_int {
    unsafe {
        let mut s: *mut session = (*c).session;
        let mut tv = timeval::default();
        let mut t: ::core::ffi::c_int = 0;
        if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_BRACKETPASTING != 0 {
            return 0 as ::core::ffi::c_int;
        }
        t = options_get_number(session_options(s), c"assume-paste-time".as_ptr())
            as ::core::ffi::c_int;
        if t == 0 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        if tty_term_has(tty_term_of(&(*c).tty), TTYC_ENBP) != 0 {
            return 0 as ::core::ffi::c_int;
        }
        tv.tv_sec = (*c).activity_time.tv_sec - (*c).last_activity_time.tv_sec;
        tv.tv_usec = (*c).activity_time.tv_usec - (*c).last_activity_time.tv_usec;
        if tv.tv_usec < 0 as __suseconds_t {
            tv.tv_sec -= 1;
            tv.tv_usec += 1000000 as __suseconds_t;
        }
        if tv.tv_sec == 0 as __time_t
            && tv.tv_usec < (t * 1000 as ::core::ffi::c_int) as __suseconds_t
        {
            if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_ASSUMEPASTING != 0 {
                return 1 as ::core::ffi::c_int;
            }
            (*c).flags =
                ((*c).flags as ::core::ffi::c_ulonglong | CLIENT_ASSUMEPASTING) as uint64_t;
            (*c).paste_time = current_time;
            log_debug(
                c"%s: assume paste on".as_ptr(),
                fmt_args![(*c).name.as_deref()],
            );
            return 0 as ::core::ffi::c_int;
        }
        if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_ASSUMEPASTING != 0 {
            (*c).flags =
                ((*c).flags as ::core::ffi::c_ulonglong & !CLIENT_ASSUMEPASTING) as uint64_t;
            log_debug(
                c"%s: assume paste off".as_ptr(),
                fmt_args![(*c).name.as_deref()],
            );
        }
        0 as ::core::ffi::c_int
    }
}
unsafe fn server_client_update_latest(mut c: *mut client) {
    unsafe {
        let mut w: *mut window = ::core::ptr::null_mut::<window>();
        if (*c).session.is_null() {
            return;
        }
        w = (*session_get_curw((*c).session)).window();
        if window_get_latest(w) == c {
            return;
        }
        window_set_latest(w, c);
        if options_get_number((*w).options_ptr(), c"window-size".as_ptr())
            == WINDOW_SIZE_LATEST as ::core::ffi::c_longlong
        {
            recalculate_size(w, 0 as ::core::ffi::c_int);
        }
        notify_client(c"client-active".as_ptr(), c);
    }
}
unsafe fn server_client_repeat_time(mut c: *mut client, mut bd: *mut key_binding) -> u_int {
    unsafe {
        let mut s: *mut session = (*c).session;
        let mut repeat: u_int = 0;
        let mut initial: u_int = 0;
        if !key_binding_flags(bd) & KEY_BINDING_REPEAT != 0 {
            return 0 as u_int;
        }
        repeat = options_get_number(session_options(s), c"repeat-time".as_ptr()) as u_int;
        if repeat == 0 as u_int {
            return 0 as u_int;
        }
        if !(*c).flags & CLIENT_REPEAT as uint64_t != 0 || key_binding_key(bd) != (*c).last_key {
            initial =
                options_get_number(session_options(s), c"initial-repeat-time".as_ptr()) as u_int;
            if initial != 0 as u_int {
                repeat = initial;
            }
        }
        repeat
    }
}
unsafe fn server_client_key_callback(
    mut item: *mut cmdq_item,
    data: CmdqCallbackData,
) -> cmd_retval {
    unsafe {
        let mut current_block: u64;
        let mut c: *mut client = cmdq_get_client(&*item);
        let mut event = match data {
            CmdqCallbackData::KeyEvent(event) => event,
            _ => return CMD_RETURN_ERROR,
        };
        let event = event.as_mut() as *mut key_event;
        let mut key: key_code = (*event).key;
        let mut m: *mut mouse_event = &raw mut (*event).m;
        let mut s: *mut session = (*c).session;
        let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
        let mut tv = timeval::default();
        let mut table: *mut key_table = ::core::ptr::null_mut::<key_table>();
        let mut table_ref: Option<KeyTableRef> = None;
        let mut first: *mut key_table = ::core::ptr::null_mut::<key_table>();
        let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
        let mut repeat: u_int = 0;
        let mut flags: uint64_t = 0;
        let mut prefix_delay: uint64_t = 0;
        let mut fs = cmd_find_state::default();
        let mut key0: key_code = 0;
        let mut prefix: key_code = 0;
        let mut prefix2: key_code = 0;
        if !(s.is_null() || (*c).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0) {
            wl = session_get_curw(s);
            (*c).last_activity_time = (*c).activity_time;
            if gettimeofday(&raw mut (*c).activity_time, ::core::ptr::null_mut())
                != 0 as ::core::ffi::c_int
            {
                fatal(c"gettimeofday failed".as_ptr(), fmt_args![]);
            }
            session_update_activity(s, &raw mut (*c).activity_time);
            (*m).valid = 0 as ::core::ffi::c_int;
            if key == KEYC_MOUSE as ::core::ffi::c_ulong as key_code
                || key == KEYC_DOUBLECLICK as ::core::ffi::c_ulong as key_code
            {
                if (*c).flags & CLIENT_READONLY as uint64_t != 0 {
                    current_block = 16906968430444679536;
                } else {
                    key = server_client_check_mouse(c, event);
                    if key == KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code {
                        current_block = 16906968430444679536;
                    } else {
                        (*m).valid = 1 as ::core::ffi::c_int;
                        (*m).key = key;
                        if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                            == KEYC_DRAGGING as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                        {
                            (*c).tty
                                .mouse_drag_update
                                .expect("non-null function pointer")(
                                c, m
                            );
                            current_block = 16906968430444679536;
                        } else {
                            (*event).key = key;
                            current_block = 5948590327928692120;
                        }
                    }
                }
            } else {
                current_block = 5948590327928692120;
            }
            match current_block {
                16906968430444679536 => {}
                _ => {
                    if !(key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                        == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                        || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                            >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int
                                as ::core::ffi::c_ulonglong)
                                << 32 as ::core::ffi::c_int
                            && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                                <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int
                                    as ::core::ffi::c_ulonglong)
                                    << 32 as ::core::ffi::c_int)
                        || cmd_find_from_mouse(&mut fs, m, 0 as ::core::ffi::c_int)
                            != 0 as ::core::ffi::c_int
                    {
                        cmd_find_from_client(&mut fs, c, 0 as ::core::ffi::c_int);
                    }
                    wp = fs.pane();
                    if (key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                        == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                        || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                            >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int
                                as ::core::ffi::c_ulonglong)
                                << 32 as ::core::ffi::c_int
                            && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                                <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int
                                    as ::core::ffi::c_ulonglong)
                                    << 32 as ::core::ffi::c_int)
                        && options_get_number(session_options(s), c"mouse".as_ptr()) == 0
                    {
                        current_block = 4899622246340545966;
                    } else {
                        if server_client_is_bracket_paste(c, key) != 0 {
                            current_block = 875716535476481131;
                        } else if !(key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                            == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                            || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                                >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int
                                    as ::core::ffi::c_ulonglong)
                                    << 32 as ::core::ffi::c_int
                                && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                                    <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int
                                        as ::core::ffi::c_ulonglong)
                                        << 32 as ::core::ffi::c_int)
                            && key != KEYC_FOCUS_IN as ::core::ffi::c_ulong as key_code
                            && key != KEYC_FOCUS_OUT as ::core::ffi::c_ulong as key_code
                            && !(key as ::core::ffi::c_ulonglong) & KEYC_SENT != 0
                            && server_client_is_assume_paste(c) != 0
                        {
                            current_block = 875716535476481131;
                        } else {
                            if server_client_is_default_key_table(c, (*c).keytable()) != 0
                                && !wp.is_null()
                                && {
                                    wme = window_pane_current_mode(wp);
                                    !wme.is_null()
                                }
                                && let Some(name) = (*wme).mode().key_table(wme)
                            {
                                table_ref = key_bindings_get_table_ref(
                                    name.as_ptr(),
                                    1 as ::core::ffi::c_int,
                                );
                                table = table_ref
                                    .as_ref()
                                    .map(KeyTableRef::as_ptr)
                                    .unwrap_or(::core::ptr::null_mut::<key_table>());
                            } else {
                                table_ref = None;
                                table = (*c).keytable();
                            }
                            first = table;
                            '_table_changed: loop {
                                prefix = options_get_number(session_options(s), c"prefix".as_ptr())
                                    as key_code;
                                prefix2 =
                                    options_get_number(session_options(s), c"prefix2".as_ptr())
                                        as key_code;
                                key0 = (key as ::core::ffi::c_ulonglong
                                    & (KEYC_MASK_KEY | KEYC_MASK_MODIFIERS))
                                    as key_code;
                                if (key0
                                    == prefix as ::core::ffi::c_ulonglong
                                        & (KEYC_MASK_KEY | KEYC_MASK_MODIFIERS)
                                    || key0
                                        == prefix2 as ::core::ffi::c_ulonglong
                                            & (KEYC_MASK_KEY | KEYC_MASK_MODIFIERS))
                                    && key_table_name(table) != c"prefix"
                                {
                                    server_client_set_key_table(c, c"prefix".as_ptr());
                                    server_status_client(c);
                                    current_block = 16906968430444679536;
                                    break;
                                } else {
                                    flags = (*c).flags;
                                    loop {
                                        if wp.is_null() {
                                            log_debug(
                                                c"key table %s (no pane)".as_ptr(),
                                                fmt_args![key_table_name(table)],
                                            );
                                        } else {
                                            log_debug(
                                                c"key table %s (pane %%%u)".as_ptr(),
                                                fmt_args![key_table_name(table), (*wp).id],
                                            );
                                        }
                                        if (*c).flags & CLIENT_REPEAT as uint64_t != 0 {
                                            log_debug(c"currently repeating".as_ptr(), fmt_args![]);
                                        }
                                        bd = key_bindings_get(table, key0);
                                        prefix_delay = options_get_number(
                                            global_options,
                                            c"prefix-timeout".as_ptr(),
                                        )
                                            as uint64_t;
                                        if prefix_delay > 0 as uint64_t
                                            && key_table_name(table) == c"prefix"
                                            && server_client_key_table_activity_diff(c)
                                                > prefix_delay
                                        {
                                            if !bd.is_null()
                                                && (*c).flags & CLIENT_REPEAT as uint64_t != 0
                                                && key_binding_flags(bd) & KEY_BINDING_REPEAT != 0
                                            {
                                                log_debug(
                                                    c"prefix timeout ignored, repeat is active"
                                                        .as_ptr(),
                                                    fmt_args![],
                                                );
                                            } else {
                                                log_debug(
                                                    c"prefix timeout exceeded".as_ptr(),
                                                    fmt_args![],
                                                );
                                                server_client_set_key_table(
                                                    c,
                                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                                );
                                                table = (*c).keytable();
                                                first = table;
                                                server_status_client(c);
                                                continue '_table_changed;
                                            }
                                        }
                                        if !bd.is_null() {
                                            if (*c).flags & CLIENT_REPEAT as uint64_t != 0
                                                && !key_binding_flags(bd) & KEY_BINDING_REPEAT != 0
                                            {
                                                current_block = 6717214610478484138;
                                                break;
                                            } else {
                                                current_block = 14220266465818359136;
                                                break;
                                            }
                                        } else if key0
                                            != KEYC_ANY as ::core::ffi::c_ulong as key_code
                                        {
                                            key0 = KEYC_ANY as ::core::ffi::c_ulong as key_code;
                                        } else {
                                            if key
                                                == KEYC_MOUSEMOVE_PANE as ::core::ffi::c_ulong
                                                    as key_code
                                                || key
                                                    == KEYC_MOUSEMOVE_STATUS as ::core::ffi::c_ulong
                                                        as key_code
                                                || key
                                                    == KEYC_MOUSEMOVE_STATUS_LEFT
                                                        as ::core::ffi::c_ulong
                                                        as key_code
                                                || key
                                                    == KEYC_MOUSEMOVE_STATUS_RIGHT
                                                        as ::core::ffi::c_ulong
                                                        as key_code
                                                || key
                                                    == KEYC_MOUSEMOVE_STATUS_DEFAULT
                                                        as ::core::ffi::c_ulong
                                                        as key_code
                                                || key
                                                    == KEYC_MOUSEMOVE_BORDER as ::core::ffi::c_ulong
                                                        as key_code
                                            {
                                                current_block = 4899622246340545966;
                                                break '_table_changed;
                                            }
                                            log_debug(
                                                c"not found in key table %s".as_ptr(),
                                                fmt_args![key_table_name(table)],
                                            );
                                            if server_client_is_default_key_table(c, table) == 0
                                                || (*c).flags & CLIENT_REPEAT as uint64_t != 0
                                            {
                                                current_block = 981995395831942902;
                                                break;
                                            } else {
                                                current_block = 13853033528615664019;
                                                break;
                                            }
                                        }
                                    }
                                    match current_block {
                                        13853033528615664019 => {
                                            if first != table
                                                && !flags & CLIENT_REPEAT as uint64_t != 0
                                            {
                                                current_block = 7178192492338286402;
                                                break;
                                            } else {
                                                current_block = 4899622246340545966;
                                                break;
                                            }
                                        }
                                        14220266465818359136 => {
                                            log_debug(
                                                c"found in key table %s".as_ptr(),
                                                fmt_args![key_table_name(table)],
                                            );
                                            repeat = server_client_repeat_time(c, bd);
                                            if repeat != 0 as u_int {
                                                (*c).flags |= CLIENT_REPEAT as uint64_t;
                                                (*c).last_key = key_binding_key(bd);
                                                tv.tv_sec =
                                                    repeat.wrapping_div(1000 as u_int) as __time_t;
                                                tv.tv_usec = (repeat.wrapping_rem(1000 as u_int)
                                                    as ::core::ffi::c_long
                                                    * 1000 as ::core::ffi::c_long)
                                                    as __suseconds_t;
                                                (*c).repeat_timer.disarm();
                                                (*c).repeat_timer.arm(tv);
                                            } else {
                                                (*c).flags &= !CLIENT_REPEAT as uint64_t;
                                                server_client_set_key_table(
                                                    c,
                                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                                );
                                            }
                                            server_status_client(c);
                                            key_bindings_dispatch(bd, item, c, event, &raw mut fs);
                                            current_block = 16906968430444679536;
                                            break;
                                        }
                                        981995395831942902 => {
                                            log_debug(
                                                c"trying in root table".as_ptr(),
                                                fmt_args![],
                                            );
                                            server_client_set_key_table(
                                                c,
                                                ::core::ptr::null::<::core::ffi::c_char>(),
                                            );
                                            table = (*c).keytable();
                                            if (*c).flags & CLIENT_REPEAT as uint64_t != 0 {
                                                first = table;
                                            }
                                            (*c).flags &= !CLIENT_REPEAT as uint64_t;
                                            server_status_client(c);
                                        }
                                        _ => {
                                            log_debug(
                                                c"found in key table %s (not repeating)".as_ptr(),
                                                fmt_args![key_table_name(table)],
                                            );
                                            server_client_set_key_table(
                                                c,
                                                ::core::ptr::null::<::core::ffi::c_char>(),
                                            );
                                            table = (*c).keytable();
                                            first = table;
                                            (*c).flags &= !CLIENT_REPEAT as uint64_t;
                                            server_status_client(c);
                                        }
                                    }
                                }
                            }
                            match current_block {
                                16906968430444679536 => {}
                                4899622246340545966 => {}
                                _ => {
                                    server_client_set_key_table(
                                        c,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                    );
                                    server_status_client(c);
                                    current_block = 16906968430444679536;
                                }
                            }
                        }
                        match current_block {
                            16906968430444679536 => {}
                            4899622246340545966 => {}
                            _ => {
                                if (*c).flags & CLIENT_READONLY as uint64_t != 0 {
                                    current_block = 16906968430444679536;
                                } else {
                                    if !(*event).buf.is_empty() {
                                        window_pane_paste(
                                            wp,
                                            key,
                                            (*event).buf.as_mut_ptr() as *mut ::core::ffi::c_char,
                                            (*event).buf.len(),
                                        );
                                    }
                                    key = KEYC_NONE as ::core::ffi::c_ulong as key_code;
                                    current_block = 16906968430444679536;
                                }
                            }
                        }
                    }
                    match current_block {
                        16906968430444679536 => {}
                        _ => {
                            if !wp.is_null()
                                && (*wp).flags & PANE_EXITED != 0
                                && !(key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                                    == KEYC_MOUSE as ::core::ffi::c_ulong
                                        as ::core::ffi::c_ulonglong
                                    || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                                        >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int
                                            as ::core::ffi::c_ulonglong)
                                            << 32 as ::core::ffi::c_int
                                        && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                                            <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int
                                                as ::core::ffi::c_ulonglong)
                                                << 32 as ::core::ffi::c_int)
                                && !(key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                                    == (KEYC_TYPE_FUNCTION as ::core::ffi::c_int
                                        as ::core::ffi::c_ulonglong)
                                        << 32 as ::core::ffi::c_int
                                    && (key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                                        == KEYC_PASTE_START as ::core::ffi::c_ulong
                                            as ::core::ffi::c_ulonglong
                                        || key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                                            == KEYC_PASTE_END as ::core::ffi::c_ulong
                                                as ::core::ffi::c_ulonglong))
                                && options_get_number(
                                    (*wp).options_ptr(),
                                    c"remain-on-exit".as_ptr(),
                                ) == 3 as ::core::ffi::c_longlong
                            {
                                options_set_number(
                                    (*wp).options_ptr(),
                                    c"remain-on-exit".as_ptr(),
                                    0 as ::core::ffi::c_longlong,
                                );
                                server_destroy_pane(wp, 0 as ::core::ffi::c_int);
                            } else if !((*c).flags & CLIENT_READONLY as uint64_t != 0)
                                && !wp.is_null()
                            {
                                window_pane_key(wp, c, s, wl, key, m);
                            }
                        }
                    }
                }
            }
        }
        if !s.is_null() && key != KEYC_FOCUS_OUT as ::core::ffi::c_ulong as key_code {
            server_client_update_latest(c);
        }
        CMD_RETURN_NORMAL
    }
}
pub unsafe fn server_client_handle_key(
    mut c: *mut client,
    mut event: Box<key_event>,
) -> ::core::ffi::c_int {
    unsafe {
        let mut s: *mut session = (*c).session;
        if s.is_null() || (*c).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0 {
            return 0 as ::core::ffi::c_int;
        }
        if event.key == KEYC_REPORT_LIGHT_THEME as ::core::ffi::c_ulong as key_code {
            server_client_report_theme(c, THEME_LIGHT);
            return 0 as ::core::ffi::c_int;
        }
        if event.key == KEYC_REPORT_DARK_THEME as ::core::ffi::c_ulong as key_code {
            server_client_report_theme(c, THEME_DARK);
            return 0 as ::core::ffi::c_int;
        }
        if !(*c).flags & CLIENT_READONLY as uint64_t != 0 {
            if (*c).message_string.is_some() {
                if (*c).message_ignore_keys != 0 {
                    return 0 as ::core::ffi::c_int;
                }
                status_message_clear(c);
            }
            if (*c).overlay().has_key() {
                match (*c).overlay().key(
                    c,
                    (*c).current_overlay_data(),
                    event.as_mut() as *mut key_event,
                ) {
                    0 => return 0 as ::core::ffi::c_int,
                    1 => {
                        server_client_clear_overlay(c);
                        return 0 as ::core::ffi::c_int;
                    }
                    _ => {}
                }
            }
            server_client_clear_overlay(c);
            if (*c).prompt_string.is_some()
                && status_prompt_key(c, event.key) == 0 as ::core::ffi::c_int
            {
                return 0 as ::core::ffi::c_int;
            }
        }
        cmdq_append(
            c,
            cmdq_get_callback1(
                c"server_client_key_callback".as_ptr(),
                Some(server_client_key_callback),
                CmdqCallbackData::KeyEvent(event),
            ),
        );
        1 as ::core::ffi::c_int
    }
}
pub fn server_client_loop() {
    unsafe {
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
        for w_ref in window_refs() {
            server_client_check_window_resize(w_ref.as_ptr());
        }
        for w_ref in window_refs() {
            let w = w_ref.as_ptr();
            wp = window_panes_first(w);
            while !wp.is_null() {
                if (*wp).flags & PANE_STYLECHANGED != 0 {
                    wme = window_pane_current_mode(wp);
                    if !wme.is_null() {
                        (*wme).mode().style_changed(wme);
                    }
                }
                wp = window_panes_next(w, wp);
            }
        }
        for c in client_walk() {
            server_client_check_exit(c);
            if !(*c).session.is_null() && !session_get_curw((*c).session).is_null() {
                server_client_check_modes(c);
                server_client_check_redraw(c);
                server_client_reset_state(c);
            }
        }
        for w_ref in window_refs() {
            let w = w_ref.as_ptr();
            wp = window_panes_first(w);
            while !wp.is_null() {
                if (*wp).fd != -(1 as ::core::ffi::c_int) {
                    server_client_check_pane_resize(wp);
                    server_client_check_pane_buffer(wp);
                }
                (*wp).flags &= !(PANE_REDRAW | PANE_REDRAWSCROLLBAR);
                wp = window_panes_next(w, wp);
            }
            check_window_name(w);
        }
        for w_ref in window_refs() {
            let w = w_ref.as_ptr();
            wp = window_panes_first(w);
            while !wp.is_null() {
                window_pane_send_theme_update(wp);
                wp = window_panes_next(w, wp);
            }
        }
    }
}
unsafe fn server_client_check_window_resize(mut w: *mut window) {
    unsafe {
        let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
        if !(*w).flags & WINDOW_RESIZE != 0 {
            return;
        }
        wl = winlinks_into(w)
            .find(|&wl| {
                session_attached((*wl).session()) != 0 as u_int
                    && session_get_curw((*wl).session()) == wl
            })
            .unwrap_or(::core::ptr::null_mut::<winlink>());
        if wl.is_null() {
            return;
        }
        log_debug(
            c"%s: resizing window @%u".as_ptr(),
            fmt_args![c"server_client_check_window_resize".as_ptr(), (*w).id],
        );
        resize_window(
            w,
            (*w).new_sx,
            (*w).new_sy,
            (*w).new_xpixel as ::core::ffi::c_int,
            (*w).new_ypixel as ::core::ffi::c_int,
        );
    }
}
unsafe fn server_client_resize_timer(wp: *mut window_pane) {
    unsafe {
        log_debug(
            c"%s: %%%u resize timer expired".as_ptr(),
            fmt_args![c"server_client_resize_timer".as_ptr(), (*wp).id],
        );
        (*wp).resize_timer.disarm();
    }
}
unsafe fn server_client_check_pane_resize(mut wp: *mut window_pane) {
    unsafe {
        let mut tv = timeval::from_usecs(250000 as __suseconds_t);
        if (*wp).resize_queue.is_empty() {
            return;
        }
        if !(*wp).resize_timer.is_set() {
            (*wp)
                .resize_timer
                .set_callback(move || server_client_resize_timer(wp));
        }
        if (*wp).resize_timer.is_armed() {
            return;
        }
        log_debug(
            c"%s: %%%u needs to be resized".as_ptr(),
            fmt_args![c"server_client_check_pane_resize".as_ptr(), (*wp).id],
        );
        for r in &(*wp).resize_queue {
            log_debug(
                c"queued resize: %ux%u -> %ux%u".as_ptr(),
                fmt_args![r.osx, r.osy, r.sx, r.sy],
            );
        }
        let first = *(*wp).resize_queue.front().unwrap();
        let last = *(*wp).resize_queue.back().unwrap();
        if (*wp).resize_queue.len() == 1 {
            window_pane_send_resize(wp, first.sx, first.sy);
            (*wp).resize_queue.pop_front();
        } else if last.sx != first.osx || last.sy != first.osy {
            window_pane_send_resize(wp, last.sx, last.sy);
            (*wp).resize_queue.clear();
        } else {
            let r = *(*wp)
                .resize_queue
                .get((*wp).resize_queue.len() - 2)
                .unwrap();
            window_pane_send_resize(wp, r.sx, r.sy);
            let last_elem = (*wp).resize_queue.pop_back().unwrap();
            (*wp).resize_queue.clear();
            (*wp).resize_queue.push_back(last_elem);
            tv.tv_usec = 10000 as __suseconds_t;
        }
        (*wp).resize_timer.arm(tv);
    }
}
unsafe fn server_client_check_pane_buffer(mut wp: *mut window_pane) {
    unsafe {
        let mut minimum: size_t = 0;
        let mut wpo: *mut window_pane_offset = ::core::ptr::null_mut::<window_pane_offset>();
        let mut off: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        let mut flag: ::core::ffi::c_int = 0;
        let mut attached_clients: u_int = 0 as u_int;
        let mut new_size: size_t = 0;
        minimum = (*wp).offset.used;
        if (*wp).pipe_fd != -(1 as ::core::ffi::c_int) && (*wp).pipe_offset.used < minimum {
            minimum = (*wp).pipe_offset.used;
        }
        for c in client_walk() {
            if !(*c).session.is_null() {
                attached_clients = attached_clients.wrapping_add(1);
                if !(*c).flags & CLIENT_CONTROL as uint64_t != 0 {
                    off = 0 as ::core::ffi::c_int;
                } else {
                    (wpo, flag) = control_pane_offset(c, wp);
                    if wpo.is_null() {
                        if flag == 0 {
                            off = 0 as ::core::ffi::c_int;
                        }
                    } else {
                        if flag == 0 {
                            off = 0 as ::core::ffi::c_int;
                        }
                        new_size = window_pane_get_new_data(wp, wpo).1;
                        log_debug(
                            c"%s: %s has %zu bytes used and %zu left for %%%u".as_ptr(),
                            fmt_args![
                                c"server_client_check_pane_buffer".as_ptr(),
                                (*c).name.as_deref(),
                                (*wpo).used.wrapping_sub((*wp).base_offset),
                                new_size,
                                (*wp).id
                            ],
                        );
                        if (*wpo).used < minimum {
                            minimum = (*wpo).used;
                        }
                    }
                }
            }
        }
        if attached_clients == 0 as u_int {
            off = 0 as ::core::ffi::c_int;
        }
        minimum = minimum.wrapping_sub((*wp).base_offset);
        if !(minimum == 0 as size_t) {
            log_debug(
                c"%s: %%%u has %zu minimum (of %zu) bytes used".as_ptr(),
                fmt_args![
                    c"server_client_check_pane_buffer".as_ptr(),
                    (*wp).id,
                    minimum,
                    (*wp).event.input_len()
                ],
            );
            (*wp).event.with_input(|buffer| buffer.drain(minimum));
            if (*wp).base_offset > (SIZE_MAX as size_t).wrapping_sub(minimum) {
                log_debug(
                    c"%s: %%%u base offset has wrapped".as_ptr(),
                    fmt_args![c"server_client_check_pane_buffer".as_ptr(), (*wp).id],
                );
                (*wp).offset.used = (*wp).offset.used.wrapping_sub((*wp).base_offset);
                if (*wp).pipe_fd != -(1 as ::core::ffi::c_int) {
                    (*wp).pipe_offset.used = (*wp).pipe_offset.used.wrapping_sub((*wp).base_offset);
                }
                for c in client_walk() {
                    if !((*c).session.is_null() || !(*c).flags & CLIENT_CONTROL as uint64_t != 0) {
                        (wpo, flag) = control_pane_offset(c, wp);
                        if !wpo.is_null() && flag == 0 {
                            (*wpo).used = (*wpo).used.wrapping_sub((*wp).base_offset);
                        }
                    }
                }
                (*wp).base_offset = minimum;
            } else {
                (*wp).base_offset = (*wp).base_offset.wrapping_add(minimum);
            }
        }
        log_debug(
            c"%s: pane %%%u is %s".as_ptr(),
            fmt_args![
                c"server_client_check_pane_buffer".as_ptr(),
                (*wp).id,
                if off != 0 {
                    c"off".as_ptr()
                } else {
                    c"on".as_ptr()
                }
            ],
        );
        if off != 0 {
            (*wp).event.disable(Interest::Read);
        } else {
            (*wp).event.enable(Interest::Read);
        };
    }
}
unsafe fn server_client_reset_state(mut c: *mut client) {
    unsafe {
        let mut tty: *mut tty = &raw mut (*c).tty;
        let mut w: *mut window = (*session_get_curw((*c).session)).window();
        let mut wp: *mut window_pane = server_client_get_pane(c);
        let mut loop_0: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut s: *mut screen = ::core::ptr::null_mut::<screen>();
        let mut oo: *mut options = session_options((*c).session);
        let mut mode: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut cursor: ::core::ffi::c_int = 0;
        let mut flags: ::core::ffi::c_int = 0;
        let mut cx: u_int = 0 as u_int;
        let mut cy: u_int = 0 as u_int;
        let mut ox: u_int = 0;
        let mut oy: u_int = 0;
        let mut sx: u_int = 0;
        let mut sy: u_int = 0;
        let mut n: u_int = 0;
        let mut ranges = visible_ranges::default();
        if (*c).flags & (CLIENT_CONTROL | CLIENT_SUSPENDED) as uint64_t != 0 {
            return;
        }
        flags = (*tty).flags & TTY_BLOCK;
        (*tty).flags &= !TTY_BLOCK;
        if (*c).overlay().is_some() {
            if (*c).overlay().has_mode() {
                (s, cx, cy) = (*c).overlay().mode(c, (*c).current_overlay_data());
            }
        } else if !wp.is_null() && (*c).prompt_string.is_none() {
            s = (*wp).screen();
        } else {
            s = (*c).status.active();
        }
        if !s.is_null() {
            mode = (*s).mode;
        }
        if log_get_level() != 0 as ::core::ffi::c_int {
            log_debug(
                c"%s: client %s mode %s".as_ptr(),
                fmt_args![
                    c"server_client_reset_state".as_ptr(),
                    (*c).name.as_deref(),
                    screen_mode_to_string(mode).as_c_str()
                ],
            );
        }
        tty_region_off(tty);
        tty_margin_off(tty);
        if (*c).prompt_string.is_some() {
            n = options_get_number(oo, c"status-position".as_ptr()) as u_int;
            if n == 0 as u_int {
                cy = status_prompt_line_at(c);
            } else {
                n = status_line_size(c).wrapping_sub(status_prompt_line_at(c));
                if n <= (*tty).sy {
                    cy = (*tty).sy.wrapping_sub(n);
                } else {
                    cy = (*tty).sy.wrapping_sub(1 as u_int);
                }
            }
            cx = (*c).prompt_cursor as u_int;
        } else if !wp.is_null() && (*c).overlay().is_none() {
            cursor = 0 as ::core::ffi::c_int;
            {
                let (window_bigger, off_x, off_y, off_sx, off_sy) = tty_window_offset(tty);
                (ox, oy, sx, sy) = (off_x, off_y, off_sx, off_sy);
                window_bigger
            };
            if (*wp).xoff + (*s).cx as ::core::ffi::c_int >= ox as ::core::ffi::c_int
                && (*wp).xoff + (*s).cx as ::core::ffi::c_int
                    <= ox as ::core::ffi::c_int + sx as ::core::ffi::c_int
                && (*wp).yoff + (*s).cy as ::core::ffi::c_int >= oy as ::core::ffi::c_int
                && (*wp).yoff + (*s).cy as ::core::ffi::c_int
                    <= oy as ::core::ffi::c_int + sy as ::core::ffi::c_int
            {
                cursor = 1 as ::core::ffi::c_int;
                cx = ((*wp).xoff + (*s).cx as ::core::ffi::c_int - ox as ::core::ffi::c_int)
                    as u_int;
                cy = ((*wp).yoff + (*s).cy as ::core::ffi::c_int - oy as ::core::ffi::c_int)
                    as u_int;
                screen_redraw_get_visible_ranges(
                    wp,
                    cx as ::core::ffi::c_int,
                    cy as ::core::ffi::c_int,
                    1 as u_int,
                    &mut ranges,
                );
                if !screen_redraw_is_visible(Some(&ranges), cx) {
                    cursor = 0 as ::core::ffi::c_int;
                }
                if status_at_line(c) == 0 as ::core::ffi::c_int {
                    cy = cy.wrapping_add(status_line_size(c));
                }
            }
            if cursor == 0 {
                mode &= !MODE_CURSOR;
            }
        } else if !(*c).overlay().has_mode() || s.is_null() {
            mode &= !MODE_CURSOR;
        }
        log_debug(
            c"%s: cursor to %u,%u".as_ptr(),
            fmt_args![c"server_client_reset_state".as_ptr(), cx, cy],
        );
        tty_cursor(tty, cx, cy);
        if options_get_number(oo, c"mouse".as_ptr()) != 0 {
            if (*c).overlay().is_none() {
                mode &= !ALL_MOUSE_MODES;
                loop_0 = window_panes_first(w);
                while !loop_0.is_null() {
                    if (*(*loop_0).screen()).mode & MODE_MOUSE_ALL != 0 {
                        mode |= MODE_MOUSE_ALL;
                    }
                    loop_0 = window_panes_next(w, loop_0);
                }
            }
            if options_get_number(oo, c"focus-follows-mouse".as_ptr()) != 0 {
                mode |= MODE_MOUSE_ALL;
            } else if !mode & MODE_MOUSE_ALL != 0 {
                mode |= MODE_MOUSE_BUTTON;
            }
        }
        if (*c).overlay().is_none() && (*c).prompt_string.is_some() {
            mode &= !MODE_BRACKETPASTE;
        }
        tty_update_mode(tty, mode, s);
        tty_reset(tty);
        tty_sync_end(tty);
        (*tty).flags |= flags;
    }
}
unsafe fn server_client_repeat_timer(c: *mut client) {
    unsafe {
        if (*c).flags & CLIENT_REPEAT as uint64_t != 0 {
            server_client_set_key_table(c, ::core::ptr::null::<::core::ffi::c_char>());
            (*c).flags &= !CLIENT_REPEAT as uint64_t;
            server_status_client(c);
        }
    }
}
unsafe fn server_client_click_timer(c: *mut client) {
    unsafe {
        log_debug(c"click timer expired".as_ptr(), fmt_args![]);
        if (*c).flags & CLIENT_TRIPLECLICK as uint64_t != 0 {
            let event = Box::new(key_event {
                key: KEYC_DOUBLECLICK as ::core::ffi::c_ulong as key_code,
                m: (*c).click_event,
                buf: Vec::new(),
            });
            server_client_handle_key(c, event);
        }
        (*c).flags &= !(CLIENT_DOUBLECLICK | CLIENT_TRIPLECLICK) as uint64_t;
    }
}
unsafe fn server_client_check_exit(mut c: *mut client) {
    unsafe {
        let mut name: *const ::core::ffi::c_char = cstr_ptr(&(*c).exit_session);
        if (*c).flags & (CLIENT_DEAD | CLIENT_EXITED) as uint64_t != 0 {
            return;
        }
        if !(*c).flags & CLIENT_EXIT as uint64_t != 0 {
            return;
        }
        if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            control_discard(c);
            if control_all_done(c) == 0 {
                return;
            }
        }
        for cf in (*c).files.values() {
            if (*cf.as_ptr()).buffer.as_ref().len() != 0 as size_t {
                return;
            }
        }
        (*c).flags |= CLIENT_EXITED as uint64_t;
        match (*c).exit_type {
            CLIENT_EXIT_RETURN => {
                let mut data = (*c).retval.to_ne_bytes().to_vec();
                if let Some(message) = &(*c).exit_message {
                    data.extend_from_slice(message.to_bytes_with_nul());
                }
                proc_send(
                    peer_ptr(&(*c).peer),
                    MSG_EXIT,
                    -(1 as ::core::ffi::c_int),
                    data.as_ptr(),
                    data.len() as size_t,
                );
            }
            CLIENT_EXIT_SHUTDOWN => {
                proc_send(
                    peer_ptr(&(*c).peer),
                    MSG_SHUTDOWN,
                    -(1 as ::core::ffi::c_int),
                    ::core::ptr::null::<u8>(),
                    0 as size_t,
                );
            }
            CLIENT_EXIT_DETACH => {
                proc_send(
                    peer_ptr(&(*c).peer),
                    (*c).exit_msgtype,
                    -(1 as ::core::ffi::c_int),
                    name as *const u8,
                    strlen(name).wrapping_add(1 as size_t),
                );
            }
            _ => {}
        }
        (*c).exit_session = None;
        (*c).exit_message = None;
    }
}
unsafe fn server_client_redraw_timer() {
    unsafe {
        log_debug(c"redraw timer fired".as_ptr(), fmt_args![]);
    }
}
unsafe fn server_client_check_modes(mut c: *mut client) {
    unsafe {
        let mut w: *mut window = (*session_get_curw((*c).session)).window();
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
        if (*c).flags & (CLIENT_CONTROL | CLIENT_SUSPENDED) as uint64_t != 0 {
            return;
        }
        if !(*c).flags & CLIENT_REDRAWSTATUS as uint64_t != 0 {
            return;
        }
        wp = window_panes_first(w);
        while !wp.is_null() {
            wme = window_pane_current_mode(wp);
            if !wme.is_null() {
                (*wme).mode().update(wme);
            }
            wp = window_panes_next(w, wp);
        }
    }
}
unsafe fn server_client_check_redraw(mut c: *mut client) {
    unsafe {
        let mut s: *mut session = (*c).session;
        let mut tty: *mut tty = &raw mut (*c).tty;
        let mut w: *mut window = (*session_get_curw((*c).session)).window();
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut needed: ::core::ffi::c_int = 0;
        let mut tty_flags: ::core::ffi::c_int = 0;
        let mut mode: ::core::ffi::c_int = (*tty).mode;
        let mut client_flags: uint64_t = 0 as uint64_t;
        let mut redraw_pane: ::core::ffi::c_int = 0;
        let mut redraw_scrollbar_only: ::core::ffi::c_int = 0;
        let mut bit: u_int = 0 as u_int;
        let mut tv = timeval::from_usecs(1000 as __suseconds_t);
        static mut ev: TimerHandle = TimerHandle::ZERO;
        let mut left: size_t = 0;
        if (*c).flags & (CLIENT_CONTROL | CLIENT_SUSPENDED) as uint64_t != 0 {
            return;
        }
        if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_ALLREDRAWFLAGS != 0 {
            log_debug(
                c"%s: redraw%s%s%s%s%s%s".as_ptr(),
                fmt_args![
                    (*c).name.as_deref(),
                    if (*c).flags & CLIENT_REDRAWWINDOW as uint64_t != 0 {
                        c" window".as_ptr()
                    } else {
                        c"".as_ptr()
                    },
                    if (*c).flags & CLIENT_REDRAWSTATUS as uint64_t != 0 {
                        c" status".as_ptr()
                    } else {
                        c"".as_ptr()
                    },
                    if (*c).flags & CLIENT_REDRAWBORDERS as uint64_t != 0 {
                        c" borders".as_ptr()
                    } else {
                        c"".as_ptr()
                    },
                    if (*c).flags & CLIENT_REDRAWOVERLAY as uint64_t != 0 {
                        c" overlay".as_ptr()
                    } else {
                        c"".as_ptr()
                    },
                    if (*c).flags & CLIENT_REDRAWPANES as uint64_t != 0 {
                        c" panes".as_ptr()
                    } else {
                        c"".as_ptr()
                    },
                    if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_REDRAWSCROLLBARS != 0 {
                        c" scrollbars".as_ptr()
                    } else {
                        c"".as_ptr()
                    }
                ],
            );
        }
        needed = 0 as ::core::ffi::c_int;
        if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_ALLREDRAWFLAGS != 0 {
            needed = 1 as ::core::ffi::c_int;
        } else {
            wp = window_panes_first(w);
            while !wp.is_null() {
                if (*wp).flags & PANE_REDRAW != 0 {
                    needed = 1 as ::core::ffi::c_int;
                    client_flags |= CLIENT_REDRAWPANES as uint64_t;
                    break;
                } else {
                    if (*wp).flags & PANE_REDRAWSCROLLBAR != 0 {
                        needed = 1 as ::core::ffi::c_int;
                        client_flags = (client_flags as ::core::ffi::c_ulonglong
                            | CLIENT_REDRAWSCROLLBARS)
                            as uint64_t;
                    }
                    wp = window_panes_next(w, wp);
                }
            }
        }
        if needed != 0 && {
            left = (*tty).out.as_ref().unwrap().len();
            left != 0 as size_t
        } {
            log_debug(
                c"%s: redraw deferred (%zu left)".as_ptr(),
                fmt_args![(*c).name.as_deref(), left],
            );
            if !ev.is_set() {
                ev.set_callback(move || {
                    server_client_redraw_timer();
                });
            }
            if !ev.is_armed() {
                log_debug(c"redraw timer started".as_ptr(), fmt_args![]);
                ev.arm(tv);
            }
            if !(*c).flags & CLIENT_REDRAWWINDOW as uint64_t != 0 {
                wp = window_panes_first(w);
                while !wp.is_null() {
                    if (*wp).flags & 0x1 as ::core::ffi::c_int != 0 {
                        log_debug(
                            c"%s: pane %%%u needs redraw".as_ptr(),
                            fmt_args![(*c).name.as_deref(), (*wp).id],
                        );
                        (*c).redraw_panes |= ((1 as ::core::ffi::c_int) << bit) as uint64_t;
                    } else if (*wp).flags & PANE_REDRAWSCROLLBAR != 0 {
                        log_debug(
                            c"%s: pane %%%u scrollbar needs redraw".as_ptr(),
                            fmt_args![(*c).name.as_deref(), (*wp).id],
                        );
                        (*c).redraw_scrollbars |= ((1 as ::core::ffi::c_int) << bit) as uint64_t;
                    }
                    bit = bit.wrapping_add(1);
                    if bit == 64 as u_int {
                        client_flags = (client_flags as ::core::ffi::c_ulonglong
                            & !(CLIENT_REDRAWPANES as ::core::ffi::c_ulonglong
                                | CLIENT_REDRAWSCROLLBARS))
                            as uint64_t;
                        client_flags |= CLIENT_REDRAWWINDOW as uint64_t;
                        break;
                    } else {
                        wp = window_panes_next(w, wp);
                    }
                }
                if (*c).redraw_panes != 0 as uint64_t {
                    (*c).flags |= CLIENT_REDRAWPANES as uint64_t;
                }
                if (*c).redraw_scrollbars != 0 as uint64_t {
                    (*c).flags = ((*c).flags as ::core::ffi::c_ulonglong | CLIENT_REDRAWSCROLLBARS)
                        as uint64_t;
                }
            }
            (*c).flags |= client_flags;
            return;
        } else if needed != 0 {
            log_debug(
                c"%s: redraw needed".as_ptr(),
                fmt_args![(*c).name.as_deref()],
            );
        }
        tty_flags = (*tty).flags & (TTY_BLOCK | TTY_FREEZE | TTY_NOCURSOR);
        (*tty).flags = (*tty).flags & !(TTY_BLOCK | TTY_FREEZE) | TTY_NOCURSOR;
        if !(*c).flags & CLIENT_REDRAWWINDOW as uint64_t != 0 {
            wp = window_panes_first(w);
            while !wp.is_null() {
                redraw_pane = 0 as ::core::ffi::c_int;
                redraw_scrollbar_only = 0 as ::core::ffi::c_int;
                if (*wp).flags & PANE_REDRAW != 0 {
                    redraw_pane = 1 as ::core::ffi::c_int;
                } else if (*c).flags & CLIENT_REDRAWPANES as uint64_t != 0 {
                    if (*c).redraw_panes & ((1 as ::core::ffi::c_int) << bit) as uint64_t != 0 {
                        redraw_pane = 1 as ::core::ffi::c_int;
                    }
                } else if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_REDRAWSCROLLBARS != 0
                    && (*c).redraw_scrollbars & ((1 as ::core::ffi::c_int) << bit) as uint64_t != 0
                {
                    redraw_scrollbar_only = 1 as ::core::ffi::c_int;
                }
                bit = bit.wrapping_add(1);
                if !(redraw_pane == 0 && redraw_scrollbar_only == 0) {
                    if redraw_scrollbar_only != 0 {
                        log_debug(
                            c"%s: redrawing (scrollbar only) pane %%%u".as_ptr(),
                            fmt_args![c"server_client_check_redraw".as_ptr(), (*wp).id],
                        );
                    } else {
                        log_debug(
                            c"%s: redrawing pane %%%u".as_ptr(),
                            fmt_args![c"server_client_check_redraw".as_ptr(), (*wp).id],
                        );
                    }
                    screen_redraw_pane(c, wp, redraw_scrollbar_only);
                }
                wp = window_panes_next(w, wp);
            }
            (*c).redraw_panes = 0 as uint64_t;
            (*c).redraw_scrollbars = 0 as uint64_t;
            (*c).flags = ((*c).flags as ::core::ffi::c_ulonglong
                & !(CLIENT_REDRAWPANES as ::core::ffi::c_ulonglong | CLIENT_REDRAWSCROLLBARS))
                as uint64_t;
        }
        if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_ALLREDRAWFLAGS != 0 {
            if options_get_number(session_options(s), c"set-titles".as_ptr()) != 0 {
                server_client_set_title(c);
                server_client_set_path(c);
            }
            server_client_set_progress_bar(c);
            screen_redraw_screen(c);
        }
        (*tty).flags = (*tty).flags & !TTY_NOCURSOR | tty_flags & TTY_NOCURSOR;
        tty_update_mode(tty, mode, ::core::ptr::null_mut::<screen>());
        (*tty).flags = (*tty).flags & !(TTY_BLOCK | TTY_FREEZE | TTY_NOCURSOR) | tty_flags;
        (*c).flags = ((*c).flags as ::core::ffi::c_ulonglong
            & !(CLIENT_ALLREDRAWFLAGS | CLIENT_STATUSFORCE as ::core::ffi::c_ulonglong))
            as uint64_t;
        if needed != 0 {
            (*c).redraw = (*tty).out.as_ref().unwrap().len();
            log_debug(
                c"%s: redraw added %zu bytes".as_ptr(),
                fmt_args![(*c).name.as_deref(), (*c).redraw],
            );
        }
    }
}
unsafe fn server_client_set_title(mut c: *mut client) {
    unsafe {
        let mut s: *mut session = (*c).session;
        let mut template: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        template = options_get_string(session_options(s), c"set-titles-string".as_ptr());
        let mut ft = format_create(
            c,
            ::core::ptr::null_mut::<cmdq_item>(),
            FORMAT_NONE,
            0 as ::core::ffi::c_int,
        );
        format_defaults(
            &mut ft,
            c,
            ::core::ptr::null_mut::<session>(),
            ::core::ptr::null_mut::<winlink>(),
            ::core::ptr::null_mut::<window_pane>(),
        );
        let title = format_expand_time(&mut ft, CStr::from_ptr(template));
        if (*c).title.as_deref() != Some(title.as_c_str()) {
            (*c).title = Some(title.clone());
            tty_set_title(&raw mut (*c).tty, cstr_ptr(&(*c).title));
        }
    }
}
unsafe fn server_client_set_path(mut c: *mut client) {
    unsafe {
        let mut s: *mut session = (*c).session;
        let mut path: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        if session_get_curw(s).is_null()
            || window_get_active((*session_get_curw(s)).window()).is_null()
        {
            return;
        }
        if (*window_get_active((*session_get_curw(s)).window()))
            .base
            .path
            .is_none()
        {
            path = c"".as_ptr();
        } else {
            path = cstr_ptr(
                &(*window_get_active((*session_get_curw(s)).window()))
                    .base
                    .path,
            );
        }
        if (*c).path.as_deref() != Some(CStr::from_ptr(path)) {
            (*c).path = Some(CStr::from_ptr(path).to_owned());
            tty_set_path(&raw mut (*c).tty, cstr_ptr(&(*c).path));
        }
    }
}
unsafe fn server_client_set_progress_bar(mut c: *mut client) {
    unsafe {
        let mut s: *mut session = (*c).session;
        let mut pane_pb: *mut progress_bar = ::core::ptr::null_mut::<progress_bar>();
        if session_get_curw(s).is_null()
            || window_get_active((*session_get_curw(s)).window()).is_null()
        {
            return;
        }
        pane_pb = &raw mut (*window_get_active((*session_get_curw(s)).window()))
            .base
            .progress_bar;
        if (*pane_pb).state as ::core::ffi::c_uint == (*c).progress_bar.state as ::core::ffi::c_uint
            && (*pane_pb).progress == (*c).progress_bar.progress
        {
            return;
        }
        (*c).progress_bar = *pane_pb;
        tty_set_progress_bar(&raw mut (*c).tty, &raw mut (*c).progress_bar);
    }
}
unsafe fn server_client_dispatch(mut imsg: *mut imsg, mut arg: *mut client) {
    unsafe {
        let mut current_block: u64;
        let mut c: *mut client = arg;
        let mut datalen: ssize_t = 0;
        let mut s: *mut session = ::core::ptr::null_mut::<session>();
        if (*c).flags & CLIENT_DEAD as uint64_t != 0 {
            return;
        }
        if imsg.is_null() {
            server_client_lost(c);
            return;
        }
        datalen = ((*imsg).hdr.len as usize).wrapping_sub(IMSG_HEADER_SIZE) as ssize_t;
        match (*imsg).hdr.type_0 {
            MSG_IDENTIFY_CLIENTPID
            | MSG_IDENTIFY_CWD
            | MSG_IDENTIFY_ENVIRON
            | MSG_IDENTIFY_FEATURES
            | MSG_IDENTIFY_FLAGS
            | MSG_IDENTIFY_LONGFLAGS
            | MSG_IDENTIFY_STDIN
            | MSG_IDENTIFY_STDOUT
            | MSG_IDENTIFY_TERM
            | MSG_IDENTIFY_TERMINFO
            | MSG_IDENTIFY_TTYNAME
            | MSG_IDENTIFY_DONE => {
                if server_client_dispatch_identify(c, imsg) != 0 as ::core::ffi::c_int {
                    current_block = 14480369916731894397;
                } else {
                    current_block = 6174974146017752131;
                }
            }
            MSG_COMMAND => {
                if server_client_dispatch_command(c, imsg) != 0 as ::core::ffi::c_int {
                    current_block = 14480369916731894397;
                } else {
                    current_block = 6174974146017752131;
                }
            }
            MSG_RESIZE => {
                if datalen != 0 as ssize_t {
                    current_block = 14480369916731894397;
                } else if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
                    current_block = 6174974146017752131;
                } else {
                    server_client_update_latest(c);
                    tty_resize(&raw mut (*c).tty);
                    tty_repeat_requests(&raw mut (*c).tty, 0 as ::core::ffi::c_int);
                    recalculate_sizes();
                    if !(*c).overlay().has_resize() {
                        server_client_clear_overlay(c);
                    } else {
                        (*c).overlay().resize(c, (*c).current_overlay_data());
                    }
                    server_redraw_client(c);
                    if !(*c).session.is_null() {
                        notify_client(c"client-resized".as_ptr(), c);
                    }
                    current_block = 6174974146017752131;
                }
            }
            MSG_EXITING => {
                if datalen != 0 as ssize_t {
                    current_block = 14480369916731894397;
                } else {
                    server_client_set_session(c, ::core::ptr::null_mut::<session>());
                    recalculate_sizes();
                    tty_close(&raw mut (*c).tty);
                    proc_send(
                        peer_ptr(&(*c).peer),
                        MSG_EXITED,
                        -(1 as ::core::ffi::c_int),
                        ::core::ptr::null::<u8>(),
                        0 as size_t,
                    );
                    current_block = 6174974146017752131;
                }
            }
            MSG_WAKEUP | MSG_UNLOCK => {
                if datalen != 0 as ssize_t {
                    current_block = 14480369916731894397;
                } else if (*c).flags & CLIENT_SUSPENDED as uint64_t == 0 {
                    current_block = 6174974146017752131;
                } else {
                    (*c).flags &= !CLIENT_SUSPENDED as uint64_t;
                    if (*c).fd == -(1 as ::core::ffi::c_int) || (*c).session.is_null() {
                        current_block = 6174974146017752131;
                    } else {
                        s = (*c).session;
                        if gettimeofday(&raw mut (*c).activity_time, ::core::ptr::null_mut())
                            != 0 as ::core::ffi::c_int
                        {
                            fatal(c"gettimeofday failed".as_ptr(), fmt_args![]);
                        }
                        tty_start_tty(&raw mut (*c).tty);
                        server_redraw_client(c);
                        recalculate_sizes();
                        if !s.is_null() {
                            session_update_activity(s, &raw mut (*c).activity_time);
                        }
                        current_block = 6174974146017752131;
                    }
                }
            }
            MSG_SHELL => {
                if datalen != 0 as ssize_t {
                    current_block = 14480369916731894397;
                } else if server_client_dispatch_shell(c) != 0 as ::core::ffi::c_int {
                    current_block = 14480369916731894397;
                } else {
                    current_block = 6174974146017752131;
                }
            }
            MSG_WRITE_READY => {
                if file_write_ready(&raw mut (*c).files, imsg) != 0 as ::core::ffi::c_int {
                    current_block = 14480369916731894397;
                } else {
                    current_block = 6174974146017752131;
                }
            }
            MSG_READ => {
                if file_read_data(&raw mut (*c).files, imsg) != 0 as ::core::ffi::c_int {
                    current_block = 14480369916731894397;
                } else {
                    current_block = 6174974146017752131;
                }
            }
            MSG_READ_DONE
                if file_read_done(&raw mut (*c).files, imsg) != 0 as ::core::ffi::c_int =>
            {
                current_block = 14480369916731894397;
            }
            _ => {
                current_block = 6174974146017752131;
            }
        }
        match current_block {
            6174974146017752131 => (),
            _ => {
                log_debug(
                    c"client %p invalid message type %d".as_ptr(),
                    fmt_args![c, (*imsg).hdr.type_0],
                );
                proc_kill_peer(peer_ptr(&(*c).peer));
            }
        }
    }
}
unsafe fn server_client_read_only(mut item: *mut cmdq_item, _data: CmdqCallbackData) -> cmd_retval {
    unsafe {
        cmdq_error(item, c"client is read-only".as_ptr(), fmt_args![]);
        CMD_RETURN_ERROR
    }
}
unsafe fn server_client_default_command(
    mut item: *mut cmdq_item,
    _data: CmdqCallbackData,
) -> cmd_retval {
    unsafe {
        let mut c: *mut client = cmdq_get_client(&*item);
        let cmdlist =
            options_get_command(global_options, c"default-client-command".as_ptr()).unwrap();
        let queued = if (*c).flags & CLIENT_READONLY as uint64_t != 0
            && cmd_list_all_have(cmdlist.as_ptr(), CMD_READONLY) == 0
        {
            cmdq_get_callback1(
                c"server_client_read_only".as_ptr(),
                Some(server_client_read_only),
                CmdqCallbackData::None,
            )
        } else {
            cmdq_get_command(&cmdlist, None)
        };
        cmdq_insert_after(item, queued);
        CMD_RETURN_NORMAL
    }
}
unsafe fn server_client_command_done(
    mut item: *mut cmdq_item,
    _data: CmdqCallbackData,
) -> cmd_retval {
    unsafe {
        let mut c: *mut client = cmdq_get_client(&*item);
        if !(*c).flags & CLIENT_ATTACHED as uint64_t != 0 {
            (*c).flags |= CLIENT_EXIT as uint64_t;
        } else if !(*c).flags & CLIENT_EXIT as uint64_t != 0 {
            if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
                control_ready(c);
            }
            tty_send_requests(&raw mut (*c).tty);
        }
        CMD_RETURN_NORMAL
    }
}
unsafe fn server_client_dispatch_command(
    mut c: *mut client,
    mut imsg: *mut imsg,
) -> ::core::ffi::c_int {
    unsafe {
        let mut current_block: u64;
        let mut data: msg_command = msg_command { argc: 0 };
        let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut len: size_t = 0;
        let mut cause = None;
        let mut queued = crate::cmd::cmdq_items::new();
        if (*c).flags & CLIENT_EXIT as uint64_t != 0 {
            return 0 as ::core::ffi::c_int;
        }
        if ((*imsg).hdr.len as usize).wrapping_sub(IMSG_HEADER_SIZE)
            < ::core::mem::size_of::<msg_command>() as usize
        {
            return -(1 as ::core::ffi::c_int);
        }
        data = ::core::ptr::read_unaligned((*imsg).data as *const msg_command);
        buf = ((*imsg).data as *mut ::core::ffi::c_char)
            .add(::core::mem::size_of::<msg_command>() as usize);
        len = ((*imsg).hdr.len as usize)
            .wrapping_sub(IMSG_HEADER_SIZE)
            .wrapping_sub(::core::mem::size_of::<msg_command>() as usize) as size_t;
        if len > 0 as size_t
            && *buf.add(len.wrapping_sub(1 as size_t)) as ::core::ffi::c_int != '\0' as i32
        {
            return -(1 as ::core::ffi::c_int);
        }
        match cmd_unpack_argv(buf, len, data.argc) {
            None => {
                cause = Some(c"command too long".to_owned());
            }
            Some(argv) => {
                if argv.is_empty() {
                    queued = cmdq_get_callback1(
                        c"server_client_default_command".as_ptr(),
                        Some(server_client_default_command),
                        CmdqCallbackData::None,
                    );
                    current_block = 13472856163611868459;
                } else {
                    let mut values = args_from_vector(&argv);
                    let mut pr = cmd_parse_from_arguments(
                        values.as_mut_ptr(),
                        argv.len() as u_int,
                        ::core::ptr::null_mut::<cmd_parse_input>(),
                    );
                    match pr.status {
                        CMD_PARSE_ERROR => {
                            cause = pr.error.take();
                            current_block = 3291689279699309301;
                        }
                        _ => {
                            let cmdlist = pr.cmdlist.take().unwrap();
                            if (*c).flags & CLIENT_READONLY as uint64_t != 0
                                && cmd_list_all_have(cmdlist.as_ptr(), CMD_READONLY) == 0
                            {
                                queued = cmdq_get_callback1(
                                    c"server_client_read_only".as_ptr(),
                                    Some(server_client_read_only),
                                    CmdqCallbackData::None,
                                );
                            } else {
                                queued = cmdq_get_command(&cmdlist, None);
                            }
                            current_block = 13472856163611868459;
                        }
                    }
                }
                match current_block {
                    3291689279699309301 => {}
                    _ => {
                        cmdq_append(c, queued);
                        cmdq_append(
                            c,
                            cmdq_get_callback1(
                                c"server_client_command_done".as_ptr(),
                                Some(server_client_command_done),
                                CmdqCallbackData::None,
                            ),
                        );
                        return 0 as ::core::ffi::c_int;
                    }
                }
            }
        }
        if let Some(cause) = cause.as_ref() {
            cmdq_append(c, cmdq_get_error(cause.as_ptr()));
        }
        (*c).flags |= CLIENT_EXIT as uint64_t;
        0 as ::core::ffi::c_int
    }
}
unsafe fn server_client_dispatch_identify(
    mut c: *mut client,
    mut imsg: *mut imsg,
) -> ::core::ffi::c_int {
    unsafe {
        let mut data: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut home: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut datalen: size_t = 0;
        let mut flags: ::core::ffi::c_int = 0;
        let mut feat: ::core::ffi::c_int = 0;
        let mut longflags: uint64_t = 0;
        if (*c).flags & CLIENT_IDENTIFIED as uint64_t != 0 {
            return -(1 as ::core::ffi::c_int);
        }
        data = (*imsg).data as *const ::core::ffi::c_char;
        datalen = ((*imsg).hdr.len as usize).wrapping_sub(IMSG_HEADER_SIZE) as size_t;
        match (*imsg).hdr.type_0 {
            MSG_IDENTIFY_FEATURES => {
                if datalen != ::core::mem::size_of::<::core::ffi::c_int>() as usize {
                    return -(1 as ::core::ffi::c_int);
                }
                feat = ::core::ptr::read_unaligned(data as *const ::core::ffi::c_int);
                (*c).term_features |= feat;
                log_debug(
                    c"client %p IDENTIFY_FEATURES %s".as_ptr(),
                    fmt_args![c, tty_get_features(feat).as_c_str()],
                );
            }
            MSG_IDENTIFY_FLAGS => {
                if datalen != ::core::mem::size_of::<::core::ffi::c_int>() as usize {
                    return -(1 as ::core::ffi::c_int);
                }
                flags = ::core::ptr::read_unaligned(data as *const ::core::ffi::c_int);
                (*c).flags |= flags as uint64_t;
                log_debug(
                    c"client %p IDENTIFY_FLAGS %#x".as_ptr(),
                    fmt_args![c, flags],
                );
            }
            MSG_IDENTIFY_LONGFLAGS => {
                if datalen != ::core::mem::size_of::<uint64_t>() as usize {
                    return -(1 as ::core::ffi::c_int);
                }
                longflags = ::core::ptr::read_unaligned(data as *const uint64_t);
                (*c).flags |= longflags;
                log_debug(
                    c"client %p IDENTIFY_LONGFLAGS %#llx".as_ptr(),
                    fmt_args![c, longflags as ::core::ffi::c_ulonglong],
                );
            }
            MSG_IDENTIFY_TERM => {
                if datalen == 0 as size_t
                    || *data.add(datalen.wrapping_sub(1 as size_t)) as ::core::ffi::c_int
                        != '\0' as i32
                {
                    return -(1 as ::core::ffi::c_int);
                }
                (*c).term_name = Some(CStr::from_ptr(data).to_owned());
                log_debug(c"client %p IDENTIFY_TERM %s".as_ptr(), fmt_args![c, data]);
            }
            MSG_IDENTIFY_TERMINFO => {
                if datalen == 0 as size_t
                    || *data.add(datalen.wrapping_sub(1 as size_t)) as ::core::ffi::c_int
                        != '\0' as i32
                {
                    return -(1 as ::core::ffi::c_int);
                }
                (*c).term_caps.push(CStr::from_ptr(data).to_owned());
                log_debug(
                    c"client %p IDENTIFY_TERMINFO %s".as_ptr(),
                    fmt_args![c, data],
                );
            }
            MSG_IDENTIFY_TTYNAME => {
                if datalen == 0 as size_t
                    || *data.add(datalen.wrapping_sub(1 as size_t)) as ::core::ffi::c_int
                        != '\0' as i32
                {
                    return -(1 as ::core::ffi::c_int);
                }
                (*c).ttyname = Some(CStr::from_ptr(data).to_owned());
                log_debug(
                    c"client %p IDENTIFY_TTYNAME %s".as_ptr(),
                    fmt_args![c, data],
                );
            }
            MSG_IDENTIFY_CWD => {
                if datalen == 0 as size_t
                    || *data.add(datalen.wrapping_sub(1 as size_t)) as ::core::ffi::c_int
                        != '\0' as i32
                {
                    return -(1 as ::core::ffi::c_int);
                }
                if access(data, X_OK) == 0 as ::core::ffi::c_int {
                    (*c).cwd = Some(CStr::from_ptr(data).to_owned());
                } else {
                    home = find_home().map_or(::core::ptr::null(), CStr::as_ptr);
                    if !home.is_null() {
                        (*c).cwd = Some(CStr::from_ptr(home).to_owned());
                    } else {
                        (*c).cwd = Some(c"/".to_owned());
                    }
                }
                log_debug(c"client %p IDENTIFY_CWD %s".as_ptr(), fmt_args![c, data]);
            }
            MSG_IDENTIFY_STDIN => {
                if datalen != 0 as size_t {
                    return -(1 as ::core::ffi::c_int);
                }
                (*c).fd = imsg_get_fd(imsg);
                log_debug(
                    c"client %p IDENTIFY_STDIN %d".as_ptr(),
                    fmt_args![c, (*c).fd],
                );
            }
            MSG_IDENTIFY_STDOUT => {
                if datalen != 0 as size_t {
                    return -(1 as ::core::ffi::c_int);
                }
                (*c).out_fd = imsg_get_fd(imsg);
                log_debug(
                    c"client %p IDENTIFY_STDOUT %d".as_ptr(),
                    fmt_args![c, (*c).out_fd],
                );
            }
            MSG_IDENTIFY_ENVIRON => {
                if datalen == 0 as size_t
                    || *data.add(datalen.wrapping_sub(1 as size_t)) as ::core::ffi::c_int
                        != '\0' as i32
                {
                    return -(1 as ::core::ffi::c_int);
                }
                if !strchr(data, '=' as i32).is_null() {
                    environ_put(environ_ptr(&(*c).environ), data, 0 as ::core::ffi::c_int);
                }
                log_debug(
                    c"client %p IDENTIFY_ENVIRON %s".as_ptr(),
                    fmt_args![c, data],
                );
            }
            MSG_IDENTIFY_CLIENTPID => {
                if datalen != ::core::mem::size_of::<pid_t>() as usize {
                    return -(1 as ::core::ffi::c_int);
                }
                (*c).pid = ::core::ptr::read_unaligned(data as *const pid_t);
                log_debug(
                    c"client %p IDENTIFY_CLIENTPID %ld".as_ptr(),
                    fmt_args![c, (*c).pid as ::core::ffi::c_long],
                );
            }
            _ => {}
        }
        if (*imsg).hdr.type_0 != MSG_IDENTIFY_DONE as ::core::ffi::c_int as uint32_t {
            return 0 as ::core::ffi::c_int;
        }
        (*c).flags |= CLIENT_IDENTIFIED as uint64_t;
        if (*c)
            .term_name
            .as_ref()
            .is_none_or(|name| name.as_bytes().is_empty())
        {
            (*c).term_name = Some(c"unknown".to_owned());
        }
        if (*c)
            .ttyname
            .as_ref()
            .is_some_and(|ttyname| !ttyname.as_bytes().is_empty())
        {
            (*c).name = (*c).ttyname.clone();
        } else {
            (*c).name = Some(xasprintf(
                c"client-%ld".as_ptr(),
                fmt_args![(*c).pid as ::core::ffi::c_long],
            ));
        }
        log_debug(
            c"client %p name is %s".as_ptr(),
            fmt_args![c, (*c).name.as_deref()],
        );
        if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            control_start(c);
        } else if (*c).fd != -(1 as ::core::ffi::c_int) {
            if tty_init(&raw mut (*c).tty, c) != 0 as ::core::ffi::c_int {
                close((*c).fd);
                (*c).fd = -(1 as ::core::ffi::c_int);
            } else {
                tty_resize(&raw mut (*c).tty);
                (*c).flags |= CLIENT_TERMINAL as uint64_t;
            }
            if (*c).out_fd != -(1 as ::core::ffi::c_int) {
                close((*c).out_fd);
            }
            (*c).out_fd = -(1 as ::core::ffi::c_int);
        }
        if (*c).flags as ::core::ffi::c_ulonglong & (CLIENT_BRACKETPASTING | CLIENT_ASSUMEPASTING)
            != 0
            && current_time - (*c).paste_time > CLIENT_PASTE_TIME_LIMIT as time_t
        {
            log_debug(
                c"%s: paste time limit exceeded".as_ptr(),
                fmt_args![(*c).name.as_deref()],
            );
            (*c).flags = ((*c).flags as ::core::ffi::c_ulonglong
                & !(CLIENT_BRACKETPASTING | CLIENT_ASSUMEPASTING))
                as uint64_t;
        }
        if !(*c).flags & CLIENT_EXIT as uint64_t != 0
            && cfg_finished == 0
            && clients.first().is_some_and(|first| first.as_ptr() == c)
        {
            start_cfg();
        }
        0 as ::core::ffi::c_int
    }
}
unsafe fn server_client_dispatch_shell(mut c: *mut client) -> ::core::ffi::c_int {
    unsafe {
        let mut shell: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        shell = options_get_string(global_s_options, c"default-shell".as_ptr());
        if checkshell(shell) == 0 {
            shell = _PATH_BSHELL.as_ptr();
        }
        proc_send(
            peer_ptr(&(*c).peer),
            MSG_SHELL,
            -(1 as ::core::ffi::c_int),
            shell as *const u8,
            strlen(shell).wrapping_add(1 as size_t),
        );
        proc_kill_peer(peer_ptr(&(*c).peer));
        0 as ::core::ffi::c_int
    }
}
pub unsafe fn server_client_get_cwd(
    mut c: *mut client,
    mut s: *mut session,
) -> *const ::core::ffi::c_char {
    unsafe {
        let mut home: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let loading = cfg_client();
        if cfg_finished == 0 && !loading.is_null() {
            return cstr_ptr(&(*loading).cwd);
        }
        if !c.is_null() && (*c).session.is_null() && (*c).cwd.is_some() {
            return cstr_ptr(&(*c).cwd);
        }
        if !s.is_null()
            && let Some(cwd) = session_cwd(s)
        {
            return cwd.as_ptr();
        }
        if !c.is_null()
            && {
                s = (*c).session;
                !s.is_null()
            }
            && let Some(cwd) = session_cwd(s)
        {
            return cwd.as_ptr();
        }
        home = find_home().map_or(::core::ptr::null(), CStr::as_ptr);
        if !home.is_null() {
            return home;
        }
        c"/".as_ptr()
    }
}
unsafe fn server_client_control_flags(
    mut c: *mut client,
    mut next: *const ::core::ffi::c_char,
) -> uint64_t {
    unsafe {
        if strcmp(next, c"pause-after".as_ptr()) == 0 as ::core::ffi::c_int {
            (*c).pause_age = 0 as u_int;
            return 0x100000000 as uint64_t;
        }
        if sscanf(next, c"pause-after=%u".as_ptr(), &raw mut (*c).pause_age)
            == 1 as ::core::ffi::c_int
        {
            (*c).pause_age = (*c).pause_age.wrapping_mul(1000 as u_int);
            return 0x100000000 as uint64_t;
        }
        if strcmp(next, c"no-output".as_ptr()) == 0 as ::core::ffi::c_int {
            return 0x4000000 as uint64_t;
        }
        if strcmp(next, c"wait-exit".as_ptr()) == 0 as ::core::ffi::c_int {
            return 0x200000000 as uint64_t;
        }
        0 as uint64_t
    }
}
pub unsafe fn server_client_set_flags(c: *mut client, flags: *const ::core::ffi::c_char) {
    unsafe {
        let mut flag: uint64_t = 0;
        for next in CStr::from_ptr(flags).to_bytes().split(|&byte| byte == b',') {
            let not = (next.first() == Some(&b'!')) as ::core::ffi::c_int;
            let next = if not != 0 { &next[1..] } else { next };
            let next = CString::new(next).expect("a C string has no interior NUL");
            if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
                flag = server_client_control_flags(c, next.as_ptr());
            } else {
                flag = 0 as uint64_t;
            }
            if strcmp(next.as_ptr(), c"read-only".as_ptr()) == 0 as ::core::ffi::c_int {
                flag = CLIENT_READONLY as uint64_t;
            } else if strcmp(next.as_ptr(), c"ignore-size".as_ptr()) == 0 as ::core::ffi::c_int {
                flag = CLIENT_IGNORESIZE as uint64_t;
            } else if strcmp(next.as_ptr(), c"active-pane".as_ptr()) == 0 as ::core::ffi::c_int {
                flag = CLIENT_ACTIVEPANE as uint64_t;
            } else if strcmp(next.as_ptr(), c"no-detach-on-destroy".as_ptr())
                == 0 as ::core::ffi::c_int
            {
                flag = CLIENT_NO_DETACH_ON_DESTROY as uint64_t;
            }
            if flag == 0 as uint64_t {
                continue;
            }
            log_debug(
                c"client %s set flag %s".as_ptr(),
                fmt_args![(*c).name.as_deref(), next.as_ptr()],
            );
            if not != 0 {
                if (*c).flags & CLIENT_READONLY as uint64_t != 0 {
                    flag &= !CLIENT_READONLY as uint64_t;
                }
                (*c).flags &= !flag;
            } else {
                (*c).flags |= flag;
            }
            if flag == CLIENT_CONTROL_NOOUTPUT as uint64_t {
                control_reset_offsets(c);
            }
        }
        proc_send(
            peer_ptr(&(*c).peer),
            MSG_FLAGS,
            -(1 as ::core::ffi::c_int),
            &raw mut (*c).flags as *const u8,
            ::core::mem::size_of::<uint64_t>() as size_t,
        );
    }
}
/// The flags a client carries, comma-separated, as the caller's own string.
pub unsafe fn server_client_get_flags(mut c: *mut client) -> ::std::ffi::CString {
    unsafe {
        let mut names: Vec<&::core::ffi::CStr> = Vec::new();
        if (*c).flags & CLIENT_ATTACHED as uint64_t != 0 {
            names.push(c"attached");
        }
        if (*c).flags & CLIENT_FOCUSED as uint64_t != 0 {
            names.push(c"focused");
        }
        if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            names.push(c"control-mode");
        }
        if (*c).flags & CLIENT_IGNORESIZE as uint64_t != 0 {
            names.push(c"ignore-size");
        }
        if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_NO_DETACH_ON_DESTROY != 0 {
            names.push(c"no-detach-on-destroy");
        }
        if (*c).flags & CLIENT_CONTROL_NOOUTPUT as uint64_t != 0 {
            names.push(c"no-output");
        }
        if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_CONTROL_WAITEXIT != 0 {
            names.push(c"wait-exit");
        }
        let paused;
        if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_CONTROL_PAUSEAFTER != 0 {
            paused = format_alloc(
                c"pause-after=%u".as_ptr(),
                fmt_args![(*c).pause_age.wrapping_div(1000 as u_int)],
            );
            names.push(&paused);
        }
        if (*c).flags & CLIENT_READONLY as uint64_t != 0 {
            names.push(c"read-only");
        }
        if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_ACTIVEPANE != 0 {
            names.push(c"active-pane");
        }
        if (*c).flags & CLIENT_SUSPENDED as uint64_t != 0 {
            names.push(c"suspended");
        }
        if (*c).flags & CLIENT_UTF8 as uint64_t != 0 {
            names.push(c"UTF-8");
        }
        let joined = names
            .iter()
            .map(|name| name.to_bytes())
            .collect::<Vec<_>>()
            .join(b",".as_slice());
        ::std::ffi::CString::new(joined).expect("a flag name has no interior NUL")
    }
}
pub unsafe fn server_client_get_client_window(
    mut c: *mut client,
    mut id: u_int,
) -> *mut client_window {
    unsafe {
        (*c).windows
            .get_mut(&id)
            .map_or(::core::ptr::null_mut::<client_window>(), |cw| {
                cw as *mut client_window
            })
    }
}
pub unsafe fn server_client_add_client_window(
    mut c: *mut client,
    mut id: u_int,
) -> *mut client_window {
    unsafe {
        (*c).windows.entry(id).or_insert(client_window {
            window: id,
            pane_id: None,
            sx: 0 as u_int,
            sy: 0 as u_int,
        }) as *mut client_window
    }
}
pub unsafe fn server_client_get_pane(mut c: *mut client) -> *mut window_pane {
    unsafe {
        let mut s: *mut session = (*c).session;
        let mut cw: *mut client_window = ::core::ptr::null_mut::<client_window>();
        if s.is_null() {
            return ::core::ptr::null_mut::<window_pane>();
        }
        if !(*c).flags as ::core::ffi::c_ulonglong & CLIENT_ACTIVEPANE != 0 {
            return window_get_active((*session_get_curw(s)).window());
        }
        cw = server_client_get_client_window(c, (*(*session_get_curw(s)).window()).id);
        if cw.is_null() {
            return window_get_active((*session_get_curw(s)).window());
        }
        match (*cw).pane_id {
            Some(id) => window_pane_find_by_id(id),
            None => ::core::ptr::null_mut::<window_pane>(),
        }
    }
}
pub unsafe fn server_client_set_pane(mut c: *mut client, mut wp: *mut window_pane) {
    unsafe {
        let mut s: *mut session = (*c).session;
        let mut cw: *mut client_window = ::core::ptr::null_mut::<client_window>();
        if s.is_null() {
            return;
        }
        cw = server_client_add_client_window(c, (*(*session_get_curw(s)).window()).id);
        (*cw).pane_id = Some((*wp).id);
        log_debug(
            c"%s pane now %%%u".as_ptr(),
            fmt_args![(*c).name.as_deref(), (*wp).id],
        );
    }
}
pub unsafe fn server_client_remove_pane(mut wp: *mut window_pane) {
    unsafe {
        let mut w: *mut window = (*wp).window;
        let mut cw: *mut client_window = ::core::ptr::null_mut::<client_window>();
        for c in client_walk() {
            cw = server_client_get_client_window(c, (*w).id);
            if !cw.is_null() && (*cw).pane_id == Some((*wp).id) {
                (*c).windows.remove(&(*cw).window);
            }
            if (*c).tty.mouse_last_pane == (*wp).id as ::core::ffi::c_int {
                (*c).tty.mouse_last_pane = -(1 as ::core::ffi::c_int);
                (*c).tty.mouse_drag_update = None;
                (*c).tty.mouse_scrolling_flag = 0 as ::core::ffi::c_int;
            }
        }
    }
}
pub unsafe fn server_client_print(
    mut c: *mut client,
    mut parse: ::core::ffi::c_int,
    buffer: &mut Buf,
) {
    unsafe {
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
        let mut source_with_nul = buffer.as_slice().to_vec();
        let size = source_with_nul.len();
        source_with_nul.push(0);
        let data = source_with_nul.as_ptr() as *const ::core::ffi::c_char;
        let visible = if parse == 0 {
            Some(utf8_stravisx(
                &source_with_nul[..size],
                VIS_OCTAL | VIS_CSTYLE | VIS_NOSLASH,
            ))
        } else {
            None
        };
        let msg: *const ::core::ffi::c_char = visible.as_ref().map_or(data, |v| v.as_ptr());
        log_debug(
            c"%s: %s".as_ptr(),
            fmt_args![c"server_client_print".as_ptr(), msg],
        );
        if !c.is_null() {
            if (*c).session.is_null() || (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
                if !(*c).flags & CLIENT_UTF8 as uint64_t != 0 {
                    let sanitized = utf8_sanitize(msg);
                    if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
                        control_write(c, c"%s".as_ptr(), fmt_args![sanitized.as_ptr()]);
                    } else {
                        file_print(c, c"%s\n".as_ptr(), fmt_args![sanitized.as_ptr()]);
                    }
                } else if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
                    control_write(c, c"%s".as_ptr(), fmt_args![msg]);
                } else {
                    file_print(c, c"%s\n".as_ptr(), fmt_args![msg]);
                }
            } else {
                wp = server_client_get_pane(c);
                wme = window_pane_current_mode(wp);
                if wme.is_null() || (*wme).mode() != WindowMode::View {
                    window_pane_set_mode(
                        wp,
                        ::core::ptr::null_mut::<window_pane>(),
                        WindowMode::View,
                        ::core::ptr::null_mut::<cmd_find_state>(),
                        ::core::ptr::null_mut::<args>(),
                    );
                }
                if parse != 0 {
                    loop {
                        let Some(line) = buffer.read_line() else {
                            break;
                        };
                        let mut line_data = line.to_vec();
                        line_data.push(0);
                        {
                            window_copy_add(
                                wp,
                                1 as ::core::ffi::c_int,
                                c"%s".as_ptr(),
                                fmt_args![line_data.as_ptr() as *const ::core::ffi::c_char],
                            );
                        }
                    }
                    let remainder = buffer.as_slice().to_vec();
                    if !remainder.is_empty() {
                        let mut remainder = remainder;
                        remainder.push(0);
                        window_copy_add(
                            wp,
                            1 as ::core::ffi::c_int,
                            c"%.*s".as_ptr(),
                            fmt_args![
                                remainder.len().wrapping_sub(1) as ::core::ffi::c_int,
                                remainder.as_ptr() as *const ::core::ffi::c_char
                            ],
                        );
                    }
                } else {
                    window_copy_add(wp, 0 as ::core::ffi::c_int, c"%s".as_ptr(), fmt_args![msg]);
                }
            }
        }
    }
}
unsafe fn server_client_report_theme(mut c: *mut client, mut theme: client_theme) {
    unsafe {
        if theme as ::core::ffi::c_uint == THEME_LIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*c).theme = THEME_LIGHT;
            notify_client(c"client-light-theme".as_ptr(), c);
        } else {
            (*c).theme = THEME_DARK;
            notify_client(c"client-dark-theme".as_ptr(), c);
        }
        tty_repeat_requests(&raw mut (*c).tty, 1 as ::core::ffi::c_int);
    }
}

#[cfg(test)]
#[path = "../tests/test_server_client.rs"]
mod tests;
