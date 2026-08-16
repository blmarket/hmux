//! `ServerState`'s view of the option tables: resolving an option for a
//! target, writing one, and deciding what a write has to repaint.
//!
//! The option storage itself is `server::options`; this module is the bridge
//! between it and the session/window/pane tree.

use std::collections::BTreeMap;
use std::io;

use super::target::pane_not_found;
use super::{CustomizeOption, RenderInvalidation, ServerState};
use crate::server::options::{
    is_array_option, option_scope, parse_option_name, GlobalOptions, OptionScope, OptionsView,
};
use hmux_vt::{set_codepoint_widths, set_variation_selector_always_wide};

impl ServerState {
    pub(crate) fn option_changed(&mut self, name: &str) {
        if name == "monitor-silence" {
            self.reset_silence_timers();
        }
        if name == "automatic-rename" {
            // tmux's `options_push_changes`: turning the option on has to name
            // the window even though nothing in the pane moved.
            for window in self.windows.values() {
                if let Some(node) = window.panes.get(window.active) {
                    node.pane.observation_state().note_changed();
                }
            }
        }
        if option_affects_render(name) {
            self.invalidate_all_clients(option_invalidation(name));
        }
        if matches!(
            name,
            "alternate-screen"
                | "allow-set-title"
                | "allow-passthrough"
                | "history-limit"
                | "input-buffer-size"
                | "scroll-on-clear"
        ) {
            self.refresh_pane_options();
        }
        // The width policy is server-wide in tmux — one cache, rebuilt whenever
        // either option is set — so it is pushed rather than looked up. Only
        // characters written after this see the change, as in tmux: a cell
        // already in the grid keeps the width it was placed with.
        if name == "codepoint-widths" {
            let options = OptionsView::one(self.global_options.server());
            set_codepoint_widths(options.array_values(name));
        }
        if name == "variation-selector-always-wide" {
            let options = OptionsView::one(self.global_options.server());
            set_variation_selector_always_wide(options.get(name) != Some("off"));
        }
    }

    pub(super) fn server_option_is_on(&self, name: &str, default: bool) -> bool {
        match self.server_options().get(name) {
            Some("on" | "yes" | "true" | "1") => true,
            Some("off" | "no" | "false" | "0") => false,
            Some(_) | None => default,
        }
    }

    pub(crate) fn global_options(&self) -> &GlobalOptions {
        &self.global_options
    }

    pub(crate) fn global_options_mut(&mut self) -> &mut GlobalOptions {
        &mut self.global_options
    }

    pub(crate) fn server_options(&self) -> OptionsView<'_> {
        OptionsView::one(self.global_options.server())
    }

    pub(crate) fn command_aliases(&self) -> Vec<(String, String)> {
        self.server_options()
            .iter_effective()
            .filter_map(|(name, value)| {
                parse_option_name(name)
                    .is_some_and(|(base, index)| base == "command-alias" && index.is_some())
                    .then(|| value.split_once('='))
                    .flatten()
                    // Only the `=` separator is required: an entry may carry an
                    // empty command, which matches the name and expands to no
                    // commands at all.
                    .filter(|(alias, _)| !alias.is_empty())
                    .map(|(alias, command)| (alias.to_string(), command.to_string()))
            })
            .collect()
    }

    pub(crate) fn session_options(&self, target: &str) -> io::Result<OptionsView<'_>> {
        let target = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        Ok(self.sessions[target.session].options(&self.global_options))
    }

    pub(crate) fn window_options(&self, target: &str) -> io::Result<OptionsView<'_>> {
        let target = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        Ok(self
            .window(target.session, target.window)
            .options(&self.global_options))
    }

    pub(crate) fn pane_options(&self, target: &str) -> io::Result<OptionsView<'_>> {
        let target = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let window = self.window(target.session, target.window);
        Ok(window.panes[target.pane].options(window, &self.global_options))
    }

    pub(crate) fn option_for_target<'a>(&'a self, target: &str, name: &str) -> Option<&'a str> {
        let base = parse_option_name(name)
            .map(|(base, _)| base)
            .unwrap_or(name);
        match option_scope(base)? {
            OptionScope::Server => self.server_options().get(name),
            OptionScope::Session => self.session_options(target).ok()?.get(name),
            OptionScope::Window => self.window_options(target).ok()?.get(name),
            OptionScope::WindowPane => self.pane_options(target).ok()?.get(name),
        }
    }

    pub(crate) fn format_option_entries<'a>(
        &'a self,
        target: &str,
    ) -> io::Result<impl Iterator<Item = (&'a str, &'a str)> + use<'a>> {
        let target = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session = &self.sessions[target.session];
        let window = self.window(target.session, target.window);
        let pane = &window.panes[target.pane];
        let mut entries = BTreeMap::new();
        for (name, value) in session.options(&self.global_options).iter_effective() {
            entries.insert(name, value);
        }
        for (name, value) in pane.options(window, &self.global_options).iter_effective() {
            entries.insert(name, value);
        }
        for (name, value) in self.server_options().iter_effective() {
            entries.insert(name, value);
        }
        Ok(entries.into_iter())
    }

    /// The option rows customize mode shows, grouped under the headings tmux's
    /// `window_customize_build` uses — one per option table, each entry
    /// carrying the scope text `window_customize_scope_text` prints for it
    /// (empty for a global one).
    pub(crate) fn customize_option_sections(
        &self,
        target: &str,
    ) -> io::Result<Vec<(&'static str, Vec<CustomizeOption>)>> {
        let resolved = self.resolve(target).ok_or_else(|| pane_not_found(target))?;
        let session = &self.sessions[resolved.session];
        let window = self.window(resolved.session, resolved.window);
        let pane = &window.panes[resolved.pane];
        let collect =
            |view: OptionsView<'_>, scope: String| {
                let mut entries = Vec::new();
                let mut arrays = BTreeMap::new();
                for (name, value) in view.iter_effective() {
                    let (base, index) = parse_option_name(name).unwrap_or((name, None));
                    if is_array_option(base) {
                        arrays
                            .entry(base.to_owned())
                            .and_modify(|has_entries| {
                                *has_entries |= index.is_some() && !value.is_empty();
                            })
                            .or_insert(index.is_some() && !value.is_empty());
                    } else {
                        entries.push(CustomizeOption {
                            name: name.to_owned(),
                            value: value.to_owned(),
                            scope: scope.clone(),
                            is_array: false,
                            array_has_entries: false,
                        });
                    }
                }
                entries.extend(arrays.into_iter().map(|(name, array_has_entries)| {
                    CustomizeOption {
                        name,
                        value: String::new(),
                        scope: scope.clone(),
                        is_array: true,
                        array_has_entries,
                    }
                }));
                entries.sort_by(|left, right| left.name.cmp(&right.name));
                entries
            };
        let mut sections = Vec::new();
        sections.push((
            "Server Options",
            collect(
                OptionsView::one(self.global_options.server()),
                String::new(),
            ),
        ));
        let mut session_options = collect(
            OptionsView::one(self.global_options.session()),
            String::new(),
        );
        let session_overrides = collect(
            OptionsView::one(session.option_overrides()),
            format!("session {}", session.name),
        );
        // tmux 3.7b's customize-mode builder resolves a session-only user
        // option through the global session table when it builds this list.
        // Keep that presentation quirk here without changing option lookup or
        // storage, which continue to expose the real session-local value.
        if let Some(global_user) = session_options
            .iter()
            .find(|entry| entry.name.starts_with('@'))
            .cloned()
        {
            if session_overrides
                .iter()
                .any(|entry| entry.name.starts_with('@'))
            {
                let insert_at = session_options
                    .iter()
                    .position(|entry| entry.name == global_user.name)
                    .map_or(session_options.len(), |index| index + 1);
                session_options.insert(insert_at, global_user);
            }
            session_options.extend(session_overrides);
        } else {
            session_options.extend(session_overrides);
        }
        sections.push(("Session Options", session_options));
        let mut window_options = collect(
            OptionsView::one(self.global_options.window()),
            String::new(),
        );
        window_options.extend(collect(
            OptionsView::one(window.option_overrides()),
            format!("window {}", session.windows[resolved.window].index),
        ));
        window_options.extend(collect(
            OptionsView::one(pane.option_overrides()),
            format!("pane {}", resolved.pane),
        ));
        sections.push(("Window & Pane Options", window_options));
        Ok(sections)
    }

    /// Set a built-in option in its global table. Command execution uses the
    /// object-scoped accessors directly; this remains a compact engine/test API.
    #[cfg(test)]
    pub(crate) fn set_global_option(&mut self, key: &str, value: &str) {
        let base = parse_option_name(key).map(|(name, _)| name).unwrap_or(key);
        let Some(scope) = option_scope(base) else {
            return;
        };
        let table = self.global_options.for_scope_mut(scope);
        let changed = table.get(key) != Some(value);
        table.set(key, value);
        if changed && option_affects_render(key) {
            self.invalidate_all_clients(option_invalidation(key));
        }
    }

    pub(crate) fn push_config_error(&mut self, error: String) {
        self.pending_config_errors.push(error);
    }

    pub(crate) fn take_config_errors(&mut self) -> Vec<String> {
        std::mem::take(&mut self.pending_config_errors)
    }
}

fn option_affects_render(key: &str) -> bool {
    matches!(
        key,
        "pane-border-indicators"
            | "set-titles"
            | "set-titles-string"
            | "status"
            | "status-interval"
            | "status-justify"
            | "status-left"
            | "status-left-length"
            | "status-left-style"
            | "status-position"
            | "status-right"
            | "status-right-length"
            | "status-right-style"
            | "status-style"
            | "status-format"
            | "terminal-features"
            | "terminal-overrides"
            | "window-pane-current-status-format"
            | "window-pane-status-format"
            | "pane-status-current-style"
            | "pane-status-style"
            | "session-status-current-style"
            | "session-status-style"
            | "window-status-activity-style"
            | "window-status-bell-style"
            | "window-status-current-format"
            | "window-status-current-style"
            | "window-status-format"
            | "window-status-last-style"
            | "window-status-separator"
            | "window-status-style"
    ) || key.starts_with("status-format[")
        || key.starts_with("terminal-features[")
        || key.starts_with("terminal-overrides[")
}

fn option_invalidation(key: &str) -> RenderInvalidation {
    if matches!(key, "terminal-features" | "terminal-overrides")
        || key.starts_with("terminal-features[")
        || key.starts_with("terminal-overrides[")
    {
        RenderInvalidation::TERMINAL | RenderInvalidation::STATUS
    } else if key == "status" || key == "status-position" {
        RenderInvalidation::STATUS | RenderInvalidation::LAYOUT
    } else if key == "pane-border-indicators" {
        RenderInvalidation::LAYOUT
    } else {
        RenderInvalidation::STATUS
    }
}
