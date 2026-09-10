use super::{window_pane, on_pane_owned as on_pane, on_pane_error_owned as on_pane_error};
use crate::WindowPane;
use crate::compat::error_message;
pub use crate::consts::{
    _PATH_BSHELL, _PATH_DEVNULL, AF_UNIX, O_WRONLY, PF_UNSPEC, SIG_BLOCK, SIG_SETMASK, SOCK_STREAM,
    STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO,
};
use crate::ffi::{
    __errno_location, _exit, close, closefrom, dup2, execl, fork, open, setpgid, sigfillset,
    sigprocmask, socketpair,
};
use crate::fmt_args;
use crate::log::{fatalx, log_debug};
use crate::pane_identity::PaneIdentity;
use crate::proc::proc_clear_signals;
use crate::reactor::Interest;
use crate::server::server_destroy_pane;
use crate::server::server_process;
use crate::tmux::setblocking;
use crate::types::*;

pub(crate) struct PanePipePair([core::ffi::c_int; 2]);

impl PanePipePair {
    pub(crate) unsafe fn open() -> Result<Self, std::ffi::CString> {
        let mut descriptors = [0; 2];
        if unsafe {
            socketpair(
                AF_UNIX,
                SOCK_STREAM as core::ffi::c_int,
                PF_UNSPEC,
                descriptors.as_mut_ptr(),
            )
        } != 0
        {
            let mut cause = b"socketpair error: ".to_vec();
            cause.extend_from_slice(unsafe { error_message(*__errno_location()) }.to_bytes());
            return Err(std::ffi::CString::new(cause).expect("error message contains no NUL"));
        }
        Ok(Self(descriptors))
    }
}

impl Drop for PanePipePair {
    fn drop(&mut self) {
        for fd in self.0 {
            if fd != -1 {
                unsafe { close(fd) };
            }
        }
    }
}

pub(crate) struct PanePipeClosed {
    pub was_open: bool,
    pub removed: bool,
}

impl RustWindowPaneWeak {
    /// # Safety
    /// Exclude conflicting pane and pipe access. Destruction runs after releasing the pane owner.
    pub(crate) unsafe fn close_pipe(&self) -> Option<PanePipeClosed> {
        let mut owner = self.upgrade()?;
        let pane = unsafe { &mut *owner.0.pane.get() };
        let was_open = pane.pipe_fd != -1;
        let mut removed = false;
        if was_open {
            pane.pipe_event.free();
            unsafe { close(pane.pipe_fd) };
            pane.pipe_fd = -1;
            removed = unsafe { pane.destroy_ready() };
        }
        drop(owner);
        if removed {
            unsafe { server_destroy_pane(self, 1) };
        }
        Some(PanePipeClosed { was_open, removed })
    }

    /// # Safety
    /// Exclude conflicting pane and pipe access while forking and installing the pipe events.
    pub(crate) unsafe fn connect_pipe(
        &self,
        mut pair: PanePipePair,
        cmd: &core::ffi::CStr,
        in_0: core::ffi::c_int,
        out: core::ffi::c_int,
    ) -> Result<(), std::ffi::CString> {
        let Some(mut owner) = self.upgrade() else {
            return Ok(());
        };
        let wp = unsafe { &mut *owner.0.pane.get() };
        let pipe_fd = core::mem::replace(&mut pair.0, [-1; 2]);
        let null_fd: core::ffi::c_int;
        let mut set: sigset_t = __sigset_t { __val: [0; 16] };
        let mut oldset: sigset_t = __sigset_t { __val: [0; 16] };
        unsafe { sigfillset(&raw mut set) };
        unsafe { sigprocmask(SIG_BLOCK, &raw mut set, &raw mut oldset) };
        unsafe { wp.pipe_pid = fork() as pid_t };
        match wp.pipe_pid {
            -1 => {
                unsafe {
                    sigprocmask(
                        SIG_SETMASK,
                        &raw mut oldset,
                        core::ptr::null_mut::<sigset_t>(),
                    )
                };
                let mut cause = b"fork error: ".to_vec();
                cause.extend_from_slice(unsafe { error_message(*__errno_location()) }.to_bytes());
                unsafe {
                    close(pipe_fd[0]);
                    close(pipe_fd[1]);
                }
                Err(std::ffi::CString::new(cause).expect("error message contains no NUL"))
            }
            0 => {
                unsafe {
                    proc_clear_signals(
                        &mut server_process
                            .get()
                            .as_ref()
                            .expect("server process is initialized")
                            .borrow_mut(),
                        1 as core::ffi::c_int,
                    )
                };
                unsafe {
                    sigprocmask(
                        SIG_SETMASK,
                        &raw mut oldset,
                        core::ptr::null_mut::<sigset_t>(),
                    )
                };
                unsafe { close(pipe_fd[0 as core::ffi::c_int as usize]) };
                if unsafe { setpgid(0 as __pid_t, 0 as __pid_t) == -(1 as core::ffi::c_int) } {
                    unsafe { _exit(1 as core::ffi::c_int) };
                }
                unsafe { null_fd = open(_PATH_DEVNULL.as_ptr(), O_WRONLY) };
                if out != 0 {
                    if unsafe {
                        dup2(pipe_fd[1 as core::ffi::c_int as usize], STDIN_FILENO)
                            == -(1 as core::ffi::c_int)
                    } {
                        unsafe { _exit(1 as core::ffi::c_int) };
                    }
                } else if unsafe { dup2(null_fd, STDIN_FILENO) == -(1 as core::ffi::c_int) } {
                    unsafe { _exit(1 as core::ffi::c_int) };
                }
                if in_0 != 0 {
                    if unsafe {
                        dup2(pipe_fd[1 as core::ffi::c_int as usize], STDOUT_FILENO)
                            == -(1 as core::ffi::c_int)
                    } {
                        unsafe { _exit(1 as core::ffi::c_int) };
                    }
                    if pipe_fd[1 as core::ffi::c_int as usize] != STDOUT_FILENO {
                        unsafe { close(pipe_fd[1 as core::ffi::c_int as usize]) };
                    }
                } else if unsafe { dup2(null_fd, STDOUT_FILENO) == -(1 as core::ffi::c_int) } {
                    unsafe { _exit(1 as core::ffi::c_int) };
                }
                if unsafe { dup2(null_fd, STDERR_FILENO) == -(1 as core::ffi::c_int) } {
                    unsafe { _exit(1 as core::ffi::c_int) };
                }
                unsafe { closefrom(STDERR_FILENO + 1 as core::ffi::c_int) };
                unsafe {
                    execl(
                        _PATH_BSHELL.as_ptr(),
                        c"sh".as_ptr(),
                        c"-c".as_ptr(),
                        cmd.as_ptr(),
                        core::ptr::null_mut::<core::ffi::c_char>(),
                    )
                };
                unsafe { _exit(1 as core::ffi::c_int) };
            }
            _ => {
                unsafe {
                    sigprocmask(
                        SIG_SETMASK,
                        &raw mut oldset,
                        core::ptr::null_mut::<sigset_t>(),
                    )
                };
                unsafe { close(pipe_fd[1 as core::ffi::c_int as usize]) };
                wp.pipe_fd = pipe_fd[0 as core::ffi::c_int as usize];
                wp.pipe_offset = wp.offset;
                setblocking(wp.pipe_fd, 0 as core::ffi::c_int);
                let id = wp.pane_id();
                wp.pipe_event = Stream::new(
                    wp.pipe_fd,
                    Some(on_pane(id, cmd_pipe_pane_read_callback)),
                    Some(on_pane(id, cmd_pipe_pane_write_callback)),
                    Some(on_pane_error(id, cmd_pipe_pane_error_callback)),
                );
                if wp.pipe_event.is_none() {
                    fatalx(c"out of memory", fmt_args![]);
                }
                if out != 0 {
                    wp.pipe_event.enable(Interest::Write);
                }
                if in_0 != 0 {
                    wp.pipe_event.enable(Interest::Read);
                }
                Ok(())
            }
        }
    }
}
fn cmd_pipe_pane_read_callback(wp: &mut window_pane) {
    unsafe {
        let data = wp
            .pipe_event
            .with_input(|buffer| buffer.copy_to_bytes(buffer.len()))
            .unwrap_or_default();
        let available = data.len();
        log_debug(c"%%%u pipe read %zu", fmt_args![wp.pane_id(), available]);
        wp.event.write(&data);
        if wp.destroy_ready() {
            server_destroy_pane(
                &(wp).observation().expect("the pane is owned"),
                1 as core::ffi::c_int,
            );
        }
    }
}
fn cmd_pipe_pane_write_callback(wp: &mut window_pane) {
    unsafe {
        log_debug(c"%%%u pipe empty", fmt_args![wp.pane_id()]);
        if wp.destroy_ready() {
            server_destroy_pane(
                &(wp).observation().expect("the pane is owned"),
                1 as core::ffi::c_int,
            );
        }
    }
}
fn cmd_pipe_pane_error_callback(wp: &mut window_pane) {
    unsafe {
        log_debug(c"%%%u pipe error", fmt_args![wp.pane_id()]);
        wp.pipe_event.free();
        close(wp.pipe_fd);
        wp.pipe_fd = -(1 as core::ffi::c_int);
        if wp.destroy_ready() {
            server_destroy_pane(
                &(wp).observation().expect("the pane is owned"),
                1 as core::ffi::c_int,
            );
        }
    }
}
