use crate::src::arguments::args_has;
use crate::src::cmd::cmd_get_args_mut;
use crate::src::cmd::find::cmd_find_from_session;
use crate::src::cmd::queue::{cmdq_error, cmdq_get_source, cmdq_get_state_owned, cmdq_get_target};
use crate::src::events::events_fire_window;
use crate::src::format::bytes::write_cstr;
use crate::src::layout::{layout_assign_pane, layout_close_pane, layout_get_tiled_cell};
use crate::src::options::options_set_parent;
use crate::src::resize::recalculate_sizes;
use crate::src::server_client::Client as _;
use crate::src::server_fn::{
    server_kill_window, server_redraw_session, server_redraw_window, server_status_session,
    server_unzoom_window,
};
use crate::src::window::Window as _;

use crate::src::session::Session;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::client::ClientRef;
use crate::src::shared::command::CMD_FIND_DEFAULT_MARKED;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::session::session;
use crate::src::shared::session::SessionRef;
use crate::src::shared::spawn::{SPAWN_BEFORE, SPAWN_FULLSIZE, SPAWN_HORIZONTAL};
use crate::src::shared::window::WindowRef;
use crate::src::shared::window::WINDOW_ZOOMED;
use crate::src::shared::window::{window, winlink};
use crate::src::window_pane::WindowPane as _;
pub static cmd_join_pane_entry: cmd_entry = {
    cmd_entry {
        name: c"join-pane",
        alias: Some(c"joinp"),
        args: args_parse {
            template: c"bdfhvp:l:s:t:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-bdfhv] [-l size] [-s src-pane] [-t dst-pane]",
        source: cmd_entry_flag {
            flag: 's' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: CMD_FIND_DEFAULT_MARKED,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(cmd_join_pane_exec),
    }
};
unsafe fn cmd_join_pane_exec(
    mut self_0: refbox::Weak<cmd>,
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
) -> cmd_retval {
    let item = item_handle.get();
    let mut args: *mut args =
        cmd_get_args_mut(self_0.get_mut_unchecked()).map_or(std::ptr::null_mut(), |args| args);
    let current = cmdq_get_state_owned(&*(item));
    let mut target: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let mut source: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_source_mut(&mut *item);
    let mut dst_s: Option<SessionRef> = None;
    let mut src_wl: refbox::Weak<winlink> = refbox::Weak::new();
    let mut dst_wl: refbox::Weak<winlink> = refbox::Weak::new();
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut dst_idx: ::core::ffi::c_int = 0;
    dst_s = (*target).session_handle();
    dst_wl = (*target).winlink_handle();
    let dst_pane_owner = (*target).pane_handle().expect("join target pane");
    let dst_window = dst_wl
        .get_unchecked()
        .window_handle()
        .cloned()
        .expect("live window");
    let mut src_window = None;
    let result = (|| {
        dst_idx = dst_wl.get_unchecked().idx;
        src_wl = (*source).winlink_handle();
        let src_pane_owner = (*source).pane_handle().expect("join source pane");
        src_window = src_wl.get_unchecked().window_handle().cloned();
        let src_owner = src_window.as_ref().expect("live source window");
        server_unzoom_window(&dst_window);
        server_unzoom_window(src_owner);
        if std::rc::Rc::ptr_eq(&src_pane_owner, &dst_pane_owner) {
            cmdq_error(item_handle, |out| {
                out.write_all(b"source and target panes must be different")
            });
            return CMD_RETURN_ERROR;
        }
        if args_has(args, 'h' as i32 as u_char) != 0 {
            flags |= SPAWN_HORIZONTAL;
        }
        if args_has(args, 'b' as i32 as u_char) != 0 {
            flags |= SPAWN_BEFORE;
        }
        if args_has(args, 'f' as i32 as u_char) != 0 {
            flags |= SPAWN_FULLSIZE;
        }
        let layout_id =
            match layout_get_tiled_cell(item_handle, args, &dst_window, &dst_pane_owner, flags) {
                Ok(cell) => cell,
                Err(cause) => {
                    cmdq_error(item_handle, |out| {
                        out.write_all(b"size or position ")?;
                        write_cstr(out, cause.as_ptr())
                    });
                    return CMD_RETURN_ERROR;
                }
            };
        layout_close_pane(&src_pane_owner);
        ClientRef::forget_pane(&src_pane_owner);
        src_owner.forget_pane(&src_pane_owner);
        assert!(
            src_owner
                .borrow_pane_order_mut()
                .remove(&std::rc::Rc::downgrade(&src_pane_owner)),
            "pane is not in its window order"
        );
        src_pane_owner.reparent(&dst_window);
        if flags & SPAWN_BEFORE != 0 {
            dst_window.borrow_pane_order_mut().insert_before(
                &std::rc::Rc::downgrade(&dst_pane_owner),
                std::rc::Rc::downgrade(&src_pane_owner),
            );
        } else {
            dst_window.borrow_pane_order_mut().insert_after(
                &std::rc::Rc::downgrade(&dst_pane_owner),
                std::rc::Rc::downgrade(&src_pane_owner),
            );
        }
        layout_assign_pane(
            &dst_window,
            layout_id,
            &src_pane_owner,
            0 as ::core::ffi::c_int,
        );
        src_pane_owner.refresh_palette();
        recalculate_sizes();
        server_redraw_window(src_owner);
        server_redraw_window(&dst_window);
        if args_has(args, 'd' as i32 as u_char) == 0 {
            dst_window.select_pane(&src_pane_owner, true);
            (dst_s.as_ref().expect("live session")).select_index(dst_idx);
            cmd_find_from_session(
                &mut *current.current.borrow_mut(),
                dst_s.as_ref().expect("live session"),
                0 as ::core::ffi::c_int,
            );
            server_redraw_session(dst_s.as_ref().expect("live session"));
        } else {
            server_status_session(dst_s.as_ref().expect("live session"));
        }
        src_pane_owner.notify_moved(src_owner, src_wl.get_unchecked().idx, &dst_window, dst_idx);
        cmd_join_pane_finish(src_window.take().expect("live source window"), &dst_window);
        CMD_RETURN_NORMAL
    })();
    if let Some(window) = src_window {
        window.release(c"join pane source");
    }
    dst_window.release(c"join pane destination");
    result
}

// Consume the retained source at the established notification boundary. In the
// empty case its close notification and explicit cleanup precede destination
// layout notification, even when no Session still retains it.
unsafe fn cmd_join_pane_finish(source: WindowRef, destination: &WindowRef) {
    if source.pane_snapshot().is_empty() {
        server_kill_window(source, 1);
    } else {
        events_fire_window(c"window-layout-changed".as_ptr(), source);
    }
    events_fire_window(
        c"window-layout-changed".as_ptr(),
        std::rc::Rc::clone(destination),
    );
}
