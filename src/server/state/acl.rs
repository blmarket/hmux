//! The server access-control list: tmux's `server-acl.c`.
//!
//! One entry per uid allowed to reach this server, each either read-write or
//! read-only. The table is seeded with the server owner — and, unless the
//! server itself runs as root, with uid 0 — edited by `server-access`, and
//! consulted once per connection by the accept path.
//!
//! An entry's access is a property of the *user*, so changing it sweeps the
//! clients that user already has connected: a uid demoted to read-only takes
//! its live clients with it, and one promoted back to write releases them.

use crate::server::format::username;

use super::ConnectionHandle;

use super::ServerState;

/// What one ACL entry grants — tmux's `SERVER_ACL_READONLY` flag.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AclAccess {
    /// The user's clients join able to run any command.
    Write,
    /// The user's clients join with `CLIENT_READONLY` set.
    ReadOnly,
}

/// What the ACL says about a connecting peer — tmux's `server_acl_join`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AclJoin {
    /// The peer's uid is unlisted, or the platform reported none. tmux treats
    /// both as a refusal.
    Denied,
    /// The peer may join, read-only when its entry says so.
    Allowed { read_only: bool },
}

/// The message a refused or evicted client is given as its exit reason —
/// tmux's `server.c` and `cmd_server_access_deny` both use this string.
pub(crate) const ACCESS_DENIED: &str = "access not allowed";

impl ServerState {
    /// Seed the table the way `server_acl_init` does: the server's owner, plus
    /// root unless the server is already running as root.
    pub(super) fn seed_server_acl(&mut self) {
        if self.owner_uid != 0 {
            self.acl.insert(0, AclAccess::Write);
        }
        self.acl.insert(self.owner_uid, AclAccess::Write);
    }

    /// The uid that started this server. `server-access` refuses to change it,
    /// and it can never be removed from the table.
    pub(crate) fn server_owner_uid(&self) -> u32 {
        self.owner_uid
    }

    /// This uid's entry, or `None` when it has none — `server_acl_user_find`.
    pub(crate) fn acl_entry(&self, uid: u32) -> Option<AclAccess> {
        self.acl.get(&uid).copied()
    }

    /// Whether a peer with this uid may be served, and under which access.
    pub(crate) fn acl_join(&self, uid: Option<u32>) -> AclJoin {
        match uid.and_then(|uid| self.acl_entry(uid)) {
            None => AclJoin::Denied,
            Some(access) => AclJoin::Allowed {
                read_only: access == AclAccess::ReadOnly,
            },
        }
    }

    /// `server_acl_user_allow`: add the uid with write access, leaving an
    /// entry that already exists alone.
    pub(crate) fn acl_allow(&mut self, uid: u32) {
        self.acl.entry(uid).or_insert(AclAccess::Write);
    }

    /// `server_acl_user_deny`: drop the uid's entry, if it has one.
    pub(crate) fn acl_deny(&mut self, uid: u32) {
        self.acl.remove(&uid);
    }

    /// `server_acl_user_allow_write` / `server_acl_user_deny_write`: change an
    /// existing entry's access and carry the change onto the clients that uid
    /// already has connected. A uid with no entry is left alone, as in tmux.
    pub(crate) fn acl_set_access(&mut self, uid: u32, access: AclAccess) {
        let Some(entry) = self.acl.get_mut(&uid) else {
            return;
        };
        *entry = access;
        self.client_renders
            .set_read_only_for_uid(uid, access == AclAccess::ReadOnly);
    }

    /// End everything `uid` has connected, giving each the reason as its exit
    /// message — the sweep `server-access -d` runs over `clients` before
    /// removing the entry.
    ///
    /// Both halves of that population are swept: the registered clients, which
    /// are reached through the render slot they already take server-initiated
    /// actions from, and the bare connections — a command client, a peer still
    /// identifying — which are reached through the connection registry.
    pub(crate) fn acl_evict(&mut self, uid: u32, message: &str) {
        self.client_renders.evict_clients_with_uid(uid, message);
        self.connections.evict_uid(uid, message);
    }

    /// Register one accepted connection so an access change can reach it. The
    /// handle deregisters on drop, so it belongs to the task serving the
    /// connection.
    pub(crate) fn register_connection(&self, peer_uid: Option<u32>) -> ConnectionHandle {
        self.connections.register(peer_uid)
    }

    /// `server_acl_display`: one line per entry in uid order, skipping root,
    /// naming each uid the password database can resolve and calling the rest
    /// `unknown`.
    pub(crate) fn acl_display(&self) -> String {
        let mut out = String::new();
        for (uid, access) in &self.acl {
            if *uid == 0 {
                continue;
            }
            let name = username(*uid).unwrap_or_else(|| "unknown".to_string());
            let access = match access {
                AclAccess::ReadOnly => 'R',
                AclAccess::Write => 'W',
            };
            out.push_str(&format!("{name} ({access})\n"));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_owned_by(uid: u32) -> ServerState {
        let mut state = ServerState::empty();
        state.acl.clear();
        state.owner_uid = uid;
        state.seed_server_acl();
        state
    }

    #[test]
    fn seeding_adds_the_owner_and_root() {
        let state = state_owned_by(1000);
        assert_eq!(state.acl_entry(1000), Some(AclAccess::Write));
        assert_eq!(state.acl_entry(0), Some(AclAccess::Write));
        assert_eq!(state.acl_entry(1001), None);
    }

    #[test]
    fn a_root_owned_server_lists_root_once() {
        let state = state_owned_by(0);
        assert_eq!(state.acl_entry(0), Some(AclAccess::Write));
        assert_eq!(state.acl.len(), 1);
    }

    #[test]
    fn an_unlisted_or_unknown_peer_is_denied() {
        let state = state_owned_by(1000);
        assert_eq!(state.acl_join(Some(1001)), AclJoin::Denied);
        assert_eq!(state.acl_join(None), AclJoin::Denied);
    }

    #[test]
    fn a_listed_peer_joins_under_its_entrys_access() {
        let mut state = state_owned_by(1000);
        state.acl_allow(1001);
        assert_eq!(
            state.acl_join(Some(1001)),
            AclJoin::Allowed { read_only: false }
        );
        state.acl_set_access(1001, AclAccess::ReadOnly);
        assert_eq!(
            state.acl_join(Some(1001)),
            AclJoin::Allowed { read_only: true }
        );
        state.acl_set_access(1001, AclAccess::Write);
        assert_eq!(
            state.acl_join(Some(1001)),
            AclJoin::Allowed { read_only: false }
        );
    }

    #[test]
    fn allowing_an_existing_entry_keeps_its_access() {
        let mut state = state_owned_by(1000);
        state.acl_allow(1001);
        state.acl_set_access(1001, AclAccess::ReadOnly);
        state.acl_allow(1001);
        assert_eq!(state.acl_entry(1001), Some(AclAccess::ReadOnly));
    }

    #[test]
    fn denying_removes_the_entry_and_setting_access_on_a_missing_one_is_a_no_op() {
        let mut state = state_owned_by(1000);
        state.acl_allow(1001);
        state.acl_deny(1001);
        assert_eq!(state.acl_entry(1001), None);
        state.acl_set_access(1001, AclAccess::ReadOnly);
        assert_eq!(state.acl_entry(1001), None);
    }

    /// A client already connected under a uid follows that uid's entry when
    /// its access changes — tmux's sweep in `server_acl_user_allow_write` and
    /// `_deny_write`.
    #[test]
    fn changing_a_users_access_sweeps_the_clients_it_already_has() {
        let mut state = ServerState::with_test_session().expect("test session");
        state.acl.clear();
        state.owner_uid = 1000;
        state.seed_server_acl();
        let client = state
            .attach_test_client("0", 80, 24)
            .expect("attach client");
        client.set_peer_identity(Some(1001), "other".to_string());
        state.acl_allow(1001);

        assert!(!client.client_flags_view().1);
        state.acl_set_access(1001, AclAccess::ReadOnly);
        assert!(client.client_flags_view().1);
        state.acl_set_access(1001, AclAccess::Write);
        assert!(!client.client_flags_view().1);
    }

    /// Another uid's clients are left alone by the sweep.
    #[test]
    fn the_sweep_only_touches_the_uid_it_names() {
        let mut state = ServerState::with_test_session().expect("test session");
        state.acl.clear();
        state.owner_uid = 1000;
        state.seed_server_acl();
        let client = state
            .attach_test_client("0", 80, 24)
            .expect("attach client");
        client.set_peer_identity(Some(1001), "other".to_string());
        state.acl_allow(1002);
        state.acl_set_access(1002, AclAccess::ReadOnly);
        assert!(!client.client_flags_view().1);
    }

    /// `server-access -d` ends that uid's clients with the reason before the
    /// entry goes.
    #[test]
    fn evicting_a_user_ends_the_clients_it_has_connected() {
        let mut state = ServerState::with_test_session().expect("test session");
        state.acl.clear();
        state.owner_uid = 1000;
        state.seed_server_acl();
        let client = state
            .attach_test_client("0", 80, 24)
            .expect("attach client");
        client.set_peer_identity(Some(1001), "other".to_string());
        state.acl_allow(1001);
        state.acl_evict(1001, ACCESS_DENIED);

        assert!(matches!(
            client.take_action(),
            Some(super::super::ClientAction::Evict { message }) if message == ACCESS_DENIED
        ));
    }

    #[test]
    fn the_listing_skips_root_and_marks_access() {
        let mut state = state_owned_by(1000);
        state.acl_allow(4_000_000_001);
        state.acl_set_access(4_000_000_001, AclAccess::ReadOnly);
        let listing = state.acl_display();
        assert!(!listing.contains("root"));
        assert!(listing.contains("unknown (R)\n"));
    }
}
