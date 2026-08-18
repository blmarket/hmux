//! Panes — splitting, killing, moving, and every path that carries bytes
//! between a pane's pty and the clients watching it.
//!
//! A pane is a leaf of its window's [`LayoutCell`] tree plus the running
//! terminal behind it, so structural changes here always pair a pane-list edit
//! with a layout edit.

use std::collections::BTreeSet;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::copy::{view_copy_state, CopyBacking};
use super::layout::{resize_panes_to_layout, LayoutCell, PaneRect, SplitDirection};
use super::sizing::{DEFAULT_XPIXEL, DEFAULT_YPIXEL};
use super::target::{pane_not_found, parse_index_target, split_pane_target, Target, TargetKind};
use super::{
    cursor_style_parameter, fill_spawn_ids, fill_spec_spawn_ids, now_epoch, theme_report,
    ClientActionResult, ExitEmpty, PaneNode, PaneSpec, RenderInvalidation, ServerState,
    TerminalReply, TerminalRequest, TerminalRequestKind, Window, Winlink, ALERT_ACTIVITY,
};
use bytes::Bytes;

use crate::server::input_keys::{
    self, ExtendedKeys, ExtendedKeysFormat, PaneKey, PaneKeyEncoding, PaneKeyModes, PaneKeyOptions,
};
use crate::server::key::parse_key_name;
use crate::server::mouse::TtyMouseMode;
use crate::server::options::{OptionSet, OptionsView, PaneHook, WindowHook};
use crate::server::pane::{
    parse_packed_colour, MouseTrackingMode, Pane, PaneClipboardEvent, PaneIo, PaneKeyState,
    PaneOutputPolicy, PanePassthrough, PaneSpawnSpec, PassthroughPolicy,
};
use hmux_vt::MouseEvent;
use hmux_vt::ScreenOptions;

impl ServerState {
    /// Hand a pipe job the loop owns from here. `copy-pipe` uses this for the
    /// child it feeds a selection to, which belongs to no pane.
    pub(crate) fn adopt_pipe(&mut self, pipe: crate::server::pane::PanePipeIo) {
        self.new_pipes.push(pipe);
    }

    /// Pipe children opened since the last call, across every pane and the
    /// pane-less jobs adopted directly.
    pub(crate) fn take_new_pane_pipes(&mut self) -> Vec<crate::server::pane::PanePipeIo> {
        let mut pipes = std::mem::take(&mut self.new_pipes);
        pipes.extend(
            self.windows
                .values_mut()
                .flat_map(|window| window.panes.iter_mut())
                .flat_map(|node| node.pane.take_new_pipes()),
        );
        pipes
    }

    pub(crate) fn take_event_pane_ios(&mut self) -> Vec<(u64, PaneIo)> {
        self.windows
            .values_mut()
            .flat_map(|window| window.panes.iter_mut())
            .filter_map(|node| {
                node.pane
                    .take_event_io()
                    .map(|pane_io| (node.pane.runtime_id(), pane_io))
            })
            .collect()
    }

    pub(crate) fn pane_runtime_ids(&self) -> BTreeSet<u64> {
        self.windows
            .values()
            .flat_map(|window| window.panes.iter())
            .map(|node| node.pane.runtime_id())
            .collect()
    }

    /// tmux's `window_pane_update_focus`: announce a pane focus change once.
    pub(crate) fn update_pane_focus(&mut self, pane_id: u32, focused: bool) {
        let changed = if focused {
            self.focused_panes.insert(pane_id)
        } else {
            self.focused_panes.remove(&pane_id)
        };
        if !changed {
            return;
        }
        // A pane that asked for focus reporting gets its own copy of the
        // escape, exactly as `window_pane_update_focus` sends it.
        let subscribed = self
            .windows
            .values()
            .flat_map(|window| window.panes.iter())
            .find(|node| node.id == pane_id)
            .is_some_and(|node| node.pane.focus_reporting_enabled());
        if subscribed {
            let _ = self.write_pane_input(pane_id, if focused { b"\x1b[I" } else { b"\x1b[O" });
        }
        let was_deferred = std::mem::replace(&mut self.notifications_are_deferred, true);
        self.notify_pane(
            if focused {
                PaneHook::FocusIn
            } else {
                PaneHook::FocusOut
            },
            pane_id,
        );
        self.notifications_are_deferred = was_deferred;
    }

    /// Apply pane child-exit lifecycle (`remain-on-exit`, then `exit-empty`).
    ///
    /// Returns true when at least one pane was removed. Empty windows and
    /// sessions are removed with it, matching tmux's ownership hierarchy.
    pub fn reap_exited_panes(&mut self) -> bool {
        let had_sessions = !self.sessions.is_empty();
        let mut removed = false;
        for window in self.windows.values_mut() {
            for pane in &mut window.panes {
                if pane.pane.has_exited() {
                    pane.pane.collect_exited_child();
                }
            }
        }
        let retained = self
            .windows
            .values()
            .flat_map(|window| {
                window.panes.iter().filter_map(|pane| {
                    // `on` and `key` keep the dead pane; `failed` keeps it
                    // only when the child exited non-zero (or to a signal).
                    let keep = match pane
                        .options(window, &self.global_options)
                        .get("remain-on-exit")
                    {
                        Some("on") | Some("key") => true,
                        Some("failed") => pane
                            .pane
                            .death()
                            .is_some_and(|death| death.status != Some(0)),
                        _ => false,
                    };
                    keep.then_some(pane.id)
                })
            })
            .collect::<BTreeSet<_>>();
        // tmux's `server_destroy_pane` reports `pane-died` for a pane held open
        // by `remain-on-exit` and `pane-exited` for one that goes away. Either
        // way it is announced once, the first time the child is seen gone.
        let newly_exited = self
            .windows
            .values_mut()
            .flat_map(|window| window.panes.iter_mut())
            .filter(|pane| pane.pane.has_exited() && !pane.exit_notified)
            .map(|pane| {
                pane.exit_notified = true;
                pane.id
            })
            .collect::<Vec<_>>();
        // tmux's `server_destroy_pane` paints the expanded remain-on-exit-format
        // onto a pane it keeps: full scroll region, cursor to the bottom-left,
        // one linefeed, then the text — and the cursor is hidden.
        for &pane_id in newly_exited.iter().filter(|id| retained.contains(id)) {
            let Some((window, pane)) = self.windows.values().find_map(|window| {
                window
                    .panes
                    .iter()
                    .find(|pane| pane.id == pane_id)
                    .map(|pane| (window, pane))
            }) else {
                continue;
            };
            let template = pane
                .options(window, &self.global_options)
                .get("remain-on-exit-format")
                .unwrap_or_default()
                .to_string();
            if template.is_empty() {
                continue;
            }
            let death = pane.pane.death();
            let mut vars = crate::server::format::Vars::new();
            vars.set("pane_id", format!("%{pane_id}"))
                .set("pane_dead", "1")
                .set(
                    "pane_dead_status",
                    death
                        .and_then(|death| death.status)
                        .map(|status| status.to_string())
                        .unwrap_or_default(),
                )
                .set(
                    "pane_dead_signal",
                    death
                        .and_then(|death| death.signal)
                        .map(|signal| signal.to_string())
                        .unwrap_or_default(),
                )
                .set(
                    "pane_dead_time",
                    death
                        .map(|death| {
                            death
                                .at
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs()
                                .to_string()
                        })
                        .unwrap_or_default(),
                );
            let expanded = crate::server::format::expand(&template, &vars);
            let (_, rows) = pane.pane.size();
            pane.pane
                .feed(format!("\x1b[r\x1b[{rows};1H\n\x1b[?25l{expanded}").as_bytes());
        }
        self.deferred_notifications(|state| {
            for pane_id in newly_exited {
                let hook = if retained.contains(&pane_id) {
                    PaneHook::Died
                } else {
                    PaneHook::Exited
                };
                state.notify_pane(hook, pane_id);
            }
        });
        for window in self.windows.values_mut() {
            let active_id = window.panes.get(window.active).map(|pane| pane.id);
            let last_id = window
                .last_pane
                .and_then(|index| window.panes.get(index))
                .map(|pane| pane.id);
            let exited_ids = window
                .panes
                .iter()
                .filter(|pane| {
                    pane.pane.has_exited()
                        && pane.pane.child_reaped()
                        && !retained.contains(&pane.id)
                })
                .map(|pane| pane.id)
                .collect::<Vec<_>>();
            let before = window.panes.len();
            window.panes.retain(|pane| {
                !pane.pane.has_exited() || !pane.pane.child_reaped() || retained.contains(&pane.id)
            });
            let panes_removed = window.panes.len() != before;
            removed |= panes_removed;
            if panes_removed && !window.panes.is_empty() {
                for pane_id in exited_ids {
                    window.layout.remove(pane_id);
                }
                let _ = resize_panes_to_layout(window);
                window.active = active_id
                    .and_then(|id| window.panes.iter().position(|pane| pane.id == id))
                    .unwrap_or_else(|| window.active.min(window.panes.len() - 1));
                window.last_pane =
                    last_id.and_then(|id| window.panes.iter().position(|pane| pane.id == id));
            }
        }

        let empty = self
            .windows
            .iter()
            .filter_map(|(id, window)| window.panes.is_empty().then_some(*id))
            .collect::<BTreeSet<_>>();
        let link_sets = self
            .sessions
            .iter()
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
            let old = self.sessions[member].windows.clone();
            let links = old
                .iter()
                .copied()
                .filter(|link| !empty.contains(&link.id))
                .collect::<Vec<_>>();
            if links.len() == old.len() {
                continue;
            }
            self.replace_link_set(member, links);
        }
        self.sessions.retain(|session| !session.windows.is_empty());
        self.remove_unlinked_windows();

        if had_sessions && self.sessions.is_empty() && self.exit_empty_policy() != ExitEmpty::Off {
            self.shutdown_requested = true;
        }
        removed
    }

    /// Push each pane the options it has to consult about its own output: how
    /// the bytes are parsed, and what an operation on the grid does.
    ///
    /// tmux reads them from `wp->options` at the moment they matter, with the
    /// whole server in reach. hmux runs both the tokenizer and the screen off
    /// the state lock, so the resolved values are pushed to the pane instead —
    /// once per server loop, and again as soon as a `set-option` touches one of
    /// them.
    pub(crate) fn refresh_pane_options(&self) {
        // `history-limit` is a session option, but the rows live in panes.
        // Apply each session's effective value to the panes in its windows so
        // lowering it trims already populated grids, as tmux does.
        for session in &self.sessions {
            let history_limit = session
                .options(&self.global_options)
                .get("history-limit")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(2000);
            for link in &session.windows {
                let Some(window) = self.windows.get(&link.id) else {
                    continue;
                };
                for node in &window.panes {
                    node.pane.set_history_limit(history_limit);
                }
            }
        }
        for window in self.windows.values() {
            for node in &window.panes {
                let options = node.options(window, &self.global_options);
                node.pane.set_screen_options(ScreenOptions {
                    scroll_on_clear: options.get("scroll-on-clear") != Some("off"),
                    // Not an option: the window's cell size in pixels, which
                    // the XTWINOPS pixel reports are answered from.
                    xpixel: u32::from(window.xpixel),
                    ypixel: u32::from(window.ypixel),
                });
                node.pane.set_output_policy(PaneOutputPolicy {
                    alternate_screen: options.get("alternate-screen") != Some("off"),
                    allow_set_title: options.get("allow-set-title") != Some("off"),
                    passthrough: match options.get("allow-passthrough") {
                        Some("on") => PassthroughPolicy::Visible,
                        Some("all") => PassthroughPolicy::Always,
                        _ => PassthroughPolicy::Off,
                    },
                    // Server-scoped, so the window view never sees it.
                    input_buffer_size: self
                        .server_options()
                        .get("input-buffer-size")
                        .and_then(|value| value.parse().ok())
                        .unwrap_or(1_048_576),
                    cursor_style: cursor_style_parameter(
                        options.get("cursor-style").unwrap_or("default"),
                    ),
                    // The index an entry is stored at *is* the palette slot it
                    // fills, so the array is read with its indexes rather than
                    // as a list.
                    palette: {
                        let entries = OptionsView::three(
                            node.option_overrides(),
                            window.option_overrides(),
                            self.global_options.window(),
                        )
                        .array_entries("pane-colours");
                        let mut palette =
                            vec![None; entries.last().map_or(0, |(index, _)| *index as usize + 1)];
                        for (index, value) in entries {
                            palette[index as usize] = parse_packed_colour(value);
                        }
                        palette
                    },
                });
            }
        }
    }

    /// Put the `DCS tmux;` payloads panes emitted since the last pass onto the
    /// client ttys they are allowed to reach, mirroring tmux's
    /// `screen_write_rawstring` and the client walk in `tty_write`.
    ///
    /// `allow-passthrough on` reaches the clients whose *current* window holds
    /// the pane; `all` also reaches those that merely have the window linked.
    /// Which of the two applied was decided when the sequence completed, since
    /// that is when tmux reads the option.
    pub(crate) fn process_pane_passthrough(&mut self) {
        let panes = self
            .windows
            .iter()
            .map(|(window_id, window)| {
                (
                    *window_id,
                    window
                        .panes
                        .iter()
                        .map(|node| node.pane.observation_state())
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();
        for (window_id, observations) in panes {
            for observation in observations {
                for PanePassthrough {
                    data,
                    invisible_panes,
                } in observation.take_passthrough()
                {
                    let sessions = self
                        .sessions
                        .iter()
                        .filter(|session| {
                            if invisible_panes {
                                session.windows.iter().any(|link| link.id == window_id)
                            } else {
                                session
                                    .windows
                                    .get(session.active)
                                    .is_some_and(|link| link.id == window_id)
                            }
                        })
                        .map(|session| session.id)
                        .collect::<BTreeSet<_>>();
                    self.client_renders
                        .write_client_output(&sessions, Bytes::from(data));
                }
            }
        }
    }

    /// Apply the clipboard policy to the OSC 52 sequences panes emitted since
    /// the last pass, mirroring tmux's `input_osc_52`.
    ///
    /// Applications may only touch the clipboard under `set-clipboard on`; the
    /// sequence is not even parsed otherwise. A query is then answered from the
    /// newest paste buffer when `get-clipboard` is `buffer`, and put to the
    /// client's own terminal under `request`/`both`, whose answer comes back
    /// through [`Self::deliver_terminal_reply`].
    pub(crate) fn process_pane_clipboard(&mut self) {
        let panes = self
            .windows
            .values()
            .flat_map(|window| window.panes.iter())
            .map(|node| (node.id, node.pane.observation_state()))
            .collect::<Vec<_>>();
        let allow_applications = self.server_options().get("set-clipboard") == Some("on");
        let get_clipboard = self
            .server_options()
            .get("get-clipboard")
            .unwrap_or("buffer");
        let answer_from_buffer = get_clipboard == "buffer";
        let forward_to_terminal = matches!(get_clipboard, "request" | "both");
        for (pane_id, observation) in panes {
            for event in observation.take_clipboard_events() {
                if !allow_applications {
                    continue;
                }
                match event {
                    PaneClipboardEvent::Set { data } => {
                        self.set_buffer(None, &data);
                        let was_deferred =
                            std::mem::replace(&mut self.notifications_are_deferred, true);
                        self.notify_pane(PaneHook::SetClipboard, pane_id);
                        self.notifications_are_deferred = was_deferred;
                    }
                    PaneClipboardEvent::Query {
                        selection,
                        string_terminator,
                    } => {
                        if forward_to_terminal {
                            // tmux's `input_add_request`: ask the client's own
                            // terminal instead, with the `Ms` capability the
                            // clipboard writes already use, and remember which
                            // pane is owed the answer. With no client to ask,
                            // the pane goes unanswered as it does in tmux.
                            let Some(client) = self.request_client_for_pane(pane_id) else {
                                continue;
                            };
                            if self.set_client_selection(Some(&client), None, None)
                                == ClientActionResult::Queued
                            {
                                self.add_terminal_request(
                                    client,
                                    pane_id,
                                    TerminalRequestKind::Clipboard {
                                        bel: !string_terminator,
                                    },
                                );
                            }
                            continue;
                        }
                        if !answer_from_buffer {
                            continue;
                        }
                        let Some(data) = self.buffer(None).map(<[u8]>::to_vec) else {
                            continue;
                        };
                        let mut reply = format!(
                            "\x1b]52;{selection};{}",
                            crate::server::cmd_send_keys::base64_encode(&data)
                        )
                        .into_bytes();
                        reply.extend_from_slice(if string_terminator {
                            b"\x1b\\"
                        } else {
                            b"\x07"
                        });
                        let _ = self.write_pane_input(pane_id, &reply);
                    }
                }
            }
        }
    }

    /// Put each pane's OSC 4 questions about palette entries it does not hold
    /// to an attached terminal, mirroring tmux's `input_osc_4` falling through
    /// to `input_add_request`.
    pub(crate) fn process_pane_palette_queries(&mut self) {
        let panes = self
            .windows
            .values()
            .flat_map(|window| window.panes.iter())
            .map(|node| (node.id, node.pane.observation_state()))
            .collect::<Vec<_>>();
        for (pane_id, observation) in panes {
            for (index, bel) in observation.take_palette_queries() {
                let Some(client) = self.request_client_for_pane(pane_id) else {
                    continue;
                };
                // tmux asks with a string terminator whatever the pane used;
                // only the answer it writes back mirrors the pane's own.
                self.client_renders.write_client_output_named(
                    &client,
                    Bytes::from(format!("\x1b]4;{index};?\x1b\\")),
                );
                self.add_terminal_request(
                    client,
                    pane_id,
                    TerminalRequestKind::Palette { index, bel },
                );
            }
        }
    }

    /// The client tmux's `input_add_request` would ask on this pane's behalf:
    /// the most recently active attached client whose session can see the
    /// pane's window.
    fn request_client_for_pane(&self, pane_id: u32) -> Option<String> {
        let window_id = self
            .windows
            .values()
            .find(|window| window.panes.iter().any(|node| node.id == pane_id))
            .map(|window| window.id)?;
        let sessions = self
            .sessions
            .iter()
            .filter(|session| session.windows.iter().any(|link| link.id == window_id))
            .map(|session| session.id)
            .collect::<BTreeSet<_>>();
        self.client_snapshots()
            .into_iter()
            .filter(|client| !client.control_mode && sessions.contains(&client.session_id))
            .max_by_key(|client| client.activity_micros)
            .map(|client| client.name)
    }

    fn add_terminal_request(&mut self, client: String, pane_id: u32, kind: TerminalRequestKind) {
        self.terminal_requests.push(TerminalRequest {
            client,
            pane_id,
            kind,
            at: Instant::now(),
        });
    }

    /// Whether anything is still owed an answer from this client's terminal,
    /// which is what makes its input worth scanning for one.
    pub(crate) fn client_awaits_terminal_reply(&self, client: &str) -> bool {
        self.terminal_requests
            .iter()
            .any(|request| request.client == client)
            || self.client_renders.clipboard_query_outstanding(client)
    }

    /// tmux's `input_request_timer_callback`: a question the terminal never
    /// answered stops being owed an answer.
    pub(crate) fn expire_terminal_requests(&mut self) {
        /// tmux's `INPUT_REQUEST_TIMEOUT`.
        const TIMEOUT: Duration = Duration::from_millis(500);
        let now = Instant::now();
        self.terminal_requests
            .retain(|request| now.saturating_duration_since(request.at) < TIMEOUT);
    }

    /// Route what an attached terminal answered, mirroring tmux's
    /// `input_request_reply`.
    ///
    /// The answer goes to the pane that asked, not to the client's active pane:
    /// the request records which one that was. A clipboard answer additionally
    /// becomes a paste buffer when `get-clipboard both` asked for one, or when
    /// the question came from `refresh-client -l` rather than from a pane.
    pub(crate) fn deliver_terminal_reply(&mut self, client: &str, reply: TerminalReply) {
        if let TerminalReply::Clipboard { data, .. } = &reply {
            // tmux's `TTY_OSC52QUERY`: `refresh-client -l` keeps its own
            // outstanding-query flag, and the answer both fills a buffer and
            // releases the flag so the next `-l` can ask again.
            if self.client_renders.take_clipboard_query(client) && !data.is_empty() {
                let data = data.clone();
                self.set_buffer(None, &data);
            }
        }
        let matching = self.terminal_requests.iter().position(|request| {
            request.client == client
                && match (&request.kind, &reply) {
                    (
                        TerminalRequestKind::Palette { index, .. },
                        TerminalReply::Palette { index: replied, .. },
                    ) => index == replied,
                    (TerminalRequestKind::Clipboard { .. }, TerminalReply::Clipboard { .. }) => {
                        true
                    }
                    _ => false,
                }
        });
        let Some(matching) = matching else {
            return;
        };
        let request = self.terminal_requests.remove(matching);
        match (request.kind, reply) {
            (
                TerminalRequestKind::Palette { index, bel },
                TerminalReply::Palette { colour, .. },
            ) => {
                let (r, g, b) = ((colour >> 16) as u8, (colour >> 8) as u8, colour as u8);
                let terminator = if bel { "\x07" } else { "\x1b\\" };
                let answer = format!(
                    "\x1b]4;{index};rgb:{r:02x}{r:02x}/{g:02x}{g:02x}/{b:02x}{b:02x}{terminator}"
                );
                let _ = self.write_pane_input(request.pane_id, answer.as_bytes());
            }
            (
                TerminalRequestKind::Clipboard { bel },
                TerminalReply::Clipboard { selection, data },
            ) => {
                // tmux re-reads `get-clipboard` when the answer lands: `both`
                // keeps a copy for itself as well as answering the pane.
                match self
                    .server_options()
                    .get("get-clipboard")
                    .unwrap_or("buffer")
                {
                    "both" => self.set_buffer(None, &data),
                    "request" => {}
                    // `off` and `buffer` never asked the terminal anything.
                    _ => return,
                }
                let selection = selection.map(char::from).unwrap_or('c');
                let mut answer = format!(
                    "\x1b]52;{selection};{}",
                    crate::server::cmd_send_keys::base64_encode(&data)
                )
                .into_bytes();
                answer.extend_from_slice(if bel { b"\x07" } else { b"\x1b\\" });
                let _ = self.write_pane_input(request.pane_id, &answer);
            }
            _ => {}
        }
    }

    /// Answer pending DSR ?996 questions and push theme changes to the panes
    /// that subscribed with DECSET 2031, mirroring tmux's
    /// `window_pane_send_theme_update`.
    pub(crate) fn process_pane_themes(&mut self) {
        let panes = self
            .windows
            .values()
            .flat_map(|window| window.panes.iter())
            .map(|node| (node.id, node.pane.observation_state()))
            .collect::<Vec<_>>();
        for (pane_id, _) in &panes {
            let theme = self.pane_theme(*pane_id);
            let node = self
                .windows
                .values()
                .flat_map(|window| window.panes.iter())
                .find(|node| node.id == *pane_id);
            let Some(node) = node else { continue };
            let queried = node.pane.take_theme_query();
            let subscribed = node.pane.theme_updates_enabled();
            if queried {
                if let Some(theme) = theme {
                    let _ = self.write_pane_input(*pane_id, theme_report(theme));
                }
            }
            // A pane learns of a change only after it subscribes, so the theme
            // in force at subscription time is recorded rather than pushed.
            if !subscribed {
                self.pane_theme_pushed.remove(pane_id);
                continue;
            }
            let previous = self.pane_theme_pushed.get(pane_id).cloned();
            let current = theme.unwrap_or("unknown").to_string();
            match previous {
                None => {
                    self.pane_theme_pushed.insert(*pane_id, current);
                }
                Some(previous) if previous != current => {
                    self.pane_theme_pushed.insert(*pane_id, current);
                    if let Some(theme) = theme {
                        let _ = self.write_pane_input(*pane_id, theme_report(theme));
                    }
                }
                Some(_) => {}
            }
        }
        let live = panes.iter().map(|(id, _)| *id).collect::<BTreeSet<_>>();
        self.pane_theme_pushed.retain(|id, _| live.contains(id));
    }

    /// tmux's `window_pane_get_theme`, restricted to the half hmux can answer:
    /// the pane's own background colour. `None` is tmux's `THEME_UNKNOWN`.
    fn pane_theme(&self, pane_id: u32) -> Option<&'static str> {
        let style = self.option_for_target(&format!("%{pane_id}"), "window-style")?;
        let background = style.split(',').find_map(|part| {
            part.trim()
                .strip_prefix("bg=")
                .and_then(crate::server::style::parse_colour)
        })?;
        crate::server::style::colour_theme(background)
    }

    /// Write bytes into a pane's input as if its terminal had produced them.
    fn write_pane_input(&self, pane_id: u32, bytes: &[u8]) -> io::Result<()> {
        let pane = self
            .windows
            .values()
            .flat_map(|window| window.panes.iter())
            .find(|node| node.id == pane_id)
            .ok_or_else(|| io::Error::other("pane disappeared"))?;
        pane.pane.input(bytes)
    }

    /// `split-window target`: add a pane to the target window (splitting the
    /// current one, geometry aside — the control plane only observes the pane
    /// set). The new pane becomes the window's active pane, as in tmux.
    /// `split-window`. `before` selects `-b`: the new pane is inserted *before*
    /// the target pane instead of appended after it. Returns the new pane's index
    /// within the window so the caller can render `-P`.
    pub fn split_window(&mut self, target: &str, select: bool, before: bool) -> io::Result<usize> {
        self.split_window_direction(target, select, before, SplitDirection::TopBottom)
    }

    pub(crate) fn split_window_direction(
        &mut self,
        target: &str,
        select: bool,
        before: bool,
        direction: SplitDirection,
    ) -> io::Result<usize> {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
        self.split_window_direction_with_spawn(
            target,
            select,
            before,
            false,
            direction,
            &[shell],
            None,
            None,
        )
    }

    pub(crate) fn split_window_direction_with_spawn(
        &mut self,
        target: &str,
        select: bool,
        before: bool,
        full: bool,
        direction: SplitDirection,
        argv: &[String],
        cwd: Option<&Path>,
        new_size: Option<u16>,
    ) -> io::Result<usize> {
        let spec = match cwd {
            Some(cwd) => PaneSpec::CommandIn(argv.to_vec(), cwd.to_path_buf()),
            None => PaneSpec::Command(argv.to_vec()),
        };
        self.split_window_direction_with_spec(
            target, select, before, full, direction, spec, new_size,
        )
    }

    pub(crate) fn split_window_direction_with_spec(
        &mut self,
        target: &str,
        select: bool,
        before: bool,
        full: bool,
        direction: SplitDirection,
        spec: PaneSpec,
        new_size: Option<u16>,
    ) -> io::Result<usize> {
        let (_, pane_part) = split_pane_target(target);
        let t = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[t.session].id;
        let window = self.window(t.session, t.window);
        let (cols, rows) = (window.cols, window.rows);
        // The id the pane will take, peeked for its environment; the allocation
        // below consumes it.
        let spec = fill_spec_spawn_ids(spec, self.next_pane_id, session_id);
        let pane = match spec {
            PaneSpec::Inert => Pane::inert(cols, rows)?,
            PaneSpec::Command(argv) => {
                let refs = argv.iter().map(String::as_str).collect::<Vec<_>>();
                Pane::spawn(&refs, None, cols, rows)?
            }
            PaneSpec::CommandIn(argv, cwd) => {
                let refs = argv.iter().map(String::as_str).collect::<Vec<_>>();
                Pane::spawn(&refs, Some(&cwd), cols, rows)?
            }
            // The child is already running and its pty already open: this pane
            // only changes owner.
            PaneSpec::Existing(pane) => *pane,
        };
        let pane_id = self.next_pane_id;
        self.next_pane_id += 1;
        let win = self.window_mut(t.session, t.window);
        // With `-b` the new pane lands at the target pane's index, pushing it (and
        // everything after) up; otherwise it follows the target. For `-f` (fullsize),
        // tmux inserts at the head (with `-b`) or tail (without `-b`) of the window's panes.
        let insert_at = if full {
            if before {
                0
            } else {
                win.panes.len()
            }
        } else if before {
            match pane_part {
                Some(_) => t.pane,
                None => win.active,
            }
        } else {
            t.pane + 1
        };
        let old_active = win.active;
        let target_index = t.pane;
        let target_id = win.panes[target_index].id;
        let split_ok = if full {
            win.layout.split_full(pane_id, direction, before, new_size)
        } else {
            win.layout
                .split_sized(target_id, pane_id, direction, before, new_size)
        };
        if !split_ok {
            return Err(io::Error::other("target pane is absent from layout"));
        }
        win.panes.insert(
            insert_at,
            PaneNode {
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
            },
        );
        resize_panes_to_layout(win)?;
        // Inserting at/at-or-before the old active pane shifts its index up by one.
        let shifted_old = if insert_at <= old_active {
            old_active + 1
        } else {
            old_active
        };
        // `-d` splits in the background: the new pane is added but the original
        // pane stays active. tmux's default is to select the new pane.
        let window_id = win.id;
        let active_changed = if select {
            win.last_pane = Some(shifted_old);
            win.set_active_pane(insert_at);
            true
        } else {
            win.active = shifted_old;
            false
        };
        self.invalidate_session(
            session_id,
            RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
        );
        if active_changed {
            self.notify_window(WindowHook::PaneChanged, window_id);
        }
        self.notify_window(WindowHook::LayoutChanged, window_id);
        Ok(insert_at)
    }

    pub(crate) fn new_floating_pane_with_spawn(
        &mut self,
        target: &str,
        select: bool,
        width: Option<u16>,
        height: Option<u16>,
        left: Option<i32>,
        top: Option<i32>,
        argv: &[String],
        cwd: Option<&Path>,
    ) -> io::Result<usize> {
        let target = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[target.session].id;
        let window = self.window(target.session, target.window);
        let (cols, rows) = (window.cols, window.rows);
        let width = width
            .unwrap_or(cols / 2)
            .clamp(1, cols.saturating_sub(1).max(1));
        let height = height
            .unwrap_or(rows / 4)
            .clamp(1, rows.saturating_sub(1).max(1));
        let argv = fill_spawn_ids(argv, self.next_pane_id, session_id);
        let refs = argv.iter().map(String::as_str).collect::<Vec<_>>();
        let pane = Pane::spawn(&refs, cwd, width, height)?;
        let pane_id = self.next_pane_id;
        self.next_pane_id += 1;
        let window = self.window_mut(target.session, target.window);
        let left = left.unwrap_or_else(|| {
            let next = if window.last_new_pane_x == 0 || window.last_new_pane_x > cols {
                4
            } else {
                window.last_new_pane_x.saturating_add(4)
            };
            window.last_new_pane_x = next;
            i32::from(next)
        });
        let top = top.unwrap_or_else(|| {
            let next = if window.last_new_pane_y == 0 || window.last_new_pane_y > rows {
                2
            } else {
                window.last_new_pane_y.saturating_add(2)
            };
            window.last_new_pane_y = next;
            i32::from(next)
        });
        let insert_at = target.pane + 1;
        let old_active = window.active;
        window.panes.insert(
            insert_at,
            PaneNode {
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
                floating: Some(PaneRect {
                    top: top.max(0) as u16,
                    left: left.max(0) as u16,
                    height,
                    width,
                }),
                scrollbar_columns: 0,
                border_status: None,
                unseen_changes: false,
                active_point: 0,
                options: OptionSet::default(),
            },
        );
        let shifted_old = if insert_at <= old_active {
            old_active + 1
        } else {
            old_active
        };
        if select {
            window.last_pane = Some(shifted_old);
            window.set_active_pane(insert_at);
        } else {
            window.active = shifted_old;
        }
        self.invalidate_session(
            session_id,
            RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
        );
        Ok(insert_at)
    }

    pub(crate) fn set_pane_option(&mut self, target: Target, name: &str, value: &str) {
        self.window_mut(target.session, target.window).panes[target.pane]
            .option_overrides_mut()
            .set(name, value);
    }

    /// `select-pane target`: make the target pane active within its window.
    /// `target` is `session[:window].pane`; a missing pane part means the active
    /// pane (a no-op). Reports tmux's `can't find pane: <part>` on a miss.
    pub fn select_pane(&mut self, target: &str) -> io::Result<()> {
        let t = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[t.session].id;
        let win = self.window_mut(t.session, t.window);
        let window_id = win.id;
        let previous_active = win.active;
        if t.pane != win.active {
            win.last_pane = Some(win.active);
            win.set_active_pane(t.pane);
            self.invalidate_session(
                session_id,
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
            );
            self.notify_window(WindowHook::PaneChanged, window_id);
            // tmux only follows the active pane with focus while
            // `focus-events` is on (`window_set_active_pane`).
            if self.server_option_is_on("focus-events", false) {
                let previous_pane = self.window(t.session, t.window).panes[previous_active].id;
                self.update_pane_focus(previous_pane, false);
                self.update_window_focus(window_id);
            }
        }
        Ok(())
    }

    /// `select-pane -T title`: pin a pane's title, overriding whatever its
    /// terminal reported. Raises `pane-title-changed` for every store, matching
    /// tmux, whose `screen_set_title` reports only whether the title was valid
    /// rather than whether it moved — so re-pinning the same title notifies
    /// again.
    pub(crate) fn set_pane_title(&mut self, target: &str, title: &str) -> io::Result<()> {
        let t = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[t.session].id;
        let node = &mut self.window_mut(t.session, t.window).panes[t.pane];
        let pane_id = node.id;
        let changed = node.title.as_deref() != Some(title);
        node.title = Some(title.to_string());
        if changed {
            self.invalidate_session(session_id, RenderInvalidation::STATUS);
        }
        self.notify_pane(PaneHook::TitleChanged, pane_id);
        Ok(())
    }

    /// The title `#{pane_title}` reports: the `select-pane -T` override when
    /// one is set, else what the pane's terminal last announced.
    pub(crate) fn pane_title(&self, node: &PaneNode) -> Option<String> {
        node.title.clone().or_else(|| {
            node.pane
                .observation_state()
                .contract_title()
                .ok()
                .flatten()
        })
    }

    pub(crate) fn set_pane_input_off(&mut self, target: &str, input_off: bool) -> io::Result<()> {
        let target = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        self.window_mut(target.session, target.window).panes[target.pane].input_off = input_off;
        Ok(())
    }

    pub(crate) fn target_pane_ids(&self, target: Target) -> (u32, u32) {
        let window = self.window(target.session, target.window);
        (window.id, window.panes[target.pane].id)
    }

    pub(crate) fn report_pane_control_colour(
        &mut self,
        target: &str,
        report: &[u8],
    ) -> io::Result<()> {
        let target = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        self.window_mut(target.session, target.window).panes[target.pane]
            .pane
            .input(report)?;
        Ok(())
    }

    pub(crate) fn pane_in_direction(
        &self,
        target: &str,
        direction: SplitDirection,
        forward: bool,
    ) -> io::Result<(u32, u32)> {
        let target = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let window = self.window(target.session, target.window);
        let active_id = window.panes[target.pane].id;
        let next_id = window
            .layout
            .neighbour(active_id, direction, forward)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no pane in direction"))?;
        Ok((window.id, next_id))
    }

    /// Select the nearest pane in the requested geometric direction.
    pub(crate) fn select_pane_direction(
        &mut self,
        target: &str,
        direction: SplitDirection,
        forward: bool,
    ) -> io::Result<()> {
        let t = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[t.session].id;
        let win = self.window_mut(t.session, t.window);
        let active_id = win.panes[t.pane].id;
        let next_id = win
            .layout
            .neighbour(active_id, direction, forward)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no pane in direction"))?;
        let next = win
            .panes
            .iter()
            .position(|pane| pane.id == next_id)
            .expect("layout pane belongs to window");
        win.last_pane = Some(win.active);
        win.set_active_pane(next);
        self.invalidate_session(
            session_id,
            RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
        );
        Ok(())
    }

    /// The server's marked pane id, or `None` if no pane is marked. Self-heals a
    /// stale mark: if the marked pane has since been destroyed the mark reads as
    /// cleared, matching tmux (which drops the mark when the pane goes away), so
    /// `#{pane_marked_set}` never reports a mark that no longer exists.
    pub fn marked_pane(&self) -> Option<u32> {
        let id = self.marked_pane_id?;
        self.windows
            .values()
            .any(|window| window.panes.iter().any(|pane| pane.id == id))
            .then_some(id)
    }

    /// `select-pane -m target`: toggle the server's marked pane. Marking the
    /// already-marked pane clears the mark (tmux's toggle); marking any other pane
    /// moves the mark to it. Does *not* change which pane is active. `target` is
    /// `session[:window].pane` (default: the active pane). Reports tmux's `can't
    /// find pane: <part>` on a miss.
    pub fn mark_pane(&mut self, target: &str) -> io::Result<()> {
        let t = self
            .find_target(target, TargetKind::Pane)
            .map_err(|miss| miss.error)?;
        let id = self.window(t.session, t.window).panes[t.pane].id;
        self.marked_pane_id = if self.marked_pane_id == Some(id) {
            None
        } else {
            Some(id)
        };
        Ok(())
    }

    /// `select-pane -M`: clear the server's marked pane unconditionally.
    pub fn clear_mark(&mut self) {
        self.marked_pane_id = None;
    }

    /// Start or stop a pipe attached to the selected pane's PTY.
    pub fn pipe_pane(
        &mut self,
        target: &str,
        command: Option<&str>,
        only_toggle: bool,
        input: bool,
        output: bool,
    ) -> io::Result<()> {
        let (win_target, pane_part) = split_pane_target(target);
        let t = self.resolve_window(win_target)?;
        let idx = match pane_part {
            None => self.window(t.session, t.window).active,
            Some(p) => self
                .pane_pos(t.session, t.window, p)
                .ok_or_else(|| pane_not_found(p))?,
        };
        let win = self.window_mut(t.session, t.window);
        let node = &mut win.panes[idx];
        if node.pane.death().is_some() || node.pane.has_exited() {
            return Err(io::Error::other("target pane has exited"));
        }
        let had_pipe = node.pane.pipe_active();
        node.pane.close_pipe();
        if let Some(command) = command.filter(|command| !command.is_empty()) {
            if !(only_toggle && had_pipe) {
                node.pane.open_pipe(command, input, output)?;
            }
        }
        Ok(())
    }

    /// `kill-pane target`: remove the target pane. Removing a window's last pane
    /// destroys the window (and the session if it was the last window), matching
    /// tmux. `target` is `session[:window].pane` (default: the active pane).
    pub fn kill_pane(&mut self, target: &str) -> io::Result<()> {
        let t = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[t.session].id;
        let window_id = self.sessions[t.session].windows[t.window].id;
        let pane_idx = t.pane;
        let window_empty = {
            let win = self.window_mut(t.session, t.window);
            let pane_id = win.panes[pane_idx].id;
            win.panes.remove(pane_idx);
            win.layout.remove(pane_id);
            win.panes.is_empty()
        };
        if window_empty {
            self.destroy_window_id(window_id);
            return Ok(());
        } else {
            let win = self.window_mut(t.session, t.window);
            if pane_idx < win.active {
                win.active -= 1;
            } else if win.active >= win.panes.len() {
                win.active = win.panes.len() - 1;
            }
            win.last_pane = win
                .last_pane
                .and_then(|last| (last != pane_idx).then_some(last - usize::from(last > pane_idx)));
            resize_panes_to_layout(win)?;
        }
        if self.sessions.iter().any(|session| session.id == session_id) {
            self.invalidate_session(
                session_id,
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
            );
        } else {
            self.invalidate_session(session_id, RenderInvalidation::SESSION_GONE);
        }
        self.notify_window(WindowHook::LayoutChanged, window_id);
        Ok(())
    }

    /// `kill-pane -a -t target`: remove every pane in the target window except
    /// the target itself. The survivor is renumbered to pane index zero.
    pub fn kill_other_panes(&mut self, target: &str) -> io::Result<()> {
        let t = self.resolve(target).ok_or_else(|| {
            let pane = split_pane_target(target).1.unwrap_or(target);
            pane_not_found(pane)
        })?;
        let session_id = self.sessions[t.session].id;
        let win = self.window_mut(t.session, t.window);
        win.zoomed = false;
        let survivor = win.panes.remove(t.pane);
        let survivor_id = survivor.id;
        win.panes.clear();
        win.panes.push(survivor);
        win.layout.keep_only(survivor_id);
        resize_panes_to_layout(win)?;
        win.active = 0;
        win.last_pane = None;
        self.invalidate_session(
            session_id,
            RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
        );
        Ok(())
    }

    /// `last-pane [-t window]`: make the previously-active pane active. Errors
    /// with `no last pane` when the window has no recorded previous pane.
    pub fn last_pane(&mut self, target: &str) -> io::Result<()> {
        let t = self.resolve_window_target(target)?;
        let session_id = self.sessions[t.session].id;
        let win = self.window_mut(t.session, t.window);
        match win.last_pane_index() {
            Some(lp) => {
                win.last_pane = Some(win.active);
                win.active = lp;
                self.invalidate_session(
                    session_id,
                    RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
                );
                Ok(())
            }
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "no last pane".to_string(),
            )),
        }
    }

    /// `last-pane -d`/`-e` (and `select-pane -l` with those flags): set or
    /// clear the input-off flag on the previously-active pane. tmux toggles
    /// the flag without switching to the pane.
    pub(crate) fn set_last_pane_input_off(
        &mut self,
        target: &str,
        input_off: bool,
    ) -> io::Result<()> {
        let t = self.resolve_window_target(target)?;
        let win = self.window_mut(t.session, t.window);
        match win.last_pane_index() {
            Some(lp) => {
                win.panes[lp].input_off = input_off;
                Ok(())
            }
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "no last pane".to_string(),
            )),
        }
    }

    /// `move-pane -s src -t dst`: move the source pane into the destination
    /// window after the target in pane-list order, becoming active. In tmux
    /// 3.7b, `-b` changes layout geometry but not this observable list order.
    /// Emptying the source window destroys it. Reports `can't find pane` on a
    /// miss.
    pub(crate) fn move_pane(
        &mut self,
        src: &str,
        dst: &str,
        before: bool,
        select: bool,
        direction: SplitDirection,
        new_size: Option<u16>,
    ) -> io::Result<()> {
        let s = self.resolve(src).ok_or_else(|| pane_not_found(src))?;
        let (dst_window_target, _) = split_pane_target(dst);
        // Validate the destination window exists before mutating the source.
        let initial_dst = self
            .resolve_window(dst_window_target)
            .map_err(|_| pane_not_found(dst))?;
        let src_session_id = self.sessions[s.session].id;
        let dst_session_id = self.sessions[initial_dst.session].id;
        let source_window_id = self.sessions[s.session].windows[s.window].id;
        // Capture the target pane id before removing the source because indices
        // and even window positions can shift during the mutation.
        let target_id = self
            .resolve(dst)
            .map(|t| self.window(t.session, t.window).panes[t.pane].id);
        let source_pane_id = self.window(s.session, s.window).panes[s.pane].id;
        if target_id == Some(source_pane_id) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "source and target panes must be different",
            ));
        }
        // Pull the pane out of the source window.
        let (node, source_empty) = {
            let win = self.window_mut(s.session, s.window);
            let node = win.panes.remove(s.pane);
            win.layout.remove(node.id);
            let empty = win.panes.is_empty();
            if !empty {
                if win.active >= win.panes.len() {
                    win.active = win.panes.len() - 1;
                }
                win.last_pane = None;
                resize_panes_to_layout(win)?;
            }
            (node, empty)
        };
        if self.marked_pane_id == Some(node.id) {
            self.marked_pane_id = None;
        }
        if source_empty {
            self.destroy_window_id(source_window_id);
        }
        // The destination window position may have shifted if the source window
        // (before it) was removed from the same session; re-resolve by target.
        let d = self
            .resolve_window(dst_window_target)
            .map_err(|_| pane_not_found(dst))?;
        let win = self.window_mut(d.session, d.window);
        let layout_target_id =
            target_id.unwrap_or_else(|| win.panes.get(win.active).map_or(node.id, |pane| pane.id));
        // tmux 3.7b inserts after the target in its pane queue even for `-b`.
        let insert_at = match target_id {
            Some(id) => win
                .panes
                .iter()
                .position(|p| p.id == id)
                .map(|index| index + 1)
                .unwrap_or(win.panes.len()),
            None => win.panes.len(),
        };
        let old_active = win.active;
        if !win
            .layout
            .split_sized(layout_target_id, node.id, direction, before, new_size)
        {
            return Err(io::Error::other("target pane is absent from layout"));
        }
        win.panes.insert(insert_at, node);
        let shifted_old = if insert_at <= old_active {
            old_active + 1
        } else {
            old_active
        };
        let window_id = win.id;
        let active_changed = if select {
            win.last_pane = Some(shifted_old);
            win.set_active_pane(insert_at);
            true
        } else {
            win.active = shifted_old;
            false
        };
        resize_panes_to_layout(win)?;
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
        if active_changed {
            self.notify_window(WindowHook::PaneChanged, window_id);
        }
        self.notify_window(WindowHook::LayoutChanged, window_id);
        Ok(())
    }

    /// Dump the plain-text screen of the pane named by `target` (`capture-pane`).
    pub fn dump_pane(&self, target: &str) -> io::Result<String> {
        let t = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        Ok(self.window(t.session, t.window).panes[t.pane].pane.dump())
    }

    /// `swap-pane -s src -t dst`: exchange the two panes' positions. Same-window
    /// swaps are a simple positional swap; cross-window swaps exchange the pane
    /// nodes. Reports `can't find pane` on a miss.
    pub fn swap_pane(&mut self, src: &str, dst: &str) -> io::Result<()> {
        let a = self.resolve(src).ok_or_else(|| pane_not_found(src))?;
        let b = self.resolve(dst).ok_or_else(|| pane_not_found(dst))?;
        let src_session_id = self.sessions[a.session].id;
        let dst_session_id = self.sessions[b.session].id;
        let first_window_id = self.sessions[a.session].windows[a.window].id;
        let second_window_id = self.sessions[b.session].windows[b.window].id;
        let first_id = self.window(a.session, a.window).panes[a.pane].id;
        let second_id = self.window(b.session, b.window).panes[b.pane].id;
        if first_window_id == second_window_id {
            let win = self.window_mut(a.session, a.window);
            win.panes.swap(a.pane, b.pane);
            win.layout.swap_panes(first_id, second_id);
            resize_panes_to_layout(win)?;
            self.invalidate_session(src_session_id, RenderInvalidation::LAYOUT);
            return Ok(());
        }
        // Cross-window/session: exchange the two pane nodes by value. Windows
        // live in the arena, so temporarily remove both to obtain disjoint
        // mutable ownership without tying it to session placement.
        let mut first = self
            .windows
            .remove(&first_window_id)
            .expect("window present");
        let mut second = self
            .windows
            .remove(&second_window_id)
            .expect("window present");
        let first_active = first.active == a.pane;
        let second_active = second.active == b.pane;
        std::mem::swap(&mut first.panes[a.pane], &mut second.panes[b.pane]);
        first.layout.replace_pane(first_id, second_id);
        second.layout.replace_pane(second_id, first_id);
        if first_active {
            second.active = b.pane;
        }
        if second_active {
            first.active = a.pane;
        }
        resize_panes_to_layout(&mut first)?;
        resize_panes_to_layout(&mut second)?;
        self.windows.insert(first_window_id, first);
        self.windows.insert(second_window_id, second);
        self.invalidate_session(src_session_id, RenderInvalidation::LAYOUT);
        if dst_session_id != src_session_id {
            self.invalidate_session(dst_session_id, RenderInvalidation::LAYOUT);
        }
        Ok(())
    }

    /// `swap-pane -U`/`-D`: swap the target pane (default active) with its previous
    /// (`-U`) or next (`-D`) neighbour in the same window, wrapping at the ends.
    /// The targeted pane stays active, following its swap to the neighbour's former
    /// index (matching tmux, which keeps the active flag on the target pane).
    pub(in crate::server) fn swap_pane_neighbour(
        &mut self,
        target: &str,
        down: bool,
        select: bool,
    ) -> io::Result<()> {
        let t = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[t.session].id;
        let win = self.window_mut(t.session, t.window);
        let n = win.panes.len();
        if n > 1 {
            let neighbour = if down {
                (t.pane + 1) % n
            } else {
                (t.pane + n - 1) % n
            };
            let first_id = win.panes[t.pane].id;
            let second_id = win.panes[neighbour].id;
            win.panes.swap(t.pane, neighbour);
            win.layout.swap_panes(first_id, second_id);
            resize_panes_to_layout(win)?;
            if select {
                win.active = neighbour;
            }
            self.invalidate_session(session_id, RenderInvalidation::LAYOUT);
        }
        Ok(())
    }

    /// `break-pane`: move a pane into a new window in `dst`. A one-pane source
    /// moves its existing window link; a multi-pane source creates a new
    /// physical window. `relative` is `Some(true)` for `-a`, `Some(false)` for
    /// `-b`, and `None` for an explicit or automatically allocated index.
    pub fn break_pane(
        &mut self,
        src: &str,
        dst: &str,
        name: Option<&str>,
        select: bool,
        relative: Option<bool>,
    ) -> io::Result<Target> {
        let t = self.resolve(src).ok_or_else(|| pane_not_found(src))?;
        let source_session_id = self.sessions[t.session].id;
        let source_window_id = self.sessions[t.session].windows[t.window].id;
        let source_index = self.sessions[t.session].windows[t.window].index;
        let source_session_name = self.sessions[t.session].name.clone();
        let source_target = format!("{source_session_name}:{source_index}");
        let (dst_name, requested_index) = parse_index_target(dst);
        let dst_session = match dst_name {
            Some(name) => self.session_pos(name).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("can't find session: {name}"),
                )
            })?,
            None => self.current_session_pos()?,
        };
        let dst_session_id = self.sessions[dst_session].id;
        let anchor = requested_index
            .and_then(|index| {
                self.sessions[dst_session]
                    .windows
                    .iter()
                    .any(|link| link.index == index)
                    .then_some(index)
            })
            .unwrap_or_else(|| {
                self.sessions[dst_session].windows[self.sessions[dst_session].active].index
            });

        if self.window(t.session, t.window).panes.len() == 1 {
            let destination = if relative.is_some() {
                format!("{}:{anchor}", self.sessions[dst_session].name)
            } else {
                let index = match requested_index {
                    Some(index) => index,
                    None => {
                        let mut index = self.sessions[dst_session]
                            .options(&self.global_options)
                            .get("base-index")
                            .and_then(|value| value.parse().ok())
                            .unwrap_or(0);
                        while self.sessions[dst_session]
                            .windows
                            .iter()
                            .any(|link| link.index == index)
                        {
                            index += 1;
                        }
                        index
                    }
                };
                format!("{}:{index}", self.sessions[dst_session].name)
            };
            if let Some(after) = relative {
                self.move_window_relative(&source_target, &destination, after, select)?;
            } else {
                self.move_window(&source_target, &destination, select)?;
            }
            if let Some(name) = name {
                if let Some(window) = self.windows.get_mut(&source_window_id) {
                    window.name = name.to_string();
                    window.options.set("automatic-rename", "off");
                }
            }
            let session = self
                .sessions
                .iter()
                .position(|session| session.id == dst_session_id)
                .ok_or_else(|| io::Error::other("destination session was destroyed"))?;
            let window = self.sessions[session]
                .windows
                .iter()
                .position(|link| link.id == source_window_id)
                .ok_or_else(|| io::Error::other("moved window is not in destination"))?;
            return Ok(Target {
                session,
                window,
                pane: 0,
            });
        }
        let index = if let Some(after) = relative {
            let desired = if after { anchor + 1 } else { anchor };
            let mut free = desired;
            while self.sessions[dst_session]
                .windows
                .iter()
                .any(|link| link.index == free)
            {
                free += 1;
            }
            for link in self.links_mut(dst_session) {
                if link.index >= desired && link.index < free {
                    link.index += 1;
                }
            }
            desired
        } else if let Some(index) = requested_index {
            if self.sessions[dst_session]
                .windows
                .iter()
                .any(|link| link.index == index)
            {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!("index in use: {index}"),
                ));
            }
            index
        } else {
            let mut index = self.sessions[dst_session]
                .options(&self.global_options)
                .get("base-index")
                .and_then(|value| value.parse().ok())
                .unwrap_or(0);
            while self.sessions[dst_session]
                .windows
                .iter()
                .any(|link| link.index == index)
            {
                index += 1;
            }
            index
        };
        let source_size = (self.sessions[t.session].cols, self.sessions[t.session].rows);
        let node = self.window_mut(t.session, t.window).panes.remove(t.pane);
        let node_id = node.id;
        self.window_mut(t.session, t.window).layout.remove(node_id);
        // Fix the source window's active pane after the removal.
        let win = self.window_mut(t.session, t.window);
        if win.active >= win.panes.len() {
            win.active = win.panes.len() - 1;
        }
        win.last_pane = None;
        let window_id = self.next_window_id;
        self.next_window_id += 1;
        let winlink_id = self.next_winlink_id;
        self.next_winlink_id += 1;
        let pos = self.sessions[dst_session]
            .windows
            .iter()
            .take_while(|w| w.index < index)
            .count();
        let mut window_options = OptionSet::default();
        if name.is_some() {
            window_options.set("automatic-rename", "off");
        }
        self.windows.insert(
            window_id,
            Window {
                id: window_id,
                name: name.unwrap_or("").to_string(),
                panes: vec![node],
                active: 0,
                last_pane: None,
                zoomed: false,
                activity_epoch: now_epoch(),
                name_time_micros: 0,
                name_in_mode: false,
                scrollbars_on_left: false,
                cols: source_size.0,
                rows: source_size.1,
                manual_size: source_size,
                latest_client: None,
                pending_size: None,
                xpixel: DEFAULT_XPIXEL,
                ypixel: DEFAULT_YPIXEL,
                layout: LayoutCell::pane(node_id, source_size.0, source_size.1),
                last_layout: None,
                old_layout: None,
                last_new_pane_x: 0,
                last_new_pane_y: 0,
                // tmux's `window_create` raises activity, so a monitor turned on
                // later still sees the window as having been active.
                pending_alerts: ALERT_ACTIVITY,
                options: window_options,
            },
        );
        self.insert_link(
            dst_session,
            pos,
            Winlink {
                link_id: winlink_id,
                index,
                id: window_id,
                alert_flags: 0,
            },
            select,
        );
        self.invalidate_session(
            source_session_id,
            RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
        );
        if dst_session_id != source_session_id {
            self.invalidate_session(
                dst_session_id,
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
            );
        }
        Ok(Target {
            session: dst_session,
            window: pos,
            pane: 0,
        })
    }

    /// Toggle the target window's zoom state (`resize-pane -Z`), returning the
    /// new state. Errors via [`Self::resolve_window`] on a bad target.
    pub fn toggle_zoom(&mut self, target: &str) -> io::Result<bool> {
        let t = self.resolve_window(target)?;
        let session_id = self.sessions[t.session].id;
        let win = self.window_mut(t.session, t.window);
        win.zoomed = !win.zoomed;
        let zoomed = win.zoomed;
        self.invalidate_session(session_id, RenderInvalidation::LAYOUT);
        Ok(zoomed)
    }

    pub(crate) fn push_zoom_at(&mut self, session: usize, window: usize) {
        let session_id = self.sessions[session].id;
        let win = self.window_mut(session, window);
        if win.zoomed {
            win.zoomed = false;
            self.invalidate_session(session_id, RenderInvalidation::LAYOUT);
        }
    }

    pub(crate) fn pop_zoom_at(&mut self, session: usize, window: usize, keep: bool) {
        let session_id = self.sessions[session].id;
        let win = self.window_mut(session, window);
        if win.zoomed != keep {
            win.zoomed = keep;
            self.invalidate_session(session_id, RenderInvalidation::LAYOUT);
        }
    }

    pub(crate) fn push_zoom(&mut self, target: &str) -> io::Result<()> {
        let t = self.resolve_window_target(target)?;
        self.push_zoom_at(t.session, t.window);
        Ok(())
    }

    pub(crate) fn pop_zoom(&mut self, target: &str, keep: bool) -> io::Result<()> {
        let t = self.resolve_window_target(target)?;
        self.pop_zoom_at(t.session, t.window, keep);
        Ok(())
    }

    pub(crate) fn resize_pane(
        &mut self,
        target: &str,
        direction: SplitDirection,
        forward: bool,
        amount: u16,
    ) -> io::Result<()> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[resolved.session].id;
        let pane_id = self.window(resolved.session, resolved.window).panes[resolved.pane].id;
        let window = self.window_mut(resolved.session, resolved.window);
        window.zoomed = false;
        window
            .layout
            .resize_pane_toward(pane_id, direction, forward, amount.max(1));
        resize_panes_to_layout(window)?;
        self.invalidate_session(
            session_id,
            RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
        );
        Ok(())
    }

    pub(crate) fn resize_pane_to(
        &mut self,
        target: &str,
        direction: SplitDirection,
        size: u16,
    ) -> io::Result<()> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[resolved.session].id;
        let pane_id = self.window(resolved.session, resolved.window).panes[resolved.pane].id;
        let window = self.window_mut(resolved.session, resolved.window);
        window.zoomed = false;
        window.layout.resize_pane_to(pane_id, direction, size);
        resize_panes_to_layout(window)?;
        self.invalidate_session(
            session_id,
            RenderInvalidation::LAYOUT | RenderInvalidation::STATUS,
        );
        Ok(())
    }

    /// Move or resize a floating pane by the border the pointer grabbed —
    /// tmux's `cmd_resize_pane_mouse_update_floating`.
    ///
    /// `grabbed` is where the button went down, which names the border, and
    /// `now` is where the pointer has reached. The top border moves the pane
    /// whole; every other one resizes it from the edge that stayed put.
    pub(crate) fn drag_floating_pane(
        &mut self,
        target: &str,
        grabbed: (u16, u16),
        now: (u16, u16),
    ) -> bool {
        const MINIMUM: i32 = 1;
        let Some(resolved) = self.resolve(target) else {
            return false;
        };
        let window = self.window_mut(resolved.session, resolved.window);
        let Some(rect) = window.panes[resolved.pane].floating else {
            return false;
        };
        let (sx, sy) = (i32::from(rect.width), i32::from(rect.height));
        let (xoff, yoff) = (i32::from(rect.left), i32::from(rect.top));
        let (left, right) = (xoff - 1, xoff + sx);
        let (lx, ly) = (i32::from(grabbed.0), i32::from(grabbed.1));
        let (x, y) = (i32::from(now.0), i32::from(now.1));

        let on_left = lx == left || lx == left + 1;
        let on_right = lx == right || lx == right + 1;
        let (mut width, mut height) = (sx, sy);
        let (mut new_left, mut new_top) = (xoff, yoff);
        if ly == yoff - 1 && on_left {
            width = (sx + (lx - x)).max(MINIMUM);
            height = (sy + (ly - y)).max(MINIMUM);
            new_left = x + 1;
            new_top = y + 1;
        } else if ly == yoff - 1 && on_right {
            width = (x - xoff).max(MINIMUM);
            height = (sy + (ly - y)).max(MINIMUM);
            new_top = y + 1;
        } else if ly == yoff + sy && on_left {
            width = (sx + (lx - x)).max(MINIMUM);
            height = y - yoff;
            if height < MINIMUM {
                return false;
            }
            new_left = x + 1;
        } else if ly == yoff + sy && on_right {
            width = (x - xoff).max(MINIMUM);
            height = (y - yoff).max(MINIMUM);
        } else if lx == right {
            width = x - xoff;
            if width < MINIMUM {
                return false;
            }
        } else if lx == left {
            width = sx + (lx - x);
            if width < MINIMUM {
                return false;
            }
            new_left = x + 1;
        } else if ly == yoff + sy {
            height = y - yoff;
            if height < MINIMUM {
                return false;
            }
        } else if ly == yoff - 1 {
            // The top border moves the pane instead of resizing it.
            new_left = xoff + (x - lx);
            new_top = y + 1;
        } else {
            return false;
        }

        let node = &mut window.panes[resolved.pane];
        node.floating = Some(PaneRect {
            left: new_left.max(0) as u16,
            top: new_top.max(0) as u16,
            width: width.max(MINIMUM) as u16,
            height: height.max(MINIMUM) as u16,
        });
        let _ = resize_panes_to_layout(window);
        let session_id = self.sessions[resolved.session].id;
        self.invalidate_session(session_id, RenderInvalidation::LAYOUT);
        true
    }

    /// The active window's name in `session` (tmux's `#W`). Used to fill in the
    /// `confirm-before` prompt text for `C-b &` (kill-window), matching tmux's
    /// default `"kill-window #W? (y/n)"`.
    pub(crate) fn active_window_name(&self, session_name: &str) -> Option<String> {
        let session = self.session_index(session_name)?;
        let sess = &self.sessions[session];
        Some(self.window(session, sess.active).name.clone())
    }

    /// The active pane's index in `session` (tmux's `#P`). Used to fill in the
    /// `confirm-before` prompt text for `C-b x` (kill-pane), matching tmux's
    /// default `"kill-pane #P? (y/n)"`.
    pub(crate) fn active_pane_index(&self, session_name: &str) -> Option<usize> {
        let session = self.session_index(session_name)?;
        let sess = &self.sessions[session];
        Some(self.window(session, sess.active).active)
    }

    /// Stable identities for every pane displayed in the active window plus
    /// the active pane's position. Attach clients use this as the key for their
    /// shared compositor-output subscription.
    pub(crate) fn active_window_pane_identities(
        &self,
        session_name: &str,
    ) -> Option<(Vec<(u32, u64)>, usize)> {
        let session = self.session_index(session_name)?;
        let sess = &self.sessions[session];
        let win = self.window(session, sess.active);
        Some((
            win.panes
                .iter()
                .map(|pane| (pane.id, pane.pane.runtime_id()))
                .collect(),
            win.active,
        ))
    }

    #[cfg(test)]
    pub(crate) fn subscribe_active_pane_output(
        &self,
        session_name: &str,
    ) -> io::Result<crate::server::pane::OutputSubscription> {
        self.active_pane(session_name)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no active pane"))?
            .subscribe_output()
    }

    pub(crate) fn subscribe_active_window_output(
        &self,
        session_name: &str,
    ) -> io::Result<crate::server::pane::OutputSubscription> {
        let (window, active) = self.active_window_panes(session_name)?;
        let active_pane = window
            .panes
            .get(active)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no active pane"))?;
        crate::server::pane::OutputSubscription::for_panes(
            window.panes.iter().map(|pane| &pane.pane),
            &active_pane.pane,
        )
    }

    /// The first `rows` lines of a target's pane, as a mode tree's preview
    /// shows them.
    pub(crate) fn pane_preview_text(&self, target: &str, rows: usize) -> io::Result<String> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let node = &self.window(resolved.session, resolved.window).panes[resolved.pane];
        let text = node.pane.visible_screen()?;
        Ok(text.lines().take(rows).collect::<Vec<_>>().join("\n"))
    }

    /// tmux's `window_pane_search`: the 1-based row of a pane's visible screen
    /// whose trailing-space-trimmed text matches `term`, or 0 when none does.
    pub(crate) fn search_pane_screen(
        &self,
        pane_id: u32,
        term: &str,
        regex: bool,
        ignore_case: bool,
    ) -> u32 {
        let Some(node) = self
            .windows
            .values()
            .flat_map(|window| window.panes.iter())
            .find(|node| node.id == pane_id)
        else {
            return 0;
        };
        let Ok(screen) = node.pane.visible_screen() else {
            return 0;
        };
        let needle = if ignore_case {
            term.to_lowercase()
        } else {
            term.to_owned()
        };
        for (index, line) in screen.lines().enumerate() {
            let line = line.trim_end();
            let line = if ignore_case {
                line.to_lowercase()
            } else {
                line.to_owned()
            };
            // tmux wraps a plain term in `*…*` and fnmatches it; a regular
            // expression is matched unanchored. hmux has no regex engine here,
            // so `r` falls back to the same containment test — which agrees for
            // every pattern that is a literal.
            let _ = regex;
            if line.contains(&needle) {
                return index as u32 + 1;
            }
        }
        0
    }

    /// Send input bytes to the active pane of a session.
    pub fn input_to_active_pane(&self, session_name: &str, bytes: &[u8]) -> io::Result<()> {
        if let Some(pane) = self.active_pane(session_name) {
            pane.input(bytes)
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("no active pane for session: {session_name}"),
            ))
        }
    }

    /// Send input bytes to the pane selected by a tmux target expression.
    pub(crate) fn input_to_pane(&self, target: &str, bytes: &[u8]) -> io::Result<()> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let window = self.window(resolved.session, resolved.window);
        if window.panes[resolved.pane].input_off {
            return Ok(());
        }
        let synchronized = window
            .options(&self.global_options)
            .get("synchronize-panes")
            .is_some_and(|value| value == "on" || value == "1");
        if synchronized {
            // tmux's `window_pane_copy_key` fans out only to panes
            // `window_pane_visible` accepts, and a zoomed window shows only its
            // active pane.
            for (index, pane) in window.panes.iter().enumerate() {
                if pane.input_off || (window.zoomed && index != window.active) {
                    continue;
                }
                pane.pane.input(bytes)?;
            }
            Ok(())
        } else {
            window.panes[resolved.pane].pane.input(bytes)
        }
    }

    pub(crate) fn input_mouse_to_pane(&self, target: &str, event: MouseEvent) -> io::Result<()> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let pane = &self.window(resolved.session, resolved.window).panes[resolved.pane];
        if pane.input_off {
            return Ok(());
        }
        let bytes = pane.pane.encode_mouse(event);
        pane.pane.input(&bytes)
    }

    pub(crate) fn pane_bracketed_paste(&self, target: &str) -> io::Result<bool> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        Ok(
            self.window(resolved.session, resolved.window).panes[resolved.pane]
                .pane
                .bracketed_paste_enabled(),
        )
    }

    /// Spell one key the way the pane's own terminal type and modes describe
    /// it, per [`crate::server::input_keys`].
    pub(crate) fn encode_pane_key(
        &self,
        target: &str,
        key: PaneKey,
    ) -> io::Result<PaneKeyEncoding> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let state = self.window(resolved.session, resolved.window).panes[resolved.pane]
            .pane
            .key_state();
        Ok(input_keys::encode(
            key,
            self.pane_key_modes(state),
            self.pane_key_options(target),
        ))
    }

    /// Apply the `extended-keys` option to what a pane asked for.
    ///
    /// tmux latches this when the pane's request arrives, so a request made
    /// while the option was `off` stays refused even if the option is turned on
    /// afterwards. Reading the option here instead means a mid-session change
    /// applies to requests the pane already made — the only difference, and one
    /// that needs both an application that asked for extended keys and a user
    /// who changed the option afterwards. See README.md.
    fn pane_key_modes(&self, state: PaneKeyState) -> PaneKeyModes {
        let extended = match self.server_options().get("extended-keys") {
            // `off`: requests are refused outright.
            Some("on") => state.extended_request,
            // `always` keeps a pane in mode 1 even when it asked for nothing,
            // which is what tmux puts a freshly reset screen into.
            Some("always") => match state.extended_request {
                ExtendedKeys::All => ExtendedKeys::All,
                ExtendedKeys::Standard | ExtendedKeys::Off => ExtendedKeys::Standard,
            },
            _ => ExtendedKeys::Off,
        };
        PaneKeyModes {
            cursor_keys: state.cursor_keys,
            application_keypad: state.application_keypad,
            bracketed_paste: state.bracketed_paste,
            extended,
        }
    }

    /// `#{pane_key_mode}`: the name tmux gives the pane's effective
    /// extended-key state, so it follows the same `extended-keys` resolution
    /// the encoder uses rather than the pane's raw request.
    pub(crate) fn pane_key_mode_name(&self, pane: &PaneNode) -> &'static str {
        match self.pane_key_modes(pane.pane.key_state()).extended {
            ExtendedKeys::All => "Ext 2",
            ExtendedKeys::Standard => "Ext 1",
            ExtendedKeys::Off => "VT10x",
        }
    }

    fn pane_key_options(&self, target: &str) -> PaneKeyOptions {
        let options = self.server_options();
        let backspace = match options.get("backspace") {
            // tmux's stored default is the bare `DEL` code; `C-?` is only how
            // it prints that byte back. Reading the printed spelling as a key
            // name would give `'?'` with control instead, which `C-BSpace`
            // then encodes as `DEL` rather than failing the way tmux does.
            // A user who sets `backspace C-?` explicitly is indistinguishable
            // here and gets the default's behaviour. See README.md.
            Some("C-?") | None => PaneKeyOptions::default().backspace,
            Some(name) => parse_key_name(name).unwrap_or(PaneKeyOptions::default().backspace),
        };
        let extended_keys_format = match options.get("extended-keys-format") {
            Some("csi-u") => ExtendedKeysFormat::CsiU,
            _ => ExtendedKeysFormat::Xterm,
        };
        // tmux 3.7b retains this window option in its catalog, but its input
        // key path no longer consults it. Resolve it here so the no-op
        // compatibility behavior is explicit in the runtime path.
        let xterm_keys = self
            .window_options(target)
            .ok()
            .and_then(|options| options.get("xterm-keys"))
            .is_none_or(|value| value != "off");
        PaneKeyOptions {
            backspace,
            extended_keys_format,
            xterm_keys,
        }
    }

    pub(crate) fn reset_pane_terminal(&self, target: &str) -> io::Result<()> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        self.window(resolved.session, resolved.window).panes[resolved.pane]
            .pane
            .reset_terminal()
    }

    pub(crate) fn clear_pane_history(&mut self, target: &str) -> io::Result<()> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[resolved.session].id;
        let node = &mut self.window_mut(resolved.session, resolved.window).panes[resolved.pane];
        node.pane.clear_history()?;
        node.mode = None;
        node.copy = None;
        node.mode_view = None;
        self.invalidate_session(session_id, RenderInvalidation::RESET_MODE);
        Ok(())
    }

    pub(crate) fn append_view_output(&mut self, target: &str, output: &[u8]) -> io::Result<()> {
        if output.is_empty() {
            return Ok(());
        }
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[resolved.session].id;
        let node = &mut self.window_mut(resolved.session, resolved.window).panes[resolved.pane];
        let (cols, rows) = node.pane.size();
        let (mut combined, previous_view) = match (
            &node.mode,
            node.copy.as_ref().map(|copy| (&copy.backing, copy)),
        ) {
            (Some(mode), Some((CopyBacking::ViewOutput(previous), copy)))
                if mode == "view-mode" =>
            {
                let top = copy.grid.scrollback_rows.saturating_sub(copy.scroll);
                (previous.clone(), Some((top, copy.cursor.clone())))
            }
            _ => (Vec::new(), None),
        };
        combined.extend_from_slice(output);
        let mut copy = view_copy_state(combined, cols, rows)?;
        if let Some((top, cursor)) = previous_view {
            copy.scroll = copy.grid.scrollback_rows.saturating_sub(top);
            copy.cursor.row = cursor.row.min(copy.grid.rows.len().saturating_sub(1));
            copy.cursor.col = cursor.col.min(copy.grid.cols.saturating_sub(1) as usize);
            copy.desired_col = copy.cursor.col;
        }
        node.mode = Some("view-mode".to_string());
        node.copy = Some(copy);
        node.mode_view = None;
        self.invalidate_session(session_id, RenderInvalidation::MODE);
        Ok(())
    }

    pub(crate) fn respawn_pane_process(
        &mut self,
        target: &str,
        argv: Option<Vec<String>>,
        cwd: Option<PathBuf>,
    ) -> io::Result<()> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session_id = self.sessions[resolved.session].id;
        let (cols, rows, saved, pane_id) = {
            let node = &self.window(resolved.session, resolved.window).panes[resolved.pane];
            let (cols, rows) = node.pane.size();
            (cols, rows, node.pane.spawn_spec(), node.id)
        };
        let spec = match argv {
            // The pane keeps its id across a respawn, so a saved spec already
            // names it and only a rebuilt argv has a placeholder to fill.
            Some(argv) if !argv.is_empty() => PaneSpawnSpec {
                argv: fill_spawn_ids(&argv, pane_id, session_id),
                cwd,
            },
            _ => match saved {
                Some(spec) => spec,
                None => return Ok(()),
            },
        };
        let pane = Pane::spawn_from_spec(&spec, cols, rows)?;
        let node = &mut self.window_mut(resolved.session, resolved.window).panes[resolved.pane];
        node.pane = pane;
        node.mode = None;
        node.copy = None;
        node.mode_view = None;
        self.invalidate_session(
            session_id,
            RenderInvalidation::LAYOUT | RenderInvalidation::STATUS | RenderInvalidation::MODE,
        );
        Ok(())
    }

    pub(crate) fn respawn_window_process(
        &mut self,
        target: &str,
        argv: Option<Vec<String>>,
        cwd: Option<PathBuf>,
    ) -> io::Result<()> {
        let resolved = self.resolve(target).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("can't find window: {target}"),
            )
        })?;
        let window_id = self.sessions[resolved.session].windows[resolved.window].id;
        let affected_sessions = self
            .sessions
            .iter()
            .filter(|session| session.windows.iter().any(|link| link.id == window_id))
            .map(|session| session.id)
            .collect::<Vec<_>>();
        if argv.as_ref().is_none_or(Vec::is_empty) {
            let replacements = self
                .window(resolved.session, resolved.window)
                .panes
                .iter()
                .map(|node| {
                    let Some(spec) = node.pane.spawn_spec() else {
                        return Ok(None);
                    };
                    let (cols, rows) = node.pane.size();
                    Pane::spawn_from_spec(&spec, cols, rows).map(Some)
                })
                .collect::<io::Result<Vec<_>>>()?;
            let window = self.window_mut(resolved.session, resolved.window);
            for (node, pane) in window.panes.iter_mut().zip(replacements) {
                if let Some(pane) = pane {
                    node.pane = pane;
                    node.mode = None;
                    node.copy = None;
                    node.mode_view = None;
                }
            }
            for session_id in affected_sessions {
                self.invalidate_session(
                    session_id,
                    RenderInvalidation::LAYOUT
                        | RenderInvalidation::STATUS
                        | RenderInvalidation::MODE,
                );
            }
            return Ok(());
        }
        let argv = argv.expect("nonempty argv checked above");
        let session_id = self.sessions[resolved.session].id;
        let pane = {
            let window = self.window(resolved.session, resolved.window);
            let (cols, rows) = (window.cols, window.rows);
            // The surviving pane keeps its id, which its rebuilt environment
            // still has to be told.
            let id = window.panes[resolved.pane].id;
            let spec = PaneSpawnSpec {
                argv: fill_spawn_ids(&argv, id, session_id),
                cwd,
            };
            Pane::spawn_from_spec(&spec, cols, rows)?
        };
        let window = self.window_mut(resolved.session, resolved.window);
        let id = window.panes[resolved.pane].id;
        window.panes.clear();
        window.panes.push(PaneNode {
            id,
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
        });
        window.layout.keep_only(id);
        resize_panes_to_layout(window)?;
        window.active = 0;
        window.last_pane = None;
        for session_id in affected_sessions {
            self.invalidate_session(
                session_id,
                RenderInvalidation::LAYOUT | RenderInvalidation::STATUS | RenderInvalidation::MODE,
            );
        }
        Ok(())
    }

    /// Instrumented input path for the attach latency monitor. The public input
    /// method above keeps its existing API while this reports whether bytes
    /// reached the non-blocking PTY immediately, remained queued, or were
    /// dropped because the bounded queue was full.
    pub(crate) fn input_to_active_pane_with_stats(
        &self,
        session_name: &str,
        bytes: &[u8],
    ) -> io::Result<crate::server::pane::PaneInputStats> {
        if let Some(pane) = self.active_pane(session_name) {
            pane.input_with_stats(bytes)
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("no active pane for session: {session_name}"),
            ))
        }
    }

    /// Drain terminal queries waiting to be sent to the attached client.
    pub fn take_active_pane_terminal_queries(
        &self,
        session_name: &str,
    ) -> io::Result<Vec<Vec<u8>>> {
        let pane = self
            .active_pane(session_name)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no active pane"))?;
        Ok(pane.take_terminal_queries())
    }

    /// Dump the active pane as VT sequences, if present.
    pub fn dump_active_pane_vt(&self, session_name: &str) -> io::Result<Vec<u8>> {
        let pane = self
            .active_pane(session_name)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no active pane"))?;
        Ok(pane.dump_vt())
    }

    pub(crate) fn dump_active_pane_viewport_vt(
        &self,
        session_name: &str,
        scroll_offset: usize,
        visible_rows: usize,
    ) -> io::Result<(Vec<u8>, usize)> {
        let pane = self
            .active_pane(session_name)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no active pane"))?;
        Ok(pane.dump_viewport_vt(scroll_offset, visible_rows))
    }

    pub(crate) fn active_window_panes(&self, session_name: &str) -> io::Result<(&Window, usize)> {
        let session = self
            .session_index(session_name)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no active session"))?;
        let sess = &self.sessions[session];
        let win = self.window(session, sess.active);
        Ok((win, win.active))
    }

    /// The mouse mode an attached client's own terminal should be in, which is
    /// what decides whether any mouse report reaches hmux at all.
    ///
    /// tmux recomputes this per client on every pass of its event loop
    /// (`server_client_reset_state`) and writes to the terminal only when the
    /// answer changed. With `mouse off` the client mirrors what the active
    /// pane asked for, so a program that tracks the mouse itself keeps being
    /// fed. With `mouse on` hmux reads the mouse instead, and needs button
    /// mode for drags — all-motion when a pane in the window wants that or
    /// when `focus-follows-mouse` needs to see the pointer cross a border.
    ///
    /// `overlay` is tmux's `c->overlay_draw`/`prompt_string` case, where the
    /// mode comes from the overlay's own screen rather than from a pane.
    pub(crate) fn client_tty_mouse_mode(&self, session_name: &str, overlay: bool) -> TtyMouseMode {
        let enabled = |name: &str| {
            self.option_for_target(session_name, name)
                .is_some_and(|value| value == "on")
        };
        let mut mode = if overlay {
            TtyMouseMode::Off
        } else {
            match self
                .active_pane(session_name)
                .and_then(|pane| pane.mouse_modes().tracking)
            {
                Some(MouseTrackingMode::Standard) => TtyMouseMode::Standard,
                Some(MouseTrackingMode::Button) => TtyMouseMode::Button,
                Some(MouseTrackingMode::All) => TtyMouseMode::All,
                None => TtyMouseMode::Off,
            }
        };
        if !enabled("mouse") {
            return mode;
        }
        if !overlay {
            // The pane's own choice stops mattering here, with one exception:
            // any pane in the window asking for all-motion keeps the client in
            // all-motion, so a drag that leaves the pane still reports.
            let any_all = self
                .active_window_panes(session_name)
                .is_ok_and(|(window, _)| {
                    window.panes.iter().any(|node| {
                        node.pane.mouse_modes().tracking == Some(MouseTrackingMode::All)
                    })
                });
            mode = if any_all {
                TtyMouseMode::All
            } else {
                TtyMouseMode::Off
            };
        }
        if enabled("focus-follows-mouse") {
            TtyMouseMode::All
        } else if mode == TtyMouseMode::All {
            mode
        } else {
            TtyMouseMode::Button
        }
    }

    /// Whether the active pane's cursor is visible (DEC mode 25), for the
    /// compositor to mirror onto the client tty.
    pub fn active_pane_cursor_visible(&self, session_name: &str) -> io::Result<bool> {
        let pane = self
            .active_pane(session_name)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no active pane"))?;
        Ok(pane.cursor_visible())
    }

    /// DECSCUSR parameter selected by the active pane.
    pub fn active_pane_cursor_shape(&self, session_name: &str) -> io::Result<u8> {
        let pane = self
            .active_pane(session_name)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no active pane"))?;
        let shape = pane.cursor_shape();
        if shape != 0 {
            return Ok(shape);
        }
        // tmux's `screen_set_default_cursor`: a pane that has applied no
        // DECSCUSR of its own shows whatever `cursor-style` asks for.
        Ok(cursor_style_parameter(
            self.option_for_target(session_name, "cursor-style")
                .unwrap_or("default"),
        ))
    }

    /// The colour a pane's cursor is drawn in: its own `OSC 12` when it set
    /// one, else the `cursor-colour` option — tmux's `s->default_ccolour`.
    pub(crate) fn active_pane_cursor_colour(&self, session_name: &str) -> Option<String> {
        let pane = self.active_pane(session_name)?;
        let own = pane.osc_state().cursor_colour;
        if !own.is_empty() && own != "none" {
            return Some(own);
        }
        self.option_for_target(session_name, "cursor-colour")
            .filter(|value| !value.is_empty() && *value != "none")
            .map(str::to_owned)
    }

    /// How many leading scrollback (history) rows the active pane's VT dump
    /// carries before the visible viewport. The compositor skips these so it
    /// paints the on-screen rows, not the oldest history (see `report.md`).
    pub fn active_pane_scrollback_rows(&self, session_name: &str) -> io::Result<usize> {
        let pane = self
            .active_pane(session_name)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no active pane"))?;
        Ok(pane.scrollback_rows())
    }

    /// How many leading scrollback (history) rows the pane named by `target`
    /// carries before its viewport. `capture-pane -p` skips these to print the
    /// visible screen rather than the top of history.
    pub fn pane_scrollback_rows(&self, target: &str) -> io::Result<usize> {
        let t = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        Ok(self.window(t.session, t.window).panes[t.pane]
            .pane
            .scrollback_rows())
    }

    /// Dump the active pane as plain text (for debugging / tests).
    pub fn dump_active_pane_plain(&self, session_name: &str) -> io::Result<String> {
        let pane = self
            .active_pane(session_name)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no active pane"))?;
        Ok(pane.dump())
    }
}
