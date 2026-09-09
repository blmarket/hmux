use crate::cmd::cmdq_item;
use super::run::client_walk;

use crate::ffi::getuid;
use crate::fmt_args;

pub use crate::types::*;
use crate::{UserAccount, UserAccountRecord};
use std::collections::BTreeMap;
use std::sync::Mutex;
/// Access granted to one user by a server ACL.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServerAclAccess {
    /// The user may attach and change server state.
    ReadWrite,
    /// The user may attach but may not change server state.
    ReadOnly,
}

/// An owned observation of one allowed user.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServerAclEntry {
    /// The allowed user's numeric id.
    pub uid: uid_t,
    /// The access currently granted to the user.
    pub access: ServerAclAccess,
}

/// An ordered store of users allowed to access a tmux server.
pub trait ServerAclStore {
    /// Walks allowed users in ascending numeric user-id order.
    fn entries(&self) -> impl Iterator<Item = ServerAclEntry>;

    /// Finds the entry for `uid`.
    fn find(&self, uid: uid_t) -> Option<ServerAclEntry>;

    /// Adds a missing user with read-write access, leaving an existing entry unchanged.
    fn allow(&mut self, uid: uid_t);

    /// Removes a user if present.
    fn deny(&mut self, uid: uid_t);

    /// Changes an existing user's access, doing nothing if the user is absent.
    fn set_access(&mut self, uid: uid_t, access: ServerAclAccess);
}

/// The server ACL implementation used by hmux.
pub struct RustServerAclStore {
    entries: BTreeMap<uid_t, ServerAclAccess>,
}

impl RustServerAclStore {
    /// Makes an independent empty ACL.
    pub fn new() -> Self {
        Self::empty()
    }

    const fn empty() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }
}

impl Default for RustServerAclStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerAclStore for RustServerAclStore {
    fn entries(&self) -> impl Iterator<Item = ServerAclEntry> {
        self.entries
            .iter()
            .map(|(&uid, &access)| ServerAclEntry { uid, access })
    }

    fn find(&self, uid: uid_t) -> Option<ServerAclEntry> {
        self.entries
            .get(&uid)
            .copied()
            .map(|access| ServerAclEntry { uid, access })
    }

    fn allow(&mut self, uid: uid_t) {
        self.entries
            .entry(uid)
            .or_insert(ServerAclAccess::ReadWrite);
    }

    fn deny(&mut self, uid: uid_t) {
        self.entries.remove(&uid);
    }

    fn set_access(&mut self, uid: uid_t, access: ServerAclAccess) {
        if let Some(current) = self.entries.get_mut(&uid) {
            *current = access;
        }
    }
}

pub use crate::consts::CLIENT_READONLY;
static SERVER_ACL: Mutex<RustServerAclStore> = Mutex::new(RustServerAclStore::empty());

pub(crate) fn with_server_acl<R>(read: impl FnOnce(&RustServerAclStore) -> R) -> R {
    let acl = SERVER_ACL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    read(&acl)
}

pub(crate) fn with_server_acl_mut<R>(mutate: impl FnOnce(&mut RustServerAclStore) -> R) -> R {
    let mut acl = SERVER_ACL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    mutate(&mut acl)
}

pub(crate) fn server_acl_init() {
    let owner = unsafe { getuid() };
    with_server_acl_mut(|acl| {
        *acl = RustServerAclStore::new();
        if owner != 0 {
            acl.allow(0);
        }
        acl.allow(owner);
    });
}

pub(crate) unsafe fn server_acl_display(item: &cmdq_item) {
    unsafe {
        let users = with_server_acl(|acl| acl.entries().collect::<Vec<_>>());
        for loop_0 in users {
            if !(loop_0.uid == 0 as uid_t) {
                let account = UserAccountRecord::lookup_uid(loop_0.uid);
                let name = account
                    .as_ref()
                    .and_then(UserAccount::account_name)
                    .unwrap_or(c"unknown");
                if loop_0.access == ServerAclAccess::ReadOnly {
                    item.print(c"%s (R)", fmt_args![name]);
                } else {
                    item.print(c"%s (W)", fmt_args![name]);
                }
            }
        }
    }
}
/// Applies one user's current access to clients that have the same peer UID.
pub(crate) fn server_acl_update_clients(uid: uid_t, access: ServerAclAccess) {
    for mut owner in client_walk() {
        let peer = { owner.peer_handle() }.uid();
        if peer == -(1 as core::ffi::c_int) as uid_t || peer != uid {
            continue;
        }
        let c = unsafe { owner.as_client_mut() };
        if access == ServerAclAccess::ReadOnly {
            c.flags |= CLIENT_READONLY as uint64_t;
        } else {
            c.flags &= !CLIENT_READONLY as uint64_t;
        }
    }
}
pub(crate) unsafe fn server_acl_join(c: &mut client) -> core::ffi::c_int {
    let uid = (c.peer_handle()).uid();
    if uid == -(1 as core::ffi::c_int) as uid_t {
        return 0 as core::ffi::c_int;
    }
    let Some(user) = with_server_acl(|acl| acl.find(uid)) else {
        return 0 as core::ffi::c_int;
    };
    if user.access == ServerAclAccess::ReadOnly {
        c.flags |= CLIENT_READONLY as uint64_t;
    }
    1 as core::ffi::c_int
}
