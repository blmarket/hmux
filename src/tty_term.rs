pub use crate::src::shared::tty::{tty_terms};
pub use crate::src::shared::arguments::{args};
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds};
pub use crate::src::shared::control::{control_state};
pub use crate::src::shared::format::{format_job_tree, format_tree};
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::menu::{menu_data};
pub use crate::src::shared::options::{
    options, options_array, options_array_item, options_entry, options_value,
};
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::process::{tmuxpeer};
pub use crate::src::shared::prompt::{prompt};
pub use crate::src::shared::redraw::{redraw_scene};
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::{spawn_editor_state};
pub use crate::src::shared::status::{status_line};
pub use crate::src::shared::terminal::{TERMTYPE, term, termtype};
pub use crate::src::shared::tty::{
    tty, tty_code, tty_code_type, tty_code_value, tty_key, tty_term, tty_term_entry,
};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
pub use crate::src::shared::environment::{environ, environ_entry, environ_entry_entry};
pub use crate::src::shared::vis::{VIS_CSTYLE, VIS_NL, VIS_OCTAL, VIS_TAB};
pub use crate::src::shared::limits::{__INT_MAX__, INT_MAX};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::tty::{
    TERM_DECFRA, TERM_DECSLRM, TERM_INVALIDMS, TERM_NOAM, TERM_RGBCOLOURS, TERM_SIXEL,
    TERM_VT100LIKE,
};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::tty::*;
use crate::src::shared::terminal::*;
use crate::src::shared::event::*;
use crate::src::shared::display::*;
use crate::src::shared::layout::*;
use crate::src::shared::message::*;
use crate::src::shared::abi::*;
use crate::src::shared::colour::*;
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
use crate::src::shared::style::*;
extern "C" {

    fn tigetflag(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn tigetnum(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn tigetstr(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn tiparm_s(
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_char,
        ...
    ) -> *mut ::core::ffi::c_char;
    fn fnmatch(
        __pattern: *const ::core::ffi::c_char,
        __name: *const ::core::ffi::c_char,
        __flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strchr(__s: *const ::core::ffi::c_char, __c: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn strcspn(
        __s: *const ::core::ffi::c_char,
        __reject: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_ulong;
    fn strstr(
        __haystack: *const ::core::ffi::c_char,
        __needle: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strcasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    static mut cur_term: *mut TERMINAL;
    fn del_curterm(_: *mut TERMINAL) -> ::core::ffi::c_int;
    fn setupterm(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn strnvis(
        _: *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: size_t,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn strunvis(_: *mut ::core::ffi::c_char, _: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn strtonum(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: *mut *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xreallocarray(_: *mut ::core::ffi::c_void, _: size_t, _: size_t)
        -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xasprintf(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn xsnprintf(
        _: *mut ::core::ffi::c_char,
        _: size_t,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    static mut global_options: *mut options;
    fn options_get_only(_: *mut options, _: *const ::core::ffi::c_char) -> *mut options_entry;
    fn options_array_first(_: *mut options_entry) -> *mut options_array_item;
    fn options_array_next(_: *mut options_array_item) -> *mut options_array_item;
    fn options_array_item_value(_: *mut options_array_item) -> *mut options_value;
    fn environ_find(_: *mut environ, _: *const ::core::ffi::c_char) -> *mut environ_entry;
    fn tty_parse_client_features(
        _: *mut client,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
    );
    fn tty_apply_features(_: *mut tty_term) -> ::core::ffi::c_int;
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn fatalx(_: *const ::core::ffi::c_char, ...) -> !;
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed {
    pub offset: u_int,
    pub data: C2RustUnnamed_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_0 {
    pub attr: u_char,
    pub fg: u_char,
    pub bg: u_char,
    pub data: u_char,
}

pub type TERMINAL = term;

pub const TTYCODE_FLAG: tty_code_type = 3;
pub const TTYCODE_NUMBER: tty_code_type = 2;
pub const TTYCODE_STRING: tty_code_type = 1;
pub const TTYCODE_NONE: tty_code_type = 0;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct tty_term_code_entry {
    pub type_0: tty_code_type,
    pub name: *const ::core::ffi::c_char,
}
pub const OK: ::core::ffi::c_int = 0 as ::core::ffi::c_int;

#[no_mangle]
pub static mut tty_terms: tty_terms = tty_terms {
    lh_first: ::core::ptr::null::<tty_term>() as *mut tty_term,
};
static mut tty_term_codes: [tty_term_code_entry; 234] = [
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"acsc\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_FLAG,
        name: b"am\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_FLAG,
        name: b"AX\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_FLAG,
        name: b"bce\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"bel\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Bidi\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"blink\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"bold\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"civis\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"clear\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Clmg\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Cmg\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"cnorm\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_NUMBER,
        name: b"colors\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Cr\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Cs\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"csr\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"cub\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"cub1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"cud\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"cud1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"cuf\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"cuf1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"cup\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"cuu\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"cuu1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"cvvis\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"dch\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"dch1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"dim\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"dl\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"dl1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Dsbp\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Dseks\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Dsfcs\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Dsmg\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"E3\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"ech\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"ed\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"el\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"el1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"enacs\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Enbp\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Eneks\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Enfcs\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Enmg\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"fsl\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Hls\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"home\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"hpa\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"ich\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"ich1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"il\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"il1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"ind\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"indn\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"invis\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kcbt\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kcub1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kcud1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kcuf1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kcuu1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kDC\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kDC3\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kDC4\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kDC5\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kDC6\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kDC7\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kdch1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kDN\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kDN3\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kDN4\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kDN5\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kDN6\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kDN7\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kend\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kEND\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kEND3\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kEND4\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kEND5\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kEND6\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kEND7\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf10\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf11\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf12\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf13\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf14\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf15\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf16\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf17\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf18\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf19\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf2\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf20\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf21\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf22\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf23\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf24\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf25\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf26\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf27\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf28\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf29\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf3\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf30\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf31\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf32\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf33\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf34\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf35\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf36\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf37\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf38\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf39\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf4\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf40\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf41\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf42\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf43\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf44\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf45\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf46\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf47\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf48\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf49\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf5\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf50\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf51\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf52\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf53\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf54\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf55\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf56\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf57\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf58\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf59\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf6\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf60\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf61\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf62\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf63\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf7\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf8\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf9\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kHOM\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kHOM3\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kHOM4\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kHOM5\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kHOM6\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kHOM7\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"khome\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kIC\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kIC3\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kIC4\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kIC5\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kIC6\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kIC7\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kich1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kind\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kLFT\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kLFT3\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kLFT4\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kLFT5\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kLFT6\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kLFT7\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kmous\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"knp\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kNXT\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kNXT3\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kNXT4\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kNXT5\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kNXT6\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kNXT7\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kpp\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kPRV\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kPRV3\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kPRV4\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kPRV5\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kPRV6\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kPRV7\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kri\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kRIT\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kRIT3\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kRIT4\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kRIT5\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kRIT6\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kRIT7\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kUP\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kUP3\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kUP4\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kUP5\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kUP6\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kUP7\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Ms\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Nobr\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"ol\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"op\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Rect\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"rev\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_FLAG,
        name: b"RGB\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"ri\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"rin\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"rmacs\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"rmcup\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"rmkx\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Se\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"setab\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"setaf\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"setal\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"setrgbb\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"setrgbf\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Setulc\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Setulc1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"sgr0\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"sitm\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"smacs\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"smcup\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"smkx\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Smol\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"smso\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"smul\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Smulx\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"smxx\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Spb\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_FLAG,
        name: b"Sxl\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Ss\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Swd\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Sync\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_FLAG,
        name: b"Tc\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"tsl\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_NUMBER,
        name: b"U8\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"vpa\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_FLAG,
        name: b"XT\0" as *const u8 as *const ::core::ffi::c_char,
    },
];
#[no_mangle]
pub unsafe extern "C" fn tty_term_ncodes() -> u_int {
    return (::core::mem::size_of::<[tty_term_code_entry; 234]>() as usize)
        .wrapping_div(::core::mem::size_of::<tty_term_code_entry>() as usize) as u_int;
}
unsafe extern "C" fn tty_term_strip(mut s: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    let mut ptr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    static mut buf: [::core::ffi::c_char; 8192] = [0; 8192];
    let mut len: size_t = 0;
    if strchr(s, '$' as i32).is_null() {
        return xstrdup(s);
    }
    len = 0 as size_t;
    ptr = s;
    while *ptr as ::core::ffi::c_int != '\0' as i32 {
        if *ptr as ::core::ffi::c_int == '$' as i32
            && *ptr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '<' as i32
        {
            while *ptr as ::core::ffi::c_int != '\0' as i32
                && *ptr as ::core::ffi::c_int != '>' as i32
            {
                ptr = ptr.offset(1);
            }
            if *ptr as ::core::ffi::c_int == '>' as i32 {
                ptr = ptr.offset(1);
            }
            if *ptr as ::core::ffi::c_int == '\0' as i32 {
                break;
            }
        }
        let fresh3 = len;
        len = len.wrapping_add(1);
        buf[fresh3 as usize] = *ptr;
        if len
            == (::core::mem::size_of::<[::core::ffi::c_char; 8192]>() as usize)
                .wrapping_sub(1 as usize)
        {
            break;
        }
        ptr = ptr.offset(1);
    }
    buf[len as usize] = '\0' as i32 as ::core::ffi::c_char;
    return xstrdup(&raw mut buf as *mut ::core::ffi::c_char);
}
unsafe extern "C" fn tty_term_override_next(
    mut s: *const ::core::ffi::c_char,
    mut offset: *mut size_t,
) -> *mut ::core::ffi::c_char {
    static mut value: [::core::ffi::c_char; 8192] = [0; 8192];
    let mut n: size_t = 0 as size_t;
    let mut at: size_t = *offset;
    if *s.offset(at as isize) as ::core::ffi::c_int == '\0' as i32 {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    while *s.offset(at as isize) as ::core::ffi::c_int != '\0' as i32 {
        if *s.offset(at as isize) as ::core::ffi::c_int == ':' as i32 {
            if !(*s.offset(at.wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int
                == ':' as i32)
            {
                break;
            }
            let fresh1 = n;
            n = n.wrapping_add(1);
            value[fresh1 as usize] = ':' as i32 as ::core::ffi::c_char;
            at = at.wrapping_add(2 as size_t);
        } else {
            let fresh2 = n;
            n = n.wrapping_add(1);
            value[fresh2 as usize] = *s.offset(at as isize);
            at = at.wrapping_add(1);
        }
        if n == (::core::mem::size_of::<[::core::ffi::c_char; 8192]>() as usize)
            .wrapping_sub(1 as usize)
        {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
    }
    if *s.offset(at as isize) as ::core::ffi::c_int != '\0' as i32 {
        *offset = at.wrapping_add(1 as size_t);
    } else {
        *offset = at;
    }
    value[n as usize] = '\0' as i32 as ::core::ffi::c_char;
    return &raw mut value as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_apply(
    mut term: *mut tty_term,
    mut capabilities: *const ::core::ffi::c_char,
    mut quiet: ::core::ffi::c_int,
) {
    let mut ent: *const tty_term_code_entry = ::core::ptr::null::<tty_term_code_entry>();
    let mut code: *mut tty_code = ::core::ptr::null_mut::<tty_code>();
    let mut offset: size_t = 0 as size_t;
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut name: *const ::core::ffi::c_char = (*term).name;
    let mut i: u_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    let mut remove: ::core::ffi::c_int = 0;
    loop {
        s = tty_term_override_next(capabilities, &raw mut offset);
        if s.is_null() {
            break;
        }
        if *s as ::core::ffi::c_int == '\0' as i32 {
            continue;
        }
        value = ::core::ptr::null_mut::<::core::ffi::c_char>();
        remove = 0 as ::core::ffi::c_int;
        cp = strchr(s, '=' as i32);
        if !cp.is_null() {
            let fresh0 = cp;
            cp = cp.offset(1);
            *fresh0 = '\0' as i32 as ::core::ffi::c_char;
            value = xstrdup(cp);
            if strunvis(value, cp) == -(1 as ::core::ffi::c_int) {
                free(value as *mut ::core::ffi::c_void);
                value = xstrdup(cp);
            }
        } else if *s.offset(strlen(s).wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
            == '@' as i32
        {
            *s.offset(strlen(s).wrapping_sub(1 as size_t) as isize) =
                '\0' as i32 as ::core::ffi::c_char;
            remove = 1 as ::core::ffi::c_int;
        } else {
            value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
        }
        if quiet == 0 {
            if remove != 0 {
                log_debug(
                    b"%s override: %s@\0" as *const u8 as *const ::core::ffi::c_char,
                    name,
                    s,
                );
            } else if *value as ::core::ffi::c_int == '\0' as i32 {
                log_debug(
                    b"%s override: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    name,
                    s,
                );
            } else {
                log_debug(
                    b"%s override: %s=%s\0" as *const u8 as *const ::core::ffi::c_char,
                    name,
                    s,
                    value,
                );
            }
        }
        i = 0 as u_int;
        while i < tty_term_ncodes() {
            ent = (&raw const tty_term_codes as *const tty_term_code_entry).offset(i as isize)
                as *const tty_term_code_entry;
            if !(strcmp(s, (*ent).name) != 0 as ::core::ffi::c_int) {
                code = (*term).codes.offset(i as isize) as *mut tty_code;
                if remove != 0 {
                    (*code).type_0 = TTYCODE_NONE;
                } else {
                    match (*ent).type_0 as ::core::ffi::c_uint {
                        1 => {
                            if (*code).type_0 as ::core::ffi::c_uint
                                == TTYCODE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                free((*code).value.string as *mut ::core::ffi::c_void);
                            }
                            (*code).value.string = xstrdup(value);
                            (*code).type_0 = (*ent).type_0;
                        }
                        2 => {
                            n = strtonum(
                                value,
                                0 as ::core::ffi::c_longlong,
                                INT_MAX as ::core::ffi::c_longlong,
                                &raw mut errstr,
                            ) as ::core::ffi::c_int;
                            if errstr.is_null() {
                                (*code).value.number = n;
                                (*code).type_0 = (*ent).type_0;
                            }
                        }
                        3 => {
                            (*code).value.flag = 1 as ::core::ffi::c_int;
                            (*code).type_0 = (*ent).type_0;
                        }
                        0 | _ => {}
                    }
                }
            }
            i = i.wrapping_add(1);
        }
        free(value as *mut ::core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_apply_overrides(mut term: *mut tty_term) {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut ov: *mut options_value = ::core::ptr::null_mut::<options_value>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut acs: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut offset: size_t = 0;
    let mut first: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    o = options_get_only(
        global_options,
        b"terminal-overrides\0" as *const u8 as *const ::core::ffi::c_char,
    );
    a = options_array_first(o);
    while !a.is_null() {
        ov = options_array_item_value(a);
        s = (*ov).string;
        offset = 0 as size_t;
        first = tty_term_override_next(s, &raw mut offset);
        if !first.is_null()
            && fnmatch(first, (*term).name, 0 as ::core::ffi::c_int) == 0 as ::core::ffi::c_int
        {
            tty_term_apply(term, s.offset(offset as isize), 0 as ::core::ffi::c_int);
        }
        a = options_array_next(a);
    }
    log_debug(
        b"SIXEL flag is %d\0" as *const u8 as *const ::core::ffi::c_char,
        ((*term).flags & TERM_SIXEL != 0) as ::core::ffi::c_int,
    );
    if tty_term_has(term, TTYC_SETRGBF) != 0 && tty_term_has(term, TTYC_SETRGBB) != 0 {
        (*term).flags |= TERM_RGBCOLOURS;
    } else {
        (*term).flags &= !TERM_RGBCOLOURS;
    }
    log_debug(
        b"RGBCOLOURS flag is %d\0" as *const u8 as *const ::core::ffi::c_char,
        ((*term).flags & TERM_RGBCOLOURS != 0) as ::core::ffi::c_int,
    );
    if tty_term_has(term, TTYC_CMG) != 0 && tty_term_has(term, TTYC_CLMG) != 0 {
        (*term).flags |= TERM_DECSLRM;
    } else {
        (*term).flags &= !TERM_DECSLRM;
    }
    log_debug(
        b"DECSLRM flag is %d\0" as *const u8 as *const ::core::ffi::c_char,
        ((*term).flags & TERM_DECSLRM != 0) as ::core::ffi::c_int,
    );
    if tty_term_has(term, TTYC_RECT) != 0 {
        (*term).flags |= TERM_DECFRA;
    } else {
        (*term).flags &= !TERM_DECFRA;
    }
    log_debug(
        b"DECFRA flag is %d\0" as *const u8 as *const ::core::ffi::c_char,
        ((*term).flags & TERM_DECFRA != 0) as ::core::ffi::c_int,
    );
    if tty_term_flag(term, TTYC_AM) == 0 {
        (*term).flags |= TERM_NOAM;
    } else {
        (*term).flags &= !TERM_NOAM;
    }
    log_debug(
        b"NOAM flag is %d\0" as *const u8 as *const ::core::ffi::c_char,
        ((*term).flags & TERM_NOAM != 0) as ::core::ffi::c_int,
    );
    memset(
        &raw mut (*term).acs as *mut [::core::ffi::c_char; 2] as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[[::core::ffi::c_char; 2]; 256]>() as size_t,
    );
    if tty_term_has(term, TTYC_ACSC) != 0 {
        acs = tty_term_string(term, TTYC_ACSC);
    } else {
        acs =
            b"a#j+k+l+m+n+o-p-q-r-s-t+u+v+w+x|y<z>~.\0" as *const u8 as *const ::core::ffi::c_char;
    }
    while *acs.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32
        && *acs.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32
    {
        (*term).acs[*acs.offset(0 as ::core::ffi::c_int as isize) as u_char as usize]
            [0 as ::core::ffi::c_int as usize] = *acs.offset(1 as ::core::ffi::c_int as isize);
        acs = acs.offset(2 as ::core::ffi::c_int as isize);
    }
    tty_term_validate(term);
}
unsafe extern "C" fn tty_term_validate(mut term: *mut tty_term) {
    let mut code: *mut tty_code =
        (*term).codes.offset(TTYC_MS as ::core::ffi::c_int as isize) as *mut tty_code;
    if (*code).type_0 as ::core::ffi::c_uint
        != TTYCODE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    if *tty_term_string_ss(
        term,
        TTYC_MS,
        b"c\0" as *const u8 as *const ::core::ffi::c_char,
        b"?\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int
        != '\0' as i32
    {
        (*term).flags &= !TERM_INVALIDMS;
        return;
    }
    log_debug(b"removing invalid Ms capability\0" as *const u8 as *const ::core::ffi::c_char);
    (*term).flags |= TERM_INVALIDMS;
    free((*code).value.string as *mut ::core::ffi::c_void);
    (*code).type_0 = TTYCODE_NONE;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_create(
    mut tty: *mut tty,
    mut name: *mut ::core::ffi::c_char,
    mut caps: *mut *mut ::core::ffi::c_char,
    mut ncaps: u_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> *mut tty_term {
    let mut c: *mut client = (*tty).client;
    let mut term: *mut tty_term = ::core::ptr::null_mut::<tty_term>();
    let mut ent: *const tty_term_code_entry = ::core::ptr::null::<tty_term_code_entry>();
    let mut code: *mut tty_code = ::core::ptr::null_mut::<tty_code>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut ov: *mut options_value = ::core::ptr::null_mut::<options_value>();
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut offset: size_t = 0;
    let mut namelen: size_t = 0;
    let mut first: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut n: ::core::ffi::c_int = 0;
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    log_debug(
        b"adding term %s\0" as *const u8 as *const ::core::ffi::c_char,
        name,
    );
    term = xcalloc(1 as size_t, ::core::mem::size_of::<tty_term>() as size_t) as *mut tty_term;
    (*term).tty = tty as *mut tty;
    (*term).name = xstrdup(name);
    (*term).codes = xcalloc(
        tty_term_ncodes() as size_t,
        ::core::mem::size_of::<tty_code>() as size_t,
    ) as *mut tty_code;
    (*term).entry.le_next = tty_terms.lh_first;
    if !(*term).entry.le_next.is_null() {
        (*tty_terms.lh_first).entry.le_prev = &raw mut (*term).entry.le_next;
    }
    tty_terms.lh_first = term;
    (*term).entry.le_prev = &raw mut tty_terms.lh_first;
    i = 0 as u_int;
    while i < ncaps {
        namelen = strcspn(
            *caps.offset(i as isize),
            b"=\0" as *const u8 as *const ::core::ffi::c_char,
        ) as size_t;
        if !(namelen == 0 as size_t) {
            value = (*caps.offset(i as isize))
                .offset(namelen as isize)
                .offset(1 as ::core::ffi::c_int as isize);
            j = 0 as u_int;
            while j < tty_term_ncodes() {
                ent = (&raw const tty_term_codes as *const tty_term_code_entry).offset(j as isize)
                    as *const tty_term_code_entry;
                if !(strncmp((*ent).name, *caps.offset(i as isize), namelen)
                    != 0 as ::core::ffi::c_int)
                {
                    if !(*(*ent).name.offset(namelen as isize) as ::core::ffi::c_int != '\0' as i32)
                    {
                        code = (*term).codes.offset(j as isize) as *mut tty_code;
                        (*code).type_0 = TTYCODE_NONE;
                        match (*ent).type_0 as ::core::ffi::c_uint {
                            1 => {
                                (*code).type_0 = TTYCODE_STRING;
                                (*code).value.string = tty_term_strip(value);
                            }
                            2 => {
                                n = strtonum(
                                    value,
                                    0 as ::core::ffi::c_longlong,
                                    INT_MAX as ::core::ffi::c_longlong,
                                    &raw mut errstr,
                                ) as ::core::ffi::c_int;
                                if !errstr.is_null() {
                                    log_debug(
                                        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
                                        (*ent).name,
                                        errstr,
                                    );
                                } else {
                                    (*code).type_0 = TTYCODE_NUMBER;
                                    (*code).value.number = n;
                                }
                            }
                            3 => {
                                (*code).type_0 = TTYCODE_FLAG;
                                (*code).value.flag = (*value as ::core::ffi::c_int == '1' as i32)
                                    as ::core::ffi::c_int;
                            }
                            0 | _ => {}
                        }
                    }
                }
                j = j.wrapping_add(1);
            }
        }
        i = i.wrapping_add(1);
    }
    o = options_get_only(
        global_options,
        b"terminal-features\0" as *const u8 as *const ::core::ffi::c_char,
    );
    a = options_array_first(o);
    while !a.is_null() {
        ov = options_array_item_value(a);
        s = (*ov).string;
        offset = 0 as size_t;
        first = tty_term_override_next(s, &raw mut offset);
        if !first.is_null()
            && fnmatch(first, (*term).name, 0 as ::core::ffi::c_int) == 0 as ::core::ffi::c_int
        {
            tty_parse_client_features(
                c,
                s.offset(offset as isize),
                b":\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        a = options_array_next(a);
    }
    del_curterm(cur_term);
    envent = environ_find(
        (*c).environ,
        b"COLORTERM\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if !envent.is_null() {
        log_debug(
            b"%s COLORTERM=%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
            (*envent).value,
        );
        if strcasecmp(
            (*envent).value,
            b"truecolor\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
            || strcasecmp(
                (*envent).value,
                b"24bit\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
        {
            tty_parse_client_features(
                c,
                b"RGB\0" as *const u8 as *const ::core::ffi::c_char,
                b",\0" as *const u8 as *const ::core::ffi::c_char,
            );
        } else if !strstr(
            (*envent).value,
            b"256\0" as *const u8 as *const ::core::ffi::c_char,
        )
        .is_null()
        {
            tty_parse_client_features(
                c,
                b"256\0" as *const u8 as *const ::core::ffi::c_char,
                b",\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
    }
    tty_term_apply_overrides(term);
    if tty_term_has(term, TTYC_CLEAR) == 0 {
        xasprintf(
            cause,
            b"terminal does not support clear\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else if tty_term_has(term, TTYC_CUP) == 0 {
        xasprintf(
            cause,
            b"terminal does not support cup\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        s = tty_term_string(term, TTYC_CLEAR);
        if tty_term_flag(term, TTYC_XT) != 0
            || strncmp(
                s,
                b"\x1B[\0" as *const u8 as *const ::core::ffi::c_char,
                2 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            (*term).flags |= TERM_VT100LIKE;
            tty_parse_client_features(
                c,
                b"bpaste,focus,title\0" as *const u8 as *const ::core::ffi::c_char,
                b",\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        if (tty_term_flag(term, TTYC_TC) != 0 || tty_term_has(term, TTYC_RGB) != 0)
            && (tty_term_has(term, TTYC_SETRGBF) == 0 || tty_term_has(term, TTYC_SETRGBB) == 0)
        {
            tty_parse_client_features(
                c,
                b"RGB\0" as *const u8 as *const ::core::ffi::c_char,
                b",\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        if tty_apply_features(term) != 0 {
            tty_term_apply_overrides(term);
        }
        i = 0 as u_int;
        while i < tty_term_ncodes() {
            log_debug(
                b"%s%s\0" as *const u8 as *const ::core::ffi::c_char,
                name,
                tty_term_describe(term, i as tty_code_code),
            );
            i = i.wrapping_add(1);
        }
        return term;
    }
    tty_term_free(term);
    return ::core::ptr::null_mut::<tty_term>();
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_free(mut term: *mut tty_term) {
    let mut i: u_int = 0;
    log_debug(
        b"removing term %s\0" as *const u8 as *const ::core::ffi::c_char,
        (*term).name,
    );
    i = 0 as u_int;
    while i < tty_term_ncodes() {
        if (*(*term).codes.offset(i as isize)).type_0 as ::core::ffi::c_uint
            == TTYCODE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            free((*(*term).codes.offset(i as isize)).value.string as *mut ::core::ffi::c_void);
        }
        i = i.wrapping_add(1);
    }
    free((*term).codes as *mut ::core::ffi::c_void);
    if !(*term).entry.le_next.is_null() {
        (*(*term).entry.le_next).entry.le_prev = (*term).entry.le_prev;
    }
    *(*term).entry.le_prev = (*term).entry.le_next;
    free((*term).name as *mut ::core::ffi::c_void);
    free(term as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_read_list(
    mut name: *const ::core::ffi::c_char,
    mut fd: ::core::ffi::c_int,
    mut caps: *mut *mut *mut ::core::ffi::c_char,
    mut ncaps: *mut u_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut ent: *const tty_term_code_entry = ::core::ptr::null::<tty_term_code_entry>();
    let mut error: ::core::ffi::c_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    let mut i: u_int = 0;
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut tmp: [::core::ffi::c_char; 11] = [0; 11];
    if setupterm(name as *mut ::core::ffi::c_char, fd, &raw mut error) != OK {
        match error {
            1 => {
                xasprintf(
                    cause,
                    b"can't use hardcopy terminal: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    name,
                );
            }
            0 => {
                xasprintf(
                    cause,
                    b"missing or unsuitable terminal: %s\0" as *const u8
                        as *const ::core::ffi::c_char,
                    name,
                );
            }
            -1 => {
                xasprintf(
                    cause,
                    b"can't find terminfo database\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            _ => {
                xasprintf(
                    cause,
                    b"unknown error\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
        }
        return -(1 as ::core::ffi::c_int);
    }
    *ncaps = 0 as u_int;
    *caps = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut current_block_23: u64;
    i = 0 as u_int;
    while i < tty_term_ncodes() {
        ent = (&raw const tty_term_codes as *const tty_term_code_entry).offset(i as isize)
            as *const tty_term_code_entry;
        match (*ent).type_0 as ::core::ffi::c_uint {
            0 => {
                current_block_23 = 1856101646708284338;
            }
            1 => {
                s = tigetstr((*ent).name as *mut ::core::ffi::c_char);
                if s.is_null()
                    || s == -(1 as ::core::ffi::c_int) as *mut ::core::ffi::c_char
                        as *const ::core::ffi::c_char
                {
                    current_block_23 = 1856101646708284338;
                } else {
                    current_block_23 = 14763689060501151050;
                }
            }
            2 => {
                n = tigetnum((*ent).name as *mut ::core::ffi::c_char);
                if n == -(1 as ::core::ffi::c_int) || n == -(2 as ::core::ffi::c_int) {
                    current_block_23 = 1856101646708284338;
                } else {
                    xsnprintf(
                        &raw mut tmp as *mut ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 11]>() as size_t,
                        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
                        n,
                    );
                    s = &raw mut tmp as *mut ::core::ffi::c_char;
                    current_block_23 = 14763689060501151050;
                }
            }
            3 => {
                n = tigetflag((*ent).name as *mut ::core::ffi::c_char);
                if n == -(1 as ::core::ffi::c_int) {
                    current_block_23 = 1856101646708284338;
                } else {
                    if n != 0 {
                        s = b"1\0" as *const u8 as *const ::core::ffi::c_char;
                    } else {
                        s = b"0\0" as *const u8 as *const ::core::ffi::c_char;
                    }
                    current_block_23 = 14763689060501151050;
                }
            }
            _ => {
                fatalx(b"unknown capability type\0" as *const u8 as *const ::core::ffi::c_char);
            }
        }
        match current_block_23 {
            14763689060501151050 => {
                *caps = xreallocarray(
                    *caps as *mut ::core::ffi::c_void,
                    (*ncaps).wrapping_add(1 as u_int) as size_t,
                    ::core::mem::size_of::<*mut ::core::ffi::c_char>() as size_t,
                ) as *mut *mut ::core::ffi::c_char;
                xasprintf(
                    (*caps).offset(*ncaps as isize) as *mut *mut ::core::ffi::c_char,
                    b"%s=%s\0" as *const u8 as *const ::core::ffi::c_char,
                    (*ent).name,
                    s,
                );
                *ncaps = (*ncaps).wrapping_add(1);
            }
            _ => {}
        }
        i = i.wrapping_add(1);
    }
    del_curterm(cur_term);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_free_list(
    mut caps: *mut *mut ::core::ffi::c_char,
    mut ncaps: u_int,
) {
    let mut i: u_int = 0;
    i = 0 as u_int;
    while i < ncaps {
        free(*caps.offset(i as isize) as *mut ::core::ffi::c_void);
        i = i.wrapping_add(1);
    }
    free(caps as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_has(
    mut term: *mut tty_term,
    mut code: tty_code_code,
) -> ::core::ffi::c_int {
    return ((*(*term).codes.offset(code as isize)).type_0 as ::core::ffi::c_uint
        != TTYCODE_NONE as ::core::ffi::c_int as ::core::ffi::c_uint)
        as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_has_name(
    mut term: *mut tty_term,
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut i: u_int = 0;
    i = 0 as u_int;
    while i < tty_term_ncodes() {
        if strcmp(tty_term_codes[i as usize].name, name) == 0 as ::core::ffi::c_int {
            return tty_term_has(term, i as tty_code_code);
        }
        i = i.wrapping_add(1);
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_string(
    mut term: *mut tty_term,
    mut code: tty_code_code,
) -> *const ::core::ffi::c_char {
    if tty_term_has(term, code) == 0 {
        return b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if (*(*term).codes.offset(code as isize)).type_0 as ::core::ffi::c_uint
        != TTYCODE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        fatalx(
            b"not a string: %d\0" as *const u8 as *const ::core::ffi::c_char,
            code as ::core::ffi::c_uint,
        );
    }
    return (*(*term).codes.offset(code as isize)).value.string;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_string_i(
    mut term: *mut tty_term,
    mut code: tty_code_code,
    mut a: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    let mut x: *const ::core::ffi::c_char = tty_term_string(term, code);
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    s = tiparm_s(1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int, x, a);
    if s.is_null() {
        log_debug(
            b"could not expand %s\0" as *const u8 as *const ::core::ffi::c_char,
            tty_term_codes[code as usize].name,
        );
        return b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return s;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_string_ii(
    mut term: *mut tty_term,
    mut code: tty_code_code,
    mut a: ::core::ffi::c_int,
    mut b: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    let mut x: *const ::core::ffi::c_char = tty_term_string(term, code);
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    s = tiparm_s(2 as ::core::ffi::c_int, 0 as ::core::ffi::c_int, x, a, b);
    if s.is_null() {
        log_debug(
            b"could not expand %s\0" as *const u8 as *const ::core::ffi::c_char,
            tty_term_codes[code as usize].name,
        );
        return b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return s;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_string_iii(
    mut term: *mut tty_term,
    mut code: tty_code_code,
    mut a: ::core::ffi::c_int,
    mut b: ::core::ffi::c_int,
    mut c: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    let mut x: *const ::core::ffi::c_char = tty_term_string(term, code);
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    s = tiparm_s(3 as ::core::ffi::c_int, 0 as ::core::ffi::c_int, x, a, b, c);
    if s.is_null() {
        log_debug(
            b"could not expand %s\0" as *const u8 as *const ::core::ffi::c_char,
            tty_term_codes[code as usize].name,
        );
        return b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return s;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_string_s(
    mut term: *mut tty_term,
    mut code: tty_code_code,
    mut a: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut x: *const ::core::ffi::c_char = tty_term_string(term, code);
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    s = tiparm_s(1 as ::core::ffi::c_int, 1 as ::core::ffi::c_int, x, a);
    if s.is_null() {
        log_debug(
            b"could not expand %s\0" as *const u8 as *const ::core::ffi::c_char,
            tty_term_codes[code as usize].name,
        );
        return b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return s;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_string_ss(
    mut term: *mut tty_term,
    mut code: tty_code_code,
    mut a: *const ::core::ffi::c_char,
    mut b: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut x: *const ::core::ffi::c_char = tty_term_string(term, code);
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    s = tiparm_s(2 as ::core::ffi::c_int, 3 as ::core::ffi::c_int, x, a, b);
    if s.is_null() {
        log_debug(
            b"could not expand %s\0" as *const u8 as *const ::core::ffi::c_char,
            tty_term_codes[code as usize].name,
        );
        return b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return s;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_number(
    mut term: *mut tty_term,
    mut code: tty_code_code,
) -> ::core::ffi::c_int {
    if tty_term_has(term, code) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    if (*(*term).codes.offset(code as isize)).type_0 as ::core::ffi::c_uint
        != TTYCODE_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        fatalx(
            b"not a number: %d\0" as *const u8 as *const ::core::ffi::c_char,
            code as ::core::ffi::c_uint,
        );
    }
    return (*(*term).codes.offset(code as isize)).value.number;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_flag(
    mut term: *mut tty_term,
    mut code: tty_code_code,
) -> ::core::ffi::c_int {
    if tty_term_has(term, code) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    if (*(*term).codes.offset(code as isize)).type_0 as ::core::ffi::c_uint
        != TTYCODE_FLAG as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        fatalx(
            b"not a flag: %d\0" as *const u8 as *const ::core::ffi::c_char,
            code as ::core::ffi::c_uint,
        );
    }
    return (*(*term).codes.offset(code as isize)).value.flag;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_describe(
    mut term: *mut tty_term,
    mut code: tty_code_code,
) -> *const ::core::ffi::c_char {
    static mut s: [::core::ffi::c_char; 256] = [0; 256];
    let mut out: [::core::ffi::c_char; 128] = [0; 128];
    match (*(*term).codes.offset(code as isize)).type_0 as ::core::ffi::c_uint {
        0 => {
            xsnprintf(
                &raw mut s as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
                b"%4u: %s: [missing]\0" as *const u8 as *const ::core::ffi::c_char,
                code as ::core::ffi::c_uint,
                tty_term_codes[code as usize].name,
            );
        }
        1 => {
            strnvis(
                &raw mut out as *mut ::core::ffi::c_char,
                (*(*term).codes.offset(code as isize)).value.string,
                ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
                VIS_OCTAL | VIS_CSTYLE | VIS_TAB | VIS_NL,
            );
            xsnprintf(
                &raw mut s as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
                b"%4u: %s: (string) %s\0" as *const u8 as *const ::core::ffi::c_char,
                code as ::core::ffi::c_uint,
                tty_term_codes[code as usize].name,
                &raw mut out as *mut ::core::ffi::c_char,
            );
        }
        2 => {
            xsnprintf(
                &raw mut s as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
                b"%4u: %s: (number) %d\0" as *const u8 as *const ::core::ffi::c_char,
                code as ::core::ffi::c_uint,
                tty_term_codes[code as usize].name,
                (*(*term).codes.offset(code as isize)).value.number,
            );
        }
        3 => {
            xsnprintf(
                &raw mut s as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
                b"%4u: %s: (flag) %s\0" as *const u8 as *const ::core::ffi::c_char,
                code as ::core::ffi::c_uint,
                tty_term_codes[code as usize].name,
                if (*(*term).codes.offset(code as isize)).value.flag != 0 {
                    b"true\0" as *const u8 as *const ::core::ffi::c_char
                } else {
                    b"false\0" as *const u8 as *const ::core::ffi::c_char
                },
            );
        }
        _ => {}
    }
    return &raw mut s as *mut ::core::ffi::c_char;
}
