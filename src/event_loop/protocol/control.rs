use super::*;

impl ProtocolClient {
    pub(super) fn begin_control(
        &mut self,
        target: &ActorRef<Self>,
        args: Vec<String>,
        outbox: &mut Outbox,
    ) {
        let tty = self.take_client_tty();
        match EventControlClient::new(
            &args,
            tty,
            Arc::clone(&self.state),
            self.hub.clone(),
            &self.context,
        ) {
            Ok(mut control) => {
                if let Err(error) = control.drive(None) {
                    self.close(target, ProtocolCloseReason::Error(error.kind()), outbox);
                    return;
                }
                self.mode = ProtocolMode::Control(Box::new(control));
                self.operation = DirectOperation::Idle;
                // Control mode moves subsequent input to the passed stdin fd.
                // The native control loop likewise stops reading imsg here.
                outbox.set_protocol_interest(target.clone(), ProtocolIoSide::Read, false);
                self.sync_control(target, outbox);
            }
            Err(error) => {
                self.close(target, ProtocolCloseReason::Error(error.kind()), outbox);
            }
        }
    }

    pub(super) fn handle_control_protocol_frame(
        &mut self,
        target: &ActorRef<Self>,
        frame: Frame,
        outbox: &mut Outbox,
    ) {
        if matches!(
            frame.msg,
            Message::Detach(_) | Message::DetachKill(_) | Message::Exit(_) | Message::Shutdown
        ) {
            self.close(target, ProtocolCloseReason::Completed, outbox);
        }
    }

    pub(super) fn handle_control_event(
        &mut self,
        target: &ActorRef<Self>,
        source: Option<EventControlSource>,
        outbox: &mut Outbox,
    ) {
        let result = match &mut self.mode {
            ProtocolMode::Control(control) => control.drive(source),
            _ => return,
        };
        if let Err(error) = result {
            tracing::warn!(%error, "event-loop control client failed");
            self.close(target, ProtocolCloseReason::Error(error.kind()), outbox);
            return;
        }
        self.sync_control(target, outbox);
    }

    pub(super) fn sync_control(&mut self, target: &ActorRef<Self>, outbox: &mut Outbox) {
        let (desired, deadline, finished) = match &self.mode {
            ProtocolMode::Control(control) => (
                control.sources().into_iter().collect::<BTreeSet<_>>(),
                control.deadline(Instant::now()),
                control.is_finished(),
            ),
            _ => return,
        };
        let registered = self.control_tokens.keys().copied().collect::<BTreeSet<_>>();
        for source in registered.union(&desired).copied() {
            outbox.set_protocol_interest(
                target.clone(),
                ProtocolIoSide::Control(source),
                desired.contains(&source),
            );
        }

        if deadline != self.control_timer_deadline {
            self.control_timer_deadline = deadline;
            self.control_timer_generation = self.control_timer_generation.wrapping_add(1);
            match deadline {
                Some(deadline) => outbox.set_protocol_timer_event(
                    target.clone(),
                    deadline,
                    ProtocolEvent::ControlTimer(self.control_timer_generation),
                ),
                None => outbox.cancel_protocol_timer(target.clone()),
            }
        }

        while self.writer_is_below_high_water() {
            let frame = match &mut self.mode {
                ProtocolMode::Control(control) => control.pop_frame(),
                _ => None,
            };
            let Some(frame) = frame else {
                break;
            };
            if !self.queue_frame(target, frame, outbox) {
                return;
            }
        }
        let continue_input = match &mut self.mode {
            ProtocolMode::Control(control) => control.take_input_continuation(),
            _ => false,
        };
        if continue_input
            && self.mark_work_queued(ProtocolIoSide::Control(EventControlSource::Input))
        {
            outbox.enqueue_protocol(
                target.clone(),
                ProtocolEvent::ControlReady(EventControlSource::Input),
            );
        }
        let (continue_command, background_commands) = match &mut self.mode {
            ProtocolMode::Control(control) => (
                control.take_command_continuation(),
                control.take_background_commands(),
            ),
            _ => (false, Vec::new()),
        };
        if continue_command {
            outbox.enqueue_protocol(target.clone(), ProtocolEvent::ControlContinue);
        }
        for request in background_commands {
            outbox.enqueue_background(
                self.background_commands.clone(),
                super::super::job::JobEvent::Start(request),
            );
        }
        if finished {
            self.close_after_flush = true;
        }
        self.drive_output(target, outbox);
    }
}
