use crate::src::arguments::{args_count, args_get, args_has, args_percentage_result, args_string};
use crate::src::cmd::queue::{cmdq_error, cmdq_get_client, cmdq_get_event, cmdq_get_target};
use crate::src::cmd::{cmd_get_args_mut, cmd_mouse_pane, cmd_mouse_window};
use crate::src::compat::strtonum::strtonum;
use crate::src::events::events_fire_window;
use crate::src::format::bytes::write_cstr;
use crate::src::layout::{
    layout_fix_offsets, layout_fix_panes, layout_resize_floating_pane,
    layout_resize_floating_pane_to, layout_resize_layout, layout_resize_pane,
    layout_resize_pane_to, layout_search_by_border,
};
use crate::src::server_client::Client as _;
use crate::src::server_fn::{
    server_redraw_window, server_redraw_window_borders, server_unzoom_window,
};
use crate::src::session::Session;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::client::ClientRef;
use crate::src::shared::command::CMD_AFTERHOOK;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::key::key_event;
use crate::src::shared::layout::layout_cell;
use crate::src::shared::layout::*;
use crate::src::shared::limits::{INT_MAX, INT_MIN};
use crate::src::shared::mouse::mouse_event;
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::{PANE_MAXIMUM, PANE_MINIMUM, PANE_STATUS_BOTTOM, PANE_STATUS_TOP};
use crate::src::shared::session::session;
use crate::src::shared::session::SessionRef;
use crate::src::shared::window::WINDOW_ZOOMED;
use crate::src::shared::window::{window, winlink};
use crate::src::window::Window as _;

use crate::src::window_pane::WindowPane as _;
pub static cmd_resize_pane_entry: cmd_entry = {
    cmd_entry {
        name: c"resize-pane",
        alias: Some(c"resizep"),
        args: args_parse {
            template: c"D::L::MR::Tt:U::x:y:Z",
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-MTZ] [-D lines] [-L columns] [-R columns] [-U lines] [-x width] [-y height] [-t target-pane]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK,
        exec: Some(cmd_resize_pane_exec),
    }
};
unsafe fn cmd_resize_pane_exec(
    mut self_0: refbox::Weak<cmd>,
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
) -> cmd_retval {
    let item = item_handle.get();
    let mut args: *mut args =
        cmd_get_args_mut(self_0.get_mut_unchecked()).map_or(std::ptr::null_mut(), |args| args);
    let mut target: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let pane_owner = (*target).wp.upgrade().expect("live resize target pane");
    let mut wl: refbox::Weak<winlink> = (*target).winlink_handle();
    let original_window = wl
        .get_unchecked()
        .window_handle()
        .cloned()
        .expect("resize target window");
    let mut layout_owner = None;
    let result = (|| {
        let mut type_0: layout_type = LAYOUT_LEFTRIGHT;
        let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut argval: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let flags: [::core::ffi::c_char; 4] = [
            'U' as i32 as ::core::ffi::c_char,
            'D' as i32 as ::core::ffi::c_char,
            'L' as i32 as ::core::ffi::c_char,
            'R' as i32 as ::core::ffi::c_char,
        ];
        let mut flag: ::core::ffi::c_char = 0;
        let mut adjust: ::core::ffi::c_int = 0;
        let mut x: ::core::ffi::c_int = 0;
        let mut y: ::core::ffi::c_int = 0;
        let mut status: ::core::ffi::c_int = 0;
        let mut opposite: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut i: ::core::ffi::c_ulong = 0;
        if args_has(args, 'T' as u_char) != 0 {
            pane_owner.trim_history();
            return CMD_RETURN_NORMAL;
        }
        if args_has(args, 'M' as i32 as u_char) != 0 {
            return cmd_resize_pane_mouse_update(item_handle);
        }
        if args_has(args, 'Z' as i32 as u_char) != 0 {
            if wl
                .get_unchecked()
                .window_handle()
                .expect("resize window")
                .is_zoomed()
            {
                (wl.get_unchecked().window_handle().expect("resize window")).unzoom(true);
            } else {
                crate::src::shared::window::WindowRef::zoom_pane(&pane_owner);
            }
            server_redraw_window(wl.get_unchecked().window_handle().expect("resize window"));
            return CMD_RETURN_NORMAL;
        }
        server_unzoom_window(wl.get_unchecked().window_handle().expect("resize window"));
        let cell_id = pane_owner
            .layout_identity(false)
            .expect("resized pane layout");
        layout_owner = Some(
            pane_owner
                .window_observer()
                .upgrade()
                .expect("pane layout window"),
        );
        if args_has(args, 'x' as i32 as u_char) != 0 {
            x = match args_percentage_result(
                args,
                'x' as i32 as u_char,
                0 as ::core::ffi::c_longlong,
                PANE_MAXIMUM as ::core::ffi::c_longlong,
                wl.get_unchecked()
                    .window_handle()
                    .expect("resize window")
                    .size()
                    .0 as ::core::ffi::c_longlong,
            ) {
                Ok(value) => value as ::core::ffi::c_int,
                Err(error) => {
                    cmdq_error(item_handle, |out| {
                        out.write_all(b"width ")?;
                        write_cstr(out, error.message().as_ptr())
                    });
                    return CMD_RETURN_ERROR;
                }
            };
            if pane_owner.is_floating() {
                if let Err(cause) =
                    layout_resize_floating_pane_to(&pane_owner, LAYOUT_LEFTRIGHT, x as u_int)
                {
                    cmdq_error(item_handle, |out| {
                        out.write_all(b"size ")?;
                        write_cstr(out, cause.as_ptr())
                    });
                    return CMD_RETURN_ERROR;
                }
            } else {
                layout_resize_pane_to(&pane_owner, LAYOUT_LEFTRIGHT, x as u_int);
            }
        }
        if args_has(args, 'y' as i32 as u_char) != 0 {
            y = match args_percentage_result(
                args,
                'y' as i32 as u_char,
                0 as ::core::ffi::c_longlong,
                PANE_MAXIMUM as ::core::ffi::c_longlong,
                wl.get_unchecked()
                    .window_handle()
                    .expect("resize window")
                    .size()
                    .1 as ::core::ffi::c_longlong,
            ) {
                Ok(value) => value as ::core::ffi::c_int,
                Err(error) => {
                    cmdq_error(item_handle, |out| {
                        out.write_all(b"height ")?;
                        write_cstr(out, error.message().as_ptr())
                    });
                    return CMD_RETURN_ERROR;
                }
            };
            status = original_window.pane_border_status();
            match status {
                PANE_STATUS_TOP => {
                    if y != INT_MAX && pane_owner.geometry().3 == 1 as ::core::ffi::c_int {
                        y += 1;
                    }
                }
                PANE_STATUS_BOTTOM => {
                    if y != INT_MAX
                        && (pane_owner.geometry().3 as u_int).wrapping_add(pane_owner.geometry().1)
                            == wl
                                .get_unchecked()
                                .window_handle()
                                .expect("resize window")
                                .size()
                                .1
                                .wrapping_sub(1 as u_int)
                    {
                        y += 1;
                    }
                }
                _ => {}
            }
            if pane_owner.is_floating() {
                if let Err(cause) =
                    layout_resize_floating_pane_to(&pane_owner, LAYOUT_TOPBOTTOM, y as u_int)
                {
                    cmdq_error(item_handle, |out| {
                        out.write_all(b"size ")?;
                        write_cstr(out, cause.as_ptr())
                    });
                    return CMD_RETURN_ERROR;
                }
            } else {
                layout_resize_pane_to(&pane_owner, LAYOUT_TOPBOTTOM, y as u_int);
            }
        }
        i = 0 as ::core::ffi::c_ulong;
        while (i as usize)
            < (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>() as usize)
        {
            flag = flags[i as usize];
            if !(args_has(args, flag as u_char) == 0) {
                argval = args_get(&*(args), flag as u_char)
                    .map_or(std::ptr::null(), |value| value.as_ptr());
                if argval.is_null() {
                    if args_count(args) == 0 as u_int {
                        argval = b"1\0" as *const u8 as *const ::core::ffi::c_char;
                    } else {
                        argval = args_string(&mut *(args), 0 as u_int)
                            .map_or(std::ptr::null(), |value| value.as_ptr());
                    }
                }
                adjust = strtonum(
                    argval,
                    INT_MIN as ::core::ffi::c_longlong,
                    INT_MAX as ::core::ffi::c_longlong,
                    &raw mut errstr,
                ) as ::core::ffi::c_int;
                if !errstr.is_null() {
                    cmdq_error(item_handle, |out| {
                        out.write_all(b"adjustment ")?;
                        write_cstr(out, errstr)
                    });
                    return CMD_RETURN_ERROR;
                }
                type_0 = LAYOUT_TOPBOTTOM;
                if flag as ::core::ffi::c_int == 'L' as i32
                    || flag as ::core::ffi::c_int == 'R' as i32
                {
                    type_0 = LAYOUT_LEFTRIGHT;
                }
                if pane_owner.is_floating() {
                    if flag as ::core::ffi::c_int == 'L' as i32
                        || flag as ::core::ffi::c_int == 'U' as i32
                    {
                        opposite = 1 as ::core::ffi::c_int;
                    }
                    if let Err(cause) =
                        layout_resize_floating_pane(&pane_owner, type_0, adjust, opposite)
                    {
                        cmdq_error(item_handle, |out| {
                            out.write_all(b"adjustment ")?;
                            write_cstr(out, cause.as_ptr())
                        });
                        return CMD_RETURN_ERROR;
                    }
                } else {
                    if flag as ::core::ffi::c_int == 'L' as i32
                        || flag as ::core::ffi::c_int == 'U' as i32
                    {
                        // Preserve tmux's signed adjustment at the i32 boundary.
                        adjust = adjust.wrapping_neg();
                    }
                    layout_resize_pane(&pane_owner, type_0, adjust);
                }
            }
            i = i.wrapping_add(1);
        }
        let has_parent = layout_owner
            .as_ref()
            .expect("pane layout window")
            .borrow_layout_cell(cell_id)
            .is_some_and(|cell| !cell.parent.is_null());
        if has_parent {
            layout_fix_offsets(wl.get_unchecked().window_handle().expect("resize window"));
        }
        layout_fix_panes(
            wl.get_unchecked().window_handle().expect("resize window"),
            None,
        );
        events_fire_window(
            b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
            wl.get_unchecked()
                .window_handle()
                .expect("resize window")
                .clone(),
        );
        server_redraw_window(wl.get_unchecked().window_handle().expect("resize window"));
        CMD_RETURN_NORMAL
    })();
    if let Some(window) = layout_owner {
        window.release(c"resize pane layout");
    }
    original_window.release(c"cmd_resize_pane_exec");
    result
}

unsafe fn cmd_resize_pane_mouse_update(
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
) -> cmd_retval {
    let item = item_handle.get();
    let mouse_pane_owner;
    let mut target: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let mut event_snapshot = cmdq_get_event(&*(item));
    let event: *mut key_event = &mut event_snapshot;

    let mut wl: refbox::Weak<winlink> = (*target).winlink_handle();

    let c_owner = cmdq_get_client((item).as_ref());
    let mut c: Option<ClientRef> = c_owner.clone();
    let mut s: Option<SessionRef> = (*target).session_handle();
    if (*event).m.valid == 0 {
        return CMD_RETURN_NORMAL;
    }
    let mut mouse_session_owner = None;
    mouse_pane_owner = cmd_mouse_pane(
        &raw mut (*event).m,
        Some(&mut mouse_session_owner),
        ::core::ptr::null_mut::<refbox::Weak<winlink>>(),
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
    let pane_owner = mouse_pane_owner.as_ref().expect("mouse pane");
    if !pane_owner.is_floating() {
        c.as_ref()
            .expect("live client")
            .borrow_terminal_mut()
            .mouse_drag_update = Some(Box::new(crate::src::tty::tty_mouse_client_callback(
            c_owner.as_ref().expect("live drag client"),
            cmd_resize_pane_mouse_resize_tiled,
        )));
        cmd_resize_pane_mouse_resize_tiled(
            c_owner.as_ref().expect("live drag client"),
            &raw mut (*event).m,
        );
        return CMD_RETURN_NORMAL;
    }
    (&std::rc::Rc::clone(&((wl.get_unchecked().window_handle().as_ref()).expect("live window"))))
        .redraw_active_switch(Some(pane_owner));
    (&std::rc::Rc::clone(&((wl.get_unchecked().window_handle().as_ref()).expect("live window"))))
        .select_pane(&pane_owner, true);
    c.as_ref()
        .expect("live client")
        .borrow_terminal_mut()
        .mouse_drag_update = Some(Box::new(crate::src::tty::tty_mouse_client_callback(
        c_owner.as_ref().expect("live drag client"),
        cmd_resize_pane_mouse_resize_move_floating,
    )));
    cmd_resize_pane_mouse_resize_move_floating(
        c_owner.as_ref().expect("live drag client"),
        &raw mut (*event).m,
    );
    return CMD_RETURN_NORMAL;
}
unsafe fn cmd_resize_pane_mouse_resize_move_floating(
    client_owner: &ClientRef,
    mut m: *mut mouse_event,
) {
    let mut c: Option<ClientRef> = Some(client_owner.clone());
    let mouse_pane_owner;
    let mut wl: refbox::Weak<winlink> = refbox::Weak::new();
    let mut y: ::core::ffi::c_int = 0;
    let mut ly: ::core::ffi::c_int = 0;
    let mut x: ::core::ffi::c_int = 0;
    let mut lx: ::core::ffi::c_int = 0;
    let mut sx: ::core::ffi::c_int = 0;
    let mut sy: ::core::ffi::c_int = 0;
    let mut new_sx: ::core::ffi::c_int = 0;
    let mut new_sy: ::core::ffi::c_int = 0;
    let mut left: ::core::ffi::c_int = 0;
    let mut right: ::core::ffi::c_int = 0;
    let mut new_xoff: ::core::ffi::c_int = 0;
    let mut new_yoff: ::core::ffi::c_int = 0;
    let mut resizes: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    mouse_pane_owner = cmd_mouse_pane(m, None, &raw mut wl);

    let Some(pane_owner) = mouse_pane_owner.as_ref() else {
        c.as_ref()
            .expect("live client")
            .borrow_terminal_mut()
            .mouse_drag_update = None;
        return;
    };
    let window_owner = wl
        .get_unchecked()
        .window_handle()
        .cloned()
        .expect("mouse window");
    (|| {
        let cell_id = pane_owner
            .layout_identity(false)
            .expect("dragged pane layout");
        let mut geometry = window_owner
            .borrow_layout_cell(cell_id)
            .expect("dragged pane belongs to layout")
            .g;
        let (pane_sx, pane_sy, _, pane_y) = pane_owner.geometry();
        sx = pane_sx as i32;
        sy = pane_sy as i32;
        let (outer_x, _, outer_sx, _) = pane_owner.outer_geometry();
        left = outer_x - 1;
        right = outer_x + outer_sx as i32;
        y = (*m).y.wrapping_add((*m).oy) as ::core::ffi::c_int;
        x = (*m).x.wrapping_add((*m).ox) as ::core::ffi::c_int;
        if (*m).statusat == 0 as ::core::ffi::c_int && y >= (*m).statuslines as ::core::ffi::c_int {
            y = (y as u_int).wrapping_sub((*m).statuslines) as ::core::ffi::c_int
                as ::core::ffi::c_int;
        } else if (*m).statusat > 0 as ::core::ffi::c_int && y >= (*m).statusat {
            y = (*m).statusat - 1 as ::core::ffi::c_int;
        }
        ly = (*m).ly.wrapping_add((*m).oy) as ::core::ffi::c_int;
        lx = (*m).lx.wrapping_add((*m).ox) as ::core::ffi::c_int;
        if (*m).statusat == 0 as ::core::ffi::c_int && ly >= (*m).statuslines as ::core::ffi::c_int
        {
            ly = (ly as u_int).wrapping_sub((*m).statuslines) as ::core::ffi::c_int
                as ::core::ffi::c_int;
        } else if (*m).statusat > 0 as ::core::ffi::c_int && ly >= (*m).statusat {
            ly = (*m).statusat - 1 as ::core::ffi::c_int;
        }
        if (lx == left || lx == left + 1 as ::core::ffi::c_int)
            && ly == pane_y - 1 as ::core::ffi::c_int
        {
            new_sx = geometry.sx.wrapping_add((lx - x) as u_int) as ::core::ffi::c_int;
            if new_sx < PANE_MINIMUM {
                new_sx = PANE_MINIMUM;
            }
            new_sy = geometry.sy.wrapping_add((ly - y) as u_int) as ::core::ffi::c_int;
            if new_sy < PANE_MINIMUM {
                new_sy = PANE_MINIMUM;
            }
            new_xoff = x + 1 as ::core::ffi::c_int;
            new_yoff = y + 1 as ::core::ffi::c_int;
            geometry = layout_geometry {
                sx: new_sx as u_int,
                sy: new_sy as u_int,
                xoff: new_xoff,
                yoff: new_yoff,
            };
            resizes += 1;
        } else if (lx == right + 1 as ::core::ffi::c_int || lx == right)
            && ly == pane_y - 1 as ::core::ffi::c_int
        {
            new_sx = x - geometry.xoff;
            if new_sx < PANE_MINIMUM {
                new_sx = PANE_MINIMUM;
            }
            new_sy = geometry.sy.wrapping_add((ly - y) as u_int) as ::core::ffi::c_int;
            if new_sy < PANE_MINIMUM {
                new_sy = PANE_MINIMUM;
            }
            new_yoff = y + 1 as ::core::ffi::c_int;
            geometry = layout_geometry {
                sx: new_sx as u_int,
                sy: new_sy as u_int,
                xoff: geometry.xoff,
                yoff: new_yoff,
            };
            resizes += 1;
        } else if (lx == left || lx == left + 1 as ::core::ffi::c_int) && ly == pane_y + sy {
            new_sx = geometry.sx.wrapping_add((lx - x) as u_int) as ::core::ffi::c_int;
            if new_sx < PANE_MINIMUM {
                new_sx = PANE_MINIMUM;
            }
            new_sy = y - geometry.yoff;
            if new_sy < PANE_MINIMUM {
                return;
            }
            new_xoff = x + 1 as ::core::ffi::c_int;
            geometry = layout_geometry {
                sx: new_sx as u_int,
                sy: new_sy as u_int,
                xoff: new_xoff,
                yoff: geometry.yoff,
            };
            resizes += 1;
        } else if (lx == right + 1 as ::core::ffi::c_int || lx == right) && ly == pane_y + sy {
            new_sx = x - geometry.xoff;
            if new_sx < PANE_MINIMUM {
                new_sx = PANE_MINIMUM;
            }
            new_sy = y - geometry.yoff;
            if new_sy < PANE_MINIMUM {
                new_sy = PANE_MINIMUM;
            }
            geometry = layout_geometry {
                sx: new_sx as u_int,
                sy: new_sy as u_int,
                xoff: geometry.xoff,
                yoff: geometry.yoff,
            };
            resizes += 1;
        } else if lx == right {
            new_sx = x - geometry.xoff;
            if new_sx < PANE_MINIMUM {
                return;
            }
            geometry = layout_geometry {
                sx: new_sx as u_int,
                sy: geometry.sy,
                xoff: geometry.xoff,
                yoff: geometry.yoff,
            };
            resizes += 1;
        } else if lx == left {
            new_sx = geometry.sx.wrapping_add((lx - x) as u_int) as ::core::ffi::c_int;
            if new_sx < PANE_MINIMUM {
                return;
            }
            new_xoff = x + 1 as ::core::ffi::c_int;
            geometry = layout_geometry {
                sx: new_sx as u_int,
                sy: geometry.sy,
                xoff: new_xoff,
                yoff: geometry.yoff,
            };
            resizes += 1;
        } else if ly == pane_y + sy {
            new_sy = y - geometry.yoff;
            if new_sy < PANE_MINIMUM {
                return;
            }
            geometry = layout_geometry {
                sx: geometry.sx,
                sy: new_sy as u_int,
                xoff: geometry.xoff,
                yoff: geometry.yoff,
            };
            resizes += 1;
        } else if ly == pane_y - 1 as ::core::ffi::c_int {
            new_xoff = geometry.xoff + (x - lx);
            new_yoff = y + 1 as ::core::ffi::c_int;
            geometry = layout_geometry {
                sx: geometry.sx,
                sy: geometry.sy,
                xoff: new_xoff,
                yoff: new_yoff,
            };
            resizes += 1;
        }
        if resizes != 0 as ::core::ffi::c_int {
            {
                let mut cell = window_owner
                    .borrow_layout_cell_mut(cell_id)
                    .expect("dragged pane belongs to layout");
                cell.g = geometry;
            }
            layout_fix_panes(&window_owner, None);
            server_redraw_window(&window_owner);
            server_redraw_window_borders(&window_owner);
        }
    })();
    window_owner.release(c"cmd_resize_pane_mouse_resize_move_floating");
}
unsafe fn cmd_resize_pane_mouse_resize_tiled(client_owner: &ClientRef, mut m: *mut mouse_event) {
    let mut c: Option<ClientRef> = Some(client_owner.clone());
    let mut wl: refbox::Weak<winlink> = refbox::Weak::new();
    let mut y: u_int = 0;
    let mut ly: u_int = 0;
    let mut x: u_int = 0;
    let mut lx: u_int = 0;
    static mut offsets: [[::core::ffi::c_int; 2]; 5] = [
        [0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int],
        [0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int],
        [1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int],
        [0 as ::core::ffi::c_int, -(1 as ::core::ffi::c_int)],
        [-(1 as ::core::ffi::c_int), 0 as ::core::ffi::c_int],
    ];
    let mut resizes = 0u32;
    wl = cmd_mouse_window(m, None);
    if !wl.is_alive() {
        c.as_ref()
            .expect("live client")
            .borrow_terminal_mut()
            .mouse_drag_update = None;
        return;
    }
    let window_owner = wl
        .get_unchecked()
        .window_handle()
        .cloned()
        .expect("mouse window");
    (|| {
        y = (*m).y.wrapping_add((*m).oy);
        x = (*m).x.wrapping_add((*m).ox);
        if (*m).statusat == 0 as ::core::ffi::c_int && y >= (*m).statuslines {
            y = y.wrapping_sub((*m).statuslines);
        } else if (*m).statusat > 0 as ::core::ffi::c_int && y >= (*m).statusat as u_int {
            y = ((*m).statusat - 1 as ::core::ffi::c_int) as u_int;
        }
        ly = (*m).ly.wrapping_add((*m).oy);
        lx = (*m).lx.wrapping_add((*m).ox);
        if (*m).statusat == 0 as ::core::ffi::c_int && ly >= (*m).statuslines {
            ly = ly.wrapping_sub((*m).statuslines);
        } else if (*m).statusat > 0 as ::core::ffi::c_int && ly >= (*m).statusat as u_int {
            ly = ((*m).statusat - 1 as ::core::ffi::c_int) as u_int;
        }
        let cells = {
            let tree = window_owner.borrow_layout_root(crate::src::window::LayoutView::Visible);
            let Some(root) = tree.as_deref() else {
                return;
            };
            let root = (root as *const layout_cell).cast_mut();
            let mut cells = Vec::new();
            for offset in offsets {
                let cell = layout_search_by_border(
                    root,
                    lx.wrapping_add(offset[0] as u_int),
                    ly.wrapping_add(offset[1] as u_int),
                );
                if !cell.is_null() && !cells.contains(&(*cell).id()) {
                    cells.push((*cell).id());
                }
            }
            cells
        };
        for id in cells {
            let direction = {
                let tree = window_owner.borrow_layout_root(crate::src::window::LayoutView::Visible);
                tree.as_deref()
                    .and_then(|root| root.find(id))
                    .and_then(|cell| cell.parent.as_ref())
                    .map(|parent| parent.type_0)
            };
            let change = match direction {
                Some(LAYOUT_TOPBOTTOM) if y != ly => y.wrapping_sub(ly) as i32,
                Some(LAYOUT_LEFTRIGHT) if x != lx => x.wrapping_sub(lx) as i32,
                _ => continue,
            };
            if layout_resize_layout(&window_owner, id, direction.unwrap(), change, 0) {
                resizes = resizes.wrapping_add(1);
            }
        }
        if resizes != 0 as u_int {
            server_redraw_window(&window_owner);
        }
    })();
    window_owner.release(c"cmd_resize_pane_mouse_resize_tiled");
}
