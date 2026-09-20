pub use crate::src::shared::arguments::{args};
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds};
pub use crate::src::shared::control::{control_state};
pub use crate::src::shared::format::{format_job_tree, format_tree};
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
pub use crate::src::shared::job::{
    job, job_complete_cb, job_entry, job_free_cb, job_state, job_update_cb,
};
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::menu::{menu_data};
pub use crate::src::shared::options::{options};
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::process::{tmuxpeer, tmuxproc};
pub use crate::src::shared::prompt::{prompt};
pub use crate::src::shared::redraw::{redraw_scene};
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::{spawn_editor_state};
pub use crate::src::shared::status::{status_line};
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
pub use crate::src::shared::environment::{environ};
pub use crate::src::shared::posix_io::{
    STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO, _PATH_BSHELL, _PATH_DEVNULL,
};
pub use crate::src::shared::job::{
    JOB_DEFAULTSHELL, JOB_KEEPWRITE, JOB_NOWAIT, JOB_PTY, JOB_SHOWSTDERR,
};
pub use crate::src::shared::posix_terminal::{winsize, TIOCSWINSZ};
pub use crate::src::shared::socket::{
    __socket_type, AF_UNIX, PF_LOCAL, PF_UNIX, PF_UNSPEC, SOCK_CLOEXEC, SOCK_DCCP, SOCK_DGRAM,
    SOCK_NONBLOCK, SOCK_PACKET, SOCK_RAW, SOCK_RDM, SOCK_SEQPACKET, SOCK_STREAM,
};
pub use crate::src::shared::signal::{
    __sigset_t, sigset_t, SIGCONT, SIGTERM, SIGTTIN, SIGTTOU, SIG_BLOCK, SIG_SETMASK,
};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::event::{EV_READ, EV_WRITE};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::terminal::*;
use crate::src::shared::event::*;
use crate::src::shared::display::*;
use crate::src::shared::layout::*;
use crate::src::shared::message::*;
use crate::src::shared::abi::*;
use crate::src::shared::colour::*;
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
use crate::src::shared::style::*;
extern "C" {

    fn ioctl(__fd: ::core::ffi::c_int, __request: ::core::ffi::c_ulong, ...) -> ::core::ffi::c_int;
    fn socketpair(
        __domain: ::core::ffi::c_int,
        __type: ::core::ffi::c_int,
        __protocol: ::core::ffi::c_int,
        __fds: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn shutdown(__fd: ::core::ffi::c_int, __how: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn kill(__pid: __pid_t, __sig: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn killpg(__pgrp: __pid_t, __sig: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn sigfillset(__set: *mut sigset_t) -> ::core::ffi::c_int;
    fn sigprocmask(
        __how: ::core::ffi::c_int,
        __set: *const sigset_t,
        __oset: *mut sigset_t,
    ) -> ::core::ffi::c_int;
    fn close(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn closefrom(__lowfd: ::core::ffi::c_int);
    fn chdir(__path: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn dup2(__fd: ::core::ffi::c_int, __fd2: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn execl(
        __path: *const ::core::ffi::c_char,
        __arg: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn execvp(
        __file: *const ::core::ffi::c_char,
        __argv: *const *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn _exit(__status: ::core::ffi::c_int) -> !;
    fn fork() -> __pid_t;
    fn open(
        __file: *const ::core::ffi::c_char,
        __oflag: ::core::ffi::c_int,
        ...
    ) -> ::core::ffi::c_int;
    fn setenv(
        __name: *const ::core::ffi::c_char,
        __value: *const ::core::ffi::c_char,
        __replace: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strlcpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_ulong;
    fn evbuffer_get_length(buf: *const evbuffer) -> size_t;
    fn bufferevent_free(bufev: *mut bufferevent);
    fn bufferevent_get_output(bufev: *mut bufferevent) -> *mut evbuffer;
    fn bufferevent_enable(
        bufev: *mut bufferevent,
        event: ::core::ffi::c_short,
    ) -> ::core::ffi::c_int;
    fn bufferevent_disable(
        bufev: *mut bufferevent,
        event: ::core::ffi::c_short,
    ) -> ::core::ffi::c_int;
    fn bufferevent_new(
        fd: ::core::ffi::c_int,
        readcb: bufferevent_data_cb,
        writecb: bufferevent_data_cb,
        errorcb: bufferevent_event_cb,
        cbarg: *mut ::core::ffi::c_void,
    ) -> *mut bufferevent;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn fdforkpty(
        _: ::core::ffi::c_int,
        _: *mut ::core::ffi::c_int,
        _: *mut ::core::ffi::c_char,
        _: *mut termios,
        _: *mut winsize,
    ) -> pid_t;
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static mut global_s_options: *mut options;
    static mut ptm_fd: ::core::ffi::c_int;
    fn checkshell(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn setblocking(_: ::core::ffi::c_int, _: ::core::ffi::c_int);
    fn shell_argv0(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn find_home() -> *const ::core::ffi::c_char;
    fn proc_clear_signals(_: *mut tmuxproc, _: ::core::ffi::c_int);
    static mut cfg_finished: ::core::ffi::c_int;
    fn options_get_string(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn environ_free(_: *mut environ);
    fn environ_copy(_: *mut environ, _: *mut environ);
    fn environ_set(
        _: *mut environ,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn environ_push(_: *mut environ);
    fn environ_for_session(_: *mut session, _: ::core::ffi::c_int) -> *mut environ;
    fn cmd_log_argv(
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn cmd_copy_argv(
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
    ) -> *mut *mut ::core::ffi::c_char;
    fn cmd_stringify_argv(
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn cmdq_print(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    static mut server_proc: *mut tmuxproc;
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn fatal(_: *const ::core::ffi::c_char, ...) -> !;
    fn fatalx(_: *const ::core::ffi::c_char, ...) -> !;
}

pub type C2RustUnnamed = ::core::ffi::c_uint;
pub const SHUT_RDWR: C2RustUnnamed = 2;
pub const SHUT_WR: C2RustUnnamed = 1;
pub const SHUT_RD: C2RustUnnamed = 0;

pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_14;

pub const JOB_CLOSED: job_state = 2;
pub const JOB_DEAD: job_state = 1;
pub const JOB_RUNNING: job_state = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct joblist {
    pub lh_first: *mut job,
}

pub const O_RDWR: ::core::ffi::c_int = 0o2 as ::core::ffi::c_int;

static mut all_jobs: joblist = joblist {
    lh_first: ::core::ptr::null::<job>() as *mut job,
};
#[no_mangle]
pub unsafe extern "C" fn job_run(
    mut cmd: *const ::core::ffi::c_char,
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
    mut e: *mut environ,
    mut s: *mut session,
    mut cwd: *const ::core::ffi::c_char,
    mut updatecb: job_update_cb,
    mut completecb: job_complete_cb,
    mut freecb: job_free_cb,
    mut data: *mut ::core::ffi::c_void,
    mut flags: ::core::ffi::c_int,
    mut sx: ::core::ffi::c_int,
    mut sy: ::core::ffi::c_int,
) -> *mut job {
    let mut current_block: u64;
    let mut job: *mut job = ::core::ptr::null_mut::<job>();
    let mut env: *mut environ = ::core::ptr::null_mut::<environ>();
    let mut pid: pid_t = 0;
    let mut nullfd: ::core::ffi::c_int = 0;
    let mut out: [::core::ffi::c_int; 2] = [0; 2];
    let mut master: ::core::ffi::c_int = 0;
    let mut do_close: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut home: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut shell: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut set: sigset_t = __sigset_t { __val: [0; 16] };
    let mut oldset: sigset_t = __sigset_t { __val: [0; 16] };
    let mut ws: winsize = winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let mut argvp: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut tty: [::core::ffi::c_char; 32] = [0; 32];
    let mut argv0: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    env = environ_for_session(s, (cfg_finished == 0) as ::core::ffi::c_int);
    if !e.is_null() {
        environ_copy(e, env);
    }
    if !flags & JOB_DEFAULTSHELL != 0 {
        shell = _PATH_BSHELL.as_ptr();
    } else {
        if !s.is_null() {
            oo = (*s).options;
        } else {
            oo = global_s_options;
        }
        shell = options_get_string(
            oo,
            b"default-shell\0" as *const u8 as *const ::core::ffi::c_char,
        );
        if checkshell(shell) == 0 {
            shell = _PATH_BSHELL.as_ptr();
        }
    }
    argv0 = shell_argv0(shell, 0 as ::core::ffi::c_int);
    sigfillset(&raw mut set);
    sigprocmask(SIG_BLOCK, &raw mut set, &raw mut oldset);
    if flags & JOB_PTY != 0 {
        memset(
            &raw mut ws as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<winsize>() as size_t,
        );
        ws.ws_col = sx as ::core::ffi::c_ushort;
        ws.ws_row = sy as ::core::ffi::c_ushort;
        pid = fdforkpty(
            ptm_fd,
            &raw mut master,
            &raw mut tty as *mut ::core::ffi::c_char,
            ::core::ptr::null_mut::<termios>(),
            &raw mut ws,
        );
        current_block = 224731115979188411;
    } else if socketpair(
        AF_UNIX,
        SOCK_STREAM as ::core::ffi::c_int,
        PF_UNSPEC,
        &raw mut out as *mut ::core::ffi::c_int,
    ) != 0 as ::core::ffi::c_int
    {
        current_block = 12393940290395533062;
    } else {
        pid = fork() as pid_t;
        current_block = 224731115979188411;
    }
    match current_block {
        224731115979188411 => {
            if cmd.is_null() {
                cmd_log_argv(
                    argc,
                    argv,
                    b"%s:\0" as *const u8 as *const ::core::ffi::c_char,
                    b"job_run\0" as *const u8 as *const ::core::ffi::c_char,
                );
                log_debug(
                    b"%s: cwd=%s, shell=%s\0" as *const u8 as *const ::core::ffi::c_char,
                    b"job_run\0" as *const u8 as *const ::core::ffi::c_char,
                    if cwd.is_null() {
                        b"\0" as *const u8 as *const ::core::ffi::c_char
                    } else {
                        cwd
                    },
                    shell,
                );
            } else {
                log_debug(
                    b"%s: cmd=%s, cwd=%s, shell=%s\0" as *const u8 as *const ::core::ffi::c_char,
                    b"job_run\0" as *const u8 as *const ::core::ffi::c_char,
                    cmd,
                    if cwd.is_null() {
                        b"\0" as *const u8 as *const ::core::ffi::c_char
                    } else {
                        cwd
                    },
                    shell,
                );
            }
            match pid {
                -1 => {
                    if !flags & JOB_PTY != 0 {
                        close(out[0 as ::core::ffi::c_int as usize]);
                        close(out[1 as ::core::ffi::c_int as usize]);
                    }
                }
                0 => {
                    proc_clear_signals(server_proc, 1 as ::core::ffi::c_int);
                    sigprocmask(
                        SIG_SETMASK,
                        &raw mut oldset,
                        ::core::ptr::null_mut::<sigset_t>(),
                    );
                    if !cwd.is_null() {
                        if chdir(cwd) == 0 as ::core::ffi::c_int {
                            environ_set(
                                env,
                                b"PWD\0" as *const u8 as *const ::core::ffi::c_char,
                                0 as ::core::ffi::c_int,
                                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                                cwd,
                            );
                        } else {
                            home = find_home();
                            if !home.is_null() && chdir(home) == 0 as ::core::ffi::c_int {
                                environ_set(
                                    env,
                                    b"PWD\0" as *const u8 as *const ::core::ffi::c_char,
                                    0 as ::core::ffi::c_int,
                                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                                    home,
                                );
                            } else if chdir(b"/\0" as *const u8 as *const ::core::ffi::c_char)
                                == 0 as ::core::ffi::c_int
                            {
                                environ_set(
                                    env,
                                    b"PWD\0" as *const u8 as *const ::core::ffi::c_char,
                                    0 as ::core::ffi::c_int,
                                    b"/\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                            } else {
                                _exit(1 as ::core::ffi::c_int);
                            }
                        }
                    }
                    environ_push(env);
                    environ_free(env);
                    if !flags & JOB_PTY != 0 {
                        if dup2(out[1 as ::core::ffi::c_int as usize], STDIN_FILENO)
                            == -(1 as ::core::ffi::c_int)
                        {
                            _exit(1 as ::core::ffi::c_int);
                        }
                        do_close = (do_close != 0
                            && out[1 as ::core::ffi::c_int as usize] != STDIN_FILENO)
                            as ::core::ffi::c_int;
                        if dup2(out[1 as ::core::ffi::c_int as usize], STDOUT_FILENO)
                            == -(1 as ::core::ffi::c_int)
                        {
                            _exit(1 as ::core::ffi::c_int);
                        }
                        do_close = (do_close != 0
                            && out[1 as ::core::ffi::c_int as usize] != STDOUT_FILENO)
                            as ::core::ffi::c_int;
                        if flags & JOB_SHOWSTDERR != 0 {
                            if dup2(out[1 as ::core::ffi::c_int as usize], STDERR_FILENO)
                                == -(1 as ::core::ffi::c_int)
                            {
                                _exit(1 as ::core::ffi::c_int);
                            }
                            do_close = (do_close != 0
                                && out[1 as ::core::ffi::c_int as usize] != STDERR_FILENO)
                                as ::core::ffi::c_int;
                        } else {
                            nullfd = open(_PATH_DEVNULL.as_ptr(), O_RDWR);
                            if nullfd == -(1 as ::core::ffi::c_int) {
                                _exit(1 as ::core::ffi::c_int);
                            }
                            if dup2(nullfd, STDERR_FILENO) == -(1 as ::core::ffi::c_int) {
                                _exit(1 as ::core::ffi::c_int);
                            }
                            if nullfd != STDERR_FILENO {
                                close(nullfd);
                            }
                        }
                        if do_close != 0 {
                            close(out[1 as ::core::ffi::c_int as usize]);
                        }
                        close(out[0 as ::core::ffi::c_int as usize]);
                    }
                    closefrom(STDERR_FILENO + 1 as ::core::ffi::c_int);
                    if !cmd.is_null() {
                        if flags & JOB_DEFAULTSHELL != 0 {
                            setenv(
                                b"SHELL\0" as *const u8 as *const ::core::ffi::c_char,
                                shell,
                                1 as ::core::ffi::c_int,
                            );
                        }
                        execl(
                            shell,
                            argv0,
                            b"-c\0" as *const u8 as *const ::core::ffi::c_char,
                            cmd,
                            NULL as *mut ::core::ffi::c_char,
                        );
                        _exit(1 as ::core::ffi::c_int);
                    } else {
                        argvp = cmd_copy_argv(argc, argv);
                        execvp(
                            *argvp.offset(0 as ::core::ffi::c_int as isize),
                            argvp as *const *mut ::core::ffi::c_char,
                        );
                        _exit(1 as ::core::ffi::c_int);
                    }
                }
                _ => {
                    sigprocmask(
                        SIG_SETMASK,
                        &raw mut oldset,
                        ::core::ptr::null_mut::<sigset_t>(),
                    );
                    environ_free(env);
                    free(argv0 as *mut ::core::ffi::c_void);
                    job = xcalloc(1 as size_t, ::core::mem::size_of::<job>() as size_t) as *mut job;
                    (*job).state = JOB_RUNNING;
                    (*job).flags = flags;
                    if !cmd.is_null() {
                        (*job).cmd = xstrdup(cmd);
                    } else {
                        (*job).cmd = cmd_stringify_argv(argc, argv);
                    }
                    (*job).pid = pid;
                    if flags & JOB_PTY != 0 {
                        strlcpy(
                            &raw mut (*job).tty as *mut ::core::ffi::c_char,
                            &raw mut tty as *mut ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
                        );
                    }
                    (*job).status = 0 as ::core::ffi::c_int;
                    (*job).entry.le_next = all_jobs.lh_first;
                    if !(*job).entry.le_next.is_null() {
                        (*all_jobs.lh_first).entry.le_prev = &raw mut (*job).entry.le_next;
                    }
                    all_jobs.lh_first = job;
                    (*job).entry.le_prev = &raw mut all_jobs.lh_first;
                    (*job).updatecb = updatecb;
                    (*job).completecb = completecb;
                    (*job).freecb = freecb;
                    (*job).data = data;
                    if !flags & JOB_PTY != 0 {
                        close(out[1 as ::core::ffi::c_int as usize]);
                        (*job).fd = out[0 as ::core::ffi::c_int as usize];
                    } else {
                        (*job).fd = master;
                    }
                    setblocking((*job).fd, 0 as ::core::ffi::c_int);
                    (*job).event = bufferevent_new(
                        (*job).fd,
                        Some(
                            job_read_callback
                                as unsafe extern "C" fn(
                                    *mut bufferevent,
                                    *mut ::core::ffi::c_void,
                                ) -> (),
                        ),
                        Some(
                            job_write_callback
                                as unsafe extern "C" fn(
                                    *mut bufferevent,
                                    *mut ::core::ffi::c_void,
                                ) -> (),
                        ),
                        Some(
                            job_error_callback
                                as unsafe extern "C" fn(
                                    *mut bufferevent,
                                    ::core::ffi::c_short,
                                    *mut ::core::ffi::c_void,
                                ) -> (),
                        ),
                        job as *mut ::core::ffi::c_void,
                    );
                    if (*job).event.is_null() {
                        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
                    }
                    bufferevent_enable((*job).event, (EV_READ | EV_WRITE) as ::core::ffi::c_short);
                    log_debug(
                        b"run job %p: %s, pid %ld\0" as *const u8 as *const ::core::ffi::c_char,
                        job,
                        (*job).cmd,
                        (*job).pid as ::core::ffi::c_long,
                    );
                    return job;
                }
            }
        }
        _ => {}
    }
    sigprocmask(
        SIG_SETMASK,
        &raw mut oldset,
        ::core::ptr::null_mut::<sigset_t>(),
    );
    environ_free(env);
    free(argv0 as *mut ::core::ffi::c_void);
    return ::core::ptr::null_mut::<job>();
}
#[no_mangle]
pub unsafe extern "C" fn job_transfer(
    mut job: *mut job,
    mut pid: *mut pid_t,
    mut tty: *mut ::core::ffi::c_char,
    mut ttylen: size_t,
) -> ::core::ffi::c_int {
    let mut fd: ::core::ffi::c_int = (*job).fd;
    log_debug(
        b"transfer job %p: %s\0" as *const u8 as *const ::core::ffi::c_char,
        job,
        (*job).cmd,
    );
    if !pid.is_null() {
        *pid = (*job).pid;
    }
    if !tty.is_null() {
        strlcpy(tty, &raw mut (*job).tty as *mut ::core::ffi::c_char, ttylen);
    }
    if !(*job).entry.le_next.is_null() {
        (*(*job).entry.le_next).entry.le_prev = (*job).entry.le_prev;
    }
    *(*job).entry.le_prev = (*job).entry.le_next;
    free((*job).cmd as *mut ::core::ffi::c_void);
    if (*job).freecb.is_some() && !(*job).data.is_null() {
        (*job).freecb.expect("non-null function pointer")((*job).data);
    }
    if !(*job).event.is_null() {
        bufferevent_free((*job).event);
    }
    free(job as *mut ::core::ffi::c_void);
    return fd;
}
#[no_mangle]
pub unsafe extern "C" fn job_free(mut job: *mut job) {
    log_debug(
        b"free job %p: %s\0" as *const u8 as *const ::core::ffi::c_char,
        job,
        (*job).cmd,
    );
    if !(*job).entry.le_next.is_null() {
        (*(*job).entry.le_next).entry.le_prev = (*job).entry.le_prev;
    }
    *(*job).entry.le_prev = (*job).entry.le_next;
    free((*job).cmd as *mut ::core::ffi::c_void);
    if (*job).freecb.is_some() && !(*job).data.is_null() {
        (*job).freecb.expect("non-null function pointer")((*job).data);
    }
    if (*job).pid != -(1 as ::core::ffi::c_int) {
        kill((*job).pid as __pid_t, SIGTERM);
    }
    if !(*job).event.is_null() {
        bufferevent_free((*job).event);
    }
    if (*job).fd != -(1 as ::core::ffi::c_int) {
        close((*job).fd);
    }
    free(job as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn job_resize(mut job: *mut job, mut sx: u_int, mut sy: u_int) {
    let mut ws: winsize = winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    if (*job).fd == -(1 as ::core::ffi::c_int) || !(*job).flags & JOB_PTY != 0 {
        return;
    }
    log_debug(
        b"resize job %p: %ux%u\0" as *const u8 as *const ::core::ffi::c_char,
        job,
        sx,
        sy,
    );
    memset(
        &raw mut ws as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<winsize>() as size_t,
    );
    ws.ws_col = sx as ::core::ffi::c_ushort;
    ws.ws_row = sy as ::core::ffi::c_ushort;
    if ioctl((*job).fd, TIOCSWINSZ as ::core::ffi::c_ulong, &raw mut ws)
        == -(1 as ::core::ffi::c_int)
    {
        fatal(b"ioctl failed\0" as *const u8 as *const ::core::ffi::c_char);
    }
}
unsafe extern "C" fn job_read_callback(
    mut bufev: *mut bufferevent,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut job: *mut job = data as *mut job;
    if (*job).updatecb.is_some() {
        (*job).updatecb.expect("non-null function pointer")(job);
    }
}
unsafe extern "C" fn job_write_callback(
    mut bufev: *mut bufferevent,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut job: *mut job = data as *mut job;
    let mut len: size_t = evbuffer_get_length(bufferevent_get_output((*job).event));
    log_debug(
        b"job write %p: %s, pid %ld, output left %zu\0" as *const u8 as *const ::core::ffi::c_char,
        job,
        (*job).cmd,
        (*job).pid as ::core::ffi::c_long,
        len,
    );
    if len == 0 as size_t && !(*job).flags & JOB_KEEPWRITE != 0 {
        shutdown((*job).fd, SHUT_WR as ::core::ffi::c_int);
        bufferevent_disable((*job).event, EV_WRITE as ::core::ffi::c_short);
    }
}
unsafe extern "C" fn job_error_callback(
    mut bufev: *mut bufferevent,
    mut events: ::core::ffi::c_short,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut job: *mut job = data as *mut job;
    log_debug(
        b"job error %p: %s, pid %ld\0" as *const u8 as *const ::core::ffi::c_char,
        job,
        (*job).cmd,
        (*job).pid as ::core::ffi::c_long,
    );
    if (*job).state as ::core::ffi::c_uint == JOB_DEAD as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*job).completecb.is_some() {
            (*job).completecb.expect("non-null function pointer")(job);
        }
        job_free(job);
    } else {
        bufferevent_disable((*job).event, EV_READ as ::core::ffi::c_short);
        (*job).state = JOB_CLOSED;
    };
}
#[no_mangle]
pub unsafe extern "C" fn job_check_died(mut pid: pid_t, mut status: ::core::ffi::c_int) {
    let mut job: *mut job = ::core::ptr::null_mut::<job>();
    job = all_jobs.lh_first;
    while !job.is_null() {
        if pid == (*job).pid {
            break;
        }
        job = (*job).entry.le_next;
    }
    if job.is_null() {
        return;
    }
    if status & 0xff as ::core::ffi::c_int == 0x7f as ::core::ffi::c_int {
        if (status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int == SIGTTIN
            || (status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int == SIGTTOU
        {
            return;
        }
        killpg((*job).pid as __pid_t, SIGCONT);
        return;
    }
    log_debug(
        b"job died %p: %s, pid %ld\0" as *const u8 as *const ::core::ffi::c_char,
        job,
        (*job).cmd,
        (*job).pid as ::core::ffi::c_long,
    );
    (*job).status = status;
    if (*job).state as ::core::ffi::c_uint
        == JOB_CLOSED as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*job).completecb.is_some() {
            (*job).completecb.expect("non-null function pointer")(job);
        }
        job_free(job);
    } else {
        (*job).pid = -(1 as ::core::ffi::c_int) as pid_t;
        (*job).state = JOB_DEAD;
    };
}
#[no_mangle]
pub unsafe extern "C" fn job_get_status(mut job: *mut job) -> ::core::ffi::c_int {
    return (*job).status;
}
#[no_mangle]
pub unsafe extern "C" fn job_get_data(mut job: *mut job) -> *mut ::core::ffi::c_void {
    return (*job).data;
}
#[no_mangle]
pub unsafe extern "C" fn job_get_event(mut job: *mut job) -> *mut bufferevent {
    return (*job).event;
}
#[no_mangle]
pub unsafe extern "C" fn job_kill_all() {
    let mut job: *mut job = ::core::ptr::null_mut::<job>();
    job = all_jobs.lh_first;
    while !job.is_null() {
        if (*job).pid != -(1 as ::core::ffi::c_int) {
            kill((*job).pid as __pid_t, SIGTERM);
        }
        job = (*job).entry.le_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn job_still_running() -> ::core::ffi::c_int {
    let mut job: *mut job = ::core::ptr::null_mut::<job>();
    job = all_jobs.lh_first;
    while !job.is_null() {
        if !(*job).flags & JOB_NOWAIT != 0
            && (*job).state as ::core::ffi::c_uint
                == JOB_RUNNING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return 1 as ::core::ffi::c_int;
        }
        job = (*job).entry.le_next;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn job_print_summary(
    mut item: *mut cmdq_item,
    mut blank: ::core::ffi::c_int,
) {
    let mut job: *mut job = ::core::ptr::null_mut::<job>();
    let mut n: u_int = 0 as u_int;
    job = all_jobs.lh_first;
    while !job.is_null() {
        if blank != 0 {
            cmdq_print(
                item,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                b"\0" as *const u8 as *const ::core::ffi::c_char,
            );
            blank = 0 as ::core::ffi::c_int;
        }
        cmdq_print(
            item,
            b"Job %u: %s [fd=%d, pid=%ld, status=%d]\0" as *const u8 as *const ::core::ffi::c_char,
            n,
            (*job).cmd,
            (*job).fd,
            (*job).pid as ::core::ffi::c_long,
            (*job).status,
        );
        n = n.wrapping_add(1);
        job = (*job).entry.le_next;
    }
}
