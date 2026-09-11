use crate::args::arguments_trait::Arguments as _;
use crate::args::RustArguments;
use crate::args::args_parse_t;
use crate::cfg::{cfg_show_causes_for_session, configuration_finished};
use crate::cmd::cmd_find_from_session_ref;
use crate::cmd::cmdq_item_weak_of;
use crate::cmd::entries::cmd_attach_session::cmd_attach_session;

use crate::compat::strtonum;
use crate::environ::EnvironmentStore;
use crate::environ::{RustEnvironment, new_environment_box};
use crate::ffi::{sscanf, tcgetattr};
use crate::fmt_args;
use crate::format::{format_create_for_client, format_defaults_for_handles, format_expand};
use crate::log::fatal;
use crate::options::{OptionsEngine, RustOptionsEngine};

use crate::server::client_working_directory;
use crate::session::{session_group_ensure, session_group_name, with_session_group_named};

use crate::cmd::cmdq_item;
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::consts::{
    CLIENT_CONTROL, CMD_FIND_CANFAIL, CMD_FIND_PANE, CMD_FIND_SESSION, CMD_RETURN_ERROR,
    CMD_RETURN_NORMAL, CMD_STARTSERVER, CMD_TARGET_SESSION_USAGE, CMDQ_STATE_REPEAT, USHRT_MAX,
};
use crate::spawn::spawn_window;
use crate::tmux::global_session_options;
use crate::tmux::{check_name, clean_name};
use crate::types::{
    ClientRef, OptionsRef, SessionRef, cmd_find_state, spawn_context, termios, u_char, u_int,
    uint64_t,
};
use ::std::ffi::{CStr, CString};

pub const NEW_SESSION_TEMPLATE: &CStr = c"#{session_name}:";
pub(crate) static cmd_new_session_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"new-session",
        alias: Some(c"new"),
        args: args_parse_t {
            template: c"Ac:dDe:EF:f:n:Ps:t:x:Xy:",
            lower: 0 as core::ffi::c_int,
            upper: -(1 as core::ffi::c_int),
            cb: None,
        },
        usage: c"[-AdDEPX] [-c start-directory] [-e environment] [-F format] [-f flags] [-n window-name] [-s session-name] [-t target-session] [-x width] [-y height] [shell-command [argument ...]]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as core::ffi::c_char,
            type_0: CMD_FIND_SESSION,
            flags: CMD_FIND_CANFAIL,
        },
        flags: CMD_STARTSERVER,
        exec: cmd_new_session_exec,
    }
};
pub(crate) static cmd_has_session_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"has-session",
        alias: Some(c"has"),
        args: args_parse_t {
            template: c"t:",
            lower: 0 as core::ffi::c_int,
            upper: 0 as core::ffi::c_int,
            cb: None,
        },
        usage: CMD_TARGET_SESSION_USAGE,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as core::ffi::c_char,
            type_0: CMD_FIND_SESSION,
            flags: 0 as core::ffi::c_int,
        },
        flags: 0 as core::ffi::c_int,
        exec: cmd_new_session_exec,
    }
};
fn join_session_group(
    requested: &CStr,
    existing: Option<&CStr>,
    groupwith: Option<&SessionRef>,
    created: &SessionRef,
) {
    {
        let group_name = groupwith.and_then(SessionRef::name);
        let name = existing.or(group_name.as_deref()).unwrap_or(requested);
        if existing.is_none() {
            session_group_ensure(name);
            if let Some(member) = groupwith {
                member.join_group(name);
            }
        }
        created.join_group(name);
    }
}

unsafe fn cmd_new_session_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let mut current_block: u64;
    let args: &RustArguments = crate::Command::command_arguments(self_0).expect("the command carries arguments");
    let current_state_ref = item.state_ref();
    let cmdq_client = item.client();
    let mut c = cmdq_client.clone();
    let target_session_ref = item.target.session();
    let mut groupwith: Option<SessionRef> = None;
    let mut env: Option<Box<RustEnvironment>>;
    let mut oo: Option<crate::options::RustOptionsRef>;
    let mut tio: termios = unsafe { core::mem::zeroed() };
    let mut tiop: Option<&termios> = None;
    let mut group_name: Option<CString> = None;
    let group: Option<&CStr>;
    let mut cause: Option<CString> = None;
    let cwd: Option<CString>;
    let mut wname: Option<CString> = None;
    let mut sname: Option<CString> = None;
    let mut prefix: Option<CString> = None;
    let mut detached: core::ffi::c_int;
    let mut already_attached: core::ffi::c_int;
    let mut is_control: core::ffi::c_int = 0 as core::ffi::c_int;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut dsx: u_int = 0;
    let mut dsy: u_int = 0;
    let count: u_int = args.argument_count();
    let mut sc = spawn_context::default();
    let retval: cmd_retval;
    let mut fs = cmd_find_state::default();
    if core::ptr::eq(crate::Command::command_entry(self_0), &cmd_has_session_entry) {
        return CMD_RETURN_NORMAL;
    }
    if ({
        let flag = 't' as i32 as u_char;
        args.argument_flag_count(flag)
    }) != 0
        && (count != 0 as u_int
            || ({
                let flag = 'n' as i32 as u_char;
                args.argument_flag_count(flag)
            }) != 0)
    {
        unsafe { item.error(c"command or window name given with target", fmt_args![]) };
        return CMD_RETURN_ERROR;
    }
    if let Some(tmp) = {
        let flag = 'n' as i32 as u_char;
        args.argument_flag_string(flag)
    } {
        let ename = unsafe {
            let mut ft = format_create_for_client(item.client().as_ref(), Some(item), 0, 0);
            format_defaults_for_handles(&mut ft, c.as_ref(), None, None, None);
            format_expand(&mut ft, tmp)
        };
        if check_name(Some(&ename)) == 0 {
            unsafe { item.error(c"invalid window name: %s", fmt_args![ename.as_c_str()]) };
            return CMD_RETURN_ERROR;
        }
        unsafe { wname = clean_name(&ename, 0 as core::ffi::c_int) };
    }
    if let Some(tmp) = {
        let flag = 's' as i32 as u_char;
        args.argument_flag_string(flag)
    } {
        let ename = unsafe {
            let mut ft = format_create_for_client(item.client().as_ref(), Some(item), 0, 0);
            format_defaults_for_handles(&mut ft, c.as_ref(), None, None, None);
            format_expand(&mut ft, tmp)
        };
        if check_name(Some(&ename)) == 0 {
            unsafe { item.error(c"invalid session name: %s", fmt_args![ename.as_c_str()]) };
            current_block = 971598175612140897;
        } else {
            unsafe { sname = clean_name(&ename, 0 as core::ffi::c_int) };
            current_block = 10043043949733653460;
        }
    } else {
        current_block = 10043043949733653460;
    }
    if current_block == 10043043949733653460 {
        if ({
            let flag = 'A' as i32 as u_char;
            args.argument_flag_count(flag)
        }) != 0
        {
            let attach_session = if let Some(sname) = sname.as_ref() {
                SessionRef::find(sname)
            } else {
                target_session_ref.clone()
            };
            if let Some(attach_session) = attach_session {
                unsafe {
                    retval = cmd_attach_session(
                        item,
                        Some(
                            attach_session
                                .name()
                                .as_deref()
                                .expect("the session has a name"),
                        ),
                        {
                            let flag = 'D' as i32 as u_char;
                            args.argument_flag_count(flag)
                        },
                        {
                            let flag = 'X' as i32 as u_char;
                            args.argument_flag_count(flag)
                        },
                        0 as core::ffi::c_int,
                        {
                            let flag = 'c' as i32 as u_char;
                            args.argument_flag_string(flag)
                        },
                        {
                            let flag = 'E' as i32 as u_char;
                            args.argument_flag_count(flag)
                        },
                        {
                            let flag = 'f' as i32 as u_char;
                            args.argument_flag_string(flag)
                        },
                    )
                };
                return retval;
            }
        }
        if sname
            .as_ref()
            .is_some_and(|sname| SessionRef::find(sname).is_some())
        {
            unsafe {
                item.error(
                    c"duplicate session: %s",
                    fmt_args![sname.as_ref().unwrap().as_c_str()],
                )
            };
        } else {
            group = {
                let flag = 't' as i32 as u_char;
                args.argument_flag_string(flag)
            };
            if let Some(group) = group {
                groupwith = target_session_ref.clone();
                group_name = match groupwith.as_ref() {
                    Some(member) => member.with_group(|group| session_group_name(group).to_owned()),
                    None => with_session_group_named(group, |group| {
                        session_group_name(group).to_owned()
                    }),
                };
                if let Some(name) = &group_name {
                    prefix = Some(name.clone());
                    current_block = 6717214610478484138;
                } else if let Some(member) = &groupwith {
                    prefix = Some(
                        member
                            .name()
                            .as_deref()
                            .expect("the session has a name")
                            .to_owned(),
                    );
                    current_block = 6717214610478484138;
                } else if check_name(Some(group)) == 0 {
                    unsafe { item.error(c"invalid session group name: %s", fmt_args![group]) };
                    current_block = 971598175612140897;
                } else {
                    unsafe { prefix = clean_name(group, 0 as core::ffi::c_int) };
                    current_block = 6717214610478484138;
                }
            } else {
                current_block = 6717214610478484138;
            }
            match current_block {
                971598175612140897 => {}
                _ => {
                    detached = {
                        let flag = 'd' as i32 as u_char;
                        args.argument_flag_count(flag)
                    };
                    if let Some(client) = &c {
                        if unsafe { client.flags() & CLIENT_CONTROL as uint64_t != 0 } {
                            is_control = 1 as core::ffi::c_int;
                        }
                    } else {
                        detached = 1 as core::ffi::c_int;
                    }
                    already_attached = 0 as core::ffi::c_int;
                    if {
                        c.is_some()
                            && !c
                                .as_ref()
                                .expect("the command has a client")
                                .attached_session()
                                .is_none()
                    } {
                        already_attached = 1 as core::ffi::c_int;
                    }
                    if let Some(tmp) = {
                        let flag = 'c' as i32 as u_char;
                        args.argument_flag_string(flag)
                    } {
                        unsafe {
                            cwd = Some({
                                let mut ft = format_create_for_client(
                                    item.client().as_ref(),
                                    Some(item),
                                    0,
                                    0,
                                );
                                format_defaults_for_handles(&mut ft, c.as_ref(), None, None, None);
                                format_expand(&mut ft, tmp)
                            })
                        };
                    } else {
                        unsafe {
                            cwd = Some(match c.as_ref() {
                                Some(client) => client.working_directory(),
                                None => client_working_directory(None, None),
                            })
                        };
                    }
                    if unsafe {
                        detached == 0
                            && already_attached == 0
                            && c.as_ref().expect("the command has a client").fd()
                                != -(1 as core::ffi::c_int)
                            && !c.as_ref().expect("the command has a client").flags()
                                & CLIENT_CONTROL as uint64_t
                                != 0
                    } {
                        if unsafe { c.as_mut().expect("the command has a client").is_nested() } {
                            unsafe {
                                item.error(
                                    c"sessions should be nested with care, unset $TMUX to force",
                                    fmt_args![],
                                )
                            };
                            current_block = 971598175612140897;
                        } else {
                            if unsafe {
                                tcgetattr(
                                    c.as_ref().expect("the command has a client").fd(),
                                    &raw mut tio,
                                ) != 0 as core::ffi::c_int
                            } {
                                fatal(c"tcgetattr failed", fmt_args![]);
                            }
                            tiop = Some(&tio);
                            current_block = 6545907279487748450;
                        }
                    } else {
                        tiop = None;
                        current_block = 6545907279487748450;
                    }
                    match current_block {
                        971598175612140897 => {}
                        _ => {
                            if detached == 0 && already_attached == 0 {
                                let mut terminal_cause = None;
                                if unsafe {
                                    c.as_mut()
                                        .expect("the command has a client")
                                        .open_terminal(&mut terminal_cause)
                                        != 0 as core::ffi::c_int
                                } {
                                    unsafe {
                                        item.error(
                                            c"open terminal failed: %s",
                                            fmt_args![terminal_cause.as_deref()],
                                        )
                                    };
                                    current_block = 971598175612140897;
                                } else {
                                    current_block = 5181772461570869434;
                                }
                            } else {
                                current_block = 5181772461570869434;
                            }
                            match current_block {
                                971598175612140897 => {}
                                _ => {
                                    if ({
                                        let flag = 'x' as i32 as u_char;
                                        args.argument_flag_count(flag)
                                    }) != 0
                                    {
                                        let tmp = {
                                            let flag = 'x' as i32 as u_char;
                                            args.argument_flag_string(flag)
                                        };
                                        if tmp == Some(c"-") {
                                            if let Some(client) = &c {
                                                dsx = client.terminal_size().width;
                                            } else {
                                                dsx = 80 as u_int;
                                            }
                                            current_block = 5873035170358615968;
                                        } else {
                                            match unsafe {
                                                strtonum(
                                                    tmp.unwrap_or(c""),
                                                    1 as core::ffi::c_longlong,
                                                    USHRT_MAX as core::ffi::c_longlong,
                                                )
                                            } {
                                                Ok(value) => {
                                                    dsx = value as u_int;
                                                    current_block = 5873035170358615968;
                                                }
                                                Err(errstr) => {
                                                    unsafe {
                                                        item.error(c"width %s", fmt_args![errstr])
                                                    };
                                                    current_block = 971598175612140897;
                                                }
                                            }
                                        }
                                    } else {
                                        dsx = 80 as u_int;
                                        current_block = 5873035170358615968;
                                    }
                                    match current_block {
                                        971598175612140897 => {}
                                        _ => {
                                            if ({
                                                let flag = 'y' as i32 as u_char;
                                                args.argument_flag_count(flag)
                                            }) != 0
                                            {
                                                let tmp = {
                                                    let flag = 'y' as i32 as u_char;
                                                    args.argument_flag_string(flag)
                                                };
                                                if tmp == Some(c"-") {
                                                    if let Some(client) = &c {
                                                        dsy = client.terminal_size().height;
                                                    } else {
                                                        dsy = 24 as u_int;
                                                    }
                                                    current_block = 15855550149339537395;
                                                } else {
                                                    match unsafe {
                                                        strtonum(
                                                            tmp.unwrap_or(c""),
                                                            1 as core::ffi::c_longlong,
                                                            USHRT_MAX as core::ffi::c_longlong,
                                                        )
                                                    } {
                                                        Ok(value) => {
                                                            dsy = value as u_int;
                                                            current_block = 15855550149339537395;
                                                        }
                                                        Err(errstr) => {
                                                            unsafe {
                                                                item.error(
                                                                    c"height %s",
                                                                    fmt_args![errstr],
                                                                )
                                                            };
                                                            current_block = 971598175612140897;
                                                        }
                                                    }
                                                }
                                            } else {
                                                dsy = 24 as u_int;
                                                current_block = 15855550149339537395;
                                            }
                                            match current_block {
                                                971598175612140897 => {}
                                                _ => {
                                                    if detached == 0 && is_control == 0 {
                                                        sx = c
                                                            .as_ref()
                                                            .expect("the command has a client")
                                                            .terminal_size()
                                                            .width;
                                                        sy = c
                                                            .as_ref()
                                                            .expect("the command has a client")
                                                            .terminal_size()
                                                            .height;
                                                        if unsafe {
                                                            sy > 0 as u_int
                                                            && (global_session_options().as_ref().expect("global options are initialized")).number(
                                                                c"status",
                                                            ) != 0
                                                        } {
                                                            sy = sy.wrapping_sub(1);
                                                        }
                                                    } else {
                                                        let default_size = unsafe {
                                                            (global_session_options().as_ref().expect(
                                                                "global options are initialized",
                                                            ))
                                                            .string_ref(c"default-size")
                                                        };
                                                        if unsafe {
                                                            sscanf(
                                                                default_size.as_ptr(),
                                                                c"%ux%u".as_ptr(),
                                                                &raw mut sx,
                                                                &raw mut sy,
                                                            ) != 2 as core::ffi::c_int
                                                        } {
                                                            sx = dsx;
                                                            sy = dsy;
                                                        } else {
                                                            if ({
                                                                let flag = 'x' as i32 as u_char;
                                                                args.argument_flag_count(flag)
                                                            }) != 0
                                                            {
                                                                sx = dsx;
                                                            }
                                                            if ({
                                                                let flag = 'y' as i32 as u_char;
                                                                args.argument_flag_count(flag)
                                                            }) != 0
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
                                                    unsafe {
                                                        oo = Some(RustOptionsEngine.create(
                                                            global_session_options().as_ref(),
                                                        ))
                                                    };
                                                    if ({
                                                        let flag = 'x' as i32 as u_char;
                                                        args.argument_flag_count(flag)
                                                    }) != 0
                                                        || ({
                                                            let flag = 'y' as i32 as u_char;
                                                            args.argument_flag_count(flag)
                                                        }) != 0
                                                    {
                                                        if ({
                                                            let flag = 'x' as i32 as u_char;
                                                            args.argument_flag_count(flag)
                                                        }) == 0
                                                        {
                                                            dsx = sx;
                                                        }
                                                        if ({
                                                            let flag = 'y' as i32 as u_char;
                                                            args.argument_flag_count(flag)
                                                        }) == 0
                                                        {
                                                            dsy = sy;
                                                        }
                                                        unsafe {
                                                            (oo.as_ref().expect(
                                                                "session options were created",
                                                            ))
                                                            .set_string(
                                                                c"default-size",
                                                                0 as core::ffi::c_int,
                                                                c"%ux%u",
                                                                fmt_args![dsx, dsy],
                                                            )
                                                        };
                                                    }
                                                    env = Some(new_environment_box());
                                                    if let Some(client) = &c
                                                        && ({
                                                            let flag = 'E' as i32 as u_char;
                                                            args.argument_flag_count(flag)
                                                        }) == 0
                                                    {
                                                        unsafe {
                                                            client.update_environment(
                                                            global_session_options().as_ref().expect("global options are initialized"),
                                                            env.as_deref_mut().unwrap(),
                                                        )
                                                        };
                                                    }
                                                    for av in {
                                                        let flag = 'e' as i32 as u_char;
                                                        args.argument_flag_values(flag)
                                                    } {
                                                        env.as_deref_mut()
                                                            .unwrap()
                                                            .put(av.string(), 0);
                                                    }
                                                    let Some(env) = env.take() else {
                                                        unreachable!();
                                                    };
                                                    let Some(oo) = oo.take() else {
                                                        unreachable!();
                                                    };
                                                    let created = unsafe {
                                                        SessionRef::create(
                                                            prefix.as_deref(),
                                                            sname.as_deref(),
                                                            cwd.as_deref().expect("cwd is set"),
                                                            env,
                                                            oo,
                                                            tiop,
                                                        )
                                                    };
                                                    sc.item = cmdq_item_weak_of(item);
                                                    sc.s = Some(created.clone());
                                                    if detached == 0 {
                                                        sc.tc = cmdq_client
                                                            .as_ref()
                                                            .map(ClientRef::downgrade);
                                                    }
                                                    sc.name = wname.as_deref();
                                                    unsafe { sc.argv = args.to_vector() };
                                                    sc.idx = -(1 as core::ffi::c_int);
                                                    sc.cwd = {
                                                        let flag = 'c' as i32 as u_char;
                                                        args.argument_flag_string(flag)
                                                    };
                                                    sc.flags = 0 as core::ffi::c_int;
                                                    if unsafe {
                                                        spawn_window(&mut sc, &mut cause).is_none()
                                                    } {
                                                        unsafe {
                                                            created.destroy(
                                                                0 as core::ffi::c_int,
                                                                c"cmd_new_session_exec",
                                                            )
                                                        };
                                                        let cause = cause.unwrap();
                                                        unsafe {
                                                            item.error(
                                                                c"create window failed: %s",
                                                                fmt_args![cause.as_c_str()],
                                                            )
                                                        };
                                                    } else {
                                                        if let Some(group) = group {
                                                            {
                                                                join_session_group(
                                                                    group,
                                                                    group_name.as_deref(),
                                                                    groupwith.as_ref(),
                                                                    &created,
                                                                )
                                                            };
                                                            unsafe {
                                                                created.synchronize_group_to()
                                                            };
                                                            let index = unsafe {
                                                                created.first_index().expect(
                                                                    "the group has a window",
                                                                )
                                                            };
                                                            unsafe { created.select(index) };
                                                        }
                                                        unsafe {
                                                            created.notify(c"session-created")
                                                        };
                                                        if detached == 0 {
                                                            if ({
                                                                let flag = 'f' as i32 as u_char;
                                                                args.argument_flag_count(flag)
                                                            }) != 0
                                                                && let Some(fflag) = {
                                                                    let flag = 'f' as i32 as u_char;
                                                                    args.argument_flag_string(flag)
                                                                }
                                                            {
                                                                unsafe {
                                                                    c.as_mut().expect("the command has a client").apply_flags(fflag)
                                                                };
                                                            }
                                                            if already_attached == 0 {
                                                                unsafe {
                                                                    c.as_ref().expect("the command has a client").send_ready()
                                                                };
                                                            } else if {
                                                                !c.as_ref()
                                                                    .expect(
                                                                        "the command has a client",
                                                                    )
                                                                    .attached_session()
                                                                    .is_none()
                                                            } {
                                                                let _last_session = unsafe {
                                                                    c.as_mut().expect("the command has a client").remember_session()
                                                                };
                                                            }
                                                            unsafe {
                                                                c.as_mut()
                                                                    .expect(
                                                                        "the command has a client",
                                                                    )
                                                                    .set_session(Some(&created))
                                                            };
                                                            if !item.flags() & CMDQ_STATE_REPEAT
                                                                != 0
                                                            {
                                                                unsafe {
                                                                    c.as_mut().expect("the command has a client").set_key_table(None)
                                                                };
                                                            }
                                                        }
                                                        if ({
                                                            let flag = 'P' as i32 as u_char;
                                                            args.argument_flag_count(flag)
                                                        }) != 0
                                                        {
                                                            let template = {
                                                                let flag = 'F' as i32 as u_char;
                                                                args.argument_flag_string(flag)
                                                            }
                                                            .unwrap_or(NEW_SESSION_TEMPLATE);
                                                            let cp = unsafe {
                                                                let link = created.curw();
                                                                let mut ft =
                                                                    format_create_for_client(
                                                                        item.client().as_ref(),
                                                                        Some(item),
                                                                        0,
                                                                        0,
                                                                    );
                                                                format_defaults_for_handles(
                                                                    &mut ft,
                                                                    c.as_ref(),
                                                                    Some(&created),
                                                                    link.as_ref(),
                                                                    None,
                                                                );
                                                                format_expand(&mut ft, template)
                                                            };
                                                            unsafe {
                                                                item.print(
                                                                    c"%s",
                                                                    fmt_args![cp.as_c_str()],
                                                                )
                                                            };
                                                        }
                                                        if detached == 0 {
                                                            unsafe {
                                                                c.as_mut()
                                                                    .expect(
                                                                        "the command has a client",
                                                                    )
                                                                    .mark_attached()
                                                            };
                                                        }
                                                        if ({
                                                            let flag = 'd' as i32 as u_char;
                                                            args.argument_flag_count(flag)
                                                        }) == 0
                                                        {
                                                            unsafe {
                                                                current_state_ref
                                                                    .update_current_session(
                                                                        &created, 0,
                                                                    )
                                                            };
                                                        }
                                                        unsafe {
                                                            cmd_find_from_session_ref(
                                                                &mut fs,
                                                                &created,
                                                                0 as core::ffi::c_int,
                                                            )
                                                        };
                                                        unsafe {
                                                            (crate::cmd::cmdq_item_ref_of(item)
                                                                .expect("the command has an owner"))
                                                            .insert_session_hook(
                                                                Some(&created),
                                                                Some(&fs),
                                                                c"after-new-session",
                                                                fmt_args![],
                                                            )
                                                        };
                                                        if configuration_finished() {
                                                            unsafe {
                                                                cfg_show_causes_for_session(Some(
                                                                    &created,
                                                                ))
                                                            };
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
    CMD_RETURN_ERROR
}
