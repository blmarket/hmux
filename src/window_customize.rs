use crate::src::options::options_owner_ptr;
use crate::src::format::bytes::write_cstr;
use crate::src::format::bytes::xformat;
use crate::src::shared::mode_tree::ModeTreeItemSnapshot;
use refbox::RefBox;
use std::borrow::Cow;
use std::cell::UnsafeCell;
use std::ffi::{CStr, CString};
use std::ptr::NonNull;
use std::rc::{Rc, Weak};

use crate::src::arguments::{args_get, args_has};
use crate::src::cmd::find::{cmd_find_copy_state, cmd_find_from_pane, cmd_find_valid_state};
use crate::src::cmd::parse::{cmd_parse_error_uppercase_first, cmd_parse_from_string};
use crate::src::cmd::cmd_list_print_cstring;
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
    key_bindings_add, key_bindings_tables, key_bindings_get, key_bindings_get_default,
    key_bindings_get_table, key_bindings_remove, key_bindings_reset,
    key_bindings_set_note,
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
    options_array_get, options_array_get_index, options_array_item_key,
    options_array_set, options_create, options_default,
    options_default_to_cstring, options_free, options_from_string, options_get,
    options_get_fire_count, options_get_fire_time, options_get_monitor_data, options_get_number,
    options_get_only, options_get_parent, options_match_owned, options_name, options_owner, options_push_changes, options_remove_or_default, options_set_number,
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
    mode_tree_data, mode_tree_help_info, mode_tree_prompt_input_cb, ModeTreeItemData,
    ModeTreeItemRef,
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
use crate::src::spawn::{
    spawn_cancel_editor, spawn_editor, spawn_editor_write, spawn_get_editor_pid,
};
use crate::src::status::status_message_set;
use crate::src::style::style_apply;
use crate::src::tmux::{global_environ, global_options, global_s_options, global_w_options};
use crate::src::window::{window_pane_find_by_id, window_pane_index, window_pane_reset_mode, window_pane_upgrade};

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
    /// Nonowning allocation observer for callbacks receiving borrowed pointers.
    pub(crate) observer: Weak<std::cell::UnsafeCell<window_customize_modedata>>,
    pub wp: Weak<UnsafeCell<window_pane>>,
    pub dead: ::core::ffi::c_int,
    pub data: Option<std::rc::Rc<std::cell::UnsafeCell<mode_tree_data>>>,
    pub editor: *mut spawn_editor_state,
    pub format: CString,
    pub hide_global: ::core::ffi::c_int,
    pub hide_default: ::core::ffi::c_int,
    pub prompt_flags: ::core::ffi::c_int,
    item_list: Vec<refbox::RefBox<window_customize_itemdata>>,
    pub fs: cmd_find_state,
    pub change: window_customize_change,
}

impl window_customize_modedata {
    fn data_ptr(&self) -> *mut mode_tree_data {
        self.data.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr)
    }
}

pub type window_customize_change = ::core::ffi::c_uint;
pub const WINDOW_CUSTOMIZE_RESET: window_customize_change = 1;
pub const WINDOW_CUSTOMIZE_UNSET: window_customize_change = 0;
#[derive(Clone)]
pub struct window_customize_itemdata {
    pub type_0: window_customize_item_type,
    pub option_type: window_customize_option_type,
    pub scope: window_customize_scope,
    pub table: Option<std::ffi::CString>,
    pub key: key_code,
    pub oo: *mut options,
    environ: Option<CustomizeEnvironment>,
    pub environ_flags: ::core::ffi::c_int,
    pub name: Option<std::ffi::CString>,
    pub array_key: Option<std::ffi::CString>,
}

impl window_customize_itemdata {
    fn new() -> Self {
        window_customize_itemdata {
            type_0: 0,
            option_type: 0,
            scope: 0,
            table: None,
            key: 0,
            oo: ::core::ptr::null_mut(),
            environ: None,
            environ_flags: 0,
            name: None,
            array_key: None,
        }
    }
}

/// Rows keep target identity without retaining a session for the entire mode.
#[derive(Clone)]
enum CustomizeEnvironment {
    Global,
    Session(std::rc::Weak<UnsafeCell<session>>),
}

impl CustomizeEnvironment {
    unsafe fn session(s: *mut session) -> Self {
        Self::Session((*s).observer.clone())
    }

    fn matches(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Global, Self::Global) => true,
            (Self::Session(a), Self::Session(b)) => a.ptr_eq(b),
            _ => false,
        }
    }

    fn resolve(&self) -> Option<CustomizeEnvironmentBorrow> {
        match self {
            Self::Global => Some(CustomizeEnvironmentBorrow::Global),
            Self::Session(session) => session.upgrade().map(CustomizeEnvironmentBorrow::Session),
        }
    }
}

/// Retain a session only for the duration of an environment operation. All
/// environment references are tied to this guard, rather than a saved pointer.
enum CustomizeEnvironmentBorrow {
    Global,
    Session(Rc<UnsafeCell<session>>),
}

impl CustomizeEnvironmentBorrow {
    unsafe fn get(&self) -> Option<&environ> {
        match self {
            Self::Global => global_environ.as_deref(),
            Self::Session(owner) => (*crate::src::shared::rc::as_ptr(owner)).environ.as_deref(),
        }
    }

    unsafe fn get_mut(&mut self) -> Option<&mut environ> {
        match self {
            Self::Global => global_environ.as_deref_mut(),
            Self::Session(owner) => (*crate::src::shared::rc::as_ptr(owner))
                .environ
                .as_deref_mut(),
        }
    }
}

impl Drop for CustomizeEnvironmentBorrow {
    fn drop(&mut self) {
        if let Self::Session(owner) = self {
            // Transfer the guard's retained lifetime to the deferred release:
            // the field drops its original Rc after this clone is scheduled.
            crate::src::shared::rc::release_later(Rc::clone(owner));
        }
    }
}

// Detached prompt and editor copies own their strings independently of the rows.
fn window_customize_set_table(item: &mut window_customize_itemdata, value: Option<&CStr>) {
    item.table = value.map(CStr::to_owned);
}

fn window_customize_set_name(item: &mut window_customize_itemdata, value: Option<&CStr>) {
    item.name = value.map(CStr::to_owned);
}

fn window_customize_set_item_array_key(item: &mut window_customize_itemdata, value: Option<&CStr>) {
    item.array_key = value.map(CStr::to_owned);
}

fn window_customize_new_item() -> Box<window_customize_itemdata> {
    Box::new(window_customize_itemdata::new())
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
    pub item: Box<window_customize_itemdata>,
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

pub const WINDOW_CUSTOMIZE_DEFAULT_FORMAT: &CStr = c"#{?is_option,#{?option_is_global,,#[reverse](#{option_scope})#[default] }#[fg=themelightgrey]#[ignore]#{option_value}#{?option_unit, #{option_unit},},#{?is_environment,#[fg=themelightgrey]#[ignore]#{environment_value},#{key}}}";
static window_customize_menu_items: [menu_item<'static>; 11] = [
    menu_item {
        name: c"Select",
        key: '\r' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: c"Edit",
        key: 'e' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: c"Expand",
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code,
        command: None,
    },
    menu_item {
        name: c"",
        key: KEYC_NONE as ::core::ffi::c_ulong as key_code,
        command: None,
    },
    menu_item {
        name: c"Tag",
        key: 't' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: c"Tag All",
        key: '\u{14}' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: c"Tag None",
        key: 'T' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: c"",
        key: KEYC_NONE as ::core::ffi::c_ulong as key_code,
        command: None,
    },
    menu_item {
        name: c"Changed Only",
        key: 'C' as i32 as key_code,
        command: None,
    },
    menu_item {
        name: c"",
        key: KEYC_NONE as ::core::ffi::c_ulong as key_code,
        command: None,
    },
    menu_item {
        name: c"Cancel",
        key: 'q' as i32 as key_code,
        command: None,
    },
];
pub static window_customize_mode: window_mode = {
    window_mode {
        name: c"options-mode",
        default_format: Some(WINDOW_CUSTOMIZE_DEFAULT_FORMAT),
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
                    &std::rc::Rc<std::cell::UnsafeCell<client>>,
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
        4 => return options_owner_ptr(&mut (*(*fs).s_ptr()).options).map_or(std::ptr::null_mut(), |options| options),
        5 => return global_w_options,
        6 => return options_owner_ptr(&mut (*(*fs).w_ptr()).options).map_or(std::ptr::null_mut(), |options| options),
        7 => return options_owner_ptr(&mut (*(*fs).wp_ptr()).options).map_or(std::ptr::null_mut(), |options| options),
        8 | 9 => return ::core::ptr::null_mut::<options>(),
        _ => {}
    }
    return ::core::ptr::null_mut::<options>();
}
unsafe fn window_customize_get_environment(
    scope: window_customize_scope,
    fs: &cmd_find_state,
) -> Option<CustomizeEnvironment> {
    match scope {
        WINDOW_CUSTOMIZE_GLOBAL_ENVIRONMENT => Some(CustomizeEnvironment::Global),
        WINDOW_CUSTOMIZE_SESSION_ENVIRONMENT if !fs.s_ptr().is_null() => {
            Some(CustomizeEnvironment::session(fs.s_ptr()))
        }
        _ => None,
    }
}
unsafe fn window_customize_check_item(
    mut data: *mut window_customize_modedata,
    item: &window_customize_itemdata,
    mut fsp: *mut cmd_find_state,
) -> ::core::ffi::c_int {
    let Some(mode_pane_owner) = window_pane_upgrade(&(*data).wp) else {
        return 0;
    };
    let mode_pane = crate::src::shared::rc::as_ptr(&mode_pane_owner);
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: Default::default(),
        wl: Default::default(),
        w: Default::default(),
        wp: Default::default(),
        idx: 0,
    };
    if fsp.is_null() {
        fsp = &raw mut fs;
    }
    if cmd_find_valid_state(&(*data).fs) != 0 {
        cmd_find_copy_state(fsp, &raw mut (*data).fs);
    } else {
        cmd_find_from_pane(fsp, mode_pane, 0 as ::core::ffi::c_int);
    }
    if item.type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return (item
            .environ
            .as_ref()
            .zip(window_customize_get_environment(item.scope, &*fsp).as_ref())
            .is_some_and(|(a, b)| a.matches(b))) as ::core::ffi::c_int;
    }
    return (item.oo == window_customize_get_tree(item.scope, fsp)) as ::core::ffi::c_int;
}
unsafe fn window_customize_get_key_table(
    item: &window_customize_itemdata,
) -> Option<Rc<std::cell::RefCell<key_table>>> {
    let table = crate::src::key_bindings::key_bindings_get_table(std::ffi::CStr::from_ptr(item.table.as_ref()?.as_ptr()), 0)?;
    key_bindings_get(&table.borrow(), item.key)?;
    Some(table)
}
unsafe fn window_customize_scope_text(
    scope: window_customize_scope,
    fs: &cmd_find_state,
) -> CString {
    let mut idx: u_int = 0;
    match scope as ::core::ffi::c_uint {
        7 => {
            idx = window_pane_index(&*fs.wp_ptr()).expect("pane belongs to window ordering");
            CString::new(format!("pane {idx}")).expect("pane index contains no NUL")
        }
        4 | 9 => {
            let mut bytes = b"session ".to_vec();
            bytes.extend_from_slice((*fs.s_ptr()).name.as_bytes());
            CString::new(bytes).expect("session name contains no NUL")
        }
        6 => {
            CString::new(format!("window {}", (*fs.wl_ptr()).idx)).expect("window index contains no NUL")
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
    if options_get_monitor_data(&mut *o).is_some() {
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
fn window_customize_add_item(
    items: &mut Vec<refbox::RefBox<window_customize_itemdata>>,
    item: window_customize_itemdata,
) -> refbox::Weak<window_customize_itemdata> {
    let item = refbox::RefBox::new(item);
    let handle = item.downgrade();
    items.push(item);
    handle
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

fn window_customize_copy_item(item: &window_customize_itemdata) -> Box<window_customize_itemdata> {
    Box::new(item.clone())
}

unsafe fn window_customize_draw_waiting(mut data: *mut window_customize_modedata) {
    let Some(mode_pane_owner) = window_pane_upgrade(&(*data).wp) else {
        return;
    };
    let mode_pane = crate::src::shared::rc::as_ptr(&mode_pane_owner);
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: std::rc::Weak::new(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        item: None,
        scrolled: 0,
        bg: 0,
    };
    let mut s: *mut screen = (*mode_pane).screen;
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
        &mut ctx,
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
        &mut ctx,
        x.wrapping_add(1 as u_int) as ::core::ffi::c_int,
        y.wrapping_add(1 as u_int) as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_clearcharacter(&mut ctx, box_w.wrapping_sub(2 as u_int), gc.bg as u_int);
    screen_write_cursormove(
        &mut ctx,
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
    item: &window_customize_itemdata,
    mut s: *const ::core::ffi::c_char,
    mut cause: *mut Option<CString>,
) -> ::core::ffi::c_int {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    let mut oo: *mut options = item.oo;
    let mut name: *const ::core::ffi::c_char = (item.name)
        .as_ref()
        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut());
    let mut array_key: *const ::core::ffi::c_char = (item.array_key)
        .as_ref()
        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut());
    let mut idx: u_int = 0;
    let mut keybuf: [::core::ffi::c_char; 32] = [0; 32];
    o = options_get(oo, name);
    if o.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    oe = options_table_entry(&*(o)).map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry);
    if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
        if array_key.is_null() {
            idx = 0 as u_int;
            while idx < INT_MAX as u_int {
                if crate::src::options::options_array_get_index_mut(&mut *(o), idx).map_or(std::ptr::null_mut(), |value| value).is_null() {
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
    if item.option_type as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_HOOKS as ::core::ffi::c_int as ::core::ffi::c_uint
        && *name as ::core::ffi::c_int == '@' as i32
    {
        hooks_add_event(name);
    }
    options_push_changes(
        (item.name)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
    );
    return 0 as ::core::ffi::c_int;
}
unsafe fn window_customize_option_editable(
    mut data: *mut window_customize_modedata,
    item: &window_customize_itemdata,
) -> ::core::ffi::c_int {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    if item.type_0 as ::core::ffi::c_uint
        != WINDOW_CUSTOMIZE_ITEM_OPTION as ::core::ffi::c_int as ::core::ffi::c_uint
        || window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0
    {
        return 0 as ::core::ffi::c_int;
    }
    o = options_get(
        item.oo,
        (item.name)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
    );
    if o.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    oe = options_table_entry(&*(o)).map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry);
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
    item: &window_customize_itemdata,
    s: *const ::core::ffi::c_char,
    cause: *mut Option<CString>,
) -> ::core::ffi::c_int {
    let Some(table) = window_customize_get_key_table(item) else {
        return -(1 as ::core::ffi::c_int);
    };
    let mut kt = table.borrow_mut();
    let mut pr = cmd_parse_from_string(
        CStr::from_ptr(s),
        ::core::ptr::null_mut::<cmd_parse_input>(),
    );
    if pr.status == CMD_PARSE_ERROR {
        if !cause.is_null() {
            *cause = pr.error;
        }
        return -(1 as ::core::ffi::c_int);
    }
    let Some(bd) = kt.key_bindings.get_mut(item.key) else {
        drop(pr.cmdlist.take());
        return -1;
    };
    bd.commands = pr.cmdlist.take().expect("successful command parse");
    return 0 as ::core::ffi::c_int;
}
unsafe fn window_customize_set_note_value(
    item: &window_customize_itemdata,
    mut s: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let Some(table) = window_customize_get_key_table(item) else {
        return -(1 as ::core::ffi::c_int);
    };
    let mut kt = table.borrow_mut();
    let bd = kt.key_bindings.get_mut(item.key).expect("live binding");
    if *s as ::core::ffi::c_int == '\0' as i32 {
        key_bindings_set_note(bd, None);
    } else {
        key_bindings_set_note(bd, Some(CStr::from_ptr(s)));
    }
    return 0 as ::core::ffi::c_int;
}
unsafe fn window_customize_set_environment_value(
    item: &window_customize_itemdata,
    mut s: *const ::core::ffi::c_char,
) {
    let Some(mut environment) = item
        .environ
        .as_ref()
        .and_then(CustomizeEnvironment::resolve)
    else {
        return;
    };
    let Some(env) = environment.get_mut() else {
        return;
    };

    let mut envent: Option<&environ_entry> = None;
    let mut flags: ::core::ffi::c_int = 0;
    flags = item.environ_flags;
    envent = environ_find(
        env,
        (item.name)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
    );
    if !envent.is_none() {
        flags = envent.unwrap().flags;
    }
    environ_set(
        env,
        (item.name)
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
    let mut oe: *const options_table_entry = options_table_entry(&*(o)).map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry);
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    let mut defaults: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut ov: *mut options_value = ::core::ptr::null_mut::<options_value>();
    let mut default_ov: *mut options_value = ::core::ptr::null_mut::<options_value>();
    let mut changed: ::core::ffi::c_int = 0;
    if oe.is_null() || options_get_monitor_data(&mut *o).is_some() {
        return 1 as ::core::ffi::c_int;
    }
    if *options_name(&*(o)).as_ptr() as ::core::ffi::c_int == '@' as i32 && hooks_is_event(options_name(&*(o)).as_ptr()) != 0
    {
        return 1 as ::core::ffi::c_int;
    }
    if (*oe).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
        oo = options_create(::core::ptr::null_mut::<options>());
        defaults = options_default(oo, oe);
        if !array_key.is_null() {
            ov = crate::src::options::options_array_get_mut(&mut *(o), std::ffi::CStr::from_ptr(array_key)).map_or(std::ptr::null_mut(), |value| value);
            default_ov = crate::src::options::options_array_get_mut(&mut *(defaults), std::ffi::CStr::from_ptr(array_key)).map_or(std::ptr::null_mut(), |value| value);
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
unsafe fn window_customize_key_is_changed(kt: &key_table, bd: &key_binding) -> ::core::ffi::c_int {
    let Some(default_bd) = key_bindings_get_default(kt, bd.key) else {
        return 1;
    };
    if bd.flags != default_bd.flags || bd.note != default_bd.note {
        return 1;
    }
    let cmd = cmd_list_print_cstring(&bd.cmdlist().borrow(), 0);
    let default_cmd = cmd_list_print_cstring(&default_bd.cmdlist().borrow(), 0);
    (cmd.as_bytes() != default_cmd.as_bytes()) as ::core::ffi::c_int
}
unsafe fn window_customize_build_array(
    mut data: *mut window_customize_modedata,
    top: &ModeTreeItemRef,
    mut scope: window_customize_scope,
    mut o: *mut options_entry,
    mut ft: *mut format_tree,
) -> u_int {
    let mut oe: *const options_table_entry = options_table_entry(&*(o)).map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry);
    let mut oo: *mut options = options_owner(o);
    let mut ai: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut array_key: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut count: u_int = 0 as u_int;
    let ai_root = o;
    let mut ai_keys = crate::src::options::options_array_iter(&*ai_root).map(|item| item.key.clone()).collect::<Vec<_>>().into_iter();
    ai = ai_keys.next().map_or(std::ptr::null_mut(), |key| crate::src::options::options_array_item(ai_root, key.as_ptr()));
    while !ai.is_null() {
        array_key = options_array_item_key(&*(ai)).as_ptr();
        if (*data).hide_default != 0 && window_customize_option_is_changed(o, array_key) == 0 {
            ai = ai_keys.next().map_or(std::ptr::null_mut(), |key| crate::src::options::options_array_item(ai_root, key.as_ptr()));
        } else {
            let mut name = CStr::from_ptr(options_name(&*(o)).as_ptr()).to_bytes().to_vec();
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
            let item_owner = window_customize_add_item(
                &mut (*data).item_list,
                window_customize_itemdata {
                    type_0: WINDOW_CUSTOMIZE_ITEM_OPTION,
                    option_type: if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_HOOK != 0 {
                        WINDOW_CUSTOMIZE_HOOKS
                    } else {
                        WINDOW_CUSTOMIZE_OPTIONS
                    },
                    scope,
                    oo,
                    name: Some(CStr::from_ptr(options_name(&*(o)).as_ptr()).to_owned()),
                    array_key: if array_key.is_null() {
                        None
                    } else {
                        Some(CStr::from_ptr(array_key).to_owned())
                    },
                    ..window_customize_itemdata::new()
                },
            );
            let text = format_expand_cstring(ft, (*data).format.as_ptr());
            mode_tree_add(
                (*data).data_ptr(),
                Some(top),
                ModeTreeItemData::Customize(item_owner.clone()),
                window_customize_get_tag(o, ai, oe),
                &name,
                Some(&text),
                -(1 as ::core::ffi::c_int),
            );
            drop(value);
            count = count.wrapping_add(1);
            ai = ai_keys.next().map_or(std::ptr::null_mut(), |key| crate::src::options::options_array_item(ai_root, key.as_ptr()));
        }
    }
    return count;
}
unsafe fn window_customize_build_option(
    mut data: *mut window_customize_modedata,
    top: &ModeTreeItemRef,
    mut scope: window_customize_scope,
    mut o: *mut options_entry,
    mut ft: *mut format_tree,
    mut filter: *const ::core::ffi::c_char,
    mut fs: *mut cmd_find_state,
    mut type_0: window_customize_option_type,
) -> u_int {
    let mut oe: *const options_table_entry = options_table_entry(&*(o)).map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry);
    let mut oo: *mut options = options_owner(o);
    let mut name: *const ::core::ffi::c_char = options_name(&*(o)).as_ptr();
    let mut global: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut array: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut is_hook: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut is_monitor: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut is_user_hook: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut is_any_hook: ::core::ffi::c_int = 0;
    if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_HOOK != 0 {
        is_hook = 1 as ::core::ffi::c_int;
    }
    if options_get_monitor_data(&mut *o).is_some() {
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
    if !oe.is_null() && !(*oe).unit_ptr().is_null() {
        format_add(
            ft,
            b"option_unit\0" as *const u8 as *const ::core::ffi::c_char,
            |out| write_cstr(out, (*oe).unit_ptr()),
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
    let item_owner = window_customize_add_item(
        &mut (*data).item_list,
        window_customize_itemdata {
            type_0: WINDOW_CUSTOMIZE_ITEM_OPTION,
            option_type: type_0,
            scope,
            oo,
            name: Some(CStr::from_ptr(name).to_owned()),
            ..window_customize_itemdata::new()
        },
    );
    let text = (array == 0).then(|| format_expand_cstring(ft, (*data).format.as_ptr()));
    let top = mode_tree_add(
        (*data).data_ptr(),
        Some(top),
        ModeTreeItemData::Customize(item_owner.clone()),
        window_customize_get_tag(o, ::core::ptr::null_mut(), oe),
        CStr::from_ptr(name),
        text.as_deref(),
        0 as ::core::ffi::c_int,
    );
    if array == 0 {
        return 1 as u_int;
    }
    return (1 as u_int).wrapping_add(window_customize_build_array(data, &top, scope, o, ft));
}
unsafe fn window_customize_find_user_options(oo: *mut options, list: &mut Vec<CString>) {
    let o_root = oo;
    let mut o_names = crate::src::options::options_iter(&*o_root).map(|entry| entry.name.clone()).collect::<Vec<_>>().into_iter();
    let mut o = o_names.next().and_then(|name| crate::src::options::options_get_only_mut(&mut *o_root, &name)).map_or(std::ptr::null_mut(), |entry| entry);
    while !o.is_null() {
        let name = CStr::from_ptr(options_name(&*(o)).as_ptr());
        if name.to_bytes().first() == Some(&b'@') && !list.iter().any(|entry| entry == name) {
            // Later row builders can call format callbacks before the list is exhausted.
            list.push(name.to_owned());
        }
        o = o_names.next().and_then(|name| crate::src::options::options_get_only_mut(&mut *o_root, &name)).map_or(std::ptr::null_mut(), |entry| entry);
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
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut loop_0: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut list = Vec::<CString>::new();
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut count: u_int = 0 as u_int;
    let mut scope: window_customize_scope = WINDOW_CUSTOMIZE_NONE;
    let top = mode_tree_add(
        (*data).data_ptr(),
        None,
        ModeTreeItemData::None,
        window_customize_top_tag(group),
        CStr::from_ptr(title),
        None,
        0 as ::core::ffi::c_int,
    );
    mode_tree_no_tag(&top);
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
            data, &top, scope, o, ft, filter, fs, type_0,
        ));
    }
    drop(list);
    let loop_0_root = oo0;
    let mut loop_0_names = crate::src::options::options_iter(&*loop_0_root).map(|entry| entry.name.clone()).collect::<Vec<_>>().into_iter();
    loop_0 = loop_0_names.next().and_then(|name| crate::src::options::options_get_only_mut(&mut *loop_0_root, &name)).map_or(std::ptr::null_mut(), |entry| entry);
    while !loop_0.is_null() {
        name = options_name(&*(loop_0)).as_ptr();
        if *name as ::core::ffi::c_int == '@' as i32 {
            loop_0 = loop_0_names.next().and_then(|name| crate::src::options::options_get_only_mut(&mut *loop_0_root, &name)).map_or(std::ptr::null_mut(), |entry| entry);
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
                data, &top, scope, o, ft, filter, fs, type_0,
            ));
            loop_0 = loop_0_names.next().and_then(|name| crate::src::options::options_get_only_mut(&mut *loop_0_root, &name)).map_or(std::ptr::null_mut(), |entry| entry);
        }
    }
    if (*data).hide_default != 0 && count == 0 as u_int {
        mode_tree_remove((*data).data_ptr(), &top);
    }
}
fn window_customize_key_detail(value: &[u8]) -> CString {
    let mut text = b"#[fg=themelightgrey]#[ignore]".to_vec();
    text.extend_from_slice(value);
    CString::new(text).expect("key detail contains no NUL")
}

unsafe fn window_customize_build_keys(
    mut data: *mut window_customize_modedata,
    kt: &key_table,
    mut ft: *mut format_tree,
    mut filter: *const ::core::ffi::c_char,
    mut fs: *mut cmd_find_state,
) {
    let mut count: u_int = 0 as u_int;
    let mut title_bytes = b"Key Table - ".to_vec();
    title_bytes.extend_from_slice(kt.name.as_bytes());
    let title = CString::new(title_bytes).expect("key table name contains no NUL");
    let top = mode_tree_add(
        (*data).data_ptr(),
        None,
        ModeTreeItemData::None,
        (1_u64 << 62) | kt.identity,
        &title,
        None,
        0 as ::core::ffi::c_int,
    );
    mode_tree_no_tag(&top);
    drop(title);
    ft = format_create_from_state(
        ::core::ptr::null_mut::<cmdq_item>(),
        ::core::ptr::null_mut::<client>(),
        &*fs,
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
    for bd in kt.key_bindings.iter() {
        if (*data).hide_default != 0 && window_customize_key_is_changed(&*kt, bd) == 0 {
            continue;
        } else {
            let key_string = key_string_format(bd.key, false);
            format_add(
                ft,
                b"key\0" as *const u8 as *const ::core::ffi::c_char,
                |out| write_cstr(out, key_string.as_ptr()),
            );
            if !bd.note.is_none() {
                format_add(
                    ft,
                    b"key_note\0" as *const u8 as *const ::core::ffi::c_char,
                    |out| {
                        write_cstr(
                            out,
                            (bd.note)
                                .as_ref()
                                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                        )
                    },
                );
            }
            if !filter.is_null() {
                let expanded = format_expand_cstring(ft, filter);
                if format_true(expanded.as_ptr()) == 0 {
                    continue;
                }
            }
            let item_owner = window_customize_add_item(
                &mut (*data).item_list,
                window_customize_itemdata {
                    type_0: WINDOW_CUSTOMIZE_ITEM_KEY,
                    scope: WINDOW_CUSTOMIZE_KEY,
                    table: Some(kt.name.clone()),
                    key: bd.key,
                    name: Some(key_string_format(bd.key, false)),
                    ..window_customize_itemdata::new()
                },
            );
            let expanded = format_expand_cstring(ft, (*data).format.as_ptr());
            let child = mode_tree_add(
                (*data).data_ptr(),
                Some(&top),
                ModeTreeItemData::Customize(item_owner.clone()),
                window_customize_key_tag(std::ptr::from_ref(bd).cast(), 0),
                &expanded,
                None,
                0 as ::core::ffi::c_int,
            );
            let tmp = cmd_list_print_cstring(&bd.cmdlist().borrow(), 0);
            let text = window_customize_key_detail(tmp.as_bytes());
            let mti = mode_tree_add(
                (*data).data_ptr(),
                Some(&child),
                ModeTreeItemData::Customize(item_owner.clone()),
                window_customize_key_tag(std::ptr::from_ref(bd).cast(), 1),
                c"Command",
                Some(&text),
                -(1 as ::core::ffi::c_int),
            );
            mode_tree_draw_as_parent(&mti);
            mode_tree_no_tag(&mti);
            drop(text);
            let text = if !bd.note.is_none() {
                window_customize_key_detail(
                    CStr::from_ptr(
                        (bd.note)
                            .as_ref()
                            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                    )
                    .to_bytes(),
                )
            } else {
                CString::new(Vec::new()).expect("empty key note")
            };
            let mti = mode_tree_add(
                (*data).data_ptr(),
                Some(&child),
                ModeTreeItemData::Customize(item_owner.clone()),
                window_customize_key_tag(std::ptr::from_ref(bd).cast(), 2),
                c"Note",
                Some(&text),
                -(1 as ::core::ffi::c_int),
            );
            mode_tree_draw_as_parent(&mti);
            mode_tree_no_tag(&mti);
            drop(text);
            let flag = if bd.flags & KEY_BINDING_REPEAT != 0 {
                b"on".as_slice()
            } else {
                b"off".as_slice()
            };
            let text = window_customize_key_detail(flag);
            let mti = mode_tree_add(
                (*data).data_ptr(),
                Some(&child),
                ModeTreeItemData::Customize(item_owner.clone()),
                window_customize_key_tag(std::ptr::from_ref(bd).cast(), 3),
                c"Repeat",
                Some(&text),
                -(1 as ::core::ffi::c_int),
            );
            mode_tree_draw_as_parent(&mti);
            mode_tree_no_tag(&mti);
            drop(text);
            count = count.wrapping_add(1);
        }
    }
    format_free(ft);
    if (*data).hide_default != 0 && count == 0 as u_int {
        mode_tree_remove((*data).data_ptr(), &top);
    }
}
unsafe fn window_customize_build_environment(
    mut data: *mut window_customize_modedata,
    mut title: *const ::core::ffi::c_char,
    group: u_int,
    mut scope: window_customize_scope,
    target: CustomizeEnvironment,
    mut ft: *mut format_tree,
    mut filter: *const ::core::ffi::c_char,
    mut fs: *mut cmd_find_state,
) {
    let Some(environment) = target.resolve() else {
        return;
    };
    let Some(env) = environment.get() else {
        return;
    };

    let mut global: ::core::ffi::c_int = 0;
    if (*data).hide_default != 0 {
        return;
    }
    let top = mode_tree_add(
        (*data).data_ptr(),
        None,
        ModeTreeItemData::None,
        window_customize_top_tag(group),
        CStr::from_ptr(title),
        None,
        0 as ::core::ffi::c_int,
    );
    mode_tree_no_tag(&top);
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
        let envent = entry;
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
        let item_owner = window_customize_add_item(
            &mut (*data).item_list,
            window_customize_itemdata {
                type_0: WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT,
                scope,
                environ: Some(target.clone()),
                environ_flags: (*envent).flags,
                name: Some((*envent).name.clone()),
                ..window_customize_itemdata::new()
            },
        );
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
            (*data).data_ptr(),
            Some(&top),
            ModeTreeItemData::Customize(item_owner.clone()),
            (2_u64 << 62) | std::ptr::from_ref(envent) as uint64_t,
            &name,
            text.as_deref(),
            0 as ::core::ffi::c_int,
        );
    }
}
unsafe fn window_customize_build(
    mut modedata: *mut ::core::ffi::c_void,
    mut filter: *const ::core::ffi::c_char,
) {
    let mut data: *mut window_customize_modedata = modedata as *mut window_customize_modedata;
    let Some(mode_pane_owner) = window_pane_upgrade(&(*data).wp) else {
        return;
    };
    let mode_pane = crate::src::shared::rc::as_ptr(&mode_pane_owner);
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: Default::default(),
        wl: Default::default(),
        w: Default::default(),
        wp: Default::default(),
        idx: 0,
    };
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    (*data).item_list.clear();
    if cmd_find_valid_state(&(*data).fs) != 0 {
        cmd_find_copy_state(&raw mut fs, &raw mut (*data).fs);
    } else {
        cmd_find_from_pane(&raw mut fs, mode_pane, 0 as ::core::ffi::c_int);
    }
    ft = format_create_from_state(
        ::core::ptr::null_mut::<cmdq_item>(),
        ::core::ptr::null_mut::<client>(),
        &fs,
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
        options_owner_ptr(&mut (*fs.s_ptr()).options).map_or(std::ptr::null_mut(), |options| options),
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
        options_owner_ptr(&mut (*fs.w_ptr()).options).map_or(std::ptr::null_mut(), |options| options),
        WINDOW_CUSTOMIZE_PANE,
        options_owner_ptr(&mut (*fs.wp_ptr()).options).map_or(std::ptr::null_mut(), |options| options),
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
        options_owner_ptr(&mut (*fs.s_ptr()).options).map_or(std::ptr::null_mut(), |options| options),
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
        options_owner_ptr(&mut (*fs.w_ptr()).options).map_or(std::ptr::null_mut(), |options| options),
        WINDOW_CUSTOMIZE_PANE,
        options_owner_ptr(&mut (*fs.wp_ptr()).options).map_or(std::ptr::null_mut(), |options| options),
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
        CustomizeEnvironment::Global,
        ft,
        filter,
        &raw mut fs,
    );
    window_customize_build_environment(
        data,
        b"Session Environment\0" as *const u8 as *const ::core::ffi::c_char,
        CUSTOMIZE_SESSION_ENVIRONMENT,
        WINDOW_CUSTOMIZE_SESSION_ENVIRONMENT,
        CustomizeEnvironment::session(fs.s_ptr()),
        ft,
        filter,
        &raw mut fs,
    );
    format_free(ft);
    ft = format_create_from_state(
        ::core::ptr::null_mut::<cmdq_item>(),
        ::core::ptr::null_mut::<client>(),
        &fs,
    );
    format_add(
        ft,
        b"is_environment\0" as *const u8 as *const ::core::ffi::c_char,
        |out| out.write_all(b"0"),
    );
    for table_owner in key_bindings_tables() {
        let kt = table_owner.borrow();
        if kt.key_bindings.storage.is_some() {
            window_customize_build_keys(data, &kt, ft, filter, &raw mut fs);
        }
    }
    format_free(ft);
}
unsafe fn window_customize_draw_key(
    item: &window_customize_itemdata,
    mut ctx: *mut screen_write_ctx,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut cx: u_int = (*s).cx;
    let mut cy: u_int = (*s).cy;
    let mut note: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut period: *const ::core::ffi::c_char = b"\0" as *const u8 as *const ::core::ffi::c_char;
    let Some(table) = window_customize_get_key_table(item) else {
        return;
    };
    let kt = table.borrow();
    let bd = kt.key_bindings.get(item.key).expect("live binding");
    note = (bd.note)
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
        &mut *ctx,
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
            write_cstr(out, (kt.name).as_ptr().cast_mut())?;
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
                if bd.flags & KEY_BINDING_REPEAT != 0 {
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
        &mut *ctx,
        cx as ::core::ffi::c_int,
        (*s).cy.wrapping_add(1 as u_int) as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    if (*s).cy >= cy.wrapping_add(sy).wrapping_sub(1 as u_int) {
        return;
    }
    let cmd = cmd_list_print_cstring(&bd.cmdlist().borrow(), 0);
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
    if let Some(default_bd) = key_bindings_get_default(&*kt, bd.key) {
        let default_cmd = cmd_list_print_cstring(&default_bd.cmdlist().borrow(), 0);
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
    item: &window_customize_itemdata,
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
        s: Default::default(),
        wl: Default::default(),
        w: Default::default(),
        wp: Default::default(),
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
    name = (item.name)
        .as_ref()
        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut());
    array_key = (item.array_key)
        .as_ref()
        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut());
    o = options_get(item.oo, name);
    if o.is_null() {
        return;
    }
    oe = options_table_entry(&*(o)).map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry);
    is_hook = (!oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_HOOK != 0) as ::core::ffi::c_int;
    is_monitor = options_get_monitor_data(&mut *o).is_some() as ::core::ffi::c_int;
    is_user_hook = (*name as ::core::ffi::c_int == '@' as i32 && hooks_is_event(name) != 0)
        as ::core::ffi::c_int;
    is_any_hook = (is_hook != 0 || is_monitor != 0 || is_user_hook != 0) as ::core::ffi::c_int;
    if !oe.is_null() && !(*oe).unit_ptr().is_null() {
        space = b" \0" as *const u8 as *const ::core::ffi::c_char;
        unit = (*oe).unit_ptr();
    }
    ft = format_create_from_state(
        ::core::ptr::null_mut::<cmdq_item>(),
        ::core::ptr::null_mut::<client>(),
        &fs,
    );
    if oe.is_null() || (*oe).text_ptr().is_null() {
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
        text = (*oe).text_ptr();
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
            &mut *ctx,
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
                                        &mut *ctx,
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
                                                            for choice in (*oe).choices {
                                                                strlcat(
                                                                    &raw mut choices
                                                                        as *mut ::core::ffi::c_char,
                                                                    choice.as_ptr(),
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
                                                                    if screen_write_text(
                                                                        &mut *ctx,
                                                                        cx,
                                                                        sx,
                                                                        sy.wrapping_sub(
                                                                            (*s).cy
                                                                                .wrapping_sub(cy),
                                                                        ),
                                                                        1 as ::core::ffi::c_int,
                                                                        &grid_default_cell,
                                                                        |out| {
                                                                            out.write_all(b"This is a colour option: ")
                                                                        },
                                                                    ) == 0
                                                                    {
                                                                        current_block =
                                                                            4086289836260337793;
                                                                    } else {
                                                                        memcpy(
                                                                            &raw mut gc as *mut ::core::ffi::c_void,
                                                                            &raw const grid_default_cell as *const ::core::ffi::c_void,
                                                                            ::core::mem::size_of::<grid_cell>() as size_t,
                                                                        );
                                                                        gc.fg = options_get_number(
                                                                            item.oo, name,
                                                                        )
                                                                            as ::core::ffi::c_int;
                                                                        if screen_write_text(
                                                                            &mut *ctx,
                                                                            cx,
                                                                            sx,
                                                                            sy.wrapping_sub(
                                                                                (*s).cy
                                                                                    .wrapping_sub(
                                                                                        cy,
                                                                                    ),
                                                                            ),
                                                                            0 as ::core::ffi::c_int,
                                                                            &gc,
                                                                            |out| {
                                                                                out.write_all(
                                                                                    b"EXAMPLE",
                                                                                )
                                                                            },
                                                                        ) == 0
                                                                        {
                                                                            current_block =
                                                                                4086289836260337793;
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
                                                                                style_apply(&raw mut gc, item.oo, name, ft);
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
                                                                                        style_apply(&raw mut gc, item.oo, name, ft);
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
                                                                                                    &mut *ctx,
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
                                                                                                        match item.scope as ::core::ffi::c_uint {
                                                                                                            7 => {
                                                                                                                wo = options_get_parent(item.oo);
                                                                                                                go = options_get_parent(wo);
                                                                                                            }
                                                                                                            6 | 4 => {
                                                                                                                wo = ::core::ptr::null_mut::<options>();
                                                                                                                go = options_get_parent(item.oo);
                                                                                                            }
                                                                                                            _ => {
                                                                                                                wo = ::core::ptr::null_mut::<options>();
                                                                                                                go = ::core::ptr::null_mut::<options>();
                                                                                                            }
                                                                                                        }
                                                                                                    }
                                                                                                    if !wo.is_null() && options_owner(o) != wo {
                                                                                                        parent = crate::src::options::options_get_only_mut(&mut *(wo), std::ffi::CStr::from_ptr(name)).map_or(std::ptr::null_mut(), |entry| entry);
                                                                                                        if !parent.is_null() {
                                                                                                            value_owner = Some(options_to_string(
                                                                                                                parent,
                                                                                                                ::core::ptr::null::<::core::ffi::c_char>(),
                                                                                                            ));
                                                                                                            value = value_owner.as_ref().unwrap().as_ptr().cast_mut();
                                                                                                            xformat(&mut label, format_args!("Window value (from window {}): " , ((*fs.wl_ptr()).idx) as u32));
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
                                                                                                                parent = crate::src::options::options_get_only_mut(&mut *(go), std::ffi::CStr::from_ptr(name)).map_or(std::ptr::null_mut(), |entry| entry);
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
    item: &window_customize_itemdata,
    mut ctx: *mut screen_write_ctx,
    mut sx: u_int,
    mut sy: u_int,
) {
    let Some(mut environment) = item
        .environ
        .as_ref()
        .and_then(CustomizeEnvironment::resolve)
    else {
        return;
    };
    let Some(env) = environment.get() else {
        return;
    };

    let mut s: *mut screen = (*ctx).s;
    let mut cx: u_int = (*s).cx;
    let mut cy: u_int = (*s).cy;
    let mut envent: Option<&environ_entry> = None;
    let mut parent: Option<&environ_entry> = None;
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: Default::default(),
        wl: Default::default(),
        w: Default::default(),
        wp: Default::default(),
        idx: 0,
    };
    let mut text: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if window_customize_check_item(data, item, &raw mut fs) == 0 {
        return;
    }
    envent = environ_find(
        env,
        (item.name)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
    );
    if envent.is_none() {
        return;
    }
    if item.scope as ::core::ffi::c_uint
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
    if envent.unwrap().flags & ENVIRON_HIDDEN != 0 {
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
        &mut *ctx,
        cx as ::core::ffi::c_int,
        (*s).cy.wrapping_add(1 as u_int) as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    if (*s).cy >= cy.wrapping_add(sy).wrapping_sub(1 as u_int) {
        return;
    }
    if envent.unwrap().value.is_none() {
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
                (envent.unwrap().value)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            )
        },
    ) == 0
    {
        return;
    }
    if item.scope as ::core::ffi::c_uint
        != WINDOW_CUSTOMIZE_SESSION_ENVIRONMENT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    parent = environ_find(
        global_environ.as_deref().expect("environment"),
        (item.name)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
    );
    if parent.is_none() {
        return;
    }
    if parent.unwrap().value.is_none() {
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
                (parent.unwrap().value)
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
    item: &window_customize_itemdata,
    ctx: &mut screen_write_ctx,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut data: *mut window_customize_modedata = modedata as *mut window_customize_modedata;
    if item.type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_ITEM_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        window_customize_draw_key(item, ctx, sx, sy);
    } else if item.type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        window_customize_draw_environment(data, item, ctx, sx, sy);
    } else {
        window_customize_draw_option(data, item, ctx, sx, sy);
    };
}
unsafe fn window_customize_menu(
    mut modedata: *mut ::core::ffi::c_void,
    c: &std::rc::Rc<std::cell::UnsafeCell<client>>,
    mut key: key_code,
) {
    let mut data: *mut window_customize_modedata = modedata as *mut window_customize_modedata;
    let Some(mode_pane_owner) = window_pane_upgrade(&(*data).wp) else {
        return;
    };
    let mode_pane = crate::src::shared::rc::as_ptr(&mode_pane_owner);
    let mut wp: *mut window_pane = mode_pane;
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    wme = (*wp).modes.active;
    if wme.is_null() || (*wme).data != modedata {
        return;
    }
    window_customize_key(
        wme,
        c,
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
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut wp: *mut window_pane = mode_pane;
    let mut data: *mut window_customize_modedata =
        ::core::ptr::null_mut::<window_customize_modedata>();
    let mut s: *mut screen = ::core::ptr::null_mut::<screen>();
    let format = if args.is_null() || args_has(args, 'F' as i32 as u_char) == 0 {
        CStr::from_ptr(WINDOW_CUSTOMIZE_DEFAULT_FORMAT.as_ptr()).to_owned()
    } else {
        CStr::from_ptr(args_get(&*(args), 'F' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr())).to_owned()
    };
    let owner = Rc::new_cyclic(|observer| std::cell::UnsafeCell::new(window_customize_modedata {
        observer: observer.clone(),
        wp: std::rc::Rc::downgrade(&mode_pane_owner),
        dead: 0,
        data: None,
        editor: ::core::ptr::null_mut(),
        format,
        hide_global: 0,
        hide_default: 0,
        prompt_flags: 0,
        item_list: Vec::new(),
        fs: (*fs).clone(),
        change: WINDOW_CUSTOMIZE_UNSET,
    }));
    data = crate::src::shared::rc::as_ptr(&owner);
    (*wme).data_owner = Some(owner);
    (*wme).data = data as *mut ::core::ffi::c_void;
    let data_handle = std::ptr::NonNull::new(data).expect("live customize mode data");
    if args_has(args, 'y' as i32 as u_char) != 0 {
        (*data).prompt_flags = PROMPT_ACCEPT;
    }
    (*data).data = Some(mode_tree_start(
        &mode_pane_owner,
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
            let Some(item_owner) = itemdata.as_customize() else {
                return;
            };
            window_customize_draw(data_handle.as_ptr().cast(), &item_owner, ctx, sx, sy)
        })),
        None,
        Some(Box::new(move |client, key| {
            window_customize_menu(data_handle.as_ptr().cast(), client, key)
        })),
        Some(Box::new(move |_| window_customize_height())),
        None,
        None,
        None,
        Some(window_customize_help),
        &window_customize_menu_items,
        &raw mut s,
    ));
    mode_tree_zoom((*data).data.clone().as_ref().expect("mode tree owner"), args);
    mode_tree_build((*data).data.clone().as_ref().expect("mode tree owner"));
    mode_tree_draw((*data).data.clone().as_ref().expect("mode tree owner"));
    return s;
}
unsafe fn window_customize_free(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_customize_modedata = (*wme).data as *mut window_customize_modedata;
    if data.is_null() {
        return;
    }
    (*data).dead = 1 as ::core::ffi::c_int;
    if !(*data).editor.is_null() {
        spawn_cancel_editor((*data).editor);
    }
    mode_tree_free((*data).data.take().expect("mode tree owner"));
    drop((*wme).data_owner.take());
    (*wme).data = std::ptr::null_mut();
}
unsafe fn window_customize_resize(mut wme: *mut window_mode_entry, mut sx: u_int, mut sy: u_int) {
    let mut data: *mut window_customize_modedata = (*wme).data as *mut window_customize_modedata;
    mode_tree_resize((*data).data.clone().as_ref().expect("mode tree owner"), sx, sy);
}
unsafe fn window_customize_update(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_customize_modedata = (*wme).data as *mut window_customize_modedata;
    window_customize_draw_waiting(data);
}

// Fields drop in declaration order: the detached item before the retained mode.
struct CustomizePromptItem {
    item: Box<window_customize_itemdata>,
    mode: Rc<UnsafeCell<window_customize_modedata>>,
}

fn window_customize_prompt_callbacks<T: 'static>(
    owner: RefBox<T>,
    callback: unsafe fn(
        Option<NonNull<client>>,
        &T,
        Option<&CStr>,
        prompt_key_result,
    ) -> prompt_result,
) -> (mode_tree_prompt_input_cb, prompt_free_cb) {
    let weak = RefBox::downgrade(&owner);
    let inputcb: mode_tree_prompt_input_cb = Some(Box::new(move |client, text, key| {
        let Ok(owner) = weak.try_borrow_mut() else {
            return PROMPT_CLOSE;
        };
        // Keep the callback's borrowed input alive if it closes its own prompt.
        unsafe { callback(client, &owner, text, key) }
    }));
    // mode_tree invokes cleanup before releasing its own retained tree. Cached
    // input callbacks only keep a weak reference and cannot extend this lifetime.
    let freecb: prompt_free_cb = Some(Box::new(move || drop(owner)));
    (inputcb, freecb)
}

fn window_customize_mode_prompt_callbacks(
    owner: Rc<UnsafeCell<window_customize_modedata>>,
    callback: unsafe fn(
        Option<NonNull<client>>,
        &UnsafeCell<window_customize_modedata>,
        Option<&CStr>,
        prompt_key_result,
    ) -> prompt_result,
) -> (mode_tree_prompt_input_cb, prompt_free_cb) {
    let weak = Rc::downgrade(&owner);
    let inputcb: mode_tree_prompt_input_cb = Some(Box::new(move |client, text, key| {
        let Some(owner) = weak.upgrade() else {
            return PROMPT_CLOSE;
        };
        // Keep the callback's borrowed input alive if it closes its own prompt.
        unsafe { callback(client, &owner, text, key) }
    }));
    // mode_tree invokes cleanup before releasing its own retained tree. Cached
    // input callbacks only keep a weak reference and cannot extend this lifetime.
    let freecb: prompt_free_cb = Some(Box::new(move || drop(owner)));
    (inputcb, freecb)
}

unsafe fn window_customize_set_option_callback(
    c: Option<NonNull<client>>,
    owner: &CustomizePromptItem,
    s: Option<&CStr>,
    _key: prompt_key_result,
) -> prompt_result {
    let c = c.map_or(std::ptr::null_mut(), NonNull::as_ptr);
    let mut s = s.map_or(::core::ptr::null(), CStr::as_ptr);
    let mut current_block: u64;
    let item = &*owner.item;
    let data = crate::src::shared::rc::as_ptr(&owner.mode);
    let Some(mode_pane_owner) = window_pane_upgrade(&(*data).wp) else {
        return PROMPT_CLOSE;
    };
    let mode_pane = crate::src::shared::rc::as_ptr(&mode_pane_owner);
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    let mut oo: *mut options = item.oo;
    let mut name: *const ::core::ffi::c_char = (item.name)
        .as_ref()
        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut());
    let mut array_key: *const ::core::ffi::c_char = (item.array_key)
        .as_ref()
        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut());
    let mut cause: Option<CString> = None;
    let mut idx: u_int = 0;
    let mut keybuf: [::core::ffi::c_char; 32] = [0; 32];
    if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 || (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    if window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0 {
        return PROMPT_CLOSE;
    }
    o = options_get(oo, name);
    if o.is_null() {
        return PROMPT_CLOSE;
    }
    oe = options_table_entry(&*(o)).map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry);
    if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
        if array_key.is_null() {
            idx = 0 as u_int;
            while idx < INT_MAX as u_int {
                if crate::src::options::options_array_get_index_mut(&mut *(o), idx).map_or(std::ptr::null_mut(), |value| value).is_null() {
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
            if item.option_type as ::core::ffi::c_uint
                == WINDOW_CUSTOMIZE_HOOKS as ::core::ffi::c_int as ::core::ffi::c_uint
                && *name as ::core::ffi::c_int == '@' as i32
            {
                hooks_add_event(name);
            }
            options_push_changes(
                (item.name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            );
            mode_tree_build((*data).data.clone().as_ref().expect("mode tree owner"));
            mode_tree_draw((*data).data.clone().as_ref().expect("mode tree owner"));
            (*mode_pane).flags |= PANE_REDRAW;
            return PROMPT_CLOSE;
        }
    };
}
unsafe fn window_customize_set_environment_callback(
    _c: Option<NonNull<client>>,
    owner: &CustomizePromptItem,
    s: Option<&CStr>,
    _key: prompt_key_result,
) -> prompt_result {
    let mut s = s.map_or(::core::ptr::null(), CStr::as_ptr);
    let item = &*owner.item;
    let Some(mut environment) = item
        .environ
        .as_ref()
        .and_then(CustomizeEnvironment::resolve)
    else {
        return PROMPT_CLOSE;
    };
    let Some(env) = environment.get_mut() else {
        return PROMPT_CLOSE;
    };
    let data = crate::src::shared::rc::as_ptr(&owner.mode);
    let Some(mode_pane_owner) = window_pane_upgrade(&(*data).wp) else {
        return PROMPT_CLOSE;
    };
    let mode_pane = crate::src::shared::rc::as_ptr(&mode_pane_owner);
    let mut envent: Option<&environ_entry> = None;
    let mut flags: ::core::ffi::c_int = 0;
    if s.is_null() || (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    if window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0 {
        return PROMPT_CLOSE;
    }
    flags = item.environ_flags;
    envent = environ_find(
        env,
        (item.name)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
    );
    if !envent.is_none() {
        flags = envent.unwrap().flags;
    }
    environ_set(
        env,
        (item.name)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        flags,
        |out| write_cstr(out, s),
    );
    mode_tree_build((*data).data.clone().as_ref().expect("mode tree owner"));
    mode_tree_draw((*data).data.clone().as_ref().expect("mode tree owner"));
    (*mode_pane).flags |= PANE_REDRAW;
    return PROMPT_CLOSE;
}
unsafe fn window_customize_set_environment(
    mut c: *mut client,
    mut data: *mut window_customize_modedata,
    item: &window_customize_itemdata,
    mut global: ::core::ffi::c_int,
) {
    let Some(mut environment) = item
        .environ
        .as_ref()
        .and_then(CustomizeEnvironment::resolve)
    else {
        return;
    };
    let Some(env) = environment.get() else {
        return;
    };

    let mut envent: Option<&environ_entry> = None;
    let target;
    let mut scope: window_customize_scope = WINDOW_CUSTOMIZE_NONE;
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: Default::default(),
        wl: Default::default(),
        w: Default::default(),
        wp: Default::default(),
        idx: 0,
    };
    let mut space: *const ::core::ffi::c_char = b"\0" as *const u8 as *const ::core::ffi::c_char;
    if window_customize_check_item(data, item, &raw mut fs) == 0 {
        return;
    }
    envent = environ_find(
        env,
        (item.name)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
    );
    if envent.is_none() {
        return;
    }
    if global != 0 {
        scope = WINDOW_CUSTOMIZE_GLOBAL_ENVIRONMENT;
        target = CustomizeEnvironment::Global;
    } else {
        scope = item.scope;
        target = item.environ.clone().expect("environment target");
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
            (item.name)
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
    let mut new_item = window_customize_new_item();

    new_item.type_0 = WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT;
    new_item.scope = scope;
    new_item.environ = Some(target);
    new_item.environ_flags = envent.unwrap().flags;
    window_customize_set_name(&mut *new_item, item.name.as_deref());
    let value = envent.unwrap().value.clone().unwrap_or_default();
    let owner = RefBox::new(CustomizePromptItem {
        item: new_item,
        mode: (*data).observer
            .upgrade()
            .expect("live customize mode"),
    });
    let (inputcb, freecb) =
        window_customize_prompt_callbacks(owner, window_customize_set_environment_callback);
    mode_tree_set_prompt(
        (*data).data.as_ref().expect("mode tree owner").clone(),
        c,
        &prompt,
        Some(&value),
        PROMPT_TYPE_COMMAND,
        PROMPT_NOFORMAT,
        inputcb,
        freecb,
    );
}
unsafe fn window_customize_add_option_callback(
    c: Option<NonNull<client>>,
    owner: &CustomizePromptItem,
    s: Option<&CStr>,
    _key: prompt_key_result,
) -> prompt_result {
    let c = c.map_or(std::ptr::null_mut(), NonNull::as_ptr);
    let mut s = s.map_or(::core::ptr::null(), CStr::as_ptr);
    let item = &*owner.item;
    let data = crate::src::shared::rc::as_ptr(&owner.mode);
    let Some(mode_pane_owner) = window_pane_upgrade(&(*data).wp) else {
        return PROMPT_CLOSE;
    };
    let mode_pane = crate::src::shared::rc::as_ptr(&mode_pane_owner);
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut what: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut namelen: size_t = 0;
    if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 || (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    if window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0 {
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
        what = if item.option_type as ::core::ffi::c_uint
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
    options_set_string(item.oo, name, 0 as ::core::ffi::c_int, |out| {
        write_cstr(out, value)
    });
    if item.option_type as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_HOOKS as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        hooks_add_event(name);
    }
    options_push_changes(name);
    mode_tree_build((*data).data.clone().as_ref().expect("mode tree owner"));
    mode_tree_draw((*data).data.clone().as_ref().expect("mode tree owner"));
    (*mode_pane).flags |= PANE_REDRAW;
    return PROMPT_CLOSE;
}
unsafe fn window_customize_add_option(
    mut c: *mut client,
    mut data: *mut window_customize_modedata,
    mut scope: window_customize_scope,
    mut oo: *mut options,
    mut type_0: window_customize_option_type,
) {
    let prompt = if type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_HOOKS as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        c"New user hook: "
    } else {
        c"New user option: "
    };
    let mut new_item = window_customize_new_item();

    new_item.type_0 = WINDOW_CUSTOMIZE_ITEM_OPTION;
    new_item.option_type = type_0;
    new_item.scope = scope;
    new_item.oo = oo;
    let owner = RefBox::new(CustomizePromptItem {
        item: new_item,
        mode: (*data).observer
            .upgrade()
            .expect("live customize mode"),
    });
    let (inputcb, freecb) =
        window_customize_prompt_callbacks(owner, window_customize_add_option_callback);
    mode_tree_set_prompt(
        (*data).data.as_ref().expect("mode tree owner").clone(),
        c,
        prompt,
        Some(c"@"),
        PROMPT_TYPE_COMMAND,
        PROMPT_NOFORMAT,
        inputcb,
        freecb,
    );
}
unsafe fn window_customize_add_environment_callback(
    c: Option<NonNull<client>>,
    owner: &CustomizePromptItem,
    s: Option<&CStr>,
    _key: prompt_key_result,
) -> prompt_result {
    let c = c.map_or(std::ptr::null_mut(), NonNull::as_ptr);
    let mut s = s.map_or(::core::ptr::null(), CStr::as_ptr);
    let item = &*owner.item;
    let Some(mut environment) = item
        .environ
        .as_ref()
        .and_then(CustomizeEnvironment::resolve)
    else {
        return PROMPT_CLOSE;
    };
    let Some(env) = environment.get_mut() else {
        return PROMPT_CLOSE;
    };
    let data = crate::src::shared::rc::as_ptr(&owner.mode);
    let Some(mode_pane_owner) = window_pane_upgrade(&(*data).wp) else {
        return PROMPT_CLOSE;
    };
    let mode_pane = crate::src::shared::rc::as_ptr(&mode_pane_owner);
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 || (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    if window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0 {
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
        environ_clear(env, s.offset(1 as ::core::ffi::c_int as isize));
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
        environ_set(env, name.as_ptr(), 0 as ::core::ffi::c_int, |out| {
            write_cstr(out, value.offset(1 as ::core::ffi::c_int as isize))
        });
    }
    mode_tree_build((*data).data.clone().as_ref().expect("mode tree owner"));
    mode_tree_draw((*data).data.clone().as_ref().expect("mode tree owner"));
    (*mode_pane).flags |= PANE_REDRAW;
    return PROMPT_CLOSE;
}
unsafe fn window_customize_add_environment(
    mut c: *mut client,
    mut data: *mut window_customize_modedata,
    mut scope: window_customize_scope,
    target: CustomizeEnvironment,
) {
    let mut new_item = window_customize_new_item();

    new_item.type_0 = WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT;
    new_item.scope = scope;
    new_item.environ = Some(target);
    let owner = RefBox::new(CustomizePromptItem {
        item: new_item,
        mode: (*data).observer
            .upgrade()
            .expect("live customize mode"),
    });
    let (inputcb, freecb) =
        window_customize_prompt_callbacks(owner, window_customize_add_environment_callback);
    mode_tree_set_prompt(
        (*data).data.as_ref().expect("mode tree owner").clone(),
        c,
        c"New environment: ",
        Some(c""),
        PROMPT_TYPE_COMMAND,
        PROMPT_NOFORMAT,
        inputcb,
        freecb,
    );
}
unsafe fn window_customize_edit_close_cb(
    editor: NonNull<spawn_editor_state>,
    buf: Option<Vec<u8>>,
    ed: Box<window_customize_editdata>,
) {
    let mut current_block: u64;
    let item = &*ed.item;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut data: *mut window_customize_modedata =
        ::core::ptr::null_mut::<window_customize_modedata>();
    let mut cause: Option<CString> = None;
    let lookup_wp_owner = window_pane_find_by_id(ed.wp_id);
    wp = lookup_wp_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    if !wp.is_null() {
        wme = (*wp).modes.active;
        if !wme.is_null() && std::ptr::eq((*wme).mode, &window_customize_mode) {
            data = (*wme).data as *mut window_customize_modedata;
            if NonNull::new((*data).editor) == Some(editor) {
                (*data).editor = ::core::ptr::null_mut::<spawn_editor_state>();
            }
        }
    }
    let Some(mut value) = buf else {
        return;
    };
    if value.is_empty() || data.is_null() || (*data).dead != 0 {
        return;
    }
    if value.last() == Some(&b'\n') {
        value.pop();
    }
    value.push(0);
    let value_ptr = value.as_ptr().cast::<::core::ffi::c_char>();
    match ed.edit_type as ::core::ffi::c_uint {
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
            mode_tree_build((*data).data.clone().as_ref().expect("mode tree owner"));
            mode_tree_draw((*data).data.clone().as_ref().expect("mode tree owner"));
            (*wp).flags |= PANE_REDRAW;
        }
        _ => {}
    }
    drop(value);
}
unsafe fn window_customize_start_edit(
    mut data: *mut window_customize_modedata,
    item: &window_customize_itemdata,
    mut c: *mut client,
) {
    let Some(mode_pane_owner) = window_pane_upgrade(&(*data).wp) else {
        return;
    };
    let mode_pane = crate::src::shared::rc::as_ptr(&mode_pane_owner);
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut envent: Option<&environ_entry> = None;
    let value: Cow<'_, CStr>;
    let mut edit_type: window_customize_edit_type = WINDOW_CUSTOMIZE_EDIT_OPTION;
    if !(*data).editor.is_null() {
        return;
    }
    if item.type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_ITEM_OPTION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if window_customize_option_editable(data, item) == 0 {
            return;
        }
        o = options_get(
            item.oo,
            (item.name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        );
        if o.is_null() {
            return;
        }
        value = Cow::Owned(options_to_cstring(
            o,
            (item.array_key)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            0 as ::core::ffi::c_int,
        ));
        edit_type = WINDOW_CUSTOMIZE_EDIT_OPTION;
    } else if item.type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_ITEM_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let name = mode_tree_get_current_name(&*(*data).data_ptr());
        let Some(table) = window_customize_get_key_table(item) else {
            return;
        };
        let kt = table.borrow();
        let bd = kt.key_bindings.get(item.key).expect("live binding");
        if name.as_ref() == c"Command" {
            value = Cow::Owned(cmd_list_print_cstring(
                &bd.cmdlist().borrow(),
                0 as ::core::ffi::c_int,
            ));
            edit_type = WINDOW_CUSTOMIZE_EDIT_KEY_COMMAND;
        } else if name.as_ref() == c"Note" {
            value = Cow::Owned(bd.note.clone().unwrap_or_default());
            edit_type = WINDOW_CUSTOMIZE_EDIT_KEY_NOTE;
        } else {
            return;
        }
    } else if item.type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0 {
            return;
        }
        let Some(environment) = item
            .environ
            .as_ref()
            .and_then(CustomizeEnvironment::resolve)
        else {
            return;
        };
        let Some(env) = environment.get() else {
            return;
        };
        envent = environ_find(
            env,
            (item.name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        );
        if envent.is_none() || envent.unwrap().value.is_none() {
            return;
        }
        value = Cow::Owned(envent.unwrap().value.clone().expect("environment value"));
        edit_type = WINDOW_CUSTOMIZE_EDIT_ENVIRONMENT;
    } else {
        return;
    }
    // The callback owns the record and receives editor identity at dispatch.
    // Startup failure and cancellation drop the capture normally.
    let ed = Box::new(window_customize_editdata {
        wp_id: (*mode_pane).id,
        edit_type,
        item: window_customize_copy_item(item),
    });
    let bytes = value.to_bytes();
    let bytes = if bytes.is_empty() { b"\n" } else { bytes };
    let editor = spawn_editor(
        c,
        |stream| spawn_editor_write(stream, bytes),
        Some(Box::new(move |editor, buf| unsafe {
            window_customize_edit_close_cb(editor, buf, ed)
        })),
    );
    if let Some(editor) = NonNull::new(editor) {
        (*data).editor = editor.as_ptr();
    }
}
unsafe fn window_customize_set_option(
    mut c: *mut client,
    mut data: *mut window_customize_modedata,
    item: &window_customize_itemdata,
    mut global: ::core::ffi::c_int,
    mut pane: ::core::ffi::c_int,
) {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    let mut flag: ::core::ffi::c_int = 0;
    let mut scope: window_customize_scope = WINDOW_CUSTOMIZE_NONE;
    let mut choice: u_int = 0;
    let mut name: *const ::core::ffi::c_char = (item.name)
        .as_ref()
        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut());
    let mut space: *const ::core::ffi::c_char = b"\0" as *const u8 as *const ::core::ffi::c_char;
    let mut array_key: *const ::core::ffi::c_char = (item.array_key)
        .as_ref()
        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut());
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: Default::default(),
        wl: Default::default(),
        w: Default::default(),
        wp: Default::default(),
        idx: 0,
    };
    if window_customize_check_item(data, item, &raw mut fs) == 0 {
        return;
    }
    o = options_get(item.oo, name);
    if o.is_null() {
        return;
    }
    oe = options_table_entry(&*(o)).map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry);
    if !oe.is_null() && !(*oe).scope & OPTIONS_TABLE_PANE != 0 {
        pane = 0 as ::core::ffi::c_int;
    }
    if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
        scope = item.scope;
        oo = item.oo;
    } else {
        if global != 0 {
            match item.scope as ::core::ffi::c_uint {
                0 | 1 | 2 | 3 | 5 | 8 | 9 => {
                    scope = item.scope;
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
            match item.scope as ::core::ffi::c_uint {
                0 | 1 | 2 | 4 => {
                    scope = item.scope;
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
                    scope = item.scope;
                }
                _ => {}
            }
        }
        if scope as ::core::ffi::c_uint == item.scope as ::core::ffi::c_uint {
            oo = item.oo;
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
        if choice as usize + 1 >= (&(*oe).choices).len()
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
        let mut new_item = window_customize_new_item();

        new_item.type_0 = WINDOW_CUSTOMIZE_ITEM_OPTION;
        new_item.option_type = item.option_type;
        new_item.scope = scope;
        new_item.oo = oo;
        window_customize_set_name(&mut *new_item, Some(CStr::from_ptr(name)));
        if !array_key.is_null() {
            window_customize_set_item_array_key(&mut *new_item, Some(CStr::from_ptr(array_key)));
        }
        let owner = RefBox::new(CustomizePromptItem {
            item: new_item,
            mode: (*data).observer
                .upgrade()
                .expect("live customize mode"),
        });
        let (inputcb, freecb) =
            window_customize_prompt_callbacks(owner, window_customize_set_option_callback);
        mode_tree_set_prompt(
            (*data).data.as_ref().expect("mode tree owner").clone(),
            c,
            &prompt,
            Some(&value),
            PROMPT_TYPE_COMMAND,
            PROMPT_NOFORMAT,
            inputcb,
            freecb,
        );
    };
}
unsafe fn window_customize_set_array_key_callback(
    c: Option<NonNull<client>>,
    owner: &CustomizePromptItem,
    s: Option<&CStr>,
    _key: prompt_key_result,
) -> prompt_result {
    let c = c.map_or(std::ptr::null_mut(), NonNull::as_ptr);
    let mut s = s.map_or(::core::ptr::null(), CStr::as_ptr);
    let item = &*owner.item;
    let data = crate::src::shared::rc::as_ptr(&owner.mode);
    let Some(mode_pane_owner) = window_pane_upgrade(&(*data).wp) else {
        return PROMPT_CLOSE;
    };
    let mode_pane = crate::src::shared::rc::as_ptr(&mode_pane_owner);
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut array_key: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cause: Option<CString> = None;
    if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 || (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    name = (item.name)
        .as_ref()
        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut());
    array_key = (item.array_key)
        .as_ref()
        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut());
    if array_key.is_null()
        || window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0
    {
        return PROMPT_CLOSE;
    }
    o = options_get(item.oo, name);
    if o.is_null() {
        return PROMPT_CLOSE;
    }
    if !crate::src::options::options_array_get_mut(&mut *(o), std::ffi::CStr::from_ptr(s)).map_or(std::ptr::null_mut(), |value| value).is_null() {
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
            (item.name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        );
        mode_tree_build((*data).data.clone().as_ref().expect("mode tree owner"));
        mode_tree_draw((*data).data.clone().as_ref().expect("mode tree owner"));
        (*mode_pane).flags |= PANE_REDRAW;
        return PROMPT_CLOSE;
    };
}
unsafe fn window_customize_set_array_key(
    mut c: *mut client,
    mut data: *mut window_customize_modedata,
    item: &window_customize_itemdata,
) {
    if item.array_key.is_none()
        || window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0
    {
        return;
    }
    let mut prompt_bytes = Vec::new();
    prompt_bytes.extend_from_slice(b"(");
    prompt_bytes.extend_from_slice(
        CStr::from_ptr(
            (item.name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        )
        .to_bytes(),
    );
    prompt_bytes.extend_from_slice(b"[");
    prompt_bytes.extend_from_slice(
        CStr::from_ptr(
            (item.array_key)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        )
        .to_bytes(),
    );
    prompt_bytes.extend_from_slice(b"]) ");
    let prompt = CString::new(prompt_bytes).expect("array-key prompt contains no NUL");
    let mut new_item = window_customize_new_item();

    new_item.type_0 = WINDOW_CUSTOMIZE_ITEM_OPTION;
    new_item.option_type = item.option_type;
    new_item.scope = item.scope;
    new_item.oo = item.oo;
    window_customize_set_name(&mut *new_item, item.name.as_deref());
    window_customize_set_item_array_key(&mut *new_item, item.array_key.as_deref());
    let owner = RefBox::new(CustomizePromptItem {
        item: new_item,
        mode: (*data).observer
            .upgrade()
            .expect("live customize mode"),
    });
    let (inputcb, freecb) =
        window_customize_prompt_callbacks(owner, window_customize_set_array_key_callback);
    mode_tree_set_prompt(
        (*data).data.as_ref().expect("mode tree owner").clone(),
        c,
        &prompt,
        item.array_key.as_deref(),
        PROMPT_TYPE_COMMAND,
        PROMPT_NOFORMAT,
        inputcb,
        freecb,
    );
}
unsafe fn window_customize_unset_environment(
    mut data: *mut window_customize_modedata,
    item: &ModeTreeItemSnapshot<window_customize_itemdata>,
) {
    let Some(mut environment) = item
        .environ
        .as_ref()
        .and_then(CustomizeEnvironment::resolve)
    else {
        return;
    };
    let Some(env) = environment.get_mut() else {
        return;
    };

    if window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0 {
        return;
    }
    if environ_find(
        env,
        (item.name)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
    )
    .is_none()
    {
        return;
    }
    if mode_tree_get_current(&*(*data).data_ptr())
        .is_customize(item)
    {
        mode_tree_up((*data).data_ptr(), 0 as ::core::ffi::c_int);
    }
    environ_unset(
        env,
        (item.name)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
    );
}
unsafe fn window_customize_unset_option(
    mut data: *mut window_customize_modedata,
    item: &ModeTreeItemSnapshot<window_customize_itemdata>,
) {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    if window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0 {
        return;
    }
    o = options_get(
        item.oo,
        (item.name)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
    );
    if o.is_null() {
        return;
    }
    if !item.array_key.is_none()
        && mode_tree_get_current(&*(*data).data_ptr())
            .is_customize(item)
    {
        mode_tree_up((*data).data_ptr(), 0 as ::core::ffi::c_int);
    }
    options_remove_or_default(
        o,
        (item.array_key)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        ::core::ptr::null_mut::<Option<CString>>(),
    );
}
unsafe fn window_customize_reset_option(
    mut data: *mut window_customize_modedata,
    item: &window_customize_itemdata,
) {
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    if window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0 {
        return;
    }
    if !item.array_key.is_none() {
        return;
    }
    oo = item.oo;
    while !oo.is_null() {
        o = crate::src::options::options_get_only_mut(&mut *(oo), std::ffi::CStr::from_ptr((item.name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))).map_or(std::ptr::null_mut(), |entry| entry);
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
    c: Option<NonNull<client>>,
    owner: &CustomizePromptItem,
    s: Option<&CStr>,
    _key: prompt_key_result,
) -> prompt_result {
    let c = c.map_or(std::ptr::null_mut(), NonNull::as_ptr);
    let mut s = s.map_or(::core::ptr::null(), CStr::as_ptr);
    let item = &*owner.item;
    let data = crate::src::shared::rc::as_ptr(&owner.mode);
    let Some(mode_pane_owner) = window_pane_upgrade(&(*data).wp) else {
        return PROMPT_CLOSE;
    };
    let mode_pane = crate::src::shared::rc::as_ptr(&mode_pane_owner);
    if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 || (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    let Some(table) = window_customize_get_key_table(item) else {
        return PROMPT_CLOSE;
    };
    let mut kt = table.borrow_mut();
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
            let Some(bd) = kt.key_bindings.get_mut(item.key) else {
                drop(pr.cmdlist.take());
                return PROMPT_CLOSE;
            };
            bd.commands = pr.cmdlist.take().expect("successful command parse");
            drop(kt);
            mode_tree_build((*data).data.clone().as_ref().expect("mode tree owner"));
            mode_tree_draw((*data).data.clone().as_ref().expect("mode tree owner"));
            (*mode_pane).flags |= PANE_REDRAW;
            return PROMPT_CLOSE;
        }
    };
}
unsafe fn window_customize_set_note_callback(
    _c: Option<NonNull<client>>,
    owner: &CustomizePromptItem,
    s: Option<&CStr>,
    _key: prompt_key_result,
) -> prompt_result {
    let mut s = s.map_or(::core::ptr::null(), CStr::as_ptr);
    let item = &*owner.item;
    let data = crate::src::shared::rc::as_ptr(&owner.mode);
    let Some(mode_pane_owner) = window_pane_upgrade(&(*data).wp) else {
        return PROMPT_CLOSE;
    };
    let mode_pane = crate::src::shared::rc::as_ptr(&mode_pane_owner);
    if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 || (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    let Some(table) = window_customize_get_key_table(item) else {
        return PROMPT_CLOSE;
    };
    let mut kt = table.borrow_mut();
    let bd = kt.key_bindings.get_mut(item.key).expect("live binding");
    key_bindings_set_note(bd, Some(CStr::from_ptr(s)));
    drop(kt);
    mode_tree_build((*data).data.clone().as_ref().expect("mode tree owner"));
    mode_tree_draw((*data).data.clone().as_ref().expect("mode tree owner"));
    (*mode_pane).flags |= PANE_REDRAW;
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
    item: &window_customize_itemdata,
) {
    let mut key: key_code = item.key;
    let Some(table) = window_customize_get_key_table(item) else {
        return;
    };
    let mut kt = table.borrow_mut();
    let bd = kt.key_bindings.get_mut(item.key).expect("live binding");
    let s = mode_tree_get_current_name(&*(*data).data_ptr());
    if s.as_ref() == c"Repeat" {
        bd.flags ^= KEY_BINDING_REPEAT;
    } else if s.as_ref() == c"Command" {
        let key_string = key_string_format(key, false);
        let prompt = window_customize_key_prompt(&key_string);
        let value = cmd_list_print_cstring(&bd.cmdlist().borrow(), 0 as ::core::ffi::c_int);
        let mut new_item = window_customize_new_item();

        new_item.type_0 = WINDOW_CUSTOMIZE_ITEM_KEY;
        new_item.scope = item.scope;
        window_customize_set_table(&mut *new_item, item.table.as_deref());
        new_item.key = key;
        let owner = RefBox::new(CustomizePromptItem {
            item: new_item,
            mode: (*data).observer
                .upgrade()
                .expect("live customize mode"),
        });
        let (inputcb, freecb) =
            window_customize_prompt_callbacks(owner, window_customize_set_command_callback);
        drop(kt);
        mode_tree_set_prompt(
            (*data).data.as_ref().expect("mode tree owner").clone(),
            c,
            &prompt,
            Some(&value),
            PROMPT_TYPE_COMMAND,
            PROMPT_NOFORMAT,
            inputcb,
            freecb,
        );
    } else if s.as_ref() == c"Note" {
        let note = bd.note.clone().unwrap_or_default();
        let key_string = key_string_format(key, false);
        let prompt = window_customize_key_prompt(&key_string);
        let mut new_item = window_customize_new_item();

        new_item.type_0 = WINDOW_CUSTOMIZE_ITEM_KEY;
        new_item.scope = item.scope;
        window_customize_set_table(&mut *new_item, item.table.as_deref());
        new_item.key = key;
        let owner = RefBox::new(CustomizePromptItem {
            item: new_item,
            mode: (*data).observer
                .upgrade()
                .expect("live customize mode"),
        });
        let (inputcb, freecb) =
            window_customize_prompt_callbacks(owner, window_customize_set_note_callback);
        drop(kt);
        mode_tree_set_prompt(
            (*data).data.as_ref().expect("mode tree owner").clone(),
            c,
            &prompt,
            Some(&note),
            PROMPT_TYPE_COMMAND,
            PROMPT_NOFORMAT,
            inputcb,
            freecb,
        );
    }
}
unsafe fn window_customize_add_key_callback(
    c: Option<NonNull<client>>,
    owner: &CustomizePromptItem,
    s: Option<&CStr>,
    _key0: prompt_key_result,
) -> prompt_result {
    let c = c.map_or(std::ptr::null_mut(), NonNull::as_ptr);
    let mut s = s.map_or(::core::ptr::null(), CStr::as_ptr);
    let item = &*owner.item;
    let data = crate::src::shared::rc::as_ptr(&owner.mode);
    let Some(mode_pane_owner) = window_pane_upgrade(&(*data).wp) else {
        return PROMPT_CLOSE;
    };
    let mode_pane = crate::src::shared::rc::as_ptr(&mode_pane_owner);
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
            key_bindings_add(item.table.as_deref().expect("key binding table name"), key, { let note: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>(); (!note.is_null()).then(|| std::ffi::CStr::from_ptr(note)) }, 0 as ::core::ffi::c_int, pr.take_cmdlist());
            mode_tree_build((*data).data.clone().as_ref().expect("mode tree owner"));
            mode_tree_draw((*data).data.clone().as_ref().expect("mode tree owner"));
            (*mode_pane).flags |= PANE_REDRAW;
            return PROMPT_CLOSE;
        }
    };
}
unsafe fn window_customize_add_key(
    mut c: *mut client,
    mut data: *mut window_customize_modedata,
    table: &CStr,
) {
    let mut prompt_bytes = b"New key in ".to_vec();
    prompt_bytes.extend_from_slice(table.to_bytes());
    prompt_bytes.extend_from_slice(b": ");
    let prompt = CString::new(prompt_bytes).expect("key table name contains no NUL");
    let mut new_item = window_customize_new_item();

    new_item.type_0 = WINDOW_CUSTOMIZE_ITEM_KEY;
    new_item.scope = WINDOW_CUSTOMIZE_KEY;
    window_customize_set_table(&mut *new_item, Some(table));
    let owner = RefBox::new(CustomizePromptItem {
        item: new_item,
        mode: (*data).observer
            .upgrade()
            .expect("live customize mode"),
    });
    let (inputcb, freecb) =
        window_customize_prompt_callbacks(owner, window_customize_add_key_callback);
    mode_tree_set_prompt(
        (*data).data.as_ref().expect("mode tree owner").clone(),
        c,
        &prompt,
        Some(c""),
        PROMPT_TYPE_COMMAND,
        PROMPT_NOFORMAT,
        inputcb,
        freecb,
    );
}
unsafe fn window_customize_unset_key(
    data: *mut window_customize_modedata,
    item: &ModeTreeItemSnapshot<window_customize_itemdata>,
) {
    let Some(table) = window_customize_get_key_table(item) else {
        return;
    };
    let name = (&table.borrow()).name.clone();
    if mode_tree_get_current(&*(*data).data_ptr())
        .is_customize(item)
    {
        mode_tree_up((*data).data_ptr(), 0);
    }
    key_bindings_remove(std::ffi::CStr::from_ptr(name.as_ptr()), item.key);
}
unsafe fn window_customize_reset_key(
    data: *mut window_customize_modedata,
    item: &ModeTreeItemSnapshot<window_customize_itemdata>,
) {
    let Some(table) = window_customize_get_key_table(item) else {
        return;
    };
    let table_ref = table.borrow();
    let bd = key_bindings_get(&table_ref, item.key).expect("live binding");
    let default = key_bindings_get_default(&table_ref, item.key);
    if default.is_some_and(|default| Rc::ptr_eq(&bd.commands, &default.commands)) {
        return;
    }
    let has_default = default.is_some();
    let name = table_ref.name.clone();
    drop(table_ref);
    if !has_default
        && mode_tree_get_current(&*(*data).data_ptr())
            .is_customize(item)
    {
        mode_tree_up((*data).data_ptr(), 0);
    }
    key_bindings_reset(std::ffi::CStr::from_ptr(name.as_ptr()), item.key);
}
unsafe fn window_customize_change_each(
    mut data: *mut window_customize_modedata,
    item: &ModeTreeItemSnapshot<window_customize_itemdata>,
) {
    let mut type_0: window_customize_item_type = item.type_0;
    let name = if type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_ITEM_OPTION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        Some(
            CStr::from_ptr(
                (item.name)
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
    _c: Option<NonNull<client>>,
    owner: &UnsafeCell<window_customize_modedata>,
    s: Option<&CStr>,
    _key: prompt_key_result,
) -> prompt_result {
    let data = owner.get();
    let Some(mode_pane_owner) = window_pane_upgrade(&(*data).wp) else {
        return PROMPT_CLOSE;
    };
    let mode_pane = crate::src::shared::rc::as_ptr(&mode_pane_owner);
    let mut s = s.map_or(::core::ptr::null(), CStr::as_ptr);
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
    let item_owner = mode_tree_get_current(&*(*data).data_ptr());
    let Some(item) = item_owner.as_customize() else {
        return PROMPT_CLOSE;
    };
    type_0 = item.type_0;
    let name = if type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_ITEM_OPTION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        Some(
            CStr::from_ptr(
                (item.name)
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
                window_customize_unset_key(data, &item);
            } else if type_0 as ::core::ffi::c_uint
                == WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                window_customize_unset_environment(data, &item);
            } else {
                window_customize_unset_option(data, &item);
            }
        }
        1 => {
            if type_0 as ::core::ffi::c_uint
                == WINDOW_CUSTOMIZE_ITEM_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                window_customize_reset_key(data, &item);
            } else if type_0 as ::core::ffi::c_uint
                == WINDOW_CUSTOMIZE_ITEM_OPTION as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                window_customize_reset_option(data, &item);
            }
        }
        _ => {}
    }
    if type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_ITEM_OPTION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        options_push_changes(name.as_ref().expect("option name was copied").as_ptr());
    }
    mode_tree_build((*data).data.clone().as_ref().expect("mode tree owner"));
    mode_tree_draw((*data).data.clone().as_ref().expect("mode tree owner"));
    (*mode_pane).flags |= PANE_REDRAW;
    return PROMPT_CLOSE;
}
unsafe fn window_customize_change_tagged_callback(
    c: Option<NonNull<client>>,
    owner: &UnsafeCell<window_customize_modedata>,
    s: Option<&CStr>,
    _key: prompt_key_result,
) -> prompt_result {
    let c = c.map_or(std::ptr::null_mut(), NonNull::as_ptr);
    let data = owner.get();
    let Some(mode_pane_owner) = window_pane_upgrade(&(*data).wp) else {
        return PROMPT_CLOSE;
    };
    let mode_pane = crate::src::shared::rc::as_ptr(&mode_pane_owner);
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
        (*data).data.clone().as_ref().expect("live mode tree"),
        |row, _| unsafe {
            let itemdata = row.borrow().itemdata.clone();
            window_customize_change_each(
                data,
                &itemdata.as_customize().expect("tagged customize row"),
            )
        },
        KEYC_NONE as ::core::ffi::c_ulong as key_code,
        0 as ::core::ffi::c_int,
    );
    mode_tree_build((*data).data.clone().as_ref().expect("mode tree owner"));
    mode_tree_draw((*data).data.clone().as_ref().expect("mode tree owner"));
    (*mode_pane).flags |= PANE_REDRAW;
    return PROMPT_CLOSE;
}
unsafe fn window_customize_add_current(
    mut c: *mut client,
    mut data: *mut window_customize_modedata,
) -> ::core::ffi::c_int {
    let Some(mode_pane_owner) = window_pane_upgrade(&(*data).wp) else {
        return 1;
    };
    let mode_pane = crate::src::shared::rc::as_ptr(&mode_pane_owner);
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: Default::default(),
        wl: Default::default(),
        w: Default::default(),
        wp: Default::default(),
        idx: 0,
    };
    let name = mode_tree_get_current_name(&*(*data).data_ptr());
    if cmd_find_valid_state(&(*data).fs) != 0 {
        cmd_find_copy_state(&raw mut fs, &raw mut (*data).fs);
    } else {
        cmd_find_from_pane(&raw mut fs, mode_pane, 0 as ::core::ffi::c_int);
    }
    if name.as_ref() == c"Server Options" {
        window_customize_add_option(
            c,
            data,
            WINDOW_CUSTOMIZE_SERVER,
            global_options,
            WINDOW_CUSTOMIZE_OPTIONS,
        );
        return 1 as ::core::ffi::c_int;
    }
    if name.as_ref() == c"Session Options" {
        window_customize_add_option(
            c,
            data,
            WINDOW_CUSTOMIZE_SESSION,
            options_owner_ptr(&mut (*fs.s_ptr()).options).map_or(std::ptr::null_mut(), |options| options),
            WINDOW_CUSTOMIZE_OPTIONS,
        );
        return 1 as ::core::ffi::c_int;
    }
    if name.as_ref() == c"Window & Pane Options" {
        window_customize_add_option(
            c,
            data,
            WINDOW_CUSTOMIZE_PANE,
            options_owner_ptr(&mut (*fs.wp_ptr()).options).map_or(std::ptr::null_mut(), |options| options),
            WINDOW_CUSTOMIZE_OPTIONS,
        );
        return 1 as ::core::ffi::c_int;
    }
    if name.as_ref() == c"Session Hooks" {
        window_customize_add_option(
            c,
            data,
            WINDOW_CUSTOMIZE_SESSION,
            options_owner_ptr(&mut (*fs.s_ptr()).options).map_or(std::ptr::null_mut(), |options| options),
            WINDOW_CUSTOMIZE_HOOKS,
        );
        return 1 as ::core::ffi::c_int;
    }
    if name.as_ref() == c"Window & Pane Hooks" {
        window_customize_add_option(
            c,
            data,
            WINDOW_CUSTOMIZE_PANE,
            options_owner_ptr(&mut (*fs.wp_ptr()).options).map_or(std::ptr::null_mut(), |options| options),
            WINDOW_CUSTOMIZE_HOOKS,
        );
        return 1 as ::core::ffi::c_int;
    }
    if name.as_ref() == c"Global Environment" {
        window_customize_add_environment(
            c,
            data,
            WINDOW_CUSTOMIZE_GLOBAL_ENVIRONMENT,
            CustomizeEnvironment::Global,
        );
        return 1 as ::core::ffi::c_int;
    }
    if name.as_ref() == c"Session Environment" {
        window_customize_add_environment(
            c,
            data,
            WINDOW_CUSTOMIZE_SESSION_ENVIRONMENT,
            CustomizeEnvironment::session(fs.s_ptr()),
        );
        return 1 as ::core::ffi::c_int;
    }
    if let Some(table) = name.to_bytes_with_nul().strip_prefix(b"Key Table - ") {
        let table = CStr::from_bytes_with_nul(table).expect("key table suffix is NUL terminated");
        window_customize_add_key(c, data, table);
        return 1;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe fn window_customize_key(
    mut wme: *mut window_mode_entry,
    client_owner: &std::rc::Rc<std::cell::UnsafeCell<client>>,
    _wl: *mut winlink,
    mut key: key_code,
    mut m: *mut mouse_event,
) {
    let c = client_owner.get();
    let mode_pane_owner = (*wme).wp.upgrade().expect("mode belongs to a live pane");
    let mode_pane = mode_pane_owner.get();
    let mut wp: *mut window_pane = mode_pane;
    let mut data: *mut window_customize_modedata = (*wme).data as *mut window_customize_modedata;
    let mut finished: ::core::ffi::c_int = 0;
    let mut tagged: u_int = 0;
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
            (*data).data.as_ref().expect("mode tree owner").clone(),
            Some(client_owner),
            &raw mut key,
            m,
            ::core::ptr::null_mut::<u_int>(),
            ::core::ptr::null_mut::<u_int>(),
        );
        let item_owner = mode_tree_get_current(&*(*data).data_ptr());
        let item = item_owner.as_customize();
        match key {
            101 => {
                if let Some(item) = item {
                    window_customize_start_edit(data, &item, c);
                }
            }
            97 => {
                if let Some(item) = item.filter(|item| item.type_0 == WINDOW_CUSTOMIZE_ITEM_OPTION)
                {
                    window_customize_set_array_key(c, data, &item);
                }
            }
            13 | 115 => {
                if let Some(item) = item {
                    if item.type_0 as ::core::ffi::c_uint
                        == WINDOW_CUSTOMIZE_ITEM_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        window_customize_set_key(c, data, &item);
                    } else if item.type_0 as ::core::ffi::c_uint
                        == WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT as ::core::ffi::c_int
                            as ::core::ffi::c_uint
                    {
                        window_customize_set_environment(c, data, &item, 0 as ::core::ffi::c_int);
                    } else {
                        window_customize_set_option(
                            c,
                            data,
                            &item,
                            0 as ::core::ffi::c_int,
                            1 as ::core::ffi::c_int,
                        );
                        options_push_changes(
                            (item.name)
                                .as_ref()
                                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                        );
                    }
                    mode_tree_build((*data).data.clone().as_ref().expect("mode tree owner"));
                } else if window_customize_add_current(c, data) != 0 {
                    mode_tree_build((*data).data.clone().as_ref().expect("mode tree owner"));
                }
            }
            119 => {
                if let Some(item) = item.filter(|item| item.type_0 == WINDOW_CUSTOMIZE_ITEM_OPTION)
                {
                    window_customize_set_option(
                        c,
                        data,
                        &item,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                    options_push_changes(
                        (item.name)
                            .as_ref()
                            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                    );
                    mode_tree_build((*data).data.clone().as_ref().expect("mode tree owner"));
                }
            }
            83 | 87 => {
                if let Some(item) = item.filter(|item| item.type_0 != WINDOW_CUSTOMIZE_ITEM_KEY) {
                    if item.type_0 as ::core::ffi::c_uint
                        == WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT as ::core::ffi::c_int
                            as ::core::ffi::c_uint
                    {
                        window_customize_set_environment(c, data, &item, 1 as ::core::ffi::c_int);
                    } else {
                        window_customize_set_option(
                            c,
                            data,
                            &item,
                            1 as ::core::ffi::c_int,
                            0 as ::core::ffi::c_int,
                        );
                        options_push_changes(
                            (item.name)
                                .as_ref()
                                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                        );
                    }
                    mode_tree_build((*data).data.clone().as_ref().expect("mode tree owner"));
                }
            }
            100 => {
                if let Some(item) = item.filter(|item| {
                    !(item.type_0 == WINDOW_CUSTOMIZE_ITEM_OPTION && item.array_key.is_some()
                        || item.type_0 == WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT)
                }) {
                    let mut prompt_bytes = b"Reset ".to_vec();
                    prompt_bytes.extend_from_slice(
                        CStr::from_ptr(
                            (item.name)
                                .as_ref()
                                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                        )
                        .to_bytes(),
                    );
                    prompt_bytes.extend_from_slice(b" to default? ");
                    let reset_prompt =
                        CString::new(prompt_bytes).expect("C string parts have no NUL");
                    let owner = (*data).observer
                        .upgrade()
                        .expect("live customize mode");
                    let (inputcb, freecb) = window_customize_mode_prompt_callbacks(
                        owner,
                        window_customize_change_current_callback,
                    );
                    (*data).change = WINDOW_CUSTOMIZE_RESET;
                    mode_tree_set_prompt(
                        (*data).data.as_ref().expect("mode tree owner").clone(),
                        c,
                        &reset_prompt,
                        Some(c""),
                        PROMPT_TYPE_COMMAND,
                        PROMPT_SINGLE | PROMPT_NOFORMAT | (*data).prompt_flags,
                        inputcb,
                        freecb,
                    );
                }
            }
            68 => {
                tagged = mode_tree_count_tagged((*data).data_ptr());
                if !(tagged == 0 as u_int) {
                    let reset_prompt = CString::new(format!("Reset {tagged} tagged to default? "))
                        .expect("formatted number has no NUL");
                    let owner = (*data).observer
                        .upgrade()
                        .expect("live customize mode");
                    let (inputcb, freecb) = window_customize_mode_prompt_callbacks(
                        owner,
                        window_customize_change_tagged_callback,
                    );
                    (*data).change = WINDOW_CUSTOMIZE_RESET;
                    mode_tree_set_prompt(
                        (*data).data.as_ref().expect("mode tree owner").clone(),
                        c,
                        &reset_prompt,
                        Some(c""),
                        PROMPT_TYPE_COMMAND,
                        PROMPT_SINGLE | PROMPT_NOFORMAT | (*data).prompt_flags,
                        inputcb,
                        freecb,
                    );
                }
            }
            117 => {
                if let Some(item) = item {
                    let mut prompt_bytes = b"Unset ".to_vec();
                    prompt_bytes.extend_from_slice(
                        CStr::from_ptr(
                            (item.name)
                                .as_ref()
                                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                        )
                        .to_bytes(),
                    );
                    if !item.array_key.is_none() {
                        prompt_bytes.push(b'[');
                        prompt_bytes.extend_from_slice(
                            CStr::from_ptr(
                                (item.array_key)
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
                    let owner = (*data).observer
                        .upgrade()
                        .expect("live customize mode");
                    let (inputcb, freecb) = window_customize_mode_prompt_callbacks(
                        owner,
                        window_customize_change_current_callback,
                    );
                    (*data).change = WINDOW_CUSTOMIZE_UNSET;
                    mode_tree_set_prompt(
                        (*data).data.as_ref().expect("mode tree owner").clone(),
                        c,
                        &prompt,
                        Some(c""),
                        PROMPT_TYPE_COMMAND,
                        PROMPT_SINGLE | PROMPT_NOFORMAT | (*data).prompt_flags,
                        inputcb,
                        freecb,
                    );
                }
            }
            85 => {
                tagged = mode_tree_count_tagged((*data).data_ptr());
                if !(tagged == 0 as u_int) {
                    let prompt = CString::new(format!("Unset {tagged} tagged? ")).unwrap();
                    let owner = (*data).observer
                        .upgrade()
                        .expect("live customize mode");
                    let (inputcb, freecb) = window_customize_mode_prompt_callbacks(
                        owner,
                        window_customize_change_tagged_callback,
                    );
                    (*data).change = WINDOW_CUSTOMIZE_UNSET;
                    mode_tree_set_prompt(
                        (*data).data.as_ref().expect("mode tree owner").clone(),
                        c,
                        &prompt,
                        Some(c""),
                        PROMPT_TYPE_COMMAND,
                        PROMPT_SINGLE | PROMPT_NOFORMAT | (*data).prompt_flags,
                        inputcb,
                        freecb,
                    );
                }
            }
            72 => {
                (*data).hide_global = ((*data).hide_global == 0) as ::core::ffi::c_int;
                mode_tree_build((*data).data.clone().as_ref().expect("mode tree owner"));
            }
            67 => {
                (*data).hide_default = ((*data).hide_default == 0) as ::core::ffi::c_int;
                mode_tree_build((*data).data.clone().as_ref().expect("mode tree owner"));
            }
            _ => {}
        }
    }
    if finished != 0 {
        window_pane_reset_mode(&mode_pane_owner);
    } else {
        mode_tree_draw((*data).data.clone().as_ref().expect("mode tree owner"));
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
    use std::cell::Cell;
    use std::cell::RefCell;

    #[test]
    fn detached_copy_owns_byte_preserving_strings() {
        let mut item = window_customize_new_item();
        let table = CString::new(b"\xfftable".to_vec()).unwrap();
        window_customize_set_table(&mut item, Some(&table));
        window_customize_set_name(&mut item, Some(c"first"));
        window_customize_set_item_array_key(&mut item, Some(c""));
        let copy = window_customize_copy_item(&item);

        window_customize_set_table(&mut item, Some(c"changed"));
        window_customize_set_name(&mut item, Some(c"second"));
        drop(item);

        assert_eq!(copy.table.as_ref().unwrap().as_bytes(), b"\xfftable");
        assert_eq!(copy.name.as_deref(), Some(c"first"));
        assert_eq!(copy.array_key.as_deref(), Some(c""));
    }

    #[test]
    fn mode_list_keeps_items_stable_until_last_callback_reference() {
        unsafe fn read_item(
            _client: Option<NonNull<client>>,
            owner: &CustomizePromptItem,
            _text: Option<&CStr>,
            _key: prompt_key_result,
        ) -> prompt_result {
            assert_eq!(owner.item.name.as_deref(), Some(c"stable"));
            assert_eq!(Rc::strong_count(&owner.mode), 1);
            PROMPT_CONTINUE
        }
        unsafe {
            let owner = Rc::new_cyclic(|observer| std::cell::UnsafeCell::new(window_customize_modedata {
        observer: observer.clone(),
                wp: Weak::new(),
                dead: 0,
                data: None,
                editor: std::ptr::null_mut(),
                format: CString::new(Vec::new()).unwrap(),
                hide_global: 0,
                hide_default: 0,
                prompt_flags: 0,
                item_list: Vec::new(),
                fs: cmd_find_state {
                    flags: 0,
                    current: std::ptr::null_mut(),
                    s: Default::default(),
                    wl: Default::default(),
                    w: Default::default(),
                    wp: Default::default(),
                    idx: 0,
                },
                change: WINDOW_CUSTOMIZE_UNSET,
            }));
            let data = crate::src::shared::rc::as_ptr(&owner);
            let first_owner = window_customize_add_item(
                &mut (*data).item_list,
                window_customize_itemdata {
                    name: Some(c"stable".to_owned()),
                    ..window_customize_itemdata::new()
                },
            );
            let observer = first_owner.clone();
            // A key's parent and detail rows share a single immutable record.
            let selected = ModeTreeItemData::Customize(first_owner.clone());
            let detail = selected.clone();
            assert!(selected.same_identity(&detail));
            let snapshot = selected.as_customize().unwrap();
            for _ in 0..128 {
                window_customize_add_item(&mut (*data).item_list, window_customize_itemdata::new());
            }
            assert!(first_owner.is(&(&(*data).item_list)[0]));
            let mode_observer = (*data).observer.clone();
            let prompt_owner = RefBox::new(CustomizePromptItem {
                item: window_customize_copy_item(&snapshot),
                mode: mode_observer.upgrade().unwrap(),
            });
            let prompt_observer = RefBox::downgrade(&prompt_owner);
            let (mut inputcb, freecb) = window_customize_prompt_callbacks(prompt_owner, read_item);
            drop(owner);
            assert_eq!(
                inputcb.as_mut().unwrap()(None, None, PROMPT_KEY_CLOSE),
                PROMPT_CONTINUE
            );
            freecb.unwrap()();
            assert!(mode_observer.upgrade().is_none());
            assert!(matches!(
                prompt_observer.try_borrow_mut(),
                Err(refbox::BorrowError::Dropped)
            ));
            // Caching the input callback must not keep a closed prompt alive.
            assert_eq!(
                inputcb.as_mut().unwrap()(None, None, PROMPT_KEY_CLOSE),
                PROMPT_CLOSE
            );
            assert!(!observer.is_alive());
            assert!(selected.as_customize().is_none());
            assert!(detail.as_customize().is_none());
            assert_eq!(snapshot.name.as_deref(), Some(c"stable"));
        }
    }

    #[test]
    fn prompt_input_keeps_owner_alive_while_closing_its_own_prompt() {
        struct Owner {
            drops: Rc<Cell<usize>>,
            cleanup: RefCell<prompt_free_cb>,
        }
        impl Drop for Owner {
            fn drop(&mut self) {
                self.drops.set(self.drops.get() + 1);
            }
        }
        unsafe fn close(
            _client: Option<NonNull<client>>,
            owner: &Owner,
            _text: Option<&CStr>,
            _key: prompt_key_result,
        ) -> prompt_result {
            let freecb = owner.cleanup.borrow_mut().take().unwrap();
            freecb();
            assert_eq!(owner.drops.get(), 0);
            PROMPT_CONTINUE
        }
        let drops = Rc::new(Cell::new(0));
        let owner = RefBox::new(Owner {
            drops: drops.clone(),
            cleanup: RefCell::new(None),
        });
        let observer = RefBox::downgrade(&owner);
        let (mut inputcb, freecb) = window_customize_prompt_callbacks(owner, close);
        *observer.try_borrow_mut().unwrap().cleanup.borrow_mut() = freecb;
        assert_eq!(
            inputcb.as_mut().unwrap()(None, None, PROMPT_KEY_CLOSE),
            PROMPT_CONTINUE
        );
        assert_eq!(drops.get(), 1);
        assert!(matches!(
            observer.try_borrow_mut(),
            Err(refbox::BorrowError::Dropped)
        ));
        assert_eq!(
            inputcb.as_mut().unwrap()(None, None, PROMPT_KEY_CLOSE),
            PROMPT_CLOSE
        );
        drop(inputcb);
        assert_eq!(drops.get(), 1);
    }

    #[test]
    fn editor_cancellation_drops_its_owned_record_without_dispatch() {
        let edit = Box::new(window_customize_editdata {
            wp_id: u_int::MAX,
            edit_type: WINDOW_CUSTOMIZE_EDIT_OPTION,
            item: window_customize_new_item(),
            });
        let capture = (edit, Rc::new(()));
        let observer = Rc::downgrade(&capture.1);
        let mut state = spawn_editor_state {
            path: c"unused".to_owned(),
            pid: 0,
            cb: Some(Box::new(move |_, _| {
                drop(capture);
                panic!("cancelled editor callback ran");
            })),
        };
        assert!(observer.upgrade().is_some());
        unsafe {
            spawn_cancel_editor(&mut state);
        }
        assert!(observer.upgrade().is_none());
        assert!(state.cb.is_none());
        unsafe {
            spawn_cancel_editor(&mut state);
        }
    }
}

#[cfg(test)]
mod environment_lifetime_tests {
    use super::*;
    use crate::src::environ::environ_create;
    use crate::src::shared::rc;

    #[test]
    fn detached_environment_rows_expire_after_their_session() {
        unsafe {
            let owner = session::new();
            let session = rc::as_ptr(&owner);
            (*session).environ = Some(environ_create());
            (*session)
                .environ
                .as_deref_mut()
                .unwrap()
                .set(b"NAME", 0, b"value")
                .unwrap();
                        let target = CustomizeEnvironment::session(session);
            let mut row = window_customize_new_item();
            row.environ = Some(target.clone());
            let detached = window_customize_copy_item(&row);
            assert!(target.matches(detached.environ.as_ref().unwrap()));
            let guard = target.resolve().unwrap();
            drop(owner);
            assert_eq!(
                guard.get().unwrap().find(c"NAME").unwrap().value(),
                Some(c"value")
            );
            assert!(target.resolve().is_some());
            let lifetime = (*session).observer.clone();
            drop(guard);
            assert!(lifetime.upgrade().is_some());
            crate::src::reactor::event_loop();
            assert!(target.resolve().is_none());
            crate::src::reactor::shutdown_runtime();
            assert!(detached.environ.as_ref().unwrap().resolve().is_none());
        }
    }

    #[test]
    fn environment_guards_reborrow_the_current_box_and_detect_removal() {
        unsafe {
            let owner = session::new();
            let session = rc::as_ptr(&owner);
            (*session).environ = Some(environ_create());
                        let target = CustomizeEnvironment::session(session);
            let mut guard = target.resolve().unwrap();
            guard
                .get_mut()
                .unwrap()
                .set(b"NAME", ENVIRON_HIDDEN, b"first")
                .unwrap();
            let replacement = environ_create();
            (*session).environ = Some(replacement);
            assert!(guard.get().unwrap().find(c"NAME").is_none());
            drop((*session).environ.take());
            assert!(guard.get().is_none());
            let lifetime = (*session).observer.clone();
            drop(guard);
            drop(owner);
            assert!(lifetime.upgrade().is_some());
            crate::src::reactor::shutdown_runtime();
            assert!(target.resolve().is_none());
        }
    }
}
