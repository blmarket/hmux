use crate::src::arguments::args_has;
use crate::src::cmd::cmd_get_args_mut;
use crate::src::cmd::queue::{cmdq_error, cmdq_get_source, cmdq_get_target};
use crate::src::resize::recalculate_sizes;
use crate::src::server::marked_pane;
use crate::src::server_fn::server_redraw_session_group;
use crate::src::session::{session_group_contains, session_group_synchronize_from, session_select};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::command::{CMD_AFTERHOOK, CMD_FIND_DEFAULT_MARKED};
use crate::src::shared::session::session;
use crate::src::shared::session::session_group;
use crate::src::shared::window::{window, winlink};
use crate::src::window::{window_winlinks_append, window_winlinks_remove};
pub static cmd_swap_window_entry: cmd_entry = {
    cmd_entry {
        name: c"swap-window",
        alias: Some(c"swapw"),
        args: args_parse {
            template: c"ds:t:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-d] [-s src-window] [-t dst-window]",
        source: cmd_entry_flag {
            flag: 's' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: CMD_FIND_DEFAULT_MARKED,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK,
        exec: Some(cmd_swap_window_exec),
    }
};
unsafe fn cmd_swap_window_exec(mut self_0: *mut cmd, item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>) -> cmd_retval {
    let item = item_handle.get();
    let mut args: *mut args = cmd_get_args_mut(&mut *self_0).map_or(std::ptr::null_mut(), |args| args);
    let mut source: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_source_mut(&mut *item);
    let mut target: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let mut src: *mut session = (*source).session_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut dst: *mut session = (*target).session_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut sg_src: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut sg_dst: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut wl_src: *mut winlink = (*source).wl_ptr();
    let mut wl_dst: *mut winlink = (*target).wl_ptr();
    let mut w_src: *mut window = ::core::ptr::null_mut::<window>();
    let mut w_dst: *mut window = ::core::ptr::null_mut::<window>();
    sg_src = session_group_contains((src).as_ref());
    sg_dst = session_group_contains((dst).as_ref());
    if src != dst && !sg_src.is_null() && !sg_dst.is_null() && sg_src == sg_dst {
        cmdq_error(item_handle, |out| {
            out.write_all(b"can't move window, sessions are grouped")
        });
        return CMD_RETURN_ERROR;
    }
    if (*wl_dst).window_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()) == (*wl_src).window_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()) {
        return CMD_RETURN_NORMAL;
    }
    w_dst = (*wl_dst).window_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    w_src = (*wl_src).window_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    window_winlinks_remove(&mut *(w_dst), wl_dst);
    window_winlinks_remove(&mut *(w_src), wl_src);
    if wl_dst != wl_src {
        std::mem::swap(&mut (*wl_dst).window_owner, &mut (*wl_src).window_owner);
    }
    window_winlinks_append(&mut *(w_src), wl_dst);
    window_winlinks_append(&mut *(w_dst), wl_src);
    if marked_pane.wl_ptr() == wl_src {
        marked_pane.set_wl(wl_dst);
    }
    if args_has(args, 'd' as i32 as u_char) != 0 {
        session_select(&(*dst).observer.upgrade().expect("live session"), (*wl_dst).idx);
        if src != dst {
            session_select(&(*src).observer.upgrade().expect("live session"), (*wl_src).idx);
        }
    }
    session_group_synchronize_from(&(*src).observer.upgrade().expect("live session"));
    server_redraw_session_group(&*(src));
    if src != dst {
        session_group_synchronize_from(&(*dst).observer.upgrade().expect("live session"));
        server_redraw_session_group(&*(dst));
    }
    recalculate_sizes();
    return CMD_RETURN_NORMAL;
}
