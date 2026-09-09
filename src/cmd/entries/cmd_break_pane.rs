//! `break-pane`: takes a pane out of its window and gives it a window of its
//! own in the destination session.
//!
//! The command has two halves, told apart by how many panes the source window
//! holds. A window with nothing but the pane in it already *is* the window
//! being asked for, so `server_link_window` relinks it into the destination
//! and [`SessionRef::unlink_window`] takes it out of the source. Otherwise the pane
//! is unlinked from its window's `panes` and `z_index` lists, `window_create`
//! hands back an empty window, the pane becomes its only one, and after a
//! layout and a name `session_attach` puts that window into the destination
//! session. `-a`/`-b` shuffle the destination's later windows up first to free
//! an index, and `-P` prints the result through `-F`'s format or
//! [`BREAK_PANE_TEMPLATE`].
//!
//! Each window owns its panes. Breaking a pane transfers that owned value
//! while retaining the source and destination owners. The index changes
//! reported by insertion keep the source link current within one session.

use crate::cmd::cmd_get_args;

use crate::fmt_args;
use crate::format::{format_create_for_client, format_defaults_for_handles, format_expand};

use crate::server::server_link_window;

use crate::cmd::cmdq_item;
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::consts::{
    CMD_FIND_PANE, CMD_FIND_WINDOW, CMD_FIND_WINDOW_INDEX, CMD_RETURN_ERROR, CMD_RETURN_NORMAL,
};
use crate::tmux::check_name;
use crate::types::{OptionsRef, args_parse_t};
#[cfg(test)]
use crate::types::{SessionRef, WindowRef, cmd_find_state, u_int, winlink};
use crate::window::WinlinkRef;
use ::core::ffi::{c_char, c_int};

pub const BREAK_PANE_TEMPLATE: &core::ffi::CStr = c"#{session_name}:#{window_index}.#{pane_index}";
pub(crate) static cmd_break_pane_entry: RustCommandEntry = RustCommandEntry {
    name: c"break-pane",
    alias: Some(c"breakp"),
    args: args_parse_t {
        template: c"abdPF:n:s:t:",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"[-abdP] [-F format] [-n window-name] [-s src-pane] [-t dst-window]",
    source: cmd_entry_flag {
        flag: b's' as c_char,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: b't' as c_char,
        type_0: CMD_FIND_WINDOW,
        flags: CMD_FIND_WINDOW_INDEX,
    },
    flags: 0,
    exec: cmd_break_pane_exec,
};

/// Runs `break-pane`, as the command queue calls it.
///
/// The single-pane half's look-up of the relinked window cannot fail, so the
/// C's refusal when it does is not written out. A zero from
/// `server_link_window` means its `session_attach` linked `w` into
/// `dst_s->windows`. `SessionRef::unlink_window` removes the source link and
/// may destroy the source group, but the destination link survives: linking
/// refuses an index already holding `w` ("same index") and two different
/// sessions sharing a group. Group synchronization skips the source itself.
/// So a winlink for `w` remains in `dst_s->windows` when it is looked up.
unsafe fn cmd_break_pane_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args = cmd_get_args(self_0);
    let current_state_ref = item.state_ref();
    let tc = item.target_client();
    let mut source = item.source.session().expect("a break source has a session");
    let mut destination = item
        .target
        .session()
        .expect("a break destination has a session");
    let mut source_index = item
        .source
        .wl_idx
        .expect("a break source has a window link");
    let source_pane = item.source.pane_ref().expect("a break source has a pane");
    let mut window = item.source.window().expect("a break source has a window");
    let mut index = item.target.idx;
    let name = args.argument_flag_string(b'n');
    if let Some(name) = name
        && unsafe { check_name(Some(name)) == 0 }
    {
        unsafe { item.error(c"invalid window name: %s", fmt_args![name]) };
        return CMD_RETURN_ERROR;
    }
    let before = args.argument_flag_count(b'b');
    if args.argument_flag_count(b'a') != 0 || before != 0 {
        let around = item
            .target
            .wl_idx
            .filter(|index| unsafe { destination.link(*index).is_some() })
            .or_else(|| unsafe { destination.current_index() });
        let Some(shift) = (unsafe { destination.shuffle_windows(around, before) }) else {
            return CMD_RETURN_ERROR;
        };
        index = shift.index;
        if source.ptr_eq(&destination) {
            source_index = shift.remap(source_index);
        }
    }
    unsafe { window.unzoom_and_redraw() };

    let destination_index;
    if window.pane_count() == 1 {
        let source_link = WinlinkRef::new(source.clone(), source_index)
            .expect("the break source link is present");
        let mut cause = None;
        if unsafe {
            server_link_window(
                &source_link,
                &mut destination,
                index,
                0,
                (args.argument_flag_count(b'd') == 0) as c_int,
                &mut cause,
            ) != 0
        } {
            unsafe { item.error(c"%s", fmt_args![cause.as_deref()]) };
            return CMD_RETURN_ERROR;
        }
        if let Some(name) = name {
            unsafe { window.set_name(name, 0) };
            unsafe { window.options().set_number(c"automatic-rename", 0) };
        }
        unsafe { source.unlink_window(source_index) };
        destination_index = unsafe {
            destination
                .first_link_to(&window)
                .expect("the moved window remains linked")
                .index()
        };
    } else {
        if unsafe { index != -1 && destination.link(index).is_some() } {
            unsafe { item.error(c"index in use: %d", fmt_args![index]) };
            return CMD_RETURN_ERROR;
        }
        window = unsafe { window.break_pane_into_window(&source_pane) };
        unsafe { window.set_latest_client(tc.as_ref()) };
        if let Some(name) = name {
            unsafe { window.set_name(name, 0) };
            unsafe { window.options().set_number(c"automatic-rename", 0) };
        } else {
            let name = unsafe { window.default_name() };
            unsafe { window.set_name(&name, 0) };
        }
        unsafe { window.finish_broken_pane_layout(&source_pane) };
        if index == -1 {
            unsafe { index = (-1 - destination.options().number(c"base-index")) as c_int };
        }
        let mut cause = None;
        unsafe {
            destination_index = destination
                .attach(window.clone(), index, &mut cause)
                .expect("the destination index is available")
        };
        if args.argument_flag_count(b'd') == 0 {
            unsafe { destination.select(destination_index) };
            unsafe { current_state_ref.update_current_session(&destination, 0) };
        }
        unsafe { source.request_redraw() };
        if !source.ptr_eq(&destination) {
            unsafe { destination.request_redraw() };
        }
        unsafe { source.status_group() };
        if !source.ptr_eq(&destination) {
            unsafe { destination.status_group() };
        }
    }

    if args.argument_flag_count(b'P') != 0 {
        let template = args
            .argument_flag_string(b'F')
            .unwrap_or(BREAK_PANE_TEMPLATE);
        let destination_link = unsafe { destination.link(destination_index) };
        let cp = unsafe {
            let mut ft = format_create_for_client(item.client().as_ref(), Some(item), 0, 0);
            format_defaults_for_handles(
                &mut ft,
                tc.as_ref(),
                Some(&destination),
                destination_link.as_ref(),
                Some(&source_pane),
            );
            format_expand(&mut ft, template)
        };
        unsafe { item.print(c"%s", fmt_args![cp.as_c_str()]) };
    }
    CMD_RETURN_NORMAL
}

#[cfg(test)]
#[path = "../../tests/test_cmd_break_pane.rs"]
mod tests;
