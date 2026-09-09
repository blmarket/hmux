//! `copy-mode` and `clock-mode`: put a pane into one of the window modes.
//! Both are the same exec routine, told apart by the entry the command
//! carries — `clock-mode` opens the clock and stops there, everything else in
//! the routine belongs to `copy-mode`.
//!
//! Which pane the mode lands on is the resolved target, or, with `-M`, the
//! pane the mouse report names; `-s` names a second pane whose screen the copy
//! mode reads from instead of the target's own. `-q` is the way back out and
//! takes every mode off the pane. Once the mode is open the remaining flags
//! move the view: `-u` a page back, `-d` a page on (with `-e` to leave the
//! mode at the bottom) and `-S` to wherever the scrollbar slider was dragged.
//!
//! Line numbers are on unless the key that ran the command was a mouse key,
//! which is what [`key_is_mouse`] decides, and they are applied whether the
//! mode was opened by this call or was already there.
//!
//! Quirks kept: `-M` reads the event's mouse report before the null check the
//! line-number test makes of the same pointer, and `-S` reads the client's
//! terminal without checking that there is a client, so either would follow a
//! null pointer on a command queue that had neither; `-q` is tested first, so
//! `copy-mode -q` ignores every other flag; `clock-mode` is tested before `-s`
//! and the flags below it, so a clock item never reaches them; and the mode's
//! line numbers are set on both halves of the "was it already open" branch,
//! only the drag start being conditional.

use crate::arguments::args_has;

use crate::cmd::{cmd_get_args, cmd_get_entry, cmd_mouse_pane};
pub use crate::consts::{
    CMD_AFTERHOOK, CMD_FIND_PANE, CMD_READONLY, CMD_RETURN_NORMAL, KEYC_MASK_KEY, KEYC_MASK_TYPE,
    KEYC_MOUSE, KEYC_TYPE_MOUSEMOVE, KEYC_TYPE_TRIPLECLICK,
};
pub use crate::types::{
    ClientRef, RustCommandEntry, WindowMode, args_parse_t, cmd, cmd_entry_flag, cmd_retval,
    cmdq_item, key_code,
};

pub const CMD_TARGET_PANE_USAGE: &core::ffi::CStr = c"[-t target-pane]";
pub(crate) static cmd_copy_mode_entry: RustCommandEntry = RustCommandEntry {
    name: c"copy-mode",
    alias: None,
    args: args_parse_t {
        template: c"deHMqSs:t:u",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"[-deHMqSu] [-s src-pane] [-t target-pane]",
    source: cmd_entry_flag {
        flag: b's' as core::ffi::c_char,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: b't' as core::ffi::c_char,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    flags: CMD_AFTERHOOK | CMD_READONLY,
    exec: cmd_copy_mode_exec,
};
pub(crate) static cmd_clock_mode_entry: RustCommandEntry = RustCommandEntry {
    name: c"clock-mode",
    alias: None,
    args: args_parse_t {
        template: c"t:",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: CMD_TARGET_PANE_USAGE,
    source: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: b't' as core::ffi::c_char,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    flags: CMD_AFTERHOOK,
    exec: cmd_copy_mode_exec,
};

/// tmux's `KEYC_IS_MOUSE`: a key is a mouse key when it is the mouse key
/// itself, or when its type is one of the mouse types, which the enum keeps in
/// one run from a move to a triple click.
fn key_is_mouse(key: key_code) -> bool {
    let type_0 = key & KEYC_MASK_TYPE;
    key & KEYC_MASK_KEY == KEYC_MOUSE as key_code
        || (type_0 >= (KEYC_TYPE_MOUSEMOVE as key_code) << 32
            && type_0 <= (KEYC_TYPE_TRIPLECLICK as key_code) << 32)
}

unsafe fn cmd_copy_mode_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args = cmd_get_args(self_0);
    let event_state_ref = item.state_ref();
    let event = event_state_ref.event_snapshot();
    let mut c = item.client();
    let mut pane = item.target.pane_ref().expect("copy target has a pane");

    if args_has(args, b'q') != 0 {
        unsafe { pane.reset_modes() };
        return CMD_RETURN_NORMAL;
    }

    if args_has(args, b'M') != 0 {
        let Some((s, _, mouse_wp)) = (unsafe { cmd_mouse_pane(&event.m) }) else {
            return CMD_RETURN_NORMAL;
        };
        pane = mouse_wp;
        if !c
            .as_ref()
            .and_then(|client| client.attached_session())
            .is_some_and(|session| s.ptr_eq(&session))
        {
            return CMD_RETURN_NORMAL;
        }
    }

    let source_pane = if args_has(args, b's') != 0 {
        item.source.pane_ref()
    } else {
        Some(pane.clone())
    };
    if core::ptr::eq(cmd_get_entry(self_0), &cmd_clock_mode_entry) {
        unsafe {
            pane.set_mode(None, WindowMode::Clock, None, None)
                .expect("mode target is present")
        };
        return CMD_RETURN_NORMAL;
    }

    let line_numbers = if key_is_mouse(event.key) { 0 } else { 1 };
    let opened = unsafe {
        pane.set_mode(source_pane, WindowMode::Copy, None, Some(args))
            .expect("mode target is present")
            == 0
    };
    unsafe { pane.copy_line_numbers(line_numbers) };
    if opened && args_has(args, b'M') != 0 {
        unsafe { ClientRef::start_copy_drag(c.as_mut(), &event.m) };
    }

    if args_has(args, b'u') != 0 {
        unsafe { pane.copy_page_up(0) };
    }
    if args_has(args, b'd') != 0 {
        unsafe { pane.copy_page_down(0, args_has(args, b'e')) };
    }
    if args_has(args, b'S') != 0 {
        let c = c.as_ref().expect("the command has a client");
        unsafe { c.scroll_copy_pane(&pane, event.m.y, args_has(args, b'e')) };
        return CMD_RETURN_NORMAL;
    }

    CMD_RETURN_NORMAL
}
