use crate::src::arguments::{
    args_count, args_get, args_has, args_make_commands, args_make_commands_free,
    args_make_commands_prepare, args_string,
};
use crate::src::cmd::find::cmd_find_from_nothing;
use crate::src::cmd::parse::cmd_parse_error_uppercase_first;
use crate::src::cmd::queue::{
    cmdq_append, cmdq_continue, cmdq_error, cmdq_get_client, cmdq_get_command, cmdq_get_state,
    cmdq_get_target, cmdq_get_target_client, cmdq_insert_after, cmdq_print,
};
use crate::src::cmd::{cmd_get_args, cmd_list_free};
use crate::src::ffi::libc::{__ctype_toupper_loc, strtod};
use crate::src::format::{
    format_add, format_create_from_target, format_expand_cstring, format_free,
};
use crate::src::job::{job_get_event, job_get_status, job_run};
use crate::src::reactor::{
    evbuffer_get_length, evbuffer_pullup, evbuffer_readln, event_active, event_add, event_del,
    event_set,
};
use crate::src::server_client::{server_client_get_cwd, server_client_unref};
use crate::src::session::{session_add_ref, session_remove_ref};
use crate::src::shared::abi::ssize_t;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::args_command_state;
use crate::src::shared::arguments::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::command::CMD_FIND_CANFAIL;
use crate::src::shared::command::*;
use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item, cmdq_state,
};
use crate::src::shared::environment::environ;
use crate::src::shared::event::*;
use crate::src::shared::event::{EVBUFFER_EOL_LF, EV_TIMEOUT};
use crate::src::shared::format::format_tree;
use crate::src::shared::job::job;
use crate::src::shared::job::{JOB_NOWAIT, JOB_SHOWSTDERR};
use crate::src::shared::pane::window_pane;
use crate::src::shared::session::session;
use crate::src::shared::window::{window, window_mode_entry, winlink};
use crate::src::status::status_message_set;
use crate::src::window::{window_pane_find_by_id, window_pane_set_mode};
use crate::src::window_copy::{window_copy_add, window_view_mode};
use crate::src::xmalloc::xsnprintf;
use std::ffi::{CStr, CString};

#[repr(C)]
pub struct cmd_run_shell_data {
    pub client: *mut client,
    pub cmd: Option<CString>,
    pub state: *mut args_command_state,
    pub cwd: CString,
    pub item: *mut cmdq_item,
    pub s: *mut session,
    pub wp_id: ::core::ffi::c_int,
    pub timer: event,
    pub flags: ::core::ffi::c_int,
}
#[inline]
unsafe extern "C" fn toupper(mut __c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return if __c >= -(128 as ::core::ffi::c_int) && __c < 256 as ::core::ffi::c_int {
        *(*__ctype_toupper_loc()).offset(__c as isize) as ::core::ffi::c_int
    } else {
        __c
    };
}

#[no_mangle]
pub static mut cmd_run_shell_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"run-shell\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"run\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"bd:Ct:Es:c:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
            cb: Some(
                cmd_run_shell_args_parse
            ),
        },
        usage: b"[-bCE] [-c start-directory] [-d delay] [-t target-pane] [shell-command [argument ...]]\0"
            as *const u8 as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: CMD_FIND_CANFAIL,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_run_shell_exec
                as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
fn cmd_run_shell_args_parse(args: &mut args, _idx: u_int) -> args_parse_type {
    if unsafe { args_has(args as *mut args, 'C' as i32 as u_char) } != 0 {
        return ARGS_PARSE_COMMANDS_OR_STRING;
    }
    return ARGS_PARSE_STRING;
}
unsafe fn cmd_run_shell_print(
    mut job: *mut job,
    mut cdata: *mut cmd_run_shell_data,
    mut msg: *const ::core::ffi::c_char,
) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    if (*cdata).wp_id != -(1 as ::core::ffi::c_int) {
        wp = window_pane_find_by_id((*cdata).wp_id as u_int);
    }
    if wp.is_null() {
        if !(*cdata).item.is_null() {
            cmdq_print(
                (*cdata).item,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                msg,
            );
            return;
        }
        if !(*cdata).client.is_null() && !(*(*cdata).client).session.is_null() {
            wp = (*(*(*(*(*cdata).client).session).curw).window).active;
        }
        if wp.is_null()
            && cmd_find_from_nothing(&raw mut fs, 0 as ::core::ffi::c_int)
                == 0 as ::core::ffi::c_int
        {
            wp = fs.wp;
        }
        if wp.is_null() {
            return;
        }
    }
    wme = (*wp).modes.active;
    if wme.is_null() || (*wme).mode != &raw const window_view_mode {
        window_pane_set_mode(
            wp,
            ::core::ptr::null_mut::<window_pane>(),
            &raw const window_view_mode,
            ::core::ptr::null_mut::<cmdq_item>(),
            ::core::ptr::null_mut::<cmd_find_state>(),
            ::core::ptr::null_mut::<args>(),
        );
    }
    window_copy_add(
        wp,
        1 as ::core::ffi::c_int,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        msg,
    );
}

fn cmd_run_shell_status_message(cmd: &CStr, suffix: &[u8], code: ::core::ffi::c_int) -> CString {
    let cmd = cmd.to_bytes();
    let code = code.to_string();
    let mut message = Vec::with_capacity(1 + cmd.len() + suffix.len() + code.len());
    message.push(b'\'');
    message.extend_from_slice(cmd);
    message.extend_from_slice(suffix);
    message.extend_from_slice(code.as_bytes());
    CString::new(message).expect("run-shell command and status text contain no NUL")
}

unsafe extern "C" fn cmd_run_shell_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut cdata: *mut cmd_run_shell_data = ::core::ptr::null_mut::<cmd_run_shell_data>();
    let mut c: *mut client = cmdq_get_client(item);
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut s: *mut session = (*target).s;
    let mut wp: *mut window_pane = (*target).wp;
    let mut delay: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cmd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut d: ::core::ffi::c_double = 0.;
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut key: [::core::ffi::c_char; 16] = [0; 16];
    let mut i: u_int = 0;
    let mut wait: ::core::ffi::c_int =
        (args_has(args, 'b' as i32 as u_char) == 0) as ::core::ffi::c_int;
    delay = args_get(args, 'd' as i32 as u_char);
    if !delay.is_null() {
        d = strtod(delay, &raw mut end);
        if *end as ::core::ffi::c_int != '\0' as i32 {
            cmdq_error(
                item,
                b"invalid delay time: %s\0" as *const u8 as *const ::core::ffi::c_char,
                delay,
            );
            return CMD_RETURN_ERROR;
        }
    } else if args_count(args) == 0 as u_int {
        return CMD_RETURN_NORMAL;
    }
    let cwd = if args_has(args, 'c' as i32 as u_char) != 0 {
        args_get(args, 'c' as i32 as u_char)
    } else {
        server_client_get_cwd(c, s)
    };
    cdata = Box::into_raw(Box::new(cmd_run_shell_data {
        client: ::core::ptr::null_mut(),
        cmd: None,
        state: ::core::ptr::null_mut(),
        cwd: CStr::from_ptr(cwd).to_owned(),
        item: ::core::ptr::null_mut(),
        s: ::core::ptr::null_mut(),
        wp_id: 0,
        timer: Default::default(),
        flags: 0,
    }));
    if args_has(args, 'C' as i32 as u_char) == 0 {
        cmd = args_string(args, 0 as u_int);
        if !cmd.is_null() {
            ft = format_create_from_target(item);
            i = 1 as u_int;
            while i < args_count(args) {
                xsnprintf(
                    &raw mut key as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 16]>() as size_t,
                    b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                    i,
                );
                format_add(
                    ft,
                    &raw mut key as *mut ::core::ffi::c_char,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    args_string(args, i),
                );
                i = i.wrapping_add(1);
            }
            (*cdata).cmd = Some(format_expand_cstring(ft, cmd));
            format_free(ft);
        }
    } else {
        (*cdata).state = args_make_commands_prepare(
            self_0,
            item,
            0 as u_int,
            ::core::ptr::null::<::core::ffi::c_char>(),
            wait,
            1 as ::core::ffi::c_int,
        );
    }
    if args_has(args, 't' as i32 as u_char) != 0 && !wp.is_null() {
        (*cdata).wp_id = (*wp).id as ::core::ffi::c_int;
    } else {
        (*cdata).wp_id = -(1 as ::core::ffi::c_int);
    }
    if wait != 0 {
        (*cdata).client = c;
        (*cdata).item = item;
    } else {
        (*cdata).client = tc;
        (*cdata).flags |= JOB_NOWAIT;
    }
    if !(*cdata).client.is_null() {
        (*(*cdata).client).references += 1;
    }
    if args_has(args, 'E' as i32 as u_char) != 0 {
        (*cdata).flags |= JOB_SHOWSTDERR;
    }
    (*cdata).s = s;
    if !s.is_null() {
        session_add_ref(
            s,
            b"cmd_run_shell_exec\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    event_set(
        &raw mut (*cdata).timer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        Some(
            cmd_run_shell_timer
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        cdata as *mut ::core::ffi::c_void,
    );
    if !delay.is_null() {
        tv.tv_usec = 0 as __suseconds_t;
        tv.tv_sec = tv.tv_usec as __time_t;
        tv.tv_sec = d as time_t as __time_t;
        tv.tv_usec = ((d - tv.tv_sec as ::core::ffi::c_double)
            * 1000000 as ::core::ffi::c_uint as ::core::ffi::c_double)
            as __suseconds_t;
        event_add(&raw mut (*cdata).timer, &raw mut tv);
    } else {
        event_active(
            &raw mut (*cdata).timer,
            EV_TIMEOUT,
            1 as ::core::ffi::c_short,
        );
    }
    if wait == 0 {
        return CMD_RETURN_NORMAL;
    }
    return CMD_RETURN_WAIT;
}
unsafe extern "C" fn cmd_run_shell_timer(
    _fd: ::core::ffi::c_int,
    _events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut cdata: *mut cmd_run_shell_data = arg as *mut cmd_run_shell_data;
    let mut c: *mut client = (*cdata).client;
    let cmd = (*cdata).cmd.as_deref();
    let mut item: *mut cmdq_item = (*cdata).item;
    let mut new_item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    if (*cdata).state.is_null() {
        if cmd.is_none() {
            if !(*cdata).item.is_null() {
                cmdq_continue((*cdata).item);
            }
            cmd_run_shell_free(cdata);
            return;
        }
        if job_run(
            cmd,
            &Vec::new(),
            ::core::ptr::null_mut::<environ>(),
            (*cdata).s,
            Some((*cdata).cwd.as_c_str()),
            None,
            Some(Box::new(move |job| unsafe {
                cmd_run_shell_callback(job, cdata)
            })),
            Some(Box::new(move || unsafe {
                cmd_run_shell_free(cdata)
            })),
            cdata as *mut ::core::ffi::c_void,
            (*cdata).flags,
            -(1 as ::core::ffi::c_int),
            -(1 as ::core::ffi::c_int),
        )
        .is_null()
        {
            if (*cdata).item.is_null() {
                status_message_set(
                    c,
                    -(1 as ::core::ffi::c_int),
                    1 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    b"failed to run command: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    cmd.unwrap().as_ptr(),
                );
            } else {
                cmdq_error(
                    (*cdata).item,
                    b"failed to run command: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    cmd.unwrap().as_ptr(),
                );
                cmdq_continue((*cdata).item);
            }
            cmd_run_shell_free(cdata);
        }
        return;
    }
    match args_make_commands((*cdata).state, &Vec::new()) {
        Err(mut error) => {
            if (*cdata).item.is_null() {
                cmd_parse_error_uppercase_first(&mut error);
            }
            let error_ptr = error
                .as_ref()
                .map_or(::core::ptr::null(), |cause| cause.as_ptr());
            if (*cdata).item.is_null() {
                status_message_set(
                    c,
                    -(1 as ::core::ffi::c_int),
                    1 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    error_ptr,
                );
            } else {
                cmdq_error(
                    (*cdata).item,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    error_ptr,
                );
            }
        }
        Ok(commands) if item.is_null() => {
            new_item = cmdq_get_command(commands, ::core::ptr::null_mut::<cmdq_state>());
            cmdq_append(c, new_item);
            cmd_list_free(commands);
        }
        Ok(commands) => {
            new_item = cmdq_get_command(commands, cmdq_get_state(item));
            cmdq_insert_after(item, new_item);
            cmd_list_free(commands);
        }
    }
    if !(*cdata).item.is_null() {
        cmdq_continue((*cdata).item);
    }
    cmd_run_shell_free(cdata);
}
unsafe fn cmd_run_shell_callback(mut job: *mut job, mut cdata: *mut cmd_run_shell_data) {
    let mut event: *mut bufferevent = job_get_event(job);
    let mut item: *mut cmdq_item = (*cdata).item;
    let cmd = (*cdata)
        .cmd
        .as_ref()
        .expect("run-shell callback requires a command")
        .as_c_str();
    let mut msg: Option<CString> = None;
    let mut size: size_t = 0;
    let mut retcode: ::core::ffi::c_int = 0;
    let mut status: ::core::ffi::c_int = 0;
    loop {
        let Some(line) = evbuffer_readln(
            (*event).input,
            ::core::ptr::null_mut::<size_t>(),
            EVBUFFER_EOL_LF,
        ) else {
            break;
        };
        cmd_run_shell_print(job, cdata, line.as_ptr().cast());
    }
    size = evbuffer_get_length(&*((*event).input));
    if size != 0 as size_t {
        let input = evbuffer_pullup((*event).input, -(1 as ::core::ffi::c_int) as ssize_t);
        let mut partial_line = ::core::slice::from_raw_parts(input as *const u8, size).to_vec();
        partial_line.push(0);
        cmd_run_shell_print(job, cdata, partial_line.as_ptr().cast());
    }
    status = job_get_status(job);
    if status & 0x7f as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        retcode = (status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int;
        if retcode != 0 as ::core::ffi::c_int {
            msg = Some(cmd_run_shell_status_message(cmd, b"' returned ", retcode));
        }
    } else if ((status & 0x7f as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
        as ::core::ffi::c_schar as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int
        > 0 as ::core::ffi::c_int
    {
        retcode = status & 0x7f as ::core::ffi::c_int;
        msg = Some(cmd_run_shell_status_message(
            cmd,
            b"' terminated by signal ",
            retcode,
        ));
        retcode += 128 as ::core::ffi::c_int;
    } else {
        retcode = 0 as ::core::ffi::c_int;
    }
    if let Some(msg) = msg.as_ref() {
        cmd_run_shell_print(job, cdata, msg.as_ptr());
    }
    if !item.is_null() {
        if !cmdq_get_client(item).is_null() && (*cmdq_get_client(item)).session.is_null() {
            (*cmdq_get_client(item)).retval = retcode;
        }
        cmdq_continue(item);
    }
}
unsafe fn cmd_run_shell_free(mut data: *mut cmd_run_shell_data) {
    let mut cdata = Box::from_raw(data);
    event_del(&raw mut (*cdata).timer);
    if !(*cdata).s.is_null() {
        session_remove_ref(
            (*cdata).s,
            b"cmd_run_shell_free\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if !(*cdata).client.is_null() {
        server_client_unref((*cdata).client);
    }
    if !(*cdata).state.is_null() {
        args_make_commands_free((*cdata).state);
    }
}
