//! Task-hosted tmux protocol clients.
//!
//! One connection is one task, and what the connection is doing is where that
//! task is suspended. A client identifies itself, becomes a command client, a
//! control-mode client or an interactive attach, and runs to its end as that —
//! no shared event enum, no effect queue, and no central loop holding the state
//! each connection is in.

mod attach;
mod command;
mod control;
mod identify;
mod task;
mod wire;

use std::cell::Cell;
use std::io;
use std::rc::Rc;

use crate::integration::status::StatusHub;
use crate::server::command::CommandRuntime;
use crate::server::state::SharedState;
use hmux_rt::TaskHandle;

use super::job::BackgroundRunner;

pub(crate) use task::{spawn, ProtocolHandle};

/// How much of one stream a single protocol frame carries.
const OUTPUT_CHUNK: usize = 8 * 1024;

/// Why a protocol client stopped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProtocolCloseReason {
    Completed,
    PeerClosed,
    Error(io::ErrorKind),
    IdentifyExceedsLimit,
    FrameExceedsQueueLimit,
}

/// What every client on a connection shares: the server it speaks for, and the
/// runtime it starts work on.
pub(super) struct ClientRuntime {
    pub(super) tasks: TaskHandle,
    pub(super) state: SharedState,
    pub(super) hub: StatusHub,
    pub(super) background: BackgroundRunner,
    pub(super) commands: Rc<dyn CommandRuntime>,
}

/// What a connection has become, published for its owner.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum ProtocolKind {
    #[default]
    Identifying,
    Command,
    Control,
    Attach,
}

/// The part of a connection that outlives its task.
///
/// The owner reaps clients after their tasks end, so why one ended has to be
/// readable once nothing of it is left.
#[derive(Clone, Default)]
pub(crate) struct ProtocolStatus {
    close_reason: Rc<Cell<Option<ProtocolCloseReason>>>,
    kind: Rc<Cell<ProtocolKind>>,
}

impl ProtocolStatus {
    fn close(&self, reason: ProtocolCloseReason) {
        self.close_reason.set(Some(reason));
    }

    fn set_kind(&self, kind: ProtocolKind) {
        self.kind.set(kind);
    }

    pub(crate) fn close_reason(&self) -> Option<ProtocolCloseReason> {
        self.close_reason.get()
    }

    #[cfg(test)]
    fn kind(&self) -> ProtocolKind {
        self.kind.get()
    }
}
