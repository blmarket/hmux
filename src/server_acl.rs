use crate::src::cmd::queue::cmdq_print;
use crate::src::ffi::libc::{getgrgid, getpwuid, getuid};
use crate::src::format::bytes::write_cstr;
use crate::src::proc::{proc_get_peer_gid, proc_get_peer_uid};
use crate::src::server::clients;
use crate::src::server_client::Client as _;
use crate::src::shared::abi::*;
use crate::src::shared::abi::{__gid_t, __uid_t, gid_t, id_t, uid_t};
use crate::src::shared::account::{group, passwd};
use crate::src::shared::client::client;
use crate::src::shared::client::ClientRef;
use crate::src::shared::client::{CLIENT_EXIT, CLIENT_READONLY};
use crate::src::shared::command::cmdq_item;
use crate::src::shared::server_acl::SERVER_ACL_IS_GROUP;
use std::ffi::CString;

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
pub static mut server_acl_entries: server_acl_entries = server_acl_entries {
    entries: std::collections::BTreeMap::new(),
};

fn server_acl_entry_find(
    head: &server_acl_entries,
    id: id_t,
    flags: ::core::ffi::c_int,
) -> *mut server_acl_entry {
    head.entries
        .get(&server_acl_key(id, flags))
        .map(|entry| entry.as_ref() as *const server_acl_entry as *mut server_acl_entry)
        .unwrap_or(::core::ptr::null_mut::<server_acl_entry>())
}

fn server_acl_entries_minmax(head: &server_acl_entries) -> *mut server_acl_entry {
    let entry = head.entries.iter().next();
    entry
        .map(|(_, entry)| entry.as_ref() as *const server_acl_entry as *mut server_acl_entry)
        .unwrap_or(::core::ptr::null_mut::<server_acl_entry>())
}

fn server_acl_entries_next(
    head: &server_acl_entries,
    entry: &server_acl_entry,
) -> *mut server_acl_entry {
    let key = server_acl_entry_key(entry);
    head.entries
        .range((std::ops::Bound::Excluded(key), std::ops::Bound::Unbounded))
        .next()
        .map(|(_, entry)| entry.as_ref() as *const server_acl_entry as *mut server_acl_entry)
        .unwrap_or(::core::ptr::null_mut::<server_acl_entry>())
}

fn server_acl_entries_insert(
    head: &mut server_acl_entries,
    entry: Box<server_acl_entry>,
) -> *mut server_acl_entry {
    match head.entries.entry(server_acl_key(entry.id, entry.flags)) {
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

unsafe fn server_acl_check(c: &ClientRef) -> *mut server_acl_entry {
    let mut entry: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
    let mut uid: uid_t = 0;
    let mut gid: gid_t = 0;
    uid = c.peer_uid();
    if uid == -(1 as ::core::ffi::c_int) as uid_t {
        return ::core::ptr::null_mut::<server_acl_entry>();
    }
    entry = server_acl_entry_find(&server_acl_entries, uid as id_t, 0);
    if !entry.is_null() {
        return entry;
    }
    gid = c.peer_gid();
    if gid == -(1 as ::core::ffi::c_int) as gid_t {
        return ::core::ptr::null_mut::<server_acl_entry>();
    }
    server_acl_entry_find(&server_acl_entries, gid as id_t, SERVER_ACL_IS_GROUP)
}
unsafe fn server_acl_update() {
    let mut entry: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
    let mut c: Option<ClientRef> = None;
    let mut registry_c_owner = clients.first();
    c = registry_c_owner.clone();
    while !c.is_none() {
        entry = server_acl_check(c.as_ref().expect("live client"));
        if entry.is_null() {
            c.as_ref()
                .expect("live client")
                .exit_with_message(c"access not allowed".to_owned(), None);
        } else if (*entry).flags & SERVER_ACL_READONLY != 0 {
            c.as_ref()
                .expect("live client")
                .update_flags(CLIENT_READONLY as uint64_t, 0);
        } else {
            c.as_ref()
                .expect("live client")
                .update_flags(0, !(!CLIENT_READONLY as uint64_t));
        }
        registry_c_owner =
            clients.next(registry_c_owner.as_ref().expect("current registry client"));
        c = registry_c_owner.clone();
    }
}
pub unsafe fn server_acl_init() {
    server_acl_entries_clear(&mut server_acl_entries);
    if getuid() != 0 as __uid_t {
        server_acl_allow(0 as id_t, 0 as ::core::ffi::c_int);
    }
    server_acl_allow(getuid() as id_t, 0 as ::core::ffi::c_int);
}
pub unsafe fn server_acl_find(mut id: id_t, mut flags: ::core::ffi::c_int) -> ::core::ffi::c_int {
    (server_acl_entry_find(&server_acl_entries, id, flags) != NULL as *mut server_acl_entry)
        as ::core::ffi::c_int
}
pub unsafe fn server_acl_display(item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>) {
    let mut loop_0: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
    let mut pw: *mut passwd = ::core::ptr::null_mut::<passwd>();
    let mut gr: *mut group = ::core::ptr::null_mut::<group>();
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut type_0: ::core::ffi::c_char = 0;
    let mut current_block_12: u64;
    loop_0 = server_acl_entries_minmax(&server_acl_entries);
    while !loop_0.is_null() {
        if !(*loop_0).flags & SERVER_ACL_IS_GROUP != 0 {
            if (*loop_0).id == 0 as id_t {
                current_block_12 = 14916268686031723178;
            } else {
                pw = getpwuid((*loop_0).id as __uid_t);
                if !pw.is_null() {
                    name = (*pw).pw_name;
                } else {
                    name = c"unknown".as_ptr();
                }
                type_0 = 'U' as i32 as ::core::ffi::c_char;
                current_block_12 = 11050875288958768710;
            }
        } else {
            gr = getgrgid((*loop_0).id as __gid_t);
            if !gr.is_null() {
                name = (*gr).gr_name;
            } else {
                name = c"unknown".as_ptr();
            }
            type_0 = 'G' as i32 as ::core::ffi::c_char;
            current_block_12 = 11050875288958768710;
        }
        if current_block_12 == 11050875288958768710 {
            if (*loop_0).flags & SERVER_ACL_READONLY != 0 {
                cmdq_print(item_handle, |out| {
                    write_cstr(out, name)?;
                    out.write_all(b" (")?;
                    out.write_all(&[(type_0 as ::core::ffi::c_int) as u8])?;
                    out.write_all(b",R)")
                });
            } else {
                cmdq_print(item_handle, |out| {
                    write_cstr(out, name)?;
                    out.write_all(b" (")?;
                    out.write_all(&[(type_0 as ::core::ffi::c_int) as u8])?;
                    out.write_all(b",W)")
                });
            }
        }
        loop_0 = server_acl_entries_next(&server_acl_entries, &*loop_0);
    }
}
pub unsafe fn server_acl_allow(mut id: id_t, mut flags: ::core::ffi::c_int) {
    let mut entry: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
    entry = server_acl_entry_find(&server_acl_entries, id, flags);
    if entry.is_null() {
        server_acl_entries_insert(
            &mut server_acl_entries,
            Box::new(server_acl_entry {
                id,
                flags: flags & SERVER_ACL_IS_GROUP,
            }),
        );
    }
}
pub unsafe fn server_acl_deny(mut id: id_t, mut flags: ::core::ffi::c_int) {
    let mut entry: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
    entry = server_acl_entry_find(&server_acl_entries, id, flags);
    if !entry.is_null() {
        server_acl_entries_remove(&raw mut server_acl_entries, entry);
        server_acl_update();
    }
}
pub unsafe fn server_acl_allow_write(mut id: id_t, mut flags: ::core::ffi::c_int) {
    let mut entry: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
    entry = server_acl_entry_find(&server_acl_entries, id, flags);
    if entry.is_null() {
        return;
    }
    (*entry).flags &= !SERVER_ACL_READONLY;
    server_acl_update();
}
pub unsafe fn server_acl_deny_write(mut id: id_t, mut flags: ::core::ffi::c_int) {
    let mut entry: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
    entry = server_acl_entry_find(&server_acl_entries, id, flags);
    if entry.is_null() {
        return;
    }
    (*entry).flags |= SERVER_ACL_READONLY;
    server_acl_update();
}
pub unsafe fn server_acl_join(c: &ClientRef) -> ::core::ffi::c_int {
    let mut entry: *mut server_acl_entry = ::core::ptr::null_mut::<server_acl_entry>();
    entry = server_acl_check(c);
    if entry.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if (*entry).flags & SERVER_ACL_READONLY != 0 {
        c.update_flags(CLIENT_READONLY as u64, 0);
    }
    1 as ::core::ffi::c_int
}
