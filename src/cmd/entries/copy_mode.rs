use crate::src::arguments::args_has;
use crate::src::cmd::queue::{cmdq_get_client, cmdq_get_event, cmdq_get_source, cmdq_get_target};
use crate::src::cmd::{cmd_get_args_mut, cmd_get_entry, cmd_mouse_pane};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::command::{CMD_AFTERHOOK, CMD_READONLY, CMD_TARGET_PANE_USAGE};
use crate::src::shared::key::key_event;
use crate::src::shared::key::*;
use crate::src::shared::pane::window_pane;
use crate::src::shared::session::session;
use crate::src::shared::window::winlink;
use crate::src::tty::tty_window_offset;
use crate::src::window::{window_pane_reset_mode_all, window_pane_set_mode};
use crate::src::window_clock::window_clock_mode;
use crate::src::window_copy::{
    window_copy_mode, window_copy_pagedown, window_copy_pageup, window_copy_scroll,
    window_copy_set_line_numbers, window_copy_start_drag,
};
pub static cmd_copy_mode_entry: cmd_entry = {
    cmd_entry {
        name: c"copy-mode",
        alias: None,
        args: args_parse {
            template: c"dekHMqSs:t:u",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-dekHMqSu] [-s src-pane] [-t target-pane]",
        source: cmd_entry_flag {
            flag: 's' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK | CMD_READONLY,
        exec: Some(cmd_copy_mode_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
pub static cmd_clock_mode_entry: cmd_entry = {
    cmd_entry {
        name: c"clock-mode",
        alias: None,
        args: args_parse {
            template: c"t:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: CMD_TARGET_PANE_USAGE,
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
        exec: Some(cmd_copy_mode_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
unsafe fn cmd_copy_mode_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args_mut(&mut *self_0).map_or(std::ptr::null_mut(), |args| args);
    let mut event: *mut key_event = cmdq_get_event(item);
    let mut source: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_source_mut(&mut *item);
    let mut target: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let c_owner = cmdq_get_client(item);
    let mut c: *mut client = c_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wp: *mut window_pane = (*target).wp_ptr();
    let mut swp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut line_numbers: ::core::ffi::c_int = 0;
    if args_has(args, 'q' as i32 as u_char) != 0 {
        window_pane_reset_mode_all(wp);
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'M' as i32 as u_char) != 0 {
        wp = cmd_mouse_pane(
            &raw mut (*event).m,
            &raw mut s,
            ::core::ptr::null_mut::<*mut winlink>(),
        );
        if wp.is_null() {
            return CMD_RETURN_NORMAL;
        }
        if c.is_null() || (*c).session != s {
            return CMD_RETURN_NORMAL;
        }
    }
    if std::ptr::eq(cmd_get_entry(&*self_0), &cmd_clock_mode_entry) {
        window_pane_set_mode(
            wp,
            ::core::ptr::null_mut::<window_pane>(),
            &window_clock_mode,
            item,
            ::core::ptr::null_mut::<cmd_find_state>(),
            ::core::ptr::null_mut::<args>(),
        );
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 's' as i32 as u_char) != 0 {
        swp = (*source).wp_ptr();
    } else {
        swp = wp;
    }
    line_numbers = 1 as ::core::ffi::c_int;
    if !event.is_null()
        && ((*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
            == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
            || (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
                && (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                    <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                        << 32 as ::core::ffi::c_int)
    {
        line_numbers = 0 as ::core::ffi::c_int;
    }
    if window_pane_set_mode(
        wp,
        swp,
        &window_copy_mode,
        item,
        ::core::ptr::null_mut::<cmd_find_state>(),
        args,
    ) == 0
    {
        window_copy_set_line_numbers(wp, line_numbers);
        if args_has(args, 'M' as i32 as u_char) != 0 {
            window_copy_start_drag(c, &raw mut (*event).m);
        }
    } else {
        window_copy_set_line_numbers(wp, line_numbers);
    }
    if args_has(args, 'u' as i32 as u_char) != 0 {
        window_copy_pageup(wp);
    }
    if args_has(args, 'd' as i32 as u_char) != 0 {
        window_copy_pagedown(wp, args_has(args, 'e' as i32 as u_char));
    }
    if args_has(args, 'S' as i32 as u_char) != 0 {
        let tty_oy = tty_window_offset(&(*c).tty).oy;
        window_copy_scroll(
            wp,
            (*c).tty.mouse_slider_mpos,
            (*event).m.y,
            tty_oy,
            args_has(args, 'e' as i32 as u_char),
        );
        return CMD_RETURN_NORMAL;
    }
    return CMD_RETURN_NORMAL;
}
