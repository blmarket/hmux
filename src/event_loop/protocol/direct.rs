use std::io;
use std::os::unix::ffi::OsStrExt;
use std::rc::Rc;
use std::time::Instant;

use crate::server::command::{self, CommandResult};
use crate::server::task::{ReadySet, TaskState};
use crate::tmux::message::{Frame, Message};

use super::super::actor::ActorRef;
use super::super::driver::Outbox;
use super::super::job::JobEvent;
use super::client::{
    CommandClientState, CommandOperation, ProtocolClient, ProtocolCloseReason, ProtocolEvent,
    ProtocolIoSide, ProtocolState, COMMAND_QUEUE_BUDGET, FILE_STREAM,
};
use super::command::{run_command_work, ActiveResumableCommand, CommandStep, CommandWork};

impl ProtocolClient {
    pub(super) fn begin_command(
        &mut self,
        target: &ActorRef<Self>,
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
            timer_generation: 0,
        });
        self.start_command_work(target, CommandWork::Initial { args, context }, outbox);
    }

    pub(super) fn start_command_work(
        &mut self,
        target: &ActorRef<Self>,
        work: CommandWork,
        outbox: &mut Outbox,
    ) {
        let ProtocolState::Command(command) = &mut self.protocol_state else {
            return;
        };
        command.operation = CommandOperation::AwaitingStep;
        let step = run_command_work(work, &self.state);
        outbox.enqueue_protocol(target.clone(), ProtocolEvent::CommandStepReady(step));
    }

    pub(super) fn drive_resumable_command(&mut self, target: &ActorRef<Self>, outbox: &mut Outbox) {
        let (complete, has_source, deadline) = {
            let ProtocolState::Command(command) = &mut self.protocol_state else {
                return;
            };
            let CommandOperation::AwaitingQueue(active) = &mut command.operation else {
                return;
            };
            let complete = active.task.poll(&ReadySet::default());
            let wait = active.task.wait();
            (
                complete,
                wait.as_ref().is_some_and(|wait| !wait.sources().is_empty()),
                wait.and_then(|wait| wait.deadline()),
            )
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
                    self.close(target, ProtocolCloseReason::Error(error.kind()), outbox);
                    return;
                }
                None => return,
            };
            for request in result.background_commands.drain(..) {
                outbox
                    .enqueue_background(self.background_commands.clone(), JobEvent::Start(request));
            }
            let mut transaction = active.transaction;
            if transaction.complete_group(&result) {
                transaction.groups.clear();
            }
            self.start_command_work(target, CommandWork::Advance(transaction), outbox);
            return;
        }

        if !has_source {
            outbox.enqueue_protocol(target.clone(), ProtocolEvent::CommandQueueContinue);
            return;
        }

        let ProtocolState::Command(command) = &mut self.protocol_state else {
            return;
        };
        let operation = std::mem::replace(&mut command.operation, CommandOperation::AwaitingStep);
        let CommandOperation::AwaitingQueue(active) = operation else {
            return;
        };
        command.operation = CommandOperation::WaitingCommand(active);
        outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Command, true);
        if let Some(deadline) = deadline {
            command.timer_generation = command.timer_generation.wrapping_add(1);
            outbox.set_protocol_timer_event(
                target.clone(),
                deadline,
                ProtocolEvent::CommandTimeout(command.timer_generation),
            );
        }
    }

    pub(super) fn handle_command_completed(
        &mut self,
        target: &ActorRef<Self>,
        source_ready: bool,
        outbox: &mut Outbox,
    ) {
        let (mut active, had_deadline) = {
            let ProtocolState::Command(command) = &mut self.protocol_state else {
                return;
            };
            let operation =
                std::mem::replace(&mut command.operation, CommandOperation::AwaitingStep);
            let CommandOperation::WaitingCommand(active) = operation else {
                return;
            };
            let had_deadline = active
                .task
                .wait()
                .and_then(|wait| wait.deadline())
                .is_some();
            (active, had_deadline)
        };
        active
            .task
            .poll_after_single_source_wakeup(source_ready, Instant::now());
        outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Command, false);
        if had_deadline {
            outbox.cancel_protocol_timer(target.clone());
        }
        let ProtocolState::Command(command) = &mut self.protocol_state else {
            return;
        };
        command.operation = CommandOperation::AwaitingQueue(active);
        outbox.enqueue_protocol(target.clone(), ProtocolEvent::CommandQueueContinue);
    }

    pub(super) fn handle_command_step(
        &mut self,
        target: &ActorRef<Self>,
        step: CommandStep,
        outbox: &mut Outbox,
    ) {
        let awaiting_step = matches!(
            &self.protocol_state,
            ProtocolState::Command(CommandClientState {
                operation: CommandOperation::AwaitingStep,
                ..
            })
        );
        if !awaiting_step {
            return;
        }
        match step {
            CommandStep::Complete(result) => self.begin_response(target, result, outbox),
            CommandStep::Execute {
                transaction,
                args,
                context,
            } => {
                let agents = self.hub.snapshot().panes;
                match command::start_resumable_command(&args, &self.state, &agents, &context) {
                    Ok(queue) => {
                        let ProtocolState::Command(command) = &mut self.protocol_state else {
                            return;
                        };
                        command.operation =
                            CommandOperation::AwaitingQueue(ActiveResumableCommand {
                                transaction,
                                task: TaskState::new(command::CommandCoroutine::new(
                                    queue,
                                    Rc::clone(&self.state),
                                    Rc::clone(&self.command_runtime),
                                    COMMAND_QUEUE_BUDGET,
                                )),
                            });
                        self.drive_resumable_command(target, outbox);
                    }
                    Err(result) => {
                        let mut transaction = transaction;
                        if transaction.complete_group(&result) {
                            transaction.groups.clear();
                        }
                        self.start_command_work(target, CommandWork::Advance(transaction), outbox);
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
                    target,
                    Frame::new(Message::ReadOpen {
                        stream: FILE_STREAM,
                        fd,
                        path: wire_path,
                    }),
                    outbox,
                );
                self.drive_output(target, outbox);
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
                    target,
                    Frame::new(Message::WriteOpen {
                        stream: FILE_STREAM,
                        fd: -1,
                        flags,
                        path,
                    }),
                    outbox,
                );
                self.drive_output(target, outbox);
            }
        }
    }

    pub(super) fn handle_direct_frame(
        &mut self,
        target: &ActorRef<Self>,
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
                outbox.enqueue_protocol(
                    target.clone(),
                    ProtocolEvent::CommandStepReady(CommandStep::Execute {
                        transaction,
                        args,
                        context,
                    }),
                );
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
                    self.start_command_work(target, CommandWork::Advance(transaction), outbox);
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
                    self.drive_output(target, outbox);
                }
            }
            Message::WriteReady { stream, error } => {
                if let ProtocolState::Command(CommandClientState {
                    operation: CommandOperation::Responding(response),
                    ..
                }) = &mut self.protocol_state
                {
                    response.acknowledge(stream, error);
                    self.drive_output(target, outbox);
                }
            }
            Message::Detach(_) | Message::DetachKill(_) | Message::Exit(_) | Message::Shutdown => {
                self.close(target, ProtocolCloseReason::Completed, outbox);
            }
            _ => {}
        }
    }
}
