use super::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(in crate::server) enum Command {
    List,
    Detach,
    Switch,
    Refresh,
    Suspend,
    Lock,
    Prompt,
    ConfirmBefore,
    DisplayMessage,
    DisplayMenu,
    DisplayPopup,
    DisplayPanes,
    ChooseTree,
    ChooseClient,
    ChooseBuffer,
    ClockMode,
    CustomizeMode,
    ShowPromptHistory,
    ClearPromptHistory,
}

impl Command {
    pub(super) fn execute(
        self,
        args: &[String],
        context: &mut CommandContext<'_>,
    ) -> CommandResult {
        match self {
            Self::List => list_clients(args, context.state, context.agents),
            Self::Detach => detach_client(args, context.state, context.client),
            Self::Refresh => refresh_client(args, context.state, context.client),
            Self::Switch => switch_client(args, context.state, context.client),
            Self::Suspend => suspend_client(args, context.state, context.client),
            Self::Prompt => CommandResult::err("no current client\n"),
            Self::ConfirmBefore => confirm_before(args, context.state, context.client),
            Self::DisplayMenu => display_menu(args, context.state, context.agents, context.client),
            Self::DisplayPopup => display_popup(args, context.state, context.client),
            Self::DisplayPanes => display_panes(args, context.state, context.client),
            Self::Lock => lock_client(args, context.state, context.client),
            Self::DisplayMessage => {
                display_message(args, context.state, context.agents, context.client)
            }
            Self::ChooseTree => choose_tree(args, context.state, context.agents),
            Self::ChooseClient => choose_client(args, context.state, context.agents),
            Self::ChooseBuffer => choose_buffer(args, context.state),
            Self::ClockMode => clock_mode(args, context.state),
            Self::CustomizeMode => customize_mode(args, context.state),
            Self::ShowPromptHistory => show_prompt_history(args, context.state),
            Self::ClearPromptHistory => clear_prompt_history(args, context.state),
        }
    }
}

fn confirm_before(args: &[String], state: &ServerState, client: &ClientContext) -> CommandResult {
    let raw = trailing_command(args, &["-c", "-p", "-t"]);
    if raw.is_empty() {
        return CommandResult::err("command confirm-before: too few arguments (need at least 1)\n");
    }
    let command = if let [line] = raw.as_slice() {
        command_string_groups(line)
            .ok()
            .map(|groups| {
                let mut command = Vec::new();
                for group in groups {
                    if !command.is_empty() {
                        command.push(";".to_string());
                    }
                    command.extend(group);
                }
                command
            })
            .unwrap_or_else(|| vec![(*line).to_string()])
    } else {
        raw.into_iter().map(str::to_string).collect()
    };
    let confirm_key = match flag_value(args, "-c") {
        Some(value)
            if value.len() == 1 && value.as_bytes()[0] > 31 && value.as_bytes()[0] < 127 =>
        {
            value.as_bytes()[0]
        }
        Some(_) => return CommandResult::err("invalid confirm key\n"),
        None => b'y',
    };
    let prompt = flag_value(args, "-p").map_or_else(
        || {
            let name = command.first().map(String::as_str).unwrap_or_default();
            format!("Confirm '{name}'? ({}/n) ", confirm_key as char)
        },
        |prompt| format!("{prompt} "),
    );
    let target = flag_value(args, "-t");
    overlay_result(
        state.confirm_client(
            target,
            client.tty_name.as_deref(),
            prompt,
            command,
            confirm_key,
            has_flag(args, "-y"),
            client.interaction_reply.clone(),
        ),
        target,
    )
}

fn mode_target(args: &[String], state: &ServerState) -> Option<String> {
    flag_value(args, "-t")
        .map(str::to_string)
        .or_else(|| current_target(state))
}

fn enter_mode(args: &[String], state: &mut ServerState, view: ModeView) -> CommandResult {
    let Some(target) = mode_target(args, state) else {
        return CommandResult::err("no current session\n");
    };
    match state.enter_mode_view(&target, view) {
        Ok(()) => CommandResult::ok(""),
        Err(_) => CommandResult::err(format!("{}\n", state.pane_target_error(&target))),
    }
}

fn validate_mode_target(args: &[String], state: &ServerState) -> Result<(), CommandResult> {
    let Some(target) = mode_target(args, state) else {
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

fn template_command(template: &str, value: &str) -> Vec<String> {
    let expanded = template.replace("%%", value);
    command_string_groups(&expanded)
        .ok()
        .map(|groups| {
            let mut command = Vec::new();
            for group in groups {
                if !command.is_empty() {
                    command.push(";".to_string());
                }
                command.extend(group);
            }
            command
        })
        .unwrap_or_default()
}

/// The `-F`, `-f` and `-O` every `choose-*` command shares, as tmux's
/// `mode_tree_start` takes them.
struct ChooseOptions<'a> {
    format: Option<&'a str>,
    filter: Option<&'a str>,
    /// `-O`, resolved through the same table the `list-*` commands use. `None`
    /// means no `-O` at all, which leaves each mode on its own first order.
    order: Option<ListSortOrder>,
    /// `-r`: negate the comparison the sort order made.
    reversed: bool,
}

impl<'a> ChooseOptions<'a> {
    /// tmux's `cmd_choose_tree_exec` runs `-O` through
    /// `sort_order_from_string` before it enters any mode, so a name that
    /// table does not know fails the command rather than picking an order.
    fn parse(args: &'a [String]) -> Result<Self, CommandResult> {
        let (order, reversed) = list_sort_criteria(args)?;
        Ok(Self {
            format: flag_value(args, "-F"),
            filter: flag_value(args, "-f"),
            order,
            reversed,
        })
    }

    /// Whether this row survives `-f`.
    fn keep(&self, state: &ServerState, vars: &format::Vars) -> bool {
        self.filter.is_none_or(|filter| {
            format::is_true(&super::expand_command_format(state, filter, vars, None))
        })
    }

    /// The row's text: `-F` when one was given, else the command's own.
    fn label(&self, state: &ServerState, vars: &format::Vars, default: String) -> String {
        match self.format {
            Some(format) => super::expand_command_format(state, format, vars, None),
            None => default,
        }
    }

    /// The order this mode sorts on: `-O` when it was given, and otherwise the
    /// mode's own first order, which is what tmux's per-mode `sortcb` writes
    /// over an unset `sort_crit->order`.
    fn order(&self, default: ListSortOrder) -> Option<ListSortOrder> {
        Some(self.order.unwrap_or(default))
    }

    /// Sort `rows` the way tmux's `sort_qsort` sorts a flat mode: the `-O`
    /// comparison first, ties broken by the row's name, and `-r` negating the
    /// combined result rather than reversing the sorted list.
    fn sort<T>(
        &self,
        rows: &mut [T],
        default: ListSortOrder,
        compare: impl Fn(ListSortOrder, &T, &T) -> std::cmp::Ordering,
        name: impl Fn(&T) -> String,
    ) {
        apply_list_sort(rows, self.order(default), self.reversed, compare, name);
    }

    /// [`Self::sort`] for the tree, whose rows are two levels rather than one.
    ///
    /// tmux's `window_tree_build` sorts the sessions against each other and
    /// then each session's own windows within it, so a window never leaves the
    /// session that built it however the sessions are ordered. `-r` negates
    /// both levels, which keeps a session ahead of its windows.
    fn sort_tree(&self, groups: &mut [TreeGroup]) {
        // tmux's `window_tree_sort`: without `-O` the tree takes the first of
        // its own sequence, which is by index.
        self.sort(
            groups,
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

fn choose_tree(args: &[String], state: &mut ServerState, agents: &PaneAgents) -> CommandResult {
    let template = positionals(args, &["-F", "-f", "-K", "-O", "-t"])
        .first()
        .copied();
    let options = match ChooseOptions::parse(args) {
        Ok(options) => options,
        Err(error) => return error,
    };
    // `-s` lists only sessions and `-w` only windows; without either, a session
    // is followed by its own windows.
    let sessions_only = has_flag(args, "-s");
    let windows_only = has_flag(args, "-w");
    let marked = state.marked_pane();
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
        if !windows_only && options.keep(state, &session_vars) {
            group.session_row = Some(ChooseRow {
                item: ModeItem {
                    label: options.label(
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
                },
                vars: session_vars,
            });
        }
        if sessions_only {
            groups.push(group);
            continue;
        }
        for (position, link) in session.windows.iter().enumerate() {
            let window = state.session_window(session, position);
            let target = format!("{}:{}", session.name, link.index);
            let window_vars = super::vars_for(state, session, position, agents, marked);
            if !options.keep(state, &window_vars) {
                continue;
            }
            group.windows.push(TreeWindow {
                id: window.id,
                activity: window.activity_epoch,
                row: ChooseRow {
                    item: ModeItem {
                        label: options.label(
                            state,
                            &window_vars,
                            format!(
                                "  {}: {} ({} panes)",
                                link.index,
                                window.name,
                                window.panes.len()
                            ),
                        ),
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
                        depth: 0,
                        expanded: None,
                    },
                    vars: window_vars,
                },
            });
        }
        groups.push(group);
    }
    options.sort_tree(&mut groups);
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
    view.preview = !has_flag(args, "-N");
    enter_mode(args, state, view)
}

fn choose_client(args: &[String], state: &mut ServerState, agents: &PaneAgents) -> CommandResult {
    if let Err(error) = validate_mode_target(args, state) {
        return error;
    }
    let template = positionals(args, &["-F", "-f", "-K", "-O", "-t"])
        .first()
        .copied()
        .unwrap_or("detach-client -t '%%'");
    let options = match ChooseOptions::parse(args) {
        Ok(options) => options,
        Err(error) => return error,
    };
    let mut rows = Vec::new();
    for client in state.client_snapshots() {
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
                },
                vars,
            },
        });
    }
    // tmux's `window_client_sort`: without `-O` the mode sorts by name.
    options.sort(
        &mut rows,
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
        args,
        state,
        ModeView::list(ModeKind::Client, "Clients", items),
    )
}

fn choose_buffer(args: &[String], state: &mut ServerState) -> CommandResult {
    if let Err(error) = validate_mode_target(args, state) {
        return error;
    }
    let template = positionals(args, &["-F", "-f", "-K", "-O", "-t"])
        .first()
        .copied()
        .unwrap_or("paste-buffer -p -b '%%'");
    let options = match ChooseOptions::parse(args) {
        Ok(options) => options,
        Err(error) => return error,
    };
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
                },
                vars,
            },
        });
    }
    // tmux's `window_buffer_sort`: without `-O` the mode sorts by creation.
    options.sort(
        &mut rows,
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
        args,
        state,
        ModeView::list(ModeKind::Buffer, "Buffers", items),
    )
}

fn customize_mode(args: &[String], state: &mut ServerState) -> CommandResult {
    let Some(target) = mode_target(args, state) else {
        return CommandResult::err("no current session\n");
    };
    // `customize-mode` takes no `-O`, so this only ever carries `-F` and `-f`.
    let options = match ChooseOptions::parse(args) {
        Ok(options) => options,
        Err(error) => return error,
    };
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
            expanded: Some(true),
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
                .set("option_is_array", "0")
                .set("is_option", "1")
                .set("is_key", "0");
            if !options.keep(state, &vars) {
                continue;
            }
            items.push(ModeItem {
                label: options.label(state, &vars, format!("{} {}", entry.name, entry.value)),
                command: Vec::new(),
                prompt_target: None,
                edit: Some(ModeEdit::Option {
                    name: entry.name,
                    value: entry.value,
                }),
                tagged: false,
                preview_target: None,
                depth: 1,
                expanded: None,
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
                binding.command.clone(),
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
        });
    }
    let mut view = ModeView::list(ModeKind::Customize, "Customize", items);
    view.preview = !has_flag(args, "-N");
    enter_mode(args, state, view)
}

fn clock_mode(args: &[String], state: &mut ServerState) -> CommandResult {
    enter_mode(args, state, ModeView::clock())
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

fn display_menu(
    args: &[String],
    state: &ServerState,
    agents: &PaneAgents,
    client: &ClientContext,
) -> CommandResult {
    let values = positionals(
        args,
        &["-b", "-c", "-C", "-H", "-s", "-S", "-t", "-T", "-x", "-y"],
    );
    // Both halves of an item are formats — tmux expands them when the menu is
    // built, against the target the menu was asked for.
    let vars = flag_value(args, "-t")
        .map(str::to_string)
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
    let mut items = Vec::new();
    let mut index = 0;
    while index < values.len() {
        let label = values[index];
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
        let key = values[index];
        let command = values[index + 1];
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
    let selected = flag_value(args, "-C")
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    let request = OverlayRequest::Menu(MenuRequest {
        title: flag_value(args, "-T").unwrap_or("").to_string(),
        items,
        selected,
        x: flag_value(args, "-x").map(str::to_string),
        y: flag_value(args, "-y").map(str::to_string),
    });
    let target = flag_value(args, "-c");
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

fn display_popup(args: &[String], state: &ServerState, client: &ClientContext) -> CommandResult {
    let target = flag_value(args, "-c");
    if has_flag(args, "-C") {
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
    let argv = trailing_command(
        args,
        &[
            "-b", "-c", "-d", "-e", "-h", "-s", "-S", "-t", "-T", "-w", "-x", "-y",
        ],
    )
    .into_iter()
    .map(str::to_string)
    .collect();
    let exit_flags = args.iter().filter(|word| word.as_str() == "-E").count();
    let overrides = flag_values(args, "-e");
    let request = OverlayRequest::Popup(PopupRequest {
        on_close: Vec::new(),
        on_close_remove: None,
        title: flag_value(args, "-T").unwrap_or("").to_string(),
        argv,
        environment: state.spawn_environment(
            current_target(state).as_deref(),
            &client.environment,
            &overrides,
        ),
        cwd: flag_value(args, "-d")
            .map(PathBuf::from)
            .or_else(|| client.cwd.clone()),
        width: flag_value(args, "-w").map(str::to_string),
        height: flag_value(args, "-h").map(str::to_string),
        x: flag_value(args, "-x").map(str::to_string),
        y: flag_value(args, "-y").map(str::to_string),
        close_on_exit: exit_flags == 1,
        close_on_success: exit_flags >= 2,
        close_on_key: has_flag(args, "-k"),
        border: !has_flag(args, "-B"),
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

fn display_panes(args: &[String], state: &ServerState, client: &ClientContext) -> CommandResult {
    let duration_ms = flag_value(args, "-d")
        .and_then(|value| value.parse().ok())
        .or_else(|| {
            current_target(state).and_then(|target| {
                state
                    .option_for_target(&target, "display-panes-time")
                    .and_then(|value| value.parse().ok())
            })
        })
        .unwrap_or(1000);
    let command = positionals(args, &["-d", "-t"])
        .first()
        .map(|template| template_command(template, "%%"))
        .unwrap_or_default();
    let target = flag_value(args, "-t");
    overlay_result(
        state.overlay_client(
            target,
            client.tty_name.as_deref(),
            OverlayRequest::DisplayPanes {
                duration_ms,
                command,
                accept_input: !has_flag(args, "-N"),
            },
            client.interaction_reply.clone(),
        ),
        target,
    )
}

#[cfg(test)]
pub(super) const ALL: &[Command] = &[
    Command::List,
    Command::Detach,
    Command::Switch,
    Command::Refresh,
    Command::Suspend,
    Command::Lock,
    Command::Prompt,
    Command::ConfirmBefore,
    Command::DisplayMessage,
    Command::DisplayMenu,
    Command::DisplayPopup,
    Command::DisplayPanes,
    Command::ChooseTree,
    Command::ChooseClient,
    Command::ChooseBuffer,
    Command::ClockMode,
    Command::CustomizeMode,
    Command::ShowPromptHistory,
    Command::ClearPromptHistory,
];

fn list_clients(args: &[String], state: &ServerState, agents: &PaneAgents) -> CommandResult {
    const DEFAULT_FORMAT: &str = "#{client_name}: #{session_name} [#{client_width}x#{client_height} #{client_termname}] #{?client_flags,(,}#{client_flags}#{?client_flags,),}";
    let template = flag_value(args, "-F").unwrap_or(DEFAULT_FORMAT);
    let requested_session = flag_value(args, "-t");
    let target_session = requested_session.and_then(|target| state.resolve_session(target));
    if requested_session.is_some() && target_session.is_none() {
        return CommandResult::err(format!(
            "can't find session: {}\n",
            requested_session.unwrap_or_default()
        ));
    }
    let filter = flag_value(args, "-f");
    let (sort_order, reversed) = match super::list_sort_criteria(args) {
        Ok(criteria) => criteria,
        Err(error) => return error,
    };
    let mut clients = state.client_snapshots();
    super::apply_list_sort(
        &mut clients,
        sort_order,
        reversed,
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
    for client in clients {
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
        let Some(vars) = client_vars(state, agents, &client) else {
            continue;
        };
        if filter.is_some_and(|filter| !format::is_true(&format::expand(filter, &vars))) {
            continue;
        }
        output.push_str(&format::expand(template, &vars));
        output.push('\n');
    }
    CommandResult::ok(output)
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
    // hmux never learns the terminal's pixel size; tmux without one reports
    // zero cell dimensions the same way.
    .set("client_cell_width", "0")
    .set("client_cell_height", "0")
    .set("client_termfeatures", client.termfeatures.clone())
    // The secondary-DA terminal type: empty until the terminal answers a
    // query the attach flow does not send.
    .set("client_termtype", "")
    .set("client_written", client.written.to_string())
    // hmux never discards output the way tmux's backoff does.
    .set("client_discarded", "0")
    .set("client_last_session", last_session.unwrap_or_default());
}

/// The format variables one attached client answers to, shared by
/// `list-clients` and `choose-client`. `None` when the client's session has
/// gone away underneath it.
fn client_vars(
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

fn detach_client(args: &[String], state: &ServerState, client: &ClientContext) -> CommandResult {
    let target = flag_value(args, "-t");
    overlay_result(
        state.detach_client(target, client.tty_name.as_deref()),
        target,
    )
}

fn suspend_client(args: &[String], state: &ServerState, client: &ClientContext) -> CommandResult {
    let target = flag_value(args, "-t");
    overlay_result(
        state.suspend_client(target, client.tty_name.as_deref()),
        target,
    )
}

fn refresh_client(
    args: &[String],
    state: &mut ServerState,
    client: &ClientContext,
) -> CommandResult {
    let target = flag_value(args, "-t");
    // `-c` and the four pan directions are handled first and alone, as tmux's
    // `cmd_refresh_client_exec` returns straight after them.
    let pan = if has_flag(args, "-L") {
        Some(Some(WindowResizeAdjust::Left))
    } else if has_flag(args, "-R") {
        Some(Some(WindowResizeAdjust::Right))
    } else if has_flag(args, "-U") {
        Some(Some(WindowResizeAdjust::Up))
    } else if has_flag(args, "-D") {
        Some(Some(WindowResizeAdjust::Down))
    } else if has_flag(args, "-c") {
        Some(None)
    } else {
        None
    };
    if let Some(adjust) = pan {
        let adjustment = match positionals(args, &["-t", "-A", "-B", "-C", "-f", "-F", "-r"])
            .first()
            .copied()
            .unwrap_or("1")
            .parse::<u16>()
        {
            Ok(0) => return CommandResult::err("adjustment too small\n"),
            Ok(value) => value,
            Err(_) => return CommandResult::err("adjustment invalid\n"),
        };
        return overlay_result(
            state.pan_client(target, client.tty_name.as_deref(), adjust, adjustment),
            target,
        );
    }
    // `-F` is the historical spelling of `-f`.
    let flags = flag_values(args, "-f")
        .into_iter()
        .chain(flag_values(args, "-F"))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if !flags.is_empty() {
        let result = state.refresh_client_flags(target, client.tty_name.as_deref(), &flags);
        if result != ClientActionResult::Queued {
            return overlay_result(result, target);
        }
    }
    // `-l` asks the client's terminal for its selection. tmux keeps one
    // outstanding query per terminal, so a repeat inside the timeout is
    // dropped rather than queued.
    if has_flag(args, "-l") && state.begin_clipboard_query(target, client.tty_name.as_deref()) {
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

fn switch_client(
    args: &[String],
    state: &mut ServerState,
    client: &ClientContext,
) -> CommandResult {
    let Some(target_session) = flag_value(args, "-t") else {
        return CommandResult::err("no current client\n");
    };
    // tmux resolves a target naming a window or pane — including `=`, the
    // mouse's — as a pane target and makes that window current before moving
    // the client, which is what the default `MouseDown1Status` binding relies
    // on to turn the status line into a window switcher.
    let resolved_pane_target = (target_session == "=" || target_session.contains([':', '.', '%']))
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
    let toggled = if has_flag(args, "-r") {
        match state.toggle_client_read_only(flag_value(args, "-c"), client.tty_name.as_deref()) {
            Ok(name) => Some(name),
            Err(ClientActionResult::NoCurrentClient) => {
                return CommandResult::err("no current client\n");
            }
            Err(_) => return CommandResult::err("can't find client\n"),
        }
    } else {
        None
    };
    match state.switch_client(
        flag_value(args, "-c").or(toggled.as_deref()),
        client.tty_name.as_deref(),
        session_id,
    ) {
        ClientActionResult::Queued => CommandResult::ok(""),
        ClientActionResult::NoCurrentClient => CommandResult::err("no current client\n"),
        ClientActionResult::TargetNotFound => CommandResult::err("can't find client\n"),
    }
}
