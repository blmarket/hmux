//! Typed mirror of `tmux-protocol.h` (PROTOCOL_VERSION 8) plus the [`Frame`]
//! wrapper that carries the imsg version byte and an optional `SCM_RIGHTS` fd.
//!
//! Faithfulness contract: for every variant we model, `encode` reproduces the
//! exact payload bytes the client/server would have sent, so a decode → encode
//! round-trip is semantically identical (the `peerid` version byte is preserved
//! separately on the [`Frame`]). Anything we don't model with confidence decodes
//! to [`Message::Unknown`], which stores the raw payload and re-emits it
//! verbatim, so decoding never drops or corrupts an unrecognized frame.

use std::os::fd::OwnedFd;

/// imsg message types (`enum msgtype`, tmux-protocol.h).
pub mod msgtype {
    pub const MSG_VERSION: u32 = 12;

    pub const MSG_IDENTIFY_FLAGS: u32 = 100;
    pub const MSG_IDENTIFY_TERM: u32 = 101;
    pub const MSG_IDENTIFY_TTYNAME: u32 = 102;
    pub const MSG_IDENTIFY_OLDCWD: u32 = 103; // unused by tmux
    pub const MSG_IDENTIFY_STDIN: u32 = 104;
    pub const MSG_IDENTIFY_ENVIRON: u32 = 105;
    pub const MSG_IDENTIFY_DONE: u32 = 106;
    pub const MSG_IDENTIFY_CLIENTPID: u32 = 107;
    pub const MSG_IDENTIFY_CWD: u32 = 108;
    pub const MSG_IDENTIFY_FEATURES: u32 = 109;
    pub const MSG_IDENTIFY_STDOUT: u32 = 110;
    pub const MSG_IDENTIFY_LONGFLAGS: u32 = 111;
    pub const MSG_IDENTIFY_TERMINFO: u32 = 112;

    pub const MSG_COMMAND: u32 = 200;
    pub const MSG_DETACH: u32 = 201;
    pub const MSG_DETACHKILL: u32 = 202;
    pub const MSG_EXIT: u32 = 203;
    pub const MSG_EXITED: u32 = 204;
    pub const MSG_EXITING: u32 = 205;
    pub const MSG_LOCK: u32 = 206;
    pub const MSG_READY: u32 = 207;
    pub const MSG_RESIZE: u32 = 208;
    pub const MSG_SHELL: u32 = 209;
    pub const MSG_SHUTDOWN: u32 = 210;
    pub const MSG_OLDSTDERR: u32 = 211; // unused
    pub const MSG_OLDSTDIN: u32 = 212; // unused
    pub const MSG_OLDSTDOUT: u32 = 213; // unused
    pub const MSG_SUSPEND: u32 = 214;
    pub const MSG_UNLOCK: u32 = 215;
    pub const MSG_WAKEUP: u32 = 216;
    pub const MSG_EXEC: u32 = 217;
    pub const MSG_FLAGS: u32 = 218;

    pub const MSG_READ_OPEN: u32 = 300;
    pub const MSG_READ: u32 = 301;
    pub const MSG_READ_DONE: u32 = 302;
    pub const MSG_WRITE_OPEN: u32 = 303;
    pub const MSG_WRITE: u32 = 304;
    pub const MSG_WRITE_READY: u32 = 305;
    pub const MSG_WRITE_CLOSE: u32 = 306;
    pub const MSG_READ_CANCEL: u32 = 307;

    // ---- hmux-private extensions (see PROTOCOL.md) -----------------------
    // The `900–999` block is reserved for hmux and is disjoint from every tmux
    // assignment above. A real tmux client never sends these; a real tmux server
    // would `fatalx` on them — so they are native-hmux-only by construction.
    /// Long-poll request for pane agent status: client -> server.
    pub const MSG_HMUX_STATUS_WAIT: u32 = 900;
    /// Pane agent status snapshot/heartbeat: server -> client.
    pub const MSG_HMUX_STATUS: u32 = 901;
}

/// The wire protocol version tmux speaks (`PROTOCOL_VERSION`, tmux-protocol.h).
/// It rides the low byte of the imsg header `peerid`.
pub const PROTOCOL_VERSION: u8 = 8;

/// A single imsg: the version byte, the decoded message, and at most one
/// kernel-level `SCM_RIGHTS` descriptor.
#[derive(Debug)]
pub struct Frame {
    /// `peerid & 0xff`. MUST be preserved on re-encode (a peer whose byte != 8
    /// is marked `PEER_BAD` and dropped — see proc.c `peer_check_version`).
    pub version: u8,
    /// The decoded control message.
    pub msg: Message,
    /// SCM_RIGHTS payload (kernel fd). At most one per imsg (OpenBSD imsg wire
    /// limit). This is *distinct* from any `fd` field inside a message payload
    /// (e.g. `ReadOpen.fd`), which is just an integer the peers exchange.
    pub fd: Option<OwnedFd>,
}

impl Frame {
    /// Build a frame at the current protocol version with no fd.
    pub fn new(msg: Message) -> Self {
        Frame {
            version: PROTOCOL_VERSION,
            msg,
            fd: None,
        }
    }

    /// Build a frame carrying an `SCM_RIGHTS` fd.
    pub fn with_fd(msg: Message, fd: OwnedFd) -> Self {
        Frame {
            version: PROTOCOL_VERSION,
            msg,
            fd: Some(fd),
        }
    }
}

/// A decoded control-plane message.
///
/// Note on scope (see design.md "Scope & limits"): keystrokes and screen
/// redraws do **not** appear here — they travel directly over the tty fd passed
/// during identify. This enum is the control plane only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    // ---- identify: client -> server -------------------------------------
    /// Legacy 32-bit flags (`MSG_IDENTIFY_FLAGS`).
    IdentifyFlags(i32),
    /// 64-bit client flags (`MSG_IDENTIFY_LONGFLAGS`) — what tmux actually sends.
    IdentifyLongFlags(i64),
    IdentifyTerm(String),
    IdentifyTtyName(String),
    IdentifyCwd(String),
    IdentifyEnviron(String),
    IdentifyTerminfo(String),
    IdentifyFeatures(i32),
    /// stdin tty; the descriptor rides in [`Frame::fd`], payload is empty.
    IdentifyStdin,
    /// stdout tty; the descriptor rides in [`Frame::fd`], payload is empty.
    IdentifyStdout,
    IdentifyClientPid(i32),
    IdentifyDone,

    // ---- commands / lifecycle: client -> server -------------------------
    /// `MSG_COMMAND`: `argc` followed by NUL-packed argv (see cmd_pack_argv).
    Command(Vec<String>),
    /// `MSG_DETACH`: server→client on detach, carrying the detached session's
    /// name (`server_client_check_exit`, `CLIENT_EXIT_DETACH`); the client prints
    /// `[detached (from session <name>)]`. The payload is empty in other
    /// (nameless) uses, so the name is optional.
    Detach(Option<String>),
    /// `MSG_DETACHKILL`: like [`Message::Detach`] but the client exits with a
    /// hangup reason. Also carries the session name.
    DetachKill(Option<String>),
    /// `MSG_EXIT`: optional return code. (tmux may also append an exit message;
    /// that shape decodes to [`Message::Unknown`] and forwards verbatim.)
    Exit(Option<i32>),
    /// `MSG_LOCK`: server asks the client to temporarily leave tty mode and run
    /// this shell command. The client replies with [`Message::Unlock`].
    Lock(String),
    /// `MSG_UNLOCK`: client finished the lock command and is ready to resume.
    Unlock,
    /// `MSG_SUSPEND`: server asks the client to stop itself with `SIGTSTP`.
    Suspend,
    /// `MSG_WAKEUP`: client resumed after `SIGCONT` and is ready to redraw.
    Wakeup,
    /// `MSG_RESIZE`: sent with an empty payload; the server recomputes size from
    /// the tty.
    Resize,
    Shutdown,
    /// `MSG_FLAGS`: 64-bit client flags update.
    Flags(i64),

    // ---- lifecycle: server -> client ------------------------------------
    Ready,
    /// `MSG_VERSION`: sent server->client only, to reject a bad-version peer.
    Version,
    Exited,
    Exiting,

    // ---- control-mode / command-client file I/O (fixed structs + path) --
    // `path` holds the raw trailing bytes after the fixed struct, verbatim. tmux
    // sends the bare struct (no path) for the stdout/stderr streams and
    // `path\0` for real file opens (file.c), so we keep it as bytes to round-trip
    // both forms exactly. The `fd` here is a payload integer, not the kernel fd
    // (that would be [`Frame::fd`]).
    /// `msg_read_open { int stream; int fd; }` + optional path bytes.
    ReadOpen {
        stream: i32,
        fd: i32,
        path: Vec<u8>,
    },
    Read {
        stream: i32,
        data: Vec<u8>,
    },
    ReadDone {
        stream: i32,
        error: i32,
    },
    ReadCancel {
        stream: i32,
    },
    /// `msg_write_open { int stream; int fd; int flags; }` + optional path bytes.
    WriteOpen {
        stream: i32,
        fd: i32,
        flags: i32,
        path: Vec<u8>,
    },
    Write {
        stream: i32,
        data: Vec<u8>,
    },
    WriteReady {
        stream: i32,
        error: i32,
    },
    WriteClose {
        stream: i32,
    },

    // ---- hmux-private extensions (see PROTOCOL.md) ----------------------
    /// `MSG_HMUX_STATUS_WAIT`: long-poll request — "reply once the status hub's
    /// revision exceeds `since`". Payload is `since` as a native-endian `u64`.
    StatusWait {
        since: u64,
    },
    /// `MSG_HMUX_STATUS`: the status response — the hub `revision` (native-endian
    /// `u64`) followed by `body`, the tab/newline-delimited per-pane records
    /// specified in PROTOCOL.md. `body` may be empty (no panes).
    Status {
        revision: u64,
        body: Vec<u8>,
    },

    // ---- fallback -------------------------------------------------------
    /// Anything not modeled above. Forwarded byte-for-byte, never dropped.
    Unknown {
        type_: u32,
        payload: Vec<u8>,
    },
}

impl Message {
    /// Decode a message body. Infallible: unrecognized or malformed payloads
    /// fall back to [`Message::Unknown`] so they remain lossless.
    pub fn decode(type_: u32, payload: &[u8]) -> Message {
        use msgtype::*;

        let unknown = || Message::Unknown {
            type_,
            payload: payload.to_vec(),
        };

        match type_ {
            MSG_IDENTIFY_FLAGS => i32_body(payload)
                .map(Message::IdentifyFlags)
                .unwrap_or_else(unknown),
            MSG_IDENTIFY_LONGFLAGS => i64_body(payload)
                .map(Message::IdentifyLongFlags)
                .unwrap_or_else(unknown),
            MSG_IDENTIFY_TERM => cstr_body(payload)
                .map(Message::IdentifyTerm)
                .unwrap_or_else(unknown),
            MSG_IDENTIFY_TTYNAME => cstr_body(payload)
                .map(Message::IdentifyTtyName)
                .unwrap_or_else(unknown),
            MSG_IDENTIFY_CWD => cstr_body(payload)
                .map(Message::IdentifyCwd)
                .unwrap_or_else(unknown),
            MSG_IDENTIFY_ENVIRON => cstr_body(payload)
                .map(Message::IdentifyEnviron)
                .unwrap_or_else(unknown),
            MSG_IDENTIFY_TERMINFO => cstr_body(payload)
                .map(Message::IdentifyTerminfo)
                .unwrap_or_else(unknown),
            MSG_IDENTIFY_FEATURES => i32_body(payload)
                .map(Message::IdentifyFeatures)
                .unwrap_or_else(unknown),
            MSG_IDENTIFY_STDIN if payload.is_empty() => Message::IdentifyStdin,
            MSG_IDENTIFY_STDOUT if payload.is_empty() => Message::IdentifyStdout,
            MSG_IDENTIFY_CLIENTPID => i32_body(payload)
                .map(Message::IdentifyClientPid)
                .unwrap_or_else(unknown),
            MSG_IDENTIFY_DONE if payload.is_empty() => Message::IdentifyDone,

            MSG_COMMAND => decode_command(payload).unwrap_or_else(unknown),
            MSG_DETACH if payload.is_empty() => Message::Detach(None),
            MSG_DETACH => cstr_body(payload)
                .map(|s| Message::Detach(Some(s)))
                .unwrap_or_else(unknown),
            MSG_DETACHKILL if payload.is_empty() => Message::DetachKill(None),
            MSG_DETACHKILL => cstr_body(payload)
                .map(|s| Message::DetachKill(Some(s)))
                .unwrap_or_else(unknown),
            MSG_EXIT => match payload.len() {
                0 => Message::Exit(None),
                4 => Message::Exit(Some(i32::from_ne_bytes(payload.try_into().unwrap()))),
                _ => unknown(), // retval + trailing message: forward verbatim
            },
            MSG_LOCK => cstr_body(payload)
                .map(Message::Lock)
                .unwrap_or_else(unknown),
            MSG_UNLOCK if payload.is_empty() => Message::Unlock,
            MSG_SUSPEND if payload.is_empty() => Message::Suspend,
            MSG_WAKEUP if payload.is_empty() => Message::Wakeup,
            MSG_RESIZE if payload.is_empty() => Message::Resize,
            MSG_SHUTDOWN if payload.is_empty() => Message::Shutdown,
            MSG_FLAGS => i64_body(payload)
                .map(Message::Flags)
                .unwrap_or_else(unknown),

            MSG_READY if payload.is_empty() => Message::Ready,
            MSG_VERSION if payload.is_empty() => Message::Version,
            MSG_EXITED if payload.is_empty() => Message::Exited,
            MSG_EXITING if payload.is_empty() => Message::Exiting,

            MSG_READ_OPEN => decode_read_open(payload).unwrap_or_else(unknown),
            MSG_READ => decode_read(payload).unwrap_or_else(unknown),
            MSG_READ_DONE => decode_two_i32(payload)
                .map(|(stream, error)| Message::ReadDone { stream, error })
                .unwrap_or_else(unknown),
            MSG_READ_CANCEL => i32_body(payload)
                .map(|stream| Message::ReadCancel { stream })
                .unwrap_or_else(unknown),
            MSG_WRITE_OPEN => decode_write_open(payload).unwrap_or_else(unknown),
            MSG_WRITE => decode_write(payload).unwrap_or_else(unknown),
            MSG_WRITE_READY => decode_two_i32(payload)
                .map(|(stream, error)| Message::WriteReady { stream, error })
                .unwrap_or_else(unknown),
            MSG_WRITE_CLOSE => i32_body(payload)
                .map(|stream| Message::WriteClose { stream })
                .unwrap_or_else(unknown),

            MSG_HMUX_STATUS_WAIT => u64_body(payload)
                .map(|since| Message::StatusWait { since })
                .unwrap_or_else(unknown),
            MSG_HMUX_STATUS => decode_status(payload).unwrap_or_else(unknown),

            _ => unknown(),
        }
    }

    /// Re-encode to `(type, payload)`. The inverse of [`Message::decode`] for
    /// every modeled variant (round-trip tested in the codec module).
    pub fn encode(&self) -> (u32, Vec<u8>) {
        use msgtype::*;

        match self {
            Message::IdentifyFlags(v) => (MSG_IDENTIFY_FLAGS, v.to_ne_bytes().to_vec()),
            Message::IdentifyLongFlags(v) => (MSG_IDENTIFY_LONGFLAGS, v.to_ne_bytes().to_vec()),
            Message::IdentifyTerm(s) => (MSG_IDENTIFY_TERM, cstr_bytes(s)),
            Message::IdentifyTtyName(s) => (MSG_IDENTIFY_TTYNAME, cstr_bytes(s)),
            Message::IdentifyCwd(s) => (MSG_IDENTIFY_CWD, cstr_bytes(s)),
            Message::IdentifyEnviron(s) => (MSG_IDENTIFY_ENVIRON, cstr_bytes(s)),
            Message::IdentifyTerminfo(s) => (MSG_IDENTIFY_TERMINFO, cstr_bytes(s)),
            Message::IdentifyFeatures(v) => (MSG_IDENTIFY_FEATURES, v.to_ne_bytes().to_vec()),
            Message::IdentifyStdin => (MSG_IDENTIFY_STDIN, Vec::new()),
            Message::IdentifyStdout => (MSG_IDENTIFY_STDOUT, Vec::new()),
            Message::IdentifyClientPid(v) => (MSG_IDENTIFY_CLIENTPID, v.to_ne_bytes().to_vec()),
            Message::IdentifyDone => (MSG_IDENTIFY_DONE, Vec::new()),

            Message::Command(args) => (MSG_COMMAND, encode_command(args)),
            Message::Detach(None) => (MSG_DETACH, Vec::new()),
            Message::Detach(Some(s)) => (MSG_DETACH, cstr_bytes(s)),
            Message::DetachKill(None) => (MSG_DETACHKILL, Vec::new()),
            Message::DetachKill(Some(s)) => (MSG_DETACHKILL, cstr_bytes(s)),
            Message::Exit(None) => (MSG_EXIT, Vec::new()),
            Message::Exit(Some(code)) => (MSG_EXIT, code.to_ne_bytes().to_vec()),
            Message::Lock(command) => (MSG_LOCK, cstr_bytes(command)),
            Message::Unlock => (MSG_UNLOCK, Vec::new()),
            Message::Suspend => (MSG_SUSPEND, Vec::new()),
            Message::Wakeup => (MSG_WAKEUP, Vec::new()),
            Message::Resize => (MSG_RESIZE, Vec::new()),
            Message::Shutdown => (MSG_SHUTDOWN, Vec::new()),
            Message::Flags(v) => (MSG_FLAGS, v.to_ne_bytes().to_vec()),

            Message::Ready => (MSG_READY, Vec::new()),
            Message::Version => (MSG_VERSION, Vec::new()),
            Message::Exited => (MSG_EXITED, Vec::new()),
            Message::Exiting => (MSG_EXITING, Vec::new()),

            Message::ReadOpen { stream, fd, path } => {
                (MSG_READ_OPEN, encode_two_i32_path(*stream, *fd, path))
            }
            Message::Read { stream, data } => (MSG_READ, encode_write(*stream, data)),
            Message::ReadDone { stream, error } => (MSG_READ_DONE, two_i32_bytes(*stream, *error)),
            Message::ReadCancel { stream } => (MSG_READ_CANCEL, stream.to_ne_bytes().to_vec()),
            Message::WriteOpen {
                stream,
                fd,
                flags,
                path,
            } => (
                MSG_WRITE_OPEN,
                encode_write_open(*stream, *fd, *flags, path),
            ),
            Message::Write { stream, data } => (MSG_WRITE, encode_write(*stream, data)),
            Message::WriteReady { stream, error } => {
                (MSG_WRITE_READY, two_i32_bytes(*stream, *error))
            }
            Message::WriteClose { stream } => (MSG_WRITE_CLOSE, stream.to_ne_bytes().to_vec()),

            Message::StatusWait { since } => (MSG_HMUX_STATUS_WAIT, since.to_ne_bytes().to_vec()),
            Message::Status { revision, body } => (MSG_HMUX_STATUS, encode_status(*revision, body)),

            Message::Unknown { type_, payload } => (*type_, payload.clone()),
        }
    }

    /// Short human-readable name for introspection logging.
    pub fn name(&self) -> &'static str {
        match self {
            Message::IdentifyFlags(_) => "IdentifyFlags",
            Message::IdentifyLongFlags(_) => "IdentifyLongFlags",
            Message::IdentifyTerm(_) => "IdentifyTerm",
            Message::IdentifyTtyName(_) => "IdentifyTtyName",
            Message::IdentifyCwd(_) => "IdentifyCwd",
            Message::IdentifyEnviron(_) => "IdentifyEnviron",
            Message::IdentifyTerminfo(_) => "IdentifyTerminfo",
            Message::IdentifyFeatures(_) => "IdentifyFeatures",
            Message::IdentifyStdin => "IdentifyStdin",
            Message::IdentifyStdout => "IdentifyStdout",
            Message::IdentifyClientPid(_) => "IdentifyClientPid",
            Message::IdentifyDone => "IdentifyDone",
            Message::Command(_) => "Command",
            Message::Detach(_) => "Detach",
            Message::DetachKill(_) => "DetachKill",
            Message::Exit(_) => "Exit",
            Message::Lock(_) => "Lock",
            Message::Unlock => "Unlock",
            Message::Suspend => "Suspend",
            Message::Wakeup => "Wakeup",
            Message::Resize => "Resize",
            Message::Shutdown => "Shutdown",
            Message::Flags(_) => "Flags",
            Message::Ready => "Ready",
            Message::Version => "Version",
            Message::Exited => "Exited",
            Message::Exiting => "Exiting",
            Message::ReadOpen { .. } => "ReadOpen",
            Message::Read { .. } => "Read",
            Message::ReadDone { .. } => "ReadDone",
            Message::ReadCancel { .. } => "ReadCancel",
            Message::WriteOpen { .. } => "WriteOpen",
            Message::Write { .. } => "Write",
            Message::WriteReady { .. } => "WriteReady",
            Message::WriteClose { .. } => "WriteClose",
            Message::StatusWait { .. } => "StatusWait",
            Message::Status { .. } => "Status",
            Message::Unknown { .. } => "Unknown",
        }
    }
}

// ---- payload codecs ----------------------------------------------------

fn i32_body(p: &[u8]) -> Option<i32> {
    (p.len() == 4).then(|| i32::from_ne_bytes(p.try_into().unwrap()))
}

fn i64_body(p: &[u8]) -> Option<i64> {
    (p.len() == 8).then(|| i64::from_ne_bytes(p.try_into().unwrap()))
}

fn u64_body(p: &[u8]) -> Option<u64> {
    (p.len() == 8).then(|| u64::from_ne_bytes(p.try_into().unwrap()))
}

/// `MSG_HMUX_STATUS` payload: a native-endian `u64` revision then the raw record
/// body (PROTOCOL.md §status). The body may be empty, so the minimum length is 8.
fn decode_status(p: &[u8]) -> Option<Message> {
    if p.len() < 8 {
        return None;
    }
    let revision = u64::from_ne_bytes(p[0..8].try_into().unwrap());
    Some(Message::Status {
        revision,
        body: p[8..].to_vec(),
    })
}

fn encode_status(revision: u64, body: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(8 + body.len());
    v.extend_from_slice(&revision.to_ne_bytes());
    v.extend_from_slice(body);
    v
}

/// A C string payload: bytes followed by exactly one trailing NUL. Returns the
/// string without the NUL. Rejects non-UTF-8 or a missing terminator so those
/// fall back to `Unknown` and round-trip byte-for-byte.
fn cstr_body(p: &[u8]) -> Option<String> {
    let (&last, rest) = p.split_last()?;
    if last != 0 {
        return None;
    }
    // Interior NULs would not survive our `cstr_bytes` re-encode.
    if rest.contains(&0) {
        return None;
    }
    std::str::from_utf8(rest).ok().map(str::to_owned)
}

fn cstr_bytes(s: &str) -> Vec<u8> {
    let mut v = Vec::with_capacity(s.len() + 1);
    v.extend_from_slice(s.as_bytes());
    v.push(0);
    v
}

fn decode_two_i32(p: &[u8]) -> Option<(i32, i32)> {
    if p.len() != 8 {
        return None;
    }
    let a = i32::from_ne_bytes(p[0..4].try_into().unwrap());
    let b = i32::from_ne_bytes(p[4..8].try_into().unwrap());
    Some((a, b))
}

fn two_i32_bytes(a: i32, b: i32) -> Vec<u8> {
    let mut v = Vec::with_capacity(8);
    v.extend_from_slice(&a.to_ne_bytes());
    v.extend_from_slice(&b.to_ne_bytes());
    v
}

/// `msg_command`: `int argc` then `argc` NUL-terminated args (cmd_pack_argv).
fn decode_command(p: &[u8]) -> Option<Message> {
    if p.len() < 4 {
        return None;
    }
    let argc = i32::from_ne_bytes(p[0..4].try_into().unwrap());
    if !(0..=1000).contains(&argc) {
        return None;
    }
    let argc = argc as usize;
    let rest = &p[4..];
    if argc == 0 {
        return rest.is_empty().then(|| Message::Command(Vec::new()));
    }
    // Packed args are `argc` NUL-terminated strings, so `rest` ends in NUL and
    // splitting on NUL yields exactly `argc` pieces plus a trailing empty one.
    if rest.last() != Some(&0) {
        return None;
    }
    let mut pieces = rest.split(|&b| b == 0);
    let mut args = Vec::with_capacity(argc);
    for _ in 0..argc {
        let chunk = pieces.next()?;
        args.push(std::str::from_utf8(chunk).ok()?.to_owned());
    }
    // Exactly one trailing empty piece (after the final NUL) must remain.
    match (pieces.next(), pieces.next()) {
        (Some([]), None) => Some(Message::Command(args)),
        _ => None,
    }
}

fn encode_command(args: &[String]) -> Vec<u8> {
    let mut v = Vec::new();
    v.extend_from_slice(&(args.len() as i32).to_ne_bytes());
    for a in args {
        v.extend_from_slice(a.as_bytes());
        v.push(0);
    }
    v
}

/// `msg_read_open { int stream; int fd; }` then optional raw path bytes.
fn decode_read_open(p: &[u8]) -> Option<Message> {
    if p.len() < 8 {
        return None;
    }
    let stream = i32::from_ne_bytes(p[0..4].try_into().unwrap());
    let fd = i32::from_ne_bytes(p[4..8].try_into().unwrap());
    Some(Message::ReadOpen {
        stream,
        fd,
        path: p[8..].to_vec(),
    })
}

fn decode_read(p: &[u8]) -> Option<Message> {
    if p.len() < 4 {
        return None;
    }
    Some(Message::Read {
        stream: i32::from_ne_bytes(p[0..4].try_into().unwrap()),
        data: p[4..].to_vec(),
    })
}

fn encode_two_i32_path(stream: i32, fd: i32, path: &[u8]) -> Vec<u8> {
    let mut v = two_i32_bytes(stream, fd);
    v.extend_from_slice(path);
    v
}

/// `msg_write_open { int stream; int fd; int flags; }` then optional path bytes.
fn decode_write_open(p: &[u8]) -> Option<Message> {
    if p.len() < 12 {
        return None;
    }
    let stream = i32::from_ne_bytes(p[0..4].try_into().unwrap());
    let fd = i32::from_ne_bytes(p[4..8].try_into().unwrap());
    let flags = i32::from_ne_bytes(p[8..12].try_into().unwrap());
    Some(Message::WriteOpen {
        stream,
        fd,
        flags,
        path: p[12..].to_vec(),
    })
}

fn encode_write_open(stream: i32, fd: i32, flags: i32, path: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(12 + path.len());
    v.extend_from_slice(&stream.to_ne_bytes());
    v.extend_from_slice(&fd.to_ne_bytes());
    v.extend_from_slice(&flags.to_ne_bytes());
    v.extend_from_slice(path);
    v
}

/// `msg_write_data { int stream; }` followed by raw data.
fn decode_write(p: &[u8]) -> Option<Message> {
    if p.len() < 4 {
        return None;
    }
    let stream = i32::from_ne_bytes(p[0..4].try_into().unwrap());
    Some(Message::Write {
        stream,
        data: p[4..].to_vec(),
    })
}

fn encode_write(stream: i32, data: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(4 + data.len());
    v.extend_from_slice(&stream.to_ne_bytes());
    v.extend_from_slice(data);
    v
}
