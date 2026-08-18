//! The window commands.

use super::*;

#[derive(Clone, Debug)]
pub(in crate::server) enum Command {
    Find(FindWindow),
    New(NewWindow),
    Kill(KillWindow),
    Rename(RenameWindow),
    List(ListWindows),
    Select(SelectWindow),
    Next(NextWindow),
    Previous(PreviousWindow),
    Last(LastWindow),
    Swap(SwapWindow),
    Move(MoveWindow),
    Link(LinkWindow),
    Unlink(UnlinkWindow),
    Respawn(RespawnWindow),
}

impl Command {
    pub(super) fn execute(self, context: &mut CommandContext<'_>) -> CommandResult {
        let st = &mut *context.state;
        match self {
            Self::Find(command) => command.execute(st, context.agents),
            Self::New(command) => command.execute(st, context.client),
            Self::Kill(command) => command.execute(st),
            Self::Rename(command) => command.execute(st),
            Self::List(command) => command.execute(st, context.agents),
            Self::Select(command) => command.execute(st),
            Self::Next(command) => command.execute(st),
            Self::Previous(command) => command.execute(st),
            Self::Last(command) => command.execute(st),
            Self::Swap(command) => command.execute(st),
            Self::Move(command) => command.execute(st),
            Self::Link(command) => command.execute(st),
            Self::Unlink(command) => command.execute(st),
            Self::Respawn(command) => command.execute(st, context.client),
        }
    }
}

/// Run a state operation against a window target, defaulting to the current
/// session's active window.
fn window_command(
    target: Option<&str>,
    st: &mut ServerState,
    operation: fn(&mut ServerState, &str) -> std::io::Result<()>,
) -> CommandResult {
    let target = target
        .map(str::to_string)
        .or_else(|| current_session(st).map(|session| format!("{session}:")));
    let Some(target) = target else {
        return CommandResult::err("can't establish current session\n");
    };
    match operation(st, &target) {
        Ok(()) => CommandResult::ok(""),
        Err(error) => CommandResult::err(format!("{error}\n")),
    }
}

/// Run a state operation against the session part of a target, defaulting to
/// the current session.
fn session_command(
    target: Option<&str>,
    st: &mut ServerState,
    operation: fn(&mut ServerState, &str) -> std::io::Result<()>,
) -> CommandResult {
    let session = target
        .map(|target| target.split(':').next().unwrap_or(target).to_string())
        .or_else(|| current_session(st));
    let Some(session) = session else {
        return CommandResult::err("can't establish current session\n");
    };
    match operation(st, &session) {
        Ok(()) => CommandResult::ok(""),
        Err(error) => CommandResult::err(format!("{error}\n")),
    }
}

/// `find-window [-CiNrTZ] [-t target-pane] match-string`.
#[derive(Clone, Debug)]
pub(in crate::server) struct FindWindow {
    content: bool,
    ignore_case: bool,
    name: bool,
    regex: bool,
    title: bool,
    zoom: bool,
    /// `-t`: the pane the results are shown in.
    target: Option<String>,
    pattern: Option<String>,
}

impl FindWindow {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            content: args.has('C'),
            ignore_case: args.has('i'),
            name: args.has('N'),
            regex: args.has('r'),
            title: args.has('T'),
            zoom: args.has('Z'),
            target: args.value('t').map(str::to_string),
            pattern: args.positionals().first().cloned(),
        })
    }

    fn execute(self, state: &mut ServerState, agents: &PaneAgents) -> CommandResult {
        let Some(pattern) = self.pattern.as_deref() else {
            return CommandResult::err(
                "command find-window: too few arguments (need at least 1)\n",
            );
        };
        let mut c = self.content;
        let mut n = self.name;
        let mut t = self.title;
        if !c && !n && !t {
            c = true;
            n = true;
            t = true;
        }
        let star = if self.regex { "" } else { "*" };
        let suffix = match (self.regex, self.ignore_case) {
            (true, true) => "/ri",
            (true, false) => "/r",
            (false, true) => "/i",
            (false, false) => "",
        };
        let s = pattern;
        let filter = if c && n && t {
            format!("#{{||:#{{C{suffix}:{s}}},#{{||:#{{m{suffix}:{star}{s}{star},#{{window_name}}}},#{{m{suffix}:{star}{s}{star},#{{pane_title}}}}}}}}")
        } else if c && n {
            format!("#{{||:#{{C{suffix}:{s}}},#{{m{suffix}:{star}{s}{star},#{{window_name}}}}}}")
        } else if c && t {
            format!("#{{||:#{{C{suffix}:{s}}},#{{m{suffix}:{star}{s}{star},#{{pane_title}}}}}}")
        } else if n && t {
            format!("#{{||:#{{m{suffix}:{star}{s}{star},#{{window_name}}}},#{{m{suffix}:{star}{s}{star},#{{pane_title}}}}}}")
        } else if c {
            format!("#{{C{suffix}:{s}}}")
        } else if n {
            format!("#{{m{suffix}:{star}{s}{star},#{{window_name}}}}")
        } else {
            format!("#{{m{suffix}:{star}{s}{star},#{{pane_title}}}}")
        };

        let choose_tree = clients::ChooseTree {
            no_preview: false,
            sessions_only: false,
            windows_only: false,
            target: self.target,
            options: clients::ChooseOptions {
                reversed: false,
                zoom: self.zoom,
                format: None,
                filter: Some(filter),
                order: None,
            },
            template: None,
        };
        choose_tree.run(state, agents)
    }
}

/// `new-window [-abdkPS] [-c start-directory] [-e environment] [-F format]
/// [-n window-name] [-t target-window] [shell-command [argument ...]]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct NewWindow {
    /// `-a`/`-b`: insert after/before the anchor window rather than at an index.
    after: bool,
    before: bool,
    /// `-d`: leave the session's current window where it is.
    detached: bool,
    /// `-k`: replace an existing window at the destination index.
    kill: bool,
    /// `-P`: print the new window.
    print: bool,
    /// `-S`: select an existing window of the given name instead of creating one.
    select_existing: bool,
    /// `-c`: the new pane's working directory.
    cwd: Option<String>,
    /// `-e`: environment assignments for the new pane, repeatable.
    environment: Vec<String>,
    /// `-F`: what `-P` prints.
    format: Option<String>,
    /// `-n`: the new window's name.
    name: Option<String>,
    /// `-t`: where the window goes.
    target: Option<String>,
    /// The new pane's command line.
    command: Vec<String>,
}

impl NewWindow {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            after: args.has('a'),
            before: args.has('b'),
            detached: args.has('d'),
            kill: args.has('k'),
            print: args.has('P'),
            select_existing: args.has('S'),
            cwd: args.value('c').map(str::to_string),
            environment: args.values('e').map(str::to_string).collect(),
            format: args.value('F').map(str::to_string),
            name: args.value('n').map(str::to_string),
            target: args.value('t').map(str::to_string),
            command: args.positionals().to_vec(),
        })
    }

    /// Opens a window in the target's session (or the current session). With
    /// `-P`, prints the new window via `-F` (or `NEW_WINDOW_TEMPLATE`).
    fn execute(self, st: &mut ServerState, context: &ClientContext) -> CommandResult {
        let requested_target = self.target.as_deref();
        let (session, explicit) = match parse_window_target(requested_target, st) {
            Some(target) => target,
            None => return CommandResult::err("can't establish current session\n"),
        };
        // tmux expands `-n` as a format before it does anything else with it,
        // and a name that expands to nothing names nothing at all.
        let requested_name = self
            .name
            .as_deref()
            .map(|name| {
                execution::expand_target_format(name, requested_target, st, &PaneAgents::new(), &[])
            })
            .filter(|name| !name.is_empty());
        // `-S` with a name and no explicit window index selects an existing window
        // of that name instead of creating one. tmux errors on an ambiguous name,
        // skips the selection under `-d`, and returns before the `-P` print either
        // way.
        if self.select_existing && explicit.is_none() {
            if let Some(name) = requested_name.as_deref() {
                let sess = st.find(&session).expect("session present");
                let matches: Vec<u32> = sess
                    .windows
                    .iter()
                    .filter(|link| st.window_for_link(link).name == name)
                    .map(|link| link.index)
                    .collect();
                if matches.len() > 1 {
                    return CommandResult::err(format!("multiple windows named {name}\n"));
                }
                if let Some(index) = matches.first() {
                    if !self.detached {
                        if let Err(error) = st.select_window(&format!("{session}:{index}")) {
                            return CommandResult::err(format!("{error}\n"));
                        }
                    }
                    return CommandResult::ok("");
                }
            }
        }
        // `-a` (after) / `-b` (before) insert *relative* to an anchor window rather
        // than at an explicit index. `-b` takes precedence over `-a` (tmux keys the
        // shuffle on whether `-b` is present). The anchor is the `-t` window part when
        // it names an existing window, else the session's active window. When `-t`
        // names an index that does *not* exist, tmux ignores the relative flag and
        // creates at that index — the same explicit path as without `-a`/`-b`.
        let relative = self.after || self.before;
        // `-d` creates the window in the background: it is not selected, so the
        // session's current window stays put (tmux's default is to follow).
        let select = !self.detached;
        let explicit_cwd = self.cwd.as_deref().map(|cwd| {
            execution::spawn_start_directory(cwd, Some(&session), st, context, &PaneAgents::new())
        });
        let cwd = explicit_cwd.as_deref().or(context.cwd.as_deref());
        let argv = pane_command_argv(&self.command, st, Some(&session));
        let environment = self
            .environment
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let argv = pane_argv(
            argv,
            context,
            &environment,
            st,
            SpawnSession::Existing(&session),
        );
        // tmux names the window inside `spawn_window`, before it raises
        // `window-linked`, so the name is settled here rather than after the
        // window exists. `-n` is that name; otherwise the command decides it.
        let initial_name = match requested_name.as_deref() {
            Some(name) => name.to_string(),
            None => initial_window_name(st, &session, &self.command),
        };
        let result = if relative {
            match anchor_window_index(&session, explicit, st) {
                Some(anchor) => st.new_window_relative_with_spawn(
                    &session,
                    anchor,
                    !self.before,
                    select,
                    &argv,
                    cwd,
                    &initial_name,
                ),
                None if self.kill => st.new_window_replacing_with_spawn(
                    &session,
                    explicit,
                    select,
                    &argv,
                    cwd,
                    &initial_name,
                ),
                None => {
                    st.new_window_with_spawn(&session, explicit, select, &argv, cwd, &initial_name)
                }
            }
        } else if self.kill {
            st.new_window_replacing_with_spawn(
                &session,
                explicit,
                select,
                &argv,
                cwd,
                &initial_name,
            )
        } else {
            st.new_window_with_spawn(&session, explicit, select, &argv, cwd, &initial_name)
        };
        match result {
            Ok(win_idx) => {
                // `-n name` sets the new window's name (otherwise it stays the
                // model's default, empty).
                match requested_name.as_deref() {
                    Some(name) => {
                        let index =
                            st.find(&session).expect("session present").windows[win_idx].index;
                        let _ = st.rename_window(&format!("{session}:{index}"), name);
                    }
                    None => apply_initial_window_name(st, &session, win_idx, &self.command),
                }
                // tmux hands `after-new-window` the find-state of the window it
                // just created, not the `-t` the command was given.
                let created = st.find(&session).expect("session present").windows[win_idx].id;
                let mut result = if self.print {
                    let sess = st.find(&session).expect("session present");
                    let template = self.format.as_deref().unwrap_or(NEW_WINDOW_TEMPLATE);
                    let marked = st.marked_pane();
                    let line = expand_command_format(
                        st,
                        template,
                        &vars_for(st, sess, win_idx, &PaneAgents::new(), marked),
                        None,
                    );
                    CommandResult::ok(format!("{line}\n"))
                } else {
                    CommandResult::ok("")
                };
                result.after_hook_target = Some(format!("@{created}"));
                result
            }
            Err(error) => match requested_target {
                Some(target) => command_target_error(error, target, "window"),
                None => CommandResult::err(format!("{error}\n")),
            },
        }
    }
}

/// `kill-window [-a] [-t target-window]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct KillWindow {
    /// `-a`: kill every window but the target.
    all_but: bool,
    /// `-t`: the window to kill.
    target: Option<String>,
}

impl KillWindow {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            all_but: args.has('a'),
            target: args.value('t').map(str::to_string),
        })
    }

    fn execute(self, st: &mut ServerState) -> CommandResult {
        let operation = if self.all_but {
            ServerState::kill_other_windows
        } else {
            ServerState::kill_window
        };
        window_command(self.target.as_deref(), st, operation)
    }
}

/// `rename-window [-t target-window] new-name`.
#[derive(Clone, Debug)]
pub(in crate::server) struct RenameWindow {
    /// `-t`: the window to rename; the current session's active one otherwise.
    target: Option<String>,
    new_name: Option<String>,
}

impl RenameWindow {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            target: args.value('t').map(str::to_string),
            new_name: args.positionals().first().cloned(),
        })
    }

    fn execute(self, st: &mut ServerState) -> CommandResult {
        let target = self
            .target
            .or_else(|| current_session(st).map(|session| format!("{session}:")));
        match (target, self.new_name) {
            (Some(target), Some(name)) => {
                let resolved = match st.resolve_window_target(&target) {
                    Ok(resolved) => resolved,
                    Err(error) => return CommandResult::err(format!("{error}\n")),
                };
                let session = &st.sessions()[resolved.session];
                let expanded = expand_command_format(
                    st,
                    &name,
                    &vars_full(
                        st,
                        session,
                        resolved.window,
                        resolved.pane,
                        &PaneAgents::new(),
                        st.marked_pane(),
                    ),
                    None,
                );
                match st.rename_window(&target, &expanded) {
                    Ok(()) => CommandResult::ok(""),
                    Err(error) => CommandResult::err(format!("{error}\n")),
                }
            }
            (None, _) => CommandResult::err("no current target\n"),
            (_, None) => {
                CommandResult::err("command rename-window: too few arguments (need at least 1)\n")
            }
        }
    }
}

/// `list-windows [-ar] [-F format] [-f filter] [-O order] [-t target-session]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct ListWindows {
    /// `-a`: list every window of every session.
    all: bool,
    /// `-F`: the line each window expands to.
    format: Option<String>,
    /// `-f`: only list windows this format is true for.
    filter: Option<String>,
    /// `-O`: the sort key, resolved when the command runs.
    order: Option<String>,
    /// `-r`: reverse the sort.
    reversed: bool,
    /// `-t`: the session whose windows are listed.
    target: Option<String>,
}

impl ListWindows {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            all: args.has('a'),
            format: args.value('F').map(str::to_string),
            filter: args.value('f').map(str::to_string),
            order: args.value('O').map(str::to_string),
            reversed: args.has('r'),
            target: args.value('t').map(str::to_string),
        })
    }

    /// Lists a session's windows in index order. `-F` overrides the (structural)
    /// default line.
    fn execute(self, st: &ServerState, agents: &PaneAgents) -> CommandResult {
        let default_line = "#{window_index}: #{window_name} (#{window_panes} panes)";

        // `-a` lists every window across all sessions (sorted by session name), like
        // tmux; otherwise just the target session's windows.
        let sessions: Vec<&Session> = if self.all {
            let mut all: Vec<&Session> = st.sessions().iter().collect();
            all.sort_by(|a, b| a.name.cmp(&b.name));
            all
        } else {
            let session = self
                .target
                .as_deref()
                .and_then(|target| target.split(':').next())
                .filter(|session| !session.is_empty())
                .map(str::to_string)
                .or_else(|| current_session(st));
            let Some(session) = session else {
                return CommandResult::err("no current target\n");
            };
            match st.resolve_session(&session) {
                Some(session) => vec![session],
                None => return CommandResult::err(format!("can't find session: {session}\n")),
            }
        };

        let sort_order = match list_sort_order(self.order.as_deref()) {
            Ok(order) => order,
            Err(error) => return error,
        };
        let mut links: Vec<(&Session, usize)> = sessions
            .iter()
            .flat_map(|sess| (0..sess.windows.len()).map(move |idx| (*sess, idx)))
            .collect();
        apply_list_sort(
            &mut links,
            sort_order,
            self.reversed,
            |key, (sess_a, idx_a), (sess_b, idx_b)| {
                let (link_a, link_b) = (&sess_a.windows[*idx_a], &sess_b.windows[*idx_b]);
                let (win_a, win_b) = (st.window_for_link(link_a), st.window_for_link(link_b));
                match key {
                    ListSortOrder::Index => link_a.index.cmp(&link_b.index),
                    // `activity_epoch` is written once at creation (see `Window`),
                    // so it is the creation key; tmux's activity sort is inverted.
                    ListSortOrder::Creation => win_a.activity_micros.cmp(&win_b.activity_micros),
                    ListSortOrder::Activity => win_b.activity_micros.cmp(&win_a.activity_micros),
                    ListSortOrder::Name => win_a.name.cmp(&win_b.name),
                    ListSortOrder::Size => (u32::from(win_a.cols) * u32::from(win_a.rows))
                        .cmp(&(u32::from(win_b.cols) * u32::from(win_b.rows))),
                    _ => std::cmp::Ordering::Equal,
                }
            },
            |(sess, idx)| st.window_for_link(&sess.windows[*idx]).name.clone(),
        );

        let marked = st.marked_pane();
        let mut out = String::new();
        // Where `list-sessions` publishes each row's position, `cmd_list_windows`
        // hands `format_add` the row *count* instead, the same on every row.
        let rows = links.len();
        for (sess, idx) in links {
            let mut vars = vars_for(st, sess, idx, agents, marked);
            // tmux's `FORMAT_TYPE_WINDOW` marker for this list context.
            vars.set("window_format", "1");
            vars.set("line", rows.to_string());
            if let Some(filter) = self.filter.as_deref() {
                if !format::is_true(&expand_command_format(st, filter, &vars, None)) {
                    continue;
                }
            }
            // Structural default (tmux's includes a volatile layout id + @id, so
            // this isn't byte-identical; the suite pins `list-windows` via -F).
            let line = expand_command_format(
                st,
                self.format.as_deref().unwrap_or(default_line),
                &vars,
                None,
            );
            out.push_str(&line);
            out.push('\n');
        }
        CommandResult::ok(out)
    }
}

/// `select-window [-lnpT] [-t target-window]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct SelectWindow {
    /// `-l`/`-n`/`-p`: select the last, next, or previous window instead of the
    /// target.
    last: bool,
    next: bool,
    previous: bool,
    /// `-T`: toggle to the last window when the target is already current.
    toggle: bool,
    /// `-t`: the window to select.
    target: Option<String>,
}

impl SelectWindow {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            last: args.has('l'),
            next: args.has('n'),
            previous: args.has('p'),
            toggle: args.has('T'),
            target: args.value('t').map(str::to_string),
        })
    }

    fn execute(self, st: &mut ServerState) -> CommandResult {
        let target = self.target.as_deref();
        if self.next {
            return session_command(target, st, ServerState::next_window);
        }
        if self.previous {
            return session_command(target, st, ServerState::previous_window);
        }
        if self.last {
            return session_command(target, st, ServerState::last_window);
        }
        if self.toggle {
            let target = target.map(str::to_string).or_else(|| current_target(st));
            let Some(target) = target else {
                return CommandResult::err("can't establish current session\n");
            };
            let is_current = st.resolve(&target).is_some_and(|resolved| {
                let session = &st.sessions()[resolved.session];
                session.active == resolved.window
            });
            if is_current {
                let session = target.split(':').next().unwrap_or(&target);
                return match st.last_window(session) {
                    Ok(()) => CommandResult::ok(""),
                    Err(error) => CommandResult::err(format!("{error}\n")),
                };
            }
            return match st.select_window(&target) {
                Ok(()) => CommandResult::ok(""),
                Err(error) => CommandResult::err(format!("{error}\n")),
            };
        }
        window_command(target, st, ServerState::select_window)
    }
}

/// `next-window [-a] [-t target-session]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct NextWindow {
    /// `-a`: move to the next window with an alert.
    alert: bool,
    /// `-t`: the session to move within.
    target: Option<String>,
}

impl NextWindow {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            alert: args.has('a'),
            target: args.value('t').map(str::to_string),
        })
    }

    fn execute(self, st: &mut ServerState) -> CommandResult {
        let operation = if self.alert {
            ServerState::next_window_alert
        } else {
            ServerState::next_window
        };
        session_command(self.target.as_deref(), st, operation)
    }
}

/// `previous-window [-a] [-t target-session]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct PreviousWindow {
    /// `-a`: move to the previous window with an alert.
    alert: bool,
    /// `-t`: the session to move within.
    target: Option<String>,
}

impl PreviousWindow {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            alert: args.has('a'),
            target: args.value('t').map(str::to_string),
        })
    }

    fn execute(self, st: &mut ServerState) -> CommandResult {
        let operation = if self.alert {
            ServerState::previous_window_alert
        } else {
            ServerState::previous_window
        };
        session_command(self.target.as_deref(), st, operation)
    }
}

/// `last-window [-t target-session]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct LastWindow {
    /// `-t`: the session to move within.
    target: Option<String>,
}

impl LastWindow {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            target: args.value('t').map(str::to_string),
        })
    }

    fn execute(self, st: &mut ServerState) -> CommandResult {
        session_command(self.target.as_deref(), st, ServerState::last_window)
    }
}

/// `swap-window [-d] [-s src-window] [-t dst-window]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct SwapWindow {
    /// `-d`: select the swapped target. tmux inverts the usual `-d` sense here:
    /// the plain form leaves the current window where it is.
    select: bool,
    /// `-s`/`-t`: the windows to exchange.
    source: Option<String>,
    target: Option<String>,
}

impl SwapWindow {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            select: args.has('d'),
            source: args.value('s').map(str::to_string),
            target: args.value('t').map(str::to_string),
        })
    }

    /// Exchanges two windows' contents (keeping their indices). Missing
    /// `-s`/`-t` default to the current session's active window.
    fn execute(self, st: &mut ServerState) -> CommandResult {
        let source = self
            .source
            .or_else(|| {
                st.marked_pane().and_then(|id| {
                    let target = st.resolve(&format!("%{id}"))?;
                    Some(format!(
                        "@{}",
                        st.sessions()[target.session].windows[target.window].id
                    ))
                })
            })
            .or_else(|| current_target(st));
        let target = self.target.or_else(|| current_target(st));
        match (source, target) {
            (Some(source), Some(target)) => match st.swap_window(&source, &target, self.select) {
                Ok(()) => CommandResult::ok(""),
                Err(error) => command_target_error_candidates(
                    error,
                    &[(&source, "window"), (&target, "window")],
                ),
            },
            _ => CommandResult::err("can't establish current session\n"),
        }
    }
}

/// `move-window [-abdkr] [-s src-window] [-t dst-window]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct MoveWindow {
    /// `-a`/`-b`: move after/before the destination index rather than onto it.
    after: bool,
    before: bool,
    /// `-d`: leave the current window where it is.
    detached: bool,
    /// `-k`: replace an existing window at the destination index.
    kill: bool,
    /// `-r`: renumber the target session's windows instead of moving one.
    renumber: bool,
    /// `-s`/`-t`: the window to move, and where it goes.
    source: Option<String>,
    target: Option<String>,
}

impl MoveWindow {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            after: args.has('a'),
            before: args.has('b'),
            detached: args.has('d'),
            kill: args.has('k'),
            renumber: args.has('r'),
            source: args.value('s').map(str::to_string),
            target: args.value('t').map(str::to_string),
        })
    }

    /// Renumbers a window to the destination index. With `-r`, renumbers *all*
    /// windows of the target session to close gaps (ignoring `-s`), matching
    /// tmux.
    fn execute(self, st: &mut ServerState) -> CommandResult {
        if self.renumber {
            let session = self
                .target
                .as_deref()
                .map(|target| target.split(':').next().unwrap_or(target).to_string())
                .or_else(|| current_session(st));
            let Some(session) = session else {
                return CommandResult::err("can't establish current session\n");
            };
            return match st.renumber_windows(&session) {
                Ok(()) => CommandResult::ok(""),
                Err(error) => CommandResult::err(format!("{error}\n")),
            };
        }
        let source = self.source.or_else(|| current_target(st));
        // tmux selects the moved window by default; `-d` leaves the current window
        // where it is.
        let select = !self.detached;
        // `-a` (after) / `-b` (before) relocate the window *relative* to the `-t`
        // anchor index rather than onto it. `-b` takes precedence over `-a`.
        let relative = self.after || self.before;
        let target = self.target.or_else(|| {
            if !relative {
                return None;
            }
            let session = current_session(st)?;
            let window = st
                .sessions()
                .iter()
                .find(|candidate| candidate.name == session)
                .and_then(|candidate| candidate.windows.get(candidate.active))?;
            Some(format!("{session}:{}", window.index))
        });
        match (source, target) {
            (Some(source), Some(target)) => match if relative {
                st.move_window_relative(&source, &target, !self.before, select)
            } else if self.kill {
                st.move_window_replacing(&source, &target, select)
            } else {
                st.move_window(&source, &target, select)
            } {
                Ok(()) => CommandResult::ok(""),
                Err(error) => command_target_error_candidates(
                    error,
                    &[(&source, "window"), (&target, "window")],
                ),
            },
            (None, _) => CommandResult::err("can't establish current session\n"),
            (_, None) => CommandResult::err("move-window: missing destination\n"),
        }
    }
}

/// `link-window [-abdk] [-s src-window] [-t dst-window]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct LinkWindow {
    /// `-a`/`-b`: link after/before the destination index rather than onto it.
    after: bool,
    before: bool,
    /// `-d`: do not follow the linked window.
    detached: bool,
    /// `-k`: replace an existing window at the destination index.
    kill: bool,
    /// `-s`/`-t`: the window to link, and where it goes.
    source: Option<String>,
    target: Option<String>,
}

impl LinkWindow {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            after: args.has('a'),
            before: args.has('b'),
            detached: args.has('d'),
            kill: args.has('k'),
            source: args.value('s').map(str::to_string),
            target: args.value('t').map(str::to_string),
        })
    }

    /// Adds the source window to the destination session at the destination
    /// index. By default the linked window becomes the destination session's
    /// current window; `-d` suppresses the follow.
    fn execute(self, st: &mut ServerState) -> CommandResult {
        let source = self.source.or_else(|| current_target(st));
        let select = !self.detached;
        // `-a` (after) / `-b` (before) link the window *relative* to the `-t` anchor
        // index rather than onto it. `-b` takes precedence over `-a`.
        let relative = self.after || self.before;
        match (source, self.target) {
            (Some(source), Some(target)) => match if relative {
                st.link_window_relative(&source, &target, !self.before, select)
            } else {
                st.link_window(&source, &target, self.kill, select)
            } {
                Ok(()) => CommandResult::ok(""),
                Err(error) => command_target_error_candidates(
                    error,
                    &[(&source, "window"), (&target, "window")],
                ),
            },
            (None, _) => CommandResult::err("can't establish current session\n"),
            (_, None) => CommandResult::err("link-window: missing destination\n"),
        }
    }
}

/// `unlink-window [-k] [-t target-window]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct UnlinkWindow {
    /// `-k`: kill the window when it was linked only here.
    kill: bool,
    /// `-t`: the window to unlink.
    target: Option<String>,
}

impl UnlinkWindow {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            kill: args.has('k'),
            target: args.value('t').map(str::to_string),
        })
    }

    fn execute(self, st: &mut ServerState) -> CommandResult {
        let target = self
            .target
            .or_else(|| current_session(st).map(|session| format!("{session}:")));
        let Some(target) = target else {
            return CommandResult::err("can't establish current session\n");
        };
        match st.unlink_window(&target, self.kill) {
            Ok(()) => CommandResult::ok(""),
            Err(error) => CommandResult::err(format!("{error}\n")),
        }
    }
}

/// `respawn-window [-k] [-c start-directory] [-e environment] [-t target-window]
/// [shell-command [argument ...]]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct RespawnWindow {
    /// `-k`: kill the window's live panes first.
    kill: bool,
    /// `-c`: the replacement panes' working directory.
    cwd: Option<String>,
    /// `-e`: environment assignments for the replacement, repeatable.
    environment: Vec<String>,
    /// `-t`: the window to respawn.
    target: Option<String>,
    /// The replacement command line.
    command: Vec<String>,
}

impl RespawnWindow {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            kill: args.has('k'),
            cwd: args.value('c').map(str::to_string),
            environment: args.values('e').map(str::to_string).collect(),
            target: args.value('t').map(str::to_string),
            command: args.positionals().to_vec(),
        })
    }

    /// Refuses to respawn a window with a live pane unless `-k` (kill first) is
    /// given, failing with `respawn window failed: window <t> still active`
    /// (exit 1). The target must resolve (else tmux's `can't find window`), and
    /// a window is "still active" while any of its panes has not exited. With
    /// `-k`, or an all-exited window, the command succeeds silently — the actual
    /// re-exec of the panes' commands is interactive byte work outside this
    /// control-plane layer.
    fn execute(self, st: &mut ServerState, context: &ClientContext) -> CommandResult {
        let target = self.target.clone().or_else(|| current_target(st));
        let Some(target) = target else {
            return CommandResult::err("can't establish current session\n");
        };
        let resolved = match st.resolve_window_target(&target) {
            Ok(resolved) => resolved,
            Err(error) => return CommandResult::err(format!("{error}\n")),
        };
        let sess = &st.sessions()[resolved.session];
        let link = &sess.windows[resolved.window];
        let win = st.window_for_link(link);
        // `-k` bypasses the guard; a window whose panes have all exited may respawn.
        let live = win.panes.iter().any(|node| !node.pane.has_exited());
        if !self.kill && live {
            return CommandResult::err(format!(
                "respawn window failed: window {}:{} still active\n",
                sess.name, link.index,
            ));
        }
        let argv =
            (!self.command.is_empty()).then(|| pane_command_argv(&self.command, st, Some(&target)));
        // `-e` reaches the replacement the way a spawn's environment does.
        let environment = self
            .environment
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let mut cwd_from_spec = None;
        let argv = if environment.is_empty() {
            argv
        } else {
            // With no replacement command, tmux's `spawn_pane` reuses the
            // pane's saved argv and still copies `-e` into the child, so the
            // saved command is materialized here to carry the wrap.
            let base = argv.or_else(|| {
                let spec = st
                    .window(resolved.session, resolved.window)
                    .panes
                    .first()?
                    .pane
                    .spawn_spec()?;
                cwd_from_spec = spec.cwd;
                Some(unwrap_pane_argv(spec.argv))
            });
            base.map(|argv| {
                pane_argv(
                    argv,
                    context,
                    &environment,
                    st,
                    SpawnSession::Existing(&target),
                )
            })
        };
        let cwd = self
            .cwd
            .as_deref()
            .map(|cwd| {
                execution::spawn_start_directory(
                    cwd,
                    Some(&target),
                    st,
                    context,
                    &PaneAgents::new(),
                )
            })
            .or(cwd_from_spec);
        match st.respawn_window_process(&target, argv, cwd) {
            Ok(()) => CommandResult::ok(""),
            Err(error) => CommandResult::err(format!("{error}\n")),
        }
    }
}
