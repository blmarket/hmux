use super::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(in crate::server) enum Command {
    New,
    Split,
    Kill,
    Select,
    Last,
    Swap,
    Move,
    Join,
    Break,
    Respawn,
    Resize,
    ResizeWindow,
    RotateWindow,
    SelectLayout,
    NextLayout,
    PreviousLayout,
    ClearHistory,
    Pipe,
    Capture,
    List,
    CopyMode,
}

impl Command {
    pub(super) fn execute(
        self,
        args: &[String],
        context: &mut CommandContext<'_>,
    ) -> CommandResult {
        let st = &mut *context.state;
        match self {
            Self::New => new_pane(args, st, context.client),
            Self::Split => split_window(args, st, context.client),
            Self::Kill => kill_pane(args, st),
            Self::Select => select_pane(args, st, context.client),
            Self::Last => last_pane_cmd(args, st),
            Self::Swap => swap_pane(args, st),
            Self::Move | Self::Join => move_pane(args, st),
            Self::Break => break_pane(args, st),
            Self::Respawn => respawn_pane(args, st),
            Self::Resize => resize_pane(args, st),
            Self::ResizeWindow => resize_window(args, st),
            Self::RotateWindow => rotate_window(args, st),
            Self::SelectLayout => select_layout(args, st),
            Self::NextLayout => cycle_layout(args, st, true),
            Self::PreviousLayout => cycle_layout(args, st, false),
            Self::ClearHistory => clear_history(args, st),
            Self::Pipe => pipe_pane(args, st),
            Self::Capture => capture_pane(args, st, context.agents),
            Self::List => list_panes(args, st, context.agents),
            Self::CopyMode => {
                let target = flag_value(args, "-t")
                    .map(str::to_string)
                    .or_else(|| current_session(st));
                match target {
                    Some(target) => {
                        if has_flag(args, "-q") {
                            return match st.set_pane_mode(&target, None) {
                                Ok(()) => CommandResult::ok(""),
                                Err(_) => {
                                    CommandResult::err(format!("can't find pane: {target}\n"))
                                }
                            };
                        }
                        let source = flag_value(args, "-s");
                        if st
                            .set_copy_mode(
                                &target,
                                source,
                                has_flag(args, "-e"),
                                has_flag(args, "-H"),
                            )
                            .is_err()
                        {
                            let missing = source.unwrap_or(&target);
                            return CommandResult::err(format!("can't find pane: {missing}\n"));
                        }
                        let vi = st
                            .window_options(&target)
                            .ok()
                            .and_then(|view| view.get("mode-keys"))
                            .unwrap_or("emacs")
                            == "vi";
                        let separators = st
                            .session_options(&target)
                            .ok()
                            .and_then(|view| view.get("word-separators"))
                            .unwrap_or(" !\"#$%&'()*+,-./:;<=>?@[\\]^`{|}~")
                            .to_string();
                        if has_flag(args, "-u") {
                            let _ = st.copy_mode_command(&target, "page-up", vi, &separators);
                        }
                        if has_flag(args, "-d") {
                            let _ = st.copy_mode_command(&target, "page-down", vi, &separators);
                        }
                        CommandResult::ok("")
                    }
                    None => CommandResult::err("can't establish current session\n"),
                }
            }
        }
    }
}

#[cfg(test)]
pub(super) const ALL: &[Command] = &[
    Command::New,
    Command::Split,
    Command::Kill,
    Command::Select,
    Command::Last,
    Command::Swap,
    Command::Move,
    Command::Join,
    Command::Break,
    Command::Respawn,
    Command::Resize,
    Command::ResizeWindow,
    Command::RotateWindow,
    Command::SelectLayout,
    Command::NextLayout,
    Command::PreviousLayout,
    Command::ClearHistory,
    Command::Pipe,
    Command::Capture,
    Command::List,
    Command::CopyMode,
];
