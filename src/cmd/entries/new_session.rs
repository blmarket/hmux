use crate::src::arguments::{args_count, args_flag_values, args_get, args_has, args_to_vector};
use crate::src::cfg::{cfg_finished, cfg_show_causes};
use crate::src::cmd::entries::attach_session::cmd_attach_session;
use crate::src::cmd::find::cmd_find_from_session;
use crate::src::cmd::queue::{
    cmdq_error, cmdq_get_client, cmdq_get_flags, cmdq_get_state_owned, cmdq_get_target,
    cmdq_insert_hook, cmdq_print,
};
use crate::src::cmd::{cmd_get_args_mut, cmd_get_entry};
use crate::src::compat::imsg::*;
use crate::src::compat::strtonum::strtonum;
use crate::src::environ::{environ_create, environ_put, environ_update};
use crate::src::events::events_fire_session;
use crate::src::ffi::libc::{sscanf, strcmp, tcgetattr};
use crate::src::format::bytes::write_cstr;
use crate::src::format::format_single_cstring;
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
use crate::src::shared::arguments::{args, args_parse, args_value};
use crate::src::shared::client::client;
use crate::src::shared::client::{CLIENT_ATTACHED, CLIENT_CONTROL};
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::command::{
    CMDQ_STATE_REPEAT, CMD_FIND_CANFAIL, CMD_STARTSERVER, CMD_TARGET_SESSION_USAGE,
};
use crate::src::shared::environment::environ;
use crate::src::shared::layout::layout_cell;
use crate::src::shared::limits::USHRT_MAX;
use crate::src::shared::options::options;
use crate::src::shared::pane::window_pane;
use crate::src::shared::session::session;
use crate::src::shared::session::session_group;
use crate::src::shared::spawn::spawn_context;
use crate::src::shared::terminal::*;
use crate::src::shared::tree::RB_NEGINF;
use crate::src::shared::vis::{VIS_CSTYLE, VIS_NL, VIS_OCTAL, VIS_TAB};
use crate::src::shared::window::{window, winlink};
use crate::src::spawn::spawn_window;
use crate::src::text::utf8::utf8_stravis_cstring;
use crate::src::tmux::{check_name, clean_name_cstring, global_s_options};
use crate::src::window::winlinks_minmax;
use std::ffi::{CStr, CString};

pub const NEW_SESSION_TEMPLATE: [::core::ffi::c_char; 17] = unsafe {
    ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b"#{session_name}:\0")
};
pub static cmd_new_session_entry: cmd_entry = {
    cmd_entry {
        name: c"new-session",
        alias: Some(c"new"),
        args: args_parse {
            template: c"Ac:dDe:EF:f:n:Ps:t:x:Xy:",
            lower: 0 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
            cb: None,
        },
        usage: c"[-AdDEPX] [-c start-directory] [-e environment] [-F format] [-f flags] [-n window-name] [-s session-name] [-t target-session] [-x width] [-y height] [shell-command [argument ...]]",
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
        exec: Some(cmd_new_session_exec),
    }
};
pub static cmd_has_session_entry: cmd_entry = {
    cmd_entry {
        name: c"has-session",
        alias: Some(c"has"),
        args: args_parse {
            template: c"t:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: CMD_TARGET_SESSION_USAGE,
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
        exec: Some(cmd_new_session_exec),
    }
};
unsafe fn cmd_new_session_exec(
    mut self_0: refbox::Weak<cmd>,
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
) -> cmd_retval {
    let item = item_handle.get();
    let queue_client = cmdq_get_client((item).as_ref());
    let queue_client_ptr = queue_client
        .as_ref()
        .map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    let mut current_block: u64;
    let mut args: *mut args =
        cmd_get_args_mut(self_0.get_mut_unchecked()).map_or(std::ptr::null_mut(), |args| args);
    let current = cmdq_get_state_owned(&*(item));
    let mut target: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let c_owner = cmdq_get_client((item).as_ref());
    let mut c: *mut client = c_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut as_0: *mut session = ::core::ptr::null_mut::<session>();
    let mut groupwith: *mut session = ::core::ptr::null_mut::<session>();
    let mut groupwith_owner = None;
    let mut env: Option<Box<environ>> = None;
    let mut oo: Option<Box<options>> = None;
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
    let mut cause: Option<std::ffi::CString> = None;
    let mut cwd: *const ::core::ffi::c_char = ::core::ptr::null();
    let mut formatted_cwd: Option<CString> = None;
    let mut wname: *const ::core::ffi::c_char = ::core::ptr::null();
    let mut sname: *const ::core::ffi::c_char = ::core::ptr::null();
    let mut wname_owned: Option<CString> = None;
    let mut sname_owned: Option<CString> = None;
    let mut prefix: Option<CString> = None;
    let mut detached: ::core::ffi::c_int = 0;
    let mut already_attached: ::core::ffi::c_int = 0;
    let mut is_control: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut dsx: u_int = 0;
    let mut dsy: u_int = 0;
    let mut count: u_int = args_count(args);
    let mut sc: spawn_context = spawn_context {
        item: std::rc::Weak::new(),
        s: std::rc::Weak::new(),
        wl: refbox::Weak::new(),
        tc: std::rc::Weak::new(),
        wp0: std::rc::Weak::new(),
        lc: ::core::ptr::null_mut::<layout_cell>(),
        name: None,
        argv: Vec::new(),
        environ: None,
        idx: 0,
        cwd: None,
        flags: 0,
    };
    let mut argv_owner = Vec::new();
    let mut retval: cmd_retval = CMD_RETURN_NORMAL;
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        s: Default::default(),
        wl: Default::default(),
        w: Default::default(),
        wp: Default::default(),
        idx: 0,
    };
    if std::ptr::eq(
        cmd_get_entry(self_0.get_unchecked()),
        &cmd_has_session_entry,
    ) {
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 't' as i32 as u_char) != 0
        && (count != 0 as u_int || args_has(args, 'n' as i32 as u_char) != 0)
    {
        cmdq_error(item_handle, |out| {
            out.write_all(b"command or window name given with target")
        });
        return CMD_RETURN_ERROR;
    }
    tmp = args_get(&*(args), 'n' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
    if !tmp.is_null() {
        let ename = format_single_cstring(
            Some(item_handle),
            tmp,
            (c).as_ref()
                .and_then(|model| model.observer.upgrade())
                .as_ref(),
            None,
            (refbox::Weak::new()).clone(),
            None,
        );
        if !check_name(&ename) {
            cmdq_error(item_handle, |out| {
                out.write_all(b"invalid window name: ")?;
                out.write_all(ename.as_bytes())
            });
            return CMD_RETURN_ERROR;
        }
        wname_owned = Some(
            clean_name_cstring(ename.as_c_str(), 0).expect("check_name validated the window name"),
        );
        wname = wname_owned
            .as_ref()
            .expect("window name was cleaned")
            .as_ptr();
    }
    tmp = args_get(&*(args), 's' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
    if !tmp.is_null() {
        let ename = format_single_cstring(
            Some(item_handle),
            tmp,
            (c).as_ref()
                .and_then(|model| model.observer.upgrade())
                .as_ref(),
            None,
            (refbox::Weak::new()).clone(),
            None,
        );
        if !check_name(&ename) {
            cmdq_error(item_handle, |out| {
                out.write_all(b"invalid session name: ")?;
                out.write_all(ename.as_bytes())
            });
            current_block = 5193972633326621385;
        } else {
            sname_owned = Some(
                clean_name_cstring(ename.as_c_str(), 0)
                    .expect("check_name validated the session name"),
            );
            sname = sname_owned
                .as_ref()
                .expect("session name was cleaned")
                .as_ptr();
            current_block = 10043043949733653460;
        }
    } else {
        current_block = 10043043949733653460;
    }
    match current_block {
        10043043949733653460 => {
            if args_has(args, 'A' as i32 as u_char) != 0 {
                if !sname.is_null() {
                    as_0 = session_find(std::ffi::CStr::from_ptr(sname))
                        .as_ref()
                        .map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
                } else {
                    as_0 = (*target)
                        .session_handle()
                        .as_ref()
                        .map_or(std::ptr::null_mut(), |owner| owner.get());
                }
                if !as_0.is_null() {
                    retval = cmd_attach_session(
                        item_handle,
                        ((*as_0).name).as_ptr().cast_mut(),
                        args_has(args, 'D' as i32 as u_char),
                        args_has(args, 'X' as i32 as u_char),
                        0 as ::core::ffi::c_int,
                        args_get(&*(args), 'c' as i32 as u_char)
                            .map_or(std::ptr::null(), |value| value.as_ptr()),
                        args_has(args, 'E' as i32 as u_char),
                        args_get(&*(args), 'f' as i32 as u_char)
                            .map_or(std::ptr::null(), |value| value.as_ptr()),
                    );
                    return retval;
                }
            }
            if !sname.is_null()
                && !session_find(std::ffi::CStr::from_ptr(sname))
                    .as_ref()
                    .map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr)
                    .is_null()
            {
                cmdq_error(item_handle, |out| {
                    out.write_all(b"duplicate session: ")?;
                    write_cstr(out, sname)
                });
            } else {
                group = args_get(&*(args), 't' as i32 as u_char)
                    .map_or(std::ptr::null(), |value| value.as_ptr());
                if !group.is_null() {
                    groupwith_owner = (*target).s.upgrade();
                    groupwith = groupwith_owner
                        .as_ref()
                        .map_or(std::ptr::null_mut(), |owner| owner.get());
                    if groupwith.is_null() {
                        sg = session_group_find(group);
                    } else {
                        sg = session_group_contains((groupwith).as_ref());
                    }
                    if !sg.is_null() {
                        prefix = Some((*sg).name.clone());
                        current_block = 6717214610478484138;
                    } else if !groupwith.is_null() {
                        prefix = Some((*groupwith).name.clone());
                        current_block = 6717214610478484138;
                    } else if !check_name(CStr::from_ptr(group)) {
                        cmdq_error(item_handle, |out| {
                            out.write_all(b"invalid session group name: ")?;
                            write_cstr(out, group)
                        });
                        current_block = 5193972633326621385;
                    } else {
                        prefix = Some(utf8_stravis_cstring(
                            CStr::from_ptr(group),
                            VIS_OCTAL | VIS_CSTYLE | VIS_TAB | VIS_NL,
                        ));
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
                        if !c.is_null() && !(*c).session_handle().is_none() {
                            already_attached = 1 as ::core::ffi::c_int;
                        }
                        tmp = args_get(&*(args), 'c' as i32 as u_char)
                            .map_or(std::ptr::null(), |value| value.as_ptr());
                        if !tmp.is_null() {
                            formatted_cwd = Some(format_single_cstring(
                                Some(item_handle),
                                tmp,
                                (c).as_ref()
                                    .and_then(|model| model.observer.upgrade())
                                    .as_ref(),
                                None,
                                (refbox::Weak::new()).clone(),
                                None,
                            ));
                            cwd = formatted_cwd
                                .as_ref()
                                .expect("formatted cwd was just set")
                                .as_ptr();
                        } else {
                            // session_create copies this borrowed cwd into session.
                            formatted_cwd = server_client_get_cwd(c.as_ref(), None);
                            cwd = formatted_cwd
                                .as_ref()
                                .map_or(std::ptr::null(), |value| value.as_ptr());
                        }
                        if detached == 0
                            && already_attached == 0
                            && (*c).fd != -(1 as ::core::ffi::c_int)
                            && !(*c).flags & CLIENT_CONTROL as uint64_t != 0
                        {
                            if server_client_check_nested(&*(queue_client_ptr)) != 0 {
                                cmdq_error(item_handle, |out| {
                                    out.write_all(b"sessions should be nested with care, unset $TMUX to force")
                                });
                                current_block = 5193972633326621385;
                            } else {
                                if tcgetattr((*c).fd, &raw mut tio) != 0 as ::core::ffi::c_int {
                                    fatal(|out| out.write_all(b"tcgetattr failed"));
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
                                    if let Err(open_error) = server_client_open(
                                        c_owner.as_ref().expect("terminal client"),
                                    ) {
                                        cmdq_error(item_handle, |out| {
                                            out.write_all(b"open terminal failed: ")?;
                                            out.write_all(open_error.as_bytes())
                                        });
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
                                            tmp = args_get(&*(args), 'x' as i32 as u_char)
                                                .map_or(std::ptr::null(), |value| value.as_ptr());
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
                                                    cmdq_error(item_handle, |out| {
                                                        out.write_all(b"width ")?;
                                                        write_cstr(out, errstr)
                                                    });
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
                                                    tmp = args_get(&*(args), 'y' as i32 as u_char)
                                                        .map_or(std::ptr::null(), |value| {
                                                            value.as_ptr()
                                                        });
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
                                                            cmdq_error(item_handle, |out| {
                                                                out.write_all(b"height ")?;
                                                                write_cstr(out, errstr)
                                                            });
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
                                                        oo = Some(crate::src::options::options_create_owned(global_s_options));
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
                                                                &mut **oo
                                                                    .as_mut()
                                                                    .expect("new session options"),
                                                                b"default-size\0" as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                                0 as ::core::ffi::c_int,
                                                                |out| {
                                                                    write!(
                                                                        out,
                                                                        "{}x{}",
                                                                        (dsx) as u32,
                                                                        (dsy) as u32
                                                                    )
                                                                },
                                                            );
                                                        }
                                                        env = Some(environ_create());
                                                        if !c.is_null()
                                                            && args_has(args, 'E' as i32 as u_char)
                                                                == 0
                                                        {
                                                            environ_update(
                                                                global_s_options,
                                                                (*c).environ
                                                                    .as_deref()
                                                                    .expect("client environment"),
                                                                env.as_deref_mut()
                                                                    .expect("new environment"),
                                                            );
                                                        }
                                                        for av in args_flag_values(
                                                            &*args,
                                                            'e' as i32 as u_char,
                                                        ) {
                                                            environ_put(
                                                                env.as_deref_mut()
                                                                    .expect("environment"),
                                                                av.string_ptr(),
                                                                0 as ::core::ffi::c_int,
                                                            );
                                                        }
                                                        let session_owner = session_create(
                                                            prefix.as_deref(),
                                                            (!sname.is_null())
                                                                .then(|| CStr::from_ptr(sname)),
                                                            CStr::from_ptr(cwd),
                                                            env.take()
                                                                .expect("new session environment"),
                                                            oo.take(),
                                                            tiop.as_ref(),
                                                        );
                                                        s = session_owner.get();
                                                        sc.item = (*item).observer.clone();
                                                        sc.s =
                                                            std::rc::Rc::downgrade(&session_owner);
                                                        if detached == 0 {
                                                            sc.tc = c_owner.as_ref().map_or_else(
                                                                std::rc::Weak::new,
                                                                std::rc::Rc::downgrade,
                                                            );
                                                        }
                                                        sc.name = (!wname.is_null()).then(|| {
                                                            CStr::from_ptr(wname).to_owned()
                                                        });
                                                        argv_owner = args_to_vector(&*args);
                                                        sc.argv = argv_owner;
                                                        sc.idx = -(1 as ::core::ffi::c_int);
                                                        sc.cwd = args_get(
                                                            &*(args),
                                                            'c' as i32 as u_char,
                                                        )
                                                        .map(CStr::to_owned);
                                                        sc.flags = 0 as ::core::ffi::c_int;
                                                        if !spawn_window(
                                                            &raw mut sc,
                                                            &raw mut cause,
                                                        )
                                                        .is_alive()
                                                        {
                                                            session_destroy(
                                                                &(*s)
                                                                    .observer
                                                                    .upgrade()
                                                                    .expect("live session"),
                                                                0 as ::core::ffi::c_int,
                                                                b"cmd_new_session_exec\0"
                                                                    as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                            );
                                                            cmdq_error(item_handle, |out| {
                                                                out.write_all(
                                                                    b"create window failed: ",
                                                                )?;
                                                                write_cstr(
                                                                    out,
                                                                    cause.as_ref().map_or(
                                                                        ::core::ptr::null(),
                                                                        |value| value.as_ptr(),
                                                                    ),
                                                                )
                                                            });
                                                        } else {
                                                            if !group.is_null() {
                                                                if sg.is_null() {
                                                                    if !groupwith.is_null() {
                                                                        sg = session_group_new(
                                                                            ((*groupwith).name)
                                                                                .as_ptr()
                                                                                .cast_mut(),
                                                                        );
                                                                        session_group_add(
                                                                            sg, groupwith_owner.as_ref().expect("retained group session"),
                                                                        );
                                                                    } else {
                                                                        sg = session_group_new(
                                                                            group,
                                                                        );
                                                                    }
                                                                }
                                                                session_group_add(
                                                                    sg,
                                                                    &session_owner,
                                                                );
                                                                session_group_synchronize_to(
                                                                    &(*s)
                                                                        .observer
                                                                        .upgrade()
                                                                        .expect("live session"),
                                                                );
                                                                session_select(
                                                                    &(*s)
                                                                        .observer
                                                                        .upgrade()
                                                                        .expect("live session"),
                                                                    (winlinks_minmax(
                                                                        &(*s).windows,
                                                                        RB_NEGINF,
                                                                    ))
                                                                    .get_unchecked()
                                                                    .idx,
                                                                );
                                                            }
                                                            events_fire_session(
                                                                b"session-created\0" as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                                (*(s))
                                                                    .observer
                                                                    .upgrade()
                                                                    .expect("live session"),
                                                            );
                                                            if detached == 0 {
                                                                if args_has(
                                                                    args,
                                                                    'f' as i32 as u_char,
                                                                ) != 0
                                                                {
                                                                    server_client_set_flags(
                                                                        &(*(c))
                                                                            .observer
                                                                            .upgrade()
                                                                            .expect("live client"),
                                                                        args_get(
                                                                            &*(args),
                                                                            'f' as i32 as u_char,
                                                                        )
                                                                        .map_or(
                                                                            std::ptr::null(),
                                                                            |value| value.as_ptr(),
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
                                                                } else if !(*c)
                                                                    .session_handle()
                                                                    .is_none()
                                                                {
                                                                    (*c).last_session =
                                                                        (*c).session.clone();
                                                                }
                                                                server_client_set_session(
                                                                    &(*(c))
                                                                        .observer
                                                                        .upgrade()
                                                                        .expect("live client"),
                                                                    (s).as_ref()
                                                                        .and_then(|model| {
                                                                            model.observer.upgrade()
                                                                        })
                                                                        .as_ref(),
                                                                );
                                                                if !cmdq_get_flags(&*(item))
                                                                    & CMDQ_STATE_REPEAT
                                                                    != 0
                                                                {
                                                                    server_client_set_key_table(
                                                                        &(*(c))
                                                                            .observer
                                                                            .upgrade()
                                                                            .expect("live client"),
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
                                                                    &*(args),
                                                                    'F' as i32 as u_char,
                                                                )
                                                                .map_or(std::ptr::null(), |value| {
                                                                    value.as_ptr()
                                                                });
                                                                if template.is_null() {
                                                                    template = NEW_SESSION_TEMPLATE
                                                                        .as_ptr();
                                                                }
                                                                let cp = format_single_cstring(
                                                                    Some(item_handle),
                                                                    template,
                                                                    (c).as_ref()
                                                                        .and_then(|model| {
                                                                            model.observer.upgrade()
                                                                        })
                                                                        .as_ref(),
                                                                    (s).as_ref()
                                                                        .and_then(|model| {
                                                                            model.observer.upgrade()
                                                                        })
                                                                        .as_ref(),
                                                                    ((*s).current_winlink())
                                                                        .clone(),
                                                                    None,
                                                                );
                                                                cmdq_print(item_handle, |out| {
                                                                    out.write_all(cp.as_bytes())
                                                                });
                                                            }
                                                            if detached == 0 {
                                                                (*c).flags |=
                                                                    CLIENT_ATTACHED as uint64_t;
                                                            }
                                                            if args_has(args, 'd' as i32 as u_char)
                                                                == 0
                                                            {
                                                                cmd_find_from_session(
                                                                    &mut *current
                                                                        .current
                                                                        .borrow_mut(),
                                                                    &(*(s))
                                                                        .observer
                                                                        .upgrade()
                                                                        .expect("live session"),
                                                                    0 as ::core::ffi::c_int,
                                                                );
                                                            }
                                                            cmd_find_from_session(
                                                                &raw mut fs,
                                                                &(*(s))
                                                                    .observer
                                                                    .upgrade()
                                                                    .expect("live session"),
                                                                0 as ::core::ffi::c_int,
                                                            );
                                                            cmdq_insert_hook(
                                                                (s).as_ref()
                                                                    .and_then(|model| {
                                                                        model.observer.upgrade()
                                                                    })
                                                                    .as_ref(),
                                                                item_handle,
                                                                &raw mut fs,
                                                                |out| {
                                                                    out.write_all(
                                                                        b"after-new-session",
                                                                    )
                                                                },
                                                            );
                                                            if cfg_finished != 0 {
                                                                cfg_show_causes(
                                                                    (s).as_ref()
                                                                        .and_then(|model| {
                                                                            model.observer.upgrade()
                                                                        })
                                                                        .as_ref(),
                                                                );
                                                            }
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
    return CMD_RETURN_ERROR;
}
