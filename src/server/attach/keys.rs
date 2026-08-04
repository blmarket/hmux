//! Per-client key-table state: tmux's `c->keytable`, the `prefix-timeout`
//! fallback, and `bind -r` repeat chains.
//!
//! tmux keeps the *table* on the client rather than a "prefix is pending" flag:
//! pressing the prefix moves the client into the `prefix` table and it stays
//! there until a binding runs, the table times out, or a repeat chain expires.
//! Modelling it the same way is what makes `prefix-timeout`,
//! `repeat-time`/`initial-repeat-time` and `#{client_key_table}` expressible;
//! the walk in [`ClientKeyState::resolve`] follows
//! `server_client_key_callback`'s `table_changed`/`try_again` loop.

use std::time::{Duration, Instant};

use super::super::key::{KeyBase, KeyCode, SpecialKey};
use super::super::state::{SharedState, DEFAULT_KEY_TABLE};
use super::copy_mode;

/// The table the prefix key switches a client into.
pub(super) const PREFIX_TABLE: &str = "prefix";

/// What the key machinery decided about one key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum KeyResolution {
    /// Dispatch the key's binding from `table`.
    Binding { table: String },
    /// The key belongs to the key machinery: it was the prefix, or it fell out
    /// of a non-default table with nothing bound. Either way the pane must not
    /// see it.
    Consumed,
    /// Nothing claimed the key; its bytes belong to the pane.
    Forward,
}

/// The parts of a binding the resolution walk needs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct BindingInfo {
    pub(super) repeat: bool,
}

/// Everything [`ClientKeyState::resolve`] asks the server, kept behind a trait
/// so the walk itself stays free of locking and is unit-testable.
pub(super) trait KeyTableSource {
    /// tmux's `server_client_get_key_table`: the session's `key-table` option,
    /// or `root`.
    fn default_table(&self) -> String;
    /// The active mode's table, when the pane is in one.
    fn mode_table(&self) -> Option<String>;
    fn is_prefix(&self, key: KeyCode) -> bool;
    fn binding(&self, table: &str, key: KeyCode) -> Option<BindingInfo>;
    /// `prefix-timeout` in milliseconds; 0 disables the timeout.
    fn prefix_timeout(&self) -> u64;
    /// `repeat-time` in milliseconds; 0 disables repeat chains entirely.
    fn repeat_time(&self) -> u64;
    /// `initial-repeat-time` in milliseconds; 0 means "use `repeat-time`".
    fn initial_repeat_time(&self) -> u64;
    /// `assume-paste-time` in milliseconds; 0 disables the heuristic.
    fn assume_paste_time(&self) -> u64 {
        0
    }
}

/// One attached client's key table and repeat chain.
#[derive(Clone, Debug)]
pub(super) struct ClientKeyState {
    table: String,
    /// When the table was last entered — tmux's `keytable->activity_time`,
    /// which `prefix-timeout` measures against.
    table_activity: Instant,
    /// tmux's `CLIENT_REPEAT`: a `bind -r` chain is holding the table open.
    repeat: bool,
    /// tmux's `c->last_key`: the key that started the chain.
    last_key: Option<KeyCode>,
    /// When the repeat chain lapses on its own, with no further input.
    repeat_deadline: Option<Instant>,
    /// tmux's `c->last_activity_time`: when the previous key arrived, which is
    /// what the `assume-paste-time` gap is measured against.
    last_key_at: Option<Instant>,
    /// tmux's `CLIENT_ASSUMEPASTING`: the previous gap was short enough that
    /// the next key is taken as pasted text rather than a command key.
    assume_pasting: bool,
    /// tmux's `CLIENT_BRACKETPASTING`: the client's terminal said a paste is
    /// in progress, so there is nothing to guess about.
    bracket_pasting: bool,
}

impl ClientKeyState {
    pub(super) fn new(now: Instant) -> Self {
        Self {
            table: DEFAULT_KEY_TABLE.to_string(),
            table_activity: now,
            repeat: false,
            last_key: None,
            repeat_deadline: None,
            last_key_at: None,
            assume_pasting: false,
            bracket_pasting: false,
        }
    }

    /// tmux's `server_client_is_bracket_paste`: the markers themselves are
    /// ordinary keys — they go on to the tables, and reach a pane that asked
    /// for them — but everything between goes straight to the pane.
    pub(super) fn is_bracket_paste(&mut self, key: KeyCode) -> bool {
        match key.base {
            KeyBase::Special(SpecialKey::PasteStart) => {
                self.bracket_pasting = true;
                false
            }
            KeyBase::Special(SpecialKey::PasteEnd) => {
                self.bracket_pasting = false;
                false
            }
            _ => self.bracket_pasting,
        }
    }

    /// tmux's `server_client_is_assume_paste`: whether this key arrived fast
    /// enough after the last one to be pasted text rather than typing.
    ///
    /// The first fast key only arms the heuristic; the one after it — and every
    /// one that keeps up the pace — goes to the pane. A gap longer than the
    /// option disarms it again. `assume_paste_time` of 0 disables it, and so
    /// does a terminal that can bracket its own pastes: there is no need to
    /// guess when the terminal will say so.
    pub(super) fn is_assume_paste(
        &mut self,
        now: Instant,
        assume_paste_time: u64,
        terminal_brackets_pastes: bool,
    ) -> bool {
        let previous = self.last_key_at.replace(now);
        if self.bracket_pasting || assume_paste_time == 0 || terminal_brackets_pastes {
            self.assume_pasting = false;
            return false;
        }
        let fast = previous
            .is_some_and(|previous| now.duration_since(previous) < Duration::from_millis(assume_paste_time));
        if fast {
            if self.assume_pasting {
                return true;
            }
            self.assume_pasting = true;
            return false;
        }
        self.assume_pasting = false;
        false
    }

    pub(super) fn table(&self) -> &str {
        &self.table
    }

    pub(super) fn repeat_deadline(&self) -> Option<Instant> {
        self.repeat_deadline
    }

    fn set_table(&mut self, name: &str, now: Instant) {
        self.table.clear();
        self.table.push_str(name);
        self.table_activity = now;
    }

    fn end_repeat(&mut self) {
        self.repeat = false;
        self.repeat_deadline = None;
    }

    /// tmux's `server_client_repeat_timer`: the chain lapses with no input, so
    /// the client leaves the prefix table on its own. Returns whether anything
    /// changed, so the caller can invalidate the status line.
    pub(super) fn expire_repeat(&mut self, now: Instant, default_table: &str) -> bool {
        if !self.repeat || self.repeat_deadline.is_none_or(|deadline| now < deadline) {
            return false;
        }
        self.end_repeat();
        self.set_table(default_table, now);
        true
    }

    /// tmux's `server_client_repeat_time`: how long this binding holds the
    /// table open, or `None` when it does not repeat.
    fn repeat_duration(
        &self,
        binding: BindingInfo,
        key: KeyCode,
        tables: &impl KeyTableSource,
    ) -> Option<Duration> {
        if !binding.repeat {
            return None;
        }
        let repeat = tables.repeat_time();
        if repeat == 0 {
            return None;
        }
        // A chain that is starting, or that changed key, gets the longer
        // initial window instead.
        let repeat = if !self.repeat || self.last_key != Some(key) {
            match tables.initial_repeat_time() {
                0 => repeat,
                initial => initial,
            }
        } else {
            repeat
        };
        Some(Duration::from_millis(repeat))
    }

    /// Walk the key tables for one key and update the client's state.
    ///
    /// `now` is the key's arrival time; `prefix-timeout` measures it against
    /// the time the current table was entered.
    pub(super) fn resolve(
        &mut self,
        key: KeyCode,
        now: Instant,
        tables: &impl KeyTableSource,
    ) -> KeyResolution {
        let default = tables.default_table();
        // While the client is still in its default table, a pane in a mode
        // lends its own table to the lookup (tmux's `wme->mode->key_table`).
        let mut table = if self.table == default {
            tables.mode_table().unwrap_or_else(|| self.table.clone())
        } else {
            self.table.clone()
        };
        let mut first = table.clone();
        loop {
            // The prefix always wins and forces the prefix table, unless the
            // client is already in it.
            if table != PREFIX_TABLE && tables.is_prefix(key) {
                self.set_table(PREFIX_TABLE, now);
                return KeyResolution::Consumed;
            }
            // tmux snapshots the repeat flag per pass, before the lookup.
            let entry_repeat = self.repeat;
            let binding = tables.binding(&table, key);

            // An idle prefix table reverts to the default one and the key is
            // retried there — the prefix command is lost. A live repeat chain
            // whose key still repeats is exempt.
            if table == PREFIX_TABLE {
                let timeout = tables.prefix_timeout();
                let idle = now.saturating_duration_since(self.table_activity);
                let exempt = self.repeat && binding.is_some_and(|binding| binding.repeat);
                if timeout > 0 && idle.as_millis() > u128::from(timeout) && !exempt {
                    self.set_table(&default, now);
                    table = default.clone();
                    first = table.clone();
                    continue;
                }
            }

            if let Some(binding) = binding {
                // A chain broken by a non-repeating binding reverts to the
                // default table and retries there rather than running here.
                if entry_repeat && !binding.repeat {
                    self.end_repeat();
                    self.set_table(&default, now);
                    table = default.clone();
                    first = table.clone();
                    continue;
                }
                match self.repeat_duration(binding, key, tables) {
                    Some(duration) => {
                        self.repeat = true;
                        self.last_key = Some(key);
                        self.repeat_deadline = Some(now + duration);
                    }
                    None => {
                        self.end_repeat();
                        self.set_table(&default, now);
                    }
                }
                return KeyResolution::Binding { table };
            }

            // Nothing bound here: fall back to the default table and retry,
            // which also ends any repeat chain.
            if table != default || entry_repeat {
                self.set_table(&default, now);
                table = default.clone();
                if entry_repeat {
                    first = table.clone();
                }
                self.end_repeat();
                continue;
            }

            // Nothing in the default table either. A key that started in
            // another table is swallowed rather than reaching the pane.
            if first != table {
                self.set_table(&default, now);
                return KeyResolution::Consumed;
            }
            return KeyResolution::Forward;
        }
    }
}

/// The live server view [`ClientKeyState::resolve`] runs against.
pub(super) struct ServerKeyTables<'a> {
    state: &'a SharedState,
    target: &'a str,
}

impl<'a> ServerKeyTables<'a> {
    pub(super) fn new(state: &'a SharedState, target: &'a str) -> Self {
        Self { state, target }
    }

    fn option_number(&self, name: &str) -> u64 {
        self.state
            .lock()
            .ok()
            .and_then(|st| {
                st.option_for_target(self.target, name)
                    .or_else(|| super::super::options::option_default(name))
                    .and_then(|value| value.parse::<u64>().ok())
            })
            .unwrap_or(0)
    }
}

impl KeyTableSource for ServerKeyTables<'_> {
    fn default_table(&self) -> String {
        super::client_key_table(self.state, self.target)
    }

    fn mode_table(&self) -> Option<String> {
        copy_mode::is_active(self.state, self.target)
            .then(|| copy_mode::key_table(self.state, self.target).to_string())
    }

    fn is_prefix(&self, key: KeyCode) -> bool {
        super::is_configured_prefix(self.state, self.target, key)
    }

    fn binding(&self, table: &str, key: KeyCode) -> Option<BindingInfo> {
        let st = self.state.lock().ok()?;
        let binding = st.key_binding(table, key)?;
        Some(BindingInfo {
            repeat: binding.repeat,
        })
    }

    fn prefix_timeout(&self) -> u64 {
        self.option_number("prefix-timeout")
    }

    fn repeat_time(&self) -> u64 {
        self.option_number("repeat-time")
    }

    fn initial_repeat_time(&self) -> u64 {
        self.option_number("initial-repeat-time")
    }

    fn assume_paste_time(&self) -> u64 {
        self.option_number("assume-paste-time")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::key::key_from_byte;

    #[derive(Default)]
    struct FakeTables {
        default_table: String,
        mode_table: Option<String>,
        prefix: Option<KeyCode>,
        bindings: Vec<(String, KeyCode, BindingInfo)>,
        prefix_timeout: u64,
        repeat_time: u64,
        initial_repeat_time: u64,
    }

    impl FakeTables {
        fn new() -> Self {
            Self {
                default_table: DEFAULT_KEY_TABLE.to_string(),
                prefix: Some(key_from_byte(0x02)),
                repeat_time: 500,
                ..Self::default()
            }
        }

        fn bind(mut self, table: &str, key: KeyCode, repeat: bool) -> Self {
            self.bindings
                .push((table.to_string(), key, BindingInfo { repeat }));
            self
        }
    }

    impl KeyTableSource for FakeTables {
        fn default_table(&self) -> String {
            self.default_table.clone()
        }

        fn mode_table(&self) -> Option<String> {
            self.mode_table.clone()
        }

        fn is_prefix(&self, key: KeyCode) -> bool {
            self.prefix == Some(key)
        }

        fn binding(&self, table: &str, key: KeyCode) -> Option<BindingInfo> {
            self.bindings
                .iter()
                .find(|(bound_table, bound_key, _)| bound_table == table && *bound_key == key)
                .map(|(_, _, binding)| *binding)
        }

        fn prefix_timeout(&self) -> u64 {
            self.prefix_timeout
        }

        fn repeat_time(&self) -> u64 {
            self.repeat_time
        }

        fn initial_repeat_time(&self) -> u64 {
            self.initial_repeat_time
        }
    }

    fn binding(table: &str) -> KeyResolution {
        KeyResolution::Binding {
            table: table.to_string(),
        }
    }

    #[test]
    fn prefix_switches_tables_and_the_next_key_runs_its_binding() {
        let now = Instant::now();
        let prefix = key_from_byte(0x02);
        let c = key_from_byte(b'c');
        let tables = FakeTables::new().bind(PREFIX_TABLE, c, false);
        let mut keys = ClientKeyState::new(now);

        assert_eq!(keys.resolve(prefix, now, &tables), KeyResolution::Consumed);
        assert_eq!(keys.table(), PREFIX_TABLE);

        assert_eq!(keys.resolve(c, now, &tables), binding(PREFIX_TABLE));
        assert_eq!(keys.table(), DEFAULT_KEY_TABLE);
        assert_eq!(keys.repeat_deadline(), None);
    }

    #[test]
    fn unbound_keys_reach_the_pane_but_not_after_a_prefix() {
        let now = Instant::now();
        let prefix = key_from_byte(0x02);
        let z = key_from_byte(b'z');
        let tables = FakeTables::new();
        let mut keys = ClientKeyState::new(now);

        assert_eq!(keys.resolve(z, now, &tables), KeyResolution::Forward);

        keys.resolve(prefix, now, &tables);
        // Unbound in `prefix` and in `root`: tmux swallows it.
        assert_eq!(keys.resolve(z, now, &tables), KeyResolution::Consumed);
        assert_eq!(keys.table(), DEFAULT_KEY_TABLE);
    }

    #[test]
    fn a_prefix_key_falls_back_to_the_root_table() {
        let now = Instant::now();
        let prefix = key_from_byte(0x02);
        let z = key_from_byte(b'z');
        let tables = FakeTables::new().bind(DEFAULT_KEY_TABLE, z, false);
        let mut keys = ClientKeyState::new(now);

        keys.resolve(prefix, now, &tables);
        assert_eq!(keys.resolve(z, now, &tables), binding(DEFAULT_KEY_TABLE));
    }

    #[test]
    fn prefix_timeout_drops_the_late_command_and_retries_in_root() {
        let start = Instant::now();
        let prefix = key_from_byte(0x02);
        let c = key_from_byte(b'c');
        let mut tables = FakeTables::new().bind(PREFIX_TABLE, c, false);
        tables.prefix_timeout = 200;
        let mut keys = ClientKeyState::new(start);

        keys.resolve(prefix, start, &tables);
        // The retry happens in the root table with `first` reset to it, so the
        // key reaches the pane rather than being swallowed — the prefix command
        // is simply lost.
        let late = start + Duration::from_millis(800);
        assert_eq!(keys.resolve(c, late, &tables), KeyResolution::Forward);
        assert_eq!(keys.table(), DEFAULT_KEY_TABLE);

        // Inside the window the command still runs.
        let mut keys = ClientKeyState::new(start);
        keys.resolve(prefix, start, &tables);
        let soon = start + Duration::from_millis(150);
        assert_eq!(keys.resolve(c, soon, &tables), binding(PREFIX_TABLE));
    }

    #[test]
    fn a_repeating_binding_keeps_the_prefix_table_for_repeat_time() {
        let start = Instant::now();
        let prefix = key_from_byte(0x02);
        let f1 = key_from_byte(b'q');
        let mut tables = FakeTables::new().bind(PREFIX_TABLE, f1, true);
        tables.repeat_time = 2000;
        let mut keys = ClientKeyState::new(start);

        keys.resolve(prefix, start, &tables);
        assert_eq!(keys.resolve(f1, start, &tables), binding(PREFIX_TABLE));
        assert_eq!(keys.table(), PREFIX_TABLE);
        assert_eq!(
            keys.repeat_deadline(),
            Some(start + Duration::from_millis(2000))
        );

        // The bare key repeats without the prefix.
        let next = start + Duration::from_millis(250);
        assert_eq!(keys.resolve(f1, next, &tables), binding(PREFIX_TABLE));
        assert_eq!(
            keys.repeat_deadline(),
            Some(next + Duration::from_millis(2000))
        );
    }

    #[test]
    fn the_repeat_chain_lapses_on_its_own_timer() {
        let start = Instant::now();
        let prefix = key_from_byte(0x02);
        let f1 = key_from_byte(b'q');
        let mut tables = FakeTables::new().bind(PREFIX_TABLE, f1, true);
        tables.repeat_time = 200;
        let mut keys = ClientKeyState::new(start);

        keys.resolve(prefix, start, &tables);
        keys.resolve(f1, start, &tables);

        assert!(!keys.expire_repeat(start + Duration::from_millis(100), DEFAULT_KEY_TABLE));
        assert!(keys.expire_repeat(start + Duration::from_millis(300), DEFAULT_KEY_TABLE));
        assert_eq!(keys.table(), DEFAULT_KEY_TABLE);
        assert_eq!(keys.repeat_deadline(), None);

        // With the chain gone the bare key is the pane's.
        let later = start + Duration::from_millis(900);
        assert_eq!(keys.resolve(f1, later, &tables), KeyResolution::Forward);
    }

    #[test]
    fn initial_repeat_time_only_lengthens_the_first_press() {
        let start = Instant::now();
        let prefix = key_from_byte(0x02);
        let f1 = key_from_byte(b'q');
        let mut tables = FakeTables::new().bind(PREFIX_TABLE, f1, true);
        tables.repeat_time = 200;
        tables.initial_repeat_time = 1500;
        let mut keys = ClientKeyState::new(start);

        keys.resolve(prefix, start, &tables);
        keys.resolve(f1, start, &tables);
        assert_eq!(
            keys.repeat_deadline(),
            Some(start + Duration::from_millis(1500))
        );

        let second = start + Duration::from_millis(700);
        assert_eq!(keys.resolve(f1, second, &tables), binding(PREFIX_TABLE));
        assert_eq!(
            keys.repeat_deadline(),
            Some(second + Duration::from_millis(200))
        );
    }

    #[test]
    fn a_live_repeat_chain_survives_the_prefix_timeout() {
        let start = Instant::now();
        let prefix = key_from_byte(0x02);
        let f1 = key_from_byte(b'q');
        let mut tables = FakeTables::new().bind(PREFIX_TABLE, f1, true);
        tables.prefix_timeout = 200;
        tables.repeat_time = 3000;
        let mut keys = ClientKeyState::new(start);

        keys.resolve(prefix, start, &tables);
        keys.resolve(f1, start, &tables);
        // Later than prefix-timeout, well inside repeat-time.
        let late = start + Duration::from_millis(500);
        assert_eq!(keys.resolve(f1, late, &tables), binding(PREFIX_TABLE));
    }

    #[test]
    fn a_non_repeating_binding_ends_the_chain_and_retries_in_root() {
        let start = Instant::now();
        let prefix = key_from_byte(0x02);
        let f1 = key_from_byte(b'q');
        let c = key_from_byte(b'c');
        let mut tables = FakeTables::new()
            .bind(PREFIX_TABLE, f1, true)
            .bind(PREFIX_TABLE, c, false);
        tables.repeat_time = 3000;
        let mut keys = ClientKeyState::new(start);

        keys.resolve(prefix, start, &tables);
        keys.resolve(f1, start, &tables);
        // `c` is bound in `prefix` but does not repeat: the chain stops and the
        // key is retried in `root`, where nothing has it.
        let next = start + Duration::from_millis(300);
        assert_eq!(keys.resolve(c, next, &tables), KeyResolution::Forward);
        assert_eq!(keys.table(), DEFAULT_KEY_TABLE);
        assert_eq!(keys.repeat_deadline(), None);
    }

    #[test]
    fn repeat_time_zero_disables_chains() {
        let start = Instant::now();
        let prefix = key_from_byte(0x02);
        let f1 = key_from_byte(b'q');
        let mut tables = FakeTables::new().bind(PREFIX_TABLE, f1, true);
        tables.repeat_time = 0;
        let mut keys = ClientKeyState::new(start);

        keys.resolve(prefix, start, &tables);
        assert_eq!(keys.resolve(f1, start, &tables), binding(PREFIX_TABLE));
        assert_eq!(keys.table(), DEFAULT_KEY_TABLE);
        assert_eq!(keys.repeat_deadline(), None);
    }

    #[test]
    fn a_mode_table_lends_itself_to_the_default_table() {
        let now = Instant::now();
        let key = key_from_byte(b'q');
        let other = key_from_byte(b'x');
        let mut tables = FakeTables::new().bind("copy-mode", key, false);
        tables.mode_table = Some("copy-mode".to_string());
        let mut keys = ClientKeyState::new(now);

        assert_eq!(keys.resolve(key, now, &tables), binding("copy-mode"));
        // Unbound in the mode table and in root: consumed, not forwarded.
        assert_eq!(keys.resolve(other, now, &tables), KeyResolution::Consumed);
    }

    #[test]
    fn a_root_binding_wins_when_the_mode_table_has_none() {
        let now = Instant::now();
        let key = key_from_byte(b'q');
        let mut tables = FakeTables::new().bind(DEFAULT_KEY_TABLE, key, false);
        tables.mode_table = Some("copy-mode".to_string());
        let mut keys = ClientKeyState::new(now);

        assert_eq!(keys.resolve(key, now, &tables), binding(DEFAULT_KEY_TABLE));
    }
}
