pub use crate::src::shared::arguments::{args_command_state};
pub use crate::src::shared::arguments::{args, args_parse, args_parse_cb};
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmdq_state,
    cmds,
};
pub use crate::src::shared::control::{control_state};
pub use crate::src::shared::format::{format_job_tree, format_tree};
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
pub use crate::src::shared::job::{job, job_complete_cb, job_free_cb, job_update_cb};
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
pub use crate::src::shared::process::{tmuxpeer};
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
pub use crate::src::shared::job::{JOB_NOWAIT, JOB_SHOWSTDERR};
pub use crate::src::shared::event::{
    EVBUFFER_EOL_ANY, EVBUFFER_EOL_CRLF, EVBUFFER_EOL_CRLF_STRICT, EVBUFFER_EOL_LF,
    EVBUFFER_EOL_NUL, EV_TIMEOUT, evbuffer_eol_style,
};
pub use crate::src::shared::abi::{__int32_t, ssize_t};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::command::{CMD_FIND_CANFAIL};
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

    fn __ctype_toupper_loc() -> *mut *const __int32_t;
    fn strtod(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_double;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn event_add(ev: *mut event, timeout: *const timeval) -> ::core::ffi::c_int;
    fn event_del(_: *mut event) -> ::core::ffi::c_int;
    fn event_active(ev: *mut event, res: ::core::ffi::c_int, ncalls: ::core::ffi::c_short);
    fn event_set(
        _: *mut event,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_short,
        _: Option<
            unsafe extern "C" fn(
                ::core::ffi::c_int,
                ::core::ffi::c_short,
                *mut ::core::ffi::c_void,
            ) -> (),
        >,
        _: *mut ::core::ffi::c_void,
    );
    fn evbuffer_get_length(buf: *const evbuffer) -> size_t;
    fn evbuffer_readln(
        buffer: *mut evbuffer,
        n_read_out: *mut size_t,
        eol_style: evbuffer_eol_style,
    ) -> *mut ::core::ffi::c_char;
    fn evbuffer_pullup(buf: *mut evbuffer, size: ssize_t) -> *mut ::core::ffi::c_uchar;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xmalloc(_: size_t) -> *mut ::core::ffi::c_void;
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xasprintf(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn xsnprintf(
        _: *mut ::core::ffi::c_char,
        _: size_t,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn format_free(_: *mut format_tree);
    fn format_add(
        _: *mut format_tree,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn format_expand(
        _: *mut format_tree,
        _: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn format_create_from_target(_: *mut cmdq_item) -> *mut format_tree;
    fn job_run(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
        _: *mut environ,
        _: *mut session,
        _: *const ::core::ffi::c_char,
        _: job_update_cb,
        _: job_complete_cb,
        _: job_free_cb,
        _: *mut ::core::ffi::c_void,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    ) -> *mut job;
    fn job_get_status(_: *mut job) -> ::core::ffi::c_int;
    fn job_get_data(_: *mut job) -> *mut ::core::ffi::c_void;
    fn job_get_event(_: *mut job) -> *mut bufferevent;
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_get(_: *mut args, _: u_char) -> *const ::core::ffi::c_char;
    fn args_count(_: *mut args) -> u_int;
    fn args_string(_: *mut args, _: u_int) -> *const ::core::ffi::c_char;
    fn args_make_commands_prepare(
        _: *mut cmd,
        _: *mut cmdq_item,
        _: u_int,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    ) -> *mut args_command_state;
    fn args_make_commands(
        _: *mut args_command_state,
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
        _: *mut *mut ::core::ffi::c_char,
    ) -> *mut cmd_list;
    fn args_make_commands_free(_: *mut args_command_state);
    fn cmd_find_from_nothing(_: *mut cmd_find_state, _: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmd_list_free(_: *mut cmd_list);
    fn cmdq_get_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_target_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_state(_: *mut cmdq_item) -> *mut cmdq_state;
    fn cmdq_get_target(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_get_command(_: *mut cmd_list, _: *mut cmdq_state) -> *mut cmdq_item;
    fn cmdq_insert_after(_: *mut cmdq_item, _: *mut cmdq_item) -> *mut cmdq_item;
    fn cmdq_append(_: *mut client, _: *mut cmdq_item) -> *mut cmdq_item;
    fn cmdq_continue(_: *mut cmdq_item);
    fn cmdq_print(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn server_client_unref(_: *mut client);
    fn server_client_get_cwd(_: *mut client, _: *mut session) -> *const ::core::ffi::c_char;
    fn status_message_set(
        _: *mut client,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn window_pane_find_by_id(_: u_int) -> *mut window_pane;
    fn window_pane_set_mode(
        _: *mut window_pane,
        _: *mut window_pane,
        _: *const window_mode,
        _: *mut cmdq_item,
        _: *mut cmd_find_state,
        _: *mut args,
    ) -> ::core::ffi::c_int;
    static window_view_mode: window_mode;
    fn window_copy_add(
        _: *mut window_pane,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn session_add_ref(_: *mut session, _: *const ::core::ffi::c_char);
    fn session_remove_ref(_: *mut session, _: *const ::core::ffi::c_char);
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_12 {
    pub offset: u_int,
    pub data: C2RustUnnamed_13,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_13 {
    pub attr: u_char,
    pub fg: u_char,
    pub bg: u_char,
    pub data: u_char,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_run_shell_data {
    pub client: *mut client,
    pub cmd: *mut ::core::ffi::c_char,
    pub state: *mut args_command_state,
    pub cwd: *mut ::core::ffi::c_char,
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
                    as unsafe extern "C" fn(
                        *mut args,
                        u_int,
                        *mut *mut ::core::ffi::c_char,
                    ) -> args_parse_type,
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
unsafe extern "C" fn cmd_run_shell_args_parse(
    mut args: *mut args,
    mut idx: u_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> args_parse_type {
    if args_has(args, 'C' as i32 as u_char) != 0 {
        return ARGS_PARSE_COMMANDS_OR_STRING;
    }
    return ARGS_PARSE_STRING;
}
unsafe extern "C" fn cmd_run_shell_print(mut job: *mut job, mut msg: *const ::core::ffi::c_char) {
    let mut cdata: *mut cmd_run_shell_data = job_get_data(job) as *mut cmd_run_shell_data;
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
    wme = (*wp).modes.tqh_first;
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
    cdata = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<cmd_run_shell_data>() as size_t,
    ) as *mut cmd_run_shell_data;
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
            (*cdata).cmd = format_expand(ft, cmd);
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
    if args_has(args, 'c' as i32 as u_char) != 0 {
        (*cdata).cwd = xstrdup(args_get(args, 'c' as i32 as u_char));
    } else {
        (*cdata).cwd = xstrdup(server_client_get_cwd(c, s));
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
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut cdata: *mut cmd_run_shell_data = arg as *mut cmd_run_shell_data;
    let mut c: *mut client = (*cdata).client;
    let mut cmd: *const ::core::ffi::c_char = (*cdata).cmd;
    let mut item: *mut cmdq_item = (*cdata).item;
    let mut new_item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut cmdlist: *mut cmd_list = ::core::ptr::null_mut::<cmd_list>();
    let mut error: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*cdata).state.is_null() {
        if cmd.is_null() {
            if !(*cdata).item.is_null() {
                cmdq_continue((*cdata).item);
            }
            cmd_run_shell_free(cdata as *mut ::core::ffi::c_void);
            return;
        }
        if job_run(
            cmd,
            0 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<environ>(),
            (*cdata).s,
            (*cdata).cwd,
            None,
            Some(cmd_run_shell_callback as unsafe extern "C" fn(*mut job) -> ()),
            Some(cmd_run_shell_free as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
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
                    cmd,
                );
            } else {
                cmdq_error(
                    (*cdata).item,
                    b"failed to run command: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    cmd,
                );
                cmdq_continue((*cdata).item);
            }
            cmd_run_shell_free(cdata as *mut ::core::ffi::c_void);
        }
        return;
    }
    cmdlist = args_make_commands(
        (*cdata).state,
        0 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        &raw mut error,
    );
    if cmdlist.is_null() {
        if (*cdata).item.is_null() {
            *error = ({
                let mut __res: ::core::ffi::c_int = 0;
                if ::core::mem::size_of::<u_char>() as usize > 1 as usize {
                    if 0 != 0 {
                        let mut __c: ::core::ffi::c_int = *error as u_char as ::core::ffi::c_int;
                        __res = (if __c < -(128 as ::core::ffi::c_int)
                            || __c > 255 as ::core::ffi::c_int
                        {
                            __c as __int32_t
                        } else {
                            *(*__ctype_toupper_loc()).offset(__c as isize)
                        }) as ::core::ffi::c_int;
                    } else {
                        __res = toupper(*error as u_char as ::core::ffi::c_int);
                    }
                } else {
                    __res = *(*__ctype_toupper_loc())
                        .offset(*error as u_char as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int;
                }
                __res
            }) as ::core::ffi::c_char;
            status_message_set(
                c,
                -(1 as ::core::ffi::c_int),
                1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                error,
            );
        } else {
            cmdq_error(
                (*cdata).item,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                error,
            );
        }
        free(error as *mut ::core::ffi::c_void);
    } else if item.is_null() {
        new_item = cmdq_get_command(cmdlist, ::core::ptr::null_mut::<cmdq_state>());
        cmdq_append(c, new_item);
        cmd_list_free(cmdlist);
    } else {
        new_item = cmdq_get_command(cmdlist, cmdq_get_state(item));
        cmdq_insert_after(item, new_item);
        cmd_list_free(cmdlist);
    }
    if !(*cdata).item.is_null() {
        cmdq_continue((*cdata).item);
    }
    cmd_run_shell_free(cdata as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn cmd_run_shell_callback(mut job: *mut job) {
    let mut cdata: *mut cmd_run_shell_data = job_get_data(job) as *mut cmd_run_shell_data;
    let mut event: *mut bufferevent = job_get_event(job);
    let mut item: *mut cmdq_item = (*cdata).item;
    let mut cmd: *mut ::core::ffi::c_char = (*cdata).cmd;
    let mut msg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut size: size_t = 0;
    let mut retcode: ::core::ffi::c_int = 0;
    let mut status: ::core::ffi::c_int = 0;
    loop {
        line = evbuffer_readln(
            (*event).input,
            ::core::ptr::null_mut::<size_t>(),
            EVBUFFER_EOL_LF,
        );
        if !line.is_null() {
            cmd_run_shell_print(job, line);
            free(line as *mut ::core::ffi::c_void);
        }
        if line.is_null() {
            break;
        }
    }
    size = evbuffer_get_length((*event).input);
    if size != 0 as size_t {
        line = xmalloc(size.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        memcpy(
            line as *mut ::core::ffi::c_void,
            evbuffer_pullup((*event).input, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            size,
        );
        *line.offset(size as isize) = '\0' as i32 as ::core::ffi::c_char;
        cmd_run_shell_print(job, line);
        free(line as *mut ::core::ffi::c_void);
    }
    status = job_get_status(job);
    if status & 0x7f as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        retcode = (status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int;
        if retcode != 0 as ::core::ffi::c_int {
            xasprintf(
                &raw mut msg,
                b"'%s' returned %d\0" as *const u8 as *const ::core::ffi::c_char,
                cmd,
                retcode,
            );
        }
    } else if ((status & 0x7f as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
        as ::core::ffi::c_schar as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int
        > 0 as ::core::ffi::c_int
    {
        retcode = status & 0x7f as ::core::ffi::c_int;
        xasprintf(
            &raw mut msg,
            b"'%s' terminated by signal %d\0" as *const u8 as *const ::core::ffi::c_char,
            cmd,
            retcode,
        );
        retcode += 128 as ::core::ffi::c_int;
    } else {
        retcode = 0 as ::core::ffi::c_int;
    }
    if !msg.is_null() {
        cmd_run_shell_print(job, msg);
    }
    free(msg as *mut ::core::ffi::c_void);
    if !item.is_null() {
        if !cmdq_get_client(item).is_null() && (*cmdq_get_client(item)).session.is_null() {
            (*cmdq_get_client(item)).retval = retcode;
        }
        cmdq_continue(item);
    }
}
unsafe extern "C" fn cmd_run_shell_free(mut data: *mut ::core::ffi::c_void) {
    let mut cdata: *mut cmd_run_shell_data = data as *mut cmd_run_shell_data;
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
    free((*cdata).cwd as *mut ::core::ffi::c_void);
    free((*cdata).cmd as *mut ::core::ffi::c_void);
    free(cdata as *mut ::core::ffi::c_void);
}
