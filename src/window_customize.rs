use crate::src::format::bytes::write_cstr;
use crate::src::format::bytes::xformat;
use std::borrow::Cow;
use std::ffi::{CStr, CString};

use crate::src::arguments::{args_get, args_has};
use crate::src::cmd::find::{cmd_find_copy_state, cmd_find_from_pane, cmd_find_valid_state};
use crate::src::cmd::parse::{cmd_parse_error_uppercase_first, cmd_parse_from_string};
use crate::src::cmd::{cmd_list_free, cmd_list_print_cstring};
use crate::src::environ::{environ_clear, environ_find, environ_iter, environ_set, environ_unset};
use crate::src::ffi::libc::{
    __ctype_tolower_loc, __ctype_toupper_loc, memcpy, strchr, strcmp, strcspn, strlcat, strlen,
    strncmp,
};
use crate::src::format::bytes::format_message_with;
use crate::src::format::{
    format_add, format_create_from_state, format_expand_cstring, format_free,
    format_pretty_time_cstring, format_true,
};
use crate::src::grid::grid_default_cell;
use crate::src::hooks::{
    hooks_add_event, hooks_is_event, hooks_monitor_get_fire_count, hooks_monitor_get_fire_time,
    hooks_monitor_to_cstring,
};
use crate::src::key_bindings::{
    key_bindings_add, key_bindings_first, key_bindings_first_table, key_bindings_get,
    key_bindings_get_default, key_bindings_get_table, key_bindings_next, key_bindings_next_table,
    key_bindings_remove, key_bindings_reset, key_bindings_set_note,
};
use crate::src::key_string::{key_string_format, key_string_parse_cstr};
use crate::src::mode_tree::{
    mode_tree_add, mode_tree_build, mode_tree_count_tagged, mode_tree_draw,
    mode_tree_draw_as_parent, mode_tree_each_tagged, mode_tree_free, mode_tree_get_current,
    mode_tree_get_current_name, mode_tree_key, mode_tree_no_tag, mode_tree_remove,
    mode_tree_resize, mode_tree_set_prompt, mode_tree_start, mode_tree_up, mode_tree_zoom,
};
use crate::src::options::options_table_entry;
use crate::src::options::{
    options_array_first, options_array_get, options_array_get_index, options_array_item_key,
    options_array_next, options_array_set, options_create, options_default,
    options_default_to_cstring, options_first, options_free, options_from_string, options_get,
    options_get_fire_count, options_get_fire_time, options_get_monitor_data, options_get_number,
    options_get_only, options_get_parent, options_match_owned, options_name, options_next,
    options_owner, options_push_changes, options_remove_or_default, options_set_number,
    options_set_string, options_to_cstring, options_to_string,
};
use crate::src::screen_write::{
    screen_write_box, screen_write_clearcharacter, screen_write_cursormove, screen_write_nputs,
    screen_write_start, screen_write_stop, screen_write_text,
};
use crate::src::shared::abi::*;
use crate::src::shared::abi::{__int32_t, ssize_t};
use crate::src::shared::arguments::args;
use crate::src::shared::client::client;
use crate::src::shared::colour::{COLOUR_FLAG_THEME, COLOUR_THEME_LIGHT_GREY};
use crate::src::shared::command::cmd_parse_input;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd_find_state, cmdq_item};
use crate::src::shared::environment::ENVIRON_HIDDEN;
use crate::src::shared::environment::{environ, environ_entry};
use crate::src::shared::format::format_tree;
use crate::src::shared::grid::*;
use crate::src::shared::key::KEY_BINDING_REPEAT;
use crate::src::shared::key::*;
use crate::src::shared::key::{key_binding, key_table};
use crate::src::shared::layout::*;
use crate::src::shared::limits::INT_MAX;
use crate::src::shared::menu::menu_item;
use crate::src::shared::mode_tree::{
    mode_tree_data, mode_tree_help_info, mode_tree_item, mode_tree_prompt_input_cb,
};
use crate::src::shared::mouse::mouse_event;
use crate::src::shared::options::*;
use crate::src::shared::options::{options, options_array_item, options_entry, options_value};
use crate::src::shared::options::{
    OPTIONS_TABLE_IS_ARRAY, OPTIONS_TABLE_IS_COLOUR, OPTIONS_TABLE_IS_HOOK, OPTIONS_TABLE_IS_STYLE,
    OPTIONS_TABLE_PANE, OPTIONS_TABLE_SERVER, OPTIONS_TABLE_SESSION, OPTIONS_TABLE_WINDOW,
};
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::PANE_REDRAW;
use crate::src::shared::prompt::*;
use crate::src::shared::prompt::{
    prompt_free_cb, prompt_result, PROMPT_ACCEPT, PROMPT_CLOSE, PROMPT_NOFORMAT, PROMPT_SINGLE,
};
use crate::src::shared::screen::screen;
use crate::src::shared::screen_write::screen_write_ctx;
use crate::src::shared::session::session;
use crate::src::shared::sort::sort_criteria;
use crate::src::shared::spawn::spawn_editor_state;
use crate::src::shared::window::{window, window_mode, window_mode_entry, winlink};
use crate::src::spawn::{spawn_cancel_editor, spawn_editor, spawn_get_editor_pid};
use crate::src::status::status_message_set;
use crate::src::style::style_apply;
use crate::src::tmux::{global_environ, global_options, global_s_options, global_w_options};
use crate::src::window::{window_pane_find_by_id, window_pane_index, window_pane_reset_mode};

fn window_customize_uppercase_cause(cause: &mut Option<CString>) {
    if let Some(message) = cause.take() {
        let mut bytes = message.into_bytes();
        if let Some(first) = bytes.first_mut() {
            first.make_ascii_uppercase();
        }
        *cause = Some(CString::new(bytes).expect("error message has no interior NUL"));
    }
}

#[repr(C)]
pub struct window_customize_modedata {
    pub wp: *mut window_pane,
    pub dead: ::core::ffi::c_int,
    pub data: *mut mode_tree_data,
    pub editor: *mut spawn_editor_state,
    pub edit: *mut window_customize_editdata,
    pub format: CString,
    pub hide_global: ::core::ffi::c_int,
    pub hide_default: ::core::ffi::c_int,
    pub prompt_flags: ::core::ffi::c_int,
    item_list: Vec<Box<window_customize_itemdata>>,
    pub fs: cmd_find_state,
    pub change: window_customize_change,
}
pub type window_customize_change = ::core::ffi::c_uint;
pub const WINDOW_CUSTOMIZE_RESET: window_customize_change = 1;
pub const WINDOW_CUSTOMIZE_UNSET: window_customize_change = 0;
#[repr(C)]
pub struct window_customize_itemdata {
    pub data: *mut window_customize_modedata,
    pub type_0: window_customize_item_type,
    pub option_type: window_customize_option_type,
    pub scope: window_customize_scope,
    pub table: Option<std::ffi::CString>,
    pub key: key_code,
    pub oo: *mut options,
    pub environ: *mut environ,
    pub environ_flags: ::core::ffi::c_int,
    pub name: Option<std::ffi::CString>,
    pub array_key: Option<std::ffi::CString>,
}

impl window_customize_itemdata {
    fn new() -> Self {
        window_customize_itemdata {
            data: ::core::ptr::null_mut(),
            type_0: 0,
            option_type: 0,
            scope: 0,
            table: None,
            key: 0,
            oo: ::core::ptr::null_mut(),
            environ: ::core::ptr::null_mut(),
            environ_flags: 0,
            name: None,
            array_key: None,
        }
    }
}

// All callers pass an item created as a window_customize_itemdata. C-facing fields
// remain borrowed pointers, invalidated only when the owner is dropped.
fn window_customize_set_table(item: &mut window_customize_itemdata, value: Option<&CStr>) {
    item.table = value.map(CStr::to_owned);
}

fn window_customize_set_name(item: &mut window_customize_itemdata, value: Option<&CStr>) {
    item.name = value.map(CStr::to_owned);
}

fn window_customize_set_item_array_key(item: &mut window_customize_itemdata, value: Option<&CStr>) {
    item.array_key = value.map(CStr::to_owned);
}

fn window_customize_new_item() -> *mut window_customize_itemdata {
    Box::into_raw(Box::new(window_customize_itemdata::new())) as *mut window_customize_itemdata
}
pub type window_customize_scope = ::core::ffi::c_uint;
pub const WINDOW_CUSTOMIZE_SESSION_ENVIRONMENT: window_customize_scope = 9;
pub const WINDOW_CUSTOMIZE_GLOBAL_ENVIRONMENT: window_customize_scope = 8;
pub const WINDOW_CUSTOMIZE_PANE: window_customize_scope = 7;
pub const WINDOW_CUSTOMIZE_WINDOW: window_customize_scope = 6;
pub const WINDOW_CUSTOMIZE_GLOBAL_WINDOW: window_customize_scope = 5;
pub const WINDOW_CUSTOMIZE_SESSION: window_customize_scope = 4;
pub const WINDOW_CUSTOMIZE_GLOBAL_SESSION: window_customize_scope = 3;
pub const WINDOW_CUSTOMIZE_SERVER: window_customize_scope = 2;
pub const WINDOW_CUSTOMIZE_KEY: window_customize_scope = 1;
pub const WINDOW_CUSTOMIZE_NONE: window_customize_scope = 0;
pub type window_customize_option_type = ::core::ffi::c_uint;
pub const WINDOW_CUSTOMIZE_HOOKS: window_customize_option_type = 1;
pub const WINDOW_CUSTOMIZE_OPTIONS: window_customize_option_type = 0;
pub type window_customize_item_type = ::core::ffi::c_uint;
pub const WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT: window_customize_item_type = 2;
pub const WINDOW_CUSTOMIZE_ITEM_KEY: window_customize_item_type = 1;
pub const WINDOW_CUSTOMIZE_ITEM_OPTION: window_customize_item_type = 0;
#[repr(C)]
pub struct window_customize_editdata {
    pub wp_id: u_int,
    pub edit_type: window_customize_edit_type,
    pub item: *mut window_customize_itemdata,
    pub editor: *mut spawn_editor_state,
}
pub type window_customize_edit_type = ::core::ffi::c_uint;
pub const WINDOW_CUSTOMIZE_EDIT_ENVIRONMENT: window_customize_edit_type = 3;
pub const WINDOW_CUSTOMIZE_EDIT_KEY_NOTE: window_customize_edit_type = 2;
pub const WINDOW_CUSTOMIZE_EDIT_KEY_COMMAND: window_customize_edit_type = 1;
pub const WINDOW_CUSTOMIZE_EDIT_OPTION: window_customize_edit_type = 0;

#[inline]
unsafe fn tolower(mut __c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return if __c >= -(128 as ::core::ffi::c_int) && __c < 256 as ::core::ffi::c_int {
        *(*__ctype_tolower_loc()).offset(__c as isize) as ::core::ffi::c_int
    } else {
        __c
    };
}

pub const WINDOW_CUSTOMIZE_DEFAULT_FORMAT: [::core::ffi::c_char; 227] = unsafe {
    ::core::mem::transmute::<
        [u8; 227],
        [::core::ffi::c_char; 227],
    >(
        *b"#{?is_option,#{?option_is_global,,#[reverse](#{option_scope})#[default] }#[fg=themelightgrey]#[ignore]#{option_value}#{?option_unit, #{option_unit},},#{?is_environment,#[fg=themelightgrey]#[ignore]#{environment_value},#{key}}}\0",
    )
};
static mut window_customize_menu_items: [menu_item; 12] = [
    menu_item {
        name: b"Select\0" as *const u8 as *const ::core::ffi::c_char,
        key: '\r' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Edit\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'e' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Expand\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_NONE as ::core::ffi::c_ulong as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Tag\0" as *const u8 as *const ::core::ffi::c_char,
        key: 't' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Tag All\0" as *const u8 as *const ::core::ffi::c_char,
        key: '\u{14}' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Tag None\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'T' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_NONE as ::core::ffi::c_ulong as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Changed Only\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'C' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_NONE as ::core::ffi::c_ulong as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Cancel\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'q' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        key: KEYC_NONE as ::core::ffi::c_ulong as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
];
pub static mut window_customize_mode: window_mode = {
    window_mode {
        name: c"options-mode",
        default_format: WINDOW_CUSTOMIZE_DEFAULT_FORMAT.as_ptr(),
        flags: 0,
        init: Some(
            window_customize_init
                as unsafe fn(
                    *mut window_mode_entry,
                    *mut cmdq_item,
                    *mut cmd_find_state,
                    *mut args,
                ) -> *mut screen,
        ),
        free: Some(window_customize_free as unsafe fn(*mut window_mode_entry) -> ()),
        resize: Some(
            window_customize_resize as unsafe fn(*mut window_mode_entry, u_int, u_int) -> (),
        ),
        update: Some(window_customize_update as unsafe fn(*mut window_mode_entry) -> ()),
        style_changed: None,
        key: Some(
            window_customize_key
                as unsafe fn(
                    *mut window_mode_entry,
                    *mut client,
                    *mut session,
                    *mut winlink,
                    key_code,
                    *mut mouse_event,
                ) -> (),
        ),
        key_table: None,
        command: None,
        formats: None,
        get_screen: None,
    }
};
const CUSTOMIZE_SERVER_OPTIONS: u_int = 1;
const CUSTOMIZE_SESSION_OPTIONS: u_int = 2;
const CUSTOMIZE_WINDOW_OPTIONS: u_int = 3;
const CUSTOMIZE_SESSION_HOOKS: u_int = 4;
const CUSTOMIZE_WINDOW_HOOKS: u_int = 5;
const CUSTOMIZE_GLOBAL_ENVIRONMENT: u_int = 6;
const CUSTOMIZE_SESSION_ENVIRONMENT: u_int = 7;

unsafe fn window_customize_get_tag(
    o: *mut options_entry,
    a: *mut options_array_item,
    oe: *const options_table_entry,
) -> uint64_t {
    if !a.is_null() {
        return a as uint64_t;
    }
    if oe.is_null() {
        return o as uint64_t;
    }
    let offset =
        oe.offset_from((&raw const crate::src::options_table::options_table).cast()) as u64;
    (2_u64 << 62) | (offset << 32) | 1
}

fn window_customize_top_tag(group: u_int) -> uint64_t {
    let (kind, table, scope) = match group {
        CUSTOMIZE_SERVER_OPTIONS => (0, OPTIONS_TABLE_SERVER, 1),
        CUSTOMIZE_SESSION_OPTIONS => (0, OPTIONS_TABLE_SESSION, 1),
        CUSTOMIZE_WINDOW_OPTIONS => (0, OPTIONS_TABLE_WINDOW, 1),
        CUSTOMIZE_SESSION_HOOKS => (1, OPTIONS_TABLE_SESSION, 1),
        CUSTOMIZE_WINDOW_HOOKS => (1, OPTIONS_TABLE_WINDOW, 1),
        CUSTOMIZE_GLOBAL_ENVIRONMENT => (2, 0, 1),
        CUSTOMIZE_SESSION_ENVIRONMENT => (2, 0, 3),
        _ => unreachable!("unknown customize section"),
    };
    (3_u64 << 62) | ((kind as u64) << 8) | ((table as u64) << 1) | scope
}

fn window_customize_key_tag(ptr: *const ::core::ffi::c_void, field: u_int) -> uint64_t {
    ptr as uint64_t | field as uint64_t
}
unsafe fn window_customize_get_tree(
    mut scope: window_customize_scope,
    mut fs: *mut cmd_find_state,
) -> *mut options {
    match scope as ::core::ffi::c_uint {
        0 | 1 => return ::core::ptr::null_mut::<options>(),
        2 => return global_options,
        3 => return global_s_options,
        4 => return (*(*fs).s).options,
        5 => return global_w_options,
        6 => return (*(*fs).w).options,
        7 => return (*(*fs).wp).options,
        8 | 9 => return ::core::ptr::null_mut::<options>(),
        _ => {}
    }
    return ::core::ptr::null_mut::<options>();
}
unsafe fn window_customize_get_environment(
    mut scope: window_customize_scope,
    mut fs: *mut cmd_find_state,
) -> *mut environ {
    match scope as ::core::ffi::c_uint {
        8 => return global_environ,
        9 => return (*(*fs).s).environ,
        _ => return ::core::ptr::null_mut::<environ>(),
    };
}
unsafe fn window_customize_check_item(
    mut data: *mut window_customize_modedata,
    mut item: *mut window_customize_itemdata,
    mut fsp: *mut cmd_find_state,
) -> ::core::ffi::c_int {
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    if fsp.is_null() {
        fsp = &raw mut fs;
    }
    if cmd_find_valid_state(&raw mut (*data).fs) != 0 {
        cmd_find_copy_state(fsp, &raw mut (*data).fs);
    } else {
        cmd_find_from_pane(fsp, (*data).wp, 0 as ::core::ffi::c_int);
    }
    if (*item).type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ((*item).environ == window_customize_get_environment((*item).scope, fsp))
            as ::core::ffi::c_int;
    }
    return ((*item).oo == window_customize_get_tree((*item).scope, fsp)) as ::core::ffi::c_int;
}
unsafe fn window_customize_get_key(
    mut item: *mut window_customize_itemdata,
    mut ktp: *mut *mut key_table,
    mut bdp: *mut *mut key_binding,
) -> ::core::ffi::c_int {
    let mut kt: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    kt = key_bindings_get_table(
        ((*item).table)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        0 as ::core::ffi::c_int,
    );
    if kt.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    bd = key_bindings_get(kt, (*item).key);
    if bd.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if !ktp.is_null() {
        *ktp = kt;
    }
    if !bdp.is_null() {
        *bdp = bd;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe fn window_customize_scope_text(
    scope: window_customize_scope,
    fs: &cmd_find_state,
) -> CString {
    let mut idx: u_int = 0;
    match scope as ::core::ffi::c_uint {
        7 => {
            window_pane_index(fs.wp, &raw mut idx);
            CString::new(format!("pane {idx}")).expect("pane index contains no NUL")
        }
        4 | 9 => {
            let mut bytes = b"session ".to_vec();
            bytes.extend_from_slice((*fs.s).name.as_bytes());
            CString::new(bytes).expect("session name contains no NUL")
        }
        6 => {
            CString::new(format!("window {}", (*fs.wl).idx)).expect("window index contains no NUL")
        }
        _ => CString::new(Vec::new()).expect("empty scope text"),
    }
}
unsafe fn window_customize_write_hook_fire(
    mut ctx: *mut screen_write_ctx,
    mut cx: u_int,
    mut sx: u_int,
    mut sy: u_int,
    mut o: *mut options_entry,
) -> ::core::ffi::c_int {
    let mut fire_count: u_int = 0;
    let mut fire_time: time_t = 0;
    if !options_get_monitor_data(o).is_null() {
        fire_count = hooks_monitor_get_fire_count(o);
        fire_time = hooks_monitor_get_fire_time(o);
    } else {
        fire_count = options_get_fire_count(o);
        fire_time = options_get_fire_time(o);
    }
    if fire_time != 0 as time_t {
        let fire_time_string = format_pretty_time_cstring(fire_time);
        if screen_write_text(
            &mut *ctx,
            cx,
            sx,
            sy,
            0 as ::core::ffi::c_int,
            &grid_default_cell,
            |out| {
                write!(
                    out,
                    "This hook has been fired {} times, last ",
                    (fire_count) as u32
                )?;
                write_cstr(out, fire_time_string.as_ptr())?;
                out.write_all(b".")
            },
        ) == 0
        {
            return 0 as ::core::ffi::c_int;
        }
        return 1 as ::core::ffi::c_int;
    }
    return screen_write_text(
        &mut *ctx,
        cx,
        sx,
        sy,
        0 as ::core::ffi::c_int,
        &grid_default_cell,
        |out| {
            write!(
                out,
                "This hook has been fired {} times.",
                (fire_count) as u32
            )
        },
    );
}
unsafe fn window_customize_add_item(
    mut data: *mut window_customize_modedata,
) -> *mut window_customize_itemdata {
    let mut owner = Box::new(window_customize_itemdata::new());
    let item = &mut *owner as *mut window_customize_itemdata;
    (*data).item_list.push(owner);
    item
}
unsafe fn window_customize_write_value(
    mut ctx: *mut screen_write_ctx,
    mut cx: u_int,
    mut sx: u_int,
    mut sy: u_int,
    mut more: ::core::ffi::c_int,
    mut label: *const ::core::ffi::c_char,
    write: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
) -> ::core::ffi::c_int {
    let mut s: *mut screen = (*ctx).s;
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
    let mut cy: u_int = (*s).cy;
    let mut retval: ::core::ffi::c_int = 0;
    if sy == 0 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    if screen_write_text(
        &mut *ctx,
        cx,
        sx,
        sy,
        1 as ::core::ffi::c_int,
        &grid_default_cell,
        |out| write_cstr(out, label),
    ) == 0
    {
        return 0 as ::core::ffi::c_int;
    }
    if (*s).cy.wrapping_sub(cy) >= sy {
        return 0 as ::core::ffi::c_int;
    }
    sy = sy.wrapping_sub((*s).cy.wrapping_sub(cy));
    memcpy(
        &raw mut gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    gc.fg = COLOUR_THEME_LIGHT_GREY as ::core::ffi::c_int | COLOUR_FLAG_THEME;
    let value = format_message_with(write);
    retval = screen_write_text(&mut *ctx, cx, sx, sy, more, &gc, |out| {
        write_cstr(out, value.as_ptr())
    });
    return retval;
}
unsafe fn window_customize_free_item(mut item: *mut window_customize_itemdata) {
    drop(Box::from_raw(item as *mut window_customize_itemdata));
}
unsafe fn window_customize_copy_item(
    mut item: *mut window_customize_itemdata,
) -> *mut window_customize_itemdata {
    let mut new_item: *mut window_customize_itemdata =
        ::core::ptr::null_mut::<window_customize_itemdata>();
    new_item = window_customize_new_item();
    (*new_item).data = (*item).data;
    (*new_item).type_0 = (*item).type_0;
    (*new_item).option_type = (*item).option_type;
    (*new_item).scope = (*item).scope;
    (*new_item).key = (*item).key;
    (*new_item).oo = (*item).oo;
    (*new_item).environ = (*item).environ;
    (*new_item).environ_flags = (*item).environ_flags;
    window_customize_set_table(&mut *new_item, (*item).table.as_deref());
    window_customize_set_name(&mut *new_item, (*item).name.as_deref());
    window_customize_set_item_array_key(&mut *new_item, (*item).array_key.as_deref());
    return new_item;
}
unsafe fn window_customize_finish_edit(mut ed: *mut window_customize_editdata) {
    window_customize_free_item((*ed).item);
    drop(Box::from_raw(ed));
}
unsafe fn window_customize_draw_waiting(mut data: *mut window_customize_modedata) {
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: ::core::ptr::null_mut::<window_pane>(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
    };
    let mut s: *mut screen = (*(*data).wp).screen;
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
    let mut text: [::core::ffi::c_char; 128] = [0; 128];
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut box_w: u_int = 0;
    let mut box_h: u_int = 0;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut text_x: u_int = 0;
    let mut textlen: size_t = 0;
    let mut pid: pid_t = 0;
    if (*data).editor.is_null() {
        return;
    }
    sx = (*s).grid().sx;
    sy = (*s).grid().sy;
    if sx == 0 as u_int || sy == 0 as u_int {
        return;
    }
    pid = spawn_get_editor_pid((*data).editor);
    if pid == -(1 as ::core::ffi::c_int) {
        xformat(&mut text, format_args!("WAITING FOR EDITOR"));
    } else {
        xformat(
            &mut text,
            format_args!("WAITING FOR EDITOR (PID {})", pid as ::core::ffi::c_long),
        );
    }
    textlen = strlen(&raw mut text as *mut ::core::ffi::c_char);
    box_w = textlen.wrapping_add(4 as size_t) as u_int;
    box_h = 3 as u_int;
    if sx < box_w || sy < box_h {
        return;
    }
    x = sx.wrapping_sub(box_w).wrapping_div(2 as u_int);
    y = sy.wrapping_sub(box_h).wrapping_div(2 as u_int);
    text_x = (x as size_t).wrapping_add(
        (box_w as size_t)
            .wrapping_sub(textlen)
            .wrapping_div(2 as size_t),
    ) as u_int;
    memcpy(
        &raw mut gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    screen_write_start(&mut ctx, s);
    screen_write_cursormove(
        &raw mut ctx,
        x as ::core::ffi::c_int,
        y as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_box(
        &mut ctx,
        box_w,
        box_h,
        BOX_LINES_DEFAULT,
        Some(&gc),
        None,
    );
    screen_write_cursormove(
        &raw mut ctx,
        x.wrapping_add(1 as u_int) as ::core::ffi::c_int,
        y.wrapping_add(1 as u_int) as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_clearcharacter(&raw mut ctx, box_w.wrapping_sub(2 as u_int), gc.bg as u_int);
    screen_write_cursormove(
        &raw mut ctx,
        text_x as ::core::ffi::c_int,
        y.wrapping_add(1 as u_int) as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_nputs(
        &mut ctx,
        box_w.wrapping_sub(2 as u_int) as ssize_t,
        &gc,
        |out| write_cstr(out, &raw mut text as *mut ::core::ffi::c_char),
    );
    screen_write_stop(&mut ctx);
}
unsafe fn window_customize_set_option_value(
    mut item: *mut window_customize_itemdata,
    mut s: *const ::core::ffi::c_char,
    mut cause: *mut Option<CString>,
) -> ::core::ffi::c_int {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    let mut oo: *mut options = (*item).oo;
    let mut name: *const ::core::ffi::c_char = ((*item).name)
        .as_ref()
        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut());
    let mut array_key: *const ::core::ffi::c_char = ((*item).array_key)
        .as_ref()
        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut());
    let mut idx: u_int = 0;
    let mut keybuf: [::core::ffi::c_char; 32] = [0; 32];
    o = options_get(oo, name);
    if o.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    oe = options_table_entry(o);
    if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
        if array_key.is_null() {
            idx = 0 as u_int;
            while idx < INT_MAX as u_int {
                if options_array_get_index(o, idx).is_null() {
                    break;
                }
                idx = idx.wrapping_add(1);
            }
            xformat(&mut keybuf, format_args!("{}", idx as u32));
            array_key = &raw mut keybuf as *mut ::core::ffi::c_char;
        }
        if options_array_set(o, array_key, s, 0 as ::core::ffi::c_int, cause)
            != 0 as ::core::ffi::c_int
        {
            return -(1 as ::core::ffi::c_int);
        }
    } else if options_from_string(oo, oe, name, s, 0 as ::core::ffi::c_int, cause)
        != 0 as ::core::ffi::c_int
    {
        return -(1 as ::core::ffi::c_int);
    }
    if (*item).option_type as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_HOOKS as ::core::ffi::c_int as ::core::ffi::c_uint
        && *name as ::core::ffi::c_int == '@' as i32
    {
        hooks_add_event(name);
    }
    options_push_changes(
        ((*item).name)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
    );
    return 0 as ::core::ffi::c_int;
}
unsafe fn window_customize_option_editable(
    mut data: *mut window_customize_modedata,
    mut item: *mut window_customize_itemdata,
) -> ::core::ffi::c_int {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    if (*item).type_0 as ::core::ffi::c_uint
        != WINDOW_CUSTOMIZE_ITEM_OPTION as ::core::ffi::c_int as ::core::ffi::c_uint
        || window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0
    {
        return 0 as ::core::ffi::c_int;
    }
    o = options_get(
        (*item).oo,
        ((*item).name)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
    );
    if o.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    oe = options_table_entry(o);
    if oe.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    if (*oe).type_0 as ::core::ffi::c_uint
        == OPTIONS_TABLE_FLAG as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*oe).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_CHOICE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe fn window_customize_set_command_value(
    item: *mut window_customize_itemdata,
    s: *const ::core::ffi::c_char,
    cause: *mut Option<CString>,
) -> ::core::ffi::c_int {
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    if window_customize_get_key(item, ::core::ptr::null_mut::<*mut key_table>(), &raw mut bd) == 0 {
        return -(1 as ::core::ffi::c_int);
    }
    let pr = cmd_parse_from_string(
        CStr::from_ptr(s),
        ::core::ptr::null_mut::<cmd_parse_input>(),
    );
    if pr.status == CMD_PARSE_ERROR {
        if !cause.is_null() {
            *cause = pr.error;
        }
        return -(1 as ::core::ffi::c_int);
    }
    cmd_list_free((*bd).cmdlist);
    (*bd).cmdlist = pr.cmdlist;
    return 0 as ::core::ffi::c_int;
}
unsafe fn window_customize_set_note_value(
    mut item: *mut window_customize_itemdata,
    mut s: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    if window_customize_get_key(item, ::core::ptr::null_mut::<*mut key_table>(), &raw mut bd) == 0 {
        return -(1 as ::core::ffi::c_int);
    }
    if *s as ::core::ffi::c_int == '\0' as i32 {
        key_bindings_set_note(bd, None);
    } else {
        key_bindings_set_note(bd, Some(CStr::from_ptr(s)));
    }
    return 0 as ::core::ffi::c_int;
}
unsafe fn window_customize_set_environment_value(
    mut item: *mut window_customize_itemdata,
    mut s: *const ::core::ffi::c_char,
) {
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut flags: ::core::ffi::c_int = 0;
    flags = (*item).environ_flags;
    envent = environ_find(
        (*item).environ,
        ((*item).name)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
    );
    if !envent.is_null() {
        flags = (*envent).flags;
    }
    environ_set(
        (*item).environ,
        ((*item).name)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        flags,
        |out| write_cstr(out, s),
    );
}
unsafe fn window_customize_option_is_changed(
    mut o: *mut options_entry,
    mut array_key: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut oe: *const options_table_entry = options_table_entry(o);
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    let mut defaults: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut ov: *mut options_value = ::core::ptr::null_mut::<options_value>();
    let mut default_ov: *mut options_value = ::core::ptr::null_mut::<options_value>();
    let mut changed: ::core::ffi::c_int = 0;
    if oe.is_null() || !options_get_monitor_data(o).is_null() {
        return 1 as ::core::ffi::c_int;
    }
    if *options_name(o) as ::core::ffi::c_int == '@' as i32 && hooks_is_event(options_name(o)) != 0
    {
        return 1 as ::core::ffi::c_int;
    }
    if (*oe).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
        oo = options_create(::core::ptr::null_mut::<options>());
        defaults = options_default(oo, oe);
        if !array_key.is_null() {
            ov = options_array_get(o, array_key);
            default_ov = options_array_get(defaults, array_key);
            if ov.is_null() || default_ov.is_null() {
                changed = (ov != default_ov) as ::core::ffi::c_int;
                options_free(oo);
                return changed;
            }
        }
        let value = options_to_cstring(o, array_key, 0 as ::core::ffi::c_int);
        let default_value = options_to_cstring(defaults, array_key, 0 as ::core::ffi::c_int);
        changed = (strcmp(value.as_ptr(), default_value.as_ptr()) != 0 as ::core::ffi::c_int)
            as ::core::ffi::c_int;
        drop(value);
        drop(default_value);
        options_free(oo);
        return changed;
    }
    let value = options_to_cstring(
        o,
        ::core::ptr::null::<::core::ffi::c_char>(),
        0 as ::core::ffi::c_int,
    );
    let default_value = options_default_to_cstring(&*oe);
    changed = (strcmp(value.as_ptr(), default_value.as_ptr()) != 0 as ::core::ffi::c_int)
        as ::core::ffi::c_int;
    drop(value);
    drop(default_value);
    return changed;
}
unsafe fn window_customize_key_is_changed(
    mut kt: *mut key_table,
    mut bd: *mut key_binding,
) -> ::core::ffi::c_int {
    let mut default_bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    default_bd = key_bindings_get_default(kt, (*bd).key);
    if default_bd.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    if (*bd).flags != (*default_bd).flags {
        return 1 as ::core::ffi::c_int;
    }
    if ((*bd).note
        == if (NULL as *const ::core::ffi::c_char).is_null() {
            None
        } else {
            Some(::std::ffi::CStr::from_ptr(NULL as *const ::core::ffi::c_char).to_owned())
        }) as ::core::ffi::c_int
        != ((*default_bd).note
            == if (NULL as *const ::core::ffi::c_char).is_null() {
                None
            } else {
                Some(::std::ffi::CStr::from_ptr(NULL as *const ::core::ffi::c_char).to_owned())
            }) as ::core::ffi::c_int
    {
        return 1 as ::core::ffi::c_int;
    }
    if !(*bd).note.is_none()
        && strcmp(
            ((*bd).note)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            ((*default_bd).note)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        ) != 0 as ::core::ffi::c_int
    {
        return 1 as ::core::ffi::c_int;
    }
    let cmd = cmd_list_print_cstring(&*(*bd).cmdlist, 0);
    let default_cmd = cmd_list_print_cstring(&*(*default_bd).cmdlist, 0);
    return (cmd.as_bytes() != default_cmd.as_bytes()) as ::core::ffi::c_int;
}
unsafe fn window_customize_build_array(
    mut data: *mut window_customize_modedata,
    mut top: *mut mode_tree_item,
    mut scope: window_customize_scope,
    mut o: *mut options_entry,
    mut ft: *mut format_tree,
) -> u_int {
    let mut oe: *const options_table_entry = options_table_entry(o);
    let mut oo: *mut options = options_owner(o);
    let mut item: *mut window_customize_itemdata =
        ::core::ptr::null_mut::<window_customize_itemdata>();
    let mut ai: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut array_key: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut count: u_int = 0 as u_int;
    ai = options_array_first(o);
    while !ai.is_null() {
        array_key = options_array_item_key(ai);
        if (*data).hide_default != 0 && window_customize_option_is_changed(o, array_key) == 0 {
            ai = options_array_next(ai);
        } else {
            let mut name = CStr::from_ptr(options_name(o)).to_bytes().to_vec();
            name.push(b'[');
            name.extend_from_slice(CStr::from_ptr(array_key).to_bytes());
            name.push(b']');
            let name = CString::new(name).expect("option name and array key contain no NUL");
            format_add(
                ft,
                b"option_name\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write_cstr(out, name.as_ptr()),
            );
            let value = options_to_cstring(o, array_key, 0 as ::core::ffi::c_int);
            format_add(
                ft,
                b"option_value\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write_cstr(out, value.as_ptr()),
            );
            item = window_customize_add_item(data);
            (*item).type_0 = WINDOW_CUSTOMIZE_ITEM_OPTION;
            if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_HOOK != 0 {
                (*item).option_type = WINDOW_CUSTOMIZE_HOOKS;
            }
            (*item).scope = scope;
            (*item).oo = oo;
            window_customize_set_name(&mut *item, Some(CStr::from_ptr(options_name(o))));
            window_customize_set_item_array_key(
                &mut *item,
                if array_key.is_null() {
                    None
                } else {
                    Some(CStr::from_ptr(array_key))
                },
            );
            let text = format_expand_cstring(ft, (*data).format.as_ptr());
            mode_tree_add(
                (*data).data,
                top,
                item as *mut ::core::ffi::c_void,
                window_customize_get_tag(o, ai, oe),
                name.as_ptr(),
                text.as_ptr(),
                -(1 as ::core::ffi::c_int),
            );
            drop(value);
            count = count.wrapping_add(1);
            ai = options_array_next(ai);
        }
    }
    return count;
}
unsafe fn window_customize_build_option(
    mut data: *mut window_customize_modedata,
    mut top: *mut mode_tree_item,
    mut scope: window_customize_scope,
    mut o: *mut options_entry,
    mut ft: *mut format_tree,
    mut filter: *const ::core::ffi::c_char,
    mut fs: *mut cmd_find_state,
    mut type_0: window_customize_option_type,
) -> u_int {
    let mut oe: *const options_table_entry = options_table_entry(o);
    let mut oo: *mut options = options_owner(o);
    let mut name: *const ::core::ffi::c_char = options_name(o);
    let mut item: *mut window_customize_itemdata =
        ::core::ptr::null_mut::<window_customize_itemdata>();
    let mut global: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut array: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut is_hook: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut is_monitor: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut is_user_hook: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut is_any_hook: ::core::ffi::c_int = 0;
    if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_HOOK != 0 {
        is_hook = 1 as ::core::ffi::c_int;
    }
    if !options_get_monitor_data(o).is_null() {
        is_monitor = 1 as ::core::ffi::c_int;
    }
    if *name as ::core::ffi::c_int == '@' as i32 && hooks_is_event(name) != 0 {
        is_user_hook = 1 as ::core::ffi::c_int;
    }
    is_any_hook = (is_hook != 0 || is_monitor != 0 || is_user_hook != 0) as ::core::ffi::c_int;
    match type_0 as ::core::ffi::c_uint {
        0 => {
            if is_any_hook != 0 {
                return 0 as u_int;
            }
        }
        1 => {
            if is_any_hook == 0 {
                return 0 as u_int;
            }
        }
        _ => {}
    }
    if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
        array = 1 as ::core::ffi::c_int;
    }
    if scope as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_SERVER as ::core::ffi::c_int as ::core::ffi::c_uint
        || scope as ::core::ffi::c_uint
            == WINDOW_CUSTOMIZE_GLOBAL_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
        || scope as ::core::ffi::c_uint
            == WINDOW_CUSTOMIZE_GLOBAL_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        global = 1 as ::core::ffi::c_int;
    }
    if (*data).hide_global != 0 && global != 0 {
        return 0 as u_int;
    }
    if (*data).hide_default != 0
        && window_customize_option_is_changed(o, ::core::ptr::null::<::core::ffi::c_char>()) == 0
    {
        return 0 as u_int;
    }
    format_add(
        ft,
        b"option_name\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write_cstr(out, name),
    );
    format_add(
        ft,
        b"option_is_global\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", (global) as i32),
    );
    format_add(
        ft,
        b"option_is_array\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", (array) as i32),
    );
    format_add(
        ft,
        b"option_is_hook\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", (is_hook) as i32),
    );
    format_add(
        ft,
        b"option_is_monitor\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", (is_monitor) as i32),
    );
    let scope_text = window_customize_scope_text(scope, &*fs);
    format_add(
        ft,
        b"option_scope\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write_cstr(out, scope_text.as_ptr()),
    );
    drop(scope_text);
    if !oe.is_null() && !(*oe).unit.is_null() {
        format_add(
            ft,
            b"option_unit\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write_cstr(out, (*oe).unit),
        );
    } else {
        format_add(
            ft,
            b"option_unit\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write_cstr(out, b"\0" as *const u8 as *const ::core::ffi::c_char),
        );
    }
    if is_monitor != 0 {
        if let Some(monitor) = hooks_monitor_to_cstring(o) {
            format_add(
                ft,
                b"option_monitor\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write_cstr(out, monitor.as_ptr()),
            );
        } else {
            format_add(
                ft,
                b"option_monitor\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write_cstr(out, b"\0" as *const u8 as *const ::core::ffi::c_char),
            );
        }
    } else {
        format_add(
            ft,
            b"option_monitor\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write_cstr(out, b"\0" as *const u8 as *const ::core::ffi::c_char),
        );
    }
    if array == 0 {
        let value = options_to_cstring(
            o,
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as ::core::ffi::c_int,
        );
        format_add(
            ft,
            b"option_value\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write_cstr(out, value.as_ptr()),
        );
    }
    if !filter.is_null() {
        let expanded = format_expand_cstring(ft, filter);
        if format_true(expanded.as_ptr()) == 0 {
            return 0 as u_int;
        }
    }
    item = window_customize_add_item(data);
    (*item).type_0 = WINDOW_CUSTOMIZE_ITEM_OPTION;
    (*item).option_type = type_0;
    (*item).oo = oo;
    (*item).scope = scope;
    window_customize_set_name(&mut *item, Some(CStr::from_ptr(name)));
    let text = (array == 0).then(|| format_expand_cstring(ft, (*data).format.as_ptr()));
    top = mode_tree_add(
        (*data).data,
        top,
        item as *mut ::core::ffi::c_void,
        window_customize_get_tag(o, ::core::ptr::null_mut(), oe),
        name,
        text.as_ref()
            .map_or(::core::ptr::null(), |text| text.as_ptr()),
        0 as ::core::ffi::c_int,
    ) as *mut mode_tree_item;
    if array == 0 {
        return 1 as u_int;
    }
    return (1 as u_int).wrapping_add(window_customize_build_array(data, top, scope, o, ft));
}
unsafe fn window_customize_find_user_options(oo: *mut options, list: &mut Vec<CString>) {
    let mut o = options_first(oo);
    while !o.is_null() {
        let name = CStr::from_ptr(options_name(o));
        if name.to_bytes().first() == Some(&b'@') && !list.iter().any(|entry| entry == name) {
            // Later row builders can call format callbacks before the list is exhausted.
            list.push(name.to_owned());
        }
        o = options_next(o);
    }
}
unsafe fn window_customize_build_options(
    mut data: *mut window_customize_modedata,
    mut title: *const ::core::ffi::c_char,
    group: u_int,
    mut scope0: window_customize_scope,
    mut oo0: *mut options,
    mut scope1: window_customize_scope,
    mut oo1: *mut options,
    mut scope2: window_customize_scope,
    mut oo2: *mut options,
    mut ft: *mut format_tree,
    mut filter: *const ::core::ffi::c_char,
    mut fs: *mut cmd_find_state,
    mut type_0: window_customize_option_type,
) {
    let mut top: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut loop_0: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut list = Vec::<CString>::new();
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut count: u_int = 0 as u_int;
    let mut scope: window_customize_scope = WINDOW_CUSTOMIZE_NONE;
    top = mode_tree_add(
        (*data).data,
        ::core::ptr::null_mut::<mode_tree_item>(),
        NULL,
        window_customize_top_tag(group),
        title,
        ::core::ptr::null::<::core::ffi::c_char>(),
        0 as ::core::ffi::c_int,
    ) as *mut mode_tree_item;
    mode_tree_no_tag(top);
    window_customize_find_user_options(oo0, &mut list);
    if !oo1.is_null() {
        window_customize_find_user_options(oo1, &mut list);
    }
    if !oo2.is_null() {
        window_customize_find_user_options(oo2, &mut list);
    }
    for name in &list {
        o = ::core::ptr::null_mut::<options_entry>();
        if !oo2.is_null() {
            o = options_get(oo2, name.as_ptr());
        }
        if o.is_null() && !oo1.is_null() {
            o = options_get(oo1, name.as_ptr());
        }
        if o.is_null() {
            o = options_get(oo0, name.as_ptr());
        }
        if options_owner(o) == oo2 {
            scope = scope2;
        } else if options_owner(o) == oo1 {
            scope = scope1;
        } else {
            scope = scope0;
        }
        count = count.wrapping_add(window_customize_build_option(
            data, top, scope, o, ft, filter, fs, type_0,
        ));
    }
    drop(list);
    loop_0 = options_first(oo0);
    while !loop_0.is_null() {
        name = options_name(loop_0);
        if *name as ::core::ffi::c_int == '@' as i32 {
            loop_0 = options_next(loop_0);
        } else {
            if !oo2.is_null() {
                o = options_get(oo2, name);
            } else if !oo1.is_null() {
                o = options_get(oo1, name);
            } else {
                o = loop_0;
            }
            if options_owner(o) == oo2 {
                scope = scope2;
            } else if options_owner(o) == oo1 {
                scope = scope1;
            } else {
                scope = scope0;
            }
            count = count.wrapping_add(window_customize_build_option(
                data, top, scope, o, ft, filter, fs, type_0,
            ));
            loop_0 = options_next(loop_0);
        }
    }
    if (*data).hide_default != 0 && count == 0 as u_int {
        mode_tree_remove((*data).data, top);
    }
}
fn window_customize_key_detail(value: &[u8]) -> CString {
    let mut text = b"#[fg=themelightgrey]#[ignore]".to_vec();
    text.extend_from_slice(value);
    CString::new(text).expect("key detail contains no NUL")
}

unsafe fn window_customize_build_keys(
    mut data: *mut window_customize_modedata,
    mut kt: *mut key_table,
    mut ft: *mut format_tree,
    mut filter: *const ::core::ffi::c_char,
    mut fs: *mut cmd_find_state,
) {
    let mut top: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut child: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut mti: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut item: *mut window_customize_itemdata =
        ::core::ptr::null_mut::<window_customize_itemdata>();
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut count: u_int = 0 as u_int;
    let mut title_bytes = b"Key Table - ".to_vec();
    title_bytes.extend_from_slice((*kt).name.as_bytes());
    let title = CString::new(title_bytes).expect("key table name contains no NUL");
    top = mode_tree_add(
        (*data).data,
        ::core::ptr::null_mut::<mode_tree_item>(),
        NULL,
        window_customize_key_tag(kt.cast(), 0),
        title.as_ptr(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        0 as ::core::ffi::c_int,
    ) as *mut mode_tree_item;
    mode_tree_no_tag(top);
    drop(title);
    ft = format_create_from_state(
        ::core::ptr::null_mut::<cmdq_item>(),
        ::core::ptr::null_mut::<client>(),
        fs,
    );
    format_add(
        ft,
        b"is_option\0" as *const u8 as *const ::core::ffi::c_char,
        |out| out.write_all(b"0"),
    );
    format_add(
        ft,
        b"is_key\0" as *const u8 as *const ::core::ffi::c_char,
        |out| out.write_all(b"1"),
    );
    format_add(
        ft,
        b"is_environment\0" as *const u8 as *const ::core::ffi::c_char,
        |out| out.write_all(b"0"),
    );
    bd = key_bindings_first(kt);
    while !bd.is_null() {
        if (*data).hide_default != 0 && window_customize_key_is_changed(kt, bd) == 0 {
            bd = key_bindings_next(bd);
        } else {
            let key_string = key_string_format((*bd).key, false);
            format_add(
                ft,
                b"key\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write_cstr(out, key_string.as_ptr()),
            );
            if !(*bd).note.is_none() {
                format_add(
                    ft,
                    b"key_note\0" as *const u8 as *const ::core::ffi::c_char,
                    |out| {
                        write_cstr(
                            out,
                            ((*bd).note)
                                .as_ref()
                                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                        )
                    },
                );
            }
            if !filter.is_null() {
                let expanded = format_expand_cstring(ft, filter);
                if format_true(expanded.as_ptr()) == 0 {
                    bd = key_bindings_next(bd);
                    continue;
                }
            }
            item = window_customize_add_item(data);
            (*item).type_0 = WINDOW_CUSTOMIZE_ITEM_KEY;
            (*item).scope = WINDOW_CUSTOMIZE_KEY;
            window_customize_set_table(&mut *item, Some((*kt).name.as_c_str()));
            (*item).key = (*bd).key;
            let key_string = key_string_format((*item).key, false);
            window_customize_set_name(&mut *item, Some(key_string.as_c_str()));
            let expanded = format_expand_cstring(ft, (*data).format.as_ptr());
            child = mode_tree_add(
                (*data).data,
                top,
                item as *mut ::core::ffi::c_void,
                window_customize_key_tag(bd.cast(), 0),
                expanded.as_ptr(),
                ::core::ptr::null::<::core::ffi::c_char>(),
                0 as ::core::ffi::c_int,
            ) as *mut mode_tree_item;
            let tmp = cmd_list_print_cstring(&*(*bd).cmdlist, 0);
            let text = window_customize_key_detail(tmp.as_bytes());
            mti = mode_tree_add(
                (*data).data,
                child,
                item as *mut ::core::ffi::c_void,
                window_customize_key_tag(bd.cast(), 1),
                b"Command\0" as *const u8 as *const ::core::ffi::c_char,
                text.as_ptr(),
                -(1 as ::core::ffi::c_int),
            ) as *mut mode_tree_item;
            mode_tree_draw_as_parent(mti);
            mode_tree_no_tag(mti);
            drop(text);
            let text = if !(*bd).note.is_none() {
                window_customize_key_detail(
                    CStr::from_ptr(
                        ((*bd).note)
                            .as_ref()
                            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                    )
                    .to_bytes(),
                )
            } else {
                CString::new(Vec::new()).expect("empty key note")
            };
            mti = mode_tree_add(
                (*data).data,
                child,
                item as *mut ::core::ffi::c_void,
                window_customize_key_tag(bd.cast(), 2),
                b"Note\0" as *const u8 as *const ::core::ffi::c_char,
                text.as_ptr(),
                -(1 as ::core::ffi::c_int),
            ) as *mut mode_tree_item;
            mode_tree_draw_as_parent(mti);
            mode_tree_no_tag(mti);
            drop(text);
            let flag = if (*bd).flags & KEY_BINDING_REPEAT != 0 {
                b"on".as_slice()
            } else {
                b"off".as_slice()
            };
            let text = window_customize_key_detail(flag);
            mti = mode_tree_add(
                (*data).data,
                child,
                item as *mut ::core::ffi::c_void,
                window_customize_key_tag(bd.cast(), 3),
                b"Repeat\0" as *const u8 as *const ::core::ffi::c_char,
                text.as_ptr(),
                -(1 as ::core::ffi::c_int),
            ) as *mut mode_tree_item;
            mode_tree_draw_as_parent(mti);
            mode_tree_no_tag(mti);
            drop(text);
            count = count.wrapping_add(1);
            bd = key_bindings_next(bd);
        }
    }
    format_free(ft);
    if (*data).hide_default != 0 && count == 0 as u_int {
        mode_tree_remove((*data).data, top);
    }
}
unsafe fn window_customize_build_environment(
    mut data: *mut window_customize_modedata,
    mut title: *const ::core::ffi::c_char,
    group: u_int,
    mut scope: window_customize_scope,
    mut env: *mut environ,
    mut ft: *mut format_tree,
    mut filter: *const ::core::ffi::c_char,
    mut fs: *mut cmd_find_state,
) {
    let mut top: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut item: *mut window_customize_itemdata =
        ::core::ptr::null_mut::<window_customize_itemdata>();
    let mut global: ::core::ffi::c_int = 0;
    if (*data).hide_default != 0 {
        return;
    }
    top = mode_tree_add(
        (*data).data,
        ::core::ptr::null_mut::<mode_tree_item>(),
        NULL,
        window_customize_top_tag(group),
        title,
        ::core::ptr::null::<::core::ffi::c_char>(),
        0 as ::core::ffi::c_int,
    ) as *mut mode_tree_item;
    mode_tree_no_tag(top);
    global = (scope as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_GLOBAL_ENVIRONMENT as ::core::ffi::c_int as ::core::ffi::c_uint)
        as ::core::ffi::c_int;
    format_add(
        ft,
        b"is_option\0" as *const u8 as *const ::core::ffi::c_char,
        |out| out.write_all(b"0"),
    );
    format_add(
        ft,
        b"is_key\0" as *const u8 as *const ::core::ffi::c_char,
        |out| out.write_all(b"0"),
    );
    format_add(
        ft,
        b"is_environment\0" as *const u8 as *const ::core::ffi::c_char,
        |out| out.write_all(b"1"),
    );
    format_add(
        ft,
        b"environment_is_global\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write!(out, "{}", (global) as i32),
    );
    let scope_text = window_customize_scope_text(scope, &*fs);
    format_add(
        ft,
        b"environment_scope\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write_cstr(out, scope_text.as_ptr()),
    );
    drop(scope_text);
    for entry in environ_iter(&*env) {
        let envent = entry.as_ptr();
        format_add(
            ft,
            b"environment_name\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write_cstr(out, ((*envent).name).as_ptr().cast_mut()),
        );
        format_add(
            ft,
            b"environment_hidden\0" as *const u8 as *const ::core::ffi::c_char,
            |out| {
                write!(
                    out,
                    "{}",
                    (((*envent).flags & ENVIRON_HIDDEN != 0) as ::core::ffi::c_int) as i32
                )
            },
        );
        format_add(
            ft,
            b"environment_removed\0" as *const u8 as *const ::core::ffi::c_char,
            |out| {
                write!(
                    out,
                    "{}",
                    (((*envent).value
                        == if (NULL as *mut ::core::ffi::c_char).is_null() {
                            None
                        } else {
                            Some(
                                ::std::ffi::CStr::from_ptr(NULL as *mut ::core::ffi::c_char)
                                    .to_owned(),
                            )
                        }) as ::core::ffi::c_int) as i32
                )
            },
        );
        let value = if (*envent).value.is_none() {
            CStr::from_bytes_with_nul(b"\0").expect("empty C string")
        } else {
            CStr::from_ptr(
                ((*envent).value)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            )
        };
        format_add(
            ft,
            b"environment_value\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write_cstr(out, value.as_ptr()),
        );
        if !filter.is_null() {
            let expanded = format_expand_cstring(ft, filter);
            if format_true(expanded.as_ptr()) == 0 {
                continue;
            }
        }
        item = window_customize_add_item(data);
        (*item).type_0 = WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT;
        (*item).scope = scope;
        (*item).environ = env;
        (*item).environ_flags = (*envent).flags;
        window_customize_set_name(&mut *item, Some((*envent).name.as_c_str()));
        let text;
        let name: Cow<'_, CStr> = if (*envent).value.is_none() {
            let entry_name = (*envent).name.as_c_str();
            let mut bytes = Vec::with_capacity(entry_name.to_bytes().len() + 1);
            bytes.push(b'-');
            bytes.extend_from_slice(entry_name.to_bytes());
            text = None;
            Cow::Owned(CString::new(bytes).expect("environment name contains no NUL"))
        } else {
            text = Some(format_expand_cstring(ft, (*data).format.as_ptr()));
            Cow::Borrowed((*envent).name.as_c_str())
        };
        mode_tree_add(
            (*data).data,
            top,
            item as *mut ::core::ffi::c_void,
            (2_u64 << 62) | envent as uint64_t,
            name.as_ptr(),
            text.as_ref()
                .map_or(::core::ptr::null(), |text| text.as_ptr()),
            0 as ::core::ffi::c_int,
        );
    }
}
unsafe fn window_customize_build(
    mut modedata: *mut ::core::ffi::c_void,
    mut filter: *const ::core::ffi::c_char,
) {
    let mut data: *mut window_customize_modedata = modedata as *mut window_customize_modedata;
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut kt: *mut key_table = ::core::ptr::null_mut::<key_table>();
    for item in (*data).item_list.drain(..) {
        drop(item);
    }
    if cmd_find_valid_state(&raw mut (*data).fs) != 0 {
        cmd_find_copy_state(&raw mut fs, &raw mut (*data).fs);
    } else {
        cmd_find_from_pane(&raw mut fs, (*data).wp, 0 as ::core::ffi::c_int);
    }
    ft = format_create_from_state(
        ::core::ptr::null_mut::<cmdq_item>(),
        ::core::ptr::null_mut::<client>(),
        &raw mut fs,
    );
    format_add(
        ft,
        b"is_option\0" as *const u8 as *const ::core::ffi::c_char,
        |out| out.write_all(b"1"),
    );
    format_add(
        ft,
        b"is_key\0" as *const u8 as *const ::core::ffi::c_char,
        |out| out.write_all(b"0"),
    );
    format_add(
        ft,
        b"is_environment\0" as *const u8 as *const ::core::ffi::c_char,
        |out| out.write_all(b"0"),
    );
    window_customize_build_options(
        data,
        b"Server Options\0" as *const u8 as *const ::core::ffi::c_char,
        CUSTOMIZE_SERVER_OPTIONS,
        WINDOW_CUSTOMIZE_SERVER,
        global_options,
        WINDOW_CUSTOMIZE_NONE,
        ::core::ptr::null_mut::<options>(),
        WINDOW_CUSTOMIZE_NONE,
        ::core::ptr::null_mut::<options>(),
        ft,
        filter,
        &raw mut fs,
        WINDOW_CUSTOMIZE_OPTIONS,
    );
    window_customize_build_options(
        data,
        b"Session Options\0" as *const u8 as *const ::core::ffi::c_char,
        CUSTOMIZE_SESSION_OPTIONS,
        WINDOW_CUSTOMIZE_GLOBAL_SESSION,
        global_s_options,
        WINDOW_CUSTOMIZE_SESSION,
        (*fs.s).options,
        WINDOW_CUSTOMIZE_NONE,
        ::core::ptr::null_mut::<options>(),
        ft,
        filter,
        &raw mut fs,
        WINDOW_CUSTOMIZE_OPTIONS,
    );
    window_customize_build_options(
        data,
        b"Window & Pane Options\0" as *const u8 as *const ::core::ffi::c_char,
        CUSTOMIZE_WINDOW_OPTIONS,
        WINDOW_CUSTOMIZE_GLOBAL_WINDOW,
        global_w_options,
        WINDOW_CUSTOMIZE_WINDOW,
        (*fs.w).options,
        WINDOW_CUSTOMIZE_PANE,
        (*fs.wp).options,
        ft,
        filter,
        &raw mut fs,
        WINDOW_CUSTOMIZE_OPTIONS,
    );
    window_customize_build_options(
        data,
        b"Session Hooks\0" as *const u8 as *const ::core::ffi::c_char,
        CUSTOMIZE_SESSION_HOOKS,
        WINDOW_CUSTOMIZE_GLOBAL_SESSION,
        global_s_options,
        WINDOW_CUSTOMIZE_SESSION,
        (*fs.s).options,
        WINDOW_CUSTOMIZE_NONE,
        ::core::ptr::null_mut::<options>(),
        ft,
        filter,
        &raw mut fs,
        WINDOW_CUSTOMIZE_HOOKS,
    );
    window_customize_build_options(
        data,
        b"Window & Pane Hooks\0" as *const u8 as *const ::core::ffi::c_char,
        CUSTOMIZE_WINDOW_HOOKS,
        WINDOW_CUSTOMIZE_GLOBAL_WINDOW,
        global_w_options,
        WINDOW_CUSTOMIZE_WINDOW,
        (*fs.w).options,
        WINDOW_CUSTOMIZE_PANE,
        (*fs.wp).options,
        ft,
        filter,
        &raw mut fs,
        WINDOW_CUSTOMIZE_HOOKS,
    );
    window_customize_build_environment(
        data,
        b"Global Environment\0" as *const u8 as *const ::core::ffi::c_char,
        CUSTOMIZE_GLOBAL_ENVIRONMENT,
        WINDOW_CUSTOMIZE_GLOBAL_ENVIRONMENT,
        global_environ,
        ft,
        filter,
        &raw mut fs,
    );
    window_customize_build_environment(
        data,
        b"Session Environment\0" as *const u8 as *const ::core::ffi::c_char,
        CUSTOMIZE_SESSION_ENVIRONMENT,
        WINDOW_CUSTOMIZE_SESSION_ENVIRONMENT,
        (*fs.s).environ,
        ft,
        filter,
        &raw mut fs,
    );
    format_free(ft);
    ft = format_create_from_state(
        ::core::ptr::null_mut::<cmdq_item>(),
        ::core::ptr::null_mut::<client>(),
        &raw mut fs,
    );
    format_add(
        ft,
        b"is_environment\0" as *const u8 as *const ::core::ffi::c_char,
        |out| out.write_all(b"0"),
    );
    kt = key_bindings_first_table();
    while !kt.is_null() {
        if (*kt).key_bindings.storage.is_some() {
            window_customize_build_keys(data, kt, ft, filter, &raw mut fs);
        }
        kt = key_bindings_next_table(kt);
    }
    format_free(ft);
}
unsafe fn window_customize_draw_key(
    mut item: *mut window_customize_itemdata,
    mut ctx: *mut screen_write_ctx,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut cx: u_int = (*s).cx;
    let mut cy: u_int = (*s).cy;
    let mut kt: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut default_bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut note: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut period: *const ::core::ffi::c_char = b"\0" as *const u8 as *const ::core::ffi::c_char;
    if item.is_null() || window_customize_get_key(item, &raw mut kt, &raw mut bd) == 0 {
        return;
    }
    note = ((*bd).note)
        .as_ref()
        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut());
    if note.is_null() {
        note = b"There is no note for this key.\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if *note as ::core::ffi::c_int != '\0' as i32
        && *note.offset(strlen(note).wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
            != '.' as i32
    {
        period = b".\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if screen_write_text(
        &mut *ctx,
        cx,
        sx,
        sy,
        0 as ::core::ffi::c_int,
        &grid_default_cell,
        |out| {
            write_cstr(out, note)?;
            write_cstr(out, period)
        },
    ) == 0
    {
        return;
    }
    screen_write_cursormove(
        ctx,
        cx as ::core::ffi::c_int,
        (*s).cy.wrapping_add(1 as u_int) as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    if (*s).cy >= cy.wrapping_add(sy).wrapping_sub(1 as u_int) {
        return;
    }
    if screen_write_text(
        &mut *ctx,
        cx,
        sx,
        sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
        0 as ::core::ffi::c_int,
        &grid_default_cell,
        |out| {
            out.write_all(b"This key is in the ")?;
            write_cstr(out, ((*kt).name).as_ptr().cast_mut())?;
            out.write_all(b" table.")
        },
    ) == 0
    {
        return;
    }
    if window_customize_write_value(
        ctx,
        cx,
        sx,
        sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
        0 as ::core::ffi::c_int,
        b"Repeat: \0" as *const u8 as *const ::core::ffi::c_char,
        |out| {
            write_cstr(
                out,
                if (*bd).flags & KEY_BINDING_REPEAT != 0 {
                    b"on\0" as *const u8 as *const ::core::ffi::c_char
                } else {
                    b"off\0" as *const u8 as *const ::core::ffi::c_char
                },
            )
        },
    ) == 0
    {
        return;
    }
    screen_write_cursormove(
        ctx,
        cx as ::core::ffi::c_int,
        (*s).cy.wrapping_add(1 as u_int) as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    if (*s).cy >= cy.wrapping_add(sy).wrapping_sub(1 as u_int) {
        return;
    }
    let cmd = cmd_list_print_cstring(&*(*bd).cmdlist, 0);
    if window_customize_write_value(
        ctx,
        cx,
        sx,
        sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
        0 as ::core::ffi::c_int,
        b"Command: \0" as *const u8 as *const ::core::ffi::c_char,
        |out| write_cstr(out, cmd.as_ptr()),
    ) == 0
    {
        return;
    }
    default_bd = key_bindings_get_default(kt, (*bd).key);
    if !default_bd.is_null() {
        let default_cmd = cmd_list_print_cstring(&*(*default_bd).cmdlist, 0);
        if cmd.as_bytes() != default_cmd.as_bytes()
            && window_customize_write_value(
                ctx,
                cx,
                sx,
                sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                0 as ::core::ffi::c_int,
                b"The default is: \0" as *const u8 as *const ::core::ffi::c_char,
                |out| write_cstr(out, default_cmd.as_ptr()),
            ) == 0
        {
            return;
        }
    }
}
unsafe fn window_customize_draw_option(
    mut data: *mut window_customize_modedata,
    mut item: *mut window_customize_itemdata,
    mut ctx: *mut screen_write_ctx,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut current_block: u64;
    let mut s: *mut screen = (*ctx).s;
    let mut cx: u_int = (*s).cx;
    let mut cy: u_int = (*s).cy;
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut parent: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut go: *mut options = ::core::ptr::null_mut::<options>();
    let mut wo: *mut options = ::core::ptr::null_mut::<options>();
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
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
    let mut choice: *mut *const ::core::ffi::c_char =
        ::core::ptr::null_mut::<*const ::core::ffi::c_char>();
    let mut text: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut array_key: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut space: *const ::core::ffi::c_char = b"\0" as *const u8 as *const ::core::ffi::c_char;
    let mut unit: *const ::core::ffi::c_char = b"\0" as *const u8 as *const ::core::ffi::c_char;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value_owner: Option<CString> = None;
    let mut label: [::core::ffi::c_char; 64] = [0; 64];
    let mut default_value: Option<CString> = None;
    let mut choices: [::core::ffi::c_char; 256] = ::core::mem::transmute::<
        [u8; 256],
        [::core::ffi::c_char; 256],
    >(
        *b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
    );
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut is_hook: ::core::ffi::c_int = 0;
    let mut is_monitor: ::core::ffi::c_int = 0;
    let mut is_user_hook: ::core::ffi::c_int = 0;
    let mut is_any_hook: ::core::ffi::c_int = 0;
    if window_customize_check_item(data, item, &raw mut fs) == 0 {
        return;
    }
    name = ((*item).name)
        .as_ref()
        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut());
    array_key = ((*item).array_key)
        .as_ref()
        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut());
    o = options_get((*item).oo, name);
    if o.is_null() {
        return;
    }
    oe = options_table_entry(o);
    is_hook = (!oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_HOOK != 0) as ::core::ffi::c_int;
    is_monitor = (options_get_monitor_data(o) != NULL) as ::core::ffi::c_int;
    is_user_hook = (*name as ::core::ffi::c_int == '@' as i32 && hooks_is_event(name) != 0)
        as ::core::ffi::c_int;
    is_any_hook = (is_hook != 0 || is_monitor != 0 || is_user_hook != 0) as ::core::ffi::c_int;
    if !oe.is_null() && !(*oe).unit.is_null() {
        space = b" \0" as *const u8 as *const ::core::ffi::c_char;
        unit = (*oe).unit;
    }
    ft = format_create_from_state(
        ::core::ptr::null_mut::<cmdq_item>(),
        ::core::ptr::null_mut::<client>(),
        &raw mut fs,
    );
    if oe.is_null() || (*oe).text.is_null() {
        if is_monitor != 0 {
            text = b"This hook runs when a monitor changes.\0" as *const u8
                as *const ::core::ffi::c_char;
        } else if is_user_hook != 0 {
            text = b"This hook doesn't have a description.\0" as *const u8
                as *const ::core::ffi::c_char;
        } else {
            text = b"This option doesn't have a description.\0" as *const u8
                as *const ::core::ffi::c_char;
        }
    } else {
        text = (*oe).text;
    }
    if !(screen_write_text(
        &mut *ctx,
        cx,
        sx,
        sy,
        0 as ::core::ffi::c_int,
        &grid_default_cell,
        |out| write_cstr(out, text),
    ) == 0)
    {
        screen_write_cursormove(
            ctx,
            cx as ::core::ffi::c_int,
            (*s).cy.wrapping_add(1 as u_int) as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        if !((*s).cy >= cy.wrapping_add(sy).wrapping_sub(1 as u_int)) {
            if is_monitor != 0 {
                if screen_write_text(
                    &mut *ctx,
                    cx,
                    sx,
                    sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                    0 as ::core::ffi::c_int,
                    &grid_default_cell,
                    |out| out.write_all(b"This is a monitor hook."),
                ) == 0
                {
                    current_block = 4086289836260337793;
                } else {
                    current_block = 14134146928577265803;
                }
            } else {
                if oe.is_null() {
                    text = b"user\0" as *const u8 as *const ::core::ffi::c_char;
                } else if (*oe).scope & (OPTIONS_TABLE_WINDOW | OPTIONS_TABLE_PANE)
                    == OPTIONS_TABLE_WINDOW | OPTIONS_TABLE_PANE
                {
                    text = b"window and pane\0" as *const u8 as *const ::core::ffi::c_char;
                } else if (*oe).scope & OPTIONS_TABLE_WINDOW != 0 {
                    text = b"window\0" as *const u8 as *const ::core::ffi::c_char;
                } else if (*oe).scope & OPTIONS_TABLE_SESSION != 0 {
                    text = b"session\0" as *const u8 as *const ::core::ffi::c_char;
                } else {
                    text = b"server\0" as *const u8 as *const ::core::ffi::c_char;
                }
                if is_user_hook != 0 {
                    if screen_write_text(
                        &mut *ctx,
                        cx,
                        sx,
                        sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                        0 as ::core::ffi::c_int,
                        &grid_default_cell,
                        |out| out.write_all(b"This is a user hook."),
                    ) == 0
                    {
                        current_block = 4086289836260337793;
                    } else {
                        current_block = 14134146928577265803;
                    }
                } else if is_hook != 0 {
                    if screen_write_text(
                        &mut *ctx,
                        cx,
                        sx,
                        sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                        0 as ::core::ffi::c_int,
                        &grid_default_cell,
                        |out| {
                            out.write_all(b"This is a ")?;
                            write_cstr(out, text)?;
                            out.write_all(b" hook.")
                        },
                    ) == 0
                    {
                        current_block = 4086289836260337793;
                    } else {
                        current_block = 14134146928577265803;
                    }
                } else if screen_write_text(
                    &mut *ctx,
                    cx,
                    sx,
                    sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                    0 as ::core::ffi::c_int,
                    &grid_default_cell,
                    |out| {
                        out.write_all(b"This is a ")?;
                        write_cstr(out, text)?;
                        out.write_all(b" option.")
                    },
                ) == 0
                {
                    current_block = 4086289836260337793;
                } else {
                    current_block = 14134146928577265803;
                }
            }
            match current_block {
                4086289836260337793 => {}
                _ => {
                    if let Some(monitor) = hooks_monitor_to_cstring(o) {
                        if window_customize_write_value(
                            ctx,
                            cx,
                            sx,
                            sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                            0 as ::core::ffi::c_int,
                            b"Monitor: \0" as *const u8 as *const ::core::ffi::c_char,
                            |out| write_cstr(out, monitor.as_ptr()),
                        ) == 0
                        {
                            current_block = 4086289836260337793;
                        } else {
                            current_block = 13460095289871124136;
                        }
                    } else {
                        current_block = 13460095289871124136;
                    }
                    match current_block {
                        4086289836260337793 => {}
                        _ => {
                            if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
                                if is_hook != 0 {
                                    if array_key.is_null() {
                                        if screen_write_text(
                                            &mut *ctx,
                                            cx,
                                            sx,
                                            sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                                            0 as ::core::ffi::c_int,
                                            &grid_default_cell,
                                            |out| out.write_all(b"This is an array hook."),
                                        ) == 0
                                        {
                                            current_block = 4086289836260337793;
                                        } else {
                                            window_customize_write_hook_fire(
                                                ctx,
                                                cx,
                                                sx,
                                                sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                                                o,
                                            ) == 0;
                                            current_block = 4086289836260337793;
                                        }
                                    } else {
                                        current_block = 168769493162332264;
                                    }
                                } else if !array_key.is_null() {
                                    if screen_write_text(
                                        &mut *ctx,
                                        cx,
                                        sx,
                                        sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                                        0 as ::core::ffi::c_int,
                                        &grid_default_cell,
                                        |out| {
                                            out.write_all(b"This is an array option, key ")?;
                                            write_cstr(out, array_key)?;
                                            out.write_all(b".")
                                        },
                                    ) == 0
                                    {
                                        current_block = 4086289836260337793;
                                    } else {
                                        current_block = 168769493162332264;
                                    }
                                } else if screen_write_text(
                                    &mut *ctx,
                                    cx,
                                    sx,
                                    sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                                    0 as ::core::ffi::c_int,
                                    &grid_default_cell,
                                    |out| out.write_all(b"This is an array option."),
                                ) == 0
                                {
                                    current_block = 4086289836260337793;
                                } else {
                                    current_block = 168769493162332264;
                                }
                                match current_block {
                                    4086289836260337793 => {}
                                    _ => {
                                        if array_key.is_null() {
                                            current_block = 4086289836260337793;
                                        } else {
                                            current_block = 8835654301469918283;
                                        }
                                    }
                                }
                            } else {
                                current_block = 8835654301469918283;
                            }
                            match current_block {
                                4086289836260337793 => {}
                                _ => {
                                    screen_write_cursormove(
                                        ctx,
                                        cx as ::core::ffi::c_int,
                                        (*s).cy.wrapping_add(1 as u_int) as ::core::ffi::c_int,
                                        0 as ::core::ffi::c_int,
                                    );
                                    if !((*s).cy >= cy.wrapping_add(sy).wrapping_sub(1 as u_int)) {
                                        value_owner = Some(options_to_string(o, array_key));
                                        value = value_owner.as_ref().unwrap().as_ptr().cast_mut();
                                        if !oe.is_null() && array_key.is_null() {
                                            let rendered = options_default_to_cstring(&*oe);
                                            if strcmp(rendered.as_ptr(), value)
                                                != 0 as ::core::ffi::c_int
                                            {
                                                default_value = Some(rendered);
                                            }
                                        }
                                        if is_any_hook != 0 {
                                            if window_customize_write_value(
                                                ctx,
                                                cx,
                                                sx,
                                                sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                                                0 as ::core::ffi::c_int,
                                                b"Hook command: \0" as *const u8
                                                    as *const ::core::ffi::c_char,
                                                |out| {
                                                    write_cstr(out, value)?;
                                                    write_cstr(out, space)?;
                                                    write_cstr(out, unit)
                                                },
                                            ) == 0
                                            {
                                                current_block = 4086289836260337793;
                                            } else if window_customize_write_hook_fire(
                                                ctx,
                                                cx,
                                                sx,
                                                sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                                                o,
                                            ) == 0
                                            {
                                                current_block = 4086289836260337793;
                                            } else {
                                                current_block = 7385833325316299293;
                                            }
                                        } else if window_customize_write_value(
                                            ctx,
                                            cx,
                                            sx,
                                            sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                                            0 as ::core::ffi::c_int,
                                            b"Option value: \0" as *const u8
                                                as *const ::core::ffi::c_char,
                                            |out| {
                                                write_cstr(out, value)?;
                                                write_cstr(out, space)?;
                                                write_cstr(out, unit)
                                            },
                                        ) == 0
                                        {
                                            current_block = 4086289836260337793;
                                        } else {
                                            current_block = 7385833325316299293;
                                        }
                                        match current_block {
                                            4086289836260337793 => {}
                                            _ => {
                                                if oe.is_null()
                                                    || (*oe).type_0 as ::core::ffi::c_uint
                                                        == OPTIONS_TABLE_STRING
                                                            as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint
                                                {
                                                    let expanded = format_expand_cstring(ft, value);
                                                    if strcmp(expanded.as_ptr(), value)
                                                        != 0 as ::core::ffi::c_int
                                                    {
                                                        if window_customize_write_value(
                                                            ctx,
                                                            cx,
                                                            sx,
                                                            sy.wrapping_sub(
                                                                (*s).cy.wrapping_sub(cy),
                                                            ),
                                                            0 as ::core::ffi::c_int,
                                                            b"This expands to: \0" as *const u8
                                                                as *const ::core::ffi::c_char,
                                                            |out| {
                                                                write_cstr(out, expanded.as_ptr())
                                                            },
                                                        ) == 0
                                                        {
                                                            current_block = 4086289836260337793;
                                                        } else {
                                                            current_block = 479107131381816815;
                                                        }
                                                    } else {
                                                        current_block = 479107131381816815;
                                                    }
                                                    match current_block {
                                                        4086289836260337793 => {}
                                                        _ => {
                                                            current_block = 16108440464692313034;
                                                        }
                                                    }
                                                } else {
                                                    current_block = 16108440464692313034;
                                                }
                                                match current_block {
                                                    4086289836260337793 => {}
                                                    _ => {
                                                        if !oe.is_null()
                                                            && (*oe).type_0 as ::core::ffi::c_uint
                                                                == OPTIONS_TABLE_CHOICE
                                                                    as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                        {
                                                            choice = (*oe).choices;
                                                            while !(*choice).is_null() {
                                                                strlcat(
                                                                    &raw mut choices
                                                                        as *mut ::core::ffi::c_char,
                                                                    *choice,
                                                                    ::core::mem::size_of::<
                                                                        [::core::ffi::c_char; 256],
                                                                    >(
                                                                    )
                                                                        as size_t,
                                                                );
                                                                strlcat(
                                                                    &raw mut choices as *mut ::core::ffi::c_char,
                                                                    b", \0" as *const u8 as *const ::core::ffi::c_char,
                                                                    ::core::mem::size_of::<[::core::ffi::c_char; 256]>()
                                                                        as size_t,
                                                                );
                                                                choice = choice.offset(1);
                                                            }
                                                            choices[strlen(
                                                                &raw mut choices
                                                                    as *mut ::core::ffi::c_char,
                                                            )
                                                            .wrapping_sub(2 as size_t)
                                                                as usize] =
                                                                '\0' as i32 as ::core::ffi::c_char;
                                                            if window_customize_write_value(
                                                                ctx,
                                                                cx,
                                                                sx,
                                                                sy.wrapping_sub(
                                                                    (*s).cy.wrapping_sub(cy),
                                                                ),
                                                                0 as ::core::ffi::c_int,
                                                                b"Available values are: \0"
                                                                    as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                                |out| {
                                                                    write_cstr(out, &raw mut choices
                                                                    as *mut ::core::ffi::c_char)
                                                                },
                                                            ) == 0
                                                            {
                                                                current_block = 4086289836260337793;
                                                            } else {
                                                                current_block =
                                                                    17688141731389699982;
                                                            }
                                                        } else {
                                                            current_block = 17688141731389699982;
                                                        }
                                                        match current_block {
                                                            4086289836260337793 => {}
                                                            _ => {
                                                                if !oe.is_null()
                                                                    && (*oe).type_0
                                                                        as ::core::ffi::c_uint
                                                                        == OPTIONS_TABLE_COLOUR
                                                                            as ::core::ffi::c_int
                                                                            as ::core::ffi::c_uint
                                                                {
                                                                    if screen_write_text(&mut *ctx,
cx,
sx,
sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
1 as ::core::ffi::c_int,
&grid_default_cell,
|out| {
out.write_all(b"This is a colour option: ")
}) == 0
                                                                    {
                                                                        current_block = 4086289836260337793;
                                                                    } else {
                                                                        memcpy(
                                                                            &raw mut gc as *mut ::core::ffi::c_void,
                                                                            &raw const grid_default_cell as *const ::core::ffi::c_void,
                                                                            ::core::mem::size_of::<grid_cell>() as size_t,
                                                                        );
                                                                        gc.fg = options_get_number((*item).oo, name)
                                                                            as ::core::ffi::c_int;
                                                                        if screen_write_text(&mut *ctx,
cx,
sx,
sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
0 as ::core::ffi::c_int,
&gc,
|out| {
out.write_all(b"EXAMPLE")
}) == 0
                                                                        {
                                                                            current_block = 4086289836260337793;
                                                                        } else {
                                                                            current_block = 10887629115603254199;
                                                                        }
                                                                    }
                                                                } else {
                                                                    current_block =
                                                                        10887629115603254199;
                                                                }
                                                                match current_block {
                                                                    4086289836260337793 => {}
                                                                    _ => {
                                                                        if !oe.is_null()
                                                                            && (*oe).flags & OPTIONS_TABLE_IS_COLOUR != 0
                                                                        {
                                                                            if screen_write_text(&mut *ctx,
cx,
sx,
sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
1 as ::core::ffi::c_int,
&grid_default_cell,
|out| {
out.write_all(b"This is a colour option: ")
}) == 0
                                                                            {
                                                                                current_block = 4086289836260337793;
                                                                            } else {
                                                                                style_apply(&raw mut gc, (*item).oo, name, ft);
                                                                                if screen_write_text(&mut *ctx,
cx,
sx,
sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
0 as ::core::ffi::c_int,
&gc,
|out| {
out.write_all(b"EXAMPLE")
}) == 0
                                                                                {
                                                                                    current_block = 4086289836260337793;
                                                                                } else {
                                                                                    current_block = 17995254032144898061;
                                                                                }
                                                                            }
                                                                        } else {
                                                                            current_block = 17995254032144898061;
                                                                        }
                                                                        match current_block {
                                                                            4086289836260337793 => {
                                                                            }
                                                                            _ => {
                                                                                if !oe.is_null()
                                                                                    && (*oe).flags & OPTIONS_TABLE_IS_STYLE != 0
                                                                                {
                                                                                    if screen_write_text(&mut *ctx,
cx,
sx,
sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
1 as ::core::ffi::c_int,
&grid_default_cell,
|out| {
out.write_all(b"This is a style option: ")
}) == 0
                                                                                    {
                                                                                        current_block = 4086289836260337793;
                                                                                    } else {
                                                                                        style_apply(&raw mut gc, (*item).oo, name, ft);
                                                                                        if screen_write_text(&mut *ctx,
cx,
sx,
sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
0 as ::core::ffi::c_int,
&gc,
|out| {
out.write_all(b"EXAMPLE")
}) == 0
                                                                                        {
                                                                                            current_block = 4086289836260337793;
                                                                                        } else {
                                                                                            current_block = 1934991416718554651;
                                                                                        }
                                                                                    }
                                                                                } else {
                                                                                    current_block = 1934991416718554651;
                                                                                }
                                                                                match current_block {
                                                                                    4086289836260337793 => {}
                                                                                    _ => {
                                                                                        if let Some(default_value) = &default_value {
                                                                                            if window_customize_write_value(ctx,
cx,
sx,
sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
0 as ::core::ffi::c_int,
b"The default is: \0" as *const u8
                                                                                                    as *const ::core::ffi::c_char,
|out| {
write_cstr(out, default_value.as_ptr())?;
write_cstr(out, space)?;
write_cstr(out, unit)
}) == 0
                                                                                            {
                                                                                                current_block = 4086289836260337793;
                                                                                            } else {
                                                                                                current_block = 15622658527355336244;
                                                                                            }
                                                                                        } else {
                                                                                            current_block = 15622658527355336244;
                                                                                        }
                                                                                        match current_block {
                                                                                            4086289836260337793 => {}
                                                                                            _ => {
                                                                                                screen_write_cursormove(
                                                                                                    ctx,
                                                                                                    cx as ::core::ffi::c_int,
                                                                                                    (*s).cy.wrapping_add(1 as u_int) as ::core::ffi::c_int,
                                                                                                    0 as ::core::ffi::c_int,
                                                                                                );
                                                                                                if !((*s).cy > cy.wrapping_add(sy).wrapping_sub(1 as u_int))
                                                                                                {
                                                                                                    if !oe.is_null()
                                                                                                        && (*oe).flags & OPTIONS_TABLE_IS_ARRAY != 0
                                                                                                    {
                                                                                                        wo = ::core::ptr::null_mut::<options>();
                                                                                                        go = ::core::ptr::null_mut::<options>();
                                                                                                    } else {
                                                                                                        match (*item).scope as ::core::ffi::c_uint {
                                                                                                            7 => {
                                                                                                                wo = options_get_parent((*item).oo);
                                                                                                                go = options_get_parent(wo);
                                                                                                            }
                                                                                                            6 | 4 => {
                                                                                                                wo = ::core::ptr::null_mut::<options>();
                                                                                                                go = options_get_parent((*item).oo);
                                                                                                            }
                                                                                                            _ => {
                                                                                                                wo = ::core::ptr::null_mut::<options>();
                                                                                                                go = ::core::ptr::null_mut::<options>();
                                                                                                            }
                                                                                                        }
                                                                                                    }
                                                                                                    if !wo.is_null() && options_owner(o) != wo {
                                                                                                        parent = options_get_only(wo, name);
                                                                                                        if !parent.is_null() {
                                                                                                            value_owner = Some(options_to_string(
                                                                                                                parent,
                                                                                                                ::core::ptr::null::<::core::ffi::c_char>(),
                                                                                                            ));
                                                                                                            value = value_owner.as_ref().unwrap().as_ptr().cast_mut();
                                                                                                            xformat(&mut label, format_args!("Window value (from window {}): " , ((*fs.wl).idx) as u32));
                                                                                                            if window_customize_write_value(ctx,
(*s).cx,
sx,
sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
0 as ::core::ffi::c_int,
&raw mut label as *mut ::core::ffi::c_char,
|out| {
write_cstr(out, value)?;
write_cstr(out, space)?;
write_cstr(out, unit)
}) == 0
                                                                                                            {
                                                                                                                current_block = 4086289836260337793;
                                                                                                            } else {
                                                                                                                current_block = 9073771928613846474;
                                                                                                            }
                                                                                                        } else {
                                                                                                            current_block = 9073771928613846474;
                                                                                                        }
                                                                                                    } else {
                                                                                                        current_block = 9073771928613846474;
                                                                                                    }
                                                                                                    match current_block {
                                                                                                        4086289836260337793 => {}
                                                                                                        _ => {
                                                                                                            if !go.is_null() && options_owner(o) != go {
                                                                                                                parent = options_get_only(go, name);
                                                                                                                if !parent.is_null() {
                                                                                                                    value_owner = Some(options_to_string(
                                                                                                                       parent,
                                                                                                                       ::core::ptr::null::<::core::ffi::c_char>(),
                                                                                                                    ));
                                                                                                                    value = value_owner.as_ref().unwrap().as_ptr().cast_mut();
                                                                                                                    window_customize_write_value(ctx,
(*s).cx,
sx,
sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
0 as ::core::ffi::c_int,
b"Global value: \0" as *const u8
                                                                                                                            as *const ::core::ffi::c_char,
|out| {
write_cstr(out, value)?;
write_cstr(out, space)?;
write_cstr(out, unit)
}) == 0;
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
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    drop(default_value);
    format_free(ft);
}
unsafe fn window_customize_draw_environment(
    mut data: *mut window_customize_modedata,
    mut item: *mut window_customize_itemdata,
    mut ctx: *mut screen_write_ctx,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut cx: u_int = (*s).cx;
    let mut cy: u_int = (*s).cy;
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut parent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut text: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if window_customize_check_item(data, item, &raw mut fs) == 0 {
        return;
    }
    envent = environ_find(
        (*item).environ,
        ((*item).name)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
    );
    if envent.is_null() {
        return;
    }
    if (*item).scope as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_GLOBAL_ENVIRONMENT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        text = b"global\0" as *const u8 as *const ::core::ffi::c_char;
    } else {
        text = b"session\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if screen_write_text(
        &mut *ctx,
        cx,
        sx,
        sy,
        0 as ::core::ffi::c_int,
        &grid_default_cell,
        |out| {
            out.write_all(b"This is a ")?;
            write_cstr(out, text)?;
            out.write_all(b" environment variable.")
        },
    ) == 0
    {
        return;
    }
    if (*envent).flags & ENVIRON_HIDDEN != 0 {
        if screen_write_text(
            &mut *ctx,
            cx,
            sx,
            sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
            0 as ::core::ffi::c_int,
            &grid_default_cell,
            |out| out.write_all(b"This variable is hidden."),
        ) == 0
        {
            return;
        }
    }
    screen_write_cursormove(
        ctx,
        cx as ::core::ffi::c_int,
        (*s).cy.wrapping_add(1 as u_int) as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    if (*s).cy >= cy.wrapping_add(sy).wrapping_sub(1 as u_int) {
        return;
    }
    if (*envent).value.is_none() {
        if screen_write_text(
            &mut *ctx,
            cx,
            sx,
            sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
            0 as ::core::ffi::c_int,
            &grid_default_cell,
            |out| out.write_all(b"Variable is removed."),
        ) == 0
        {
            return;
        }
    } else if window_customize_write_value(
        ctx,
        cx,
        sx,
        sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
        0 as ::core::ffi::c_int,
        b"Variable value: \0" as *const u8 as *const ::core::ffi::c_char,
        |out| {
            write_cstr(
                out,
                ((*envent).value)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            )
        },
    ) == 0
    {
        return;
    }
    if (*item).scope as ::core::ffi::c_uint
        != WINDOW_CUSTOMIZE_SESSION_ENVIRONMENT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    parent = environ_find(
        global_environ,
        ((*item).name)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
    );
    if parent.is_null() {
        return;
    }
    if (*parent).value.is_none() {
        if screen_write_text(
            &mut *ctx,
            cx,
            sx,
            sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
            0 as ::core::ffi::c_int,
            &grid_default_cell,
            |out| out.write_all(b"Global variable is removed."),
        ) == 0
        {
            return;
        }
    } else if window_customize_write_value(
        ctx,
        cx,
        sx,
        sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
        0 as ::core::ffi::c_int,
        b"Global value: \0" as *const u8 as *const ::core::ffi::c_char,
        |out| {
            write_cstr(
                out,
                ((*parent).value)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            )
        },
    ) == 0
    {
        return;
    }
}
unsafe fn window_customize_draw(
    mut modedata: *mut ::core::ffi::c_void,
    mut itemdata: *mut ::core::ffi::c_void,
    mut ctx: *mut screen_write_ctx,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut data: *mut window_customize_modedata = modedata as *mut window_customize_modedata;
    let mut item: *mut window_customize_itemdata = itemdata as *mut window_customize_itemdata;
    if item.is_null() {
        return;
    }
    if (*item).type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_ITEM_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        window_customize_draw_key(item, ctx, sx, sy);
    } else if (*item).type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        window_customize_draw_environment(data, item, ctx, sx, sy);
    } else {
        window_customize_draw_option(data, item, ctx, sx, sy);
    };
}
unsafe fn window_customize_menu(
    mut modedata: *mut ::core::ffi::c_void,
    mut c: *mut client,
    mut key: key_code,
) {
    let mut data: *mut window_customize_modedata = modedata as *mut window_customize_modedata;
    let mut wp: *mut window_pane = (*data).wp;
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    wme = (*wp).modes.active;
    if wme.is_null() || (*wme).data != modedata {
        return;
    }
    window_customize_key(
        wme,
        c,
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        key,
        ::core::ptr::null_mut::<mouse_event>(),
    );
}
unsafe fn window_customize_height() -> u_int {
    return 12 as u_int;
}
static window_customize_help_lines: &[&'static CStr] = &[
    c"#[fg=themelightgrey]   Enter, s #[#{E:tree-mode-border-style},acs]x#[default] Set %1 value",
    c"#[fg=themelightgrey]          S #[#{E:tree-mode-border-style},acs]x#[default] Set global %1 value",
    c"#[fg=themelightgrey]          w #[#{E:tree-mode-border-style},acs]x#[default] Set window %1 value",
    c"#[fg=themelightgrey]          d #[#{E:tree-mode-border-style},acs]x#[default] Set to default value",
    c"#[fg=themelightgrey]          D #[#{E:tree-mode-border-style},acs]x#[default] Set tagged %1s to default value",
    c"#[fg=themelightgrey]          u #[#{E:tree-mode-border-style},acs]x#[default] Unset an %1",
    c"#[fg=themelightgrey]          U #[#{E:tree-mode-border-style},acs]x#[default] Unset tagged %1s",
    c"#[fg=themelightgrey]          a #[#{E:tree-mode-border-style},acs]x#[default] Change array key",
    c"#[fg=themelightgrey]          e #[#{E:tree-mode-border-style},acs]x#[default] Open %1 value in editor",
    c"#[fg=themelightgrey]          f #[#{E:tree-mode-border-style},acs]x#[default] Enter a filter",
    c"#[fg=themelightgrey]          C #[#{E:tree-mode-border-style},acs]x#[default] Toggle only changed items",
    c"#[fg=themelightgrey]          v #[#{E:tree-mode-border-style},acs]x#[default] Toggle information",
];
fn window_customize_help() -> mode_tree_help_info {
    mode_tree_help_info {
        width: 52 as u_int,
        item: c"item",
        lines: window_customize_help_lines,
    }
}
unsafe fn window_customize_init(
    mut wme: *mut window_mode_entry,
    _item: *mut cmdq_item,
    mut fs: *mut cmd_find_state,
    mut args: *mut args,
) -> *mut screen {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_customize_modedata =
        ::core::ptr::null_mut::<window_customize_modedata>();
    let mut s: *mut screen = ::core::ptr::null_mut::<screen>();
    let format = if args.is_null() || args_has(args, 'F' as i32 as u_char) == 0 {
        CStr::from_ptr(WINDOW_CUSTOMIZE_DEFAULT_FORMAT.as_ptr()).to_owned()
    } else {
        CStr::from_ptr(args_get(args, 'F' as i32 as u_char)).to_owned()
    };
    data = crate::src::shared::rc::new(window_customize_modedata {
        wp,
        dead: 0,
        data: ::core::ptr::null_mut(),
        editor: ::core::ptr::null_mut(),
        edit: ::core::ptr::null_mut(),
        format,
        hide_global: 0,
        hide_default: 0,
        prompt_flags: 0,
        item_list: Vec::new(),
        fs: ::core::ptr::read(fs),
        change: WINDOW_CUSTOMIZE_UNSET,
    });
    (*wme).data = data as *mut ::core::ffi::c_void;
    let data_handle = std::ptr::NonNull::new(data).expect("live customize mode data");
    if args_has(args, 'y' as i32 as u_char) != 0 {
        (*data).prompt_flags = PROMPT_ACCEPT;
    }
    (*data).data = mode_tree_start(
        wp,
        args,
        Some(Box::new(move |_, tag, filter| {
            let mut selected = tag.unwrap_or(::core::primitive::u64::MAX as uint64_t);
            window_customize_build(
                data_handle.as_ptr().cast(),
                filter.map_or(::core::ptr::null(), |value| value.as_ptr()),
            );
            (selected != ::core::primitive::u64::MAX as uint64_t).then_some(selected)
        })),
        Some(Box::new(move |itemdata, ctx, sx, sy| {
            window_customize_draw(
                data_handle.as_ptr().cast(),
                itemdata,
                ctx as *mut screen_write_ctx,
                sx,
                sy,
            )
        })),
        None,
        Some(Box::new(move |client, key| {
            window_customize_menu(
                data_handle.as_ptr().cast(),
                client.map_or(::core::ptr::null_mut(), std::ptr::NonNull::as_ptr),
                key,
            )
        })),
        Some(Box::new(move |_| window_customize_height())),
        None,
        None,
        None,
        Some(window_customize_help),
        &raw const window_customize_menu_items as *const menu_item,
        &raw mut s,
    );
    mode_tree_zoom((*data).data, args);
    mode_tree_build((*data).data);
    mode_tree_draw((*data).data);
    return s;
}
unsafe fn window_customize_destroy(data: *mut window_customize_modedata) {
    crate::src::shared::rc::release(data);
}
unsafe fn window_customize_free(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_customize_modedata = (*wme).data as *mut window_customize_modedata;
    if data.is_null() {
        return;
    }
    (*data).dead = 1 as ::core::ffi::c_int;
    if !(*data).editor.is_null() {
        spawn_cancel_editor((*data).editor);
        window_customize_finish_edit((*data).edit as *mut window_customize_editdata);
    }
    mode_tree_free((*data).data);
    window_customize_destroy(data);
}
unsafe fn window_customize_resize(mut wme: *mut window_mode_entry, mut sx: u_int, mut sy: u_int) {
    let mut data: *mut window_customize_modedata = (*wme).data as *mut window_customize_modedata;
    mode_tree_resize((*data).data, sx, sy);
}
unsafe fn window_customize_update(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_customize_modedata = (*wme).data as *mut window_customize_modedata;
    window_customize_draw_waiting(data);
}
unsafe fn window_customize_free_callback(mut data: *mut window_customize_modedata) {
    window_customize_destroy(data);
}
unsafe fn window_customize_free_item_callback(mut item: *mut window_customize_itemdata) {
    let mut data: *mut window_customize_modedata = (*item).data as *mut window_customize_modedata;
    window_customize_free_item(item);
    window_customize_destroy(data);
}
fn window_customize_prompt_input_cb<T: 'static>(
    callback: unsafe fn(*mut client, *mut T, Option<&CStr>, prompt_key_result) -> prompt_result,
    data: *mut T,
) -> mode_tree_prompt_input_cb {
    Some(Box::new(move |c, s, key| unsafe {
        callback(
            c.map_or(::core::ptr::null_mut(), std::ptr::NonNull::as_ptr),
            data,
            s,
            key,
        )
    }))
}
fn window_customize_prompt_free_cb<T: 'static>(
    freecb: unsafe fn(*mut T),
    data: *mut T,
) -> prompt_free_cb {
    Some(Box::new(move || unsafe { freecb(data) }))
}
unsafe fn window_customize_set_option_callback(
    mut c: *mut client,
    mut itemdata: *mut window_customize_itemdata,
    s: Option<&CStr>,
    _key: prompt_key_result,
) -> prompt_result {
    let mut s = s.map_or(::core::ptr::null(), CStr::as_ptr);
    let mut current_block: u64;
    let mut item: *mut window_customize_itemdata = itemdata as *mut window_customize_itemdata;
    let mut data: *mut window_customize_modedata = (*item).data as *mut window_customize_modedata;
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    let mut oo: *mut options = (*item).oo;
    let mut name: *const ::core::ffi::c_char = ((*item).name)
        .as_ref()
        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut());
    let mut array_key: *const ::core::ffi::c_char = ((*item).array_key)
        .as_ref()
        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut());
    let mut cause: Option<CString> = None;
    let mut idx: u_int = 0;
    let mut keybuf: [::core::ffi::c_char; 32] = [0; 32];
    if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 || (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    if item.is_null()
        || window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0
    {
        return PROMPT_CLOSE;
    }
    o = options_get(oo, name);
    if o.is_null() {
        return PROMPT_CLOSE;
    }
    oe = options_table_entry(o);
    if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
        if array_key.is_null() {
            idx = 0 as u_int;
            while idx < INT_MAX as u_int {
                if options_array_get_index(o, idx).is_null() {
                    break;
                }
                idx = idx.wrapping_add(1);
            }
            xformat(&mut keybuf, format_args!("{}", idx as u32));
            array_key = &raw mut keybuf as *mut ::core::ffi::c_char;
        }
        if options_array_set(o, array_key, s, 0 as ::core::ffi::c_int, &raw mut cause)
            != 0 as ::core::ffi::c_int
        {
            current_block = 1995505731522653903;
        } else {
            current_block = 4808432441040389987;
        }
    } else if options_from_string(oo, oe, name, s, 0 as ::core::ffi::c_int, &raw mut cause)
        != 0 as ::core::ffi::c_int
    {
        current_block = 1995505731522653903;
    } else {
        current_block = 4808432441040389987;
    }
    match current_block {
        1995505731522653903 => {
            window_customize_uppercase_cause(&mut cause);
            status_message_set(
                c,
                -(1 as ::core::ffi::c_int),
                1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                |out| write_cstr(out, cause.as_ref().unwrap().as_ptr()),
            );
            return PROMPT_CLOSE;
        }
        _ => {
            if (*item).option_type as ::core::ffi::c_uint
                == WINDOW_CUSTOMIZE_HOOKS as ::core::ffi::c_int as ::core::ffi::c_uint
                && *name as ::core::ffi::c_int == '@' as i32
            {
                hooks_add_event(name);
            }
            options_push_changes(
                ((*item).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            );
            mode_tree_build((*data).data);
            mode_tree_draw((*data).data);
            (*(*data).wp).flags |= PANE_REDRAW;
            return PROMPT_CLOSE;
        }
    };
}
unsafe fn window_customize_set_environment_callback(
    _c: *mut client,
    mut itemdata: *mut window_customize_itemdata,
    s: Option<&CStr>,
    _key: prompt_key_result,
) -> prompt_result {
    let mut s = s.map_or(::core::ptr::null(), CStr::as_ptr);
    let mut item: *mut window_customize_itemdata = itemdata as *mut window_customize_itemdata;
    let mut data: *mut window_customize_modedata = (*item).data as *mut window_customize_modedata;
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut flags: ::core::ffi::c_int = 0;
    if s.is_null() || (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    if item.is_null()
        || window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0
    {
        return PROMPT_CLOSE;
    }
    flags = (*item).environ_flags;
    envent = environ_find(
        (*item).environ,
        ((*item).name)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
    );
    if !envent.is_null() {
        flags = (*envent).flags;
    }
    environ_set(
        (*item).environ,
        ((*item).name)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        flags,
        |out| write_cstr(out, s),
    );
    mode_tree_build((*data).data);
    mode_tree_draw((*data).data);
    (*(*data).wp).flags |= PANE_REDRAW;
    return PROMPT_CLOSE;
}
unsafe fn window_customize_set_environment(
    mut c: *mut client,
    mut data: *mut window_customize_modedata,
    mut item: *mut window_customize_itemdata,
    mut global: ::core::ffi::c_int,
) {
    let mut new_item: *mut window_customize_itemdata =
        ::core::ptr::null_mut::<window_customize_itemdata>();
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut env: *mut environ = ::core::ptr::null_mut::<environ>();
    let mut scope: window_customize_scope = WINDOW_CUSTOMIZE_NONE;
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut space: *const ::core::ffi::c_char = b"\0" as *const u8 as *const ::core::ffi::c_char;
    if item.is_null() || window_customize_check_item(data, item, &raw mut fs) == 0 {
        return;
    }
    envent = environ_find(
        (*item).environ,
        ((*item).name)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
    );
    if envent.is_null() {
        return;
    }
    if global != 0 {
        scope = WINDOW_CUSTOMIZE_GLOBAL_ENVIRONMENT;
        env = global_environ;
    } else {
        scope = (*item).scope;
        env = (*item).environ;
    }
    let scope_text = window_customize_scope_text(scope, &fs);
    if !scope_text.as_bytes().is_empty() {
        space = b", for \0" as *const u8 as *const ::core::ffi::c_char;
    } else if scope as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_GLOBAL_ENVIRONMENT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        space = b", global\0" as *const u8 as *const ::core::ffi::c_char;
    }
    let mut prompt_bytes = Vec::new();
    prompt_bytes.extend_from_slice(b"(");
    prompt_bytes.extend_from_slice(
        CStr::from_ptr(
            ((*item).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        )
        .to_bytes(),
    );
    prompt_bytes.extend_from_slice(CStr::from_ptr(space).to_bytes());
    prompt_bytes.extend_from_slice(scope_text.as_bytes());
    prompt_bytes.extend_from_slice(b") ");
    let prompt = CString::new(prompt_bytes).expect("environment prompt contains no NUL");
    drop(scope_text);
    new_item = window_customize_new_item();
    (*new_item).data = data as *mut window_customize_modedata;
    (*new_item).type_0 = WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT;
    (*new_item).scope = scope;
    (*new_item).environ = env;
    (*new_item).environ_flags = (*envent).flags;
    window_customize_set_name(&mut *new_item, (*item).name.as_deref());
    crate::src::shared::rc::retain(data);
    mode_tree_set_prompt(
        (*data).data,
        c,
        prompt.as_ptr(),
        if (*envent).value.is_none() {
            b"\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            ((*envent).value)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
                as *const ::core::ffi::c_char
        },
        PROMPT_TYPE_COMMAND,
        PROMPT_NOFORMAT,
        window_customize_prompt_input_cb(window_customize_set_environment_callback, new_item),
        window_customize_prompt_free_cb(window_customize_free_item_callback, new_item),
    );
}
unsafe fn window_customize_add_option_callback(
    mut c: *mut client,
    mut itemdata: *mut window_customize_itemdata,
    s: Option<&CStr>,
    _key: prompt_key_result,
) -> prompt_result {
    let mut s = s.map_or(::core::ptr::null(), CStr::as_ptr);
    let mut item: *mut window_customize_itemdata = itemdata as *mut window_customize_itemdata;
    let mut data: *mut window_customize_modedata = (*item).data as *mut window_customize_modedata;
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut what: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut namelen: size_t = 0;
    if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 || (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    if item.is_null()
        || window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0
    {
        return PROMPT_CLOSE;
    }
    namelen = strcspn(s, b" \t\0" as *const u8 as *const ::core::ffi::c_char) as size_t;
    if namelen == 0 as size_t || *s.offset(namelen as isize) as ::core::ffi::c_int == '\0' as i32 {
        status_message_set(
            c,
            -(1 as ::core::ffi::c_int),
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            |out| out.write_all(b"User option must be @name value"),
        );
        return PROMPT_CLOSE;
    }
    value = s.offset(namelen as isize);
    while *value as ::core::ffi::c_int == ' ' as i32 || *value as ::core::ffi::c_int == '\t' as i32
    {
        value = value.offset(1);
    }
    if *value as ::core::ffi::c_int == '\0' as i32 {
        status_message_set(
            c,
            -(1 as ::core::ffi::c_int),
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            |out| out.write_all(b"User option must be @name value"),
        );
        return PROMPT_CLOSE;
    }
    let copy = std::ffi::CString::new(std::slice::from_raw_parts(s.cast::<u8>(), namelen))
        .expect("strcspn stops at the first NUL");
    let matched = options_match_owned(copy.as_c_str());
    if !matches!(&matched, Ok(parsed) if parsed.name.as_bytes().first() == Some(&b'@') && parsed.array_key.is_none())
    {
        what = if (*item).option_type as ::core::ffi::c_uint
            == WINDOW_CUSTOMIZE_HOOKS as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            b"hook\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"option\0" as *const u8 as *const ::core::ffi::c_char
        };
        status_message_set(
            c,
            -(1 as ::core::ffi::c_int),
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            |out| {
                out.write_all(b"User ")?;
                write_cstr(out, what)?;
                out.write_all(b" name must start with @")
            },
        );
        return PROMPT_CLOSE;
    }
    let name_owned = matched.expect("valid user option was checked").name;
    let name = name_owned.as_ptr();
    options_set_string((*item).oo, name, 0 as ::core::ffi::c_int, |out| {
        write_cstr(out, value)
    });
    if (*item).option_type as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_HOOKS as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        hooks_add_event(name);
    }
    options_push_changes(name);
    mode_tree_build((*data).data);
    mode_tree_draw((*data).data);
    (*(*data).wp).flags |= PANE_REDRAW;
    return PROMPT_CLOSE;
}
unsafe fn window_customize_add_option(
    mut c: *mut client,
    mut data: *mut window_customize_modedata,
    mut scope: window_customize_scope,
    mut oo: *mut options,
    mut type_0: window_customize_option_type,
) {
    let mut new_item: *mut window_customize_itemdata =
        ::core::ptr::null_mut::<window_customize_itemdata>();
    let prompt = if type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_HOOKS as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        c"New user hook: "
    } else {
        c"New user option: "
    };
    new_item = window_customize_new_item();
    (*new_item).data = data as *mut window_customize_modedata;
    (*new_item).type_0 = WINDOW_CUSTOMIZE_ITEM_OPTION;
    (*new_item).option_type = type_0;
    (*new_item).scope = scope;
    (*new_item).oo = oo;
    crate::src::shared::rc::retain(data);
    mode_tree_set_prompt(
        (*data).data,
        c,
        prompt.as_ptr(),
        b"@\0" as *const u8 as *const ::core::ffi::c_char,
        PROMPT_TYPE_COMMAND,
        PROMPT_NOFORMAT,
        window_customize_prompt_input_cb(window_customize_add_option_callback, new_item),
        window_customize_prompt_free_cb(window_customize_free_item_callback, new_item),
    );
}
unsafe fn window_customize_add_environment_callback(
    mut c: *mut client,
    mut itemdata: *mut window_customize_itemdata,
    s: Option<&CStr>,
    _key: prompt_key_result,
) -> prompt_result {
    let mut s = s.map_or(::core::ptr::null(), CStr::as_ptr);
    let mut item: *mut window_customize_itemdata = itemdata as *mut window_customize_itemdata;
    let mut data: *mut window_customize_modedata = (*item).data as *mut window_customize_modedata;
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 || (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    if item.is_null()
        || window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0
    {
        return PROMPT_CLOSE;
    }
    if *s as ::core::ffi::c_int == '-' as i32 {
        if *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '\0' as i32
            || !strchr(s.offset(1 as ::core::ffi::c_int as isize), '=' as i32).is_null()
        {
            status_message_set(
                c,
                -(1 as ::core::ffi::c_int),
                1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                |out| {
                    out.write_all(b"Bad environment variable: ")?;
                    write_cstr(out, s)
                },
            );
            return PROMPT_CLOSE;
        }
        environ_clear((*item).environ, s.offset(1 as ::core::ffi::c_int as isize));
    } else {
        value = strchr(s, '=' as i32);
        if value.is_null() || value == s {
            status_message_set(
                c,
                -(1 as ::core::ffi::c_int),
                1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                |out| out.write_all(b"Environment variable must be NAME=value"),
            );
            return PROMPT_CLOSE;
        }
        let name = CString::new(&CStr::from_ptr(s).to_bytes()[..value.offset_from(s) as usize])
            .expect("environment name contains no NUL");
        environ_set(
            (*item).environ,
            name.as_ptr(),
            0 as ::core::ffi::c_int,
            |out| write_cstr(out, value.offset(1 as ::core::ffi::c_int as isize)),
        );
    }
    mode_tree_build((*data).data);
    mode_tree_draw((*data).data);
    (*(*data).wp).flags |= PANE_REDRAW;
    return PROMPT_CLOSE;
}
unsafe fn window_customize_add_environment(
    mut c: *mut client,
    mut data: *mut window_customize_modedata,
    mut scope: window_customize_scope,
    mut env: *mut environ,
) {
    let mut new_item: *mut window_customize_itemdata =
        ::core::ptr::null_mut::<window_customize_itemdata>();
    new_item = window_customize_new_item();
    (*new_item).data = data as *mut window_customize_modedata;
    (*new_item).type_0 = WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT;
    (*new_item).scope = scope;
    (*new_item).environ = env;
    crate::src::shared::rc::retain(data);
    mode_tree_set_prompt(
        (*data).data,
        c,
        b"New environment: \0" as *const u8 as *const ::core::ffi::c_char,
        b"\0" as *const u8 as *const ::core::ffi::c_char,
        PROMPT_TYPE_COMMAND,
        PROMPT_NOFORMAT,
        window_customize_prompt_input_cb(window_customize_add_environment_callback, new_item),
        window_customize_prompt_free_cb(window_customize_free_item_callback, new_item),
    );
}
unsafe fn window_customize_edit_close_cb(
    buf: Option<Vec<u8>>,
    mut ed: *mut window_customize_editdata,
) {
    let mut current_block: u64;
    let mut item: *mut window_customize_itemdata = (*ed).item;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut data: *mut window_customize_modedata =
        ::core::ptr::null_mut::<window_customize_modedata>();
    let mut cause: Option<CString> = None;
    wp = window_pane_find_by_id((*ed).wp_id);
    if !wp.is_null() {
        wme = (*wp).modes.active;
        if !wme.is_null() && (*wme).mode == &raw const window_customize_mode {
            data = (*wme).data as *mut window_customize_modedata;
            if (*data).editor == (*ed).editor {
                (*data).editor = ::core::ptr::null_mut::<spawn_editor_state>();
                (*data).edit = ::core::ptr::null_mut::<window_customize_editdata>();
            }
        }
    }
    let Some(mut value) = buf else {
        window_customize_finish_edit(ed);
        return;
    };
    if value.is_empty() || data.is_null() || (*data).dead != 0 {
        window_customize_finish_edit(ed);
        return;
    }
    if value.last() == Some(&b'\n') {
        value.pop();
    }
    value.push(0);
    let value_ptr = value.as_ptr().cast::<::core::ffi::c_char>();
    match (*ed).edit_type as ::core::ffi::c_uint {
        0 => {
            if window_customize_option_editable(data, item) != 0
                && window_customize_set_option_value(item, value_ptr, &raw mut cause)
                    != 0 as ::core::ffi::c_int
            {
                current_block = 8846462416050848735;
            } else {
                current_block = 1608152415753874203;
            }
        }
        1 => {
            if window_customize_set_command_value(item, value_ptr, &raw mut cause)
                != 0 as ::core::ffi::c_int
            {
                current_block = 8846462416050848735;
            } else {
                current_block = 1608152415753874203;
            }
        }
        2 => {
            if window_customize_set_note_value(item, value_ptr) != 0 as ::core::ffi::c_int {
                current_block = 8846462416050848735;
            } else {
                current_block = 1608152415753874203;
            }
        }
        3 => {
            if window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>())
                == 0
            {
                current_block = 8846462416050848735;
            } else {
                window_customize_set_environment_value(item, value_ptr);
                current_block = 1608152415753874203;
            }
        }
        _ => {
            current_block = 1608152415753874203;
        }
    }
    match current_block {
        1608152415753874203 => {
            mode_tree_build((*data).data);
            mode_tree_draw((*data).data);
            (*wp).flags |= PANE_REDRAW;
        }
        _ => {}
    }
    drop(value);
    window_customize_finish_edit(ed);
}
unsafe fn window_customize_start_edit(
    mut data: *mut window_customize_modedata,
    mut item: *mut window_customize_itemdata,
    mut c: *mut client,
) {
    let mut ed: *mut window_customize_editdata =
        ::core::ptr::null_mut::<window_customize_editdata>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let value: Cow<'_, CStr>;
    let mut edit_type: window_customize_edit_type = WINDOW_CUSTOMIZE_EDIT_OPTION;
    if !(*data).editor.is_null() || item.is_null() {
        return;
    }
    if (*item).type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_ITEM_OPTION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if window_customize_option_editable(data, item) == 0 {
            return;
        }
        o = options_get(
            (*item).oo,
            ((*item).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        );
        if o.is_null() {
            return;
        }
        value = Cow::Owned(options_to_cstring(
            o,
            ((*item).array_key)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            0 as ::core::ffi::c_int,
        ));
        edit_type = WINDOW_CUSTOMIZE_EDIT_OPTION;
    } else if (*item).type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_ITEM_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        name = mode_tree_get_current_name((*data).data);
        if window_customize_get_key(item, ::core::ptr::null_mut::<*mut key_table>(), &raw mut bd)
            == 0
        {
            return;
        }
        if strcmp(
            name,
            b"Command\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            value = Cow::Owned(cmd_list_print_cstring(
                &*(*bd).cmdlist,
                0 as ::core::ffi::c_int,
            ));
            edit_type = WINDOW_CUSTOMIZE_EDIT_KEY_COMMAND;
        } else if strcmp(name, b"Note\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            value = Cow::Borrowed(if (*bd).note.is_none() {
                c""
            } else {
                CStr::from_ptr(
                    ((*bd).note)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                )
            });
            edit_type = WINDOW_CUSTOMIZE_EDIT_KEY_NOTE;
        } else {
            return;
        }
    } else if (*item).type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0 {
            return;
        }
        envent = environ_find(
            (*item).environ,
            ((*item).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        );
        if envent.is_null() || (*envent).value.is_none() {
            return;
        }
        value = Cow::Borrowed(CStr::from_ptr(
            ((*envent).value)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        ));
        edit_type = WINDOW_CUSTOMIZE_EDIT_ENVIRONMENT;
    } else {
        return;
    }
    // The editor callback and mode teardown borrow this stable record.
    ed = Box::into_raw(Box::new(window_customize_editdata {
        wp_id: (*(*data).wp).id,
        edit_type,
        item: window_customize_copy_item(item),
        editor: ::core::ptr::null_mut(),
    }));
    let mut buf = value.as_ref().as_ptr();
    let mut len = value.as_ref().to_bytes().len();
    if len == 0 as size_t {
        buf = b"\n\0" as *const u8 as *const ::core::ffi::c_char;
        len = 1 as size_t;
    }
    (*ed).editor = spawn_editor(
        c,
        buf,
        len,
        Some(Box::new(move |buf| unsafe {
            window_customize_edit_close_cb(buf, ed)
        })),
    );
    if (*ed).editor.is_null() {
        window_customize_finish_edit(ed);
    } else {
        (*data).editor = (*ed).editor;
        (*data).edit = ed as *mut window_customize_editdata;
    };
}
unsafe fn window_customize_set_option(
    mut c: *mut client,
    mut data: *mut window_customize_modedata,
    mut item: *mut window_customize_itemdata,
    mut global: ::core::ffi::c_int,
    mut pane: ::core::ffi::c_int,
) {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    let mut new_item: *mut window_customize_itemdata =
        ::core::ptr::null_mut::<window_customize_itemdata>();
    let mut flag: ::core::ffi::c_int = 0;
    let mut scope: window_customize_scope = WINDOW_CUSTOMIZE_NONE;
    let mut choice: u_int = 0;
    let mut name: *const ::core::ffi::c_char = ((*item).name)
        .as_ref()
        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut());
    let mut space: *const ::core::ffi::c_char = b"\0" as *const u8 as *const ::core::ffi::c_char;
    let mut array_key: *const ::core::ffi::c_char = ((*item).array_key)
        .as_ref()
        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut());
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    if item.is_null() || window_customize_check_item(data, item, &raw mut fs) == 0 {
        return;
    }
    o = options_get((*item).oo, name);
    if o.is_null() {
        return;
    }
    oe = options_table_entry(o);
    if !oe.is_null() && !(*oe).scope & OPTIONS_TABLE_PANE != 0 {
        pane = 0 as ::core::ffi::c_int;
    }
    if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
        scope = (*item).scope;
        oo = (*item).oo;
    } else {
        if global != 0 {
            match (*item).scope as ::core::ffi::c_uint {
                0 | 1 | 2 | 3 | 5 | 8 | 9 => {
                    scope = (*item).scope;
                }
                4 => {
                    scope = WINDOW_CUSTOMIZE_GLOBAL_SESSION;
                }
                6 | 7 => {
                    scope = WINDOW_CUSTOMIZE_GLOBAL_WINDOW;
                }
                _ => {}
            }
        } else {
            match (*item).scope as ::core::ffi::c_uint {
                0 | 1 | 2 | 4 => {
                    scope = (*item).scope;
                }
                6 | 7 => {
                    if pane != 0 {
                        scope = WINDOW_CUSTOMIZE_PANE;
                    } else {
                        scope = WINDOW_CUSTOMIZE_WINDOW;
                    }
                }
                3 => {
                    scope = WINDOW_CUSTOMIZE_SESSION;
                }
                5 => {
                    if pane != 0 {
                        scope = WINDOW_CUSTOMIZE_PANE;
                    } else {
                        scope = WINDOW_CUSTOMIZE_WINDOW;
                    }
                }
                8 | 9 => {
                    scope = (*item).scope;
                }
                _ => {}
            }
        }
        if scope as ::core::ffi::c_uint == (*item).scope as ::core::ffi::c_uint {
            oo = (*item).oo;
        } else {
            oo = window_customize_get_tree(scope, &raw mut fs);
        }
    }
    if !oe.is_null()
        && (*oe).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_FLAG as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        flag = options_get_number(oo, name) as ::core::ffi::c_int;
        options_set_number(
            oo,
            name,
            (flag == 0) as ::core::ffi::c_int as ::core::ffi::c_longlong,
        );
    } else if !oe.is_null()
        && (*oe).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_CHOICE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        choice = options_get_number(oo, name) as u_int;
        if (*(*oe)
            .choices
            .offset(choice.wrapping_add(1 as u_int) as isize))
        .is_null()
        {
            choice = 0 as u_int;
        } else {
            choice = choice.wrapping_add(1);
        }
        options_set_number(oo, name, choice as ::core::ffi::c_longlong);
    } else {
        let scope_text = window_customize_scope_text(scope, &fs);
        if !scope_text.as_bytes().is_empty() {
            space = b", for \0" as *const u8 as *const ::core::ffi::c_char;
        } else if scope as ::core::ffi::c_uint
            != WINDOW_CUSTOMIZE_SERVER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            space = b", global\0" as *const u8 as *const ::core::ffi::c_char;
        }
        let mut prompt_bytes = Vec::new();
        prompt_bytes.extend_from_slice(b"(");
        prompt_bytes.extend_from_slice(CStr::from_ptr(name).to_bytes());
        if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
            if array_key.is_null() {
                prompt_bytes.extend_from_slice(b"[+]");
            } else {
                prompt_bytes.extend_from_slice(b"[");
                prompt_bytes.extend_from_slice(CStr::from_ptr(array_key).to_bytes());
                prompt_bytes.extend_from_slice(b"]");
            }
        }
        prompt_bytes.extend_from_slice(CStr::from_ptr(space).to_bytes());
        prompt_bytes.extend_from_slice(scope_text.as_bytes());
        prompt_bytes.extend_from_slice(b") ");
        let prompt = CString::new(prompt_bytes).expect("option prompt contains no NUL");
        drop(scope_text);
        let value = options_to_cstring(o, array_key, 0 as ::core::ffi::c_int);
        new_item = window_customize_new_item();
        (*new_item).data = data as *mut window_customize_modedata;
        (*new_item).type_0 = WINDOW_CUSTOMIZE_ITEM_OPTION;
        (*new_item).option_type = (*item).option_type;
        (*new_item).scope = scope;
        (*new_item).oo = oo;
        window_customize_set_name(&mut *new_item, Some(CStr::from_ptr(name)));
        if !array_key.is_null() {
            window_customize_set_item_array_key(&mut *new_item, Some(CStr::from_ptr(array_key)));
        }
        crate::src::shared::rc::retain(data);
        mode_tree_set_prompt(
            (*data).data,
            c,
            prompt.as_ptr(),
            value.as_ptr(),
            PROMPT_TYPE_COMMAND,
            PROMPT_NOFORMAT,
            window_customize_prompt_input_cb(window_customize_set_option_callback, new_item),
            window_customize_prompt_free_cb(window_customize_free_item_callback, new_item),
        );
    };
}
unsafe fn window_customize_set_array_key_callback(
    mut c: *mut client,
    mut itemdata: *mut window_customize_itemdata,
    s: Option<&CStr>,
    _key: prompt_key_result,
) -> prompt_result {
    let mut s = s.map_or(::core::ptr::null(), CStr::as_ptr);
    let mut item: *mut window_customize_itemdata = itemdata as *mut window_customize_itemdata;
    let mut data: *mut window_customize_modedata =
        ::core::ptr::null_mut::<window_customize_modedata>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut array_key: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cause: Option<CString> = None;
    if item.is_null() {
        return PROMPT_CLOSE;
    }
    data = (*item).data as *mut window_customize_modedata;
    if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 || (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    name = ((*item).name)
        .as_ref()
        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut());
    array_key = ((*item).array_key)
        .as_ref()
        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut());
    if array_key.is_null()
        || window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0
    {
        return PROMPT_CLOSE;
    }
    o = options_get((*item).oo, name);
    if o.is_null() {
        return PROMPT_CLOSE;
    }
    if !options_array_get(o, s).is_null() {
        return PROMPT_CLOSE;
    }
    let value = options_to_cstring(o, array_key, 0 as ::core::ffi::c_int);
    if options_array_set(
        o,
        s,
        value.as_ptr(),
        0 as ::core::ffi::c_int,
        &raw mut cause,
    ) != 0 as ::core::ffi::c_int
    {
        drop(value);
        window_customize_uppercase_cause(&mut cause);
        status_message_set(
            c,
            -(1 as ::core::ffi::c_int),
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            |out| write_cstr(out, cause.as_ref().unwrap().as_ptr()),
        );
        return PROMPT_CLOSE;
    } else {
        drop(value);
        options_array_set(
            o,
            array_key,
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<Option<CString>>(),
        );
        options_push_changes(
            ((*item).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        );
        mode_tree_build((*data).data);
        mode_tree_draw((*data).data);
        (*(*data).wp).flags |= PANE_REDRAW;
        return PROMPT_CLOSE;
    };
}
unsafe fn window_customize_set_array_key(
    mut c: *mut client,
    mut data: *mut window_customize_modedata,
    mut item: *mut window_customize_itemdata,
) {
    let mut new_item: *mut window_customize_itemdata =
        ::core::ptr::null_mut::<window_customize_itemdata>();
    if item.is_null()
        || (*item).array_key.is_none()
        || window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0
    {
        return;
    }
    let mut prompt_bytes = Vec::new();
    prompt_bytes.extend_from_slice(b"(");
    prompt_bytes.extend_from_slice(
        CStr::from_ptr(
            ((*item).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        )
        .to_bytes(),
    );
    prompt_bytes.extend_from_slice(b"[");
    prompt_bytes.extend_from_slice(
        CStr::from_ptr(
            ((*item).array_key)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        )
        .to_bytes(),
    );
    prompt_bytes.extend_from_slice(b"]) ");
    let prompt = CString::new(prompt_bytes).expect("array-key prompt contains no NUL");
    new_item = window_customize_new_item();
    (*new_item).data = data as *mut window_customize_modedata;
    (*new_item).type_0 = WINDOW_CUSTOMIZE_ITEM_OPTION;
    (*new_item).option_type = (*item).option_type;
    (*new_item).scope = (*item).scope;
    (*new_item).oo = (*item).oo;
    window_customize_set_name(&mut *new_item, (*item).name.as_deref());
    window_customize_set_item_array_key(&mut *new_item, (*item).array_key.as_deref());
    crate::src::shared::rc::retain(data);
    mode_tree_set_prompt(
        (*data).data,
        c,
        prompt.as_ptr(),
        ((*item).array_key)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        PROMPT_TYPE_COMMAND,
        PROMPT_NOFORMAT,
        window_customize_prompt_input_cb(window_customize_set_array_key_callback, new_item),
        window_customize_prompt_free_cb(window_customize_free_item_callback, new_item),
    );
}
unsafe fn window_customize_unset_environment(
    mut data: *mut window_customize_modedata,
    mut item: *mut window_customize_itemdata,
) {
    if item.is_null()
        || window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0
    {
        return;
    }
    if environ_find(
        (*item).environ,
        ((*item).name)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
    )
    .is_null()
    {
        return;
    }
    if item == mode_tree_get_current((*data).data) as *mut window_customize_itemdata {
        mode_tree_up((*data).data, 0 as ::core::ffi::c_int);
    }
    environ_unset(
        (*item).environ,
        ((*item).name)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
    );
}
unsafe fn window_customize_unset_option(
    mut data: *mut window_customize_modedata,
    mut item: *mut window_customize_itemdata,
) {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    if item.is_null()
        || window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0
    {
        return;
    }
    o = options_get(
        (*item).oo,
        ((*item).name)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
    );
    if o.is_null() {
        return;
    }
    if !(*item).array_key.is_none()
        && item == mode_tree_get_current((*data).data) as *mut window_customize_itemdata
    {
        mode_tree_up((*data).data, 0 as ::core::ffi::c_int);
    }
    options_remove_or_default(
        o,
        ((*item).array_key)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        ::core::ptr::null_mut::<Option<CString>>(),
    );
}
unsafe fn window_customize_reset_option(
    mut data: *mut window_customize_modedata,
    mut item: *mut window_customize_itemdata,
) {
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    if item.is_null()
        || window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0
    {
        return;
    }
    if !(*item).array_key.is_none() {
        return;
    }
    oo = (*item).oo;
    while !oo.is_null() {
        o = options_get_only(
            oo,
            ((*item).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        );
        if !o.is_null() {
            options_remove_or_default(
                o,
                ::core::ptr::null::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<Option<CString>>(),
            );
        }
        oo = options_get_parent(oo);
    }
}
unsafe fn window_customize_set_command_callback(
    mut c: *mut client,
    mut itemdata: *mut window_customize_itemdata,
    s: Option<&CStr>,
    _key: prompt_key_result,
) -> prompt_result {
    let mut s = s.map_or(::core::ptr::null(), CStr::as_ptr);
    let mut item: *mut window_customize_itemdata = itemdata as *mut window_customize_itemdata;
    let mut data: *mut window_customize_modedata = (*item).data as *mut window_customize_modedata;
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 || (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    if item.is_null()
        || window_customize_get_key(item, ::core::ptr::null_mut::<*mut key_table>(), &raw mut bd)
            == 0
    {
        return PROMPT_CLOSE;
    }
    let mut pr = cmd_parse_from_string(
        CStr::from_ptr(s),
        ::core::ptr::null_mut::<cmd_parse_input>(),
    );
    match pr.status as ::core::ffi::c_uint {
        0 => {
            cmd_parse_error_uppercase_first(&mut pr.error);
            status_message_set(
                c,
                -(1 as ::core::ffi::c_int),
                1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                |out| {
                    write_cstr(
                        out,
                        pr.error
                            .as_ref()
                            .map_or(::core::ptr::null(), |cause| cause.as_ptr()),
                    )
                },
            );
            return PROMPT_CLOSE;
        }
        1 | _ => {
            cmd_list_free((*bd).cmdlist);
            (*bd).cmdlist = pr.cmdlist;
            mode_tree_build((*data).data);
            mode_tree_draw((*data).data);
            (*(*data).wp).flags |= PANE_REDRAW;
            return PROMPT_CLOSE;
        }
    };
}
unsafe fn window_customize_set_note_callback(
    _c: *mut client,
    mut itemdata: *mut window_customize_itemdata,
    s: Option<&CStr>,
    _key: prompt_key_result,
) -> prompt_result {
    let mut s = s.map_or(::core::ptr::null(), CStr::as_ptr);
    let mut item: *mut window_customize_itemdata = itemdata as *mut window_customize_itemdata;
    let mut data: *mut window_customize_modedata = (*item).data as *mut window_customize_modedata;
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 || (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    if item.is_null()
        || window_customize_get_key(item, ::core::ptr::null_mut::<*mut key_table>(), &raw mut bd)
            == 0
    {
        return PROMPT_CLOSE;
    }
    key_bindings_set_note(bd, Some(CStr::from_ptr(s)));
    mode_tree_build((*data).data);
    mode_tree_draw((*data).data);
    (*(*data).wp).flags |= PANE_REDRAW;
    return PROMPT_CLOSE;
}
fn window_customize_key_prompt(key_string: &CStr) -> CString {
    let mut prompt = Vec::with_capacity(key_string.to_bytes().len() + 3);
    prompt.extend_from_slice(b"(");
    prompt.extend_from_slice(key_string.to_bytes());
    prompt.extend_from_slice(b") ");
    CString::new(prompt).expect("formatted key contains no NUL")
}

unsafe fn window_customize_set_key(
    mut c: *mut client,
    mut data: *mut window_customize_modedata,
    mut item: *mut window_customize_itemdata,
) {
    let mut key: key_code = (*item).key;
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut new_item: *mut window_customize_itemdata =
        ::core::ptr::null_mut::<window_customize_itemdata>();
    if item.is_null()
        || window_customize_get_key(item, ::core::ptr::null_mut::<*mut key_table>(), &raw mut bd)
            == 0
    {
        return;
    }
    s = mode_tree_get_current_name((*data).data);
    if strcmp(s, b"Repeat\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        (*bd).flags ^= KEY_BINDING_REPEAT;
    } else if strcmp(s, b"Command\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        let key_string = key_string_format(key, false);
        let prompt = window_customize_key_prompt(&key_string);
        let value = cmd_list_print_cstring(&*(*bd).cmdlist, 0 as ::core::ffi::c_int);
        new_item = window_customize_new_item();
        (*new_item).data = data as *mut window_customize_modedata;
        (*new_item).type_0 = WINDOW_CUSTOMIZE_ITEM_KEY;
        (*new_item).scope = (*item).scope;
        window_customize_set_table(&mut *new_item, (*item).table.as_deref());
        (*new_item).key = key;
        crate::src::shared::rc::retain(data);
        mode_tree_set_prompt(
            (*data).data,
            c,
            prompt.as_ptr(),
            value.as_ptr(),
            PROMPT_TYPE_COMMAND,
            PROMPT_NOFORMAT,
            window_customize_prompt_input_cb(window_customize_set_command_callback, new_item),
            window_customize_prompt_free_cb(window_customize_free_item_callback, new_item),
        );
    } else if strcmp(s, b"Note\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        let key_string = key_string_format(key, false);
        let prompt = window_customize_key_prompt(&key_string);
        new_item = window_customize_new_item();
        (*new_item).data = data as *mut window_customize_modedata;
        (*new_item).type_0 = WINDOW_CUSTOMIZE_ITEM_KEY;
        (*new_item).scope = (*item).scope;
        window_customize_set_table(&mut *new_item, (*item).table.as_deref());
        (*new_item).key = key;
        crate::src::shared::rc::retain(data);
        mode_tree_set_prompt(
            (*data).data,
            c,
            prompt.as_ptr(),
            if (*bd).note.is_none() {
                b"\0" as *const u8 as *const ::core::ffi::c_char
            } else {
                ((*bd).note)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
            },
            PROMPT_TYPE_COMMAND,
            PROMPT_NOFORMAT,
            window_customize_prompt_input_cb(window_customize_set_note_callback, new_item),
            window_customize_prompt_free_cb(window_customize_free_item_callback, new_item),
        );
    }
}
unsafe fn window_customize_add_key_callback(
    mut c: *mut client,
    mut itemdata: *mut window_customize_itemdata,
    s: Option<&CStr>,
    _key0: prompt_key_result,
) -> prompt_result {
    let mut s = s.map_or(::core::ptr::null(), CStr::as_ptr);
    let mut item: *mut window_customize_itemdata = itemdata as *mut window_customize_itemdata;
    let mut data: *mut window_customize_modedata = (*item).data as *mut window_customize_modedata;
    let mut key: key_code = 0;
    let mut command: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut keylen: size_t = 0;
    if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 || (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    keylen = strcspn(s, b" \t\0" as *const u8 as *const ::core::ffi::c_char) as size_t;
    if keylen == 0 as size_t || *s.offset(keylen as isize) as ::core::ffi::c_int == '\0' as i32 {
        status_message_set(
            c,
            -(1 as ::core::ffi::c_int),
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            |out| out.write_all(b"Key binding must be key command"),
        );
        return PROMPT_CLOSE;
    }
    command = s.offset(keylen as isize);
    while *command as ::core::ffi::c_int == ' ' as i32
        || *command as ::core::ffi::c_int == '\t' as i32
    {
        command = command.offset(1);
    }
    if *command as ::core::ffi::c_int == '\0' as i32 {
        status_message_set(
            c,
            -(1 as ::core::ffi::c_int),
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            |out| out.write_all(b"Key binding must be key command"),
        );
        return PROMPT_CLOSE;
    }
    // strcspn stops before the first NUL, so this prefix is a valid C string.
    let keystr = std::ffi::CString::new(std::slice::from_raw_parts(s.cast::<u8>(), keylen))
        .expect("key name contains no NUL");
    key = key_string_parse_cstr(&keystr).unwrap_or(KEYC_UNKNOWN);
    if key == KEYC_NONE as ::core::ffi::c_ulong as key_code
        || key == KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code
    {
        status_message_set(
            c,
            -(1 as ::core::ffi::c_int),
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            |out| {
                out.write_all(b"Unknown key: ")?;
                write_cstr(out, keystr.as_ptr())
            },
        );
        return PROMPT_CLOSE;
    }
    drop(keystr);
    let mut pr = cmd_parse_from_string(
        CStr::from_ptr(command),
        ::core::ptr::null_mut::<cmd_parse_input>(),
    );
    match pr.status as ::core::ffi::c_uint {
        0 => {
            cmd_parse_error_uppercase_first(&mut pr.error);
            status_message_set(
                c,
                -(1 as ::core::ffi::c_int),
                1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                |out| {
                    write_cstr(
                        out,
                        pr.error
                            .as_ref()
                            .map_or(::core::ptr::null(), |cause| cause.as_ptr()),
                    )
                },
            );
            return PROMPT_CLOSE;
        }
        1 | _ => {
            key_bindings_add(
                ((*item).table)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                key,
                ::core::ptr::null::<::core::ffi::c_char>(),
                0 as ::core::ffi::c_int,
                pr.cmdlist,
            );
            mode_tree_build((*data).data);
            mode_tree_draw((*data).data);
            (*(*data).wp).flags |= PANE_REDRAW;
            return PROMPT_CLOSE;
        }
    };
}
unsafe fn window_customize_add_key(
    mut c: *mut client,
    mut data: *mut window_customize_modedata,
    mut table: *const ::core::ffi::c_char,
) {
    let mut new_item: *mut window_customize_itemdata =
        ::core::ptr::null_mut::<window_customize_itemdata>();
    let mut prompt_bytes = b"New key in ".to_vec();
    prompt_bytes.extend_from_slice(CStr::from_ptr(table).to_bytes());
    prompt_bytes.extend_from_slice(b": ");
    let prompt = CString::new(prompt_bytes).expect("key table name contains no NUL");
    new_item = window_customize_new_item();
    (*new_item).data = data as *mut window_customize_modedata;
    (*new_item).type_0 = WINDOW_CUSTOMIZE_ITEM_KEY;
    (*new_item).scope = WINDOW_CUSTOMIZE_KEY;
    window_customize_set_table(&mut *new_item, Some(CStr::from_ptr(table)));
    crate::src::shared::rc::retain(data);
    mode_tree_set_prompt(
        (*data).data,
        c,
        prompt.as_ptr(),
        b"\0" as *const u8 as *const ::core::ffi::c_char,
        PROMPT_TYPE_COMMAND,
        PROMPT_NOFORMAT,
        window_customize_prompt_input_cb(window_customize_add_key_callback, new_item),
        window_customize_prompt_free_cb(window_customize_free_item_callback, new_item),
    );
}
unsafe fn window_customize_unset_key(
    mut data: *mut window_customize_modedata,
    mut item: *mut window_customize_itemdata,
) {
    let mut kt: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    if item.is_null() || window_customize_get_key(item, &raw mut kt, &raw mut bd) == 0 {
        return;
    }
    if item == mode_tree_get_current((*data).data) as *mut window_customize_itemdata {
        mode_tree_up((*data).data, 0 as ::core::ffi::c_int);
    }
    key_bindings_remove(((*kt).name).as_ptr().cast_mut(), (*bd).key);
}
unsafe fn window_customize_reset_key(
    mut data: *mut window_customize_modedata,
    mut item: *mut window_customize_itemdata,
) {
    let mut kt: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut dd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    if item.is_null() || window_customize_get_key(item, &raw mut kt, &raw mut bd) == 0 {
        return;
    }
    dd = key_bindings_get_default(kt, (*bd).key);
    if !dd.is_null() && (*bd).cmdlist == (*dd).cmdlist {
        return;
    }
    if dd.is_null() && item == mode_tree_get_current((*data).data) as *mut window_customize_itemdata
    {
        mode_tree_up((*data).data, 0 as ::core::ffi::c_int);
    }
    key_bindings_reset(((*kt).name).as_ptr().cast_mut(), (*bd).key);
}
unsafe fn window_customize_change_each(
    mut data: *mut window_customize_modedata,
    mut item: *mut window_customize_itemdata,
) {
    let mut type_0: window_customize_item_type = (*item).type_0;
    let name = if type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_ITEM_OPTION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        Some(
            CStr::from_ptr(
                ((*item).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            )
            .to_owned(),
        )
    } else {
        None
    };
    match (*data).change as ::core::ffi::c_uint {
        0 => {
            if type_0 as ::core::ffi::c_uint
                == WINDOW_CUSTOMIZE_ITEM_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                window_customize_unset_key(data, item);
            } else if type_0 as ::core::ffi::c_uint
                == WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                window_customize_unset_environment(data, item);
            } else {
                window_customize_unset_option(data, item);
            }
        }
        1 => {
            if type_0 as ::core::ffi::c_uint
                == WINDOW_CUSTOMIZE_ITEM_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                window_customize_reset_key(data, item);
            } else if type_0 as ::core::ffi::c_uint
                == WINDOW_CUSTOMIZE_ITEM_OPTION as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                window_customize_reset_option(data, item);
            }
        }
        _ => {}
    }
    if type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_ITEM_OPTION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        options_push_changes(name.as_ref().expect("option name was copied").as_ptr());
    }
}
unsafe fn window_customize_change_current_callback(
    _c: *mut client,
    mut data: *mut window_customize_modedata,
    s: Option<&CStr>,
    _key: prompt_key_result,
) -> prompt_result {
    let mut s = s.map_or(::core::ptr::null(), CStr::as_ptr);
    let mut item: *mut window_customize_itemdata =
        ::core::ptr::null_mut::<window_customize_itemdata>();
    let mut type_0: window_customize_item_type = WINDOW_CUSTOMIZE_ITEM_OPTION;
    if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 || (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    if ({
        let mut __res: ::core::ffi::c_int = 0;
        if ::core::mem::size_of::<u_char>() as usize > 1 as usize {
            if 0 != 0 {
                let mut __c: ::core::ffi::c_int =
                    *s.offset(0 as ::core::ffi::c_int as isize) as u_char as ::core::ffi::c_int;
                __res = (if __c < -(128 as ::core::ffi::c_int) || __c > 255 as ::core::ffi::c_int {
                    __c as __int32_t
                } else {
                    *(*__ctype_tolower_loc()).offset(__c as isize)
                }) as ::core::ffi::c_int;
            } else {
                __res = tolower(
                    *s.offset(0 as ::core::ffi::c_int as isize) as u_char as ::core::ffi::c_int
                );
            }
        } else {
            __res = *(*__ctype_tolower_loc()).offset(*s.offset(0 as ::core::ffi::c_int as isize)
                as u_char as ::core::ffi::c_int
                as isize) as ::core::ffi::c_int;
        }
        __res
    }) != 'y' as i32
        || *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32
    {
        return PROMPT_CLOSE;
    }
    item = mode_tree_get_current((*data).data) as *mut window_customize_itemdata;
    if item.is_null() {
        return PROMPT_CLOSE;
    }
    type_0 = (*item).type_0;
    let name = if type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_ITEM_OPTION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        Some(
            CStr::from_ptr(
                ((*item).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            )
            .to_owned(),
        )
    } else {
        None
    };
    match (*data).change as ::core::ffi::c_uint {
        0 => {
            if type_0 as ::core::ffi::c_uint
                == WINDOW_CUSTOMIZE_ITEM_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                window_customize_unset_key(data, item);
            } else if type_0 as ::core::ffi::c_uint
                == WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                window_customize_unset_environment(data, item);
            } else {
                window_customize_unset_option(data, item);
            }
        }
        1 => {
            if type_0 as ::core::ffi::c_uint
                == WINDOW_CUSTOMIZE_ITEM_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                window_customize_reset_key(data, item);
            } else if type_0 as ::core::ffi::c_uint
                == WINDOW_CUSTOMIZE_ITEM_OPTION as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                window_customize_reset_option(data, item);
            }
        }
        _ => {}
    }
    if type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_ITEM_OPTION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        options_push_changes(name.as_ref().expect("option name was copied").as_ptr());
    }
    mode_tree_build((*data).data);
    mode_tree_draw((*data).data);
    (*(*data).wp).flags |= PANE_REDRAW;
    return PROMPT_CLOSE;
}
unsafe fn window_customize_change_tagged_callback(
    mut c: *mut client,
    mut data: *mut window_customize_modedata,
    s: Option<&CStr>,
    _key: prompt_key_result,
) -> prompt_result {
    let mut s = s.map_or(::core::ptr::null(), CStr::as_ptr);
    if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 || (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    if ({
        let mut __res: ::core::ffi::c_int = 0;
        if ::core::mem::size_of::<u_char>() as usize > 1 as usize {
            if 0 != 0 {
                let mut __c: ::core::ffi::c_int =
                    *s.offset(0 as ::core::ffi::c_int as isize) as u_char as ::core::ffi::c_int;
                __res = (if __c < -(128 as ::core::ffi::c_int) || __c > 255 as ::core::ffi::c_int {
                    __c as __int32_t
                } else {
                    *(*__ctype_tolower_loc()).offset(__c as isize)
                }) as ::core::ffi::c_int;
            } else {
                __res = tolower(
                    *s.offset(0 as ::core::ffi::c_int as isize) as u_char as ::core::ffi::c_int
                );
            }
        } else {
            __res = *(*__ctype_tolower_loc()).offset(*s.offset(0 as ::core::ffi::c_int as isize)
                as u_char as ::core::ffi::c_int
                as isize) as ::core::ffi::c_int;
        }
        __res
    }) != 'y' as i32
        || *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32
    {
        return PROMPT_CLOSE;
    }
    mode_tree_each_tagged(
        (*data).data,
        |row, _, _| unsafe { window_customize_change_each(data, (*row).itemdata.cast()) },
        c,
        KEYC_NONE as ::core::ffi::c_ulong as key_code,
        0 as ::core::ffi::c_int,
    );
    mode_tree_build((*data).data);
    mode_tree_draw((*data).data);
    (*(*data).wp).flags |= PANE_REDRAW;
    return PROMPT_CLOSE;
}
unsafe fn window_customize_add_current(
    mut c: *mut client,
    mut data: *mut window_customize_modedata,
) -> ::core::ffi::c_int {
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut table: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    name = mode_tree_get_current_name((*data).data);
    if cmd_find_valid_state(&raw mut (*data).fs) != 0 {
        cmd_find_copy_state(&raw mut fs, &raw mut (*data).fs);
    } else {
        cmd_find_from_pane(&raw mut fs, (*data).wp, 0 as ::core::ffi::c_int);
    }
    if strcmp(
        name,
        b"Server Options\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        window_customize_add_option(
            c,
            data,
            WINDOW_CUSTOMIZE_SERVER,
            global_options,
            WINDOW_CUSTOMIZE_OPTIONS,
        );
        return 1 as ::core::ffi::c_int;
    }
    if strcmp(
        name,
        b"Session Options\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        window_customize_add_option(
            c,
            data,
            WINDOW_CUSTOMIZE_SESSION,
            (*fs.s).options,
            WINDOW_CUSTOMIZE_OPTIONS,
        );
        return 1 as ::core::ffi::c_int;
    }
    if strcmp(
        name,
        b"Window & Pane Options\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        window_customize_add_option(
            c,
            data,
            WINDOW_CUSTOMIZE_PANE,
            (*fs.wp).options,
            WINDOW_CUSTOMIZE_OPTIONS,
        );
        return 1 as ::core::ffi::c_int;
    }
    if strcmp(
        name,
        b"Session Hooks\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        window_customize_add_option(
            c,
            data,
            WINDOW_CUSTOMIZE_SESSION,
            (*fs.s).options,
            WINDOW_CUSTOMIZE_HOOKS,
        );
        return 1 as ::core::ffi::c_int;
    }
    if strcmp(
        name,
        b"Window & Pane Hooks\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        window_customize_add_option(
            c,
            data,
            WINDOW_CUSTOMIZE_PANE,
            (*fs.wp).options,
            WINDOW_CUSTOMIZE_HOOKS,
        );
        return 1 as ::core::ffi::c_int;
    }
    if strcmp(
        name,
        b"Global Environment\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        window_customize_add_environment(
            c,
            data,
            WINDOW_CUSTOMIZE_GLOBAL_ENVIRONMENT,
            global_environ,
        );
        return 1 as ::core::ffi::c_int;
    }
    if strcmp(
        name,
        b"Session Environment\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        window_customize_add_environment(
            c,
            data,
            WINDOW_CUSTOMIZE_SESSION_ENVIRONMENT,
            (*fs.s).environ,
        );
        return 1 as ::core::ffi::c_int;
    }
    if strncmp(
        name,
        b"Key Table - \0" as *const u8 as *const ::core::ffi::c_char,
        12 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        table = name.offset(12 as ::core::ffi::c_int as isize);
        window_customize_add_key(c, data, table);
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe fn window_customize_key(
    mut wme: *mut window_mode_entry,
    mut c: *mut client,
    _s: *mut session,
    _wl: *mut winlink,
    mut key: key_code,
    mut m: *mut mouse_event,
) {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_customize_modedata = (*wme).data as *mut window_customize_modedata;
    let mut item: *mut window_customize_itemdata =
        ::core::ptr::null_mut::<window_customize_itemdata>();
    let mut new_item: *mut window_customize_itemdata =
        ::core::ptr::null_mut::<window_customize_itemdata>();
    let mut finished: ::core::ffi::c_int = 0;
    let mut tagged: u_int = 0;
    item = mode_tree_get_current((*data).data) as *mut window_customize_itemdata;
    if !(*data).editor.is_null() {
        if key == 'q' as i32 as key_code
            || key == '\u{1b}' as i32 as key_code
            || key == '\u{3}' as i32 as key_code
        {
            finished = 1 as ::core::ffi::c_int;
        } else {
            finished = 0 as ::core::ffi::c_int;
        }
    } else {
        finished = mode_tree_key(
            (*data).data,
            c,
            &raw mut key,
            m,
            ::core::ptr::null_mut::<u_int>(),
            ::core::ptr::null_mut::<u_int>(),
        );
        new_item = mode_tree_get_current((*data).data) as *mut window_customize_itemdata;
        if item != new_item {
            item = new_item;
        }
        match key {
            101 => {
                window_customize_start_edit(data, item, c);
            }
            97 => {
                if !(item.is_null()
                    || (*item).type_0 as ::core::ffi::c_uint
                        != WINDOW_CUSTOMIZE_ITEM_OPTION as ::core::ffi::c_int
                            as ::core::ffi::c_uint)
                {
                    window_customize_set_array_key(c, data, item);
                }
            }
            13 | 115 => {
                if item.is_null() {
                    if window_customize_add_current(c, data) != 0 {
                        mode_tree_build((*data).data);
                    }
                } else {
                    if (*item).type_0 as ::core::ffi::c_uint
                        == WINDOW_CUSTOMIZE_ITEM_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        window_customize_set_key(c, data, item);
                    } else if (*item).type_0 as ::core::ffi::c_uint
                        == WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT as ::core::ffi::c_int
                            as ::core::ffi::c_uint
                    {
                        window_customize_set_environment(c, data, item, 0 as ::core::ffi::c_int);
                    } else {
                        window_customize_set_option(
                            c,
                            data,
                            item,
                            0 as ::core::ffi::c_int,
                            1 as ::core::ffi::c_int,
                        );
                        options_push_changes(
                            ((*item).name)
                                .as_ref()
                                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                        );
                    }
                    mode_tree_build((*data).data);
                }
            }
            119 => {
                if !(item.is_null()
                    || (*item).type_0 as ::core::ffi::c_uint
                        != WINDOW_CUSTOMIZE_ITEM_OPTION as ::core::ffi::c_int
                            as ::core::ffi::c_uint)
                {
                    window_customize_set_option(
                        c,
                        data,
                        item,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                    options_push_changes(
                        ((*item).name)
                            .as_ref()
                            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                    );
                    mode_tree_build((*data).data);
                }
            }
            83 | 87 => {
                if !(item.is_null()
                    || (*item).type_0 as ::core::ffi::c_uint
                        == WINDOW_CUSTOMIZE_ITEM_KEY as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    if (*item).type_0 as ::core::ffi::c_uint
                        == WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT as ::core::ffi::c_int
                            as ::core::ffi::c_uint
                    {
                        window_customize_set_environment(c, data, item, 1 as ::core::ffi::c_int);
                    } else {
                        window_customize_set_option(
                            c,
                            data,
                            item,
                            1 as ::core::ffi::c_int,
                            0 as ::core::ffi::c_int,
                        );
                        options_push_changes(
                            ((*item).name)
                                .as_ref()
                                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                        );
                    }
                    mode_tree_build((*data).data);
                }
            }
            100 => {
                if !(item.is_null()
                    || (*item).type_0 as ::core::ffi::c_uint
                        == WINDOW_CUSTOMIZE_ITEM_OPTION as ::core::ffi::c_int
                            as ::core::ffi::c_uint
                        && !(*item).array_key.is_none()
                    || (*item).type_0 as ::core::ffi::c_uint
                        == WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT as ::core::ffi::c_int
                            as ::core::ffi::c_uint)
                {
                    let mut prompt_bytes = b"Reset ".to_vec();
                    prompt_bytes.extend_from_slice(
                        CStr::from_ptr(
                            ((*item).name)
                                .as_ref()
                                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                        )
                        .to_bytes(),
                    );
                    prompt_bytes.extend_from_slice(b" to default? ");
                    let reset_prompt =
                        CString::new(prompt_bytes).expect("C string parts have no NUL");
                    crate::src::shared::rc::retain(data);
                    (*data).change = WINDOW_CUSTOMIZE_RESET;
                    mode_tree_set_prompt(
                        (*data).data,
                        c,
                        reset_prompt.as_ptr(),
                        b"\0" as *const u8 as *const ::core::ffi::c_char,
                        PROMPT_TYPE_COMMAND,
                        PROMPT_SINGLE | PROMPT_NOFORMAT | (*data).prompt_flags,
                        window_customize_prompt_input_cb(
                            window_customize_change_current_callback,
                            data,
                        ),
                        window_customize_prompt_free_cb(window_customize_free_callback, data),
                    );
                }
            }
            68 => {
                tagged = mode_tree_count_tagged((*data).data);
                if !(tagged == 0 as u_int) {
                    let reset_prompt = CString::new(format!("Reset {tagged} tagged to default? "))
                        .expect("formatted number has no NUL");
                    crate::src::shared::rc::retain(data);
                    (*data).change = WINDOW_CUSTOMIZE_RESET;
                    mode_tree_set_prompt(
                        (*data).data,
                        c,
                        reset_prompt.as_ptr(),
                        b"\0" as *const u8 as *const ::core::ffi::c_char,
                        PROMPT_TYPE_COMMAND,
                        PROMPT_SINGLE | PROMPT_NOFORMAT | (*data).prompt_flags,
                        window_customize_prompt_input_cb(
                            window_customize_change_tagged_callback,
                            data,
                        ),
                        window_customize_prompt_free_cb(window_customize_free_callback, data),
                    );
                }
            }
            117 => {
                if !item.is_null() {
                    let mut prompt_bytes = b"Unset ".to_vec();
                    prompt_bytes.extend_from_slice(
                        CStr::from_ptr(
                            ((*item).name)
                                .as_ref()
                                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                        )
                        .to_bytes(),
                    );
                    if !(*item).array_key.is_none() {
                        prompt_bytes.push(b'[');
                        prompt_bytes.extend_from_slice(
                            CStr::from_ptr(
                                ((*item).array_key)
                                    .as_ref()
                                    .map_or(::core::ptr::null_mut(), |value| {
                                        value.as_ptr().cast_mut()
                                    }),
                            )
                            .to_bytes(),
                        );
                        prompt_bytes.push(b']');
                    }
                    prompt_bytes.extend_from_slice(b"? ");
                    let prompt =
                        CString::new(prompt_bytes).expect("C strings have no interior NUL");
                    crate::src::shared::rc::retain(data);
                    (*data).change = WINDOW_CUSTOMIZE_UNSET;
                    mode_tree_set_prompt(
                        (*data).data,
                        c,
                        prompt.as_ptr(),
                        b"\0" as *const u8 as *const ::core::ffi::c_char,
                        PROMPT_TYPE_COMMAND,
                        PROMPT_SINGLE | PROMPT_NOFORMAT | (*data).prompt_flags,
                        window_customize_prompt_input_cb(
                            window_customize_change_current_callback,
                            data,
                        ),
                        window_customize_prompt_free_cb(window_customize_free_callback, data),
                    );
                }
            }
            85 => {
                tagged = mode_tree_count_tagged((*data).data);
                if !(tagged == 0 as u_int) {
                    let prompt = CString::new(format!("Unset {tagged} tagged? ")).unwrap();
                    crate::src::shared::rc::retain(data);
                    (*data).change = WINDOW_CUSTOMIZE_UNSET;
                    mode_tree_set_prompt(
                        (*data).data,
                        c,
                        prompt.as_ptr(),
                        b"\0" as *const u8 as *const ::core::ffi::c_char,
                        PROMPT_TYPE_COMMAND,
                        PROMPT_SINGLE | PROMPT_NOFORMAT | (*data).prompt_flags,
                        window_customize_prompt_input_cb(
                            window_customize_change_tagged_callback,
                            data,
                        ),
                        window_customize_prompt_free_cb(window_customize_free_callback, data),
                    );
                }
            }
            72 => {
                (*data).hide_global = ((*data).hide_global == 0) as ::core::ffi::c_int;
                mode_tree_build((*data).data);
            }
            67 => {
                (*data).hide_default = ((*data).hide_default == 0) as ::core::ffi::c_int;
                mode_tree_build((*data).data);
            }
            _ => {}
        }
    }
    if finished != 0 {
        window_pane_reset_mode(wp);
    } else {
        mode_tree_draw((*data).data);
        window_customize_draw_waiting(data);
        (*wp).flags |= PANE_REDRAW;
    };
}

#[cfg(test)]
mod tag_tests {
    use super::*;

    #[test]
    fn section_tags_match_tmux_encoding() {
        assert_eq!(
            window_customize_top_tag(CUSTOMIZE_SERVER_OPTIONS),
            (3_u64 << 62) | ((OPTIONS_TABLE_SERVER as u64) << 1) | 1
        );
        assert_eq!(
            window_customize_top_tag(CUSTOMIZE_SESSION_ENVIRONMENT),
            (3_u64 << 62) | (2_u64 << 8) | 3
        );
    }
}

#[cfg(test)]
mod item_owner_tests {
    use super::*;

    #[test]
    fn detached_copy_owns_byte_preserving_strings() {
        unsafe {
            let item = window_customize_new_item();
            let table = CString::new(b"\xfftable".to_vec()).unwrap();
            window_customize_set_table(&mut *item, Some(table.as_c_str()));
            window_customize_set_name(&mut *item, Some(c"first"));
            window_customize_set_item_array_key(&mut *item, Some(c""));
            let copy = window_customize_copy_item(item);

            window_customize_set_table(&mut *item, Some(c"changed"));
            window_customize_set_name(&mut *item, Some(c"second"));
            window_customize_free_item(item);

            assert_eq!(
                CStr::from_ptr(
                    ((*copy).table)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
                )
                .to_bytes(),
                b"\xfftable"
            );
            assert_eq!(
                CStr::from_ptr(
                    ((*copy).name)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
                )
                .to_bytes(),
                b"first"
            );
            assert_eq!(
                CStr::from_ptr(
                    ((*copy).array_key)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
                )
                .to_bytes(),
                b""
            );
            window_customize_free_item(copy);
        }
    }

    #[test]
    fn mode_list_keeps_items_stable_until_last_callback_reference() {
        unsafe {
            let data = crate::src::shared::rc::new(window_customize_modedata {
                wp: ::core::ptr::null_mut(),
                dead: 0,
                data: ::core::ptr::null_mut(),
                editor: ::core::ptr::null_mut(),
                edit: ::core::ptr::null_mut(),
                format: CString::new(Vec::new()).unwrap(),
                hide_global: 0,
                hide_default: 0,
                prompt_flags: 0,
                item_list: Vec::new(),
                fs: cmd_find_state {
                    flags: 0,
                    current: ::core::ptr::null_mut(),
                    s: ::core::ptr::null_mut(),
                    wl: ::core::ptr::null_mut(),
                    w: ::core::ptr::null_mut(),
                    wp: ::core::ptr::null_mut(),
                    idx: 0,
                },
                change: WINDOW_CUSTOMIZE_UNSET,
            });
            let first = window_customize_add_item(data);
            (*first).data = data;
            window_customize_set_name(&mut *first, Some(c"stable"));
            for _ in 0..128 {
                window_customize_add_item(data);
            }
            let first_address = {
                let items = &mut (*data).item_list;
                &mut *items[0] as *mut window_customize_itemdata
            };
            assert_eq!(first_address, first);
            let detached = window_customize_copy_item(first);
            crate::src::shared::rc::retain(data);

            window_customize_destroy(data);
            assert_eq!(crate::src::shared::rc::strong_count(data), 1);
            assert_eq!(
                CStr::from_ptr(
                    ((*detached).name)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
                )
                .to_bytes(),
                b"stable"
            );
            window_customize_free_item_callback(detached.cast());
        }
    }
}
