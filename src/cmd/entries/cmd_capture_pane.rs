//! `capture-pane`: copies what a pane holds into a paste buffer or onto a
//! client's stdout, and `clear-history`, which throws the pane's scrollback
//! away. Both are the same exec routine, told apart by the entry the command
//! carries.
//!
//! A capture is a range of grid lines — `-S` and `-E` name its ends, counting
//! zero from the top of the screen and downwards into history — read out one
//! line at a time and joined into one buffer. Which grid is read depends on
//! `-a` (the saved alternate screen), `-M` (the screen the pane's mode hands
//! over) and otherwise the pane's own; `-P` reads the input parser's pending
//! bytes instead of the grid at all, and `-H` reads the hyperlink URIs the
//! lines carry rather than their text. The flags that shape a line —
//! escape sequences, escaped control bytes, empty cells, trimmed trailing
//! spaces — are handed to `grid_string_cells`, which is what turns cells into
//! text.
//!
//! The answer is built in a `Vec<u8>` which the paste store takes over, or which
//! `-p` writes out after appending the terminator the printing path wants.
//!
//! Quirks kept: the width every line is read out to comes from the pane's own
//! base grid even when `-a` or `-M` picked a different one; a `-J` capture
//! whose last line is wrapped ends without a newline; a line whose hyperlink
//! collection came out empty contributes nothing at all, newline included;
//! and `-p` reads the client's flags without checking that there is a client.

use crate::args::RustArguments;
use crate::args::{args_get_str, args_has, args_strtonum_and_expand};

use crate::cmd::{cmd_get_args, cmd_get_entry};
use crate::consts::{
    __INT_MAX__, CMD_AFTERHOOK, CMD_FIND_PANE, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, SHRT_MAX,
};
use crate::fmt_args;
use crate::pane_handle::{CapturePaneEdge, PaneCapture};
use crate::paste::{PasteBufferStore, paste_buffer_limit, with_paste_buffers_mut};
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::cmd::cmdq_item;
use crate::types::{RustWindowPaneWeak, args_parse_t, size_t, u_char};
use ::core::ffi::{c_char, c_int, c_longlong};
pub const INT_MIN: c_int = -__INT_MAX__ - 1 as c_int;

pub(crate) static cmd_capture_pane_entry: RustCommandEntry = RustCommandEntry {
    name: c"capture-pane",
    alias: Some(c"capturep"),
    args: args_parse_t {
        template: c"ab:CeE:FHJLMNpPqS:Tt:",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"[-aCeFHJLMNpPqT] [-b buffer-name] [-E end-line] [-S start-line] [-t target-pane]",
    source: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: b't' as c_char,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    flags: CMD_AFTERHOOK,
    exec: cmd_capture_pane_exec,
};
pub(crate) static cmd_clear_history_entry: RustCommandEntry = RustCommandEntry {
    name: c"clear-history",
    alias: Some(c"clearhist"),
    args: args_parse_t {
        template: c"Ht:",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"[-H] [-t target-pane]",
    source: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: b't' as c_char,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    flags: CMD_AFTERHOOK,
    exec: cmd_capture_pane_exec,
};

unsafe fn cmd_capture_pane_edge(args: &RustArguments, item: &cmdq_item, flag: u_char) -> CapturePaneEdge {
    unsafe {
        if args_get_str(args, flag) == Some(c"-") {
            return CapturePaneEdge::Dash;
        }
        let mut cause = None;
        let n = args_strtonum_and_expand(
            args,
            flag,
            INT_MIN as c_longlong,
            SHRT_MAX as c_longlong,
            item,
            &mut cause,
        ) as c_int;
        if cause.is_some() {
            CapturePaneEdge::Default
        } else {
            CapturePaneEdge::Line(n)
        }
    }
}

/// The captured text of the range of grid lines the arguments name, or
/// nothing at all when the capture was refused and the reason already
/// reported.
unsafe fn cmd_capture_pane_history(
    args: &RustArguments,
    item: &cmdq_item,
    pane: &RustWindowPaneWeak,
) -> Option<Vec<u8>> {
    unsafe {
        if args_has(args, b'a') != 0 && !pane.has_saved_screen()? {
            if args_has(args, b'q') == 0 {
                item.error(c"no alternate screen", fmt_args![]);
                return None;
            }
            return Some(Vec::new());
        }
        let start = cmd_capture_pane_edge(args, item, b'S');
        let end = cmd_capture_pane_edge(args, item, b'E');
        let capture = PaneCapture {
            start,
            end,
            alternate: args_has(args, b'a') != 0,
            mode: args_has(args, b'M') != 0,
            join_lines: args_has(args, b'J') != 0,
            sequences: args_has(args, b'e') != 0,
            escape: args_has(args, b'C') != 0,
            empty_cells: args_has(args, b'T') == 0,
            trim_spaces: args_has(args, b'N') == 0,
            number_lines: args_has(args, b'L') != 0,
            show_flags: args_has(args, b'F') != 0,
            hyperlinks: args_has(args, b'H') != 0,
        };
        match pane.capture_history(capture)? {
            Ok(bytes) => Some(bytes),
            Err(()) if args_has(args, b'q') != 0 => Some(Vec::new()),
            Err(()) => {
                item.error(c"no alternate screen", fmt_args![]);
                None
            }
        }
    }
}

unsafe fn cmd_capture_pane_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args = cmd_get_args(self_0);
    let mut c = item.client();
    let pane = item
        .target
        .pane_ref()
        .expect("the command target has a pane");
    if core::ptr::eq(cmd_get_entry(self_0), &cmd_clear_history_entry) {
        unsafe { pane.clear_history(args_has(args, b'H') != 0) };
        return CMD_RETURN_NORMAL;
    }
    let bytes = if args_has(args, b'P') != 0 && args_has(args, b'H') == 0 {
        unsafe {
            pane.capture_pending(args_has(args, b'C') != 0)
                .expect("the target pane is live")
        }
    } else {
        match unsafe { cmd_capture_pane_history(args, item, &pane) } {
            Some(bytes) => bytes,
            None => return CMD_RETURN_ERROR,
        }
    };
    if args_has(args, b'p') != 0 {
        let c = c.as_mut().expect("the command has a client");
        let mut len = bytes.len() as size_t;
        if len > 0 && bytes[len.wrapping_sub(1)] == b'\n' {
            len = len.wrapping_sub(1);
        }
        if !unsafe { c.print_capture(&bytes[..len as usize]) } {
            unsafe { item.error(c"can't write to client", fmt_args![]) };
            return CMD_RETURN_ERROR;
        }
    } else {
        let bufname = if args_has(args, b'b') != 0 {
            args_get_str(args, b'b')
        } else {
            None
        };
        let result = if let Some(name) = bufname {
            with_paste_buffers_mut(|buffers| buffers.set_named(name, bytes))
        } else {
            let limit = unsafe { paste_buffer_limit() };
            with_paste_buffers_mut(|buffers| buffers.add_automatic(None, bytes, limit));
            Ok(())
        };
        if let Err(cause) = result {
            unsafe { item.error(c"%s", fmt_args![cause.as_c_str()]) };
            return CMD_RETURN_ERROR;
        }
    }
    CMD_RETURN_NORMAL
}
