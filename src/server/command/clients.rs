//! The client commands: what a client is shown, asked, and moved to.

use super::*;
use crate::server::state::ModeTarget;

#[derive(Clone, Debug)]
pub(in crate::server) enum Command {
    List(ListClients),
    Detach(DetachClient),
    Switch(SwitchClient),
    Refresh(RefreshClient),
    Suspend(SuspendClient),
    Lock(LockClient),
    Prompt(CommandPrompt),
    ConfirmBefore(ConfirmBefore),
    DisplayMessage(DisplayMessage),
    DisplayMenu(DisplayMenu),
    DisplayPopup(DisplayPopup),
    DisplayPanes(DisplayPanes),
    ChooseTree(ChooseTree),
    ChooseClient(ChooseClient),
    ChooseBuffer(ChooseBuffer),
    ClockMode(ClockMode),
    CustomizeMode(CustomizeMode),
    ShowPromptHistory(ShowPromptHistory),
    ClearPromptHistory(ClearPromptHistory),
}

impl Command {
    pub(super) async fn execute(self, context: &mut ExecContext<'_>) -> SharedCommandExecution {
        match self {
            // These four put something on a client's screen and wait for the
            // answer, so the queue behind them is parked until it comes.
            Self::Prompt(command) => command.execute(context).await,
            Self::ConfirmBefore(command) => {
                let waits = !command.background;
                interactive(context, waits, |inner| {
                    command.run(inner.state, inner.client)
                })
                .await
            }
            Self::DisplayMenu(command) => {
                interactive(context, true, |inner| {
                    command.run(inner.state, inner.agents, inner.client)
                })
                .await
            }
            Self::DisplayPopup(command) => {
                let waits = !command.close;
                interactive(context, waits, |inner| {
                    command.run(inner.state, inner.client)
                })
                .await
            }
            Self::DisplayPanes(command) => {
                let waits = !command.background;
                interactive(context, waits, |inner| {
                    command.run(inner.state, inner.client)
                })
                .await
            }
            Self::List(command) => context.sync(|inner| command.run(inner.state, inner.agents)),
            Self::Detach(command) => context.sync(|inner| command.run(inner.state, inner.client)),
            Self::Refresh(command) => context.sync(|inner| command.run(inner.state, inner.client)),
            Self::Switch(command) => context.sync(|inner| command.run(inner.state, inner.client)),
            Self::Suspend(command) => context.sync(|inner| command.run(inner.state, inner.client)),
            Self::Lock(command) => context.sync(|inner| command.run(inner.state, inner.client)),
            Self::DisplayMessage(command) => {
                context.sync(|inner| command.run(inner.state, inner.agents, inner.client))
            }
            Self::ChooseTree(command) => {
                context.sync(|inner| command.run(inner.state, inner.agents))
            }
            Self::ChooseClient(command) => {
                context.sync(|inner| command.run(inner.state, inner.agents))
            }
            Self::ChooseBuffer(command) => context.sync(|inner| command.run(inner.state)),
            Self::ClockMode(command) => context.sync(|inner| command.run(inner.state)),
            Self::CustomizeMode(command) => context.sync(|inner| command.run(inner.state)),
            Self::ShowPromptHistory(command) => context.sync(|inner| command.run(inner.state)),
            Self::ClearPromptHistory(command) => context.sync(|inner| command.run(inner.state)),
        }
    }
}

/// Run one of the commands that puts an overlay on a client, and — when the
/// client is one that answers — wait for what it answered.
///
/// tmux runs the command first either way: it is what puts the overlay up, and
/// a failure there is reported instead of waited on.
async fn interactive(
    context: &mut ExecContext<'_>,
    waits: bool,
    run: impl FnOnce(&mut CommandContext<'_>) -> CommandResult,
) -> SharedCommandExecution {
    if !waits || !context.client().wait_for_interactions() {
        return context.sync(run);
    }
    let (reply, completed) = match PromptReply::new() {
        Ok(pair) => pair,
        Err(error) => {
            return SharedCommandExecution::completed(CommandResult::err(format!("{error}\n")))
        }
    };
    let mut client = context.client().clone();
    client.interaction_reply = Some(reply);
    let initial = context.run_sync(&client, run);
    if initial.exit != 0 {
        return SharedCommandExecution::completed(initial);
    }
    let start = SuspensionStart::Waiting(SuspensionWait::Interaction(completed));
    SharedCommandExecution::completed(context.wait_for_answer(true, start).await)
}

/// `command-prompt [-1CbeFiklN] [-I inputs] [-p prompts] [-t target-client]
/// [-T prompt-type] [template]`.
///
/// The prompt's own words travel to the client that answers it and come back to
/// build the template, so this command carries its argv rather than a reading
/// of it.
#[derive(Clone, Debug)]
pub(in crate::server) struct CommandPrompt {
    args: Vec<String>,
}

impl CommandPrompt {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            args: args.argv().to_vec(),
        })
    }

    async fn execute(self, context: &mut ExecContext<'_>) -> SharedCommandExecution {
        if !context.client().wait_for_interactions() {
            return SharedCommandExecution::completed(CommandResult::err("no current client\n"));
        }
        // tmux finds `-t`'s client while it prepares the queue item, so a
        // missing client is reported before the prompt's own arguments are.
        let prompt_target = command_prompt_target(&self.args);
        {
            let state = context.state().borrow();
            match prompt_target.as_deref() {
                Some(name) if !client_named(&state, name) => {
                    return SharedCommandExecution::completed(CommandResult::err(format!(
                        "can't find client: {name}\n"
                    )))
                }
                None if state.client_snapshots().is_empty() => {
                    return SharedCommandExecution::completed(CommandResult::err(
                        "no current client\n",
                    ))
                }
                _ => {}
            }
        }
        if let Err(error) = command_prompt_spec(&self.args) {
            return SharedCommandExecution::completed(CommandResult::err(error));
        }
        let registry = {
            let state = context.state().borrow_mut();
            state.client_prompt_registry()
        };
        // The prompt has to reach its client before the next command can answer
        // it, so the registry call happens now and only the waiting is deferred.
        let start = suspend::client_prompt(
            self.args.clone(),
            &registry,
            prompt_target,
            context.client().tty_name.clone(),
            command_prompt_waits(&self.args),
        );
        SharedCommandExecution::completed(context.wait_for_answer(true, start).await)
    }
}

/// `confirm-before [-by] [-c confirm-key] [-p prompt] [-t target-client] command`.
#[derive(Clone, Debug)]
pub(in crate::server) struct ConfirmBefore {
    /// `-b`: put the prompt up without waiting for its answer.
    background: bool,
    /// `-c`: the key that confirms, `y` by default.
    confirm_key: Option<String>,
    /// `-p`: the prompt shown instead of the built one.
    prompt: Option<String>,
    /// `-t`: the client that is asked.
    target: Option<String>,
    /// `-y`: answer the prompt without asking.
    yes: bool,
    /// The command line the confirmation runs.
    command: Vec<String>,
}

impl ConfirmBefore {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            background: args.has('b'),
            confirm_key: args.value('c').map(str::to_string),
            prompt: args.value('p').map(str::to_string),
            target: args.value('t').map(str::to_string),
            yes: args.has('y'),
            command: args.positionals().to_vec(),
        })
    }

    fn run(self, state: &ServerState, client: &ClientContext) -> CommandResult {
        if self.command.is_empty() {
            return CommandResult::err(
                "command confirm-before: too few arguments (need at least 1)\n",
            );
        }
        let target = self.target.as_deref();
        let snapshots = state.client_snapshots();
        if let Some(target) = target {
            if !snapshots.iter().any(|c| c.name == target) {
                return CommandResult::err(format!("can't find client: {target}\n"));
            }
        } else {
            let found_current = client
                .tty_name
                .as_deref()
                .is_some_and(|tty| snapshots.iter().any(|c| c.name == tty));
            if !found_current {
                return CommandResult::err("no current client\n");
            }
        }
        // A single operand is a command *line*, so it is compiled here and the
        // words handed on; a line that does not compile is passed through
        // verbatim so the failure is reported when the prompt is answered.
        let command = if let [line] = self.command.as_slice() {
            ExecutableCommand::compile(line, &[])
                .map(|compiled| compiled.argv())
                .unwrap_or_else(|_| vec![line.clone()])
        } else {
            self.command.clone()
        };
        let confirm_key = match self.confirm_key.as_deref() {
            Some(value)
                if value.len() == 1 && value.as_bytes()[0] > 31 && value.as_bytes()[0] < 127 =>
            {
                value.as_bytes()[0]
            }
            Some(_) => return CommandResult::err("invalid confirm key\n"),
            None => b'y',
        };
        let prompt = self.prompt.as_deref().map_or_else(
            || {
                let name = command.first().map(String::as_str).unwrap_or_default();
                format!("Confirm '{name}'? ({}/n) ", confirm_key as char)
            },
            |prompt| format!("{prompt} "),
        );
        overlay_result(
            state.confirm_client(
                target,
                client.tty_name.as_deref(),
                prompt,
                command,
                confirm_key,
                self.yes,
                client.interaction_reply.clone(),
            ),
            target,
        )
    }
}

fn enter_mode(
    target: Option<&str>,
    state: &mut ServerState,
    view: ModeView,
    zoom: bool,
) -> CommandResult {
    let Some(target) = target.map(str::to_string).or_else(|| current_target(state)) else {
        return CommandResult::err("no current session\n");
    };
    if zoom {
        if let Err(error) = state.push_zoom(&target) {
            return CommandResult::err(format!("{error}\n"));
        }
    }
    let result = match state.enter_mode_view(&target, view) {
        Ok(()) => CommandResult::ok(""),
        Err(_) => CommandResult::err(format!("{}\n", state.pane_target_error(&target))),
    };
    if zoom {
        if let Err(error) = state.pop_zoom(&target, true) {
            return CommandResult::err(format!("{error}\n"));
        }
    }
    result
}

fn validate_mode_target(target: Option<&str>, state: &ServerState) -> Result<(), CommandResult> {
    let Some(target) = target.map(str::to_string).or_else(|| current_target(state)) else {
        return Err(CommandResult::err("no current session\n"));
    };
    if state.resolve(&target).is_none() {
        return Err(CommandResult::err(format!(
            "{}\n",
            state.pane_target_error(&target)
        )));
    }
    Ok(())
}

/// Substitute `%%` and compile what comes out.
///
/// The two are distinct steps at distinct times: the template is text until
/// there is a value for it, and only the substituted line is a command.
fn template_command(template: &str, value: &str) -> Vec<String> {
    let expanded = template_replace(template, value, true);
    ExecutableCommand::compile(&expanded, &[])
        .map(|compiled| compiled.argv())
        .unwrap_or_default()
}

/// tmux's `cmd_template_replace`: the first `%%` in a template takes the value,
/// and a third `%` after it asks for the value in the form the parse that
/// follows needs — the metacharacters escaped.
pub(in crate::server) fn template_replace(template: &str, value: &str, parsed: bool) -> String {
    let mut out = String::with_capacity(template.len());
    let mut characters = template.chars().peekable();
    let mut replaced = false;
    while let Some(character) = characters.next() {
        if character != '%' || replaced || characters.peek() != Some(&'%') {
            out.push(character);
            continue;
        }
        characters.next();
        replaced = true;
        let quoted = characters.peek() == Some(&'%');
        if quoted {
            characters.next();
        }
        for character in value.chars() {
            if quoted && parsed && matches!(character, '"' | '\\' | '$' | ';' | '~') {
                out.push('\\');
            }
            out.push(character);
        }
    }
    out
}

/// The `-F`, `-f`, `-O` and `-r` every `choose-*` command shares, as tmux's
/// `mode_tree_start` takes them.
#[derive(Clone, Debug)]
pub(super) struct ChooseOptions {
    pub(super) format: Option<String>,
    pub(super) filter: Option<String>,
    /// `-O`, resolved through the same table the `list-*` commands use when the
    /// command runs. `None` means no `-O` at all, which leaves each mode on its
    /// own first order.
    pub(super) order: Option<String>,
    /// `-r`: negate the comparison the sort order made.
    pub(super) reversed: bool,
    pub(super) zoom: bool,
}

impl ChooseOptions {
    fn parse(args: &ParsedArgs) -> Self {
        Self {
            format: args.value('F').map(str::to_string),
            filter: args.value('f').map(str::to_string),
            order: args.value('O').map(str::to_string),
            reversed: args.has('r'),
            zoom: args.has('Z'),
        }
    }

    /// tmux's `cmd_choose_tree_exec` runs `-O` through `sort_order_from_string`
    /// before it enters any mode, so a name that table does not know fails the
    /// command rather than picking an order.
    fn resolve_order(&self) -> Result<Option<ListSortOrder>, CommandResult> {
        list_sort_order(self.order.as_deref())
    }

    /// Whether this row survives `-f`.
    fn keep(&self, state: &ServerState, vars: &format::Vars) -> bool {
        self.filter.as_deref().is_none_or(|filter| {
            format::is_true(&super::expand_command_format(state, filter, vars, None))
        })
    }

    /// The row's text: `-F` when one was given, else the command's own.
    fn label(&self, state: &ServerState, vars: &format::Vars, default: String) -> String {
        match self.format.as_deref() {
            Some(format) => super::expand_command_format(state, format, vars, None),
            None => default,
        }
    }

    /// Sort `rows` the way tmux's `sort_qsort` sorts a flat mode: the `-O`
    /// comparison first, ties broken by the row's name, and `-r` negating the
    /// combined result rather than reversing the sorted list.
    ///
    /// `order` is the resolved `-O`; without one the mode's own first order
    /// applies, which is what tmux's per-mode `sortcb` writes over an unset
    /// `sort_crit->order`.
    fn sort<T>(
        &self,
        rows: &mut [T],
        order: Option<ListSortOrder>,
        default: ListSortOrder,
        compare: impl Fn(ListSortOrder, &T, &T) -> std::cmp::Ordering,
        name: impl Fn(&T) -> String,
    ) {
        apply_list_sort(
            rows,
            Some(order.unwrap_or(default)),
            self.reversed,
            compare,
            name,
        );
    }

    /// [`Self::sort`] for the tree, whose rows are two levels rather than one.
    ///
    /// tmux's `window_tree_build` sorts the sessions against each other and
    /// then each session's own windows within it, so a window never leaves the
    /// session that built it however the sessions are ordered. `-r` negates
    /// both levels, which keeps a session ahead of its windows.
    fn sort_tree(&self, groups: &mut [TreeGroup], order: Option<ListSortOrder>) {
        // tmux's `window_tree_sort`: without `-O` the tree takes the first of
        // its own sequence, which is by index.
        self.sort(
            groups,
            order,
            ListSortOrder::Index,
            |order, left, right| match order {
                ListSortOrder::Index => left.id.cmp(&right.id),
                ListSortOrder::Creation => sort_number(&left.session, "session_created")
                    .cmp(&sort_number(&right.session, "session_created")),
                // Newest first, and against the stamps themselves rather than
                // the whole seconds `#{session_activity}` publishes, so
                // sessions started within a second of each other still have an
                // order.
                ListSortOrder::Activity => right.activity.cmp(&left.activity),
                // `name` and the orders a session has no key for both fall to
                // the name, which is every tmux comparator's tiebreak.
                _ => std::cmp::Ordering::Equal,
            },
            |group| sort_text(&group.session, "session_name"),
        );
        for group in groups.iter_mut() {
            self.sort(
                &mut group.windows,
                order,
                ListSortOrder::Index,
                |order, left, right| match order {
                    ListSortOrder::Index => sort_number(&left.row.vars, "window_index")
                        .cmp(&sort_number(&right.row.vars, "window_index")),
                    ListSortOrder::Creation => left.id.cmp(&right.id),
                    ListSortOrder::Activity => right.activity.cmp(&left.activity),
                    ListSortOrder::Size => {
                        sort_area(&left.row.vars).cmp(&sort_area(&right.row.vars))
                    }
                    _ => std::cmp::Ordering::Equal,
                },
                |window| sort_text(&window.row.vars, "window_name"),
            );
        }
    }
}

fn sort_text(vars: &format::Vars, name: &str) -> String {
    vars.lookup(name).unwrap_or_default().to_owned()
}

fn sort_number(vars: &format::Vars, name: &str) -> i64 {
    vars.lookup(name)
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}

/// The window area tmux's `SORT_SIZE` compares.
fn sort_area(vars: &format::Vars) -> i64 {
    sort_number(vars, "window_width") * sort_number(vars, "window_height")
}

/// One session's rows on their way into the tree: the session's own row, when
/// the scope keeps it, and the window rows built underneath it.
struct TreeGroup {
    /// The session's variables, which order the group even when `-w` dropped
    /// the session's own row.
    session: format::Vars,
    /// The session id, which tmux's `SORT_INDEX` compares for sessions.
    id: u32,
    /// When the session was last active, at the resolution the state keeps it
    /// rather than the whole seconds the format publishes.
    activity: i64,
    session_row: Option<ChooseRow>,
    windows: Vec<TreeWindow>,
}

/// A window row with the keys that order it inside its session.
struct TreeWindow {
    /// The window id, allocated in creation order as tmux's is.
    id: u32,
    activity: i64,
    row: ChooseRow,
}

/// One row on its way into a mode tree, with the variables its format, filter
/// and sort order are all resolved against.
struct ChooseRow {
    vars: format::Vars,
    item: ModeItem,
}

/// A client row with the keys `sort_client_cmp` compares, kept at the
/// resolution the state holds them rather than the whole seconds the
/// `#{client_activity}` and `#{client_created}` formats publish.
struct ClientRow {
    name: String,
    cols: u16,
    rows: u16,
    created: i64,
    activity: i64,
    row: ChooseRow,
}

/// A buffer row with the keys `sort_buffer_cmp` compares.
struct BufferRow {
    name: String,
    size: usize,
    /// Where the buffer sat in the built list. tmux walks its buffers newest
    /// first, so a lower position is a higher `pb->order` — which is why the
    /// creation order below compares these ascending.
    position: usize,
    row: ChooseRow,
}

/// `choose-tree [-GNrswZ] [-F format] [-f filter] [-K key-format]
/// [-O sort-order] [-t target-pane] [template]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct ChooseTree {
    /// `-N`: hide the preview pane.
    pub(super) no_preview: bool,
    /// `-s`/`-w`: list only sessions, or only windows.
    pub(super) sessions_only: bool,
    pub(super) windows_only: bool,
    /// `-t`: the pane the mode opens in.
    pub(super) target: Option<String>,
    pub(super) options: ChooseOptions,
    pub(super) template: Option<String>,
}

impl ChooseTree {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            no_preview: args.has('N'),
            sessions_only: args.has('s'),
            windows_only: args.has('w'),
            target: args.value('t').map(str::to_string),
            options: ChooseOptions::parse(args),
            template: args.positionals().first().cloned(),
        })
    }

    pub(super) fn run(self, state: &mut ServerState, agents: &PaneAgents) -> CommandResult {
        let order = match self.options.resolve_order() {
            Ok(order) => order,
            Err(error) => return error,
        };
        let template = self.template.as_deref();
        let options = &self.options;
        let marked = state.marked_pane();
        let build_groups = |opts: &ChooseOptions| {
            let mut groups = Vec::new();
            for session in state.sessions() {
                let session_vars = super::vars_for(state, session, session.active, agents, marked);
                let mut group = TreeGroup {
                    session: session_vars.clone(),
                    id: session.id,
                    activity: session.activity_micros,
                    session_row: None,
                    windows: Vec::new(),
                };
                let mut kept_windows = 0usize;
                for (position, link) in session.windows.iter().enumerate() {
                    let window = state.session_window(session, position);
                    let target = format!("{}:{}", session.name, link.index);
                    let window_vars = super::vars_for(state, session, position, agents, marked);
                    // tmux's `window_tree_build_window` drops a window only when
                    // its one pane fails the filter; a window holding several
                    // panes survives whatever the filter says about them.
                    if window.panes.len() == 1 && !opts.keep(state, &window_vars) {
                        continue;
                    }
                    kept_windows += 1;
                    if self.sessions_only {
                        continue;
                    }
                    let title = window_vars.lookup("pane_title").unwrap_or("");
                    let default_label = if window.panes.len() == 1 && !title.is_empty() {
                        format!(
                            "  {}: {}: \"{}\" ({} panes)",
                            link.index,
                            window.name,
                            title,
                            window.panes.len()
                        )
                    } else {
                        format!(
                            "  {}: {} ({} panes)",
                            link.index,
                            window.name,
                            window.panes.len()
                        )
                    };
                    group.windows.push(TreeWindow {
                        id: window.id,
                        activity: window.activity_micros,
                        row: ChooseRow {
                            item: ModeItem {
                                label: opts.label(state, &window_vars, default_label),
                                command: template.map_or_else(
                                    || {
                                        vec![
                                            "select-window".to_string(),
                                            "-t".to_string(),
                                            target.clone(),
                                            ";".to_string(),
                                            "switch-client".to_string(),
                                            "-t".to_string(),
                                            session.name.clone(),
                                        ]
                                    },
                                    |template| template_command(template, &target),
                                ),
                                prompt_target: Some(format!("={}:{}.", session.name, link.index)),
                                edit: None,
                                tagged: false,
                                preview_target: Some(format!("={}:{}.", session.name, link.index)),
                                depth: u16::from(!self.windows_only),
                                expanded: None,
                                target: Some(ModeTarget::Window {
                                    session: session.name.clone(),
                                    index: link.index,
                                }),
                            },
                            vars: window_vars,
                        },
                    });
                }
                // `-s` lists only sessions and `-w` only windows; without either, a
                // session is followed by its own windows. tmux keeps a session
                // whenever any of its windows survived the filter.
                if !self.windows_only && kept_windows > 0 {
                    group.session_row = Some(ChooseRow {
                        item: ModeItem {
                            label: opts.label(
                                state,
                                &session_vars,
                                format!("{} ({} windows)", session.name, session.windows.len()),
                            ),
                            command: template_command(
                                template.unwrap_or("switch-client -Zt '%%'"),
                                &session.name,
                            ),
                            prompt_target: Some(format!("={}:", session.name)),
                            edit: None,
                            tagged: false,
                            preview_target: Some(format!("={}:", session.name)),
                            depth: 0,
                            expanded: None,
                            target: Some(ModeTarget::Session {
                                name: session.name.clone(),
                            }),
                        },
                        vars: session_vars,
                    });
                }
                groups.push(group);
            }
            groups
        };
        let mut groups = build_groups(options);
        if options.filter.is_some()
            && groups
                .iter()
                .all(|g| g.windows.is_empty() && g.session_row.is_none())
        {
            let mut unfiltered = options.clone();
            unfiltered.filter = None;
            groups = build_groups(&unfiltered);
        }
        options.sort_tree(&mut groups, order);
        let items = groups
            .into_iter()
            .flat_map(|group| {
                group
                    .session_row
                    .into_iter()
                    .chain(group.windows.into_iter().map(|window| window.row))
            })
            .map(|row| row.item)
            .collect();
        let mut view = ModeView::list(ModeKind::Tree, "Tree", items);
        // tmux shows the preview unless `-N` asks it not to.
        view.preview = !self.no_preview;
        enter_mode(self.target.as_deref(), state, view, self.options.zoom)
    }
}

/// `choose-client [-NrZ] [-F format] [-f filter] [-K key-format]
/// [-O sort-order] [-t target-pane] [template]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct ChooseClient {
    /// `-t`: the pane the mode opens in.
    target: Option<String>,
    options: ChooseOptions,
    template: Option<String>,
}

impl ChooseClient {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            target: args.value('t').map(str::to_string),
            options: ChooseOptions::parse(args),
            template: args.positionals().first().cloned(),
        })
    }

    fn run(self, state: &mut ServerState, agents: &PaneAgents) -> CommandResult {
        if let Err(error) = validate_mode_target(self.target.as_deref(), state) {
            return error;
        }
        let template = self.template.as_deref().unwrap_or("detach-client -t '%%'");
        let order = match self.options.resolve_order() {
            Ok(order) => order,
            Err(error) => return error,
        };
        let options = &self.options;
        let mut rows = Vec::new();
        for client in state.client_snapshots() {
            if client.suspended {
                continue;
            }
            let Some(vars) = client_vars(state, agents, &client) else {
                continue;
            };
            if !options.keep(state, &vars) {
                continue;
            }
            rows.push(ClientRow {
                name: client.name.clone(),
                cols: client.cols,
                rows: client.rows,
                created: client.created_micros,
                activity: client.activity_micros,
                row: ChooseRow {
                    item: ModeItem {
                        label: options.label(
                            state,
                            &vars,
                            format!(
                                "{}: {}x{} {}",
                                client.name, client.cols, client.rows, client.term
                            ),
                        ),
                        command: template_command(template, &client.name),
                        prompt_target: None,
                        edit: None,
                        tagged: false,
                        preview_target: None,
                        depth: 0,
                        expanded: None,
                        target: Some(ModeTarget::Client {
                            name: client.name.clone(),
                        }),
                    },
                    vars,
                },
            });
        }
        // tmux's `window_client_sort`: without `-O` the mode sorts by name.
        options.sort(
            &mut rows,
            order,
            ListSortOrder::Name,
            |order, left, right| match order {
                // Width first and then height, as `sort_client_cmp` compares the
                // tty's `sx` before its `sy`.
                ListSortOrder::Size => left
                    .cols
                    .cmp(&right.cols)
                    .then_with(|| left.rows.cmp(&right.rows)),
                ListSortOrder::Creation => left.created.cmp(&right.created),
                // Newest first, the direction `list-clients` already reads it in.
                ListSortOrder::Activity => right.activity.cmp(&left.activity),
                // `name` and the orders a client has no key for both fall to the
                // name, which is every tmux comparator's tiebreak.
                _ => std::cmp::Ordering::Equal,
            },
            |client| client.name.clone(),
        );
        let items = rows
            .into_iter()
            .map(|client| client.row.item)
            .collect::<Vec<_>>();
        if items.is_empty() {
            return CommandResult::ok("");
        }
        enter_mode(
            self.target.as_deref(),
            state,
            ModeView::list(ModeKind::Client, "Clients", items),
            self.options.zoom,
        )
    }
}

/// `choose-buffer [-NrZ] [-F format] [-f filter] [-K key-format]
/// [-O sort-order] [-t target-pane] [template]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct ChooseBuffer {
    /// `-t`: the pane the mode opens in.
    target: Option<String>,
    options: ChooseOptions,
    template: Option<String>,
}

impl ChooseBuffer {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            target: args.value('t').map(str::to_string),
            options: ChooseOptions::parse(args),
            template: args.positionals().first().cloned(),
        })
    }

    fn run(self, state: &mut ServerState) -> CommandResult {
        if let Err(error) = validate_mode_target(self.target.as_deref(), state) {
            return error;
        }
        let template = self
            .template
            .as_deref()
            .unwrap_or("paste-buffer -p -b '%%'");
        let order = match self.options.resolve_order() {
            Ok(order) => order,
            Err(error) => return error,
        };
        let options = &self.options;
        let mut rows = Vec::new();
        for (position, (name, data)) in state.buffers().iter().enumerate() {
            let vars = super::buffer_vars(state, name, data);
            if !options.keep(state, &vars) {
                continue;
            }
            let preview = String::from_utf8_lossy(data).replace(['\n', '\r'], " ");
            rows.push(BufferRow {
                name: name.clone(),
                size: data.len(),
                position,
                row: ChooseRow {
                    item: ModeItem {
                        label: options.label(
                            state,
                            &vars,
                            format!("{name}: {} bytes: {}", data.len(), preview),
                        ),
                        command: template_command(template, name),
                        // Buffer mode's `e` key edits the buffer this row names.
                        prompt_target: Some(name.clone()),
                        edit: None,
                        tagged: false,
                        preview_target: None,
                        depth: 0,
                        expanded: None,
                        target: Some(ModeTarget::Buffer { name: name.clone() }),
                    },
                    vars,
                },
            });
        }
        // tmux's `window_buffer_sort`: without `-O` the mode sorts by creation.
        options.sort(
            &mut rows,
            order,
            ListSortOrder::Creation,
            |order, left, right| match order {
                // Newest first, which the built order already runs in.
                ListSortOrder::Creation => left.position.cmp(&right.position),
                ListSortOrder::Size => left.size.cmp(&right.size),
                // `name` and the orders a buffer has no key for both fall to the
                // name, which is every tmux comparator's tiebreak.
                _ => std::cmp::Ordering::Equal,
            },
            |buffer| buffer.name.clone(),
        );
        let items = rows
            .into_iter()
            .map(|buffer| buffer.row.item)
            .collect::<Vec<_>>();
        if items.is_empty() {
            return CommandResult::ok("");
        }
        enter_mode(
            self.target.as_deref(),
            state,
            ModeView::list(ModeKind::Buffer, "Buffers", items),
            self.options.zoom,
        )
    }
}

/// `customize-mode [-NZ] [-F format] [-f filter] [-t target-pane]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct CustomizeMode {
    /// `-N`: hide the preview pane.
    no_preview: bool,
    /// `-t`: the pane the mode opens in.
    target: Option<String>,
    /// `customize-mode` takes no `-O`, so this only ever carries `-F` and `-f`.
    options: ChooseOptions,
}

impl CustomizeMode {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            no_preview: args.has('N'),
            target: args.value('t').map(str::to_string),
            options: ChooseOptions::parse(args),
        })
    }

    fn run(self, state: &mut ServerState) -> CommandResult {
        let Some(target) = self.target.clone().or_else(|| current_target(state)) else {
            return CommandResult::err("no current session\n");
        };
        let options = &self.options;
        // tmux's `window_customize_build` groups the options by the table they
        // belong to, each under a heading of its own, and the keys after them.
        let mut items = Vec::new();
        let sections = match state.customize_option_sections(&target) {
            Ok(sections) => sections,
            Err(_) => return CommandResult::err(format!("{}\n", state.pane_target_error(&target))),
        };
        for (title, entries) in sections {
            items.push(ModeItem {
                label: title.to_string(),
                command: Vec::new(),
                prompt_target: None,
                edit: None,
                tagged: false,
                preview_target: None,
                depth: 0,
                // `M-+` expands the complete tree after the initial screen.
                expanded: Some(false),
                target: None,
            });
            for entry in entries {
                let mut vars = format::Vars::new();
                vars.set("option_name", entry.name.clone())
                    .set("option_value", entry.value.clone())
                    .set("option_scope", entry.scope.clone())
                    .set("option_unit", String::new())
                    .set(
                        "option_is_global",
                        if entry.scope.is_empty() { "1" } else { "0" },
                    )
                    .set("option_is_array", if entry.is_array { "1" } else { "0" })
                    .set("is_option", "1")
                    .set("is_key", "0");
                if !options.keep(state, &vars) {
                    continue;
                }
                items.push(ModeItem {
                    label: if entry.is_array {
                        entry.name.clone()
                    } else {
                        options.label(state, &vars, format!("{} {}", entry.name, entry.value))
                    },
                    command: Vec::new(),
                    prompt_target: None,
                    edit: (!entry.is_array).then_some(ModeEdit::Option {
                        name: entry.name,
                        value: entry.value,
                    }),
                    tagged: false,
                    preview_target: None,
                    depth: 1,
                    expanded: entry
                        .is_array
                        .then_some(entry.array_has_entries)
                        .and_then(|has_entries| has_entries.then_some(false)),
                    target: None,
                });
            }
        }
        let bindings = state
            .key_bindings(None)
            .into_iter()
            .map(|(table, key, binding)| {
                (
                    table.to_string(),
                    format_key_name(key),
                    binding.command.argv(),
                    binding.note.clone(),
                    binding.repeat,
                )
            })
            .collect::<Vec<_>>();
        for (table, key, command, note, repeat) in bindings {
            let command_value = display_command(&command);
            items.push(ModeItem {
                label: format!("key {table} {key} command {command_value}"),
                command: Vec::new(),
                prompt_target: None,
                edit: Some(ModeEdit::BindingCommand {
                    table: table.clone(),
                    key: key.clone(),
                    value: command_value,
                    note: note.clone(),
                    repeat,
                }),
                tagged: false,
                preview_target: None,
                depth: 1,
                expanded: None,
                target: None,
            });
            let note_value = note.unwrap_or_default();
            items.push(ModeItem {
                label: format!("key {table} {key} note {note_value}"),
                command: Vec::new(),
                prompt_target: None,
                edit: Some(ModeEdit::BindingNote {
                    table,
                    key,
                    value: note_value,
                    command,
                    repeat,
                }),
                tagged: false,
                preview_target: None,
                depth: 1,
                expanded: None,
                target: None,
            });
        }
        // tmux's options mode has no title row above its section tree.
        let mut view = ModeView::list(ModeKind::Customize, "", items);
        view.preview = !self.no_preview;
        enter_mode(self.target.as_deref(), state, view, self.options.zoom)
    }
}

/// `clock-mode [-t target-pane]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct ClockMode {
    /// `-t`: the pane the clock opens in.
    target: Option<String>,
}

impl ClockMode {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            target: args.value('t').map(str::to_string),
        })
    }

    fn run(self, state: &mut ServerState) -> CommandResult {
        enter_mode(self.target.as_deref(), state, ModeView::clock(), false)
    }
}

/// Whether an attached client answers to `name`, matching the lookup the client
/// registry itself does for a `-c`/`-t` client target: the client's own name, or
/// that name with `/dev/` stripped off the front.
fn client_named(state: &ServerState, name: &str) -> bool {
    let name = name.strip_suffix(':').unwrap_or(name);
    state.client_snapshots().iter().any(|client| {
        client.name == name
            || client
                .name
                .strip_prefix("/dev/")
                .is_some_and(|tty| tty == name)
    })
}

fn overlay_result(result: ClientActionResult, target: Option<&str>) -> CommandResult {
    match result {
        ClientActionResult::Queued => CommandResult::ok(""),
        ClientActionResult::NoCurrentClient => CommandResult::err("no current client\n"),
        ClientActionResult::TargetNotFound => CommandResult::err(format!(
            "can't find client: {}\n",
            target.unwrap_or_default()
        )),
    }
}

/// `display-menu [-MO] [-b border-lines] [-c target-client] [-C starting-choice]
/// [-H selected-style] [-s style] [-S border-style] [-t target-pane]
/// [-T title] [-x position] [-y position] name [key] [command] ...`.
#[derive(Clone, Debug)]
pub(in crate::server) struct DisplayMenu {
    /// `-c`: the client the menu is shown on.
    client: Option<String>,
    /// `-C`: which entry starts selected.
    selected: Option<String>,
    /// `-t`: the pane the item formats are expanded against.
    target: Option<String>,
    /// `-T`: the menu's title.
    title: Option<String>,
    /// `-x`/`-y`: where the menu is placed.
    x: Option<String>,
    y: Option<String>,
    /// The menu's entries, as name/key/command triples.
    items: Vec<String>,
}

impl DisplayMenu {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            client: args.value('c').map(str::to_string),
            selected: args.value('C').map(str::to_string),
            target: args.value('t').map(str::to_string),
            title: args.value('T').map(str::to_string),
            x: args.value('x').map(str::to_string),
            y: args.value('y').map(str::to_string),
            items: args.positionals().to_vec(),
        })
    }

    fn run(
        self,
        state: &ServerState,
        agents: &PaneAgents,
        client: &ClientContext,
    ) -> CommandResult {
        let target = self.client.as_deref();
        let snapshots = state.client_snapshots();
        if let Some(target) = target {
            if !snapshots.iter().any(|c| c.name == target) {
                return CommandResult::err(format!("can't find client: {target}\n"));
            }
        } else {
            let found_current = client
                .tty_name
                .as_deref()
                .is_some_and(|tty| snapshots.iter().any(|c| c.name == tty));
            if !found_current {
                return CommandResult::err("no current client\n");
            }
        }
        // Both halves of an item are formats — tmux expands them when the menu is
        // built, against the target the menu was asked for.
        let vars = self
            .target
            .clone()
            .or_else(|| current_target(state))
            .as_deref()
            .and_then(|target| state.resolve_or_residual(target))
            .map(|resolved| {
                vars_full(
                    state,
                    &state.sessions()[resolved.session],
                    resolved.window,
                    resolved.pane,
                    agents,
                    state.marked_pane(),
                )
            })
            .unwrap_or_default();
        let expand = |value: &str| format::expand(value, &vars);
        // tmux walks the operands rather than chunking them: an empty name is a
        // separator line on its own and consumes no key or command, which is how
        // the default `MouseDown3*` menus group their entries.
        let values = &self.items;
        let mut items = Vec::new();
        let mut index = 0;
        while index < values.len() {
            let label = values[index].as_str();
            index += 1;
            if label.is_empty() {
                items.push(MenuItem {
                    label: String::new(),
                    key: String::new(),
                    command: Vec::new(),
                });
                continue;
            }
            if values.len() - index < 2 {
                return CommandResult::err("not enough arguments\n");
            }
            let key = values[index].as_str();
            let command = values[index + 1].as_str();
            index += 2;
            // An item whose name expands to nothing is not an item at all, and one
            // whose name starts with `-` is disabled: it shows neither its key nor
            // the dash, and cannot be chosen.
            let label = expand(label);
            if label.is_empty() {
                continue;
            }
            let disabled = label.starts_with('-');
            items.push(MenuItem {
                label,
                key: if disabled {
                    String::new()
                } else {
                    key.to_string()
                },
                command: if disabled {
                    Vec::new()
                } else {
                    template_command(&expand(command), "")
                },
            });
        }
        let selected = self
            .selected
            .as_deref()
            .and_then(|value| value.parse().ok())
            .unwrap_or(0);
        let request = OverlayRequest::Menu(MenuRequest {
            title: self.title.clone().unwrap_or_default(),
            items,
            selected,
            x: self.x.clone(),
            y: self.y.clone(),
            pane: overlay_target_pane(state, self.target.as_deref()),
            mouse: overlay_mouse(client),
        });
        let target = self.client.as_deref();
        overlay_result(
            state.overlay_client(
                target,
                client.tty_name.as_deref(),
                request,
                client.interaction_reply.clone(),
            ),
            target,
        )
    }
}

/// `display-popup [-BCEkN] [-b border-lines] [-c target-client]
/// [-d start-directory] [-e environment] [-h height] [-s style]
/// [-S border-style] [-t target-pane] [-T title] [-w width] [-x position]
/// [-y position] [shell-command [argument ...]]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct DisplayPopup {
    /// `-B`: draw the popup without a border.
    no_border: bool,
    /// `-C`: close the client's popup instead of opening one.
    close: bool,
    /// `-E`: close the popup when its command exits; twice, only on success.
    exit_flags: usize,
    /// `-k`: close the popup on any key.
    close_on_key: bool,
    /// `-c`: the client the popup is shown on.
    client: Option<String>,
    /// `-d`: the popup command's working directory.
    cwd: Option<String>,
    /// `-e`: environment assignments for the popup command, repeatable.
    environment: Vec<String>,
    /// `-w`/`-h`: the popup's size.
    width: Option<String>,
    height: Option<String>,
    /// `-x`/`-y`: where the popup is placed.
    x: Option<String>,
    y: Option<String>,
    /// `-T`: the popup's title.
    title: Option<String>,
    /// `-t`: the pane the popup is placed against.
    target: Option<String>,
    /// The popup's command line.
    command: Vec<String>,
}

impl DisplayPopup {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            no_border: args.has('B'),
            close: args.has('C'),
            exit_flags: args.count('E'),
            close_on_key: args.has('k'),
            client: args.value('c').map(str::to_string),
            target: args.value('t').map(str::to_string),
            cwd: args.value('d').map(str::to_string),
            environment: args.values('e').map(str::to_string).collect(),
            width: args.value('w').map(str::to_string),
            height: args.value('h').map(str::to_string),
            x: args.value('x').map(str::to_string),
            y: args.value('y').map(str::to_string),
            title: args.value('T').map(str::to_string),
            command: args.positionals().to_vec(),
        })
    }

    fn run(self, state: &ServerState, client: &ClientContext) -> CommandResult {
        let target = self.client.as_deref();
        if self.close {
            return overlay_result(
                state.overlay_client(
                    target,
                    client.tty_name.as_deref(),
                    OverlayRequest::Clear,
                    client.interaction_reply.clone(),
                ),
                target,
            );
        }
        let overrides = self
            .environment
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let request = OverlayRequest::Popup(PopupRequest {
            on_close: Vec::new(),
            on_close_remove: None,
            content: None,
            title: self.title.clone().unwrap_or_default(),
            argv: self.command.clone(),
            environment: state.spawn_environment(
                current_target(state).as_deref(),
                &client.environment,
                &overrides,
            ),
            cwd: self
                .cwd
                .clone()
                .map(PathBuf::from)
                .or_else(|| client.cwd.clone()),
            width: self.width.clone(),
            height: self.height.clone(),
            x: self.x.clone(),
            y: self.y.clone(),
            pane: overlay_target_pane(state, self.target.as_deref()),
            mouse: overlay_mouse(client),
            close_on_exit: self.exit_flags == 1,
            close_on_success: self.exit_flags >= 2,
            close_on_key: self.close_on_key,
            border: !self.no_border,
        });
        overlay_result(
            state.overlay_client(
                target,
                client.tty_name.as_deref(),
                request,
                client.interaction_reply.clone(),
            ),
            target,
        )
    }
}

/// `display-panes [-bN] [-d duration] [-t target-client] [template]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct DisplayPanes {
    /// `-b`: put the indicators up without waiting for a choice.
    background: bool,
    /// `-N`: show the indicators without accepting a choice.
    no_input: bool,
    /// `-d`: how long the indicators stay up.
    duration: Option<String>,
    /// `-t`: the client the indicators are shown on.
    target: Option<String>,
    template: Option<String>,
}

impl DisplayPanes {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            background: args.has('b'),
            no_input: args.has('N'),
            duration: args.value('d').map(str::to_string),
            target: args.value('t').map(str::to_string),
            template: args.positionals().first().cloned(),
        })
    }

    fn run(self, state: &ServerState, client: &ClientContext) -> CommandResult {
        let duration_ms = self
            .duration
            .as_deref()
            .and_then(|value| value.parse().ok())
            .or_else(|| {
                current_target(state).and_then(|target| {
                    state
                        .option_for_target(&target, "display-panes-time")
                        .and_then(|value| value.parse().ok())
                })
            })
            .unwrap_or(1000);
        let command = self
            .template
            .as_deref()
            .map(|template| template_command(template, "%%"))
            .unwrap_or_default();
        let target = self.target.as_deref();
        overlay_result(
            state.overlay_client(
                target,
                client.tty_name.as_deref(),
                OverlayRequest::DisplayPanes {
                    duration_ms,
                    command,
                    accept_input: !self.no_input,
                },
                client.interaction_reply.clone(),
            ),
            target,
        )
    }
}

/// `list-clients [-r] [-F format] [-f filter] [-O order] [-t target-session]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct ListClients {
    /// `-F`: the line each client expands to.
    format: Option<String>,
    /// `-f`: only list clients this format is true for.
    filter: Option<String>,
    /// `-O`: the sort key, resolved when the command runs.
    order: Option<String>,
    /// `-r`: reverse the sort.
    reversed: bool,
    /// `-t`: only list the clients attached to this session.
    target: Option<String>,
}

impl ListClients {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            format: args.value('F').map(str::to_string),
            filter: args.value('f').map(str::to_string),
            order: args.value('O').map(str::to_string),
            reversed: args.has('r'),
            target: args.value('t').map(str::to_string),
        })
    }

    fn run(self, state: &ServerState, agents: &PaneAgents) -> CommandResult {
        const DEFAULT_FORMAT: &str = "#{client_name}: #{session_name} [#{client_width}x#{client_height} #{client_termname}] #{?client_flags,(,}#{client_flags}#{?client_flags,),}";
        if self.target.is_none() && state.sessions().is_empty() {
            return CommandResult::err("no current target\n");
        }
        let template = self.format.as_deref().unwrap_or(DEFAULT_FORMAT);
        let requested_session = self.target.as_deref();
        let target_session = requested_session.and_then(|target| state.resolve_session(target));
        if requested_session.is_some() && target_session.is_none() {
            return CommandResult::err(format!(
                "can't find session: {}\n",
                requested_session.unwrap_or_default()
            ));
        }
        let sort_order = match super::list_sort_order(self.order.as_deref()) {
            Ok(order) => order,
            Err(error) => return error,
        };
        // tmux lists from `sort_get_clients`, which skips a client stopped by
        // `suspend-client` until it wakes back up.
        let mut clients: Vec<_> = state
            .client_snapshots()
            .into_iter()
            .filter(|client| !client.suspended)
            .collect();
        super::apply_list_sort(
            &mut clients,
            sort_order,
            self.reversed,
            |key, a, b| match key {
                super::ListSortOrder::Name => a.name.cmp(&b.name),
                super::ListSortOrder::Size => a.cols.cmp(&b.cols).then(a.rows.cmp(&b.rows)),
                super::ListSortOrder::Creation => a.created_micros.cmp(&b.created_micros),
                // Most recent activity first, as tmux inverts this comparison.
                super::ListSortOrder::Activity => b.activity_micros.cmp(&a.activity_micros),
                _ => std::cmp::Ordering::Equal,
            },
            |client| client.name.clone(),
        );
        let mut output = String::new();
        for (line, client) in clients.into_iter().enumerate() {
            let Some(session) = state
                .sessions()
                .iter()
                .find(|session| session.id == client.session_id)
            else {
                continue;
            };
            if target_session.is_some_and(|target| target.id != session.id) {
                continue;
            }
            let Some(mut vars) = client_vars(state, agents, &client) else {
                continue;
            };
            // tmux numbers the row by its position in the sorted client array,
            // which the session filter above skips over rather than renumbers.
            vars.set("line", line.to_string());
            if self
                .filter
                .as_deref()
                .is_some_and(|filter| !format::is_true(&format::expand(filter, &vars)))
            {
                continue;
            }
            output.push_str(&format::expand(template, &vars));
            output.push('\n');
        }
        CommandResult::ok(output)
    }
}

/// The client-entry format variables shared by the `list-clients` context and
/// the client a `display-message` resolves (tmux's `format_defaults_client`).
pub(super) fn set_client_entry_vars(
    state: &ServerState,
    client: &super::super::state::ClientSnapshot,
    vars: &mut format::Vars,
) {
    let last_session = client.last_session_id.and_then(|id| {
        state
            .sessions()
            .iter()
            .find(|session| session.id == id)
            .map(|session| session.name.clone())
    });
    vars.set(
        "client_activity",
        (client.activity_micros / 1_000_000).to_string(),
    )
    .set(
        "client_created",
        (client.created_micros / 1_000_000).to_string(),
    )
    .set("client_cell_width", client.xpixel.to_string())
    .set("client_cell_height", client.ypixel.to_string())
    .set("client_termfeatures", client.termfeatures.clone())
    .set("client_termtype", client.termtype.clone())
    .set("client_written", client.written.to_string())
    // hmux never discards output the way tmux's backoff does.
    .set("client_discarded", "0")
    .set("client_last_session", last_session.unwrap_or_default());
}

/// The format variables one attached client answers to, shared by
/// `list-clients` and `choose-client`. `None` when the client's session has
/// gone away underneath it.
pub(super) fn client_vars(
    state: &ServerState,
    agents: &PaneAgents,
    client: &super::super::state::ClientSnapshot,
) -> Option<format::Vars> {
    let session = state
        .sessions()
        .iter()
        .find(|session| session.id == client.session_id)?;
    let client_utf8 = client.flags.split(',').any(|flag| flag == "UTF-8");
    let mut vars = super::vars_for(state, session, session.active, agents, state.marked_pane());
    set_client_entry_vars(state, client, &mut vars);
    // The format's session here is the client's own session.
    vars.set("session_active", "1");
    vars.set("client_name", client.name.clone())
        .set("client_tty", client.name.clone())
        .set("client_termname", client.term.clone())
        .set(
            "client_pid",
            client.pid.map(|pid| pid.to_string()).unwrap_or_default(),
        )
        .set(
            "client_uid",
            client.uid.map(|uid| uid.to_string()).unwrap_or_default(),
        )
        .set("client_user", client.user.clone())
        .set("client_width", client.cols.to_string())
        .set("client_height", client.rows.to_string())
        .set("client_session", session.name.clone())
        .set("client_flags", client.flags.clone())
        .set("client_readonly", if client.read_only { "1" } else { "0" })
        .set(
            "client_control_mode",
            if client.control_mode { "1" } else { "0" },
        )
        .set("client_utf8", if client_utf8 { "1" } else { "0" })
        .set("client_theme", client.theme.clone());
    Some(vars)
}

/// `detach-client [-aP] [-E shell-command] [-s target-session]
/// [-t target-client]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct DetachClient {
    /// `-a`: detach all other clients except the target.
    all_others: bool,
    /// `-P`: the detached client is told to hang itself up (SIGHUP).
    hangup: bool,
    /// `-E`: the command the detached client runs in place of exiting.
    command: Option<String>,
    /// `-s`: detach all clients attached to the specified session.
    session: Option<String>,
    /// `-t`: the client to detach.
    target: Option<String>,
}

impl DetachClient {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            all_others: args.has('a'),
            hangup: args.has('P'),
            command: args.value('E').map(str::to_string),
            session: args.value('s').map(str::to_string),
            target: args.value('t').map(str::to_string),
        })
    }

    fn run(self, state: &ServerState, client: &ClientContext) -> CommandResult {
        if let Some(session_target) = self.session.as_deref() {
            let Some(session) = state.find(session_target) else {
                return CommandResult::err(format!("can't find session: {session_target}\n"));
            };
            let session_id = session.id;
            state.detach_session_clients(session_id, self.command.as_deref(), self.hangup);
            return CommandResult::ok("");
        }
        let target = self.target.as_deref();
        if self.all_others {
            return overlay_result(
                state.detach_all_other_clients(
                    target,
                    client.tty_name.as_deref(),
                    self.command.as_deref(),
                    self.hangup,
                ),
                target,
            );
        }
        overlay_result(
            state.detach_client(
                target,
                client.tty_name.as_deref(),
                self.command.as_deref(),
                self.hangup,
            ),
            target,
        )
    }
}

/// `suspend-client [-t target-client]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct SuspendClient {
    /// `-t`: the client to suspend.
    target: Option<String>,
}

impl SuspendClient {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            target: args.value('t').map(str::to_string),
        })
    }

    fn run(self, state: &ServerState, client: &ClientContext) -> CommandResult {
        let target = self.target.as_deref();
        overlay_result(
            state.suspend_client(target, client.tty_name.as_deref()),
            target,
        )
    }
}

/// `lock-client [-t target-client]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct LockClient {
    /// `-t`: the client to lock.
    target: Option<String>,
}

impl LockClient {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            target: args.value('t').map(str::to_string),
        })
    }

    fn run(self, state: &ServerState, context: &ClientContext) -> CommandResult {
        let target = self.target.as_deref();
        match state.lock_client(target, context.tty_name.as_deref()) {
            ClientActionResult::Queued => CommandResult::ok(""),
            ClientActionResult::NoCurrentClient => CommandResult::err("no current client\n"),
            ClientActionResult::TargetNotFound => CommandResult::err(format!(
                "can't find client: {}\n",
                target.unwrap_or_default()
            )),
        }
    }
}

/// `refresh-client [-cDlLRSU] [-A pane:state] [-B name:what:format] [-C XxY]
/// [-f flags] [-r pane:report] [-t target-client] [adjustment]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct RefreshClient {
    /// `-L`/`-R`/`-U`/`-D`: pan the client's view by `adjustment` cells.
    left: bool,
    right: bool,
    up: bool,
    down: bool,
    /// `-c`: re-centre the client's view.
    centre: bool,
    /// `-l`: ask the client's terminal for its selection.
    clipboard: bool,
    has_control_flag: bool,
    /// `-f` (and its historical `-F` spelling): the client flags to set.
    flags: Vec<String>,
    /// `-t`: the client to refresh.
    target: Option<String>,
    /// `-r`: terminal reports a control client is answering for a pane.
    reports: Vec<String>,
    adjustment: Option<String>,
}

impl RefreshClient {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        // `-F` is the historical spelling of `-f`.
        let flags = args
            .values('f')
            .chain(args.values('F'))
            .map(str::to_string)
            .collect();
        let has_control_flag = args.has('A') || args.has('B') || args.has('C');
        Ok(Self {
            left: args.has('L'),
            right: args.has('R'),
            up: args.has('U'),
            down: args.has('D'),
            centre: args.has('c'),
            clipboard: args.has('l'),
            has_control_flag,
            flags,
            target: args.value('t').map(str::to_string),
            reports: args.values('r').map(str::to_string).collect(),
            adjustment: args.positionals().first().cloned(),
        })
    }

    fn run(self, state: &mut ServerState, client: &ClientContext) -> CommandResult {
        let target = self.target.as_deref();
        // tmux's `cmd_find_client` resolves an omitted `-c` to the current
        // client, which for a command client that owns no terminal is the sole
        // attached one.
        let target_client = match state.resolve_target_client(target, client.tty_name.as_deref()) {
            Ok(target_client) => target_client,
            Err(ClientActionResult::NoCurrentClient) => {
                return CommandResult::err("no current client\n")
            }
            Err(_) => {
                return CommandResult::err(format!(
                    "can't find client: {}\n",
                    target.unwrap_or_default()
                ))
            }
        };

        if self.has_control_flag && !target_client.control_mode {
            return CommandResult::err("not a control client\n");
        }

        // `-c` and the four pan directions are handled first and alone, as tmux's
        // `cmd_refresh_client_exec` returns straight after them.
        let pan = if self.left {
            Some(Some(WindowResizeAdjust::Left))
        } else if self.right {
            Some(Some(WindowResizeAdjust::Right))
        } else if self.up {
            Some(Some(WindowResizeAdjust::Up))
        } else if self.down {
            Some(Some(WindowResizeAdjust::Down))
        } else if self.centre {
            Some(None)
        } else {
            None
        };
        if let Some(adjust) = pan {
            let adjustment = match self.adjustment.as_deref().unwrap_or("1").parse::<u16>() {
                Ok(0) => return CommandResult::err("adjustment too small\n"),
                Ok(value) => value,
                Err(_) => return CommandResult::err("adjustment invalid\n"),
            };
            return overlay_result(
                state.pan_client(
                    Some(&target_client.name),
                    client.tty_name.as_deref(),
                    adjust,
                    adjustment,
                ),
                target,
            );
        }
        // `-r %pane:REPORT` hands the server the OSC 10/11 answer the client's
        // own terminal gave, which then answers that pane's questions.
        for report in &self.reports {
            state.set_pane_control_colour(report);
        }
        if !self.flags.is_empty() {
            let result =
                state.refresh_client_flags(target, client.tty_name.as_deref(), &self.flags);
            if result != ClientActionResult::Queued {
                return overlay_result(result, target);
            }
        }
        // `-l` asks the client's terminal for its selection. tmux keeps one
        // outstanding query per terminal, so a repeat inside the timeout is
        // dropped rather than queued.
        if self.clipboard && state.begin_clipboard_query(target, client.tty_name.as_deref()) {
            let result = state.set_client_selection(target, client.tty_name.as_deref(), None);
            if result != ClientActionResult::Queued {
                return overlay_result(result, target);
            }
        }
        overlay_result(
            state.refresh_client(target, client.tty_name.as_deref()),
            target,
        )
    }
}

/// `switch-client [-ElnprZ] [-c target-client] [-t target-session]
/// [-T key-table] [-O order]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct SwitchClient {
    /// `-c`: the client to move.
    client: Option<String>,
    /// `-r`: toggle the target client's read-only state first, and reverse the
    /// session order `-n`/`-p` cycle in.
    toggle_read_only: bool,
    /// `-Z`: keep target window zoomed.
    zoom: bool,
    /// `-n`/`-p`/`-l`: cycle to the next, previous, or last session instead of
    /// a named one.
    next: bool,
    previous: bool,
    last: bool,
    /// `-O`: the order `-n`/`-p` cycle the sessions in.
    order: Option<String>,
    /// `-T`: put the target client in a key table instead of switching it.
    key_table: Option<String>,
    /// `-E`: skip the `update-environment` copy into the destination session.
    no_environment: bool,
    /// `-t`: the session (or window, or pane) to move it to.
    target: Option<String>,
}

impl SwitchClient {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            client: args.value('c').map(str::to_string),
            toggle_read_only: args.has('r'),
            zoom: args.has('Z'),
            no_environment: args.has('E'),
            next: args.has('n'),
            previous: args.has('p'),
            last: args.has('l'),
            order: args.value('O').map(str::to_string),
            key_table: args.value('T').map(str::to_string),
            target: args.value('t').map(str::to_string),
        })
    }

    fn run(self, state: &mut ServerState, client: &ClientContext) -> CommandResult {
        // tmux resolves `-c` while it prepares the queue item, so a client that
        // does not exist is reported before the command body runs at all.
        if let Some(name) = self.client.as_deref() {
            if !client_named(state, name) {
                return CommandResult::err(format!("can't find client: {name}\n"));
            }
        }
        // `-T` names a key table for the target client and returns before any
        // session is touched, and `-n`/`-p`/`-l` pick their session from the
        // target client rather than from `-t`.
        if self.key_table.is_some() || self.next || self.previous || self.last {
            return self.run_on_target_client(state, client);
        }
        let Some(target_session) = self.target.as_deref() else {
            return CommandResult::err("no current client\n");
        };
        // tmux resolves a target naming a window or pane — including `=`, the
        // mouse's — as a pane target and makes that window current before moving
        // the client, which is what the default `MouseDown1Status` binding relies
        // on to turn the status line into a window switcher.
        let resolved_pane_target = (target_session == "="
            || target_session.contains([':', '.', '%']))
        .then(|| state.resolve(target_session))
        .flatten();
        let session_id = if let Some(resolved) = resolved_pane_target {
            let session = &state.sessions()[resolved.session];
            let session_id = session.id;
            let window_target = format!(
                "{}:{}",
                session.name, session.windows[resolved.window].index
            );
            let pane_id = state.window(resolved.session, resolved.window).panes[resolved.pane].id;
            let _ = state.select_pane(&format!("%{pane_id}"));
            if !self.zoom {
                let win = state.window_mut(resolved.session, resolved.window);
                win.zoomed = false;
            }
            let _ = state.select_window(&window_target);
            session_id
        } else {
            let Some(session_id) = state.session_id(target_session) else {
                return CommandResult::err(format!("can't find session: {target_session}\n"));
            };
            session_id
        };
        // tmux toggles the target client's read-only state — and `ignore-size`
        // with it — once the target has resolved, then moves that same client.
        let toggled = if self.toggle_read_only {
            match state.toggle_client_read_only(self.client.as_deref(), client.tty_name.as_deref())
            {
                Ok(name) => Some(name),
                Err(ClientActionResult::NoCurrentClient) => {
                    return CommandResult::err("no current client\n");
                }
                Err(_) => {
                    return CommandResult::err(format!(
                        "can't find client: {}\n",
                        self.client.as_deref().unwrap_or_default()
                    ))
                }
            }
        } else {
            None
        };
        self.refresh_destination_environment(state, client, session_id);
        match state.switch_client(
            self.client.as_deref().or(toggled.as_deref()),
            client.tty_name.as_deref(),
            session_id,
        ) {
            ClientActionResult::Queued => CommandResult::ok(""),
            ClientActionResult::NoCurrentClient => CommandResult::err("no current client\n"),
            ClientActionResult::TargetNotFound => CommandResult::err(format!(
                "can't find client: {}\n",
                self.client.as_deref().unwrap_or_default()
            )),
        }
    }

    /// tmux's `environ_update(s->options, tc->environ, s->environ)`: the
    /// destination session takes the `update-environment` names out of the
    /// environment the *moved* client attached with, not out of the client that
    /// ran the command. `-E` skips it.
    fn refresh_destination_environment(
        &self,
        state: &mut ServerState,
        client: &ClientContext,
        session_id: u32,
    ) {
        if self.no_environment {
            return;
        }
        let Ok(target) =
            state.resolve_target_client(self.client.as_deref(), client.tty_name.as_deref())
        else {
            return;
        };
        let environment = state.client_environment(&target.name);
        if environment.is_empty() {
            return;
        }
        state.update_session_environment(&format!("${session_id}"), &environment);
    }

    /// The `-T`, `-n`, `-p` and `-l` forms, which act on the target client's
    /// own state rather than on a `-t` session.
    fn run_on_target_client(
        self,
        state: &mut ServerState,
        client: &ClientContext,
    ) -> CommandResult {
        let target = match state
            .resolve_target_client(self.client.as_deref(), client.tty_name.as_deref())
        {
            Ok(target) => target,
            Err(ClientActionResult::NoCurrentClient) => {
                return CommandResult::err("no current client\n")
            }
            Err(_) => {
                return CommandResult::err(format!(
                    "can't find client: {}\n",
                    self.client.as_deref().unwrap_or_default()
                ))
            }
        };
        if self.toggle_read_only {
            let _ =
                state.toggle_client_read_only(self.client.as_deref(), client.tty_name.as_deref());
        }
        if let Some(table) = self.key_table.as_deref() {
            if !state.key_table_exists(table) {
                return CommandResult::err(format!("table {table} doesn't exist\n"));
            }
            state.set_client_key_table(&target.name, table);
            return CommandResult::ok("");
        }
        let order = match super::list_sort_order(self.order.as_deref()) {
            Ok(order) => order,
            Err(error) => return error,
        };
        let session_id = if self.last {
            match target
                .last_session_id
                .filter(|id| state.session_by_id(*id).is_some())
            {
                Some(id) => id,
                None => return CommandResult::err("can't find last session\n"),
            }
        } else {
            // tmux's session tree is keyed by name, so an unsorted cycle walks
            // the sessions in name order.
            let mut order_list: Vec<&Session> = state.sessions().iter().collect();
            order_list.sort_by(|left, right| left.name.cmp(&right.name));
            super::apply_list_sort(
                &mut order_list,
                order,
                self.toggle_read_only,
                |key, left, right| match key {
                    super::ListSortOrder::Index => left.id.cmp(&right.id),
                    super::ListSortOrder::Creation => {
                        left.created_epoch.cmp(&right.created_epoch)
                    }
                    super::ListSortOrder::Activity => {
                        right.activity_micros.cmp(&left.activity_micros)
                    }
                    super::ListSortOrder::Name => left.name.cmp(&right.name),
                    _ => std::cmp::Ordering::Equal,
                },
                |session| session.name.clone(),
            );
            let Some(position) = order_list
                .iter()
                .position(|session| session.id == target.session_id)
            else {
                return CommandResult::err(if self.next {
                    "can't find next session\n"
                } else {
                    "can't find previous session\n"
                });
            };
            let count = order_list.len();
            let next = if self.next {
                (position + 1) % count
            } else {
                (position + count - 1) % count
            };
            order_list[next].id
        };
        self.refresh_destination_environment(state, client, session_id);
        match state.switch_client(
            Some(&target.name),
            client.tty_name.as_deref(),
            session_id,
        ) {
            ClientActionResult::Queued => CommandResult::ok(""),
            ClientActionResult::NoCurrentClient => CommandResult::err("no current client\n"),
            ClientActionResult::TargetNotFound => CommandResult::err(format!(
                "can't find client: {}\n",
                self.client.as_deref().unwrap_or_default()
            )),
        }
    }
}

/// `display-message [-aCIlNpv] [-c target-client] [-d delay] [-F format]
/// [-t target-pane] [message]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct DisplayMessage {
    /// `-a`: print the target's whole format tree instead of a message.
    dump: bool,
    /// `-I`: feed the client's standard input into the target pane.
    feed_input: bool,
    /// `-l`: print the message literally, without expanding it.
    literal: bool,
    /// `-p`: print the message instead of showing it on a client.
    print: bool,
    /// `-v`: write the format engine's expansion trace to the command client.
    verbose: bool,
    /// `-c`: the client the message is shown on.
    client: Option<String>,
    /// `-d`: how long the message stays up.
    delay: Option<String>,
    /// `-F`: the message, when it is not given as an operand.
    format: Option<String>,
    /// `-t`: the pane the message is expanded against.
    target: Option<String>,
    message: Option<String>,
}

impl DisplayMessage {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            dump: args.has('a'),
            feed_input: args.has('I'),
            literal: args.has('l'),
            print: args.has('p'),
            verbose: args.has('v'),
            client: args.value('c').map(str::to_string),
            delay: args.value('d').map(str::to_string),
            format: args.value('F').map(str::to_string),
            target: args.value('t').map(str::to_string),
            message: args.positionals().first().cloned(),
        })
    }

    fn run(
        self,
        st: &mut ServerState,
        agents: &PaneAgents,
        context: &ClientContext,
    ) -> CommandResult {
        let target = self.target.clone().or_else(|| current_target(st));
        if target.is_none() && (!self.print || self.feed_input) {
            return CommandResult::err("can't establish current session\n");
        }
        // display-message uses tmux's can-fail target lookup: an unresolvable target
        // does not fail the command, it formats against however far the lookup got
        // (the named session's current window, say) or against nothing at all.
        let resolved = target
            .as_deref()
            .and_then(|target| st.resolve_or_residual(target));
        if self.feed_input {
            let resolved = match resolved {
                Some(resolved) => resolved,
                None => {
                    let target = target.as_deref().unwrap_or_default();
                    return CommandResult::err(format!("can't find session: {target}\n"));
                }
            };
            let pane =
                &mut st.window_mut(resolved.session, resolved.window).panes[resolved.pane].pane;
            if !pane.is_empty() {
                return CommandResult::err("pane is not empty\n");
            }
            return match context.input_file.as_ref() {
                Some(Ok(data)) => {
                    pane.feed(data);
                    CommandResult::ok("")
                }
                Some(Err(error)) => CommandResult::err(format!(
                    "{}: -\n",
                    io_error_message(&io::Error::from_raw_os_error(*error))
                )),
                None => CommandResult::ok(""),
            };
        }
        if self.message.is_some() && self.format.is_some() {
            return CommandResult::err("only one of -F or argument must be given\n");
        }
        // tmux reads `-d` here, before it has anything to say, so a delay that
        // is not a number in range fails even when nothing would be displayed.
        let delay_ms = match self.delay.as_deref() {
            Some(value) => match parse_delay(value) {
                Ok(delay) => Some(delay),
                Err(cause) => return CommandResult::err(format!("delay {cause}\n")),
            },
            None => None,
        };
        let message = self
            .message
            .as_deref()
            .or(self.format.as_deref())
            .unwrap_or(DISPLAY_MESSAGE_TEMPLATE);
        // `-l` prints the message as it stands and a message that names no
        // variable reads nothing, so in both cases the table below is built
        // for a lookup that never comes.
        let reads_vars = self.dump || (!self.literal && format::reads_vars(message));
        // Honor the *resolved* pane (e.g. `-t sess:win.{top}`), not just the window's
        // active pane, so pane-scoped variables reflect the target.
        let mut vars = match resolved.filter(|_| reads_vars) {
            Some(resolved) => vars_full(
                st,
                &st.sessions()[resolved.session],
                resolved.window,
                resolved.pane,
                agents,
                st.marked_pane(),
            ),
            None if reads_vars => Vars::new(),
            None => Vars::empty(),
        };
        if reads_vars {
            set_current_client_vars(
                st,
                context,
                resolved.map(|resolved| st.sessions()[resolved.session].id),
                self.client.as_deref(),
                &mut vars,
            );
            st.seed_format_environment(
                &mut vars,
                resolved.and_then(|resolved| st.sessions().get(resolved.session)),
            );
            // `-a` walks the format tree itself, which options are no part of:
            // tmux reaches an option only when a lookup misses the tree.
            if self.dump {
                let mut out = String::new();
                for (name, value) in vars.entries() {
                    out.push_str(&format!("{name}={value}\n"));
                }
                return CommandResult::ok(out);
            }
            if let Some(target) = target.as_deref() {
                if let Ok(entries) = st.format_option_entries(target) {
                    for (name, value) in entries {
                        vars.set(name.to_string(), value);
                    }
                }
            }
        }
        let loops = resolved.map(|resolved| TreeLoops {
            st,
            session: resolved.session,
            window: resolved.window,
            agents,
        });
        let (expanded, trace) = if self.literal {
            (message.to_string(), String::new())
        } else if self.verbose {
            format::expand_time_with_jobs_verbose(
                message,
                &vars,
                loops.as_ref().map(|loops| loops as &dyn format::ScopedLoopSource),
                command_jobs(st),
                Some(&ServerFormatTree(st)),
            )
        } else {
            (
                format::expand_time_with_jobs(
                    message,
                    &vars,
                    loops.as_ref().map(|loops| loops as &dyn format::ScopedLoopSource),
                    command_jobs(st),
                    Some(&ServerFormatTree(st)),
                ),
                String::new(),
            )
        };
        let mut out = String::new();
        // `-v` asks tmux's format engine to write its expansion trace to the
        // command client. Literal mode bypasses the format engine, so `-l -v`
        // produces no trace (and only prints the literal when `-p` is present).
        if self.verbose && !self.literal {
            out.push_str(&trace);
        }
        if self.print {
            out.push_str(&expanded);
            out.push('\n');
        } else {
            let duration_ms = match delay_ms {
                Some(delay) => delay,
                None => st
                    .option_for_target(target.as_deref().unwrap_or_default(), "display-time")
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(750),
            };
            let Some(resolved) = resolved else {
                return CommandResult::ok(out);
            };
            let session_id = st.sessions()[resolved.session].id;
            match st.send_client_message(
                self.client.as_deref(),
                context.tty_name.as_deref(),
                session_id,
                ClientMessage {
                    text: expanded.clone(),
                    duration_ms,
                    bell: false,
                },
            ) {
                ClientMessageResult::CurrentControl => {
                    out.push_str(&format!("%message {expanded}\n"));
                }
                ClientMessageResult::Queued | ClientMessageResult::NoClient => {}
                ClientMessageResult::TargetNotFound => {
                    let target = self.client.as_deref().unwrap_or_default();
                    return CommandResult::err(format!("can't find client: {target}\n"));
                }
            }
        }
        CommandResult::ok(out)
    }
}

/// tmux's `args_strtonum` against `strtonum(value, 0, UINT_MAX)`: the failure is
/// reported as the reason the conversion gave, not as the value that failed.
fn parse_delay(value: &str) -> Result<u64, &'static str> {
    match value.parse::<i64>() {
        Ok(delay) if delay < 0 => Err("too small"),
        Ok(delay) if delay > i64::from(u32::MAX) => Err("too large"),
        Ok(delay) => Ok(delay as u64),
        Err(_) => Err("invalid"),
    }
}

/// The `-T` shared by `show-prompt-history` and `clear-prompt-history`: one of
/// tmux's four prompt types, or every one of them when it is absent.
fn prompt_history_type(prompt_type: Option<&str>) -> Result<Option<&str>, CommandResult> {
    if prompt_type
        .is_some_and(|value| !matches!(value, "command" | "search" | "target" | "window-target"))
    {
        return Err(CommandResult::err(format!(
            "invalid type: {}\n",
            prompt_type.unwrap_or_default()
        )));
    }
    Ok(prompt_type)
}

/// `show-prompt-history [-T prompt-type]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct ShowPromptHistory {
    /// `-T`: the prompt type to show; all of them otherwise.
    prompt_type: Option<String>,
}

impl ShowPromptHistory {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            prompt_type: args.value('T').map(str::to_string),
        })
    }

    fn run(self, st: &ServerState) -> CommandResult {
        let prompt_type = match prompt_history_type(self.prompt_type.as_deref()) {
            Ok(prompt_type) => prompt_type,
            Err(error) => return error,
        };
        let types: Vec<&str> = prompt_type
            .map(|prompt_type| vec![prompt_type])
            .unwrap_or_else(|| vec!["command", "search", "target", "window-target"]);
        let mut output = String::new();
        for prompt_type in types {
            output.push_str(&format!("History for {prompt_type}:\n"));
            output.push('\n');
            let history = st.prompt_history(prompt_type);
            if !history.is_empty() {
                for (index, value) in history.iter().enumerate() {
                    output.push_str(&format!("{}: {value}\n", index + 1));
                }
            }
            output.push('\n');
        }
        CommandResult::ok(output)
    }
}

/// `clear-prompt-history [-T prompt-type]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct ClearPromptHistory {
    /// `-T`: the prompt type to clear; all of them otherwise.
    prompt_type: Option<String>,
}

impl ClearPromptHistory {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            prompt_type: args.value('T').map(str::to_string),
        })
    }

    fn run(self, st: &mut ServerState) -> CommandResult {
        let prompt_type = match prompt_history_type(self.prompt_type.as_deref()) {
            Ok(prompt_type) => prompt_type,
            Err(error) => return error,
        };
        st.clear_prompt_history(prompt_type);
        CommandResult::ok("")
    }
}

/// The pane an overlay command's `-t` names, which anchors its `-x`/`-y`.
fn overlay_target_pane(state: &ServerState, target: Option<&str>) -> Option<u32> {
    let target = target
        .map(str::to_string)
        .or_else(|| current_target(state))?;
    let resolved = state.resolve(&target)?;
    Some(state.window(resolved.session, resolved.window).panes[resolved.pane].id)
}

/// The pointer position of the key event an overlay command ran from, which
/// tmux publishes as `popup_mouse_x`/`popup_mouse_y`.
fn overlay_mouse(client: &ClientContext) -> Option<(u16, u16)> {
    client
        .mouse
        .as_ref()
        .map(|event| (event.position.x, event.position.y))
}
