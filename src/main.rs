//! `hmux` binary: bind a socket and serve `tmux attach`.
//!
//! ```text
//! hmux [--foreground] [-S <sock>] [--engine native|event-loop]
//! ```

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, ValueEnum};
use tracing::{error, info};

use hmux::integration::AgentObserver;
use hmux::serve;
use hmux::tmux::NativeServer;

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum Engine {
    /// Native libghostty-vt server (owns ptys + screen state).
    Native,
    /// Native server behind the readiness-driven forwarding adapter.
    EventLoop,
}

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

    /// Connection forwarding implementation.
    #[arg(long, value_enum, default_value_t = Engine::Native)]
    engine: Engine,
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

/// Serve the libghostty-vt server with the selected connection adapter.
fn run_server(args: Args) -> hmux::Result<()> {
    let listen_socket = args.socket.as_deref().expect("socket resolved in main");
    install_signal_teardown(listen_socket.to_path_buf());

    match args.engine {
        Engine::Native => info!("engine: native (libghostty-vt)"),
        Engine::EventLoop => info!("engine: event-loop protocol (libghostty-vt)"),
    }
    // Start as a server only. The first untargeted `tmux attach` lazily creates
    // session 0, so launching hmux does not speculatively spawn a shell or
    // commit the first session to the 80x24 fallback geometry.
    let server = NativeServer::new()?;
    // The observer publishes into the server's status hub; connection handlers
    // read the same hub for `#{pane_agent*}` and control subscriptions.
    let _agent_observer = AgentObserver::start(server.clone(), server.status_hub())?;
    let result = match args.engine {
        Engine::Native => {
            serve::run_until(listen_socket, server, |server| server.shutdown_requested())
        }
        Engine::EventLoop => serve::run_event_loop(listen_socket, server),
    };
    // Unix socket pathnames outlive a closed listener. Remove ours on a normal
    // `exit-empty` shutdown just as the signal teardown path does.
    let _ = std::fs::remove_file(listen_socket);
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

/// Block `SIGINT`/`SIGTERM` process-wide and spawn a thread that waits for one,
/// removes the listener socket and exits.
///
/// Must be called before any worker threads are spawned so they inherit the
/// blocked mask and only this dedicated thread receives the signal.
fn install_signal_teardown(listen_socket: PathBuf) {
    unsafe {
        let mut set: libc::sigset_t = std::mem::zeroed();
        libc::sigemptyset(&mut set);
        libc::sigaddset(&mut set, libc::SIGINT);
        libc::sigaddset(&mut set, libc::SIGTERM);
        libc::pthread_sigmask(libc::SIG_BLOCK, &set, std::ptr::null_mut());

        std::thread::spawn(move || {
            let mut sig: libc::c_int = 0;
            // Wait for the first blocked signal.
            libc::sigwait(&set, &mut sig);
            info!(signal = sig, "shutting down");
            let _ = std::fs::remove_file(&listen_socket);
            std::process::exit(0);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_engine_remains_the_default() {
        let args = Args::try_parse_from(["hmux", "--foreground"]).unwrap();

        assert_eq!(args.engine, Engine::Native);
    }

    #[test]
    fn event_loop_engine_is_selected_explicitly() {
        let args =
            Args::try_parse_from(["hmux", "--foreground", "--engine", "event-loop"]).unwrap();

        assert_eq!(args.engine, Engine::EventLoop);
    }
}
