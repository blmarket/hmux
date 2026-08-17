//! The pane commands.

use super::*;

#[derive(Clone, Debug)]
pub(in crate::server) enum Command {
    New(NewPane),
    Split(SplitWindow),
    Kill(KillPane),
    Select(SelectPane),
    Last(LastPane),
    Swap(SwapPane),
    Move(MovePane),
    Join(MovePane),
    Break(BreakPane),
    Respawn(RespawnPane),
    Resize(ResizePane),
    ResizeWindow(ResizeWindow),
    RotateWindow(RotateWindow),
    SelectLayout(SelectLayout),
    NextLayout(CycleLayout),
    PreviousLayout(CycleLayout),
    ClearHistory(ClearHistory),
    Pipe(PipePane),
    Capture(CapturePane),
    List(ListPanes),
    CopyMode(CopyMode),
}

impl Command {
    pub(super) fn execute(self, context: &mut CommandContext<'_>) -> CommandResult {
        let st = &mut *context.state;
        match self {
            Self::New(command) => command.execute(st, context.client),
            Self::Split(command) => command.execute(st, context.client),
            Self::Kill(command) => command.execute(st),
            Self::Select(command) => command.execute(st, context.client),
            Self::Last(command) => command.execute(st),
            Self::Swap(command) => command.execute(st),
            Self::Move(command) | Self::Join(command) => command.execute(st),
            Self::Break(command) => command.execute(st),
            Self::Respawn(command) => command.execute(st, context.client),
            Self::Resize(command) => command.execute(st),
            Self::ResizeWindow(command) => command.execute(st),
            Self::RotateWindow(command) => command.execute(st),
            Self::SelectLayout(command) => command.execute(st),
            Self::NextLayout(command) => command.execute(st, true),
            Self::PreviousLayout(command) => command.execute(st, false),
            Self::ClearHistory(command) => command.execute(st),
            Self::Pipe(command) => command.execute(st),
            Self::Capture(command) => command.execute(st, context.agents),
            Self::List(command) => command.execute(st, context.agents),
            Self::CopyMode(command) => command.execute(st),
        }
    }
}

/// `split-window [-bdefhIklPvZ] [-c start-directory] [-e environment]
/// [-F format] [-l size] [-m message] [-p percentage] [-s style]
/// [-S active-border-style] [-R inactive-border-style] [-t target-pane]
/// [shell-command [argument ...]]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct SplitWindow {
    /// `-b`: insert the new pane before the target rather than after it.
    before: bool,
    /// `-d`: leave the original pane active.
    detached: bool,
    /// `-f`: creates new pane spanning full window width or height.
    full: bool,
    /// `-h`: split left/right instead of top/bottom.
    horizontal: bool,
    /// `-E`/`-I`: create the pane with no command of its own.
    empty: bool,
    /// `-I`: feed the client's standard input into the new pane.
    feed_input: bool,
    /// `-k`: keep the pane when its command exits.
    keep: bool,
    /// `-P`: print the new pane.
    print: bool,
    zoom: bool,
    /// `-c`: the new pane's working directory.
    cwd: Option<String>,
    /// `-e`: environment assignments for the new pane, repeatable.
    environment: Vec<String>,
    /// `-F`: what `-P` prints.
    format: Option<String>,
    /// `-l`: the new pane's size in cells, or a percentage.
    size: Option<String>,
    /// `-p`: the new pane's size as a percentage.
    percentage: Option<String>,
    /// `-m`: the message a kept pane shows once its command exits.
    message: Option<String>,
    /// `-t`: the pane to split.
    target: Option<String>,
    /// The new pane's command line.
    command: Vec<String>,
}

impl SplitWindow {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            before: args.has('b'),
            detached: args.has('d'),
            full: args.has('f'),
            horizontal: args.has('h'),
            empty: args.has('E') || args.has('I'),
            feed_input: args.has('I'),
            keep: args.has('k'),
            print: args.has('P'),
            zoom: args.has('Z'),
            cwd: args.value('c').map(str::to_string),
            environment: args.values('e').map(str::to_string).collect(),
            format: args.value('F').map(str::to_string),
            size: args.value('l').map(str::to_string),
            percentage: args.value('p').map(str::to_string),
            message: args.value('m').map(str::to_string),
            target: args.value('t').map(str::to_string),
            command: args.positionals().to_vec(),
        })
    }

    /// Adds a pane to the target window (the target's, or the current session's
    /// active window). The new pane becomes active. With `-P`, prints the new
    /// pane via `-F` (or `NEW_WINDOW_TEMPLATE`).
    fn execute(self, st: &mut ServerState, context: &ClientContext) -> CommandResult {
        let target = self.target.clone().or_else(|| current_target(st));
        let Some(target) = target else {
            return CommandResult::err("can't establish current session\n");
        };
        // split-window's `-t` is a *pane* target, so a resolve failure reports
        // tmux's pane diagnostic ("can't find pane: <t>"), not the session one.
        let resolved = match st.resolve(&target) {
            Some(target) => target,
            None => return CommandResult::err(format!("{}\n", st.pane_target_error(&target))),
        };
        st.push_zoom_at(resolved.session, resolved.window);
        // `-d` splits in the background: the new pane is created but the original
        // pane stays active (tmux's default is to follow to the new pane).
        let select = !self.detached;
        let direction = if self.horizontal {
            SplitDirection::LeftRight
        } else {
            SplitDirection::TopBottom
        };
        // `-l size` (cells, or `N%`) and `-p percentage` pin the *new* pane's size
        // on the split axis; percentages are of the target pane's current extent.
        let new_size = {
            let axis_total = {
                let sess = &st.sessions()[resolved.session];
                let win = st.window_for_link(&sess.windows[resolved.window]);
                if self.full {
                    match direction {
                        SplitDirection::LeftRight => win.cols,
                        SplitDirection::TopBottom => win.rows,
                    }
                } else {
                    let rect = win.pane_rect(win.panes[resolved.pane].id).unwrap_or(
                        super::super::state::PaneRect {
                            top: 0,
                            left: 0,
                            height: win.rows,
                            width: win.cols,
                        },
                    );
                    match direction {
                        SplitDirection::LeftRight => rect.width,
                        SplitDirection::TopBottom => rect.height,
                    }
                }
            };
            let percentage_of = |value: &str| {
                value
                    .parse::<u32>()
                    .ok()
                    .map(|percentage| (u32::from(axis_total) * percentage / 100) as u16)
            };
            let parsed = if let Some(value) = self.size.as_deref() {
                Some(match value.strip_suffix('%') {
                    Some(percentage) => percentage_of(percentage),
                    None => value.parse::<u16>().ok(),
                })
            } else {
                self.percentage.as_deref().map(percentage_of)
            };
            match parsed {
                Some(None) => {
                    return CommandResult::err("size or position invalid tiled geometry\n")
                }
                Some(size) => size,
                None => None,
            }
        };
        let explicit_cwd = self.cwd.clone().map(PathBuf::from);
        let cwd = explicit_cwd.as_deref().or(context.cwd.as_deref());
        if self.empty && self.command.iter().any(|word| !word.is_empty()) {
            return CommandResult::err("command cannot be given for empty pane\n");
        }
        let argv = pane_command_argv(&self.command, st, Some(&target));
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
            SpawnSession::Existing(&target),
        );
        let created = if self.empty {
            st.split_window_direction_with_spec(
                &target,
                select,
                self.before,
                self.full,
                direction,
                PaneSpec::Inert,
                new_size,
            )
        } else {
            st.split_window_direction_with_spawn(
                &target,
                select,
                self.before,
                self.full,
                direction,
                &argv,
                cwd,
                new_size,
            )
        };
        st.pop_zoom_at(resolved.session, resolved.window, self.zoom);
        // `-k` (and `-m format`) keep the pane in place when its command exits:
        // tmux sets the pane's remain-on-exit to `key` plus the format under `-m`.
        if let Ok(pane) = &created {
            if self.keep || self.message.is_some() {
                let new_target = Target {
                    session: resolved.session,
                    window: resolved.window,
                    pane: *pane,
                };
                st.set_pane_option(new_target, "remain-on-exit", "key");
                if let Some(message) = self.message.as_deref() {
                    st.set_pane_option(new_target, "remain-on-exit-format", message);
                }
            }
        }
        // `-I` feeds the client's standard input into whatever the split created.
        let feed = |st: &mut ServerState, pane: usize| -> Option<CommandResult> {
            if !self.feed_input {
                return None;
            }
            match context.input_file.as_ref() {
                Some(Ok(data)) => {
                    st.window(resolved.session, resolved.window).panes[pane]
                        .pane
                        .feed(data);
                    None
                }
                Some(Err(error)) => Some(CommandResult::err(format!(
                    "{}: -\n",
                    io_error_message(&io::Error::from_raw_os_error(*error))
                ))),
                None => None,
            }
        };
        // tmux hands `after-split-window` the find-state of the pane it just
        // created, not the `-t` the command was given.
        let settled = created.as_ref().ok().map(|pane| {
            format!(
                "%{}",
                st.window(resolved.session, resolved.window).panes[*pane].id
            )
        });
        let mut result = match created {
            Ok(pane) if self.print => {
                let sess = &st.sessions()[resolved.session];
                // `-P` prints the *newly created* pane, at whichever index the split
                // placed it (end by default, the target's index under `-b`) — the
                // print target even under `-d`, where the active pane is the original.
                let template = self.format.as_deref().unwrap_or(NEW_WINDOW_TEMPLATE);
                let marked = st.marked_pane();
                let line = expand_command_format(
                    st,
                    template,
                    &vars_full(st, sess, resolved.window, pane, &PaneAgents::new(), marked),
                    None,
                );
                if let Some(error) = feed(st, pane) {
                    return error;
                }
                CommandResult::ok(format!("{line}\n"))
            }
            Ok(pane) => {
                if let Some(error) = feed(st, pane) {
                    return error;
                }
                CommandResult::ok("")
            }
            Err(error) => CommandResult::err(format!("{error}\n")),
        };
        result.after_hook_target = settled;
        result
    }
}

/// `new-pane [-bdefhIklPvZ] [-c start-directory] [-e environment] [-F format]
/// [-l size] [-m message] [-p percentage] [-s style] [-S active-border-style]
/// [-R inactive-border-style] [-x width] [-y height] [-X x-position]
/// [-Y y-position] [-t target-pane] [shell-command [argument ...]]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct NewPane {
    /// `-b`: with `-L`, split before the target rather than after it.
    before: bool,
    /// `-d`: leave the original pane active.
    detached: bool,
    /// `-h`: with `-L`, split left/right instead of top/bottom.
    horizontal: bool,
    /// `-k`/`-m`: keep the pane when its command exits.
    keep: bool,
    /// `-L`: place the pane in the layout instead of floating it.
    layout: bool,
    /// `-P`: print the new pane.
    print: bool,
    /// `-c`: the new pane's working directory.
    cwd: Option<String>,
    /// `-e`: environment assignments for the new pane, repeatable.
    environment: Vec<String>,
    /// `-F`: what `-P` prints.
    format: Option<String>,
    /// `-m`: the message a kept pane shows once its command exits.
    message: Option<String>,
    /// `-s`/`-S`/`-R`: the pane's own style and border styles.
    style: Option<String>,
    active_border_style: Option<String>,
    inactive_border_style: Option<String>,
    /// `-x`/`-y`: the floating pane's size.
    width: Option<String>,
    height: Option<String>,
    /// `-X`/`-Y`: the floating pane's position.
    left: Option<String>,
    top: Option<String>,
    /// `-t`: the pane the new one is placed against.
    target: Option<String>,
    /// The new pane's command line.
    command: Vec<String>,
}

impl NewPane {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            before: args.has('b'),
            detached: args.has('d'),
            horizontal: args.has('h'),
            keep: args.has('k') || args.has('m'),
            layout: args.has('L'),
            print: args.has('P'),
            cwd: args.value('c').map(str::to_string),
            environment: args.values('e').map(str::to_string).collect(),
            format: args.value('F').map(str::to_string),
            message: args.value('m').map(str::to_string),
            style: args.value('s').map(str::to_string),
            active_border_style: args.value('S').map(str::to_string),
            inactive_border_style: args.value('R').map(str::to_string),
            width: args.value('x').map(str::to_string),
            height: args.value('y').map(str::to_string),
            left: args.value('X').map(str::to_string),
            top: args.value('Y').map(str::to_string),
            target: args.value('t').map(str::to_string),
            command: args.positionals().to_vec(),
        })
    }

    fn execute(self, st: &mut ServerState, context: &ClientContext) -> CommandResult {
        let target = self.target.clone().or_else(|| current_target(st));
        let Some(target) = target else {
            return CommandResult::err("can't establish current session\n");
        };
        let resolved = match st.resolve(&target) {
            Some(target) => target,
            None => return CommandResult::err(format!("{}\n", st.pane_target_error(&target))),
        };
        let window = st.window(resolved.session, resolved.window);
        let (window_width, window_height) = (window.cols, window.rows);
        let geometry = |value: Option<&str>, total| -> Result<Option<u16>, CommandResult> {
            value
                .map(|value| parse_pane_size(value, total))
                .transpose()
                .map_err(|_| CommandResult::err("size or position invalid floating geometry\n"))
        };
        let width = match geometry(self.width.as_deref(), window_width) {
            Ok(value) => value,
            Err(error) => return error,
        };
        let height = match geometry(self.height.as_deref(), window_height) {
            Ok(value) => value,
            Err(error) => return error,
        };
        let position = |value: Option<&str>, total| -> Result<Option<i32>, CommandResult> {
            value
                .map(|value| parse_pane_position(value, total))
                .transpose()
                .map_err(|_| CommandResult::err("size or position invalid floating geometry\n"))
        };
        let left = match position(self.left.as_deref(), window_width) {
            Ok(value) => value,
            Err(error) => return error,
        };
        let top = match position(self.top.as_deref(), window_height) {
            Ok(value) => value,
            Err(error) => return error,
        };
        let explicit_cwd = self.cwd.clone().map(PathBuf::from);
        let cwd = explicit_cwd.as_deref().or(context.cwd.as_deref());
        let shell = context
            .env("SHELL")
            .map(str::to_string)
            .or_else(|| std::env::var("SHELL").ok())
            .unwrap_or_else(|| "/bin/sh".into());
        let argv = match self.command.as_slice() {
            [] => vec![shell],
            [command] => vec![shell, "-c".into(), command.clone()],
            command => command.to_vec(),
        };
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
            SpawnSession::Existing(&target),
        );
        let select = !self.detached;
        let created = if self.layout {
            let direction = if self.horizontal {
                SplitDirection::LeftRight
            } else {
                SplitDirection::TopBottom
            };
            st.split_window_direction_with_spawn(
                &target,
                select,
                self.before,
                false,
                direction,
                &argv,
                cwd,
                None,
            )
        } else {
            st.new_floating_pane_with_spawn(&target, select, width, height, left, top, &argv, cwd)
        };
        let pane = match created {
            Ok(pane) => pane,
            Err(error) => return CommandResult::err(format!("{error}\n")),
        };
        let pane_target = Target {
            session: resolved.session,
            window: resolved.window,
            pane,
        };
        if let Some(style) = self.style.as_deref() {
            st.set_pane_option(pane_target, "window-style", style);
            st.set_pane_option(pane_target, "window-active-style", style);
        }
        if let Some(style) = self.active_border_style.as_deref() {
            st.set_pane_option(pane_target, "pane-active-border-style", style);
        }
        if let Some(style) = self.inactive_border_style.as_deref() {
            st.set_pane_option(pane_target, "pane-border-style", style);
        }
        if self.keep {
            st.set_pane_option(pane_target, "remain-on-exit", "on");
        }
        if let Some(message) = self.message.as_deref() {
            st.set_pane_option(pane_target, "remain-on-exit-format", message);
        }
        if self.print {
            let session = &st.sessions()[resolved.session];
            let template = self.format.as_deref().unwrap_or(NEW_WINDOW_TEMPLATE);
            let line = expand_command_format(
                st,
                template,
                &vars_full(
                    st,
                    session,
                    resolved.window,
                    pane,
                    &PaneAgents::new(),
                    st.marked_pane(),
                ),
                None,
            );
            CommandResult::ok(format!("{line}\n"))
        } else {
            CommandResult::ok("")
        }
    }
}

fn parse_pane_size(value: &str, total: u16) -> Result<u16, ()> {
    let size = if let Some(percent) = value.strip_suffix('%') {
        u32::from(total) * percent.parse::<u32>().map_err(|_| ())? / 100
    } else {
        value.parse::<u32>().map_err(|_| ())?
    };
    u16::try_from(size)
        .ok()
        .filter(|size| *size < total)
        .ok_or(())
}

fn parse_pane_position(value: &str, total: u16) -> Result<i32, ()> {
    if let Some(percent) = value.strip_suffix('%') {
        let percent = percent.parse::<i32>().map_err(|_| ())?;
        Ok(i32::from(total) * percent / 100)
    } else {
        value.parse::<i32>().map_err(|_| ())
    }
}

/// `kill-pane [-a] [-t target-pane]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct KillPane {
    /// `-a`: kill every pane of the window but the target.
    all_but: bool,
    /// `-t`: the pane to kill.
    target: Option<String>,
}

impl KillPane {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            all_but: args.has('a'),
            target: args.value('t').map(str::to_string),
        })
    }

    /// Removes the target pane (destroying its window, and the session, if it
    /// was the last).
    fn execute(self, st: &mut ServerState) -> CommandResult {
        let target = self.target.or_else(|| current_target(st));
        let Some(target) = target else {
            return CommandResult::err("can't establish current session\n");
        };
        let result = if self.all_but {
            st.kill_other_panes(&target)
        } else {
            st.kill_pane(&target)
        };
        match result {
            Ok(()) => CommandResult::ok(""),
            Err(error) => CommandResult::err(format!("{error}\n")),
        }
    }
}

/// `select-pane [-DdeLlMmRUZ] [-T title] [-t target-pane]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct SelectPane {
    /// `-U`/`-D`/`-L`/`-R`: select the pane in that direction.
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    /// `-d`/`-e`: disable or enable input on the pane.
    disable: bool,
    enable: bool,
    /// `-l`: take the last-pane path instead.
    last: bool,
    /// `-M`: clear the server's marked pane.
    clear_mark: bool,
    /// `-m`: mark the target pane without activating it.
    mark: bool,
    /// `-T`: set the pane's title.
    title: Option<String>,
    /// `-t`: the pane to act on.
    target: Option<String>,
}

impl SelectPane {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            up: args.has('U'),
            down: args.has('D'),
            left: args.has('L'),
            right: args.has('R'),
            disable: args.has('d'),
            enable: args.has('e'),
            last: args.has('l'),
            clear_mark: args.has('M'),
            mark: args.has('m'),
            title: args.value('T').map(str::to_string),
            target: args.value('t').map(str::to_string),
        })
    }

    /// Makes the target pane active.
    fn execute(self, st: &mut ServerState, context: &ClientContext) -> CommandResult {
        // `-l` takes the whole last-pane path first, exactly as tmux routes
        // `select-pane -l` and `last-pane` through one branch — including the
        // `-d`/`-e` input toggles, which then act on the last pane, not the
        // target pane.
        if self.last {
            return LastPane {
                disable: self.disable,
                enable: self.enable,
                target: self.target,
            }
            .execute(st);
        }
        // `-M` clears the server's marked pane and ignores everything else.
        if self.clear_mark {
            st.clear_mark();
            return CommandResult::ok("");
        }
        let target = self.target.or_else(|| current_target(st));
        let Some(target) = target else {
            return CommandResult::err("can't establish current session\n");
        };
        if let Some(title) = self.title.as_deref() {
            return match st.set_pane_title(&target, title) {
                Ok(()) => CommandResult::ok(""),
                Err(error) => CommandResult::err(format!("{error}\n")),
            };
        }
        if self.disable || self.enable {
            return match st.set_pane_input_off(&target, self.disable) {
                Ok(()) => CommandResult::ok(""),
                Err(error) => CommandResult::err(format!("{error}\n")),
            };
        }
        let directional = [
            (self.up, SplitDirection::TopBottom, false),
            (self.down, SplitDirection::TopBottom, true),
            (self.left, SplitDirection::LeftRight, false),
            (self.right, SplitDirection::LeftRight, true),
        ]
        .into_iter()
        .find(|(given, _, _)| *given);
        if let Some((_, direction, forward)) = directional {
            if context.control_active_panes().is_some() {
                return match st.pane_in_direction(&target, direction, forward) {
                    Ok((window_id, pane_id)) => {
                        context.set_active_pane(window_id, pane_id);
                        CommandResult::ok("")
                    }
                    Err(error) => CommandResult::err(format!("{error}\n")),
                };
            }
            return match st.select_pane_direction(&target, direction, forward) {
                Ok(()) => CommandResult::ok(""),
                Err(error) => CommandResult::err(format!("{error}\n")),
            };
        }
        // `-m` toggles the mark on the target without changing the active pane; the
        // plain form selects (activates) it.
        let result = if self.mark {
            st.mark_pane(&target)
        } else if context.control_active_panes().is_some() {
            match st.resolve(&target) {
                Some(target) => {
                    let (window_id, pane_id) = st.target_pane_ids(target);
                    context.set_active_pane(window_id, pane_id);
                    Ok(())
                }
                None => Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("can't find pane: {target}"),
                )),
            }
        } else {
            st.select_pane(&target)
        };
        match result {
            Ok(()) => CommandResult::ok(""),
            Err(error) => CommandResult::err(format!("{error}\n")),
        }
    }
}

/// `last-pane [-deZ] [-t target-window]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct LastPane {
    /// `-d`/`-e`: disable or enable input on the last pane instead of switching
    /// to it. `-e` wins when both are given.
    disable: bool,
    enable: bool,
    /// `-t`: the window whose last pane is used.
    target: Option<String>,
}

impl LastPane {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            disable: args.has('d'),
            enable: args.has('e'),
            target: args.value('t').map(str::to_string),
        })
    }

    /// Switches to the previously-active pane; with `-e` or `-d` it instead
    /// enables or disables input on that pane without switching, as tmux's
    /// `cmd-select-pane.c` does.
    fn execute(self, st: &mut ServerState) -> CommandResult {
        let Some(target) = self.target.or_else(|| current_target(st)) else {
            return CommandResult::err("can't establish current session\n");
        };
        let result = if self.enable {
            st.set_last_pane_input_off(&target, false)
        } else if self.disable {
            st.set_last_pane_input_off(&target, true)
        } else {
            st.last_pane(&target)
        };
        match result {
            Ok(()) => CommandResult::ok(""),
            Err(error) => command_target_error(error, &target, "window"),
        }
    }
}

/// `swap-pane [-dDUZ] [-s src-pane] [-t dst-pane]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct SwapPane {
    /// `-D`/`-U`: swap with the next/previous neighbour. `-D` wins.
    down: bool,
    up: bool,
    /// `-d`: do not select the pane after an adjacent swap.
    detached: bool,
    zoom: bool,
    /// `-s`/`-t`: the panes to exchange.
    source: Option<String>,
    target: Option<String>,
}

impl SwapPane {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            down: args.has('D'),
            up: args.has('U'),
            detached: args.has('d'),
            zoom: args.has('Z'),
            source: args.value('s').map(str::to_string),
            target: args.value('t').map(str::to_string),
        })
    }

    /// Both targets default to the current session's active pane.
    fn execute(self, st: &mut ServerState) -> CommandResult {
        let current = current_target(st);
        // `-U`/`-D` swap the target (default active) pane with its previous/next
        // neighbour; `-D` takes precedence when both are given (as in tmux).
        if self.down || self.up {
            let target = self.target.or_else(|| current.clone());
            return match target {
                Some(target) => {
                    let resolved = match st.resolve(&target) {
                        Some(resolved) => resolved,
                        None => {
                            return CommandResult::err(format!(
                                "{}\n",
                                st.pane_target_error(&target)
                            ))
                        }
                    };
                    st.push_zoom_at(resolved.session, resolved.window);
                    let result = st.swap_pane_neighbour(&target, self.down, !self.detached);
                    st.pop_zoom_at(resolved.session, resolved.window, self.zoom);
                    match result {
                        Ok(()) => CommandResult::ok(""),
                        Err(error) => CommandResult::err(format!("{error}\n")),
                    }
                }
                None => CommandResult::err("can't establish current session\n"),
            };
        }
        let source = self.source.or_else(|| current.clone());
        let target = self.target.or(current);
        match (source, target) {
            (Some(source), Some(target)) => {
                let src = match st.resolve(&source) {
                    Some(resolved) => resolved,
                    None => {
                        return CommandResult::err(format!("{}\n", st.pane_target_error(&source)))
                    }
                };
                let dst = match st.resolve(&target) {
                    Some(resolved) => resolved,
                    None => {
                        return CommandResult::err(format!("{}\n", st.pane_target_error(&target)))
                    }
                };
                st.push_zoom_at(src.session, src.window);
                st.push_zoom_at(dst.session, dst.window);
                let result = st.swap_pane(&source, &target);
                st.pop_zoom_at(src.session, src.window, self.zoom);
                st.pop_zoom_at(dst.session, dst.window, self.zoom);
                match result {
                    Ok(()) => CommandResult::ok(""),
                    Err(error) => CommandResult::err(format!("{error}\n")),
                }
            }
            _ => CommandResult::err("can't establish current session\n"),
        }
    }
}

/// `move-pane`/`join-pane [-bdfhv] [-l size] [-s src-pane] [-t dst-pane]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct MovePane {
    /// `-b`: place the moved pane before the destination rather than after it.
    before: bool,
    /// `-d`: do not make the moved pane active.
    detached: bool,
    /// `-f`: creates new pane spanning full window width or height.
    _full: bool,
    /// `-h`: horizontal split.
    horizontal: bool,
    /// `-v`: vertical split.
    _vertical: bool,
    /// `-l`: size of the new pane (lines/cells or percentage).
    size: Option<String>,
    /// `-p`: percentage.
    percentage: Option<String>,
    /// `-s`/`-t`: the pane to move, and where it goes.
    source: Option<String>,
    target: Option<String>,
}

impl MovePane {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            before: args.has('b'),
            detached: args.has('d'),
            _full: args.has('f'),
            horizontal: args.has('h'),
            _vertical: args.has('v'),
            size: args.value('l').map(str::to_string),
            percentage: args.value('p').map(str::to_string),
            source: args.value('s').map(str::to_string),
            target: args.value('t').map(str::to_string),
        })
    }

    /// Moves a pane into another window.
    fn execute(self, st: &mut ServerState) -> CommandResult {
        let source = self
            .source
            .or_else(|| st.marked_pane().map(|id| format!("%{id}")))
            .or_else(|| current_target(st));
        let target = self.target.or_else(|| current_target(st));
        let (Some(source), Some(target)) = (source, target) else {
            return CommandResult::err("can't establish current session\n");
        };
        let target_resolved = match st.resolve(&target) {
            Some(t) => t,
            None => return CommandResult::err(format!("{}\n", st.pane_target_error(&target))),
        };
        let direction = if self.horizontal {
            SplitDirection::LeftRight
        } else {
            SplitDirection::TopBottom
        };
        let new_size = {
            let axis_total = {
                let sess = &st.sessions()[target_resolved.session];
                let win = st.window_for_link(&sess.windows[target_resolved.window]);
                let rect = win.pane_rect(win.panes[target_resolved.pane].id).unwrap_or(
                    super::super::state::PaneRect {
                        top: 0,
                        left: 0,
                        height: win.rows,
                        width: win.cols,
                    },
                );
                match direction {
                    SplitDirection::LeftRight => rect.width,
                    SplitDirection::TopBottom => rect.height,
                }
            };
            let percentage_of = |value: &str| {
                value
                    .parse::<u32>()
                    .ok()
                    .map(|percentage| (u32::from(axis_total) * percentage / 100) as u16)
            };
            let parsed = if let Some(value) = self.size.as_deref() {
                Some(match value.strip_suffix('%') {
                    Some(percentage) => percentage_of(percentage),
                    None => value.parse::<u16>().ok(),
                })
            } else {
                self.percentage.as_deref().map(percentage_of)
            };
            match parsed {
                Some(None) => return CommandResult::err("create pane failed: size invalid\n"),
                Some(size) => size,
                None => None,
            }
        };
        let select = !self.detached;
        match st.move_pane(&source, &target, self.before, select, direction, new_size) {
            Ok(()) => CommandResult::ok(""),
            Err(error) => {
                command_target_error_candidates(error, &[(&source, "pane"), (&target, "pane")])
            }
        }
    }
}

/// `break-pane [-abdP] [-F format] [-n window-name] [-s src-pane]
/// [-t dst-window]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct BreakPane {
    /// `-a`/`-b`: place the new window after/before the destination index.
    after: bool,
    before: bool,
    /// `-d`: leave the current window where it is.
    detached: bool,
    /// `-P`: print the new window.
    print: bool,
    /// `-F`: what `-P` prints.
    format: Option<String>,
    /// `-n`: the new window's name.
    name: Option<String>,
    /// `-s`/`-t`: the pane to break out, and the window it becomes.
    source: Option<String>,
    target: Option<String>,
}

impl BreakPane {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            after: args.has('a'),
            before: args.has('b'),
            detached: args.has('d'),
            print: args.has('P'),
            format: args.value('F').map(str::to_string),
            name: args.value('n').map(str::to_string),
            source: args.value('s').map(str::to_string),
            target: args.value('t').map(str::to_string),
        })
    }

    /// Moves a pane into a new window. A `-t` that carries a pane part is
    /// rejected exactly like tmux. `-n` names the new window (default empty).
    /// With `-P`, prints the new window via `-F` (or `NEW_WINDOW_TEMPLATE`).
    fn execute(self, st: &mut ServerState) -> CommandResult {
        // `-t` names a *window*; a pane part (a `.` after the window) is an error.
        if let Some(destination) = self.target.as_deref() {
            if destination
                .rsplit_once('.')
                .is_some_and(|(_, pane)| !pane.is_empty() && pane.parse::<u32>().is_ok())
            {
                return CommandResult::err("can't specify pane here\n");
            }
        }
        let source = self.source.clone().or_else(|| current_target(st));
        let target = self
            .target
            .clone()
            .or_else(|| current_session(st).map(|session| format!("{session}:")));
        let relative = (self.after || self.before).then_some(!self.before);
        match (source, target) {
            (Some(source), Some(target)) => match st.break_pane(
                &source,
                &target,
                self.name.as_deref(),
                !self.detached,
                relative,
            ) {
                Ok(broken) if self.print => {
                    let sess = &st.sessions()[broken.session];
                    let template = self.format.as_deref().unwrap_or(NEW_WINDOW_TEMPLATE);
                    let marked = st.marked_pane();
                    let line = expand_command_format(
                        st,
                        template,
                        &vars_full(
                            st,
                            sess,
                            broken.window,
                            broken.pane,
                            &PaneAgents::new(),
                            marked,
                        ),
                        None,
                    );
                    CommandResult::ok(format!("{line}\n"))
                }
                Ok(_) => CommandResult::ok(""),
                Err(error) => command_target_error(error, &target, "window"),
            },
            _ => CommandResult::err("can't establish current session\n"),
        }
    }
}

/// `respawn-pane [-k] [-c start-directory] [-e environment] [-t target-pane]
/// [shell-command [argument ...]]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct RespawnPane {
    /// `-k`: kill the pane's child first.
    kill: bool,
    /// `-c`: the replacement's working directory.
    cwd: Option<String>,
    /// `-e`: environment assignments for the replacement, repeatable.
    environment: Vec<String>,
    /// `-t`: the pane to respawn.
    target: Option<String>,
    /// The replacement command line.
    command: Vec<String>,
}

impl RespawnPane {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            kill: args.has('k'),
            cwd: args.value('c').map(str::to_string),
            environment: args.values('e').map(str::to_string).collect(),
            target: args.value('t').map(str::to_string),
            command: args.positionals().to_vec(),
        })
    }

    /// Real tmux refuses to respawn a pane whose child is still running unless
    /// `-k` (kill first) is given, failing with `respawn pane failed: pane <t>
    /// still active` (exit 1). Native models that guard: the target must resolve
    /// (else tmux's `can't find pane`), and a pane that has not exited is "still
    /// active". With `-k`, or against an exited pane, the command succeeds
    /// silently — the actual re-exec of the pane's command is interactive byte
    /// work outside this control-plane layer.
    fn execute(self, st: &mut ServerState, context: &ClientContext) -> CommandResult {
        let target = self.target.clone().or_else(|| current_target(st));
        let Some(target) = target else {
            return CommandResult::err("can't establish current session\n");
        };
        let resolved = match st.resolve(&target) {
            Some(resolved) => resolved,
            None => return CommandResult::err(format!("{}\n", st.pane_target_error(&target))),
        };
        let sess = &st.sessions()[resolved.session];
        let link = &sess.windows[resolved.window];
        let win = st.window_for_link(link);
        let node = &win.panes[resolved.pane];
        // `-k` bypasses the guard; a pane that has already exited may be respawned.
        if !self.kill && !node.pane.has_exited() {
            return CommandResult::err(format!(
                "respawn pane failed: pane {}:{}.{} still active\n",
                sess.name, link.index, resolved.pane,
            ));
        }
        // A respawn spells its command the way a spawn does: one argument is a
        // shell command line, several are an argv (tmux's `spawn_pane`).
        let argv =
            (!self.command.is_empty()).then(|| pane_command_argv(&self.command, st, Some(&target)));
        let mut cwd = self.cwd.clone().map(PathBuf::from);
        // `-e` reaches the replacement the way a spawn's environment does. With no
        // command the saved spawn spec is materialized so the wrap has an argv to
        // carry, keeping its stored working directory.
        let environment = self
            .environment
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let argv = if environment.is_empty() {
            argv
        } else {
            let base = argv.or_else(|| {
                let saved = node.pane.spawn_spec()?;
                if cwd.is_none() {
                    cwd = saved.cwd;
                }
                Some(saved.argv)
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
        match st.respawn_pane_process(&target, argv, cwd) {
            Ok(()) => CommandResult::ok(""),
            Err(error) => CommandResult::err(format!("{error}\n")),
        }
    }
}

/// `clear-history [-H] [-t target-pane]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct ClearHistory {
    /// `-t`: the pane whose history is cleared.
    target: Option<String>,
}

impl ClearHistory {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            target: args.value('t').map(str::to_string),
        })
    }

    /// The emulator owns the pane's scrollback, so ask it to erase history while
    /// retaining the viewport, then leave any copy mode associated with the pane.
    fn execute(self, st: &mut ServerState) -> CommandResult {
        let Some(target) = self.target.or_else(|| current_target(st)) else {
            return CommandResult::err("can't establish current session\n");
        };
        match st.clear_pane_history(&target) {
            Ok(()) => CommandResult::ok(""),
            Err(_) => CommandResult::err(format!("{}\n", st.pane_target_error(&target))),
        }
    }
}

/// `pipe-pane [-IOo] [-t target-pane] [shell-command]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct PipePane {
    /// `-I`: connect the command's output to the pane's input.
    input: bool,
    /// `-O`: write the pane's output to the command.
    output: bool,
    /// `-o`: only toggle the pipe.
    toggle: bool,
    /// `-t`: the pane to pipe.
    target: Option<String>,
    /// The shell command the pane is piped through.
    command: Vec<String>,
}

impl PipePane {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            input: args.has('I'),
            output: args.has('O'),
            toggle: args.has('o'),
            target: args.value('t').map(str::to_string),
            command: args.positionals().to_vec(),
        })
    }

    /// By default pane output is written to the command. `-I` additionally
    /// connects command output to pane input; specifying only `-I` suppresses
    /// the default output direction.
    fn execute(self, st: &mut ServerState) -> CommandResult {
        let Some(target) = self.target.or_else(|| current_target(st)) else {
            return CommandResult::err("can't establish current session\n");
        };
        // `pipe-pane` has a pane target, so tmux interprets an unqualified value
        // as a pane before falling back to a window or session. Keep the existing
        // fallback for targets that resolve, but report the target-type-specific
        // diagnostic when a bare value cannot resolve at all.
        if !target.contains([':', '.'])
            && !target.starts_with(['$', '@', '%'])
            && st.resolve(&target).is_none()
        {
            return CommandResult::err(format!("can't find pane: {target}\n"));
        }
        let command = self.command.join(" ");
        let output = self.output || !self.input;
        match st.pipe_pane(
            &target,
            (!command.is_empty()).then_some(command.as_str()),
            self.toggle,
            self.input,
            output,
        ) {
            Ok(()) => CommandResult::ok(""),
            Err(error) => CommandResult::err(format!("{error}\n")),
        }
    }
}

/// `resize-pane [-DLMRTUZ] [-x width] [-y height] [-t target-pane] [adjustment]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct ResizePane {
    /// `-L`/`-R`/`-U`/`-D`: move one boundary by `adjustment` cells.
    left: bool,
    right: bool,
    up: bool,
    down: bool,
    /// `-M`: drag the border the mouse grabbed to where it is now.
    mouse: bool,
    /// `-T`: trim the blank rows below the cursor.
    trim: bool,
    /// `-Z`: toggle the pane's zoom.
    zoom: bool,
    /// `-x`/`-y`: set an axis outright.
    width: Option<String>,
    height: Option<String>,
    /// `-t`: the pane to resize.
    target: Option<String>,
    adjustment: Option<String>,
}

impl ResizePane {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            left: args.has('L'),
            right: args.has('R'),
            up: args.has('U'),
            down: args.has('D'),
            mouse: args.has('M'),
            trim: args.has('T'),
            zoom: args.has('Z'),
            width: args.value('x').map(str::to_string),
            height: args.value('y').map(str::to_string),
            target: args.value('t').map(str::to_string),
            adjustment: args.positionals().first().cloned(),
        })
    }

    /// Toggles zoom or moves one boundary in the retained layout tree.
    fn execute(self, st: &mut ServerState) -> CommandResult {
        let Some(target) = self.target.or_else(|| current_target(st)) else {
            return CommandResult::err("can't establish current session\n");
        };
        if st.resolve(&target).is_none() {
            return CommandResult::err(format!("{}\n", st.pane_target_error(&target)));
        }
        // `-T` trims the blank rows below the cursor and pulls the same number of
        // rows out of the history. tmux does nothing at all when the pane is in a
        // mode, whose screen is not the one that would be trimmed.
        if self.trim {
            let Some(resolved) = st.resolve(&target) else {
                return CommandResult::ok("");
            };
            let node = &st.window(resolved.session, resolved.window).panes[resolved.pane];
            if node.copy.is_some() {
                return CommandResult::ok("");
            }
            node.pane.trim_history_below_cursor();
            return CommandResult::ok("");
        }
        if self.zoom {
            return match st.toggle_zoom(&target) {
                Ok(_) => CommandResult::ok(""),
                Err(error) => CommandResult::err(format!("{error}\n")),
            };
        }
        if self.mouse {
            return resize_pane_to_mouse(st);
        }
        let resolved = st.resolve(&target).expect("validated target");
        let (window_cols, window_rows) = {
            let win = st.window(resolved.session, resolved.window);
            (win.cols, win.rows)
        };
        for (value, direction, label) in [
            (self.width.as_deref(), SplitDirection::LeftRight, "width"),
            (self.height.as_deref(), SplitDirection::TopBottom, "height"),
        ] {
            if let Some(value) = value {
                let total = match direction {
                    SplitDirection::LeftRight => window_cols,
                    SplitDirection::TopBottom => window_rows,
                };
                let size = if let Some(pct_str) = value.strip_suffix('%') {
                    match pct_str.parse::<u32>() {
                        Ok(pct) => (u32::from(total) * pct / 100) as u16,
                        Err(_) => return CommandResult::err(format!("{label} invalid\n")),
                    }
                } else {
                    match value.parse::<u16>() {
                        Ok(size) => size,
                        Err(_) => return CommandResult::err(format!("{label} invalid\n")),
                    }
                };
                if let Err(error) = st.resize_pane_to(&target, direction, size) {
                    return CommandResult::err(format!("{error}\n"));
                }
            }
        }
        let adjustment = match self.adjustment.as_deref().unwrap_or("1").parse::<u16>() {
            Ok(0) => return CommandResult::err("adjustment too small\n"),
            Ok(value) => value,
            Err(_) => return CommandResult::err("adjustment invalid\n"),
        };
        let direction = if self.left {
            Some((SplitDirection::LeftRight, false))
        } else if self.right {
            Some((SplitDirection::LeftRight, true))
        } else if self.up {
            Some((SplitDirection::TopBottom, false))
        } else if self.down {
            Some((SplitDirection::TopBottom, true))
        } else {
            None
        };
        match direction {
            Some((direction, forward)) => {
                match st.resize_pane(&target, direction, forward, adjustment) {
                    Ok(()) => CommandResult::ok(""),
                    Err(error) => CommandResult::err(format!("{error}\n")),
                }
            }
            None => CommandResult::ok(""),
        }
    }
}

/// `resize-pane -M`: drag the border the mouse grabbed to where it is now.
///
/// tmux installs a drag callback that runs on every later report; hmux instead
/// keeps the drag pinned to the border it started on (see
/// `MouseInputState::observe`) and re-runs this on each `MouseDrag1Border`, so
/// the border tracks the pointer the same way.
fn resize_pane_to_mouse(st: &mut ServerState) -> CommandResult {
    // tmux's `cmd_resize_pane_exec` returns quietly when there is no mouse
    // event, so a command client running `resize-pane -M` is not an error.
    let Some(mouse) = st.command_mouse() else {
        return CommandResult::ok("");
    };
    let Some((pane_id, side)) = mouse
        .target
        .as_ref()
        .filter(|target| target.location == super::super::key::MouseLocation::Border)
        .and_then(|target| Some((target.pane_id?, target.border_side?)))
    else {
        return CommandResult::ok("");
    };
    let position = mouse.position;
    let grabbed = mouse.last_position.unwrap_or(position);
    let target = format!("%{pane_id}");
    let Some(resolved) = st.resolve(&target) else {
        return CommandResult::ok("");
    };
    // A floating pane is not in the layout, so it is moved and resized
    // directly by whichever of its own borders the pointer grabbed.
    if st.drag_floating_pane(&target, (grabbed.x, grabbed.y), (position.x, position.y)) {
        return CommandResult::ok("");
    }
    let status_offset = if super::super::status::at_top(st, &target) {
        super::super::status::height(st, &target)
    } else {
        0
    };
    let Some(rect) = st
        .window(resolved.session, resolved.window)
        .pane_rect(pane_id)
    else {
        return CommandResult::ok("");
    };
    // Moving a pane's own top or left border grows it; moving its bottom or
    // right border shrinks or grows it from the other end. Either way the new
    // size is the distance from the edge that stayed put to the pointer.
    let pane_y = position.y.saturating_sub(status_offset);
    let (direction, size) = match side {
        super::super::mouse::BorderSide::Bottom => {
            (SplitDirection::TopBottom, pane_y.saturating_sub(rect.top))
        }
        super::super::mouse::BorderSide::Top => (
            SplitDirection::TopBottom,
            rect.top
                .saturating_add(rect.height)
                .saturating_sub(pane_y)
                .saturating_sub(1),
        ),
        super::super::mouse::BorderSide::Right => (
            SplitDirection::LeftRight,
            position.x.saturating_sub(rect.left),
        ),
        super::super::mouse::BorderSide::Left => (
            SplitDirection::LeftRight,
            rect.left
                .saturating_add(rect.width)
                .saturating_sub(position.x)
                .saturating_sub(1),
        ),
    };
    if size == 0 {
        return CommandResult::ok("");
    }
    match st.resize_pane_to(&target, direction, size) {
        Ok(()) => CommandResult::ok(""),
        Err(_) => CommandResult::ok(""),
    }
}

/// `resize-window [-aADLRU] [-x width] [-y height] [-t target-window]
/// [adjustment]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct ResizeWindow {
    /// `-a`/`-A`: snap to the smallest/largest client that can see the window.
    smallest: bool,
    largest: bool,
    /// `-L`/`-R`/`-U`/`-D`: move one edge by `adjustment` cells.
    left: bool,
    right: bool,
    up: bool,
    down: bool,
    /// `-x`/`-y`: set an axis outright.
    width: Option<String>,
    height: Option<String>,
    /// `-t`: the window to resize.
    target: Option<String>,
    adjustment: Option<String>,
}

impl ResizeWindow {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            smallest: args.has('a'),
            largest: args.has('A'),
            left: args.has('L'),
            right: args.has('R'),
            up: args.has('U'),
            down: args.has('D'),
            width: args.value('x').map(str::to_string),
            height: args.value('y').map(str::to_string),
            target: args.value('t').map(str::to_string),
            adjustment: args.positionals().first().cloned(),
        })
    }

    /// Every form pins the window at a manual size: `-x`/`-y` set an axis
    /// outright, `-L/-R/-U/-D` move one edge by `adjustment` cells (default 1),
    /// and `-a`/`-A` snap to the smallest/largest client that can see the
    /// window. An out-of-range or non-numeric `-x`/`-y` value, or a bad
    /// adjustment, is rejected exactly like tmux.
    fn execute(self, st: &mut ServerState) -> CommandResult {
        let cols = match parse_size_flag(self.width.as_deref(), "width") {
            Ok(value) => value,
            Err(error) => return CommandResult::err(error),
        };
        let rows = match parse_size_flag(self.height.as_deref(), "height") {
            Ok(value) => value,
            Err(error) => return CommandResult::err(error),
        };
        let adjustment = match self.adjustment.as_deref().unwrap_or("1").parse::<u16>() {
            Ok(0) => return CommandResult::err("adjustment too small\n"),
            Ok(value) => value,
            Err(_) => return CommandResult::err("adjustment invalid\n"),
        };
        let adjust = if self.left {
            Some(WindowResizeAdjust::Left)
        } else if self.right {
            Some(WindowResizeAdjust::Right)
        } else if self.up {
            Some(WindowResizeAdjust::Up)
        } else if self.down {
            Some(WindowResizeAdjust::Down)
        } else {
            None
        };
        // `-A` wins over `-a`, as tmux's flag order does.
        let snap = if self.largest {
            Some(WindowSizePolicy::Largest)
        } else if self.smallest {
            Some(WindowSizePolicy::Smallest)
        } else {
            None
        };
        if cols.is_none() && rows.is_none() && adjust.is_none() && snap.is_none() {
            return CommandResult::ok("");
        }
        let Some(target) = self.target.or_else(|| current_target(st)) else {
            return CommandResult::err("can't establish current session\n");
        };
        let request = WindowResizeRequest {
            cols,
            rows,
            adjust,
            adjustment,
            snap,
        };
        match st.resize_window(&target, request) {
            Ok(()) => CommandResult::ok(""),
            Err(error) => command_target_error(error, &target, "window"),
        }
    }
}

/// tmux's inclusive size bounds for `resize-window -x/-y` (`strtonum` range).
const WINDOW_SIZE_MIN: i64 = 1;
const WINDOW_SIZE_MAX: i64 = 10000;

/// Parse a `-x`/`-y` size flag value the way tmux does: a number in
/// `[1, 10000]`. `label` is the axis word ("width"/"height") tmux prints in its
/// diagnostic. Returns `Ok(None)` when the flag is absent, `Ok(Some(n))` for a
/// valid size, or tmux's `<label> invalid|too small|too large` on a bad one.
fn parse_size_flag(value: Option<&str>, label: &str) -> Result<Option<u16>, String> {
    match value {
        None => Ok(None),
        Some(value) => match value.parse::<i64>() {
            Err(_) => Err(format!("{label} invalid\n")),
            Ok(size) if size < WINDOW_SIZE_MIN => Err(format!("{label} too small\n")),
            Ok(size) if size > WINDOW_SIZE_MAX => Err(format!("{label} too large\n")),
            Ok(size) => Ok(Some(size as u16)),
        },
    }
}

/// `rotate-window [-DUZ] [-t target-window]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct RotateWindow {
    /// `-D`: rotate the other way.
    down: bool,
    zoom: bool,
    /// `-t`: the window to rotate.
    target: Option<String>,
}

impl RotateWindow {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            down: args.has('D'),
            zoom: args.has('Z'),
            target: args.value('t').map(str::to_string),
        })
    }

    /// Rotates the target window's panes.
    fn execute(self, st: &mut ServerState) -> CommandResult {
        let Some(target) = self.target.or_else(|| current_target(st)) else {
            return CommandResult::err("can't establish current session\n");
        };
        if let Err(error) = st.push_zoom(&target) {
            return command_target_error(error, &target, "window");
        }
        let result = if self.down {
            st.rotate_window_down(&target)
        } else {
            st.rotate_window(&target)
        };
        let zoom_result = st.pop_zoom(&target, self.zoom);
        match (result, zoom_result) {
            (Ok(()), Ok(())) => CommandResult::ok(""),
            (Err(error), _) | (Ok(()), Err(error)) => {
                command_target_error(error, &target, "window")
            }
        }
    }
}

/// `select-layout [-Enop] [-t target-pane] [layout-name]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct SelectLayout {
    /// `-E`: spread the panes out evenly.
    spread: bool,
    /// `-n`/`-p`: cycle to the next/previous layout.
    next: bool,
    previous: bool,
    /// `-o`: restore the layout from before the last command.
    restore: bool,
    /// `-t`: the window whose layout changes.
    target: Option<String>,
    layout: Option<String>,
}

impl SelectLayout {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            spread: args.has('E'),
            next: args.has('n'),
            previous: args.has('p'),
            restore: args.has('o'),
            target: args.value('t').map(str::to_string),
            layout: args.positionals().first().cloned(),
        })
    }

    /// Every invocation snapshots the window's layout into tmux's
    /// `w->old_layout` slot first, which is what `-o` restores; an error puts
    /// the previous snapshot back, as tmux does.
    fn execute(self, st: &mut ServerState) -> CommandResult {
        let Some(target) = self.target.clone().or_else(|| current_target(st)) else {
            return CommandResult::err("can't establish current session\n");
        };
        let resolved = match st.resolve(&target) {
            Some(resolved) => resolved,
            None => return CommandResult::err(format!("{}\n", st.pane_target_error(&target))),
        };
        st.push_zoom_at(resolved.session, resolved.window);
        let previous_old = st.snapshot_window_layout(&target).ok().flatten();
        let result = self.act(st, &target, previous_old.as_deref());
        if result.exit != 0 {
            st.restore_window_old_layout(&target, previous_old);
        }
        st.pop_zoom_at(resolved.session, resolved.window, false);
        result
    }

    fn act(&self, st: &mut ServerState, target: &str, previous_old: Option<&str>) -> CommandResult {
        if self.next || self.previous {
            return match st.cycle_layout(target, self.next) {
                Ok(()) => CommandResult::ok(""),
                Err(error) => command_target_error(error, target, "window"),
            };
        }
        if self.spread {
            return match st.spread_window_layout(target) {
                Ok(()) => CommandResult::ok(""),
                Err(error) => command_target_error(error, target, "pane"),
            };
        }
        if let Some(layout) = self.layout.as_deref() {
            let known = LAYOUT_NAMES.iter().position(|name| name == &layout);
            let valid = known.is_some()
                // A custom layout dump carries a checksum + `,`-separated cells.
                || layout.contains(',');
            if !valid {
                return CommandResult::err(format!("invalid layout: {layout}\n"));
            }
            if let Some(layout) = known {
                return match st.select_named_layout(target, layout) {
                    Ok(()) => CommandResult::ok(""),
                    Err(error) => command_target_error(error, target, "pane"),
                };
            }
            return match st.select_custom_layout(target, layout) {
                Ok(()) => CommandResult::ok(""),
                Err(error) if error.to_string() == "invalid layout" => {
                    CommandResult::err(format!("invalid layout: {layout}\n"))
                }
                Err(error) => CommandResult::err(format!("can't set layout: {error}\n")),
            };
        }
        if self.restore {
            // `-o` re-applies the layout the *previous* command snapshot; with no
            // history it is a no-op, as in tmux.
            return match previous_old {
                Some(old) => match st.select_custom_layout(target, old) {
                    Ok(()) => CommandResult::ok(""),
                    Err(error) => CommandResult::err(format!("can't set layout: {error}\n")),
                },
                None => CommandResult::ok(""),
            };
        }
        // Bare `select-layout` reapplies the last preset (tmux's `w->lastlayout`).
        match st.window_last_preset_layout(target) {
            Ok(Some(preset)) => match st.select_named_layout(target, preset) {
                Ok(()) => CommandResult::ok(""),
                Err(error) => command_target_error(error, target, "pane"),
            },
            Ok(None) => CommandResult::ok(""),
            Err(error) => command_target_error(error, target, "pane"),
        }
    }
}

/// `next-layout`/`previous-layout [-t target-window]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct CycleLayout {
    /// `-t`: the window whose layout cycles.
    target: Option<String>,
}

impl CycleLayout {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            target: args.value('t').map(str::to_string),
        })
    }

    fn execute(self, st: &mut ServerState, forward: bool) -> CommandResult {
        let Some(target) = self.target.or_else(|| current_target(st)) else {
            return CommandResult::err("can't establish current session\n");
        };
        let resolved = match st.resolve_window_target(&target) {
            Ok(resolved) => resolved,
            Err(error) => return command_target_error(error, &target, "window"),
        };
        st.push_zoom_at(resolved.session, resolved.window);
        let previous_old = st.snapshot_window_layout(&target).ok().flatten();
        let result = match st.cycle_layout(&target, forward) {
            Ok(()) => CommandResult::ok(""),
            Err(error) => {
                st.restore_window_old_layout(&target, previous_old);
                command_target_error(error, &target, "window")
            }
        };
        st.pop_zoom_at(resolved.session, resolved.window, false);
        result
    }
}

/// `list-panes [-asr] [-F format] [-f filter] [-O order] [-t target-window]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct ListPanes {
    /// `-a`: list every pane of every session.
    all: bool,
    /// `-s`: list every pane of the target session.
    session: bool,
    /// `-F`: the line each pane expands to.
    format: Option<String>,
    /// `-f`: only list panes this format is true for.
    filter: Option<String>,
    /// `-O`: the sort key, resolved when the command runs.
    order: Option<String>,
    /// `-r`: reverse the sort.
    reversed: bool,
    /// `-t`: the window whose panes are listed.
    target: Option<String>,
}

impl ListPanes {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            all: args.has('a'),
            session: args.has('s'),
            format: args.value('F').map(str::to_string),
            filter: args.value('f').map(str::to_string),
            order: args.value('O').map(str::to_string),
            reversed: args.has('r'),
            target: args.value('t').map(str::to_string),
        })
    }

    /// Lists the panes of the target window (the target's `session[:window]`,
    /// defaulting to the current session's active window) in index order. `-F`
    /// overrides the (structural) default line.
    fn execute(self, st: &ServerState, agents: &PaneAgents) -> CommandResult {
        // Structural default (tmux's includes volatile history/byte counts + %id, so
        // it isn't byte-identical; the suite pins list-panes via -F).
        let default_line = "#{pane_index}: [#{pane_active}]";

        // Each entry is (session, window Vec-position) to enumerate panes of.
        let windows: Vec<(&Session, usize)> = if self.all {
            let mut all: Vec<&Session> = st.sessions().iter().collect();
            all.sort_by(|a, b| a.name.cmp(&b.name));
            all.into_iter()
                .flat_map(|session| (0..session.windows.len()).map(move |window| (session, window)))
                .collect()
        } else if self.session {
            let target = self
                .target
                .as_deref()
                .and_then(|target| target.split(':').next())
                .filter(|session| !session.is_empty())
                .map(str::to_string)
                .or_else(|| current_session(st));
            let Some(target) = target else {
                return CommandResult::err("can't establish current session\n");
            };
            let session = match st.resolve_session(&target) {
                Some(session) => session,
                None => return CommandResult::err(format!("can't find session: {target}\n")),
            };
            (0..session.windows.len())
                .map(|window| (session, window))
                .collect()
        } else {
            let Some(target) = self.target.clone().or_else(|| current_target(st)) else {
                return CommandResult::err("can't establish current session\n");
            };
            // `list-panes -t` is a *window* target, so tmux names the window (or the
            // session) part that went missing rather than the whole target.
            let resolved = match st.resolve_window_target(&target) {
                Ok(resolved) => resolved,
                Err(error) => return CommandResult::err(format!("{error}\n")),
            };
            vec![(&st.sessions()[resolved.session], resolved.window)]
        };

        let sort_order = match list_sort_order(self.order.as_deref()) {
            Ok(order) => order,
            Err(error) => return error,
        };
        let mut panes: Vec<(&Session, usize, usize)> = windows
            .iter()
            .flat_map(|(sess, win_pos)| {
                (0..st.session_window(sess, *win_pos).panes.len())
                    .map(move |pane_idx| (*sess, *win_pos, pane_idx))
            })
            .collect();
        let pane_title = |sess: &Session, win_pos: usize, pane_idx: usize| {
            st.session_window(sess, win_pos)
                .panes
                .get(pane_idx)
                .and_then(|pane| st.pane_title(pane))
                .unwrap_or_else(format::hostname)
        };
        apply_list_sort(
            &mut panes,
            sort_order,
            self.reversed,
            |key, (sess_a, win_a, idx_a), (sess_b, win_b, idx_b)| {
                let win_a = st.session_window(sess_a, *win_a);
                let win_b = st.session_window(sess_b, *win_b);
                let (pane_a, pane_b) = (&win_a.panes[*idx_a], &win_b.panes[*idx_b]);
                match key {
                    ListSortOrder::Index => idx_a.cmp(idx_b),
                    // Pane ids are allocated in creation order, as tmux's are.
                    ListSortOrder::Creation => pane_a.id.cmp(&pane_b.id),
                    ListSortOrder::Size => {
                        let area = |win: &super::super::state::Window, id: u32| {
                            win.pane_rect(id)
                                .map(|rect| u32::from(rect.width) * u32::from(rect.height))
                                .unwrap_or(0)
                        };
                        area(win_a, pane_a.id).cmp(&area(win_b, pane_b.id))
                    }
                    ListSortOrder::Name => std::cmp::Ordering::Equal,
                    // tmux's activity key is the pane's `active_point`: least
                    // recently active first, with ties falling to the title.
                    ListSortOrder::Activity => pane_a.active_point.cmp(&pane_b.active_point),
                    _ => std::cmp::Ordering::Equal,
                }
            },
            |(sess, win_pos, pane_idx)| pane_title(sess, *win_pos, *pane_idx),
        );

        let marked = st.marked_pane();
        let mut out = String::new();
        for (sess, win_pos, pane_idx) in panes {
            let vars = vars_full(st, sess, win_pos, pane_idx, agents, marked);
            if let Some(filter) = self.filter.as_deref() {
                if !format::is_true(&expand_command_format(st, filter, &vars, None)) {
                    continue;
                }
            }
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

/// `copy-mode [-deHMqSu] [-s src-pane] [-t target-pane]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct CopyMode {
    /// `-d`: scroll a page down on entry.
    page_down: bool,
    /// `-u`: scroll a page up on entry.
    page_up: bool,
    /// `-e`: leave the mode when the bottom is reached.
    exit_at_bottom: bool,
    /// `-H`: hide the mode's position indicator.
    hide_position: bool,
    /// `-M`: begin a selection where the mouse button went down.
    mouse_drag: bool,
    /// `-S`: drag the scrollbar's slider.
    scrollbar_drag: bool,
    /// `-q`: leave the mode instead of entering it.
    quit: bool,
    /// `-s`: the pane whose contents the mode shows.
    source: Option<String>,
    /// `-t`: the pane that enters the mode.
    target: Option<String>,
}

impl CopyMode {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            page_down: args.has('d'),
            page_up: args.has('u'),
            exit_at_bottom: args.has('e'),
            hide_position: args.has('H'),
            mouse_drag: args.has('M'),
            scrollbar_drag: args.has('S'),
            quit: args.has('q'),
            source: args.value('s').map(str::to_string),
            target: args.value('t').map(str::to_string),
        })
    }

    fn execute(self, st: &mut ServerState) -> CommandResult {
        let Some(target) = self.target.or_else(|| current_target(st)) else {
            return CommandResult::err("can't establish current session\n");
        };
        if self.quit {
            return match st.set_pane_mode(&target, None) {
                Ok(()) => CommandResult::ok(""),
                Err(_) => CommandResult::err(format!("{}\n", st.pane_target_error(&target))),
            };
        }
        let source = self.source.as_deref();
        if st
            .set_copy_mode(&target, source, self.exit_at_bottom, self.hide_position)
            .is_err()
        {
            let missing = source.unwrap_or(&target);
            return CommandResult::err(format!("{}\n", st.pane_target_error(missing)));
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
        // `-M` starts a drag: the selection opens where the button went down,
        // not where the pointer has already reached (tmux's
        // `window_copy_start_drag`).
        if self.mouse_drag {
            if let Some(position) = st
                .command_mouse()
                .and_then(|mouse| mouse.pane_last_position())
            {
                let _ = st.position_copy_cursor_from_mouse(&target, position.x, position.y, vi);
                let _ = st.copy_mode_command(&target, "begin-selection", vi, &separators);
                // tmux ends `window_copy_start_drag` with one drag update, so
                // the pointer's current position is already selected.
                if let Some(now) = st.command_mouse().map(|mouse| mouse.pane_position()) {
                    st.drag_copy_selection_to_mouse(&target, now.x, now.y, vi);
                }
            }
        }
        if self.page_up {
            let _ = st.copy_mode_command(&target, "page-up", vi, &separators);
        }
        // `-S` drags the scrollbar's slider, which carries its own grab offset
        // from where the drag took hold.
        if self.scrollbar_drag {
            if let Some((row, grab)) = st.command_mouse().map(|mouse| {
                (
                    mouse.position.y,
                    mouse.target.as_ref().and_then(|t| t.slider_offset),
                )
            }) {
                let _ = st.scroll_copy_to_mouse(&target, row, grab, vi, self.exit_at_bottom);
            }
        }
        if self.page_down {
            let _ = st.copy_mode_command(&target, "page-down", vi, &separators);
        }
        CommandResult::ok("")
    }
}

/// A resolved physical row range. Rows are zero-based from the oldest retained
/// history row, matching tmux's internal grid coordinates.
#[derive(Clone, Copy, Debug)]
struct CaptureRange {
    top: usize,
    bottom: usize,
}

/// `capture-pane [-aCeFHJLMNpPqT] [-b buffer-name] [-E end-line]
/// [-S start-line] [-t target-pane]`.
///
/// The command surface and operation selection follow tmux 3.7b, and so does
/// what they read: physical rows, soft wraps, the two per-row extents, the
/// prompt/output line flags and hyperlinks all come out of the engine's port of
/// `grid.c` rather than from a tmux-shaped text dump laid over something else.
#[derive(Clone, Debug)]
pub(in crate::server) struct CapturePane {
    /// `-a`: read the screen the alternate-screen switch displaced.
    alternate: bool,
    /// `-b`: the buffer the capture is stored in.
    buffer: Option<String>,
    /// `-C`: escape non-printable characters.
    escape: bool,
    /// `-e`: include the rows' style sequences.
    styled: bool,
    /// `-E`/`-S`: the last and first row to read.
    end: Option<String>,
    start: Option<String>,
    /// `-F`: prefix each row with its flags.
    line_flags: bool,
    /// `-H`: capture the rows' hyperlinks instead of their text.
    hyperlinks: bool,
    /// `-J`: join wrapped rows, keeping their trailing blanks.
    join: bool,
    /// `-L`: prefix each row with its number.
    line_numbers: bool,
    /// `-M`: read the screen copy mode froze.
    mode: bool,
    /// `-N`: keep the blanks trailing each row.
    keep_trailing: bool,
    /// `-p`: print the capture instead of storing it in a buffer.
    print: bool,
    /// `-P`: capture what the pane's parser is part-way through.
    pending: bool,
    /// `-q`: an absent alternate screen is an empty capture rather than a
    /// failure.
    quiet: bool,
    /// `-T`: read each row to its written extent.
    written_extent: bool,
    /// `-t`: the pane to capture.
    target: Option<String>,
}

impl CapturePane {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            alternate: args.has('a'),
            buffer: args.value('b').map(str::to_string),
            escape: args.has('C'),
            styled: args.has('e'),
            end: args.value('E').map(str::to_string),
            start: args.value('S').map(str::to_string),
            line_flags: args.has('F'),
            hyperlinks: args.has('H'),
            join: args.has('J'),
            line_numbers: args.has('L'),
            mode: args.has('M'),
            keep_trailing: args.has('N'),
            print: args.has('p'),
            pending: args.has('P'),
            quiet: args.has('q'),
            written_extent: args.has('T'),
            target: args.value('t').map(str::to_string),
        })
    }

    fn execute(self, st: &mut ServerState, agents: &PaneAgents) -> CommandResult {
        let Some(target) = self.target.clone().or_else(|| current_target(st)) else {
            return CommandResult::err("can't establish current session\n");
        };

        let resolved = match st.resolve(&target) {
            Some(resolved) => resolved,
            None => return CommandResult::err(format!("{}\n", st.pane_target_error(&target))),
        };

        // `-P` returns the bytes the pane's tokenizer is part-way through, tmux's
        // `input_pending`. `-H` takes precedence and continues to the grid
        // hyperlink operation, as in tmux.
        if self.pending && !self.hyperlinks {
            let pending = st.window(resolved.session, resolved.window).panes[resolved.pane]
                .pane
                .pending_input();
            let pending = if self.escape {
                capture_escape_pending(&pending)
            } else {
                pending
            };
            return self.finish(st, pending);
        }

        // `-a` reads the screen the alternate-screen switch displaced. With no
        // alternate screen up there is nothing to read and tmux fails, which `-q`
        // turns into an empty capture.
        let inactive = if self.alternate {
            let snapshot = st.window(resolved.session, resolved.window).panes[resolved.pane]
                .pane
                .inactive_snapshot();
            match snapshot {
                Some(snapshot) => Some(snapshot),
                None if self.quiet => return self.finish(st, Vec::new()),
                None => return CommandResult::err("no alternate screen\n"),
            }
        } else {
            None
        };

        let mut vars = vars_full(
            st,
            &st.sessions()[resolved.session],
            resolved.window,
            resolved.pane,
            agents,
            st.marked_pane(),
        );
        for (name, value) in st.env_iter() {
            vars.set(name.to_string(), value);
        }
        if let Ok(entries) = st.format_option_entries(&target) {
            for (name, value) in entries {
                vars.set(name.to_string(), value);
            }
        }
        let history_limit = st
            .option_for_target(&target, "history-limit")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(2000);
        let styled = self.styled && !self.hyperlinks;
        // The snapshot walk is priced per cell, so the range is decided first —
        // from the frozen copy-mode grid when `-M` selects one, otherwise from the
        // grid's row geometry alone — and only the rows inside it are snapshotted.
        let text = {
            let node = &st.window(resolved.session, resolved.window).panes[resolved.pane];
            // Two reads arrive already materialized as a whole grid plus its `-e`
            // bytes: the screen copy mode froze, and the one the alternate screen
            // displaced. Neither is the live grid, and both are served the same way.
            let whole = inactive
                .as_ref()
                .map(|snapshot| (&snapshot.grid, &snapshot.vt))
                .or_else(|| match self.mode {
                    true => node.copy.as_ref().map(|copy| (&copy.grid, &copy.vt)),
                    false => None,
                });
            if let Some((grid, vt)) = whole {
                if grid.rows.is_empty() {
                    String::new()
                } else {
                    let range = self.range(
                        grid.rows.len(),
                        grid.scrollback_rows,
                        grid.viewport_rows,
                        &vars,
                        history_limit,
                    );
                    let styled_rows = if styled {
                        let all = capture_vt_normalize_rows(vt, grid.rows.len());
                        Some(all[range.top..=range.bottom].to_vec())
                    } else {
                        None
                    };
                    self.serialize(grid, 0, range, styled_rows.as_deref())
                }
            } else {
                let dims = node.pane.grid_dims();
                if dims.total_rows == 0 {
                    String::new()
                } else {
                    let range = self.range(
                        dims.total_rows,
                        dims.scrollback_rows,
                        dims.viewport_rows,
                        &vars,
                        history_limit,
                    );
                    let rows = range.bottom - range.top + 1;
                    let grid = node.pane.grid_snapshot_range(range.top, rows);
                    let styled_rows = styled.then(|| {
                        let bytes = node.pane.dump_rows_vt(range.top, rows, self.extent());
                        capture_vt_normalize_rows(&bytes, rows)
                    });
                    self.serialize(&grid, range.top, range, styled_rows.as_deref())
                }
            }
        };
        self.finish(st, text.into_bytes())
    }

    /// Deliver a capture, which is bytes rather than text: `-P` returns whatever
    /// the pane's parser is holding, and a half-read UTF-8 character in an OSC
    /// payload is not a string.
    fn finish(&self, st: &mut ServerState, mut bytes: Vec<u8>) -> CommandResult {
        if self.print {
            // tmux always prints one terminating newline, including for an empty
            // capture. Row serializers already end in one, so do not add a second.
            if !bytes.ends_with(b"\n") {
                bytes.push(b'\n');
            }
            return CommandResult::ok_bytes(bytes);
        }
        // Buffer captures retain the row serializer's final newline. An empty
        // parser-pending or hyperlink capture stores a genuinely empty buffer.
        st.set_buffer(self.buffer.as_deref(), &bytes);
        CommandResult::ok("")
    }

    fn range(
        &self,
        total_rows: usize,
        scrollback_rows: usize,
        viewport_rows: u16,
        vars: &format::Vars,
        history_limit: usize,
    ) -> CaptureRange {
        let last = total_rows.saturating_sub(1);
        let history = scrollback_rows.min(last);
        // History past `history-limit` is gone in tmux, so it must not be readable
        // here either; the screen can hold more rows than the limit between
        // re-pushes, so the limit is applied again where the rows are read back.
        let floor = history.saturating_sub(history_limit);
        let default_top = history;
        let default_bottom = history
            .saturating_add(viewport_rows.saturating_sub(1) as usize)
            .min(last);

        let mut top =
            capture_endpoint(self.start.as_deref(), default_top, history, last, vars).max(floor);
        let mut bottom =
            capture_endpoint(self.end.as_deref(), default_bottom, history, last, vars).max(floor);
        if bottom < top {
            std::mem::swap(&mut top, &mut bottom);
        }
        CaptureRange { top, bottom }
    }

    /// Serialize the rows of `range`. `grid.rows[0]` is physical row `start_row`,
    /// so a range-limited snapshot indexes relative to it while `-L` numbering and
    /// the range itself stay in physical-row terms.
    fn serialize(
        &self,
        grid: &Grid,
        start_row: usize,
        range: CaptureRange,
        styled_rows: Option<&[String]>,
    ) -> String {
        let mut seen_links = std::collections::HashSet::new();
        let mut out = String::new();

        for (relative, row_index) in (range.top..=range.bottom).enumerate() {
            let row = &grid.rows[row_index - start_row];
            let mut line = if self.hyperlinks {
                // tmux checks the row's flag before walking it and stops at the
                // written extent, so a link in a cell nothing wrote into is not
                // reported.
                let mut links = Vec::new();
                if row.flags.hyperlink {
                    for cell in row.cells.iter().take(row.used.min(row.cells.len())) {
                        // Identity is the screen's, not the URI's: tmux compares
                        // `gc.link`, so a second anonymous OSC 8 naming an address
                        // already listed is a second link and is listed again.
                        let Some(link) = cell.hyperlink.as_ref() else {
                            continue;
                        };
                        // `cmd_capture_pane_hyperlinks` stops at one link per
                        // column of the grid, for the whole capture rather than
                        // per row.
                        if seen_links.len() == grid.cols as usize {
                            break;
                        }
                        if seen_links.insert(cell.hyperlink_slot) {
                            links.push(link.clone());
                        }
                    }
                }
                if links.is_empty() {
                    continue;
                }
                links.join(" ")
            } else if let Some(styled) = styled_rows {
                let mut styled = styled.get(relative).cloned().unwrap_or_default();
                self.trim_trailing(&mut styled);
                styled
            } else {
                self.plain_row(row)
            };
            if self.escape && !self.hyperlinks {
                line = capture_escape(&line);
            }
            if self.line_numbers {
                let number = row_index as isize - grid.scrollback_rows as isize;
                out.push_str(&format!("{number} "));
            }
            if self.line_flags {
                out.push_str(&capture_row_flags(row));
                out.push(' ');
            }
            out.push_str(&line);
            if !self.join || !row.wrapped {
                out.push('\n');
            }
        }
        out
    }

    /// How far along each row this capture reads, tmux's
    /// `GRID_STRING_EMPTY_CELLS`: to the written extent when `-J` or `-T` asked
    /// for it, and to the allocated one otherwise.
    fn extent(&self) -> CaptureExtent {
        if self.join || self.written_extent {
            CaptureExtent::Written
        } else {
            CaptureExtent::Allocated
        }
    }

    /// One row as text, following `grid_string_cells`'s two independent
    /// decisions: how far along the row to read, and whether to trim what
    /// trails.
    fn plain_row(&self, row: &GridRow) -> String {
        let end = match self.extent() {
            CaptureExtent::Written => row.used,
            CaptureExtent::Allocated => row.size,
        }
        .min(row.cells.len());

        let mut line = String::new();
        for cell in row.cells.iter().take(end) {
            if matches!(cell.width, CellWidth::SpacerTail) {
                continue;
            }
            if cell.tab {
                line.push('\t');
            } else if cell.text.is_empty() {
                line.push(' ');
            } else {
                line.push_str(&cell.text);
            }
        }
        self.trim_trailing(&mut line);
        line
    }

    /// tmux's `GRID_STRING_TRIM_SPACES`: a capture drops the blanks trailing a
    /// row unless `-J` or `-N` asked to keep them.
    ///
    /// It trims spaces and nothing else, so anything written just before them
    /// survives. That is how a `-e` capture can end in a style change with no
    /// text left to style: the row was read into its allocated blanks, the
    /// transition into those blanks was emitted, and only the blanks went.
    fn trim_trailing(&self, line: &mut String) {
        if self.join || self.keep_trailing {
            return;
        }
        line.truncate(line.trim_end_matches(' ').len());
    }
}

/// `capture-pane -PC`, which escapes by its own rule rather than the one grid
/// rows use: tmux writes a byte literally only when it is at least a space and
/// not a backslash, and everything else as a three-digit octal escape. The
/// comparison is against a signed `char`, which puts every byte with the high
/// bit set on the escaped side.
fn capture_escape_pending(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    for &byte in bytes {
        if (b' '..0x80).contains(&byte) && byte != b'\\' {
            out.push(byte);
        } else {
            out.extend_from_slice(format!("\\{byte:03o}").as_bytes());
        }
    }
    out
}

fn capture_endpoint(
    value: Option<&str>,
    default: usize,
    history: usize,
    last: usize,
    vars: &format::Vars,
) -> usize {
    let Some(value) = value else {
        return default;
    };
    if value == "-" {
        return if default == history { 0 } else { last };
    }
    let expanded = format::expand(value, vars);
    let Ok(offset) = expanded.parse::<i32>() else {
        return default;
    };
    if offset > i16::MAX as i32 {
        return default;
    }
    if offset < 0 && offset.unsigned_abs() as usize > history {
        return 0;
    }
    history.saturating_add_signed(offset as isize).min(last)
}

/// The `-F` flags, in tmux's order. `D` is absent because no grid a capture can
/// reach holds a dead line, and `X` is an allocation decision tmux lets show.
fn capture_row_flags(row: &GridRow) -> String {
    let mut flags = String::new();
    for (present, flag) in [
        (row.flags.hyperlink, 'H'),
        (row.flags.start_output, 'O'),
        (row.flags.start_prompt, 'P'),
        (row.wrapped, 'W'),
        (row.flags.extended, 'X'),
    ] {
        if present {
            flags.push(flag);
        }
    }
    if flags.is_empty() {
        flags.push('-');
    }
    flags
}

fn capture_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '\\' => out.push_str("\\\\"),
            '\t' => out.push('\t'),
            ' '..='~' => out.push(character),
            character if !character.is_ascii() => out.push(character),
            character => out.push_str(&format!("\\{:03o}", character as u32)),
        }
    }
    out
}

#[derive(Clone, Copy)]
enum CaptureToken<'a> {
    Text(&'a [u8]),
    Sgr(&'a [u8]),
    Osc(&'a [u8]),
    Acs(bool),
    Other,
}

pub(super) fn capture_vt_normalize_rows(bytes: &[u8], rows: usize) -> Vec<String> {
    let mut out = Vec::with_capacity(rows);
    let mut decoder = SgrDecoder::default();
    let mut writer = CaptureStyleWriter::default();
    let mut presentation = CellPresentation::default();
    let mut start = 0usize;
    for end in bytes
        .iter()
        .enumerate()
        .filter_map(|(index, byte)| (*byte == b'\n').then_some(index))
    {
        let row = bytes[start..end]
            .strip_suffix(b"\r")
            .unwrap_or(&bytes[start..end]);
        out.push(capture_vt_normalize_row(
            row,
            &mut decoder,
            &mut writer,
            &mut presentation,
        ));
        start = end + 1;
        if out.len() == rows {
            return out;
        }
    }
    if out.len() < rows && start < bytes.len() {
        let row = bytes[start..]
            .strip_suffix(b"\r")
            .unwrap_or(&bytes[start..]);
        out.push(capture_vt_normalize_row(
            row,
            &mut decoder,
            &mut writer,
            &mut presentation,
        ));
    }
    out.resize(rows, String::new());
    out
}

fn capture_vt_normalize_row(
    bytes: &[u8],
    decoder: &mut SgrDecoder,
    writer: &mut CaptureStyleWriter,
    presentation: &mut CellPresentation,
) -> String {
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != 0x1b {
            if matches!(bytes[index], 0x0e | 0x0f) {
                tokens.push(CaptureToken::Acs(bytes[index] == 0x0e));
                index += 1;
                continue;
            }
            let start = index;
            while index < bytes.len()
                && bytes[index] != 0x1b
                && !matches!(bytes[index], 0x0e | 0x0f)
            {
                index += 1;
            }
            tokens.push(CaptureToken::Text(&bytes[start..index]));
            continue;
        }
        if bytes.get(index + 1) == Some(&b'[') {
            let start = index;
            index += 2;
            while index < bytes.len() && !(0x40..=0x7e).contains(&bytes[index]) {
                index += 1;
            }
            if index < bytes.len() {
                let final_byte = bytes[index];
                index += 1;
                if final_byte == b'm' {
                    tokens.push(CaptureToken::Sgr(&bytes[start + 2..index - 1]));
                } else {
                    tokens.push(CaptureToken::Other);
                }
            }
            continue;
        }
        if bytes.get(index + 1) == Some(&b']') {
            index += 2;
            let content = index;
            while index < bytes.len() {
                if bytes[index] == 0x07 {
                    index += 1;
                    break;
                }
                if bytes[index] == 0x1b && bytes.get(index + 1) == Some(&b'\\') {
                    index += 2;
                    break;
                }
                index += 1;
            }
            let end = if bytes.get(index.wrapping_sub(1)) == Some(&0x07) {
                index - 1
            } else {
                index.saturating_sub(2)
            };
            tokens.push(CaptureToken::Osc(&bytes[content..end]));
            continue;
        }
        // The grid dump selects the line-drawing set with `ESC ( 0` and ASCII
        // back with `ESC ( B`; a capture spells the same run SO … SI, so both
        // ends of it have to be recognized — a missed `ESC ( B` would leave the
        // `B` in the captured text and the run open to the end of the row.
        if bytes.get(index + 1) == Some(&b'(') && matches!(bytes.get(index + 2), Some(b'0' | b'B'))
        {
            let acs = bytes[index + 2] == b'0';
            index += 3;
            tokens.push(CaptureToken::Acs(acs));
        } else {
            index = (index + 2).min(bytes.len());
            tokens.push(CaptureToken::Other);
        }
    }

    let Some(last_text) = tokens.iter().rposition(|token| {
        matches!(
            token,
            CaptureToken::Text(text)
                if text.iter().any(|byte| !matches!(byte, b'\r' | b'\n'))
        )
    }) else {
        for token in tokens {
            apply_capture_control(token, decoder, presentation);
        }
        return String::new();
    };

    // Every cell of the dump writes text, so the only sequences past the last
    // text are the row's closing ones: the sequences the last cell needed,
    // repeated, and then the OSC 8 that closes the row's link. The repeat is
    // there exactly when the last cell is where the last transition happened,
    // which is the one thing this pass cannot see for itself — it works in
    // runs, not cells. So the dump says *whether* to repeat and the sequences
    // themselves are re-derived here, in the capture's own spelling.
    let repeat_last_code = tokens.len() > last_text + 2;

    let mut out = Vec::new();
    for (token_index, token) in tokens.iter().enumerate() {
        match token {
            CaptureToken::Text(text) if token_index <= last_text => {
                presentation.style = decoder.style();
                writer.transition(&mut out, presentation);
                out.extend_from_slice(text);
            }
            token => apply_capture_control(*token, decoder, presentation),
        }
    }
    writer.finish_row(&mut out, repeat_last_code);
    String::from_utf8_lossy(&out).into_owned()
}

fn apply_capture_control(
    token: CaptureToken<'_>,
    decoder: &mut SgrDecoder,
    presentation: &mut CellPresentation,
) {
    match token {
        CaptureToken::Sgr(parameters) => decoder.apply(parameters),
        CaptureToken::Osc(content) => {
            let Some(rest) = content.strip_prefix(b"8;") else {
                return;
            };
            let Some(separator) = rest.iter().position(|byte| *byte == b';') else {
                return;
            };
            let parameters = String::from_utf8_lossy(&rest[..separator]);
            let uri = String::from_utf8_lossy(&rest[separator + 1..]).into_owned();
            if uri.is_empty() {
                presentation.hyperlink = None;
                presentation.hyperlink_epoch = 0;
            } else {
                presentation.hyperlink = Some(Hyperlink {
                    id: parameters
                        .split(':')
                        .find_map(|field| field.strip_prefix("id=").map(str::to_string))
                        .unwrap_or_default(),
                    uri,
                });
                // The dump writes an OSC 8 exactly where the cell's link
                // changes, so counting them is what tells two anonymous links
                // naming one URI apart.
                presentation.hyperlink_epoch = presentation.hyperlink_epoch.wrapping_add(1);
            }
        }
        CaptureToken::Acs(value) => presentation.acs = value,
        CaptureToken::Text(_) | CaptureToken::Other => {}
    }
    presentation.style = decoder.style();
}
