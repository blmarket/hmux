//! macOS implementation of the operating-system boundary.

use std::ffi::{OsStr, OsString};
use std::io;
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd, RawFd};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::ptr;

use super::{ForkOutcome, OutputWakeup, Platform, ProcessInfo};

/// The macOS platform implementation selected by the native server.
pub(crate) struct Darwin;

/// A non-blocking self-pipe used as a coalescing readiness notification.
///
/// macOS has no `eventfd`, so the readable end stands in for it: any pending
/// byte makes it readable, and draining every byte clears the notification.
pub(crate) struct SelfPipe {
    read: OwnedFd,
    write: OwnedFd,
}

#[repr(C)]
struct ProcFileInfo {
    fi_openflags: u32,
    fi_status: u32,
    fi_offset: libc::off_t,
    fi_type: i32,
    fi_guardflags: u32,
}

#[repr(C)]
struct VnodeFdInfoWithPath {
    pfi: ProcFileInfo,
    pvip: libc::vnode_info_path,
}

const PROC_PIDFDVNODEPATHINFO: libc::c_int = 2;

/// Set both the non-blocking and close-on-exec flags on `fd`.
///
/// macOS lacks the atomic `pipe2`/`O_CLOEXEC` creation flags, so the pipe ends
/// are configured after creation with `fcntl`.
fn set_nonblocking_cloexec(fd: RawFd) -> io::Result<()> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 {
        return Err(io::Error::last_os_error());
    }
    if unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        return Err(io::Error::last_os_error());
    }
    let descriptor_flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if descriptor_flags < 0 {
        return Err(io::Error::last_os_error());
    }
    if unsafe { libc::fcntl(fd, libc::F_SETFD, descriptor_flags | libc::FD_CLOEXEC) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

impl AsFd for SelfPipe {
    fn as_fd(&self) -> BorrowedFd<'_> {
        // The readable end is what a consumer polls for readiness.
        self.read.as_fd()
    }
}

impl OutputWakeup for SelfPipe {
    fn wake(&self) -> io::Result<()> {
        let byte = 0u8;
        loop {
            let written =
                unsafe { libc::write(self.write.as_raw_fd(), (&byte as *const u8).cast(), 1) };
            if written == 1 {
                return Ok(());
            }
            if written >= 0 {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "short write to self-pipe",
                ));
            }
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            // A full self-pipe is already readable, which satisfies the
            // coalescing notification contract.
            if error.kind() == io::ErrorKind::WouldBlock {
                return Ok(());
            }
            return Err(error);
        }
    }

    fn clear(&self) -> io::Result<()> {
        let mut buffer = [0u8; 64];
        loop {
            let read = unsafe {
                libc::read(
                    self.read.as_raw_fd(),
                    buffer.as_mut_ptr().cast(),
                    buffer.len(),
                )
            };
            if read > 0 {
                // Keep draining: several `wake`s may have queued several bytes.
                continue;
            }
            if read == 0 {
                // The write end is still held here, so EOF cannot occur; treat a
                // zero-length read as fully drained.
                return Ok(());
            }
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            // Drained: no more pending bytes.
            if error.kind() == io::ErrorKind::WouldBlock {
                return Ok(());
            }
            return Err(error);
        }
    }
}

impl Platform for Darwin {
    type OutputWakeup = SelfPipe;

    fn new_output_wakeup() -> io::Result<Self::OutputWakeup> {
        let mut fds = [0 as RawFd; 2];
        if unsafe { libc::pipe(fds.as_mut_ptr()) } != 0 {
            return Err(io::Error::last_os_error());
        }
        let read = unsafe { OwnedFd::from_raw_fd(fds[0]) };
        let write = unsafe { OwnedFd::from_raw_fd(fds[1]) };
        set_nonblocking_cloexec(read.as_raw_fd())?;
        set_nonblocking_cloexec(write.as_raw_fd())?;
        let pipe = SelfPipe { read, write };
        // Start signalled so a fresh subscriber performs one initial scan.
        pipe.wake()?;
        Ok(pipe)
    }

    unsafe fn fork_pty(size: libc::winsize) -> io::Result<ForkOutcome> {
        let mut master = -1;
        // Apple's `forkpty` takes `*mut winsize`; copy so the argument is owned.
        let mut size = size;
        let pid =
            unsafe { libc::forkpty(&mut master, ptr::null_mut(), ptr::null_mut(), &mut size) };
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
        // macOS has no `close_range`; sweep up to the descriptor-table ceiling.
        // `getdtablesize` and `close` are async-signal-safe.
        let max = unsafe { libc::getdtablesize() };
        let mut fd = lowest;
        while fd < max {
            unsafe {
                libc::close(fd);
            }
            fd += 1;
        }
    }

    fn pane_cwd(pty: BorrowedFd<'_>) -> Option<PathBuf> {
        let foreground_pgrp = unsafe { libc::tcgetpgrp(pty.as_raw_fd()) };
        if foreground_pgrp <= 0 {
            return None;
        }

        process_vnode_cwd(foreground_pgrp)
    }

    fn peer_uid(socket: BorrowedFd<'_>) -> Option<u32> {
        let mut uid: libc::uid_t = 0;
        let mut gid: libc::gid_t = 0;
        let status = unsafe { libc::getpeereid(socket.as_raw_fd(), &mut uid, &mut gid) };
        (status == 0).then_some(uid)
    }

    fn process_table() -> Option<Vec<ProcessInfo>> {
        const PROC_ALL_PIDS: u32 = 1;

        let needed = unsafe { libc::proc_listpids(PROC_ALL_PIDS, 0, ptr::null_mut(), 0) };
        if needed <= 0 {
            return None;
        }

        let pid_size = std::mem::size_of::<libc::pid_t>();
        let mut pids = vec![0 as libc::pid_t; needed as usize / pid_size + 256];
        loop {
            let bytes = (pids.len() * pid_size).min(libc::c_int::MAX as usize) as libc::c_int;
            let returned =
                unsafe { libc::proc_listpids(PROC_ALL_PIDS, 0, pids.as_mut_ptr().cast(), bytes) };
            if returned <= 0 {
                return None;
            }
            let count = returned as usize / pid_size;
            if count < pids.len() {
                pids.truncate(count);
                break;
            }
            pids.resize(pids.len() * 2, 0);
        }

        let mut table = Vec::new();
        for pid in pids {
            if pid <= 0 {
                continue;
            }
            if let Some(info) = bsd_info(pid) {
                table.push(ProcessInfo {
                    pid: info.pbi_pid,
                    ppid: info.pbi_ppid,
                });
            }
        }
        Some(table)
    }

    fn process_programs(pid: u32) -> Vec<OsString> {
        let mut programs = Vec::new();

        if let Some(program) = Self::process_arguments(pid).first() {
            programs.push(program.clone());
        }

        let mut name = [0 as libc::c_char; libc::PROC_PIDPATHINFO_MAXSIZE as usize];
        let length = unsafe {
            libc::proc_name(
                pid as libc::c_int,
                name.as_mut_ptr().cast(),
                name.len() as u32,
            )
        };
        if length > 0 {
            programs.push(OsString::from(bytes_from_c_buffer(&name, length as usize)));
        }

        let mut path = [0 as libc::c_char; libc::PROC_PIDPATHINFO_MAXSIZE as usize];
        let length = unsafe {
            libc::proc_pidpath(
                pid as libc::c_int,
                path.as_mut_ptr().cast(),
                path.len() as u32,
            )
        };
        if length > 0 {
            programs.push(OsString::from(bytes_from_c_buffer(&path, length as usize)));
        }

        programs
    }

    fn process_arguments(pid: u32) -> Vec<OsString> {
        procargs2(pid)
            .map(|buffer| parse_procargs2(&buffer))
            .unwrap_or_default()
    }

    fn process_cwd(pid: u32) -> Option<PathBuf> {
        process_vnode_cwd(pid as libc::pid_t)
    }

    fn process_open_files(pid: u32) -> Vec<PathBuf> {
        list_fds(pid)
            .into_iter()
            .filter(|fd| fd.proc_fdtype == libc::PROX_FDTYPE_VNODE as u32)
            .filter_map(|fd| vnode_fd_path(pid, fd.proc_fd))
            .collect()
    }
}

/// Fetch the raw `KERN_PROCARGS2` buffer for `pid`: `argc`, the exec path, then
/// the argument strings, all in one sysctl. The argument vector is parsed out of
/// this buffer by [`parse_procargs2`].
fn procargs2(pid: u32) -> Option<Vec<u8>> {
    let mut mib = [libc::CTL_KERN, libc::KERN_PROCARGS2, pid as libc::c_int];
    let mut size = 0usize;
    if unsafe {
        libc::sysctl(
            mib.as_mut_ptr(),
            mib.len() as libc::c_uint,
            ptr::null_mut(),
            &mut size,
            ptr::null_mut(),
            0,
        )
    } != 0
        || size < std::mem::size_of::<libc::c_int>()
    {
        return None;
    }

    let mut buffer = vec![0u8; size];
    if unsafe {
        libc::sysctl(
            mib.as_mut_ptr(),
            mib.len() as libc::c_uint,
            buffer.as_mut_ptr().cast(),
            &mut size,
            ptr::null_mut(),
            0,
        )
    } != 0
    {
        return None;
    }
    buffer.truncate(size);
    Some(buffer)
}

fn bsd_info(pid: libc::pid_t) -> Option<libc::proc_bsdinfo> {
    let mut info: libc::proc_bsdinfo = unsafe { std::mem::zeroed() };
    let size = std::mem::size_of::<libc::proc_bsdinfo>() as libc::c_int;
    let written = unsafe {
        libc::proc_pidinfo(
            pid,
            libc::PROC_PIDTBSDINFO,
            0,
            (&mut info as *mut libc::proc_bsdinfo).cast(),
            size,
        )
    };
    (written == size).then_some(info)
}

fn process_vnode_cwd(pid: libc::pid_t) -> Option<PathBuf> {
    let mut info: libc::proc_vnodepathinfo = unsafe { std::mem::zeroed() };
    let size = std::mem::size_of::<libc::proc_vnodepathinfo>() as libc::c_int;
    let written = unsafe {
        libc::proc_pidinfo(
            pid,
            libc::PROC_PIDVNODEPATHINFO,
            0,
            (&mut info as *mut libc::proc_vnodepathinfo).cast(),
            size,
        )
    };
    (written == size)
        .then(|| path_from_vnode_info(&info.pvi_cdir))
        .flatten()
}

fn list_fds(pid: u32) -> Vec<libc::proc_fdinfo> {
    let fd_size = std::mem::size_of::<libc::proc_fdinfo>();
    let needed = unsafe {
        libc::proc_pidinfo(
            pid as libc::c_int,
            libc::PROC_PIDLISTFDS,
            0,
            ptr::null_mut(),
            0,
        )
    };
    if needed <= 0 {
        return Vec::new();
    }

    let mut fds = vec![zeroed_proc_fdinfo(); needed as usize / fd_size + 16];
    loop {
        let bytes = (fds.len() * fd_size).min(libc::c_int::MAX as usize) as libc::c_int;
        let returned = unsafe {
            libc::proc_pidinfo(
                pid as libc::c_int,
                libc::PROC_PIDLISTFDS,
                0,
                fds.as_mut_ptr().cast(),
                bytes,
            )
        };
        if returned <= 0 {
            return Vec::new();
        }
        let count = returned as usize / fd_size;
        if count < fds.len() {
            fds.truncate(count);
            return fds;
        }
        fds.resize(fds.len() * 2, zeroed_proc_fdinfo());
    }
}

fn zeroed_proc_fdinfo() -> libc::proc_fdinfo {
    libc::proc_fdinfo {
        proc_fd: 0,
        proc_fdtype: 0,
    }
}

fn vnode_fd_path(pid: u32, fd: libc::c_int) -> Option<PathBuf> {
    let mut info: VnodeFdInfoWithPath = unsafe { std::mem::zeroed() };
    let size = std::mem::size_of::<VnodeFdInfoWithPath>() as libc::c_int;
    let written = unsafe {
        libc::proc_pidfdinfo(
            pid as libc::c_int,
            fd,
            PROC_PIDFDVNODEPATHINFO,
            (&mut info as *mut VnodeFdInfoWithPath).cast(),
            size,
        )
    };
    (written == size)
        .then(|| path_from_vnode_info(&info.pvip))
        .flatten()
}

fn path_from_vnode_info(info: &libc::vnode_info_path) -> Option<PathBuf> {
    // `vip_path` is a fixed NUL-terminated buffer of `c_char`.
    let path = &info.vip_path;
    let bytes = unsafe {
        std::slice::from_raw_parts(path.as_ptr().cast::<u8>(), std::mem::size_of_val(path))
    };
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    if end == 0 {
        return None;
    }
    Some(PathBuf::from(OsStr::from_bytes(&bytes[..end])))
}

fn bytes_from_c_buffer(buffer: &[libc::c_char], length: usize) -> &OsStr {
    let bytes = unsafe { std::slice::from_raw_parts(buffer.as_ptr().cast::<u8>(), buffer.len()) };
    let end = bytes[..length.min(bytes.len())]
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(length.min(bytes.len()));
    OsStr::from_bytes(&bytes[..end])
}

fn parse_procargs2(buffer: &[u8]) -> Vec<OsString> {
    let Some(argc_bytes) = buffer.get(..std::mem::size_of::<libc::c_int>()) else {
        return Vec::new();
    };
    let argc = libc::c_int::from_ne_bytes(argc_bytes.try_into().unwrap());
    if argc <= 0 {
        return Vec::new();
    }

    let mut index = std::mem::size_of::<libc::c_int>();
    while index < buffer.len() && buffer[index] != 0 {
        index += 1;
    }
    while index < buffer.len() && buffer[index] == 0 {
        index += 1;
    }

    let mut args = Vec::new();
    while args.len() < argc as usize && index < buffer.len() {
        while index < buffer.len() && buffer[index] == 0 {
            index += 1;
        }
        let start = index;
        while index < buffer.len() && buffer[index] != 0 {
            index += 1;
        }
        if start < index {
            args.push(OsString::from(OsStr::from_bytes(&buffer[start..index])));
        }
    }
    args
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};
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
        let wakeup = Darwin::new_output_wakeup().expect("create self-pipe");
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
    fn pane_cwd_absent_for_non_tty() {
        // A pipe has no controlling terminal, so `tcgetpgrp` fails and no cwd is
        // reported.
        let mut fds = [0 as RawFd; 2];
        assert_eq!(unsafe { libc::pipe(fds.as_mut_ptr()) }, 0);
        let read = unsafe { OwnedFd::from_raw_fd(fds[0]) };
        let _write = unsafe { OwnedFd::from_raw_fd(fds[1]) };
        assert!(Darwin::pane_cwd(read.as_fd()).is_none());
    }

    #[test]
    fn process_cwd_reports_current_process_cwd() {
        let cwd = Darwin::process_cwd(std::process::id()).expect("read current process cwd");
        assert_eq!(
            fs::canonicalize(cwd).expect("canonicalize reported cwd"),
            fs::canonicalize(std::env::current_dir().expect("current dir"))
                .expect("canonicalize current dir")
        );
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
        let canonical_path = fs::canonicalize(&path).expect("canonicalize temporary file");

        let open_files = Darwin::process_open_files(std::process::id());
        let canonical_open_files = open_files
            .iter()
            .filter_map(|path| fs::canonicalize(path).ok())
            .collect::<Vec<_>>();
        assert!(
            canonical_open_files.contains(&canonical_path),
            "open files: {open_files:?}"
        );

        drop(file);
        fs::remove_file(path).expect("remove temporary file");
    }
}
