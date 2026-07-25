//! Client for the hmux status push protocol (`MSG_HMUX_STATUS_WAIT` /
//! `MSG_HMUX_STATUS`, see PROTOCOL.md).
//!
//! A bespoke status TUI can't use the stock `tmux` binary for push — the long
//! poll is an hmux extension a real tmux client never speaks — so it talks imsg
//! directly. This helper wraps that: connect, then repeatedly [`wait`] on the
//! last revision to get near-real-time per-pane agent status.
//!
//! It is a **native-hmux-only** client. Pointed at a real tmux server, the first
//! `StatusWait` (type 900) would be rejected; only hmux recognizes the type.
//!
//! [`wait`]: StatusClient::wait

use std::io;
use std::os::unix::net::UnixStream;
use std::path::Path;

use crate::tmux::codec::split_stream;
use crate::tmux::codec::{ImsgReader, ImsgWriter};
use crate::tmux::message::{Frame, Message};

/// One pane's decoded status record from a `MSG_HMUX_STATUS` body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaneStatus {
    /// tmux pane id, `%N`.
    pub pane_id: String,
    /// Owning session name.
    pub session: String,
    /// `#{window_index}` of the pane's window.
    pub window_index: String,
    /// `#{pane_index}` within the window.
    pub pane_index: String,
    /// Agent label: `""`, `"codex"`, `"claude"`, or `"pi"`.
    pub agent: String,
    /// Lifecycle state: `idle` | `working` | `blocked` | `exited` | `none`.
    pub state: String,
}

/// A decoded status response: the hub revision plus one record per pane.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusUpdate {
    /// The hub revision this snapshot represents. Pass it back to [`wait`] to
    /// long-poll for the next change.
    ///
    /// [`wait`]: StatusClient::wait
    pub revision: u64,
    /// Per-pane records across the whole server (`list-panes -a` order).
    pub panes: Vec<PaneStatus>,
}

/// A push client speaking the hmux status long-poll protocol.
pub struct StatusClient {
    reader: ImsgReader,
    writer: ImsgWriter,
}

impl StatusClient {
    /// Connect to a native hmux server listening on the unix socket at `path`.
    pub fn connect(path: impl AsRef<Path>) -> io::Result<Self> {
        let stream = UnixStream::connect(path)?;
        let (reader, writer) = split_stream(stream)?;
        Ok(StatusClient { reader, writer })
    }

    /// Wrap an already-connected frame pair (e.g. from
    /// [`TmuxServer::connect`](crate::tmux::TmuxServer::connect)).
    pub fn from_pair(reader: ImsgReader, writer: ImsgWriter) -> Self {
        StatusClient { reader, writer }
    }

    /// Send `StatusWait { since }` and block until the server replies with a
    /// `Status` frame, returning the decoded update.
    ///
    /// Start with `since = 0` to get the current full snapshot immediately, then
    /// pass the returned [`StatusUpdate::revision`] back to block until the next
    /// change (or the server's ~30 s heartbeat, which returns the same revision).
    /// An error means the server is gone.
    pub fn wait(&mut self, since: u64) -> io::Result<StatusUpdate> {
        self.writer
            .send(Frame::new(Message::StatusWait { since }))?;
        loop {
            let frame = self.reader.recv()?;
            if let Message::Status { revision, body } = frame.msg {
                return Ok(StatusUpdate {
                    revision,
                    panes: parse_body(&body),
                });
            }
            // Ignore any other control frame and keep waiting for the Status.
        }
    }
}

/// Parse a `MSG_HMUX_STATUS` body into records — the inverse of
/// `command::encode_status_body`. Lines that don't have exactly six
/// tab-separated fields are skipped, so a forward-compatible server can't crash
/// an older client.
fn parse_body(body: &[u8]) -> Vec<PaneStatus> {
    let text = String::from_utf8_lossy(body);
    let mut panes = Vec::new();
    for line in text.lines() {
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 6 {
            continue;
        }
        panes.push(PaneStatus {
            pane_id: fields[0].to_string(),
            session: fields[1].to_string(),
            window_index: fields[2].to_string(),
            pane_index: fields[3].to_string(),
            agent: fields[4].to_string(),
            state: fields[5].to_string(),
        });
    }
    panes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_body_decodes_records_and_skips_malformed() {
        let body = b"%0\t0\t0\t0\t\tnone\n\
                     %1\twork\t2\t1\tclaude\tworking\n\
                     garbage-line-without-tabs\n";
        let panes = parse_body(body);
        assert_eq!(panes.len(), 2);
        assert_eq!(
            panes[0],
            PaneStatus {
                pane_id: "%0".into(),
                session: "0".into(),
                window_index: "0".into(),
                pane_index: "0".into(),
                agent: "".into(),
                state: "none".into(),
            }
        );
        assert_eq!(
            panes[1],
            PaneStatus {
                pane_id: "%1".into(),
                session: "work".into(),
                window_index: "2".into(),
                pane_index: "1".into(),
                agent: "claude".into(),
                state: "working".into(),
            }
        );
    }

    #[test]
    fn parse_body_empty_is_no_panes() {
        assert!(parse_body(b"").is_empty());
    }
}
