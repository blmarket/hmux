use crate::src::cmd::parse::cmd_parse_from_string;
use crate::src::cmd::queue::{
    cmdq_append, cmdq_error, cmdq_free_state, cmdq_get_callback1, cmdq_get_command,
    cmdq_insert_after, cmdq_new_state,
};
use crate::src::cmd::{cmd_list_all_have, cmd_list_free, cmd_list_print_cstring};
use crate::src::ffi::libc::strcmp;
use crate::src::key_string::key_string_format;
use crate::src::log::{fatalx, log_debug};
use crate::src::server::clients;
use crate::src::server_client::server_client_set_key_table;
use crate::src::shared::abi::*;
use crate::src::shared::client::client;
use crate::src::shared::client::CLIENT_READONLY;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_state};
use crate::src::shared::command::{cmd_parse_input, cmd_parse_result};
use crate::src::shared::command::{CMDQ_STATE_REPEAT, CMD_READONLY};
use crate::src::shared::key::KEY_BINDING_REPEAT;
use crate::src::shared::key::*;
use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
use crate::src::shared::tree::RB_NEGINF;
use std::ffi::CStr;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct key_tables {
    pub storage: *mut std::collections::BTreeMap<Vec<u8>, *mut key_table>,
}

static mut key_tables: key_tables = key_tables {
    storage: std::ptr::null_mut(),
};

unsafe fn key_bindings_new() -> *mut key_binding {
    Box::into_raw(Box::new(key_binding {
        note: None,
        ..key_binding::empty()
    }))
    .cast()
}

pub(crate) unsafe fn key_bindings_set_note(bd: *mut key_binding, note: Option<&CStr>) {
    // Copy before replacing: a caller may pass the binding's current note.
    let next = note.map(CStr::to_owned);
    let owner = &mut *bd;
    owner.note = Default::default();
    owner.note = next;
}
unsafe extern "C" fn key_table_cmp(
    mut table1: *mut key_table,
    mut table2: *mut key_table,
) -> ::core::ffi::c_int {
    return strcmp(
        ((*table1).name).as_ptr().cast_mut(),
        ((*table2).name).as_ptr().cast_mut(),
    );
}
unsafe extern "C" fn key_bindings_cmp(
    mut bd1: *mut key_binding,
    mut bd2: *mut key_binding,
) -> ::core::ffi::c_int {
    if (*bd1).key < (*bd2).key {
        return -(1 as ::core::ffi::c_int);
    }
    if (*bd1).key > (*bd2).key {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn key_bindings_free(mut bd: *mut key_binding) {
    cmd_list_free((*bd).cmdlist);
    drop(Box::from_raw(bd));
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_get_table(
    mut name: *const ::core::ffi::c_char,
    mut create: ::core::ffi::c_int,
) -> *mut key_table {
    let mut table_find: key_table = key_table {
        name: Default::default(),
        activity_time: timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        key_bindings: key_bindings {
            storage: std::ptr::null_mut(),
        },
        default_key_bindings: key_bindings {
            storage: std::ptr::null_mut(),
        },
        references: 0,
        entry: key_table_entry {
            owner: std::ptr::null_mut(),
        },
    };
    let mut table: *mut key_table = ::core::ptr::null_mut::<key_table>();
    table_find.name = ::std::ffi::CStr::from_ptr(name).to_owned();
    table = key_tables_find(&*std::ptr::addr_of!(key_tables), &table_find);
    if !table.is_null() || create == 0 {
        return table;
    }
    let mut owner = Box::new(key_table {
        name: CStr::from_ptr(name).to_owned(),
        ..key_table::empty()
    });

    table = Box::into_raw(owner).cast();
    (*table).key_bindings.storage = std::ptr::null_mut();
    (*table).default_key_bindings.storage = std::ptr::null_mut();
    (*table).references = 1 as u_int;
    key_tables_insert(&raw mut key_tables, table);
    return table;
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_first_table() -> *mut key_table {
    return key_tables_minmax(&*std::ptr::addr_of!(key_tables), RB_NEGINF);
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_next_table(mut table: *mut key_table) -> *mut key_table {
    return key_tables_next(&*table);
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_unref_table(mut table: *mut key_table) {
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut bd1: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    (*table).references = (*table).references.wrapping_sub(1);
    if (*table).references != 0 as u_int {
        return;
    }
    bd = key_bindings_index_minmax(&(*table).key_bindings, RB_NEGINF);
    while !bd.is_null() && {
        bd1 = key_bindings_index_next(&*bd);
        1 as ::core::ffi::c_int != 0
    } {
        key_bindings_index_remove(&raw mut (*table).key_bindings, bd);
        key_bindings_free(bd);
        bd = bd1;
    }
    bd = key_bindings_index_minmax(&(*table).default_key_bindings, RB_NEGINF);
    while !bd.is_null() && {
        bd1 = key_bindings_index_next(&*bd);
        1 as ::core::ffi::c_int != 0
    } {
        key_bindings_index_remove(&raw mut (*table).default_key_bindings, bd);
        key_bindings_free(bd);
        bd = bd1;
    }
    drop(Box::from_raw(table));
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_get(
    mut table: *mut key_table,
    mut key: key_code,
) -> *mut key_binding {
    let mut bd: key_binding = key_binding {
        key: 0,
        cmdlist: ::core::ptr::null_mut::<cmd_list>(),
        note: Default::default(),
        tablename: None,
        flags: 0,
        entry: key_binding_entry {
            owner: std::ptr::null_mut(),
        },
    };
    bd.key = key;
    return key_bindings_index_find(&(*table).key_bindings, &bd);
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_get_default(
    mut table: *mut key_table,
    mut key: key_code,
) -> *mut key_binding {
    let mut bd: key_binding = key_binding {
        key: 0,
        cmdlist: ::core::ptr::null_mut::<cmd_list>(),
        note: Default::default(),
        tablename: None,
        flags: 0,
        entry: key_binding_entry {
            owner: std::ptr::null_mut(),
        },
    };
    bd.key = key;
    return key_bindings_index_find(&(*table).default_key_bindings, &bd);
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_first(mut table: *mut key_table) -> *mut key_binding {
    return key_bindings_index_minmax(&(*table).key_bindings, RB_NEGINF);
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_next(
    mut table: *mut key_table,
    mut bd: *mut key_binding,
) -> *mut key_binding {
    return key_bindings_index_next(&*bd);
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_add(
    mut name: *const ::core::ffi::c_char,
    mut key: key_code,
    mut note: *const ::core::ffi::c_char,
    mut repeat: ::core::ffi::c_int,
    mut cmdlist: *mut cmd_list,
) {
    let mut table: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    table = key_bindings_get_table(name, 1 as ::core::ffi::c_int);
    bd = key_bindings_get(table, key & !KEYC_MASK_FLAGS);
    if cmdlist.is_null() {
        if !bd.is_null() {
            if !note.is_null() {
                key_bindings_set_note(bd, Some(CStr::from_ptr(note)));
            }
            if repeat != 0 {
                (*bd).flags |= KEY_BINDING_REPEAT;
            }
        }
        return;
    }
    if !bd.is_null() {
        key_bindings_index_remove(&raw mut (*table).key_bindings, bd);
        key_bindings_free(bd);
    }
    bd = key_bindings_new();
    (*bd).key = (key as ::core::ffi::c_ulonglong & !KEYC_MASK_FLAGS) as key_code;
    (*bd).tablename = Some((*table).name.clone());
    if !note.is_null() {
        key_bindings_set_note(bd, Some(CStr::from_ptr(note)));
    }
    key_bindings_index_insert(&raw mut (*table).key_bindings, bd);
    if repeat != 0 {
        (*bd).flags |= KEY_BINDING_REPEAT;
    }
    (*bd).cmdlist = cmdlist;
    let s = cmd_list_print_cstring(&*(*bd).cmdlist, 0);
    let key_string = key_string_format((*bd).key, true);
    log_debug(
        b"%s: %#llx %s = %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"key_bindings_add\0" as *const u8 as *const ::core::ffi::c_char,
        (*bd).key,
        key_string.as_ptr(),
        s.as_ptr(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_remove(
    mut name: *const ::core::ffi::c_char,
    mut key: key_code,
) {
    let mut table: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    table = key_bindings_get_table(name, 0 as ::core::ffi::c_int);
    if table.is_null() {
        return;
    }
    bd = key_bindings_get(table, key & !KEYC_MASK_FLAGS);
    if bd.is_null() {
        return;
    }
    let key_string = key_string_format((*bd).key, true);
    log_debug(
        b"%s: %#llx %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"key_bindings_remove\0" as *const u8 as *const ::core::ffi::c_char,
        (*bd).key,
        key_string.as_ptr(),
    );
    key_bindings_index_remove(&raw mut (*table).key_bindings, bd);
    key_bindings_free(bd);
    if (*table).key_bindings.storage.is_null() && (*table).default_key_bindings.storage.is_null() {
        key_tables_remove(&raw mut key_tables, table);
        key_bindings_unref_table(table);
    }
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_reset(
    mut name: *const ::core::ffi::c_char,
    mut key: key_code,
) {
    let mut table: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut dd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    table = key_bindings_get_table(name, 0 as ::core::ffi::c_int);
    if table.is_null() {
        return;
    }
    bd = key_bindings_get(table, key & !KEYC_MASK_FLAGS);
    if bd.is_null() {
        return;
    }
    dd = key_bindings_get_default(table, (*bd).key);
    if dd.is_null() {
        key_bindings_remove(name, (*bd).key);
        return;
    }
    cmd_list_free((*bd).cmdlist);
    (*bd).cmdlist = (*dd).cmdlist;
    (*(*bd).cmdlist).references += 1;
    key_bindings_set_note(bd, (*dd).note.as_deref());
    (*bd).flags = (*dd).flags;
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_remove_table(mut name: *const ::core::ffi::c_char) {
    let mut table: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    table = key_bindings_get_table(name, 0 as ::core::ffi::c_int);
    if !table.is_null() {
        key_tables_remove(&raw mut key_tables, table);
        c = clients.first();
        while !c.is_null() {
            if (*c).keytable == table {
                server_client_set_key_table(c, ::core::ptr::null::<::core::ffi::c_char>());
            }
            c = clients.next(c);
        }
        key_bindings_unref_table(table);
    }
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_reset_table(mut name: *const ::core::ffi::c_char) {
    let mut table: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut bd1: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    table = key_bindings_get_table(name, 0 as ::core::ffi::c_int);
    if table.is_null() {
        return;
    }
    if (*table).default_key_bindings.storage.is_null() {
        key_bindings_remove_table(name);
        return;
    }
    bd = key_bindings_index_minmax(&(*table).key_bindings, RB_NEGINF);
    while !bd.is_null() && {
        bd1 = key_bindings_index_next(&*bd);
        1 as ::core::ffi::c_int != 0
    } {
        key_bindings_reset(name, (*bd).key);
        bd = bd1;
    }
}

/// Insert a default snapshot, consuming one existing command-list reference.
/// Its note is copied; its table name remains absent as in the original
/// startup snapshot.
pub unsafe fn key_bindings_add_default(
    table: *mut key_table,
    key: key_code,
    cmdlist: *mut cmd_list,
    note: Option<&CStr>,
    flags: ::core::ffi::c_int,
) -> *mut key_binding {
    let bd = key_bindings_new();
    (*bd).key = key;
    (*bd).cmdlist = cmdlist;
    (*bd).flags = flags;
    key_bindings_set_note(bd, note);
    key_bindings_index_insert(&raw mut (*table).default_key_bindings, bd);
    bd
}
unsafe extern "C" fn key_bindings_init_done(
    mut item: *mut cmdq_item,
    mut data: *mut ::core::ffi::c_void,
) -> cmd_retval {
    let mut table: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut new_bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    table = key_tables_minmax(&*std::ptr::addr_of!(key_tables), RB_NEGINF);
    while !table.is_null() {
        bd = key_bindings_index_minmax(&(*table).key_bindings, RB_NEGINF);
        while !bd.is_null() {
            (*(*bd).cmdlist).references += 1;
            new_bd = key_bindings_add_default(
                table,
                (*bd).key,
                (*bd).cmdlist,
                (*bd).note.as_deref(),
                (*bd).flags,
            );
            bd = key_bindings_index_next(&*bd);
        }
        table = key_tables_next(&*table);
    }
    return CMD_RETURN_NORMAL;
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_init() {
    static mut defaults: [*const ::core::ffi::c_char; 308] = [
        b"bind -N 'Send the prefix key' C-b { send-prefix }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Rotate through the panes' C-o { rotate-window }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Suspend the current client' C-z { suspend-client }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Select next layout' Space { next-layout }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Break pane to a new window' ! { break-pane }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Split window vertically' '\"' { split-window }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'List all paste buffers' '#' { list-buffers }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Rename current session' '$' { command-prompt -I'#S' { rename-session -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Split window horizontally' % { split-window -h }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Kill current window' & { confirm-before -p\"kill-window #W? (y/n)\" kill-window }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Prompt for window index to select' \"'\" { command-prompt -pindex { select-window -t ':%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'New floating pane' * { new-pane }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Toggle pane between floating and tiled' @ { if -F '#{pane_floating_flag}' { join-pane } { break-pane -W } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Switch to previous client' ( { switch-client -p }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Switch to next client' ) { switch-client -n }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Rename current window' , { command-prompt -I'#W' { rename-window -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Delete the most recent paste buffer' - { delete-buffer }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Move the current window' . { command-prompt { move-window -t '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Describe key binding' '/' { command-prompt -kpkey  { list-keys -1N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Select window 0' 0 { select-window -t:=0 }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Select window 1' 1 { select-window -t:=1 }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Select window 2' 2 { select-window -t:=2 }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Select window 3' 3 { select-window -t:=3 }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Select window 4' 4 { select-window -t:=4 }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Select window 5' 5 { select-window -t:=5 }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Select window 6' 6 { select-window -t:=6 }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Select window 7' 7 { select-window -t:=7 }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Select window 8' 8 { select-window -t:=8 }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Select window 9' 9 { select-window -t:=9 }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Prompt for a command' : { command-prompt }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Move to the previously active pane' \\; { last-pane }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Choose a paste buffer from a list' = { choose-buffer -Z }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'List key bindings' ? { list-keys -N }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Choose and detach a client from a list' D { choose-client -Z }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Spread panes out evenly' E { select-layout -E }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Switch to the last client' L { switch-client -l }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Clear the marked pane' M { select-pane -M }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Change the pane title' T { command-prompt -I'#T' { select-pane -T '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Enter copy mode' [ { copy-mode }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Paste the most recent paste buffer' ] { paste-buffer -p }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Create a new window' c { new-window }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Detach the current client' d { detach-client }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Search for a pane' f { command-prompt { find-window -Z -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Display window information' i { display-message }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Select the previously current window' l { last-window }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Toggle the marked pane' m { select-pane -m }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Select the next window' n { next-window }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Select the next pane' o { select-pane -t:.+ }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Customize options' C { customize-mode -Z }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Select the previous window' p { previous-window }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Display pane numbers' q { display-panes }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Redraw the current client' r { refresh-client }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Choose a session from a list' s { choose-tree -Zs }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Show a clock' t { clock-mode }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Switch to a window' Tab { new-pane -E -x75% -y30% -X0 -Y0; move-pane -P bottom-centre; switch-mode -wk }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Switch to a session' BTab { new-pane -E -x75% -y30% -X0 -Y0; move-pane -P bottom-centre; switch-mode -sk }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Choose a window from a list' w { choose-tree -Zw }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Kill the active pane' x { confirm-before -p\"kill-pane #P? (y/n)\" kill-pane }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Zoom the active pane' z { resize-pane -Z }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Swap the active pane with the pane above' '{' { swap-pane -U }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Swap the active pane with the pane below' '}' { swap-pane -D }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Show messages' '~' { show-messages }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Enter copy mode and scroll up' PPage { copy-mode -u }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Select the pane above the active pane' -r Up { select-pane -U }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Select the pane below the active pane' -r Down { select-pane -D }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Select the pane to the left of the active pane' -r Left { select-pane -L }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Select the pane to the right of the active pane' -r Right { select-pane -R }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Set the even-horizontal layout' M-1 { select-layout even-horizontal }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Set the even-vertical layout' M-2 { select-layout even-vertical }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Set the main-horizontal layout' M-3 { select-layout main-horizontal }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Set the main-vertical layout' M-4 { select-layout main-vertical }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Select the tiled layout' M-5 { select-layout tiled }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Set the main-horizontal-mirrored layout' M-6 { select-layout main-horizontal-mirrored }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Set the main-vertical-mirrored layout' M-7 { select-layout main-vertical-mirrored }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Select the next window with an alert' M-n { next-window -a }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Rotate through the panes in reverse' M-o { rotate-window -D }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Select the previous window with an alert' M-p { previous-window -a }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Move the visible part of the window up' -r S-Up { refresh-client -U 10 }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Move the visible part of the window down' -r S-Down { refresh-client -D 10 }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Move the visible part of the window left' -r S-Left { refresh-client -L 10 }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Move the visible part of the window right' -r S-Right { refresh-client -R 10 }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Reset so the visible part of the window follows the cursor' -r DC { refresh-client -c }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Resize the pane up by 5' -r M-Up if -F '#{?floating_pane_flag}' { resizep -D-5 } { resize-pane -U 5 }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Resize the pane down by 5' -r M-Down { resize-pane -D 5 }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Resize the pane left by 5' -r M-Left if -F '#{?floating_pane_flag}' { resizep -R-5 } { resize-pane -L 5 }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Resize the pane right by 5' -r M-Right resize-pane -R 5\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Resize the pane up' -r C-Up if -F '#{?floating_pane_flag}' { resizep -D-1 } { resize-pane -U }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Resize the pane down' -r C-Down { resize-pane -D }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Resize the pane left' -r C-Left if -F '#{?floating_pane_flag}' { resizep -R-1 } { resize-pane -L }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Resize the pane right' -r C-Right { resize-pane -R }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Move a floating pane' g { switch-client -Tmove }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to top-left corner' 1 { move-pane -P top-left }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to top-right corner' 2 { move-pane -P top-right }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to bottom-left corner' 3 { move-pane -P bottom-left }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to bottom-right corner' 4 { move-pane -P bottom-right }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to top-left corner and resize' M-1 { resize-pane -x50% -y50%; move-pane -P top-left }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to top-right corner and resize' M-2 { resize-pane -x50% -y50%; move-pane -P top-right }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to bottom-left corner and resize' M-3 { resize-pane -x50% -y50%; move-pane -P bottom-left }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to bottom-right corner and resize' M-4 { resize-pane -x50% -y50%; move-pane -P bottom-right }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to top' 'Up' { move-pane -P top-centre }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to bottom' 'Down' { move-pane -P bottom-centre  }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to left' 'Left' { move-pane -P centre-left }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to right' 'Right' { move-pane -P centre-right }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to top and resize' 'M-Up' { resizep -x100% -y50%; move-pane -P top-centre }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to bottom and resize' 'M-Down' { resizep -x100% -y50%; move-pane -P bottom-centre  }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to left and resize' 'M-Left' { resizep -x50% -y100%; move-pane -P centre-left }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to right and resize' 'M-Right' { resizep -x50% -y100%; move-pane -P centre-right }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to fill the window' 0 { resize-pane -x100% -y100%; move-pane -P top-left }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Display move menu' , { if -F '#{pane_floating_flag}' { display-menu -xP -yP -T '#[align=centre]Move'  'Centre' 'c' {move-pane -P centre} '' 'Top Left' '1' {move-pane -P top-left} 'Top Right' '2' {move-pane -P top-right} 'Bottom Left' '3' {move-pane -P bottom-left} 'Bottom Right' '4' {move-pane -P bottom-right} '' 'Top' 't' {move-pane -P top-centre} 'Bottom' 'b' {move-pane -P bottom-centre} 'Left' 'l' {move-pane -P centre-left} 'Right' 'r' {move-pane -P centre-right} } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Display move and resize menu' . { if -F '#{pane_floating_flag}' { display-menu -xP -yP -T '#[align=centre]Move & Resize'  'Fill' '0' {resize-pane -x100% -y100%; move-pane -P top-left} '' 'Top Left' '1' {resize-pane -x50% -y50%; move-pane -P top-left} 'Top Right' '2' {resize-pane -x50% -y50%; move-pane -P top-right} 'Bottom Left' '3' {resize-pane -x50% -y50%; move-pane -P bottom-left} 'Bottom Right' '4' {resize-pane -x50% -y50%; move-pane -P bottom-right} '' 'Top' 't' {resize-pane -x100% -y50%; move-pane -P top-centre} 'Bottom' 'b' {resize-pane -x100% -y50%; move-pane -P bottom-centre} 'Left' 'l' {resize-pane -x50% -y100%; move-pane -P centre-left} 'Right' 'r' {resize-pane -x50% -y100%; move-pane -P centre-right} } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Display window menu' < { display-menu -xW -yW -T '#[align=centre]#{window_index}:#{window_name}'  '#{?#{>:#{session_windows},1},,-}Swap Left' 'l' {swap-window -t:-1} '#{?#{>:#{session_windows},1},,-}Swap Right' 'r' {swap-window -t:+1} '#{?pane_marked_set,,-}Swap Marked' 's' {swap-window} '' 'Kill' 'X' {kill-window} 'Respawn' 'R' {respawn-window -k} '#{?pane_marked,Unmark,Mark}' 'm' {select-pane -m} 'Rename' 'n' {command-prompt -FI \"#W\" {rename-window -t '#{window_id}' -- '%%'}} '' 'New After' 'w' {new-window -a} 'New At End' 'W' {new-window} }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Display pane menu' > { display-menu -xP -yP -T '#[align=centre]#{pane_index} (#{pane_id})'  '#{?#{m/r:(copy|view)-mode,#{pane_mode}},Go To Top,}' '<' {send -X history-top} '#{?#{m/r:(copy|view)-mode,#{pane_mode}},Go To Bottom,}' '>' {send -X history-bottom} '' '#{?#{==:#{pane_mode},copy-mode},#{?copy_line_numbers,Hide Line Numbers,Show Line Numbers},}' 'L' {send -X line-numbers-toggle} '#{?#{==:#{pane_mode},copy-mode},#{?refresh_active,Refresh Off,Refresh On},}' 'r' {send -X refresh-toggle} '' '#{?#{&&:#{buffer_size},#{!:#{pane_in_mode}}},Paste #[underscore]#{=/9/...:buffer_sample},}' 'p' {paste-buffer} '' '#{?mouse_word,Search For #[underscore]#{=/9/...:mouse_word},}' 'C-r' {if -F '#{?#{m/r:(copy|view)-mode,#{pane_mode}},0,1}' 'copy-mode -t='; send -Xt= search-backward -- \"#{q:mouse_word}\"} '#{?mouse_word,Type #[underscore]#{=/9/...:mouse_word},}' 'C-y' {copy-mode -q; send-keys -l -- \"#{q:mouse_word}\"} '#{?mouse_word,Copy #[underscore]#{=/9/...:mouse_word},}' 'c' {copy-mode -q; set-buffer -- \"#{q:mouse_word}\"} '#{?mouse_line,Copy Line,}' 'l' {copy-mode -q; set-buffer -- \"#{q:mouse_line}\"} '' '#{?mouse_hyperlink,Type #[underscore]#{=/9/...:mouse_hyperlink},}' 'C-h' {copy-mode -q; send-keys -l -- \"#{q:mouse_hyperlink}\"} '#{?mouse_hyperlink,Copy #[underscore]#{=/9/...:mouse_hyperlink},}' 'h' {copy-mode -q; set-buffer -- \"#{q:mouse_hyperlink}\"} '' '#{?#{#{pane_floating_flag}},Move,}' '' {display-menu -xL -yL -T '#[align=centre]Move'  'Centre' 'c' {move-pane -P centre} '' 'Top Left' '1' {move-pane -P top-left} 'Top Right' '2' {move-pane -P top-right} 'Bottom Left' '3' {move-pane -P bottom-left} 'Bottom Right' '4' {move-pane -P bottom-right} '' 'Top' 't' {move-pane -P top-centre} 'Bottom' 'b' {move-pane -P bottom-centre} 'Left' 'l' {move-pane -P centre-left} 'Right' 'r' {move-pane -P centre-right} } '#{?#{#{pane_floating_flag}},Move & Resize,}' '' {display-menu -xL -yL -T '#[align=centre]Move & Resize'  'Fill' '0' {resize-pane -x100% -y100%; move-pane -P top-left} '' 'Top Left' '1' {resize-pane -x50% -y50%; move-pane -P top-left} 'Top Right' '2' {resize-pane -x50% -y50%; move-pane -P top-right} 'Bottom Left' '3' {resize-pane -x50% -y50%; move-pane -P bottom-left} 'Bottom Right' '4' {resize-pane -x50% -y50%; move-pane -P bottom-right} '' 'Top' 't' {resize-pane -x100% -y50%; move-pane -P top-centre} 'Bottom' 'b' {resize-pane -x100% -y50%; move-pane -P bottom-centre} 'Left' 'l' {resize-pane -x50% -y100%; move-pane -P centre-left} 'Right' 'r' {resize-pane -x50% -y100%; move-pane -P centre-right} } '#{?#{#{pane_floating_flag}},Tile,}' 't' { join-pane } '#{?#{!:#{pane_floating_flag}},Float,}' 'f' { break-pane -W } '#{?#{!:#{pane_floating_flag}},Horizontal Split,}' 'h' {split-window -h} '#{?#{!:#{pane_floating_flag}},Vertical Split,}' 'v' {split-window -v} '' '#{?#{&&:#{!:#{pane_floating_flag}},#{>:#{window_panes},1}},Swap Up,}' 'u' {swap-pane -U} '#{?#{&&:#{!:#{pane_floating_flag}},#{>:#{window_panes},1}},Swap Down,}' 'd' {swap-pane -D} '#{?pane_marked_set,,-}Swap Marked' 's' {swap-pane} '' 'Kill' 'X' {kill-pane} 'Respawn' 'R' {respawn-pane -k} '#{?pane_marked,Unmark,Mark}' 'm' {select-pane -m} '#{?#{>:#{window_panes},1},,-}#{?window_zoomed_flag,Unzoom,Zoom}' 'z' {resize-pane -Z} }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n MouseDown1Pane { select-pane -t=; send -M }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -n MouseDrag1Pane { if -F '#{||:#{pane_in_mode},#{mouse_any_flag}}' { send -M } { copy-mode -M } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n C-MouseDrag1Pane { new-pane -M }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -n C-MouseDrag1Empty { new-pane -M }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -n M-MouseDrag1Pane { move-pane -M }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -n WheelUpPane { if -F '#{||:#{alternate_on},#{pane_in_mode},#{mouse_any_flag}}' { send -M } { copy-mode -e } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n MouseDown2Pane { select-pane -t=; if -F '#{||:#{pane_in_mode},#{mouse_any_flag}}' { send -M } { paste -p } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n DoubleClick1Pane { select-pane -t=; if -F '#{||:#{pane_in_mode},#{mouse_any_flag}}' { send -M } { copy-mode -H; send -X select-word; run -d0.3; send -X copy-pipe-and-cancel } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n TripleClick1Pane { select-pane -t=; if -F '#{||:#{pane_in_mode},#{mouse_any_flag}}' { send -M } { copy-mode -H; send -X select-line; run -d0.3; send -X copy-pipe-and-cancel } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n MouseDown1Border { select-pane -M }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -n MouseDrag1Border { resize-pane -M }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -n M-MouseDrag1Border { move-pane -M }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -n MouseDown1Status { switch-client -t= }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -n C-MouseDown1Status { swap-window -t@ }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -n MouseDown1Control9 { display-menu -t= -xM -yM -O -T 'Kill pane #{pane_index}?' 'Yes' 'y' { kill-pane -t= } 'No' 'n' {}}\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n MouseDown1Control8 { resize-pane -Z }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -n MouseDown1Control7 { if -Ft= '#{pane_floating_flag}' { join-pane } { break-pane -W } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n WheelDownStatus { next-window }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -n WheelUpStatus { previous-window }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -n MouseDown3StatusLeft { run -C \"display-menu -t= -xM -yW -T '#[align=centre]#{session_name}'  #{S/t:#{?#{&&:#{<:#{loop_index},6},#{!:#{session_active}}},'Switch To #[underscore]#{session_name}' '' {switch-client -t=#{session_id}#} ,}} '' 'Renumber' 'N' {move-window -r} 'Rename' 'r' {command-prompt -I '#S' {rename-session -- '%%'}} 'Detach' 'd' {detach-client} '' 'New Session' 's' {new-session} 'New Window' 'w' {new-window}\" }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n M-MouseDown3StatusLeft { run -C \"display-menu -t= -xM -yW -T '#[align=centre]#{session_name}'  #{S/t:#{?#{&&:#{<:#{loop_index},6},#{!:#{session_active}}},'Switch To #[underscore]#{session_name}' '' {switch-client -t=#{session_id}#} ,}} '' 'Renumber' 'N' {move-window -r} 'Rename' 'r' {command-prompt -I '#S' {rename-session -- '%%'}} 'Detach' 'd' {detach-client} '' 'New Session' 's' {new-session} 'New Window' 'w' {new-window}\" }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n MouseDown3Status { display-menu -t= -xW -yW -T '#[align=centre]#{window_index}:#{window_name}'  '#{?#{>:#{session_windows},1},,-}Swap Left' 'l' {swap-window -t:-1} '#{?#{>:#{session_windows},1},,-}Swap Right' 'r' {swap-window -t:+1} '#{?pane_marked_set,,-}Swap Marked' 's' {swap-window} '' 'Kill' 'X' {kill-window} 'Respawn' 'R' {respawn-window -k} '#{?pane_marked,Unmark,Mark}' 'm' {select-pane -m} 'Rename' 'n' {command-prompt -FI \"#W\" {rename-window -t '#{window_id}' -- '%%'}} '' 'New After' 'w' {new-window -a} 'New At End' 'W' {new-window}}\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n M-MouseDown3Status { display-menu -t= -xW -yW -T '#[align=centre]#{window_index}:#{window_name}'  '#{?#{>:#{session_windows},1},,-}Swap Left' 'l' {swap-window -t:-1} '#{?#{>:#{session_windows},1},,-}Swap Right' 'r' {swap-window -t:+1} '#{?pane_marked_set,,-}Swap Marked' 's' {swap-window} '' 'Kill' 'X' {kill-window} 'Respawn' 'R' {respawn-window -k} '#{?pane_marked,Unmark,Mark}' 'm' {select-pane -m} 'Rename' 'n' {command-prompt -FI \"#W\" {rename-window -t '#{window_id}' -- '%%'}} '' 'New After' 'w' {new-window -a} 'New At End' 'W' {new-window}}\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n MouseDown3Pane { if -Ft= '#{||:#{mouse_any_flag},#{&&:#{pane_in_mode},#{?#{m/r:(copy|view)-mode,#{pane_mode}},0,1}}}' { select-pane -t=; send -M } { display-menu -t= -xM -yM -T '#[align=centre]#{pane_index} (#{pane_id})'  '#{?#{m/r:(copy|view)-mode,#{pane_mode}},Go To Top,}' '<' {send -X history-top} '#{?#{m/r:(copy|view)-mode,#{pane_mode}},Go To Bottom,}' '>' {send -X history-bottom} '' '#{?#{==:#{pane_mode},copy-mode},#{?copy_line_numbers,Hide Line Numbers,Show Line Numbers},}' 'L' {send -X line-numbers-toggle} '#{?#{==:#{pane_mode},copy-mode},#{?refresh_active,Refresh Off,Refresh On},}' 'r' {send -X refresh-toggle} '' '#{?#{&&:#{buffer_size},#{!:#{pane_in_mode}}},Paste #[underscore]#{=/9/...:buffer_sample},}' 'p' {paste-buffer} '' '#{?mouse_word,Search For #[underscore]#{=/9/...:mouse_word},}' 'C-r' {if -F '#{?#{m/r:(copy|view)-mode,#{pane_mode}},0,1}' 'copy-mode -t='; send -Xt= search-backward -- \"#{q:mouse_word}\"} '#{?mouse_word,Type #[underscore]#{=/9/...:mouse_word},}' 'C-y' {copy-mode -q; send-keys -l -- \"#{q:mouse_word}\"} '#{?mouse_word,Copy #[underscore]#{=/9/...:mouse_word},}' 'c' {copy-mode -q; set-buffer -- \"#{q:mouse_word}\"} '#{?mouse_line,Copy Line,}' 'l' {copy-mode -q; set-buffer -- \"#{q:mouse_line}\"} '' '#{?mouse_hyperlink,Type #[underscore]#{=/9/...:mouse_hyperlink},}' 'C-h' {copy-mode -q; send-keys -l -- \"#{q:mouse_hyperlink}\"} '#{?mouse_hyperlink,Copy #[underscore]#{=/9/...:mouse_hyperlink},}' 'h' {copy-mode -q; set-buffer -- \"#{q:mouse_hyperlink}\"} '' '#{?#{#{pane_floating_flag}},Move,}' '' {display-menu -xL -yL -T '#[align=centre]Move'  'Centre' 'c' {move-pane -P centre} '' 'Top Left' '1' {move-pane -P top-left} 'Top Right' '2' {move-pane -P top-right} 'Bottom Left' '3' {move-pane -P bottom-left} 'Bottom Right' '4' {move-pane -P bottom-right} '' 'Top' 't' {move-pane -P top-centre} 'Bottom' 'b' {move-pane -P bottom-centre} 'Left' 'l' {move-pane -P centre-left} 'Right' 'r' {move-pane -P centre-right} } '#{?#{#{pane_floating_flag}},Move & Resize,}' '' {display-menu -xL -yL -T '#[align=centre]Move & Resize'  'Fill' '0' {resize-pane -x100% -y100%; move-pane -P top-left} '' 'Top Left' '1' {resize-pane -x50% -y50%; move-pane -P top-left} 'Top Right' '2' {resize-pane -x50% -y50%; move-pane -P top-right} 'Bottom Left' '3' {resize-pane -x50% -y50%; move-pane -P bottom-left} 'Bottom Right' '4' {resize-pane -x50% -y50%; move-pane -P bottom-right} '' 'Top' 't' {resize-pane -x100% -y50%; move-pane -P top-centre} 'Bottom' 'b' {resize-pane -x100% -y50%; move-pane -P bottom-centre} 'Left' 'l' {resize-pane -x50% -y100%; move-pane -P centre-left} 'Right' 'r' {resize-pane -x50% -y100%; move-pane -P centre-right} } '#{?#{#{pane_floating_flag}},Tile,}' 't' { join-pane } '#{?#{!:#{pane_floating_flag}},Float,}' 'f' { break-pane -W } '#{?#{!:#{pane_floating_flag}},Horizontal Split,}' 'h' {split-window -h} '#{?#{!:#{pane_floating_flag}},Vertical Split,}' 'v' {split-window -v} '' '#{?#{&&:#{!:#{pane_floating_flag}},#{>:#{window_panes},1}},Swap Up,}' 'u' {swap-pane -U} '#{?#{&&:#{!:#{pane_floating_flag}},#{>:#{window_panes},1}},Swap Down,}' 'd' {swap-pane -D} '#{?pane_marked_set,,-}Swap Marked' 's' {swap-pane} '' 'Kill' 'X' {kill-pane} 'Respawn' 'R' {respawn-pane -k} '#{?pane_marked,Unmark,Mark}' 'm' {select-pane -m} '#{?#{>:#{window_panes},1},,-}#{?window_zoomed_flag,Unzoom,Zoom}' 'z' {resize-pane -Z} } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n M-MouseDown3Pane { display-menu -t= -xM -yM -T '#[align=centre]#{pane_index} (#{pane_id})'  '#{?#{m/r:(copy|view)-mode,#{pane_mode}},Go To Top,}' '<' {send -X history-top} '#{?#{m/r:(copy|view)-mode,#{pane_mode}},Go To Bottom,}' '>' {send -X history-bottom} '' '#{?#{==:#{pane_mode},copy-mode},#{?copy_line_numbers,Hide Line Numbers,Show Line Numbers},}' 'L' {send -X line-numbers-toggle} '#{?#{==:#{pane_mode},copy-mode},#{?refresh_active,Refresh Off,Refresh On},}' 'r' {send -X refresh-toggle} '' '#{?#{&&:#{buffer_size},#{!:#{pane_in_mode}}},Paste #[underscore]#{=/9/...:buffer_sample},}' 'p' {paste-buffer} '' '#{?mouse_word,Search For #[underscore]#{=/9/...:mouse_word},}' 'C-r' {if -F '#{?#{m/r:(copy|view)-mode,#{pane_mode}},0,1}' 'copy-mode -t='; send -Xt= search-backward -- \"#{q:mouse_word}\"} '#{?mouse_word,Type #[underscore]#{=/9/...:mouse_word},}' 'C-y' {copy-mode -q; send-keys -l -- \"#{q:mouse_word}\"} '#{?mouse_word,Copy #[underscore]#{=/9/...:mouse_word},}' 'c' {copy-mode -q; set-buffer -- \"#{q:mouse_word}\"} '#{?mouse_line,Copy Line,}' 'l' {copy-mode -q; set-buffer -- \"#{q:mouse_line}\"} '' '#{?mouse_hyperlink,Type #[underscore]#{=/9/...:mouse_hyperlink},}' 'C-h' {copy-mode -q; send-keys -l -- \"#{q:mouse_hyperlink}\"} '#{?mouse_hyperlink,Copy #[underscore]#{=/9/...:mouse_hyperlink},}' 'h' {copy-mode -q; set-buffer -- \"#{q:mouse_hyperlink}\"} '' '#{?#{#{pane_floating_flag}},Move,}' '' {display-menu -xL -yL -T '#[align=centre]Move'  'Centre' 'c' {move-pane -P centre} '' 'Top Left' '1' {move-pane -P top-left} 'Top Right' '2' {move-pane -P top-right} 'Bottom Left' '3' {move-pane -P bottom-left} 'Bottom Right' '4' {move-pane -P bottom-right} '' 'Top' 't' {move-pane -P top-centre} 'Bottom' 'b' {move-pane -P bottom-centre} 'Left' 'l' {move-pane -P centre-left} 'Right' 'r' {move-pane -P centre-right} } '#{?#{#{pane_floating_flag}},Move & Resize,}' '' {display-menu -xL -yL -T '#[align=centre]Move & Resize'  'Fill' '0' {resize-pane -x100% -y100%; move-pane -P top-left} '' 'Top Left' '1' {resize-pane -x50% -y50%; move-pane -P top-left} 'Top Right' '2' {resize-pane -x50% -y50%; move-pane -P top-right} 'Bottom Left' '3' {resize-pane -x50% -y50%; move-pane -P bottom-left} 'Bottom Right' '4' {resize-pane -x50% -y50%; move-pane -P bottom-right} '' 'Top' 't' {resize-pane -x100% -y50%; move-pane -P top-centre} 'Bottom' 'b' {resize-pane -x100% -y50%; move-pane -P bottom-centre} 'Left' 'l' {resize-pane -x50% -y100%; move-pane -P centre-left} 'Right' 'r' {resize-pane -x50% -y100%; move-pane -P centre-right} } '#{?#{#{pane_floating_flag}},Tile,}' 't' { join-pane } '#{?#{!:#{pane_floating_flag}},Float,}' 'f' { break-pane -W } '#{?#{!:#{pane_floating_flag}},Horizontal Split,}' 'h' {split-window -h} '#{?#{!:#{pane_floating_flag}},Vertical Split,}' 'v' {split-window -v} '' '#{?#{&&:#{!:#{pane_floating_flag}},#{>:#{window_panes},1}},Swap Up,}' 'u' {swap-pane -U} '#{?#{&&:#{!:#{pane_floating_flag}},#{>:#{window_panes},1}},Swap Down,}' 'd' {swap-pane -D} '#{?pane_marked_set,,-}Swap Marked' 's' {swap-pane} '' 'Kill' 'X' {kill-pane} 'Respawn' 'R' {respawn-pane -k} '#{?pane_marked,Unmark,Mark}' 'm' {select-pane -m} '#{?#{>:#{window_panes},1},,-}#{?window_zoomed_flag,Unzoom,Zoom}' 'z' {resize-pane -Z} }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n MouseDown3Empty { display-menu -t= -xM -yM -T '#[align=centre]#{window_index}:#{window_name}'  'New Pane' 'p' {new-pane; join-pane} 'New Window' 'w' {new-window} }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n M-MouseDown3Empty { display-menu -t= -xM -yM -T '#[align=centre]#{window_index}:#{window_name}'  'New Pane' 'p' {new-pane; join-pane} 'New Window' 'w' {new-window} }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n MouseDown1ScrollbarUp { if -Ft= '#{pane_in_mode}' { send -X page-up } {copy-mode -u } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n MouseDown1ScrollbarDown { if -Ft= '#{pane_in_mode}' { send -X page-down } {copy-mode -d } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n MouseDrag1ScrollbarSlider { if -Ft= '#{pane_in_mode}' { send -X scroll-to-mouse } { copy-mode -S } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-Space { send -X begin-selection }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-a { send -X start-of-line }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-c { send -X cancel }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-e { send -X end-of-line }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-f { send -X cursor-right }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-b { send -X cursor-left }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-g { send -X clear-selection }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-k { send -X copy-pipe-end-of-line-and-cancel }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-l { send -X recentre-top-bottom }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-l { send -X cursor-centre-horizontal }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-n { send -X cursor-down }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-p { send -X cursor-up }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-r { command-prompt -P -T search -ip'(search up)' -I'#{pane_search_string}' { send -X search-backward-incremental -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-s { command-prompt -P -T search -ip'(search down)' -I'#{pane_search_string}' { send -X search-forward-incremental -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-v { send -X page-down }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-w { send -X copy-pipe-and-cancel }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode Escape { send -X cancel }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-[ { send -X cancel }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode Space { send -X page-down }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode , { send -X jump-reverse }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode \\; { send -X jump-again }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode F { command-prompt -P -1p'(jump backward)' { send -X jump-backward -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode L { send -X line-numbers-toggle }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode N { send -X search-reverse }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode P { send -X toggle-position }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode R { send -X rectangle-toggle }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode T { command-prompt -P -1p'(jump to backward)' { send -X jump-to-backward -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode X { send -X set-mark }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode f { command-prompt -P -1p'(jump forward)' { send -X jump-forward -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode g { command-prompt -P -p'(goto line)' { send -X goto-line -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode n { send -X search-again }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode q { send -X cancel }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode r { send -X refresh-now }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode t { command-prompt -P -1p'(jump to forward)' { send -X jump-to-forward -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode Home { send -X start-of-line }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode End { send -X end-of-line }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode MouseDown1Pane select-pane\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode MouseDrag1Pane { select-pane; send -X begin-selection }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode MouseDragEnd1Pane { send -X copy-pipe-and-cancel }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode WheelUpPane { select-pane; send -N5 -X scroll-up }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode WheelDownPane { select-pane; send -N5 -X scroll-down }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode DoubleClick1Pane { select-pane; send -X select-word; run -d0.3; send -X copy-pipe-and-cancel }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode TripleClick1Pane { select-pane; send -X select-line; run -d0.3; send -X copy-pipe-and-cancel }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode NPage { send -X page-down }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode PPage { send -X page-up }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode Up { send -X cursor-up }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode Down { send -X cursor-down }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode Left { send -X cursor-left }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode Right { send -X cursor-right }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-1 { command-prompt -P -Np'(repeat)' -I1 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-2 { command-prompt -P -Np'(repeat)' -I2 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-3 { command-prompt -P -Np'(repeat)' -I3 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-4 { command-prompt -P -Np'(repeat)' -I4 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-5 { command-prompt -P -Np'(repeat)' -I5 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-6 { command-prompt -P -Np'(repeat)' -I6 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-7 { command-prompt -P -Np'(repeat)' -I7 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-8 { command-prompt -P -Np'(repeat)' -I8 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-9 { command-prompt -P -Np'(repeat)' -I9 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-< { send -X history-top }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-> { send -X history-bottom }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-R { send -X top-line }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-b { send -X previous-word }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-M-b { send -X previous-matching-bracket }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-f { send -X next-word-end }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-M-f { send -X next-matching-bracket }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-m { send -X back-to-indentation }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-r { send -X middle-line }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-v { send -X page-up }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-w { send -X copy-pipe-and-cancel }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-x { send -X jump-to-mark }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode 'M-{' { send -X previous-paragraph }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode 'M-}' { send -X next-paragraph }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-Up { send -X halfpage-up }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-Down { send -X halfpage-down }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-Up { send -X scroll-up }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-Down { send -X scroll-down }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-C-Up { send -X previous-prompt }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-C-Down { send -X next-prompt }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi '#' { send -FX search-backward -- '#{copy_cursor_word}' }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi * { send -FX search-forward -- '#{copy_cursor_word}' }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi C-c { send -X cancel }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi C-d { send -X halfpage-down }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi C-e { send -X scroll-down }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi C-b { send -X page-up }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi C-f { send -X page-down }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi C-h { send -X cursor-left }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi C-j { send -X copy-pipe-and-cancel }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi Enter { send -X copy-pipe-and-cancel }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi C-u { send -X halfpage-up }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi C-v { send -X rectangle-toggle }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi C-y { send -X scroll-up }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi Escape { send -X clear-selection }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi C-[ { send -X clear-selection }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi Space { send -X begin-selection }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi '$' { send -X end-of-line }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi , { send -X jump-reverse }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi / { command-prompt -P -T search -p'(search down)' { send -X search-forward -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi 0 { send -X start-of-line }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi 1 { command-prompt -P -Np'(repeat)' -I1 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi 2 { command-prompt -P -Np'(repeat)' -I2 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi 3 { command-prompt -P -Np'(repeat)' -I3 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi 4 { command-prompt -P -Np'(repeat)' -I4 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi 5 { command-prompt -P -Np'(repeat)' -I5 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi 6 { command-prompt -P -Np'(repeat)' -I6 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi 7 { command-prompt -P -Np'(repeat)' -I7 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi 8 { command-prompt -P -Np'(repeat)' -I8 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi 9 { command-prompt -P -Np'(repeat)' -I9 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi : { command-prompt -P -p'(goto line)' { send -X goto-line -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi \\; { send -X jump-again }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi ? { command-prompt -P -T search -p'(search up)' { send -X search-backward -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi A { send -X append-selection-and-cancel }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi B { send -X previous-space }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi D { send -X copy-pipe-end-of-line-and-cancel }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi E { send -X next-space-end }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi F { command-prompt -P -1p'(jump backward)' { send -X jump-backward -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi G { send -X history-bottom }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi H { send -X top-line }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi J { send -X scroll-down }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi K { send -X scroll-up }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi L { send -X bottom-line }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi M { send -X middle-line }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi N { send -X search-reverse }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi P { send -X toggle-position }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi T { command-prompt -P -1p'(jump to backward)' { send -X jump-to-backward -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi V { send -X select-line }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi W { send -X next-space }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi X { send -X set-mark }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi ^ { send -X back-to-indentation }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi b { send -X previous-word }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi e { send -X next-word-end }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi f { command-prompt -P -1p'(jump forward)' { send -X jump-forward -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi g { send -X history-top }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi h { send -X cursor-left }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi j { send -X cursor-down }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi k { send -X cursor-up }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi z { send -X scroll-middle }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi l { send -X cursor-right }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi n { send -X search-again }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi o { send -X other-end }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi q { send -X cancel }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi r { send -X refresh-now }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi t { command-prompt -P -1p'(jump to forward)' { send -X jump-to-forward -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi v { send -X rectangle-toggle }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi w { send -X next-word }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi '{' { send -X previous-paragraph }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi '}' { send -X next-paragraph }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi % { send -X next-matching-bracket }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi Home { send -X start-of-line }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi End { send -X end-of-line }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi MouseDown1Pane { select-pane }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi MouseDrag1Pane { select-pane; send -X begin-selection }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi MouseDragEnd1Pane { send -X copy-pipe-and-cancel }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi WheelUpPane { select-pane; send -N5 -X scroll-up }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi WheelDownPane { select-pane; send -N5 -X scroll-down }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi DoubleClick1Pane { select-pane; send -X select-word; run -d0.3; send -X copy-pipe-and-cancel }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi TripleClick1Pane { select-pane; send -X select-line; run -d0.3; send -X copy-pipe-and-cancel }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi BSpace { send -X cursor-left }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi NPage { send -X page-down }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi PPage { send -X page-up }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi Up { send -X cursor-up }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi Down { send -X cursor-down }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi Left { send -X cursor-left }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi Right { send -X cursor-right }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi M-x { send -X jump-to-mark }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi C-Up { send -X scroll-up }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi C-Down { send -X scroll-down }\0" as *const u8
            as *const ::core::ffi::c_char,
    ];
    let mut i: u_int = 0;
    let mut pr: cmd_parse_result = cmd_parse_result::empty();
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[*const ::core::ffi::c_char; 308]>() as usize)
            .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>() as usize)
    {
        pr = cmd_parse_from_string(
            CStr::from_ptr(defaults[i as usize]),
            ::core::ptr::null_mut::<cmd_parse_input>(),
        );
        if pr.status as ::core::ffi::c_uint
            != CMD_PARSE_SUCCESS as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            log_debug(
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                pr.error
                    .as_ref()
                    .map_or(::core::ptr::null(), |cause| cause.as_ptr()),
            );
            fatalx(
                b"bad default key: %s\0" as *const u8 as *const ::core::ffi::c_char,
                defaults[i as usize],
            );
        }
        cmdq_append(
            ::core::ptr::null_mut::<client>(),
            cmdq_get_command(pr.cmdlist, ::core::ptr::null_mut::<cmdq_state>()),
        );
        cmd_list_free(pr.cmdlist);
        i = i.wrapping_add(1);
    }
    cmdq_append(
        ::core::ptr::null_mut::<client>(),
        cmdq_get_callback1(
            b"key_bindings_init_done\0" as *const u8 as *const ::core::ffi::c_char,
            Some(
                key_bindings_init_done
                    as unsafe extern "C" fn(*mut cmdq_item, *mut ::core::ffi::c_void) -> cmd_retval,
            ),
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
        ),
    );
}
unsafe extern "C" fn key_bindings_read_only(
    mut item: *mut cmdq_item,
    mut data: *mut ::core::ffi::c_void,
) -> cmd_retval {
    cmdq_error(
        item,
        b"client is read-only\0" as *const u8 as *const ::core::ffi::c_char,
    );
    return CMD_RETURN_ERROR;
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_dispatch(
    mut bd: *mut key_binding,
    mut item: *mut cmdq_item,
    mut c: *mut client,
    mut event: *mut key_event,
    mut fs: *mut cmd_find_state,
) -> *mut cmdq_item {
    let mut new_item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut new_state: *mut cmdq_state = ::core::ptr::null_mut::<cmdq_state>();
    let mut readonly: ::core::ffi::c_int = 0;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if c.is_null() || !(*c).flags & CLIENT_READONLY as uint64_t != 0 {
        readonly = 1 as ::core::ffi::c_int;
    } else {
        readonly = cmd_list_all_have((*bd).cmdlist, CMD_READONLY);
    }
    if readonly == 0 {
        new_item = cmdq_get_callback1(
            b"key_bindings_read_only\0" as *const u8 as *const ::core::ffi::c_char,
            Some(
                key_bindings_read_only
                    as unsafe extern "C" fn(*mut cmdq_item, *mut ::core::ffi::c_void) -> cmd_retval,
            ),
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
        );
    } else {
        if (*bd).flags & KEY_BINDING_REPEAT != 0 {
            flags |= CMDQ_STATE_REPEAT;
        }
        new_state = cmdq_new_state(fs, event, flags);
        new_item = cmdq_get_command((*bd).cmdlist, new_state);
        cmdq_free_state(new_state);
    }
    if !item.is_null() {
        new_item = cmdq_insert_after(item, new_item);
    } else {
        new_item = cmdq_append(c, new_item);
    }
    return new_item;
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_has_repeat(
    mut l: *mut *mut key_binding,
    mut n: u_int,
) -> ::core::ffi::c_int {
    let mut i: u_int = 0;
    i = 0 as u_int;
    while i < n {
        if (**l.offset(i as isize)).flags & KEY_BINDING_REPEAT != 0 {
            return 1 as ::core::ffi::c_int;
        }
        i = i.wrapping_add(1);
    }
    return 0 as ::core::ffi::c_int;
}

fn key_bindings_key(elm: &key_binding) -> u64 {
    elm.key
}
pub unsafe fn key_bindings_index_find(head: &key_bindings, elm: &key_binding) -> *mut key_binding {
    let Some(map) = head.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = key_bindings_key(elm);
    map.get(&key).copied().unwrap_or(std::ptr::null_mut())
}
pub unsafe fn key_bindings_index_nfind(head: &key_bindings, elm: &key_binding) -> *mut key_binding {
    let Some(map) = head.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = key_bindings_key(elm);
    map.range((std::ops::Bound::Included(&key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn key_bindings_index_insert(
    head: *mut key_bindings,
    elm: *mut key_binding,
) -> *mut key_binding {
    let key = key_bindings_key(&*elm);
    if (*head).storage.is_null() {
        (*head).storage = Box::into_raw(Box::new(std::collections::BTreeMap::new()));
    }
    let map = &mut *(*head).storage;
    match map.entry(key) {
        std::collections::btree_map::Entry::Occupied(entry) => return *entry.get(),
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(elm);
        }
    }
    (*elm).entry.owner = map as *mut _;
    std::ptr::null_mut()
}
pub unsafe fn key_bindings_index_remove(
    head: *mut key_bindings,
    elm: *mut key_binding,
) -> *mut key_binding {
    if elm.is_null() {
        return std::ptr::null_mut();
    }
    let key = key_bindings_key(&*elm);
    let Some(map) = (*head).storage.as_mut() else {
        return std::ptr::null_mut();
    };
    if map.get(&key).copied() != Some(elm) {
        return std::ptr::null_mut();
    }
    map.remove(&key);
    (*elm).entry.owner = std::ptr::null_mut();
    if map.is_empty() {
        drop(Box::from_raw((*head).storage));
        (*head).storage = std::ptr::null_mut();
    }
    elm
}
pub unsafe fn key_bindings_index_minmax(
    head: &key_bindings,
    direction: ::core::ffi::c_int,
) -> *mut key_binding {
    let Some(map) = head.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let pair = if direction < 0 {
        map.first_key_value()
    } else {
        map.last_key_value()
    };
    pair.map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn key_bindings_index_next(elm: &key_binding) -> *mut key_binding {
    let Some(map) = elm.entry.owner.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = key_bindings_key(elm);
    map.range((std::ops::Bound::Excluded(&key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn key_bindings_index_prev(elm: &key_binding) -> *mut key_binding {
    let Some(map) = elm.entry.owner.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = key_bindings_key(elm);
    map.range((std::ops::Bound::Unbounded, std::ops::Bound::Excluded(&key)))
        .next_back()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}

pub unsafe fn key_tables_find(head: &key_tables, elm: &key_table) -> *mut key_table {
    let Some(map) = head.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = elm.name.as_c_str().to_bytes();
    map.get(key).copied().unwrap_or(std::ptr::null_mut())
}
pub unsafe fn key_tables_nfind(head: &key_tables, elm: &key_table) -> *mut key_table {
    let Some(map) = head.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = elm.name.as_c_str().to_bytes();
    map.range::<[u8], _>((std::ops::Bound::Included(key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn key_tables_insert(head: *mut key_tables, elm: *mut key_table) -> *mut key_table {
    let key = std::ffi::CStr::from_ptr(((*elm).name).as_ptr().cast_mut()).to_bytes();
    if (*head).storage.is_null() {
        (*head).storage = Box::into_raw(Box::new(std::collections::BTreeMap::new()));
    }
    let map = &mut *(*head).storage;
    match map.entry(key.to_vec()) {
        std::collections::btree_map::Entry::Occupied(entry) => return *entry.get(),
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(elm);
        }
    }
    (*elm).entry.owner = map as *mut _;
    std::ptr::null_mut()
}
pub unsafe fn key_tables_remove(head: *mut key_tables, elm: *mut key_table) -> *mut key_table {
    if elm.is_null() {
        return std::ptr::null_mut();
    }
    let key = std::ffi::CStr::from_ptr(((*elm).name).as_ptr().cast_mut()).to_bytes();
    let Some(map) = (*head).storage.as_mut() else {
        return std::ptr::null_mut();
    };
    if map.get(key).copied() != Some(elm) {
        return std::ptr::null_mut();
    }
    map.remove(key);
    (*elm).entry.owner = std::ptr::null_mut();
    if map.is_empty() {
        drop(Box::from_raw((*head).storage));
        (*head).storage = std::ptr::null_mut();
    }
    elm
}
pub unsafe fn key_tables_minmax(
    head: &key_tables,
    direction: ::core::ffi::c_int,
) -> *mut key_table {
    let Some(map) = head.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let pair = if direction < 0 {
        map.first_key_value()
    } else {
        map.last_key_value()
    };
    pair.map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn key_tables_next(elm: &key_table) -> *mut key_table {
    let Some(map) = elm.entry.owner.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = std::ffi::CStr::from_ptr(elm.name.as_ptr().cast_mut()).to_bytes();
    map.range::<[u8], _>((std::ops::Bound::Excluded(key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn key_tables_prev(elm: &key_table) -> *mut key_table {
    let Some(map) = elm.entry.owner.as_ref() else {
        return std::ptr::null_mut();
    };
    let key = std::ffi::CStr::from_ptr(elm.name.as_ptr().cast_mut()).to_bytes();
    map.range::<[u8], _>((std::ops::Bound::Unbounded, std::ops::Bound::Excluded(key)))
        .next_back()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}
