//! Linux implementation of the operating-system boundary.

use std::ffi::OsString;
use std::fs;
use std::io;
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd, RawFd};
use std::path::PathBuf;
use std::ptr;

use super::{ForkOutcome, OutputWakeup, Platform, ProcessInfo};

/// The Linux platform implementation selected by the native server.
pub(crate) struct Linux;

/// A non-blocking `eventfd` used as a coalescing readiness notification.
pub(crate) struct EventFd(OwnedFd);

impl AsFd for EventFd {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl OutputWakeup for EventFd {
    fn wake(&self) -> io::Result<()> {
        let value = 1u64;
        loop {
            let written = unsafe {
                libc::write(
                    self.0.as_raw_fd(),
                    (&value as *const u64).cast(),
                    std::mem::size_of::<u64>(),
                )
            };
            if written == std::mem::size_of::<u64>() as isize {
                return Ok(());
            }
            if written >= 0 {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "short write to eventfd",
                ));
            }
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            // A saturated eventfd is already readable, which satisfies the
            // coalescing notification contract.
            if error.kind() == io::ErrorKind::WouldBlock {
                return Ok(());
            }
            return Err(error);
        }
    }

    fn clear(&self) -> io::Result<()> {
        let mut value = 0u64;
        loop {
            let read = unsafe {
                libc::read(
                    self.0.as_raw_fd(),
                    (&mut value as *mut u64).cast(),
                    std::mem::size_of::<u64>(),
                )
            };
            if read == std::mem::size_of::<u64>() as isize {
                return Ok(());
            }
            if read >= 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "short read from eventfd",
                ));
            }
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            // Clearing an already-clear coalescing notification is harmless.
            if error.kind() == io::ErrorKind::WouldBlock {
                return Ok(());
            }
            return Err(error);
        }
    }
}

impl Platform for Linux {
    type OutputWakeup = EventFd;

    fn new_output_wakeup() -> io::Result<Self::OutputWakeup> {
        let fd = unsafe { libc::eventfd(1, libc::EFD_CLOEXEC | libc::EFD_NONBLOCK) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(EventFd(unsafe { OwnedFd::from_raw_fd(fd) }))
    }

    unsafe fn fork_pty(size: libc::winsize) -> io::Result<ForkOutcome> {
        let mut master = -1;
        let pid = unsafe { libc::forkpty(&mut master, ptr::null_mut(), ptr::null(), &size) };
        if pid < 0 {
            return Err(io::Error::last_os_error());
        }
        if pid == 0 {
            return Ok(ForkOutcome::Child);
        }
        Ok(ForkOutcome::Parent {
            pid,
            master: unsafe { OwnedFd::from_raw_fd(master) },
        })
    }

    unsafe fn close_fds_from(lowest: RawFd) {
        debug_assert!(lowest >= 0);
        unsafe {
            libc::close_range(lowest as libc::c_uint, libc::c_uint::MAX, 0);
        }
    }

    fn pane_cwd(pty: BorrowedFd<'_>) -> Option<PathBuf> {
        let foreground_pgrp = unsafe { libc::tcgetpgrp(pty.as_raw_fd()) };
        (foreground_pgrp > 0)
            .then(|| PathBuf::from(format!("/proc/{foreground_pgrp}/cwd")))
            .and_then(|path| fs::read_link(path).ok())
    }

    fn process_open_files(pid: u32) -> Vec<PathBuf> {
        let Ok(entries) = fs::read_dir(format!("/proc/{pid}/fd")) else {
            return Vec::new();
        };
        entries
            .flatten()
            .filter_map(|entry| fs::read_link(entry.path()).ok())
            .collect()
    }

    fn process_table() -> Option<Vec<ProcessInfo>> {
        let entries = fs::read_dir("/proc").ok()?;
        let mut table = Vec::new();
        for entry in entries.flatten() {
            let Some(pid) = entry
                .file_name()
                .to_str()
                .and_then(|name| name.parse::<u32>().ok())
            else {
                continue;
            };
            if let Some(ppid) = read_ppid(pid) {
                table.push(ProcessInfo { pid, ppid });
            }
        }
        Some(table)
    }

    fn process_programs(pid: u32) -> Vec<OsString> {
        let mut programs = Vec::new();

        if let Ok(comm) = fs::read_to_string(format!("/proc/{pid}/comm")) {
            let comm = comm.trim();
            if !comm.is_empty() {
                programs.push(OsString::from(comm));
            }
        }

        if let Some(program) = Self::process_arguments(pid).first() {
            programs.push(program.clone());
        }

        programs
    }

    fn process_arguments(pid: u32) -> Vec<OsString> {
        fs::read(format!("/proc/{pid}/cmdline"))
            .map(|cmdline| {
                cmdline
                    .split(|byte| *byte == 0)
                    .filter(|arg| !arg.is_empty())
                    .map(|arg| OsString::from(String::from_utf8_lossy(arg).as_ref()))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn process_cwd(pid: u32) -> Option<PathBuf> {
        fs::read_link(format!("/proc/{pid}/cwd")).ok()
    }
}

/// Parse the parent pid from `/proc/<pid>/stat`. The `comm` field can contain
/// spaces and parentheses, so fields are read after the final `)`: the tokens
/// there are `state ppid ...`, making ppid the second one.
fn read_ppid(pid: u32) -> Option<u32> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let after_comm = &stat[stat.rfind(')')? + 1..];
    after_comm.split_whitespace().nth(1)?.parse().ok()
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::os::fd::AsRawFd;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn is_readable(fd: RawFd) -> bool {
        let mut pollfd = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };
        unsafe { libc::poll(&mut pollfd, 1, 0) == 1 }
    }

    #[test]
    fn output_wakeup_starts_signalled_and_coalesces() {
        let wakeup = Linux::new_output_wakeup().expect("create eventfd");
        assert!(is_readable(wakeup.as_fd().as_raw_fd()));

        wakeup.wake().expect("coalesced wake");
        wakeup.wake().expect("second coalesced wake");
        wakeup.clear().expect("clear wakeup");
        assert!(!is_readable(wakeup.as_fd().as_raw_fd()));

        wakeup.clear().expect("clear an already-clear wakeup");
        wakeup.wake().expect("wake after clear");
        assert!(is_readable(wakeup.as_fd().as_raw_fd()));
    }

    #[test]
    fn process_open_files_reports_current_process_file() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("hmux-open-file-{}-{nonce}", std::process::id()));
        let file = File::create(&path).expect("create temporary file");

        let open_files = Linux::process_open_files(std::process::id());
        assert!(open_files.contains(&path), "open files: {open_files:?}");

        drop(file);
        fs::remove_file(path).expect("remove temporary file");
    }
}
