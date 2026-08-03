use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::server::command::{self, ClientContext, CommandResult};
use crate::server::state::ServerState;
use crate::server::task::TaskState;

/// One validated command line, possibly split around client-side file work.
pub(crate) struct CommandTransaction {
    pub(super) groups: VecDeque<Vec<String>>,
    pub(super) output: CommandResult,
    pub(super) context: ClientContext,
}

pub(super) struct ActiveResumableCommand {
    pub(super) transaction: CommandTransaction,
    pub(super) task: TaskState<command::CommandCoroutine>,
}

impl CommandTransaction {
    pub(super) fn new(groups: Vec<Vec<String>>, context: ClientContext) -> Self {
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
        /// The `save-buffer` invocation, kept so its `after-*` hook can run
        /// once the client has written the file.
        args: Vec<String>,
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
                        args,
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
