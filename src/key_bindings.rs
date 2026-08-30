use crate::cmd::cmd_parse_from_string;
use crate::cmd::{cmd_list_all_have, cmd_list_print};
use crate::cmd::{
    cmdq_append, cmdq_error, cmdq_get_callback1, cmdq_get_command, cmdq_insert_after,
    cmdq_new_state,
};
use crate::fmt_args;
use crate::log::{fatalx, log_debug};
use crate::server::client_walk;
use crate::server::server_client_set_key_table;
use crate::text::key_string_lookup_key;
use crate::tree::GlobalTree;
pub use crate::types::*;
use ::core::ffi::CStr;
use ::core::ops::Bound;
use ::std::ffi::CString;
/// One key binding: the key, the commands it runs, the note that describes
/// it, the table it belongs to and whether it repeats.
///
/// The fields are the binding's own. A binding is made and replaced through
/// `key_bindings_add`, read through the `key_binding_*` accessors, and
/// changed in place only by the three setters the customize mode needs.
#[derive(Clone)]
#[repr(C)]
pub struct key_binding {
    key: key_code,
    cmdlist: Option<CmdListRef>,
    note: Option<CString>,
    tablename: Option<CString>,
    flags: ::core::ffi::c_int,
}

/// The bindings of one table, by key, which is the order the C's `tree.h`
/// comparison put them in.
pub type key_bindings = ::std::collections::BTreeMap<key_code, ::std::boxed::Box<key_binding>>;

/// One key table: the bindings the user has made, the defaults they were
/// built on, and when a key was last looked up in it.
///
/// The fields are the table's own; the walks over it are the
/// `key_bindings_first`/`key_bindings_next` pairs and the accessors below.
pub struct key_table {
    name: Option<CString>,
    activity_time: timeval,
    key_bindings: key_bindings,
    default_key_bindings: key_bindings,
}

impl key_table {
    /// An empty table under `name`.
    pub fn new(name: Option<CString>) -> key_table {
        key_table {
            name,
            activity_time: timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            key_bindings: key_bindings::new(),
            default_key_bindings: key_bindings::new(),
        }
    }
}

/// The name the table is held under, which is what `#{client_key_table}` and
/// `list-keys -T` show.
pub unsafe fn key_table_name(kt: *const key_table) -> *const ::core::ffi::c_char {
    unsafe { cstr_ptr(&(*kt).name) }
}

/// Whether the table has a default tree behind it, which is what decides
/// whether resetting it puts bindings back or takes the table away.
pub unsafe fn key_table_has_defaults(kt: *const key_table) -> bool {
    unsafe { !(*kt).default_key_bindings.is_empty() }
}

/// The table's name as a copy of its own, for a caller that keeps it after
/// the walk that found the table is over.
pub unsafe fn key_table_name_owned(kt: *const key_table) -> Option<CString> {
    unsafe { (*kt).name.clone() }
}

/// Whether the table holds no bindings of its own, which is what keeps an
/// empty table out of the customize tree.
pub unsafe fn key_table_is_empty(kt: *const key_table) -> bool {
    unsafe { (*kt).key_bindings.is_empty() }
}

/// When a key was last looked up in the table, which is what the repeat
/// timeout is measured from.
pub unsafe fn key_table_activity_time(kt: *const key_table) -> timeval {
    unsafe { (*kt).activity_time }
}

/// Says a key has just been looked up in the table.
pub unsafe fn key_table_set_activity_time(kt: *mut key_table, at: timeval) {
    unsafe { (*kt).activity_time = at };
}

/// The key the binding is for, without the flag bits a lookup carries.
pub unsafe fn key_binding_key(bd: *const key_binding) -> key_code {
    unsafe { (*bd).key }
}

/// The note written against the binding, or nothing when it has none.
pub unsafe fn key_binding_note(bd: *const key_binding) -> Option<&'static CStr> {
    unsafe { (*bd).note.as_deref() }
}

/// The binding's flags, of which `KEY_BINDING_REPEAT` is the only one.
pub unsafe fn key_binding_flags(bd: *const key_binding) -> ::core::ffi::c_int {
    unsafe { (*bd).flags }
}

/// The name of the table the binding was made in.
pub unsafe fn key_binding_tablename(bd: *const key_binding) -> *const ::core::ffi::c_char {
    unsafe { cstr_ptr(&(*bd).tablename) }
}

/// The commands the key runs, as a borrowed view for running or printing
/// them, or null before the binding has been given any.
pub unsafe fn key_binding_cmdlist(bd: *const key_binding) -> *mut cmd_list {
    unsafe {
        (*bd)
            .cmdlist
            .as_ref()
            .map_or(::core::ptr::null_mut(), CmdListRef::as_ptr)
    }
}

/// The commands the key runs, as a handle that keeps them alive past the
/// binding itself.
pub unsafe fn key_binding_cmdlist_ref(bd: *const key_binding) -> Option<CmdListRef> {
    unsafe { (*bd).cmdlist.clone() }
}

/// Whether two bindings run the same commands, which is how a binding is
/// found to be the default one already.
pub unsafe fn key_binding_same_cmdlist(bd: *const key_binding, other: *const key_binding) -> bool {
    unsafe { (*bd).cmdlist == (*other).cmdlist }
}

/// Points the binding at `cmdlist`, which is what the customize mode does
/// once the command a key runs has been retyped and parsed.
pub unsafe fn key_binding_set_cmdlist(bd: *mut key_binding, cmdlist: Option<CmdListRef>) {
    unsafe { (*bd).cmdlist = cmdlist };
}

/// Writes `note` against the binding.
pub unsafe fn key_binding_set_note(bd: *mut key_binding, note: *const ::core::ffi::c_char) {
    unsafe { (*bd).note = Some(CStr::from_ptr(note).to_owned()) };
}

/// Turns the key's repeat on if it was off and off if it was on.
pub unsafe fn key_binding_toggle_repeat(bd: *mut key_binding) {
    unsafe { (*bd).flags ^= KEY_BINDING_REPEAT };
}

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
pub const CMD_RETURN_STOP: cmd_retval = 2;
pub const CMD_RETURN_WAIT: cmd_retval = 1;
pub const CMD_RETURN_NORMAL: cmd_retval = 0;
pub const CMD_RETURN_ERROR: cmd_retval = -1;
pub const CMD_PARSE_SUCCESS: cmd_parse_status = 1;
pub const CMD_PARSE_ERROR: cmd_parse_status = 0;
pub const RB_BLACK: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RB_RED: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RB_NEGINF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const KEYC_MASK_FLAGS: ::core::ffi::c_ulonglong = 0xff000000000000 as ::core::ffi::c_ulonglong;
pub const CMDQ_STATE_REPEAT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CMD_READONLY: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CLIENT_READONLY: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const KEY_BINDING_REPEAT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
/// The live key-binding tables, by name. A client keeps a strong handle after
/// the table is removed from this registry.
static key_tables: GlobalTree<CString, KeyTableRef> = GlobalTree::new();
pub(crate) unsafe fn key_bindings_get_table_ref(
    name: *const ::core::ffi::c_char,
    create: ::core::ffi::c_int,
) -> Option<KeyTableRef> {
    unsafe {
        if let Some(table) = key_tables.map().get(CStr::from_ptr(name)) {
            return Some(table.clone());
        }
        if create == 0 {
            return None;
        }
        let name_cstr = CStr::from_ptr(name).to_owned();
        let table = KeyTableRef::new(key_table::new(Some(name_cstr.clone())));
        key_tables.map().insert(name_cstr, table.clone());
        Some(table)
    }
}
pub unsafe fn key_bindings_get_table(
    name: *const ::core::ffi::c_char,
    create: ::core::ffi::c_int,
) -> *mut key_table {
    unsafe {
        key_bindings_get_table_ref(name, create)
            .map(|table| table.as_ptr())
            .unwrap_or(::core::ptr::null_mut::<key_table>())
    }
}
pub fn key_bindings_first_table() -> *mut key_table {
    key_tables
        .map()
        .values()
        .next()
        .map(KeyTableRef::as_ptr)
        .unwrap_or(::core::ptr::null_mut::<key_table>())
}
pub unsafe fn key_bindings_next_table(table: *mut key_table) -> *mut key_table {
    unsafe {
        key_tables
            .map()
            .range::<CStr, _>((
                Bound::Excluded(CStr::from_ptr(cstr_ptr(&(*table).name))),
                Bound::Unbounded,
            ))
            .next()
            .map(|(_, table)| table.as_ptr())
            .unwrap_or(::core::ptr::null_mut::<key_table>())
    }
}
pub unsafe fn key_bindings_get(table: *mut key_table, key: key_code) -> *mut key_binding {
    unsafe {
        (*table)
            .key_bindings
            .get(&key)
            .map(|bd| bd.as_ref() as *const key_binding as *mut key_binding)
            .unwrap_or(::core::ptr::null_mut::<key_binding>())
    }
}
pub unsafe fn key_bindings_get_default(table: *mut key_table, key: key_code) -> *mut key_binding {
    unsafe {
        (*table)
            .default_key_bindings
            .get(&key)
            .map(|bd| bd.as_ref() as *const key_binding as *mut key_binding)
            .unwrap_or(::core::ptr::null_mut::<key_binding>())
    }
}
pub unsafe fn key_bindings_first(table: *mut key_table) -> *mut key_binding {
    unsafe {
        (*table)
            .key_bindings
            .values()
            .next()
            .map(|bd| bd.as_ref() as *const key_binding as *mut key_binding)
            .unwrap_or(::core::ptr::null_mut::<key_binding>())
    }
}
pub unsafe fn key_bindings_next(table: *mut key_table, bd: *mut key_binding) -> *mut key_binding {
    unsafe {
        (*table)
            .key_bindings
            .range((Bound::Excluded((*bd).key), Bound::Unbounded))
            .next()
            .map(|(_, bd)| bd.as_ref() as *const key_binding as *mut key_binding)
            .unwrap_or(::core::ptr::null_mut::<key_binding>())
    }
}
pub(crate) unsafe fn key_bindings_add(
    mut name: *const ::core::ffi::c_char,
    mut key: key_code,
    mut note: *const ::core::ffi::c_char,
    mut repeat: ::core::ffi::c_int,
    mut cmdlist: Option<CmdListRef>,
) {
    unsafe {
        let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
        let Some(table_ref) = key_bindings_get_table_ref(name, 1 as ::core::ffi::c_int) else {
            return;
        };
        let table = table_ref.as_ptr();
        bd = key_bindings_get(table, key & !KEYC_MASK_FLAGS);
        if cmdlist.is_none() {
            if !bd.is_null() {
                if !note.is_null() {
                    (*bd).note = Some(CStr::from_ptr(note).to_owned());
                }
                if repeat != 0 {
                    (*bd).flags |= KEY_BINDING_REPEAT;
                }
            }
            return;
        }
        if !bd.is_null() {
            let _ = (*table).key_bindings.remove(&(*bd).key);
        }
        let mut bd_box = Box::new(key_binding {
            key: (key as ::core::ffi::c_ulonglong & !KEYC_MASK_FLAGS) as key_code,
            cmdlist: None,
            note: if !note.is_null() {
                Some(CStr::from_ptr(note).to_owned())
            } else {
                None
            },
            tablename: (*table).name.clone(),
            flags: 0,
        });
        bd = &raw mut *bd_box;
        (*table).key_bindings.insert((*bd).key, bd_box);
        if repeat != 0 {
            (*bd).flags |= KEY_BINDING_REPEAT;
        }
        (*bd).cmdlist = cmdlist;
        let s = cmd_list_print(
            (*bd).cmdlist.as_ref().unwrap().as_ptr(),
            0 as ::core::ffi::c_int,
        );
        log_debug(
            c"%s: %#llx %s = %s".as_ptr(),
            fmt_args![
                c"key_bindings_add".as_ptr(),
                (*bd).key,
                key_string_lookup_key((*bd).key, 1 as ::core::ffi::c_int),
                s.as_ptr()
            ],
        );
    }
}
pub unsafe fn key_bindings_remove(mut name: *const ::core::ffi::c_char, mut key: key_code) {
    unsafe {
        let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
        let Some(table_ref) = key_bindings_get_table_ref(name, 0 as ::core::ffi::c_int) else {
            return;
        };
        let table = table_ref.as_ptr();
        bd = key_bindings_get(table, key & !KEYC_MASK_FLAGS);
        if bd.is_null() {
            return;
        }
        log_debug(
            c"%s: %#llx %s".as_ptr(),
            fmt_args![
                c"key_bindings_remove".as_ptr(),
                (*bd).key,
                key_string_lookup_key((*bd).key, 1 as ::core::ffi::c_int)
            ],
        );
        let _ = (*table).key_bindings.remove(&(*bd).key);
        if (*table).key_bindings.is_empty() && !key_table_has_defaults(table) {
            key_tables
                .map()
                .remove(CStr::from_ptr(cstr_ptr(&(*table).name)));
        }
    }
}
pub unsafe fn key_bindings_reset(mut name: *const ::core::ffi::c_char, mut key: key_code) {
    unsafe {
        let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
        let mut dd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
        let Some(table_ref) = key_bindings_get_table_ref(name, 0 as ::core::ffi::c_int) else {
            return;
        };
        let table = table_ref.as_ptr();
        bd = key_bindings_get(table, key & !KEYC_MASK_FLAGS);
        if bd.is_null() {
            return;
        }
        dd = key_bindings_get_default(table, (*bd).key);
        if dd.is_null() {
            key_bindings_remove(name, (*bd).key);
            return;
        }
        (*bd).cmdlist = (*dd).cmdlist.clone();
        (*bd).note = (*dd).note.clone();
        (*bd).flags = (*dd).flags;
    }
}
pub unsafe fn key_bindings_remove_table(mut name: *const ::core::ffi::c_char) {
    unsafe {
        let Some(table_ref) = key_bindings_get_table_ref(name, 0 as ::core::ffi::c_int) else {
            return;
        };
        let table = table_ref.as_ptr();
        if key_tables
            .map()
            .remove(CStr::from_ptr(cstr_ptr(&(*table).name)))
            .is_some()
        {
            for c in client_walk() {
                if (*c).keytable() == table {
                    server_client_set_key_table(c, ::core::ptr::null::<::core::ffi::c_char>());
                }
            }
        }
    }
}
pub unsafe fn key_bindings_reset_table(mut name: *const ::core::ffi::c_char) {
    unsafe {
        let Some(table_ref) = key_bindings_get_table_ref(name, 0 as ::core::ffi::c_int) else {
            return;
        };
        let table = table_ref.as_ptr();
        if !key_table_has_defaults(table) {
            key_bindings_remove_table(name);
            return;
        }
        let keys: Vec<key_code> = (*table).key_bindings.keys().copied().collect();
        for key in keys {
            key_bindings_reset(name, key);
        }
    }
}
/// Copies what `table` holds now into its default tree, which is what makes
/// those bindings the ones `key_bindings_reset` puts back. The server does
/// this once the bindings it starts with have been parsed.
pub(crate) unsafe fn key_bindings_take_defaults(table: *mut key_table) {
    unsafe {
        let bindings: Vec<*mut key_binding> = (*table)
            .key_bindings
            .values()
            .map(|bd| bd.as_ref() as *const key_binding as *mut key_binding)
            .collect();
        for bd in bindings {
            let new_bd = Box::new(key_binding {
                key: (*bd).key,
                cmdlist: (*bd).cmdlist.clone(),
                note: (*bd).note.clone(),
                tablename: None,
                flags: (*bd).flags,
            });
            (*table).default_key_bindings.insert(new_bd.key, new_bd);
        }
    }
}

unsafe fn key_bindings_init_done(_item: *mut cmdq_item, _data: CmdqCallbackData) -> cmd_retval {
    unsafe {
        let tables: Vec<KeyTableRef> = key_tables.map().values().cloned().collect();
        for table_ref in tables {
            key_bindings_take_defaults(table_ref.as_ptr());
        }
        CMD_RETURN_NORMAL
    }
}
pub fn key_bindings_init() {
    unsafe {
        static defaults: ReadOnly<[*const ::core::ffi::c_char; 275]> = ReadOnly::new([
        c"bind -N 'Send the prefix key' C-b { send-prefix }".as_ptr(),
        c"bind -N 'Rotate through the panes' C-o { rotate-window }".as_ptr(),
        c"bind -N 'Suspend the current client' C-z { suspend-client }".as_ptr(),
        c"bind -N 'Select next layout' Space { next-layout }".as_ptr(),
        c"bind -N 'Break pane to a new window' ! { break-pane }".as_ptr(),
        c"bind -N 'Split window vertically' '\"' { split-window }".as_ptr(),
        c"bind -N 'List all paste buffers' '#' { list-buffers }".as_ptr(),
        c"bind -N 'Rename current session' '$' { command-prompt -I'#S' { rename-session -- '%%' } }".as_ptr(),
        c"bind -N 'Split window horizontally' % { split-window -h }".as_ptr(),
        c"bind -N 'Kill current window' & { confirm-before -p\"kill-window #W? (y/n)\" kill-window }".as_ptr(),
        c"bind -N 'Prompt for window index to select' \"'\" { command-prompt -T window-target -pindex { select-window -t ':%%' } }".as_ptr(),
        c"bind -N 'New floating pane' * { new-pane }".as_ptr(),
        c"bind -N 'Switch to previous client' ( { switch-client -p }".as_ptr(),
        c"bind -N 'Switch to next client' ) { switch-client -n }".as_ptr(),
        c"bind -N 'Rename current window' , { command-prompt -I'#W' { rename-window -- '%%' } }".as_ptr(),
        c"bind -N 'Delete the most recent paste buffer' - { delete-buffer }".as_ptr(),
        c"bind -N 'Move the current window' . { command-prompt -T target { move-window -t '%%' } }".as_ptr(),
        c"bind -N 'Describe key binding' '/' { command-prompt -kpkey  { list-keys -1N '%%' } }".as_ptr(),
        c"bind -N 'Select window 0' 0 { select-window -t:=0 }".as_ptr(),
        c"bind -N 'Select window 1' 1 { select-window -t:=1 }".as_ptr(),
        c"bind -N 'Select window 2' 2 { select-window -t:=2 }".as_ptr(),
        c"bind -N 'Select window 3' 3 { select-window -t:=3 }".as_ptr(),
        c"bind -N 'Select window 4' 4 { select-window -t:=4 }".as_ptr(),
        c"bind -N 'Select window 5' 5 { select-window -t:=5 }".as_ptr(),
        c"bind -N 'Select window 6' 6 { select-window -t:=6 }".as_ptr(),
        c"bind -N 'Select window 7' 7 { select-window -t:=7 }".as_ptr(),
        c"bind -N 'Select window 8' 8 { select-window -t:=8 }".as_ptr(),
        c"bind -N 'Select window 9' 9 { select-window -t:=9 }".as_ptr(),
        c"bind -N 'Prompt for a command' : { command-prompt }".as_ptr(),
        c"bind -N 'Move to the previously active pane' \\; { last-pane }".as_ptr(),
        c"bind -N 'Choose a paste buffer from a list' = { choose-buffer -Z }".as_ptr(),
        c"bind -N 'List key bindings' ? { list-keys -N }".as_ptr(),
        c"bind -N 'Choose and detach a client from a list' D { choose-client -Z }".as_ptr(),
        c"bind -N 'Spread panes out evenly' E { select-layout -E }".as_ptr(),
        c"bind -N 'Switch to the last client' L { switch-client -l }".as_ptr(),
        c"bind -N 'Clear the marked pane' M { select-pane -M }".as_ptr(),
        c"bind -N 'Enter copy mode' [ { copy-mode }".as_ptr(),
        c"bind -N 'Paste the most recent paste buffer' ] { paste-buffer -p }".as_ptr(),
        c"bind -N 'Create a new window' c { new-window }".as_ptr(),
        c"bind -N 'Detach the current client' d { detach-client }".as_ptr(),
        c"bind -N 'Search for a pane' f { command-prompt { find-window -Z -- '%%' } }".as_ptr(),
        c"bind -N 'Display window information' i { display-message }".as_ptr(),
        c"bind -N 'Select the previously current window' l { last-window }".as_ptr(),
        c"bind -N 'Toggle the marked pane' m { select-pane -m }".as_ptr(),
        c"bind -N 'Select the next window' n { next-window }".as_ptr(),
        c"bind -N 'Select the next pane' o { select-pane -t:.+ }".as_ptr(),
        c"bind -N 'Customize options' C { customize-mode -Z }".as_ptr(),
        c"bind -N 'Select the previous window' p { previous-window }".as_ptr(),
        c"bind -N 'Display pane numbers' q { display-panes }".as_ptr(),
        c"bind -N 'Redraw the current client' r { refresh-client }".as_ptr(),
        c"bind -N 'Choose a session from a list' s { choose-tree -Zs }".as_ptr(),
        c"bind -N 'Show a clock' t { clock-mode }".as_ptr(),
        c"bind -N 'Choose a window from a list' w { choose-tree -Zw }".as_ptr(),
        c"bind -N 'Kill the active pane' x { confirm-before -p\"kill-pane #P? (y/n)\" kill-pane }".as_ptr(),
        c"bind -N 'Zoom the active pane' z { resize-pane -Z }".as_ptr(),
        c"bind -N 'Swap the active pane with the pane above' '{' { swap-pane -U }".as_ptr(),
        c"bind -N 'Swap the active pane with the pane below' '}' { swap-pane -D }".as_ptr(),
        c"bind -N 'Show messages' '~' { show-messages }".as_ptr(),
        c"bind -N 'Enter copy mode and scroll up' PPage { copy-mode -u }".as_ptr(),
        c"bind -N 'Select the pane above the active pane' -r Up { select-pane -U }".as_ptr(),
        c"bind -N 'Select the pane below the active pane' -r Down { select-pane -D }".as_ptr(),
        c"bind -N 'Select the pane to the left of the active pane' -r Left { select-pane -L }".as_ptr(),
        c"bind -N 'Select the pane to the right of the active pane' -r Right { select-pane -R }".as_ptr(),
        c"bind -N 'Set the even-horizontal layout' M-1 { select-layout even-horizontal }".as_ptr(),
        c"bind -N 'Set the even-vertical layout' M-2 { select-layout even-vertical }".as_ptr(),
        c"bind -N 'Set the main-horizontal layout' M-3 { select-layout main-horizontal }".as_ptr(),
        c"bind -N 'Set the main-vertical layout' M-4 { select-layout main-vertical }".as_ptr(),
        c"bind -N 'Select the tiled layout' M-5 { select-layout tiled }".as_ptr(),
        c"bind -N 'Set the main-horizontal-mirrored layout' M-6 { select-layout main-horizontal-mirrored }".as_ptr(),
        c"bind -N 'Set the main-vertical-mirrored layout' M-7 { select-layout main-vertical-mirrored }".as_ptr(),
        c"bind -N 'Select the next window with an alert' M-n { next-window -a }".as_ptr(),
        c"bind -N 'Rotate through the panes in reverse' M-o { rotate-window -D }".as_ptr(),
        c"bind -N 'Select the previous window with an alert' M-p { previous-window -a }".as_ptr(),
        c"bind -N 'Move the visible part of the window up' -r S-Up { refresh-client -U 10 }".as_ptr(),
        c"bind -N 'Move the visible part of the window down' -r S-Down { refresh-client -D 10 }".as_ptr(),
        c"bind -N 'Move the visible part of the window left' -r S-Left { refresh-client -L 10 }".as_ptr(),
        c"bind -N 'Move the visible part of the window right' -r S-Right { refresh-client -R 10 }".as_ptr(),
        c"bind -N 'Reset so the visible part of the window follows the cursor' -r DC { refresh-client -c }".as_ptr(),
        c"bind -N 'Resize the pane up by 5' -r M-Up { resize-pane -U 5 }".as_ptr(),
        c"bind -N 'Resize the pane down by 5' -r M-Down { resize-pane -D 5 }".as_ptr(),
        c"bind -N 'Resize the pane left by 5' -r M-Left { resize-pane -L 5 }".as_ptr(),
        c"bind -N 'Resize the pane right by 5' -r M-Right { resize-pane -R 5 }".as_ptr(),
        c"bind -N 'Resize the pane up' -r C-Up { resize-pane -U }".as_ptr(),
        c"bind -N 'Resize the pane down' -r C-Down { resize-pane -D }".as_ptr(),
        c"bind -N 'Resize the pane left' -r C-Left { resize-pane -L }".as_ptr(),
        c"bind -N 'Resize the pane right' -r C-Right { resize-pane -R }".as_ptr(),
        c"bind -N 'Display window menu' < { display-menu -xW -yW -T '#[align=centre]#{window_index}:#{window_name}'  '#{?#{>:#{session_windows},1},,-}Swap Left' 'l' {swap-window -t:-1} '#{?#{>:#{session_windows},1},,-}Swap Right' 'r' {swap-window -t:+1} '#{?pane_marked_set,,-}Swap Marked' 's' {swap-window} '' 'Kill' 'X' {kill-window} 'Respawn' 'R' {respawn-window -k} '#{?pane_marked,Unmark,Mark}' 'm' {select-pane -m} 'Rename' 'n' {command-prompt -FI \"#W\" {rename-window -t '#{window_id}' -- '%%'}} '' 'New After' 'w' {new-window -a} 'New At End' 'W' {new-window} }".as_ptr(),
        c"bind -N 'Display pane menu' > { display-menu -xP -yP -T '#[align=centre]#{pane_index} (#{pane_id})'  '#{?#{m/r:(copy|view)-mode,#{pane_mode}},Go To Top,}' '<' {send -X history-top} '#{?#{m/r:(copy|view)-mode,#{pane_mode}},Go To Bottom,}' '>' {send -X history-bottom} '' '#{?#{&&:#{buffer_size},#{!:#{pane_in_mode}}},Paste #[underscore]#{=/9/...:buffer_sample},}' 'p' {paste-buffer} '' '#{?mouse_word,Search For #[underscore]#{=/9/...:mouse_word},}' 'C-r' {if -F '#{?#{m/r:(copy|view)-mode,#{pane_mode}},0,1}' 'copy-mode -t='; send -Xt= search-backward -- \"#{q:mouse_word}\"} '#{?mouse_word,Type #[underscore]#{=/9/...:mouse_word},}' 'C-y' {copy-mode -q; send-keys -l -- \"#{q:mouse_word}\"} '#{?mouse_word,Copy #[underscore]#{=/9/...:mouse_word},}' 'c' {copy-mode -q; set-buffer -- \"#{q:mouse_word}\"} '#{?mouse_line,Copy Line,}' 'l' {copy-mode -q; set-buffer -- \"#{q:mouse_line}\"} '' '#{?mouse_hyperlink,Type #[underscore]#{=/9/...:mouse_hyperlink},}' 'C-h' {copy-mode -q; send-keys -l -- \"#{q:mouse_hyperlink}\"} '#{?mouse_hyperlink,Copy #[underscore]#{=/9/...:mouse_hyperlink},}' 'h' {copy-mode -q; set-buffer -- \"#{q:mouse_hyperlink}\"} '' '#{?#{!:#{pane_floating_flag}},Horizontal Split,}' 'h' {split-window -h} '#{?#{!:#{pane_floating_flag}},Vertical Split,}' 'v' {split-window -v} '' '#{?#{&&:#{!:#{pane_floating_flag}},#{>:#{window_panes},1}},Swap Up,}' 'u' {swap-pane -U} '#{?#{&&:#{!:#{pane_floating_flag}},#{>:#{window_panes},1}},Swap Down,}' 'd' {swap-pane -D} '#{?pane_marked_set,,-}Swap Marked' 's' {swap-pane} '' 'Kill' 'X' {kill-pane} 'Respawn' 'R' {respawn-pane -k} '#{?pane_marked,Unmark,Mark}' 'm' {select-pane -m} '#{?#{>:#{window_panes},1},,-}#{?window_zoomed_flag,Unzoom,Zoom}' 'z' {resize-pane -Z} }".as_ptr(),
        c"bind -n MouseDown1Pane { select-pane -t=; send -M }".as_ptr(),
        c"bind -n C-MouseDown1Pane { swap-pane -s@ }".as_ptr(),
        c"bind -n MouseDrag1Pane { if -F '#{||:#{pane_in_mode},#{mouse_any_flag}}' { send -M } { copy-mode -M } }".as_ptr(),
        c"bind -n WheelUpPane { if -F '#{||:#{alternate_on},#{pane_in_mode},#{mouse_any_flag}}' { send -M } { copy-mode -e } }".as_ptr(),
        c"bind -n MouseDown2Pane { select-pane -t=; if -F '#{||:#{pane_in_mode},#{mouse_any_flag}}' { send -M } { paste -p } }".as_ptr(),
        c"bind -n DoubleClick1Pane { select-pane -t=; if -F '#{||:#{pane_in_mode},#{mouse_any_flag}}' { send -M } { copy-mode -H; send -X select-word; run -d0.3; send -X copy-pipe-and-cancel } }".as_ptr(),
        c"bind -n TripleClick1Pane { select-pane -t=; if -F '#{||:#{pane_in_mode},#{mouse_any_flag}}' { send -M } { copy-mode -H; send -X select-line; run -d0.3; send -X copy-pipe-and-cancel } }".as_ptr(),
        c"bind -n MouseDown1Border { select-pane -M }".as_ptr(),
        c"bind -n MouseDrag1Border { resize-pane -M }".as_ptr(),
        c"bind -n MouseDown1Status { switch-client -t= }".as_ptr(),
        c"bind -n C-MouseDown1Status { swap-window -t@ }".as_ptr(),
        c"bind -n MouseDown1Control9 { display-menu -t= -xM -yM -O -T 'Kill pane #{pane_index}?' 'Yes' 'y' { kill-pane -t= } 'No' 'n' {}}".as_ptr(),
        c"bind -n MouseDown1Control8 { resize-pane -Z }".as_ptr(),
        c"bind -n WheelDownStatus { next-window }".as_ptr(),
        c"bind -n WheelUpStatus { previous-window }".as_ptr(),
        c"bind -n MouseDown3StatusLeft { display-menu -t= -xM -yW -T '#[align=centre]#{session_name}'  'Next' 'n' {switch-client -n} 'Previous' 'p' {switch-client -p} '' 'Renumber' 'N' {move-window -r} 'Rename' 'r' {command-prompt -I \"#S\" {rename-session -- '%%'}} 'Detach' 'd' {detach-client} '' 'New Session' 's' {new-session} 'New Window' 'w' {new-window} }".as_ptr(),
        c"bind -n M-MouseDown3StatusLeft { display-menu -t= -xM -yW -T '#[align=centre]#{session_name}'  'Next' 'n' {switch-client -n} 'Previous' 'p' {switch-client -p} '' 'Renumber' 'N' {move-window -r} 'Rename' 'r' {command-prompt -I \"#S\" {rename-session -- '%%'}} 'Detach' 'd' {detach-client} '' 'New Session' 's' {new-session} 'New Window' 'w' {new-window} }".as_ptr(),
        c"bind -n MouseDown3Status { display-menu -t= -xW -yW -T '#[align=centre]#{window_index}:#{window_name}'  '#{?#{>:#{session_windows},1},,-}Swap Left' 'l' {swap-window -t:-1} '#{?#{>:#{session_windows},1},,-}Swap Right' 'r' {swap-window -t:+1} '#{?pane_marked_set,,-}Swap Marked' 's' {swap-window} '' 'Kill' 'X' {kill-window} 'Respawn' 'R' {respawn-window -k} '#{?pane_marked,Unmark,Mark}' 'm' {select-pane -m} 'Rename' 'n' {command-prompt -FI \"#W\" {rename-window -t '#{window_id}' -- '%%'}} '' 'New After' 'w' {new-window -a} 'New At End' 'W' {new-window}}".as_ptr(),
        c"bind -n M-MouseDown3Status { display-menu -t= -xW -yW -T '#[align=centre]#{window_index}:#{window_name}'  '#{?#{>:#{session_windows},1},,-}Swap Left' 'l' {swap-window -t:-1} '#{?#{>:#{session_windows},1},,-}Swap Right' 'r' {swap-window -t:+1} '#{?pane_marked_set,,-}Swap Marked' 's' {swap-window} '' 'Kill' 'X' {kill-window} 'Respawn' 'R' {respawn-window -k} '#{?pane_marked,Unmark,Mark}' 'm' {select-pane -m} 'Rename' 'n' {command-prompt -FI \"#W\" {rename-window -t '#{window_id}' -- '%%'}} '' 'New After' 'w' {new-window -a} 'New At End' 'W' {new-window}}".as_ptr(),
        c"bind -n MouseDown3Pane { if -Ft= '#{||:#{mouse_any_flag},#{&&:#{pane_in_mode},#{?#{m/r:(copy|view)-mode,#{pane_mode}},0,1}}}' { select-pane -t=; send -M } { display-menu -t= -xM -yM -T '#[align=centre]#{pane_index} (#{pane_id})'  '#{?#{m/r:(copy|view)-mode,#{pane_mode}},Go To Top,}' '<' {send -X history-top} '#{?#{m/r:(copy|view)-mode,#{pane_mode}},Go To Bottom,}' '>' {send -X history-bottom} '' '#{?#{&&:#{buffer_size},#{!:#{pane_in_mode}}},Paste #[underscore]#{=/9/...:buffer_sample},}' 'p' {paste-buffer} '' '#{?mouse_word,Search For #[underscore]#{=/9/...:mouse_word},}' 'C-r' {if -F '#{?#{m/r:(copy|view)-mode,#{pane_mode}},0,1}' 'copy-mode -t='; send -Xt= search-backward -- \"#{q:mouse_word}\"} '#{?mouse_word,Type #[underscore]#{=/9/...:mouse_word},}' 'C-y' {copy-mode -q; send-keys -l -- \"#{q:mouse_word}\"} '#{?mouse_word,Copy #[underscore]#{=/9/...:mouse_word},}' 'c' {copy-mode -q; set-buffer -- \"#{q:mouse_word}\"} '#{?mouse_line,Copy Line,}' 'l' {copy-mode -q; set-buffer -- \"#{q:mouse_line}\"} '' '#{?mouse_hyperlink,Type #[underscore]#{=/9/...:mouse_hyperlink},}' 'C-h' {copy-mode -q; send-keys -l -- \"#{q:mouse_hyperlink}\"} '#{?mouse_hyperlink,Copy #[underscore]#{=/9/...:mouse_hyperlink},}' 'h' {copy-mode -q; set-buffer -- \"#{q:mouse_hyperlink}\"} '' '#{?#{!:#{pane_floating_flag}},Horizontal Split,}' 'h' {split-window -h} '#{?#{!:#{pane_floating_flag}},Vertical Split,}' 'v' {split-window -v} '' '#{?#{&&:#{!:#{pane_floating_flag}},#{>:#{window_panes},1}},Swap Up,}' 'u' {swap-pane -U} '#{?#{&&:#{!:#{pane_floating_flag}},#{>:#{window_panes},1}},Swap Down,}' 'd' {swap-pane -D} '#{?pane_marked_set,,-}Swap Marked' 's' {swap-pane} '' 'Kill' 'X' {kill-pane} 'Respawn' 'R' {respawn-pane -k} '#{?pane_marked,Unmark,Mark}' 'm' {select-pane -m} '#{?#{>:#{window_panes},1},,-}#{?window_zoomed_flag,Unzoom,Zoom}' 'z' {resize-pane -Z} } }".as_ptr(),
        c"bind -n M-MouseDown3Pane { display-menu -t= -xM -yM -T '#[align=centre]#{pane_index} (#{pane_id})'  '#{?#{m/r:(copy|view)-mode,#{pane_mode}},Go To Top,}' '<' {send -X history-top} '#{?#{m/r:(copy|view)-mode,#{pane_mode}},Go To Bottom,}' '>' {send -X history-bottom} '' '#{?#{&&:#{buffer_size},#{!:#{pane_in_mode}}},Paste #[underscore]#{=/9/...:buffer_sample},}' 'p' {paste-buffer} '' '#{?mouse_word,Search For #[underscore]#{=/9/...:mouse_word},}' 'C-r' {if -F '#{?#{m/r:(copy|view)-mode,#{pane_mode}},0,1}' 'copy-mode -t='; send -Xt= search-backward -- \"#{q:mouse_word}\"} '#{?mouse_word,Type #[underscore]#{=/9/...:mouse_word},}' 'C-y' {copy-mode -q; send-keys -l -- \"#{q:mouse_word}\"} '#{?mouse_word,Copy #[underscore]#{=/9/...:mouse_word},}' 'c' {copy-mode -q; set-buffer -- \"#{q:mouse_word}\"} '#{?mouse_line,Copy Line,}' 'l' {copy-mode -q; set-buffer -- \"#{q:mouse_line}\"} '' '#{?mouse_hyperlink,Type #[underscore]#{=/9/...:mouse_hyperlink},}' 'C-h' {copy-mode -q; send-keys -l -- \"#{q:mouse_hyperlink}\"} '#{?mouse_hyperlink,Copy #[underscore]#{=/9/...:mouse_hyperlink},}' 'h' {copy-mode -q; set-buffer -- \"#{q:mouse_hyperlink}\"} '' '#{?#{!:#{pane_floating_flag}},Horizontal Split,}' 'h' {split-window -h} '#{?#{!:#{pane_floating_flag}},Vertical Split,}' 'v' {split-window -v} '' '#{?#{&&:#{!:#{pane_floating_flag}},#{>:#{window_panes},1}},Swap Up,}' 'u' {swap-pane -U} '#{?#{&&:#{!:#{pane_floating_flag}},#{>:#{window_panes},1}},Swap Down,}' 'd' {swap-pane -D} '#{?pane_marked_set,,-}Swap Marked' 's' {swap-pane} '' 'Kill' 'X' {kill-pane} 'Respawn' 'R' {respawn-pane -k} '#{?pane_marked,Unmark,Mark}' 'm' {select-pane -m} '#{?#{>:#{window_panes},1},,-}#{?window_zoomed_flag,Unzoom,Zoom}' 'z' {resize-pane -Z} }".as_ptr(),
        c"bind -n MouseDown1ScrollbarUp { if -Ft= '#{pane_in_mode}' { send -X page-up } {copy-mode -u } }".as_ptr(),
        c"bind -n MouseDown1ScrollbarDown { if -Ft= '#{pane_in_mode}' { send -X page-down } {copy-mode -d } }".as_ptr(),
        c"bind -n MouseDrag1ScrollbarSlider { if -Ft= '#{pane_in_mode}' { send -X scroll-to-mouse } { copy-mode -S } }".as_ptr(),
        c"bind -Tcopy-mode C-Space { send -X begin-selection }".as_ptr(),
        c"bind -Tcopy-mode C-a { send -X start-of-line }".as_ptr(),
        c"bind -Tcopy-mode C-c { send -X cancel }".as_ptr(),
        c"bind -Tcopy-mode C-e { send -X end-of-line }".as_ptr(),
        c"bind -Tcopy-mode C-f { send -X cursor-right }".as_ptr(),
        c"bind -Tcopy-mode C-b { send -X cursor-left }".as_ptr(),
        c"bind -Tcopy-mode C-g { send -X clear-selection }".as_ptr(),
        c"bind -Tcopy-mode C-k { send -X copy-pipe-end-of-line-and-cancel }".as_ptr(),
        c"bind -Tcopy-mode C-l { send -X recentre-top-bottom }".as_ptr(),
        c"bind -Tcopy-mode M-l { send -X cursor-centre-horizontal }".as_ptr(),
        c"bind -Tcopy-mode C-n { send -X cursor-down }".as_ptr(),
        c"bind -Tcopy-mode C-p { send -X cursor-up }".as_ptr(),
        c"bind -Tcopy-mode C-r { command-prompt -T search -ip'(search up)' -I'#{pane_search_string}' { send -X search-backward-incremental -- '%%' } }".as_ptr(),
        c"bind -Tcopy-mode C-s { command-prompt -T search -ip'(search down)' -I'#{pane_search_string}' { send -X search-forward-incremental -- '%%' } }".as_ptr(),
        c"bind -Tcopy-mode C-v { send -X page-down }".as_ptr(),
        c"bind -Tcopy-mode C-w { send -X copy-pipe-and-cancel }".as_ptr(),
        c"bind -Tcopy-mode Escape { send -X cancel }".as_ptr(),
        c"bind -Tcopy-mode C-[ { send -X cancel }".as_ptr(),
        c"bind -Tcopy-mode Space { send -X page-down }".as_ptr(),
        c"bind -Tcopy-mode , { send -X jump-reverse }".as_ptr(),
        c"bind -Tcopy-mode \\; { send -X jump-again }".as_ptr(),
        c"bind -Tcopy-mode F { command-prompt -1p'(jump backward)' { send -X jump-backward -- '%%' } }".as_ptr(),
        c"bind -Tcopy-mode N { send -X search-reverse }".as_ptr(),
        c"bind -Tcopy-mode P { send -X toggle-position }".as_ptr(),
        c"bind -Tcopy-mode R { send -X rectangle-toggle }".as_ptr(),
        c"bind -Tcopy-mode T { command-prompt -1p'(jump to backward)' { send -X jump-to-backward -- '%%' } }".as_ptr(),
        c"bind -Tcopy-mode X { send -X set-mark }".as_ptr(),
        c"bind -Tcopy-mode f { command-prompt -1p'(jump forward)' { send -X jump-forward -- '%%' } }".as_ptr(),
        c"bind -Tcopy-mode g { command-prompt -p'(goto line)' { send -X goto-line -- '%%' } }".as_ptr(),
        c"bind -Tcopy-mode n { send -X search-again }".as_ptr(),
        c"bind -Tcopy-mode q { send -X cancel }".as_ptr(),
        c"bind -Tcopy-mode r { send -X refresh-from-pane }".as_ptr(),
        c"bind -Tcopy-mode t { command-prompt -1p'(jump to forward)' { send -X jump-to-forward -- '%%' } }".as_ptr(),
        c"bind -Tcopy-mode Home { send -X start-of-line }".as_ptr(),
        c"bind -Tcopy-mode End { send -X end-of-line }".as_ptr(),
        c"bind -Tcopy-mode MouseDown1Pane select-pane".as_ptr(),
        c"bind -Tcopy-mode MouseDrag1Pane { select-pane; send -X begin-selection }".as_ptr(),
        c"bind -Tcopy-mode MouseDragEnd1Pane { send -X copy-pipe-and-cancel }".as_ptr(),
        c"bind -Tcopy-mode WheelUpPane { select-pane; send -N5 -X scroll-up }".as_ptr(),
        c"bind -Tcopy-mode WheelDownPane { select-pane; send -N5 -X scroll-down }".as_ptr(),
        c"bind -Tcopy-mode DoubleClick1Pane { select-pane; send -X select-word; run -d0.3; send -X copy-pipe-and-cancel }".as_ptr(),
        c"bind -Tcopy-mode TripleClick1Pane { select-pane; send -X select-line; run -d0.3; send -X copy-pipe-and-cancel }".as_ptr(),
        c"bind -Tcopy-mode NPage { send -X page-down }".as_ptr(),
        c"bind -Tcopy-mode PPage { send -X page-up }".as_ptr(),
        c"bind -Tcopy-mode Up { send -X cursor-up }".as_ptr(),
        c"bind -Tcopy-mode Down { send -X cursor-down }".as_ptr(),
        c"bind -Tcopy-mode Left { send -X cursor-left }".as_ptr(),
        c"bind -Tcopy-mode Right { send -X cursor-right }".as_ptr(),
        c"bind -Tcopy-mode M-1 { command-prompt -Np'(repeat)' -I1 { send -N '%%' } }".as_ptr(),
        c"bind -Tcopy-mode M-2 { command-prompt -Np'(repeat)' -I2 { send -N '%%' } }".as_ptr(),
        c"bind -Tcopy-mode M-3 { command-prompt -Np'(repeat)' -I3 { send -N '%%' } }".as_ptr(),
        c"bind -Tcopy-mode M-4 { command-prompt -Np'(repeat)' -I4 { send -N '%%' } }".as_ptr(),
        c"bind -Tcopy-mode M-5 { command-prompt -Np'(repeat)' -I5 { send -N '%%' } }".as_ptr(),
        c"bind -Tcopy-mode M-6 { command-prompt -Np'(repeat)' -I6 { send -N '%%' } }".as_ptr(),
        c"bind -Tcopy-mode M-7 { command-prompt -Np'(repeat)' -I7 { send -N '%%' } }".as_ptr(),
        c"bind -Tcopy-mode M-8 { command-prompt -Np'(repeat)' -I8 { send -N '%%' } }".as_ptr(),
        c"bind -Tcopy-mode M-9 { command-prompt -Np'(repeat)' -I9 { send -N '%%' } }".as_ptr(),
        c"bind -Tcopy-mode M-< { send -X history-top }".as_ptr(),
        c"bind -Tcopy-mode M-> { send -X history-bottom }".as_ptr(),
        c"bind -Tcopy-mode M-R { send -X top-line }".as_ptr(),
        c"bind -Tcopy-mode M-b { send -X previous-word }".as_ptr(),
        c"bind -Tcopy-mode C-M-b { send -X previous-matching-bracket }".as_ptr(),
        c"bind -Tcopy-mode M-f { send -X next-word-end }".as_ptr(),
        c"bind -Tcopy-mode C-M-f { send -X next-matching-bracket }".as_ptr(),
        c"bind -Tcopy-mode M-m { send -X back-to-indentation }".as_ptr(),
        c"bind -Tcopy-mode M-r { send -X middle-line }".as_ptr(),
        c"bind -Tcopy-mode M-v { send -X page-up }".as_ptr(),
        c"bind -Tcopy-mode M-w { send -X copy-pipe-and-cancel }".as_ptr(),
        c"bind -Tcopy-mode M-x { send -X jump-to-mark }".as_ptr(),
        c"bind -Tcopy-mode 'M-{' { send -X previous-paragraph }".as_ptr(),
        c"bind -Tcopy-mode 'M-}' { send -X next-paragraph }".as_ptr(),
        c"bind -Tcopy-mode M-Up { send -X halfpage-up }".as_ptr(),
        c"bind -Tcopy-mode M-Down { send -X halfpage-down }".as_ptr(),
        c"bind -Tcopy-mode C-Up { send -X scroll-up }".as_ptr(),
        c"bind -Tcopy-mode C-Down { send -X scroll-down }".as_ptr(),
        c"bind -Tcopy-mode-vi '#' { send -FX search-backward -- '#{copy_cursor_word}' }".as_ptr(),
        c"bind -Tcopy-mode-vi * { send -FX search-forward -- '#{copy_cursor_word}' }".as_ptr(),
        c"bind -Tcopy-mode-vi C-c { send -X cancel }".as_ptr(),
        c"bind -Tcopy-mode-vi C-d { send -X halfpage-down }".as_ptr(),
        c"bind -Tcopy-mode-vi C-e { send -X scroll-down }".as_ptr(),
        c"bind -Tcopy-mode-vi C-b { send -X page-up }".as_ptr(),
        c"bind -Tcopy-mode-vi C-f { send -X page-down }".as_ptr(),
        c"bind -Tcopy-mode-vi C-h { send -X cursor-left }".as_ptr(),
        c"bind -Tcopy-mode-vi C-j { send -X copy-pipe-and-cancel }".as_ptr(),
        c"bind -Tcopy-mode-vi Enter { send -X copy-pipe-and-cancel }".as_ptr(),
        c"bind -Tcopy-mode-vi C-u { send -X halfpage-up }".as_ptr(),
        c"bind -Tcopy-mode-vi C-v { send -X rectangle-toggle }".as_ptr(),
        c"bind -Tcopy-mode-vi C-y { send -X scroll-up }".as_ptr(),
        c"bind -Tcopy-mode-vi Escape { send -X clear-selection }".as_ptr(),
        c"bind -Tcopy-mode-vi C-[ { send -X clear-selection }".as_ptr(),
        c"bind -Tcopy-mode-vi Space { send -X begin-selection }".as_ptr(),
        c"bind -Tcopy-mode-vi '$' { send -X end-of-line }".as_ptr(),
        c"bind -Tcopy-mode-vi , { send -X jump-reverse }".as_ptr(),
        c"bind -Tcopy-mode-vi / { command-prompt -T search -p'(search down)' { send -X search-forward -- '%%' } }".as_ptr(),
        c"bind -Tcopy-mode-vi 0 { send -X start-of-line }".as_ptr(),
        c"bind -Tcopy-mode-vi 1 { command-prompt -Np'(repeat)' -I1 { send -N '%%' } }".as_ptr(),
        c"bind -Tcopy-mode-vi 2 { command-prompt -Np'(repeat)' -I2 { send -N '%%' } }".as_ptr(),
        c"bind -Tcopy-mode-vi 3 { command-prompt -Np'(repeat)' -I3 { send -N '%%' } }".as_ptr(),
        c"bind -Tcopy-mode-vi 4 { command-prompt -Np'(repeat)' -I4 { send -N '%%' } }".as_ptr(),
        c"bind -Tcopy-mode-vi 5 { command-prompt -Np'(repeat)' -I5 { send -N '%%' } }".as_ptr(),
        c"bind -Tcopy-mode-vi 6 { command-prompt -Np'(repeat)' -I6 { send -N '%%' } }".as_ptr(),
        c"bind -Tcopy-mode-vi 7 { command-prompt -Np'(repeat)' -I7 { send -N '%%' } }".as_ptr(),
        c"bind -Tcopy-mode-vi 8 { command-prompt -Np'(repeat)' -I8 { send -N '%%' } }".as_ptr(),
        c"bind -Tcopy-mode-vi 9 { command-prompt -Np'(repeat)' -I9 { send -N '%%' } }".as_ptr(),
        c"bind -Tcopy-mode-vi : { command-prompt -p'(goto line)' { send -X goto-line -- '%%' } }".as_ptr(),
        c"bind -Tcopy-mode-vi \\; { send -X jump-again }".as_ptr(),
        c"bind -Tcopy-mode-vi ? { command-prompt -T search -p'(search up)' { send -X search-backward -- '%%' } }".as_ptr(),
        c"bind -Tcopy-mode-vi A { send -X append-selection-and-cancel }".as_ptr(),
        c"bind -Tcopy-mode-vi B { send -X previous-space }".as_ptr(),
        c"bind -Tcopy-mode-vi D { send -X copy-pipe-end-of-line-and-cancel }".as_ptr(),
        c"bind -Tcopy-mode-vi E { send -X next-space-end }".as_ptr(),
        c"bind -Tcopy-mode-vi F { command-prompt -1p'(jump backward)' { send -X jump-backward -- '%%' } }".as_ptr(),
        c"bind -Tcopy-mode-vi G { send -X history-bottom }".as_ptr(),
        c"bind -Tcopy-mode-vi H { send -X top-line }".as_ptr(),
        c"bind -Tcopy-mode-vi J { send -X scroll-down }".as_ptr(),
        c"bind -Tcopy-mode-vi K { send -X scroll-up }".as_ptr(),
        c"bind -Tcopy-mode-vi L { send -X bottom-line }".as_ptr(),
        c"bind -Tcopy-mode-vi M { send -X middle-line }".as_ptr(),
        c"bind -Tcopy-mode-vi N { send -X search-reverse }".as_ptr(),
        c"bind -Tcopy-mode-vi P { send -X toggle-position }".as_ptr(),
        c"bind -Tcopy-mode-vi T { command-prompt -1p'(jump to backward)' { send -X jump-to-backward -- '%%' } }".as_ptr(),
        c"bind -Tcopy-mode-vi V { send -X select-line }".as_ptr(),
        c"bind -Tcopy-mode-vi W { send -X next-space }".as_ptr(),
        c"bind -Tcopy-mode-vi X { send -X set-mark }".as_ptr(),
        c"bind -Tcopy-mode-vi ^ { send -X back-to-indentation }".as_ptr(),
        c"bind -Tcopy-mode-vi b { send -X previous-word }".as_ptr(),
        c"bind -Tcopy-mode-vi e { send -X next-word-end }".as_ptr(),
        c"bind -Tcopy-mode-vi f { command-prompt -1p'(jump forward)' { send -X jump-forward -- '%%' } }".as_ptr(),
        c"bind -Tcopy-mode-vi g { send -X history-top }".as_ptr(),
        c"bind -Tcopy-mode-vi h { send -X cursor-left }".as_ptr(),
        c"bind -Tcopy-mode-vi j { send -X cursor-down }".as_ptr(),
        c"bind -Tcopy-mode-vi k { send -X cursor-up }".as_ptr(),
        c"bind -Tcopy-mode-vi z { send -X scroll-middle }".as_ptr(),
        c"bind -Tcopy-mode-vi l { send -X cursor-right }".as_ptr(),
        c"bind -Tcopy-mode-vi n { send -X search-again }".as_ptr(),
        c"bind -Tcopy-mode-vi o { send -X other-end }".as_ptr(),
        c"bind -Tcopy-mode-vi q { send -X cancel }".as_ptr(),
        c"bind -Tcopy-mode-vi r { send -X refresh-from-pane }".as_ptr(),
        c"bind -Tcopy-mode-vi t { command-prompt -1p'(jump to forward)' { send -X jump-to-forward -- '%%' } }".as_ptr(),
        c"bind -Tcopy-mode-vi v { send -X rectangle-toggle }".as_ptr(),
        c"bind -Tcopy-mode-vi w { send -X next-word }".as_ptr(),
        c"bind -Tcopy-mode-vi '{' { send -X previous-paragraph }".as_ptr(),
        c"bind -Tcopy-mode-vi '}' { send -X next-paragraph }".as_ptr(),
        c"bind -Tcopy-mode-vi % { send -X next-matching-bracket }".as_ptr(),
        c"bind -Tcopy-mode-vi Home { send -X start-of-line }".as_ptr(),
        c"bind -Tcopy-mode-vi End { send -X end-of-line }".as_ptr(),
        c"bind -Tcopy-mode-vi MouseDown1Pane { select-pane }".as_ptr(),
        c"bind -Tcopy-mode-vi MouseDrag1Pane { select-pane; send -X begin-selection }".as_ptr(),
        c"bind -Tcopy-mode-vi MouseDragEnd1Pane { send -X copy-pipe-and-cancel }".as_ptr(),
        c"bind -Tcopy-mode-vi WheelUpPane { select-pane; send -N5 -X scroll-up }".as_ptr(),
        c"bind -Tcopy-mode-vi WheelDownPane { select-pane; send -N5 -X scroll-down }".as_ptr(),
        c"bind -Tcopy-mode-vi DoubleClick1Pane { select-pane; send -X select-word; run -d0.3; send -X copy-pipe-and-cancel }".as_ptr(),
        c"bind -Tcopy-mode-vi TripleClick1Pane { select-pane; send -X select-line; run -d0.3; send -X copy-pipe-and-cancel }".as_ptr(),
        c"bind -Tcopy-mode-vi BSpace { send -X cursor-left }".as_ptr(),
        c"bind -Tcopy-mode-vi NPage { send -X page-down }".as_ptr(),
        c"bind -Tcopy-mode-vi PPage { send -X page-up }".as_ptr(),
        c"bind -Tcopy-mode-vi Up { send -X cursor-up }".as_ptr(),
        c"bind -Tcopy-mode-vi Down { send -X cursor-down }".as_ptr(),
        c"bind -Tcopy-mode-vi Left { send -X cursor-left }".as_ptr(),
        c"bind -Tcopy-mode-vi Right { send -X cursor-right }".as_ptr(),
        c"bind -Tcopy-mode-vi M-x { send -X jump-to-mark }".as_ptr(),
        c"bind -Tcopy-mode-vi C-Up { send -X scroll-up }".as_ptr(),
        c"bind -Tcopy-mode-vi C-Down { send -X scroll-down }".as_ptr(),
    ]);
        let mut i: u_int = 0;
        i = 0 as u_int;
        while (i as usize)
            < (::core::mem::size_of::<[*const ::core::ffi::c_char; 275]>() as usize)
                .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>() as usize)
        {
            let mut pr = cmd_parse_from_string(
                defaults[i as usize],
                ::core::ptr::null_mut::<cmd_parse_input>(),
            );
            if pr.status as ::core::ffi::c_uint
                != CMD_PARSE_SUCCESS as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let error = pr.error.take().unwrap();
                log_debug(c"%s".as_ptr(), fmt_args![error.as_ptr()]);
                fatalx(
                    c"bad default key: %s".as_ptr(),
                    fmt_args![defaults[i as usize]],
                );
            }
            let cmdlist = pr.cmdlist.take().unwrap();
            cmdq_append(
                ::core::ptr::null_mut::<client>(),
                cmdq_get_command(&cmdlist, None),
            );
            i = i.wrapping_add(1);
        }
        cmdq_append(
            ::core::ptr::null_mut::<client>(),
            cmdq_get_callback1(
                c"key_bindings_init_done".as_ptr(),
                Some(key_bindings_init_done),
                CmdqCallbackData::None,
            ),
        );
    }
}
unsafe fn key_bindings_read_only(mut item: *mut cmdq_item, _data: CmdqCallbackData) -> cmd_retval {
    unsafe {
        cmdq_error(item, c"client is read-only".as_ptr(), fmt_args![]);
        CMD_RETURN_ERROR
    }
}
pub unsafe fn key_bindings_dispatch(
    mut bd: *mut key_binding,
    mut item: *mut cmdq_item,
    mut c: *mut client,
    mut event: *mut key_event,
    mut fs: *mut cmd_find_state,
) -> *mut cmdq_item {
    unsafe {
        let mut readonly: ::core::ffi::c_int = 0;
        let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if c.is_null() || !(*c).flags & CLIENT_READONLY as uint64_t != 0 {
            readonly = 1 as ::core::ffi::c_int;
        } else {
            readonly = cmd_list_all_have((*bd).cmdlist.as_ref().unwrap().as_ptr(), CMD_READONLY);
        }
        let queued = if readonly == 0 {
            cmdq_get_callback1(
                c"key_bindings_read_only".as_ptr(),
                Some(key_bindings_read_only),
                CmdqCallbackData::None,
            )
        } else {
            if (*bd).flags & KEY_BINDING_REPEAT != 0 {
                flags |= CMDQ_STATE_REPEAT;
            }
            let new_state = cmdq_new_state(fs, event, flags);
            cmdq_get_command((*bd).cmdlist.as_ref().unwrap(), Some(&new_state))
        };
        if !item.is_null() {
            cmdq_insert_after(item, queued)
        } else {
            cmdq_append(c, queued)
        }
    }
}
pub unsafe fn key_bindings_has_repeat(
    mut l: *mut *mut key_binding,
    mut n: u_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut i: u_int = 0;
        i = 0 as u_int;
        while i < n {
            if (**l.offset(i as isize)).flags & KEY_BINDING_REPEAT != 0 {
                return 1 as ::core::ffi::c_int;
            }
            i = i.wrapping_add(1);
        }
        0 as ::core::ffi::c_int
    }
}
