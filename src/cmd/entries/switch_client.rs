use crate::src::arguments::{args_get, args_has};
use crate::src::cmd::cmd_get_args_mut;
use crate::src::cmd::find::{cmd_find_from_session, cmd_find_target};
use crate::src::cmd::queue::{
    cmdq_error, cmdq_get_client, cmdq_get_flags, cmdq_get_state_owned, cmdq_get_target_client,
};
use crate::src::environ::environ_update;
use crate::src::ffi::libc::{getuid, strcmp, strcspn};
use crate::src::format::bytes::write_cstr;
use crate::src::key_bindings::key_bindings_get_table;
use crate::src::options::options_owner_ptr;
use crate::src::proc::proc_get_peer_uid;
use crate::src::server_client::{server_client_set_key_table, server_client_set_session};
use crate::src::server_fn::server_redraw_window;
use crate::src::session::{session_next_session, session_previous_session, session_set_current};
use crate::src::shared::abi::uid_t;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::client::{CLIENT_IGNORESIZE, CLIENT_READONLY};
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::command::{
    CMDQ_STATE_REPEAT, CMD_CLIENT_CFLAG, CMD_FIND_PREFER_UNATTACHED, CMD_READONLY,
};
use crate::src::shared::key::key_table;
use crate::src::shared::pane::window_pane;
use crate::src::shared::session::session;
use crate::src::shared::sort::sort_criteria;
use crate::src::shared::sort::*;
use crate::src::shared::window::{window, winlink};
use crate::src::sort::sort_order_from_string;
use crate::src::window::{
    window_pane_is_visible, window_pop_zoom, window_push_zoom, window_redraw_active_switch,
    window_set_active_pane,
};
pub static cmd_switch_client_entry: cmd_entry = {
    cmd_entry {
        name: c"switch-client",
        alias: Some(c"switchc"),
        args: args_parse {
            template: c"c:EFlnO:pt:rT:Z",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-ElnprZ] [-c target-client] [-t target-session] [-T key-table] [-O order]",
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
        flags: CMD_READONLY | CMD_CLIENT_CFLAG,
        exec: Some(cmd_switch_client_exec),
    }
};
unsafe fn cmd_switch_client_exec(
    mut self_0: refbox::Weak<cmd>,
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
) -> cmd_retval {
    let item = item_handle.get();
    let mut args: *mut args =
        cmd_get_args_mut(self_0.get_mut_unchecked()).map_or(std::ptr::null_mut(), |args| args);
    let current = cmdq_get_state_owned(&*(item));
    let mut target: cmd_find_state = cmd_find_state {
        flags: 0,
        s: Default::default(),
        wl: Default::default(),
        w: Default::default(),
        wp: Default::default(),
        idx: 0,
    };
    let mut tflag: *const ::core::ffi::c_char =
        args_get(&*(args), 't' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
    let mut type_0: cmd_find_type = CMD_FIND_PANE;
    let mut flags: ::core::ffi::c_int = 0;
    let mut visible: ::core::ffi::c_int = 0;
    let mut Zflag: ::core::ffi::c_int = args_has(args, 'Z' as i32 as u_char);
    let c_owner = cmdq_get_client((item).as_ref());
    let mut c: *mut client = c_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    let tc_owner = cmdq_get_target_client((item).as_ref());
    let mut tc: *mut client = tc_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    let last_session_owner;
    let adjacent_session_owner;
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: refbox::Weak<winlink> = refbox::Weak::new();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut tablename: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut sort_crit: sort_criteria = sort_criteria {
        order: SORT_ACTIVITY,
        reversed: 0,
        order_seq: &[],
    };
    let mut uid: uid_t = 0;
    if !tflag.is_null()
        && (*tflag
            .offset(strcspn(tflag, b":.%\0" as *const u8 as *const ::core::ffi::c_char) as isize)
            as ::core::ffi::c_int
            != '\0' as i32
            || strcmp(tflag, b"=\0" as *const u8 as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int)
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
    s = target
        .session_handle()
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    wl = target.winlink_handle();
    wp = target
        .pane_handle()
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    if args_has(args, 'r' as i32 as u_char) != 0 {
        if (*tc).flags & CLIENT_READONLY as uint64_t != 0 {
            uid = proc_get_peer_uid((*c).peer);
            if uid != getuid() {
                cmdq_error(item_handle, |out| out.write_all(b"client is read-only"));
                return CMD_RETURN_ERROR;
            }
        }
        if (*tc).flags & CLIENT_READONLY as uint64_t != 0 {
            (*tc).flags &= !(CLIENT_READONLY | CLIENT_IGNORESIZE) as uint64_t;
        } else {
            (*tc).flags |= (CLIENT_READONLY | CLIENT_IGNORESIZE) as uint64_t;
        }
    }
    tablename =
        args_get(&*(args), 'T' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
    if !tablename.is_null() {
        let Some(table) = key_bindings_get_table(std::ffi::CStr::from_ptr(tablename), 0) else {
            cmdq_error(item_handle, |out| {
                out.write_all(b"table ")?;
                write_cstr(out, tablename)?;
                out.write_all(b" doesn't exist")
            });
            return CMD_RETURN_ERROR;
        };
        (*tc).keytable = Some(table);
        return CMD_RETURN_NORMAL;
    }
    sort_crit.order = sort_order_from_string(
        args_get(&*(args), 'O' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr()),
    );
    if sort_crit.order as ::core::ffi::c_uint
        == SORT_END as ::core::ffi::c_int as ::core::ffi::c_uint
        && args_has(args, 'O' as i32 as u_char) != 0
    {
        cmdq_error(item_handle, |out| out.write_all(b"invalid sort order"));
        return CMD_RETURN_ERROR;
    }
    sort_crit.reversed = args_has(args, 'r' as i32 as u_char);
    if args_has(args, 'n' as i32 as u_char) != 0 {
        adjacent_session_owner = session_next_session(
            (*tc)
                .session_handle()
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get())
                .as_ref(),
            &sort_crit,
        );
        s = adjacent_session_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
        if s.is_null() {
            cmdq_error(item_handle, |out| out.write_all(b"can't find next session"));
            return CMD_RETURN_ERROR;
        }
    } else if args_has(args, 'p' as i32 as u_char) != 0 {
        adjacent_session_owner = session_previous_session(
            (*tc)
                .session_handle()
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get())
                .as_ref(),
            &sort_crit,
        );
        s = adjacent_session_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
        if s.is_null() {
            cmdq_error(item_handle, |out| {
                out.write_all(b"can't find previous session")
            });
            return CMD_RETURN_ERROR;
        }
    } else if args_has(args, 'l' as i32 as u_char) != 0 {
        last_session_owner = crate::src::session::sessions_resolve(
            &crate::src::session::sessions,
            &(*tc).last_session,
        );
        s = last_session_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
        if s.is_null() {
            cmdq_error(item_handle, |out| out.write_all(b"can't find last session"));
            return CMD_RETURN_ERROR;
        }
    } else {
        if cmdq_get_client((item).as_ref()).is_none() {
            return CMD_RETURN_NORMAL;
        }
        if wl.is_alive()
            && !wp.is_null()
            && wp
                != (*wl
                    .get_unchecked()
                    .window_handle()
                    .as_ref()
                    .map_or(std::ptr::null_mut(), |owner| owner.get()))
                .active_pane()
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get())
        {
            w = wl
                .get_unchecked()
                .window_handle()
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get());
            if (*w).modal.upgrade().is_some()
                && !wp
                    .as_ref()
                    .is_some_and(|pane| (*w).modal.ptr_eq(&pane.observer))
            {
                visible = 1 as ::core::ffi::c_int;
            } else {
                visible = window_pane_is_visible(&target.pane_handle().expect("target pane"));
            }
            if visible == 0
                && window_push_zoom(
                    &(*(w)).observer.upgrade().expect("live window"),
                    0 as ::core::ffi::c_int,
                    Zflag,
                ) != 0
            {
                server_redraw_window(&*(w));
            }
            window_redraw_active_switch(
                &(*(w)).observer.upgrade().expect("live window"),
                (wp).as_ref()
                    .and_then(|model| model.observer.upgrade())
                    .as_ref(),
            );
            window_set_active_pane(
                &(*(w)).observer.upgrade().expect("live window"),
                &(*(wp)).observer.upgrade().expect("live window_pane"),
                1 as ::core::ffi::c_int,
            );
            if visible == 0
                && window_pop_zoom(&(*(w)).observer.upgrade().expect("live window")) != 0
            {
                server_redraw_window(&*(w));
            }
        }
        if wl.is_alive() {
            session_set_current(&(*s).observer.upgrade().expect("live session"), wl.clone());
            cmd_find_from_session(
                &mut *current.current.borrow_mut(),
                &(*(s)).observer.upgrade().expect("live session"),
                0 as ::core::ffi::c_int,
            );
        }
    }
    if args_has(args, 'E' as i32 as u_char) == 0 {
        environ_update(
            options_owner_ptr(&mut (*s).options).map_or(std::ptr::null_mut(), |options| options),
            (*tc).environ.as_deref().expect("client environment"),
            (*s).environ.as_deref_mut().expect("session environment"),
        );
    }
    server_client_set_session(
        &(*(tc)).observer.upgrade().expect("live client"),
        (s).as_ref()
            .and_then(|model| model.observer.upgrade())
            .as_ref(),
    );
    if !cmdq_get_flags(&*(item)) & CMDQ_STATE_REPEAT != 0 {
        server_client_set_key_table(
            &(*(tc)).observer.upgrade().expect("live client"),
            ::core::ptr::null::<::core::ffi::c_char>(),
        );
    }
    return CMD_RETURN_NORMAL;
}
