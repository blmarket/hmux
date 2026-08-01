//! Threaded driver for shared nonblocking pane I/O.

use std::io;
use std::os::fd::AsRawFd;
use std::path::Path;
use std::thread::{self, JoinHandle};

use crate::server::pane::{Pane, PaneIo, PaneIoMode};

impl Pane {
    /// Spawn `argv` on a fresh pty and drain its output into the screen.
    pub fn spawn(argv: &[&str], cols: u16, rows: u16) -> io::Result<Pane> {
        Self::spawn_in(argv, None, cols, rows)
    }

    /// Spawn `argv` on a fresh pty in an optional child working directory.
    pub(crate) fn spawn_in(
        argv: &[&str],
        cwd: Option<&Path>,
        cols: u16,
        rows: u16,
    ) -> io::Result<Pane> {
        Self::spawn_in_mode(argv, cwd, cols, rows, PaneIoMode::Threaded(spawn_reader))
    }
}

/// Spawn the compatibility thread that waits around the same nonblocking pane
/// state used by the central event loop.
pub(crate) fn spawn_reader(mut pane_io: PaneIo) -> JoinHandle<()> {
    thread::spawn(move || loop {
        let mut wait = libc::pollfd {
            fd: pane_io.as_fd().as_raw_fd(),
            events: libc::POLLIN
                | if pane_io.wants_write() {
                    libc::POLLOUT
                } else {
                    0
                },
            revents: 0,
        };
        let r = unsafe { libc::poll(&mut wait, 1, -1) };
        if r < 0 {
            let err = io::Error::last_os_error();
            if err.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            break;
        }
        if wait.revents & libc::POLLOUT != 0 {
            pane_io.drive_writable();
        }
        if wait.revents & (libc::POLLIN | libc::POLLHUP | libc::POLLERR) == 0 {
            continue;
        }
        match pane_io.drive_readable() {
            Ok(result) if result.closed => break,
            Ok(result) if result.continuation => continue,
            Ok(_) => {}
            Err(_) => break,
        }
    })
}
