use super::*;
use crate::server::command;

/// One validated command line, possibly split around client-side file work.
pub(crate) struct CommandTransaction {
    pub(super) groups: VecDeque<Vec<String>>,
    output: CommandResult,
    pub(super) context: ClientContext,
}

pub(super) struct ActiveResumableCommand {
    pub(super) transaction: CommandTransaction,
    pub(super) queue: command::ResumableCommandQueue,
}

impl CommandTransaction {
    fn new(groups: Vec<Vec<String>>, context: ClientContext) -> Self {
        Self {
            groups: groups.into(),
            output: CommandResult::ok(""),
            context,
        }
    }

    /// Merge a group result and report whether the remaining group tail stops.
    pub(super) fn complete_group(&mut self, result: &CommandResult) -> bool {
        let exit = result.exit;
        self.output.continue_queue |= result.continue_queue;
        self.output.append_stdout(result);
        self.output.stderr.push_str(&result.stderr);
        if self.output.exit == 0 || exit != 0 {
            self.output.exit = exit;
        }
        exit != 0 && !result.continue_queue
    }
}

pub(super) enum CommandWork {
    Initial {
        args: Vec<String>,
        context: ClientContext,
    },
    Advance(CommandTransaction),
}

pub(crate) enum CommandStep {
    Complete(CommandResult),
    Read {
        transaction: CommandTransaction,
        args: Vec<String>,
        path: PathBuf,
    },
    Write {
        transaction: CommandTransaction,
        request: command::ClientFileWrite,
    },
    Execute {
        transaction: CommandTransaction,
        args: Vec<String>,
        context: ClientContext,
    },
}

pub(super) fn run_command_work(work: CommandWork, state: &Arc<Mutex<ServerState>>) -> CommandStep {
    match work {
        CommandWork::Initial { args, context } => {
            let aliases = match state.lock() {
                Ok(state) => state.command_aliases(),
                Err(_) => {
                    return CommandStep::Complete(CommandResult::err("server state poisoned\n"));
                }
            };
            let groups = match command::command_line_groups(&args, &aliases) {
                Ok(groups) => groups,
                Err(result) => return CommandStep::Complete(result),
            };
            let groups = if groups.len() > 1
                && groups
                    .iter()
                    .any(|group| command::uses_client_file_protocol(group))
            {
                groups
            } else {
                vec![args]
            };
            advance_command_transaction(CommandTransaction::new(groups, context), state)
        }
        CommandWork::Advance(transaction) => advance_command_transaction(transaction, state),
    }
}

fn advance_command_transaction(
    mut transaction: CommandTransaction,
    state: &Arc<Mutex<ServerState>>,
) -> CommandStep {
    loop {
        let Some(args) = transaction.groups.pop_front() else {
            return CommandStep::Complete(transaction.output);
        };

        let file_write = match state.lock() {
            Ok(state) => command::save_buffer_client_request(&args, &state, &transaction.context),
            Err(_) => Some(Err(CommandResult::err("server state poisoned\n"))),
        };
        if let Some(request) = file_write {
            match request {
                Err(result) => {
                    if transaction.complete_group(&result) {
                        transaction.groups.clear();
                    }
                    continue;
                }
                Ok(request) => {
                    return CommandStep::Write {
                        transaction,
                        request,
                    };
                }
            }
        }

        if let Some(path) = command::client_input_path(&args, &transaction.context) {
            return CommandStep::Read {
                transaction,
                args,
                path,
            };
        }

        let context = transaction.context.clone();
        return CommandStep::Execute {
            transaction,
            args,
            context,
        };
    }
}

pub(super) enum PendingCommand {
    Worker {
        completion: UnixStream,
        result: Arc<Mutex<Option<command::CommandSuspensionResult>>>,
    },
    PaneOutput(command::PaneOutputSuspension),
}

impl PendingCommand {
    pub(super) fn start_suspension(suspension: command::CommandSuspension) -> io::Result<Self> {
        if let command::CommandSuspension::PaneOutput(wait) = suspension {
            return Ok(Self::PaneOutput(wait));
        }
        let (completion, mut signal) = UnixStream::pair()?;
        completion.set_nonblocking(true)?;
        let result = Arc::new(Mutex::new(None));
        let worker_result = Arc::clone(&result);
        thread::spawn(move || {
            let completed = suspension.resolve();
            if let Ok(mut result) = worker_result.lock() {
                *result = Some(completed);
            }
            let _ = signal.write_all(&[1]);
        });
        Ok(Self::Worker { completion, result })
    }

    pub(super) fn fd(&self) -> BorrowedFd<'_> {
        match self {
            Self::Worker { completion, .. } => completion.as_fd(),
            Self::PaneOutput(wait) => wait.as_fd(),
        }
    }

    pub(super) fn deadline(&self) -> Option<Instant> {
        match self {
            Self::Worker { .. } => None,
            Self::PaneOutput(wait) => Some(wait.deadline()),
        }
    }

    pub(super) fn is_complete(&self) -> bool {
        match self {
            Self::Worker { .. } => false,
            Self::PaneOutput(wait) => wait.is_complete(),
        }
    }

    pub(super) fn take_result(&mut self) -> io::Result<command::CommandSuspensionResult> {
        let Self::Worker { completion, result } = self else {
            let Self::PaneOutput(wait) = self else {
                unreachable!();
            };
            return Ok(wait.complete());
        };
        let mut byte = [0u8; 1];
        completion.read_exact(&mut byte)?;
        result
            .lock()
            .map_err(|_| io::Error::other("command result poisoned"))?
            .take()
            .ok_or_else(|| io::Error::other("command completed without a result"))
    }
}
