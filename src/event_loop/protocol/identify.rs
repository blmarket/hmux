//! The handshake every connection starts with, up to the point where the
//! client says what it is.
//!
//! tmux clients open with a run of identify messages and then one command,
//! which is what decides whether the connection becomes a command client, a
//! control-mode client or an interactive attach. That is a script rather than a
//! state: read frames, fold each into the identification being built, and stop
//! at the frame that decides.

use crate::server::attach::ClientTty;
use crate::server::command::{self, ClientContext, ClientKind};
use crate::server::state::{ACCESS_DENIED, AclJoin, SharedState};
use crate::tmux::codec::encode_bytes;
use crate::tmux::message::{Frame, Message, PROTOCOL_VERSION};

use super::wire::Wire;
use super::{ProtocolCloseReason, ProtocolKind};

const CLIENT_CONTROL: i64 = 0x2000;
const IDENTIFY_LIMIT: usize = crate::tmux::codec::MAX_IMSGSIZE * 64;

/// What the deciding frame turned the connection into.
pub(super) enum Role {
    Command {
        args: Vec<String>,
        context: ClientContext,
    },
    Control {
        args: Vec<String>,
        tty: ClientTty,
        context: ClientContext,
    },
    Attach {
        args: Vec<String>,
        tty: ClientTty,
        context: ClientContext,
    },
}

impl Role {
    /// The command line the peer asked for, whichever kind of client it turned
    /// out to be.
    pub(super) fn args(&self) -> &[String] {
        match self {
            Self::Command { args, .. } | Self::Control { args, .. } | Self::Attach { args, .. } => {
                args
            }
        }
    }

    /// Which kind of client this connection became, for the status its owner
    /// reads.
    pub(super) fn kind(&self) -> ProtocolKind {
        match self {
            Self::Command { .. } => ProtocolKind::Command,
            Self::Control { .. } => ProtocolKind::Control,
            Self::Attach { .. } => ProtocolKind::Attach,
        }
    }

    pub(super) fn context(&self) -> &ClientContext {
        match self {
            Self::Command { context, .. }
            | Self::Control { context, .. }
            | Self::Attach { context, .. } => context,
        }
    }
}

/// The identification being folded together, one frame at a time.
struct Identification {
    context: ClientContext,
    tty: ClientTty,
    bytes: usize,
    control_mode: bool,
}

/// Read frames until the client says what it is.
///
/// An `Err` is the connection ending here rather than a failure to identify:
/// a client that asks for the default shell, one that leaves, and one speaking
/// another protocol version all get their answer and their close from inside.
pub(super) async fn identify(
    wire: &mut Wire,
    state: &SharedState,
    peer_uid: Option<u32>,
) -> Result<Role, ProtocolCloseReason> {
    let mut identification = Identification {
        context: ClientContext {
            kind: ClientKind::Command,
            peer_uid,
            ..ClientContext::default()
        },
        tty: ClientTty::new(),
        bytes: 0,
        control_mode: false,
    };
    let mut joined = false;

    loop {
        let mut frame = wire.recv().await?;
        if frame.version != PROTOCOL_VERSION {
            // The client is told the version it should have been speaking and
            // then left to close; answering first is what makes the mismatch
            // legible to it.
            wire.queue(Frame::new(Message::Version))?;
            wire.drain().await?;
            return Err(ProtocolCloseReason::Completed);
        }

        // tmux's `server_acl_join`, which it runs at accept: the peer's uid
        // decides whether it is served at all, and whether it is served
        // read-only. It runs here instead so a peer speaking another protocol
        // version still gets the version answer it came for, and a refused one
        // is told why over a connection it can read.
        if !std::mem::replace(&mut joined, true) {
            let join = state.borrow_mut().acl_join(peer_uid);
            match join {
                AclJoin::Denied => {
                    wire.queue(Frame::new(Message::Exit(
                        Some(0),
                        Some(ACCESS_DENIED.to_owned()),
                    )))?;
                    wire.drain().await?;
                    return Err(ProtocolCloseReason::Completed);
                }
                AclJoin::Allowed { read_only } => {
                    identification.context.read_only = read_only;
                }
            }
        }

        if is_identify_message(&frame.msg) {
            let frame_bytes = encode_bytes(&frame).len();
            if frame_bytes > IDENTIFY_LIMIT.saturating_sub(identification.bytes) {
                return Err(ProtocolCloseReason::IdentifyExceedsLimit);
            }
            identification.bytes += frame_bytes;
        }
        identification.observe(&mut frame);

        let args = match frame.msg {
            Message::Command(args) => args,
            // `tmux -c command`: the client wants the shell to exec, not a
            // command run here. tmux answers with `default-shell` and drops the
            // peer — the client execs it itself — so this never becomes an
            // attach or a command client.
            Message::Shell(None) => {
                let shell = command::default_shell(&state.borrow_mut(), None);
                wire.queue(Frame::new(Message::Shell(Some(shell))))?;
                // The answer has to reach the client before the peer goes: it
                // execs on receipt, and a close that overtook the frame would
                // look like the server dying.
                wire.drain().await?;
                return Err(ProtocolCloseReason::Completed);
            }
            Message::Detach(_) | Message::DetachKill(_) | Message::Exit(..) | Message::Shutdown => {
                return Err(ProtocolCloseReason::Completed);
            }
            _ => continue,
        };

        let args = if args.is_empty() {
            command::default_client_command(&state.borrow_mut())
        } else {
            args
        };
        let Identification {
            context,
            tty,
            control_mode,
            ..
        } = identification;
        return Ok(if control_mode {
            Role::Control { args, tty, context }
        } else if command::classify(&args) == command::Intent::Command {
            Role::Command { args, context }
        } else {
            Role::Attach { args, tty, context }
        });
    }
}

impl Identification {
    /// Fold one identify frame in, taking any descriptor it carries.
    fn observe(&mut self, frame: &mut Frame) {
        match &frame.msg {
            Message::IdentifyFlags(flags) => {
                self.control_mode |= i64::from(*flags) & CLIENT_CONTROL != 0;
                self.tty.flags |= i64::from(*flags);
            }
            Message::IdentifyLongFlags(flags) | Message::Flags(flags) => {
                self.control_mode |= *flags & CLIENT_CONTROL != 0;
                self.tty.flags |= *flags;
            }
            Message::IdentifyTerminfo(capability) => {
                self.tty.terminfo.push(capability.clone());
            }
            Message::IdentifyFeatures(features) => {
                self.tty.features |= *features as u32;
            }
            Message::IdentifyTerm(term) => {
                self.tty.term = Some(term.clone());
            }
            Message::IdentifyTtyName(tty_name) => {
                self.context.tty_name = Some(tty_name.clone());
                self.tty.tty_name = Some(tty_name.clone());
            }
            Message::IdentifyClientPid(pid) => {
                self.context.client_pid = Some(*pid);
                self.tty.client_pid = Some(*pid);
            }
            Message::IdentifyCwd(cwd) => {
                self.context.cwd = Some(cwd.into());
            }
            Message::IdentifyEnviron(entry) => {
                self.context.environment.push(entry.clone());
            }
            Message::IdentifyStdin => {
                self.tty.stdin = frame.fd.take();
            }
            Message::IdentifyStdout => {
                self.tty.stdout = frame.fd.take();
            }
            _ => {}
        }
    }
}

fn is_identify_message(message: &Message) -> bool {
    matches!(
        message,
        Message::IdentifyFlags(_)
            | Message::IdentifyLongFlags(_)
            | Message::Flags(_)
            | Message::IdentifyTerminfo(_)
            | Message::IdentifyFeatures(_)
            | Message::IdentifyTerm(_)
            | Message::IdentifyTtyName(_)
            | Message::IdentifyClientPid(_)
            | Message::IdentifyCwd(_)
            | Message::IdentifyEnviron(_)
            | Message::IdentifyStdin
            | Message::IdentifyStdout
            | Message::IdentifyDone
    )
}
