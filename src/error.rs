//! Single error type for the whole crate.
//!
//! Decode never lives here: an imsg we can't classify becomes
//! [`crate::tmux::Message::Unknown`] and is forwarded verbatim, so a malformed
//! *frame body* is never fatal. `Codec` is reserved for genuine framing
//! violations (a length field that can't be a valid imsg, a short header on a
//! stream that isn't at EOF, etc.).

use std::io;

use thiserror::Error;

/// Crate-wide result alias.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    /// Underlying socket / syscall failure.
    #[error("io: {0}")]
    Io(#[from] io::Error),

    /// The imsg framing itself is invalid (bad length, impossible header).
    #[error("codec: {0}")]
    Codec(String),

    /// Failure managing the backing tmux process.
    #[error("backing tmux: {0}")]
    Backing(String),

    /// A control-plane protocol violation surfaced by a peer.
    #[error("protocol: {0}")]
    Protocol(String),
}

impl Error {
    pub fn codec(msg: impl Into<String>) -> Self {
        Error::Codec(msg.into())
    }

    pub fn backing(msg: impl Into<String>) -> Self {
        Error::Backing(msg.into())
    }
}
