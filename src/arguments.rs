use crate::src::cmd::{
    cmd_get_args, cmd_get_entry, cmd_get_source, cmd_list_copy, cmd_list_first, cmd_list_free,
    cmd_list_print_cstring, cmd_log_argv, cmd_template_replace_cstring,
};
use crate::src::cmd_find::cmd_find_copy_state;
use crate::src::cmd_parse::cmd_parse_from_string;
use crate::src::cmd_queue::{cmdq_error, cmdq_get_target, cmdq_get_target_client};
use crate::src::compat::strtonum::strtonum;
use crate::src::ffi::libc::{__ctype_b_loc, free, strchr, strcspn};
use crate::src::format::format_single_from_target_cstring;
use crate::src::log::{fatalx, log_debug};
use crate::src::server_client::server_client_unref;
use crate::src::shared::abi::*;
pub use crate::src::shared::arguments::args_command_state;
use crate::src::shared::arguments::*;
pub use crate::src::shared::arguments::{
    args, args_entry, args_entry_entry, args_parse, args_parse_cb, args_tree, args_tree_storage,
    args_value, args_value_c2rust_unnamed, args_value_entry, args_values, args_values_storage,
};
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_file, client_file_cb, client_file_entry, client_files, overlay_check_cb,
    overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb, overlay_resize_cb,
};
use crate::src::shared::colour::*;
use crate::src::shared::command::*;
use crate::src::shared::command::*;
pub use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds,
};
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
pub use crate::src::shared::menu::menu_data;
use crate::src::shared::message::*;
pub use crate::src::shared::mouse::mouse_event;
pub use crate::src::shared::options::options;
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
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
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::variadic::{__builtin_va_list, __va_list_tag, va_list};
pub use crate::src::shared::vis::{VIS_CSTYLE, VIS_DQ, VIS_NL, VIS_OCTAL, VIS_TAB};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
use crate::src::utf8::utf8_strvis;
use crate::src::xmalloc::xvasprintf_cstring;
use std::borrow::Cow;
use std::ffi::{CStr, CString};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_21;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_20;

pub const ARGS_ENTRY_OPTIONAL_VALUE: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ArgumentValueError {
    Missing,
    Empty,
    Invalid,
    TooSmall,
    TooLarge,
}

#[derive(Debug)]
pub enum ArgsParseError {
    Usage,
    Message(CString),
}

fn parse_flag_error(prefix: &[u8], flag: u_char, suffix: &[u8]) -> CString {
    let mut bytes = Vec::with_capacity(prefix.len() + 1 + suffix.len());
    bytes.extend_from_slice(prefix);
    bytes.push(flag);
    bytes.extend_from_slice(suffix);
    CString::new(bytes).expect("argument flag diagnostics contain no NUL")
}

fn parse_number_error(message: String) -> CString {
    CString::new(message).expect("argument diagnostics contain no NUL")
}

impl ArgumentValueError {
    pub fn message(self) -> &'static CStr {
        unsafe {
            CStr::from_bytes_with_nul_unchecked(match self {
                Self::Missing => b"missing\0",
                Self::Empty => b"empty\0",
                Self::Invalid => b"invalid\0",
                Self::TooSmall => b"too small\0",
                Self::TooLarge => b"too large\0",
            })
        }
    }
}
unsafe fn args_tree_find(head: *mut args_tree, flag: u_char) -> *mut args_entry {
    if head.is_null() || (*head).entries.is_null() {
        return ::core::ptr::null_mut::<args_entry>();
    }
    (*(*head).entries)
        .entries
        .get(&flag)
        .copied()
        .unwrap_or(::core::ptr::null_mut::<args_entry>())
}

unsafe fn args_tree_insert(head: *mut args_tree, elm: *mut args_entry) -> *mut args_entry {
    if head.is_null() || elm.is_null() {
        return ::core::ptr::null_mut::<args_entry>();
    }
    if (*head).entries.is_null() {
        (*head).entries = Box::into_raw(Box::new(args_tree_storage::default()));
    }
    match (*(*head).entries).entries.entry((*elm).flag) {
        std::collections::btree_map::Entry::Occupied(entry) => *entry.get(),
        std::collections::btree_map::Entry::Vacant(entry) => {
            // Preserve the C ABI of args_next, which receives only an entry.
            // The legacy parent slot is now a non-owning storage back-pointer;
            // the BTreeMap is the sole source of ordering and membership.
            (*elm).entry.rbe_parent = (*head).entries as *mut args_entry;
            entry.insert(elm);
            ::core::ptr::null_mut::<args_entry>()
        }
    }
}

unsafe fn args_tree_remove(head: *mut args_tree, elm: *mut args_entry) -> *mut args_entry {
    if head.is_null() || (*head).entries.is_null() || elm.is_null() {
        return ::core::ptr::null_mut::<args_entry>();
    }
    let key = (*elm).flag;
    if (*(*head).entries).entries.get(&key).copied() != Some(elm) {
        return ::core::ptr::null_mut::<args_entry>();
    }
    let removed = (*(*head).entries)
        .entries
        .remove(&key)
        .unwrap_or(::core::ptr::null_mut::<args_entry>());
    (*elm).entry.rbe_parent = ::core::ptr::null_mut::<args_entry>();
    removed
}

unsafe fn args_tree_minmax(head: *mut args_tree, val: ::core::ffi::c_int) -> *mut args_entry {
    if head.is_null() || (*head).entries.is_null() {
        return ::core::ptr::null_mut::<args_entry>();
    }
    let entry = if val < 0 {
        (*(*head).entries).entries.values().next()
    } else {
        (*(*head).entries).entries.values().next_back()
    };
    entry
        .copied()
        .unwrap_or(::core::ptr::null_mut::<args_entry>())
}

unsafe fn args_tree_next(head: *mut args_tree, elm: *mut args_entry) -> *mut args_entry {
    if head.is_null() || (*head).entries.is_null() || elm.is_null() {
        return ::core::ptr::null_mut::<args_entry>();
    }
    args_tree_next_storage((*head).entries, elm)
}

unsafe fn args_tree_next_storage(
    storage: *mut args_tree_storage,
    elm: *mut args_entry,
) -> *mut args_entry {
    if storage.is_null() || elm.is_null() {
        return ::core::ptr::null_mut::<args_entry>();
    }
    (*storage)
        .entries
        .range((
            std::ops::Bound::Excluded((*elm).flag),
            std::ops::Bound::Unbounded,
        ))
        .next()
        .map(|(_, entry)| *entry)
        .unwrap_or(::core::ptr::null_mut::<args_entry>())
}

unsafe fn args_tree_next_from_entry(elm: *mut args_entry) -> *mut args_entry {
    if elm.is_null() {
        return ::core::ptr::null_mut::<args_entry>();
    }
    args_tree_next_storage((*elm).entry.rbe_parent as *mut args_tree_storage, elm)
}

#[cfg(test)]
mod args_tree_tests {
    use super::*;
    use crate::src::shared::tree::RB_INF;

    unsafe fn new_entry(flag: u_char) -> *mut args_entry {
        Box::into_raw(Box::new(args_entry {
            flag,
            values: args_values {
                first: ::core::ptr::null_mut::<args_value>(),
                storage: ::core::ptr::null_mut::<args_values_storage>(),
            },
            count: 0,
            flags: 0,
            entry: args_entry_entry {
                rbe_left: ::core::ptr::null_mut::<args_entry>(),
                rbe_right: ::core::ptr::null_mut::<args_entry>(),
                rbe_parent: ::core::ptr::null_mut::<args_entry>(),
                rbe_color: 0,
            },
        }))
    }

    #[test]
    fn args_tree_matches_unsigned_flag_order_and_duplicate_behavior() {
        unsafe {
            // args_cmp compared u_char values after promotion to c_int, so the
            // Rust key must order 0..=255 rather than signed bytes.
            let flags = [0x80, 0, 0xff, 1];
            let mut head = args_tree {
                entries: Box::into_raw(Box::new(args_tree_storage::default())),
            };
            let mut items = Vec::new();
            for flag in flags {
                let item = new_entry(flag);
                assert!(args_tree_insert(&mut head, item).is_null());
                items.push(item);
            }

            let duplicate = new_entry(0x80);
            assert_eq!(args_tree_insert(&mut head, duplicate), items[0]);
            drop(Box::from_raw(duplicate));
            assert_eq!(args_tree_find(&mut head, 0x80), items[0]);

            assert_eq!((*args_tree_minmax(&mut head, RB_NEGINF)).flag, 0);
            assert_eq!((*args_tree_minmax(&mut head, RB_INF)).flag, 0xff);

            let mut ordered = Vec::new();
            let mut item = args_tree_minmax(&mut head, RB_NEGINF);
            while !item.is_null() {
                ordered.push((*item).flag);
                // Exercise the public args_next-compatible path, which has no
                // tree-head argument and therefore uses the owner back-pointer.
                args_next(&raw mut item);
            }
            assert_eq!(ordered, vec![0, 1, 0x80, 0xff]);

            let removed = args_tree_remove(&mut head, items[2]);
            assert_eq!(removed, items[2]);
            assert!(args_tree_find(&mut head, 0xff).is_null());
            drop(Box::from_raw(removed));

            for (index, item) in items.into_iter().enumerate() {
                if index != 2 {
                    drop(Box::from_raw(item));
                }
            }
            drop(Box::from_raw(head.entries));
            head.entries = ::core::ptr::null_mut::<args_tree_storage>();
        }
    }
}

unsafe extern "C" fn args_find(args: *mut args, flag: u_char) -> *mut args_entry {
    args_tree_find(&raw mut (*args).tree, flag)
}

unsafe fn args_values_owner(entry: *mut args_entry) -> *mut args_values_storage {
    if entry.is_null() {
        return ::core::ptr::null_mut::<args_values_storage>();
    }
    (*entry).values.storage
}

unsafe fn args_values_ensure_owner(entry: *mut args_entry) -> *mut args_values_storage {
    let mut owner = args_values_owner(entry);
    if owner.is_null() {
        owner = Box::into_raw(Box::new(args_values_storage::default()));
        (*entry).values.first = ::core::ptr::null_mut::<args_value>();
        (*entry).values.storage = owner;
    }
    owner
}

unsafe fn args_value_at(entry: *mut args_entry, index: usize) -> *mut args_value {
    let owner = args_values_owner(entry);
    if owner.is_null() {
        return ::core::ptr::null_mut::<args_value>();
    }
    (&(*owner).values)
        .get(index)
        .map(|value| (&**value as *const args_value).cast_mut())
        .unwrap_or(::core::ptr::null_mut::<args_value>())
}

unsafe fn args_value_count(entry: *mut args_entry) -> usize {
    let owner = args_values_owner(entry);
    if owner.is_null() {
        0
    } else {
        (*owner).values.len()
    }
}

unsafe fn args_last_value(args: *mut args, flag: u_char) -> Option<*mut args_value> {
    let entry = args_find(args, flag);
    if entry.is_null() {
        return None;
    }
    let count = args_value_count(entry);
    (count != 0).then(|| args_value_at(entry, count - 1))
}

unsafe fn args_last_string(args: *mut args, flag: u_char) -> Option<*const ::core::ffi::c_char> {
    let value = args_last_value(args, flag)?;
    if (*value).type_0 as ::core::ffi::c_uint
        != ARGS_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*value).c2rust_unnamed.string.is_null()
    {
        return None;
    }
    Some((*value).c2rust_unnamed.string)
}
/// Owns the payload of a temporary args_value while it is being assembled or
/// transferred into an args collection. The raw record borrows its CString.
struct OwnedArgsValue {
    value: Box<args_value>,
    string: Option<CString>,
    owns_cmdlist: bool,
}

impl OwnedArgsValue {
    fn empty() -> Self {
        Self {
            value: Box::new(unsafe { ::core::mem::zeroed() }),
            string: None,
            owns_cmdlist: false,
        }
    }

    fn string(string: CString) -> Self {
        let mut result = Self::empty();
        result.value.type_0 = ARGS_STRING;
        result.value.c2rust_unnamed.string = string.as_ptr().cast_mut();
        result.string = Some(string);
        result
    }

    fn commands(cmdlist: *mut cmd_list) -> Self {
        let mut result = Self::empty();
        result.value.type_0 = ARGS_COMMANDS;
        result.value.c2rust_unnamed.cmdlist = cmdlist;
        result.owns_cmdlist = !cmdlist.is_null();
        result
    }

    fn as_value(&self) -> &args_value {
        &self.value
    }

    fn into_parts(self) -> (Box<args_value>, Option<CString>) {
        let owner = ::core::mem::ManuallyDrop::new(self);
        unsafe {
            (
                ::core::ptr::read(&owner.value),
                ::core::ptr::read(&owner.string),
            )
        }
    }
}

impl Drop for OwnedArgsValue {
    fn drop(&mut self) {
        if self.owns_cmdlist {
            unsafe {
                cmd_list_free(self.value.c2rust_unnamed.cmdlist);
            }
        }
    }
}

unsafe fn args_copy_value(from: *mut args_value) -> OwnedArgsValue {
    match (*from).type_0 as ::core::ffi::c_uint {
        2 => {
            let cmdlist = (*from).c2rust_unnamed.cmdlist;
            (*cmdlist).references += 1;
            OwnedArgsValue::commands(cmdlist)
        }
        1 => OwnedArgsValue::string(CStr::from_ptr((*from).c2rust_unnamed.string).to_owned()),
        0 | _ => OwnedArgsValue::empty(),
    }
}
unsafe extern "C" fn args_type_to_string(mut type_0: args_type) -> *const ::core::ffi::c_char {
    match type_0 as ::core::ffi::c_uint {
        0 => return b"NONE\0" as *const u8 as *const ::core::ffi::c_char,
        1 => return b"STRING\0" as *const u8 as *const ::core::ffi::c_char,
        2 => return b"COMMANDS\0" as *const u8 as *const ::core::ffi::c_char,
        _ => {}
    }
    return b"INVALID\0" as *const u8 as *const ::core::ffi::c_char;
}
unsafe fn args_value_for_log(value: &args_value) -> Cow<'_, CStr> {
    match value.type_0 as ::core::ffi::c_uint {
        0 => Cow::Borrowed(CStr::from_bytes_with_nul_unchecked(b"\0")),
        1 => Cow::Borrowed(CStr::from_ptr(value.c2rust_unnamed.string)),
        2 => Cow::Owned(cmd_list_print_cstring(value.c2rust_unnamed.cmdlist, 0)),
        _ => fatalx(b"unexpected argument type\0" as *const u8 as *const ::core::ffi::c_char),
    }
}
#[no_mangle]
pub unsafe extern "C" fn args_create() -> *mut args {
    let owner = Box::new(args {
        tree: args_tree {
            entries: Box::into_raw(Box::new(args_tree_storage::default())),
        },
        count: 0,
        values: Vec::new(),
        positional_strings: Vec::new(),
        positional_caches: Vec::new(),
    });
    Box::into_raw(owner).cast::<args>()
}
unsafe fn args_push_positional_owned(args: *mut args, value: OwnedArgsValue) {
    let owner = &mut *args;
    let (value, string) = value.into_parts();
    owner.values.push(*value);
    owner.positional_strings.push(string);
    owner.count = owner.values.len() as u_int;
}

/// Append a positional string and transfer its storage into `args`.
pub unsafe fn args_push_positional_string(args: *mut args, value: CString) {
    args_push_positional_owned(args, OwnedArgsValue::string(value));
}

/// Append a positional command list and transfer its reference into `args`.
pub unsafe fn args_push_positional_commands(args: *mut args, cmdlist: *mut cmd_list) {
    args_push_positional_owned(args, OwnedArgsValue::commands(cmdlist));
}

unsafe fn args_parse_flag_argument(
    values: *mut args_value,
    count: u_int,
    args: *mut args,
    i: *mut u_int,
    string: *const ::core::ffi::c_char,
    flag: ::core::ffi::c_int,
    optional_argument: ::core::ffi::c_int,
) -> Result<(), CString> {
    let new = if *string != 0 {
        OwnedArgsValue::string(CStr::from_ptr(string).to_owned())
    } else {
        let argument = if *i == count {
            ::core::ptr::null_mut::<args_value>()
        } else {
            values.add(*i as usize)
        };
        if !argument.is_null()
            && (*argument).type_0 as ::core::ffi::c_uint != ARGS_STRING as ::core::ffi::c_int as u32
        {
            return Err(parse_flag_error(
                b"-",
                flag as u_char,
                b" argument must be a string",
            ));
        }
        if argument.is_null() {
            if optional_argument != 0 {
                log_debug(
                    b"%s: -%c (optional)\0" as *const u8 as *const ::core::ffi::c_char,
                    b"args_parse_flag_argument\0" as *const u8 as *const ::core::ffi::c_char,
                    flag,
                );
                args_set_flag(args, flag as u_char, ARGS_ENTRY_OPTIONAL_VALUE);
                return Ok(());
            }
            return Err(parse_flag_error(
                b"-",
                flag as u_char,
                b" expects an argument",
            ));
        }
        if optional_argument != 0 {
            let value = (*argument).c2rust_unnamed.string;
            if *value == b'-' as ::core::ffi::c_char
                && (*value.add(1) == b'-' as ::core::ffi::c_char
                    || *(*__ctype_b_loc()).offset(*value.add(1) as u_char as isize)
                        as ::core::ffi::c_int
                        & _ISalpha as ::core::ffi::c_int as ::core::ffi::c_ushort
                            as ::core::ffi::c_int
                        != 0)
            {
                log_debug(
                    b"%s: -%c (optional)\0" as *const u8 as *const ::core::ffi::c_char,
                    b"args_parse_flag_argument\0" as *const u8 as *const ::core::ffi::c_char,
                    flag,
                );
                args_set_flag(args, flag as u_char, ARGS_ENTRY_OPTIONAL_VALUE);
                return Ok(());
            }
        }
        *i = (*i).wrapping_add(1);
        args_copy_value(argument)
    };
    let printed = args_value_for_log(new.as_value());
    log_debug(
        b"%s: -%c = %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"args_parse_flag_argument\0" as *const u8 as *const ::core::ffi::c_char,
        flag,
        printed.as_ptr(),
    );
    args_set_value(args, flag as u_char, Some(new), 0);
    Ok(())
}
unsafe fn args_parse_flags(
    mut parse: *const args_parse,
    mut values: *mut args_value,
    mut count: u_int,
    mut args: *mut args,
    mut i: *mut u_int,
) -> Result<::core::ffi::c_int, ArgsParseError> {
    let mut value: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut flag: u_char = 0;
    let mut found: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut string: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut optional_argument: ::core::ffi::c_int = 0;
    value = values.offset(*i as isize) as *mut args_value;
    if (*value).type_0 as ::core::ffi::c_uint
        != ARGS_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return Ok(1 as ::core::ffi::c_int);
    }
    string = (*value).c2rust_unnamed.string;
    log_debug(
        b"%s: next %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"args_parse_flags\0" as *const u8 as *const ::core::ffi::c_char,
        string,
    );
    let fresh1 = string;
    string = string.offset(1);
    if *fresh1 as ::core::ffi::c_int != '-' as i32 || *string as ::core::ffi::c_int == '\0' as i32 {
        return Ok(1 as ::core::ffi::c_int);
    }
    *i = (*i).wrapping_add(1);
    if *string.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '-' as i32
        && *string.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '\0' as i32
    {
        return Ok(1 as ::core::ffi::c_int);
    }
    loop {
        let fresh2 = string;
        string = string.offset(1);
        flag = *fresh2 as u_char;
        if flag as ::core::ffi::c_int == '\0' as i32 {
            return Ok(0 as ::core::ffi::c_int);
        }
        if flag as ::core::ffi::c_int == '?' as i32 {
            return Err(ArgsParseError::Usage);
        }
        if *(*__ctype_b_loc()).offset(flag as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            & _ISalnum as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            == 0
        {
            return Err(ArgsParseError::Message(parse_flag_error(
                b"invalid flag -",
                flag,
                b"",
            )));
        }
        found = strchr((*parse).template, flag as ::core::ffi::c_int);
        if found.is_null() {
            return Err(ArgsParseError::Message(parse_flag_error(
                b"unknown flag -",
                flag,
                b"",
            )));
        }
        if *found.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != ':' as i32 {
            log_debug(
                b"%s: -%c\0" as *const u8 as *const ::core::ffi::c_char,
                b"args_parse_flags\0" as *const u8 as *const ::core::ffi::c_char,
                flag as ::core::ffi::c_int,
            );
            args_set_flag(args, flag, 0);
        } else {
            optional_argument = (*found.offset(2 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                == ':' as i32) as ::core::ffi::c_int;
            return args_parse_flag_argument(
                values,
                count,
                args,
                i,
                string,
                flag as ::core::ffi::c_int,
                optional_argument,
            )
            .map(|()| 0)
            .map_err(ArgsParseError::Message);
        }
    }
}
pub unsafe fn args_parse(
    mut parse: *const args_parse,
    mut values: *mut args_value,
    mut count: u_int,
) -> Result<*mut args, ArgsParseError> {
    let mut args: *mut args = ::core::ptr::null_mut::<args>();
    let mut i: u_int = 0;
    let mut type_0: args_parse_type = ARGS_PARSE_INVALID;
    let mut value: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut stop: ::core::ffi::c_int = 0;
    if count == 0 as u_int {
        return Ok(args_create());
    }
    args = args_create();
    i = 1 as u_int;
    while i < count {
        stop = match args_parse_flags(parse, values, count, args, &raw mut i) {
            Ok(stop) => stop,
            Err(error) => {
                args_free(args);
                return Err(error);
            }
        };
        if stop == 1 as ::core::ffi::c_int {
            break;
        }
    }
    log_debug(
        b"%s: flags end at %u of %u\0" as *const u8 as *const ::core::ffi::c_char,
        b"args_parse\0" as *const u8 as *const ::core::ffi::c_char,
        i,
        count,
    );
    if i != count {
        while i < count {
            value = values.offset(i as isize) as *mut args_value;
            let printed = args_value_for_log(&*value);
            s = printed.as_ptr();
            log_debug(
                b"%s: %u = %s (type %s)\0" as *const u8 as *const ::core::ffi::c_char,
                b"args_parse\0" as *const u8 as *const ::core::ffi::c_char,
                i,
                s,
                args_type_to_string((*value).type_0),
            );
            if (*parse).cb.is_some() {
                let mut callback_error = ::core::ptr::null_mut::<::core::ffi::c_char>();
                type_0 = (*parse).cb.expect("non-null function pointer")(
                    args,
                    (*args).count,
                    &raw mut callback_error,
                );
                if type_0 as ::core::ffi::c_uint
                    == ARGS_PARSE_INVALID as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    args_free(args);
                    if callback_error.is_null() {
                        return Err(ArgsParseError::Usage);
                    }
                    let error = CStr::from_ptr(callback_error).to_owned();
                    free(callback_error.cast());
                    return Err(ArgsParseError::Message(error));
                }
                if !callback_error.is_null() {
                    free(callback_error.cast());
                }
            } else {
                type_0 = ARGS_PARSE_STRING;
            }
            let copied = match type_0 as ::core::ffi::c_uint {
                0 => {
                    fatalx(
                        b"unexpected argument type\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
                1 => {
                    if (*value).type_0 as ::core::ffi::c_uint
                        != ARGS_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        let error = parse_number_error(format!(
                            "argument {} must be \"string\"",
                            (*args).count.wrapping_add(1)
                        ));
                        args_free(args);
                        return Err(ArgsParseError::Message(error));
                    }
                    args_copy_value(value)
                }
                2 => args_copy_value(value),
                3 => {
                    if (*value).type_0 as ::core::ffi::c_uint
                        != ARGS_COMMANDS as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        let error = parse_number_error(format!(
                            "argument {} must be {{ commands }}",
                            (*args).count.wrapping_add(1)
                        ));
                        args_free(args);
                        return Err(ArgsParseError::Message(error));
                    }
                    args_copy_value(value)
                }
                _ => OwnedArgsValue::empty(),
            };
            args_push_positional_owned(args, copied);
            i = i.wrapping_add(1);
        }
    }
    if (*parse).lower != -(1 as ::core::ffi::c_int) && (*args).count < (*parse).lower as u_int {
        let error = parse_number_error(format!(
            "too few arguments (need at least {})",
            (*parse).lower as u_int
        ));
        args_free(args);
        return Err(ArgsParseError::Message(error));
    }
    if (*parse).upper != -(1 as ::core::ffi::c_int) && (*args).count > (*parse).upper as u_int {
        let error = parse_number_error(format!(
            "too many arguments (need at most {})",
            (*parse).upper as u_int
        ));
        args_free(args);
        return Err(ArgsParseError::Message(error));
    }
    return Ok(args);
}
unsafe fn args_copy_copy_value(from: *mut args_value, argv: &Vec<CString>) -> OwnedArgsValue {
    match (*from).type_0 as ::core::ffi::c_uint {
        1 => {
            let source = CStr::from_ptr((*from).c2rust_unnamed.string);
            if argv.is_empty() {
                return OwnedArgsValue::string(source.to_owned());
            }
            let mut expanded = cmd_template_replace_cstring(source.as_ptr(), argv[0].as_ptr(), 1);
            for i in 1..argv.len() {
                expanded = cmd_template_replace_cstring(
                    expanded.as_ptr(),
                    argv[i].as_ptr(),
                    (i + 1) as ::core::ffi::c_int,
                );
            }
            OwnedArgsValue::string(expanded)
        }
        2 => OwnedArgsValue::commands(
            cmd_list_copy((*from).c2rust_unnamed.cmdlist, argv) as *mut cmd_list
        ),
        0 | _ => OwnedArgsValue::empty(),
    }
}
pub unsafe fn args_copy(mut args: *mut args, argv: &Vec<CString>) -> *mut args {
    let mut new_args: *mut args = ::core::ptr::null_mut::<args>();
    let mut entry: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    let mut value: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut i: u_int = 0;
    cmd_log_argv(argv, c"args_copy");
    new_args = args_create();
    entry = args_tree_minmax(&raw mut (*args).tree, RB_NEGINF);
    while !entry.is_null() {
        if args_value_count(entry) == 0 {
            i = 0 as u_int;
            while i < (*entry).count {
                args_set_flag(new_args, (*entry).flag, 0);
                i = i.wrapping_add(1);
            }
        } else {
            value = args_value_at(entry, 0);
            while !value.is_null() {
                args_set_value(
                    new_args,
                    (*entry).flag,
                    Some(args_copy_copy_value(value, argv)),
                    0,
                );
                value = args_next_value(value);
            }
        }
        entry = args_tree_next(&raw mut (*args).tree, entry);
    }
    if (*args).count == 0 as u_int {
        return new_args;
    }
    i = 0 as u_int;
    while i < (*args).count {
        args_push_positional_owned(
            new_args,
            args_copy_copy_value((*args).values.as_mut_ptr().add(i as usize), argv),
        );
        i = i.wrapping_add(1);
    }
    return new_args;
}
unsafe fn args_value_release(value: *mut args_value) {
    match (*value).type_0 as ::core::ffi::c_uint {
        // String and cache pointers borrow from Rust owners alongside args.
        1 => {}
        2 => {
            cmd_list_free((*value).c2rust_unnamed.cmdlist as *mut cmd_list);
        }
        0 | _ => {}
    }
}
#[no_mangle]
pub unsafe extern "C" fn args_free(mut args: *mut args) {
    let mut entry: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    let mut entry1: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    // The C-layout fields borrow the string and cache owners kept beside them.
    // Drop command-list references here, then let the Rust side storage drop.
    let owner = &mut *args;
    for value in owner.values.iter_mut() {
        args_value_release(value as *mut args_value);
    }
    entry = args_tree_minmax(&raw mut (*args).tree, RB_NEGINF);
    while !entry.is_null() && {
        entry1 = args_tree_next(&raw mut (*args).tree, entry);
        1 as ::core::ffi::c_int != 0
    } {
        args_tree_remove(&raw mut (*args).tree, entry);
        let values_owner = args_values_owner(entry);
        if !values_owner.is_null() {
            let mut values_owner = Box::from_raw(values_owner);
            let args_values_storage { values, .. } = &mut *values_owner;
            for value in values.iter_mut() {
                args_value_release((&mut **value) as *mut args_value);
            }
        }
        drop(Box::from_raw(entry));
        entry = entry1;
    }
    if !(*args).tree.entries.is_null() {
        drop(Box::from_raw((*args).tree.entries));
        (*args).tree.entries = ::core::ptr::null_mut::<args_tree_storage>();
    }
    drop(Box::from_raw(args));
}
pub unsafe fn args_to_vector(args: *mut args) -> Vec<CString> {
    let mut argv = Vec::new();
    for value in (*args).values.iter() {
        match value.type_0 as ::core::ffi::c_uint {
            1 => {
                assert!(
                    !value.c2rust_unnamed.string.is_null(),
                    "string argument value must own a C string"
                );
                argv.push(CStr::from_ptr(value.c2rust_unnamed.string).to_owned());
            }
            2 => {
                let printed =
                    cmd_list_print_cstring(value.c2rust_unnamed.cmdlist, 0 as ::core::ffi::c_int);
                argv.push(printed);
            }
            _ => {}
        }
    }
    argv
}
unsafe extern "C" fn args_print_add(
    buf: &mut Vec<u8>,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut ap: ::core::ffi::VaList;
    ap = args.clone();
    let formatted = xvasprintf_cstring(fmt, ap);
    buf.extend_from_slice(formatted.as_bytes());
}
unsafe fn args_print_add_value(buf: &mut Vec<u8>, value: &args_value) {
    if !buf.is_empty() {
        args_print_add(buf, b" \0" as *const u8 as *const ::core::ffi::c_char);
    }
    match value.type_0 as ::core::ffi::c_uint {
        2 => {
            let expanded = cmd_list_print_cstring(value.c2rust_unnamed.cmdlist, 0);
            args_print_add(
                buf,
                b"{ %s }\0" as *const u8 as *const ::core::ffi::c_char,
                expanded.as_ptr(),
            );
        }
        1 => {
            let expanded = args_escape_cstring(CStr::from_ptr(value.c2rust_unnamed.string));
            args_print_add(
                buf,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                expanded.as_ptr(),
            );
        }
        0 | _ => {}
    }
}
pub unsafe fn args_print(args: *mut args) -> CString {
    args_print_cstring(args)
}

pub(crate) unsafe fn args_print_cstring(args: *mut args) -> CString {
    let mut buf = Vec::new();
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let mut entry: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    let mut last: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    let mut value: *mut args_value = ::core::ptr::null_mut::<args_value>();
    entry = args_tree_minmax(&raw mut (*args).tree, RB_NEGINF);
    while !entry.is_null() {
        if !((*entry).flags & ARGS_ENTRY_OPTIONAL_VALUE != 0) {
            if args_value_count(entry) == 0 {
                if buf.is_empty() {
                    args_print_add(&mut buf, b"-\0" as *const u8 as *const ::core::ffi::c_char);
                }
                j = 0 as u_int;
                while j < (*entry).count {
                    args_print_add(
                        &mut buf,
                        b"%c\0" as *const u8 as *const ::core::ffi::c_char,
                        (*entry).flag as ::core::ffi::c_int,
                    );
                    j = j.wrapping_add(1);
                }
            }
        }
        entry = args_tree_next(&raw mut (*args).tree, entry);
    }
    entry = args_tree_minmax(&raw mut (*args).tree, RB_NEGINF);
    while !entry.is_null() {
        if (*entry).flags & ARGS_ENTRY_OPTIONAL_VALUE != 0 {
            if !buf.is_empty() {
                args_print_add(
                    &mut buf,
                    b" -%c\0" as *const u8 as *const ::core::ffi::c_char,
                    (*entry).flag as ::core::ffi::c_int,
                );
            } else {
                args_print_add(
                    &mut buf,
                    b"-%c\0" as *const u8 as *const ::core::ffi::c_char,
                    (*entry).flag as ::core::ffi::c_int,
                );
            }
            last = entry;
        } else if args_value_count(entry) != 0 {
            value = args_value_at(entry, 0);
            while !value.is_null() {
                if !buf.is_empty() {
                    args_print_add(
                        &mut buf,
                        b" -%c\0" as *const u8 as *const ::core::ffi::c_char,
                        (*entry).flag as ::core::ffi::c_int,
                    );
                } else {
                    args_print_add(
                        &mut buf,
                        b"-%c\0" as *const u8 as *const ::core::ffi::c_char,
                        (*entry).flag as ::core::ffi::c_int,
                    );
                }
                args_print_add_value(&mut buf, &*value);
                value = args_next_value(value);
            }
            last = entry;
        }
        entry = args_tree_next(&raw mut (*args).tree, entry);
    }
    if !last.is_null() && (*last).flags & ARGS_ENTRY_OPTIONAL_VALUE != 0 {
        args_print_add(
            &mut buf,
            b" --\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    i = 0 as u_int;
    while i < (*args).count {
        args_print_add_value(&mut buf, &(&(*args).values)[i as usize]);
        i = i.wrapping_add(1);
    }
    CString::new(buf).expect("printed arguments contain no interior NUL")
}
pub(crate) unsafe fn args_escape_cstring(s: &CStr) -> CString {
    let source = s.to_bytes();
    if source.is_empty() {
        return CString::new(b"''".to_vec()).expect("literal has no NUL");
    }

    let quotes = if source.iter().any(|byte| b" #';${}%".contains(byte)) {
        b'"'
    } else if source.iter().any(|byte| b" \"".contains(byte)) {
        b'\''
    } else {
        0
    };
    if source.len() == 1 && source[0] != b' ' && (quotes != 0 || source[0] == b'~') {
        return CString::new(vec![b'\\', source[0]]).expect("source has no NUL");
    }

    let mut flags = VIS_OCTAL | VIS_CSTYLE | VIS_TAB | VIS_NL;
    if quotes == b'"' {
        flags |= VIS_DQ;
    }
    // utf8_strvis writes at most four bytes per source byte and one terminator.
    let mut escaped = vec![
        0;
        source
            .len()
            .checked_mul(4)
            .and_then(|n| n.checked_add(1))
            .expect("escaped argument too long")
    ];
    let length = utf8_strvis(escaped.as_mut_ptr().cast(), s.as_ptr(), source.len(), flags);
    escaped.truncate(length);

    let mut result = Vec::with_capacity(escaped.len() + 3);
    if quotes == b'\'' {
        result.push(b'\'');
        result.extend_from_slice(&escaped);
        result.push(b'\'');
    } else if quotes == b'"' {
        result.push(b'"');
        if escaped.first() == Some(&b'~') {
            result.push(b'\\');
        }
        result.extend_from_slice(&escaped);
        result.push(b'"');
    } else {
        if escaped.first() == Some(&b'~') {
            result.push(b'\\');
        }
        result.extend_from_slice(&escaped);
    }
    CString::new(result).expect("utf8_strvis output has no interior NUL")
}

pub unsafe fn args_escape(s: *const ::core::ffi::c_char) -> CString {
    args_escape_cstring(CStr::from_ptr(s))
}
#[no_mangle]
pub unsafe extern "C" fn args_has(mut args: *mut args, mut flag: u_char) -> ::core::ffi::c_int {
    let mut entry: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    entry = args_find(args, flag);
    if entry.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*entry).count as ::core::ffi::c_int;
}
unsafe fn args_set_value(
    args: *mut args,
    flag: u_char,
    value: Option<OwnedArgsValue>,
    flags: ::core::ffi::c_int,
) {
    let mut entry = args_find(args, flag);
    if entry.is_null() {
        entry = Box::into_raw(Box::new(::core::mem::zeroed::<args_entry>()));
        (*entry).flag = flag;
        (*entry).count = 1;
        (*entry).flags = flags;
        (*entry).values.first = ::core::ptr::null_mut();
        (*entry).values.storage = Box::into_raw(Box::new(args_values_storage::default()));
        args_tree_insert(&raw mut (*args).tree, entry);
    } else {
        (*entry).count = (*entry).count.wrapping_add(1);
    }
    if let Some(value) = value {
        let (mut boxed_value, string_owner) = value.into_parts();
        if boxed_value.type_0 as ::core::ffi::c_uint
            != ARGS_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let values_owner = args_values_ensure_owner(entry);
            let value_ptr = &mut *boxed_value as *mut args_value;
            let index = (*values_owner).values.len();
            boxed_value.entry.owner = values_owner;
            boxed_value.entry.index = index;
            if index == 0 {
                (*entry).values.first = value_ptr;
            }
            (*values_owner).values.push(boxed_value);
            (*values_owner).strings.push(string_owner);
        }
    }
}

pub unsafe fn args_set_owned_string(
    args: *mut args,
    flag: u_char,
    value: CString,
    flags: ::core::ffi::c_int,
) {
    args_set_value(args, flag, Some(OwnedArgsValue::string(value)), flags);
}

/// Add an occurrence of a flag that has no associated value.
pub unsafe fn args_set_flag(args: *mut args, flag: u_char, flags: ::core::ffi::c_int) {
    args_set_value(args, flag, None, flags);
}

/// Transfer a command-list reference into a flag value.
pub unsafe fn args_set_owned_commands(
    args: *mut args,
    flag: u_char,
    cmdlist: *mut cmd_list,
    flags: ::core::ffi::c_int,
) {
    args_set_value(args, flag, Some(OwnedArgsValue::commands(cmdlist)), flags);
}
#[no_mangle]
pub unsafe extern "C" fn args_get(
    mut args: *mut args,
    mut flag: u_char,
) -> *const ::core::ffi::c_char {
    let mut entry: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    entry = args_find(args, flag);
    if entry.is_null() {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    let value = args_last_value(args, flag);
    if value.is_none() {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    return (*value.unwrap()).c2rust_unnamed.string;
}
#[no_mangle]
pub unsafe extern "C" fn args_first(
    mut args: *mut args,
    mut entry: *mut *mut args_entry,
) -> u_char {
    *entry = args_tree_minmax(&raw mut (*args).tree, RB_NEGINF);
    if (*entry).is_null() {
        return 0 as u_char;
    }
    return (**entry).flag;
}
#[no_mangle]
pub unsafe extern "C" fn args_next(mut entry: *mut *mut args_entry) -> u_char {
    *entry = args_tree_next_from_entry(*entry);
    if (*entry).is_null() {
        return 0 as u_char;
    }
    return (**entry).flag;
}
#[no_mangle]
pub unsafe extern "C" fn args_count(mut args: *mut args) -> u_int {
    return (*args).count;
}
#[no_mangle]
pub unsafe extern "C" fn args_values(mut args: *mut args) -> *mut args_value {
    return ((*args).values).as_mut_ptr();
}
#[no_mangle]
pub unsafe extern "C" fn args_value(mut args: *mut args, mut idx: u_int) -> *mut args_value {
    if idx >= (*args).count {
        return ::core::ptr::null_mut::<args_value>();
    }
    return (*args).values.as_mut_ptr().offset(idx as isize) as *mut args_value;
}
#[no_mangle]
pub unsafe extern "C" fn args_string(
    mut args: *mut args,
    mut idx: u_int,
) -> *const ::core::ffi::c_char {
    if idx >= (*args).count {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    let value = (*args).values.as_mut_ptr().add(idx as usize);
    match (*value).type_0 as ::core::ffi::c_uint {
        0 => b"\0".as_ptr().cast(),
        1 => (*value).c2rust_unnamed.string,
        2 => {
            if !(*value).cached.is_null() {
                return (*value).cached;
            }
            let printed = cmd_list_print_cstring((*value).c2rust_unnamed.cmdlist, 0);
            let owner = &mut *args;
            let caches = &mut owner.positional_caches;
            if caches.len() <= idx as usize {
                caches.resize_with(idx as usize + 1, || None);
            }
            let pointer = printed.as_ptr().cast_mut();
            caches[idx as usize] = Some(printed);
            (*value).cached = pointer;
            pointer
        }
        _ => fatalx(b"unexpected argument type\0" as *const u8 as *const ::core::ffi::c_char),
    }
}
#[no_mangle]
pub unsafe extern "C" fn args_make_commands_now(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
    mut idx: u_int,
    mut expand: ::core::ffi::c_int,
) -> *mut cmd_list {
    let mut state: *mut args_command_state = ::core::ptr::null_mut::<args_command_state>();
    let mut cmdlist: *mut cmd_list = ::core::ptr::null_mut::<cmd_list>();
    state = args_make_commands_prepare(
        self_0,
        item,
        idx,
        ::core::ptr::null::<::core::ffi::c_char>(),
        0 as ::core::ffi::c_int,
        expand,
    );
    match args_make_commands(state, &Vec::new()) {
        Ok(commands) => cmdlist = commands,
        Err(error) => cmdq_error(
            item,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            error
                .as_ref()
                .map_or(::core::ptr::null(), |value| value.as_ptr()),
        ),
    }
    args_make_commands_free(state);
    return cmdlist;
}

#[no_mangle]
pub unsafe extern "C" fn args_make_commands_prepare(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
    mut idx: u_int,
    mut default_command: *const ::core::ffi::c_char,
    mut wait: ::core::ffi::c_int,
    mut expand: ::core::ffi::c_int,
) -> *mut args_command_state {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut value: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut state: *mut args_command_state = ::core::ptr::null_mut::<args_command_state>();
    let mut cmd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut file: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    state = Box::into_raw(Box::new(args_command_state {
        cmd: None,
        file: None,
        ..args_command_state::empty()
    }))
    .cast::<args_command_state>();
    if idx < (*args).count {
        value = (*args).values.as_mut_ptr().offset(idx as isize) as *mut args_value;
        if (*value).type_0 as ::core::ffi::c_uint
            == ARGS_COMMANDS as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*state).cmdlist = (*value).c2rust_unnamed.cmdlist as *mut cmd_list;
            (*(*state).cmdlist).references += 1;
            return state;
        }
        cmd = (*value).c2rust_unnamed.string;
    } else {
        if default_command.is_null() {
            fatalx(b"argument out of range\0" as *const u8 as *const ::core::ffi::c_char);
        }
        cmd = default_command;
    }
    if expand != 0 {
        (*state).cmd = Some(format_single_from_target_cstring(item, cmd));
    } else {
        (*state).cmd = Some(CStr::from_ptr(cmd).to_owned());
    }

    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"args_make_commands_prepare\0" as *const u8 as *const ::core::ffi::c_char,
        ((*state).cmd)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
    );
    if wait != 0 {
        (*state).pi.item = item;
    }
    cmd_get_source(self_0, &raw mut file, &raw mut (*state).pi.line);
    if !file.is_null() {
        (*state).file = Some(CStr::from_ptr(file).to_owned());
        (*state).pi.file = (*state).file.as_ref().unwrap().as_ptr() as *mut _;
    }
    (*state).pi.c = tc;
    if !(*state).pi.c.is_null() {
        (*(*state).pi.c).references += 1;
    }
    cmd_find_copy_state(&raw mut (*state).pi.fs, target);
    return state;
}
pub unsafe fn args_make_commands(
    mut state: *mut args_command_state,
    argv: &Vec<CString>,
) -> Result<*mut cmd_list, Option<CString>> {
    let mut i: ::core::ffi::c_int = 0;
    if !(*state).cmdlist.is_null() {
        if argv.is_empty() {
            (*(*state).cmdlist).references += 1;
            return Ok((*state).cmdlist);
        }
        return Ok(cmd_list_copy((*state).cmdlist, argv));
    }
    let mut cmd = CStr::from_ptr(
        ((*state).cmd)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
    )
    .to_owned();
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"args_make_commands\0" as *const u8 as *const ::core::ffi::c_char,
        cmd.as_ptr(),
    );
    cmd_log_argv(argv, c"args_make_commands");
    i = 0 as ::core::ffi::c_int;
    while (i as usize) < argv.len() {
        let next = cmd_template_replace_cstring(
            cmd.as_ptr(),
            argv[i as usize].as_ptr(),
            i + 1 as ::core::ffi::c_int,
        );
        log_debug(
            b"%s: %%%u %s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"args_make_commands\0" as *const u8 as *const ::core::ffi::c_char,
            i + 1 as ::core::ffi::c_int,
            argv[i as usize].as_ptr(),
            next.as_ptr(),
        );
        cmd = next;
        i += 1;
    }
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"args_make_commands\0" as *const u8 as *const ::core::ffi::c_char,
        cmd.as_ptr(),
    );
    let pr = cmd_parse_from_string(cmd.as_ptr(), &raw mut (*state).pi);
    drop(cmd);
    match pr.status as ::core::ffi::c_uint {
        0 => Err(pr.error),
        1 => Ok(pr.cmdlist),
        _ => fatalx(b"invalid parse return state\0" as *const u8 as *const ::core::ffi::c_char),
    }
}
#[no_mangle]
pub unsafe extern "C" fn args_make_commands_free(mut state: *mut args_command_state) {
    if !(*state).cmdlist.is_null() {
        cmd_list_free((*state).cmdlist);
    }
    if !(*state).pi.c.is_null() {
        server_client_unref((*state).pi.c);
    }
    drop(Box::from_raw(state));
}
pub unsafe fn args_make_commands_get_command(state: *mut args_command_state) -> CString {
    args_make_commands_get_command_cstring(state)
}

pub(crate) unsafe fn args_make_commands_get_command_cstring(
    state: *mut args_command_state,
) -> CString {
    if !(*state).cmdlist.is_null() {
        let first = cmd_list_first((*state).cmdlist);
        if first.is_null() {
            return CString::new(Vec::new()).expect("empty command name has no NUL");
        }
        return CStr::from_ptr((*cmd_get_entry(first)).name).to_owned();
    }
    let n = strcspn(
        ((*state).cmd)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        b" ,\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    let command = CStr::from_ptr(
        ((*state).cmd)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
    )
    .to_bytes();
    // A negative printf precision leaves the whole string untruncated.
    let prefix = if n < 0 {
        command
    } else {
        &command[..n as usize]
    };
    CString::new(prefix).expect("command prefix has no NUL")
}
#[no_mangle]
pub unsafe extern "C" fn args_first_value(
    mut args: *mut args,
    mut flag: u_char,
) -> *mut args_value {
    let mut entry: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    entry = args_find(args, flag);
    if entry.is_null() {
        return ::core::ptr::null_mut::<args_value>();
    }
    return args_value_at(entry, 0);
}
#[no_mangle]
pub unsafe extern "C" fn args_next_value(mut value: *mut args_value) -> *mut args_value {
    if value.is_null() || (*value).entry.owner.is_null() {
        return ::core::ptr::null_mut::<args_value>();
    }
    let next_index = (*value).entry.index.saturating_add(1);
    return (&(*(*value).entry.owner).values)
        .get(next_index)
        .map(|next| (&**next as *const args_value).cast_mut())
        .unwrap_or(::core::ptr::null_mut::<args_value>());
}

fn strtonum_error(errstr: *const ::core::ffi::c_char) -> ArgumentValueError {
    match unsafe { CStr::from_ptr(errstr).to_bytes() } {
        b"invalid" => ArgumentValueError::Invalid,
        b"too small" => ArgumentValueError::TooSmall,
        b"too large" => ArgumentValueError::TooLarge,
        _ => ArgumentValueError::Invalid,
    }
}

pub fn parse_number(value: &CStr, minval: i64, maxval: i64) -> Result<i64, ArgumentValueError> {
    let mut errstr = ::core::ptr::null::<::core::ffi::c_char>();
    let number = unsafe {
        strtonum(
            value.as_ptr(),
            minval as ::core::ffi::c_longlong,
            maxval as ::core::ffi::c_longlong,
            &raw mut errstr,
        )
    };
    if errstr.is_null() {
        Ok(number as i64)
    } else {
        Err(strtonum_error(errstr))
    }
}

fn percentage_share(
    curval: i64,
    percentage: i64,
    minval: i64,
    maxval: i64,
) -> Result<i64, ArgumentValueError> {
    let value = (curval as i128 * percentage as i128) / 100;
    if value < minval as i128 {
        return Err(ArgumentValueError::TooSmall);
    }
    if value > maxval as i128 {
        return Err(ArgumentValueError::TooLarge);
    }
    Ok(value as i64)
}

pub fn parse_percentage(
    value: &CStr,
    minval: i64,
    maxval: i64,
    curval: i64,
) -> Result<i64, ArgumentValueError> {
    let bytes = value.to_bytes();
    if bytes.is_empty() {
        return Err(ArgumentValueError::Empty);
    }
    if let Some(percentage) = bytes.strip_suffix(b"%") {
        let percentage = CString::new(percentage).map_err(|_| ArgumentValueError::Invalid)?;
        let percentage = parse_number(percentage.as_c_str(), 0, 1000)?;
        return percentage_share(curval, percentage, minval, maxval);
    }
    parse_number(value, minval, maxval)
}

/// Converts a percentage after expanding format expressions against `item`.
///
/// # Safety
/// `item` must be a valid command-queue item whenever a format expression is
/// expanded.
pub unsafe fn parse_percentage_and_expand(
    value: &CStr,
    minval: i64,
    maxval: i64,
    curval: i64,
    item: *mut cmdq_item,
) -> Result<i64, ArgumentValueError> {
    let bytes = value.to_bytes();
    if let Some(percentage) = bytes.strip_suffix(b"%") {
        let percentage = CString::new(percentage).map_err(|_| ArgumentValueError::Invalid)?;
        let formatted = format_single_from_target_cstring(item, percentage.as_ptr());
        let result = parse_number(formatted.as_c_str(), 0, 1000)
            .and_then(|percentage| percentage_share(curval, percentage, minval, maxval));
        return result;
    }
    let formatted = format_single_from_target_cstring(item, value.as_ptr());
    parse_number(formatted.as_c_str(), minval, maxval)
}

/// Converts the last string value stored for `flag` to a bounded integer.
///
/// # Safety
/// `args` must point to a valid argument store.
pub unsafe fn args_strtonum_result(
    args: *mut args,
    flag: u_char,
    minval: ::core::ffi::c_longlong,
    maxval: ::core::ffi::c_longlong,
) -> Result<i64, ArgumentValueError> {
    let value = args_last_string(args, flag).ok_or(ArgumentValueError::Missing)?;
    parse_number(CStr::from_ptr(value), minval, maxval)
}

/// Converts the last string value after format expansion to a bounded integer.
///
/// # Safety
/// `args` must point to a valid argument store and `item` must be valid for
/// format expansion.
pub unsafe fn args_strtonum_and_expand_result(
    args: *mut args,
    flag: u_char,
    minval: ::core::ffi::c_longlong,
    maxval: ::core::ffi::c_longlong,
    item: *mut cmdq_item,
) -> Result<i64, ArgumentValueError> {
    let value = args_last_string(args, flag).ok_or(ArgumentValueError::Missing)?;
    let value = CStr::from_ptr(value);
    let formatted = format_single_from_target_cstring(item, value.as_ptr());
    parse_number(formatted.as_c_str(), minval, maxval)
}

/// Converts the last stored string as an integer or percentage.
///
/// # Safety
/// `args` must point to a valid argument store.
pub unsafe fn args_percentage_result(
    args: *mut args,
    flag: u_char,
    minval: ::core::ffi::c_longlong,
    maxval: ::core::ffi::c_longlong,
    curval: ::core::ffi::c_longlong,
) -> Result<i64, ArgumentValueError> {
    let entry = args_find(args, flag);
    if entry.is_null() {
        return Err(ArgumentValueError::Missing);
    }
    let value = args_last_value(args, flag).ok_or(ArgumentValueError::Empty)?;
    if (*value).type_0 as ::core::ffi::c_uint
        != ARGS_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*value).c2rust_unnamed.string.is_null()
    {
        return Err(ArgumentValueError::Missing);
    }
    parse_percentage(
        CStr::from_ptr((*value).c2rust_unnamed.string),
        minval,
        maxval,
        curval,
    )
}

/// Converts the last stored string after format expansion as an integer or percentage.
///
/// # Safety
/// `args` must point to a valid argument store and `item` must be valid for
/// format expansion.
pub unsafe fn args_percentage_and_expand_result(
    args: *mut args,
    flag: u_char,
    minval: ::core::ffi::c_longlong,
    maxval: ::core::ffi::c_longlong,
    curval: ::core::ffi::c_longlong,
    item: *mut cmdq_item,
) -> Result<i64, ArgumentValueError> {
    let entry = args_find(args, flag);
    if entry.is_null() {
        return Err(ArgumentValueError::Missing);
    }
    let value = args_last_value(args, flag).ok_or(ArgumentValueError::Empty)?;
    if (*value).type_0 as ::core::ffi::c_uint
        != ARGS_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*value).c2rust_unnamed.string.is_null()
    {
        return Err(ArgumentValueError::Missing);
    }
    parse_percentage_and_expand(
        CStr::from_ptr((*value).c2rust_unnamed.string),
        minval,
        maxval,
        curval,
        item,
    )
}

/// Converts a C string as an integer or percentage.
///
/// # Safety
/// If non-null, `value` must point to a valid NUL-terminated C string.
pub unsafe fn args_string_percentage_result(
    value: *const ::core::ffi::c_char,
    minval: ::core::ffi::c_longlong,
    maxval: ::core::ffi::c_longlong,
    curval: ::core::ffi::c_longlong,
) -> Result<i64, ArgumentValueError> {
    if value.is_null() {
        return Err(ArgumentValueError::Missing);
    }
    parse_percentage(CStr::from_ptr(value), minval, maxval, curval)
}

/// Converts a C string after format expansion as an integer or percentage.
///
/// # Safety
/// If non-null, `value` must point to a valid NUL-terminated C string and
/// `item` must be valid for format expansion.
pub unsafe fn args_string_percentage_and_expand_result(
    value: *const ::core::ffi::c_char,
    minval: ::core::ffi::c_longlong,
    maxval: ::core::ffi::c_longlong,
    curval: ::core::ffi::c_longlong,
    item: *mut cmdq_item,
) -> Result<i64, ArgumentValueError> {
    if value.is_null() {
        return Err(ArgumentValueError::Missing);
    }
    parse_percentage_and_expand(CStr::from_ptr(value), minval, maxval, curval, item)
}
