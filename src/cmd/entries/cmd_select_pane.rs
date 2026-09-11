use crate::args::args_parse_t;
use crate::cmd::{cmd_get_args, cmd_get_entry};
use crate::fmt_args;
use crate::format::format_single_from_target;

use crate::server::client_walk;

use crate::cmd::cmdq_item;
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::consts::{
    CLIENT_ACTIVEPANE, CMD_FIND_PANE, CMD_FIND_WINDOW, CMD_RETURN_ERROR, CMD_RETURN_NORMAL,
};
#[cfg(test)]
use crate::tty::tty_window_bigger;
use crate::types::{OptionsRef, WindowRef};

pub(crate) static cmd_select_pane_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"select-pane",
        alias: Some(c"selectp"),
        args: args_parse_t {
            template: c"DdegLlMmP:RT:t:UZ",
            lower: 0 as core::ffi::c_int,
            upper: 0 as core::ffi::c_int,
            cb: None,
        },
        usage: c"[-DdeLlMmRUZ] [-T title] [-t target-pane]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as core::ffi::c_int,
        },
        flags: 0 as core::ffi::c_int,
        exec: cmd_select_pane_exec,
    }
};
pub(crate) static cmd_last_pane_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"last-pane",
        alias: Some(c"lastp"),
        args: args_parse_t {
            template: c"det:Z",
            lower: 0 as core::ffi::c_int,
            upper: 0 as core::ffi::c_int,
            cb: None,
        },
        usage: c"[-deZ] [-t target-window]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: 0 as core::ffi::c_int,
        },
        flags: 0 as core::ffi::c_int,
        exec: cmd_select_pane_exec,
    }
};
unsafe fn cmd_select_pane_redraw(w: &WindowRef) {
    unsafe {
        for mut owner in client_walk() {
            owner.redraw_pane_selection(w);
        }
    }
}
unsafe fn cmd_select_pane_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args = cmd_get_args(self_0);
    let entry = cmd_get_entry(self_0);
    let current_state_ref = item.state_ref();
    let mut c = item.client();
    let link = item
        .target
        .winlink_ref()
        .expect("the target has a window link");
    let session = link.session().clone();
    let window = link.window().expect("the target has a window");
    let mut pane = item.target.pane_ref().expect("the target has a pane");
    let options = unsafe { pane.options().expect("the target pane is present") };
    if core::ptr::eq(entry, &cmd_last_pane_entry) || args.argument_flag_count(b'l') != 0 {
        let mut last = window.last_pane();
        if last.is_none() && window.pane_count() == 2 {
            last = window
                .active_pane()
                .and_then(|pane| window.adjacent_pane(&pane));
        }
        let Some(last) = last else {
            unsafe { item.error(c"no last pane", fmt_args![]) };
            return CMD_RETURN_ERROR;
        };
        if args.argument_flag_count(b'e') != 0 {
            unsafe { window.set_pane_input_enabled(&last, true) };
        } else if args.argument_flag_count(b'd') != 0 {
            unsafe { window.set_pane_input_enabled(&last, false) };
        } else {
            if unsafe { window.push_zoom(0, args.argument_flag_count(b'Z')) != 0 } {
                unsafe { window.redraw() };
            }
            unsafe {
                window.redraw_active_switch(
                    &crate::window::window_pane_find_by_id(last.id())
                        .expect("the selected pane exists"),
                )
            };
            if unsafe {
                window.set_active_pane(
                    &crate::window::window_pane_find_by_id(last.id())
                        .expect("the selected pane exists"),
                    1,
                ) != 0
            } {
                unsafe { current_state_ref.update_current_link(&link, None, 0) };
                unsafe { cmd_select_pane_redraw(&window) };
            }
            if unsafe { window.pop_zoom() != 0 } {
                unsafe { window.redraw() };
            }
        }
        return CMD_RETURN_NORMAL;
    }
    if args.argument_flag_count(b'm') != 0 || args.argument_flag_count(b'M') != 0 {
        if unsafe { args.argument_flag_count(b'm') != 0 && !window.contains_visible_pane(&pane) } {
            return CMD_RETURN_NORMAL;
        }
        unsafe {
            crate::server::server_toggle_marked_pane(
                &link,
                &pane,
                args.argument_flag_count(b'M') != 0,
            )
        };
        if window.pane_is_floating(&pane) {
            unsafe {
                window.redraw_active_switch(
                    &crate::window::window_pane_find_by_id(pane.id())
                        .expect("the selected pane exists"),
                )
            };
            unsafe {
                window.set_active_pane(
                    &crate::window::window_pane_find_by_id(pane.id())
                        .expect("the selected pane exists"),
                    1,
                )
            };
        }
        return CMD_RETURN_NORMAL;
    }
    if let Some(style) = args.argument_flag_string(b'P') {
        unsafe { pane.set_window_style(style) };
    }
    if args.argument_flag_count(b'g') != 0 {
        unsafe {
            item.print(
                c"%s",
                fmt_args![options.string_ref(c"window-style").as_ref()],
            )
        };
        return CMD_RETURN_NORMAL;
    }
    let direction = if args.argument_flag_count(b'L') != 0 {
        Some(crate::window_pane::PaneDirection::Left)
    } else if args.argument_flag_count(b'R') != 0 {
        Some(crate::window_pane::PaneDirection::Right)
    } else if args.argument_flag_count(b'U') != 0 {
        Some(crate::window_pane::PaneDirection::Up)
    } else if args.argument_flag_count(b'D') != 0 {
        Some(crate::window_pane::PaneDirection::Down)
    } else {
        None
    };
    if let Some(find) = direction {
        unsafe { window.push_zoom(0, 1) };
        let selected = unsafe { pane.neighbor(find) };
        unsafe { window.pop_zoom() };
        let Some(selected) = selected else {
            return CMD_RETURN_NORMAL;
        };
        pane = selected;
    }
    if args.argument_flag_count(b'e') != 0 {
        unsafe { window.set_pane_input_enabled(&pane, true) };
        return CMD_RETURN_NORMAL;
    }
    if args.argument_flag_count(b'd') != 0 {
        unsafe { window.set_pane_input_enabled(&pane, false) };
        return CMD_RETURN_NORMAL;
    }
    if let Some(given) = args.argument_flag_string(b'T') {
        let title = unsafe { format_single_from_target(item, given) };
        let Some(changed) = (unsafe { pane.set_title(&title) }) else {
            return CMD_RETURN_NORMAL;
        };
        if changed {
            unsafe { pane.notify(c"pane-title-changed") };
            if let Some(window) = pane.window() {
                unsafe { window.redraw_borders() };
                unsafe { window.redraw_status() };
            }
        }
        return CMD_RETURN_NORMAL;
    }
    let client_active = c.as_ref().is_some_and(|client| {
        !{ client.attached_session() }.is_none()
            && unsafe { client.flags() } & CLIENT_ACTIVEPANE != 0
    });
    let active_id = if client_active {
        unsafe { c.as_ref().unwrap().selected_pane().map(|pane| pane.id()) }
    } else {
        window.active_pane_id()
    };
    if Some(pane.id()) == active_id {
        return CMD_RETURN_NORMAL;
    }
    if unsafe { window.push_zoom(0, args.argument_flag_count(b'Z')) != 0 } {
        unsafe { window.redraw() };
    }
    unsafe {
        window.redraw_active_switch(
            &crate::window::window_pane_find_by_id(pane.id()).expect("the selected pane exists"),
        )
    };
    if client_active {
        unsafe { c.as_mut().unwrap().select_client_pane(&pane) };
    } else if unsafe {
        window.set_active_pane(
            &crate::window::window_pane_find_by_id(pane.id()).expect("the selected pane exists"),
            1,
        ) != 0
    } {
        unsafe { current_state_ref.update_current_link(&link, Some(&pane), 0) };
    }
    let current = current_state_ref.current_snapshot();
    unsafe {
        (crate::cmd::cmdq_item_ref_of(item).expect("the command has an owner")).insert_session_hook(
            Some(&session),
            Some(&current),
            c"after-select-pane",
            fmt_args![],
        )
    };
    unsafe { cmd_select_pane_redraw(&window) };
    if unsafe { window.pop_zoom() != 0 } {
        unsafe { window.redraw() };
    }
    CMD_RETURN_NORMAL
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::test_fixtures::{Clients, Target, globals};

    #[test]
    fn redraw_with_no_current_window_updates_status_and_skips_control_clients() {
        let _guard = globals();
        let mut target = Target::new(20, 5);
        let window = target.state().window().unwrap();
        let mut session = target.session_handle().clone();
        let mut clients = Clients::new();
        clients.add("attached", 20, 5);
        clients.add("control", 20, 5);
        let mut owners: Vec<_> = client_walk().collect();
        unsafe {
            let tty_owner = owners[0].downgrade();
            owners[0].as_tty_mut().client = Some(tty_owner);
            owners[0].set_attached_session(Some(&session));
            *owners[0].flags_mut() = 0;
            owners[1].set_attached_session(Some(&session));
            *owners[1].flags_mut() = CLIENT_CONTROL as u64;
            let current = session.as_session_mut().curw.take();
            cmd_select_pane_redraw(&window);
            assert_eq!(owners[0].flags(), CLIENT_REDRAWSTATUS as u64);
            assert_eq!(owners[1].flags(), CLIENT_CONTROL as u64);
            assert_eq!(tty_window_bigger(owners[0].as_tty()), 0);
            session.as_session_mut().curw = current;
        }
    }
}

#[cfg(test)]
use crate::consts::{CLIENT_CONTROL, CLIENT_REDRAWSTATUS};
