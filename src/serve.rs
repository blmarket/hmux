//! The hmux listener and per-connection pairing loop.
//!
//! For each accepted client, hmux opens a fresh native server connection and
//! runs two threads: client→server and server→client. Each thread reads a
//! frame, taps it for introspection, and re-encodes it to the other side —
//! forwarding any `SCM_RIGHTS` fd. A broken half tears down only that pairing.

use std::io;
use std::os::fd::AsRawFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use tracing::{info, warn};

use crate::tmux::codec::split_stream;
use crate::tmux::introspect::{Direction, LoggingReader, LoggingWriter};
use crate::tmux::traits::{FrameReader, FrameWriter, TmuxServer};

/// Bind `listen_path` and serve clients from `server`, forever.
pub fn run<T>(listen_path: &Path, server: T) -> io::Result<()>
where
    T: TmuxServer + 'static,
    T::Reader: AsRawFd + 'static,
    T::Writer: 'static,
{
    run_until(listen_path, server, |_| false)
}

/// Bind `listen_path` and serve clients until `loop_done` returns true.
///
/// Keeping this callback separate from [`TmuxServer`] mirrors tmux's
/// `proc_loop(server_proc, server_loop)`: the protocol abstraction opens client
/// connections, while the concrete server runtime owns its lifecycle policy.
pub fn run_until<T, F>(listen_path: &Path, server: T, loop_done: F) -> io::Result<()>
where
    T: TmuxServer + 'static,
    T::Reader: AsRawFd + 'static,
    T::Writer: 'static,
    F: Fn(&T) -> bool,
{
    // Remove a stale socket, but never unlink a live tmux/hmux listener. This is
    // especially important for the discoverable default path: unlinking a live
    // socket would strand the existing server and let two servers claim the
    // same pathname at different times.
    if listen_path.exists() && UnixStream::connect(listen_path).is_err() {
        let _ = std::fs::remove_file(listen_path);
    }
    let listener = UnixListener::bind(listen_path)?;
    listener.set_nonblocking(true)?;
    info!(socket = %listen_path.display(), "hmux listening");

    let server = Arc::new(server);
    let mut workers = Vec::new();
    loop {
        if loop_done(server.as_ref()) {
            break;
        }

        match listener.accept() {
            Ok((stream, _)) => {
                // The listener is non-blocking so the accept loop can poll for
                // shutdown, but the per-connection pumps rely on blocking reads.
                // On Linux an accepted socket is always blocking; on macOS/BSD it
                // inherits the listener's O_NONBLOCK, so a blocking `recvmsg` in
                // `pump` would return EAGAIN and tear the pairing down. Force
                // blocking mode so both platforms behave the same.
                if let Err(e) = stream.set_nonblocking(false) {
                    warn!(error = %e, "failed to set client socket blocking");
                    continue;
                }
                let server = Arc::clone(&server);
                workers.push(thread::spawn(move || {
                    if let Err(e) = handle_client(stream, server.as_ref()) {
                        warn!(error = %e, "client pairing ended");
                    }
                }));
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => warn!(error = %e, "accept failed"),
        }

        let mut index = 0;
        while index < workers.len() {
            if workers[index].is_finished() {
                let worker = workers.swap_remove(index);
                let _ = worker.join();
            } else {
                index += 1;
            }
        }
    }

    // Do not let process shutdown cut off the final EXIT/EXITED handshake.
    drop(listener);
    for worker in workers {
        let _ = worker.join();
    }
    Ok(())
}

/// Pair one accepted client with a fresh server connection and pump frames.
fn handle_client<T: TmuxServer>(client: UnixStream, server: &T) -> io::Result<()>
where
    T::Reader: AsRawFd + 'static,
    T::Writer: 'static,
{
    let (client_reader, client_writer) = split_stream(client)?;
    let (server_reader, server_writer) = server.connect()?;

    // Raw fd of the server socket, captured before the reader is wrapped/moved,
    // so the c2s pump can force it down when the client goes away (see below).
    let server_fd = server_reader.as_raw_fd();

    // Wrap each half with the introspection tap.
    let mut c2s_reader = LoggingReader::new(client_reader, Direction::ClientToServer);
    let mut c2s_writer = server_writer;
    let mut s2c_reader = LoggingReader::new(server_reader, Direction::ServerToClient);
    let mut s2c_writer = LoggingWriter::new(client_writer, Direction::ServerToClient);

    // client -> server on this thread's child; server -> client here.
    //
    // When the client -> server direction ends, the client's send side is gone:
    // it detached uncleanly, was killed, or the socket dropped. `split_stream`
    // gave each half an independent dup of the same socket, so the c2s pump
    // merely dropping its writer does *not* close the socket — the s2c half keeps
    // it open. On the interactive attach path the s2c pump then blocks forever
    // reading the server socket (the attach loop renders straight to the tty and
    // rarely sends control frames), and the attach loop itself spins on its
    // control read, never observing the disconnect and never restoring the
    // client's terminal out of raw mode. An explicit `shutdown` of the server
    // socket propagates the EOF to both the s2c pump and the connection handler,
    // so the attach loop breaks and puts the terminal back. It runs only once the
    // client is actually gone, so a clean detach/exit handshake (client alive
    // until it completes) is never cut short.
    let c2s = thread::spawn(move || {
        let result = pump(&mut c2s_reader, &mut c2s_writer);
        // SAFETY: `server_fd` is still owned by the s2c reader half; `shutdown`
        // only flips socket state and is safe to race with that pump's read,
        // which then observes EOF.
        unsafe {
            libc::shutdown(server_fd, libc::SHUT_RDWR);
        }
        result
    });
    let s2c_res = pump(&mut s2c_reader, &mut s2c_writer);

    let c2s_res = c2s.join().unwrap_or(Ok(()));
    // Surface the first non-EOF error, if any.
    first_real_error(c2s_res, s2c_res)
}

/// Copy frames from `reader` to `writer` until EOF or error.
fn pump<R: FrameReader, W: FrameWriter>(reader: &mut R, writer: &mut W) -> io::Result<()> {
    loop {
        let frame = match reader.recv() {
            Ok(f) => f,
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(()),
            Err(e) => return Err(e),
        };
        writer.send(frame)?;
    }
}

fn first_real_error(a: io::Result<()>, b: io::Result<()>) -> io::Result<()> {
    a?;
    b
}
