pub use crate::src::shared::arguments::{args, args_parse, args_parse_cb};
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds,
};
pub use crate::src::shared::control::{control_state};
pub use crate::src::shared::format::{format_job_tree, format_tree};
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
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
    O_WRONLY, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO, _PATH_BSHELL, _PATH_DEVNULL,
};
pub use crate::src::shared::format::{FORMAT_NONE};
pub use crate::src::shared::socket::{
    __socket_type, AF_UNIX, PF_LOCAL, PF_UNIX, PF_UNSPEC, SOCK_CLOEXEC, SOCK_DCCP, SOCK_DGRAM,
    SOCK_NONBLOCK, SOCK_PACKET, SOCK_RAW, SOCK_RDM, SOCK_SEQPACKET, SOCK_STREAM,
};
pub use crate::src::shared::signal::{__sigset_t, sigset_t, SIG_BLOCK, SIG_SETMASK};
pub use crate::src::shared::abi::{ssize_t};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::event::{EV_READ, EV_WRITE};
pub use crate::src::shared::command::{CMD_AFTERHOOK};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::arguments::*;
use crate::src::shared::terminal::*;
use crate::src::shared::event::*;
use crate::src::shared::display::*;
use crate::src::shared::layout::*;
use crate::src::shared::message::*;
use crate::src::shared::abi::*;
use crate::src::shared::colour::*;
use crate::src::shared::command::*;
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
use crate::src::shared::style::*;
extern "C" {

    fn socketpair(
        __domain: ::core::ffi::c_int,
        __type: ::core::ffi::c_int,
        __protocol: ::core::ffi::c_int,
        __fds: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn __errno_location() -> *mut ::core::ffi::c_int;
    fn open(
        __file: *const ::core::ffi::c_char,
        __oflag: ::core::ffi::c_int,
        ...
    ) -> ::core::ffi::c_int;
    fn sigfillset(__set: *mut sigset_t) -> ::core::ffi::c_int;
    fn sigprocmask(
        __how: ::core::ffi::c_int,
        __set: *const sigset_t,
        __oset: *mut sigset_t,
    ) -> ::core::ffi::c_int;
    fn close(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn closefrom(__lowfd: ::core::ffi::c_int);
    fn dup2(__fd: ::core::ffi::c_int, __fd2: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn execl(
        __path: *const ::core::ffi::c_char,
        __arg: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn _exit(__status: ::core::ffi::c_int) -> !;
    fn setpgid(__pid: __pid_t, __pgid: __pid_t) -> ::core::ffi::c_int;
    fn fork() -> __pid_t;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strerror(__errnum: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn evbuffer_get_length(buf: *const evbuffer) -> size_t;
    fn evbuffer_drain(buf: *mut evbuffer, len: size_t) -> ::core::ffi::c_int;
    fn evbuffer_pullup(buf: *mut evbuffer, size: ssize_t) -> *mut ::core::ffi::c_uchar;
    fn bufferevent_free(bufev: *mut bufferevent);
    fn bufferevent_write(
        bufev: *mut bufferevent,
        data: *const ::core::ffi::c_void,
        size: size_t,
    ) -> ::core::ffi::c_int;
    fn bufferevent_enable(
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
    fn setblocking(_: ::core::ffi::c_int, _: ::core::ffi::c_int);
    fn proc_clear_signals(_: *mut tmuxproc, _: ::core::ffi::c_int);
    fn format_create(
        _: *mut client,
        _: *mut cmdq_item,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    ) -> *mut format_tree;
    fn format_free(_: *mut format_tree);
    fn format_expand_time(
        _: *mut format_tree,
        _: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn format_defaults(
        _: *mut format_tree,
        _: *mut client,
        _: *mut session,
        _: *mut winlink,
        _: *mut window_pane,
    );
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_count(_: *mut args) -> u_int;
    fn args_string(_: *mut args, _: u_int) -> *const ::core::ffi::c_char;
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmdq_get_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_target_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_target(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    static mut server_proc: *mut tmuxproc;
    fn server_destroy_pane(_: *mut window_pane, _: ::core::ffi::c_int);
    fn window_pane_destroy_ready(_: *mut window_pane) -> ::core::ffi::c_int;
    fn window_pane_exited(_: *mut window_pane) -> ::core::ffi::c_int;
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn fatalx(_: *const ::core::ffi::c_char, ...) -> !;
}

pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;
pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;

#[no_mangle]
pub static mut cmd_pipe_pane_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"pipe-pane\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"pipep\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"IOot:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-IOo] [-t target-pane] [shell-command]\0" as *const u8
            as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK,
        exec: Some(
            cmd_pipe_pane_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_pipe_pane_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut wp: *mut window_pane = (*target).wp;
    let mut s: *mut session = (*target).s;
    let mut wl: *mut winlink = (*target).wl;
    let mut wpo: *mut window_pane_offset = &raw mut (*wp).pipe_offset;
    let mut cmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut old_fd: ::core::ffi::c_int = 0;
    let mut pipe_fd: [::core::ffi::c_int; 2] = [0; 2];
    let mut null_fd: ::core::ffi::c_int = 0;
    let mut in_0: ::core::ffi::c_int = 0;
    let mut out: ::core::ffi::c_int = 0;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut set: sigset_t = __sigset_t { __val: [0; 16] };
    let mut oldset: sigset_t = __sigset_t { __val: [0; 16] };
    if window_pane_exited(wp) != 0 {
        cmdq_error(
            item,
            b"target pane has exited\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    old_fd = (*wp).pipe_fd;
    if (*wp).pipe_fd != -(1 as ::core::ffi::c_int) {
        bufferevent_free((*wp).pipe_event);
        close((*wp).pipe_fd);
        (*wp).pipe_fd = -(1 as ::core::ffi::c_int);
        if window_pane_destroy_ready(wp) != 0 {
            server_destroy_pane(wp, 1 as ::core::ffi::c_int);
            return CMD_RETURN_NORMAL;
        }
    }
    if args_count(args) == 0 as u_int
        || *args_string(args, 0 as u_int) as ::core::ffi::c_int == '\0' as i32
    {
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'o' as i32 as u_char) != 0 && old_fd != -(1 as ::core::ffi::c_int) {
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'I' as i32 as u_char) != 0 {
        in_0 = 1 as ::core::ffi::c_int;
        out = args_has(args, 'O' as i32 as u_char);
    } else {
        in_0 = 0 as ::core::ffi::c_int;
        out = 1 as ::core::ffi::c_int;
    }
    if socketpair(
        AF_UNIX,
        SOCK_STREAM as ::core::ffi::c_int,
        PF_UNSPEC,
        &raw mut pipe_fd as *mut ::core::ffi::c_int,
    ) != 0 as ::core::ffi::c_int
    {
        cmdq_error(
            item,
            b"socketpair error: %s\0" as *const u8 as *const ::core::ffi::c_char,
            strerror(*__errno_location()),
        );
        return CMD_RETURN_ERROR;
    }
    ft = format_create(
        cmdq_get_client(item),
        item,
        FORMAT_NONE,
        0 as ::core::ffi::c_int,
    );
    format_defaults(ft, tc, s, wl, wp);
    cmd = format_expand_time(ft, args_string(args, 0 as u_int));
    format_free(ft);
    sigfillset(&raw mut set);
    sigprocmask(SIG_BLOCK, &raw mut set, &raw mut oldset);
    (*wp).pipe_pid = fork() as pid_t;
    match (*wp).pipe_pid {
        -1 => {
            sigprocmask(
                SIG_SETMASK,
                &raw mut oldset,
                ::core::ptr::null_mut::<sigset_t>(),
            );
            cmdq_error(
                item,
                b"fork error: %s\0" as *const u8 as *const ::core::ffi::c_char,
                strerror(*__errno_location()),
            );
            close(pipe_fd[0 as ::core::ffi::c_int as usize]);
            close(pipe_fd[1 as ::core::ffi::c_int as usize]);
            free(cmd as *mut ::core::ffi::c_void);
            return CMD_RETURN_ERROR;
        }
        0 => {
            proc_clear_signals(server_proc, 1 as ::core::ffi::c_int);
            sigprocmask(
                SIG_SETMASK,
                &raw mut oldset,
                ::core::ptr::null_mut::<sigset_t>(),
            );
            close(pipe_fd[0 as ::core::ffi::c_int as usize]);
            if setpgid(0 as __pid_t, 0 as __pid_t) == -(1 as ::core::ffi::c_int) {
                _exit(1 as ::core::ffi::c_int);
            }
            null_fd = open(_PATH_DEVNULL.as_ptr(), O_WRONLY);
            if out != 0 {
                if dup2(pipe_fd[1 as ::core::ffi::c_int as usize], STDIN_FILENO)
                    == -(1 as ::core::ffi::c_int)
                {
                    _exit(1 as ::core::ffi::c_int);
                }
            } else if dup2(null_fd, STDIN_FILENO) == -(1 as ::core::ffi::c_int) {
                _exit(1 as ::core::ffi::c_int);
            }
            if in_0 != 0 {
                if dup2(pipe_fd[1 as ::core::ffi::c_int as usize], STDOUT_FILENO)
                    == -(1 as ::core::ffi::c_int)
                {
                    _exit(1 as ::core::ffi::c_int);
                }
                if pipe_fd[1 as ::core::ffi::c_int as usize] != STDOUT_FILENO {
                    close(pipe_fd[1 as ::core::ffi::c_int as usize]);
                }
            } else if dup2(null_fd, STDOUT_FILENO) == -(1 as ::core::ffi::c_int) {
                _exit(1 as ::core::ffi::c_int);
            }
            if dup2(null_fd, STDERR_FILENO) == -(1 as ::core::ffi::c_int) {
                _exit(1 as ::core::ffi::c_int);
            }
            closefrom(STDERR_FILENO + 1 as ::core::ffi::c_int);
            execl(
                _PATH_BSHELL.as_ptr(),
                b"sh\0" as *const u8 as *const ::core::ffi::c_char,
                b"-c\0" as *const u8 as *const ::core::ffi::c_char,
                cmd,
                NULL as *mut ::core::ffi::c_char,
            );
            _exit(1 as ::core::ffi::c_int);
        }
        _ => {
            sigprocmask(
                SIG_SETMASK,
                &raw mut oldset,
                ::core::ptr::null_mut::<sigset_t>(),
            );
            close(pipe_fd[1 as ::core::ffi::c_int as usize]);
            (*wp).pipe_fd = pipe_fd[0 as ::core::ffi::c_int as usize];
            memcpy(
                wpo as *mut ::core::ffi::c_void,
                &raw mut (*wp).offset as *const ::core::ffi::c_void,
                ::core::mem::size_of::<window_pane_offset>() as size_t,
            );
            setblocking((*wp).pipe_fd, 0 as ::core::ffi::c_int);
            (*wp).pipe_event = bufferevent_new(
                (*wp).pipe_fd,
                Some(
                    cmd_pipe_pane_read_callback
                        as unsafe extern "C" fn(*mut bufferevent, *mut ::core::ffi::c_void) -> (),
                ),
                Some(
                    cmd_pipe_pane_write_callback
                        as unsafe extern "C" fn(*mut bufferevent, *mut ::core::ffi::c_void) -> (),
                ),
                Some(
                    cmd_pipe_pane_error_callback
                        as unsafe extern "C" fn(
                            *mut bufferevent,
                            ::core::ffi::c_short,
                            *mut ::core::ffi::c_void,
                        ) -> (),
                ),
                wp as *mut ::core::ffi::c_void,
            );
            if (*wp).pipe_event.is_null() {
                fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
            }
            if out != 0 {
                bufferevent_enable((*wp).pipe_event, EV_WRITE as ::core::ffi::c_short);
            }
            if in_0 != 0 {
                bufferevent_enable((*wp).pipe_event, EV_READ as ::core::ffi::c_short);
            }
            free(cmd as *mut ::core::ffi::c_void);
            return CMD_RETURN_NORMAL;
        }
    };
}
unsafe extern "C" fn cmd_pipe_pane_read_callback(
    mut bufev: *mut bufferevent,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut wp: *mut window_pane = data as *mut window_pane;
    let mut evb: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut available: size_t = 0;
    if (*wp).pipe_event.is_null() {
        return;
    }
    evb = (*(*wp).pipe_event).input;
    available = evbuffer_get_length(evb);
    log_debug(
        b"%%%u pipe read %zu\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
        available,
    );
    bufferevent_write(
        (*wp).event,
        evbuffer_pullup(evb, -(1 as ::core::ffi::c_int) as ssize_t) as *const ::core::ffi::c_void,
        available,
    );
    evbuffer_drain(evb, available);
    if window_pane_destroy_ready(wp) != 0 {
        server_destroy_pane(wp, 1 as ::core::ffi::c_int);
    }
}
unsafe extern "C" fn cmd_pipe_pane_write_callback(
    mut bufev: *mut bufferevent,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut wp: *mut window_pane = data as *mut window_pane;
    log_debug(
        b"%%%u pipe empty\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
    );
    if window_pane_destroy_ready(wp) != 0 {
        server_destroy_pane(wp, 1 as ::core::ffi::c_int);
    }
}
unsafe extern "C" fn cmd_pipe_pane_error_callback(
    mut bufev: *mut bufferevent,
    mut what: ::core::ffi::c_short,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut wp: *mut window_pane = data as *mut window_pane;
    log_debug(
        b"%%%u pipe error\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
    );
    bufferevent_free((*wp).pipe_event);
    close((*wp).pipe_fd);
    (*wp).pipe_fd = -(1 as ::core::ffi::c_int);
    if window_pane_destroy_ready(wp) != 0 {
        server_destroy_pane(wp, 1 as ::core::ffi::c_int);
    }
}
