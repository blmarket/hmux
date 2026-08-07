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
            Self::Respawn => respawn_pane(args, st, context.client),
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
                    .or_else(|| current_target(st));
                match target {
                    Some(target) => {
                        if has_flag(args, "-q") {
                            return match st.set_pane_mode(&target, None) {
                                Ok(()) => CommandResult::ok(""),
                                Err(_) => CommandResult::err(format!(
                                    "{}\n",
                                    st.pane_target_error(&target)
                                )),
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
                            return CommandResult::err(format!(
                                "{}\n",
                                st.pane_target_error(missing)
                            ));
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
                        // `-M` starts a drag: the selection opens where the
                        // button went down, not where the pointer has already
                        // reached (tmux's `window_copy_start_drag`).
                        if has_flag(args, "-M") {
                            if let Some(position) = st
                                .command_mouse()
                                .and_then(|mouse| mouse.pane_last_position())
                            {
                                let _ = st.position_copy_cursor_from_mouse(
                                    &target, position.x, position.y, vi,
                                );
                                let _ = st.copy_mode_command(
                                    &target,
                                    "begin-selection",
                                    vi,
                                    &separators,
                                );
                                // tmux ends `window_copy_start_drag` with one
                                // drag update, so the pointer's current
                                // position is already selected.
                                if let Some(now) =
                                    st.command_mouse().map(|mouse| mouse.pane_position())
                                {
                                    st.drag_copy_selection_to_mouse(&target, now.x, now.y, vi);
                                }
                            }
                        }
                        if has_flag(args, "-u") {
                            let _ = st.copy_mode_command(&target, "page-up", vi, &separators);
                        }
                        // `-S` drags the scrollbar's slider, which carries its
                        // own grab offset from where the drag took hold.
                        if has_flag(args, "-S") {
                            if let Some((row, grab)) = st.command_mouse().map(|mouse| {
                                (
                                    mouse.position.y,
                                    mouse.target.as_ref().and_then(|t| t.slider_offset),
                                )
                            }) {
                                let _ = st.scroll_copy_to_mouse(
                                    &target,
                                    row,
                                    grab,
                                    vi,
                                    has_flag(args, "-e"),
                                );
                            }
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
