//! Turning a parsed [`Target`] into the session, window, or pane it names.
//!
//! The parsing itself is in `state::target`; what lives here is the lookup
//! against live state — resolving `{next}`/`-`/`!` against the current
//! session, disambiguating name prefixes, and reporting the error tmux would.

use std::io;

use super::target::{
    pane_not_found, pane_pos_in, session_not_found, unique, window_not_found, window_offset,
    window_special, Target, TargetKind, TargetMiss, TargetParts, WindowMatch,
};
use super::{ServerState, Session};
use crate::server::format::glob_match;

impl ServerState {
    /// The id of a session's current window.
    pub(super) fn current_window_of_session(&self, session_id: u32) -> Option<u32> {
        let session = self.sessions.iter().find(|s| s.id == session_id)?;
        session.windows.get(session.active).map(|link| link.id)
    }

    pub fn find(&self, name: &str) -> Option<&Session> {
        self.resolve_session(name)
    }

    /// Resolve a full pane target to concrete `Vec` positions. Accepts every
    /// tmux target form the control plane uses:
    /// - `%N` — a pane id (searched across all sessions/windows);
    /// - `@N` — a window id (`[.pane]` optional);
    /// - `$N:...` / `name:...` — a session (by id, name, unique name prefix or
    ///   unique `fnmatch` pattern) with an optional `:window` (index, `@id`,
    ///   special token, name, prefix or pattern) and `.pane` (index or `%id`)
    ///   suffix.
    ///
    /// A missing window part means the session's active window; a missing pane
    /// part means that window's active pane. Returns `None` if any part can't be
    /// resolved.
    pub fn resolve(&self, target: &str) -> Option<Target> {
        self.find_target(target, TargetKind::Pane).ok()
    }

    /// [`Self::resolve`] for tmux's *can-fail* target lookups (`CMD_FIND_CANFAIL`,
    /// e.g. `display-message`). Those commands do not fail on an unresolvable
    /// target: they run against however far the lookup got — the named session's
    /// current window, say — or against nothing at all.
    pub fn resolve_or_residual(&self, target: &str) -> Option<Target> {
        match self.find_target(target, TargetKind::Pane) {
            Ok(target) => Some(target),
            Err(miss) => miss.residual,
        }
    }

    /// tmux's diagnostic for a pane target that doesn't resolve. tmux names only
    /// the part that went missing, so `can't find pane: <t>` is wrong for a
    /// target whose *session* or *window* part is the unresolvable one.
    pub fn pane_target_error(&self, target: &str) -> io::Error {
        match self.find_target(target, TargetKind::Pane) {
            Ok(_) => pane_not_found(target),
            Err(miss) => miss.error,
        }
    }

    /// Resolve a target of either kind, following tmux's `cmd_find_target`: split
    /// the target into its session, window and pane parts, then resolve each part
    /// against the state the previous one selected, falling back to the current
    /// session/window for the parts the target leaves out.
    pub(super) fn find_target(&self, target: &str, kind: TargetKind) -> Result<Target, TargetMiss> {
        // `~`/`{marked}` — the server's marked pane, whichever window it is in.
        if target == "~" || target == "{marked}" {
            return self
                .marked_pane()
                .and_then(|id| self.pane_position(id))
                .ok_or_else(|| {
                    TargetMiss::new(
                        io::Error::new(io::ErrorKind::NotFound, "no marked target"),
                        None,
                    )
                });
        }
        // `=` — whatever the mouse event that dispatched this command hit
        // (tmux's `cmd_find_from_mouse`). A status click that only names a
        // window resolves to that window rather than to a pane.
        if target == "=" {
            let miss = || TargetMiss::new(pane_not_found(target), None);
            let mouse = self.command_mouse().ok_or_else(miss)?;
            let mouse = mouse.target.as_ref().ok_or_else(miss)?;
            let target = if let Some(pane) = mouse.pane_id {
                format!("%{pane}")
            } else if let Some(window) = mouse.window_id {
                format!("@{window}")
            } else {
                format!("${}", mouse.session_id)
            };
            return self.find_target(&target, kind);
        }

        let parts = TargetParts::parse(target, kind);
        let Some(session) = parts.session else {
            return match (parts.window, parts.pane) {
                (Some(window), pane) => {
                    let (session, window) =
                        self.window_anywhere(window, parts.exact_window, parts.window_only)?;
                    self.with_pane(session, window, pane)
                }
                (None, Some(pane)) => self.pane_anywhere(pane, parts.pane_only),
                (None, None) => {
                    let session = self
                        .current_session_pos()
                        .map_err(|error| TargetMiss::new(error, None))?;
                    Ok(self.session_current(session))
                }
            };
        };
        let Some(position) = self.session_lookup(session, parts.exact_session) else {
            return Err(TargetMiss::new(session_not_found(session), None));
        };
        let window = match parts.window {
            Some(spec) => match self.window_in_session(position, spec, parts.exact_window) {
                WindowMatch::Found(window) => window,
                found => {
                    let residual = found.ambiguous().map_or_else(
                        || self.session_current(position),
                        |first| self.window_current(position, first),
                    );
                    return Err(TargetMiss::new(window_not_found(spec), Some(residual)));
                }
            },
            None => self.sessions[position].active,
        };
        self.with_pane(position, window, parts.pane)
    }

    /// Attach the pane part of a target to an already-resolved window. Without
    /// one the window's active pane is the target, as in tmux.
    fn with_pane(
        &self,
        session: usize,
        window: usize,
        pane: Option<&str>,
    ) -> Result<Target, TargetMiss> {
        let Some(spec) = pane else {
            return Ok(self.window_current(session, window));
        };
        match self.pane_pos(session, window, spec) {
            Some(pane) => Ok(Target {
                session,
                window,
                pane,
            }),
            None => Err(TargetMiss::new(
                pane_not_found(spec),
                Some(self.window_current(session, window)),
            )),
        }
    }

    /// Resolve a window part that came without a session part: a global `@id`,
    /// else a window of the current session and — unless the target spelled the
    /// part out as a window with `:` or `.` (`only`) — finally a session name,
    /// whose current window it is then.
    fn window_anywhere(
        &self,
        spec: &str,
        exact: bool,
        only: bool,
    ) -> Result<(usize, usize), TargetMiss> {
        if spec.starts_with('@') {
            return self
                .window_by_id(spec)
                .ok_or_else(|| TargetMiss::new(window_not_found(spec), None));
        }
        let session = self
            .current_session_pos()
            .map_err(|error| TargetMiss::new(error, None))?;
        let found = self.window_in_session(session, spec, exact);
        if let WindowMatch::Found(window) = found {
            return Ok((session, window));
        }
        if !only {
            // tmux's session fallback overwrites the state it reached with the
            // (missing) session, so a target that gets this far and still fails
            // leaves nothing behind for a can-fail lookup to format against.
            return match self.session_lookup(spec, false) {
                Some(position) => Ok((position, self.sessions[position].active)),
                None => Err(TargetMiss::new(window_not_found(spec), None)),
            };
        }
        let residual = found.ambiguous().map_or_else(
            || self.session_current(session),
            |first| self.window_current(session, first),
        );
        Err(TargetMiss::new(window_not_found(spec), Some(residual)))
    }

    /// Resolve a pane part that came without a session or window part: a global
    /// `%id`, else a pane of the current window and — unless the target spelled
    /// the part out as a pane with `.` (`only`) — a window or session, whose
    /// active pane it is then.
    fn pane_anywhere(&self, spec: &str, only: bool) -> Result<Target, TargetMiss> {
        if spec.starts_with('%') {
            return self
                .pane_by_id(spec)
                .ok_or_else(|| TargetMiss::new(pane_not_found(spec), None));
        }
        let session = self
            .current_session_pos()
            .map_err(|error| TargetMiss::new(error, None))?;
        let window = self.sessions[session].active;
        if let Some(pane) = self.pane_pos(session, window, spec) {
            return Ok(Target {
                session,
                window,
                pane,
            });
        }
        if !only {
            if let Ok((session, window)) = self.window_anywhere(spec, false, false) {
                return Ok(Target {
                    session,
                    window,
                    pane: self.active_pane_pos(session, window),
                });
            }
            // Still tmux's *pane* diagnostic: the window and session attempts
            // were fallbacks for a target that named a pane.
            return Err(TargetMiss::new(pane_not_found(spec), None));
        }
        Err(TargetMiss::new(
            pane_not_found(spec),
            Some(self.session_current(session)),
        ))
    }

    /// A session's current window and that window's active pane — what a target
    /// naming only the session resolves to, and what tmux's can-fail lookups
    /// fall back to once they have gotten as far as the session.
    fn session_current(&self, session: usize) -> Target {
        self.window_current(session, self.sessions[session].active)
    }

    /// A window and its active pane — what a target naming only the window
    /// resolves to.
    fn window_current(&self, session: usize, window: usize) -> Target {
        Target {
            session,
            window,
            pane: self.active_pane_pos(session, window),
        }
    }

    /// The active pane of a window, honoring the per-client active pane the
    /// running command was dispatched with.
    fn active_pane_pos(&self, session: usize, window: usize) -> usize {
        let window = self.window(session, window);
        self.command_active_panes
            .as_ref()
            .and_then(|panes| panes.get(&window.id))
            .and_then(|pane_id| window.panes.iter().position(|pane| pane.id == *pane_id))
            .unwrap_or(window.active)
    }

    /// Locate a `@id` window anywhere in the tree.
    fn window_by_id(&self, spec: &str) -> Option<(usize, usize)> {
        let id: u32 = spec.strip_prefix('@')?.parse().ok()?;
        self.sessions.iter().enumerate().find_map(|(si, sess)| {
            let wi = sess.windows.iter().position(|link| link.id == id)?;
            Some((si, wi))
        })
    }

    /// Locate a `%id` pane anywhere in the tree.
    fn pane_by_id(&self, spec: &str) -> Option<Target> {
        self.pane_position(spec.strip_prefix('%')?.parse().ok()?)
    }

    /// Locate a pane by id anywhere in the tree.
    fn pane_position(&self, id: u32) -> Option<Target> {
        self.sessions.iter().enumerate().find_map(|(si, sess)| {
            sess.windows.iter().enumerate().find_map(|(wi, link)| {
                let pi = self
                    .window_for_link(link)
                    .panes
                    .iter()
                    .position(|pane| pane.id == id)?;
                Some(Target {
                    session: si,
                    window: wi,
                    pane: pi,
                })
            })
        })
    }

    pub fn resolve_session(&self, target: &str) -> Option<&Session> {
        let spec = target.split_once(':').map(|(s, _)| s).unwrap_or(target);
        self.session_index(spec).map(|i| &self.sessions[i])
    }

    /// The session named exactly `name`, for the checks that are about the name
    /// itself rather than about resolving a target — tmux rejects a duplicate
    /// session name, but only an identical one.
    pub fn find_exact(&self, name: &str) -> Option<&Session> {
        self.sessions.iter().find(|session| session.name == name)
    }

    /// Position of a session named by `spec`: a session name, a `$id`, a unique
    /// name prefix, or a unique `fnmatch` pattern.
    pub(super) fn session_index(&self, spec: &str) -> Option<usize> {
        // A leading `=` requests an exact target-name match in tmux target
        // syntax (`-t=test:`). Session names in the model are stored without
        // that selector marker.
        match spec.strip_prefix('=') {
            Some(spec) => self.session_lookup(spec, true),
            None => self.session_lookup(spec, false),
        }
    }

    /// Position of a session named by `spec`, in tmux's `cmd_find_get_session`
    /// order: a `$id`, an exact name, then — unless `exact` (the `=` selector) —
    /// a unique name prefix and a unique `fnmatch` pattern. A prefix or pattern
    /// matching more than one session is a miss, as in tmux.
    fn session_lookup(&self, spec: &str, exact: bool) -> Option<usize> {
        if let Some(id) = spec.strip_prefix('$') {
            let id: u32 = id.parse().ok()?;
            return self.sessions.iter().position(|s| s.id == id);
        }
        let named = |name: &str| self.sessions.iter().position(|s| s.name == name);
        if let Some(position) = named(spec) {
            return Some(position);
        }
        if exact {
            return None;
        }
        let names = || self.sessions.iter().enumerate().map(|(i, s)| (i, &s.name));
        unique(names().filter(|(_, name)| name.starts_with(spec))).or_else(|| {
            unique(names().filter(|(_, name)| glob_match(spec.as_bytes(), name.as_bytes())))
        })
    }

    /// Position of a session (by name or `$id`), for the navigation commands.
    pub(super) fn session_pos(&self, name: &str) -> Option<usize> {
        self.session_index(name)
    }

    /// Position of the "current" session — the one a target with no (or empty)
    /// session part refers to. Real tmux picks the most-recently-active session;
    /// we approximate with the newest, matching `command::current_session`.
    pub(super) fn current_session_pos(&self) -> io::Result<usize> {
        self.sessions.len().checked_sub(1).ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "no current session".to_string())
        })
    }

    /// Resolve a pane index within a window from a pane spec: a numeric index
    /// (matched against `Vec` position), a `%id`, or one of tmux's special pane
    /// tokens (`{last}`, `{next}`, `{previous}`, `{top}`, `{bottom}`, `+`, `-`).
    pub(super) fn pane_pos(&self, session: usize, window: usize, spec: &str) -> Option<usize> {
        let win = self.window(session, window);
        let base = win
            .options(&self.global_options)
            .get("pane-base-index")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(0);
        pane_pos_in(win, spec, base)
    }

    /// Resolve the window part of a target within one session, in tmux's
    /// `cmd_find_get_window_with_session` order: a `@id` linked into the session,
    /// a `+`/`-` offset, a special token, an index, an exact name, a name prefix
    /// and finally an `fnmatch` pattern. A name, prefix or pattern matching more
    /// than one window does not resolve, as in tmux. `exact` (the `=` selector)
    /// stops the search after the exact name.
    fn window_in_session(&self, session: usize, spec: &str, exact: bool) -> WindowMatch {
        let sess = &self.sessions[session];
        if let Some(id) = spec.strip_prefix('@') {
            let position = id
                .parse::<u32>()
                .ok()
                .and_then(|id| sess.windows.iter().position(|link| link.id == id));
            return WindowMatch::of(position);
        }
        if !exact {
            if let Some(position) = window_offset(sess, spec).or_else(|| window_special(sess, spec))
            {
                return WindowMatch::Found(position);
            }
        }
        // An offset that got this far named no window; it is never an index.
        if !spec.starts_with(['+', '-']) {
            if let Ok(index) = spec.parse::<u32>() {
                if let Some(position) = sess.windows.iter().position(|link| link.index == index) {
                    return WindowMatch::Found(position);
                }
            }
        }
        let names = || {
            sess.windows
                .iter()
                .enumerate()
                .map(|(position, link)| (position, &self.window_for_link(link).name))
        };
        let by_name = WindowMatch::single(names().filter(|(_, name)| name.as_str() == spec));
        if !matches!(by_name, WindowMatch::None) || exact {
            return by_name;
        }
        let by_prefix = WindowMatch::single(names().filter(|(_, name)| name.starts_with(spec)));
        if !matches!(by_prefix, WindowMatch::None) {
            return by_prefix;
        }
        WindowMatch::single(
            names().filter(|(_, name)| glob_match(spec.as_bytes(), name.as_bytes())),
        )
    }

    /// Resolve `session[:window]` for the window-lifecycle commands, returning
    /// tmux's exact diagnostics: `can't find session: <name>` if the session is
    /// missing, `can't find window: <idx>` if the window index doesn't exist.
    pub(super) fn resolve_window(&self, target: &str) -> io::Result<Target> {
        // Pane/session target (split-window, select-pane, …): a colon-less bare
        // number is a pane index in the current window before it is anything else.
        self.find_target(target, TargetKind::Pane)
            .map_err(|miss| miss.error)
    }

    /// Like [`Self::resolve_window`], but for commands whose `-t` is a genuine
    /// *window* target (`select-window`, `kill-window`, `list-panes`): there a
    /// colon-less bare word is a window in the current session before it is a
    /// session name, per tmux's `cmd_find_target`, and a miss reports tmux's
    /// window diagnostic rather than its pane one.
    pub fn resolve_window_target(&self, target: &str) -> io::Result<Target> {
        self.find_target(target, TargetKind::Window)
            .map_err(|miss| miss.error)
    }

    /// Find a session mutably.
    pub fn find_mut(&mut self, name: &str) -> Option<&mut Session> {
        let pos = self.session_index(name)?;
        self.sessions.get_mut(pos)
    }
}
