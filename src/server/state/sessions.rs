//! Session lifecycle: creating, renaming, grouping, and destroying sessions,
//! and the notifications a control client is told about them.
//!
//! Session groups (`new-session -t`) share a window list, so most operations
//! here fan out across the group rather than touching one session.

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::PathBuf;

use super::sizing::{clamp_window_size, parse_size_pair, DEFAULT_XPIXEL, DEFAULT_YPIXEL};
use super::{
    fill_spec_spawn_ids, now_epoch, now_micros, pane_start_command, ExitEmpty, LayoutCell,
    Notification, Pane, PaneNode, PaneSpec, RenderInvalidation, ServerState, Session, Window,
    Winlink, ALERT_ACTIVITY,
};
use crate::server::options::{HookName, OptionSet, PaneHook, SessionHook, WindowHook};

impl ServerState {
    pub(crate) fn is_grouped(&self, session: &Session) -> bool {
        self.session_groups.contains_key(&session.link_set_id)
    }

    pub(crate) fn session_group_name(&self, session: &Session) -> Option<&str> {
        self.session_groups
            .get(&session.link_set_id)
            .map(String::as_str)
    }

    pub(crate) fn session_group_size(&self, session: &Session) -> usize {
        self.sessions
            .iter()
            .filter(|member| member.link_set_id == session.link_set_id)
            .count()
    }

    pub(crate) fn session_attached_client_names(&self, session: &Session) -> Vec<String> {
        self.client_renders
            .names_for_sessions(&BTreeSet::from([session.id]))
    }

    /// When this server state was created (`#{start_time}`).
    pub(crate) fn started_epoch(&self) -> i64 {
        self.started_epoch
    }

    pub(crate) fn session_group_attached_client_names(&self, session: &Session) -> Vec<String> {
        if !self.is_grouped(session) {
            return Vec::new();
        }
        let ids = self
            .sessions
            .iter()
            .filter_map(|member| (member.link_set_id == session.link_set_id).then_some(member.id))
            .collect();
        self.client_renders.names_for_sessions(&ids)
    }

    /// Synchronize every other member from `source`, preserving each member's
    /// current and previous window by index. This mirrors tmux's
    /// `session_group_synchronize_from` rather than sharing one collection.
    pub(super) fn synchronize_group_from(&mut self, source: usize) {
        if !self.is_grouped(&self.sessions[source]) {
            return;
        }
        let source_id = self.sessions[source].id;
        let link_set_id = self.sessions[source].link_set_id;
        let links = self.sessions[source].windows.clone();
        for member in self
            .sessions
            .iter_mut()
            .filter(|member| member.link_set_id == link_set_id && member.id != source_id)
        {
            Self::install_links_by_index(member, links.clone());
        }
    }

    pub(crate) fn session_id(&self, name: &str) -> Option<u32> {
        self.resolve_session(name).map(|session| session.id)
    }

    /// Record where a session was created, which is where the `#()` jobs of the
    /// clients attached to it run.
    pub(crate) fn set_session_cwd(&mut self, session_id: u32, cwd: Option<PathBuf>) {
        if let Some(session) = self
            .sessions
            .iter_mut()
            .find(|session| session.id == session_id)
        {
            session.cwd = cwd;
        }
    }

    pub(crate) fn session_by_id(&self, session_id: u32) -> Option<&Session> {
        self.sessions
            .iter()
            .find(|session| session.id == session_id)
    }

    /// The next unused numeric session name (tmux names sessions 0,1,2,… by
    /// default).
    pub fn next_session_name(&self) -> String {
        let mut n = 0u32;
        while self.sessions.iter().any(|s| s.name == n.to_string()) {
            n += 1;
        }
        n.to_string()
    }

    /// Create a session named `name` with a single window holding one pane.
    pub fn create_session(&mut self, name: &str, spec: PaneSpec) -> io::Result<u32> {
        if self.find_exact(name).is_some() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("duplicate session: {name}"),
            ));
        }
        // The session does not exist yet, so only the global `default-size` can
        // apply — tmux reads `global_s_options` here for the same reason.
        let (cols, rows) = self
            .global_options
            .session()
            .get("default-size")
            .and_then(parse_size_pair)
            .map_or((self.default_cols, self.default_rows), clamp_window_size);
        // tmux creates the session and the pane before spawning the pane's
        // process, so both ids are in its environment. hmux allocates them once
        // the spawn has succeeded, so their values are peeked here and consumed
        // unchanged below.
        let spec = fill_spec_spawn_ids(spec, self.next_pane_id, self.next_session_id);
        let start_command = pane_start_command(&spec);
        let pane = match spec {
            PaneSpec::Inert => Pane::inert(cols, rows)?,
            PaneSpec::Command(argv) => {
                let refs: Vec<&str> = argv.iter().map(String::as_str).collect();
                Pane::spawn(&refs, None, cols, rows)?
            }
            PaneSpec::CommandIn(argv, cwd) => {
                let refs: Vec<&str> = argv.iter().map(String::as_str).collect();
                Pane::spawn(&refs, Some(&cwd), cols, rows)?
            }
            // Only the popup conversion produces one of these, and it splits an
            // existing window rather than creating a session.
            PaneSpec::Existing(pane) => *pane,
        };

        let session_id = self.next_session_id;
        self.next_session_id += 1;
        let link_set_id = self.next_link_set_id;
        self.next_link_set_id += 1;
        let window_id = self.next_window_id;
        self.next_window_id += 1;
        let winlink_id = self.next_winlink_id;
        self.next_winlink_id += 1;
        let pane_id = self.next_pane_id;
        self.next_pane_id += 1;

        let base = self.base_index();
        self.windows.insert(
            window_id,
            Window {
                id: window_id,
                name: String::new(),
                panes: vec![PaneNode {
                    id: pane_id,
                    pane,
                    start_command,
                    input_off: false,
                    title: None,
                    exit_notified: false,
                    mode: None,
                    copy: None,
                    mode_view: None,
                    search_string: None,
                    search_regex: false,
                    floating: None,
                    scrollbar_columns: 0,
                    border_status: None,
                    unseen_changes: false,
                    active_point: 0,
                    options: OptionSet::default(),
                }],
                active: 0,
                last_pane: None,
                zoomed: false,
                activity_epoch: now_epoch(),
                name_time_micros: 0,
                name_in_mode: false,
                scrollbars_on_left: false,
                cols,
                rows,
                manual_size: (cols, rows),
                latest_client: None,
                pending_size: None,
                xpixel: DEFAULT_XPIXEL,
                ypixel: DEFAULT_YPIXEL,
                layout: LayoutCell::pane(pane_id, cols, rows),
                last_layout: None,
                old_layout: None,
                last_new_pane_x: 0,
                last_new_pane_y: 0,
                // tmux's `window_create` raises activity, so a monitor turned on
                // later still sees the window as having been active.
                pending_alerts: ALERT_ACTIVITY,
                options: OptionSet::default(),
            },
        );
        self.sessions.push(Session {
            id: session_id,
            name: name.to_string(),
            cols,
            rows,
            active: 0,
            last_windows: Vec::new(),
            created_epoch: now_epoch(),
            // tmux seeds activity from the creation time, so creation order is
            // the initial `detach-on-destroy` ordering.
            activity_micros: now_micros(),
            last_attached_micros: 0,
            locked_at_activity_micros: None,
            windows: vec![Winlink {
                link_id: winlink_id,
                index: base,
                id: window_id,
                alert_flags: 0,
            }],
            link_set_id,
            environment: BTreeMap::new(),
            removed_environment: BTreeSet::new(),
            hidden_environment: BTreeSet::new(),
            options: OptionSet::default(),
            cwd: None,
        });
        self.initial_attach_pending = false;
        self.notify_session(SessionHook::SessionCreated, session_id);
        Ok(session_id)
    }

    /// Create a session in a tmux session group. `target` may name an existing
    /// group, an existing session (creating a group named after it if needed),
    /// or a new one-member group. Group members share their link set but keep
    /// independent current and previous windows.
    pub fn create_grouped_session(
        &mut self,
        name: &str,
        target: &str,
        spec: PaneSpec,
    ) -> io::Result<u32> {
        if self.find_exact(name).is_some() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("duplicate session: {name}"),
            ));
        }
        let target_session = self.session_index(target);
        let link_set_id = if let Some(target_session) = target_session {
            let link_set_id = self.sessions[target_session].link_set_id;
            if !self.session_groups.contains_key(&link_set_id) {
                let group_name = self.sessions[target_session].name.clone();
                self.session_groups.insert(link_set_id, group_name);
            }
            Some(link_set_id)
        } else {
            self.session_groups
                .iter()
                .find_map(|(link_set_id, group_name)| {
                    (group_name == target).then_some(*link_set_id)
                })
        };

        let Some(link_set_id) = link_set_id else {
            let session_id = self.create_session(name, spec)?;
            let link_set_id = self
                .sessions
                .iter()
                .find(|session| session.id == session_id)
                .expect("new session is present")
                .link_set_id;
            self.session_groups.insert(link_set_id, target.to_string());
            return Ok(session_id);
        };

        let target = self
            .sessions
            .iter()
            .position(|session| session.link_set_id == link_set_id)
            .expect("a live session owns every live group");
        // tmux creates the new session's initial window before adding it to the
        // group, then discards that window while synchronizing. Going through
        // `create_session` preserves observable window/pane ID consumption and
        // spawn failures.
        let source_windows = self.sessions[target].windows.clone();
        let source_size = (self.sessions[target].cols, self.sessions[target].rows);
        let session_id = self.create_session(name, spec)?;
        let created = self
            .sessions
            .iter()
            .position(|session| session.id == session_id)
            .expect("new session is present");
        self.sessions[created].windows = source_windows;
        self.sessions[created].link_set_id = link_set_id;
        self.sessions[created].active = 0;
        self.sessions[created].last_windows.clear();
        self.sessions[created].cols = source_size.0;
        self.sessions[created].rows = source_size.1;
        self.remove_unlinked_windows();
        Ok(session_id)
    }

    /// Rename session `from` to `to`, mirroring tmux's `rename-session`. Errors
    /// if `from` is missing (`can't find session`) or `to` is already taken
    /// (`duplicate session`), matching tmux's diagnostics.
    pub fn rename_session(&mut self, from: &str, to: &str) -> io::Result<()> {
        let Some(session_id) = self.find(from).map(|session| session.id) else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("can't find session: {from}"),
            ));
        };
        if from != to && self.find_exact(to).is_some() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("duplicate session: {to}"),
            ));
        }
        let mut renamed = false;
        if let Some(s) = self.sessions.iter_mut().find(|s| s.id == session_id) {
            if s.name != to {
                s.name = to.to_string();
                renamed = true;
            }
        }
        if renamed {
            self.invalidate_session(session_id, RenderInvalidation::STATUS);
            self.notify_session(SessionHook::SessionRenamed, session_id);
        }
        Ok(())
    }

    /// Resolve just the session named by a target (its part before any `:`),
    /// accepting a name or a `$id`. Used by `has-session`.
    /// Seed the global environment from the daemon's own, as tmux fills
    /// `global_environ` from the environment its server was started with. The
    /// unit tests build a state without this, so they stay hermetic.
    pub fn seed_global_environment(&mut self) {
        self.environment_generation += 1;
        for (name, value) in std::env::vars() {
            self.seeded_environment.insert(name.clone());
            self.environment.insert(name, value);
        }
        if let Ok(cwd) = std::env::current_dir() {
            self.seeded_environment.insert("PWD".to_owned());
            self.environment
                .insert("PWD".to_owned(), cwd.to_string_lossy().into_owned());
        }
    }

    /// `kill-server`: tear down all sessions. A command client sees exit 0.
    pub fn kill_server(&mut self) {
        self.sessions.clear();
        self.windows.clear();
        self.session_groups.clear();
        self.shutdown_requested = true;
        self.invalidate_all_clients(RenderInvalidation::SESSION_GONE);
    }

    /// Remove a session by name. Returns whether it existed (its panes' children
    /// are killed as the panes drop).
    pub fn kill_session(&mut self, name: &str) -> bool {
        let had_sessions = !self.sessions.is_empty();
        let removed_id = self
            .sessions
            .iter()
            .find(|session| session.name == name)
            .map(|session| session.id);
        if let Some(session_id) = removed_id {
            self.apply_detach_on_destroy(session_id);
        }
        let before = self.sessions.len();
        self.sessions.retain(|s| s.name != name);
        let removed = self.sessions.len() != before;
        if removed {
            self.remove_unlinked_windows();
            self.request_shutdown_if_became_empty(had_sessions);
            if let Some(session_id) = removed_id {
                self.invalidate_session(session_id, RenderInvalidation::SESSION_GONE);
                self.notify_closed_session(SessionHook::SessionClosed, session_id, name);
            }
        }
        removed
    }

    /// Append an event notification to the server-wide queue, for its runner
    /// to turn into hook bodies. Mirrors tmux's `notify_add`, including the
    /// drop while the global queue is running a body of its own.
    fn notify(
        &mut self,
        name: &'static str,
        target: Option<String>,
        mut vars: Vec<(String, String)>,
    ) {
        if self.suppressed_notification_frames > 0 {
            return;
        }
        vars.insert(0, ("hook".to_string(), name.to_string()));
        self.pending_notifications.push_back(Notification {
            name: name.to_string(),
            target,
            vars,
        });
    }

    /// Latch a hook body as actively on the stack: what is raised from here
    /// until [`Self::end_notification_suppression`] is dropped where it is
    /// raised, as tmux's `notify_add` drops what a running
    /// `CMDQ_STATE_NOHOOKS` item raises. Held per poll, not per command — a
    /// parked body is tmux's waiting item, which `cmdq_running` masks.
    pub(crate) fn begin_notification_suppression(&mut self) {
        self.suppressed_notification_frames += 1;
    }

    pub(crate) fn end_notification_suppression(&mut self) {
        self.suppressed_notification_frames = self.suppressed_notification_frames.saturating_sub(1);
    }

    /// tmux's `notify_session`: an event about a session as a whole.
    pub(crate) fn notify_session(&mut self, hook: SessionHook, session_id: u32) {
        let Some(session) = self.sessions.iter().find(|s| s.id == session_id) else {
            // A closed session is still named by its own event.
            return;
        };
        let vars = vec![
            ("hook_session".to_string(), format!("${session_id}")),
            ("hook_session_name".to_string(), session.name.clone()),
        ];
        self.notify(hook.as_str(), Some(format!("${session_id}")), vars);
    }

    /// Like [`Self::notify_session`], for a session that has already been
    /// removed from the tree and can only be named by the caller.
    pub(crate) fn notify_closed_session(
        &mut self,
        hook: SessionHook,
        session_id: u32,
        session: &str,
    ) {
        let vars = vec![
            ("hook_session".to_string(), format!("${session_id}")),
            ("hook_session_name".to_string(), session.to_string()),
        ];
        self.notify(hook.as_str(), None, vars);
    }

    /// tmux's `notify_window`: an event about a window, carrying no session.
    pub(crate) fn notify_window(&mut self, hook: WindowHook, window_id: u32) {
        let Some(window) = self.windows.get(&window_id) else {
            return;
        };
        let vars = vec![
            ("hook_window".to_string(), format!("@{window_id}")),
            ("hook_window_name".to_string(), window.name.clone()),
        ];
        self.notify(hook.as_str(), Some(format!("@{window_id}")), vars);
    }

    /// tmux's `notify_session_window`/`notify_winlink`: an event about a window
    /// as seen from one session, so both layers are published.
    pub(crate) fn notify_session_window(
        &mut self,
        hook: SessionHook,
        session_id: u32,
        window_id: u32,
    ) {
        let session = self.sessions.iter().find(|s| s.id == session_id);
        let window_name = self.windows.get(&window_id).map(|w| w.name.clone());
        let mut vars = Vec::new();
        if let Some(session) = session {
            vars.push(("hook_session".to_string(), format!("${session_id}")));
            vars.push(("hook_session_name".to_string(), session.name.clone()));
        }
        if let Some(window_name) = window_name {
            vars.push(("hook_window".to_string(), format!("@{window_id}")));
            vars.push(("hook_window_name".to_string(), window_name));
        }
        // The window may already be unlinked, so the session is the target that
        // still resolves.
        let target = self
            .sessions
            .iter()
            .find(|s| s.id == session_id && s.windows.iter().any(|link| link.id == window_id))
            .map(|_| format!("${session_id}:@{window_id}"))
            .or_else(|| session.map(|_| format!("${session_id}")));
        self.notify(hook.as_str(), target, vars);
    }

    /// tmux's `notify_pane`: an event about a pane, which also publishes the
    /// window the pane belongs to.
    pub(crate) fn notify_pane(&mut self, hook: PaneHook, pane_id: u32) {
        let Some((window_id, window_name)) = self.windows.iter().find_map(|(id, window)| {
            window
                .panes
                .iter()
                .any(|node| node.id == pane_id)
                .then(|| (*id, window.name.clone()))
        }) else {
            return;
        };
        let vars = vec![
            ("hook_pane".to_string(), format!("%{pane_id}")),
            ("hook_window".to_string(), format!("@{window_id}")),
            ("hook_window_name".to_string(), window_name),
        ];
        self.notify(hook.as_str(), Some(format!("%{pane_id}")), vars);
    }

    /// tmux's `notify_client`: an event about one client.
    pub(crate) fn notify_client(&mut self, hook: SessionHook, client: &str, session_id: Option<u32>) {
        let vars = vec![("hook_client".to_string(), client.to_string())];
        self.notify(hook.as_str(), session_id.map(|id| format!("${id}")), vars);
    }

    /// Take the next item off the server-wide queue, in the order the
    /// mutations raised them, and mark it as the queue's running item until
    /// [`Self::finish_notification`].
    pub(crate) fn next_notification(&mut self) -> Option<Notification> {
        let notification = self.pending_notifications.pop_front()?;
        self.notification_running = true;
        Some(notification)
    }

    /// The running item's bodies are done; the queue is idle until the next
    /// [`Self::next_notification`].
    pub(crate) fn finish_notification(&mut self) {
        self.notification_running = false;
    }

    /// tmux's `server_client_set_session` tail: the client's new current window
    /// loses its alert flags, and the whole session's alert state is examined
    /// again now that somebody is looking at it.
    pub(super) fn take_session_for_client(&mut self, session_id: u32) {
        if let Some(session) = self.sessions.iter_mut().find(|s| s.id == session_id) {
            let active = session.active;
            if let Some(link) = session.windows.get_mut(active) {
                link.alert_flags = 0;
            }
        }
        self.alert_check_sessions.insert(session_id);
    }

    /// Raise the client-layer notifications tmux emits from `server_client_*`.
    ///
    /// The client registry is owned by the attach loops rather than by the
    /// command path, so what changed is read from a snapshot of it instead of
    /// from a call at each site.
    fn sync_client_notifications(&mut self) {
        let current = self.client_renders.with_entries(|entries| {
            entries
                .map(|entry| {
                    (
                        entry.name.clone(),
                        (entry.session_id, entry.cols, entry.rows),
                    )
                })
                .collect::<BTreeMap<_, _>>()
        });
        let previous = std::mem::replace(&mut self.known_clients, current.clone());
        for (name, (session_id, cols, rows)) in &current {
            match previous.get(name) {
                None => {
                    self.notify_client(SessionHook::ClientAttached, name, Some(*session_id));
                    self.notify_client(SessionHook::ClientSessionChanged, name, Some(*session_id));
                    // A new client announces its terminal size as part of
                    // attaching, which tmux reports as a resize of its own.
                    self.notify_client(SessionHook::ClientResized, name, Some(*session_id));
                    self.take_session_for_client(*session_id);
                }
                Some((old_session, old_cols, old_rows)) => {
                    if old_session != session_id {
                        self.notify_client(SessionHook::ClientSessionChanged, name, Some(*session_id));
                        self.take_session_for_client(*session_id);
                    }
                    if (old_cols, old_rows) != (cols, rows) {
                        self.notify_client(SessionHook::ClientResized, name, Some(*session_id));
                    }
                }
            }
        }
        for name in previous.keys().filter(|name| !current.contains_key(*name)) {
            self.notify_client(SessionHook::ClientDetached, name, None);
        }
        // Gaining or losing a client moves pane focus, for the window it
        // arrived at and the one it left.
        let touched = previous
            .values()
            .chain(current.values())
            .filter_map(|(session_id, _, _)| self.current_window_of_session(*session_id))
            .collect::<BTreeSet<_>>();
        for window_id in touched {
            self.update_window_focus(window_id);
        }
    }

    /// tmux's `session_update_activity`. `attached` additionally stamps
    /// `last_attached_time`, which only a client taking the session does.
    pub(crate) fn touch_session_activity(&mut self, session_id: u32, attached: bool) {
        let now = now_micros();
        if let Some(session) = self
            .sessions
            .iter_mut()
            .find(|session| session.id == session_id)
        {
            session.activity_micros = now;
            if attached {
                session.last_attached_micros = now;
            }
        }
    }

    /// The sessions that currently have at least one client attached.
    pub(super) fn attached_session_ids(&self) -> BTreeSet<u32> {
        self.client_renders
            .with_entries(|entries| entries.map(|entry| entry.session_id).collect())
    }

    /// The session other than `exclude` with the newest activity time, in
    /// tmux's `server_find_session(server_newer_session)` order: sessions are
    /// walked by name and the comparison is strict, so equal stamps keep the
    /// alphabetically first candidate.
    fn newest_session_excluding(&self, exclude: u32, detached_only: bool) -> Option<u32> {
        let attached = detached_only.then(|| self.attached_session_ids());
        let mut candidates = self
            .sessions
            .iter()
            .filter(|session| session.id != exclude)
            .filter(|session| {
                attached
                    .as_ref()
                    .is_none_or(|attached| !attached.contains(&session.id))
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|a, b| a.name.cmp(&b.name));
        candidates
            .into_iter()
            .fold(None::<&Session>, |best, session| match best {
                Some(best) if session.activity_micros <= best.activity_micros => Some(best),
                _ => Some(session),
            })
            .map(|session| session.id)
    }

    /// tmux's `session_previous_session`/`session_next_session`: the neighbour
    /// of `exclude` in the name-ordered session list, wrapping at both ends.
    fn neighbour_session(&self, exclude: u32, forward: bool) -> Option<u32> {
        let mut ordered = self.sessions.iter().collect::<Vec<_>>();
        ordered.sort_by(|a, b| a.name.cmp(&b.name));
        let count = ordered.len();
        if count < 2 {
            return None;
        }
        let position = ordered.iter().position(|session| session.id == exclude)?;
        let next = if forward {
            (position + 1) % count
        } else {
            (position + count - 1) % count
        };
        Some(ordered[next].id)
    }

    /// tmux's `server_destroy_session`: move the clients of a session that is
    /// about to disappear, according to that session's `detach-on-destroy`.
    /// Clients with no destination are left in place and exit themselves.
    fn apply_detach_on_destroy(&mut self, session_id: u32) {
        let Some(session) = self
            .sessions
            .iter()
            .find(|session| session.id == session_id)
        else {
            return;
        };
        let policy = session
            .options(&self.global_options)
            .get("detach-on-destroy")
            .unwrap_or("on")
            .to_owned();
        let new_session = match policy.as_str() {
            "off" | "0" => self.newest_session_excluding(session_id, false),
            "no-detached" => self.newest_session_excluding(session_id, true),
            "previous" => self.neighbour_session(session_id, false),
            "next" => self.neighbour_session(session_id, true),
            _ => None,
        };
        // `on` and `no-detached` still offer a session to clients that asked
        // not to be detached when the policy itself found none.
        let no_detach_session = match (&new_session, policy.as_str()) {
            (None, "on" | "1" | "no-detached") => self.newest_session_excluding(session_id, false),
            _ => None,
        };
        self.client_renders
            .reassign_session(session_id, new_session, no_detach_session);
        // Taking over a session counts as activity on it, exactly as an
        // ordinary attach does.
        for adopted in [new_session, no_detach_session].into_iter().flatten() {
            self.touch_session_activity(adopted, true);
        }
    }

    /// tmux's `server_check_unattached`, run whenever a client is lost or
    /// changes session — not only when the option is set.
    pub(crate) fn enforce_unattached_options(&mut self) {
        // tmux walks its name-ordered session tree and destroys in place, so a
        // group shrinking under `keep-last` spares whichever member is reached
        // once it is the last one. Re-scanning after each destroy reproduces
        // that without depending on hmux's creation-ordered session vector.
        while let Some(name) = self.next_unattached_session_to_destroy() {
            self.kill_session(&name);
        }
    }

    fn next_unattached_session_to_destroy(&self) -> Option<String> {
        let attached = self.attached_session_ids();
        let mut ordered = self.sessions.iter().collect::<Vec<_>>();
        ordered.sort_by(|a, b| a.name.cmp(&b.name));
        ordered
            .into_iter()
            .find(|session| {
                if attached.contains(&session.id) {
                    return false;
                }
                let grouped = self.is_grouped(session);
                let group_size = if grouped {
                    self.session_group_size(session)
                } else {
                    1
                };
                match session
                    .options(&self.global_options)
                    .get("destroy-unattached")
                    .unwrap_or("off")
                {
                    "on" | "1" => true,
                    // Only a session sharing its group with others is destroyed.
                    "keep-last" => grouped && group_size > 1,
                    // A session alone in an explicit group is spared.
                    "keep-group" => !(grouped && group_size == 1),
                    _ => false,
                }
            })
            .map(|session| session.name.clone())
    }

    /// One pass of tmux's per-loop lifecycle policies: destroy sessions that
    /// lost their last client, then decide whether the server itself should
    /// exit. The unattached sweep is skipped while the client set is unchanged.
    pub(crate) fn enforce_lifecycle_policies(&mut self) {
        let generation = self.client_renders.generation();
        if generation != self.lifecycle_generation {
            self.lifecycle_generation = generation;
            self.sync_client_notifications();
            self.enforce_unattached_options();
        }
        self.enforce_exit_options();
    }

    /// tmux's `server_loop` shutdown test: with `exit-empty` on, the server
    /// exits once nothing holds it — no sessions (or `exit-unattached` on) and
    /// no client still attached to one.
    pub(crate) fn enforce_exit_options(&mut self) {
        match self.exit_empty_policy() {
            ExitEmpty::Off => return,
            // An hmux daemon is launched before any client exists, so
            // `after-session` holds the policy back until the server has held
            // a session at least once; tmux never sees this window because its
            // server is forked by the client that creates the first session.
            ExitEmpty::AfterSession if self.initial_attach_pending => return,
            ExitEmpty::AfterSession | ExitEmpty::On => {}
        }
        if !self.server_option_is_on("exit-unattached", false) && !self.sessions.is_empty() {
            return;
        }
        if !self.attached_session_ids().is_empty() {
            return;
        }
        self.shutdown_requested = true;
    }

    /// `kill-session -g`: destroy every member when the target is grouped, or
    /// just the target when it is not.
    pub fn kill_session_group(&mut self, name: &str) -> bool {
        let Some(position) = self.session_index(name) else {
            return false;
        };
        let link_set_id = self.sessions[position].link_set_id;
        if !self.session_groups.contains_key(&link_set_id) {
            return self.kill_session(name);
        }
        let had_sessions = !self.sessions.is_empty();
        let removed = self
            .sessions
            .iter()
            .filter_map(|session| (session.link_set_id == link_set_id).then_some(session.id))
            .collect::<Vec<_>>();
        self.sessions
            .retain(|session| session.link_set_id != link_set_id);
        self.remove_unlinked_windows();
        self.request_shutdown_if_became_empty(had_sessions);
        for session_id in removed {
            self.invalidate_session(session_id, RenderInvalidation::SESSION_GONE);
        }
        true
    }

    /// `kill-session -a [-t keep]`: destroy every session except `keep`.
    /// Errors (`can't find session`) if `keep` doesn't exist.
    pub fn kill_other_sessions(&mut self, keep: &str) -> io::Result<()> {
        if self.session_index(keep).is_none() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("can't find session: {keep}"),
            ));
        }
        let keep_id = self.sessions[self.session_index(keep).unwrap()].id;
        let removed: Vec<u32> = self
            .sessions
            .iter()
            .filter_map(|session| (session.id != keep_id).then_some(session.id))
            .collect();
        self.sessions.retain(|s| s.id == keep_id);
        self.remove_unlinked_windows();
        for session_id in removed {
            self.invalidate_session(session_id, RenderInvalidation::SESSION_GONE);
        }
        Ok(())
    }
}
