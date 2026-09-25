use crate::src::cmd::queue::cmdq_print;
use crate::src::ffi::libc::{getgrgid, getpwuid, getuid};
use crate::src::proc::{proc_get_peer_gid, proc_get_peer_uid};
use crate::src::server::clients;
use crate::src::server_client::server_client_set_exit_message;
use crate::src::shared::abi::*;
use crate::src::shared::abi::{__gid_t, __id_t, __uid_t, gid_t, id_t, uid_t};
use crate::src::shared::account::{group, passwd};
use crate::src::shared::arguments::args;
use crate::src::shared::client::*;
use crate::src::shared::client::{
    client, client_file, client_file_cb, client_file_entry, client_files, overlay_check_cb,
    overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb, overlay_resize_cb,
};
use crate::src::shared::client::{CLIENT_EXIT, CLIENT_READONLY};
use crate::src::shared::colour::*;
use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds};
use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
use crate::src::shared::display::{visible_range, visible_ranges};
use crate::src::shared::environment::environ;
use crate::src::shared::event::*;
use crate::src::shared::format::{format_job_tree, format_tree};
use crate::src::shared::grid::*;
use crate::src::shared::hyperlinks::hyperlinks;
use crate::src::shared::input::{input_ctx, input_request, input_requests};
use crate::src::shared::key::*;
use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
use crate::src::shared::layout::layout_geometry;
use crate::src::shared::layout::*;
use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
use crate::src::shared::menu::menu_data;
use crate::src::shared::message::*;
use crate::src::shared::mouse::mouse_event;
use crate::src::shared::options::options;
use crate::src::shared::pane::{
    window_pane, window_pane_modes, window_pane_prompt, window_pane_tree_entry, window_panes,
};
use crate::src::shared::pane::{window_pane_offset, window_pane_resize, window_pane_resizes};
use crate::src::shared::process::tmuxpeer;
use crate::src::shared::prompt::prompt;
use crate::src::shared::redraw::redraw_scene;
use crate::src::shared::screen::{screen, screen_sel, screen_titles};
use crate::src::shared::screen_write::screen_write_cline;
use crate::src::shared::server_acl::SERVER_ACL_IS_GROUP;
use crate::src::shared::session::{session, session_entry};
use crate::src::shared::spawn::spawn_editor_state;
use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
use crate::src::shared::tree::{RB_BLACK, RB_NEGINF, RB_RED};
use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
use crate::src::shared::window::{
    window, window_entry, window_mode, window_mode_entry, window_winlinks, winlink, winlink_entry,
    winlink_stack, winlinks,
};
use std::ffi::CString;

use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

#[repr(C)]
pub struct server_acl_entry {
    pub id: id_t,
    pub flags: ::core::ffi::c_int,
}
#[derive(Default)]
pub struct server_acl_entries {
    // The old comparator ordered non-groups before groups, then by id.
    entries: std::collections::BTreeMap<(bool, id_t), Box<server_acl_entry>>,
}
pub const SERVER_ACL_READONLY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;

fn server_acl_key(id: id_t, flags: ::core::ffi::c_int) -> (bool, id_t) {
    (flags & SERVER_ACL_IS_GROUP != 0, id)
}

fn server_acl_entry_key(entry: &server_acl_entry) -> (bool, id_t) {
    server_acl_key(entry.id, entry.flags)
}

#[no_mangle]
pub static mut server_acl_entries: server_acl_entries = server_acl_entries {
    entries: std::collections::BTreeMap::new(),
};

unsafe fn server_acl_entry_find(
    head: *mut server_acl_entries,
    id: id_t,
    flags: ::core::ffi::c_int,
) -> *mut server_acl_entry {
    (*head)
        .entries
        .get(&server_acl_key(id, flags))
        .map(|entry| entry.as_ref() as *const server_acl_entry as *mut server_acl_entry)
        .unwrap_or(::core::ptr::null_mut::<server_acl_entry>())
}

unsafe fn server_acl_entries_minmax(
    head: *mut server_acl_entries,
    val: ::core::ffi::c_int,
) -> *mut server_acl_entry {
    let entry = if val < 0 {
        (*head).entries.iter().next()
    } else {
        (*head).entries.iter().next_back()
    };
    entry
        .map(|(_, entry)| entry.as_ref() as *const server_acl_entry as *mut server_acl_entry)
        .unwrap_or(::core::ptr::null_mut::<server_acl_entry>())
}

unsafe fn server_acl_entries_next(
    head: *mut server_acl_entries,
    entry: *mut server_acl_entry,
) -> *mut server_acl_entry {
    let key = server_acl_entry_key(&*entry);
    (*head)
        .entries
        .range((std::ops::Bound::Excluded(key), std::ops::Bound::Unbounded))
        .next()
        .map(|(_, entry)| entry.as_ref() as *const server_acl_entry as *mut server_acl_entry)
        .unwrap_or(::core::ptr::null_mut::<server_acl_entry>())
}

unsafe fn server_acl_entries_insert(
    head: *mut server_acl_entries,
    entry: Box<server_acl_entry>,
) -> *mut server_acl_entry {
    match (*head).entries.entry(server_acl_key(entry.id, entry.flags)) {
        std::collections::btree_map::Entry::Occupied(existing) => {
            existing.get().as_ref() as *const server_acl_entry as *mut server_acl_entry
        }
        std::collections::btree_map::Entry::Vacant(existing) => {
            existing.insert(entry);
            ::core::ptr::null_mut::<server_acl_entry>()
        }
    }
}

unsafe fn server_acl_entries_remove(
    head: *mut server_acl_entries,
    entry: *mut server_acl_entry,
) -> Option<Box<server_acl_entry>> {
    (*head).entries.remove(&server_acl_entry_key(&*entry))
}

fn server_acl_entries_clear(head: &mut server_acl_entries) {
    head.entries.clear();
}

unsafe extern "C" fn server_acl_check(mut c: *mut client) -> *mut server_acl_entry {
    let mut entry: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
    let mut uid: uid_t = 0;
    let mut gid: gid_t = 0;
    uid = proc_get_peer_uid((*c).peer);
    if uid == -(1 as ::core::ffi::c_int) as uid_t {
        return ::core::ptr::null_mut::<server_acl_entry>();
    }
    entry = server_acl_entry_find(&raw mut server_acl_entries, uid as id_t, 0);
    if !entry.is_null() {
        return entry;
    }
    gid = proc_get_peer_gid((*c).peer);
    if gid == -(1 as ::core::ffi::c_int) as gid_t {
        return ::core::ptr::null_mut::<server_acl_entry>();
    }
    return server_acl_entry_find(
        &raw mut server_acl_entries,
        gid as id_t,
        SERVER_ACL_IS_GROUP,
    );
}
unsafe extern "C" fn server_acl_update() {
    let mut entry: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    c = clients.first();
    while !c.is_null() {
        entry = server_acl_check(c);
        if entry.is_null() {
            server_client_set_exit_message(&mut *c, Some(CString::new("access not allowed").unwrap()));
            (*c).flags |= CLIENT_EXIT as uint64_t;
        } else if (*entry).flags & SERVER_ACL_READONLY != 0 {
            (*c).flags |= CLIENT_READONLY as uint64_t;
        } else {
            (*c).flags &= !CLIENT_READONLY as uint64_t;
        }
        c = clients.next(c);
    }
}
#[no_mangle]
pub unsafe extern "C" fn server_acl_init() {
    server_acl_entries_clear(&mut *(&raw mut server_acl_entries));
    if getuid() != 0 as __uid_t {
        server_acl_allow(0 as id_t, 0 as ::core::ffi::c_int);
    }
    server_acl_allow(getuid() as id_t, 0 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn server_acl_find(
    mut id: id_t,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return (server_acl_entry_find(&raw mut server_acl_entries, id, flags)
        != NULL as *mut server_acl_entry) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn server_acl_display(mut item: *mut cmdq_item) {
    let mut loop_0: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
    let mut pw: *mut passwd = ::core::ptr::null_mut::<passwd>();
    let mut gr: *mut group = ::core::ptr::null_mut::<group>();
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut type_0: ::core::ffi::c_char = 0;
    let mut current_block_12: u64;
    loop_0 = server_acl_entries_minmax(&raw mut server_acl_entries, -1);
    while !loop_0.is_null() {
        if !(*loop_0).flags & SERVER_ACL_IS_GROUP != 0 {
            if (*loop_0).id == 0 as id_t {
                current_block_12 = 14916268686031723178;
            } else {
                pw = getpwuid((*loop_0).id as __uid_t);
                if !pw.is_null() {
                    name = (*pw).pw_name;
                } else {
                    name = b"unknown\0" as *const u8 as *const ::core::ffi::c_char;
                }
                type_0 = 'U' as i32 as ::core::ffi::c_char;
                current_block_12 = 11050875288958768710;
            }
        } else {
            gr = getgrgid((*loop_0).id as __gid_t);
            if !gr.is_null() {
                name = (*gr).gr_name;
            } else {
                name = b"unknown\0" as *const u8 as *const ::core::ffi::c_char;
            }
            type_0 = 'G' as i32 as ::core::ffi::c_char;
            current_block_12 = 11050875288958768710;
        }
        match current_block_12 {
            11050875288958768710 => {
                if (*loop_0).flags & SERVER_ACL_READONLY != 0 {
                    cmdq_print(
                        item,
                        b"%s (%c,R)\0" as *const u8 as *const ::core::ffi::c_char,
                        name,
                        type_0 as ::core::ffi::c_int,
                    );
                } else {
                    cmdq_print(
                        item,
                        b"%s (%c,W)\0" as *const u8 as *const ::core::ffi::c_char,
                        name,
                        type_0 as ::core::ffi::c_int,
                    );
                }
            }
            _ => {}
        }
        loop_0 = server_acl_entries_next(&raw mut server_acl_entries, loop_0);
    }
}
#[no_mangle]
pub unsafe extern "C" fn server_acl_allow(mut id: id_t, mut flags: ::core::ffi::c_int) {
    let mut entry: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
    entry = server_acl_entry_find(&raw mut server_acl_entries, id, flags);
    if entry.is_null() {
        server_acl_entries_insert(
            &raw mut server_acl_entries,
            Box::new(server_acl_entry {
                id,
                flags: flags & SERVER_ACL_IS_GROUP,
            }),
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn server_acl_deny(mut id: id_t, mut flags: ::core::ffi::c_int) {
    let mut entry: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
    entry = server_acl_entry_find(&raw mut server_acl_entries, id, flags);
    if !entry.is_null() {
        server_acl_entries_remove(&raw mut server_acl_entries, entry);
        server_acl_update();
    }
}
#[no_mangle]
pub unsafe extern "C" fn server_acl_allow_write(mut id: id_t, mut flags: ::core::ffi::c_int) {
    let mut entry: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
    entry = server_acl_entry_find(&raw mut server_acl_entries, id, flags);
    if entry.is_null() {
        return;
    }
    (*entry).flags &= !SERVER_ACL_READONLY;
    server_acl_update();
}
#[no_mangle]
pub unsafe extern "C" fn server_acl_deny_write(mut id: id_t, mut flags: ::core::ffi::c_int) {
    let mut entry: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
    entry = server_acl_entry_find(&raw mut server_acl_entries, id, flags);
    if entry.is_null() {
        return;
    }
    (*entry).flags |= SERVER_ACL_READONLY;
    server_acl_update();
}
#[no_mangle]
pub unsafe extern "C" fn server_acl_join(mut c: *mut client) -> ::core::ffi::c_int {
    let mut entry: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
    entry = server_acl_check(c);
    if entry.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if (*entry).flags & SERVER_ACL_READONLY != 0 {
        (*c).flags |= CLIENT_READONLY as uint64_t;
    }
    return 1 as ::core::ffi::c_int;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_acl_tree_matches_red_black_tree_semantics() {
        unsafe {
            let mut tree = server_acl_entries::default();
            let first = Box::new(server_acl_entry { id: 9, flags: 0 });
            let second = Box::new(server_acl_entry { id: 4, flags: 0 });
            let group = Box::new(server_acl_entry {
                id: 1,
                flags: SERVER_ACL_IS_GROUP,
            });
            let duplicate = Box::new(server_acl_entry {
                id: 1,
                flags: SERVER_ACL_IS_GROUP | SERVER_ACL_READONLY,
            });
            let first_ptr = first.as_ref() as *const server_acl_entry as *mut server_acl_entry;
            let second_ptr = second.as_ref() as *const server_acl_entry as *mut server_acl_entry;
            let group_ptr = group.as_ref() as *const server_acl_entry as *mut server_acl_entry;

            assert!(server_acl_entries_insert(&raw mut tree, first).is_null());
            assert!(server_acl_entries_insert(&raw mut tree, second).is_null());
            assert!(server_acl_entries_insert(&raw mut tree, group).is_null());
            assert_eq!(
                server_acl_entries_insert(&raw mut tree, duplicate),
                group_ptr,
                "duplicate keys keep the original item"
            );

            assert_eq!(server_acl_entries_minmax(&raw mut tree, -1), second_ptr);
            assert_eq!(
                server_acl_entries_next(&raw mut tree, second_ptr),
                first_ptr
            );
            assert_eq!(server_acl_entries_next(&raw mut tree, first_ptr), group_ptr);
            assert!(server_acl_entries_next(&raw mut tree, group_ptr).is_null());
            assert_eq!(server_acl_entries_minmax(&raw mut tree, 1), group_ptr);
            assert_eq!(
                server_acl_entry_find(&raw mut tree, 1, SERVER_ACL_IS_GROUP | SERVER_ACL_READONLY),
                group_ptr,
            );
            let removed = server_acl_entries_remove(&raw mut tree, first_ptr).unwrap();
            assert_eq!(
                removed.as_ref() as *const server_acl_entry as *mut server_acl_entry,
                first_ptr
            );
            assert!(server_acl_entry_find(&raw mut tree, 9, 0).is_null());
            server_acl_entries_clear(&mut tree);
            assert!(server_acl_entries_minmax(&raw mut tree, -1).is_null());
        }
    }
}
