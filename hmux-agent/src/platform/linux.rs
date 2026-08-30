//! Linux implementation of the operating-system boundary.

use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd, RawFd};
use std::path::PathBuf;
use std::ptr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::{ForkOutcome, OutputWakeup, Platform, ProcessInfo};

/// The Linux platform implementation selected by the native server.
pub struct Linux;

/// A non-blocking `eventfd` used as a coalescing readiness notification.
pub struct EventFd(OwnedFd);

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
        // Mirrors tmux's osdep_get_cwd: prefer the foreground process group,
        // then fall back to the session leader. The group id is only a pid
        // while the group leader lives; a job whose leader has exited (a shell
        // pipeline, or a wrapper that exec'd away) leaves a group whose id
        // names no process, and /proc/<pgrp>/cwd is then unreadable. The
        // session leader is the pane's own shell, so it still answers.
        let read_cwd = |pid: libc::pid_t| {
            (pid > 0)
                .then(|| PathBuf::from(format!("/proc/{pid}/cwd")))
                .and_then(|path| fs::read_link(path).ok())
        };
        let foreground_pgrp = unsafe { libc::tcgetpgrp(pty.as_raw_fd()) };
        read_cwd(foreground_pgrp).or_else(|| read_cwd(unsafe { libc::tcgetsid(pty.as_raw_fd()) }))
    }

    fn peer_uid(socket: BorrowedFd<'_>) -> Option<u32> {
        let mut credentials = unsafe { std::mem::zeroed::<libc::ucred>() };
        let mut length = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
        let status = unsafe {
            libc::getsockopt(
                socket.as_raw_fd(),
                libc::SOL_SOCKET,
                libc::SO_PEERCRED,
                (&mut credentials as *mut libc::ucred).cast(),
                &mut length,
            )
        };
        (status == 0).then_some(credentials.uid)
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

    fn process_start_time(pid: u32) -> Option<SystemTime> {
        let since_boot = read_start_ticks(pid)? as f64 / clock_ticks_per_second()? as f64;
        boot_time()?.checked_add(Duration::from_secs_f64(since_boot))
    }

    fn process_environ(pid: u32) -> Vec<(OsString, OsString)> {
        use std::os::unix::ffi::OsStrExt;
        let Ok(environ) = fs::read(format!("/proc/{pid}/environ")) else {
            return Vec::new();
        };
        environ
            .split(|byte| *byte == 0)
            .filter(|entry| !entry.is_empty())
            .filter_map(|entry| {
                // Only the first `=` separates name from value; values may
                // contain any number of them.
                let split = entry.iter().position(|byte| *byte == b'=')?;
                Some((
                    OsStr::from_bytes(&entry[..split]).to_owned(),
                    OsStr::from_bytes(&entry[split + 1..]).to_owned(),
                ))
            })
            .collect()
    }

    fn process_waiting_for_tty(pid: u32) -> Option<bool> {
        // Two readings have to agree, because neither is conclusive alone.
        //
        // `/proc/PID/wchan` names the kernel function the task is parked in.
        // A tty read parks in `wait_woken` — but so does a socket read, so the
        // symbol only narrows the field to "asleep in a driver read".
        //
        // `/proc/PID/syscall` then says which descriptor that read is on: for
        // the read-family calls a tty read can be sitting in, the first
        // argument is the file descriptor. Resolving it through `fd/` tells a
        // terminal apart from a socket, which is what the symbol could not.
        //
        // A process waiting on the terminal through `poll`/`select` instead
        // parks in `poll_schedule_timeout` and passes a pointer rather than a
        // descriptor, so it reads as running. That is the conservative
        // direction: a busy pane is never mistaken for one waiting on you.
        let wchan = fs::read_to_string(format!("/proc/{pid}/wchan")).ok()?;
        if !TTY_READ_WCHAN.contains(&wchan.trim()) {
            return Some(false);
        }
        let syscall = fs::read_to_string(format!("/proc/{pid}/syscall")).ok()?;
        // "running" for a task on a cpu, "-1 ..." when the registers are gone.
        let Some(descriptor) = syscall
            .split_whitespace()
            .nth(1)
            .and_then(|argument| argument.strip_prefix("0x"))
            .and_then(|argument| u32::from_str_radix(argument, 16).ok())
        else {
            return Some(false);
        };
        let Ok(target) = fs::read_link(format!("/proc/{pid}/fd/{descriptor}")) else {
            return Some(false);
        };
        Some(target.to_str().is_some_and(|target| {
            target.starts_with("/dev/pts/") || target.starts_with("/dev/tty")
        }))
    }
}

/// The `wchan` symbols a task blocked reading a terminal parks in.
/// `wait_woken` is what current kernels report for `n_tty_read`; the two named
/// functions are what older ones reported directly.
const TTY_READ_WCHAN: [&str; 3] = ["wait_woken", "n_tty_read", "tty_read"];

/// Seconds since the epoch at which the kernel booted, from `/proc/stat`'s
/// `btime` line. Process start times are recorded relative to this instant.
fn boot_time() -> Option<SystemTime> {
    let stat = fs::read_to_string("/proc/stat").ok()?;
    let seconds: u64 = stat
        .lines()
        .find_map(|line| line.strip_prefix("btime "))?
        .trim()
        .parse()
        .ok()?;
    UNIX_EPOCH.checked_add(Duration::from_secs(seconds))
}

/// Parse the start time from `/proc/<pid>/stat`, in clock ticks since boot.
/// The `comm` field can contain spaces and parentheses, so fields are read
/// after the final `)`: the tokens there begin at `state`, making `starttime`
/// (field 22 overall) the twentieth.
fn read_start_ticks(pid: u32) -> Option<u64> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let after_comm = &stat[stat.rfind(')')? + 1..];
    after_comm.split_whitespace().nth(19)?.parse().ok()
}

fn clock_ticks_per_second() -> Option<u64> {
    let ticks = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    (ticks > 0).then_some(ticks as u64)
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

    /// A foreground process group outlives its leader whenever the leader exits
    /// while another member keeps running, and the group id then names no
    /// process at all. `/proc/<pgid>/cwd` is unreadable in that state, so the
    /// pane must fall back to its session leader the way tmux does.
    #[test]
    fn pane_cwd_falls_back_to_session_leader_for_a_leaderless_group() {
        use std::ffi::CString;

        // The session leader parks here; /proc is always present and is not the
        // directory the test process itself runs in.
        let parked_cwd = CString::new("/proc").expect("nul-free path");

        let (mut master, mut slave) = (-1, -1);
        let opened = unsafe {
            libc::openpty(
                &mut master,
                &mut slave,
                ptr::null_mut(),
                ptr::null(),
                ptr::null(),
            )
        };
        assert_eq!(opened, 0, "openpty: {}", io::Error::last_os_error());

        let mut report = [-1; 2];
        assert_eq!(
            unsafe { libc::pipe(report.as_mut_ptr()) },
            0,
            "pipe: {}",
            io::Error::last_os_error()
        );
        let (report_read, report_write) = (report[0], report[1]);

        // SAFETY: every branch below the fork terminates in `_exit` or blocks
        // forever in `pause`, and touches only async-signal-safe libc calls.
        let leader = unsafe { libc::fork() };
        assert!(leader >= 0, "fork: {}", io::Error::last_os_error());
        if leader == 0 {
            unsafe {
                libc::close(report_read);
                // tcsetpgrp from a background group would otherwise stop us.
                libc::signal(libc::SIGTTOU, libc::SIG_IGN);
                libc::setsid();
                libc::ioctl(slave, libc::TIOCSCTTY, 0);
                libc::chdir(parked_cwd.as_ptr());

                let group_leader = libc::fork();
                if group_leader == 0 {
                    libc::setpgid(0, 0);
                    let pgid = libc::getpid();
                    // The member inherits the new group, so the group survives
                    // its leader without any handshake.
                    if libc::fork() == 0 {
                        loop {
                            libc::pause();
                        }
                    }
                    libc::tcsetpgrp(slave, pgid);
                    libc::_exit(0);
                }

                let mut status = 0;
                libc::waitpid(group_leader, &mut status, 0);
                // Reported only once the group is provably leaderless.
                libc::write(
                    report_write,
                    (&group_leader as *const libc::pid_t).cast(),
                    std::mem::size_of::<libc::pid_t>(),
                );
                loop {
                    libc::pause();
                }
            }
        }

        unsafe { libc::close(report_write) };
        let mut pgid: libc::pid_t = -1;
        let read = unsafe {
            libc::read(
                report_read,
                (&mut pgid as *mut libc::pid_t).cast(),
                std::mem::size_of::<libc::pid_t>(),
            )
        };
        let cwd = (read == std::mem::size_of::<libc::pid_t>() as isize).then(|| {
            let foreground = unsafe { libc::tcgetpgrp(master) };
            let cwd = Linux::pane_cwd(unsafe { BorrowedFd::borrow_raw(master) });
            (foreground, cwd)
        });

        unsafe {
            if pgid > 0 {
                libc::kill(-pgid, libc::SIGKILL);
            }
            libc::kill(leader, libc::SIGKILL);
            let mut status = 0;
            libc::waitpid(leader, &mut status, 0);
            libc::close(report_read);
            libc::close(slave);
            libc::close(master);
        }

        let (foreground, cwd) = cwd.expect("session leader reported the leaderless group");
        assert_eq!(foreground, pgid, "the leaderless group holds the terminal");
        assert!(
            !PathBuf::from(format!("/proc/{pgid}")).exists(),
            "the group id must name no process for this test to mean anything"
        );
        assert_eq!(cwd, Some(PathBuf::from("/proc")));
    }

    /// Fork a child onto the slave end of a fresh pty and run `probe` against
    /// it once it has had a moment to reach its steady state. `body` runs in
    /// the child and must never return.
    fn with_pty_child(body: unsafe fn() -> !, probe: impl FnOnce(u32)) {
        let (mut master, mut slave) = (-1, -1);
        let opened = unsafe {
            libc::openpty(
                &mut master,
                &mut slave,
                ptr::null_mut(),
                ptr::null(),
                ptr::null(),
            )
        };
        assert_eq!(opened, 0, "openpty: {}", io::Error::last_os_error());

        // SAFETY: the child branch touches only async-signal-safe libc calls
        // and terminates in `_exit`, which `body` is required to guarantee.
        let child = unsafe { libc::fork() };
        assert!(child >= 0, "fork: {}", io::Error::last_os_error());
        if child == 0 {
            unsafe {
                libc::close(master);
                libc::dup2(slave, 0);
                libc::setsid();
                libc::ioctl(slave, libc::TIOCSCTTY, 0);
                body();
            }
        }

        // The child has to reach its read (or its loop) before the probe means
        // anything; a freshly forked task is briefly running either way.
        std::thread::sleep(Duration::from_millis(250));
        let observed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            probe(child as u32);
        }));

        unsafe {
            libc::kill(child, libc::SIGKILL);
            let mut status = 0;
            libc::waitpid(child, &mut status, 0);
            libc::close(slave);
            libc::close(master);
        }
        if let Err(panic) = observed {
            std::panic::resume_unwind(panic);
        }
    }

    #[test]
    fn a_process_parked_in_a_terminal_read_is_reported_as_waiting() {
        unsafe fn read_stdin_forever() -> ! {
            let mut byte = 0u8;
            loop {
                unsafe { libc::read(0, (&mut byte as *mut u8).cast(), 1) };
            }
        }

        with_pty_child(read_stdin_forever, |pid| {
            assert_eq!(Linux::process_waiting_for_tty(pid), Some(true));
        });
    }

    /// The reading above is only worth anything if work does not look like it.
    /// A spinning process is the plain case, and a process asleep on something
    /// that is not the terminal — a timer — is the one `wchan` alone would get
    /// wrong, since it parks in a different symbol than a terminal read does.
    #[test]
    fn a_busy_or_otherwise_sleeping_process_is_not_reported_as_waiting() {
        unsafe fn spin_forever() -> ! {
            loop {
                std::hint::spin_loop();
            }
        }

        unsafe fn sleep_forever() -> ! {
            loop {
                unsafe { libc::sleep(30) };
            }
        }

        with_pty_child(spin_forever, |pid| {
            assert_eq!(Linux::process_waiting_for_tty(pid), Some(false));
        });
        with_pty_child(sleep_forever, |pid| {
            assert_eq!(Linux::process_waiting_for_tty(pid), Some(false));
        });
    }
}
