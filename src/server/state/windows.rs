//! Windows and the winlinks that hang them off sessions.
//!
//! A window is owned by the server and linked into any number of sessions by a
//! [`Winlink`], so creating, moving, renumbering, and killing windows is mostly
//! link bookkeeping. Window alerts live here too — they are raised on the
//! window and delivered to every session that links it.

use std::collections::BTreeSet;
use std::io;
use std::path::Path;

use super::layout::resize_panes_to_layout;
use super::sizing::{DEFAULT_XPIXEL, DEFAULT_YPIXEL};
use super::target::{parse_index_target, window_not_found};
use super::{
    fill_spawn_ids, now_micros, LayoutCell, Pane, PaneNode, RenderInvalidation,
    ServerState, Session, Window, Winlink, ALERT_ACTIVITY, ALERT_BELL, ALERT_SILENCE,
};
use crate::server::options::{OptionSet, SessionHook, WindowHook};

impl ServerState {
    pub(crate) fn window(&self, session: usize, window: usize) -> &Window {
        let id = self.sessions[session].windows[window].id;
        self.windows
            .get(&id)
            .expect("every winlink references a live window")
    }

    pub(crate) fn printable_window_flags(
        &self,
        session: &Session,
        position: usize,
        escape_activity: bool,
    ) -> String {
        let Some(link) = session.windows.get(position) else {
            return String::new();
        };
        let mut flags = String::new();
        if link.alert_flags & ALERT_ACTIVITY != 0 {
            flags.push('#');
            if escape_activity {
                flags.push('#');
            }
        }
        if link.alert_flags & ALERT_BELL != 0 {
            flags.push('!');
        }
        if link.alert_flags & ALERT_SILENCE != 0 {
            flags.push('~');
        }
        if position == session.active {
            flags.push('*');
        }
        if session.last_active() == Some(position) {
            flags.push('-');
        }
        if self.marked_pane_id.is_some_and(|marked| {
            self.window_for_link(link)
                .panes
                .iter()
                .any(|pane| pane.id == marked)
        }) {
            flags.push('M');
        }
        if self.window_for_link(link).zoomed {
            flags.push('Z');
        }
        flags
    }

    pub(crate) fn session_alert(&self, session: &Session) -> String {
        let mut alerts = String::new();
        let combined = session
            .windows
            .iter()
            .fold(0u8, |flags, link| flags | link.alert_flags);
        if combined & ALERT_ACTIVITY != 0 {
            alerts.push('#');
        }
        if combined & ALERT_BELL != 0 {
            alerts.push('!');
        }
        if combined & ALERT_SILENCE != 0 {
            alerts.push('~');
        }
        alerts
    }

    pub(crate) fn session_alerts(&self, session: &Session) -> String {
        session
            .windows
            .iter()
            .filter(|link| link.alert_flags != 0)
            .map(|link| {
                let mut alert = link.index.to_string();
                if link.alert_flags & ALERT_ACTIVITY != 0 {
                    alert.push('#');
                }
                if link.alert_flags & ALERT_BELL != 0 {
                    alert.push('!');
                }
                if link.alert_flags & ALERT_SILENCE != 0 {
                    alert.push('~');
                }
                alert
            })
            .collect::<Vec<_>>()
            .join(",")
    }

    pub(crate) fn window_mut(&mut self, session: usize, window: usize) -> &mut Window {
        let id = self.sessions[session].windows[window].id;
        self.windows
            .get_mut(&id)
            .expect("every winlink references a live window")
    }

    pub(crate) fn window_for_link(&self, link: &Winlink) -> &Window {
        self.windows
            .get(&link.id)
            .expect("every winlink references a live window")
    }

    pub(crate) fn session_window(&self, session: &Session, window: usize) -> &Window {
        self.window_for_link(&session.windows[window])
    }

    pub(crate) fn all_windows(&self) -> impl Iterator<Item = &Window> {
        self.windows.values()
    }

    pub(crate) fn window_reference_count(&self, window_id: u32) -> usize {
        self.sessions
            .iter()
            .map(|session| {
                session
                    .windows
                    .iter()
                    .filter(|link| link.id == window_id)
                    .count()
            })
            .sum()
    }

    pub(crate) fn window_linked_session_count(&self, window_id: u32) -> usize {
        self.sessions
            .iter()
            .filter(|session| session.windows.iter().any(|link| link.id == window_id))
            .map(|session| session.link_set_id)
            .collect::<BTreeSet<_>>()
            .len()
    }

    pub(crate) fn window_linked_session_list(&self, window_id: u32) -> String {
        self.sessions
            .iter()
            .flat_map(|session| {
                session
                    .windows
                    .iter()
                    .filter(move |link| link.id == window_id)
                    .map(move |_| session.name.as_str())
            })
            .collect::<Vec<_>>()
            .join(",")
    }

    pub(crate) fn window_active_session_list(&self, window_id: u32) -> Vec<&str> {
        self.sessions
            .iter()
            .filter_map(|session| {
                session
                    .windows
                    .get(session.active)
                    .is_some_and(|link| link.id == window_id)
                    .then_some(session.name.as_str())
            })
            .collect()
    }

    /// The clients whose session currently shows `window_id` — tmux's
    /// `#{window_active_clients}` walk over `c->session->curw`.
    pub(crate) fn window_active_client_names(&self, window_id: u32) -> Vec<String> {
        let showing = self
            .sessions
            .iter()
            .filter(|session| {
                session
                    .windows
                    .get(session.active)
                    .is_some_and(|link| self.window_for_link(link).id == window_id)
            })
            .map(|session| session.id)
            .collect::<BTreeSet<u32>>();
        self.client_renders.names_for_sessions(&showing)
    }

    /// Drop the `lastw` entries that no longer name a linked window, plus the
    /// one the session just made active, so the stack's head is the window
    /// `last-window` goes back to.
    fn refresh_last_window(session: &mut Session) {
        let active_id = session.windows.get(session.active).map(|link| link.link_id);
        let live = session
            .windows
            .iter()
            .map(|link| link.link_id)
            .collect::<BTreeSet<_>>();
        session
            .last_windows
            .retain(|id| live.contains(id) && Some(*id) != active_id);
        let mut seen = BTreeSet::new();
        session.last_windows.retain(|id| seen.insert(*id));
    }

    fn select_window_position(session: &mut Session, position: usize) {
        if position >= session.windows.len() {
            return;
        }
        session.windows[position].alert_flags = 0;
        if position == session.active {
            return;
        }
        let old_id = session.windows[session.active].link_id;
        let new_id = session.windows[position].link_id;
        session
            .last_windows
            .retain(|id| *id != new_id && *id != old_id);
        session.last_windows.insert(0, old_id);
        session.active = position;
        Self::refresh_last_window(session);
    }

    /// [`Self::select_window_position`] plus tmux's `session-window-changed`
    /// notification, which only fires when the active window really moved.
    fn select_session_window(&mut self, session_pos: usize, position: usize) {
        let session = &mut self.sessions[session_pos];
        let previous = session.active;
        Self::select_window_position(session, position);
        if session.active != previous {
            let session_id = session.id;
            self.notify_session(SessionHook::SessionWindowChanged, session_id);
            // tmux's `session_select` tail: a selection runs
            // `window_update_activity`, so the alert check fires again and an
            // unattached session's monitor-activity re-flags even the winlink
            // it just selected.
            let window_id = self.sessions[session_pos].windows[position].id;
            let monitor_activity = self.windows.get(&window_id).is_some_and(|window| {
                window.options(&self.global_options).get("monitor-activity") == Some("on")
            });
            self.window_last_activity
                .insert(window_id, std::time::Instant::now());
            if monitor_activity {
                self.deliver_alert(window_id, ALERT_ACTIVITY);
            }
        }
    }

    pub(super) fn install_links_by_index(session: &mut Session, links: Vec<Winlink>) {
        let active_index = session.windows.get(session.active).map(|link| link.index);
        // Carried by index, not by position: the caller's list renumbers them.
        let stack_indices = session
            .last_windows
            .iter()
            .filter_map(|id| {
                session
                    .windows
                    .iter()
                    .find(|link| link.link_id == *id)
                    .map(|link| link.index)
            })
            .collect::<Vec<_>>();
        session.windows = links;
        if session.windows.is_empty() {
            session.active = 0;
            session.last_windows.clear();
            return;
        }
        session.active = active_index
            .and_then(|index| session.windows.iter().position(|link| link.index == index))
            .or_else(|| {
                stack_indices
                    .iter()
                    .find_map(|index| session.windows.iter().position(|link| link.index == *index))
            })
            .or_else(|| {
                active_index.and_then(|index| {
                    session
                        .windows
                        .iter()
                        .enumerate()
                        .rfind(|(_, link)| link.index < index)
                        .map(|(position, _)| position)
                })
            })
            .unwrap_or(0);
        let active_id = session.windows[session.active].link_id;
        session.last_windows = stack_indices
            .into_iter()
            .filter_map(|index| {
                session
                    .windows
                    .iter()
                    .find(|link| link.index == index)
                    .map(|link| link.link_id)
            })
            .filter(|id| *id != active_id)
            .collect();
        Self::refresh_last_window(session);
    }

    pub(super) fn replace_link_set(&mut self, session: usize, links: Vec<Winlink>) {
        Self::install_links_by_index(&mut self.sessions[session], links);
        self.synchronize_group_from(session);
    }

    fn replace_link_set_preserving_positions(&mut self, session: usize, links: Vec<Winlink>) {
        let member = &mut self.sessions[session];
        member.windows = links;
        member.active = member.active.min(member.windows.len().saturating_sub(1));
        Self::refresh_last_window(member);
        self.synchronize_group_from(session);
    }

    pub(super) fn insert_link(
        &mut self,
        session: usize,
        position: usize,
        link: Winlink,
        select: bool,
    ) {
        let link_id = link.link_id;
        let had_windows = !self.sessions[session].windows.is_empty();
        let active_id = self.sessions[session]
            .windows
            .get(self.sessions[session].active)
            .map(|candidate| candidate.link_id);
        self.sessions[session].windows.insert(position, link);
        self.sessions[session].active = active_id
            .and_then(|id| {
                self.sessions[session]
                    .windows
                    .iter()
                    .position(|candidate| candidate.link_id == id)
            })
            .unwrap_or(0);
        Self::refresh_last_window(&mut self.sessions[session]);
        self.synchronize_group_from(session);
        let (session_id, window_id) = {
            let member = &self.sessions[session];
            let link = member
                .windows
                .iter()
                .find(|candidate| candidate.link_id == link_id)
                .expect("inserted winlink is present");
            (member.id, link.id)
        };
        self.notify_session_window(SessionHook::WindowLinked, session_id, window_id);
        if select || !had_windows {
            let new_active = self.sessions[session]
                .windows
                .iter()
                .position(|candidate| candidate.link_id == link_id)
                .expect("inserted winlink is present");
            self.select_session_window(session, new_active);
        }
    }

    fn remove_link(&mut self, session: usize, position: usize) -> Winlink {
        let session_id = self.sessions[session].id;
        let mut links = self.sessions[session].windows.clone();
        let removed = links.remove(position);
        Self::install_links_by_index(&mut self.sessions[session], links);
        self.synchronize_group_from(session);
        self.notify_session_window(SessionHook::WindowUnlinked, session_id, removed.id);
        removed
    }

    pub(super) fn remove_unlinked_windows(&mut self) {
        let linked = self
            .sessions
            .iter()
            .flat_map(|session| session.windows.iter().map(|link| link.id))
            .collect::<BTreeSet<_>>();
        self.windows.retain(|id, _| linked.contains(id));
        let live_link_sets = self
            .sessions
            .iter()
            .map(|session| session.link_set_id)
            .collect::<BTreeSet<_>>();
        self.session_groups
            .retain(|link_set_id, _| live_link_sets.contains(link_set_id));
    }

    /// tmux's `window_update_focus`: recompute whether the window's active pane
    /// holds focus and announce a change. Deliberately ungated — the
    /// `focus-events` option decides whether tmux *asks* the terminal for focus
    /// reports, not whether it acts on what it knows.
    pub(crate) fn update_window_focus(&mut self, window_id: u32) {
        let Some(active) = self
            .windows
            .get(&window_id)
            .and_then(|window| window.panes.get(window.active))
            .map(|pane| pane.id)
        else {
            return;
        };
        let focused = self.window_has_focused_client(window_id);
        self.update_pane_focus(active, focused);
    }

    /// Whether some attached, focused client is currently showing this window.
    fn window_has_focused_client(&self, window_id: u32) -> bool {
        self.client_renders.with_entries(|mut entries| {
            entries.any(|entry| {
                entry.focused
                    && self
                        .current_window_of_session(entry.session_id)
                        .is_some_and(|current| current == window_id)
            })
        })
    }

    /// Open a new window (one shell-backed pane) in session `session`, as tmux's
    /// `new-window` does.
    ///
    /// `explicit` is the requested window index (`new-window -t sess:N`); `None`
    /// means append at the next free index (`new-window -t sess:`), which tmux
    /// picks as one past the highest existing index (starting from `base-index`,
    /// which defaults to 0). An `explicit` index already in use is rejected with
    /// tmux's `create window failed: index N in use`. Errors if the session is
    /// missing. Windows are kept sorted by `index`; the returned value is the
    /// `Vec` position of the new window (now the session's active window).
    pub fn new_window(
        &mut self,
        session: &str,
        explicit: Option<u32>,
        select: bool,
    ) -> io::Result<usize> {
        self.new_window_impl(session, explicit, false, select)
    }

    pub(crate) fn new_window_with_spawn(
        &mut self,
        session: &str,
        explicit: Option<u32>,
        select: bool,
        argv: &[String],
        cwd: Option<&Path>,
        name: &str,
    ) -> io::Result<usize> {
        self.new_window_spawn_impl(session, explicit, false, select, argv, cwd, name)
    }

    /// Open a new window, replacing an existing explicit target for
    /// `new-window -k`.
    pub(crate) fn new_window_replacing_with_spawn(
        &mut self,
        session: &str,
        explicit: Option<u32>,
        select: bool,
        argv: &[String],
        cwd: Option<&Path>,
        name: &str,
    ) -> io::Result<usize> {
        self.new_window_spawn_impl(session, explicit, true, select, argv, cwd, name)
    }

    fn new_window_impl(
        &mut self,
        session: &str,
        explicit: Option<u32>,
        replace: bool,
        select: bool,
    ) -> io::Result<usize> {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
        let name = super::default_window_name(std::slice::from_ref(&shell));
        self.new_window_spawn_impl(session, explicit, replace, select, &[shell], None, &name)
    }

    fn new_window_spawn_impl(
        &mut self,
        session: &str,
        explicit: Option<u32>,
        replace: bool,
        select: bool,
        argv: &[String],
        cwd: Option<&Path>,
        name: &str,
    ) -> io::Result<usize> {
        let session_pos = self.session_index(session);
        let s = match session_pos.and_then(|pos| self.sessions.get(pos)) {
            Some(s) => s,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("can't find session: {session}"),
                ))
            }
        };
        let session_id = s.id;
        let had_windows = !s.windows.is_empty();
        let index = match explicit {
            Some(i) => {
                if !replace && s.windows.iter().any(|w| w.index == i) {
                    return Err(io::Error::new(
                        io::ErrorKind::AlreadyExists,
                        format!("create window failed: index {i} in use"),
                    ));
                }
                i
            }
            None => {
                // tmux appends at the lowest unused index at or above
                // `base-index`, not simply one past the maximum.
                let mut i = s
                    .options(&self.global_options)
                    .get("base-index")
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(0);
                while s.windows.iter().any(|w| w.index == i) {
                    i += 1;
                }
                i
            }
        };

        // tmux's `default_window_size`: a new window is sized from the clients
        // that can see it, or from the session's `default-size` when none can.
        let session_pos = session_pos.expect("session presence checked above");
        let (cols, rows) = self.default_window_size(session_pos, None, None);
        // Spawn the pane before mutating counters so a spawn failure leaves state
        // untouched; the id the pane will take is peeked for its environment.
        let argv = fill_spawn_ids(argv, self.next_pane_id, session_id);
        let refs: Vec<&str> = argv.iter().map(String::as_str).collect();
        let pane = Pane::spawn(&refs, cwd, cols, rows)?;

        let window_id = self.next_window_id;
        self.next_window_id += 1;
        let winlink_id = self.next_winlink_id;
        self.next_winlink_id += 1;
        let pane_id = self.next_pane_id;
        self.next_pane_id += 1;

        let session_pos = self
            .sessions
            .iter()
            .position(|s| s.id == session_id)
            .expect("session presence checked above");
        if replace {
            if let Some(pos) = self.sessions[session_pos]
                .windows
                .iter()
                .position(|w| w.index == index)
            {
                self.remove_link(session_pos, pos);
            }
        }
        // Insert keeping the window list sorted by index.
        let pos = self.sessions[session_pos]
            .windows
            .iter()
            .take_while(|w| w.index < index)
            .count();
        self.windows.insert(
            window_id,
            Window {
                id: window_id,
                // tmux names a window before it announces the link, so
                // `window-linked` already carries `hook_window_name`.
                name: name.to_string(),
                panes: vec![PaneNode {
                    id: pane_id,
                    pane,
                    start_command: String::new(),
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
                activity_micros: now_micros(),
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
        self.insert_link(
            session_pos,
            pos,
            Winlink {
                link_id: winlink_id,
                index,
                id: window_id,
                alert_flags: 0,
            },
            select,
        );
        self.remove_unlinked_windows();
        let reason = if select || !had_windows {
            RenderInvalidation::LAYOUT | RenderInvalidation::STATUS
        } else {
            RenderInvalidation::STATUS
        };
        self.invalidate_session(session_id, reason);
        Ok(pos)
    }

    /// `new-window -a`/`-b`: open a new window *relative* to an existing anchor
    /// window (tmux's `winlink_shuffle_up`). `anchor_index` is the client-visible
    /// index of an existing window in `session`; `after` selects `-a` (insert just
    /// after the anchor) vs `-b` (insert at the anchor's own index, pushing it up).
    ///
    /// The new window's desired index is `anchor_index + 1` (`-a`) or
    /// `anchor_index` (`-b`). tmux then finds the first *free* index at or above
    /// that desired index and shifts the contiguous run of occupied windows in
    /// `[desired, free)` up by one to open the slot — windows past the first gap
    /// keep their indices. The new window becomes the session's active window and
    /// the previous active window becomes "last". Returns the new window's `Vec`
    /// position. Errors (`can't find session`/`can't find window`) mirror tmux.
    pub fn new_window_relative(
        &mut self,
        session: &str,
        anchor_index: u32,
        after: bool,
        select: bool,
    ) -> io::Result<usize> {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
        let name = super::default_window_name(std::slice::from_ref(&shell));
        self.new_window_relative_with_spawn(
            session,
            anchor_index,
            after,
            select,
            &[shell],
            None,
            &name,
        )
    }

    pub(crate) fn new_window_relative_with_spawn(
        &mut self,
        session: &str,
        anchor_index: u32,
        after: bool,
        select: bool,
        argv: &[String],
        cwd: Option<&Path>,
        name: &str,
    ) -> io::Result<usize> {
        let session_pos = self.session_index(session).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("can't find session: {session}"),
            )
        })?;
        let s = &self.sessions[session_pos];
        let session_id = s.id;
        if !s.windows.iter().any(|w| w.index == anchor_index) {
            return Err(window_not_found(&anchor_index.to_string()));
        }
        let desired = if after {
            anchor_index + 1
        } else {
            anchor_index
        };
        // First free index at or above `desired` (end of the contiguous run to shift).
        let mut free = desired;
        while s.windows.iter().any(|w| w.index == free) {
            free += 1;
        }

        // tmux's `default_window_size`, as for an appended window. Spawn before
        // mutating counters so a failure leaves state untouched; the id the pane
        // will take is peeked for its environment.
        let (cols, rows) = self.default_window_size(session_pos, None, None);
        let argv = fill_spawn_ids(argv, self.next_pane_id, session_id);
        let refs: Vec<&str> = argv.iter().map(String::as_str).collect();
        let pane = Pane::spawn(&refs, cwd, cols, rows)?;

        let window_id = self.next_window_id;
        self.next_window_id += 1;
        let winlink_id = self.next_winlink_id;
        self.next_winlink_id += 1;
        let pane_id = self.next_pane_id;
        self.next_pane_id += 1;

        let session_pos = self
            .sessions
            .iter()
            .position(|s| s.id == session_id)
            .expect("session presence checked above");
        // Shift the contiguous occupied run [desired, free) up by one to open the
        // `desired` slot. This preserves the sorted-by-index invariant (a monotone
        // shift of a contiguous prefix of indices keeps their relative order).
        for win in self.links_mut(session_pos).iter_mut() {
            if win.index >= desired && win.index < free {
                win.index += 1;
            }
        }
        let pos = self.sessions[session_pos]
            .windows
            .iter()
            .take_while(|w| w.index < desired)
            .count();
        self.windows.insert(
            window_id,
            Window {
                id: window_id,
                // tmux names a window before it announces the link, so
                // `window-linked` already carries `hook_window_name`.
                name: name.to_string(),
                panes: vec![PaneNode {
                    id: pane_id,
                    pane,
                    start_command: String::new(),
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
                activity_micros: now_micros(),
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
        self.insert_link(
            session_pos,
            pos,
            Winlink {
                link_id: winlink_id,
                index: desired,
                id: window_id,
                alert_flags: 0,
            },
            select,
        );
        self.invalidate_session(
            session_id,
            if select {
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS
            } else {
                RenderInvalidation::STATUS
            },
        );
        Ok(pos)
    }

    fn set_window_name(
        &mut self,
        target: &str,
        name: &str,
        disable_automatic_rename: bool,
    ) -> io::Result<()> {
        let t = self.resolve_window_target(target)?;
        let session_id = self.sessions[t.session].id;
        let window = self.window_mut(t.session, t.window);
        let window_id = window.id;
        let name_changed = window.name != name;
        let rename_changed =
            disable_automatic_rename && window.options.get("automatic-rename") != Some("off");
        if name_changed {
            window.name = name.to_string();
        }
        if disable_automatic_rename {
            window.options.set("automatic-rename", "off");
        }
        if name_changed || rename_changed {
            self.invalidate_session(session_id, RenderInvalidation::STATUS);
        }
        // tmux's `window_set_name` notifies for every store, so renaming a
        // window to the name it already has raises the hook again. The
        // automatic-rename pass compares before it gets here, which is where
        // tmux's own unchanged-name filter lives too.
        self.notify_window(WindowHook::Renamed, window_id);
        Ok(())
    }

    /// Rename a window (`rename-window`). `target` is `session[:window]`; a
    /// missing window part means the session's active window. Errors if the
    /// target can't be resolved (`can't find window`). Like tmux, an explicit
    /// name disables automatic renaming for the window.
    pub fn rename_window(&mut self, target: &str, name: &str) -> io::Result<()> {
        self.set_window_name(target, name, true)
    }

    /// Store a name produced by `automatic-rename-format` without disabling
    /// subsequent automatic updates.
    pub(crate) fn rename_window_automatically(
        &mut self,
        target: &str,
        name: &str,
    ) -> io::Result<()> {
        self.set_window_name(target, name, false)
    }

    /// Destroy a physical window and every winlink to it. tmux's `kill-window`
    /// is global to the window rather than an unlink from only the target
    /// session.
    pub(super) fn destroy_window_id(&mut self, window_id: u32) {
        let affected = self
            .sessions
            .iter()
            .filter(|session| session.windows.iter().any(|link| link.id == window_id))
            .map(|session| session.id)
            .collect::<Vec<_>>();
        let link_sets = self
            .sessions
            .iter()
            .filter(|session| session.windows.iter().any(|link| link.id == window_id))
            .map(|session| session.link_set_id)
            .collect::<BTreeSet<_>>();
        for link_set_id in link_sets {
            let Some(member) = self
                .sessions
                .iter()
                .position(|session| session.link_set_id == link_set_id)
            else {
                continue;
            };
            let links = self.sessions[member]
                .windows
                .iter()
                .copied()
                .filter(|link| link.id != window_id)
                .collect();
            self.replace_link_set(member, links);
        }
        // Named while the window is still known, but reported per session it
        // was linked into, exactly as tmux's `session_detach` does.
        for session_id in &affected {
            self.notify_session_window(SessionHook::WindowUnlinked, *session_id, window_id);
        }
        self.sessions.retain(|session| !session.windows.is_empty());
        self.windows.remove(&window_id);
        self.remove_unlinked_windows();
        self.renumber_affected_sessions(&affected);
        for session_id in affected {
            if self.sessions.iter().any(|session| session.id == session_id) {
                self.invalidate_session(
                    session_id,
                    RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
                );
            } else {
                self.invalidate_session(session_id, RenderInvalidation::SESSION_GONE);
            }
        }
    }

    /// `kill-window target`: destroy the target window everywhere it is linked.
    pub fn kill_window(&mut self, target: &str) -> io::Result<()> {
        let t = self.resolve_window_target(target)?;
        let window_id = self.sessions[t.session].windows[t.window].id;
        self.destroy_window_id(window_id);
        Ok(())
    }

    /// `kill-window -a [-t target]`: kill every window in the target's session
    /// *except* the target itself, matching tmux. The survivor becomes the
    /// session's only (and active) window.
    pub fn kill_other_windows(&mut self, target: &str) -> io::Result<()> {
        let t = self.resolve_window_target(target)?;
        // A session holding one winlink has nothing to kill, and tmux returns
        // before its renumber pass in that case.
        if self.sessions[t.session].windows.len() < 2 {
            return Ok(());
        }
        let keep_id = self.sessions[t.session].windows[t.window].id;
        let links = &self.sessions[t.session].windows;
        let mut destroy = links
            .iter()
            .filter_map(|link| (link.id != keep_id).then_some(link.id))
            .collect::<BTreeSet<_>>();
        if links.iter().filter(|link| link.id == keep_id).count() > 1 {
            destroy.insert(keep_id);
        }
        for window_id in destroy {
            self.destroy_window_id(window_id);
        }
        // tmux finishes the bulk kill with `server_renumber_all`, so every
        // session that asked for renumbering is renumbered, not only the ones
        // that lost a window.
        self.renumber_all_sessions();
        Ok(())
    }

    /// tmux's `server_renumber_all`: renumber every session whose
    /// `renumber-windows` option is on.
    fn renumber_all_sessions(&mut self) {
        let session_ids = self
            .sessions
            .iter()
            .map(|session| session.id)
            .collect::<Vec<_>>();
        self.renumber_affected_sessions(&session_ids);
    }

    /// `unlink-window`: remove one logical winlink from its session (and thus
    /// the synchronized position from every member of its group). `-k` only
    /// bypasses the singly-linked guard; it does not kill links in other
    /// sessions.
    pub fn unlink_window(&mut self, target: &str, kill: bool) -> io::Result<()> {
        let t = self.resolve_window_target(target)?;
        let link_set_id = self.sessions[t.session].link_set_id;
        let window_id = self.sessions[t.session].windows[t.window].id;
        let group_members = self
            .sessions
            .iter()
            .filter(|session| session.link_set_id == link_set_id)
            .count();
        if !kill && self.window_reference_count(window_id) == group_members {
            return Err(io::Error::other("window only linked to one session"));
        }
        let affected = self
            .sessions
            .iter()
            .filter(|session| session.link_set_id == link_set_id)
            .map(|session| session.id)
            .collect::<Vec<_>>();
        self.remove_link(t.session, t.window);
        if self
            .sessions
            .iter()
            .find(|session| session.link_set_id == link_set_id)
            .is_some_and(|session| session.windows.is_empty())
        {
            self.sessions
                .retain(|session| session.link_set_id != link_set_id);
        }
        self.remove_unlinked_windows();
        self.renumber_affected_sessions(&affected);
        for session_id in affected {
            if self.sessions.iter().any(|session| session.id == session_id) {
                self.invalidate_session(
                    session_id,
                    RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
                );
            } else {
                self.invalidate_session(session_id, RenderInvalidation::SESSION_GONE);
            }
        }
        Ok(())
    }

    /// `select-window target`: make the target window active (recording the
    /// previous active window as "last").
    pub fn select_window(&mut self, target: &str) -> io::Result<()> {
        let t = self.resolve_window_target(target)?;
        let session_id = self.sessions[t.session].id;
        if self.sessions[t.session].active != t.window {
            self.select_session_window(t.session, t.window);
            self.recalculate_sizes()?;
            self.invalidate_session(
                session_id,
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
            );
        }
        Ok(())
    }

    /// `next-window`/`previous-window`: cycle the active window within a session.
    /// A single-window session has no next/previous window, matching tmux's
    /// `no next window` / `no previous window` errors.
    pub fn next_window(&mut self, session: &str) -> io::Result<()> {
        self.cycle_window(session, true)
    }

    pub fn previous_window(&mut self, session: &str) -> io::Result<()> {
        self.cycle_window(session, false)
    }

    /// `next-window -a` / `previous-window -a`: step to the next/previous window
    /// carrying a bell, activity, or silence alert.
    pub fn next_window_alert(&mut self, session: &str) -> io::Result<()> {
        self.cycle_window_alert(session, true)
    }

    pub fn previous_window_alert(&mut self, session: &str) -> io::Result<()> {
        self.cycle_window_alert(session, false)
    }

    fn cycle_window_alert(&mut self, session: &str, forward: bool) -> io::Result<()> {
        let session_pos = self.session_pos(session).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("can't find session: {session}"),
            )
        })?;
        let (active, len) = {
            let session = &self.sessions[session_pos];
            (session.active, session.windows.len())
        };
        for distance in 1..len {
            let position = if forward {
                (active + distance) % len
            } else {
                (active + len - distance) % len
            };
            if self.sessions[session_pos].windows[position].alert_flags != 0 {
                let session_id = self.sessions[session_pos].id;
                self.select_session_window(session_pos, position);
                self.recalculate_sizes()?;
                self.invalidate_session(
                    session_id,
                    RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
                );
                return Ok(());
            }
        }
        let which = if forward { "next" } else { "previous" };
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("no {which} window"),
        ))
    }

    pub(crate) fn clear_session_alerts(&mut self, session: &str) -> io::Result<()> {
        let session_pos = self.session_pos(session).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("can't find session: {session}"),
            )
        })?;
        for link in &mut self.sessions[session_pos].windows {
            link.alert_flags = 0;
        }
        let session_id = self.sessions[session_pos].id;
        self.invalidate_session(session_id, RenderInvalidation::STATUS);
        Ok(())
    }

    /// tmux's `alerts_reset_all`, which `options_push_changes` runs whenever
    /// `monitor-silence` is set. Every window drops its raised silence
    /// condition and restarts its timer from the option change, so a window
    /// that has been quiet since before the change is not alerted for the
    /// silence that preceded it.
    pub(super) fn reset_silence_timers(&mut self) {
        let now = std::time::Instant::now();
        for window in self.windows.values_mut() {
            window.pending_alerts &= !ALERT_SILENCE;
        }
        for window_id in self.windows.keys().copied().collect::<Vec<_>>() {
            self.window_last_activity.insert(window_id, now);
        }
        self.silence_alerted.clear();
    }

    pub(crate) fn alert_poll_timeout(&self) -> Option<std::time::Duration> {
        self.windows
            .values()
            .any(|window| {
                window
                    .options(&self.global_options)
                    .get("monitor-silence")
                    .and_then(|value| value.parse::<u64>().ok())
                    .is_some_and(|seconds| seconds != 0)
            })
            .then(|| std::time::Duration::from_millis(100))
    }

    pub(crate) fn refresh_alerts(&mut self, now: std::time::Instant) -> bool {
        struct WindowActivity {
            id: u32,
            output: bool,
            bell: bool,
            last_output: Option<std::time::Instant>,
            monitor_bell: bool,
            monitor_activity: bool,
            monitor_silence: u64,
        }

        let live_panes = self
            .windows
            .values()
            .flat_map(|window| &window.panes)
            .map(|pane| pane.id)
            .collect::<BTreeSet<_>>();
        self.pane_alert_seen
            .retain(|pane_id, _| live_panes.contains(pane_id));

        let mut activities = Vec::new();
        let mut unseen_changes = Vec::new();
        for window in self.windows.values() {
            let options = window.options(&self.global_options);
            let monitor_bell = options.get("monitor-bell") == Some("on");
            let monitor_activity = options.get("monitor-activity") == Some("on");
            let monitor_silence = options
                .get("monitor-silence")
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(0);
            let mut output = false;
            let mut bell = false;
            let mut last_output = None;
            for pane in &window.panes {
                let (revision, bells, at) = pane.pane.observation_state().alert_snapshot();
                let previous = self.pane_alert_seen.insert(pane.id, (revision, bells));
                let (previous_revision, previous_bells) = previous.unwrap_or((0, 0));
                let pane_output = revision > previous_revision;
                // tmux flags the pane itself when output lands while a mode is
                // showing a frozen grid; the flag is not about the window's
                // activity condition, which is why it is per pane.
                if pane_output && pane.mode.is_some() {
                    unseen_changes.push(pane.id);
                }
                output |= pane_output;
                bell |= bells > previous_bells;
                if at > last_output {
                    last_output = at;
                }
            }
            activities.push(WindowActivity {
                id: window.id,
                output,
                bell,
                last_output,
                monitor_bell,
                monitor_activity,
                monitor_silence,
            });
        }

        // Leaving the last mode drops the flag, exactly as tmux's
        // `window_pane_reset_mode` does.
        for pane in self.windows.values_mut().flat_map(|w| &mut w.panes) {
            pane.unseen_changes =
                pane.mode.is_some() && (pane.unseen_changes || unseen_changes.contains(&pane.id));
        }

        // Windows whose whole condition set is re-examined this pass because a
        // client took their session (tmux's `alerts_check_session`).
        let sessions_to_check = std::mem::take(&mut self.alert_check_sessions);
        let recheck = self
            .sessions
            .iter()
            .filter(|session| sessions_to_check.contains(&session.id))
            .flat_map(|session| session.windows.iter().map(|link| link.id))
            .collect::<BTreeSet<_>>();

        // Record each window's raised conditions, exactly as tmux's
        // `alerts_queue` sets `WINDOW_*` flags before its check runs.
        let mut deliveries = Vec::new();
        for activity in activities {
            if activity.output {
                self.window_last_activity
                    .insert(activity.id, activity.last_output.unwrap_or(now));
                self.silence_alerted.remove(&activity.id);
            } else {
                self.window_last_activity.entry(activity.id).or_insert(now);
            }
            let silence = activity.monitor_silence != 0
                && !self.silence_alerted.contains(&activity.id)
                && self
                    .window_last_activity
                    .get(&activity.id)
                    .is_some_and(|last| {
                        now.saturating_duration_since(*last)
                            >= std::time::Duration::from_secs(activity.monitor_silence)
                    });
            if silence {
                self.silence_alerted.insert(activity.id);
            }
            let Some(window) = self.windows.get_mut(&activity.id) else {
                continue;
            };
            if activity.bell {
                window.pending_alerts |= ALERT_BELL;
            }
            if activity.output {
                window.pending_alerts |= ALERT_ACTIVITY;
            }
            if silence {
                window.pending_alerts |= ALERT_SILENCE;
            }
            // tmux checks bell, then activity, then silence, and only clears
            // the condition whose monitor is on. A condition is examined when
            // something queues it — new output, a bell, the silence timer — or
            // when a client attaches and runs `alerts_check_session`; a
            // monitor merely being switched on does not, which is what leaves
            // creation-time activity pending until the first client arrives.
            let rechecked = recheck.contains(&activity.id);
            for (bit, monitored, raised) in [
                (ALERT_BELL, activity.monitor_bell, activity.bell),
                (ALERT_ACTIVITY, activity.monitor_activity, activity.output),
                (ALERT_SILENCE, activity.monitor_silence != 0, silence),
            ] {
                if monitored && (raised || rechecked) && window.pending_alerts & bit != 0 {
                    window.pending_alerts &= !bit;
                    deliveries.push((activity.id, bit));
                }
            }
        }

        let mut changed = false;
        for (window_id, bit) in deliveries {
            changed |= self.deliver_alert(window_id, bit);
        }
        let live_windows = self.windows.keys().copied().collect::<BTreeSet<_>>();
        self.window_last_activity
            .retain(|window_id, _| live_windows.contains(window_id));
        self.silence_alerted
            .retain(|window_id| live_windows.contains(window_id));
        changed
    }

    /// Write `data` to the terminal selection of every client showing this
    /// window, which is the client walk `tty_write` does for any other pane
    /// output.
    pub(super) fn write_window_selection(&self, window_id: u32, data: Vec<u8>) {
        let sessions = self
            .sessions
            .iter()
            .filter(|session| {
                session
                    .windows
                    .get(session.active)
                    .is_some_and(|link| link.id == window_id)
            })
            .map(|session| session.id)
            .collect::<BTreeSet<_>>();
        for client in self
            .client_snapshots()
            .into_iter()
            .filter(|client| !client.control_mode && sessions.contains(&client.session_id))
        {
            self.set_client_selection(Some(&client.name), None, Some(data.clone()));
        }
    }

    /// Apply the `allow-rename` policy to the `ESC k … ST` renames panes
    /// emitted since the last pass, mirroring tmux's `input_exit_rename`.
    ///
    /// The option decides a rename once, when it is drained, and a refused
    /// rename is dropped: turning `allow-rename` on later does not replay a
    /// name the pane sent while it was off. An empty payload is tmux's request
    /// to fall back to the automatic name, which hmux resolves from the pane's
    /// command whenever `automatic-rename` is on, so there is nothing to store.
    pub(crate) fn process_pane_renames(&mut self) {
        let mut renamed = Vec::new();
        for window in self.windows.values() {
            for node in &window.panes {
                let allowed = node
                    .options(window, &self.global_options)
                    .get("allow-rename")
                    .is_some_and(|value| value == "on" || value == "1");
                // Drained either way: a refused rename must not outlive the
                // option that refused it.
                let queued = node.pane.observation_state().take_renames();
                let Some(name) = queued.into_iter().next_back() else {
                    continue;
                };
                if allowed && !name.is_empty() {
                    renamed.push((window.id, name));
                }
            }
        }
        for (window_id, name) in renamed {
            // tmux also clears `automatic-rename` here; hmux does not, which is
            // the difference gap07's rename test records.
            let _ = self.set_window_name(&format!("@{window_id}"), &name, false);
        }
    }

    /// tmux's `check_window_name`, run once per server loop: re-derive the name
    /// of every `automatic-rename` window whose active pane has changed since
    /// the last pass, and store it.
    ///
    /// The name is stored rather than resolved whenever a format asks for it,
    /// because that is what makes it lag: a pane that execs a different command
    /// without printing anything keeps the name it already had, since nothing
    /// tells the server to look again. One re-derivation per `NAME_INTERVAL`
    /// keeps a chatty pane from walking `/proc` on every pass.
    pub(crate) fn process_window_names(&mut self) {
        /// tmux's `NAME_INTERVAL`.
        const NAME_INTERVAL_MICROS: i64 = 500_000;
        let now = now_micros();
        let mut pending = Vec::new();
        for window in self.windows.values() {
            let options = window.options(&self.global_options);
            if !options
                .get("automatic-rename")
                .is_some_and(|value| value == "on" || value == "1")
            {
                continue;
            }
            let Some(node) = window.panes.get(window.active) else {
                continue;
            };
            let observation = node.pane.observation_state();
            let in_mode = node.mode.is_some();
            if (!observation.changed() && in_mode == window.name_in_mode)
                || now.saturating_sub(window.name_time_micros) < NAME_INTERVAL_MICROS
            {
                continue;
            }
            pending.push((
                window.id,
                observation,
                options
                    .get("automatic-rename-format")
                    .unwrap_or("#{pane_current_command}")
                    .to_string(),
                node.pane.process_probe(),
                in_mode,
                window.name.clone(),
            ));
        }
        for (window_id, observation, source, probe, in_mode, current) in pending {
            observation.clear_changed();
            if let Some(window) = self.windows.get_mut(&window_id) {
                window.name_time_micros = now;
                window.name_in_mode = in_mode;
            }
            let mut vars = crate::server::format::Vars::new();
            vars.set(
                "pane_current_command",
                probe
                    .as_ref()
                    .and_then(crate::server::pane::PaneProcessProbe::current_command)
                    .unwrap_or_default(),
            )
            .set("pane_in_mode", if in_mode { "1" } else { "0" })
            .set("pane_dead", "0");
            let name = crate::server::format::expand(&source, &vars);
            // An empty name would blank the window; tmux's `format_window_name`
            // cannot produce one, and hmux keeps the name it has instead.
            if name.is_empty() || name == current {
                continue;
            }
            let _ = self.rename_window_automatically(&format!("@{window_id}"), &name);
        }
    }

    /// tmux's `alerts_check_bell`/`_activity`/`_silence` body: flag every
    /// winlink of the window, then let the session's `*-action` decide whether
    /// the hook and the user-visible notification follow.
    fn deliver_alert(&mut self, window_id: u32, bit: u8) -> bool {
        let (label, action_option, visual_option, hook) = match bit {
            ALERT_BELL => (
                "Bell",
                "bell-action",
                "visual-bell",
                SessionHook::AlertBell,
            ),
            ALERT_ACTIVITY => (
                "Activity",
                "activity-action",
                "visual-activity",
                SessionHook::AlertActivity,
            ),
            _ => (
                "Silence",
                "silence-action",
                "visual-silence",
                SessionHook::AlertSilence,
            ),
        };
        let attached = self.attached_session_ids();
        let links = self
            .sessions
            .iter()
            .enumerate()
            .flat_map(|(session_index, session)| {
                session
                    .windows
                    .iter()
                    .enumerate()
                    .filter(|(_, link)| link.id == window_id)
                    .map(move |(link_index, _)| (session_index, link_index))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let mut alerted_sessions = BTreeSet::new();
        let mut changed = false;
        for (session_index, link_index) in links {
            let session_id = self.sessions[session_index].id;
            let is_current = self.sessions[session_index].active == link_index;
            let is_attached = attached.contains(&session_id);
            // A bell is announced even on an already-flagged winlink; the
            // other two are not.
            if bit != ALERT_BELL
                && self.sessions[session_index].windows[link_index].alert_flags & bit != 0
            {
                continue;
            }
            if !is_current || !is_attached {
                let link = &mut self.sessions[session_index].windows[link_index];
                if link.alert_flags & bit == 0 {
                    link.alert_flags |= bit;
                    changed = true;
                    self.invalidate_session(session_id, RenderInvalidation::STATUS);
                }
            }
            let applies = match self.sessions[session_index]
                .options(&self.global_options)
                .get(action_option)
                .unwrap_or("other")
            {
                "any" => true,
                "current" => is_current,
                "other" => !is_current,
                _ => false,
            };
            if !applies {
                continue;
            }
            self.notify_session_window(hook, session_id, window_id);
            // One visual notification per session, however many of its
            // winlinks alerted.
            if !alerted_sessions.insert(session_id) {
                continue;
            }
            self.announce_alert(session_index, link_index, label, visual_option);
        }
        changed
    }

    /// tmux's `alerts_set_message`: send each non-control client of the session
    /// a bell, a status message, or both, per `visual-*`.
    fn announce_alert(
        &mut self,
        session_index: usize,
        link_index: usize,
        label: &str,
        visual_option: &str,
    ) {
        let session = &self.sessions[session_index];
        let session_id = session.id;
        let options = session.options(&self.global_options);
        let visual = options.get(visual_option).unwrap_or("off").to_owned();
        let duration_ms = options
            .get("display-time")
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(750);
        let bell = visual != "on";
        let text = (visual != "off").then(|| {
            if session.active == link_index {
                format!("{label} in current window")
            } else {
                format!("{label} in window {}", session.windows[link_index].index)
            }
        });
        self.client_renders
            .announce_alert(session_id, bell, text, duration_ms);
    }

    fn cycle_window(&mut self, session: &str, forward: bool) -> io::Result<()> {
        let pos = self.session_pos(session).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("can't find session: {session}"),
            )
        })?;
        let session_id = self.sessions[pos].id;
        let sess = &mut self.sessions[pos];
        let n = sess.windows.len();
        if n <= 1 {
            let which = if forward { "next" } else { "previous" };
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("no {which} window"),
            ));
        }
        let next = if forward {
            (sess.active + 1) % n
        } else {
            (sess.active + n - 1) % n
        };
        self.select_session_window(pos, next);
        self.recalculate_sizes()?;
        self.invalidate_session(
            session_id,
            RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
        );
        Ok(())
    }

    /// `last-window`: swap to the previously-active window. Without one (a
    /// single-window session, or a session never switched), tmux reports
    /// `no last window`.
    pub fn last_window(&mut self, session: &str) -> io::Result<()> {
        let pos = self.session_pos(session).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("can't find session: {session}"),
            )
        })?;
        let session_id = self.sessions[pos].id;
        let sess = &mut self.sessions[pos];
        match sess
            .last_windows
            .first()
            .copied()
            .and_then(|id| sess.windows.iter().position(|link| link.link_id == id))
        {
            Some(last) => {
                self.select_session_window(pos, last);
                self.recalculate_sizes()?;
                self.invalidate_session(
                    session_id,
                    RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
                );
                Ok(())
            }
            _ => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "no last window".to_string(),
            )),
        }
    }

    /// `swap-window -s src -t dst`: exchange the two windows' contents, keeping
    /// each window at its original index (tmux swaps the winlinks' windows). Both
    /// targets are `session[:window]`; a missing window part means the session's
    /// active window.
    pub fn swap_window(&mut self, src: &str, dst: &str, select: bool) -> io::Result<()> {
        let a = self.resolve_window_target(src)?;
        let b = self.resolve_window_target(dst)?;
        if a.session != b.session
            && self.sessions[a.session].link_set_id == self.sessions[b.session].link_set_id
        {
            return Err(io::Error::other("can't move window, sessions are grouped"));
        }
        let src_session_id = self.sessions[a.session].id;
        let dst_session_id = self.sessions[b.session].id;
        // tmux inverts the usual `-d` sense here: the plain form leaves every
        // session's current window untouched, and `-d` selects the destination
        // slot (and, for a cross-session swap, the source slot in its session).
        let src_index = self.sessions[a.session].windows[a.window].index;
        let dst_index = self.sessions[b.session].windows[b.window].index;
        if self.sessions[a.session].windows[a.window].id
            == self.sessions[b.session].windows[b.window].id
        {
            return Ok(());
        }
        if self.sessions[a.session].link_set_id == self.sessions[b.session].link_set_id {
            let mut links = self.sessions[a.session].windows.clone();
            let first = links[a.window].id;
            links[a.window].id = links[b.window].id;
            links[b.window].id = first;
            self.replace_link_set_preserving_positions(a.session, links);
            if select {
                Self::select_window_index(&mut self.sessions[a.session], dst_index);
            }
            self.invalidate_session(
                src_session_id,
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
            );
            return Ok(());
        } else {
            let mut a_links = self.sessions[a.session].windows.clone();
            let mut b_links = self.sessions[b.session].windows.clone();
            std::mem::swap(&mut a_links[a.window].id, &mut b_links[b.window].id);
            self.replace_link_set_preserving_positions(a.session, a_links);
            self.replace_link_set_preserving_positions(b.session, b_links);
            if select {
                Self::select_window_index(&mut self.sessions[b.session], dst_index);
                Self::select_window_index(&mut self.sessions[a.session], src_index);
            }
        }
        self.invalidate_session(
            src_session_id,
            RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
        );
        self.invalidate_session(
            dst_session_id,
            RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
        );
        Ok(())
    }

    /// Point a session's current window at the window occupying `index`,
    /// recording the outgoing window as its last-active. A no-op if no window
    /// sits at `index` or it is already current.
    fn select_window_index(sess: &mut Session, index: u32) {
        let Some(new_pos) = sess.windows.iter().position(|w| w.index == index) else {
            return;
        };
        Self::select_window_position(sess, new_pos);
    }

    /// `move-window -s src -t dst`: renumber the src window to the dst index
    /// (optionally into another session). Errors if the destination index is
    /// already in use (tmux's `index in use: N`), matching real
    /// tmux without `-k`.
    /// Move a window to `dst`. `select` follows tmux's default of making the
    /// moved window the destination session's current window; `-d` passes
    /// `false` to leave the current window where it is.
    pub fn move_window(&mut self, src: &str, dst: &str, select: bool) -> io::Result<()> {
        self.move_window_impl(src, dst, false, select)
    }

    /// Move a window, replacing an occupied destination for `move-window -k`.
    pub(crate) fn move_window_replacing(
        &mut self,
        src: &str,
        dst: &str,
        select: bool,
    ) -> io::Result<()> {
        self.move_window_impl(src, dst, true, select)
    }

    /// `move-window -a`/`-b`: relocate the source window relative to an anchor
    /// window in the destination session — `after` places it after the anchor
    /// (`-a`), else before it (`-b`). Mirrors tmux's `winlink_shuffle_up`: the
    /// contiguous occupied run at/above the desired index (the source counted as
    /// occupied) is shifted up by one to open the slot, then the source is
    /// relocated into it. When the anchor index does not name an existing window,
    /// tmux ignores `-a`/`-b` and moves to the explicit index (the plain path).
    pub fn move_window_relative(
        &mut self,
        src: &str,
        dst: &str,
        after: bool,
        select: bool,
    ) -> io::Result<()> {
        let s = self.resolve_window_target(src)?;
        let src_session = s.session;
        let src_session_id = self.sessions[src_session].id;
        let moved_link_id = self.sessions[src_session].windows[s.window].link_id;
        let (dst_sess_name, dst_idx) = parse_index_target(dst);
        let mut dst_session = match dst_sess_name {
            Some(name) => self.session_pos(name).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("can't find session: {name}"),
                )
            })?,
            None => src_session,
        };
        let dst_session_id = self.sessions[dst_session].id;
        if src_session != dst_session
            && self.sessions[src_session].link_set_id == self.sessions[dst_session].link_set_id
        {
            return Err(io::Error::other("sessions are grouped"));
        }
        let anchor = dst_idx.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "move-window: missing destination index".to_string(),
            )
        })?;
        // No such anchor window → tmux ignores the relative flag and moves to the
        // explicit index.
        if !self.sessions[dst_session]
            .windows
            .iter()
            .any(|w| w.index == anchor)
        {
            return self.move_window_impl(src, dst, false, select);
        }
        let desired = if after { anchor + 1 } else { anchor };
        // First free index at or above `desired` (end of the contiguous run to
        // shift). The source counts as occupied, matching tmux.
        let mut free = desired;
        while self.sessions[dst_session]
            .windows
            .iter()
            .any(|w| w.index == free)
        {
            free += 1;
        }
        let source_set = self.sessions[src_session].link_set_id;
        let destination_set = self.sessions[dst_session].link_set_id;
        if source_set == destination_set {
            let mut links = self.sessions[src_session].windows.clone();
            for link in &mut links {
                if link.index >= desired && link.index < free {
                    link.index += 1;
                }
            }
            let source = links
                .iter()
                .position(|link| link.link_id == moved_link_id)
                .expect("source window present after the shuffle");
            let mut link = links.remove(source);
            link.index = desired;
            links.push(link);
            links.sort_by_key(|link| link.index);
            self.replace_link_set(src_session, links);
        } else {
            let mut source_links = self.sessions[src_session].windows.clone();
            let source = source_links
                .iter()
                .position(|link| link.link_id == moved_link_id)
                .expect("source window present");
            let mut link = source_links.remove(source);
            link.index = desired;
            self.replace_link_set(src_session, source_links);

            let mut destination_links = self.sessions[dst_session].windows.clone();
            for existing in &mut destination_links {
                if existing.index >= desired && existing.index < free {
                    existing.index += 1;
                }
            }
            destination_links.push(link);
            destination_links.sort_by_key(|link| link.index);
            self.replace_link_set(dst_session, destination_links);

            if self.sessions[src_session].windows.is_empty() {
                self.sessions
                    .retain(|session| session.link_set_id != source_set);
                if src_session < dst_session {
                    dst_session -= 1;
                }
            }
        }
        if select {
            let sess = &mut self.sessions[dst_session];
            let new_pos = sess
                .windows
                .iter()
                .position(|w| w.link_id == moved_link_id)
                .expect("moved window is present in the destination");
            Self::select_window_position(sess, new_pos);
        }
        self.remove_unlinked_windows();
        if self
            .sessions
            .iter()
            .any(|session| session.id == src_session_id)
        {
            self.invalidate_session(
                src_session_id,
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
            );
        } else {
            self.invalidate_session(src_session_id, RenderInvalidation::SESSION_GONE);
        }
        if dst_session_id != src_session_id {
            self.invalidate_session(
                dst_session_id,
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
            );
        }
        Ok(())
    }

    fn move_window_impl(
        &mut self,
        src: &str,
        dst: &str,
        replace: bool,
        select: bool,
    ) -> io::Result<()> {
        let s = self.resolve_window_target(src)?;
        let src_session_id = self.sessions[s.session].id;
        let (dst_session, new_index) = self.window_destination(dst, s.session)?;
        let dst_session_id = self.sessions[dst_session].id;
        if s.session != dst_session
            && self.sessions[s.session].link_set_id == self.sessions[dst_session].link_set_id
        {
            return Err(io::Error::other("sessions are grouped"));
        }
        // A window already at the destination index blocks the move.
        let occupied = self.sessions[dst_session]
            .windows
            .iter()
            .any(|w| w.index == new_index);
        let same_slot = dst_session == s.session
            && self.sessions[s.session].windows[s.window].index == new_index;
        let source_window_id = self.sessions[s.session].windows[s.window].id;
        if let Some(destination) = self.sessions[dst_session]
            .windows
            .iter()
            .find(|link| link.index == new_index)
        {
            if destination.id == source_window_id {
                return Err(io::Error::other(format!("same index: {new_index}")));
            }
        }
        if occupied && !replace {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("index in use: {new_index}"),
            ));
        }
        let moved_link_id = self.sessions[s.session].windows[s.window].link_id;
        let source_set = self.sessions[s.session].link_set_id;
        let destination_set = self.sessions[dst_session].link_set_id;
        if source_set == destination_set {
            let mut links = self.sessions[s.session].windows.clone();
            if occupied && !same_slot {
                links.retain(|link| link.index != new_index);
            }
            let moved = links
                .iter_mut()
                .find(|link| link.link_id == moved_link_id)
                .expect("source window remains present");
            moved.index = new_index;
            links.sort_by_key(|link| link.index);
            self.replace_link_set(s.session, links);
        } else {
            let mut source_links = self.sessions[s.session].windows.clone();
            let source_position = source_links
                .iter()
                .position(|link| link.link_id == moved_link_id)
                .expect("source window present");
            let mut moved = source_links.remove(source_position);
            moved.index = new_index;
            self.replace_link_set(s.session, source_links);

            let mut destination_links = self.sessions[dst_session].windows.clone();
            if occupied && !same_slot {
                destination_links.retain(|link| link.index != new_index);
            }
            destination_links.push(moved);
            destination_links.sort_by_key(|link| link.index);
            self.replace_link_set(dst_session, destination_links);
            if self.sessions[s.session].windows.is_empty() {
                self.sessions
                    .retain(|session| session.link_set_id != source_set);
            }
        }
        let dst_session = self
            .sessions
            .iter()
            .position(|session| session.id == dst_session_id)
            .expect("destination session remains present");
        if select {
            let sess = &mut self.sessions[dst_session];
            let new_pos = sess
                .windows
                .iter()
                .position(|w| w.link_id == moved_link_id)
                .expect("moved window is present in the destination");
            Self::select_window_position(sess, new_pos);
        }
        self.remove_unlinked_windows();
        if self
            .sessions
            .iter()
            .any(|session| session.id == src_session_id)
        {
            self.invalidate_session(
                src_session_id,
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
            );
        } else {
            self.invalidate_session(src_session_id, RenderInvalidation::SESSION_GONE);
        }
        if dst_session_id != src_session_id {
            self.invalidate_session(
                dst_session_id,
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
            );
        }
        Ok(())
    }

    /// `move-window -r`: renumber every window in `session` to consecutive
    /// indices starting at `base-index`, in their current order, closing any
    /// gaps. Errors if the session is missing (`can't find session`).
    pub fn renumber_windows(&mut self, session: &str) -> io::Result<()> {
        let pos = self.session_pos(session).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("can't find session: {session}"),
            )
        })?;
        self.renumber_windows_at(pos);
        Ok(())
    }

    fn renumber_affected_sessions(&mut self, session_ids: &[u32]) {
        let mut link_sets = BTreeSet::new();
        for session_id in session_ids {
            let Some(pos) = self
                .sessions
                .iter()
                .position(|session| session.id == *session_id)
            else {
                continue;
            };
            if !link_sets.insert(self.sessions[pos].link_set_id) {
                continue;
            }
            let enabled = self.sessions[pos]
                .options(&self.global_options)
                .get("renumber-windows")
                == Some("on");
            if enabled {
                self.renumber_windows_at(pos);
            }
        }
    }

    fn renumber_windows_at(&mut self, pos: usize) {
        let base = self.sessions[pos]
            .options(&self.global_options)
            .get("base-index")
            .and_then(|value| value.parse().ok())
            .unwrap_or(0);
        let session_id = self.sessions[pos].id;
        let active_id = self.sessions[pos]
            .windows
            .get(self.sessions[pos].active)
            .map(|link| link.link_id);
        let mut links = self.sessions[pos].windows.clone();
        // Winlinks are kept sorted by index, so position order is index order.
        for (i, win) in links.iter_mut().enumerate() {
            win.index = base + i as u32;
        }
        self.sessions[pos].windows = links;
        self.sessions[pos].active = active_id
            .and_then(|id| {
                self.sessions[pos]
                    .windows
                    .iter()
                    .position(|link| link.link_id == id)
            })
            .unwrap_or(0);
        Self::refresh_last_window(&mut self.sessions[pos]);
        self.invalidate_session(session_id, RenderInvalidation::STATUS);
    }

    /// `rotate-window [-t target]`: rotate the target window's panes upward (each
    /// pane moves to the previous position; the first wraps to the last). Real
    /// tmux keeps the active pane at the *same index* — the panes rotate under the
    /// cursor — so the pane content selected changes. Default target is the
    /// current window.
    pub fn rotate_window(&mut self, target: &str) -> io::Result<()> {
        self.rotate_window_direction(target, false)
    }

    /// `rotate-window -D [-t target]`: rotate the target window's panes downward,
    /// likewise keeping the active pane at the same index.
    pub(in crate::server) fn rotate_window_down(&mut self, target: &str) -> io::Result<()> {
        self.rotate_window_direction(target, true)
    }

    fn rotate_window_direction(&mut self, target: &str, down: bool) -> io::Result<()> {
        let t = self.resolve_window_target(target)?;
        let session_id = self.sessions[t.session].id;
        let win = self.window_mut(t.session, t.window);
        let n = win.panes.len();
        if n > 1 {
            // tmux rotates the pane *contents* while the active pane stays at its
            // current index (`win.active` is left unchanged): the cursor holds
            // still and the panes move beneath it.
            if down {
                win.panes.rotate_right(1);
            } else {
                win.panes.rotate_left(1);
            }
            let mut pane_ids = win.panes.iter().map(|pane| pane.id);
            win.layout.assign_panes(&mut pane_ids);
            resize_panes_to_layout(win)?;
            self.invalidate_session(session_id, RenderInvalidation::LAYOUT);
        }
        Ok(())
    }

    /// `link-window -s src -t dst`: make the source window appear in the
    /// destination session at the destination index. Both winlinks reference
    /// the same globally owned window.
    /// Link the source window into the destination session at the destination
    /// index. `select` follows tmux's default of making the linked window the
    /// destination session's current window (the previously-active window becomes
    /// its "last"); `-d` passes `false` to leave the current window where it is.
    pub fn link_window(
        &mut self,
        src: &str,
        dst: &str,
        kill: bool,
        select: bool,
    ) -> io::Result<()> {
        let s = self.resolve_window_target(src)?;
        let source_window_id = self.sessions[s.session].windows[s.window].id;
        let (dst_session, index) = self.window_destination(dst, s.session)?;
        if s.session != dst_session
            && self.sessions[s.session].link_set_id == self.sessions[dst_session].link_set_id
        {
            return Err(io::Error::other("sessions are grouped"));
        }
        if let Some(pos) = self.sessions[dst_session]
            .windows
            .iter()
            .position(|w| w.index == index)
        {
            if self.sessions[dst_session].windows[pos].id == source_window_id {
                return Err(io::Error::other(format!("same index: {index}")));
            }
            if kill {
                self.remove_link(dst_session, pos);
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!("index in use: {index}"),
                ));
            }
        }
        self.link_window_at(dst_session, index, source_window_id, select)
    }

    /// Resolve a plain link/move destination. An omitted index selects the
    /// lowest free index at or above `base-index`, matching tmux's `session:`
    /// destination behavior.
    fn window_destination(
        &self,
        target: &str,
        fallback_session: usize,
    ) -> io::Result<(usize, u32)> {
        let (session_name, requested_index) = parse_index_target(target);
        let session = match session_name {
            Some(name) => self.session_pos(name).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("can't find session: {name}"),
                )
            })?,
            None => fallback_session,
        };
        let index = requested_index.unwrap_or_else(|| {
            let destination = &self.sessions[session];
            let mut index = destination
                .options(&self.global_options)
                .get("base-index")
                .and_then(|value| value.parse().ok())
                .unwrap_or(0);
            while destination.windows.iter().any(|link| link.index == index) {
                index += 1;
            }
            index
        });
        Ok((session, index))
    }

    /// `link-window -a`/`-b`: link the source window relative to an anchor window
    /// in the destination session — `after` links it after the anchor (`-a`), else
    /// at the anchor's own index (`-b`). Mirrors tmux's `winlink_shuffle_up`: the
    /// contiguous occupied run at/above the desired index is shifted up by one to
    /// open the slot, then the new link is added there. Unlike `move-window -a`, the
    /// source keeps its own slot (this is a link, not a move). When the anchor index
    /// does not name an existing window, tmux ignores `-a`/`-b` and links at the
    /// explicit index (the plain path).
    pub fn link_window_relative(
        &mut self,
        src: &str,
        dst: &str,
        after: bool,
        select: bool,
    ) -> io::Result<()> {
        let s = self.resolve_window_target(src)?;
        let source_window_id = self.sessions[s.session].windows[s.window].id;
        let (dst_sess_name, dst_idx) = parse_index_target(dst);
        let dst_session = match dst_sess_name {
            Some(name) => self.session_pos(name).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("can't find session: {name}"),
                )
            })?,
            None => s.session,
        };
        if s.session != dst_session
            && self.sessions[s.session].link_set_id == self.sessions[dst_session].link_set_id
        {
            return Err(io::Error::other("sessions are grouped"));
        }
        let anchor = dst_idx.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "link-window: missing destination index".to_string(),
            )
        })?;
        // No such anchor window → tmux ignores the relative flag and links at the
        // explicit index.
        if !self.sessions[dst_session]
            .windows
            .iter()
            .any(|w| w.index == anchor)
        {
            return self.link_window(src, dst, false, select);
        }
        let desired = if after { anchor + 1 } else { anchor };
        // First free index at or above `desired` (end of the contiguous run to
        // shift).
        let mut free = desired;
        while self.sessions[dst_session]
            .windows
            .iter()
            .any(|w| w.index == free)
        {
            free += 1;
        }
        // Shift the contiguous occupied run [desired, free) up by one to open the
        // `desired` slot, matching tmux's shuffle.
        for win in self.links_mut(dst_session) {
            if win.index >= desired && win.index < free {
                win.index += 1;
            }
        }
        self.link_window_at(dst_session, desired, source_window_id, select)
    }

    /// Add a winlink to an existing window into `dst_session` at `index`,
    /// keeping the session sorted by index. When
    /// `select`, the new link becomes the session's current window and the
    /// previously-active window becomes its "last" (tmux's default; `-d` suppresses
    /// it). The caller must have already opened the `index` slot.
    fn link_window_at(
        &mut self,
        dst_session: usize,
        index: u32,
        window_id: u32,
        select: bool,
    ) -> io::Result<()> {
        let session_id = self.sessions[dst_session].id;
        let winlink_id = self.next_winlink_id;
        self.next_winlink_id += 1;
        let position = self.sessions[dst_session]
            .windows
            .iter()
            .take_while(|link| link.index < index)
            .count();
        self.insert_link(
            dst_session,
            position,
            Winlink {
                link_id: winlink_id,
                index,
                id: window_id,
                alert_flags: 0,
            },
            select,
        );
        self.remove_unlinked_windows();
        self.invalidate_session(
            session_id,
            if select {
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS
            } else {
                RenderInvalidation::STATUS
            },
        );
        Ok(())
    }

    /// Whether a session links the window at all — tmux's `session_has`.
    pub(super) fn session_links_window(&self, session_id: u32, window_id: u32) -> bool {
        self.sessions
            .iter()
            .find(|session| session.id == session_id)
            .is_some_and(|session| session.windows.iter().any(|link| link.id == window_id))
    }

    /// Whether the window is the one a session is currently showing.
    pub(super) fn session_shows_window(&self, session_id: u32, window_id: u32) -> bool {
        self.current_window_of_session(session_id) == Some(window_id)
    }

    pub(super) fn sessions_linking_window(&self, window_id: u32) -> Vec<u32> {
        self.sessions
            .iter()
            .filter(|session| session.windows.iter().any(|link| link.id == window_id))
            .map(|session| session.id)
            .collect()
    }
}
