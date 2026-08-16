//! The built-in tmux **option catalog** — the set of valid server/session/window
//! option names and the stable default values `show-options` reports for the
//! ones that aren't explicitly set.
//!
//! Why this exists: without it the native engine can't tell a real option
//! (`status`, `mode-keys`, …) from a typo, so `set-option NAME` silently accepts
//! garbage and `show-options -g NAME` returns nothing where real tmux prints the
//! option's default. Modeling the catalog closes that gap: unknown non-user
//! names now produce tmux's `invalid option: NAME` diagnostic, and a global
//! (`-g`/`-s`) query of an unset-but-valid option returns its default.
//!
//! Each catalog entry also belongs to its tmux server, session, window, or
//! window/pane scope. Runtime values live in sparse [`OptionSet`] instances;
//! [`OptionsView`] resolves those sets through the same inheritance order as
//! tmux. Defaults are the build constants of the tmux this was generated
//! against (tmux 3.7b). Environment-derived shell and key-mode defaults are
//! initialized when [`GlobalOptions`] is created. Array defaults are stored as
//! indexed entries (or an explicit empty array marker) so complete listings and
//! global unsets reproduce the tmux catalog. Regenerate from a running tmux if
//! the pinned version changes.

use std::collections::{BTreeMap, BTreeSet};

/// One table of option values. Local object tables are sparse; global tables
/// contain the modeled defaults for their scope.
#[derive(Clone, Debug, Default)]
pub(crate) struct OptionSet {
    entries: BTreeMap<String, String>,
}

impl OptionSet {
    pub(crate) fn get(&self, name: &str) -> Option<&str> {
        self.entries.get(name).map(String::as_str)
    }

    pub(crate) fn contains(&self, name: &str) -> bool {
        self.entries.contains_key(name)
    }

    pub(crate) fn contains_array(&self, name: &str) -> bool {
        self.entries.contains_key(name)
            || self
                .entries
                .range(format!("{name}[")..)
                .next()
                .is_some_and(|(entry, _)| entry.starts_with(&format!("{name}[")))
    }

    pub(crate) fn set(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.entries.insert(name.into(), value.into());
    }

    pub(crate) fn append(&mut self, name: &str, value: &str) {
        self.entries
            .entry(name.to_string())
            .or_default()
            .push_str(value);
    }

    pub(crate) fn remove(&mut self, name: &str) -> bool {
        self.entries.remove(name).is_some()
    }

    pub(crate) fn clear_array(&mut self, base: &str) -> bool {
        let before = self.entries.len();
        self.entries.retain(|name, _| {
            parse_option_name(name)
                .map(|(name, index)| name != base || index.is_none())
                .unwrap_or(true)
        });
        self.entries.remove(base);
        before != self.entries.len()
    }

    pub(crate) fn next_array_index(&self, base: &str) -> u32 {
        (0..=i32::MAX as u32)
            .find(|index| !self.entries.contains_key(&format!("{base}[{index}]")))
            .unwrap_or(i32::MAX as u32)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.entries
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
    }
}

/// Borrowed effective option lookup. Layers are ordered from most local to
/// least local. Array options are inherited as a whole: once a layer contains
/// any entry for an array, a missing index does not fall through to its parent.
#[derive(Clone, Copy)]
pub(crate) struct OptionsView<'a> {
    layers: [Option<&'a OptionSet>; 3],
}

impl<'a> OptionsView<'a> {
    pub(crate) fn one(first: &'a OptionSet) -> Self {
        Self {
            layers: [Some(first), None, None],
        }
    }

    pub(crate) fn two(first: &'a OptionSet, second: &'a OptionSet) -> Self {
        Self {
            layers: [Some(first), Some(second), None],
        }
    }

    pub(crate) fn three(first: &'a OptionSet, second: &'a OptionSet, third: &'a OptionSet) -> Self {
        Self {
            layers: [Some(first), Some(second), Some(third)],
        }
    }

    pub(crate) fn get(&self, name: &str) -> Option<&'a str> {
        let (base, _) = parse_option_name(name).unwrap_or((name, None));
        let array = option_kind(base) == Some(OptionKind::Array);
        for layer in self.layers.into_iter().flatten() {
            if let Some(value) = layer.get(name) {
                return Some(value);
            }
            if array && layer.contains_array(base) {
                return None;
            }
        }
        None
    }

    pub(crate) fn contains_array(&self, name: &str) -> bool {
        self.layers
            .into_iter()
            .flatten()
            .any(|layer| layer.contains_array(name))
    }

    /// The entries of an array option with the indexes they are stored at, for
    /// the options whose index *is* the meaning — a palette entry, a user key.
    pub(crate) fn array_entries(&self, base: &str) -> Vec<(u32, &'a str)> {
        let mut entries = self
            .iter_effective()
            .filter_map(|(name, value)| {
                let (name, index) = parse_option_name(name)?;
                (name == base && !value.is_empty()).then_some((index.unwrap_or(0), value))
            })
            .collect::<Vec<_>>();
        entries.sort_unstable_by_key(|(index, _)| *index);
        entries
    }

    /// The entries of an array option in index order, as
    /// `options_array_first`/`_next` walk them: gaps are skipped, and an array
    /// with no entries yields nothing.
    pub(crate) fn array_values(&self, base: &str) -> Vec<&'a str> {
        self.array_entries(base)
            .into_iter()
            .map(|(_, value)| value)
            .collect()
    }

    pub(crate) fn iter_effective(&self) -> impl Iterator<Item = (&'a str, &'a str)> {
        let mut entries: BTreeMap<&'a str, &'a str> = BTreeMap::new();
        for layer in self.layers.into_iter().rev().flatten() {
            // A layer holding any entry of an array replaces that array whole,
            // so the bases it names have to go from what the layers behind it
            // contributed. There is nothing to drop while nothing has been
            // contributed — which is the first layer, every time, and the one
            // carrying the whole default catalog.
            if !entries.is_empty() {
                let array_bases: BTreeSet<&str> = layer
                    .iter()
                    .filter_map(|(name, _)| {
                        let (base, _) = parse_option_name(name)?;
                        (option_kind(base) == Some(OptionKind::Array)).then_some(base)
                    })
                    .collect();
                if !array_bases.is_empty() {
                    entries.retain(|name, _| {
                        parse_option_name(name)
                            .map(|(base, _)| !array_bases.contains(base))
                            .unwrap_or(true)
                    });
                }
            }
            for (name, value) in layer.iter() {
                entries.insert(name, value);
            }
        }
        entries.into_iter()
    }
}

/// The three independent global option tables created by tmux.
#[derive(Clone, Debug)]
pub(crate) struct GlobalOptions {
    server: OptionSet,
    session: OptionSet,
    window: OptionSet,
}

impl GlobalOptions {
    pub(crate) fn new() -> Self {
        let mut options = Self {
            server: OptionSet::default(),
            session: OptionSet::default(),
            window: OptionSet::default(),
        };
        for &(name, value) in OPTION_DEFAULTS {
            options
                .for_scope_mut(option_scope(name).expect("default option has scope"))
                .set(name, value);
        }
        options
            .window
            .set("mode-keys", mode_keys_default().to_string());
        options
            .session
            .set("status-keys", mode_keys_default().to_string());
        options.session.set("default-command", String::new());
        options.session.set(
            "default-shell",
            std::env::var("SHELL")
                .ok()
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "/bin/sh".to_string()),
        );
        options.server.set(
            "editor",
            std::env::var("VISUAL")
                .ok()
                .filter(|value| !value.is_empty())
                .or_else(|| std::env::var("EDITOR").ok())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "vi".to_string()),
        );
        for name in OPTION_ARRAYS {
            if let Some(values) = option_array_default(name) {
                install_array_default(
                    options.for_scope_mut(option_scope(name).expect("array option has scope")),
                    name,
                    values,
                );
            }
        }
        options
    }

    pub(crate) fn server(&self) -> &OptionSet {
        &self.server
    }

    pub(crate) fn session(&self) -> &OptionSet {
        &self.session
    }

    pub(crate) fn window(&self) -> &OptionSet {
        &self.window
    }

    pub(crate) fn for_scope_mut(&mut self, scope: OptionScope) -> &mut OptionSet {
        match scope {
            OptionScope::Server => &mut self.server,
            OptionScope::Session => &mut self.session,
            OptionScope::Window | OptionScope::WindowPane => &mut self.window,
        }
    }
}

impl Default for GlobalOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// `(name, default)` for every option with a stable, environment-independent
/// scalar default. A global (`-g`/`-s`) `show-options` of an unset option in
/// this table returns the default; `set-option` accepts the name.
const OPTION_DEFAULTS: &[(&str, &str)] = &[
    ("activity-action", "other"),
    ("aggressive-resize", "off"),
    ("allow-passthrough", "off"),
    ("allow-rename", "off"),
    ("allow-set-title", "on"),
    ("alternate-screen", "on"),
    ("assume-paste-time", "1"),
    ("automatic-rename", "on"),
    (
        "automatic-rename-format",
        "#{?pane_in_mode,[tmux],#{pane_current_command}}#{?pane_dead,[dead],}",
    ),
    ("backspace", "C-?"),
    ("base-index", "0"),
    ("bell-action", "any"),
    ("buffer-limit", "50"),
    ("clock-mode-colour", "blue"),
    ("clock-mode-style", "24"),
    ("copy-mode-current-line-number-style", "fg=yellow"),
    ("copy-mode-current-match-style", "bg=magenta,fg=black"),
    ("copy-mode-line-number-style", "fg=white,dim"),
    ("copy-mode-line-numbers", "off"),
    ("copy-mode-mark-style", "bg=red,fg=black"),
    ("copy-mode-match-style", "bg=cyan,fg=black"),
    ("copy-mode-position-style", "#{E:mode-style}"),
    ("copy-mode-selection-style", "#{E:mode-style}"),
    (
        "copy-mode-position-format",
        "#[align=right]#{t/p:top_line_time}#{?#{e|>:#{top_line_time},0}, ,}[#{copy_position}/#{copy_position_limit}]#{?search_timed_out, (timed out),#{?search_count, (#{search_count}#{?search_count_partial,+,} results),}}",
    ),
    ("copy-command", ""),
    ("cursor-colour", "none"),
    ("cursor-style", "default"),
    ("default-client-command", "new-session"),
    ("default-size", "80x24"),
    ("default-terminal", "tmux-256color"),
    ("destroy-unattached", "off"),
    ("detach-on-destroy", "on"),
    ("display-panes-active-colour", "red"),
    ("display-panes-colour", "blue"),
    ("display-panes-time", "1000"),
    ("display-time", "750"),
    ("escape-time", "10"),
    // tmux defaults this to `on`, which would shut an hmux daemon down before
    // its first client arrives: hmux's server is not forked by the client that
    // creates the first session, so it starts empty. See
    // `EXIT_EMPTY_AFTER_SESSION`.
    ("exit-empty", EXIT_EMPTY_AFTER_SESSION),
    ("exit-unattached", "off"),
    ("extended-keys", "off"),
    ("extended-keys-format", "xterm"),
    ("focus-events", "off"),
    ("focus-follows-mouse", "off"),
    ("fill-character", ""),
    ("get-clipboard", "buffer"),
    ("history-limit", "2000"),
    ("history-file", ""),
    ("initial-repeat-time", "0"),
    ("input-buffer-size", "1048576"),
    ("key-table", "root"),
    ("lock-after-time", "0"),
    ("lock-command", "lock -np"),
    ("main-pane-height", "24"),
    ("main-pane-width", "80"),
    ("menu-border-lines", "single"),
    ("menu-border-style", "default"),
    ("menu-selected-style", "bg=yellow,fg=black"),
    ("menu-style", "default"),
    ("message-command-style", "bg=black,fg=yellow,fill=black"),
    (
        "message-format",
        "#[#{?#{command_prompt},#{E:message-command-style},#{E:message-style}}]#{message}",
    ),
    ("message-limit", "1000"),
    ("message-line", "0"),
    ("message-style", "bg=yellow,fg=black,fill=yellow"),
    ("mode-style", "noattr,bg=yellow,fg=black"),
    ("monitor-activity", "off"),
    ("monitor-bell", "on"),
    ("monitor-silence", "0"),
    ("mouse", "off"),
    ("other-pane-height", "0"),
    ("other-pane-width", "0"),
    (
        "pane-active-border-style",
        "#{?pane_in_mode,fg=yellow,#{?synchronize-panes,fg=red,fg=green}}",
    ),
    ("pane-base-index", "0"),
    (
        "pane-border-format",
        "#{?pane_active,#[reverse],}#{pane_index}#[default] \"#{pane_title}\"#{?#{mouse},#[align=right]#[range=control|8][#{?#{window_zoomed_flag},u,z}]#[norange]#[range=control|9][x]#[norange],}",
    ),
    ("pane-border-indicators", "colour"),
    ("pane-border-lines", "single"),
    ("pane-border-status", "off"),
    ("pane-border-style", "default"),
    ("pane-scrollbars", "off"),
    ("pane-scrollbars-position", "right"),
    ("pane-scrollbars-style", "bg=black,fg=white,width=1,pad=0"),
    ("pane-status-current-style", "default"),
    ("pane-status-style", "default"),
    ("popup-border-lines", "single"),
    ("popup-border-style", "default"),
    ("popup-style", "default"),
    ("prefix", "C-b"),
    ("prefix2", "None"),
    ("prefix-timeout", "0"),
    ("prompt-command-cursor-style", "default"),
    ("prompt-cursor-colour", "none"),
    ("prompt-cursor-style", "default"),
    ("prompt-history-limit", "100"),
    ("remain-on-exit", "off"),
    (
        "remain-on-exit-format",
        "Pane is dead (#{?#{!=:#{pane_dead_status},},status #{pane_dead_status},}#{?#{!=:#{pane_dead_signal},},signal #{pane_dead_signal},}, #{t:pane_dead_time})",
    ),
    ("renumber-windows", "off"),
    ("repeat-time", "500"),
    ("scroll-on-clear", "on"),
    ("session-status-current-style", "default"),
    ("session-status-style", "default"),
    ("set-clipboard", "external"),
    ("set-titles", "off"),
    (
        "set-titles-string",
        "#S:#I:#W - \"#T\" #{session_alerts}",
    ),
    ("silence-action", "other"),
    ("status", "on"),
    ("status-bg", "default"),
    ("status-fg", "default"),
    ("status-interval", "15"),
    ("status-justify", "left"),
    ("status-left", "[#{session_name}] "),
    ("status-left-length", "10"),
    ("status-left-style", "default"),
    ("status-position", "bottom"),
    (
        "status-right",
        "#{?window_bigger,[#{window_offset_x}#,#{window_offset_y}] ,}\"#{=21:pane_title}\" %H:%M %d-%b-%y",
    ),
    ("status-right-length", "40"),
    ("status-right-style", "default"),
    ("status-style", "bg=green,fg=black"),
    ("synchronize-panes", "off"),
    ("tiled-layout-max-columns", "0"),
    (
        "tree-mode-preview-format",
        "#{?pane_format,#{pane_index}:#{pane_title},#{window_index}:#{window_name}}",
    ),
    (
        "tree-mode-preview-style",
        "fg=#{?#{||:#{&&:#{pane_format},#{pane_active}},#{&&:#{window_format},#{window_active}}},#{display-panes-active-colour},#{display-panes-colour}}",
    ),
    ("variation-selector-always-wide", "on"),
    ("visual-activity", "off"),
    ("visual-bell", "off"),
    ("visual-silence", "off"),
    ("window-active-style", "default"),
    ("window-size", "latest"),
    ("window-status-activity-style", "reverse"),
    ("window-status-bell-style", "reverse"),
    (
        "window-status-current-format",
        "#I:#{?#{m:*fable*,#{pane_agent_model}},#[bg=red],#{?#{m:*luna*,#{pane_agent_model}},#[bg=brightblue],}}#{pane_state_emoji}#[default] #{?pane_current_path,#{b:pane_current_path},#{b:session_path}}#{?window_flags,#{window_flags}, }",
    ),
    ("window-status-current-style", "default"),
    (
        "window-status-format",
        "#I:#{?#{m:*fable*,#{pane_agent_model}},#[bg=red],#{?#{m:*luna*,#{pane_agent_model}},#[bg=brightblue],}}#{pane_state_emoji}#[default] #{?pane_current_path,#{b:pane_current_path},#{b:session_path}}#{?window_flags,#{window_flags}, }",
    ),
    ("window-status-last-style", "default"),
    ("window-status-separator", " "),
    ("window-status-style", "default"),
    (
        "window-pane-current-status-format",
        "#P:[#T]#{?pane_flags,#{pane_flags}, }",
    ),
    (
        "window-pane-status-format",
        "#P:[#T]#{?pane_flags,#{pane_flags}, }",
    ),
    ("window-style", "default"),
    ("word-separators", "!\"#$%&'()*+,-./:;<=>?@[\\]^`{|}~"),
    ("wrap-search", "on"),
    ("xterm-keys", "on"),
];

/// Valid option names that are not part of the stable scalar-default table.
/// This includes arrays, format strings, and environment-derived options.
const OPTION_VALID_ONLY: &[&str] = &[
    "codepoint-widths",
    "command-alias",
    "default-command",
    "default-shell",
    "editor",
    "mode-keys",
    "pane-colours",
    "status-format",
    "status-keys",
    "terminal-features",
    "terminal-overrides",
    "update-environment",
    "user-keys",
];

pub(crate) const OPTIONS_CATALOG_ORDER: &[&str] = &[
    "backspace",
    "buffer-limit",
    "command-alias",
    "codepoint-widths",
    "copy-command",
    "cursor-colour",
    "cursor-style",
    "default-client-command",
    "default-terminal",
    "editor",
    "escape-time",
    "exit-empty",
    "exit-unattached",
    "extended-keys",
    "extended-keys-format",
    "focus-events",
    "get-clipboard",
    "history-file",
    "input-buffer-size",
    "menu-style",
    "menu-selected-style",
    "menu-border-style",
    "menu-border-lines",
    "message-limit",
    "prefix-timeout",
    "prompt-history-limit",
    "set-clipboard",
    "terminal-overrides",
    "terminal-features",
    "user-keys",
    "variation-selector-always-wide",
    "activity-action",
    "assume-paste-time",
    "base-index",
    "bell-action",
    "default-command",
    "default-shell",
    "default-size",
    "destroy-unattached",
    "detach-on-destroy",
    "display-panes-active-colour",
    "display-panes-colour",
    "display-panes-time",
    "display-time",
    "focus-follows-mouse",
    "history-limit",
    "initial-repeat-time",
    "key-table",
    "lock-after-time",
    "lock-command",
    "message-command-style",
    "message-format",
    "message-line",
    "message-style",
    "mouse",
    "prefix",
    "prefix2",
    "prompt-cursor-colour",
    "prompt-cursor-style",
    "prompt-command-cursor-colour",
    "prompt-command-cursor-style",
    "renumber-windows",
    "repeat-time",
    "set-titles",
    "set-titles-string",
    "silence-action",
    "status",
    "status-bg",
    "status-fg",
    "status-format",
    "status-interval",
    "status-justify",
    "status-keys",
    "status-left",
    "status-left-length",
    "status-left-style",
    "status-position",
    "status-right",
    "status-right-length",
    "status-right-style",
    "status-style",
    "update-environment",
    "visual-activity",
    "visual-bell",
    "visual-silence",
    "word-separators",
    "pane-status-current-style",
    "pane-status-style",
    "session-status-current-style",
    "session-status-style",
    "aggressive-resize",
    "allow-passthrough",
    "allow-rename",
    "allow-set-title",
    "alternate-screen",
    "automatic-rename",
    "automatic-rename-format",
    "clock-mode-colour",
    "clock-mode-style",
    "copy-mode-match-style",
    "copy-mode-current-match-style",
    "copy-mode-mark-style",
    "copy-mode-position-format",
    "copy-mode-position-style",
    "copy-mode-selection-style",
    "copy-mode-current-line-number-style",
    "copy-mode-line-number-style",
    "copy-mode-line-numbers",
    "fill-character",
    "main-pane-height",
    "main-pane-width",
    "mode-keys",
    "mode-style",
    "monitor-activity",
    "monitor-bell",
    "monitor-silence",
    "other-pane-height",
    "other-pane-width",
    "pane-active-border-style",
    "pane-base-index",
    "pane-border-format",
    "pane-border-indicators",
    "pane-border-lines",
    "pane-border-status",
    "pane-border-style",
    "pane-colours",
    "pane-scrollbars",
    "pane-scrollbars-style",
    "pane-scrollbars-position",
    "popup-style",
    "popup-border-style",
    "popup-border-lines",
    "remain-on-exit",
    "remain-on-exit-format",
    "scroll-on-clear",
    "synchronize-panes",
    "tiled-layout-max-columns",
    "tree-mode-preview-format",
    "tree-mode-preview-style",
    "window-active-style",
    "window-pane-current-status-format",
    "window-pane-status-format",
    "window-size",
    "window-style",
    "window-status-activity-style",
    "window-status-bell-style",
    "window-status-current-format",
    "window-status-current-style",
    "window-status-format",
    "window-status-last-style",
    "window-status-separator",
    "window-status-style",
    "wrap-search",
    "xterm-keys",
];

pub(crate) fn option_names() -> impl Iterator<Item = &'static str> {
    OPTION_DEFAULTS
        .iter()
        .map(|(name, _)| *name)
        .chain(OPTION_VALID_ONLY.iter().copied())
}

const COMMAND_ALIAS_DEFAULTS: &[&str] = &[
    "split-pane=split-window",
    "splitp=split-window",
    "server-info=show-messages -JT",
    "info=show-messages -JT",
    "choose-window=choose-tree -w",
    "choose-session=choose-tree -s",
];

const TERMINAL_FEATURE_DEFAULTS: &[&str] = &[
    "xterm*:clipboard:ccolour:cstyle:focus:title",
    "screen*:title",
    "rxvt*:ignorefkeys",
];

const UPDATE_ENVIRONMENT_DEFAULTS: &[&str] = &[
    "DISPLAY",
    "KRB5CCNAME",
    "MSYSTEM",
    "SSH_ASKPASS",
    "SSH_AUTH_SOCK",
    "SSH_AGENT_PID",
    "SSH_CONNECTION",
    "WAYLAND_DISPLAY",
    "WINDOWID",
    "XAUTHORITY",
    "XDG_CURRENT_DESKTOP",
    "XDG_SESSION_DESKTOP",
    "XDG_SESSION_TYPE",
];

const STATUS_FORMAT_DEFAULTS: &[&str] = &[
    r###"#[align=left range=left #{E:status-left-style}]#[push-default]#{T;=/#{status-left-length}:status-left}#[pop-default]#[norange default]#[list=on align=#{status-justify}]#[list=left-marker]<#[list=right-marker]>#[list=on]#{W:#[range=window|#{window_index} #{E:window-status-style}#{?#{&&:#{window_last_flag},#{!=:#{E:window-status-last-style},default}}, #{E:window-status-last-style},}#{?#{&&:#{window_bell_flag},#{!=:#{E:window-status-bell-style},default}}, #{E:window-status-bell-style},#{?#{&&:#{||:#{window_activity_flag},#{window_silence_flag}},#{!=:#{E:window-status-activity-style},default}}, #{E:window-status-activity-style},}}]#[push-default]#{T:window-status-format}#[pop-default]#[norange default]#{?loop_last_flag,,#{E:window-status-separator}},#[range=window|#{window_index} list=focus #{?#{!=:#{E:window-status-current-style},default},#{E:window-status-current-style},#{E:window-status-style}}#{?#{&&:#{window_last_flag},#{!=:#{E:window-status-last-style},default}}, #{E:window-status-last-style},}#{?#{&&:#{window_bell_flag},#{!=:#{E:window-status-bell-style},default}}, #{E:window-status-bell-style},#{?#{&&:#{||:#{window_activity_flag},#{window_silence_flag}},#{!=:#{E:window-status-activity-style},default}}, #{E:window-status-activity-style},}}]#[push-default]#{T:window-status-current-format}#[pop-default]#[norange list=on default]#{?loop_last_flag,,#{E:window-status-separator}}}#[nolist align=right range=right #{E:status-right-style}]#[push-default]#{T;=/#{status-right-length}:status-right}#[pop-default]#[norange default]"###,
    r###"#[align=left]#{R: ,#{n:#{session_name}}}P: #[norange default]#[list=on align=#{status-justify}]#[list=left-marker]<#[list=right-marker]>#[list=on]#{P:#[range=pane|#{pane_id} #{E:pane-status-style}]#[push-default]#{T:window-pane-status-format}#[pop-default]#[norange list=on default]  ,#[range=pane|#{pane_id} list=focus #{?#{!=:#{E:pane-status-current-style},default},#{E:pane-status-current-style},#{E:pane-status-style}}]#[push-default]#{T:window-pane-current-status-format}#[pop-default]#[norange list=on default] }"###,
    r###"#[align=left]#{R: ,#{n:#{session_name}}}S: #[norange default]#[list=on align=#{status-justify}]#[list=left-marker]<#[list=right-marker]>#[list=on]#{S:#[range=session|#{session_id} #{E:session-status-style}]#[push-default]#S#{session_alert}#[pop-default]#[norange list=on default]  ,#[range=session|#{session_id} list=focus #{?#{!=:#{E:session-status-current-style},default},#{E:session-status-current-style},#{E:session-status-style}}]#[push-default]#S*#{session_alert}#[pop-default]#[norange list=on default] }"###,
];

/// Whether an option name — indexed or not — belongs to an array option.
pub(crate) fn is_array_option(name: &str) -> bool {
    let (base, _) = parse_option_name(name).unwrap_or((name, None));
    option_kind(base) == Some(OptionKind::Array)
}

fn option_array_default(name: &str) -> Option<&'static [&'static str]> {
    match name {
        "codepoint-widths" | "pane-colours" | "user-keys" => Some(&[]),
        "command-alias" => Some(COMMAND_ALIAS_DEFAULTS),
        "status-format" => Some(STATUS_FORMAT_DEFAULTS),
        "terminal-features" => Some(TERMINAL_FEATURE_DEFAULTS),
        "terminal-overrides" => Some(&["linux*:AX@"]),
        "update-environment" => Some(UPDATE_ENVIRONMENT_DEFAULTS),
        _ => None,
    }
}

fn install_array_default(table: &mut OptionSet, name: &str, values: &[&str]) {
    if values.is_empty() {
        table.set(name, "");
    } else {
        for (index, value) in values.iter().enumerate() {
            table.set(format!("{name}[{index}]"), *value);
        }
    }
}

pub(crate) fn restore_array_default(table: &mut OptionSet, name: &str) {
    table.clear_array(name);
    if let Some(values) = option_array_default(name) {
        install_array_default(table, name, values);
    }
}

/// Built-in options whose values are indexed arrays in tmux 3.7b.
///
/// Keep this separate from `OPTION_VALID_ONLY`: that table also contains
/// scalar options whose defaults are environment-derived or otherwise not
/// modeled. tmux rejects an index on those scalars when changing an option.
const OPTION_ARRAYS: &[&str] = &[
    "codepoint-widths",
    "command-alias",
    "pane-colours",
    "status-format",
    "terminal-features",
    "terminal-overrides",
    "update-environment",
    "user-keys",
];

/// `(name, minimum, maximum)` for the numeric options in tmux 3.7b's options
/// table. Keeping the type metadata beside the catalog lets `set-option`
/// reject malformed and out-of-range values instead of storing strings that
/// runtime consumers cannot interpret. Ranges were probed from the pinned
/// tmux 3.7b binary.
const OPTION_NUMBERS: &[(&str, i64, i64)] = &[
    ("assume-paste-time", 0, i32::MAX as i64),
    ("base-index", 0, i32::MAX as i64),
    ("buffer-limit", 1, i32::MAX as i64),
    ("display-panes-time", 1, i32::MAX as i64),
    ("display-time", 0, i32::MAX as i64),
    ("escape-time", 0, i32::MAX as i64),
    ("history-limit", 0, i32::MAX as i64),
    ("initial-repeat-time", 0, 2_000_000),
    ("input-buffer-size", 1_048_576, u32::MAX as i64),
    ("lock-after-time", 0, i32::MAX as i64),
    ("message-limit", 0, i32::MAX as i64),
    ("monitor-silence", 0, i32::MAX as i64),
    ("pane-base-index", 0, u16::MAX as i64),
    ("prefix-timeout", 0, i32::MAX as i64),
    ("prompt-history-limit", 0, i32::MAX as i64),
    ("repeat-time", 0, 2_000_000),
    ("status-interval", 0, i32::MAX as i64),
    ("status-left-length", 0, i16::MAX as i64),
    ("status-right-length", 0, i16::MAX as i64),
    ("tiled-layout-max-columns", 0, u16::MAX as i64),
];

const CURSOR_STYLE_CHOICES: &[&str] = &[
    "default",
    "blinking-block",
    "block",
    "blinking-underline",
    "underline",
    "blinking-bar",
    "bar",
];

const BORDER_LINES_CHOICES: &[&str] = &[
    "single", "double", "heavy", "simple", "rounded", "padded", "none",
];

const ALERT_ACTION_CHOICES: &[&str] = &["any", "none", "current", "other"];

const VISUAL_ALERT_CHOICES: &[&str] = &["off", "on", "both"];

/// `(name, choices)` for the choice options in tmux 3.7b's options table.
/// `set-option` rejects any value outside the list with `unknown value`, and
/// with no value at all toggles between the first two entries (only when the
/// current value is one of them). Lists were probed from the pinned tmux 3.7b
/// binary; membership and the first two entries are the observable parts of
/// the order.
const OPTION_CHOICES: &[(&str, &[&str])] = &[
    ("activity-action", ALERT_ACTION_CHOICES),
    ("allow-passthrough", &["off", "on", "all"]),
    ("bell-action", ALERT_ACTION_CHOICES),
    (
        "clock-mode-style",
        &["12", "24", "12-with-seconds", "24-with-seconds"],
    ),
    (
        "copy-mode-line-numbers",
        &["off", "default", "absolute", "relative", "hybrid"],
    ),
    ("cursor-style", CURSOR_STYLE_CHOICES),
    (
        "destroy-unattached",
        &["off", "on", "keep-last", "keep-group"],
    ),
    (
        "detach-on-destroy",
        &["off", "on", "no-detached", "previous", "next"],
    ),
    ("extended-keys", &["off", "on", "always"]),
    ("extended-keys-format", &["csi-u", "xterm"]),
    ("get-clipboard", &["off", "buffer", "request", "both"]),
    ("menu-border-lines", BORDER_LINES_CHOICES),
    ("message-line", &["0", "1", "2", "3", "4"]),
    ("mode-keys", &["emacs", "vi"]),
    (
        "pane-border-indicators",
        &["off", "colour", "arrows", "both"],
    ),
    (
        "pane-border-lines",
        &["single", "double", "heavy", "simple", "number"],
    ),
    ("pane-border-status", &["off", "top", "bottom"]),
    ("pane-scrollbars", &["off", "modal", "on"]),
    ("pane-scrollbars-position", &["right", "left"]),
    ("popup-border-lines", BORDER_LINES_CHOICES),
    ("prompt-command-cursor-style", CURSOR_STYLE_CHOICES),
    ("prompt-cursor-style", CURSOR_STYLE_CHOICES),
    ("remain-on-exit", &["off", "on", "failed"]),
    ("set-clipboard", &["off", "external", "on"]),
    ("silence-action", ALERT_ACTION_CHOICES),
    ("status", &["off", "on", "2", "3", "4", "5"]),
    (
        "status-justify",
        &["left", "centre", "right", "absolute-centre"],
    ),
    ("status-keys", &["emacs", "vi"]),
    ("status-position", &["top", "bottom"]),
    ("visual-activity", VISUAL_ALERT_CHOICES),
    ("visual-bell", VISUAL_ALERT_CHOICES),
    ("visual-silence", VISUAL_ALERT_CHOICES),
    ("window-size", &["largest", "smallest", "manual", "latest"]),
];

/// Boolean flag options in tmux 3.7b's options table.
const OPTION_FLAGS: &[&str] = &[
    "exit-empty",
    "exit-unattached",
    "focus-events",
    "variation-selector-always-wide",
    "focus-follows-mouse",
    "mouse",
    "renumber-windows",
    "set-titles",
    "aggressive-resize",
    "allow-rename",
    "allow-set-title",
    "alternate-screen",
    "automatic-rename",
    "monitor-activity",
    "monitor-bell",
    "scroll-on-clear",
    "synchronize-panes",
    "wrap-search",
    "xterm-keys",
];

/// `exit-empty` value hmux accepts on top of tmux's flag, and defaults to:
/// exit once the server is empty, but only after it has held a session. tmux
/// 3.7b rejects the value with `bad value:`, since its server is forked by the
/// client that creates the first session and so never sits empty at startup
/// the way a daemon does; tmux instead forces `exit-empty` off at startup for
/// the one server of its own that nobody forked (`tmux -D`).
pub(crate) const EXIT_EMPTY_AFTER_SESSION: &str = "after-session";

/// The extra value a flag option accepts beyond tmux's `on`/`off` spellings.
/// Flags keep their tmux parsing — including the `yes`/`no`/`1`/`0` aliases a
/// choice option would not take — and gain only this one value.
pub(crate) fn flag_extension_value(name: &str) -> Option<&'static str> {
    (name == "exit-empty").then_some(EXIT_EMPTY_AFTER_SESSION)
}

pub(crate) const SESSION_HOOKS: &[&str] = &[
    "after-bind-key",
    "after-capture-pane",
    "after-copy-mode",
    "after-display-message",
    "after-display-panes",
    "after-kill-pane",
    "after-list-buffers",
    "after-list-clients",
    "after-list-keys",
    "after-list-panes",
    "after-list-sessions",
    "after-list-windows",
    "after-load-buffer",
    "after-lock-server",
    "after-new-session",
    "after-new-window",
    "after-paste-buffer",
    "after-pipe-pane",
    "after-queue",
    "after-refresh-client",
    "after-rename-session",
    "after-rename-window",
    "after-resize-pane",
    "after-resize-window",
    "after-save-buffer",
    "after-select-layout",
    "after-select-pane",
    "after-select-window",
    "after-send-keys",
    "after-set-buffer",
    "after-set-environment",
    "after-set-hook",
    "after-set-option",
    "after-show-environment",
    "after-show-messages",
    "after-show-options",
    "after-split-window",
    "after-unbind-key",
    "alert-activity",
    "alert-bell",
    "alert-silence",
    "client-active",
    "client-attached",
    "client-detached",
    "client-focus-in",
    "client-focus-out",
    "client-resized",
    "client-session-changed",
    "client-light-theme",
    "client-dark-theme",
    "command-error",
    "session-closed",
    "session-created",
    "session-renamed",
    "session-window-changed",
    "window-linked",
    "window-unlinked",
];

pub(crate) const PANE_HOOKS: &[&str] = &[
    "pane-died",
    "pane-exited",
    "pane-focus-in",
    "pane-focus-out",
    "pane-mode-changed",
    "pane-set-clipboard",
    "pane-title-changed",
];

pub(crate) const WINDOW_HOOKS: &[&str] = &[
    "window-layout-changed",
    "window-pane-changed",
    "window-renamed",
    "window-resized",
];

/// A name index over the tables above, sorted once and searched thereafter.
///
/// The tables are written to be read: grouped by meaning, in the order tmux's
/// `options-table.c` lists them. That is the right shape for a human and the
/// wrong one for `option_kind`, which the effective-options view calls once per
/// option per layer — a linear scan of 150 names, per name. Sorting a copy on
/// first use keeps both.
struct NameIndex(std::sync::OnceLock<Vec<&'static str>>);

impl NameIndex {
    const fn new() -> NameIndex {
        NameIndex(std::sync::OnceLock::new())
    }

    fn contains(&self, name: &str, tables: &[&[&'static str]]) -> bool {
        self.0
            .get_or_init(|| {
                let mut names: Vec<&'static str> = tables
                    .iter()
                    .flat_map(|table| table.iter().copied())
                    .collect();
                names.sort_unstable();
                names
            })
            .binary_search_by(|probe| (*probe).cmp(name))
            .is_ok()
    }
}

pub(crate) fn is_hook(name: &str) -> bool {
    static HOOKS: NameIndex = NameIndex::new();
    HOOKS.contains(name, &[SESSION_HOOKS, WINDOW_HOOKS, PANE_HOOKS])
}

const SERVER_OPTIONS: &[&str] = &[
    "backspace",
    "buffer-limit",
    "codepoint-widths",
    "command-alias",
    "copy-command",
    "default-client-command",
    "default-terminal",
    "editor",
    "escape-time",
    "exit-empty",
    "exit-unattached",
    "extended-keys",
    "extended-keys-format",
    "focus-events",
    "get-clipboard",
    "history-file",
    "input-buffer-size",
    "message-limit",
    "prefix-timeout",
    "prompt-history-limit",
    "set-clipboard",
    "terminal-features",
    "terminal-overrides",
    "user-keys",
    "variation-selector-always-wide",
];

const SESSION_OPTIONS: &[&str] = &[
    "activity-action",
    "assume-paste-time",
    "base-index",
    "bell-action",
    "default-command",
    "default-shell",
    "default-size",
    "destroy-unattached",
    "detach-on-destroy",
    "display-panes-active-colour",
    "display-panes-colour",
    "display-panes-time",
    "display-time",
    "focus-follows-mouse",
    "history-limit",
    "initial-repeat-time",
    "key-table",
    "lock-after-time",
    "lock-command",
    "message-command-style",
    "message-format",
    "message-line",
    "message-style",
    "mouse",
    "prefix",
    "prefix2",
    "prompt-command-cursor-style",
    "prompt-cursor-colour",
    "prompt-cursor-style",
    "renumber-windows",
    "repeat-time",
    "set-titles",
    "set-titles-string",
    "silence-action",
    "status",
    "status-bg",
    "status-fg",
    "status-format",
    "status-interval",
    "status-justify",
    "status-keys",
    "status-left",
    "status-left-length",
    "status-left-style",
    "status-position",
    "status-right",
    "status-right-length",
    "status-right-style",
    "status-style",
    "update-environment",
    "visual-activity",
    "visual-bell",
    "visual-silence",
    "word-separators",
];

const WINDOW_OPTIONS: &[&str] = &[
    "aggressive-resize",
    "automatic-rename",
    "automatic-rename-format",
    "clock-mode-colour",
    "clock-mode-style",
    "copy-mode-current-line-number-style",
    "copy-mode-current-match-style",
    "copy-mode-line-number-style",
    "copy-mode-line-numbers",
    "copy-mode-mark-style",
    "copy-mode-match-style",
    "copy-mode-position-style",
    "copy-mode-selection-style",
    "fill-character",
    "main-pane-height",
    "main-pane-width",
    "menu-border-lines",
    "menu-border-style",
    "menu-selected-style",
    "menu-style",
    "mode-keys",
    "mode-style",
    "monitor-activity",
    "monitor-bell",
    "monitor-silence",
    "other-pane-height",
    "other-pane-width",
    "pane-base-index",
    "pane-border-indicators",
    "pane-border-lines",
    "pane-border-status",
    "pane-scrollbars",
    "pane-scrollbars-position",
    "pane-status-current-style",
    "pane-status-style",
    "popup-border-lines",
    "popup-border-style",
    "popup-style",
    "session-status-current-style",
    "session-status-style",
    "tiled-layout-max-columns",
    "tree-mode-preview-style",
    "window-pane-current-status-format",
    "window-pane-status-format",
    "window-size",
    "window-status-activity-style",
    "window-status-bell-style",
    "window-status-current-format",
    "window-status-current-style",
    "window-status-format",
    "window-status-last-style",
    "window-status-separator",
    "window-status-style",
    "wrap-search",
    "xterm-keys",
];

const WINDOW_PANE_OPTIONS: &[&str] = &[
    "allow-passthrough",
    "allow-rename",
    "allow-set-title",
    "alternate-screen",
    "copy-mode-position-format",
    "cursor-colour",
    "cursor-style",
    "pane-active-border-style",
    "pane-border-format",
    "pane-border-style",
    "pane-colours",
    "pane-scrollbars-style",
    "remain-on-exit",
    "remain-on-exit-format",
    "scroll-on-clear",
    "synchronize-panes",
    "tree-mode-preview-format",
    "window-active-style",
    "window-style",
];

/// The built-in table in which an option lives.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OptionScope {
    Server,
    Session,
    Window,
    WindowPane,
}

pub(crate) fn option_scope(name: &str) -> Option<OptionScope> {
    if SESSION_HOOKS.contains(&name) {
        Some(OptionScope::Session)
    } else if WINDOW_HOOKS.contains(&name) {
        Some(OptionScope::Window)
    } else if PANE_HOOKS.contains(&name) {
        Some(OptionScope::WindowPane)
    } else if SERVER_OPTIONS.contains(&name) {
        Some(OptionScope::Server)
    } else if SESSION_OPTIONS.contains(&name) {
        Some(OptionScope::Session)
    } else if WINDOW_OPTIONS.contains(&name) {
        Some(OptionScope::Window)
    } else if WINDOW_PANE_OPTIONS.contains(&name) {
        Some(OptionScope::WindowPane)
    } else {
        None
    }
}

pub(crate) fn option_array_separator(name: &str) -> Option<&'static str> {
    if is_hook(name) {
        return Some("");
    }
    match name {
        "codepoint-widths" | "command-alias" | "terminal-features" | "terminal-overrides"
        | "user-keys" => Some(","),
        "pane-colours" | "update-environment" => Some(" ,"),
        _ => None,
    }
}

/// The catalog type of a built-in option.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OptionKind {
    Scalar,
    Array,
}

/// The default value for `name`, if it has a stable modeled default.
pub fn option_default(name: &str) -> Option<&'static str> {
    OPTION_DEFAULTS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, v)| *v)
}

pub(crate) fn option_initial_default(name: &str) -> Option<String> {
    option_default(name)
        .map(str::to_string)
        .or_else(|| match name {
            "mode-keys" | "status-keys" => Some(mode_keys_default().to_string()),
            "default-command" => Some(String::new()),
            "default-shell" => Some(
                std::env::var("SHELL")
                    .ok()
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "/bin/sh".to_string()),
            ),
            "editor" => Some(
                std::env::var("VISUAL")
                    .ok()
                    .filter(|value| !value.is_empty())
                    .or_else(|| std::env::var("EDITOR").ok())
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "vi".to_string()),
            ),
            _ => None,
        })
}

/// Runtime default tmux derives from VISUAL/EDITOR.
pub(crate) fn mode_keys_default() -> &'static str {
    let editor = std::env::var("VISUAL")
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| std::env::var("EDITOR").ok())
        .unwrap_or_default();
    if editor.contains("vi") {
        "vi"
    } else {
        "emacs"
    }
}

/// Parse tmux's optional numeric `[N]` suffix.
///
/// tmux stores the index in a signed C `int`, so values above `i32::MAX` are
/// invalid even though they would fit in Rust's `u32`.
pub(crate) fn parse_option_name(name: &str) -> Option<(&str, Option<u32>)> {
    if name.is_empty() {
        return None;
    }
    let Some(open) = name.find('[') else {
        return Some((name, None));
    };
    let base = &name[..open];
    let index = name.get(open + 1..)?.strip_suffix(']')?;
    if base.is_empty() || index.is_empty() || index.contains(['[', ']']) {
        return None;
    }
    let index = index.parse::<i32>().ok()?;
    Some((base, Some(u32::try_from(index).ok()?)))
}

/// Look up the scalar-versus-array metadata for an exact built-in option name.
pub(crate) fn option_kind(name: &str) -> Option<OptionKind> {
    static ARRAYS: NameIndex = NameIndex::new();
    static SCALARS: std::sync::OnceLock<Vec<&'static str>> = std::sync::OnceLock::new();
    if ARRAYS.contains(name, &[OPTION_ARRAYS]) || is_hook(name) {
        return Some(OptionKind::Array);
    }
    let scalars = SCALARS.get_or_init(|| {
        let mut names: Vec<&'static str> = OPTION_DEFAULTS
            .iter()
            .map(|(name, _)| *name)
            .chain(OPTION_VALID_ONLY.iter().copied())
            .collect();
        names.sort_unstable();
        names
    });
    scalars
        .binary_search_by(|probe| (*probe).cmp(name))
        .is_ok()
        .then_some(OptionKind::Scalar)
}

pub(crate) fn option_number_range(name: &str) -> Option<(i64, i64)> {
    OPTION_NUMBERS
        .iter()
        .find(|(n, _, _)| *n == name)
        .map(|&(_, min, max)| (min, max))
}

pub(crate) fn option_choices(name: &str) -> Option<&'static [&'static str]> {
    OPTION_CHOICES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|&(_, choices)| choices)
}

pub(crate) fn option_is_flag(name: &str) -> bool {
    // Every format variable goes through this on its way into the table, so it
    // is asked a few hundred times per command.
    static FLAGS: NameIndex = NameIndex::new();
    FLAGS.contains(name, &[OPTION_FLAGS])
}

pub(crate) fn is_style_option(name: &str) -> bool {
    name.ends_with("-style") && option_choices(name).is_none()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OptionResolveResult {
    Found(&'static str, Option<u32>),
    Ambiguous,
    Invalid,
}

/// Resolve an exact option name or an unambiguous prefix, preserving an array
/// index. tmux accepts prefixes such as `status-int` for `status-interval`.
pub(crate) fn resolve_option_name(name: &str) -> OptionResolveResult {
    let Some((base, index)) = parse_option_name(name) else {
        return OptionResolveResult::Invalid;
    };
    let mut candidates = OPTION_DEFAULTS
        .iter()
        .map(|(name, _)| *name)
        .chain(OPTION_VALID_ONLY.iter().copied())
        .chain(SESSION_HOOKS.iter().copied())
        .chain(WINDOW_HOOKS.iter().copied())
        .chain(PANE_HOOKS.iter().copied())
        .filter(|candidate| candidate.starts_with(base))
        .collect::<Vec<_>>();
    candidates.sort_unstable();
    candidates.dedup();
    if let Some(exact) = candidates
        .iter()
        .copied()
        .find(|candidate| *candidate == base)
    {
        return OptionResolveResult::Found(exact, index);
    }
    match candidates.len() {
        0 => OptionResolveResult::Invalid,
        1 => OptionResolveResult::Found(candidates[0], index),
        _ => OptionResolveResult::Ambiguous,
    }
}

/// Whether `name` resolves to a real built-in tmux option (in either table).
/// This performs tmux-style suffix parsing but does not impose the
/// command-specific rule that only array options may be changed by index.
#[cfg(test)]
fn is_valid_option(name: &str) -> bool {
    parse_option_name(name)
        .and_then(|(base, _)| option_kind(base))
        .is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_scalar_options_have_defaults() {
        assert_eq!(option_default("status"), Some("on"));
        assert_eq!(option_default("base-index"), Some("0"));
        assert_eq!(option_default("synchronize-panes"), Some("off"));
        assert_eq!(option_default("history-limit"), Some("2000"));
        assert_eq!(option_default("get-clipboard"), Some("buffer"));
        assert_eq!(option_default("copy-mode-line-numbers"), Some("off"));
        assert_eq!(
            option_default("message-style"),
            Some("bg=yellow,fg=black,fill=yellow")
        );
        assert_eq!(option_default("status-left"), Some("[#{session_name}] "));
        assert_eq!(
            option_default("window-status-format"),
            Some("#I:#{?#{m:*fable*,#{pane_agent_model}},#[bg=red],#{?#{m:*luna*,#{pane_agent_model}},#[bg=brightblue],}}#{pane_state_emoji}#[default] #{?pane_current_path,#{b:pane_current_path},#{b:session_path}}#{?window_flags,#{window_flags}, }")
        );
    }

    #[test]
    fn env_derived_and_array_options_are_valid_without_a_default() {
        // Valid names (so set-option accepts them) but no baked default.
        assert!(is_valid_option("mode-keys"));
        assert!(is_valid_option("default-shell"));
        assert!(is_valid_option("command-alias"));
        assert!(is_valid_option("message-format"));
        assert!(is_valid_option("window-pane-status-format"));
        assert_eq!(option_default("mode-keys"), None);
        assert_eq!(option_default("command-alias"), None);
        assert!(option_default("window-pane-status-format").is_some());
    }

    #[test]
    fn option_catalog_distinguishes_arrays_from_scalars() {
        for name in OPTION_ARRAYS {
            assert_eq!(option_kind(name), Some(OptionKind::Array), "{name}");
            assert!(is_valid_option(&format!("{name}[7]")), "{name}");
        }
        for name in ["status-left", "message-format", "mode-keys", "mouse"] {
            assert_eq!(option_kind(name), Some(OptionKind::Scalar), "{name}");
            // The suffix is syntactically valid and resolves to this scalar.
            // set-option applies the separate "not an array" rule.
            assert!(is_valid_option(&format!("{name}[7]")), "{name}");
        }
    }

    #[test]
    fn every_modeled_option_has_exactly_one_scope() {
        for &(name, _) in OPTION_DEFAULTS {
            assert!(option_scope(name).is_some(), "missing scope for {name}");
        }
        for name in OPTION_VALID_ONLY {
            assert!(option_scope(name).is_some(), "missing scope for {name}");
        }
    }

    #[test]
    fn option_views_resolve_scalars_and_shadow_arrays_by_whole_entry() {
        let mut global = OptionSet::default();
        global.set("status-position", "bottom");
        global.set("status-format[0]", "global-zero");
        global.set("status-format[1]", "global-one");

        let mut local = OptionSet::default();
        local.set("status-position", "top");
        local.set("status-format[1]", "local-one");
        let view = OptionsView::two(&local, &global);

        assert_eq!(view.get("status-position"), Some("top"));
        assert_eq!(view.get("status-format[0]"), None);
        assert_eq!(view.get("status-format[1]"), Some("local-one"));
        assert_eq!(
            view.iter_effective().collect::<Vec<_>>(),
            vec![
                ("status-format[1]", "local-one"),
                ("status-position", "top")
            ]
        );
    }

    #[test]
    fn indexed_name_parser_matches_tmux_integer_bounds() {
        assert_eq!(
            parse_option_name("status-format[001]"),
            Some(("status-format", Some(1)))
        );
        assert_eq!(
            parse_option_name("status-format[2147483647]"),
            Some(("status-format", Some(2_147_483_647)))
        );
        for name in [
            "",
            "status-format[]",
            "status-format[-1]",
            "status-format[x]",
            "status-format[2147483648]",
            "status-format[1][2]",
        ] {
            assert_eq!(parse_option_name(name), None, "{name}");
        }
    }

    #[test]
    fn unknown_names_are_not_valid() {
        assert!(!is_valid_option("no-such-option-xyz"));
        assert!(!is_valid_option("@user")); // user options handled separately
    }
}
