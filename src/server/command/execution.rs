//! The commands that wait: for a shell, for a file, for a channel.
//!
//! Each drives its own wait. The command queue is the task they run on, so a
//! wait here is an `await` in the command's own body rather than a suspension
//! reported up to a driver; the state borrow is taken in tight scopes around
//! it, never across it.

use std::time::Duration;

use super::*;

#[derive(Clone, Debug)]
pub(in crate::server) enum Command {
    RunShell(RunShell),
    IfShell(IfShell),
    SourceFile(SourceFile),
    WaitFor(WaitFor),
}

impl Command {
    pub(super) async fn execute(self, context: &mut ExecContext<'_>) -> SharedCommandExecution {
        match self {
            Self::RunShell(command) => command.execute(context).await,
            Self::IfShell(command) => command.execute(context).await,
            Self::SourceFile(command) => command.execute(context).await,
            Self::WaitFor(command) => command.execute(context).await,
        }
    }
}

// ---- run-shell -------------------------------------------------------------

/// `run-shell [-bCE] [-c start-directory] [-d delay] [-t target-pane]
/// [shell-command [argument ...]]`.
///
/// The fields the child needs are read by [`suspend`], which spawns it.
#[derive(Clone, Debug)]
pub(crate) struct RunShell {
    /// `-b`: run the child as a detached job instead of waiting for it.
    background: bool,
    /// `-C`: run the argument as tmux commands, with no child at all.
    as_commands: bool,
    /// `-E`: include the child's standard error in the report.
    pub(super) stderr: bool,
    /// `-c`: the child's working directory.
    pub(super) cwd: Option<String>,
    /// `-d`: how long to wait before starting the child.
    delay: Option<String>,
    /// `-t`: the pane the child's output is written into.
    pub(super) target: Option<String>,
    /// The command line the child runs.
    pub(super) command: Option<String>,
    /// Arguments exposed to the command's format expansion as `#{1}`, `#{2}`,
    /// and so on.
    operands: Vec<String>,
}

impl RunShell {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        let positionals = args.positionals();
        Ok(Self {
            background: args.has('b'),
            as_commands: args.has('C'),
            stderr: args.has('E'),
            cwd: args.value('c').map(str::to_string),
            delay: args.value('d').map(str::to_string),
            target: args.value('t').map(str::to_string),
            command: positionals.first().cloned(),
            operands: positionals.iter().skip(1).cloned().collect(),
        })
    }

    fn expand_command(&mut self, context: &ExecContext<'_>) {
        if self.as_commands {
            return;
        }
        let Some(command) = self.command.clone() else {
            return;
        };
        let target = self.target.clone();
        let operands = self.operands.clone();
        let mut state = context.state().borrow_mut();
        let previous = install_command_target_context(&mut state, context.client());
        self.command = Some(expand_run_shell_command(
            &command,
            target.as_deref(),
            &state,
            context.agents(),
            &operands,
        ));
        restore_command_target_context(&mut state, previous);
    }

    /// How long the job waits before it starts, from `-d`.
    pub(super) fn delay(&self) -> Result<Duration, CommandResult> {
        let Some(value) = self.delay.as_deref() else {
            return Ok(Duration::ZERO);
        };
        let seconds = value
            .parse::<f64>()
            .ok()
            .filter(|seconds| seconds.is_finite() && *seconds >= 0.0)
            .ok_or_else(|| CommandResult::err(format!("invalid delay time: {value}\n")))?;
        Ok(Duration::from_secs_f64(seconds))
    }

    /// Resolve `-t` to the pane it names right now.
    ///
    /// The child's output is written when the child finishes, by which time the
    /// target may name a different pane — or none. tmux resolves it up front,
    /// so pin it to the pane id here.
    fn pin_view_target(&mut self, state: &ServerState) {
        let Some(resolved) = self
            .target
            .as_deref()
            .and_then(|target| state.resolve(target))
        else {
            return;
        };
        let pane_id = state.window(resolved.session, resolved.window).panes[resolved.pane].id;
        self.target = Some(format!("%{pane_id}"));
    }

    async fn execute(mut self, context: &mut ExecContext<'_>) -> SharedCommandExecution {
        self.expand_command(context);
        if self.as_commands && self.background {
            // `-C` never runs a child, `-b` or not (`cmd_run_shell_timer` only
            // reaches `job_run` when there is no prepared command list). With
            // `-b` the line is appended to the client's queue rather than
            // spliced in here, so the rest of this command line runs first.
            let mut result = CommandResult::ok("");
            if let Some(line) = self.command.clone() {
                result
                    .background_commands
                    .push(BackgroundCommandRequest::Command {
                        command: LazyCommand::Line(line),
                        context: context.client().clone(),
                    });
            }
            return SharedCommandExecution::completed(result);
        }
        if self.background {
            let (command, jobs) = {
                let state = context.state().borrow_mut();
                let mut command = self;
                command.pin_view_target(&state);
                let jobs = state.background_job_registry();
                (command, jobs)
            };
            let mut result = CommandResult::ok("");
            result
                .background_commands
                .push(BackgroundCommandRequest::RunShell {
                    command,
                    context: context.client().clone(),
                    jobs,
                });
            return SharedCommandExecution::completed(result);
        }
        if self.as_commands {
            // `-C` runs its argument as tmux commands, which the queue takes
            // over as items of its own; the command's own hooks wait for them.
            let Some(line) = self.command.as_deref() else {
                return SharedCommandExecution::completed(CommandResult::ok(""));
            };
            return match context.plan_nested_command_line(line, NestedCapture::Inserted) {
                Ok(commands) => {
                    let mut insert_next = Vec::new();
                    if !commands.is_empty() {
                        insert_next.push(commands);
                    }
                    insert_next.push(context.finalize_hooks("run-shell"));
                    SharedCommandExecution {
                        result: CommandResult::ok(""),
                        insert_next,
                        defer_success_hooks: true,
                    }
                }
                Err(result) => SharedCommandExecution::completed(result),
            };
        }
        let (command, job_context) = {
            let state = context.state().borrow_mut();
            let mut command = self;
            command.pin_view_target(&state);
            let job_context = context.client().with_job_environment(&state);
            (command, job_context)
        };
        let completion = suspend::run_shell(context.tasks(), command, job_context).await;
        let result = context.run_sync(context.client(), |inner| {
            finish_run_shell(completion, inner.state)
        });
        SharedCommandExecution::completed(result)
    }
}

/// Deliver a finished job's report: into the pane `-t` named, or — with that
/// pane gone — back to the client that asked, the way `cmdq_print` does without
/// a pane to draw into.
fn finish_run_shell(completion: RunShellCompletion, state: &mut ServerState) -> CommandResult {
    let mut result = completion.result;
    if let Some((target, output)) = completion.view {
        if state.append_view_output(&target, &output).is_err() {
            result.stdout.push_str(&String::from_utf8_lossy(&output));
        }
    }
    result
}

// ---- if-shell --------------------------------------------------------------

/// `if-shell [-bF] [-t target-pane] shell-command command [command]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct IfShell {
    /// `-b`: run the condition as a detached job. A `-F` condition has no job,
    /// so tmux's `-F` path never consults this flag and neither does ours.
    background: bool,
    /// `-F`: the condition is a format, not a shell command.
    format: bool,
    /// `-t`: the pane a `-F` condition is expanded against.
    target: Option<String>,
    /// The condition, then the branch taken when it holds, then the other one.
    operands: Vec<String>,
}

impl IfShell {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            background: args.has('b'),
            format: args.has('F'),
            target: args.value('t').map(str::to_string),
            operands: args.positionals().to_vec(),
        })
    }

    fn condition(&self) -> Option<&str> {
        self.operands.first().map(String::as_str)
    }

    fn branch(&self, matched: bool) -> Option<&str> {
        self.operands
            .get(if matched { 1 } else { 2 })
            .map(String::as_str)
    }

    /// Expand a `-F` condition and read tmux's truth rule off the result.
    fn matches_format(&self, condition: &str, context: &ExecContext<'_>) -> bool {
        let mut state = context.state().borrow_mut();
        let previous = install_command_target_context(&mut state, context.client());
        let expanded = expand_if_cond(condition, self.target.as_deref(), &state, context.agents());
        restore_command_target_context(&mut state, previous);
        format::is_true_first_byte(&expanded)
    }

    /// Queue the branch the condition picked, and hold the command's own hooks
    /// back until it has run.
    fn plan_branch(&self, matched: bool, context: &ExecContext<'_>) -> SharedCommandExecution {
        let Some(branch) = self.branch(matched) else {
            return SharedCommandExecution::completed(CommandResult::ok(""));
        };
        let commands = match context.plan_nested_command_line(branch, NestedCapture::Inserted) {
            Ok(commands) => commands,
            Err(error) => return SharedCommandExecution::completed(error),
        };
        let mut insert_next = Vec::new();
        if !commands.is_empty() {
            insert_next.push(commands);
        }
        insert_next.push(context.finalize_hooks("if-shell"));
        SharedCommandExecution {
            result: CommandResult::ok(""),
            insert_next,
            defer_success_hooks: true,
        }
    }

    async fn execute(self, context: &mut ExecContext<'_>) -> SharedCommandExecution {
        let Some(condition) = self.condition() else {
            return SharedCommandExecution::completed(CommandResult::err(
                "if-shell: too few arguments\n",
            ));
        };
        if self.format {
            let matched = self.matches_format(condition, context);
            return self.plan_branch(matched, context);
        }
        let condition = {
            let mut state = context.state().borrow_mut();
            let previous = install_command_target_context(&mut state, context.client());
            let expanded =
                expand_if_cond(condition, self.target.as_deref(), &state, context.agents());
            restore_command_target_context(&mut state, previous);
            expanded
        };
        if self.background {
            let request = BackgroundCommandRequest::IfShell {
                condition,
                then_command: self.branch(true).map(str::to_string),
                else_command: self.branch(false).map(str::to_string),
                context: context.client().clone(),
            };
            let mut result = CommandResult::ok("");
            result.background_commands.push(request);
            return SharedCommandExecution::completed(result);
        }
        let job_context = {
            let state = context.state().borrow_mut();
            context.client().with_job_environment(&state)
        };
        let matched = suspend::if_shell(context.tasks(), condition, job_context).await;
        self.plan_branch(matched, context)
    }
}

/// Expand a condition — or a `source-file -F` path — against the pane `-t`
/// names, falling back to the current one.
pub(super) fn expand_if_cond(
    cond: &str,
    requested_target: Option<&str>,
    st: &ServerState,
    agents: &PaneAgents,
) -> String {
    expand_target_format(cond, requested_target, st, agents, &[])
}

pub(super) fn expand_run_shell_command(
    command: &str,
    requested_target: Option<&str>,
    st: &ServerState,
    agents: &PaneAgents,
    operands: &[String],
) -> String {
    let positional = operands
        .iter()
        .enumerate()
        .map(|(index, value)| ((index + 1).to_string(), value.clone()))
        .collect::<Vec<_>>();
    expand_target_format(command, requested_target, st, agents, &positional)
}

fn expand_target_format(
    source: &str,
    requested_target: Option<&str>,
    st: &ServerState,
    agents: &PaneAgents,
    extra: &[(String, String)],
) -> String {
    // Nothing here reads the table when the source names no variable, and the
    // positional operands are the only entries a caller can observe missing —
    // `#{1}` is itself a `#`.
    if !format::reads_vars(source) {
        return source.to_string();
    }
    let target = requested_target
        .map(str::to_string)
        .or_else(|| current_target(st));
    if let Some(target) = target {
        if let Some(resolved) = st.resolve(&target) {
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
            for (name, value) in extra {
                vars.set(name.clone(), value.clone());
            }
            let loops = TreeLoops {
                st,
                session: resolved.session,
                window: resolved.window,
                agents,
            };
            return expand_command_format(st, source, &vars, Some(&loops));
        }
    }
    let mut vars = Vars::default();
    for (name, value) in extra {
        vars.set(name.clone(), value.clone());
    }
    expand_command_format(st, source, &vars, None)
}

// ---- source-file -----------------------------------------------------------

/// `source-file [-Fnqv] [-t target-pane] path ...`.
#[derive(Clone, Debug)]
pub(in crate::server) struct SourceFile {
    /// `-F`: the paths are formats.
    expand: bool,
    /// `-n`: parse the files without running what they hold.
    parse_only: bool,
    /// `-q`: a path that cannot be read is not an error.
    quiet: bool,
    /// `-v`: echo every command as it is parsed.
    verbose: bool,
    /// `-t`: the pane a `-F` path is expanded against.
    target: Option<String>,
    paths: Vec<String>,
}

/// How deep `source-file` may nest before tmux gives up.
const SOURCE_FILE_DEPTH_LIMIT: u8 = 50;

impl SourceFile {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            expand: args.has('F'),
            parse_only: args.has('n'),
            quiet: args.has('q'),
            verbose: args.has('v'),
            target: args.value('t').map(str::to_string),
            paths: args.positionals().to_vec(),
        })
    }

    /// The paths to read, with `-F` expanded against the command's target.
    fn resolved_paths(&self, context: &ExecContext<'_>) -> Vec<String> {
        if !self.expand {
            return self.paths.clone();
        }
        self.paths
            .iter()
            .map(|raw_path| {
                let mut state = context.state().borrow_mut();
                let previous = install_command_target_context(&mut state, context.client());
                let path =
                    expand_if_cond(raw_path, self.target.as_deref(), &state, context.agents());
                restore_command_target_context(&mut state, previous);
                path
            })
            .collect()
    }

    async fn execute(self, context: &mut ExecContext<'_>) -> SharedCommandExecution {
        if context.source_depth() >= SOURCE_FILE_DEPTH_LIMIT {
            return SharedCommandExecution::completed(CommandResult::err(
                "too many nested files\n",
            ));
        }
        let paths = self.resolved_paths(context);
        let reads = suspend::source_file(context.tasks(), paths).await;
        self.queue_contents(reads, context)
    }

    /// Turn what each path held into queue items behind this command.
    fn queue_contents(
        &self,
        reads: Vec<SourceFileRead>,
        context: &ExecContext<'_>,
    ) -> SharedCommandExecution {
        let state = context.state();
        let mut out = CommandResult::ok("");
        let mut insert_next = Vec::new();
        for SourceFileRead {
            path,
            contents,
            existed,
        } in reads
        {
            match contents {
                Ok(contents) => {
                    let mut file_insertions = Vec::new();
                    let mut file_parse_error = false;
                    let environment = {
                        let state = state.borrow_mut();
                        state
                            .env_iter()
                            .map(|(name, value)| (name.clone(), value.clone()))
                            .collect::<BTreeMap<_, _>>()
                    };
                    let parsed = match source_lines(&contents, &environment) {
                        Ok(parsed) => parsed,
                        Err((line, message)) => {
                            out.stderr.push_str(&format!("{path}:{line}: {message}\n"));
                            out.exit = 1;
                            out.continue_queue = true;
                            continue;
                        }
                    };
                    if !self.parse_only {
                        let mut state = state.borrow_mut();
                        for (name, value, hidden) in &parsed.assignments {
                            if *hidden {
                                state.set_hidden_env(name, value);
                            } else {
                                state.set_env(name, value);
                            }
                        }
                    }
                    for (line_number, line) in parsed.lines {
                        if self.verbose
                            && !matches!(context.client().kind, ClientKind::Control { .. })
                        {
                            out.stdout.push_str(&format!(
                                "{path}:{line_number}: {}\n",
                                source_verbose_line(&line)
                            ));
                        }
                        let owned_groups = tokenized_command_groups(&line);
                        let groups = owned_groups.iter().map(Vec::as_slice).collect::<Vec<_>>();
                        // `-n` only checks the file, so it compiles without the
                        // alias table: an alias is a runtime replacement, not
                        // part of the file's own syntax.
                        let aliases = if self.parse_only {
                            Vec::new()
                        } else {
                            let state = state.borrow_mut();
                            state.command_aliases()
                        };
                        match ExecutableCommand::compile_groups(groups, &aliases)
                            .map(ExecutableCommand::into_commands)
                            .map_err(CommandResult::err)
                        {
                            Ok(parsed) if !self.parse_only && !parsed.is_empty() => {
                                let source_line = line_number;
                                file_insertions.push(
                                    parsed
                                        .into_iter()
                                        .map(|command| SharedQueueItem::Command {
                                            command,
                                            source: Some(SourceLocation {
                                                path: path.clone(),
                                                line: source_line,
                                            }),
                                            source_depth: context.source_depth() + 1,
                                            contributes_status: true,
                                        })
                                        .collect(),
                                );
                            }
                            Ok(_) => {}
                            Err(result) => {
                                file_parse_error = true;
                                let diagnostic = result.stderr.trim_end();
                                let location = format!("{path}:{line_number}");
                                let diagnostic = if matches!(
                                    line.iter().find_map(LineToken::word),
                                    Some("if" | "if-shell")
                                ) {
                                    format!("{location}: {location}: {diagnostic}")
                                } else {
                                    format!("{location}: {diagnostic}")
                                };
                                out.stdout.push_str(&diagnostic);
                                out.stdout.push('\n');
                                out.exit = 1;
                                out.continue_queue = true;
                                if !self.parse_only {
                                    let mut state = state.borrow_mut();
                                    state.push_config_error(diagnostic);
                                }
                            }
                        }
                    }
                    if !self.parse_only && !file_parse_error {
                        insert_next.extend(file_insertions);
                    }
                }
                Err(_) if self.quiet => {}
                Err(error) => {
                    out.stderr
                        .push_str(&format!("{}: {}\n", io_error_message(&error), path));
                    out.exit = 1;
                    // tmux has already returned WAIT once glob expansion found the
                    // path. A later read failure updates the client status, then
                    // resumes the original queue group.
                    out.continue_queue |= existed;
                }
            }
        }
        let defer_success_hooks = !self.parse_only && (out.exit == 0 || out.continue_queue);
        if defer_success_hooks {
            insert_next.push(vec![SharedQueueItem::FinalizeSource {
                args: context.args().to_vec(),
            }]);
        }
        SharedCommandExecution {
            result: out,
            insert_next,
            defer_success_hooks,
        }
    }
}

// ---- wait-for --------------------------------------------------------------

/// `wait-for [-L|-S|-U] channel`.
#[derive(Clone, Debug)]
pub(crate) struct WaitFor {
    /// `-L`: take the channel's lock.
    lock: bool,
    /// `-S`: signal everything waiting on the channel.
    signal: bool,
    /// `-U`: release the channel's lock.
    unlock: bool,
    channel: String,
}

impl WaitFor {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            lock: args.has('L'),
            signal: args.has('S'),
            unlock: args.has('U'),
            channel: args.positionals().first().cloned().unwrap_or_default(),
        })
    }

    async fn execute(self, context: &mut ExecContext<'_>) -> SharedCommandExecution {
        let registry = {
            let state = context.state().borrow_mut();
            state.wait_registry()
        };
        // Whatever the registry does happens now, in command order; only the
        // waiting is deferred.
        let start = suspend::wait_for(&self, &registry);
        let result = context.wait_for_answer(false, start).await;
        SharedCommandExecution::completed(result)
    }
}

/// What a `wait-for` did: either it finished, or it is queued behind another
/// client and resumes when the returned completion fires.
pub(super) enum WaitForOutcome {
    Done(CommandResult),
    Pending(Completion<()>),
}

pub(super) fn wait_for(command: &WaitFor, registry: &WaitRegistry) -> WaitForOutcome {
    let channel = command.channel.as_str();
    let outcome = if command.signal {
        registry.signal(channel);
        WaitOutcome::Ready
    } else if command.lock {
        registry.lock(channel)
    } else if command.unlock {
        if !registry.unlock(channel) {
            return WaitForOutcome::Done(CommandResult::err(format!(
                "channel {channel} not locked\n"
            )));
        }
        WaitOutcome::Ready
    } else {
        registry.wait(channel)
    };
    match outcome {
        WaitOutcome::Ready => WaitForOutcome::Done(CommandResult::ok("")),
        WaitOutcome::Pending(completion) => WaitForOutcome::Pending(completion),
    }
}

/// The `run-shell` a detached job runs, rebuilt from an argv for the tests that
/// drive one directly.
#[cfg(test)]
pub(crate) fn run_shell_from_argv(argv: &[String]) -> RunShell {
    let normalized = normalize_argv("run-shell", argv);
    RunShell::parse(&ParsedArgs::lex("run-shell", &normalized)).expect("run-shell arguments")
}

/// The same, for the tests that drive a `wait-for` against a bare registry.
#[cfg(test)]
pub(super) fn wait_for_from_argv(argv: &[String]) -> WaitFor {
    let normalized = normalize_argv("wait-for", argv);
    WaitFor::parse(&ParsedArgs::lex("wait-for", &normalized)).expect("wait-for arguments")
}

// ---- reading a configuration file ------------------------------------------

/// One parsed configuration file: the command lines with their file line
/// numbers, plus the parser assignments made in active branches, which the
/// caller publishes to the global environment as tmux does.
struct SourcedConfig {
    lines: Vec<(usize, Vec<LineToken>)>,
    assignments: Vec<(String, String, bool)>,
}

/// One state of the `%if`/`%elif`/`%else` conditional stack.
struct SourceCondition {
    parent_active: bool,
    /// Whether any branch of this chain has been taken yet.
    taken: bool,
    active: bool,
    seen_else: bool,
}

/// Split a sourced config file into command argv lines. This preprocessing
/// layer handles the configuration-only syntax which cannot be represented by
/// ordinary command argv: conditional directives, parser assignments, and
/// brace command blocks. `environment` is the server's global environment,
/// which seeds `$NAME` expansion; an undefined name expands to nothing.
///
/// Like tmux, the whole file is parsed before anything runs, so a structural
/// error — an unbalanced conditional or an invalid escape — rejects the file:
/// the error is `(line, diagnostic)`.
fn source_lines(
    contents: &str,
    environment: &BTreeMap<String, String>,
) -> Result<SourcedConfig, (usize, String)> {
    let mut lines = Vec::new();
    let mut logical = String::new();
    let mut logical_start = 1;
    let mut assignments = environment.clone();
    let mut published = Vec::new();
    let mut conditions = Vec::<SourceCondition>::new();
    let mut brace_block: Option<(usize, Vec<LineToken>, String, usize)> = None;
    let mut line_number = 0;
    for raw in contents.lines() {
        line_number += 1;
        let continued = raw.trim_end().ends_with('\\');
        let part = if continued {
            raw.trim_end().strip_suffix('\\').unwrap_or(raw)
        } else {
            raw
        };
        if logical.is_empty() {
            logical_start = line_number;
        }
        // A backslash-newline splices the lines together at the character
        // level, continuing the same word, so no separator is inserted.
        logical.push_str(part);
        if continued {
            continue;
        }
        let trimmed = logical.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            logical.clear();
            continue;
        }

        if let Some(condition) = trimmed.strip_prefix("%if ") {
            let parent_active = conditions.iter().all(|condition| condition.active);
            let expanded = format::expand(condition.trim(), &Vars::default());
            let active = parent_active && format::is_true(&expanded);
            conditions.push(SourceCondition {
                parent_active,
                taken: active,
                active,
                seen_else: false,
            });
            logical.clear();
            continue;
        }
        if let Some(condition) = trimmed.strip_prefix("%elif ") {
            let Some(top) = conditions.last_mut() else {
                return Err((logical_start, "syntax error".to_string()));
            };
            if top.seen_else {
                return Err((logical_start, "syntax error".to_string()));
            }
            let expanded = format::expand(condition.trim(), &Vars::default());
            top.active = top.parent_active && !top.taken && format::is_true(&expanded);
            top.taken |= top.active;
            logical.clear();
            continue;
        }
        if trimmed == "%else" {
            let Some(top) = conditions.last_mut() else {
                return Err((logical_start, "syntax error".to_string()));
            };
            if top.seen_else {
                return Err((logical_start, "syntax error".to_string()));
            }
            top.seen_else = true;
            top.active = top.parent_active && !top.taken;
            top.taken |= top.active;
            logical.clear();
            continue;
        }
        if trimmed == "%endif" {
            if conditions.pop().is_none() {
                return Err((logical_start, "syntax error".to_string()));
            }
            logical.clear();
            continue;
        }
        if !conditions.iter().all(|condition| condition.active) {
            logical.clear();
            continue;
        }

        if brace_block.is_none() {
            let (assignment, hidden) = match trimmed.strip_prefix("%hidden") {
                Some(rest) if rest.starts_with(char::is_whitespace) => (rest.trim_start(), true),
                _ => (trimmed, false),
            };
            if let Some((name, value)) = parse_source_assignment(assignment) {
                assignments.insert(name.to_string(), value.to_string());
                published.push((name.to_string(), value.to_string(), hidden));
                logical.clear();
                continue;
            }
        }
        let expanded = expand_source_assignments(trimmed, &assignments);

        if let Some((_line, _prefix, body, depth)) = brace_block.as_mut() {
            let opens = expanded.chars().filter(|ch| *ch == '{').count();
            let closes = expanded.chars().filter(|ch| *ch == '}').count();
            let new_depth = depth.saturating_add(opens).saturating_sub(closes);
            // Only the block's own closing brace is syntax; an inner one
            // belongs to the body verbatim.
            if !(new_depth == 0 && expanded.trim() == "}") {
                if !body.is_empty() {
                    body.push_str(" ; ");
                }
                body.push_str(expanded.trim());
            }
            *depth = new_depth;
            if *depth == 0 {
                let (line, mut prefix, body, _) = brace_block.take().expect("active brace block");
                prefix.push(LineToken::Word(body));
                lines.push((line, prefix));
            }
            logical.clear();
            continue;
        }

        if let Some(prefix) = expanded.trim_end().strip_suffix('{') {
            let tokens = tokenize_line_checked(prefix.trim_end())
                .map_err(|message| (logical_start, message))?;
            brace_block = Some((logical_start, tokens, String::new(), 1));
            logical.clear();
            continue;
        }

        let argv = tokenize_line_checked(&expanded).map_err(|message| (logical_start, message))?;
        if !argv.is_empty() {
            lines.push((logical_start, argv));
        }
        logical.clear();
    }
    if !logical.trim().is_empty() {
        let argv = tokenize_line_checked(logical.trim_start())
            .map_err(|message| (logical_start, message))?;
        if !argv.is_empty() {
            lines.push((logical_start, argv));
        }
    }
    if !conditions.is_empty() {
        return Err((line_number + 1, "syntax error".to_string()));
    }
    Ok(SourcedConfig {
        lines,
        assignments: published,
    })
}

fn parse_source_assignment(line: &str) -> Option<(&str, &str)> {
    let (name, value) = line.split_once('=')?;
    if name.is_empty()
        || !name
            .chars()
            .all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
        || name.as_bytes().first().is_some_and(u8::is_ascii_digit)
    {
        return None;
    }
    Some((name, value.trim_matches('"')))
}

fn expand_source_assignments(line: &str, assignments: &BTreeMap<String, String>) -> String {
    let mut output = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    let mut single_quoted = false;
    while let Some(ch) = chars.next() {
        if ch == '\'' {
            single_quoted = !single_quoted;
            output.push(ch);
            continue;
        }
        if ch != '$' || single_quoted {
            output.push(ch);
            continue;
        }
        let braced = chars.peek() == Some(&'{');
        if braced {
            chars.next();
        }
        let mut name = String::new();
        while chars
            .peek()
            .is_some_and(|next| *next == '_' || next.is_ascii_alphanumeric())
        {
            name.push(chars.next().expect("peeked assignment name"));
        }
        if braced && chars.peek() == Some(&'}') {
            chars.next();
        }
        if let Some(value) = assignments.get(&name) {
            output.push_str(value);
        } else if name.is_empty() {
            // A bare `$` is not a variable reference; keep it.
            output.push('$');
            if braced {
                output.push('{');
            }
        }
        // An undefined `$NAME` expands to nothing, as in tmux.
    }
    output
}

fn source_verbose_line(tokens: &[LineToken]) -> String {
    let mut words = tokens
        .iter()
        .map(|token| match token {
            LineToken::Word(word) => word.clone(),
            LineToken::Separator => ";".to_string(),
        })
        .collect::<Vec<_>>();
    if let Some(first) = words.first_mut() {
        if let Resolution::Name(canonical) = registry::resolve(first) {
            *first = canonical.to_string();
        }
    }
    words.join(" ")
}
