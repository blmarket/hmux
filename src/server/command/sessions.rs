//! The session commands.

use super::*;

#[derive(Clone, Debug)]
pub(in crate::server) enum Command {
    Attach(AttachSession),
    Has(HasSession),
    New(NewSession),
    Kill(KillSession),
    Rename(RenameSession),
    List(ListSessions),
}

impl Command {
    pub(super) fn execute(self, context: &mut CommandContext<'_>) -> CommandResult {
        match self {
            Self::List(command) => command.execute(context.state, context.agents),
            Self::Has(command) => command.execute(context.state),
            Self::New(command) => command.execute(context.state, context.client),
            Self::Kill(command) => command.execute(context.state),
            Self::Rename(command) => command.execute(context.state),
            Self::Attach(command) => command.execute(context.state),
        }
    }
}

/// `list-sessions [-r] [-F format] [-f filter] [-O order]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct ListSessions {
    /// `-F`: the line each session expands to.
    format: Option<String>,
    /// `-f`: only list sessions this format is true for.
    filter: Option<String>,
    /// `-O`: the sort key, resolved when the command runs.
    order: Option<String>,
    /// `-r`: reverse the sort.
    reversed: bool,
}

impl ListSessions {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            format: args.value('F').map(str::to_string),
            filter: args.value('f').map(str::to_string),
            order: args.value('O').map(str::to_string),
            reversed: args.has('r'),
        })
    }

    /// Sessions are listed sorted by name (tmux keys its session tree by name),
    /// one per line; `-F` overrides the default summary.
    fn execute(self, st: &ServerState, agents: &PaneAgents) -> CommandResult {
        let sort_order = match list_sort_order(self.order.as_deref()) {
            Ok(order) => order,
            Err(error) => return error,
        };
        let mut order: Vec<&Session> = st.sessions().iter().collect();
        order.sort_by(|a, b| a.name.cmp(&b.name));
        apply_list_sort(
            &mut order,
            sort_order,
            self.reversed,
            |key, a, b| match key {
                ListSortOrder::Index => a.id.cmp(&b.id),
                ListSortOrder::Creation => a.created_epoch.cmp(&b.created_epoch),
                // Most recent activity first, as tmux inverts this comparison.
                ListSortOrder::Activity => b.activity_micros.cmp(&a.activity_micros),
                ListSortOrder::Name => a.name.cmp(&b.name),
                _ => std::cmp::Ordering::Equal,
            },
            |session| session.name.clone(),
        );

        let marked = st.marked_pane();
        let mut out = String::new();
        for (line, session) in order.into_iter().enumerate() {
            let mut vars = vars_for(st, session, session.active, agents, marked);
            // tmux's `FORMAT_TYPE_SESSION` marker for this list context.
            vars.set("session_format", "1");
            // tmux publishes the row's own position here.
            vars.set("line", line.to_string());
            if let Some(filter) = self.filter.as_deref() {
                if !format::is_true(&expand_command_format(st, filter, &vars, None)) {
                    continue;
                }
            }
            let line = match self.format.as_deref() {
                Some(template) => expand_command_format(st, template, &vars, None),
                None => match st.session_group_name(session) {
                    Some(group) => format!("{} (group {group})", session.summary()),
                    None => session.summary(),
                },
            };
            out.push_str(&line);
            out.push('\n');
        }
        CommandResult::ok(out)
    }
}

/// `has-session [-t target-session]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct HasSession {
    /// `-t`: the session to look for.
    target: Option<String>,
}

impl HasSession {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            target: args.value('t').map(str::to_string),
        })
    }

    /// With `-t`, resolves the named session (exit 1 with "can't find session"
    /// if missing). Without `-t`, tmux resolves the *current* session, which
    /// always exists for a running server → exit 0.
    fn execute(self, st: &ServerState) -> CommandResult {
        match self.target.as_deref() {
            Some(name) if st.resolve_session(name).is_some() => CommandResult::ok(""),
            Some(_) if st.sessions().is_empty() => CommandResult::err("no current target\n"),
            Some(name) => CommandResult::err(format!("can't find session: {name}\n")),
            None => match current_session(st) {
                Some(_) => CommandResult::ok(""),
                None => CommandResult::err("no current target\n"),
            },
        }
    }
}

/// `new-session [-AdDEPX] [-c start-directory] [-e environment] [-F format]
/// [-f flags] [-n window-name] [-s session-name] [-t target-session]
/// [-x width] [-y height] [shell-command [argument ...]]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct NewSession {
    /// `-A`: attach to the named session if it already exists.
    attach_or_create: bool,
    /// `-E`: skip the `update-environment` copy-in.
    no_environment: bool,
    /// `-P`: print the new session.
    print: bool,
    /// `-c`: the session's working directory.
    cwd: Option<String>,
    /// `-e`: environment assignments for the first pane, repeatable.
    environment: Vec<String>,
    /// `-F`: what `-P` prints.
    format: Option<String>,
    /// `-n`: the name of the session's first window.
    window_name: Option<String>,
    /// `-s`: the new session's name.
    name: Option<String>,
    /// `-t`: the session to group the new one with.
    group_target: Option<String>,
    /// `-x`/`-y`: the client size, checked when the command runs.
    width: Option<String>,
    height: Option<String>,
    /// The first pane's command line.
    command: Vec<String>,
}

impl NewSession {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            attach_or_create: args.has('A'),
            no_environment: args.has('E'),
            print: args.has('P'),
            cwd: args.value('c').map(str::to_string),
            environment: args.values('e').map(str::to_string).collect(),
            format: args.value('F').map(str::to_string),
            window_name: args.value('n').map(str::to_string),
            name: args.value('s').map(str::to_string),
            group_target: args.value('t').map(str::to_string),
            width: args.value('x').map(str::to_string),
            height: args.value('y').map(str::to_string),
            command: args.positionals().to_vec(),
        })
    }

    /// Creates a detached session with a shell pane. With `-P`, prints the new
    /// session via `-F` (or the default `NEW_SESSION_TEMPLATE`) and a trailing
    /// newline, like real tmux.
    fn execute(self, st: &mut ServerState, context: &ClientContext) -> CommandResult {
        // tmux expands `-s` as a format before it looks for a session of that
        // name, and the expansion sees no session of its own — only the server.
        let name = match self.name.as_deref() {
            Some(name) => expand_command_format(st, name, &Vars::default(), None),
            None => st.next_session_name(),
        };
        // `-A` (attach-or-create): if the named session already exists, tmux
        // attaches to it instead of failing with "duplicate session". Over the
        // command path with no real tty that attach fails the same way
        // attach-session does ("open terminal failed"), which is what real tmux
        // reports here.
        if self.attach_or_create && st.find(&name).is_some() {
            return CommandResult::err("open terminal failed: not a terminal\n");
        }
        if let Err(error) = self.reject_target_with_command() {
            return CommandResult::err(error);
        }
        let dimensions = match self.dimensions() {
            Ok(dimensions) => dimensions,
            Err(error) => return CommandResult::err(error),
        };
        match self.create(&name, st, context, dimensions) {
            Ok(()) => {
                if self.print {
                    let session = st.find(&name).expect("session just created");
                    let template = self.format.as_deref().unwrap_or(NEW_SESSION_TEMPLATE);
                    let marked = st.marked_pane();
                    let line = expand_command_format(
                        st,
                        template,
                        &vars_for(st, session, session.active, &PaneAgents::new(), marked),
                        None,
                    );
                    CommandResult::ok(format!("{line}\n"))
                } else {
                    CommandResult::ok("")
                }
            }
            Err(error) => CommandResult::err(error),
        }
    }

    /// Create — or, with `-A`, find-or-create — the session an interactive
    /// `new-session` (or a bare `tmux`) should attach to, applying the same
    /// options [`NewSession::execute`] does. Returns the session name to attach
    /// to, or an already-newline-terminated error line to report to the client
    /// (e.g. `duplicate session: 0`).
    ///
    /// This is the interactive twin of the command path: it does the create but
    /// not the `-P` print (an attached client shows the session, it doesn't
    /// print it) and not the command-path `-A`/no-tty error (attaching is
    /// exactly what we go on to do).
    pub(in crate::server) fn create_for_attach(
        &self,
        st: &mut ServerState,
        context: &ClientContext,
    ) -> Result<String, String> {
        // `-A` (attach-or-create): an existing named session is attached as-is.
        if let Some(name) = self.existing_attach_target(st) {
            return Ok(name);
        }
        let requested = self
            .name
            .as_deref()
            .map(|name| expand_command_format(st, name, &Vars::default(), None));
        let name = requested.unwrap_or_else(|| st.next_session_name());
        self.reject_target_with_command()?;
        let dimensions = self.dimensions()?;
        self.create(&name, st, context, dimensions)?;
        Ok(name)
    }

    /// The session `-A` attaches to instead of creating, when the name it was
    /// given already exists. tmux hands that case straight to
    /// `cmd_attach_session`, so the caller owes the target the attach-side
    /// state work rather than the create-side option pass.
    pub(in crate::server) fn existing_attach_target(&self, st: &ServerState) -> Option<String> {
        if !self.attach_or_create {
            return None;
        }
        let name = expand_command_format(st, self.name.as_deref()?, &Vars::default(), None);
        st.find(&name).map(|_| name)
    }

    /// tmux refuses to group a session and give it a first window of its own.
    fn reject_target_with_command(&self) -> Result<(), String> {
        if self.group_target.is_some() && (self.window_name.is_some() || !self.command.is_empty()) {
            return Err("command or window name given with target\n".to_string());
        }
        Ok(())
    }

    fn create(
        &self,
        name: &str,
        st: &mut ServerState,
        context: &ClientContext,
        dimensions: (Option<u16>, Option<u16>),
    ) -> Result<(), String> {
        let spec = self.pane_spec(st, context);
        let result = match self.group_target.as_deref() {
            Some(target) => st.create_grouped_session(name, target, spec),
            None => st.create_session(name, spec),
        };
        match result {
            // create_session already yields tmux's "duplicate session: <name>".
            Err(error) => Err(format!("{error}\n")),
            Ok(_) => {
                self.apply_options(name, st, dimensions, context);
                Ok(())
            }
        }
    }

    /// Build the first pane exactly once for detached and interactive
    /// `new-session`. Both paths receive the same command tail, identity
    /// environment, terminal name, and working-directory semantics.
    pub(super) fn pane_spec(&self, st: &ServerState, context: &ClientContext) -> PaneSpec {
        let argv = pane_command_argv(&self.command, st, None);
        let environment = self
            .environment
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let argv = pane_argv(argv, context, &environment, st, SpawnSession::Pending);
        match self
            .cwd
            .clone()
            .map(PathBuf::from)
            .or_else(|| context.cwd.clone())
        {
            Some(cwd) => PaneSpec::CommandIn(argv, cwd),
            None => PaneSpec::Command(argv),
        }
    }

    /// Apply the options a created session takes from its command line: `-c`
    /// (working directory), `-x`/`-y` (client size), `-e` (environment), `-n`
    /// (first window name).
    fn apply_options(
        &self,
        name: &str,
        st: &mut ServerState,
        dimensions: (Option<u16>, Option<u16>),
        context: &ClientContext,
    ) {
        // tmux's `s->cwd`: the `-c` directory, else where the creating client was.
        // It is what the session's `#()` jobs run in.
        if let Some(session_id) = st.session_id(name) {
            let cwd = self
                .cwd
                .clone()
                .map(PathBuf::from)
                .or_else(|| context.cwd.clone());
            st.set_session_cwd(session_id, cwd);
        }
        // `-x W -y H` sets the new session's client size.
        let (x, y) = dimensions;
        let joined_existing_group = st
            .find(name)
            .is_some_and(|session| st.is_grouped(session) && st.session_group_size(session) > 1);
        if !joined_existing_group {
            if let Some(session) = st.find(name) {
                let (session_cols, session_rows) = (session.cols, session.rows);
                if x.is_some() || y.is_some() {
                    // tmux writes `-x`/`-y` onto the new session's own `default-size`,
                    // which is what shadows the global option for every window later
                    // created in it. An unspecified axis keeps the session's size.
                    let _ = st.set_session_default_size(
                        name,
                        x.unwrap_or(session_cols),
                        y.unwrap_or(session_rows),
                    );
                }
            }
        }
        // The `update-environment` copy-in, which `-E` skips. tmux reads the
        // option from the *global* session table here, since the session being
        // created has no options of its own yet.
        if !self.no_environment {
            st.update_session_environment(name, &context.environment);
        }
        // `-e VAR=value` seeds environment variables (repeatable), after the
        // copy-in so an explicit assignment wins.
        for entry in &self.environment {
            if let Some((key, value)) = entry.split_once('=') {
                let _ = st.set_session_env(name, key, value, false);
            }
        }
        // `-n name` names the session's first window. tmux expands it as a
        // format first, and a name that comes out empty leaves the automatic
        // name in place.
        let window_name = self
            .window_name
            .as_deref()
            .map(|window_name| expand_command_format(st, window_name, &Vars::default(), None))
            .filter(|window_name| !window_name.is_empty());
        if let Some(window_name) = window_name.as_deref() {
            let _ = st.name_new_window(&format!("{name}:"), window_name, true);
        } else if self.group_target.is_none() {
            apply_initial_window_name(st, name, 0, &self.command);
        }
    }

    fn dimensions(&self) -> Result<(Option<u16>, Option<u16>), String> {
        Ok((
            new_session_dimension(self.width.as_deref(), "width")?,
            new_session_dimension(self.height.as_deref(), "height")?,
        ))
    }
}

fn new_session_dimension(value: Option<&str>, label: &str) -> Result<Option<u16>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    // tmux uses "-" to request the invoking client's size. Detached command
    // clients have no usable dimensions, so preserve the existing fallback.
    if value == "-" {
        return Ok(None);
    }
    match value.parse::<i128>() {
        Ok(value) if value < 1 => Err(format!("{label} too small\n")),
        Ok(value) if value > i128::from(u16::MAX) => Err(format!("{label} too large\n")),
        Ok(value) => Ok(Some((value as u16).min(NEW_SESSION_MAX_SIZE))),
        Err(_) if numeric_with_sign(value, '-') => Err(format!("{label} too small\n")),
        Err(_)
            if numeric_with_sign(value, '+') || value.bytes().all(|byte| byte.is_ascii_digit()) =>
        {
            Err(format!("{label} too large\n"))
        }
        Err(_) => Err(format!("{label} invalid\n")),
    }
}

fn numeric_with_sign(value: &str, sign: char) -> bool {
    value.strip_prefix(sign).is_some_and(|digits| {
        !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
    })
}

/// `kill-session [-aCg] [-t target-session]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct KillSession {
    /// `-C`: clear the session's alerts instead of killing it.
    clear_alerts: bool,
    /// `-a`: kill every session but the target.
    all_but: bool,
    /// `-g`: kill the target's whole session group.
    group: bool,
    /// `-t`: the session to act on; the current one otherwise.
    target: Option<String>,
}

impl KillSession {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            clear_alerts: args.has('C'),
            all_but: args.has('a'),
            group: args.has('g'),
            target: args.value('t').map(str::to_string),
        })
    }

    fn execute(self, st: &mut ServerState) -> CommandResult {
        // Every form but the plain kill takes the session part of the target.
        let session = || {
            self.target
                .as_deref()
                .map(|target| target.split(':').next().unwrap_or(target).to_string())
                .or_else(|| current_session(st))
        };
        if self.clear_alerts {
            return match session() {
                Some(name) => match st.clear_session_alerts(&name) {
                    Ok(()) => CommandResult::ok(""),
                    Err(_) => CommandResult::err(format!("can't find session: {name}\n")),
                },
                None => CommandResult::err("no current target\n"),
            };
        }
        if self.all_but {
            return match session() {
                Some(name) => match st.kill_other_sessions(&name) {
                    Ok(()) => CommandResult::ok(""),
                    Err(error) => CommandResult::err(format!("{error}\n")),
                },
                None => CommandResult::err("no current target\n"),
            };
        }
        if self.group {
            return match session() {
                Some(name) if st.kill_session_group(&name) => CommandResult::ok(""),
                Some(name) => CommandResult::err(format!("can't find session: {name}\n")),
                None => CommandResult::err("no current target\n"),
            };
        }
        match self.target.as_deref() {
            Some(name) if st.kill_session(name) => CommandResult::ok(""),
            Some(name) => CommandResult::err(format!("can't find session: {name}\n")),
            None => match current_session(st) {
                Some(name) if st.kill_session(&name) => CommandResult::ok(""),
                _ => CommandResult::err("no current target\n"),
            },
        }
    }
}

/// `rename-session [-t target-session] new-name`.
#[derive(Clone, Debug)]
pub(in crate::server) struct RenameSession {
    /// `-t`: the session to rename; the current (newest) one otherwise.
    target: Option<String>,
    new_name: Option<String>,
}

impl RenameSession {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            target: args.value('t').map(str::to_string),
            new_name: args.positionals().first().cloned(),
        })
    }

    fn execute(self, st: &mut ServerState) -> CommandResult {
        let from = self.target.or_else(|| current_target(st));
        match (from, self.new_name) {
            (Some(from), Some(to)) => match st.rename_session(&from, &to) {
                Ok(()) => CommandResult::ok(""),
                Err(error) => CommandResult::err(format!("{error}\n")),
            },
            (None, _) => CommandResult::err("no current target\n"),
            (_, None) => {
                CommandResult::err("command rename-session: too few arguments (need at least 1)\n")
            }
        }
    }
}

/// `attach-session [-dErx] [-c working-directory] [-f flags] [-t target-session]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct AttachSession {
    /// `-t`: the session to attach to; tmux's own default is session `0`.
    target: Option<String>,
}

impl AttachSession {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            target: args.value('t').map(str::to_string),
        })
    }

    /// Over the command path there is no tty to attach, so a resolvable target
    /// still fails the way real tmux does.
    fn execute(self, st: &ServerState) -> CommandResult {
        if st.sessions().is_empty() {
            return CommandResult::err("no sessions\n");
        }
        let target = self.target.as_deref().unwrap_or("0");
        if st.find(target).is_none() {
            CommandResult::err(format!("can't find session: {target}\n"))
        } else {
            CommandResult::err("open terminal failed: not a terminal\n")
        }
    }
}
