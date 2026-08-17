//! Commands that act on the server as a whole.

use super::*;

#[derive(Clone, Debug)]
pub(in crate::server) enum Command {
    ListCommands(ListCommands),
    Start,
    Kill,
    Access(ServerAccess),
    ShowMessages(ShowMessages),
    Lock,
    LockSession(LockSession),
}

impl Command {
    pub(super) fn execute(self, context: &mut CommandContext<'_>) -> CommandResult {
        match self {
            Self::ListCommands(command) => command.execute(),
            Self::Start => CommandResult::ok(""),
            Self::Kill => {
                context.state.kill_server();
                CommandResult::ok("")
            }
            Self::Access(command) => command.execute(context.state),
            Self::ShowMessages(command) => command.execute(context.state),
            Self::Lock => {
                context.state.lock_all_clients();
                CommandResult::ok("")
            }
            Self::LockSession(command) => command.execute(context.state),
        }
    }
}

/// `list-commands [-F format] [command]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct ListCommands {
    /// `-F`: the template each listed command expands.
    format: Option<String>,
    /// The single command to list, instead of the whole table.
    command: Option<String>,
}

impl ListCommands {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            format: args.value('F').map(str::to_string),
            command: args.positionals().first().cloned(),
        })
    }

    /// Prints the usage line for every command in tmux's table, or — given a
    /// command argument — just that one. The argument resolves through the same
    /// command resolver the interpreter uses everywhere else
    /// (`cmd-list-commands.c` calls `cmd_find`), so an alias or unambiguous
    /// prefix hits the right command while an ambiguous or unknown one reports
    /// the resolver's own diagnostic (exit 1). `-F` expands once per command
    /// with tmux's three command-list variables; an expansion that comes out
    /// empty prints nothing at all, as `cmd-list-commands.c` only calls
    /// `cmdq_print` for a non-empty line.
    fn execute(self) -> CommandResult {
        let render = |name: &'static str| {
            if let Some(template) = self.format.as_deref() {
                let spec = registry::spec(name).expect("resolved command is in the table");
                let mut vars = Vars::new();
                vars.set("command_list_name", name)
                    .set("command_list_alias", spec.alias.unwrap_or(""))
                    .set("command_list_usage", spec.usage);
                format::expand(template, &vars)
            } else {
                registry::command_line(name).expect("table command has a usage line")
            }
        };

        let push = |out: &mut String, name: &'static str| {
            let line = render(name);
            if !line.is_empty() {
                out.push_str(&line);
                out.push('\n');
            }
        };

        match self.command.as_deref() {
            Some(word) => match registry::resolve(word) {
                Resolution::Name(name) => {
                    let mut out = String::new();
                    push(&mut out, name);
                    CommandResult::ok(out)
                }
                Resolution::Ambiguous { error } | Resolution::Unknown { error } => {
                    CommandResult::err(error)
                }
            },
            None => {
                let mut out = String::new();
                for spec in registry::COMMAND_SPECS {
                    push(&mut out, spec.name);
                }
                CommandResult::ok(out)
            }
        }
    }
}

/// `show-messages [-JT] [-t target-client]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct ShowMessages {
    /// `-J`: list the server's jobs instead of its message log.
    jobs: bool,
    /// `-T`: list the clients' terminals instead of the message log.
    terminals: bool,
    /// `-t`: restrict the terminal listing to one client.
    target: Option<String>,
}

impl ShowMessages {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            jobs: args.has('J'),
            terminals: args.has('T'),
            target: args.value('t').map(str::to_string),
        })
    }

    fn execute(self, st: &ServerState) -> CommandResult {
        if self.terminals || self.jobs {
            let mut output = String::new();
            if self.terminals {
                let target = self.target.as_deref();
                let mut terminal_number = 0;
                for (name, term, terminal) in st.client_terminals() {
                    if target.is_some_and(|target| {
                        let target = target.strip_suffix(':').unwrap_or(target);
                        name != target
                            && name.strip_prefix("/dev/").is_none_or(|name| name != target)
                    }) {
                        continue;
                    }
                    output.push_str(&format!(
                        "Terminal {terminal_number}: {term} for {name}, flags=0x{:x}:\n",
                        terminal.flags()
                    ));
                    for description in terminal.descriptions() {
                        output.push_str(&description);
                        output.push('\n');
                    }
                    terminal_number += 1;
                }
            }
            if self.jobs && !output.is_empty() {
                output.push('\n');
            }
            if self.jobs {
                // tmux keeps one job list for the whole server, so a `#()` still
                // running for some format is listed beside a `run-shell -b`.
                let background = st
                    .background_job_registry()
                    .jobs()
                    .into_iter()
                    .map(|job| (job.command, job.fd, job.pid as i32));
                let format_jobs = st
                    .running_format_jobs()
                    .into_iter()
                    .map(|job| (job.command, job.fd, job.pid));
                for (id, (command, fd, pid)) in background.chain(format_jobs).enumerate() {
                    output.push_str(&format!(
                        "Job {id}: {command} [fd={fd}, pid={pid}, status=0]\n"
                    ));
                }
            }
            return CommandResult::ok(output);
        }
        let mut output = String::new();
        for message in st.messages().iter().rev() {
            output.push_str(&format!(
                "{}: {}\n",
                message_time(message.time),
                message.text
            ));
        }
        CommandResult::ok(output)
    }
}

fn message_time(epoch: i64) -> String {
    let mut vars = Vars::new();
    vars.set("message_time", epoch.to_string());
    format::expand("#{t/p:message_time}", &vars)
}

/// `lock-session [-t target-session]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct LockSession {
    /// `-t`: the session whose clients lock; the current one otherwise.
    target: Option<String>,
}

impl LockSession {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            target: args.value('t').map(str::to_string),
        })
    }

    fn execute(self, st: &ServerState) -> CommandResult {
        let target = self
            .target
            .as_deref()
            .map(|target| target.split(':').next().unwrap_or(target).to_string())
            .or_else(|| current_session(st));
        let Some(target) = target else {
            return CommandResult::err("can't establish current session\n");
        };
        match st.lock_session_clients(&target) {
            Ok(()) => CommandResult::ok(""),
            Err(error) => CommandResult::err(format!("{error}\n")),
        }
    }
}

/// `server-access [-adglrw] [-t target-pane] [user|group]`.
#[derive(Clone, Debug)]
pub(in crate::server) struct ServerAccess {
    list: bool,
    user: Option<String>,
}

impl ServerAccess {
    pub(in crate::server) fn parse(args: &ParsedArgs) -> Result<Self, String> {
        Ok(Self {
            list: args.has('l'),
            user: args.positionals().first().cloned(),
        })
    }

    fn execute(self, _st: &mut ServerState) -> CommandResult {
        if self.list {
            let uid = unsafe { libc::getuid() };
            let user_name = unsafe {
                let pw = libc::getpwuid(uid);
                if pw.is_null() {
                    "unknown".to_string()
                } else {
                    std::ffi::CStr::from_ptr((*pw).pw_name)
                        .to_str()
                        .unwrap_or("unknown")
                        .to_string()
                }
            };
            return CommandResult::ok(format!("{user_name} (W)\n"));
        }
        let Some(user) = self.user.as_deref() else {
            return CommandResult::err("missing user argument\n");
        };
        let c_user = match std::ffi::CString::new(user) {
            Ok(c_user) => c_user,
            Err(_) => return CommandResult::err(format!("unknown user: {user}\n")),
        };
        let exists = unsafe {
            let pw = libc::getpwnam(c_user.as_ptr());
            !pw.is_null()
        };
        if !exists {
            return CommandResult::err(format!("unknown user: {user}\n"));
        }
        CommandResult::ok("")
    }
}
