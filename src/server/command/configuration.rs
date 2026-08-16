//! The option, hook, and environment commands.

use super::*;

#[derive(Clone, Debug)]
pub(in crate::server) enum Command {
    SetEnvironment(SetEnvironment),
    ShowEnvironment(ShowEnvironment),
    SetOption(SetOption),
    ShowOptions(ShowOptions),
    SetWindowOption(SetOption),
    ShowWindowOptions(ShowOptions),
    SetHook(SetHook),
    ShowHooks(ShowHooks),
}

impl Command {
    pub(super) async fn execute(self, context: &mut ExecContext<'_>) -> SharedCommandExecution {
        match self {
            // `set-hook -R` runs the hook's body, which the queue takes over as
            // items of its own.
            Self::SetHook(command) => command.execute(context),
            Self::SetEnvironment(command) => context.sync(|inner| command.run(inner.state)),
            Self::ShowEnvironment(command) => context.sync(|inner| command.run(inner.state)),
            Self::SetOption(command) => context.sync(|inner| command.run(inner.state, false)),
            Self::ShowOptions(command) => context.sync(|inner| command.run(inner.state, false)),
            Self::SetWindowOption(command) => context.sync(|inner| command.run(inner.state, true)),
            Self::ShowWindowOptions(command) => {
                context.sync(|inner| command.run(inner.state, true))
            }
            Self::ShowHooks(command) => context.sync(|inner| command.run(inner.state)),
        }
    }
}

// ---- option scope ----------------------------------------------------------

/// The flags every option command shares to pick which table it reads or
/// writes, and the target that table hangs off.
#[derive(Clone, Debug, Default)]
struct OptionScopeFlags {
    /// `-s`: the server table.
    server: bool,
    /// `-p`: the pane table.
    pane: bool,
    /// `-w`: the window table.
    window: bool,
    /// `-g`: the global table for the scope, rather than a target's own.
    global: bool,
    /// `-t`: the session, window, or pane the table hangs off.
    target: Option<String>,
}

impl OptionScopeFlags {
    fn parse(args: &ParsedArgs) -> Self {
        Self {
            server: args.has('s'),
            pane: args.has('p'),
            window: args.has('w'),
            global: args.has('g'),
            target: args.value('t').map(str::to_string),
        }
    }
}

#[derive(Clone, Copy)]
enum OptionTarget {
    Server,
    GlobalSession,
    Session(usize),
    GlobalWindow,
    Window(Target),
    Pane(Target),
}

impl OptionTarget {
    fn local(self, st: &ServerState) -> &OptionSet {
        match self {
            Self::Server => st.global_options().server(),
            Self::GlobalSession => st.global_options().session(),
            Self::Session(session) => st.sessions()[session].option_overrides(),
            Self::GlobalWindow => st.global_options().window(),
            Self::Window(target) => st.window(target.session, target.window).option_overrides(),
            Self::Pane(target) => {
                st.window(target.session, target.window).panes[target.pane].option_overrides()
            }
        }
    }

    fn view<'a>(self, st: &'a ServerState) -> OptionsView<'a> {
        match self {
            Self::Server => st.server_options(),
            Self::GlobalSession => OptionsView::one(st.global_options().session()),
            Self::Session(session) => st.sessions()[session].options(st.global_options()),
            Self::GlobalWindow => OptionsView::one(st.global_options().window()),
            Self::Window(target) => st
                .window(target.session, target.window)
                .options(st.global_options()),
            Self::Pane(target) => {
                let window = st.window(target.session, target.window);
                window.panes[target.pane].options(window, st.global_options())
            }
        }
    }

    fn local_mut(self, st: &mut ServerState) -> &mut OptionSet {
        match self {
            Self::Server => st.global_options_mut().for_scope_mut(OptionScope::Server),
            Self::GlobalSession => st.global_options_mut().for_scope_mut(OptionScope::Session),
            Self::Session(session) => st.session_mut(session).option_overrides_mut(),
            Self::GlobalWindow => st.global_options_mut().for_scope_mut(OptionScope::Window),
            Self::Window(target) => st
                .window_mut(target.session, target.window)
                .option_overrides_mut(),
            Self::Pane(target) => st.window_mut(target.session, target.window).panes[target.pane]
                .option_overrides_mut(),
        }
    }

    fn is_global(self) -> bool {
        matches!(
            self,
            Self::Server | Self::GlobalSession | Self::GlobalWindow
        )
    }

    fn scope(self) -> OptionScope {
        match self {
            Self::Server => OptionScope::Server,
            Self::GlobalSession | Self::Session(_) => OptionScope::Session,
            Self::GlobalWindow | Self::Window(_) => OptionScope::Window,
            Self::Pane(_) => OptionScope::WindowPane,
        }
    }
}

#[derive(Clone, Copy)]
enum OptionTargetKind {
    Session,
    Window,
    Pane,
}

impl OptionTargetKind {
    fn name(self) -> &'static str {
        match self {
            Self::Session => "session",
            Self::Window => "window",
            Self::Pane => "pane",
        }
    }
}

fn option_command_target(
    scope: &OptionScopeFlags,
    st: &ServerState,
    kind: OptionTargetKind,
) -> Result<Target, CommandResult> {
    let explicit = scope.target.as_deref();
    let target = scope.target.clone().or_else(|| current_target(st));
    let Some(target) = target else {
        return Err(CommandResult::err(format!("no current {}\n", kind.name())));
    };
    st.resolve(&target).ok_or_else(|| {
        if explicit.is_some() {
            CommandResult::err(format!("no such {}: {target}\n", kind.name()))
        } else {
            CommandResult::err(format!("no current {}\n", kind.name()))
        }
    })
}

fn option_target_from_flags(
    scope: &OptionScopeFlags,
    st: &ServerState,
    window_command: bool,
) -> Result<OptionTarget, CommandResult> {
    if scope.server {
        return Ok(OptionTarget::Server);
    }
    if scope.pane {
        Ok(OptionTarget::Pane(option_command_target(
            scope,
            st,
            OptionTargetKind::Pane,
        )?))
    } else if window_command || scope.window {
        if scope.global {
            Ok(OptionTarget::GlobalWindow)
        } else {
            Ok(OptionTarget::Window(option_command_target(
                scope,
                st,
                OptionTargetKind::Window,
            )?))
        }
    } else if scope.global {
        Ok(OptionTarget::GlobalSession)
    } else {
        Ok(OptionTarget::Session(
            option_command_target(scope, st, OptionTargetKind::Session)?.session,
        ))
    }
}

fn option_target_from_name(
    scope: &OptionScopeFlags,
    st: &ServerState,
    option_scope: OptionScope,
) -> Result<OptionTarget, CommandResult> {
    match option_scope {
        OptionScope::Server => Ok(OptionTarget::Server),
        OptionScope::Session if scope.global => Ok(OptionTarget::GlobalSession),
        OptionScope::Session => Ok(OptionTarget::Session(
            option_command_target(scope, st, OptionTargetKind::Session)?.session,
        )),
        OptionScope::WindowPane if scope.pane => Ok(OptionTarget::Pane(option_command_target(
            scope,
            st,
            OptionTargetKind::Pane,
        )?)),
        OptionScope::Window | OptionScope::WindowPane if scope.global => {
            Ok(OptionTarget::GlobalWindow)
        }
        OptionScope::Window | OptionScope::WindowPane => Ok(OptionTarget::Window(
            option_command_target(scope, st, OptionTargetKind::Window)?,
        )),
    }
}

/// Re-print a command list with each command's canonical name, the way tmux's
/// `cmd_list_print` does for a stored command option. A body that does not
/// parse is kept verbatim so a not-yet-valid hook still round-trips.
fn canonical_command_list(value: &str, st: &ServerState) -> String {
    let tokens = tokenize_line(value);
    let groups = tokenized_command_groups(&tokens);
    if groups.is_empty() {
        return value.to_string();
    }
    let aliases = st.command_aliases();
    let mut printed = Vec::with_capacity(groups.len());
    for group in &groups {
        let Some(first) = group.first() else {
            return value.to_string();
        };
        let canonical = match registry::resolve(first) {
            Resolution::Name(name) => name.to_string(),
            _ => match aliases.iter().find(|(alias, _)| alias == first) {
                // An alias expands to a whole command line; leave it alone
                // rather than half-rewriting it.
                Some(_) => return value.to_string(),
                None => return value.to_string(),
            },
        };
        let mut rewritten = group.clone();
        rewritten[0] = canonical;
        printed.push(display_command(&rewritten));
    }
    printed.join(" ; ")
}

/// Expand an option-name operand the way tmux's `format_single_from_target`
/// does: against the command's `-t` target (its pane, window and session), the
/// server environment and the options visible there. A target that no longer
/// resolves leaves the operand untouched, so the caller still reports it
/// verbatim.
fn expand_option_name_argument(
    scope: &OptionScopeFlags,
    st: &ServerState,
    argument: &str,
) -> String {
    if !format::reads_vars(argument) {
        return argument.to_string();
    }
    let target = scope.target.clone().or_else(|| current_target(st));
    let Some(resolved) = target.as_deref().and_then(|target| st.resolve(target)) else {
        return argument.to_string();
    };
    let mut vars = vars_full(
        st,
        &st.sessions()[resolved.session],
        resolved.window,
        resolved.pane,
        &PaneAgents::new(),
        st.marked_pane(),
    );
    for (name, value) in st.env_iter() {
        vars.set(name.to_string(), value);
    }
    if let Ok(entries) = st.format_option_entries(target.as_deref().unwrap_or_default()) {
        for (name, value) in entries {
            vars.set(name.to_string(), value);
        }
    }
    expand_command_format(st, argument, &vars, None)
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum OptionArgumentResult<'a> {
    Resolved(&'a str, Option<u32>),
    Ambiguous,
    Invalid,
}

fn resolve_option_argument(argument: &str) -> OptionArgumentResult<'_> {
    let Some((name, index)) = options::parse_option_name(argument) else {
        return OptionArgumentResult::Invalid;
    };
    if name.starts_with('@') {
        OptionArgumentResult::Resolved(name, index)
    } else {
        match options::resolve_option_name(argument) {
            options::OptionResolveResult::Found(resolved, index) => {
                OptionArgumentResult::Resolved(resolved, index)
            }
            options::OptionResolveResult::Ambiguous => OptionArgumentResult::Ambiguous,
            options::OptionResolveResult::Invalid => OptionArgumentResult::Invalid,
        }
    }
}

/// A numeric option value with tmux's strtonum syntax: an optional sign and
/// decimal digits. Values beyond i64 saturate so range checks still report
/// too large / too small rather than invalid.
fn parse_option_number(value: &str) -> Result<i64, ()> {
    let digits = value.strip_prefix('-').unwrap_or(value);
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(());
    }
    Ok(value.parse::<i64>().unwrap_or(if value.starts_with('-') {
        i64::MIN
    } else {
        i64::MAX
    }))
}

fn option_in_listing_scope(name: &str, scope: OptionScope) -> bool {
    if name.starts_with('@') {
        return true;
    }
    let base = options::parse_option_name(name)
        .map(|(base, _)| base)
        .unwrap_or(name);
    match (options::option_scope(base), scope) {
        (Some(OptionScope::WindowPane), OptionScope::Window | OptionScope::WindowPane) => true,
        (candidate, expected) => candidate == Some(expected),
    }
}

fn tmux_escape_argument(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    let quote = if value.chars().any(|ch| " #';${}%".contains(ch)) {
        Some('"')
    } else if value.chars().any(|ch| " \"".contains(ch)) {
        Some('\'')
    } else {
        None
    };
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{1b}' => escaped.push_str("\\e"),
            '\\' => escaped.push_str("\\\\"),
            '"' if quote == Some('"') => escaped.push_str("\\\""),
            '\'' if quote == Some('\'') => escaped.push_str("\\'"),
            other => escaped.push(other),
        }
    }
    match quote {
        Some(quote) => format!("{quote}{escaped}{quote}"),
        None => escaped,
    }
}

fn show_option_value(name: &str, value: &str) -> String {
    let base = options::parse_option_name(name)
        .map(|(base, _)| base)
        .unwrap_or(name);
    if options::is_hook(base) {
        value.to_string()
    } else {
        tmux_escape_argument(value)
    }
}

// ---- set-option / set-window-option ----------------------------------------

/// `set-option [-aFgopqsuUw] [-t target-pane] option [value]`, and its
/// `set-window-option` spelling.
#[derive(Clone, Debug)]
pub(in crate::server) struct SetOption {
    /// `-a`: append to the option's current value.
    append: bool,
    /// `-F`: expand the value as a format first.
    expand: bool,
    /// `-o`: fail rather than overwrite an option that is already set.
    only_if_unset: bool,
    /// `-q`: report nothing when the option name is not usable.
    quiet: bool,
    /// `-u`: unset the option.
    unset: bool,
    /// `-U`: unset it in the target window's panes too.
    unset_panes: bool,
    scope: OptionScopeFlags,
    /// The option name, then its value.
    operands: Vec<String>,
}

impl SetOption {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            append: args.has('a'),
            scope: OptionScopeFlags::parse(args),
            expand: args.has('F'),
            only_if_unset: args.has('o'),
            quiet: args.has('q'),
            unset: args.has('u'),
            unset_panes: args.has('U'),
            operands: args.positionals().to_vec(),
        })
    }

    /// Set an option in the local table selected by its catalog scope and target.
    fn run(self, st: &mut ServerState, window_command: bool) -> CommandResult {
        let argument = match self.operands.first() {
            Some(argument) => argument.as_str(),
            None => return CommandResult::err("set-option: missing option\n"),
        };
        let (name, index) = match resolve_option_argument(argument) {
            OptionArgumentResult::Resolved(name, index) => (name, index),
            OptionArgumentResult::Ambiguous => {
                return if self.quiet {
                    CommandResult::ok("")
                } else {
                    CommandResult::err(format!("ambiguous option: {argument}\n"))
                };
            }
            OptionArgumentResult::Invalid => {
                return if self.quiet {
                    CommandResult::ok("")
                } else {
                    CommandResult::err(format!("invalid option: {argument}\n"))
                };
            }
        };
        let user_option = name.starts_with('@');
        let kind = (!user_option).then(|| options::option_kind(name)).flatten();
        if !user_option && kind.is_none() {
            return if self.quiet {
                CommandResult::ok("")
            } else {
                CommandResult::err(format!("invalid option: {argument}\n"))
            };
        }
        // Unlike invalid names, tmux never suppresses this diagnostic with `-q`.
        if index.is_some() && (user_option || kind != Some(options::OptionKind::Array)) {
            return CommandResult::err(format!("not an array: {argument}\n"));
        }
        let target = if user_option {
            match option_target_from_flags(&self.scope, st, window_command) {
                Ok(target) => target,
                Err(error) => return error,
            }
        } else {
            match option_target_from_name(
                &self.scope,
                st,
                options::option_scope(name).expect("known option has scope"),
            ) {
                Ok(target) => target,
                Err(error) => return error,
            }
        };
        let storage_name = match index {
            Some(index) => format!("{name}[{index}]"),
            None => name.to_string(),
        };
        let already_set = if index.is_none() && kind == Some(options::OptionKind::Array) {
            target.local(st).contains_array(name)
        } else {
            target.local(st).contains(&storage_name)
        };
        if self.only_if_unset && already_set {
            return if self.quiet {
                CommandResult::ok("")
            } else {
                CommandResult::err(format!("already set: {argument}\n"))
            };
        }
        let unset = self.unset || self.unset_panes;
        let raw_value = self.operands.get(1).map(String::as_str);
        let mut value = match raw_value {
            Some(raw) if self.expand => match current_session(st).and_then(|name| st.find(&name)) {
                Some(sess) => {
                    let window = st.command_window_index(sess);
                    expand_command_format(
                        st,
                        raw,
                        &vars_for(st, sess, window, &PaneAgents::new(), st.marked_pane()),
                        None,
                    )
                }
                None => raw.to_string(),
            },
            Some(raw) => raw.to_string(),
            None => String::new(),
        };
        // tmux parses a hook (or any command-typed option) into a command list at
        // assignment time, so what `show-hooks` prints back is the canonical
        // command name rather than whatever alias the body was written with.
        if !unset && raw_value.is_some() && options::is_hook(name) {
            value = canonical_command_list(&value, st);
        }
        if !unset {
            if let Some((min, max)) = options::option_number_range(name) {
                match parse_option_number(&value) {
                    Err(()) => return CommandResult::err(format!("value is invalid: {value}\n")),
                    Ok(number) if number < min => {
                        return CommandResult::err(format!("value is too small: {value}\n"))
                    }
                    Ok(number) if number > max => {
                        return CommandResult::err(format!("value is too large: {value}\n"))
                    }
                    Ok(_) => {}
                }
            } else if let Some(choices) = options::option_choices(name) {
                if raw_value.is_none() {
                    // No value toggles between the first two choices; from any
                    // later choice the current value is kept (tmux
                    // options_from_string_choice).
                    let current = target
                        .view(st)
                        .get(name)
                        .map(str::to_string)
                        .or_else(|| options::option_initial_default(name));
                    let index = current
                        .as_deref()
                        .and_then(|current| choices.iter().position(|choice| *choice == current));
                    value = match index {
                        Some(0) => choices[1].to_string(),
                        Some(index) if index >= 2 => choices[index].to_string(),
                        _ => choices[0].to_string(),
                    };
                } else if !choices.contains(&value.as_str()) {
                    return CommandResult::err(format!("unknown value: {value}\n"));
                }
            } else if options::option_is_flag(name) {
                let extension = options::flag_extension_value(name);
                value = match value.as_str() {
                    "on" | "yes" | "1" => "on".to_string(),
                    "off" | "no" | "0" => "off".to_string(),
                    "" => match target.view(st).get(name) {
                        // No value toggles the flag, except from an extension
                        // value, which is kept the way tmux keeps a choice past
                        // the first two (options_from_string_choice).
                        Some(current) if Some(current) == extension => current.to_string(),
                        Some("on") => "off".to_string(),
                        _ => "on".to_string(),
                    },
                    other if Some(other) == extension => other.to_string(),
                    _ => return CommandResult::err(format!("bad value: {value}\n")),
                };
            }
        }
        if self.unset_panes {
            let window_target = match target {
                OptionTarget::Window(target) => Some(target),
                OptionTarget::GlobalWindow => {
                    match option_command_target(&self.scope, st, OptionTargetKind::Window) {
                        Ok(target) => Some(target),
                        Err(error) => return error,
                    }
                }
                _ => None,
            };
            if let Some(window_target) = window_target {
                for pane in &mut st
                    .window_mut(window_target.session, window_target.window)
                    .panes
                {
                    if kind == Some(options::OptionKind::Array) && index.is_none() {
                        pane.option_overrides_mut().clear_array(name);
                    } else {
                        pane.option_overrides_mut().remove(&storage_name);
                    }
                }
            }
        }
        if kind == Some(options::OptionKind::Array) && index.is_none() {
            let global = target.is_global();
            let table = target.local_mut(st);
            if unset {
                table.clear_array(name);
                if global {
                    options::restore_array_default(table, name);
                }
            } else {
                if !self.append {
                    table.clear_array(name);
                }
                let separator = options::option_array_separator(name).unwrap_or(" ,");
                let values = if separator.is_empty() {
                    (!value.is_empty())
                        .then_some(value.as_str())
                        .into_iter()
                        .collect::<Vec<_>>()
                } else {
                    value
                        .split(|character| separator.contains(character))
                        .filter(|value| !value.is_empty())
                        .collect::<Vec<_>>()
                };
                for value in values {
                    let index = table.next_array_index(name);
                    table.set(format!("{name}[{index}]"), value);
                }
            }
            st.option_changed(name);
            return CommandResult::ok("");
        }
        if unset {
            let table = target.local_mut(st);
            table.remove(&storage_name);
            if target.is_global() && index.is_none() && !user_option {
                if let Some(default) = options::option_initial_default(name) {
                    table.set(name, default);
                }
            }
        } else if self.append {
            let table = target.local_mut(st);
            if !table.contains(&storage_name) && !user_option && index.is_none() {
                if let Some(default) = options::option_initial_default(name) {
                    table.set(&storage_name, default);
                }
            }
            if options::is_style_option(name) {
                if let Some(existing) = table.get(&storage_name) {
                    if !existing.is_empty() && !value.is_empty() {
                        table.append(&storage_name, ",");
                    }
                }
            }
            table.append(&storage_name, &value);
        } else {
            target.local_mut(st).set(&storage_name, &value);
        }
        st.option_changed(name);
        if name == "history-file" {
            st.load_prompt_history();
        }
        st.enforce_unattached_options();
        CommandResult::ok("")
    }
}

// ---- show-options / show-window-options ------------------------------------

/// `show-options [-AgHpqsvw] [-t target-pane] [option]`, and its
/// `show-window-options` spelling.
#[derive(Clone, Debug)]
pub(in crate::server) struct ShowOptions {
    /// `-A`: include options inherited from a parent table.
    inherited: bool,
    /// `-q`: report nothing when the option name is not usable.
    quiet: bool,
    /// `-v`: print the value alone, without its name.
    value_only: bool,
    scope: OptionScopeFlags,
    /// The single option to show, instead of the whole table.
    option: Option<String>,
}

impl ShowOptions {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            inherited: args.has('A'),
            quiet: args.has('q'),
            value_only: args.has('v'),
            scope: OptionScopeFlags::parse(args),
            option: args.positionals().first().cloned(),
        })
    }

    fn run(&self, st: &ServerState, window_command: bool) -> CommandResult {
        let value_only = self.value_only;
        // `-q` swallows the invalid-option diagnostic, turning any name error into a
        // silent success (exit 0, no output) — matching real tmux.
        let quiet = self.quiet;
        // tmux format-expands the option name before matching it against the option
        // table, so `show-options #{@lookup}` looks up whatever `@lookup` holds.
        let expanded = self
            .option
            .as_deref()
            .map(|argument| expand_option_name_argument(&self.scope, st, argument));
        let argument = match expanded.as_deref() {
            Some(argument) => argument,
            None => {
                let target = match option_target_from_flags(&self.scope, st, window_command) {
                    Ok(target) => target,
                    Err(_error) if quiet => return CommandResult::ok(""),
                    Err(error) => return error,
                };
                let inherited = self.inherited;
                let local = target.local(st);
                let entries: Vec<_> = if inherited {
                    target.view(st).iter_effective().collect()
                } else {
                    local.iter().collect()
                };
                let mut output = String::new();
                let mut user_options = entries
                    .iter()
                    .filter(|(name, _)| name.starts_with('@'))
                    .collect::<Vec<_>>();
                user_options.sort_by_key(|(name, _)| *name);
                for (name, value) in user_options {
                    let parent = inherited && !local.contains(name);
                    if value_only {
                        output.push_str(value);
                    } else if parent {
                        output.push_str(&format!("{name}* {}\n", show_option_value(name, value)));
                    } else {
                        output.push_str(&format!("{name} {}\n", show_option_value(name, value)));
                    }
                }

                for &name in options::OPTIONS_CATALOG_ORDER {
                    if !option_in_listing_scope(name, target.scope()) {
                        continue;
                    }
                    if options::is_array_option(name) {
                        let view = target.view(st);
                        let array_entries: Vec<_> = if inherited {
                            view.iter_effective().collect()
                        } else {
                            local.iter().collect()
                        };
                        let mut array_entries = array_entries
                            .into_iter()
                            .filter_map(|(entry, val)| {
                                let (base, index) = options::parse_option_name(entry)?;
                                (base == name).then_some((index?, val))
                            })
                            .collect::<Vec<_>>();
                        array_entries.sort_by_key(|(idx, _)| *idx);
                        if array_entries.is_empty() {
                            if !value_only
                                && (local.contains_array(name)
                                    || (inherited && target.view(st).contains_array(name)))
                            {
                                output.push_str(name);
                                output.push('\n');
                            }
                        } else {
                            let parent = inherited && !local.contains_array(name);
                            for (index, val) in array_entries {
                                if value_only {
                                    output.push_str(val);
                                } else if parent {
                                    output.push_str(&format!(
                                        "{name}[{index}]* {}\n",
                                        show_option_value(name, val)
                                    ));
                                } else {
                                    output.push_str(&format!(
                                        "{name}[{index}] {}\n",
                                        show_option_value(name, val)
                                    ));
                                }
                            }
                        }
                    } else {
                        let local_val = local.get(name);
                        let parent = inherited && local_val.is_none();
                        let val = local_val
                            .or_else(|| inherited.then(|| target.view(st).get(name)).flatten());
                        if let Some(val) = val {
                            if value_only {
                                output.push_str(val);
                            } else if parent {
                                output.push_str(&format!(
                                    "{name}* {}\n",
                                    show_option_value(name, val)
                                ));
                            } else {
                                output.push_str(&format!(
                                    "{name} {}\n",
                                    show_option_value(name, val)
                                ));
                            }
                        }
                    }
                }
                return CommandResult::ok(output);
            }
        };

        let (name, index) = match resolve_option_argument(argument) {
            OptionArgumentResult::Resolved(name, index) => (name, index),
            OptionArgumentResult::Ambiguous => {
                return if quiet {
                    CommandResult::ok("")
                } else {
                    CommandResult::err(format!("ambiguous option: {argument}\n"))
                };
            }
            OptionArgumentResult::Invalid => {
                return if quiet {
                    CommandResult::ok("")
                } else {
                    CommandResult::err(format!("invalid option: {argument}\n"))
                };
            }
        };
        let display_name = match index {
            Some(index) => format!("{name}[{index}]"),
            None => name.to_string(),
        };

        let user_option = name.starts_with('@');
        let kind = (!user_option).then(|| options::option_kind(name)).flatten();
        if !user_option && kind.is_none() {
            return if quiet {
                CommandResult::ok("")
            } else {
                CommandResult::err(format!("invalid option: {argument}\n"))
            };
        };

        let target = if user_option {
            match option_target_from_flags(&self.scope, st, window_command) {
                Ok(target) => target,
                Err(_error) if quiet => return CommandResult::ok(""),
                Err(error) => return error,
            }
        } else {
            match option_target_from_name(
                &self.scope,
                st,
                options::option_scope(name).expect("known option has scope"),
            ) {
                Ok(target) => target,
                Err(_error) if quiet => return CommandResult::ok(""),
                Err(error) => return error,
            }
        };
        let storage_name = if index.is_some() && kind == Some(options::OptionKind::Array) {
            display_name.as_str()
        } else {
            name
        };
        let inherited = self.inherited;
        let local = target.local(st);
        if kind == Some(options::OptionKind::Array) && index.is_none() {
            let entries: Vec<_> = if inherited {
                target.view(st).iter_effective().collect()
            } else {
                local.iter().collect()
            };
            let mut entries = entries
                .into_iter()
                .filter_map(|(entry, value)| {
                    let (base, index) = options::parse_option_name(entry)?;
                    (base == name).then_some((index?, value))
                })
                .collect::<Vec<_>>();
            entries.sort_by_key(|(index, _)| *index);
            let parent = inherited && !local.contains_array(name);
            let mut output = String::new();
            for (index, value) in entries {
                if value_only {
                    output.push_str(value);
                } else if parent {
                    output.push_str(&format!(
                        "{name}[{index}]* {}",
                        show_option_value(name, value)
                    ));
                } else {
                    output.push_str(&format!(
                        "{name}[{index}] {}",
                        show_option_value(name, value)
                    ));
                }
                output.push('\n');
            }
            if output.is_empty()
                && !value_only
                && (local.contains_array(name)
                    || (inherited && target.view(st).contains_array(name)))
            {
                output.push_str(name);
                output.push('\n');
            }
            return CommandResult::ok(output);
        }
        let local_value = local.get(storage_name);
        let parent = inherited && local_value.is_none();
        let value = local_value
            .or_else(|| {
                inherited
                    .then(|| target.view(st).get(storage_name))
                    .flatten()
            })
            .map(str::to_string)
            .or_else(|| {
                (index.is_some()
                    && kind == Some(options::OptionKind::Array)
                    && (target.is_global()
                        || local.contains_array(name)
                        || (inherited && target.view(st).contains_array(name))))
                .then(String::new)
            });
        if user_option && value.is_none() {
            return if quiet {
                CommandResult::ok("")
            } else {
                CommandResult::err(format!("invalid option: {argument}\n"))
            };
        }
        match value {
            Some(value) if value_only => CommandResult::ok(format!("{value}\n")),
            Some(value) if parent => CommandResult::ok(format!(
                "{display_name}* {}\n",
                show_option_value(name, &value)
            )),
            Some(value) => CommandResult::ok(format!(
                "{display_name} {}\n",
                show_option_value(name, &value)
            )),
            None => CommandResult::ok(""),
        }
    }
}

// ---- set-hook / show-hooks -------------------------------------------------

/// `set-hook [-agpRuw] [-t target-pane] hook [command]`. A hook is an option,
/// so setting one is `set-option` under a different name.
#[derive(Clone, Debug)]
pub(in crate::server) struct SetHook {
    /// `-R`: run the hook's body rather than change it.
    run: bool,
    set: SetOption,
}

impl SetHook {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            run: args.has('R'),
            set: SetOption::parse(args)?,
        })
    }

    fn execute(self, context: &mut ExecContext<'_>) -> SharedCommandExecution {
        if !self.run {
            return context.sync(|inner| self.set.run(inner.state, false));
        }
        let Some(hook) = self.set.operands.first() else {
            return SharedCommandExecution::completed(CommandResult::err(
                "set-hook: missing hook\n",
            ));
        };
        if !options::is_hook(hook) {
            return SharedCommandExecution::completed(CommandResult::err(format!(
                "invalid option: {hook}\n"
            )));
        }
        // `-R` runs the hook's body as queue items of its own; the command's
        // own hooks wait until they have run.
        let mut insert_next = context.plan_hook_with_capture(
            hook,
            self.set.scope.target.as_deref(),
            vec![("hook".to_string(), hook.to_string())],
            NestedCapture::Discard,
            HookOrigin::Command,
        );
        insert_next.push(context.finalize_hooks("set-hook"));
        SharedCommandExecution {
            result: CommandResult::ok(""),
            insert_next,
            defer_success_hooks: true,
        }
    }
}

/// `show-hooks [-gpw] [-t target-pane] [hook]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct ShowHooks {
    /// `-p`: the pane hooks.
    pane: bool,
    /// `-w`: the window hooks as well as the pane ones.
    window: bool,
    /// `-g`: the global table, which also lists hooks that are unset.
    global: bool,
    /// Showing a hook is showing an option, one hook name at a time.
    show: ShowOptions,
}

impl ShowHooks {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            pane: args.has('p'),
            window: args.has('w'),
            global: args.has('g'),
            show: ShowOptions::parse(args)?,
        })
    }

    fn run(self, st: &ServerState) -> CommandResult {
        if let Some(hook) = self.show.option.as_deref() {
            if !options::is_hook(
                options::parse_option_name(hook)
                    .map(|(base, _)| base)
                    .unwrap_or(hook),
            ) {
                return CommandResult::err(format!("invalid option: {hook}\n"));
            }
            let result = self.show.run(st, false);
            if self.global && result.exit == 0 && result.stdout.is_empty() && !hook.contains('[') {
                return CommandResult::ok(format!("{hook}\n"));
            }
            return result;
        }

        let hooks = if self.pane {
            if self.global {
                Vec::new()
            } else {
                options::PANE_HOOKS.to_vec()
            }
        } else if self.window {
            options::PANE_HOOKS
                .iter()
                .chain(options::WINDOW_HOOKS)
                .copied()
                .collect()
        } else {
            options::SESSION_HOOKS.to_vec()
        };
        let mut output = String::new();
        for hook in hooks {
            let mut one = self.show.clone();
            one.option = Some((*hook).to_string());
            let shown = one.run(st, false);
            if shown.exit != 0 {
                return shown;
            } else if !shown.stdout.is_empty() {
                output.push_str(&shown.stdout);
            } else if self.global {
                output.push_str(hook);
                output.push('\n');
            }
        }
        CommandResult::ok(output)
    }
}

// ---- environment -----------------------------------------------------------

/// `set-environment [-Fhgru] [-t target-session] variable [value]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct SetEnvironment {
    /// `-F`: expand the value as a format first.
    expand: bool,
    /// `-h`: mark the variable hidden.
    hidden: bool,
    /// `-g`: act on the server environment rather than a session's.
    global: bool,
    /// `-r`: remove the variable outright.
    remove: bool,
    /// `-u`: unset the variable, leaving a tombstone.
    unset: bool,
    /// `-t`: the session whose environment is changed.
    target: Option<String>,
    /// The variable name, then its value.
    operands: Vec<String>,
}

impl SetEnvironment {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            expand: args.has('F'),
            hidden: args.has('h'),
            global: args.has('g'),
            remove: args.has('r'),
            unset: args.has('u'),
            target: args.value('t').map(str::to_string),
            operands: args.positionals().to_vec(),
        })
    }

    fn run(self, st: &mut ServerState) -> CommandResult {
        let name = match self.operands.first() {
            Some(name) => name.as_str(),
            None => return CommandResult::err("set-environment: missing variable\n"),
        };
        if name.is_empty() {
            return CommandResult::err("empty variable name\n");
        }
        if name.contains('=') {
            return CommandResult::err("variable name contains =\n");
        }
        let global = self.global;
        let target = if global {
            None
        } else {
            self.target.clone().or_else(|| current_target(st))
        };
        if !global && target.is_none() {
            return CommandResult::err("no current session\n");
        }
        if !global {
            let resolved = target.as_deref().unwrap_or_default();
            if st.resolve_session(resolved).is_none() {
                return match self.target.as_deref() {
                    Some(target) => CommandResult::err(format!("no such session: {target}\n")),
                    None => CommandResult::err("no current session\n"),
                };
            }
        }
        if self.unset || self.remove {
            if self.operands.get(1).is_some() {
                let flag = if self.unset { "-u" } else { "-r" };
                return CommandResult::err(format!("can't specify a value with {flag}\n"));
            }
            if global {
                if self.remove {
                    st.remove_env(name);
                } else {
                    st.unset_env(name);
                }
            } else if let Err(error) =
                st.unset_session_env(target.as_deref().unwrap(), name, self.remove)
            {
                return CommandResult::err(format!("{error}\n"));
            }
        } else {
            let Some(raw_value) = self.operands.get(1).map(String::as_str) else {
                return CommandResult::err("no value specified\n");
            };
            let value = if self.expand {
                let format_target = target.clone().or_else(|| current_session(st));
                match format_target.as_deref().and_then(|name| st.find(name)) {
                    Some(sess) => format::expand(
                        raw_value,
                        &vars_for(st, sess, sess.active, &PaneAgents::new(), st.marked_pane()),
                    ),
                    None => raw_value.to_string(),
                }
            } else {
                raw_value.to_string()
            };
            if global {
                if self.hidden {
                    st.set_hidden_env(name, &value);
                } else {
                    st.set_env(name, &value);
                }
            } else if let Err(error) =
                st.set_session_env(target.as_deref().unwrap(), name, &value, self.hidden)
            {
                return CommandResult::err(format!("{error}\n"));
            }
        }
        CommandResult::ok("")
    }
}

/// `show-environment [-hgs] [-t target-session] [variable]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct ShowEnvironment {
    /// `-h`: list the hidden entries instead of the visible ones.
    hidden_only: bool,
    /// `-g`: read the server environment rather than a session's.
    global: bool,
    /// `-s`: render shell commands instead of the environment-file form.
    shell: bool,
    /// `-t`: the session whose environment is read.
    target: Option<String>,
    variable: Option<String>,
}

impl ShowEnvironment {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            hidden_only: args.has('h'),
            global: args.has('g'),
            shell: args.has('s'),
            target: args.value('t').map(str::to_string),
            variable: args.positionals().first().cloned(),
        })
    }

    fn run(self, st: &ServerState) -> CommandResult {
        let hidden_only = self.hidden_only;
        let shell = self.shell;
        let requested = self.variable.as_deref();
        if !self.global {
            let target = self.target.clone().or_else(|| current_target(st));
            let Some(target) = target else {
                return CommandResult::err("no current session\n");
            };
            let (environment, removed, hidden) = match st.session_env(&target) {
                Ok(environment) => environment,
                Err(_) => {
                    return match self.target.as_deref() {
                        Some(target) => CommandResult::err(format!("no such session: {target}\n")),
                        None => CommandResult::err("no current session\n"),
                    }
                }
            };
            return match requested {
                Some(name) if hidden.contains(name) != hidden_only => CommandResult::ok(""),
                Some(name) if removed.contains(name) => show_environment_value(name, None, shell),
                Some(name) => match environment.get(name) {
                    Some(value) => show_environment_value(name, Some(value), shell),
                    None => CommandResult::err(format!("unknown variable: {name}\n")),
                },
                None => {
                    let mut entries = environment
                        .iter()
                        .filter(|(name, _)| hidden.contains(*name) == hidden_only)
                        .map(|(name, value)| (name.as_str(), Some(value.as_str())))
                        .collect::<Vec<_>>();
                    if !hidden_only {
                        entries.extend(removed.iter().map(|name| (name.as_str(), None)));
                    }
                    entries.sort_by_key(|(name, _)| *name);
                    show_environment_entries(entries, shell)
                }
            };
        }
        match requested {
            Some(name) if st.env_is_hidden(name) != hidden_only => CommandResult::ok(""),
            Some(name) if st.env_is_removed(name) => show_environment_value(name, None, shell),
            Some(name) => match st.get_env(name) {
                Some(value) => show_environment_value(name, Some(value), shell),
                None => CommandResult::err(format!("unknown variable: {name}\n")),
            },
            None => {
                let mut entries = st
                    .env_iter()
                    .filter(|(name, _)| st.env_is_hidden(name) == hidden_only)
                    .map(|(name, value)| (name.as_str(), Some(value.as_str())))
                    .collect::<Vec<_>>();
                if !hidden_only {
                    entries.extend(st.removed_env_iter().map(|name| (name.as_str(), None)));
                }
                entries.sort_by_key(|(name, _)| *name);
                show_environment_entries(entries, shell)
            }
        }
    }
}

fn show_environment_entries(entries: Vec<(&str, Option<&str>)>, shell: bool) -> CommandResult {
    let mut output = String::new();
    for (name, value) in entries {
        output.push_str(&show_environment_line(name, value, shell));
        output.push('\n');
    }
    CommandResult::ok(output)
}

fn show_environment_value(name: &str, value: Option<&str>, shell: bool) -> CommandResult {
    CommandResult::ok(format!("{}\n", show_environment_line(name, value, shell)))
}

fn show_environment_line(name: &str, value: Option<&str>, shell: bool) -> String {
    if !shell {
        return value
            .map(|value| format!("{name}={value}"))
            .unwrap_or_else(|| format!("-{name}"));
    }
    match value {
        Some(value) => {
            let mut escaped = String::with_capacity(value.len());
            for character in value.chars() {
                if matches!(character, '$' | '`' | '"' | '\\') {
                    escaped.push('\\');
                }
                escaped.push(character);
            }
            format!("{name}=\"{escaped}\"; export {name};")
        }
        None => format!("unset {name};"),
    }
}
