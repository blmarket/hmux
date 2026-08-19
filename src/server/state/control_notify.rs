//! Control-mode notifications: tmux's `control-notify.c`, reached through the
//! same server-wide event queue the hooks run on.
//!
//! A mutation site raises an event holding identity only. The queue's runner
//! fires it — tmux's `notify_callback` — and firing resolves the event against
//! the state as it stands then, producing one [`ControlRecord`] appended to a
//! sequenced log. Every control client reads that log at its own pace and
//! decides per record what line, if any, it writes: the audience test is the
//! last step, because tmux's emitters branch per client rather than per event.


use super::client::RenderInvalidation;
use super::ServerState;

/// How many resolved notifications the log holds for clients that have not
/// caught up. Records are a line's worth of identity each, so the bound is
/// generous; a client further behind than this has stopped reading, which is
/// what `CONTROL_MAXIMUM_AGE` already ends.
const CONTROL_NOTIFICATION_LIMIT: usize = 4096;

/// What a mutation site raises, carrying identity only.
///
/// Values are read when the event fires, the way tmux's `notify_entry` holds
/// ref-counted pointers and its emitters read `w->name`, `s->curw` and the
/// per-client winlink lookup at callback time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NotifyEvent {
    SessionCreated,
    SessionClosed,
    SessionRenamed { session_id: u32 },
    SessionWindowChanged { session_id: u32 },
    WindowLinked { window_id: u32 },
    WindowUnlinked { window_id: u32 },
    WindowRenamed { window_id: u32 },
    WindowPaneChanged { window_id: u32 },
    WindowLayoutChanged { window_id: u32 },
    PaneModeChanged { pane_id: u32 },
    ClientSessionChanged { client: String },
    ClientDetached { client: String },
    PasteBufferChanged { name: String },
    PasteBufferDeleted { name: String },
}

/// A fired notification: everything its line needs, plus the audience test,
/// read once for every client that will see it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ControlRecord {
    /// Both `session-created` and `session-closed`; tmux writes the same
    /// argument-less line for either.
    SessionsChanged,
    SessionRenamed {
        session_id: u32,
        name: String,
    },
    SessionWindowChanged {
        session_id: u32,
        window_id: u32,
    },
    /// `sessions` is the set linking the window when the event fired, which is
    /// what each client's `%window-*` / `%unlinked-window-*` branch tests.
    WindowLinked {
        window_id: u32,
        sessions: Vec<u32>,
    },
    WindowUnlinked {
        window_id: u32,
        sessions: Vec<u32>,
    },
    WindowRenamed {
        window_id: u32,
        name: String,
        sessions: Vec<u32>,
    },
    WindowPaneChanged {
        window_id: u32,
        pane_id: u32,
    },
    /// Formatted once against the window's first link, as tmux formats once
    /// against `TAILQ_FIRST(&w->winlinks)`.
    WindowLayoutChanged {
        line: String,
        sessions: Vec<u32>,
    },
    PaneModeChanged {
        pane_id: u32,
    },
    /// One event, two lines: the client that moved reads `%session-changed`,
    /// every other control client reads `%client-session-changed`.
    ClientSessionChanged {
        client: String,
        session_id: u32,
        session_name: String,
    },
    ClientDetached {
        client: String,
    },
    PasteBufferChanged {
        name: String,
    },
    PasteBufferDeleted {
        name: String,
    },
}

impl ControlRecord {
    /// The line a control client named `client`, currently on `session_id`,
    /// writes for this record — or `None` when the record is not for it.
    pub(crate) fn line_for(&self, client: &str, session_id: u32) -> Option<String> {
        let linked = |sessions: &[u32]| sessions.contains(&session_id);
        Some(match self {
            Self::SessionsChanged => "%sessions-changed".to_string(),
            Self::SessionRenamed { session_id, name } => {
                format!("%session-renamed ${session_id} {name}")
            }
            Self::SessionWindowChanged {
                session_id,
                window_id,
            } => format!("%session-window-changed ${session_id} @{window_id}"),
            Self::WindowLinked {
                window_id,
                sessions,
            } => match linked(sessions) {
                true => format!("%window-add @{window_id}"),
                false => format!("%unlinked-window-add @{window_id}"),
            },
            Self::WindowUnlinked {
                window_id,
                sessions,
            } => match linked(sessions) {
                true => format!("%window-close @{window_id}"),
                false => format!("%unlinked-window-close @{window_id}"),
            },
            Self::WindowRenamed {
                window_id,
                name,
                sessions,
            } => match linked(sessions) {
                true => format!("%window-renamed @{window_id} {name}"),
                false => format!("%unlinked-window-renamed @{window_id} {name}"),
            },
            Self::WindowPaneChanged { window_id, pane_id } => {
                format!("%window-pane-changed @{window_id} %{pane_id}")
            }
            Self::WindowLayoutChanged { line, sessions } => {
                if !linked(sessions) {
                    return None;
                }
                line.clone()
            }
            Self::PaneModeChanged { pane_id } => format!("%pane-mode-changed %{pane_id}"),
            Self::ClientSessionChanged {
                client: moved,
                session_id,
                session_name,
            } => match moved == client {
                true => format!("%session-changed ${session_id} {session_name}"),
                false => {
                    format!("%client-session-changed {moved} ${session_id} {session_name}")
                }
            },
            Self::ClientDetached { client } => format!("%client-detached {client}"),
            Self::PasteBufferChanged { name } => format!("%paste-buffer-changed {name}"),
            Self::PasteBufferDeleted { name } => format!("%paste-buffer-deleted {name}"),
        })
    }
}

impl ServerState {
    /// The sequence a client that has seen everything sits at.
    pub(crate) fn control_notification_end(&self) -> u64 {
        self.next_control_notification
    }

    /// Everything fired since `cursor`, and the cursor to resume from.
    pub(crate) fn control_notifications_since(&self, cursor: u64) -> (u64, Vec<ControlRecord>) {
        let records = self
            .control_notifications
            .iter()
            .filter(|(sequence, _)| *sequence > cursor)
            .map(|(_, record)| record.clone())
            .collect();
        (self.next_control_notification, records)
    }

    /// tmux's global-queue drain reaching `control-notify.c`: resolve every
    /// event raised since the last drain against the state as it stands, append
    /// them in the order they were raised, and wake every control client —
    /// including the ones whose own session did not move, which is the audience
    /// an event in another session has.
    ///
    /// Called where a mutation is over rather than where a client is ready:
    /// what a record says is read at fire time, so firing late would report a
    /// window's name, a session's current window, or the set of sessions
    /// linking a window as some later command left them.
    pub(crate) fn fire_control_notifications(&mut self) {
        while let Some(event) = self.pending_control_events.pop_front() {
            self.fire_control_notification(&event);
        }
    }

    fn fire_control_notification(&mut self, event: &NotifyEvent) {
        let Some(record) = self.resolve_control_notification(event) else {
            return;
        };
        self.next_control_notification = self.next_control_notification.saturating_add(1);
        self.control_notifications
            .push_back((self.next_control_notification, record));
        while self.control_notifications.len() > CONTROL_NOTIFICATION_LIMIT {
            self.control_notifications.pop_front();
        }
        self.client_renders
            .publish_control(RenderInvalidation::NOTIFY);
    }

    fn resolve_control_notification(&self, event: &NotifyEvent) -> Option<ControlRecord> {
        Some(match event {
            NotifyEvent::SessionCreated | NotifyEvent::SessionClosed => {
                ControlRecord::SessionsChanged
            }
            NotifyEvent::SessionRenamed { session_id } => ControlRecord::SessionRenamed {
                session_id: *session_id,
                name: self.session_by_id(*session_id)?.name.clone(),
            },
            NotifyEvent::SessionWindowChanged { session_id } => {
                ControlRecord::SessionWindowChanged {
                    session_id: *session_id,
                    // tmux reads `s->curw` here; a session that has gone away
                    // between the mutation and the fire has no current window
                    // to name, and its clients have a closed session to report
                    // instead.
                    window_id: self.current_window_of_session(*session_id)?,
                }
            }
            NotifyEvent::WindowLinked { window_id } => ControlRecord::WindowLinked {
                window_id: *window_id,
                sessions: self.sessions_linking_window(*window_id),
            },
            NotifyEvent::WindowUnlinked { window_id } => ControlRecord::WindowUnlinked {
                window_id: *window_id,
                sessions: self.sessions_linking_window(*window_id),
            },
            NotifyEvent::WindowRenamed { window_id } => ControlRecord::WindowRenamed {
                window_id: *window_id,
                name: self.windows.get(window_id)?.name.clone(),
                sessions: self.sessions_linking_window(*window_id),
            },
            NotifyEvent::WindowPaneChanged { window_id } => {
                let window = self.windows.get(window_id)?;
                ControlRecord::WindowPaneChanged {
                    window_id: *window_id,
                    pane_id: window.panes.get(window.active)?.id,
                }
            }
            NotifyEvent::WindowLayoutChanged { window_id } => ControlRecord::WindowLayoutChanged {
                line: self.control_layout_line(*window_id)?,
                sessions: self.sessions_linking_window(*window_id),
            },
            NotifyEvent::PaneModeChanged { pane_id } => ControlRecord::PaneModeChanged {
                pane_id: *pane_id,
            },
            NotifyEvent::ClientSessionChanged { client } => {
                let (session_id, session_name) = self.control_client_session(client)?;
                ControlRecord::ClientSessionChanged {
                    client: client.clone(),
                    session_id,
                    session_name,
                }
            }
            NotifyEvent::ClientDetached { client } => ControlRecord::ClientDetached {
                client: client.clone(),
            },
            NotifyEvent::PasteBufferChanged { name } => ControlRecord::PasteBufferChanged {
                name: name.clone(),
            },
            NotifyEvent::PasteBufferDeleted { name } => ControlRecord::PasteBufferDeleted {
                name: name.clone(),
            },
        })
    }

    /// The session a named client is on when the event fires. tmux reads
    /// `cc->session` and returns early when the client has none.
    fn control_client_session(&self, client: &str) -> Option<(u32, String)> {
        let session_id = self
            .client_renders
            .with_entries(|mut entries| entries.find(|entry| entry.name == client).map(|entry| entry.session_id))?;
        Some((session_id, self.session_by_id(session_id)?.name.clone()))
    }

    /// `%layout-change`'s line, formatted against the window's first link the
    /// way tmux formats it against `TAILQ_FIRST(&w->winlinks)`.
    ///
    /// `None` when the window has no link or no layout left — tmux skips the
    /// line for a window that is on its way out rather than reporting a layout
    /// nobody can act on.
    fn control_layout_line(&self, window_id: u32) -> Option<String> {
        let window = self.windows.get(&window_id)?;
        if window.panes.is_empty() {
            return None;
        }
        let (session, position) = self.sessions.iter().find_map(|session| {
            session
                .windows
                .iter()
                .position(|link| link.id == window_id)
                .map(|position| (session, position))
        })?;
        let layout = checksummed_layout(&window.layout.dump());
        let visible = match window.zoomed {
            true => checksummed_layout(&format!(
                "{}x{},0,0,{}",
                window.cols,
                window.rows,
                window.panes.get(window.active).map_or(0, |pane| pane.id)
            )),
            false => layout.clone(),
        };
        let flags = self.printable_window_flags(session, position, false);
        Some(format!(
            "%layout-change @{window_id} {layout} {visible} {flags}"
        ))
    }
}

/// A layout dump with tmux's checksum prefix.
fn checksummed_layout(body: &str) -> String {
    let checksum = body.bytes().fold(0u16, |sum, byte| {
        sum.rotate_right(1).wrapping_add(u16::from(byte))
    });
    format!("{checksum:04x},{body}")
}
