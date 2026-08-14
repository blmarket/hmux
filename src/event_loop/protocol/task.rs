//! One accepted connection, one task.
//!
//! The task is the client: it registers the socket, identifies the peer, and
//! then *is* whichever kind of client the peer turned out to be until that
//! client ends. Its owner keeps only what outlives it — whether it is still
//! running, and why it stopped.

use crate::server::Server;
use crate::tmux::codec::{ImsgReader, NonblockingImsgWriter};
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
