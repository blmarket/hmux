use std::collections::BTreeSet;
use std::rc::Rc;
use std::time::Instant;

use crate::server::control::{EventControlClient, EventControlSource};
use crate::tmux::message::{Frame, Message};

use super::task::Outbox;
use super::client::{
    ControlClientState, ProtocolClient, ProtocolCloseReason, ProtocolEvent, ProtocolIoSide,
    ProtocolState,
};

impl ProtocolClient {
    pub(super) fn begin_control(
        &mut self,
        args: Vec<String>,
        outbox: &mut Outbox,
    ) {
        let Some((tty, context)) = self.take_identification() else {
            return;
        };
        match EventControlClient::new(
            &args,
            tty,
            Rc::clone(&self.state),
            self.hub.clone(),
            &context,
            Rc::clone(&self.command_runtime),
        ) {
            Ok(mut control) => {
                if let Err(error) = control.drive(None) {
                    self.close(ProtocolCloseReason::Error(error.kind()), outbox);
                    return;
                }
                self.protocol_state = ProtocolState::Control(ControlClientState {
                    client: Box::new(control),
                    timer_deadline: None,
                    timer_generation: 0,
                });
                // Control mode moves subsequent input to the passed stdin fd.
                // The native control loop likewise stops reading imsg here.
                outbox.set_protocol_interest(ProtocolIoSide::Read, false);
                self.sync_control(outbox);
            }
            Err(error) => {
                self.close(ProtocolCloseReason::Error(error.kind()), outbox);
            }
        }
    }

    pub(super) fn handle_control_protocol_frame(
        &mut self,
        frame: Frame,
        outbox: &mut Outbox,
    ) {
        if matches!(
            frame.msg,
            Message::Detach(_) | Message::DetachKill(_) | Message::Exit(_) | Message::Shutdown
        ) {
            self.close(ProtocolCloseReason::Completed, outbox);
        }
    }

    pub(super) fn handle_control_event(
        &mut self,
        source: Option<EventControlSource>,
        outbox: &mut Outbox,
    ) {
        let result = match &mut self.protocol_state {
            ProtocolState::Control(control) => control.client.drive(source),
            _ => return,
        };
        if let Err(error) = result {
            tracing::warn!(%error, "event-loop control client failed");
            self.close(ProtocolCloseReason::Error(error.kind()), outbox);
            return;
        }
        self.sync_control(outbox);
    }

    pub(super) fn sync_control(&mut self, outbox: &mut Outbox) {
        let (desired, deadline, finished) = match &self.protocol_state {
            ProtocolState::Control(control) => (
                control
                    .client
                    .sources()
                    .into_iter()
                    .collect::<BTreeSet<_>>(),
                control.client.deadline(Instant::now()),
                control.client.is_finished(),
            ),
            _ => return,
        };
        // A command source holds a wake rather than a token, but it is still
        // something this client has taken out and has to give back: without it
        // here, the source is never disabled and the client accumulates one
        // stale entry per suspension it goes through.
        let registered = self
            .registrations
            .control
            .iter()
            .copied()
            .chain(
                self.registrations
                    .command_wakes
                    .iter()
                    .filter_map(|side| match side {
                        ProtocolIoSide::Control(source) => Some(*source),
                        _ => None,
                    }),
            )
            .collect::<BTreeSet<_>>();
        for source in registered.union(&desired).copied() {
            outbox.set_protocol_interest(
                ProtocolIoSide::Control(source),
                desired.contains(&source),
            );
        }

        let timer_changed = match &self.protocol_state {
            ProtocolState::Control(control) => deadline != control.timer_deadline,
            _ => false,
        };
        if timer_changed {
            let generation = match &mut self.protocol_state {
                ProtocolState::Control(control) => {
                    control.timer_deadline = deadline;
                    control.timer_generation = control.timer_generation.wrapping_add(1);
                    control.timer_generation
                }
                _ => return,
            };
            if let Some(deadline) = deadline {
                outbox.set_protocol_timer_event(
                    deadline,
                    ProtocolEvent::ControlTimer(generation),
                );
            }
        }

        while self.writer_is_below_high_water() {
            let frame = match &mut self.protocol_state {
                ProtocolState::Control(control) => control.client.pop_frame(),
                _ => None,
            };
            let Some(frame) = frame else {
                break;
            };
            if !self.queue_frame(frame, outbox) {
                return;
            }
        }
        let continue_input = match &mut self.protocol_state {
            ProtocolState::Control(control) => control.client.take_input_continuation(),
            _ => false,
        };
        if continue_input
            && self.mark_work_queued(ProtocolIoSide::Control(EventControlSource::Input))
        {
            outbox.enqueue_protocol(ProtocolEvent::ControlReady(EventControlSource::Input));
        }
        let (continue_command, background_commands) = match &mut self.protocol_state {
            ProtocolState::Control(control) => (
                control.client.take_command_continuation(),
                control.client.take_background_commands(),
            ),
            _ => (false, Vec::new()),
        };
        if continue_command {
            outbox.enqueue_protocol(ProtocolEvent::ControlContinue);
        }
        for request in background_commands {
            self.background_commands.start(request);
        }
        if finished {
            self.protocol_state = ProtocolState::Draining;
            outbox.set_protocol_interest(ProtocolIoSide::Read, false);
        }
        self.drive_output(outbox);
    }
}
