use crate::src::arguments::{args_get, args_has, args_percentage_and_expand_result};
use crate::src::cmd::find::cmd_find_from_session;
use crate::src::cmd::queue::{
    cmdq_error, cmdq_get_client, cmdq_get_event, cmdq_get_source, cmdq_get_state_owned,
    cmdq_get_target,
};
use crate::src::cmd::{cmd_get_args_mut, cmd_get_entry, cmd_mouse_pane};
use crate::src::compat::strtonum::strtonum;
use crate::src::events::events_fire_window;
use crate::src::ffi::libc::strcmp;
use crate::src::format::bytes::write_cstr;
use crate::src::layout::{
    layout_assign_pane, layout_close_pane, layout_fix_offsets, layout_fix_panes,
    layout_get_tiled_cell, layout_tile_pane,
};
use crate::src::options::options_set_parent;
use crate::src::resize::recalculate_sizes;
use crate::src::window::Window as _;
use crate::src::server_client::Client as _;
use crate::src::server_fn::{
    server_kill_window, server_redraw_session, server_redraw_window, server_redraw_window_borders,
    server_status_session, server_unzoom_window,
};

use crate::src::session::Session;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::client::ClientRef;
use crate::src::shared::command::CMD_FIND_DEFAULT_MARKED;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::key::key_event;
use crate::src::shared::layout::*;
use crate::src::shared::limits::{INT_MAX, INT_MIN, UINT_MAX};
use crate::src::shared::mouse::mouse_event;
use crate::src::shared::pane::window_pane;
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
pub static cmd_move_pane_entry: cmd_entry = {
    cmd_entry {
        name: c"move-pane",
        alias: Some(c"movep"),
        args: args_parse {
            template: c"bdD::fhMvl:L::P:R::s:t:U::X:Y:z:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-bdfhMv] [-D lines] [-l size] [-L columns] [-P position] [-R columns] [-s src-pane] [-t dst-pane] [-U lines] [-X x-position] [-Y y-position] [-z z-index]",
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
unsafe fn cmd_join_pane_place(
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
    mut wl: refbox::Weak<winlink>,
    wp_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    mut position: *const ::core::ffi::c_char,
) -> cmd_retval {
    let window_owner = wl
        .get_unchecked()
        .window_handle()
        .cloned()
        .expect("pane window");
    let result = (|| {
        let cell_id = wp_owner.layout_identity(false).expect("placed pane layout");
        let geometry = window_owner
            .borrow_layout_cell(cell_id)
            .expect("placed pane belongs to layout")
            .g;
        let mut owp: Option<std::rc::Rc<std::cell::UnsafeCell<window_pane>>> = None;
        let mut wx: ::core::ffi::c_int = ((wl.get_unchecked().window_handle().as_ref())
            .expect("live window"))
        .size()
        .0 as ::core::ffi::c_int;
        let mut wy: ::core::ffi::c_int = ((wl.get_unchecked().window_handle().as_ref())
            .expect("live window"))
        .size()
        .1 as ::core::ffi::c_int;
        let mut px: ::core::ffi::c_int = geometry.sx as ::core::ffi::c_int;
        let mut py: ::core::ffi::c_int = geometry.sy as ::core::ffi::c_int;
        let mut xoff: ::core::ffi::c_int = geometry.xoff;
        let mut yoff: ::core::ffi::c_int = geometry.yoff;
        let mut border: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        if wp_owner.pane_lines() as ::core::ffi::c_uint
            == PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            border = 0 as ::core::ffi::c_int;
        }
        if strcmp(
            position,
            b"top-left\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            xoff = border;
            yoff = border;
        } else if strcmp(
            position,
            b"top-centre\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
            || strcmp(
                position,
                b"top-center\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
        {
            xoff = (wx - px) / 2 as ::core::ffi::c_int;
            yoff = border;
        } else if strcmp(
            position,
            b"top-right\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            xoff = wx - px - border;
            yoff = border;
        } else if strcmp(
            position,
            b"centre-left\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
            || strcmp(
                position,
                b"center-left\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
        {
            xoff = border;
            yoff = (wy - py) / 2 as ::core::ffi::c_int;
        } else if strcmp(
            position,
            b"centre\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
            || strcmp(
                position,
                b"center\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
        {
            xoff = (wx - px) / 2 as ::core::ffi::c_int;
            yoff = (wy - py) / 2 as ::core::ffi::c_int;
        } else if strcmp(
            position,
            b"centre-right\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
            || strcmp(
                position,
                b"center-right\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
        {
            xoff = wx - px - border;
            yoff = (wy - py) / 2 as ::core::ffi::c_int;
        } else if strcmp(
            position,
            b"bottom-left\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            xoff = border;
            yoff = wy - py - border;
        } else if strcmp(
            position,
            b"bottom-centre\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
            || strcmp(
                position,
                b"bottom-center\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
        {
            xoff = (wx - px) / 2 as ::core::ffi::c_int;
            yoff = wy - py - border;
        } else if strcmp(
            position,
            b"bottom-right\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            xoff = wx - px - border;
            yoff = wy - py - border;
        } else if strcmp(
            position,
            b"top-left-centre\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
            || strcmp(
                position,
                b"top-left-center\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
        {
            xoff = wx / 4 as ::core::ffi::c_int - px / 2 as ::core::ffi::c_int;
            yoff = wy / 4 as ::core::ffi::c_int - py / 2 as ::core::ffi::c_int;
        } else if strcmp(
            position,
            b"top-right-centre\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
            || strcmp(
                position,
                b"top-right-center\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
        {
            xoff = 3 as ::core::ffi::c_int * wx / 4 as ::core::ffi::c_int
                - px / 2 as ::core::ffi::c_int;
            yoff = wy / 4 as ::core::ffi::c_int - py / 2 as ::core::ffi::c_int;
        } else if strcmp(
            position,
            b"bottom-left-centre\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
            || strcmp(
                position,
                b"bottom-left-center\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
        {
            xoff = wx / 4 as ::core::ffi::c_int - px / 2 as ::core::ffi::c_int;
            yoff = 3 as ::core::ffi::c_int * wy / 4 as ::core::ffi::c_int
                - py / 2 as ::core::ffi::c_int;
        } else if strcmp(
            position,
            b"bottom-right-centre\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
            || strcmp(
                position,
                b"bottom-right-center\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
        {
            xoff = 3 as ::core::ffi::c_int * wx / 4 as ::core::ffi::c_int
                - px / 2 as ::core::ffi::c_int;
            yoff = 3 as ::core::ffi::c_int * wy / 4 as ::core::ffi::c_int
                - py / 2 as ::core::ffi::c_int;
        } else if strcmp(
            position,
            b"front\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            assert!(
                window_owner
                    .borrow_pane_order_mut(crate::src::window::PaneOrder::Stacking)
                    .remove(&std::rc::Rc::downgrade(wp_owner)),
                "pane is not in its stacking order"
            );
            window_owner
                .borrow_pane_order_mut(crate::src::window::PaneOrder::Stacking)
                .push_front(std::rc::Rc::downgrade(wp_owner));
        } else if strcmp(
            position,
            b"back\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            assert!(
                window_owner
                    .borrow_pane_order_mut(crate::src::window::PaneOrder::Stacking)
                    .remove(&std::rc::Rc::downgrade(wp_owner)),
                "pane is not in its stacking order"
            );
            owp = window_owner.step_pane(crate::src::window::PaneOrder::Stacking, None, false);
            while owp.is_some() {
                if !owp.as_ref().expect("ordered pane").is_floating() {
                    break;
                }
                owp = owp.as_ref().and_then(|pane| {
                    window_owner.step_pane(
                        crate::src::window::PaneOrder::Stacking,
                        Some(&std::rc::Rc::downgrade(pane)),
                        false,
                    )
                });
            }
            if owp.is_some() {
                window_owner
                    .borrow_pane_order_mut(crate::src::window::PaneOrder::Stacking)
                    .insert_before(
                        &std::rc::Rc::downgrade(owp.as_ref().expect("ordered pane")),
                        std::rc::Rc::downgrade(wp_owner),
                    );
            } else {
                window_owner
                    .borrow_pane_order_mut(crate::src::window::PaneOrder::Stacking)
                    .push_back(std::rc::Rc::downgrade(wp_owner));
            }
        } else if strcmp(
            position,
            b"forward\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            owp = window_owner.step_pane(
                crate::src::window::PaneOrder::Stacking,
                Some(&std::rc::Rc::downgrade(wp_owner)),
                true,
            );
            if owp.is_some() {
                assert!(
                    window_owner
                        .borrow_pane_order_mut(crate::src::window::PaneOrder::Stacking)
                        .remove(&std::rc::Rc::downgrade(wp_owner)),
                    "pane is not in its stacking order"
                );
                window_owner
                    .borrow_pane_order_mut(crate::src::window::PaneOrder::Stacking)
                    .insert_before(
                        &std::rc::Rc::downgrade(owp.as_ref().expect("ordered pane")),
                        std::rc::Rc::downgrade(wp_owner),
                    );
            }
        } else if strcmp(
            position,
            b"backward\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            owp = window_owner.step_pane(
                crate::src::window::PaneOrder::Stacking,
                Some(&std::rc::Rc::downgrade(wp_owner)),
                false,
            );
            if owp.is_some() && owp.as_ref().expect("ordered pane").is_floating() {
                assert!(
                    window_owner
                        .borrow_pane_order_mut(crate::src::window::PaneOrder::Stacking)
                        .remove(&std::rc::Rc::downgrade(wp_owner)),
                    "pane is not in its stacking order"
                );
                window_owner
                    .borrow_pane_order_mut(crate::src::window::PaneOrder::Stacking)
                    .insert_after(
                        &std::rc::Rc::downgrade(owp.as_ref().expect("ordered pane")),
                        std::rc::Rc::downgrade(wp_owner),
                    );
            }
        } else if strcmp(
            position,
            b"forward-loop\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            owp = window_owner.step_pane(
                crate::src::window::PaneOrder::Stacking,
                Some(&std::rc::Rc::downgrade(wp_owner)),
                true,
            );
            assert!(
                window_owner
                    .borrow_pane_order_mut(crate::src::window::PaneOrder::Stacking)
                    .remove(&std::rc::Rc::downgrade(wp_owner)),
                "pane is not in its stacking order"
            );
            if owp.is_some() {
                window_owner
                    .borrow_pane_order_mut(crate::src::window::PaneOrder::Stacking)
                    .insert_before(
                        &std::rc::Rc::downgrade(owp.as_ref().expect("ordered pane")),
                        std::rc::Rc::downgrade(wp_owner),
                    );
            } else {
                owp = window_owner.step_pane(crate::src::window::PaneOrder::Stacking, None, false);
                while owp.is_some() {
                    if !owp.as_ref().expect("ordered pane").is_floating() {
                        break;
                    }
                    owp = owp.as_ref().and_then(|pane| {
                        window_owner.step_pane(
                            crate::src::window::PaneOrder::Stacking,
                            Some(&std::rc::Rc::downgrade(pane)),
                            false,
                        )
                    });
                }
                if owp.is_some() {
                    window_owner
                        .borrow_pane_order_mut(crate::src::window::PaneOrder::Stacking)
                        .insert_before(
                            &std::rc::Rc::downgrade(owp.as_ref().expect("ordered pane")),
                            std::rc::Rc::downgrade(wp_owner),
                        );
                } else {
                    window_owner
                        .borrow_pane_order_mut(crate::src::window::PaneOrder::Stacking)
                        .push_back(std::rc::Rc::downgrade(wp_owner));
                }
            }
        } else if strcmp(
            position,
            b"backward-loop\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            owp = window_owner.step_pane(
                crate::src::window::PaneOrder::Stacking,
                Some(&std::rc::Rc::downgrade(wp_owner)),
                false,
            );
            if owp.is_some() && owp.as_ref().expect("ordered pane").is_floating() {
                assert!(
                    window_owner
                        .borrow_pane_order_mut(crate::src::window::PaneOrder::Stacking)
                        .remove(&std::rc::Rc::downgrade(wp_owner)),
                    "pane is not in its stacking order"
                );
                window_owner
                    .borrow_pane_order_mut(crate::src::window::PaneOrder::Stacking)
                    .insert_after(
                        &std::rc::Rc::downgrade(owp.as_ref().expect("ordered pane")),
                        std::rc::Rc::downgrade(wp_owner),
                    );
            } else {
                assert!(
                    window_owner
                        .borrow_pane_order_mut(crate::src::window::PaneOrder::Stacking)
                        .remove(&std::rc::Rc::downgrade(wp_owner)),
                    "pane is not in its stacking order"
                );
                window_owner
                    .borrow_pane_order_mut(crate::src::window::PaneOrder::Stacking)
                    .push_front(std::rc::Rc::downgrade(wp_owner));
            }
        } else {
            cmdq_error(item_handle, |out| {
                out.write_all(b"unknown position: ")?;
                write_cstr(out, position)
            });
            return CMD_RETURN_ERROR;
        }
        let moved = {
            window_owner
                .borrow_layout_cell_mut(cell_id)
                .is_some_and(|mut cell| {
                    if xoff == cell.g.xoff && yoff == cell.g.yoff {
                        return false;
                    }
                    cell.g.xoff = xoff;
                    cell.g.yoff = yoff;
                    true
                })
        };
        if moved {
            layout_fix_panes(&window_owner, None);
        }
        window_owner.invalidate_scene();
        events_fire_window(
            b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
            std::rc::Rc::clone(&window_owner),
        );
        server_redraw_window(&window_owner);
        return CMD_RETURN_NORMAL;
    })();
    window_owner.release(c"cmd_join_pane_place");
    result
}
unsafe fn cmd_join_pane_move(
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
    mut args: *mut args,
    mut wl: refbox::Weak<winlink>,
    wp_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
) -> cmd_retval {
    let window_owner = wl
        .get_unchecked()
        .window_handle()
        .cloned()
        .expect("moved pane window");
    let result = (|| {
        let cell_id = wp_owner.layout_identity(false).expect("moved pane layout");
        let geometry = window_owner
            .borrow_layout_cell(cell_id)
            .expect("moved pane belongs to layout")
            .g;
        let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut argval: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let flags: [::core::ffi::c_char; 4] = [
            'U' as i32 as ::core::ffi::c_char,
            'D' as i32 as ::core::ffi::c_char,
            'L' as i32 as ::core::ffi::c_char,
            'R' as i32 as ::core::ffi::c_char,
        ];
        let mut flag: ::core::ffi::c_char = 0;
        let mut xoff: ::core::ffi::c_int = geometry.xoff;
        let mut yoff: ::core::ffi::c_int = geometry.yoff;
        let mut adjust: ::core::ffi::c_int = 0;
        let mut i: u_int = 0;
        let mut lines: pane_lines = wp_owner.pane_lines();
        if args_has(args, 'X' as i32 as u_char) != 0 {
            xoff = match args_percentage_and_expand_result(
                args,
                'X' as i32 as u_char,
                -(wl.get_unchecked()
                    .window_handle()
                    .expect("moved pane window")
                    .size()
                    .0 as ::core::ffi::c_int) as ::core::ffi::c_longlong,
                wl.get_unchecked()
                    .window_handle()
                    .expect("moved pane window")
                    .size()
                    .0 as ::core::ffi::c_longlong,
                wl.get_unchecked()
                    .window_handle()
                    .expect("moved pane window")
                    .size()
                    .0 as ::core::ffi::c_longlong,
                Some(item_handle),
            ) {
                Ok(value) => value as ::core::ffi::c_int,
                Err(error) => {
                    cmdq_error(item_handle, |out| {
                        out.write_all(b"position ")?;
                        write_cstr(out, error.message().as_ptr())
                    });
                    return CMD_RETURN_ERROR;
                }
            };
            if lines as ::core::ffi::c_uint
                != PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                xoff += 1 as ::core::ffi::c_int;
            }
        }
        if args_has(args, 'Y' as i32 as u_char) != 0 {
            yoff = match args_percentage_and_expand_result(
                args,
                'Y' as i32 as u_char,
                -(wl.get_unchecked()
                    .window_handle()
                    .expect("moved pane window")
                    .size()
                    .1 as ::core::ffi::c_int) as ::core::ffi::c_longlong,
                wl.get_unchecked()
                    .window_handle()
                    .expect("moved pane window")
                    .size()
                    .1 as ::core::ffi::c_longlong,
                wl.get_unchecked()
                    .window_handle()
                    .expect("moved pane window")
                    .size()
                    .1 as ::core::ffi::c_longlong,
                Some(item_handle),
            ) {
                Ok(value) => value as ::core::ffi::c_int,
                Err(error) => {
                    cmdq_error(item_handle, |out| {
                        out.write_all(b"position ")?;
                        write_cstr(out, error.message().as_ptr())
                    });
                    return CMD_RETURN_ERROR;
                }
            };
            if lines as ::core::ffi::c_uint
                != PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                yoff += 1 as ::core::ffi::c_int;
            }
        }
        i = 0 as u_int;
        while (i as usize)
            < (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>() as usize)
        {
            flag = flags[i as usize];
            if !(args_has(args, flag as u_char) == 0) {
                argval = args_get(&*(args), flag as u_char)
                    .map_or(std::ptr::null(), |value| value.as_ptr());
                if argval.is_null() {
                    argval = b"1\0" as *const u8 as *const ::core::ffi::c_char;
                }
                adjust = strtonum(
                    argval,
                    INT_MIN as ::core::ffi::c_longlong,
                    INT_MAX as ::core::ffi::c_longlong,
                    &raw mut errstr,
                ) as ::core::ffi::c_int;
                if !errstr.is_null() {
                    cmdq_error(item_handle, |out| {
                        out.write_all(b"offset ")?;
                        write_cstr(out, errstr)
                    });
                    return CMD_RETURN_ERROR;
                }
                if flag as ::core::ffi::c_int == 'U' as i32 {
                    yoff -= adjust;
                } else if flag as ::core::ffi::c_int == 'D' as i32 {
                    yoff += adjust;
                } else if flag as ::core::ffi::c_int == 'L' as i32 {
                    xoff -= adjust;
                } else {
                    xoff += adjust;
                }
            }
            i = i.wrapping_add(1);
        }
        let moved = {
            window_owner
                .borrow_layout_cell_mut(cell_id)
                .is_some_and(|mut cell| {
                    if xoff == cell.g.xoff && yoff == cell.g.yoff {
                        return false;
                    }
                    cell.g.xoff = xoff;
                    cell.g.yoff = yoff;
                    true
                })
        };
        if moved {
            layout_fix_panes(
                wl.get_unchecked()
                    .window_handle()
                    .expect("moved pane window"),
                None,
            );
            events_fire_window(
                c"window-layout-changed".as_ptr(),
                wl.get_unchecked()
                    .window_handle()
                    .expect("moved pane window")
                    .clone(),
            );
            server_redraw_window(
                wl.get_unchecked()
                    .window_handle()
                    .expect("moved pane window"),
            );
        }
        CMD_RETURN_NORMAL
    })();
    window_owner.release(c"cmd_join_pane_move");
    result
}
unsafe fn cmd_join_pane_mouse_update(
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
) -> cmd_retval {
    let item = item_handle.get();
    let mouse_pane_owner;
    let mut target: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let mut event_snapshot = cmdq_get_event(&*(item));
    let event: *mut key_event = &mut event_snapshot;
    let c_owner = cmdq_get_client((item).as_ref());
    let mut c: Option<ClientRef> = c_owner.clone();
    let mut s: Option<SessionRef> = (*target).session_handle();
    let mut wl: refbox::Weak<winlink> = refbox::Weak::new();
    if (*event).m.valid == 0 {
        return CMD_RETURN_NORMAL;
    }
    let mut mouse_session_owner = None;
    mouse_pane_owner = cmd_mouse_pane(
        &raw mut (*event).m,
        Some(&mut mouse_session_owner),
        &raw mut wl,
    );
    s = mouse_session_owner.clone();

    if mouse_pane_owner.is_none()
        || c.is_none()
        || !crate::src::shared::rc::same(
            c.as_ref()
                .expect("live client")
                .attached_session()
                .upgrade()
                .as_ref(),
            s.as_ref(),
        )
    {
        return CMD_RETURN_NORMAL;
    }
    if !mouse_pane_owner.as_ref().expect("mouse pane").is_floating() {
        return CMD_RETURN_NORMAL;
    }
    let window = wl
        .get_unchecked()
        .window_handle()
        .cloned()
        .expect("live window");
    window.redraw_active_switch(mouse_pane_owner.as_ref());
    window.select_pane(mouse_pane_owner.as_ref().expect("mouse pane"), true);
    window.release(c"join pane mouse selection");
    let drag_update: crate::src::shared::tty::mouse_drag_update_cb =
        Some(Box::new(crate::src::tty::tty_mouse_client_callback(
            c_owner.as_ref().expect("live drag client"),
            cmd_join_pane_mouse_move,
        )));
    c_owner
        .as_ref()
        .expect("live drag client")
        .borrow_terminal_mut()
        .mouse_drag_update = drag_update;
    cmd_join_pane_mouse_move(
        c_owner.as_ref().expect("live drag client"),
        &raw mut (*event).m,
    );
    return CMD_RETURN_NORMAL;
}
unsafe fn cmd_join_pane_mouse_move(client_owner: &ClientRef, mut m: *mut mouse_event) {
    let mut c: Option<ClientRef> = Some(client_owner.clone());
    let mouse_pane_owner;
    let mut wl: refbox::Weak<winlink> = refbox::Weak::new();
    let mut y: ::core::ffi::c_int = 0;
    let mut ly: ::core::ffi::c_int = 0;
    let mut x: ::core::ffi::c_int = 0;
    let mut lx: ::core::ffi::c_int = 0;
    mouse_pane_owner = cmd_mouse_pane(m, None, &raw mut wl);

    if mouse_pane_owner.is_none() {
        client_owner.borrow_terminal_mut().mouse_drag_update = None;
        return;
    }
    let window_owner = wl
        .get_unchecked()
        .window_handle()
        .cloned()
        .expect("mouse window");

    let cell_id = mouse_pane_owner
        .as_ref()
        .expect("mouse pane")
        .layout_identity(false)
        .expect("dragged pane layout");
    y = (*m).y.wrapping_add((*m).oy) as ::core::ffi::c_int;
    x = (*m).x.wrapping_add((*m).ox) as ::core::ffi::c_int;
    if (*m).statusat == 0 as ::core::ffi::c_int && y >= (*m).statuslines as ::core::ffi::c_int {
        y = (y as u_int).wrapping_sub((*m).statuslines) as ::core::ffi::c_int as ::core::ffi::c_int;
    } else if (*m).statusat > 0 as ::core::ffi::c_int && y >= (*m).statusat {
        y = (*m).statusat - 1 as ::core::ffi::c_int;
    }
    ly = (*m).ly.wrapping_add((*m).oy) as ::core::ffi::c_int;
    lx = (*m).lx.wrapping_add((*m).ox) as ::core::ffi::c_int;
    if (*m).statusat == 0 as ::core::ffi::c_int && ly >= (*m).statuslines as ::core::ffi::c_int {
        ly = (ly as u_int).wrapping_sub((*m).statuslines) as ::core::ffi::c_int
            as ::core::ffi::c_int;
    } else if (*m).statusat > 0 as ::core::ffi::c_int && ly >= (*m).statusat {
        ly = (*m).statusat - 1 as ::core::ffi::c_int;
    }
    if x != lx || y != ly {
        {
            let mut cell = window_owner
                .borrow_layout_cell_mut(cell_id)
                .expect("dragged pane belongs to layout");
            cell.g.xoff += x - lx;
            cell.g.yoff += y - ly;
        }
        layout_fix_panes(&window_owner, None);
        server_redraw_window(&window_owner);
        server_redraw_window_borders(&window_owner);
    }
    window_owner.release(c"cmd_join_pane_mouse_move");
}
unsafe fn cmd_join_pane_zindex(
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
    mut wl: refbox::Weak<winlink>,
    wp_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    mut s: *const ::core::ffi::c_char,
) -> cmd_retval {
    let window_owner = wl
        .get_unchecked()
        .window_handle()
        .cloned()
        .expect("pane window");
    let result = (|| {
        let mut owp: Option<std::rc::Rc<std::cell::UnsafeCell<window_pane>>> = None;
        let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut n: u_int = 0;
        let mut z: u_int = 0;
        z = strtonum(
            s,
            0 as ::core::ffi::c_longlong,
            UINT_MAX as ::core::ffi::c_longlong,
            &raw mut errstr,
        ) as u_int;
        if !errstr.is_null() {
            cmdq_error(item_handle, |out| {
                out.write_all(b"z-index ")?;
                write_cstr(out, errstr)
            });
            return CMD_RETURN_ERROR;
        }
        assert!(
            window_owner
                .borrow_pane_order_mut(crate::src::window::PaneOrder::Stacking)
                .remove(&std::rc::Rc::downgrade(wp_owner)),
            "pane is not in its stacking order"
        );
        n = 0 as u_int;
        owp = window_owner.step_pane(crate::src::window::PaneOrder::Stacking, None, false);
        while owp.is_some() {
            if !owp.as_ref().expect("ordered pane").is_floating() {
                break;
            }
            if n >= z {
                break;
            }
            n = n.wrapping_add(1);
            owp = owp.as_ref().and_then(|pane| {
                window_owner.step_pane(
                    crate::src::window::PaneOrder::Stacking,
                    Some(&std::rc::Rc::downgrade(pane)),
                    false,
                )
            });
        }
        if owp.is_some() {
            window_owner
                .borrow_pane_order_mut(crate::src::window::PaneOrder::Stacking)
                .insert_before(
                    &std::rc::Rc::downgrade(owp.as_ref().expect("ordered pane")),
                    std::rc::Rc::downgrade(wp_owner),
                );
        } else {
            window_owner
                .borrow_pane_order_mut(crate::src::window::PaneOrder::Stacking)
                .push_back(std::rc::Rc::downgrade(wp_owner));
        }
        window_owner.invalidate_scene();
        events_fire_window(
            b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
            std::rc::Rc::clone(&window_owner),
        );
        server_redraw_window(&window_owner);
        return CMD_RETURN_NORMAL;
    })();
    window_owner.release(c"cmd_join_pane_zindex");
    result
}
unsafe fn cmd_join_pane_tile(
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
    mut args: *mut args,
    w_owner: &WindowRef,
    wp_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
) -> cmd_retval {
    if !wp_owner.is_floating() {
        cmdq_error(item_handle, |out| out.write_all(b"pane is not floating"));
        return CMD_RETURN_ERROR;
    }
    if w_owner.is_zoomed() {
        cmdq_error(item_handle, |out| {
            out.write_all(b"can't tile a pane while window is zoomed")
        });
        return CMD_RETURN_ERROR;
    }
    if !layout_tile_pane(w_owner, wp_owner) {
        cmdq_error(item_handle, |out| out.write_all(b"no space for a new pane"));
        return CMD_RETURN_ERROR;
    }
    assert!(
        w_owner
            .borrow_pane_order_mut(crate::src::window::PaneOrder::Stacking)
            .remove(&std::rc::Rc::downgrade(wp_owner)),
        "pane is not in its stacking order"
    );
    w_owner
        .borrow_pane_order_mut(crate::src::window::PaneOrder::Stacking)
        .push_back(std::rc::Rc::downgrade(wp_owner));
    if args_has(args, 'd' as i32 as u_char) == 0 {
        w_owner.select_pane(wp_owner, true);
    }
    layout_fix_offsets(w_owner);
    layout_fix_panes(w_owner, None);
    w_owner.invalidate_scene();
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        std::rc::Rc::clone(&(w_owner)),
    );
    server_redraw_window(&(w_owner));
    return CMD_RETURN_NORMAL;
}
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
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
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
        if std::ptr::eq(cmd_get_entry(self_0.get_unchecked()), &cmd_move_pane_entry) {
            if args_has(args, 'M' as i32 as u_char) != 0 {
                return cmd_join_pane_mouse_update(item_handle);
            }
            if args_has(args, 'P' as i32 as u_char) != 0
                || args_has(args, 'z' as i32 as u_char) != 0
                || args_has(args, 'X' as i32 as u_char) != 0
                || args_has(args, 'Y' as i32 as u_char) != 0
                || args_has(args, 'U' as i32 as u_char) != 0
                || args_has(args, 'D' as i32 as u_char) != 0
                || args_has(args, 'L' as i32 as u_char) != 0
                || args_has(args, 'R' as i32 as u_char) != 0
            {
                if !dst_pane_owner.is_floating() {
                    cmdq_error(item_handle, |out| out.write_all(b"pane is not floating"));
                    return CMD_RETURN_ERROR;
                }
                server_unzoom_window(&dst_window);
                s = args_get(&*(args), 'P' as i32 as u_char)
                    .map_or(std::ptr::null(), |value| value.as_ptr());
                if !s.is_null() {
                    return cmd_join_pane_place(item_handle, (dst_wl).clone(), &dst_pane_owner, s);
                }
                s = args_get(&*(args), 'z' as i32 as u_char)
                    .map_or(std::ptr::null(), |value| value.as_ptr());
                if !s.is_null() {
                    return cmd_join_pane_zindex(item_handle, (dst_wl).clone(), &dst_pane_owner, s);
                }
                return cmd_join_pane_move(item_handle, args, (dst_wl).clone(), &dst_pane_owner);
            }
        }
        src_wl = (*source).winlink_handle();
        let src_pane_owner = (*source).pane_handle().expect("join source pane");
        src_window = src_wl.get_unchecked().window_handle().cloned();
        let src_owner = src_window.as_ref().expect("live source window");
        if src_owner
            .modal_pane()
            .is_some_and(|pane| std::rc::Rc::ptr_eq(&pane, &src_pane_owner))
            || dst_window
                .modal_pane()
                .is_some_and(|pane| std::rc::Rc::ptr_eq(&pane, &dst_pane_owner))
        {
            cmdq_error(item_handle, |out| out.write_all(b"pane is modal"));
            return CMD_RETURN_ERROR;
        }
        server_unzoom_window(&dst_window);
        server_unzoom_window(src_owner);
        if std::rc::Rc::ptr_eq(&src_pane_owner, &dst_pane_owner) {
            if src_pane_owner.is_floating() {
                return cmd_join_pane_tile(item_handle, args, src_owner, &src_pane_owner);
            }
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
                .borrow_pane_order_mut(crate::src::window::PaneOrder::Index)
                .remove(&std::rc::Rc::downgrade(&src_pane_owner)),
            "pane is not in its window order"
        );
        assert!(
            src_owner
                .borrow_pane_order_mut(crate::src::window::PaneOrder::Stacking)
                .remove(&std::rc::Rc::downgrade(&src_pane_owner)),
            "pane is not in its stacking order"
        );
        src_pane_owner.reparent(&dst_window);
        if flags & SPAWN_BEFORE != 0 {
            dst_window
                .borrow_pane_order_mut(crate::src::window::PaneOrder::Index)
                .insert_before(
                    &std::rc::Rc::downgrade(&dst_pane_owner),
                    std::rc::Rc::downgrade(&src_pane_owner),
                );
            dst_window
                .borrow_pane_order_mut(crate::src::window::PaneOrder::Stacking)
                .insert_before(
                    &std::rc::Rc::downgrade(&dst_pane_owner),
                    std::rc::Rc::downgrade(&src_pane_owner),
                );
        } else {
            dst_window
                .borrow_pane_order_mut(crate::src::window::PaneOrder::Index)
                .insert_after(
                    &std::rc::Rc::downgrade(&dst_pane_owner),
                    std::rc::Rc::downgrade(&src_pane_owner),
                );
            dst_window
                .borrow_pane_order_mut(crate::src::window::PaneOrder::Stacking)
                .insert_after(
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
