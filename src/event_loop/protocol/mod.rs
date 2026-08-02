//! Event-loop-owned tmux command-client and control-mode protocol state.

mod attach;
mod client;
mod command;
mod control;
mod direct;
mod response;

use super::actor::ActorRef;
use super::driver::Envelope;
use super::driver::Outbox;
use super::job::BackgroundCommands;
use super::reactor::Token;
use super::timer::TimerId;
use crate::integration::status::StatusHub;
use crate::server::attach::ClientTty;
use crate::server::command::ClientContext;
use crate::server::command::CommandResult;
use crate::server::control::EventControlClient;
use crate::server::control::EventControlSource;
use crate::server::state::ServerState;
use crate::server::Server;
use crate::tmux::codec::encode_bytes;
use crate::tmux::codec::ImsgReader;
use crate::tmux::codec::NonblockingImsgWriter;
use crate::tmux::codec::MAX_IMSGSIZE;
use crate::tmux::introspect::log_frame;
use crate::tmux::introspect::Direction;
use crate::tmux::message::Frame;
use crate::tmux::message::Message;
use crate::tmux::message::PROTOCOL_VERSION;
use crate::tmux::traits::NonblockingFrameReader;
use crate::tmux::traits::NonblockingFrameWriter;
use std::cell::Cell;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;
use std::io;
use std::io::Read;
use std::io::Write;
use std::os::fd::AsFd;
use std::os::fd::BorrowedFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Instant;

use attach::{EventAttachClient, EventAttachSource};
pub(crate) use client::{
    dispatch_inbox, ClientInbox, ClientInboxEvent, ClientIo, ClientIoEvent, ProtocolClient,
    ProtocolCloseReason, ProtocolEvent, ProtocolIoSide, ProtocolStatus, COMMAND_QUEUE_BUDGET,
};
#[cfg(test)]
pub(crate) use client::{CloseReason, READ_FRAME_BUDGET};
use client::{ProtocolMode, FILE_STREAM};
use command::{
    run_command_work, ActiveResumableCommand, CommandStep, CommandTransaction, CommandWork,
    PendingCommand,
};
use direct::DirectOperation;
use response::CommandResponse;

const OUTPUT_CHUNK: usize = 8 * 1024;
