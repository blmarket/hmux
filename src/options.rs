use crate::src::alerts::alerts_reset_all;
use crate::src::arguments::{args_get, args_has};
use crate::src::cmd::cmd_list_print_cstring;
use crate::src::cmd::parse::cmd_parse_from_string;
use crate::src::compat::strtonum::strtonum;
use crate::src::ffi::libc::{fnmatch, strcasecmp, strcmp, strncmp, strsep, strstr};
use crate::src::format::bytes::format_message_with;
use crate::src::format::bytes::write_cstr;
use crate::src::format::bytes::xformat;
use crate::src::format::format_expand_cstring;
use crate::src::grid::grid_default_cell;
use crate::src::input::input_set_buffer_size;
use crate::src::key_string::{key_string_format, key_string_parse_cstr};
use crate::src::layout::layout_fix_panes;
use crate::src::log::{fatalx, log_cstr, log_debug};
use crate::src::options_parse::{
    match_option_name, parse_array_index, parse_option_name, ArrayIndex, ArrayIndexError,
    OptionNameMatch, OptionNameMatchError,
};
use crate::src::options_table::{options_other_names, options_table};
use crate::src::resize::recalculate_sizes;
use crate::src::screen_redraw::redraw_invalidate_all_scenes;
use crate::src::server::clients;
use crate::src::server::current_time;
use crate::src::server_client::{server_client_set_key_table, server_client_update_theme_colours};
use crate::src::server_fn::server_redraw_client;
use crate::src::session::sessions;
use crate::src::session::{session_update_history, sessions_minmax, sessions_next};
use crate::src::shared::options::options_name_map;
use crate::src::status::{status_timer_start_all, status_update_cache};
use crate::src::style::colour::{colour_format, colour_palette_from_option, colour_parse_cstr};
use crate::src::style::{
    style_parse, style_parse_colour, style_set, style_set_scrollbar_style_from_option,
};
use crate::src::text::utf8::utf8_update_width_cache;
use crate::src::tmux::{checkshell, global_options, global_s_options, global_w_options};
use crate::src::tty::tty_invalidate;
use crate::src::tty_keys::tty_keys_build;
use crate::src::window::windows;
use crate::src::window::{
    all_window_panes, window_pane_default_cursor, window_pane_scrollbar_hide,
    window_pane_tree_minmax, window_pane_tree_next, windows_minmax, windows_next,
};
use crate::src::window_border::window_set_fill_cells;
use std::ffi::{CStr, CString};

macro_rules! store_options_cause {
    ($cause:expr, $message:expr) => {{
        let cause = $cause;
        if !cause.is_null() {
            *cause = Some($message);
        }
    }};
}

macro_rules! format_options_cause {
    ($cause:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {{
        let cause = $cause;
        if !cause.is_null() {
            *cause = Some(options_string_cause(CStr::from_ptr($fmt), &[$($arg),*]));
        }
    }};
}

/// The option diagnostics use only literal text and `%s` substitutions.
unsafe fn options_string_cause(fmt: &CStr, args: &[*const ::core::ffi::c_char]) -> CString {
    let fmt = fmt.to_bytes();
    let mut message = Vec::with_capacity(fmt.len());
    let mut at = 0;
    let mut arg = 0;
    while at < fmt.len() {
        if fmt[at..].starts_with(b"%s") {
            let ptr = args[arg];
            if ptr.is_null() {
                message.extend_from_slice(b"(null)");
            } else {
                message.extend_from_slice(CStr::from_ptr(ptr).to_bytes());
            }
            at += 2;
            arg += 1;
        } else {
            message.push(fmt[at]);
            at += 1;
        }
    }
    debug_assert_eq!(arg, args.len());
    CString::new(message).expect("option diagnostic contains no NUL")
}

use crate::src::shared::abi::*;
use crate::src::shared::arguments::args;
use crate::src::shared::client::client;
use crate::src::shared::command::{cmd_find_state, cmd_list};
use crate::src::shared::command::{cmd_parse_input, cmd_parse_result};
use crate::src::shared::format::format_tree;
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
use crate::src::shared::limits::UINT_MAX;
use crate::src::shared::options::*;
use crate::src::shared::options::{
    options, options_array_item, options_array_storage, options_entry, options_table_entry,
    options_value, OptionCommand, OptionsArrayKey,
};
use crate::src::shared::options::{
    OPTIONS_TABLE_IS_ARRAY, OPTIONS_TABLE_IS_COLOUR, OPTIONS_TABLE_IS_STYLE, OPTIONS_TABLE_NONE,
    OPTIONS_TABLE_PANE, OPTIONS_TABLE_SERVER, OPTIONS_TABLE_SESSION, OPTIONS_TABLE_WINDOW,
};
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::{PANE_CHANGED, PANE_STYLECHANGED, PANE_THEMECHANGED};
use crate::src::shared::session::session;
use crate::src::shared::style::*;
use crate::src::shared::tree::RB_NEGINF;
use crate::src::shared::tty::TTY_OPENED;
use crate::src::shared::window::{window, winlink};

fn set_scalar_string(o: &mut options_entry, value: CString) {
    // Formatting has completed, so callers may have supplied the old value
    // as a %s argument. Replace its owner before publishing the new pointer.
    o.value = options_value::String(value);
}

fn set_array_string(a: &mut options_array_item, value: CString) {
    a.value = options_value::String(value);
}

fn options_array_correct_key(key: &CStr) -> Option<CString> {
    match parse_array_index(key.to_bytes()) {
        Ok(ArrayIndex::Numeric(number)) => {
            Some(CString::new(number.to_string()).expect("numeric key has no NUL"))
        }
        Ok(ArrayIndex::Text(bytes)) => {
            Some(CString::new(bytes).expect("C string key has no interior NUL"))
        }
        Err(ArrayIndexError::Empty | ArrayIndexError::NumericOverflow) => None,
    }
}
// Internal callers pass keys already validated by options_array_correct_key.
pub(crate) fn options_array_index(key: &CStr) -> OptionsArrayKey {
    match parse_array_index(key.to_bytes()).expect("validated array key") {
        ArrayIndex::Numeric(number) => OptionsArrayKey::Numeric(number),
        ArrayIndex::Text(bytes) => OptionsArrayKey::Text(bytes.to_vec()),
    }
}

unsafe fn options_map_name(mut name: *const ::core::ffi::c_char) -> *const ::core::ffi::c_char {
    let mut map: *const options_name_map = ::core::ptr::null::<options_name_map>();
    map = &raw const options_other_names as *const options_name_map;
    while !(*map).from.to_bytes().is_empty() {
        if strcmp((*map).from.as_ptr(), name) == 0 as ::core::ffi::c_int {
            return (*map).to.as_ptr();
        }
        map = map.offset(1);
    }
    return name;
}
unsafe fn options_parent_table_entry(
    mut oo: *mut options,
    mut s: *const ::core::ffi::c_char,
) -> *const options_table_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    if (*oo).parent.is_null() {
        fatalx(|out| {
            out.write_all(b"no parent options for ")?;
            write_cstr(out, s)
        });
    }
    o = options_get((*oo).parent, s);
    if o.is_null() {
        fatalx(|out| {
            write_cstr(out, s)?;
            out.write_all(b" not in parent options")
        });
    }
    return (*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry);
}
unsafe fn options_value_free(ov: *mut options_value) {
    *ov = options_value::Empty;
}
unsafe fn options_value_to_cstring(
    o: &options_entry,
    ov: &options_value,
    numeric: ::core::ffi::c_int,
) -> CString {
    let tableentry = o.tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry);
    if !tableentry.is_null()
        && (*tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_COMMAND as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return cmd_list_print_cstring(&ov.commands().expect("command option").borrow(), 0);
    }
    if !tableentry.is_null()
        && ((*tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_COLOUR as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_FLAG as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_CHOICE as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        return match (*tableentry).type_0 as ::core::ffi::c_uint {
            1 => CString::new(ov.number().to_string()).expect("decimal number has no NUL"),
            2 => key_string_format(ov.number() as key_code, false),
            3 => colour_format(ov.number() as ::core::ffi::c_int),
            4 => {
                if numeric != 0 {
                    CString::new(ov.number().to_string()).expect("decimal number has no NUL")
                } else {
                    CStr::from_ptr(if ov.number() != 0 {
                        b"on\0" as *const u8 as *const ::core::ffi::c_char
                    } else {
                        b"off\0" as *const u8 as *const ::core::ffi::c_char
                    })
                    .to_owned()
                }
            }
            5 => (*tableentry).choices[ov.number() as usize].to_owned(),
            _ => {
                fatalx(|out| out.write_all(b"not a number option type"));
            }
        };
    }
    if tableentry.is_null()
        || (*tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return CStr::from_ptr(ov.string_ptr().map_or(std::ptr::null_mut(), |value| value.as_ptr().cast_mut())).to_owned();
    }
    c"".to_owned()
}
/// Project only the owning field for a legacy raw-pointer call. The result is
/// an observer, valid only while the box remains installed and live.
pub fn options_owner_ptr(owner: &mut Option<Box<options>>) -> Option<&mut options> {
    owner.as_deref_mut()
}

pub fn options_create_owned(parent: *mut options) -> Box<options> {
    Box::new(options {
        tree: Default::default(),
        parent,
    })
}

/// Legacy ownership transfer. The caller must eventually consume the returned
/// allocation with options_free or transfer it to a model owner.
pub unsafe fn options_create(parent: *mut options) -> *mut options {
    Box::into_raw(options_create_owned(parent))
}

pub unsafe fn options_free(oo: *mut options) {
    drop(Box::from_raw(oo));
}

impl Drop for options {
    fn drop(&mut self) {
        unsafe {
            // Preserve value-before-monitor cleanup and the existing key order.
            let names: Vec<_> = options_iter(self).map(|entry| entry.name.clone()).collect();
            for name in names {
                if let Some(entry) = options_get_only_mut(self, &name) {
                    options_remove(entry);
                }
            }
        }
    }
}

pub unsafe fn options_get_parent(mut oo: *mut options) -> *mut options {
    return (*oo).parent;
}
pub unsafe fn options_set_parent(mut oo: *mut options, mut parent: *mut options) {
    (*oo).parent = parent;
}
pub fn options_iter(oo: &options) -> impl DoubleEndedIterator<Item = &options_entry> {
    oo.tree.values().map(Box::as_ref)
}
pub fn options_iter_mut(oo: &mut options) -> impl DoubleEndedIterator<Item = &mut options_entry> {
    oo.tree.values_mut().map(Box::as_mut)
}
pub fn options_get_only<'a>(oo: &'a options, name: &CStr) -> Option<&'a options_entry> {
    let key = if oo.tree.contains_key(name.to_bytes()) {
        name.to_bytes()
    } else {
        unsafe { CStr::from_ptr(options_map_name(name.as_ptr())).to_bytes() }
    };
    oo.tree.get(key).map(Box::as_ref)
}
pub fn options_get_only_mut<'a>(oo: &'a mut options, name: &CStr) -> Option<&'a mut options_entry> {
    let key = if oo.tree.contains_key(name.to_bytes()) {
        name.to_bytes()
    } else {
        unsafe { CStr::from_ptr(options_map_name(name.as_ptr())).to_bytes() }
    };
    oo.tree.get_mut(key).map(Box::as_mut)
}
pub unsafe fn options_get(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
) -> *mut options_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    o = crate::src::options::options_get_only_mut(&mut *(oo), std::ffi::CStr::from_ptr(name)).map_or(std::ptr::null_mut(), |entry| entry);
    while o.is_null() {
        oo = (*oo).parent;
        if oo.is_null() {
            break;
        }
        o = crate::src::options::options_get_only_mut(&mut *(oo), std::ffi::CStr::from_ptr(name)).map_or(std::ptr::null_mut(), |entry| entry);
    }
    return o;
}
pub unsafe fn options_empty(
    oo: *mut options,
    definition: &'static options_table_entry,
) -> *mut options_entry {
    let o = options_add(oo, definition.name_ptr());
    (*o).tableentry = Some(definition);
    if definition.flags & OPTIONS_TABLE_IS_ARRAY != 0 {
        (*o).value = options_value::Array(options_array_storage::default());
    }
    return o;
}
pub unsafe fn options_default(
    mut oo: *mut options,
    mut oe: *const options_table_entry,
) -> *mut options_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut ov: *mut options_value = ::core::ptr::null_mut::<options_value>();
    let mut key: [::core::ffi::c_char; 32] = [0; 32];
    let mut pr: cmd_parse_result = cmd_parse_result::empty();
    o = options_empty(oo, options_table.iter().find(|entry| std::ptr::eq(*entry, oe)).expect("static option definition"));
    ov = &raw mut (*o).value;
    if (*oe).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
        if (*oe).default_arr.is_none() {
            options_array_assign(
                o,
                (*oe).default_str_ptr(),
                ::core::ptr::null_mut::<Option<CString>>(),
            );
            return o;
        }
        for (i, value) in (*oe).default_arr.expect("array defaults").iter().enumerate() {
            xformat(&mut key, format_args!("{}", i as u32));
            options_array_set(
                o,
                &raw mut key as *mut ::core::ffi::c_char,
                value.as_ptr(),
                0 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<Option<CString>>(),
            );
        }
        return o;
    }
    match (*oe).type_0 as ::core::ffi::c_uint {
        0 => {
            set_scalar_string(&mut *o, CStr::from_ptr((*oe).default_str_ptr()).to_owned());
        }
        6 => {
            pr = cmd_parse_from_string(
                CStr::from_ptr((*oe).default_str_ptr()),
                ::core::ptr::null_mut::<cmd_parse_input>(),
            );
            match pr.status as ::core::ffi::c_uint {
                0 => {}
                1 => {
                    (*ov) = options_value::Command(OptionCommand(pr.cmdlist.take()));
                }
                _ => {}
            }
        }
        _ => {
            (*ov) = options_value::Number((*oe).default_num);
        }
    }
    return o;
}
/// Format a built-in default for Rust callers. The exported C API keeps its
/// libc-owned return value below.
pub(crate) unsafe fn options_default_to_cstring(oe: &options_table_entry) -> CString {
    match oe.type_0 as ::core::ffi::c_uint {
        0 | 6 => CStr::from_ptr(oe.default_str_ptr()).to_owned(),
        1 => CString::new(oe.default_num.to_string())
            .expect("decimal option default contains no NUL"),
        2 => key_string_format(oe.default_num as key_code, false),
        3 => colour_format(oe.default_num as ::core::ffi::c_int),
        4 => if oe.default_num != 0 { c"on" } else { c"off" }.to_owned(),
        5 => oe.choices[oe.default_num as usize].to_owned(),
        _ => {
            fatalx(|out| out.write_all(b"unknown option type"));
        }
    }
}
unsafe fn options_add(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
) -> *mut options_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let name = CStr::from_ptr(name).to_owned();
    let lookup_name = name.as_ptr();
    o = crate::src::options::options_get_only_mut(&mut *(oo), std::ffi::CStr::from_ptr(lookup_name)).map_or(std::ptr::null_mut(), |entry| entry);
    if !o.is_null() {
        options_remove(o);
    }
    let mut owned = Box::new(options_entry {
        owner: oo,
        name,
        tableentry: None,
        value: options_value::Empty,
        cached: 0,
        style: Default::default(),
        monitor_data: None,
        fire_count: 0,
        fire_time: 0,
    });
    o = &raw mut *owned;
    (*oo).tree.insert(owned.name.as_bytes().to_vec(), owned);
    return o;
}
unsafe fn options_remove(mut o: *mut options_entry) {
    let mut oo: *mut options = (*o).owner;
    if !(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry).is_null() && (*(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry)).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
        options_array_clear(o);
    }
    // Release the value before the monitor, including array storage after its
    // items have been cleared.
    options_value_free(&raw mut (*o).value);
    drop((*o).monitor_data.take());
    // Cleanup above must precede unlinking and dropping the boxed entry.
    let key = (*o).name.as_bytes().to_vec();
    drop((*oo).tree.remove(key.as_slice()));
}
pub fn options_name(o: &options_entry) -> &CStr {
    &o.name
}
pub unsafe fn options_owner(mut o: *mut options_entry) -> *mut options {
    return (*o).owner;
}
pub fn options_get_monitor_data(o: &mut options_entry) -> Option<&mut crate::src::hooks::hooks_monitor> {
    o.monitor_data.as_deref_mut()
}
pub unsafe fn options_set_monitor_data(
    mut o: *mut options_entry,
    data: Option<Box<crate::src::hooks::hooks_monitor>>,
) {
    let previous = std::mem::replace(&mut (*o).monitor_data, data);
    drop(previous);
}
pub unsafe fn options_hook_fired(mut o: *mut options_entry) {
    (*o).fire_count = (*o).fire_count.wrapping_add(1);
    (*o).fire_time = current_time;
}
pub unsafe fn options_get_fire_count(mut o: *mut options_entry) -> u_int {
    return (*o).fire_count;
}
pub unsafe fn options_get_fire_time(mut o: *mut options_entry) -> time_t {
    return (*o).fire_time;
}
pub fn options_table_entry(o: &options_entry) -> Option<&'static options_table_entry> {
    o.tableentry
}
pub(crate) unsafe fn options_array_item(
    mut o: *mut options_entry,
    mut key: *const ::core::ffi::c_char,
) -> *mut options_array_item {
    (*o).value
        .array_storage()
        .entries
        .get_mut(&options_array_index(CStr::from_ptr(key)))
        .map(|item| &mut **item as *mut options_array_item)
        .unwrap_or(::core::ptr::null_mut())
}
unsafe fn options_array_new(
    mut o: *mut options_entry,
    mut key: *const ::core::ffi::c_char,
) -> *mut options_array_item {
    let mut owner = Box::new(options_array_item {
        key: CStr::from_ptr(key).to_owned(),
        value: options_value::Empty,
        owner: o,
    });
    let a = &raw mut *owner;
    (*o).value
        .array_storage()
        .entries
        .insert(options_array_index(CStr::from_ptr(key)), owner);
    return a;
}
unsafe fn options_array_free(mut o: *mut options_entry, mut a: *mut options_array_item) {
    options_value_free(&raw mut (*a).value);
    let key = options_array_index((*a).key.as_c_str());
    drop((*o).value.array_storage().entries.remove(&key));
}
pub unsafe fn options_array_clear(mut o: *mut options_entry) {
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut a1: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    if !(!(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry).is_null() && (*(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry)).flags & OPTIONS_TABLE_IS_ARRAY != 0) {
        return;
    }
    let keys: Vec<_> = options_array_iter(&*o).map(|item| item.key.clone()).collect();
    for key in keys {
        let item = options_array_item(o, key.as_ptr());
        if !item.is_null() { options_array_free(o, item); }
    }
}
pub fn options_array_get<'a>(o: &'a options_entry, key: &CStr) -> Option<&'a options_value> {
    let key = options_array_correct_key(key)?;
    let options_value::Array(array) = &o.value else { return None; };
    array.entries.get(&options_array_index(&key)).map(|item| &item.value)
}
pub fn options_array_get_mut<'a>(o: &'a mut options_entry, key: &CStr) -> Option<&'a mut options_value> {
    let key = options_array_correct_key(key)?;
    let options_value::Array(array) = &mut o.value else { return None; };
    array.entries.get_mut(&options_array_index(&key)).map(|item| &mut item.value)
}
/// Look up an array value by its decimal numeric index.
pub fn options_array_get_index(o: &options_entry, index: u_int) -> Option<&options_value> {
    let key = CString::new(index.to_string()).expect("decimal index contains no NUL");
    options_array_get(o, &key)
}
pub fn options_array_get_index_mut(o: &mut options_entry, index: u_int) -> Option<&mut options_value> {
    let key = CString::new(index.to_string()).expect("decimal index contains no NUL");
    options_array_get_mut(o, &key)
}
pub unsafe fn options_array_set(
    mut o: *mut options_entry,
    mut key: *const ::core::ffi::c_char,
    mut value: *const ::core::ffi::c_char,
    mut append: ::core::ffi::c_int,
    mut cause: *mut Option<CString>,
) -> ::core::ffi::c_int {
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut pr: cmd_parse_result = cmd_parse_result::empty();
    let mut number: ::core::ffi::c_longlong = 0;
    if !(!(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry).is_null() && (*(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry)).flags & OPTIONS_TABLE_IS_ARRAY != 0) {
        if !cause.is_null() {
            store_options_cause!(cause, CString::new(b"not an array".to_vec()).unwrap());
        }
        return -(1 as ::core::ffi::c_int);
    }
    let Some(new_key) = options_array_correct_key(CStr::from_ptr(key)) else {
        if !cause.is_null() {
            format_options_cause!(
                cause,
                b"bad array key: %s\0" as *const u8 as *const ::core::ffi::c_char,
                key,
            );
        }
        return -(1 as ::core::ffi::c_int);
    };
    if value.is_null() {
        a = options_array_item(o, new_key.as_ptr());
        if !a.is_null() {
            options_array_free(o, a);
        }
        return 0 as ::core::ffi::c_int;
    }
    if !(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry).is_null()
        && (*(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry)).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_COMMAND as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        pr = cmd_parse_from_string(
            CStr::from_ptr(value),
            ::core::ptr::null_mut::<cmd_parse_input>(),
        );
        match pr.status as ::core::ffi::c_uint {
            0 => {
                if !cause.is_null() {
                    *cause = pr.error;
                }
                return -(1 as ::core::ffi::c_int);
            }
            1 | _ => {}
        }
        a = options_array_item(o, new_key.as_ptr());
        if a.is_null() {
            a = options_array_new(o, new_key.as_ptr());
        } else {
            options_value_free(&raw mut (*a).value);
        }
        (*a).value = options_value::Command(OptionCommand(pr.cmdlist.take()));
        return 0 as ::core::ffi::c_int;
    }
    if (*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry).is_null()
        || (*(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry)).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        a = options_array_item(o, new_key.as_ptr());
        let owned_value = if !a.is_null() && append != 0 {
            let previous = CStr::from_ptr((*a).value.string_ptr().map_or(std::ptr::null_mut(), |value| value.as_ptr().cast_mut())).to_bytes();
            let suffix = CStr::from_ptr(value).to_bytes();
            let mut bytes = Vec::with_capacity(previous.len() + suffix.len());
            bytes.extend_from_slice(previous);
            bytes.extend_from_slice(suffix);
            CString::new(bytes).expect("C string parts contain no NUL")
        } else {
            CStr::from_ptr(value).to_owned()
        };
        if a.is_null() {
            a = options_array_new(o, new_key.as_ptr());
        }
        set_array_string(&mut *a, owned_value);
        return 0 as ::core::ffi::c_int;
    }
    if (*(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry)).type_0 as ::core::ffi::c_uint
        == OPTIONS_TABLE_COLOUR as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        number = colour_parse_cstr(std::ffi::CStr::from_ptr(value)).unwrap_or(-1)
            as ::core::ffi::c_longlong;
        if number == -(1 as ::core::ffi::c_int) as ::core::ffi::c_longlong {
            format_options_cause!(
                cause,
                b"bad colour: %s\0" as *const u8 as *const ::core::ffi::c_char,
                value,
            );
            return -(1 as ::core::ffi::c_int);
        }
        a = options_array_item(o, new_key.as_ptr());
        if a.is_null() {
            a = options_array_new(o, new_key.as_ptr());
        } else {
            options_value_free(&raw mut (*a).value);
        }
        (*a).value = options_value::Number(number);
        return 0 as ::core::ffi::c_int;
    }
    if !cause.is_null() {
        store_options_cause!(cause, CString::new(b"wrong array type".to_vec()).unwrap());
    }
    return -(1 as ::core::ffi::c_int);
}
pub unsafe fn options_array_assign(
    mut o: *mut options_entry,
    mut s: *const ::core::ffi::c_char,
    mut cause: *mut Option<CString>,
) -> ::core::ffi::c_int {
    let mut separator: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut key: [::core::ffi::c_char; 32] = [0; 32];
    let mut i: u_int = 0;
    separator = (*(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry)).separator_ptr();
    if separator.is_null() {
        separator = b" ,\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if *separator as ::core::ffi::c_int == '\0' as i32 {
        if *s as ::core::ffi::c_int == '\0' as i32 {
            return 0 as ::core::ffi::c_int;
        }
        i = 0 as u_int;
        while i < UINT_MAX {
            if crate::src::options::options_array_get_index_mut(&mut *(o), i).map_or(std::ptr::null_mut(), |value| value).is_null() {
                break;
            }
            i = i.wrapping_add(1);
        }
        xformat(&mut key, format_args!("{}", i as u32));
        return options_array_set(
            o,
            &raw mut key as *mut ::core::ffi::c_char,
            s,
            0 as ::core::ffi::c_int,
            cause,
        );
    }
    if *s as ::core::ffi::c_int == '\0' as i32 {
        return 0 as ::core::ffi::c_int;
    }
    let mut copy = CStr::from_ptr(s).to_bytes_with_nul().to_vec();
    let mut string = copy.as_mut_ptr().cast::<::core::ffi::c_char>();
    loop {
        next = strsep(&raw mut string, separator);
        if next.is_null() {
            break;
        }
        if *next as ::core::ffi::c_int == '\0' as i32 {
            continue;
        }
        i = 0 as u_int;
        while i < UINT_MAX {
            if crate::src::options::options_array_get_index_mut(&mut *(o), i).map_or(std::ptr::null_mut(), |value| value).is_null() {
                break;
            }
            i = i.wrapping_add(1);
        }
        if i == UINT_MAX {
            break;
        }
        xformat(&mut key, format_args!("{}", i as u32));
        if options_array_set(
            o,
            &raw mut key as *mut ::core::ffi::c_char,
            next,
            0 as ::core::ffi::c_int,
            cause,
        ) != 0 as ::core::ffi::c_int
        {
            return -(1 as ::core::ffi::c_int);
        }
    }
    return 0 as ::core::ffi::c_int;
}
pub fn options_array_iter(o: &options_entry) -> impl DoubleEndedIterator<Item = &options_array_item> {
    let entries = match &o.value {
        options_value::Array(array) => Some(&array.entries),
        _ => None,
    };
    entries.into_iter().flat_map(|entries| entries.values().map(Box::as_ref))
}
pub fn options_array_iter_mut(o: &mut options_entry) -> impl DoubleEndedIterator<Item = &mut options_array_item> {
    let entries = match &mut o.value {
        options_value::Array(array) => Some(&mut array.entries),
        _ => None,
    };
    entries.into_iter().flat_map(|entries| entries.values_mut().map(Box::as_mut))
}
pub fn options_array_item_key(a: &options_array_item) -> &CStr {
    &a.key
}
pub fn options_array_item_value(a: &options_array_item) -> &options_value {
    &a.value
}
pub fn options_array_item_value_mut(a: &mut options_array_item) -> &mut options_value {
    &mut a.value
}
pub unsafe fn options_is_array(mut o: *mut options_entry) -> ::core::ffi::c_int {
    return (!(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry).is_null() && (*(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry)).flags & OPTIONS_TABLE_IS_ARRAY != 0)
        as ::core::ffi::c_int;
}
pub unsafe fn options_is_string(mut o: *mut options_entry) -> ::core::ffi::c_int {
    return ((*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry).is_null()
        || (*(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry)).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint)
        as ::core::ffi::c_int;
}
pub(crate) unsafe fn options_to_string(
    o: *mut options_entry,
    key: *const ::core::ffi::c_char,
) -> CString {
    let numeric: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    options_to_cstring(o, key, numeric)
}

pub unsafe fn options_to_cstring(
    o: *mut options_entry,
    key: *const ::core::ffi::c_char,
    numeric: ::core::ffi::c_int,
) -> CString {
    if !(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry).is_null() && (*(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry)).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
        if key.is_null() {
            let mut result = Vec::new();
            let a_root = o;
            let mut a_keys = crate::src::options::options_array_iter(&*a_root).map(|item| item.key.clone()).collect::<Vec<_>>().into_iter();
            let mut a = a_keys.next().map_or(std::ptr::null_mut(), |key| crate::src::options::options_array_item(a_root, key.as_ptr()));
            let mut first = true;
            while !a.is_null() {
                let value = options_value_to_cstring(&*o, &(*a).value, numeric);
                if !first {
                    result.push(b' ');
                }
                result.extend_from_slice(value.as_bytes());
                first = false;
                a = a_keys.next().map_or(std::ptr::null_mut(), |key| crate::src::options::options_array_item(a_root, key.as_ptr()));
            }
            return CString::new(result).expect("option values have no NUL");
        }
        let Some(new_key) = options_array_correct_key(CStr::from_ptr(key)) else {
            return c"".to_owned();
        };
        let a = options_array_item(o, new_key.as_ptr());
        if a.is_null() {
            return c"".to_owned();
        }
        return options_value_to_cstring(&*o, &(*a).value, numeric);
    }
    options_value_to_cstring(&*o, &(*o).value, numeric)
}
/// Name and normalized array key owned through the command's synchronous use.
pub struct OwnedOptionName {
    pub name: CString,
    pub array_key: Option<CString>,
}

pub fn options_parse_owned(input: &CStr) -> Option<OwnedOptionName> {
    let bytes = input.to_bytes();
    let parsed = parse_option_name(bytes).ok()?;
    let array_key = match parsed.array_key {
        Some(ArrayIndex::Numeric(number)) => {
            Some(CString::new(number.to_string()).expect("numeric key has no NUL"))
        }
        Some(ArrayIndex::Text(bytes)) => {
            Some(CString::new(bytes).expect("C string key has no interior NUL"))
        }
        None => None,
    };
    Some(OwnedOptionName {
        name: CString::new(parsed.name).expect("C string option name has no interior NUL"),
        array_key,
    })
}
pub unsafe fn options_search(name: *const ::core::ffi::c_char) -> Option<&'static options_table_entry> {
    let name = CStr::from_ptr(name);
    options_table.iter().find(|entry| entry.name == Some(name))
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionMatchFailure {
    Parse,
    Ambiguous,
    Invalid,
}

pub unsafe fn options_match_owned(s: &CStr) -> Result<OwnedOptionName, OptionMatchFailure> {
    let mut parsed = options_parse_owned(s).ok_or(OptionMatchFailure::Parse)?;

    let mut candidates: [&[u8]; 273] = [&[]; 273];
    let mut entries: [*const options_table_entry; 273] =
        [::core::ptr::null::<options_table_entry>(); 273];
    let mut candidate_count = 0usize;
    let mut oe = &raw const options_table as *const options_table_entry;
    while candidate_count < candidates.len() && !(*oe).name_ptr().is_null() {
        candidates[candidate_count] = std::ffi::CStr::from_ptr((*oe).name_ptr()).to_bytes();
        entries[candidate_count] = oe;
        candidate_count += 1;
        oe = oe.offset(1);
    }

    let mut aliases: [(&[u8], &[u8]); 8] = [(&[], &[]); 8];
    let mut alias_count = 0usize;
    let mut map = &raw const options_other_names as *const options_name_map;
    while alias_count < aliases.len() && !(*map).from.to_bytes().is_empty() {
        aliases[alias_count] = ((*map).from.to_bytes(), (*map).to.to_bytes());
        alias_count += 1;
        map = map.offset(1);
    }

    match match_option_name(
        s.to_bytes(),
        &aliases[..alias_count],
        &candidates[..candidate_count],
    ) {
        Ok(OptionNameMatch::User) => Ok(parsed),
        Ok(OptionNameMatch::BuiltIn(index)) => {
            parsed.name = CStr::from_ptr((*entries[index]).name_ptr()).to_owned();
            Ok(parsed)
        }
        Err(OptionNameMatchError::Ambiguous) => Err(OptionMatchFailure::Ambiguous),
        Err(OptionNameMatchError::Invalid(_) | OptionNameMatchError::NotFound) => {
            Err(OptionMatchFailure::Invalid)
        }
    }
}
pub unsafe fn options_get_string(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    o = options_get(oo, name);
    if o.is_null() {
        fatalx(|out| {
            out.write_all(b"missing option ")?;
            write_cstr(out, name)
        });
    }
    if !((*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry).is_null()
        || (*(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry)).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        fatalx(|out| {
            out.write_all(b"option ")?;
            write_cstr(out, name)?;
            out.write_all(b" is not a string")
        });
    }
    return (*o).value.string_ptr().map_or(std::ptr::null_mut(), |value| value.as_ptr().cast_mut());
}
/// Read a numeric option without borrowing the option tree mutably.
///
/// The caller must ensure that the raw parent chain remains valid.
pub unsafe fn options_get_number_ref(mut oo: &options, name: &CStr) -> ::core::ffi::c_longlong {
    let entry = loop {
        if let Some(entry) = options_get_only(oo, name) {
            break entry;
        }
        match oo.parent.as_ref() {
            Some(parent) => oo = parent,
            None => fatalx(|out| {
                out.write_all(b"missing option ")?;
                write_cstr(out, name.as_ptr())
            }),
        }
    };
    if !entry.tableentry_ptr().is_some_and(|table| matches!(table.type_0,
        OPTIONS_TABLE_NUMBER | OPTIONS_TABLE_KEY | OPTIONS_TABLE_COLOUR |
        OPTIONS_TABLE_FLAG | OPTIONS_TABLE_CHOICE))
    {
        fatalx(|out| {
            out.write_all(b"option ")?;
            write_cstr(out, name.as_ptr())?;
            out.write_all(b" is not a number")
        });
    }
    entry.value.number()
}

pub unsafe fn options_get_number(
    oo: *mut options,
    name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_longlong {
    options_get_number_ref(&*oo, CStr::from_ptr(name))
}
pub unsafe fn options_get_command(mut oo: *mut options) -> std::rc::Rc<std::cell::RefCell<cmd_list>> {
    let mut name: *const ::core::ffi::c_char =
        b"default-client-command\0" as *const u8 as *const ::core::ffi::c_char;
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    o = options_get(oo, name);
    if o.is_null() {
        fatalx(|out| {
            out.write_all(b"missing option ")?;
            write_cstr(out, name)
        });
    }
    if !(!(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry).is_null()
        && (*(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry)).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_COMMAND as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        fatalx(|out| {
            out.write_all(b"option ")?;
            write_cstr(out, name)?;
            out.write_all(b" is not a command")
        });
    }
    return (*o).value.commands().expect("default client command").clone();
}
pub unsafe fn options_set_string(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
    mut append: ::core::ffi::c_int,
    write: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
) -> *mut options_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut separator: *const ::core::ffi::c_char =
        b"\0" as *const u8 as *const ::core::ffi::c_char;
    let formatted = format_message_with(write);
    o = crate::src::options::options_get_only_mut(&mut *(oo), std::ffi::CStr::from_ptr(name)).map_or(std::ptr::null_mut(), |entry| entry);
    let value = if !o.is_null()
        && append != 0
        && ((*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry).is_null()
            || (*(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry)).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        if *name as ::core::ffi::c_int != '@' as i32 {
            separator = (*(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry)).separator_ptr();
            if separator.is_null() {
                separator = b"\0" as *const u8 as *const ::core::ffi::c_char;
            }
        }
        // glibc printf renders a null %s argument as "(null)". An entry
        // created by options_empty can reach this append path with no value.
        let previous = if (*o).value.string_ptr().map_or(std::ptr::null_mut(), |value| value.as_ptr().cast_mut()).is_null() {
            b"(null)".as_slice()
        } else {
            CStr::from_ptr((*o).value.string_ptr().map_or(std::ptr::null_mut(), |value| value.as_ptr().cast_mut())).to_bytes()
        };
        let separator = CStr::from_ptr(separator).to_bytes();
        let mut bytes =
            Vec::with_capacity(previous.len() + separator.len() + formatted.as_bytes().len());
        bytes.extend_from_slice(previous);
        bytes.extend_from_slice(separator);
        bytes.extend_from_slice(formatted.as_bytes());
        CString::new(bytes).expect("C-string fragments contain no NUL")
    } else {
        formatted
    };
    if o.is_null() && *name as ::core::ffi::c_int == '@' as i32 {
        o = options_add(oo, name);
    } else if o.is_null() {
        o = options_default(oo, options_parent_table_entry(oo, name));
        if o.is_null() {
            return ::core::ptr::null_mut::<options_entry>();
        }
    }
    if !((*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry).is_null()
        || (*(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry)).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        fatalx(|out| {
            out.write_all(b"option ")?;
            write_cstr(out, name)?;
            out.write_all(b" is not a string")
        });
    }
    set_scalar_string(&mut *o, value);
    (*o).cached = 0 as ::core::ffi::c_int;
    return o;
}
pub unsafe fn options_set_number(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
    mut value: ::core::ffi::c_longlong,
) -> *mut options_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    if *name as ::core::ffi::c_int == '@' as i32 {
        fatalx(|out| {
            out.write_all(b"user option ")?;
            write_cstr(out, name)?;
            out.write_all(b" must be a string")
        });
    }
    o = crate::src::options::options_get_only_mut(&mut *(oo), std::ffi::CStr::from_ptr(name)).map_or(std::ptr::null_mut(), |entry| entry);
    if o.is_null() {
        o = options_default(oo, options_parent_table_entry(oo, name));
        if o.is_null() {
            return ::core::ptr::null_mut::<options_entry>();
        }
    }
    if !(!(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry).is_null()
        && ((*(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry)).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry)).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry)).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_COLOUR as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry)).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_FLAG as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry)).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_CHOICE as ::core::ffi::c_int as ::core::ffi::c_uint))
    {
        fatalx(|out| {
            out.write_all(b"option ")?;
            write_cstr(out, name)?;
            out.write_all(b" is not a number")
        });
    }
    (*o).value = options_value::Number(value);
    return o;
}
pub unsafe fn options_set_command(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
    value: Option<std::rc::Rc<std::cell::RefCell<cmd_list>>>,
) -> *mut options_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    if *name as ::core::ffi::c_int == '@' as i32 {
        fatalx(|out| {
            out.write_all(b"user option ")?;
            write_cstr(out, name)?;
            out.write_all(b" must be a string")
        });
    }
    o = crate::src::options::options_get_only_mut(&mut *(oo), std::ffi::CStr::from_ptr(name)).map_or(std::ptr::null_mut(), |entry| entry);
    if o.is_null() {
        o = options_default(oo, options_parent_table_entry(oo, name));
        if o.is_null() {
            return ::core::ptr::null_mut::<options_entry>();
        }
    }
    if !(!(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry).is_null()
        && (*(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry)).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_COMMAND as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        fatalx(|out| {
            out.write_all(b"option ")?;
            write_cstr(out, name)?;
            out.write_all(b" is not a command")
        });
    }
    (*o).value = options_value::Command(OptionCommand(value));
    return o;
}
pub unsafe fn options_scope_from_name(
    mut args: *mut args,
    mut window: ::core::ffi::c_int,
    mut name: *const ::core::ffi::c_char,
    mut fs: *mut cmd_find_state,
    mut oo: *mut *mut options,
    mut cause: *mut Option<CString>,
) -> ::core::ffi::c_int {
    let mut s: *mut session = (*fs).s_ptr();
    let mut wl: *mut winlink = (*fs).wl_ptr();
    let mut wp: *mut window_pane = (*fs).wp_ptr();
    let mut target: *const ::core::ffi::c_char = args_get(&*(args), 't' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    let mut scope: ::core::ffi::c_int = OPTIONS_TABLE_NONE;
    if *name as ::core::ffi::c_int == '@' as i32 {
        return options_scope_from_flags(args, window, fs, oo, cause);
    }
    oe = &raw const options_table as *const options_table_entry;
    while !(*oe).name_ptr().is_null() {
        if strcmp((*oe).name_ptr(), name) == 0 as ::core::ffi::c_int {
            break;
        }
        oe = oe.offset(1);
    }
    if (*oe).name_ptr().is_null() {
        format_options_cause!(
            cause,
            b"unknown option: %s\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
        return 0 as ::core::ffi::c_int;
    }
    let mut current_block_38: u64;
    match (*oe).scope {
        OPTIONS_TABLE_SERVER => {
            *oo = global_options;
            scope = OPTIONS_TABLE_SERVER;
            current_block_38 = 980989089337379490;
        }
        OPTIONS_TABLE_SESSION => {
            if args_has(args, 'g' as i32 as u_char) != 0 {
                *oo = global_s_options;
                scope = OPTIONS_TABLE_SESSION;
            } else if s.is_null() && !target.is_null() {
                format_options_cause!(
                    cause,
                    b"no such session: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    target,
                );
            } else if s.is_null() {
                format_options_cause!(
                    cause,
                    b"no current session\0" as *const u8 as *const ::core::ffi::c_char,
                );
            } else {
                *oo = options_owner_ptr(&mut (*s).options).map_or(std::ptr::null_mut(), |options| options);
                scope = OPTIONS_TABLE_SESSION;
            }
            current_block_38 = 980989089337379490;
        }
        12 => {
            if args_has(args, 'p' as i32 as u_char) != 0 {
                if wp.is_null() && !target.is_null() {
                    format_options_cause!(
                        cause,
                        b"no such pane: %s\0" as *const u8 as *const ::core::ffi::c_char,
                        target,
                    );
                } else if wp.is_null() {
                    format_options_cause!(
                        cause,
                        b"no current pane\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                } else {
                    *oo = options_owner_ptr(&mut (*wp).options).map_or(std::ptr::null_mut(), |options| options);
                    scope = OPTIONS_TABLE_PANE;
                }
                current_block_38 = 980989089337379490;
            } else {
                current_block_38 = 7230205663690532434;
            }
        }
        OPTIONS_TABLE_WINDOW => {
            current_block_38 = 7230205663690532434;
        }
        _ => {
            current_block_38 = 980989089337379490;
        }
    }
    match current_block_38 {
        7230205663690532434 => {
            if args_has(args, 'g' as i32 as u_char) != 0 {
                *oo = global_w_options;
                scope = OPTIONS_TABLE_WINDOW;
            } else if wl.is_null() && !target.is_null() {
                format_options_cause!(
                    cause,
                    b"no such window: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    target,
                );
            } else if wl.is_null() {
                format_options_cause!(
                    cause,
                    b"no current window\0" as *const u8 as *const ::core::ffi::c_char,
                );
            } else {
                *oo = options_owner_ptr(&mut (*(*wl).window_ptr()).options).map_or(std::ptr::null_mut(), |options| options);
                scope = OPTIONS_TABLE_WINDOW;
            }
        }
        _ => {}
    }
    return scope;
}
pub unsafe fn options_scope_from_flags(
    mut args: *mut args,
    mut window: ::core::ffi::c_int,
    mut fs: *mut cmd_find_state,
    mut oo: *mut *mut options,
    mut cause: *mut Option<CString>,
) -> ::core::ffi::c_int {
    let mut s: *mut session = (*fs).s_ptr();
    let mut wl: *mut winlink = (*fs).wl_ptr();
    let mut wp: *mut window_pane = (*fs).wp_ptr();
    let mut target: *const ::core::ffi::c_char = args_get(&*(args), 't' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
    if args_has(args, 's' as i32 as u_char) != 0 {
        *oo = global_options;
        return 0x1 as ::core::ffi::c_int;
    }
    if args_has(args, 'p' as i32 as u_char) != 0 {
        if wp.is_null() {
            if !target.is_null() {
                format_options_cause!(
                    cause,
                    b"no such pane: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    target,
                );
            } else {
                format_options_cause!(
                    cause,
                    b"no current pane\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            return 0 as ::core::ffi::c_int;
        }
        *oo = options_owner_ptr(&mut (*wp).options).map_or(std::ptr::null_mut(), |options| options);
        return 0x8 as ::core::ffi::c_int;
    } else if window != 0 || args_has(args, 'w' as i32 as u_char) != 0 {
        if args_has(args, 'g' as i32 as u_char) != 0 {
            *oo = global_w_options;
            return 0x4 as ::core::ffi::c_int;
        }
        if wl.is_null() {
            if !target.is_null() {
                format_options_cause!(
                    cause,
                    b"no such window: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    target,
                );
            } else {
                format_options_cause!(
                    cause,
                    b"no current window\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            return 0 as ::core::ffi::c_int;
        }
        *oo = options_owner_ptr(&mut (*(*wl).window_ptr()).options).map_or(std::ptr::null_mut(), |options| options);
        return 0x4 as ::core::ffi::c_int;
    } else {
        if args_has(args, 'g' as i32 as u_char) != 0 {
            *oo = global_s_options;
            return 0x2 as ::core::ffi::c_int;
        }
        if s.is_null() {
            if !target.is_null() {
                format_options_cause!(
                    cause,
                    b"no such session: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    target,
                );
            } else {
                format_options_cause!(
                    cause,
                    b"no current session\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            return 0 as ::core::ffi::c_int;
        }
        *oo = options_owner_ptr(&mut (*s).options).map_or(std::ptr::null_mut(), |options| options);
        return 0x2 as ::core::ffi::c_int;
    };
}
pub unsafe fn options_string_to_style(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
    mut ft: *mut format_tree,
) -> *mut style {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    let mut dgc: *const grid_cell = &raw const grid_default_cell;
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut failed: ::core::ffi::c_int = 0;
    o = options_get(oo, name);
    if o.is_null()
        || !((*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry).is_null()
            || (*(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry)).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        return ::core::ptr::null_mut::<style>();
    }
    if (*o).cached != 0 {
        return &raw mut (*o).style;
    }
    s = (*o).value.string_ptr().map_or(std::ptr::null_mut(), |value| value.as_ptr().cast_mut());
    oe = (*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry);
    log_debug(format_args!(
        "{}: {} is '{}'",
        "options_string_to_style",
        log_cstr((name) as *const _),
        log_cstr((s) as *const _)
    ));
    style_set(&raw mut (*o).style, dgc);
    (*o).cached = (strstr(s, b"#{\0" as *const u8 as *const ::core::ffi::c_char)
        == NULL as *mut ::core::ffi::c_char) as ::core::ffi::c_int;
    if !ft.is_null() && (*o).cached == 0 {
        let expanded = format_expand_cstring(ft, s);
        if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_COLOUR != 0 {
            failed = style_parse_colour(&raw mut (*o).style, dgc, expanded.as_ptr());
        } else {
            failed = style_parse(&raw mut (*o).style, dgc, expanded.as_ptr());
        }
        if failed != 0 as ::core::ffi::c_int {
            return ::core::ptr::null_mut::<style>();
        }
    } else {
        if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_COLOUR != 0 {
            failed = style_parse_colour(&raw mut (*o).style, dgc, s);
        } else {
            failed = style_parse(&raw mut (*o).style, dgc, s);
        }
        if failed != 0 as ::core::ffi::c_int {
            return ::core::ptr::null_mut::<style>();
        }
    }
    return &raw mut (*o).style;
}
unsafe fn options_from_string_check(
    mut oe: *const options_table_entry,
    mut value: *const ::core::ffi::c_char,
    mut cause: *mut Option<CString>,
) -> ::core::ffi::c_int {
    let mut sy: style = style {
        gc: grid_cell {
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
        },
        ignore: 0,
        dim: 0,
        fill: 0,
        align: STYLE_ALIGN_DEFAULT,
        list: STYLE_LIST_OFF,
        range_type: STYLE_RANGE_NONE,
        range_argument: 0,
        range_string: [0; 16],
        width: 0,
        width_percentage: 0,
        pad: 0,
        default_type: STYLE_DEFAULT_BASE,
        link: 0,
    };
    if oe.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if strcmp(
        (*oe).name_ptr(),
        b"default-shell\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        && checkshell(value) == 0
    {
        format_options_cause!(
            cause,
            b"not a suitable shell: %s\0" as *const u8 as *const ::core::ffi::c_char,
            value,
        );
        return -(1 as ::core::ffi::c_int);
    }
    if !(*oe).pattern_ptr().is_null()
        && fnmatch((*oe).pattern_ptr(), value, 0 as ::core::ffi::c_int) != 0 as ::core::ffi::c_int
    {
        format_options_cause!(
            cause,
            b"value is invalid: %s\0" as *const u8 as *const ::core::ffi::c_char,
            value,
        );
        return -(1 as ::core::ffi::c_int);
    }
    if (*oe).flags & OPTIONS_TABLE_IS_STYLE != 0
        && strstr(value, b"#{\0" as *const u8 as *const ::core::ffi::c_char).is_null()
        && style_parse(&raw mut sy, &raw const grid_default_cell, value) != 0 as ::core::ffi::c_int
    {
        format_options_cause!(
            cause,
            b"invalid style: %s\0" as *const u8 as *const ::core::ffi::c_char,
            value,
        );
        return -(1 as ::core::ffi::c_int);
    }
    if (*oe).flags & OPTIONS_TABLE_IS_COLOUR != 0
        && strstr(value, b"#{\0" as *const u8 as *const ::core::ffi::c_char).is_null()
        && style_parse_colour(&raw mut sy, &raw const grid_default_cell, value)
            != 0 as ::core::ffi::c_int
    {
        format_options_cause!(
            cause,
            b"invalid colour: %s\0" as *const u8 as *const ::core::ffi::c_char,
            value,
        );
        return -(1 as ::core::ffi::c_int);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe fn options_from_string_flag(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
    mut value: *const ::core::ffi::c_char,
    mut cause: *mut Option<CString>,
) -> ::core::ffi::c_int {
    let mut flag: ::core::ffi::c_int = 0;
    if value.is_null() || *value as ::core::ffi::c_int == '\0' as i32 {
        flag = (options_get_number(oo, name) == 0) as ::core::ffi::c_int;
    } else if strcmp(value, b"1\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
        || strcasecmp(value, b"on\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        || strcasecmp(value, b"yes\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
    {
        flag = 1 as ::core::ffi::c_int;
    } else if strcmp(value, b"0\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
        || strcasecmp(value, b"off\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        || strcasecmp(value, b"no\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
    {
        flag = 0 as ::core::ffi::c_int;
    } else {
        format_options_cause!(
            cause,
            b"bad value: %s\0" as *const u8 as *const ::core::ffi::c_char,
            value,
        );
        return -(1 as ::core::ffi::c_int);
    }
    options_set_number(oo, name, flag as ::core::ffi::c_longlong);
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn options_find_choice(
    mut oe: *const options_table_entry,
    mut value: *const ::core::ffi::c_char,
    mut cause: *mut Option<CString>,
) -> ::core::ffi::c_int {
    let mut choice: ::core::ffi::c_int = -1;
    for (n, candidate) in (*oe).choices.iter().enumerate() {
        if *candidate == CStr::from_ptr(value) {
            choice = n as ::core::ffi::c_int;
        }
    }
    if choice == -(1 as ::core::ffi::c_int) {
        format_options_cause!(
            cause,
            b"unknown value: %s\0" as *const u8 as *const ::core::ffi::c_char,
            value,
        );
        return -(1 as ::core::ffi::c_int);
    }
    return choice;
}
unsafe fn options_from_string_choice(
    mut oe: *const options_table_entry,
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
    mut value: *const ::core::ffi::c_char,
    mut cause: *mut Option<CString>,
) -> ::core::ffi::c_int {
    let mut choice: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    if value.is_null() {
        choice = options_get_number(oo, name) as ::core::ffi::c_int;
        if choice < 2 as ::core::ffi::c_int {
            choice = (choice == 0) as ::core::ffi::c_int;
        }
    } else {
        choice = options_find_choice(oe, value, cause);
        if choice < 0 as ::core::ffi::c_int {
            return -(1 as ::core::ffi::c_int);
        }
    }
    options_set_number(oo, name, choice as ::core::ffi::c_longlong);
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn options_from_string(
    mut oo: *mut options,
    mut oe: *const options_table_entry,
    mut name: *const ::core::ffi::c_char,
    mut value: *const ::core::ffi::c_char,
    mut append: ::core::ffi::c_int,
    mut cause: *mut Option<CString>,
) -> ::core::ffi::c_int {
    let mut type_0: options_table_type = OPTIONS_TABLE_STRING;
    let mut number: ::core::ffi::c_longlong = 0;
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut new: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut key: key_code = 0;
    let mut pr: cmd_parse_result = cmd_parse_result::empty();
    if !oe.is_null() {
        if value.is_null()
            && (*oe).type_0 as ::core::ffi::c_uint
                != OPTIONS_TABLE_FLAG as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*oe).type_0 as ::core::ffi::c_uint
                != OPTIONS_TABLE_CHOICE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            format_options_cause!(
                cause,
                b"empty value\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return -(1 as ::core::ffi::c_int);
        }
        type_0 = (*oe).type_0;
    } else {
        if *name as ::core::ffi::c_int != '@' as i32 {
            format_options_cause!(
                cause,
                b"bad option name\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return -(1 as ::core::ffi::c_int);
        }
        type_0 = OPTIONS_TABLE_STRING;
    }
    match type_0 as ::core::ffi::c_uint {
        0 => {
            // Snapshot before options_set_string replaces the scalar owner.
            let old = CStr::from_ptr(options_get_string(oo, name)).to_owned();
            options_set_string(oo, name, append, |out| write_cstr(out, value));
            new = options_get_string(oo, name);
            if options_from_string_check(oe, new, cause) != 0 as ::core::ffi::c_int {
                options_set_string(oo, name, 0 as ::core::ffi::c_int, |out| {
                    write_cstr(out, old.as_ptr())
                });
                return -(1 as ::core::ffi::c_int);
            }
            return 0 as ::core::ffi::c_int;
        }
        1 => {
            number = strtonum(
                value,
                (*oe).minimum as ::core::ffi::c_longlong,
                (*oe).maximum as ::core::ffi::c_longlong,
                &raw mut errstr,
            );
            if !errstr.is_null() {
                format_options_cause!(
                    cause,
                    b"value is %s: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    errstr,
                    value,
                );
                return -(1 as ::core::ffi::c_int);
            }
            options_set_number(oo, name, number);
            return 0 as ::core::ffi::c_int;
        }
        2 => {
            key = key_string_parse_cstr(std::ffi::CStr::from_ptr(value)).unwrap_or(KEYC_UNKNOWN);
            if key == KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code {
                format_options_cause!(
                    cause,
                    b"bad key: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    value,
                );
                return -(1 as ::core::ffi::c_int);
            }
            options_set_number(oo, name, key as ::core::ffi::c_longlong);
            return 0 as ::core::ffi::c_int;
        }
        3 => {
            number = colour_parse_cstr(std::ffi::CStr::from_ptr(value)).unwrap_or(-1)
                as ::core::ffi::c_longlong;
            if number == -(1 as ::core::ffi::c_int) as ::core::ffi::c_longlong {
                format_options_cause!(
                    cause,
                    b"bad colour: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    value,
                );
                return -(1 as ::core::ffi::c_int);
            }
            options_set_number(oo, name, number);
            return 0 as ::core::ffi::c_int;
        }
        4 => return options_from_string_flag(oo, name, value, cause),
        5 => return options_from_string_choice(oe, oo, name, value, cause),
        6 => {
            pr = cmd_parse_from_string(
                CStr::from_ptr(value),
                ::core::ptr::null_mut::<cmd_parse_input>(),
            );
            match pr.status as ::core::ffi::c_uint {
                0 => {
                    if !cause.is_null() {
                        *cause = pr.error;
                    }
                    return -(1 as ::core::ffi::c_int);
                }
                1 => {
                    options_set_command(oo, name, pr.take_cmdlist());
                    return 0 as ::core::ffi::c_int;
                }
                _ => {}
            }
        }
        _ => {}
    }
    return -(1 as ::core::ffi::c_int);
}
pub unsafe fn options_push_changes(mut name: *const ::core::ffi::c_char) {
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    log_debug(format_args!(
        "{}: {}",
        "options_push_changes",
        log_cstr((name) as *const _)
    ));
    if strcmp(name, b"theme\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
        || strncmp(
            name,
            b"dark-theme-\0" as *const u8 as *const ::core::ffi::c_char,
            11 as size_t,
        ) == 0 as ::core::ffi::c_int
        || strncmp(
            name,
            b"light-theme-\0" as *const u8 as *const ::core::ffi::c_char,
            12 as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        let mut registry_loop_0_owner = clients.first();
        loop_0 = registry_loop_0_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        while !loop_0.is_null() {
            server_client_update_theme_colours(loop_0);
            if (*loop_0).tty.flags & TTY_OPENED != 0 {
                tty_invalidate(&raw mut (*loop_0).tty);
            }
            server_redraw_client(&mut *(loop_0));
            registry_loop_0_owner = clients.next(registry_loop_0_owner.as_ref().expect("current registry client"));
            loop_0 = registry_loop_0_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        }
    }
    if strcmp(
        name,
        b"automatic-rename\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        let mut window_cursor = windows_minmax(&*std::ptr::addr_of!(windows));
        while let Some(window_owner) = window_cursor.take() {
            w = window_owner.as_ptr();
            if !(*w).active_ptr().is_null() {
                if options_get_number(options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options), name) != 0 {
                    (*(*w).active_ptr()).flags |= PANE_CHANGED;
                }
            }
            window_cursor = windows_next(&*w);
        }
    }
    if strcmp(
        name,
        b"cursor-colour\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        let mut indexed_pane_owner = window_pane_tree_minmax(&*std::ptr::addr_of!(all_window_panes));
        wp = indexed_pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        while !wp.is_null() {
            window_pane_default_cursor(wp);
            indexed_pane_owner = window_pane_tree_next(&*wp);
            wp = indexed_pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        }
    }
    if strcmp(
        name,
        b"cursor-style\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        let mut indexed_pane_owner = window_pane_tree_minmax(&*std::ptr::addr_of!(all_window_panes));
        wp = indexed_pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        while !wp.is_null() {
            window_pane_default_cursor(wp);
            indexed_pane_owner = window_pane_tree_next(&*wp);
            wp = indexed_pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        }
    }
    if strcmp(
        name,
        b"fill-character\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        let mut window_cursor = windows_minmax(&*std::ptr::addr_of!(windows));
        while let Some(window_owner) = window_cursor.take() {
            w = window_owner.as_ptr();
            window_set_fill_cells(w);
            window_cursor = windows_next(&*w);
        }
    }
    if strcmp(
        name,
        b"key-table\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        let mut registry_loop_0_owner = clients.first();
        loop_0 = registry_loop_0_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        while !loop_0.is_null() {
            server_client_set_key_table(loop_0, ::core::ptr::null::<::core::ffi::c_char>());
            registry_loop_0_owner = clients.next(registry_loop_0_owner.as_ref().expect("current registry client"));
            loop_0 = registry_loop_0_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        }
    }
    if strcmp(
        name,
        b"user-keys\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        let mut registry_loop_0_owner = clients.first();
        loop_0 = registry_loop_0_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        while !loop_0.is_null() {
            if (*loop_0).tty.flags & TTY_OPENED != 0 {
                tty_keys_build(&raw mut (*loop_0).tty);
            }
            registry_loop_0_owner = clients.next(registry_loop_0_owner.as_ref().expect("current registry client"));
            loop_0 = registry_loop_0_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        }
    }
    if strcmp(name, b"status\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"status-interval\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        status_timer_start_all();
    }
    if strcmp(name, b"status\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"status-position\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"pane-border-indicators\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"pane-border-lines\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"pane-border-status\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"pane-scrollbars\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"pane-scrollbars-timeout\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"pane-scrollbars-position\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"pane-scrollbars-style\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        redraw_invalidate_all_scenes();
    }
    if strcmp(
        name,
        b"monitor-silence\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        alerts_reset_all();
    }
    if strcmp(
        name,
        b"window-style\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"window-active-style\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        let mut indexed_pane_owner = window_pane_tree_minmax(&*std::ptr::addr_of!(all_window_panes));
        wp = indexed_pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        while !wp.is_null() {
            (*wp).flags |= PANE_STYLECHANGED | PANE_THEMECHANGED;
            indexed_pane_owner = window_pane_tree_next(&*wp);
            wp = indexed_pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        }
    }
    if *name as ::core::ffi::c_int == '@' as i32 {
        let mut indexed_pane_owner = window_pane_tree_minmax(&*std::ptr::addr_of!(all_window_panes));
        wp = indexed_pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        while !wp.is_null() {
            (*wp).flags |= PANE_STYLECHANGED;
            indexed_pane_owner = window_pane_tree_next(&*wp);
            wp = indexed_pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        }
    }
    if strcmp(
        name,
        b"pane-colours\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        let mut indexed_pane_owner = window_pane_tree_minmax(&*std::ptr::addr_of!(all_window_panes));
        wp = indexed_pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        while !wp.is_null() {
            colour_palette_from_option(Some(&mut (*wp).palette), options_owner_ptr(&mut (*wp).options).map_or(std::ptr::null_mut(), |options| options));
            indexed_pane_owner = window_pane_tree_next(&*wp);
            wp = indexed_pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        }
    }
    if strcmp(
        name,
        b"pane-border-status\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"pane-scrollbars\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"pane-scrollbars-position\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        let mut window_cursor = windows_minmax(&*std::ptr::addr_of!(windows));
        while let Some(window_owner) = window_cursor.take() {
            w = window_owner.as_ptr();
            (*w).sb = options_get_number(
                options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
                b"pane-scrollbars\0" as *const u8 as *const ::core::ffi::c_char,
            ) as ::core::ffi::c_int;
            (*w).sb_pos = options_get_number(
                options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
                b"pane-scrollbars-position\0" as *const u8 as *const ::core::ffi::c_char,
            ) as ::core::ffi::c_int;
            layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
            window_cursor = windows_next(&*w);
        }
    }
    if strcmp(
        name,
        b"pane-scrollbars\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        let mut indexed_pane_owner = window_pane_tree_minmax(&*std::ptr::addr_of!(all_window_panes));
        wp = indexed_pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        while !wp.is_null() {
            window_pane_scrollbar_hide(indexed_pane_owner.as_ref().expect("indexed pane"));
            indexed_pane_owner = window_pane_tree_next(&*wp);
            wp = indexed_pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        }
    }
    if strcmp(
        name,
        b"pane-scrollbars-style\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        let mut indexed_pane_owner = window_pane_tree_minmax(&*std::ptr::addr_of!(all_window_panes));
        wp = indexed_pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        while !wp.is_null() {
            style_set_scrollbar_style_from_option(&raw mut (*wp).scrollbar_style, options_owner_ptr(&mut (*wp).options).map_or(std::ptr::null_mut(), |options| options));
            indexed_pane_owner = window_pane_tree_next(&*wp);
            wp = indexed_pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        }
        let mut window_cursor = windows_minmax(&*std::ptr::addr_of!(windows));
        while let Some(window_owner) = window_cursor.take() {
            w = window_owner.as_ptr();
            layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
            window_cursor = windows_next(&*w);
        }
    }
    if strcmp(
        name,
        b"codepoint-widths\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        utf8_update_width_cache();
    }
    if strcmp(
        name,
        b"input-buffer-size\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        input_set_buffer_size(options_get_number(global_options, name) as size_t);
    }
    if strcmp(
        name,
        b"history-limit\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        let mut s_owner = sessions_minmax(&*std::ptr::addr_of!(sessions));
        s = s_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
        while !s.is_null() {
            session_update_history(&*s);
            s_owner = sessions_next(&*s);
            s = s_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
        }
    }
    let mut s_owner = sessions_minmax(&*std::ptr::addr_of!(sessions));
    s = s_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    while !s.is_null() {
        status_update_cache(s);
        s_owner = sessions_next(&*s);
        s = s_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    }
    recalculate_sizes();
    let mut registry_loop_0_owner = clients.first();
    loop_0 = registry_loop_0_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    while !loop_0.is_null() {
        if !(*loop_0).session_ptr().is_null() {
            server_redraw_client(&mut *(loop_0));
        }
        registry_loop_0_owner = clients.next(registry_loop_0_owner.as_ref().expect("current registry client"));
        loop_0 = registry_loop_0_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    }
}
pub unsafe fn options_remove_or_default(
    mut o: *mut options_entry,
    mut key: *const ::core::ffi::c_char,
    mut cause: *mut Option<CString>,
) -> ::core::ffi::c_int {
    let mut oo: *mut options = (*o).owner;
    if key.is_null() {
        if !(*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry).is_null()
            && (oo == global_options || oo == global_s_options || oo == global_w_options)
        {
            options_default(oo, (*o).tableentry_ptr().map_or(std::ptr::null(), |entry| entry as *const crate::src::shared::options::options_table_entry));
        } else {
            options_remove(o);
        }
    } else if options_array_set(
        o,
        key,
        ::core::ptr::null::<::core::ffi::c_char>(),
        0 as ::core::ffi::c_int,
        cause,
    ) != 0 as ::core::ffi::c_int
    {
        return -(1 as ::core::ffi::c_int);
    }
    return 0 as ::core::ffi::c_int;
}

#[cfg(test)]
mod array_string_owner_tests {
    use super::*;

    #[test]
    fn formatted_diagnostic_preserves_non_utf8_arguments() {
        unsafe {
            let message =
                options_string_cause(c"value is %s: %s", &[c"invalid".as_ptr(), c"\xff".as_ptr()]);
            assert_eq!(message.to_bytes(), b"value is invalid: \xff");
        }
    }


}
