use crate::src::alerts::alerts_reset_all;
use crate::src::arguments::{args_get, args_has};
use crate::src::cmd::{cmd_list_free, cmd_list_print_cstring};
use crate::src::cmd_parse::cmd_parse_from_string;
use crate::src::colour::{colour_format, colour_palette_from_option, colour_parse_cstr};
use crate::src::compat::strtonum::strtonum;
use crate::src::ffi::libc::{fnmatch, free, strcasecmp, strcmp, strncmp, strsep, strstr};
use crate::src::format::format_expand;
use crate::src::grid::grid_default_cell;
use crate::src::hooks::hooks_monitor_free;
use crate::src::input::input_set_buffer_size;
use crate::src::key_string::{key_string_format, key_string_parse_cstr};
use crate::src::layout::layout_fix_panes;
use crate::src::log::{fatalx, log_debug};
pub use crate::src::options_parse::{
    match_option_name, parse_array_index, parse_option_name, ArrayIndex, ArrayIndexError,
    OptionNameError, OptionNameMatch, OptionNameMatchError, ParsedOptionName,
};
use crate::src::options_table::{options_other_names, options_table};
use crate::src::resize::recalculate_sizes;
use crate::src::screen_redraw::redraw_invalidate_all_scenes;
pub use crate::src::server::clients;
use crate::src::server::current_time;
use crate::src::server_client::{server_client_set_key_table, server_client_update_theme_colours};
use crate::src::server_fn::server_redraw_client;
pub use crate::src::session::sessions;
use crate::src::session::{session_update_history, sessions_minmax, sessions_next};
pub use crate::src::shared::options::options_name_map;
pub use crate::src::shared::pane::window_pane_tree;
use crate::src::status::{status_timer_start_all, status_update_cache};
use crate::src::style::{
    style_parse, style_parse_colour, style_set, style_set_scrollbar_style_from_option,
};
use crate::src::tmux::{checkshell, global_options, global_s_options, global_w_options};
use crate::src::tty::tty_invalidate;
use crate::src::tty_keys::tty_keys_build;
use crate::src::utf8::utf8_update_width_cache;
pub use crate::src::window::windows;
use crate::src::window::{
    all_window_panes, window_pane_default_cursor, window_pane_scrollbar_hide,
    window_pane_tree_minmax, window_pane_tree_next, windows_minmax, windows_next,
};
use crate::src::window_border::window_set_fill_cells;
use crate::src::xmalloc::{xasprintf, xsnprintf, xstrdup, xvasprintf_cstring};
use std::ffi::{CStr, CString};

use crate::src::shared::abi::*;
pub use crate::src::shared::arguments::args;
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
use crate::src::shared::command::*;
pub use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds};
pub use crate::src::shared::command::{cmd_parse_input, cmd_parse_result};
pub use crate::src::shared::control::control_state;
pub use crate::src::shared::ctype::{
    _ISalnum, _ISalpha, _ISblank, _IScntrl, _ISdigit, _ISgraph, _ISlower, _ISprint, _ISpunct,
    _ISspace, _ISupper, _ISxdigit, ctype_code,
};
use crate::src::shared::display::*;
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::environment::environ;
use crate::src::shared::event::*;
pub use crate::src::shared::format::{format_job_tree, format_tree};
use crate::src::shared::grid::*;
pub use crate::src::shared::hyperlinks::hyperlinks;
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
use crate::src::shared::key::*;
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::layout_geometry;
use crate::src::shared::layout::*;
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::limits::{__INT_MAX__, UINT_MAX};
pub use crate::src::shared::menu::menu_data;
use crate::src::shared::message::*;
pub use crate::src::shared::mouse::mouse_event;
use crate::src::shared::options::*;
pub use crate::src::shared::options::{
    options, options_array, options_array_item, options_array_storage, options_entry,
    options_storage, options_table_entry, options_value, OptionsArrayKey,
};
pub use crate::src::shared::options::{
    OPTIONS_TABLE_IS_ARRAY, OPTIONS_TABLE_IS_COLOUR, OPTIONS_TABLE_IS_STYLE, OPTIONS_TABLE_NONE,
    OPTIONS_TABLE_PANE, OPTIONS_TABLE_SERVER, OPTIONS_TABLE_SESSION, OPTIONS_TABLE_WINDOW,
};
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
    PANE_CHANGED, PANE_STYLECHANGED, PANE_THEMECHANGED,
};
pub use crate::src::shared::process::tmuxpeer;
pub use crate::src::shared::prompt::prompt;
pub use crate::src::shared::redraw::redraw_scene;
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::screen_write_cline;
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::spawn_editor_state;
pub use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
pub use crate::src::shared::tree::{RB_BLACK, RB_NEGINF, RB_RED};
pub use crate::src::shared::tty::TTY_OPENED;
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::variadic::{__builtin_va_list, __va_list_tag, va_list};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_14;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_13;

// The first field keeps pointers returned by the options API ABI-compatible.
// Its name and scalar string pointers borrow the adjacent owners until removal
// or replacement, respectively.
#[repr(C)]
struct OwnedOptionEntry {
    record: options_entry,
    name: CString,
    string: Option<CString>,
}

const _: () = assert!(std::mem::offset_of!(OwnedOptionEntry, record) == 0);

// The public item key and optional string value borrow these CStrings until
// removal or value replacement.
#[repr(C)]
struct OwnedOptionArrayItem {
    record: options_array_item,
    key: CString,
    string: Option<CString>,
}

const _: () = assert!(std::mem::offset_of!(OwnedOptionArrayItem, record) == 0);

unsafe fn owned_option(o: *mut options_entry) -> *mut OwnedOptionEntry {
    o.cast()
}

unsafe fn set_scalar_string(o: *mut options_entry, value: CString) {
    let owned = &mut *owned_option(o);
    // Formatting has completed, so callers may have supplied the old value
    // as a %s argument. Replace its owner before publishing the new pointer.
    owned.string = Some(value);
    owned.record.value.string = owned.string.as_ref().unwrap().as_ptr().cast_mut();
}

unsafe fn set_array_string(a: *mut options_array_item, value: CString) {
    let owned = &mut *a.cast::<OwnedOptionArrayItem>();
    owned.record.value.string = ::core::ptr::null_mut();
    owned.string = Some(value);
    owned.record.value.string = owned.string.as_ref().unwrap().as_ptr().cast_mut();
}

pub use crate::src::shared::key::key_code_enum as C2RustUnnamed_38;

unsafe fn options_array_correct_key(key: *const ::core::ffi::c_char) -> Option<CString> {
    match parse_array_index(CStr::from_ptr(key).to_bytes()) {
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
unsafe fn options_array_index(key: *const ::core::ffi::c_char) -> OptionsArrayKey {
    match parse_array_index(CStr::from_ptr(key).to_bytes()).expect("validated array key") {
        ArrayIndex::Numeric(number) => OptionsArrayKey::Numeric(number),
        ArrayIndex::Text(bytes) => OptionsArrayKey::Text(bytes.to_vec()),
    }
}

unsafe extern "C" fn options_map_name(
    mut name: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut map: *const options_name_map = ::core::ptr::null::<options_name_map>();
    map = &raw const options_other_names as *const options_name_map;
    while !(*map).from.is_null() {
        if strcmp((*map).from, name) == 0 as ::core::ffi::c_int {
            return (*map).to;
        }
        map = map.offset(1);
    }
    return name;
}
unsafe extern "C" fn options_parent_table_entry(
    mut oo: *mut options,
    mut s: *const ::core::ffi::c_char,
) -> *const options_table_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    if (*oo).parent.is_null() {
        fatalx(
            b"no parent options for %s\0" as *const u8 as *const ::core::ffi::c_char,
            s,
        );
    }
    o = options_get((*oo).parent, s);
    if o.is_null() {
        fatalx(
            b"%s not in parent options\0" as *const u8 as *const ::core::ffi::c_char,
            s,
        );
    }
    return (*o).tableentry;
}
unsafe extern "C" fn options_value_free(mut o: *mut options_entry, mut ov: *mut options_value) {
    if !(*o).tableentry.is_null()
        && (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_COMMAND as ::core::ffi::c_int as ::core::ffi::c_uint
        && !(*ov).cmdlist.is_null()
    {
        cmd_list_free((*ov).cmdlist);
    }
}
unsafe fn options_value_to_cstring(
    o: *mut options_entry,
    ov: *mut options_value,
    numeric: ::core::ffi::c_int,
) -> CString {
    if !(*o).tableentry.is_null()
        && (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_COMMAND as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return cmd_list_print_cstring((*ov).cmdlist, 0);
    }
    if !(*o).tableentry.is_null()
        && ((*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_COLOUR as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_FLAG as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_CHOICE as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        return match (*(*o).tableentry).type_0 as ::core::ffi::c_uint {
            1 => CString::new((*ov).number.to_string()).expect("decimal number has no NUL"),
            2 => key_string_format((*ov).number as key_code, false),
            3 => colour_format((*ov).number as ::core::ffi::c_int),
            4 => {
                if numeric != 0 {
                    CString::new((*ov).number.to_string()).expect("decimal number has no NUL")
                } else {
                    CStr::from_ptr(if (*ov).number != 0 {
                        b"on\0" as *const u8 as *const ::core::ffi::c_char
                    } else {
                        b"off\0" as *const u8 as *const ::core::ffi::c_char
                    })
                    .to_owned()
                }
            }
            5 => {
                CStr::from_ptr(*(*(*o).tableentry).choices.offset((*ov).number as isize)).to_owned()
            }
            _ => {
                fatalx(b"not a number option type\0" as *const u8 as *const ::core::ffi::c_char);
            }
        };
    }
    if (*o).tableentry.is_null()
        || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return CStr::from_ptr((*ov).string).to_owned();
    }
    c"".to_owned()
}
#[no_mangle]
pub unsafe extern "C" fn options_create(mut parent: *mut options) -> *mut options {
    let mut oo = Box::new(options {
        tree: ::core::ptr::null_mut(),
        parent,
    });
    oo.tree = Box::into_raw(Box::new(options_storage::default()));
    Box::into_raw(oo)
}
#[no_mangle]
pub unsafe extern "C" fn options_free(mut oo: *mut options) {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut tmp: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    o = options_first(oo);
    while !o.is_null() && {
        tmp = options_next(o);
        1 as ::core::ffi::c_int != 0
    } {
        options_remove(o);
        o = tmp;
    }
    drop(Box::from_raw((*oo).tree));
    drop(Box::from_raw(oo));
}
#[no_mangle]
pub unsafe extern "C" fn options_get_parent(mut oo: *mut options) -> *mut options {
    return (*oo).parent;
}
#[no_mangle]
pub unsafe extern "C" fn options_set_parent(mut oo: *mut options, mut parent: *mut options) {
    (*oo).parent = parent;
}
#[no_mangle]
pub unsafe extern "C" fn options_first(mut oo: *mut options) -> *mut options_entry {
    (*(*oo).tree)
        .entries
        .values()
        .next()
        .copied()
        .unwrap_or(std::ptr::null_mut())
}
#[no_mangle]
pub unsafe extern "C" fn options_next(mut o: *mut options_entry) -> *mut options_entry {
    let key = CStr::from_ptr((*o).name).to_bytes();
    (*(*(*o).owner).tree)
        .entries
        .range::<[u8], _>((std::ops::Bound::Excluded(key), std::ops::Bound::Unbounded))
        .next()
        .map(|(_, entry)| *entry)
        .unwrap_or(std::ptr::null_mut())
}
#[no_mangle]
pub unsafe extern "C" fn options_get_only(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
) -> *mut options_entry {
    let entries = &(*(*oo).tree).entries;
    entries
        .get(CStr::from_ptr(name).to_bytes())
        .or_else(|| entries.get(CStr::from_ptr(options_map_name(name)).to_bytes()))
        .copied()
        .unwrap_or(std::ptr::null_mut())
}
#[no_mangle]
pub unsafe extern "C" fn options_get(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
) -> *mut options_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    o = options_get_only(oo, name);
    while o.is_null() {
        oo = (*oo).parent;
        if oo.is_null() {
            break;
        }
        o = options_get_only(oo, name);
    }
    return o;
}
#[no_mangle]
pub unsafe extern "C" fn options_empty(
    mut oo: *mut options,
    mut oe: *const options_table_entry,
) -> *mut options_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    o = options_add(oo, (*oe).name);
    (*o).tableentry = oe;
    if (*oe).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
        (*o).value.array.storage = Box::into_raw(Box::new(options_array_storage::default()));
    }
    return o;
}
#[no_mangle]
pub unsafe extern "C" fn options_default(
    mut oo: *mut options,
    mut oe: *const options_table_entry,
) -> *mut options_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut ov: *mut options_value = ::core::ptr::null_mut::<options_value>();
    let mut key: [::core::ffi::c_char; 32] = [0; 32];
    let mut i: u_int = 0;
    let mut pr: *mut cmd_parse_result = ::core::ptr::null_mut::<cmd_parse_result>();
    o = options_empty(oo, oe);
    ov = &raw mut (*o).value;
    if (*oe).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
        if (*oe).default_arr.is_null() {
            options_array_assign(
                o,
                (*oe).default_str,
                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            );
            return o;
        }
        i = 0 as u_int;
        while !(*(*oe).default_arr.offset(i as isize)).is_null() {
            xsnprintf(
                &raw mut key as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                i,
            );
            options_array_set(
                o,
                &raw mut key as *mut ::core::ffi::c_char,
                *(*oe).default_arr.offset(i as isize),
                0 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            );
            i = i.wrapping_add(1);
        }
        return o;
    }
    match (*oe).type_0 as ::core::ffi::c_uint {
        0 => {
            set_scalar_string(o, CStr::from_ptr((*oe).default_str).to_owned());
        }
        6 => {
            pr = cmd_parse_from_string(
                (*oe).default_str,
                ::core::ptr::null_mut::<cmd_parse_input>(),
            );
            match (*pr).status as ::core::ffi::c_uint {
                0 => {
                    free((*pr).error as *mut ::core::ffi::c_void);
                }
                1 => {
                    (*ov).cmdlist = (*pr).cmdlist;
                }
                _ => {}
            }
        }
        _ => {
            (*ov).number = (*oe).default_num;
        }
    }
    return o;
}
#[no_mangle]
pub unsafe extern "C" fn options_default_to_string(
    mut oe: *const options_table_entry,
) -> *mut ::core::ffi::c_char {
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    match (*oe).type_0 as ::core::ffi::c_uint {
        0 | 6 => {
            s = xstrdup((*oe).default_str);
        }
        1 => {
            xasprintf(
                &raw mut s,
                b"%lld\0" as *const u8 as *const ::core::ffi::c_char,
                (*oe).default_num,
            );
        }
        2 => {
            let key_string = key_string_format((*oe).default_num as key_code, false);
            s = xstrdup(key_string.as_ptr());
        }
        3 => {
            s = xstrdup(colour_format((*oe).default_num as ::core::ffi::c_int).as_ptr());
        }
        4 => {
            s = xstrdup(if (*oe).default_num != 0 {
                b"on\0" as *const u8 as *const ::core::ffi::c_char
            } else {
                b"off\0" as *const u8 as *const ::core::ffi::c_char
            });
        }
        5 => {
            s = xstrdup(*(*oe).choices.offset((*oe).default_num as isize));
        }
        _ => {
            fatalx(b"unknown option type\0" as *const u8 as *const ::core::ffi::c_char);
        }
    }
    return s;
}
unsafe extern "C" fn options_add(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
) -> *mut options_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let name = CStr::from_ptr(name).to_owned();
    let lookup_name = name.as_ptr();
    o = options_get_only(oo, lookup_name);
    if !o.is_null() {
        options_remove(o);
    }
    let mut owned = Box::new(OwnedOptionEntry {
        record: ::core::mem::zeroed(),
        name,
        string: None,
    });
    owned.record.owner = oo;
    owned.record.name = owned.name.as_ptr();
    o = &raw mut owned.record;
    let _ = Box::into_raw(owned);
    (*(*oo).tree)
        .entries
        .insert(CStr::from_ptr((*o).name).to_bytes().to_vec(), o);
    return o;
}
unsafe extern "C" fn options_remove(mut o: *mut options_entry) {
    let mut oo: *mut options = (*o).owner;
    if !(*o).tableentry.is_null() && (*(*o).tableentry).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
        options_array_clear(o);
        drop(Box::from_raw((*o).value.array.storage));
    } else if (*o).tableentry.is_null()
        || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        // Match the old string-before-monitor cleanup order.
        drop((*owned_option(o)).string.take());
    } else if !(*o).tableentry.is_null()
        && (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_COMMAND as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        options_value_free(o, &raw mut (*o).value);
    }
    if !(*o).monitor_data.is_null() {
        hooks_monitor_free((*o).monitor_data);
    }
    (*(*oo).tree)
        .entries
        .remove(CStr::from_ptr((*o).name).to_bytes());
    drop(Box::from_raw(owned_option(o)));
}
#[no_mangle]
pub unsafe extern "C" fn options_name(mut o: *mut options_entry) -> *const ::core::ffi::c_char {
    return (*o).name;
}
#[no_mangle]
pub unsafe extern "C" fn options_owner(mut o: *mut options_entry) -> *mut options {
    return (*o).owner;
}
#[no_mangle]
pub unsafe extern "C" fn options_get_monitor_data(
    mut o: *mut options_entry,
) -> *mut ::core::ffi::c_void {
    return (*o).monitor_data;
}
#[no_mangle]
pub unsafe extern "C" fn options_set_monitor_data(
    mut o: *mut options_entry,
    mut data: *mut ::core::ffi::c_void,
) {
    (*o).monitor_data = data;
}
#[no_mangle]
pub unsafe extern "C" fn options_hook_fired(mut o: *mut options_entry) {
    (*o).fire_count = (*o).fire_count.wrapping_add(1);
    (*o).fire_time = current_time;
}
#[no_mangle]
pub unsafe extern "C" fn options_get_fire_count(mut o: *mut options_entry) -> u_int {
    return (*o).fire_count;
}
#[no_mangle]
pub unsafe extern "C" fn options_get_fire_time(mut o: *mut options_entry) -> time_t {
    return (*o).fire_time;
}
#[no_mangle]
pub unsafe extern "C" fn options_table_entry(
    mut o: *mut options_entry,
) -> *const options_table_entry {
    return (*o).tableentry;
}
unsafe extern "C" fn options_array_item(
    mut o: *mut options_entry,
    mut key: *const ::core::ffi::c_char,
) -> *mut options_array_item {
    (*(*o).value.array.storage)
        .entries
        .get(&options_array_index(key))
        .copied()
        .unwrap_or(::core::ptr::null_mut())
}
unsafe extern "C" fn options_array_new(
    mut o: *mut options_entry,
    mut key: *const ::core::ffi::c_char,
) -> *mut options_array_item {
    let mut owner = Box::new(OwnedOptionArrayItem {
        record: ::core::mem::zeroed(),
        key: CStr::from_ptr(key).to_owned(),
        string: None,
    });
    owner.record.key = owner.key.as_ptr().cast_mut();
    owner.record.owner = o;
    let a = Box::into_raw(owner).cast::<options_array_item>();
    (*(*o).value.array.storage)
        .entries
        .insert(options_array_index(key), a);
    return a;
}
unsafe extern "C" fn options_array_free(mut o: *mut options_entry, mut a: *mut options_array_item) {
    options_value_free(o, &raw mut (*a).value);
    (*(*o).value.array.storage)
        .entries
        .remove(&options_array_index((*a).key));
    drop(Box::from_raw(a.cast::<OwnedOptionArrayItem>()));
}
#[no_mangle]
pub unsafe extern "C" fn options_array_clear(mut o: *mut options_entry) {
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut a1: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    if !(!(*o).tableentry.is_null() && (*(*o).tableentry).flags & OPTIONS_TABLE_IS_ARRAY != 0) {
        return;
    }
    a = options_array_first(o);
    while !a.is_null() && {
        a1 = options_array_next(a);
        1 as ::core::ffi::c_int != 0
    } {
        options_array_free(o, a);
        a = a1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn options_array_get(
    mut o: *mut options_entry,
    mut key: *const ::core::ffi::c_char,
) -> *mut options_value {
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    if !(!(*o).tableentry.is_null() && (*(*o).tableentry).flags & OPTIONS_TABLE_IS_ARRAY != 0) {
        return ::core::ptr::null_mut::<options_value>();
    }
    let Some(new_key) = options_array_correct_key(key) else {
        return ::core::ptr::null_mut::<options_value>();
    };
    a = options_array_item(o, new_key.as_ptr());
    if a.is_null() {
        return ::core::ptr::null_mut::<options_value>();
    }
    return &raw mut (*a).value;
}
#[no_mangle]
pub unsafe extern "C" fn options_array_getv(
    mut o: *mut options_entry,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) -> *mut options_value {
    let mut ap: ::core::ffi::VaList;
    ap = args.clone();
    let key = xvasprintf_cstring(fmt, ap);
    options_array_get(o, key.as_ptr())
}
#[no_mangle]
pub unsafe extern "C" fn options_array_set(
    mut o: *mut options_entry,
    mut key: *const ::core::ffi::c_char,
    mut value: *const ::core::ffi::c_char,
    mut append: ::core::ffi::c_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut pr: *mut cmd_parse_result = ::core::ptr::null_mut::<cmd_parse_result>();
    let mut number: ::core::ffi::c_longlong = 0;
    if !(!(*o).tableentry.is_null() && (*(*o).tableentry).flags & OPTIONS_TABLE_IS_ARRAY != 0) {
        if !cause.is_null() {
            *cause = xstrdup(b"not an array\0" as *const u8 as *const ::core::ffi::c_char);
        }
        return -(1 as ::core::ffi::c_int);
    }
    let Some(new_key) = options_array_correct_key(key) else {
        if !cause.is_null() {
            xasprintf(
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
    if !(*o).tableentry.is_null()
        && (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_COMMAND as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        pr = cmd_parse_from_string(value, ::core::ptr::null_mut::<cmd_parse_input>());
        match (*pr).status as ::core::ffi::c_uint {
            0 => {
                if !cause.is_null() {
                    *cause = (*pr).error;
                } else {
                    free((*pr).error as *mut ::core::ffi::c_void);
                }
                return -(1 as ::core::ffi::c_int);
            }
            1 | _ => {}
        }
        a = options_array_item(o, new_key.as_ptr());
        if a.is_null() {
            a = options_array_new(o, new_key.as_ptr());
        } else {
            options_value_free(o, &raw mut (*a).value);
        }
        (*a).value.cmdlist = (*pr).cmdlist;
        return 0 as ::core::ffi::c_int;
    }
    if (*o).tableentry.is_null()
        || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        a = options_array_item(o, new_key.as_ptr());
        let owned_value = if !a.is_null() && append != 0 {
            let previous = CStr::from_ptr((*a).value.string).to_bytes();
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
        set_array_string(a, owned_value);
        return 0 as ::core::ffi::c_int;
    }
    if (*(*o).tableentry).type_0 as ::core::ffi::c_uint
        == OPTIONS_TABLE_COLOUR as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        number = colour_parse_cstr(std::ffi::CStr::from_ptr(value)).unwrap_or(-1)
            as ::core::ffi::c_longlong;
        if number == -(1 as ::core::ffi::c_int) as ::core::ffi::c_longlong {
            xasprintf(
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
            options_value_free(o, &raw mut (*a).value);
        }
        (*a).value.number = number;
        return 0 as ::core::ffi::c_int;
    }
    if !cause.is_null() {
        *cause = xstrdup(b"wrong array type\0" as *const u8 as *const ::core::ffi::c_char);
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn options_array_assign(
    mut o: *mut options_entry,
    mut s: *const ::core::ffi::c_char,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut separator: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut key: [::core::ffi::c_char; 32] = [0; 32];
    let mut i: u_int = 0;
    separator = (*(*o).tableentry).separator;
    if separator.is_null() {
        separator = b" ,\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if *separator as ::core::ffi::c_int == '\0' as i32 {
        if *s as ::core::ffi::c_int == '\0' as i32 {
            return 0 as ::core::ffi::c_int;
        }
        i = 0 as u_int;
        while i < UINT_MAX {
            if options_array_getv(o, b"%u\0" as *const u8 as *const ::core::ffi::c_char, i)
                .is_null()
            {
                break;
            }
            i = i.wrapping_add(1);
        }
        xsnprintf(
            &raw mut key as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            i,
        );
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
            if options_array_getv(o, b"%u\0" as *const u8 as *const ::core::ffi::c_char, i)
                .is_null()
            {
                break;
            }
            i = i.wrapping_add(1);
        }
        if i == UINT_MAX {
            break;
        }
        xsnprintf(
            &raw mut key as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            i,
        );
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
#[no_mangle]
pub unsafe extern "C" fn options_array_first(mut o: *mut options_entry) -> *mut options_array_item {
    if !(!(*o).tableentry.is_null() && (*(*o).tableentry).flags & OPTIONS_TABLE_IS_ARRAY != 0) {
        return ::core::ptr::null_mut::<options_array_item>();
    }
    (*(*o).value.array.storage)
        .entries
        .first_key_value()
        .map_or(::core::ptr::null_mut(), |(_, &item)| item)
}
#[no_mangle]
pub unsafe extern "C" fn options_array_next(
    mut a: *mut options_array_item,
) -> *mut options_array_item {
    let key = options_array_index((*a).key);
    (*(*(*a).owner).value.array.storage)
        .entries
        .range((std::ops::Bound::Excluded(key), std::ops::Bound::Unbounded))
        .next()
        .map_or(::core::ptr::null_mut(), |(_, &item)| item)
}
#[no_mangle]
pub unsafe extern "C" fn options_array_item_key(
    mut a: *mut options_array_item,
) -> *const ::core::ffi::c_char {
    return (*a).key;
}
#[no_mangle]
pub unsafe extern "C" fn options_array_item_value(
    mut a: *mut options_array_item,
) -> *mut options_value {
    return &raw mut (*a).value;
}
#[no_mangle]
pub unsafe extern "C" fn options_is_array(mut o: *mut options_entry) -> ::core::ffi::c_int {
    return (!(*o).tableentry.is_null() && (*(*o).tableentry).flags & OPTIONS_TABLE_IS_ARRAY != 0)
        as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn options_is_string(mut o: *mut options_entry) -> ::core::ffi::c_int {
    return ((*o).tableentry.is_null()
        || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint)
        as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn options_to_string(
    o: *mut options_entry,
    key: *const ::core::ffi::c_char,
    numeric: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    // Preserve the exported malloc/free contract while internal callers keep
    // their string's Rust owner for the duration of their use.
    xstrdup(options_to_cstring(o, key, numeric).as_ptr())
}

pub(crate) unsafe fn options_to_cstring(
    o: *mut options_entry,
    key: *const ::core::ffi::c_char,
    numeric: ::core::ffi::c_int,
) -> CString {
    if !(*o).tableentry.is_null() && (*(*o).tableentry).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
        if key.is_null() {
            let mut result = Vec::new();
            let mut a = options_array_first(o);
            let mut first = true;
            while !a.is_null() {
                let value = options_value_to_cstring(o, &raw mut (*a).value, numeric);
                if !first {
                    result.push(b' ');
                }
                result.extend_from_slice(value.as_bytes());
                first = false;
                a = options_array_next(a);
            }
            return CString::new(result).expect("option values have no NUL");
        }
        let Some(new_key) = options_array_correct_key(key) else {
            return c"".to_owned();
        };
        let a = options_array_item(o, new_key.as_ptr());
        if a.is_null() {
            return c"".to_owned();
        }
        return options_value_to_cstring(o, &raw mut (*a).value, numeric);
    }
    options_value_to_cstring(o, &raw mut (*o).value, numeric)
}
#[no_mangle]
pub unsafe extern "C" fn options_parse(
    name: *const ::core::ffi::c_char,
    key: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let input = std::ffi::CStr::from_ptr(name).to_bytes();
    if input.is_empty() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    *key = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let copy = xstrdup(name);
    let parsed = match parse_option_name(input) {
        Ok(parsed) => parsed,
        Err(_) => {
            free(copy as *mut ::core::ffi::c_void);
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
    };
    if parsed.array_key.is_some() {
        let open = input
            .iter()
            .position(|&byte| byte == b'[')
            .expect("parsed array option has an opening bracket");
        let raw = CString::new(&input[open + 1..input.len() - 1])
            .expect("C string option name has no interior NUL");
        let Some(new_key) = options_array_correct_key(raw.as_ptr()) else {
            free(copy as *mut ::core::ffi::c_void);
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        };
        // options_parse returns a C-owned key. Copy at this ABI boundary;
        // normalized scratch remains owned by the local CString.
        *key = xstrdup(new_key.as_ptr());
        *copy.add(parsed.name.len()) = '\0' as ::core::ffi::c_char;
    }
    return copy;
}
#[no_mangle]
pub unsafe extern "C" fn options_parse_get(
    mut oo: *mut options,
    mut s: *const ::core::ffi::c_char,
    mut key: *mut *mut ::core::ffi::c_char,
    mut only: ::core::ffi::c_int,
) -> *mut options_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    name = options_parse(s, key);
    if name.is_null() {
        return ::core::ptr::null_mut::<options_entry>();
    }
    if only != 0 {
        o = options_get_only(oo, name);
    } else {
        o = options_get(oo, name);
    }
    free(name as *mut ::core::ffi::c_void);
    if o.is_null() {
        free(*key as *mut ::core::ffi::c_void);
        *key = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return o;
}
#[no_mangle]
pub unsafe extern "C" fn options_search(
    mut name: *const ::core::ffi::c_char,
) -> *const options_table_entry {
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    oe = &raw const options_table as *const options_table_entry;
    while !(*oe).name.is_null() {
        if strcmp((*oe).name, name) == 0 as ::core::ffi::c_int {
            return oe;
        }
        oe = oe.offset(1);
    }
    return ::core::ptr::null::<options_table_entry>();
}
#[no_mangle]
pub unsafe extern "C" fn options_match(
    s: *const ::core::ffi::c_char,
    key: *mut *mut ::core::ffi::c_char,
    ambiguous: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    options_match_command(s, key, ambiguous)
}

/// Rust-facing command adapter for option-name matching.
///
/// The returned name and array key use the historical C allocator and remain
/// owned by the caller. The ABI entry point above delegates here so the
/// set-option and show-options commands can share the byte parser without
/// changing their ownership rules.
pub unsafe fn options_match_command(
    s: *const ::core::ffi::c_char,
    key: *mut *mut ::core::ffi::c_char,
    ambiguous: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let parsed = options_parse(s, key);
    if parsed.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }

    let mut candidates: [&[u8]; 273] = [&[]; 273];
    let mut entries: [*const options_table_entry; 273] =
        [::core::ptr::null::<options_table_entry>(); 273];
    let mut candidate_count = 0usize;
    let mut oe = &raw const options_table as *const options_table_entry;
    while candidate_count < candidates.len() && !(*oe).name.is_null() {
        candidates[candidate_count] = std::ffi::CStr::from_ptr((*oe).name).to_bytes();
        entries[candidate_count] = oe;
        candidate_count += 1;
        oe = oe.offset(1);
    }

    let mut aliases: [(&[u8], &[u8]); 8] = [(&[], &[]); 8];
    let mut alias_count = 0usize;
    let mut map = &raw const options_other_names as *const options_name_map;
    while alias_count < aliases.len() && !(*map).from.is_null() {
        aliases[alias_count] = (
            std::ffi::CStr::from_ptr((*map).from).to_bytes(),
            std::ffi::CStr::from_ptr((*map).to).to_bytes(),
        );
        alias_count += 1;
        map = map.offset(1);
    }

    let result = match match_option_name(
        std::ffi::CStr::from_ptr(s).to_bytes(),
        &aliases[..alias_count],
        &candidates[..candidate_count],
    ) {
        Ok(OptionNameMatch::User) => {
            *ambiguous = 0 as ::core::ffi::c_int;
            return parsed;
        }
        Ok(OptionNameMatch::BuiltIn(index)) => {
            let result = xstrdup((*entries[index]).name);
            free(parsed as *mut ::core::ffi::c_void);
            result
        }
        Err(OptionNameMatchError::Ambiguous) => {
            *ambiguous = 1 as ::core::ffi::c_int;
            free(parsed as *mut ::core::ffi::c_void);
            free(*key as *mut ::core::ffi::c_void);
            *key = ::core::ptr::null_mut::<::core::ffi::c_char>();
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        Err(OptionNameMatchError::Invalid(_) | OptionNameMatchError::NotFound) => {
            *ambiguous = 0 as ::core::ffi::c_int;
            free(parsed as *mut ::core::ffi::c_void);
            free(*key as *mut ::core::ffi::c_void);
            *key = ::core::ptr::null_mut::<::core::ffi::c_char>();
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
    };
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn options_match_get(
    mut oo: *mut options,
    mut s: *const ::core::ffi::c_char,
    mut key: *mut *mut ::core::ffi::c_char,
    mut only: ::core::ffi::c_int,
    mut ambiguous: *mut ::core::ffi::c_int,
) -> *mut options_entry {
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    name = options_match(s, key, ambiguous);
    if name.is_null() {
        return ::core::ptr::null_mut::<options_entry>();
    }
    *ambiguous = 0 as ::core::ffi::c_int;
    if only != 0 {
        o = options_get_only(oo, name);
    } else {
        o = options_get(oo, name);
    }
    free(name as *mut ::core::ffi::c_void);
    if o.is_null() {
        free(*key as *mut ::core::ffi::c_void);
        *key = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return o;
}
#[no_mangle]
pub unsafe extern "C" fn options_get_string(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    o = options_get(oo, name);
    if o.is_null() {
        fatalx(
            b"missing option %s\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    if !((*o).tableentry.is_null()
        || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        fatalx(
            b"option %s is not a string\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    return (*o).value.string;
}
#[no_mangle]
pub unsafe extern "C" fn options_get_number(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_longlong {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    o = options_get(oo, name);
    if o.is_null() {
        fatalx(
            b"missing option %s\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    if !(!(*o).tableentry.is_null()
        && ((*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_COLOUR as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_FLAG as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_CHOICE as ::core::ffi::c_int as ::core::ffi::c_uint))
    {
        fatalx(
            b"option %s is not a number\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    return (*o).value.number;
}
#[no_mangle]
pub unsafe extern "C" fn options_get_command(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
) -> *mut cmd_list {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    o = options_get(oo, name);
    if o.is_null() {
        fatalx(
            b"missing option %s\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    if !(!(*o).tableentry.is_null()
        && (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_COMMAND as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        fatalx(
            b"option %s is not a command\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    return (*o).value.cmdlist;
}
#[no_mangle]
pub unsafe extern "C" fn options_set_string(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
    mut append: ::core::ffi::c_int,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) -> *mut options_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut ap: ::core::ffi::VaList;
    let mut separator: *const ::core::ffi::c_char =
        b"\0" as *const u8 as *const ::core::ffi::c_char;
    ap = args.clone();
    let formatted = xvasprintf_cstring(fmt, ap);
    o = options_get_only(oo, name);
    let value = if !o.is_null()
        && append != 0
        && ((*o).tableentry.is_null()
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        if *name as ::core::ffi::c_int != '@' as i32 {
            separator = (*(*o).tableentry).separator;
            if separator.is_null() {
                separator = b"\0" as *const u8 as *const ::core::ffi::c_char;
            }
        }
        // glibc printf renders a null %s argument as "(null)". An entry
        // created by options_empty can reach this append path with no value.
        let previous = if (*o).value.string.is_null() {
            b"(null)".as_slice()
        } else {
            CStr::from_ptr((*o).value.string).to_bytes()
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
    if !((*o).tableentry.is_null()
        || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        fatalx(
            b"option %s is not a string\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    set_scalar_string(o, value);
    (*o).cached = 0 as ::core::ffi::c_int;
    return o;
}
#[no_mangle]
pub unsafe extern "C" fn options_set_number(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
    mut value: ::core::ffi::c_longlong,
) -> *mut options_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    if *name as ::core::ffi::c_int == '@' as i32 {
        fatalx(
            b"user option %s must be a string\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    o = options_get_only(oo, name);
    if o.is_null() {
        o = options_default(oo, options_parent_table_entry(oo, name));
        if o.is_null() {
            return ::core::ptr::null_mut::<options_entry>();
        }
    }
    if !(!(*o).tableentry.is_null()
        && ((*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_COLOUR as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_FLAG as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_CHOICE as ::core::ffi::c_int as ::core::ffi::c_uint))
    {
        fatalx(
            b"option %s is not a number\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    (*o).value.number = value;
    return o;
}
#[no_mangle]
pub unsafe extern "C" fn options_set_command(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
    mut value: *mut cmd_list,
) -> *mut options_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    if *name as ::core::ffi::c_int == '@' as i32 {
        fatalx(
            b"user option %s must be a string\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    o = options_get_only(oo, name);
    if o.is_null() {
        o = options_default(oo, options_parent_table_entry(oo, name));
        if o.is_null() {
            return ::core::ptr::null_mut::<options_entry>();
        }
    }
    if !(!(*o).tableentry.is_null()
        && (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_COMMAND as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        fatalx(
            b"option %s is not a command\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    if !(*o).value.cmdlist.is_null() {
        cmd_list_free((*o).value.cmdlist);
    }
    (*o).value.cmdlist = value;
    return o;
}
#[no_mangle]
pub unsafe extern "C" fn options_scope_from_name(
    mut args: *mut args,
    mut window: ::core::ffi::c_int,
    mut name: *const ::core::ffi::c_char,
    mut fs: *mut cmd_find_state,
    mut oo: *mut *mut options,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut s: *mut session = (*fs).s;
    let mut wl: *mut winlink = (*fs).wl;
    let mut wp: *mut window_pane = (*fs).wp;
    let mut target: *const ::core::ffi::c_char = args_get(args, 't' as i32 as u_char);
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    let mut scope: ::core::ffi::c_int = OPTIONS_TABLE_NONE;
    if *name as ::core::ffi::c_int == '@' as i32 {
        return options_scope_from_flags(args, window, fs, oo, cause);
    }
    oe = &raw const options_table as *const options_table_entry;
    while !(*oe).name.is_null() {
        if strcmp((*oe).name, name) == 0 as ::core::ffi::c_int {
            break;
        }
        oe = oe.offset(1);
    }
    if (*oe).name.is_null() {
        xasprintf(
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
                xasprintf(
                    cause,
                    b"no such session: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    target,
                );
            } else if s.is_null() {
                xasprintf(
                    cause,
                    b"no current session\0" as *const u8 as *const ::core::ffi::c_char,
                );
            } else {
                *oo = (*s).options;
                scope = OPTIONS_TABLE_SESSION;
            }
            current_block_38 = 980989089337379490;
        }
        12 => {
            if args_has(args, 'p' as i32 as u_char) != 0 {
                if wp.is_null() && !target.is_null() {
                    xasprintf(
                        cause,
                        b"no such pane: %s\0" as *const u8 as *const ::core::ffi::c_char,
                        target,
                    );
                } else if wp.is_null() {
                    xasprintf(
                        cause,
                        b"no current pane\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                } else {
                    *oo = (*wp).options;
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
                xasprintf(
                    cause,
                    b"no such window: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    target,
                );
            } else if wl.is_null() {
                xasprintf(
                    cause,
                    b"no current window\0" as *const u8 as *const ::core::ffi::c_char,
                );
            } else {
                *oo = (*(*wl).window).options;
                scope = OPTIONS_TABLE_WINDOW;
            }
        }
        _ => {}
    }
    return scope;
}
#[no_mangle]
pub unsafe extern "C" fn options_scope_from_flags(
    mut args: *mut args,
    mut window: ::core::ffi::c_int,
    mut fs: *mut cmd_find_state,
    mut oo: *mut *mut options,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut s: *mut session = (*fs).s;
    let mut wl: *mut winlink = (*fs).wl;
    let mut wp: *mut window_pane = (*fs).wp;
    let mut target: *const ::core::ffi::c_char = args_get(args, 't' as i32 as u_char);
    if args_has(args, 's' as i32 as u_char) != 0 {
        *oo = global_options;
        return 0x1 as ::core::ffi::c_int;
    }
    if args_has(args, 'p' as i32 as u_char) != 0 {
        if wp.is_null() {
            if !target.is_null() {
                xasprintf(
                    cause,
                    b"no such pane: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    target,
                );
            } else {
                xasprintf(
                    cause,
                    b"no current pane\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            return 0 as ::core::ffi::c_int;
        }
        *oo = (*wp).options;
        return 0x8 as ::core::ffi::c_int;
    } else if window != 0 || args_has(args, 'w' as i32 as u_char) != 0 {
        if args_has(args, 'g' as i32 as u_char) != 0 {
            *oo = global_w_options;
            return 0x4 as ::core::ffi::c_int;
        }
        if wl.is_null() {
            if !target.is_null() {
                xasprintf(
                    cause,
                    b"no such window: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    target,
                );
            } else {
                xasprintf(
                    cause,
                    b"no current window\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            return 0 as ::core::ffi::c_int;
        }
        *oo = (*(*wl).window).options;
        return 0x4 as ::core::ffi::c_int;
    } else {
        if args_has(args, 'g' as i32 as u_char) != 0 {
            *oo = global_s_options;
            return 0x2 as ::core::ffi::c_int;
        }
        if s.is_null() {
            if !target.is_null() {
                xasprintf(
                    cause,
                    b"no such session: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    target,
                );
            } else {
                xasprintf(
                    cause,
                    b"no current session\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            return 0 as ::core::ffi::c_int;
        }
        *oo = (*s).options;
        return 0x2 as ::core::ffi::c_int;
    };
}
#[no_mangle]
pub unsafe extern "C" fn options_string_to_style(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
    mut ft: *mut format_tree,
) -> *mut style {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    let mut dgc: *const grid_cell = &raw const grid_default_cell;
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut failed: ::core::ffi::c_int = 0;
    o = options_get(oo, name);
    if o.is_null()
        || !((*o).tableentry.is_null()
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        return ::core::ptr::null_mut::<style>();
    }
    if (*o).cached != 0 {
        return &raw mut (*o).style;
    }
    s = (*o).value.string;
    oe = (*o).tableentry;
    log_debug(
        b"%s: %s is '%s'\0" as *const u8 as *const ::core::ffi::c_char,
        b"options_string_to_style\0" as *const u8 as *const ::core::ffi::c_char,
        name,
        s,
    );
    style_set(&raw mut (*o).style, dgc);
    (*o).cached = (strstr(s, b"#{\0" as *const u8 as *const ::core::ffi::c_char)
        == NULL as *mut ::core::ffi::c_char) as ::core::ffi::c_int;
    if !ft.is_null() && (*o).cached == 0 {
        expanded = format_expand(ft, s);
        if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_COLOUR != 0 {
            failed = style_parse_colour(&raw mut (*o).style, dgc, expanded);
        } else {
            failed = style_parse(&raw mut (*o).style, dgc, expanded);
        }
        free(expanded as *mut ::core::ffi::c_void);
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
unsafe extern "C" fn options_from_string_check(
    mut oe: *const options_table_entry,
    mut value: *const ::core::ffi::c_char,
    mut cause: *mut *mut ::core::ffi::c_char,
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
        (*oe).name,
        b"default-shell\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        && checkshell(value) == 0
    {
        xasprintf(
            cause,
            b"not a suitable shell: %s\0" as *const u8 as *const ::core::ffi::c_char,
            value,
        );
        return -(1 as ::core::ffi::c_int);
    }
    if !(*oe).pattern.is_null()
        && fnmatch((*oe).pattern, value, 0 as ::core::ffi::c_int) != 0 as ::core::ffi::c_int
    {
        xasprintf(
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
        xasprintf(
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
        xasprintf(
            cause,
            b"invalid colour: %s\0" as *const u8 as *const ::core::ffi::c_char,
            value,
        );
        return -(1 as ::core::ffi::c_int);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn options_from_string_flag(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
    mut value: *const ::core::ffi::c_char,
    mut cause: *mut *mut ::core::ffi::c_char,
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
        xasprintf(
            cause,
            b"bad value: %s\0" as *const u8 as *const ::core::ffi::c_char,
            value,
        );
        return -(1 as ::core::ffi::c_int);
    }
    options_set_number(oo, name, flag as ::core::ffi::c_longlong);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn options_find_choice(
    mut oe: *const options_table_entry,
    mut value: *const ::core::ffi::c_char,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut cp: *mut *const ::core::ffi::c_char =
        ::core::ptr::null_mut::<*const ::core::ffi::c_char>();
    let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut choice: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    cp = (*oe).choices;
    while !(*cp).is_null() {
        if strcmp(*cp, value) == 0 as ::core::ffi::c_int {
            choice = n;
        }
        n += 1;
        cp = cp.offset(1);
    }
    if choice == -(1 as ::core::ffi::c_int) {
        xasprintf(
            cause,
            b"unknown value: %s\0" as *const u8 as *const ::core::ffi::c_char,
            value,
        );
        return -(1 as ::core::ffi::c_int);
    }
    return choice;
}
unsafe extern "C" fn options_from_string_choice(
    mut oe: *const options_table_entry,
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
    mut value: *const ::core::ffi::c_char,
    mut cause: *mut *mut ::core::ffi::c_char,
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
#[no_mangle]
pub unsafe extern "C" fn options_from_string(
    mut oo: *mut options,
    mut oe: *const options_table_entry,
    mut name: *const ::core::ffi::c_char,
    mut value: *const ::core::ffi::c_char,
    mut append: ::core::ffi::c_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut type_0: options_table_type = OPTIONS_TABLE_STRING;
    let mut number: ::core::ffi::c_longlong = 0;
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut new: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut key: key_code = 0;
    let mut pr: *mut cmd_parse_result = ::core::ptr::null_mut::<cmd_parse_result>();
    if !oe.is_null() {
        if value.is_null()
            && (*oe).type_0 as ::core::ffi::c_uint
                != OPTIONS_TABLE_FLAG as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*oe).type_0 as ::core::ffi::c_uint
                != OPTIONS_TABLE_CHOICE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            xasprintf(
                cause,
                b"empty value\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return -(1 as ::core::ffi::c_int);
        }
        type_0 = (*oe).type_0;
    } else {
        if *name as ::core::ffi::c_int != '@' as i32 {
            xasprintf(
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
            options_set_string(
                oo,
                name,
                append,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                value,
            );
            new = options_get_string(oo, name);
            if options_from_string_check(oe, new, cause) != 0 as ::core::ffi::c_int {
                options_set_string(
                    oo,
                    name,
                    0 as ::core::ffi::c_int,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    old.as_ptr(),
                );
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
                xasprintf(
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
                xasprintf(
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
                xasprintf(
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
            pr = cmd_parse_from_string(value, ::core::ptr::null_mut::<cmd_parse_input>());
            match (*pr).status as ::core::ffi::c_uint {
                0 => {
                    *cause = (*pr).error;
                    return -(1 as ::core::ffi::c_int);
                }
                1 => {
                    options_set_command(oo, name, (*pr).cmdlist);
                    return 0 as ::core::ffi::c_int;
                }
                _ => {}
            }
        }
        _ => {}
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn options_push_changes(mut name: *const ::core::ffi::c_char) {
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"options_push_changes\0" as *const u8 as *const ::core::ffi::c_char,
        name,
    );
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
        loop_0 = clients.tqh_first;
        while !loop_0.is_null() {
            server_client_update_theme_colours(loop_0);
            if (*loop_0).tty.flags & TTY_OPENED != 0 {
                tty_invalidate(&raw mut (*loop_0).tty);
            }
            server_redraw_client(loop_0);
            loop_0 = (*loop_0).entry.tqe_next;
        }
    }
    if strcmp(
        name,
        b"automatic-rename\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        w = windows_minmax(&raw mut windows, RB_NEGINF);
        while !w.is_null() {
            if !(*w).active.is_null() {
                if options_get_number((*w).options, name) != 0 {
                    (*(*w).active).flags |= PANE_CHANGED;
                }
            }
            w = windows_next(w);
        }
    }
    if strcmp(
        name,
        b"cursor-colour\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        wp = window_pane_tree_minmax(&raw mut all_window_panes, RB_NEGINF);
        while !wp.is_null() {
            window_pane_default_cursor(wp);
            wp = window_pane_tree_next(wp);
        }
    }
    if strcmp(
        name,
        b"cursor-style\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        wp = window_pane_tree_minmax(&raw mut all_window_panes, RB_NEGINF);
        while !wp.is_null() {
            window_pane_default_cursor(wp);
            wp = window_pane_tree_next(wp);
        }
    }
    if strcmp(
        name,
        b"fill-character\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        w = windows_minmax(&raw mut windows, RB_NEGINF);
        while !w.is_null() {
            window_set_fill_cells(w);
            w = windows_next(w);
        }
    }
    if strcmp(
        name,
        b"key-table\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        loop_0 = clients.tqh_first;
        while !loop_0.is_null() {
            server_client_set_key_table(loop_0, ::core::ptr::null::<::core::ffi::c_char>());
            loop_0 = (*loop_0).entry.tqe_next;
        }
    }
    if strcmp(
        name,
        b"user-keys\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        loop_0 = clients.tqh_first;
        while !loop_0.is_null() {
            if (*loop_0).tty.flags & TTY_OPENED != 0 {
                tty_keys_build(&raw mut (*loop_0).tty);
            }
            loop_0 = (*loop_0).entry.tqe_next;
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
        wp = window_pane_tree_minmax(&raw mut all_window_panes, RB_NEGINF);
        while !wp.is_null() {
            (*wp).flags |= PANE_STYLECHANGED | PANE_THEMECHANGED;
            wp = window_pane_tree_next(wp);
        }
    }
    if *name as ::core::ffi::c_int == '@' as i32 {
        wp = window_pane_tree_minmax(&raw mut all_window_panes, RB_NEGINF);
        while !wp.is_null() {
            (*wp).flags |= PANE_STYLECHANGED;
            wp = window_pane_tree_next(wp);
        }
    }
    if strcmp(
        name,
        b"pane-colours\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        wp = window_pane_tree_minmax(&raw mut all_window_panes, RB_NEGINF);
        while !wp.is_null() {
            colour_palette_from_option(&raw mut (*wp).palette, (*wp).options);
            wp = window_pane_tree_next(wp);
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
        w = windows_minmax(&raw mut windows, RB_NEGINF);
        while !w.is_null() {
            (*w).sb = options_get_number(
                (*w).options,
                b"pane-scrollbars\0" as *const u8 as *const ::core::ffi::c_char,
            ) as ::core::ffi::c_int;
            (*w).sb_pos = options_get_number(
                (*w).options,
                b"pane-scrollbars-position\0" as *const u8 as *const ::core::ffi::c_char,
            ) as ::core::ffi::c_int;
            layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
            w = windows_next(w);
        }
    }
    if strcmp(
        name,
        b"pane-scrollbars\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        wp = window_pane_tree_minmax(&raw mut all_window_panes, RB_NEGINF);
        while !wp.is_null() {
            window_pane_scrollbar_hide(wp);
            wp = window_pane_tree_next(wp);
        }
    }
    if strcmp(
        name,
        b"pane-scrollbars-style\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        wp = window_pane_tree_minmax(&raw mut all_window_panes, RB_NEGINF);
        while !wp.is_null() {
            style_set_scrollbar_style_from_option(&raw mut (*wp).scrollbar_style, (*wp).options);
            wp = window_pane_tree_next(wp);
        }
        w = windows_minmax(&raw mut windows, RB_NEGINF);
        while !w.is_null() {
            layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
            w = windows_next(w);
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
        s = sessions_minmax(&raw mut sessions, RB_NEGINF);
        while !s.is_null() {
            session_update_history(s);
            s = sessions_next(s);
        }
    }
    s = sessions_minmax(&raw mut sessions, RB_NEGINF);
    while !s.is_null() {
        status_update_cache(s);
        s = sessions_next(s);
    }
    recalculate_sizes();
    loop_0 = clients.tqh_first;
    while !loop_0.is_null() {
        if !(*loop_0).session.is_null() {
            server_redraw_client(loop_0);
        }
        loop_0 = (*loop_0).entry.tqe_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn options_remove_or_default(
    mut o: *mut options_entry,
    mut key: *const ::core::ffi::c_char,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut oo: *mut options = (*o).owner;
    if key.is_null() {
        if !(*o).tableentry.is_null()
            && (oo == global_options || oo == global_s_options || oo == global_w_options)
        {
            options_default(oo, (*o).tableentry);
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
    fn append_and_replace_accept_the_previous_items_string_pointer() {
        unsafe {
            let oo = options_create(::core::ptr::null_mut());
            let mut table: options_table_entry = ::core::mem::zeroed();
            table.name = c"sample-array".as_ptr();
            table.type_0 = OPTIONS_TABLE_STRING;
            table.flags = OPTIONS_TABLE_IS_ARRAY;
            let o = options_empty(oo, &raw const table);
            let key = c"7";
            let initial = CString::new(vec![b'a', 0xff]).unwrap();
            assert_eq!(
                options_array_set(
                    o,
                    key.as_ptr(),
                    initial.as_ptr(),
                    0,
                    ::core::ptr::null_mut()
                ),
                0
            );

            let old = (*options_array_get(o, key.as_ptr())).string;
            assert_eq!(
                options_array_set(o, key.as_ptr(), old, 1, ::core::ptr::null_mut()),
                0
            );
            let doubled = (*options_array_get(o, key.as_ptr())).string;
            assert_eq!(CStr::from_ptr(doubled).to_bytes(), b"a\xffa\xff");
            assert_eq!(
                options_array_set(o, key.as_ptr(), doubled, 0, ::core::ptr::null_mut()),
                0
            );
            assert_eq!(
                CStr::from_ptr((*options_array_get(o, key.as_ptr())).string).to_bytes(),
                b"a\xffa\xff"
            );

            assert_eq!(
                options_array_set(
                    o,
                    key.as_ptr(),
                    ::core::ptr::null(),
                    0,
                    ::core::ptr::null_mut()
                ),
                0
            );
            assert!(options_array_get(o, key.as_ptr()).is_null());
            options_free(oo);
        }
    }
}
