//! Event-loop-owned tmux protocol actors.

mod attach;
mod client;
mod command;
mod control;
mod direct;
mod response;

const OUTPUT_CHUNK: usize = 8 * 1024;

pub(crate) use client::{
    ProtocolClient, ProtocolCloseReason, ProtocolEvent, ProtocolIoSide, ProtocolStatus,
};
