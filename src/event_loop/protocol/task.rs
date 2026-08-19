//! One accepted connection, one task.
//!
//! The task is the client: it registers the socket, identifies the peer, and
//! then *is* whichever kind of client the peer turned out to be until that
//! client ends. Its owner keeps only what outlives it — whether it is still
//! running, and why it stopped.

use std::io;
use std::os::fd::AsRawFd;

use crate::server::attach::ClientTty;
use crate::server::command::{self as server_command, ClientContext, CommandResult};
use crate::server::state::SharedState;
use crate::server::Server;
use crate::tmux::codec::{ImsgReader, NonblockingImsgWriter};
use crate::tmux::message::{Frame, Message};
use hmux_rt::{JoinHandle, TaskHandle};

use super::super::job::BackgroundRunner;
use super::identify::{identify, Role};
use super::wire::Wire;
use super::{
    attach, command, control, ClientRuntime, ProtocolCloseReason, ProtocolKind, ProtocolStatus,
};

/// One running protocol client, from its owner's side.
pub(crate) struct ProtocolHandle {
    status: ProtocolStatus,
    task: JoinHandle<()>,
}

impl ProtocolHandle {
    pub(crate) fn is_alive(&self) -> bool {
        !self.task.is_finished()
    }

    pub(crate) fn close_reason(&self) -> Option<ProtocolCloseReason> {
        self.status.close_reason()
    }

    /// Whether the peer has identified itself as something in particular.
    #[cfg(test)]
    pub(crate) fn is_direct(&self) -> bool {
        self.status.kind() != ProtocolKind::Identifying
    }

    #[cfg(test)]
    pub(crate) fn is_control(&self) -> bool {
        self.status.kind() == ProtocolKind::Control
    }

    #[cfg(test)]
    pub(crate) fn is_attach(&self) -> bool {
        self.status.kind() == ProtocolKind::Attach
    }
}

impl Drop for ProtocolHandle {
    fn drop(&mut self) {
        self.task.cancel();
    }
}

/// Serve one accepted connection on the loop.
pub(crate) fn spawn(
    tasks: &TaskHandle,
    reader: ImsgReader,
    writer: NonblockingImsgWriter,
    server: Server,
    background: BackgroundRunner,
    peer_uid: Option<u32>,
) -> ProtocolHandle {
    let runtime = ClientRuntime {
        tasks: tasks.clone(),
        state: server.state(),
        hub: server.status_hub(),
        background,
    };
    let status = ProtocolStatus::default();
    let task_status = status.clone();
    let handle = tasks.clone();
    let task = tasks.spawn_join(async move {
        let reason = serve(&handle, reader, writer, runtime, peer_uid, &task_status).await;
        task_status.close(reason);
    });
    ProtocolHandle { status, task }
}

async fn serve(
    tasks: &TaskHandle,
    reader: ImsgReader,
    writer: NonblockingImsgWriter,
    runtime: ClientRuntime,
    peer_uid: Option<u32>,
    status: &ProtocolStatus,
) -> ProtocolCloseReason {
    let mut wire = match Wire::new(tasks, reader, writer) {
        Ok(wire) => wire,
        Err(error) => return ProtocolCloseReason::Error(error.kind()),
    };
    let role = match identify(&mut wire, &runtime.state, peer_uid).await {
        Ok(role) => role,
        Err(reason) => return reason,
    };
    if let Some(refusal) = read_only_refusal(role.args(), role.context(), &runtime.state) {
        status.set_kind(role.kind());
        return match role {
            // tmux's `cmdq_error` routes a diagnostic by the kind of client
            // that asked for it: a control client's goes to its own output
            // stream as a bare line, and a client with no session yet — a
            // command client, or an attach whose command never ran — gets it
            // on standard error.
            Role::Control { tty, .. } => refuse_control_client(&mut wire, &tty, &refusal).await,
            _ => command::report(&mut wire, refusal).await,
        };
    }
    match role {
        Role::Command { args, context } => {
            status.set_kind(ProtocolKind::Command);
            command::run(&mut wire, &runtime, args, context).await
        }
        Role::Control { args, tty, context } => {
            status.set_kind(ProtocolKind::Control);
            control::run(&mut wire, &runtime, args, tty, context).await
        }
        Role::Attach { args, tty, context } => {
            status.set_kind(ProtocolKind::Attach);
            attach::run(&mut wire, &runtime, args, tty, context).await
        }
    }
}

/// Report a refusal to a control client the way `control_write` does — on the
/// stream the client passed as its stdout, outside any `%begin`/`%end` guard,
/// because the command list it named never became a queue item.
async fn refuse_control_client(
    wire: &mut Wire,
    tty: &ClientTty,
    refusal: &CommandResult,
) -> ProtocolCloseReason {
    if let Some(stdout) = tty.stdout.as_ref() {
        let mut written = 0;
        let bytes = refusal.stderr.as_bytes();
        while written < bytes.len() {
            let count = unsafe {
                libc::write(
                    stdout.as_raw_fd(),
                    bytes[written..].as_ptr().cast(),
                    bytes.len() - written,
                )
            };
            match count {
                -1 if io::Error::last_os_error().kind() == io::ErrorKind::Interrupted => {}
                count if count > 0 => written += count as usize,
                _ => break,
            }
        }
    }
    if let Err(reason) = wire.send(Frame::new(Message::Exit(Some(refusal.exit), None))).await {
        return reason;
    }
    if let Err(reason) = wire.flush() {
        return reason;
    }
    command::await_client_close(wire).await
}

/// tmux's read-only check in `server_client_dispatch_command`: a client the
/// ACL joined read-only runs a command line only when *every* command of it is
/// one tmux marks `CMD_READONLY`.
///
/// A line that does not compile is not refused here — the role that owns it
/// reports the parse error, exactly as tmux does before reaching this check.
fn read_only_refusal(
    args: &[String],
    context: &ClientContext,
    state: &SharedState,
) -> Option<CommandResult> {
    if !context.read_only {
        return None;
    }
    let aliases = state.borrow_mut().command_aliases();
    let compiled = server_command::ExecutableCommand::compile_argv(args, &aliases).ok()?;
    (!compiled.all_read_only()).then(|| CommandResult::err("client is read-only\n"))
}
