//! Task-hosted tmux protocol clients.

mod attach;
mod client;
mod command;
mod control;
mod direct;
mod response;
mod task;

const OUTPUT_CHUNK: usize = 8 * 1024;

pub(crate) use client::{
    ProtocolClient, ProtocolCloseReason, ProtocolEvent, ProtocolIoSide, ProtocolStatus,
};
pub(crate) use task::{spawn, ProtocolHandle};
