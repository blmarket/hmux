use super::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(in crate::server) enum Command {
    Find,
    New,
    Kill,
    Rename,
    List,
    Select,
    Next,
    Previous,
    Last,
    Swap,
    Move,
    Link,
    Unlink,
    Respawn,
}

impl Command {
    pub(super) fn execute(
        self,
        args: &[String],
        context: &mut CommandContext<'_>,
    ) -> CommandResult {
        let st = &mut *context.state;
        match self {
            Self::Find => find_window(args, st),
            Self::New => new_window(args, st, context.client),
            Self::Kill if has_flag(args, "-a") => {
                target_command(args, st, ServerState::kill_other_windows)
            }
            Self::Kill => target_command(args, st, ServerState::kill_window),
            Self::Rename => rename_window(args, st),
            Self::List => list_windows(args, st, context.agents),
            Self::Select if has_flag(args, "-n") => {
                session_command(args, st, ServerState::next_window)
            }
            Self::Select if has_flag(args, "-p") => {
                session_command(args, st, ServerState::previous_window)
            }
            Self::Select if has_flag(args, "-l") => {
                session_command(args, st, ServerState::last_window)
            }
            Self::Select => target_command(args, st, ServerState::select_window),
            Self::Next if has_flag(args, "-a") => {
                session_command(args, st, ServerState::next_window_alert)
            }
            Self::Next => session_command(args, st, ServerState::next_window),
            Self::Previous if has_flag(args, "-a") => {
                session_command(args, st, ServerState::previous_window_alert)
            }
            Self::Previous => session_command(args, st, ServerState::previous_window),
            Self::Last => session_command(args, st, ServerState::last_window),
            Self::Swap => swap_window(args, st),
            Self::Move => move_window(args, st),
            Self::Link => link_window(args, st),
            Self::Unlink => unlink_window(args, st),
            Self::Respawn => respawn_window(args, st),
        }
    }
}

fn find_window(args: &[String], state: &mut ServerState) -> CommandResult {
    let Some(pattern) = positionals(args, &["-t"]).first().copied() else {
        return CommandResult::err("command find-window: too few arguments (need at least 1)\n");
    };
    let folded = pattern.to_lowercase();
    let Some(target) = flag_value(args, "-t")
        .map(str::to_string)
        .or_else(|| current_target(state))
    else {
        return CommandResult::err("no current session\n");
    };
    if state.resolve(&target).is_none() {
        return CommandResult::err(format!("{}\n", state.pane_target_error(&target)));
    }
    let mut items = Vec::new();
    for session in state.sessions() {
        for (position, link) in session.windows.iter().enumerate() {
            let window = state.session_window(session, position);
            if !window.name.to_lowercase().contains(&folded) {
                continue;
            }
            items.push(ModeItem {
                tagged: false,
                preview_target: None,
                depth: 0,
                expanded: None,
                label: format!("{}:{} {}", session.name, link.index, window.name),
                command: vec![
                    "select-window".to_string(),
                    "-t".to_string(),
                    format!("{}:{}", session.name, link.index),
                ],
                prompt_target: None,
                edit: None,
            });
        }
    }
    if items.is_empty() {
        return CommandResult::ok("");
    }
    match state.enter_mode_view(
        &target,
        ModeView::list(ModeKind::Tree, format!("Find: {pattern}"), items),
    ) {
        Ok(()) => CommandResult::ok(""),
        Err(_) => CommandResult::err(format!("{}\n", state.pane_target_error(&target))),
    }
}

#[cfg(test)]
pub(super) const ALL: &[Command] = &[
    Command::Find,
    Command::New,
    Command::Kill,
    Command::Rename,
    Command::List,
    Command::Select,
    Command::Next,
    Command::Previous,
    Command::Last,
    Command::Swap,
    Command::Move,
    Command::Link,
    Command::Unlink,
    Command::Respawn,
];
