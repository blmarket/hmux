use crate::src::arguments::{
    args_count, args_first_value, args_get, args_has, args_next_value, args_to_vector,
};
use crate::src::cfg::{cfg_finished, cfg_show_causes};
use crate::src::cmd::{cmd_free_argv, cmd_get_args, cmd_get_entry};
use crate::src::cmd_attach_session::cmd_attach_session;
use crate::src::cmd_find::cmd_find_from_session;
use crate::src::cmd_queue::{
    cmdq_error, cmdq_get_client, cmdq_get_current, cmdq_get_flags, cmdq_get_target,
    cmdq_insert_hook, cmdq_print,
};
use crate::src::compat::strtonum::strtonum;
use crate::src::environ::{environ_create, environ_put, environ_update};
use crate::src::events::events_fire_session;
use crate::src::ffi::libc::{free, sscanf, strcmp, tcgetattr};
use crate::src::format::format_single;
use crate::src::log::fatal;
use crate::src::options::{
    options_create, options_get_number, options_get_string, options_set_string,
};
use crate::src::proc::proc_send;
use crate::src::server_client::{
    server_client_check_nested, server_client_get_cwd, server_client_open, server_client_set_flags,
    server_client_set_key_table, server_client_set_session,
};
use crate::src::session::{
    session_create, session_destroy, session_find, session_group_add, session_group_contains,
    session_group_find, session_group_new, session_group_synchronize_to, session_select,
};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::*;
pub use crate::src::shared::arguments::{
    args, args_parse, args_parse_cb, args_value, args_value_c2rust_unnamed, args_value_entry,
};
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::client::{CLIENT_ATTACHED, CLIENT_CONTROL};
use crate::src::shared::colour::*;
use crate::src::shared::command::*;
pub use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds,
};
pub use crate::src::shared::command::{
    CMDQ_STATE_REPEAT, CMD_FIND_CANFAIL, CMD_STARTSERVER, CMD_TARGET_SESSION_USAGE,
};
pub use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::environment::environ;
use crate::src::shared::event::*;
pub use crate::src::shared::format::{format_job_tree, format_tree};
use crate::src::shared::grid::*;
pub use crate::src::shared::hyperlinks::hyperlinks;
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
use crate::src::shared::key::*;
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::layout_geometry;
use crate::src::shared::layout::*;
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::limits::{__SHRT_MAX__, USHRT_MAX};
pub use crate::src::shared::menu::menu_data;
use crate::src::shared::message::*;
pub use crate::src::shared::mouse::mouse_event;
pub use crate::src::shared::options::{options, options_entry};
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::process::tmuxpeer;
pub use crate::src::shared::prompt::prompt;
pub use crate::src::shared::redraw::redraw_scene;
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::screen_write_cline;
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::session::{session_group, session_group_entry, session_group_sessions};
pub use crate::src::shared::spawn::spawn_context;
pub use crate::src::shared::spawn::spawn_editor_state;
pub use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
pub use crate::src::shared::tree::RB_NEGINF;
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
use crate::src::spawn::spawn_window;
use crate::src::tmux::{check_name, clean_name, global_s_options};
use crate::src::window::winlinks_minmax;
use crate::src::xmalloc::xstrdup;

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

pub const NEW_SESSION_TEMPLATE: [::core::ffi::c_char; 17] = unsafe {
    ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b"#{session_name}:\0")
};
#[no_mangle]
pub static mut cmd_new_session_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"new-session\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"new\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"Ac:dDe:EF:f:n:Ps:t:x:Xy:\0" as *const u8
                as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
            cb: None,
        },
        usage: b"[-AdDEPX] [-c start-directory] [-e environment] [-F format] [-f flags] [-n window-name] [-s session-name] [-t target-session] [-x width] [-y height] [shell-command [argument ...]]\0"
            as *const u8 as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_SESSION,
            flags: CMD_FIND_CANFAIL,
        },
        flags: CMD_STARTSERVER,
        exec: Some(
            cmd_new_session_exec
                as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub static mut cmd_has_session_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"has-session\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"has\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"t:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: CMD_TARGET_SESSION_USAGE.as_ptr(),
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_SESSION,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_new_session_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_new_session_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut current_block: u64;
    let mut args: *mut args = cmd_get_args(self_0);
    let mut current: *mut cmd_find_state = cmdq_get_current(item);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut c: *mut client = cmdq_get_client(item);
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut as_0: *mut session = ::core::ptr::null_mut::<session>();
    let mut groupwith: *mut session = ::core::ptr::null_mut::<session>();
    let mut env: *mut environ = ::core::ptr::null_mut::<environ>();
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    let mut tio: termios = termios {
        c_iflag: 0,
        c_oflag: 0,
        c_cflag: 0,
        c_lflag: 0,
        c_line: 0,
        c_cc: [0; 32],
        c2rust_unnamed: termios_input_speed { __ispeed: 0 },
        c2rust_unnamed_0: termios_output_speed { __ospeed: 0 },
    };
    let mut tiop: *mut termios = ::core::ptr::null_mut::<termios>();
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut template: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut group: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut tmp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cwd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ename: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut wname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut sname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut prefix: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut detached: ::core::ffi::c_int = 0;
    let mut already_attached: ::core::ffi::c_int = 0;
    let mut is_control: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut dsx: u_int = 0;
    let mut dsy: u_int = 0;
    let mut count: u_int = args_count(args);
    let mut sc: spawn_context = spawn_context {
        item: ::core::ptr::null_mut::<cmdq_item>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        tc: ::core::ptr::null_mut::<client>(),
        wp0: ::core::ptr::null_mut::<window_pane>(),
        lc: ::core::ptr::null_mut::<layout_cell>(),
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        argv: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        argc: 0,
        environ: ::core::ptr::null_mut::<environ>(),
        idx: 0,
        cwd: ::core::ptr::null::<::core::ffi::c_char>(),
        flags: 0,
    };
    let mut retval: cmd_retval = CMD_RETURN_NORMAL;
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut av: *mut args_value = ::core::ptr::null_mut::<args_value>();
    if cmd_get_entry(self_0) == &raw const cmd_has_session_entry {
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 't' as i32 as u_char) != 0
        && (count != 0 as u_int || args_has(args, 'n' as i32 as u_char) != 0)
    {
        cmdq_error(
            item,
            b"command or window name given with target\0" as *const u8
                as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    tmp = args_get(args, 'n' as i32 as u_char);
    if !tmp.is_null() {
        ename = format_single(
            item,
            tmp,
            c,
            ::core::ptr::null_mut::<session>(),
            ::core::ptr::null_mut::<winlink>(),
            ::core::ptr::null_mut::<window_pane>(),
        );
        if check_name(ename) == 0 {
            cmdq_error(
                item,
                b"invalid window name: %s\0" as *const u8 as *const ::core::ffi::c_char,
                ename,
            );
            free(ename as *mut ::core::ffi::c_void);
            return CMD_RETURN_ERROR;
        }
        wname = clean_name(ename, 0 as ::core::ffi::c_int);
        free(ename as *mut ::core::ffi::c_void);
    }
    tmp = args_get(args, 's' as i32 as u_char);
    if !tmp.is_null() {
        ename = format_single(
            item,
            tmp,
            c,
            ::core::ptr::null_mut::<session>(),
            ::core::ptr::null_mut::<winlink>(),
            ::core::ptr::null_mut::<window_pane>(),
        );
        if check_name(ename) == 0 {
            cmdq_error(
                item,
                b"invalid session name: %s\0" as *const u8 as *const ::core::ffi::c_char,
                ename,
            );
            free(ename as *mut ::core::ffi::c_void);
            current_block = 5193972633326621385;
        } else {
            sname = clean_name(ename, 0 as ::core::ffi::c_int);
            free(ename as *mut ::core::ffi::c_void);
            current_block = 10043043949733653460;
        }
    } else {
        current_block = 10043043949733653460;
    }
    match current_block {
        10043043949733653460 => {
            if args_has(args, 'A' as i32 as u_char) != 0 {
                if !sname.is_null() {
                    as_0 = session_find(sname);
                } else {
                    as_0 = (*target).s;
                }
                if !as_0.is_null() {
                    retval = cmd_attach_session(
                        item,
                        (*as_0).name,
                        args_has(args, 'D' as i32 as u_char),
                        args_has(args, 'X' as i32 as u_char),
                        0 as ::core::ffi::c_int,
                        args_get(args, 'c' as i32 as u_char),
                        args_has(args, 'E' as i32 as u_char),
                        args_get(args, 'f' as i32 as u_char),
                    );
                    free(wname as *mut ::core::ffi::c_void);
                    free(sname as *mut ::core::ffi::c_void);
                    return retval;
                }
            }
            if !sname.is_null() && !session_find(sname).is_null() {
                cmdq_error(
                    item,
                    b"duplicate session: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    sname,
                );
            } else {
                group = args_get(args, 't' as i32 as u_char);
                if !group.is_null() {
                    groupwith = (*target).s;
                    if groupwith.is_null() {
                        sg = session_group_find(group);
                    } else {
                        sg = session_group_contains(groupwith);
                    }
                    if !sg.is_null() {
                        prefix = xstrdup((*sg).name);
                        current_block = 6717214610478484138;
                    } else if !groupwith.is_null() {
                        prefix = xstrdup((*groupwith).name);
                        current_block = 6717214610478484138;
                    } else if check_name(group) == 0 {
                        cmdq_error(
                            item,
                            b"invalid session group name: %s\0" as *const u8
                                as *const ::core::ffi::c_char,
                            group,
                        );
                        current_block = 5193972633326621385;
                    } else {
                        prefix = clean_name(group, 0 as ::core::ffi::c_int);
                        current_block = 6717214610478484138;
                    }
                } else {
                    current_block = 6717214610478484138;
                }
                match current_block {
                    5193972633326621385 => {}
                    _ => {
                        detached = args_has(args, 'd' as i32 as u_char);
                        if c.is_null() {
                            detached = 1 as ::core::ffi::c_int;
                        } else if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
                            is_control = 1 as ::core::ffi::c_int;
                        }
                        already_attached = 0 as ::core::ffi::c_int;
                        if !c.is_null() && !(*c).session.is_null() {
                            already_attached = 1 as ::core::ffi::c_int;
                        }
                        tmp = args_get(args, 'c' as i32 as u_char);
                        if !tmp.is_null() {
                            cwd = format_single(
                                item,
                                tmp,
                                c,
                                ::core::ptr::null_mut::<session>(),
                                ::core::ptr::null_mut::<winlink>(),
                                ::core::ptr::null_mut::<window_pane>(),
                            );
                        } else {
                            cwd = xstrdup(server_client_get_cwd(
                                c,
                                ::core::ptr::null_mut::<session>(),
                            ));
                        }
                        if detached == 0
                            && already_attached == 0
                            && (*c).fd != -(1 as ::core::ffi::c_int)
                            && !(*c).flags & CLIENT_CONTROL as uint64_t != 0
                        {
                            if server_client_check_nested(cmdq_get_client(item)) != 0 {
                                cmdq_error(
                                    item,
                                    b"sessions should be nested with care, unset $TMUX to force\0"
                                        as *const u8
                                        as *const ::core::ffi::c_char,
                                );
                                current_block = 5193972633326621385;
                            } else {
                                if tcgetattr((*c).fd, &raw mut tio) != 0 as ::core::ffi::c_int {
                                    fatal(
                                        b"tcgetattr failed\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                    );
                                }
                                tiop = &raw mut tio;
                                current_block = 6545907279487748450;
                            }
                        } else {
                            tiop = ::core::ptr::null_mut::<termios>();
                            current_block = 6545907279487748450;
                        }
                        match current_block {
                            5193972633326621385 => {}
                            _ => {
                                if detached == 0 && already_attached == 0 {
                                    if server_client_open(c, &raw mut cause)
                                        != 0 as ::core::ffi::c_int
                                    {
                                        cmdq_error(
                                            item,
                                            b"open terminal failed: %s\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                            cause,
                                        );
                                        free(cause as *mut ::core::ffi::c_void);
                                        current_block = 5193972633326621385;
                                    } else {
                                        current_block = 5181772461570869434;
                                    }
                                } else {
                                    current_block = 5181772461570869434;
                                }
                                match current_block {
                                    5193972633326621385 => {}
                                    _ => {
                                        if args_has(args, 'x' as i32 as u_char) != 0 {
                                            tmp = args_get(args, 'x' as i32 as u_char);
                                            if strcmp(
                                                tmp,
                                                b"-\0" as *const u8 as *const ::core::ffi::c_char,
                                            ) == 0 as ::core::ffi::c_int
                                            {
                                                if !c.is_null() {
                                                    dsx = (*c).tty.sx;
                                                } else {
                                                    dsx = 80 as u_int;
                                                }
                                                current_block = 5873035170358615968;
                                            } else {
                                                dsx = strtonum(
                                                    tmp,
                                                    1 as ::core::ffi::c_longlong,
                                                    USHRT_MAX as ::core::ffi::c_longlong,
                                                    &raw mut errstr,
                                                )
                                                    as u_int;
                                                if !errstr.is_null() {
                                                    cmdq_error(
                                                        item,
                                                        b"width %s\0" as *const u8
                                                            as *const ::core::ffi::c_char,
                                                        errstr,
                                                    );
                                                    current_block = 5193972633326621385;
                                                } else {
                                                    current_block = 5873035170358615968;
                                                }
                                            }
                                        } else {
                                            dsx = 80 as u_int;
                                            current_block = 5873035170358615968;
                                        }
                                        match current_block {
                                            5193972633326621385 => {}
                                            _ => {
                                                if args_has(args, 'y' as i32 as u_char) != 0 {
                                                    tmp = args_get(args, 'y' as i32 as u_char);
                                                    if strcmp(
                                                        tmp,
                                                        b"-\0" as *const u8
                                                            as *const ::core::ffi::c_char,
                                                    ) == 0 as ::core::ffi::c_int
                                                    {
                                                        if !c.is_null() {
                                                            dsy = (*c).tty.sy;
                                                        } else {
                                                            dsy = 24 as u_int;
                                                        }
                                                        current_block = 15855550149339537395;
                                                    } else {
                                                        dsy = strtonum(
                                                            tmp,
                                                            1 as ::core::ffi::c_longlong,
                                                            USHRT_MAX as ::core::ffi::c_longlong,
                                                            &raw mut errstr,
                                                        )
                                                            as u_int;
                                                        if !errstr.is_null() {
                                                            cmdq_error(
                                                                item,
                                                                b"height %s\0" as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                                errstr,
                                                            );
                                                            current_block = 5193972633326621385;
                                                        } else {
                                                            current_block = 15855550149339537395;
                                                        }
                                                    }
                                                } else {
                                                    dsy = 24 as u_int;
                                                    current_block = 15855550149339537395;
                                                }
                                                match current_block {
                                                    5193972633326621385 => {}
                                                    _ => {
                                                        if detached == 0 && is_control == 0 {
                                                            sx = (*c).tty.sx;
                                                            sy = (*c).tty.sy;
                                                            if sy > 0 as u_int
                                                                && options_get_number(
                                                                    global_s_options,
                                                                    b"status\0" as *const u8 as *const ::core::ffi::c_char,
                                                                ) != 0
                                                            {
                                                                sy = sy.wrapping_sub(1);
                                                            }
                                                        } else {
                                                            tmp = options_get_string(
                                                                global_s_options,
                                                                b"default-size\0" as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                            );
                                                            if sscanf(
                                                                tmp,
                                                                b"%ux%u\0" as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                                &raw mut sx,
                                                                &raw mut sy,
                                                            ) != 2 as ::core::ffi::c_int
                                                            {
                                                                sx = dsx;
                                                                sy = dsy;
                                                            } else {
                                                                if args_has(
                                                                    args,
                                                                    'x' as i32 as u_char,
                                                                ) != 0
                                                                {
                                                                    sx = dsx;
                                                                }
                                                                if args_has(
                                                                    args,
                                                                    'y' as i32 as u_char,
                                                                ) != 0
                                                                {
                                                                    sy = dsy;
                                                                }
                                                            }
                                                        }
                                                        if sx == 0 as u_int {
                                                            sx = 1 as u_int;
                                                        }
                                                        if sy == 0 as u_int {
                                                            sy = 1 as u_int;
                                                        }
                                                        oo = options_create(global_s_options);
                                                        if args_has(args, 'x' as i32 as u_char) != 0
                                                            || args_has(args, 'y' as i32 as u_char)
                                                                != 0
                                                        {
                                                            if args_has(args, 'x' as i32 as u_char)
                                                                == 0
                                                            {
                                                                dsx = sx;
                                                            }
                                                            if args_has(args, 'y' as i32 as u_char)
                                                                == 0
                                                            {
                                                                dsy = sy;
                                                            }
                                                            options_set_string(
                                                                oo,
                                                                b"default-size\0" as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                                0 as ::core::ffi::c_int,
                                                                b"%ux%u\0" as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                                dsx,
                                                                dsy,
                                                            );
                                                        }
                                                        env = environ_create();
                                                        if !c.is_null()
                                                            && args_has(args, 'E' as i32 as u_char)
                                                                == 0
                                                        {
                                                            environ_update(
                                                                global_s_options,
                                                                (*c).environ,
                                                                env,
                                                            );
                                                        }
                                                        av = args_first_value(
                                                            args,
                                                            'e' as i32 as u_char,
                                                        );
                                                        while !av.is_null() {
                                                            environ_put(
                                                                env,
                                                                (*av).c2rust_unnamed.string,
                                                                0 as ::core::ffi::c_int,
                                                            );
                                                            av = args_next_value(av);
                                                        }
                                                        s = session_create(
                                                            prefix, sname, cwd, env, oo, tiop,
                                                        );
                                                        sc.item = item;
                                                        sc.s = s;
                                                        if detached == 0 {
                                                            sc.tc = c;
                                                        }
                                                        sc.name = wname;
                                                        args_to_vector(
                                                            args,
                                                            &raw mut sc.argc,
                                                            &raw mut sc.argv,
                                                        );
                                                        sc.idx = -(1 as ::core::ffi::c_int);
                                                        sc.cwd =
                                                            args_get(args, 'c' as i32 as u_char);
                                                        sc.flags = 0 as ::core::ffi::c_int;
                                                        if spawn_window(&raw mut sc, &raw mut cause)
                                                            .is_null()
                                                        {
                                                            session_destroy(
                                                                s,
                                                                0 as ::core::ffi::c_int,
                                                                b"cmd_new_session_exec\0"
                                                                    as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                            );
                                                            cmdq_error(
                                                                item,
                                                                b"create window failed: %s\0"
                                                                    as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                                cause,
                                                            );
                                                            free(cause as *mut ::core::ffi::c_void);
                                                        } else {
                                                            if !group.is_null() {
                                                                if sg.is_null() {
                                                                    if !groupwith.is_null() {
                                                                        sg = session_group_new(
                                                                            (*groupwith).name,
                                                                        );
                                                                        session_group_add(
                                                                            sg, groupwith,
                                                                        );
                                                                    } else {
                                                                        sg = session_group_new(
                                                                            group,
                                                                        );
                                                                    }
                                                                }
                                                                session_group_add(sg, s);
                                                                session_group_synchronize_to(s);
                                                                session_select(
                                                                    s,
                                                                    (*winlinks_minmax(
                                                                        &raw mut (*s).windows,
                                                                        RB_NEGINF,
                                                                    ))
                                                                    .idx,
                                                                );
                                                            }
                                                            events_fire_session(
                                                                b"session-created\0" as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                                s,
                                                            );
                                                            if detached == 0 {
                                                                if args_has(
                                                                    args,
                                                                    'f' as i32 as u_char,
                                                                ) != 0
                                                                {
                                                                    server_client_set_flags(
                                                                        c,
                                                                        args_get(
                                                                            args,
                                                                            'f' as i32 as u_char,
                                                                        ),
                                                                    );
                                                                }
                                                                if already_attached == 0 {
                                                                    if !(*c).flags
                                                                        & CLIENT_CONTROL as uint64_t
                                                                        != 0
                                                                    {
                                                                        proc_send(
                                                                            (*c).peer,
                                                                            MSG_READY,
                                                                            -(1 as ::core::ffi::c_int),
                                                                            ::core::ptr::null::<::core::ffi::c_void>(),
                                                                            0 as size_t,
                                                                        );
                                                                    }
                                                                } else if !(*c).session.is_null() {
                                                                    (*c).last_session =
                                                                        (*c).session;
                                                                }
                                                                server_client_set_session(c, s);
                                                                if !cmdq_get_flags(item)
                                                                    & CMDQ_STATE_REPEAT
                                                                    != 0
                                                                {
                                                                    server_client_set_key_table(
                                                                        c,
                                                                        ::core::ptr::null::<
                                                                            ::core::ffi::c_char,
                                                                        >(
                                                                        ),
                                                                    );
                                                                }
                                                            }
                                                            if args_has(args, 'P' as i32 as u_char)
                                                                != 0
                                                            {
                                                                template = args_get(
                                                                    args,
                                                                    'F' as i32 as u_char,
                                                                );
                                                                if template.is_null() {
                                                                    template = NEW_SESSION_TEMPLATE
                                                                        .as_ptr();
                                                                }
                                                                cp = format_single(
                                                                    item,
                                                                    template,
                                                                    c,
                                                                    s,
                                                                    (*s).curw,
                                                                    ::core::ptr::null_mut::<
                                                                        window_pane,
                                                                    >(
                                                                    ),
                                                                );
                                                                cmdq_print(
                                                                    item,
                                                                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                                                                    cp,
                                                                );
                                                                free(
                                                                    cp as *mut ::core::ffi::c_void,
                                                                );
                                                            }
                                                            if detached == 0 {
                                                                (*c).flags |=
                                                                    CLIENT_ATTACHED as uint64_t;
                                                            }
                                                            if args_has(args, 'd' as i32 as u_char)
                                                                == 0
                                                            {
                                                                cmd_find_from_session(
                                                                    current,
                                                                    s,
                                                                    0 as ::core::ffi::c_int,
                                                                );
                                                            }
                                                            cmd_find_from_session(
                                                                &raw mut fs,
                                                                s,
                                                                0 as ::core::ffi::c_int,
                                                            );
                                                            cmdq_insert_hook(
                                                                s,
                                                                item,
                                                                &raw mut fs,
                                                                b"after-new-session\0" as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                            );
                                                            if cfg_finished != 0 {
                                                                cfg_show_causes(s);
                                                            }
                                                            if !sc.argv.is_null() {
                                                                cmd_free_argv(sc.argc, sc.argv);
                                                            }
                                                            free(cwd as *mut ::core::ffi::c_void);
                                                            free(wname as *mut ::core::ffi::c_void);
                                                            free(sname as *mut ::core::ffi::c_void);
                                                            free(
                                                                prefix as *mut ::core::ffi::c_void,
                                                            );
                                                            return CMD_RETURN_NORMAL;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    if !sc.argv.is_null() {
        cmd_free_argv(sc.argc, sc.argv);
    }
    free(cwd as *mut ::core::ffi::c_void);
    free(wname as *mut ::core::ffi::c_void);
    free(sname as *mut ::core::ffi::c_void);
    free(prefix as *mut ::core::ffi::c_void);
    return CMD_RETURN_ERROR;
}
