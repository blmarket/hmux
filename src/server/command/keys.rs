//! The key-binding and key-injection commands.

use super::*;

/// `send-keys` and `send-prefix` keep their bodies beside tmux's
/// `cmd-send-keys.c` counterpart, so their argument types live there too.
pub(in crate::server) use crate::server::cmd_send_keys::{SendKeys, SendPrefix};

#[derive(Clone, Debug)]
pub(in crate::server) enum Command {
    Bind(BindKey),
    Unbind(UnbindKey),
    List(ListKeys),
    Send(SendKeys),
    SendPrefix(SendPrefix),
}

impl Command {
    pub(super) fn execute(self, context: &mut CommandContext<'_>) -> CommandResult {
        match self {
            Self::Bind(command) => command.execute(context.state),
            Self::Unbind(command) => command.execute(context.state),
            Self::List(command) => command.execute(context.state),
            Self::Send(command) => crate::server::cmd_send_keys::exec(
                &command,
                context.state,
                context.agents,
                context.client,
            ),
            Self::SendPrefix(command) => {
                crate::server::cmd_send_keys::exec_prefix(&command, context.state, context.client)
            }
        }
    }
}

/// `bind-key [-nr] [-T key-table] [-N note] key [command [argument ...]]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct BindKey {
    /// `-n`: bind in the root table instead of the prefix one.
    root: bool,
    /// `-r`: the binding repeats without the prefix.
    repeat: bool,
    /// `-T`: the table to bind in.
    table: Option<String>,
    /// `-N`: the note `list-keys -N` prints.
    note: Option<String>,
    /// The key, then the command line it runs.
    operands: Vec<String>,
}

impl BindKey {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            root: args.has('n'),
            repeat: args.has('r'),
            table: args.value('T').map(str::to_string),
            note: args.value('N').map(str::to_string),
            operands: args.positionals().to_vec(),
        })
    }

    /// Defaults and user replacements share the same semantic tables consumed by
    /// attached clients.
    fn execute(self, st: &mut ServerState) -> CommandResult {
        let table = if self.root {
            "root"
        } else {
            self.table.as_deref().unwrap_or("prefix")
        };
        let Some(key_name) = self.operands.first() else {
            return CommandResult::err("missing key\n");
        };
        let Some(key) = parse_key_name(key_name).filter(|key| key.is_bindable()) else {
            return CommandResult::err(format!("unknown key: {key_name}\n"));
        };
        // tmux parses the binding's command when the binding is made, so a
        // `command-alias` is resolved here and the binding keeps what it expanded
        // to — redefining the alias afterwards leaves it alone.
        let command = expand_command_aliases(&self.operands[1..], st);
        st.bind_key(table, key, command, self.repeat, self.note);
        CommandResult::ok("")
    }
}

/// `unbind-key [-anq] [-T key-table] key`.
#[derive(Clone, Debug)]
pub(in crate::server) struct UnbindKey {
    /// `-a`: unbind everything in the table.
    all: bool,
    /// `-n`: act on the root table instead of the prefix one.
    root: bool,
    /// `-q`: report nothing, whatever happens.
    quiet: bool,
    /// `-T`: the table to unbind in. Whether it was given matters: only an
    /// explicit table has to exist.
    table: Option<String>,
    key: Option<String>,
}

impl UnbindKey {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            all: args.has('a'),
            root: args.has('n'),
            quiet: args.has('q'),
            table: args.value('T').map(str::to_string),
            key: args.positionals().first().cloned(),
        })
    }

    fn execute(self, st: &mut ServerState) -> CommandResult {
        let quiet = self.quiet;
        let error = |message| {
            if quiet {
                CommandResult::ok("")
            } else {
                CommandResult::err(message)
            }
        };
        let table = self
            .table
            .as_deref()
            .unwrap_or(if self.root { "root" } else { "prefix" });

        if self.all {
            if self.key.is_some() {
                return error("key given with -a\n".to_string());
            }
            if !st.key_table_exists(table) {
                return error(format!("table {table} doesn't exist\n"));
            }
            st.unbind_key(table, None, true);
            return CommandResult::ok("");
        }

        let Some(key_name) = self.key.as_deref() else {
            return error("missing key\n".to_string());
        };
        let Some(key) = parse_key_name(key_name).filter(|key| key.is_bindable()) else {
            return error(format!("unknown key: {key_name}\n"));
        };
        if self.table.is_some() && !st.key_table_exists(table) {
            return error(format!("table {table} doesn't exist\n"));
        }

        // tmux treats an unbound key in an existing table as a successful no-op.
        st.unbind_key(table, Some(key), false);
        CommandResult::ok("")
    }
}

/// `list-keys [-1aNr] [-F format] [-O order] [-P prefix-string] [-T key-table] [key]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct ListKeys {
    /// `-a`: with `-N`, list bindings that have no note too.
    all: bool,
    /// `-N`: restrict the listing to bindings that carry a note.
    notes: bool,
    /// `-r`: restrict the listing to repeatable bindings.
    repeat_only: bool,
    /// `-T`: the table to list; every table otherwise.
    table: Option<String>,
    /// `-F`: the line each binding expands to.
    format: Option<String>,
    key: Option<String>,
}

impl ListKeys {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            all: args.has('a'),
            notes: args.has('N'),
            repeat_only: args.has('r'),
            table: args.value('T').map(str::to_string),
            format: args.value('F').map(str::to_string),
            key: args.positionals().first().cloned(),
        })
    }

    /// Lists the shared mutable key tables.
    fn execute(self, st: &ServerState) -> CommandResult {
        let table = self.table.as_deref();
        if let Some(table) = table {
            if !st.key_table_exists(table) {
                return CommandResult::err(format!("table {table} doesn't exist\n"));
            }
        }
        let requested = match self.key.as_deref() {
            Some(name) => match parse_key_name(name) {
                Some(key) => Some(key),
                None => return CommandResult::err(format!("invalid key: {name}\n")),
            },
            None => None,
        };
        let bindings = st.key_bindings(table);
        // `-F` expands a template per binding with tmux's `key_*` variables;
        // `-N` selects only bindings with notes unless `-a` is also given. `-r`
        // selects only repeatable bindings.
        if let Some(template) = self.format.as_deref() {
            let notes_filter = self.notes && !self.all;
            let mut filtered: Vec<_> = bindings
                .iter()
                .filter(|(_, key, binding)| {
                    !requested.is_some_and(|wanted| wanted != *key)
                        && (!notes_filter || binding.note.is_some())
                        && (!self.repeat_only || binding.repeat)
                })
                .collect();
            filtered.sort_by_key(|(_, key, _)| list_key_order(*key));
            let key_has_repeat = filtered.iter().any(|(_, _, binding)| binding.repeat);
            let key_string_width = filtered
                .iter()
                .map(|(_, key, _)| format_key_name(*key).chars().count())
                .max()
                .unwrap_or(0);
            let key_table_width = filtered
                .iter()
                .map(|(table_name, _, _)| table_name.chars().count())
                .max()
                .unwrap_or(0);
            let mut out = String::new();
            for (table_name, key, binding) in &filtered {
                let mut vars = format::Vars::new();
                vars.set("notes_only", if self.notes { "1" } else { "0" })
                    .set("key_has_repeat", if key_has_repeat { "1" } else { "0" })
                    .set("key_string_width", key_string_width.to_string())
                    .set("key_table_width", key_table_width.to_string())
                    .set("key_repeat", if binding.repeat { "1" } else { "0" })
                    .set("key_note", binding.note.clone().unwrap_or_default())
                    .set("key_table", *table_name)
                    .set("key_string", format_key_name(*key))
                    .set("key_command", binding.command.join(" "));
                let line = format::expand(template, &vars);
                if !line.is_empty() {
                    out.push_str(&line);
                    out.push('\n');
                }
            }
            // A sole match goes through the target client's status message, which
            // a detached command client sees as a successful empty result.
            if filtered.len() <= 1 {
                return CommandResult::ok("");
            }
            return CommandResult::ok(out);
        }
        let mut out = String::new();
        let mut matched = 0;
        for (table_name, key, binding) in bindings {
            if requested.is_some_and(|wanted| wanted != key) {
                continue;
            }
            out.push_str("bind-key");
            if binding.repeat {
                out.push_str(" -r");
            }
            out.push_str(" -T ");
            out.push_str(table_name);
            out.push(' ');
            out.push_str(&format_key_name(key));
            for word in &binding.command {
                out.push(' ');
                out.push_str(word);
            }
            out.push('\n');
            matched += 1;
        }
        // tmux 3.7b routes a sole match through the target client's status message
        // instead of command stdout. A detached command client therefore sees a
        // successful empty result for either zero or one matching binding.
        if matched <= 1 {
            return CommandResult::ok("");
        }
        CommandResult::ok(out)
    }
}

/// The order emitted by tmux's key table catalog: plain character keys first,
/// then named keys, followed by meta, control, and shift variants.
fn list_key_order(key: KeyCode) -> (u8, u8, u32) {
    let modifier_group = match (
        key.modifiers.meta(),
        key.modifiers.ctrl(),
        key.modifiers.shift(),
    ) {
        (false, false, false) => 0,
        (true, false, false) => 2,
        (false, true, false) => 3,
        (false, false, true) => 4,
        _ => 5,
    };
    match key.base {
        KeyBase::Char(value) => (modifier_group, 0, value as u32),
        KeyBase::Special(value) => (
            modifier_group,
            1,
            match value {
                SpecialKey::Delete => 0,
                SpecialKey::PageUp => 1,
                SpecialKey::Up => 2,
                SpecialKey::Down => 3,
                SpecialKey::Left => 4,
                SpecialKey::Right => 5,
                SpecialKey::F(number) => 10 + u32::from(number),
                SpecialKey::Insert => 30,
                SpecialKey::Home => 31,
                SpecialKey::End => 32,
                SpecialKey::PageDown => 33,
                SpecialKey::BackTab => 34,
                SpecialKey::Backspace => 35,
                SpecialKey::Keypad(value) => 40 + value as u32,
                SpecialKey::KeypadEnter => 50,
                SpecialKey::PasteStart => 51,
                SpecialKey::PasteEnd => 52,
            },
        ),
        KeyBase::User(value) => (modifier_group, 2, u32::from(value)),
        KeyBase::Mouse(_) => (modifier_group, 3, 0),
        KeyBase::Any => (modifier_group, 4, 0),
        KeyBase::None => (modifier_group, 5, 0),
    }
}
