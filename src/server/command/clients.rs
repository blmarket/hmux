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
            Self::DisplayMenu => display_menu(args, context.state, context.client),
            Self::DisplayPopup => display_popup(args, context.state, context.client),
            Self::DisplayPanes => display_panes(args, context.state, context.client),
            Self::Lock => lock_client(args, context.state, context.client),
            Self::DisplayMessage => {
                display_message(args, context.state, context.agents, context.client)
            }
            Self::ChooseTree => choose_tree(args, context.state),
            Self::ChooseClient => choose_client(args, context.state),
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
        .or_else(|| current_session(state))
}

fn enter_mode(args: &[String], state: &mut ServerState, view: ModeView) -> CommandResult {
    let Some(target) = mode_target(args, state) else {
        return CommandResult::err("no current session\n");
    };
    match state.enter_mode_view(&target, view) {
        Ok(()) => CommandResult::ok(""),
        Err(_) => CommandResult::err(format!("can't find pane: {target}\n")),
    }
}

fn validate_mode_target(args: &[String], state: &ServerState) -> Result<(), CommandResult> {
    let Some(target) = mode_target(args, state) else {
        return Err(CommandResult::err("no current session\n"));
    };
    if state.resolve(&target).is_none() {
        return Err(CommandResult::err(format!("can't find pane: {target}\n")));
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

fn choose_tree(args: &[String], state: &mut ServerState) -> CommandResult {
    let template = positionals(args, &["-F", "-f", "-K", "-O", "-t"])
        .first()
        .copied();
    let mut items = Vec::new();
    for session in state.sessions() {
        items.push(ModeItem {
            label: format!("{} ({} windows)", session.name, session.windows.len()),
            command: template_command(template.unwrap_or("switch-client -Zt '%%'"), &session.name),
            prompt_target: Some(format!("={}:", session.name)),
            edit: None,
        });
        for (position, link) in session.windows.iter().enumerate() {
            let window = state.session_window(session, position);
            let target = format!("{}:{}", session.name, link.index);
            items.push(ModeItem {
                label: format!(
                    "  {}: {} ({} panes)",
                    link.index,
                    window.name,
                    window.panes.len()
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
            });
        }
    }
    enter_mode(args, state, ModeView::list(ModeKind::Tree, "Tree", items))
}

fn choose_client(args: &[String], state: &mut ServerState) -> CommandResult {
    if let Err(error) = validate_mode_target(args, state) {
        return error;
    }
    let template = positionals(args, &["-F", "-f", "-K", "-O", "-t"])
        .first()
        .copied()
        .unwrap_or("detach-client -t '%%'");
    let items = state
        .attached_clients()
        .into_iter()
        .map(|client| ModeItem {
            label: format!(
                "{}: {}x{} {}",
                client.name, client.cols, client.rows, client.term
            ),
            command: template_command(template, &client.name),
            prompt_target: None,
            edit: None,
        })
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
    let items = state
        .buffers()
        .iter()
        .map(|(name, data)| {
            let preview = String::from_utf8_lossy(data).replace(['\n', '\r'], " ");
            ModeItem {
                label: format!("{name}: {} bytes: {}", data.len(), preview),
                command: template_command(template, name),
                prompt_target: None,
                edit: None,
            }
        })
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
    let mut items = match state.format_option_entries(&target) {
        Ok(entries) => entries
            .map(|(name, value)| ModeItem {
                label: format!("{name} {value}"),
                command: Vec::new(),
                prompt_target: None,
                edit: Some(ModeEdit::Option {
                    name: name.to_string(),
                    value: value.to_string(),
                }),
            })
            .collect::<Vec<_>>(),
        Err(_) => return CommandResult::err(format!("can't find pane: {target}\n")),
    };
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
        });
    }
    enter_mode(
        args,
        state,
        ModeView::list(ModeKind::Customize, "Customize", items),
    )
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

fn display_menu(args: &[String], state: &ServerState, client: &ClientContext) -> CommandResult {
    let values = positionals(
        args,
        &["-b", "-c", "-C", "-H", "-s", "-S", "-t", "-T", "-x", "-y"],
    );
    let items = values
        .chunks(3)
        .filter(|chunk| chunk.len() == 3)
        .map(|chunk| MenuItem {
            label: chunk[0].to_string(),
            key: chunk[1].to_string(),
            command: template_command(chunk[2], ""),
        })
        .collect::<Vec<_>>();
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
    let request = OverlayRequest::Popup(PopupRequest {
        title: flag_value(args, "-T").unwrap_or("").to_string(),
        argv,
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
            current_session(state).and_then(|target| {
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
    let mut output = String::new();
    for client in state.attached_clients() {
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
        let client_utf8 = client.flags.split(',').any(|flag| flag == "UTF-8");
        let mut vars = super::vars_for(state, session, session.active, agents, state.marked_pane());
        vars.set("client_name", client.name.clone())
            .set("client_tty", client.name)
            .set("client_termname", client.term)
            .set(
                "client_pid",
                client.pid.map(|pid| pid.to_string()).unwrap_or_default(),
            )
            .set("client_width", client.cols.to_string())
            .set("client_height", client.rows.to_string())
            .set("client_session", session.name.clone())
            .set("client_flags", client.flags)
            .set("client_readonly", if client.read_only { "1" } else { "0" })
            .set(
                "client_control_mode",
                if client.control_mode { "1" } else { "0" },
            )
            .set("client_utf8", if client_utf8 { "1" } else { "0" })
            .set("client_theme", client.theme);
        if filter.is_some_and(|filter| !format::is_true(&format::expand(filter, &vars))) {
            continue;
        }
        output.push_str(&format::expand(template, &vars));
        output.push('\n');
    }
    CommandResult::ok(output)
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

fn refresh_client(args: &[String], state: &ServerState, client: &ClientContext) -> CommandResult {
    let target = flag_value(args, "-t");
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

fn switch_client(args: &[String], state: &ServerState, client: &ClientContext) -> CommandResult {
    let Some(target_session) = flag_value(args, "-t") else {
        return CommandResult::err("no current client\n");
    };
    let Some(session_id) = state.session_id(target_session) else {
        return CommandResult::err(format!("can't find session: {target_session}\n"));
    };
    match state.switch_client(
        flag_value(args, "-c"),
        client.tty_name.as_deref(),
        session_id,
    ) {
        ClientActionResult::Queued => CommandResult::ok(""),
        ClientActionResult::NoCurrentClient => CommandResult::err("no current client\n"),
        ClientActionResult::TargetNotFound => CommandResult::err("can't find client\n"),
    }
}
