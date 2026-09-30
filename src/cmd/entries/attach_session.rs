use crate::src::arguments::{args_get, args_has};
use crate::src::cfg::{cfg_finished, cfg_show_causes};
use crate::src::cmd::cmd_get_args_mut;
use crate::src::cmd::find::{cmd_find_from_winlink, cmd_find_from_winlink_pane, cmd_find_target};
use crate::src::cmd::queue::{cmdq_error, cmdq_get_client, cmdq_get_flags, cmdq_get_state_owned};
use crate::src::compat::imsg::*;
use crate::src::environ::environ_update;
use crate::src::events::events_fire_client;
use crate::src::ffi::libc::{getuid, strcspn};
use crate::src::format::bytes::write_cstr;
use crate::src::format::format_single_cstring;
use crate::src::options::options_owner_ptr;
use crate::src::proc::{proc_get_peer_uid, proc_send};
use crate::src::server::clients;
use crate::src::server_client::Client as _;
use crate::src::server_client::{
    server_client_detach, server_client_open, server_client_set_flags, server_client_set_key_table,
    server_client_set_session,
};
use crate::src::session::session_set_current;
use crate::src::session::sessions;
use crate::src::shared::abi::uid_t;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::client::ClientRef;
use crate::src::shared::client::{
    CLIENT_ATTACHED, CLIENT_CONTROL, CLIENT_IGNORESIZE, CLIENT_READONLY,
};
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::command::{
    CMDQ_STATE_REPEAT, CMD_FIND_PREFER_UNATTACHED, CMD_READONLY, CMD_STARTSERVER,
};
use crate::src::shared::pane::window_pane;
use crate::src::shared::session::session;
use crate::src::shared::session::SessionRef;
use crate::src::shared::window::{window, winlink};
use crate::src::window::window_set_active_pane;
use crate::src::{server_client::Client, session::Session};
pub static cmd_attach_session_entry: cmd_entry = {
    cmd_entry {
        name: c"attach-session",
        alias: Some(c"attach"),
        args: args_parse {
            template: c"c:dEf:rt:x",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-dErx] [-c working-directory] [-f flags] [-t target-session]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        flags: CMD_STARTSERVER | CMD_READONLY,
        exec: Some(cmd_attach_session_exec),
    }
};
pub unsafe fn cmd_attach_session(
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
    mut tflag: *const ::core::ffi::c_char,
    mut dflag: ::core::ffi::c_int,
    mut xflag: ::core::ffi::c_int,
    mut rflag: ::core::ffi::c_int,
    mut cflag: *const ::core::ffi::c_char,
    mut Eflag: ::core::ffi::c_int,
    mut fflag: *const ::core::ffi::c_char,
) -> cmd_retval {
    let item = item_handle.get();
    let current = cmdq_get_state_owned(&*(item));
    let mut target: cmd_find_state = cmd_find_state {
        flags: 0,
        s: Default::default(),
        wl: Default::default(),
        w: Default::default(),
        wp: Default::default(),
        idx: 0,
    };
    let mut type_0: cmd_find_type = CMD_FIND_PANE;
    let mut flags: ::core::ffi::c_int = 0;
    let c_owner = cmdq_get_client((item).as_ref());
    let mut c: Option<ClientRef> = c_owner.clone();
    let mut c_loop: Option<ClientRef> = None;
    let mut s: Option<SessionRef> = None;
    let mut wl: refbox::Weak<winlink> = refbox::Weak::new();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut msgtype: msgtype = 0 as msgtype;
    let mut uid: uid_t = 0;
    if sessions.storage.is_none() {
        cmdq_error(item_handle, |out| out.write_all(b"no sessions"));
        return CMD_RETURN_ERROR;
    }
    if c.is_none() {
        return CMD_RETURN_NORMAL;
    }
    if c.as_ref().expect("live client").is_nested() {
        cmdq_error(item_handle, |out| {
            out.write_all(b"sessions should be nested with care, unset $TMUX to force")
        });
        return CMD_RETURN_ERROR;
    }
    if !tflag.is_null()
        && *tflag
            .offset(strcspn(tflag, b":.\0" as *const u8 as *const ::core::ffi::c_char) as isize)
            as ::core::ffi::c_int
            != '\0' as i32
    {
        type_0 = CMD_FIND_PANE;
        flags = 0 as ::core::ffi::c_int;
    } else {
        type_0 = CMD_FIND_SESSION;
        flags = CMD_FIND_PREFER_UNATTACHED;
    }
    if cmd_find_target(&raw mut target, Some(item_handle), tflag, type_0, flags)
        != 0 as ::core::ffi::c_int
    {
        return CMD_RETURN_ERROR;
    }
    s = target.session_handle();
    wl = target.winlink_handle();
    wp = target
        .pane_handle()
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if wl.is_alive() {
        if !wp.is_null() {
            window_set_active_pane(
                &std::rc::Rc::clone(&(((*wp).window_handle().as_ref()).expect("live window"))),
                &(*(wp)).observer.upgrade().expect("live window_pane"),
                1 as ::core::ffi::c_int,
            );
        }
        session_set_current(s.as_ref().expect("live session"), wl.clone());
        if !wp.is_null() {
            cmd_find_from_winlink_pane(
                &mut *current.current.borrow_mut(),
                wl.clone(),
                &(*(wp)).observer.upgrade().expect("live window_pane"),
                0 as ::core::ffi::c_int,
            );
        } else {
            cmd_find_from_winlink(
                &mut *current.current.borrow_mut(),
                wl.clone(),
                0 as ::core::ffi::c_int,
            );
        }
    }
    if !cflag.is_null() {
        s.as_ref()
            .expect("target session")
            .set_cwd(Some(format_single_cstring(
                Some(item_handle),
                cflag,
                c.as_ref(),
                s.as_ref(),
                wl.clone(),
                (wp).as_ref()
                    .and_then(|model| model.observer.upgrade())
                    .as_ref(),
            )));
    }
    if !fflag.is_null() {
        server_client_set_flags(&c.clone().expect("live client"), fflag);
    }
    if rflag != 0 {
        if c.as_ref().expect("live client").flags() & CLIENT_READONLY as uint64_t != 0 {
            uid = c.as_ref().expect("live client").peer_uid();
            if uid != getuid() {
                cmdq_error(item_handle, |out| out.write_all(b"client is read-only"));
                return CMD_RETURN_ERROR;
            }
        }
        c.as_ref()
            .expect("live client")
            .update_flags((CLIENT_READONLY | CLIENT_IGNORESIZE) as uint64_t, 0);
    }
    c.as_ref().expect("live client").remember_session();
    if !c
        .as_ref()
        .expect("live client")
        .attached_session()
        .upgrade()
        .is_none()
    {
        if dflag != 0 || xflag != 0 {
            if xflag != 0 {
                msgtype = MSG_DETACHKILL;
            } else {
                msgtype = MSG_DETACH;
            }
            let mut registry_c_loop_owner = clients.first();
            c_loop = registry_c_loop_owner.clone();
            while !c_loop.is_none() {
                if !(!crate::src::shared::rc::same(
                    c_loop
                        .as_ref()
                        .expect("live client")
                        .attached_session()
                        .upgrade()
                        .as_ref(),
                    s.as_ref(),
                ) || crate::src::shared::rc::same(c.as_ref(), c_loop.as_ref()))
                {
                    server_client_detach(&c_loop.clone().expect("live client"), msgtype);
                }
                registry_c_loop_owner = clients.next(
                    registry_c_loop_owner
                        .as_ref()
                        .expect("current registry client"),
                );
                c_loop = registry_c_loop_owner.clone();
            }
        }
        if Eflag == 0 {
            let source = c_owner
                .as_ref()
                .expect("terminal client")
                .with_environment(|env| env.expect("client environment").clone());
            target
                .session_handle()
                .expect("target session")
                .update_environment(&source);
        }
        server_client_set_session(&c.clone().expect("live client"), s.as_ref());
        if !cmdq_get_flags(&*(item)) & CMDQ_STATE_REPEAT != 0 {
            server_client_set_key_table(
                &c.clone().expect("live client"),
                ::core::ptr::null::<::core::ffi::c_char>(),
            );
        }
    } else {
        if let Err(cause) = server_client_open(c_owner.as_ref().expect("terminal client")) {
            cmdq_error(item_handle, |out| {
                out.write_all(b"open terminal failed: ")?;
                write_cstr(out, cause.as_ptr())
            });
            return CMD_RETURN_ERROR;
        }
        if dflag != 0 || xflag != 0 {
            if xflag != 0 {
                msgtype = MSG_DETACHKILL;
            } else {
                msgtype = MSG_DETACH;
            }
            let mut registry_c_loop_owner = clients.first();
            c_loop = registry_c_loop_owner.clone();
            while !c_loop.is_none() {
                if !(!crate::src::shared::rc::same(
                    c_loop
                        .as_ref()
                        .expect("live client")
                        .attached_session()
                        .upgrade()
                        .as_ref(),
                    s.as_ref(),
                ) || crate::src::shared::rc::same(c.as_ref(), c_loop.as_ref()))
                {
                    server_client_detach(&c_loop.clone().expect("live client"), msgtype);
                }
                registry_c_loop_owner = clients.next(
                    registry_c_loop_owner
                        .as_ref()
                        .expect("current registry client"),
                );
                c_loop = registry_c_loop_owner.clone();
            }
        }
        if Eflag == 0 {
            let source = c_owner
                .as_ref()
                .expect("terminal client")
                .with_environment(|env| env.expect("client environment").clone());
            target
                .session_handle()
                .expect("target session")
                .update_environment(&source);
        }
        server_client_set_session(&c.clone().expect("live client"), s.as_ref());
        server_client_set_key_table(
            &c.clone().expect("live client"),
            ::core::ptr::null::<::core::ffi::c_char>(),
        );
        if !c.as_ref().expect("live client").flags() & CLIENT_CONTROL as uint64_t != 0 {
            c.as_ref().expect("live client").send_ready();
        }
        events_fire_client(
            b"client-attached\0" as *const u8 as *const ::core::ffi::c_char,
            c.clone().expect("live client"),
        );
        c.as_ref()
            .expect("live client")
            .update_flags(CLIENT_ATTACHED as uint64_t, 0);
    }
    if cfg_finished != 0 {
        cfg_show_causes(s.as_ref());
    }
    return CMD_RETURN_NORMAL;
}
unsafe fn cmd_attach_session_exec(
    mut self_0: refbox::Weak<cmd>,
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
) -> cmd_retval {
    let mut args: *mut args =
        cmd_get_args_mut(self_0.get_mut_unchecked()).map_or(std::ptr::null_mut(), |args| args);
    return cmd_attach_session(
        item_handle,
        args_get(&*(args), 't' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr()),
        args_has(args, 'd' as i32 as u_char),
        args_has(args, 'x' as i32 as u_char),
        args_has(args, 'r' as i32 as u_char),
        args_get(&*(args), 'c' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr()),
        args_has(args, 'E' as i32 as u_char),
        args_get(&*(args), 'f' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr()),
    );
}
