//! `hmux` binary: bind a socket and serve `tmux attach`.
//!
//! ```text
//! hmux [--foreground] [-S <sock>]
//! ```

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;
use tracing::{error, info};

use hmux::integration::AgentObserver;
use hmux::serve;
use hmux::tmux::Server;

#[derive(Parser, Debug)]
#[command(name = "hmux", about = "native tmux imsg wire-protocol server")]
struct Args {
    /// Stay attached to the terminal instead of running as a daemon.
    #[arg(long)]
    foreground: bool,

    /// Path of the socket hmux binds. By default this is tmux's discoverable
    /// socket (`$TMUX_TMPDIR/tmux-$UID/default`, or `/tmp/tmux-$UID/default`).
    #[arg(short = 'S', long = "socket")]
    socket: Option<PathBuf>,
}

fn main() -> ExitCode {
    let mut args = Args::parse();
    let socket_is_default = args.socket.is_none();
    let socket = args.socket.get_or_insert_with(default_socket_path);
    if socket_is_default {
        if let Err(e) = prepare_default_socket_dir(socket) {
            eprintln!("hmux: failed to prepare default socket directory: {e}");
            return ExitCode::FAILURE;
        }
    }
    if !args.foreground {
        match daemonize() {
            Ok(DaemonOutcome::Parent) => return ExitCode::SUCCESS,
            Ok(DaemonOutcome::Child) => {}
            Err(e) => {
                eprintln!("hmux: failed to daemonize: {e}");
                return ExitCode::FAILURE;
            }
        }
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let result = run_server(args);

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            error!(error = %e, "hmux exited with error");
            ExitCode::FAILURE
        }
    }
}

enum DaemonOutcome {
    Parent,
    Child,
}

/// Detach with the traditional double-fork sequence. This runs before hmux
/// creates any threads or pane ptys. The daemon preserves
/// the caller's working directory because tmux sessions inherit it.
fn daemonize() -> std::io::Result<DaemonOutcome> {
    use std::fs::OpenOptions;
    use std::os::fd::AsRawFd;

    let devnull = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/null")?;
    // SAFETY: daemonize is called before tracing, observers, handlers, or pane
    // workers are initialized, so this is still a single-threaded process.
    let first = unsafe { libc::fork() };
    if first < 0 {
        return Err(std::io::Error::last_os_error());
    }
    if first > 0 {
        let mut status = 0;
        if unsafe { libc::waitpid(first, &mut status, 0) } < 0 {
            return Err(std::io::Error::last_os_error());
        }
        if !libc::WIFEXITED(status) || libc::WEXITSTATUS(status) != 0 {
            return Err(std::io::Error::other("daemon child failed to detach"));
        }
        return Ok(DaemonOutcome::Parent);
    }

    if unsafe { libc::setsid() } < 0 {
        unsafe { libc::_exit(1) };
    }
    let second = unsafe { libc::fork() };
    if second < 0 {
        unsafe { libc::_exit(1) };
    }
    if second > 0 {
        unsafe { libc::_exit(0) };
    }

    for fd in [libc::STDIN_FILENO, libc::STDOUT_FILENO, libc::STDERR_FILENO] {
        if unsafe { libc::dup2(devnull.as_raw_fd(), fd) } < 0 {
            unsafe { libc::_exit(1) };
        }
    }
    Ok(DaemonOutcome::Child)
}

/// Serve the tmux-compatible server through the event-loop protocol engine.
fn run_server(args: Args) -> hmux::Result<()> {
    let listen_socket = args.socket.as_deref().expect("socket resolved in main");
    info!("engine: event-loop protocol");
    // Start as a server only. The first untargeted `tmux attach` lazily creates
    // session 0, so launching hmux does not speculatively spawn a shell or
    // commit the first session to the 80x24 fallback geometry.
    let server = Server::new()?;
    server.set_socket_path(listen_socket)?;
    // The observer publishes into the server's status hub; format rendering
    // reads the same hub for `#{pane_agent*}` and control subscriptions. The
    // loop ticks it, so it never holds server state while the loop wants it.
    let observer = AgentObserver::new(server.status_hub());
    let result = serve::run_event_loop(listen_socket, server, observer);
    // The socket pathname is deliberately left behind, as tmux leaves its own:
    // it only ever unlinks a stale path when binding. A client that finds the
    // residue then reports "no server running on <path>" (ECONNREFUSED) rather
    // than a connect error for a missing file.
    result?;
    Ok(())
}

/// Return the same default socket pathname a tmux client uses when neither
/// `-L` nor `-S` is supplied.
fn default_socket_path() -> PathBuf {
    let base = std::env::var_os("TMUX_TMPDIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"));
    base.join(format!("tmux-{}", unsafe { libc::geteuid() }))
        .join("default")
}

/// tmux creates its per-user socket directory with private permissions. Do the
/// same so a plain `tmux attach` can safely discover hmux's default listener.
fn prepare_default_socket_dir(socket: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let directory = socket
        .parent()
        .ok_or_else(|| std::io::Error::other("default socket has no parent directory"))?;
    std::fs::create_dir_all(directory)?;
    std::fs::set_permissions(directory, std::fs::Permissions::from_mode(0o700))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foreground_flag_parses() {
        let args = Args::try_parse_from(["hmux", "--foreground"]).unwrap();

        assert!(args.foreground);
    }
}
