pub use crate::src::shared::session::{sessions};
pub use crate::src::shared::client::{clients};
pub use crate::src::shared::command::{cmd_parse_input, cmd_parse_result};
pub use crate::src::shared::arguments::{args};
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::command::{
    cmd_find_state, cmd_list, cmdq_cb, cmdq_item, cmdq_list, cmdq_state, cmds,
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
pub use crate::src::shared::errno::ENOENT;
pub use crate::src::shared::variadic::{__builtin_va_list, __gnuc_va_list, __va_list_tag, va_list};
pub use crate::src::shared::stdio::{
    FILE, _IO_FILE, _IO_codecvt, _IO_lock_t, _IO_marker, _IO_wide_data,
};
pub use crate::src::shared::abi::{__off64_t, __off_t};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::tree::{RB_NEGINF};
pub use crate::src::shared::command::{CMD_PARSE_PARSEONLY, CMD_PARSE_QUIET};
pub use crate::src::shared::client::{CLIENT_CONTROL};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::command::*;
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

    fn __errno_location() -> *mut ::core::ffi::c_int;
    fn fclose(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn fopen(
        __filename: *const ::core::ffi::c_char,
        __modes: *const ::core::ffi::c_char,
    ) -> *mut FILE;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strerror(__errnum: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xreallocarray(_: *mut ::core::ffi::c_void, _: size_t, _: size_t)
        -> *mut ::core::ffi::c_void;
    fn xvasprintf(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::VaList,
    ) -> ::core::ffi::c_int;
    fn cmd_list_free(_: *mut cmd_list);
    fn cmd_parse_from_file(_: *mut FILE, _: *mut cmd_parse_input) -> *mut cmd_parse_result;
    fn cmd_parse_from_buffer(
        _: *const ::core::ffi::c_void,
        _: size_t,
        _: *mut cmd_parse_input,
    ) -> *mut cmd_parse_result;
    fn cmdq_new_state(
        _: *mut cmd_find_state,
        _: *mut key_event,
        _: ::core::ffi::c_int,
    ) -> *mut cmdq_state;
    fn cmdq_copy_state(_: *mut cmdq_state, _: *mut cmd_find_state) -> *mut cmdq_state;
    fn cmdq_free_state(_: *mut cmdq_state);
    fn cmdq_add_format(
        _: *mut cmdq_state,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn cmdq_get_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_state(_: *mut cmdq_item) -> *mut cmdq_state;
    fn cmdq_get_command(_: *mut cmd_list, _: *mut cmdq_state) -> *mut cmdq_item;
    fn cmdq_get_callback1(
        _: *const ::core::ffi::c_char,
        _: cmdq_cb,
        _: *mut ::core::ffi::c_void,
    ) -> *mut cmdq_item;
    fn cmdq_insert_after(_: *mut cmdq_item, _: *mut cmdq_item) -> *mut cmdq_item;
    fn cmdq_append(_: *mut client, _: *mut cmdq_item) -> *mut cmdq_item;
    fn cmdq_continue(_: *mut cmdq_item);
    fn cmdq_print(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    static mut clients: clients;
    fn prompt_load_history();
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
    fn control_notify_write(_: *mut client, _: *const ::core::ffi::c_char, ...);
    static mut sessions: sessions;
    fn sessions_RB_MINMAX(_: *mut sessions, _: ::core::ffi::c_int) -> *mut session;
    fn log_debug(_: *const ::core::ffi::c_char, ...);
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

#[no_mangle]
pub static mut cfg_client: *mut client = ::core::ptr::null::<client>() as *mut client;
#[no_mangle]
pub static mut cfg_finished: ::core::ffi::c_int = 0;
static mut cfg_causes: *mut *mut ::core::ffi::c_char =
    ::core::ptr::null::<*mut ::core::ffi::c_char>() as *mut *mut ::core::ffi::c_char;
static mut cfg_ncauses: u_int = 0;
static mut cfg_item: *mut cmdq_item = ::core::ptr::null::<cmdq_item>() as *mut cmdq_item;
#[no_mangle]
pub static mut cfg_quiet: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
#[no_mangle]
pub static mut cfg_files: *mut *mut ::core::ffi::c_char =
    ::core::ptr::null::<*mut ::core::ffi::c_char>() as *mut *mut ::core::ffi::c_char;
#[no_mangle]
pub static mut cfg_nfiles: u_int = 0;
unsafe extern "C" fn cfg_client_done(
    mut item: *mut cmdq_item,
    mut data: *mut ::core::ffi::c_void,
) -> cmd_retval {
    if cfg_finished == 0 {
        return CMD_RETURN_WAIT;
    }
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cfg_done(
    mut item: *mut cmdq_item,
    mut data: *mut ::core::ffi::c_void,
) -> cmd_retval {
    if cfg_finished != 0 {
        return CMD_RETURN_NORMAL;
    }
    cfg_finished = 1 as ::core::ffi::c_int;
    cfg_show_causes(::core::ptr::null_mut::<session>());
    if !cfg_item.is_null() {
        cmdq_continue(cfg_item);
    }
    prompt_load_history();
    return CMD_RETURN_NORMAL;
}
#[no_mangle]
pub unsafe extern "C" fn start_cfg() {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut i: u_int = 0;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    c = clients.tqh_first;
    cfg_client = c;
    if !c.is_null() {
        cfg_item = cmdq_get_callback1(
            b"cfg_client_done\0" as *const u8 as *const ::core::ffi::c_char,
            Some(
                cfg_client_done
                    as unsafe extern "C" fn(*mut cmdq_item, *mut ::core::ffi::c_void) -> cmd_retval,
            ),
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
        );
        cmdq_append(c, cfg_item);
    }
    if cfg_quiet != 0 {
        flags = CMD_PARSE_QUIET;
    }
    i = 0 as u_int;
    while i < cfg_nfiles {
        load_cfg(
            *cfg_files.offset(i as isize),
            c,
            ::core::ptr::null_mut::<cmdq_item>(),
            ::core::ptr::null_mut::<cmd_find_state>(),
            flags,
            ::core::ptr::null_mut::<*mut cmdq_item>(),
        );
        i = i.wrapping_add(1);
    }
    cmdq_append(
        ::core::ptr::null_mut::<client>(),
        cmdq_get_callback1(
            b"cfg_done\0" as *const u8 as *const ::core::ffi::c_char,
            Some(
                cfg_done
                    as unsafe extern "C" fn(*mut cmdq_item, *mut ::core::ffi::c_void) -> cmd_retval,
            ),
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
        ),
    );
}
#[no_mangle]
pub unsafe extern "C" fn load_cfg(
    mut path: *const ::core::ffi::c_char,
    mut c: *mut client,
    mut item: *mut cmdq_item,
    mut current: *mut cmd_find_state,
    mut flags: ::core::ffi::c_int,
    mut new_item: *mut *mut cmdq_item,
) -> ::core::ffi::c_int {
    let mut f: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut pi: cmd_parse_input = cmd_parse_input {
        flags: 0,
        file: ::core::ptr::null::<::core::ffi::c_char>(),
        line: 0,
        item: ::core::ptr::null_mut::<cmdq_item>(),
        c: ::core::ptr::null_mut::<client>(),
        fs: cmd_find_state {
            flags: 0,
            current: ::core::ptr::null_mut::<cmd_find_state>(),
            s: ::core::ptr::null_mut::<session>(),
            wl: ::core::ptr::null_mut::<winlink>(),
            w: ::core::ptr::null_mut::<window>(),
            wp: ::core::ptr::null_mut::<window_pane>(),
            idx: 0,
        },
    };
    let mut pr: *mut cmd_parse_result = ::core::ptr::null_mut::<cmd_parse_result>();
    let mut new_item0: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut state: *mut cmdq_state = ::core::ptr::null_mut::<cmdq_state>();
    if !new_item.is_null() {
        *new_item = ::core::ptr::null_mut::<cmdq_item>();
    }
    log_debug(
        b"loading %s\0" as *const u8 as *const ::core::ffi::c_char,
        path,
    );
    f = fopen(path, b"rb\0" as *const u8 as *const ::core::ffi::c_char) as *mut FILE;
    if f.is_null() {
        if *__errno_location() == ENOENT && flags & CMD_PARSE_QUIET != 0 {
            return 0 as ::core::ffi::c_int;
        }
        cfg_add_cause(
            b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            path,
            strerror(*__errno_location()),
        );
        return -(1 as ::core::ffi::c_int);
    }
    memset(
        &raw mut pi as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<cmd_parse_input>() as size_t,
    );
    pi.flags = flags;
    pi.file = path;
    pi.line = 1 as u_int;
    pi.item = item;
    pi.c = c;
    pr = cmd_parse_from_file(f, &raw mut pi);
    fclose(f);
    if (*pr).status as ::core::ffi::c_uint
        == CMD_PARSE_ERROR as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        cfg_add_cause(
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*pr).error,
        );
        free((*pr).error as *mut ::core::ffi::c_void);
        return -(1 as ::core::ffi::c_int);
    }
    if flags & CMD_PARSE_PARSEONLY != 0 {
        cmd_list_free((*pr).cmdlist);
        return 0 as ::core::ffi::c_int;
    }
    if !item.is_null() {
        state = cmdq_copy_state(cmdq_get_state(item), current);
    } else {
        state = cmdq_new_state(
            ::core::ptr::null_mut::<cmd_find_state>(),
            ::core::ptr::null_mut::<key_event>(),
            0 as ::core::ffi::c_int,
        );
    }
    cmdq_add_format(
        state,
        b"current_file\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        pi.file,
    );
    new_item0 = cmdq_get_command((*pr).cmdlist, state);
    if !item.is_null() {
        new_item0 = cmdq_insert_after(item, new_item0);
    } else {
        new_item0 = cmdq_append(::core::ptr::null_mut::<client>(), new_item0);
    }
    cmd_list_free((*pr).cmdlist);
    cmdq_free_state(state);
    if !new_item.is_null() {
        *new_item = new_item0;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn load_cfg_from_buffer(
    mut buf: *const ::core::ffi::c_void,
    mut len: size_t,
    mut path: *const ::core::ffi::c_char,
    mut c: *mut client,
    mut item: *mut cmdq_item,
    mut current: *mut cmd_find_state,
    mut flags: ::core::ffi::c_int,
    mut new_item: *mut *mut cmdq_item,
) -> ::core::ffi::c_int {
    let mut pi: cmd_parse_input = cmd_parse_input {
        flags: 0,
        file: ::core::ptr::null::<::core::ffi::c_char>(),
        line: 0,
        item: ::core::ptr::null_mut::<cmdq_item>(),
        c: ::core::ptr::null_mut::<client>(),
        fs: cmd_find_state {
            flags: 0,
            current: ::core::ptr::null_mut::<cmd_find_state>(),
            s: ::core::ptr::null_mut::<session>(),
            wl: ::core::ptr::null_mut::<winlink>(),
            w: ::core::ptr::null_mut::<window>(),
            wp: ::core::ptr::null_mut::<window_pane>(),
            idx: 0,
        },
    };
    let mut pr: *mut cmd_parse_result = ::core::ptr::null_mut::<cmd_parse_result>();
    let mut new_item0: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut state: *mut cmdq_state = ::core::ptr::null_mut::<cmdq_state>();
    if !new_item.is_null() {
        *new_item = ::core::ptr::null_mut::<cmdq_item>();
    }
    log_debug(
        b"loading %s\0" as *const u8 as *const ::core::ffi::c_char,
        path,
    );
    memset(
        &raw mut pi as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<cmd_parse_input>() as size_t,
    );
    pi.flags = flags;
    pi.file = path;
    pi.line = 1 as u_int;
    pi.item = item;
    pi.c = c;
    pr = cmd_parse_from_buffer(buf, len, &raw mut pi);
    if (*pr).status as ::core::ffi::c_uint
        == CMD_PARSE_ERROR as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        cfg_add_cause(
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*pr).error,
        );
        free((*pr).error as *mut ::core::ffi::c_void);
        return -(1 as ::core::ffi::c_int);
    }
    if flags & CMD_PARSE_PARSEONLY != 0 {
        cmd_list_free((*pr).cmdlist);
        return 0 as ::core::ffi::c_int;
    }
    if !item.is_null() {
        state = cmdq_copy_state(cmdq_get_state(item), current);
    } else {
        state = cmdq_new_state(
            ::core::ptr::null_mut::<cmd_find_state>(),
            ::core::ptr::null_mut::<key_event>(),
            0 as ::core::ffi::c_int,
        );
    }
    cmdq_add_format(
        state,
        b"current_file\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        pi.file,
    );
    new_item0 = cmdq_get_command((*pr).cmdlist, state);
    if !item.is_null() {
        new_item0 = cmdq_insert_after(item, new_item0);
    } else {
        new_item0 = cmdq_append(::core::ptr::null_mut::<client>(), new_item0);
    }
    cmd_list_free((*pr).cmdlist);
    cmdq_free_state(state);
    if !new_item.is_null() {
        *new_item = new_item0;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn cfg_add_cause(mut fmt: *const ::core::ffi::c_char, mut args: ...) {
    let mut ap: ::core::ffi::VaList;
    let mut msg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    ap = args.clone();
    xvasprintf(&raw mut msg, fmt, ap);
    cfg_ncauses = cfg_ncauses.wrapping_add(1);
    cfg_causes = xreallocarray(
        cfg_causes as *mut ::core::ffi::c_void,
        cfg_ncauses as size_t,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as size_t,
    ) as *mut *mut ::core::ffi::c_char;
    let ref mut fresh0 = *cfg_causes.offset(cfg_ncauses.wrapping_sub(1 as u_int) as isize);
    *fresh0 = msg;
}
#[no_mangle]
pub unsafe extern "C" fn cfg_print_causes(mut item: *mut cmdq_item) {
    let mut c: *mut client = cmdq_get_client(item);
    let mut i: u_int = 0;
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    i = 0 as u_int;
    while i < cfg_ncauses {
        cause = *cfg_causes.offset(i as isize);
        if !c.is_null() && (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            control_notify_write(
                c,
                b"%%config-error %s\0" as *const u8 as *const ::core::ffi::c_char,
                cause,
            );
        } else {
            cmdq_print(
                item,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                cause,
            );
        }
        free(cause as *mut ::core::ffi::c_void);
        i = i.wrapping_add(1);
    }
    free(cfg_causes as *mut ::core::ffi::c_void);
    cfg_causes = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    cfg_ncauses = 0 as u_int;
}
#[no_mangle]
pub unsafe extern "C" fn cfg_show_causes(mut s: *mut session) {
    let mut c: *mut client = clients.tqh_first;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut i: u_int = 0;
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if cfg_ncauses == 0 as u_int {
        return;
    }
    if !c.is_null() && (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
        i = 0 as u_int;
        while i < cfg_ncauses {
            cause = *cfg_causes.offset(i as isize);
            control_notify_write(
                c,
                b"%%config-error %s\0" as *const u8 as *const ::core::ffi::c_char,
                cause,
            );
            free(cause as *mut ::core::ffi::c_void);
            i = i.wrapping_add(1);
        }
    } else {
        if s.is_null() {
            if !c.is_null() && !(*c).session.is_null() {
                s = (*c).session;
            } else {
                s = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
            }
        }
        if s.is_null() || (*s).attached == 0 as u_int {
            return;
        }
        wp = (*(*(*s).curw).window).active;
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
        i = 0 as u_int;
        while i < cfg_ncauses {
            window_copy_add(
                wp,
                0 as ::core::ffi::c_int,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                *cfg_causes.offset(i as isize),
            );
            free(*cfg_causes.offset(i as isize) as *mut ::core::ffi::c_void);
            i = i.wrapping_add(1);
        }
    }
    free(cfg_causes as *mut ::core::ffi::c_void);
    cfg_causes = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    cfg_ncauses = 0 as u_int;
}
