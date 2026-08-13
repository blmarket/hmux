use std::io;
use std::os::unix::ffi::OsStrExt;
use std::rc::Rc;

use crate::server::command::{self, CommandResult};

use crate::tmux::message::{Frame, Message};

use super::task::Outbox;
use super::client::{
    CommandClientState, CommandOperation, ProtocolClient, ProtocolCloseReason, ProtocolEvent,
    ProtocolIoSide, ProtocolState, COMMAND_QUEUE_BUDGET, FILE_STREAM,
};
use super::command::{run_command_work, ActiveResumableCommand, CommandStep, CommandWork};

impl ProtocolClient {
    pub(super) fn begin_command(
        &mut self,
        args: Vec<String>,
        outbox: &mut Outbox,
    ) {
        let Some(context) = self
            .identifying()
            .map(|identifying| identifying.context.clone())
        else {
            return;
        };
        self.protocol_state = ProtocolState::Command(CommandClientState {
            operation: CommandOperation::AwaitingStep,
        });
        self.start_command_work(CommandWork::Initial { args, context }, outbox);
    }

    pub(super) fn start_command_work(
        &mut self,
        work: CommandWork,
        outbox: &mut Outbox,
    ) {
        let ProtocolState::Command(command) = &mut self.protocol_state else {
            return;
        };
        command.operation = CommandOperation::AwaitingStep;
        let step = run_command_work(work, &self.state);
        outbox.enqueue_protocol(ProtocolEvent::CommandStepReady(step));
    }

    pub(super) fn drive_resumable_command(&mut self, outbox: &mut Outbox) {
        let complete = {
            let ProtocolState::Command(command) = &mut self.protocol_state else {
                return;
            };
            let CommandOperation::AwaitingQueue(active) = &mut command.operation else {
                return;
            };
            active.task.poll()
        };
        if complete {
            let ProtocolState::Command(command) = &mut self.protocol_state else {
                return;
            };
            let operation =
                std::mem::replace(&mut command.operation, CommandOperation::AwaitingStep);
            let CommandOperation::AwaitingQueue(mut active) = operation else {
                return;
            };
            let mut result = match active.task.take_output() {
                Some(Ok(result)) => result,
                Some(Err(error)) => {
                    self.close(ProtocolCloseReason::Error(error.kind()), outbox);
                    return;
                }
                None => return,
            };
            for request in result.background_commands.drain(..) {
                self.background_commands.start(request);
            }
            let mut transaction = active.transaction;
            if transaction.complete_group(&result) {
                transaction.groups.clear();
            }
            self.start_command_work(CommandWork::Advance(transaction), outbox);
            return;
        }

        // A queue that has not finished is waiting on something, and its wake
        // is what brings this client back.
        let ProtocolState::Command(command) = &mut self.protocol_state else {
            return;
        };
        let operation = std::mem::replace(&mut command.operation, CommandOperation::AwaitingStep);
        let CommandOperation::AwaitingQueue(active) = operation else {
            return;
        };
        command.operation = CommandOperation::WaitingCommand(active);
        outbox.set_protocol_interest(ProtocolIoSide::Command, true);
    }

    pub(super) fn handle_command_completed(
        &mut self,
        source_ready: bool,
        outbox: &mut Outbox,
    ) {
        let mut active = {
            let ProtocolState::Command(command) = &mut self.protocol_state else {
                return;
            };
            let operation =
                std::mem::replace(&mut command.operation, CommandOperation::AwaitingStep);
            let CommandOperation::WaitingCommand(active) = operation else {
                return;
            };
            active
        };
        let _ = source_ready;
        active.task.poll();
        outbox.set_protocol_interest(ProtocolIoSide::Command, false);
        let ProtocolState::Command(command) = &mut self.protocol_state else {
            return;
        };
        command.operation = CommandOperation::AwaitingQueue(active);
        outbox.enqueue_protocol(ProtocolEvent::CommandQueueContinue);
    }

    pub(super) fn handle_command_step(
        &mut self,
        step: CommandStep,
        outbox: &mut Outbox,
    ) {
        let awaiting_step = matches!(
            &self.protocol_state,
            ProtocolState::Command(CommandClientState {
                operation: CommandOperation::AwaitingStep,
            })
        );
        if !awaiting_step {
            return;
        }
        match step {
            CommandStep::Complete(result) => self.begin_response(result, outbox),
            CommandStep::Execute {
                transaction,
                args,
                context,
            } => {
                let agents = self.hub.snapshot().panes;
                match command::start_resumable_command(&args, &self.state, &agents, &context)
                    .and_then(|queue| {
                        self.command_runtime
                            .spawn_queue(queue, Rc::clone(&self.state), COMMAND_QUEUE_BUDGET)
                            .map_err(|error| CommandResult::err(format!("{error}\n")))
                    }) {
                    Ok(queued) => {
                        let ProtocolState::Command(command) = &mut self.protocol_state else {
                            return;
                        };
                        command.operation =
                            CommandOperation::AwaitingQueue(ActiveResumableCommand {
                                transaction,
                                task: queued,
                            });
                        self.drive_resumable_command(outbox);
                    }
                    Err(result) => {
                        let mut transaction = transaction;
                        if transaction.complete_group(&result) {
                            transaction.groups.clear();
                        }
                        self.start_command_work(CommandWork::Advance(transaction), outbox);
                    }
                }
            }
            CommandStep::Read {
                transaction,
                args,
                path,
            } => {
                let fd = if path.as_os_str() == "-" { 0 } else { -1 };
                let mut wire_path = path.as_os_str().as_bytes().to_vec();
                wire_path.push(0);
                let ProtocolState::Command(command) = &mut self.protocol_state else {
                    return;
                };
                command.operation = CommandOperation::WaitingClientRead {
                    transaction,
                    args,
                    data: Vec::new(),
                };
                self.queue_frame(
                    Frame::new(Message::ReadOpen {
                        stream: FILE_STREAM,
                        fd,
                        path: wire_path,
                    }),
                    outbox,
                );
                self.drive_output(outbox);
            }
            CommandStep::Write {
                transaction,
                args,
                request,
            } => {
                let mut path = request.path.as_os_str().as_bytes().to_vec();
                path.push(0);
                let flags = request.flags;
                let ProtocolState::Command(command) = &mut self.protocol_state else {
                    return;
                };
                command.operation = CommandOperation::WaitingClientWriteReady {
                    transaction,
                    args,
                    request,
                };
                self.queue_frame(
                    Frame::new(Message::WriteOpen {
                        stream: FILE_STREAM,
                        fd: -1,
                        flags,
                        path,
                    }),
                    outbox,
                );
                self.drive_output(outbox);
            }
        }
    }

    pub(super) fn handle_direct_frame(
        &mut self,
        frame: Frame,
        outbox: &mut Outbox,
    ) {
        match frame.msg {
            Message::Read {
                stream: FILE_STREAM,
                data,
            } => {
                if let ProtocolState::Command(CommandClientState {
                    operation: CommandOperation::WaitingClientRead { data: buffered, .. },
                    ..
                }) = &mut self.protocol_state
                {
                    buffered.extend_from_slice(&data);
                }
            }
            Message::ReadDone {
                stream: FILE_STREAM,
                error,
            } => {
                let ProtocolState::Command(command) = &mut self.protocol_state else {
                    return;
                };
                let operation =
                    std::mem::replace(&mut command.operation, CommandOperation::AwaitingStep);
                let CommandOperation::WaitingClientRead {
                    transaction,
                    args,
                    data,
                } = operation
                else {
                    return;
                };
                let mut context = transaction.context.clone();
                context.input_file = Some(if error == 0 { Ok(data) } else { Err(error) });
                outbox.enqueue_protocol(ProtocolEvent::CommandStepReady(CommandStep::Execute {
                    transaction,
                    args,
                    context,
                }));
            }
            Message::WriteReady {
                stream: FILE_STREAM,
                error,
            } => {
                let ProtocolState::Command(command) = &mut self.protocol_state else {
                    return;
                };
                let operation =
                    std::mem::replace(&mut command.operation, CommandOperation::AwaitingStep);
                let CommandOperation::WaitingClientWriteReady {
                    mut transaction,
                    args,
                    request,
                } = operation
                else {
                    return;
                };
                if error != 0 {
                    let mut result = CommandResult::err(format!(
                        "{}: {}\n",
                        io::Error::from_raw_os_error(error),
                        request.display_path
                    ));
                    result.continue_queue = true;
                    if transaction.complete_group(&result) {
                        transaction.groups.clear();
                    }
                    self.start_command_work(CommandWork::Advance(transaction), outbox);
                } else {
                    let ProtocolState::Command(command) = &mut self.protocol_state else {
                        return;
                    };
                    command.operation = CommandOperation::WritingClientFile {
                        transaction: Some(transaction),
                        args,
                        request,
                        offset: 0,
                        close_generated: false,
                    };
                    self.drive_output(outbox);
                }
            }
            Message::WriteReady { stream, error } => {
                if let ProtocolState::Command(CommandClientState {
                    operation: CommandOperation::Responding(response),
                    ..
                }) = &mut self.protocol_state
                {
                    response.acknowledge(stream, error);
                    self.drive_output(outbox);
                }
            }
            Message::Detach(_) | Message::DetachKill(_) | Message::Exit(_) | Message::Shutdown => {
                self.close(ProtocolCloseReason::Completed, outbox);
            }
            _ => {}
        }
    }
}
