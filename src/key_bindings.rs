use crate::src::cmd::parse::cmd_parse_from_string;
use crate::src::cmd::queue::{
    cmdq_append, cmdq_error, cmdq_get_callback_owned, cmdq_get_command, cmdq_insert_after,
    cmdq_new_state,
};
use crate::src::cmd::{cmd_list_all_have, cmd_list_print_cstring};
use crate::src::ffi::libc::strcmp;
use crate::src::format::bytes::write_cstr;
use crate::src::key_string::key_string_format;
use crate::src::log::{fatalx, log_cstr, log_debug, log_hex};
use crate::src::server::clients;
use crate::src::server_client::server_client_set_key_table;
use crate::src::shared::abi::*;
use crate::src::shared::client::CLIENT_READONLY;
use crate::src::shared::client::client;
use crate::src::shared::command::*;
use crate::src::shared::command::{CMD_READONLY, CMDQ_STATE_REPEAT};
use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_state};
use crate::src::shared::command::{cmd_parse_input, cmd_parse_result};
use crate::src::shared::key::KEY_BINDING_REPEAT;
use crate::src::shared::key::*;
use crate::src::shared::key::{key_binding, key_bindings, key_event, key_table, key_table_entry};
use crate::src::shared::tree::RB_NEGINF;
use std::ffi::CStr;

#[repr(C)]
pub struct key_tables {
    pub storage: Option<
        refbox::RefBox<
            std::collections::BTreeMap<Vec<u8>, std::rc::Rc<std::cell::RefCell<key_table>>>,
        >,
    >,
}

static mut key_tables: key_tables = key_tables { storage: None };

pub(crate) fn key_bindings_set_note(bd: &mut key_binding, note: Option<&CStr>) {
    bd.note = note.map(CStr::to_owned);
}

pub unsafe fn key_bindings_get_table(
    name: &CStr,
    create: i32,
) -> Option<std::rc::Rc<std::cell::RefCell<key_table>>> {
    unsafe {
        if let Some(index) = (*std::ptr::addr_of!(key_tables)).storage.as_ref() {
            let map = index
                .try_borrow_mut()
                .expect("key table index already borrowed");
            if let Some(table) = map.get(name.to_bytes()) {
                return Some(table.clone());
            }
        }
        if create == 0 {
            return None;
        }
        let mut value = key_table::empty();
        value.name = name.to_owned();
        let table = std::rc::Rc::new(std::cell::RefCell::new(value));
        key_tables_insert(&raw mut key_tables, table.clone());
        Some(table)
    }
}
pub fn key_bindings_get(table: &key_table, key: key_code) -> Option<&key_binding> {
    table.key_bindings.get(key)
}

pub fn key_bindings_get_default(table: &key_table, key: key_code) -> Option<&key_binding> {
    table.default_key_bindings.get(key)
}

/// Retain the tables while callers borrow their bindings for listing.
pub unsafe fn key_bindings_tables() -> Vec<std::rc::Rc<std::cell::RefCell<key_table>>> {
    unsafe {
        let Some(index) = (*std::ptr::addr_of!(key_tables)).storage.as_ref() else {
            return Vec::new();
        };
        let map = index
            .try_borrow_mut()
            .expect("key table index already borrowed");
        map.values().cloned().collect()
    }
}

pub unsafe fn key_bindings_add(
    name: &CStr,
    key: key_code,
    note: Option<&CStr>,
    repeat: ::core::ffi::c_int,
    cmdlist: Option<std::rc::Rc<std::cell::RefCell<cmd_list>>>,
) {
    unsafe {
        let owner = key_bindings_get_table(name, 1).expect("created key table");
        let mut table = owner.borrow_mut();
        let key = key & !KEYC_MASK_FLAGS;
        let Some(cmdlist) = cmdlist else {
            if let Some(bd) = table.key_bindings.get_mut(key) {
                if let Some(note) = note {
                    key_bindings_set_note(bd, Some(note));
                }
                if repeat != 0 {
                    bd.flags |= KEY_BINDING_REPEAT;
                }
            }
            return;
        };
        drop(table.key_bindings.remove(key));
        let bd = Box::new(key_binding {
            key,
            commands: cmdlist,
            note: note.map(CStr::to_owned),
            tablename: Some(table.name.clone()),
            flags: if repeat != 0 { KEY_BINDING_REPEAT } else { 0 },
        });
        let s = cmd_list_print_cstring(&bd.cmdlist().borrow(), 0);
        let key_string = key_string_format(key, true);
        table.key_bindings.insert(bd);
        log_debug(format_args!(
            "key_bindings_add: {} {} = {}",
            log_hex(key),
            crate::src::log::log_bytes(key_string.as_bytes()),
            crate::src::log::log_bytes(s.as_bytes())
        ));
    }
}

pub unsafe fn key_bindings_remove(name: &CStr, key: key_code) {
    unsafe {
        let Some(owner) = key_bindings_get_table(name, 0) else {
            return;
        };
        let mut table = owner.borrow_mut();
        let Some(bd) = table.key_bindings.remove(key & !KEYC_MASK_FLAGS) else {
            return;
        };
        let key_string = key_string_format(bd.key, true);
        log_debug(format_args!(
            "key_bindings_remove: {} {}",
            log_hex(bd.key),
            crate::src::log::log_bytes(key_string.as_bytes())
        ));
        drop(bd);
        if table.key_bindings.storage.is_none() && table.default_key_bindings.storage.is_none() {
            drop(table);
            drop(key_tables_remove(&raw mut key_tables, &owner));
        }
    }
}

pub unsafe fn key_bindings_reset(name: &CStr, key: key_code) {
    unsafe {
        let Some(owner) = key_bindings_get_table(name, 0) else {
            return;
        };
        let mut table = owner.borrow_mut();
        let key = key & !KEYC_MASK_FLAGS;
        let table_ref = &mut *table;
        let Some(bd) = table_ref.key_bindings.get_mut(key) else {
            return;
        };
        let Some(dd) = table_ref.default_key_bindings.get(key) else {
            drop(table);
            key_bindings_remove(name, key);
            return;
        };
        bd.commands = std::rc::Rc::clone(&dd.commands);
        bd.note = dd.note.clone();
        bd.flags = dd.flags;
    }
}

pub unsafe fn key_bindings_remove_table(name: &CStr) {
    unsafe {
        let mut c: *mut client = ::core::ptr::null_mut::<client>();
        if let Some(owner) = key_bindings_get_table(name, 0) {
            let detached = key_tables_remove(&raw mut key_tables, &owner);
            let mut registry_c_owner = clients.first();
            c = registry_c_owner
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get());
            while !c.is_null() {
                if (*c)
                    .keytable
                    .as_ref()
                    .is_some_and(|current| std::rc::Rc::ptr_eq(current, &owner))
                {
                    server_client_set_key_table(
                        &(*(c)).observer.upgrade().expect("live client"),
                        ::core::ptr::null::<::core::ffi::c_char>(),
                    );
                }
                registry_c_owner =
                    clients.next(registry_c_owner.as_ref().expect("current registry client"));
                c = registry_c_owner
                    .as_ref()
                    .map_or(std::ptr::null_mut(), |owner| owner.get());
            }
            drop(detached);
        }
    }
}

/// Snapshot startup defaults with their own command-list reference and note.
unsafe fn key_bindings_init_done() -> cmd_retval {
    unsafe {
        for owner in key_bindings_tables() {
            let mut table = owner.borrow_mut();
            let table_ref = &mut *table;
            for bd in table_ref.key_bindings.iter() {
                table_ref.default_key_bindings.insert(Box::new(key_binding {
                    key: bd.key,
                    commands: std::rc::Rc::clone(&bd.commands),
                    note: bd.note.clone(),
                    tablename: None,
                    flags: bd.flags,
                }));
            }
        }
        CMD_RETURN_NORMAL
    }
}

pub unsafe fn key_bindings_init() {
    unsafe {
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
                log_debug(format_args!(
                    "{}",
                    log_cstr(
                        (pr.error
                            .as_ref()
                            .map_or(::core::ptr::null(), |cause| cause.as_ptr()))
                            as *const _
                    )
                ));
                fatalx(|out| {
                    out.write_all(b"bad default key: ")?;
                    write_cstr(out, defaults[i as usize])
                });
            }
            cmdq_append(
                None,
                cmdq_get_command(pr.cmdlist.as_ref().expect("successful command parse"), None),
            );
            drop(pr.cmdlist.take());
            i = i.wrapping_add(1);
        }
        cmdq_append(
            None,
            cmdq_get_callback_owned(
                c"key_bindings_init_done",
                Some(Box::new(|_| unsafe { key_bindings_init_done() })),
            ),
        );
    }
}
unsafe fn key_bindings_read_only(
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
) -> cmd_retval {
    unsafe {
        cmdq_error(item_handle, |out| out.write_all(b"client is read-only"));
        return CMD_RETURN_ERROR;
    }
}
pub unsafe fn key_bindings_dispatch(
    bd: KeyBindingCommand,
    item_handle: Option<&std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>>,
    c_owner: Option<&std::rc::Rc<std::cell::UnsafeCell<client>>>,
    mut event: *mut key_event,
    mut fs: *mut cmd_find_state,
) -> std::rc::Weak<std::cell::UnsafeCell<cmdq_item>> {
    unsafe {
        let mut c = c_owner.map_or(std::ptr::null_mut(), |owner| owner.get());
        let new_item_allocation;
        let new_state;
        let mut readonly: ::core::ffi::c_int = 0;
        let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if c.is_null() || !(*c).flags & CLIENT_READONLY as uint64_t != 0 {
            readonly = 1 as ::core::ffi::c_int;
        } else {
            readonly = cmd_list_all_have(&bd.cmdlist().borrow());
        }
        if readonly == 0 {
            new_item_allocation = cmdq_get_callback_owned(
                c"key_bindings_read_only",
                Some(Box::new(|item| unsafe { key_bindings_read_only(item) })),
            );
        } else {
            if bd.flags & KEY_BINDING_REPEAT != 0 {
                flags |= CMDQ_STATE_REPEAT;
            }
            new_state = cmdq_new_state(fs, event, flags);
            new_item_allocation = cmdq_get_command(&bd.commands, Some(&new_state));
        }
        if let Some(item) = item_handle {
            cmdq_insert_after(item, new_item_allocation)
        } else {
            cmdq_append(c_owner, new_item_allocation)
        }
    }
}

pub fn key_bindings_has_repeat(bindings: &[&key_binding]) -> bool {
    bindings.iter().any(|bd| bd.flags & KEY_BINDING_REPEAT != 0)
}

pub fn key_tables_find(
    head: &key_tables,
    elm: &key_table,
) -> Option<std::rc::Rc<std::cell::RefCell<key_table>>> {
    let owner = head.storage.as_ref()?;
    let map = owner
        .try_borrow_mut()
        .expect("key table index already borrowed");
    map.get(elm.name.as_bytes()).cloned()
}
pub unsafe fn key_tables_insert(
    head: *mut key_tables,
    table: std::rc::Rc<std::cell::RefCell<key_table>>,
) -> Option<std::rc::Rc<std::cell::RefCell<key_table>>> {
    unsafe {
        let key = table.borrow().name.as_bytes().to_vec();
        let owner = (*head).storage.get_or_insert_with(refbox::RefBox::default);
        let observer = owner.downgrade();
        let mut map = owner
            .try_borrow_mut()
            .expect("key table index already borrowed");
        if let Some(existing) = map.get(&key) {
            return Some(existing.clone());
        }
        table.borrow_mut().entry.owner = observer;
        map.insert(key, table);
        None
    }
}
pub unsafe fn key_tables_remove(
    head: *mut key_tables,
    table: &std::rc::Rc<std::cell::RefCell<key_table>>,
) -> Option<std::rc::Rc<std::cell::RefCell<key_table>>> {
    unsafe {
        let key = table.borrow().name.as_bytes().to_vec();
        let owner = (*head).storage.as_ref()?;
        let (detached, empty) = {
            let mut map = owner
                .try_borrow_mut()
                .expect("key table index already borrowed");
            if !map
                .get(&key)
                .is_some_and(|candidate| std::rc::Rc::ptr_eq(candidate, table))
            {
                return None;
            }
            let detached = map.remove(&key).expect("matching key table");
            (detached, map.is_empty())
        };
        table.borrow_mut().entry.owner = refbox::Weak::new();
        if empty {
            (*head).storage = None;
        }
        Some(detached)
    }
}
pub fn key_tables_minmax(head: &key_tables) -> Option<std::rc::Rc<std::cell::RefCell<key_table>>> {
    let owner = head.storage.as_ref()?;
    let map = owner
        .try_borrow_mut()
        .expect("key table index already borrowed");
    map.first_key_value().map(|(_, table)| table.clone())
}
pub fn key_tables_next(elm: &key_table) -> Option<std::rc::Rc<std::cell::RefCell<key_table>>> {
    let owner = &elm.entry.owner;
    let map = match owner.try_borrow_mut() {
        Ok(map) => map,
        Err(refbox::BorrowError::Dropped) => return None,
        Err(refbox::BorrowError::Borrowed) => panic!("key table index already borrowed"),
    };
    map.range::<[u8], _>((
        std::ops::Bound::Excluded(elm.name.as_bytes()),
        std::ops::Bound::Unbounded,
    ))
    .next()
    .map(|(_, table)| table.clone())
}

#[cfg(test)]
mod ownership_tests {
    use super::*;
    use crate::src::cmd::cmd_list_new;
    use crate::src::shared::rc;
    use std::rc::Rc;

    unsafe fn binding(key: key_code) -> Box<key_binding> {
        unsafe {
            Box::new(key_binding {
                key,
                commands: cmd_list_new(),
                note: None,
                tablename: None,
                flags: 0,
            })
        }
    }

    #[test]
    fn table_index_returns_owners_and_removes_only_matching_allocations() {
        use std::cell::RefCell;
        unsafe {
            let mut index = key_tables { storage: None };
            let mut a = key_table::empty();
            a.name = c"a".to_owned();
            let a = Rc::new(RefCell::new(a));
            let mut b = key_table::empty();
            b.name = c"b".to_owned();
            let b = Rc::new(RefCell::new(b));
            let mut duplicate = key_table::empty();
            duplicate.name = c"a".to_owned();
            let duplicate = Rc::new(RefCell::new(duplicate));
            assert_ne!(a.borrow().identity, duplicate.borrow().identity);
            assert!(key_tables_insert(&mut index, a.clone()).is_none());
            assert!(key_tables_insert(&mut index, b.clone()).is_none());
            assert!(Rc::ptr_eq(
                &key_tables_insert(&mut index, duplicate.clone()).unwrap(),
                &a
            ));
            assert!(key_tables_remove(&mut index, &duplicate).is_none());
            assert!(Rc::ptr_eq(&key_tables_minmax(&index).unwrap(), &a));
            assert!(Rc::ptr_eq(&key_tables_next(&a.borrow()).unwrap(), &b));
            assert!(key_tables_next(&b.borrow()).is_none());
            let observed = Rc::downgrade(&a);
            let found = key_tables_find(&index, &duplicate.borrow()).unwrap();
            drop(key_tables_remove(&mut index, &a));
            assert!(a.borrow().entry.owner.is_empty());
            drop(a);
            assert!(observed.upgrade().is_some());
            drop(found);
            assert!(observed.upgrade().is_none());
            drop(key_tables_remove(&mut index, &b));
            assert!(index.storage.is_none());
        }
    }

    #[test]
    fn index_owns_bindings_and_keeps_addresses_stable() {
        unsafe {
            let mut index = key_bindings::default();
            let first = binding(8);
            let address = std::ptr::from_ref(&*first);
            let commands = Rc::downgrade(&first.commands);
            index.insert(first);
            index.insert(binding(2));
            index.insert(binding(13));
            assert_eq!(
                index.iter().map(|bd| bd.key).collect::<Vec<_>>(),
                [2, 8, 13]
            );
            assert_eq!(std::ptr::from_ref(index.get(8).unwrap()), address);
            let detached = index.remove(8).unwrap();
            assert_eq!(std::ptr::from_ref(&*detached), address);
            assert!(index.get(8).is_none());
            assert!(commands.upgrade().is_some());
            drop(detached);
            assert!(commands.upgrade().is_none());
            drop(index.remove(2));
            drop(index.remove(13));
            assert!(index.storage.is_none());
        }
    }

    #[test]
    fn defaults_restore_notes_flags_and_command_ownership() {
        unsafe {
            let name = c"binding-owner-defaults";
            let original = cmd_list_new();
            let original_lifetime = std::rc::Rc::downgrade(&original);
            key_bindings_add(
                name,
                65,
                {
                    let note: *const ::core::ffi::c_char = c"original".as_ptr();
                    (!note.is_null()).then(|| std::ffi::CStr::from_ptr(note))
                },
                1,
                Some(original),
            );
            let table_owner = key_bindings_get_table(name, 0).unwrap();
            key_bindings_init_done();
            assert_eq!(original_lifetime.strong_count(), 2);
            let replacement = cmd_list_new();
            let replacement_lifetime = std::rc::Rc::downgrade(&replacement);
            key_bindings_add(
                name,
                65,
                {
                    let note: *const ::core::ffi::c_char = c"replacement".as_ptr();
                    (!note.is_null()).then(|| std::ffi::CStr::from_ptr(note))
                },
                0,
                Some(replacement),
            );
            assert_eq!(original_lifetime.strong_count(), 1);
            key_bindings_reset(name, 65);
            assert!(replacement_lifetime.upgrade().is_none());
            let table_borrow = table_owner.borrow();
            let bd = key_bindings_get(&table_borrow, 65).unwrap();
            assert_eq!(bd.note.as_deref(), Some(c"original"));
            assert_eq!(bd.flags, KEY_BINDING_REPEAT);
            assert!(std::rc::Weak::ptr_eq(
                &std::rc::Rc::downgrade(bd.cmdlist()),
                &original_lifetime
            ));
            assert_eq!(original_lifetime.strong_count(), 2);
            drop(table_borrow);
            key_bindings_add(
                name,
                65,
                {
                    let note: *const ::core::ffi::c_char = c"note only".as_ptr();
                    (!note.is_null()).then(|| std::ffi::CStr::from_ptr(note))
                },
                0,
                None,
            );
            assert!(std::rc::Weak::ptr_eq(
                &std::rc::Rc::downgrade(
                    key_bindings_get(&table_owner.borrow(), 65)
                        .unwrap()
                        .cmdlist()
                ),
                &original_lifetime
            ));
            assert_eq!(original_lifetime.strong_count(), 2);
            key_bindings_add(
                name,
                66,
                {
                    let note: *const ::core::ffi::c_char = std::ptr::null();
                    (!note.is_null()).then(|| std::ffi::CStr::from_ptr(note))
                },
                0,
                Some(cmd_list_new()),
            );
            key_bindings_reset(name, 66);
            assert!(key_bindings_get(&table_owner.borrow(), 66).is_none());
            drop(table_owner);
            key_bindings_remove_table(name);
            assert!(original_lifetime.upgrade().is_none());
        }
    }

    #[test]
    fn dispatch_snapshot_survives_binding_replacement_and_table_removal() {
        unsafe {
            let name = c"binding-owner-dispatch";
            let original = cmd_list_new();
            let original_lifetime = std::rc::Rc::downgrade(&original);
            key_bindings_add(
                name,
                65,
                {
                    let note: *const ::core::ffi::c_char = std::ptr::null();
                    (!note.is_null()).then(|| std::ffi::CStr::from_ptr(note))
                },
                1,
                Some(original),
            );
            let table_owner = key_bindings_get_table(name, 0).unwrap();
            let table_lifetime = Rc::downgrade(&table_owner);
            let retained_table = table_lifetime.upgrade().unwrap();
            let command = key_bindings_get(&table_owner.borrow(), 65)
                .unwrap()
                .command();
            key_bindings_add(
                name,
                65,
                {
                    let note: *const ::core::ffi::c_char = std::ptr::null();
                    (!note.is_null()).then(|| std::ffi::CStr::from_ptr(note))
                },
                0,
                Some(cmd_list_new()),
            );
            assert!(std::rc::Weak::ptr_eq(
                &std::rc::Rc::downgrade(command.cmdlist()),
                &original_lifetime
            ));
            assert_eq!(command.key, 65);
            assert_eq!(command.flags, KEY_BINDING_REPEAT);
            assert_eq!(original_lifetime.strong_count(), 1);
            drop(table_owner);
            key_bindings_remove_table(name);
            assert!(key_bindings_get_table(name, 0).is_none());
            assert!(table_lifetime.upgrade().is_some());
            drop(retained_table);
            assert!(table_lifetime.upgrade().is_none());
            assert!(original_lifetime.upgrade().is_some());
            drop(command);
            assert!(original_lifetime.upgrade().is_none());
        }
    }
}
