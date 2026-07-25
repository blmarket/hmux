//! Spawn and own a private backing tmux server.
//!
//! The conformance harness launches `tmux -f/dev/null -S <private.sock> start-server \; set -g
//! exit-empty off`, waits for the socket to appear, and owns the lifecycle: on
//! drop it runs `kill-server` on that socket so no stray tmux is left behind. A
//! private socket per instance keeps backing state isolated and reproducible
//! (see design.md decision #3).
//!
//! Two details matter for behavioural fidelity against the tmux regress corpus:
//!
//! - **Same binary as the client.** The backing must be spawned with the *same*
//!   tmux the client attaches with; a PATH-resolved `tmux` can be a different
//!   version, and a version-skewed client↔server pair diverges on error
//!   wording, option names, etc. Callers pass the client's tmux path.
//! - **A clean server.** The base constructor starts empty; production hmux uses
//!   the default-session constructor so a client can attach immediately.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Stdio;
use std::time::{Duration, Instant};

use crate::error::{Error, Result};

/// A running, owned backing tmux server on a private socket.
pub struct Backing {
    /// The tmux binary that runs the backing (kept for `kill-server` on drop).
    tmux_bin: PathBuf,
    socket: PathBuf,
    /// Kept so the temp dir is removed when we drop.
    _dir: TempDir,
    /// Server PID used only as a fallback when a cleanup tmux client cannot be
    /// executed. The socket remains the primary, graceful shutdown path.
    server_pid: Option<u32>,
}

impl Backing {
    /// Spawn a fresh private tmux server on a temp socket, using `tmux_bin`
    /// (which must be the same tmux the client attaches with — see module docs)
    /// and loading `config` at server start (`/dev/null` for none).
    ///
    /// `config` matters because tmux loads its config file in the *server*
    /// process at start, and the path is a client-global flag (`-f`) that never
    /// crosses the imsg wire, so the fixture must load it when starting tmux.
    pub fn spawn(tmux_bin: impl Into<PathBuf>, config: impl Into<PathBuf>) -> Result<Backing> {
        let dir = TempDir::new()?;
        let socket = dir.path().join("backing.sock");
        Self::spawn_at(tmux_bin.into(), config.into(), &socket, dir)
    }

    /// Like [`spawn`](Self::spawn) but also creates the default session `0` (via
    /// `new-session -d`) on the fresh backing.
    ///
    /// The conformance/behavior harnesses use this so a client can attach
    /// immediately and both targets begin with the same
    /// session `0` fixture. Uses the same `tmux_bin` for the bootstrap as for
    /// the server, so the fixture can't drift onto a different tmux version.
    pub fn spawn_with_default_session(
        tmux_bin: impl Into<PathBuf>,
        config: impl Into<PathBuf>,
    ) -> Result<Backing> {
        let backing = Self::spawn(tmux_bin, config)?;
        let status = Command::new(&backing.tmux_bin)
            .arg("-S")
            .arg(&backing.socket)
            .arg("new-session")
            .arg("-d")
            .status()
            .map_err(|e| Error::backing(format!("failed to exec tmux: {e}")))?;
        if !status.success() {
            return Err(Error::backing(format!(
                "backing bootstrap new-session exited with {status}"
            )));
        }
        let status = Command::new(&backing.tmux_bin)
            .arg("-S")
            .arg(&backing.socket)
            .args(["set-option", "-g", "exit-empty", "on"])
            .status()
            .map_err(|e| Error::backing(format!("failed to exec tmux: {e}")))?;
        if !status.success() {
            return Err(Error::backing(format!(
                "backing exit-empty setup exited with {status}"
            )));
        }
        Ok(backing)
    }

    fn spawn_at(
        tmux_bin: PathBuf,
        config: PathBuf,
        socket: &Path,
        dir: TempDir,
    ) -> Result<Backing> {
        // Start the server with the client's config (see module docs), then
        // `set -g exit-empty off` so an *empty* server (config created no
        // session) stays alive until the client's first command instead of
        // exiting immediately. When the config does create sessions this is a
        // harmless no-op relative to what they'd expect.
        let status = Command::new(&tmux_bin)
            .arg("-f")
            .arg(&config)
            .arg("-S")
            .arg(socket)
            .arg("start-server")
            .arg(";")
            .arg("set-option")
            .arg("-g")
            .arg("exit-empty")
            .arg("off")
            .status()
            .map_err(|e| Error::backing(format!("failed to exec tmux: {e}")))?;
        if !status.success() {
            return Err(Error::backing(format!(
                "tmux start-server exited with {status}"
            )));
        }

        let mut backing = Backing {
            tmux_bin,
            socket: socket.to_path_buf(),
            _dir: dir,
            server_pid: None,
        };
        backing.wait_for_socket()?;
        backing.server_pid = Some(backing.query_server_pid()?);
        Ok(backing)
    }

    /// Path to the backing socket clients should connect to.
    pub fn socket_path(&self) -> &Path {
        &self.socket
    }

    /// Poll until the socket file exists (tmux binds it asynchronously).
    fn wait_for_socket(&self) -> Result<()> {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if self.socket.exists() {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        Err(Error::backing("backing tmux socket never appeared"))
    }

    fn query_server_pid(&self) -> Result<u32> {
        let output = Command::new(&self.tmux_bin)
            .arg("-S")
            .arg(&self.socket)
            .arg("display-message")
            .arg("-p")
            .arg("#{pid}")
            .output()
            .map_err(|e| Error::backing(format!("failed to query backing tmux pid: {e}")))?;
        if !output.status.success() {
            return Err(Error::backing(format!(
                "querying backing tmux pid exited with {}",
                output.status
            )));
        }
        let pid = std::str::from_utf8(&output.stdout)
            .ok()
            .and_then(|value| value.trim().parse().ok())
            .ok_or_else(|| Error::backing("backing tmux returned an invalid server pid"))?;
        Ok(pid)
    }
}

impl Drop for Backing {
    fn drop(&mut self) {
        let _ = try_kill_server(&self.tmux_bin, &self.socket);
        if let Some(pid) = self.server_pid {
            ensure_owned_server_stopped(pid, &self.socket);
        }
    }
}

fn try_kill_server(tmux_bin: &Path, socket: &Path) -> bool {
    Command::new(tmux_bin)
        .arg("-S")
        .arg(socket)
        .arg("kill-server")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// Kill the recorded process only when Linux still identifies it as the tmux
/// server for our private socket. This prevents a stale/reused PID from ever
/// targeting an unrelated process.
#[cfg(target_os = "linux")]
fn ensure_owned_server_stopped(pid: u32, socket: &Path) {
    // Give the graceful command a short opportunity to complete. A tmux client
    // can return success while the server remains alive briefly.
    let graceful_deadline = Instant::now() + Duration::from_millis(100);
    while Instant::now() < graceful_deadline {
        if !std::path::Path::new(&format!("/proc/{pid}")).exists() {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }

    let Ok(cmdline) = std::fs::read(format!("/proc/{pid}/cmdline")) else {
        return;
    };
    let socket = socket.as_os_str().as_encoded_bytes();
    if !cmdline.split(|byte| *byte == 0).any(|arg| arg == socket) {
        return;
    }
    unsafe {
        libc::kill(pid as libc::pid_t, libc::SIGTERM);
    }
    let deadline = Instant::now() + Duration::from_millis(100);
    while Instant::now() < deadline {
        if !std::path::Path::new(&format!("/proc/{pid}")).exists() {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    // A server stuck after SIGTERM must not retain its PTYs indefinitely.
    unsafe {
        libc::kill(pid as libc::pid_t, libc::SIGKILL);
    }
    let killed_deadline = Instant::now() + Duration::from_secs(1);
    while Instant::now() < killed_deadline {
        if !std::path::Path::new(&format!("/proc/{pid}")).exists() {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(not(target_os = "linux"))]
fn ensure_owned_server_stopped(_pid: u32, _socket: &Path) {
    // Without a reliable process-identity check, retaining the socket is safer
    // than risking termination of a process that reused the recorded PID.
}

/// A minimal owned temporary directory that is removed on drop. Avoids pulling
/// in a `tempfile` dependency for the one place we need it.
struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new() -> Result<TempDir> {
        let base = std::env::temp_dir();
        // Unique-enough per process + monotonic counter; sockets are short-lived
        // and torn down, so collisions are not a real concern.
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = base.join(format!("hmux-{}-{}", std::process::id(), n));
        std::fs::create_dir_all(&path)?;
        Ok(TempDir { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
