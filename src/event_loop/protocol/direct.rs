use super::*;
use crate::server::command;

pub(super) enum DirectOperation {
    Idle,
    WaitingClientRead {
        transaction: CommandTransaction,
        args: Vec<String>,
        data: Vec<u8>,
    },
    WaitingClientWriteReady {
        transaction: CommandTransaction,
        request: command::ClientFileWrite,
    },
    WritingClientFile {
        transaction: Option<CommandTransaction>,
        request: command::ClientFileWrite,
        offset: usize,
        close_generated: bool,
    },
    WaitingCommand {
        pending: bool,
    },
    Responding(CommandResponse),
}

impl ProtocolClient {
    pub(super) fn begin_command(
        &mut self,
        target: &ActorRef<Self>,
        args: Vec<String>,
        outbox: &mut Outbox,
    ) {
        self.start_command_work(
            target,
            CommandWork::Initial {
                args,
                context: self.context.clone(),
            },
            outbox,
        );
    }

    pub(super) fn start_command_work(
        &mut self,
        target: &ActorRef<Self>,
        work: CommandWork,
        outbox: &mut Outbox,
    ) {
        self.completion = None;
        self.operation = DirectOperation::Idle;
        let step = run_command_work(work, &self.state);
        outbox.enqueue_protocol(target.clone(), ProtocolEvent::CommandStepReady(step));
    }

    pub(super) fn drive_resumable_command(&mut self, target: &ActorRef<Self>, outbox: &mut Outbox) {
        let Some(mut active) = self.resumable_command.take() else {
            return;
        };
        match active.queue.drive(&self.state, COMMAND_QUEUE_BUDGET) {
            command::ResumableCommandTurn::Pending => {
                self.resumable_command = Some(active);
                outbox.enqueue_protocol(target.clone(), ProtocolEvent::CommandQueueContinue);
            }
            command::ResumableCommandTurn::Suspended(suspension) => {
                match PendingCommand::start_suspension(suspension) {
                    Ok(completion) => {
                        let complete = completion.is_complete();
                        let deadline = completion.deadline();
                        self.resumable_command = Some(active);
                        self.completion = Some(completion);
                        self.operation = DirectOperation::WaitingCommand { pending: true };
                        if complete {
                            outbox
                                .enqueue_protocol(target.clone(), ProtocolEvent::CommandCompleted);
                        } else {
                            outbox.set_protocol_interest(
                                target.clone(),
                                ProtocolIoSide::Command,
                                true,
                            );
                            if let Some(deadline) = deadline {
                                self.command_generation = self.command_generation.wrapping_add(1);
                                outbox.set_protocol_timer_event(
                                    target.clone(),
                                    deadline,
                                    ProtocolEvent::CommandTimeout(self.command_generation),
                                );
                            }
                        }
                    }
                    Err(error) => {
                        self.close(target, ProtocolCloseReason::Error(error.kind()), outbox);
                    }
                }
            }
            command::ResumableCommandTurn::Complete(mut result) => {
                for request in result.background_commands.drain(..) {
                    outbox.enqueue_background(
                        self.background_commands.clone(),
                        super::super::job::JobEvent::Start(request),
                    );
                }
                let mut transaction = active.transaction;
                if transaction.complete_group(&result) {
                    transaction.groups.clear();
                }
                self.start_command_work(target, CommandWork::Advance(transaction), outbox);
            }
        }
    }

    pub(super) fn handle_command_completed(
        &mut self,
        target: &ActorRef<Self>,
        outbox: &mut Outbox,
    ) {
        let had_deadline = self
            .completion
            .as_ref()
            .is_some_and(|completion| completion.deadline().is_some());
        let completed = match self
            .completion
            .as_mut()
            .expect("command readiness without a completion")
            .take_result()
        {
            Ok(result) => result,
            Err(error) => {
                self.close(target, ProtocolCloseReason::Error(error.kind()), outbox);
                return;
            }
        };
        match &mut self.operation {
            DirectOperation::WaitingCommand { pending } if *pending => *pending = false,
            _ => {
                self.close(
                    target,
                    ProtocolCloseReason::Error(io::ErrorKind::InvalidData),
                    outbox,
                );
                return;
            }
        }
        self.operation = DirectOperation::Idle;
        outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Command, false);
        if had_deadline {
            outbox.cancel_protocol_timer(target.clone());
        }
        let Some(active) = self.resumable_command.as_mut() else {
            self.close(
                target,
                ProtocolCloseReason::Error(io::ErrorKind::InvalidData),
                outbox,
            );
            return;
        };
        active.queue.resume(completed, &self.state);
        outbox.enqueue_protocol(target.clone(), ProtocolEvent::CommandQueueContinue);
    }

    pub(super) fn handle_command_step(
        &mut self,
        target: &ActorRef<Self>,
        step: CommandStep,
        outbox: &mut Outbox,
    ) {
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
                        self.resumable_command =
                            Some(ActiveResumableCommand { transaction, queue });
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
                self.operation = DirectOperation::WaitingClientRead {
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
                request,
            } => {
                let mut path = request.path.as_os_str().as_bytes().to_vec();
                path.push(0);
                let flags = request.flags;
                self.operation = DirectOperation::WaitingClientWriteReady {
                    transaction,
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
                if let DirectOperation::WaitingClientRead { data: buffered, .. } =
                    &mut self.operation
                {
                    buffered.extend_from_slice(&data);
                }
            }
            Message::ReadDone {
                stream: FILE_STREAM,
                error,
            } => {
                let operation = std::mem::replace(&mut self.operation, DirectOperation::Idle);
                let DirectOperation::WaitingClientRead {
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
                let operation = std::mem::replace(&mut self.operation, DirectOperation::Idle);
                let DirectOperation::WaitingClientWriteReady {
                    mut transaction,
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
                    self.operation = DirectOperation::WritingClientFile {
                        transaction: Some(transaction),
                        request,
                        offset: 0,
                        close_generated: false,
                    };
                    self.drive_output(target, outbox);
                }
            }
            Message::WriteReady { stream, error } => {
                if let DirectOperation::Responding(response) = &mut self.operation {
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
