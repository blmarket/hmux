//! Unit tests for [`crate::cmd::cmd_capture_pane`], the `capture-pane` and
//! `clear-history` commands: the metadata their [`cmd_entry`] statics
//! publish, and every deterministic branch of their shared exec routine that
//! can be reached without a live server — dispatch between the two commands,
//! capturing the visible screen, history ranges with their clamps and swaps,
//! line numbering, line-state letters, joined wrapped lines, padding and
//! trimming, escape handling, alternate-screen selection, pending input,
//! hyperlinks, printing to a control client, storing into the paste buffer,
//! and clearing the scrollback.
//!
//! The private helpers (`cmd_capture_pane_append`, `_pending`,
//! `_hyperlinks`, `_history`) are exercised only through the entries' own
//! function pointers, exactly as the command queue calls them, so no
//! test-only visibility changes were needed. Items carry arguments from the
//! real command parser and targets from the [`Target`] fixture; a pane there
//! has no input parser of its own, so the pending-input tests install a
//! test-owned [`input_ctx`] whose `since_ground` buffer is freed on drop.
//! Everything else the exec routine touches beyond its arguments is
//! process-global or ensure_reactor-owned: every test holds [`globals`], captured
//! buffers go into a [`Paste`] store emptied again afterwards, and the
//! control-client path writes through a [`StreamBuffer`] over a local socket
//! pair whose bytes are read back without any event loop. The plain-client
//! print path writes down a [`Peer`] of the same shape — a message buffer
//! over one end of a socket pair — and the messages it queues are read back
//! off the other end. Error branches report onto client-less items, which
//! only record a config cause. Not covered here: the control-mode block
//! queue (only the plain write is reachable without a running control loop)
//! and mode teardown inside `window_pane_reset_mode_all`, which needs panes
//! carrying real modes.

use crate::cmd::cmd_capture_pane::{
    CLIENT_CONTROL, CMD_AFTERHOOK, CMD_FIND_PANE, CMD_RETURN_ERROR, CMD_RETURN_NORMAL,
    GRID_LINE_DEAD, GRID_LINE_EXTENDED, GRID_LINE_HYPERLINK, GRID_LINE_START_OUTPUT,
    GRID_LINE_START_PROMPT, GRID_LINE_WRAPPED, cmd_capture_pane_entry, cmd_clear_history_entry,
};
use crate::compat::{
    imsg_free, imsg_get_len, imsg_get_type, imsgbuf_clear, imsgbuf_flush, imsgbuf_get,
    imsgbuf_init, imsgbuf_read,
};
use crate::file::{CLIENT_ATTACHED, MSG_WRITE, MSG_WRITE_OPEN, file_free};
use crate::fmt_args;
use crate::grid::hyperlinks_put;
use crate::grid::{grid_create, grid_scroll_history, grid_set_cell};
use crate::input::input_ctx;
use crate::modes::window_copy_add;
use crate::paste::{paste_buffer_data, paste_buffer_name, paste_get_name, paste_walk};
use crate::proc::{peer_ptr, tmuxpeer};
use crate::reactor::Buf;
use crate::reactor::IoWatch;
use crate::screen::{screen_grid_mut, screen_grid_ptr, screen_reset_hyperlinks};
use crate::tests::test_fixtures::{
    Item, Paste, StreamBuffer, Target, ascii, ensure_reactor, globals, zeroed,
};
use crate::types::*;
use crate::window::{window_pane_reset_mode, window_pane_set_mode};
use ::core::ffi::{CStr, c_char, c_int};
use ::core::slice;
use ::std::collections::VecDeque;
use ::std::ffi::CString;

/// Where the tests' items claim to come from, which is what `cmdq_error`
/// reports them under.
const FILE: &CStr = c"test-coverage-cmd-capture-pane.conf";

/// The buffer name captures are stored under.
const BUF: &CStr = c"cap";

/// Runs the shared exec routine through the entry's own function pointer,
/// the way the command queue calls it, and answers what it answers.
unsafe fn exec(item: &mut Item) -> cmd_retval {
    unsafe {
        let exec = cmd_capture_pane_entry.exec;
        exec(&*item.cmd(), item.ptr())
    }
}

/// An item carrying a parsed command line, sourced from [`FILE`] and aimed
/// at `target`'s active pane.
fn item(line: &'static CStr, number: u_int, target: &mut Target) -> Item {
    Item::new()
        .from_file(FILE, number)
        .with_args(line)
        .targeting(target)
}

/// Writes `s` over row `py` of the pane's base grid, one ASCII cell per byte.
/// Rows are counted from the first history line, and lines never move under
/// scrolling here, so a row keeps its contents once written.
unsafe fn write_row(wp: *mut window_pane, py: u_int, s: &str) {
    unsafe {
        for (i, byte) in s.bytes().enumerate() {
            let gc = ascii(byte);
            grid_set_cell(screen_grid_mut(&mut (*wp).base), i as u_int, py, &gc);
        }
    }
}

/// Marks row `py` of the pane's base grid with extra line flags.
unsafe fn mark_row(wp: *mut window_pane, py: u_int, flags: c_int) {
    unsafe {
        (*screen_grid_ptr(&mut (*wp).base)).linedata[py as usize].flags |= flags;
    }
}

/// Scrolls `n` lines off the top of the screen into history; each leaves an
/// empty row at the bottom until written over.
unsafe fn scroll(wp: *mut window_pane, n: u_int) {
    unsafe {
        for _ in 0..n {
            grid_scroll_history(screen_grid_mut(&mut (*wp).base), 8);
        }
    }
}

/// Gives every cell of row `py` a link id, as cells written under an OSC 8
/// hyperlink carry one; the grid marks such a line itself.
unsafe fn write_linked_row(wp: *mut window_pane, py: u_int, links: &[u_int]) {
    unsafe {
        for (i, &link) in links.iter().enumerate() {
            let mut gc = ascii(b'a');
            gc.link = link;
            grid_set_cell(screen_grid_mut(&mut (*wp).base), i as u_int, py, &gc);
        }
    }
}

/// The bytes of a paste buffer the store owns.
unsafe fn buffer_bytes(pb: *mut paste_buffer) -> Vec<u8> {
    unsafe { paste_buffer_data(&*pb).to_vec() }
}

/// The bytes stored under [`BUF`], or `None` when no such buffer exists.
unsafe fn captured() -> Option<Vec<u8>> {
    unsafe {
        let pb = paste_get_name(BUF.as_ptr());
        if pb.is_null() {
            None
        } else {
            Some(buffer_bytes(pb))
        }
    }
}

/// A fake input parser for a pane, owning the buffer `input_pending` answers
/// with. Only `since_ground` is read along this path, so the rest of the
/// context stays zeroed.
struct Pending {
    ctx: Option<InputCtxRef>,
    wp: *mut window_pane,
}

impl Pending {
    unsafe fn context(since_ground: Option<Box<Buf>>) -> InputCtxRef {
        InputCtxRef::new(input_ctx {
            owner_of: crate::input::InputOwner::Detached,
            event: crate::reactor::Stream::NONE,
            ctx: crate::screen::screen_write_ctx::default(),
            cell: input_cell {
                cell: unsafe { crate::grid::grid_default_cell },
                set: 0,
                g0set: 0,
                g1set: 0,
            },
            old_cell: input_cell {
                cell: unsafe { crate::grid::grid_default_cell },
                set: 0,
                g0set: 0,
                g1set: 0,
            },
            old_cx: 0,
            old_cy: 0,
            old_mode: 0,
            interm_buf: [0; 4],
            interm_len: 0,
            param_buf: [0; 64],
            param_len: 0,
            input_buf: Vec::new(),
            input_end: crate::input::INPUT_END_ST,
            param_list: [const { crate::input::InputParam::Missing }; 24],
            param_list_len: 0,
            utf8data: crate::text::utf8_data::default(),
            utf8started: 0,
            ch: 0,
            last: crate::text::utf8_data::default(),
            state: &crate::input::input_state_ground,
            flags: 0,
            requests: crate::types::input_request_list::new(),
            request_count: 0,
            request_timer: crate::reactor::TimerHandle(0),
            since_ground,
            ground_timer: crate::reactor::TimerHandle(0),
            owner: None,
        })
    }

    /// A parser whose pending queue holds `bytes`.
    fn with_bytes(bytes: &[u8]) -> Pending {
        let mut evb = Box::new(Buf::new());
        evb.append(bytes);
        Pending {
            ctx: Some(unsafe { Self::context(Some(evb)) }),
            wp: ::core::ptr::null_mut(),
        }
    }

    /// A parser with no queue at all, as a pane that never parsed input.
    fn without_buffer() -> Pending {
        Pending {
            ctx: Some(unsafe { Self::context(None) }),
            wp: ::core::ptr::null_mut(),
        }
    }

    /// Installs the parser on the pane.
    fn install(&mut self, wp: *mut window_pane) {
        self.wp = wp;
        unsafe { (*wp).ictx = self.ctx.take() };
    }
}

impl Drop for Pending {
    fn drop(&mut self) {
        if !self.wp.is_null() {
            unsafe {
                let _ = (*self.wp).ictx.take();
            }
        }
    }
}

/// A control-mode client's write side: the state `control_write` reaches and
/// the buffer event it lands in, over a local socket pair. Nothing runs the
/// event loop, so what was written stays readable through [`Self::written`].
struct Control {
    bev: StreamBuffer,
}

impl Control {
    fn new() -> Control {
        Control {
            bev: StreamBuffer::new(),
        }
    }

    /// Turns the item's client into a control client writing through here.
    fn attach_to(&mut self, item: &mut Item) {
        unsafe {
            let c = item.client();
            let cs = (*c)
                .control_state
                .insert(Box::new(crate::control::control_state::default()));
            cs.write_event = self.bev.ptr();
            (*c).flags |= CLIENT_CONTROL as uint64_t;
        }
    }

    /// What has been written since the last time this was asked.
    fn written(&self) -> Vec<u8> {
        self.bev.written()
    }
}

#[test]
fn the_capture_pane_entry_describes_the_capture_pane_command() {
    let _guard = globals();
    unsafe {
        let e = &raw const cmd_capture_pane_entry;
        assert_eq!((*e).name.to_string_lossy(), "capture-pane");
        assert_eq!(
            (*e).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "capturep"
        );
        assert_eq!(
            (*e).usage.to_string_lossy(),
            "[-aCeFHJLMNpPqT] [-b buffer-name] [-E end-line] [-S start-line] [-t target-pane]"
        );
        assert_eq!(
            (*e).args.template.to_string_lossy(),
            "ab:CeE:FHJLMNpPqS:Tt:"
        );
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, 0);
        assert!((*e).args.cb.is_none());

        assert_eq!((*e).source.flag, 0 as c_char);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);
        assert_eq!((*e).target.flag, 't' as i32 as c_char);
        assert_eq!((*e).target.type_0, CMD_FIND_PANE);
        assert_eq!((*e).target.flags, 0);

        assert_eq!((*e).flags, CMD_AFTERHOOK);
    }
}

#[test]
fn the_clear_history_entry_describes_its_command_and_shares_the_exec_routine() {
    let _guard = globals();
    unsafe {
        let e = &raw const cmd_clear_history_entry;
        assert_eq!((*e).name.to_string_lossy(), "clear-history");
        assert_eq!(
            (*e).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "clearhist"
        );
        assert_eq!((*e).usage.to_string_lossy(), "[-H] [-t target-pane]");
        assert_eq!((*e).args.template.to_string_lossy(), "Ht:");
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, 0);
        assert!((*e).args.cb.is_none());

        assert_eq!((*e).source.flag, 0 as c_char);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);
        assert_eq!((*e).target.flag, 't' as i32 as c_char);
        assert_eq!((*e).target.type_0, CMD_FIND_PANE);
        assert_eq!((*e).target.flags, 0);

        assert_eq!((*e).flags, CMD_AFTERHOOK);
        assert_eq!(
            cmd_capture_pane_entry.exec as usize, cmd_clear_history_entry.exec as usize,
            "both entries run the same routine"
        );
    }
}

#[test]
fn a_plain_capture_stores_the_visible_screen_in_a_named_buffer() {
    let _guard = globals();
    let _paste = Paste::new();
    let mut t = Target::new(10, 3);
    let wp = t.pane(0);
    unsafe {
        write_row(wp, 0, "aaa");
        write_row(wp, 1, "bbb");
        write_row(wp, 2, "ccc");

        let mut it = item(c"capture-pane -b cap", 1, &mut t);
        assert_eq!(exec(&mut it), CMD_RETURN_NORMAL);
        assert_eq!(
            captured(),
            Some(b"aaa\nbbb\nccc\n".to_vec()),
            "every visible line once, each closed with a newline"
        );
    }
}

#[test]
fn a_capture_without_b_gets_an_automatic_buffer_name() {
    let _guard = globals();
    let _paste = Paste::new();
    let mut t = Target::new(10, 3);
    let wp = t.pane(0);
    unsafe {
        write_row(wp, 0, "xyz");

        let mut it = item(c"capture-pane", 1, &mut t);
        assert_eq!(exec(&mut it), CMD_RETURN_NORMAL);

        let pb = paste_walk(::core::ptr::null_mut());
        assert!(!pb.is_null(), "an automatic buffer was created");
        assert!(
            paste_buffer_name(&*pb)
                .to_string_lossy()
                .starts_with("buffer"),
            "{} is not an automatic name",
            paste_buffer_name(&*pb).to_string_lossy()
        );
        assert_eq!(buffer_bytes(pb), b"xyz\n\n\n");
        assert!(paste_walk(pb).is_null(), "only one buffer exists");
    }
}

#[test]
fn the_S_flag_reaches_into_history_and_L_numbers_the_lines() {
    let _guard = globals();
    let _paste = Paste::new();
    let mut t = Target::new(10, 3);
    let wp = t.pane(0);
    unsafe {
        scroll(wp, 2);
        write_row(wp, 0, "h0");
        write_row(wp, 1, "h1");
        write_row(wp, 2, "s0");
        write_row(wp, 3, "s1");
        write_row(wp, 4, "s2");

        let mut deep = item(c"capture-pane -L -S -2 -E -1 -b cap", 1, &mut t);
        assert_eq!(exec(&mut deep), CMD_RETURN_NORMAL);
        assert_eq!(
            captured(),
            Some(b"-2 h0\n-1 h1\n".to_vec()),
            "history lines count up from the oldest, ending negative"
        );

        let mut all = item(c"capture-pane -L -S -1 -b cap", 2, &mut t);
        assert_eq!(exec(&mut all), CMD_RETURN_NORMAL);
        assert_eq!(
            captured(),
            Some(b"-1 h1\n0 s0\n1 s1\n2 s2\n".to_vec()),
            "visible lines count up from zero"
        );
    }
}

#[test]
fn an_S_dash_starts_at_the_first_history_line() {
    let _guard = globals();
    let _paste = Paste::new();
    let mut t = Target::new(10, 3);
    let wp = t.pane(0);
    unsafe {
        scroll(wp, 2);
        write_row(wp, 0, "h0");
        write_row(wp, 1, "h1");
        write_row(wp, 2, "s0");
        write_row(wp, 3, "s1");
        write_row(wp, 4, "s2");

        let mut it = item(c"capture-pane -S - -b cap", 1, &mut t);
        assert_eq!(exec(&mut it), CMD_RETURN_NORMAL);
        assert_eq!(captured(), Some(b"h0\nh1\ns0\ns1\ns2\n".to_vec()));
    }
}

#[test]
fn out_of_range_S_values_clamp_or_fall_back_to_defaults() {
    let _guard = globals();
    let _paste = Paste::new();
    let mut t = Target::new(10, 3);
    let wp = t.pane(0);
    unsafe {
        scroll(wp, 2);
        write_row(wp, 0, "h0");
        write_row(wp, 1, "h1");
        write_row(wp, 2, "s0");
        write_row(wp, 3, "s1");
        write_row(wp, 4, "s2");

        let mut back = item(c"capture-pane -S -100 -b cap", 1, &mut t);
        assert_eq!(exec(&mut back), CMD_RETURN_NORMAL);
        assert_eq!(
            captured(),
            Some(b"h0\nh1\ns0\ns1\ns2\n".to_vec()),
            "too far back starts at the first history line"
        );

        let mut huge = item(c"capture-pane -S 40000 -b cap", 2, &mut t);
        assert_eq!(exec(&mut huge), CMD_RETURN_NORMAL);
        assert_eq!(
            captured(),
            Some(b"s0\ns1\ns2\n".to_vec()),
            "a value past SHRT_MAX is rejected and the start falls back to \
             the top of the screen"
        );
    }
}

#[test]
fn an_E_below_S_swaps_so_the_range_stays_ascending() {
    let _guard = globals();
    let _paste = Paste::new();
    let mut t = Target::new(10, 3);
    let wp = t.pane(0);
    unsafe {
        write_row(wp, 0, "aaa");
        write_row(wp, 1, "bbb");
        write_row(wp, 2, "ccc");

        let mut it = item(c"capture-pane -S 1 -E 0 -b cap", 1, &mut t);
        assert_eq!(exec(&mut it), CMD_RETURN_NORMAL);
        assert_eq!(
            captured(),
            Some(b"aaa\nbbb\n".to_vec()),
            "rows zero and one, in order"
        );
    }
}

#[test]
fn an_E_far_into_history_clamps_to_the_top_when_there_is_none() {
    let _guard = globals();
    let _paste = Paste::new();
    let mut t = Target::new(10, 3);
    let wp = t.pane(0);
    unsafe {
        write_row(wp, 0, "aaa");
        write_row(wp, 1, "bbb");

        let mut it = item(c"capture-pane -E -100 -b cap", 1, &mut t);
        assert_eq!(exec(&mut it), CMD_RETURN_NORMAL);
        assert_eq!(captured(), Some(b"aaa\n".to_vec()));
    }
}

#[test]
fn the_J_flag_joins_a_wrapped_line_with_the_one_after_it() {
    let _guard = globals();
    let _paste = Paste::new();
    let mut t = Target::new(10, 3);
    let wp = t.pane(0);
    unsafe {
        write_row(wp, 0, "aaa");
        write_row(wp, 1, "bbb");
        write_row(wp, 2, "ccc");
        mark_row(wp, 0, GRID_LINE_WRAPPED);

        let mut split = item(c"capture-pane -b cap", 1, &mut t);
        assert_eq!(exec(&mut split), CMD_RETURN_NORMAL);
        assert_eq!(
            captured(),
            Some(b"aaa\nbbb\nccc\n".to_vec()),
            "without -J the wrapped line still ends its own capture line"
        );

        let mut joined = item(c"capture-pane -J -b cap", 2, &mut t);
        assert_eq!(exec(&mut joined), CMD_RETURN_NORMAL);
        assert_eq!(
            captured(),
            Some(b"aaabbb\nccc\n".to_vec()),
            "with -J the wrap is closed up and no separator is added"
        );
    }
}

#[test]
fn the_T_and_N_flags_control_padding_and_trimming_of_short_lines() {
    let _guard = globals();
    let _paste = Paste::new();
    let mut t = Target::new(10, 3);
    let wp = t.pane(0);
    unsafe {
        write_row(wp, 0, "abc");

        let mut plain = item(c"capture-pane -S 0 -E 0 -b cap", 1, &mut t);
        assert_eq!(exec(&mut plain), CMD_RETURN_NORMAL);
        assert_eq!(captured(), Some(b"abc\n".to_vec()));

        let mut keep = item(c"capture-pane -N -S 0 -E 0 -b cap", 2, &mut t);
        assert_eq!(exec(&mut keep), CMD_RETURN_NORMAL);
        assert_eq!(
            captured(),
            Some(b"abc  \n".to_vec()),
            "-N keeps the empty cells out to the end of the stored line"
        );

        let mut trail = item(c"capture-pane -T -S 0 -E 0 -b cap", 3, &mut t);
        assert_eq!(exec(&mut trail), CMD_RETURN_NORMAL);
        assert_eq!(
            captured(),
            Some(b"abc\n".to_vec()),
            "-T ends the line at the last written cell"
        );
    }
}

#[test]
fn the_e_flag_keeps_sequences_and_the_C_flag_escapes_backslashes() {
    let _guard = globals();
    let _paste = Paste::new();
    let mut t = Target::new(10, 3);
    let wp = t.pane(0);
    unsafe {
        write_row(wp, 0, "a\\b");

        let mut plain = item(c"capture-pane -S 0 -E 0 -b cap", 1, &mut t);
        assert_eq!(exec(&mut plain), CMD_RETURN_NORMAL);
        assert_eq!(captured(), Some(b"a\\b\n".to_vec()));

        let mut escaped = item(c"capture-pane -C -S 0 -E 0 -b cap", 2, &mut t);
        assert_eq!(exec(&mut escaped), CMD_RETURN_NORMAL);
        assert_eq!(
            captured(),
            Some(b"a\\\\b\n".to_vec()),
            "-C doubles the backslash on its way out"
        );

        let mut sequences = item(c"capture-pane -e -S 0 -E 0 -b cap", 3, &mut t);
        assert_eq!(exec(&mut sequences), CMD_RETURN_NORMAL);
        assert_eq!(
            captured(),
            Some(b"a\\b\n".to_vec()),
            "default-styled cells emit no sequences of their own"
        );
    }
}

#[test]
fn the_F_flag_prefixes_each_line_with_its_state_letters() {
    let _guard = globals();
    let _paste = Paste::new();
    let mut t = Target::new(10, 3);
    let wp = t.pane(0);
    unsafe {
        write_row(wp, 0, "r0");
        write_row(wp, 1, "r1");
        write_row(wp, 2, "r2");
        mark_row(wp, 0, GRID_LINE_DEAD | GRID_LINE_WRAPPED);
        mark_row(
            wp,
            1,
            GRID_LINE_HYPERLINK
                | GRID_LINE_START_OUTPUT
                | GRID_LINE_START_PROMPT
                | GRID_LINE_EXTENDED,
        );

        let mut it = item(c"capture-pane -F -b cap", 1, &mut t);
        assert_eq!(exec(&mut it), CMD_RETURN_NORMAL);
        assert_eq!(
            captured(),
            Some(b"DW r0\nHOPX r1\n- r2\n".to_vec()),
            "letters follow the DHOXPWX order, with - for a quiet line"
        );
    }
}

#[test]
fn capturing_hyperlinks_without_any_needs_nothing_from_the_store() {
    let _guard = globals();
    let _paste = Paste::new();
    let mut t = Target::new(10, 3);
    let wp = t.pane(0);
    unsafe {
        write_row(wp, 0, "aaa");

        let mut it = item(c"capture-pane -H -b cap", 1, &mut t);
        assert_eq!(exec(&mut it), CMD_RETURN_NORMAL);
        assert_eq!(
            captured(),
            None,
            "lines that carry no link ids contribute nothing at all, so no \
             buffer is made"
        );
    }
}

#[test]
fn the_H_flag_lists_each_new_uri_once_and_joins_those_on_one_line() {
    let _guard = globals();
    let _paste = Paste::new();
    let mut t = Target::new(10, 4);
    let wp = t.pane(0);
    unsafe {
        screen_reset_hyperlinks(&raw mut (*wp).base);
        let hl = (*wp).base.hyperlinks_ref().expect("a hyperlink store");
        let a = hyperlinks_put(hl, c"https://example.com/a", Some(c"id-a"));
        let b = hyperlinks_put(hl, c"https://example.com/b", Some(c"id-b"));
        let c = hyperlinks_put(hl, c"https://example.com/c", Some(c"id-c"));
        let d = hyperlinks_put(hl, c"https://example.com/d", Some(c"id-d"));

        write_linked_row(wp, 0, &[a, a, a]);
        write_linked_row(wp, 1, &[b, b, b]);
        write_linked_row(wp, 2, &[c, d]);
        write_linked_row(wp, 3, &[999]);

        let mut it = item(c"capture-pane -H -b cap", 1, &mut t);
        assert_eq!(exec(&mut it), CMD_RETURN_NORMAL);
        assert_eq!(
            captured(),
            Some(
                b"https://example.com/a\n\
                  https://example.com/b\n\
                  https://example.com/c https://example.com/d\n"
                    .to_vec()
            ),
            "a URI already listed on an earlier line is not repeated; several \
             new ones on one line are joined with spaces; a link id missing \
             from the store makes its whole line vanish, newline and all"
        );
    }
}

#[test]
fn the_P_flag_captures_pending_input_raw() {
    let _guard = globals();
    let _paste = Paste::new();
    let mut t = Target::new(10, 3);
    let wp = t.pane(0);
    unsafe {
        let mut pending = Pending::with_bytes(b"ab\x01\\z");
        pending.install(wp);

        let mut it = item(c"capture-pane -P -b cap", 1, &mut t);
        assert_eq!(exec(&mut it), CMD_RETURN_NORMAL);
        assert_eq!(
            captured(),
            Some(b"ab\x01\\z".to_vec()),
            "pending bytes come out exactly as they went in, without a newline"
        );
    }
}

#[test]
fn the_C_flag_escapes_pending_input_by_octet() {
    let _guard = globals();
    let _paste = Paste::new();
    let mut t = Target::new(10, 3);
    let wp = t.pane(0);
    unsafe {
        let mut pending = Pending::with_bytes(b"ab\x01\\z");
        pending.install(wp);

        let mut it = item(c"capture-pane -P -C -b cap", 1, &mut t);
        assert_eq!(exec(&mut it), CMD_RETURN_NORMAL);
        assert_eq!(
            captured(),
            Some(b"ab\\001\\134z".to_vec()),
            "printable bytes pass through; others become \\ooo escapes"
        );
    }
}

#[test]
fn empty_or_absent_pending_input_captures_nothing() {
    let _guard = globals();
    let _paste = Paste::new();
    let mut t = Target::new(10, 3);
    let wp = t.pane(0);
    unsafe {
        let mut empty = Pending::with_bytes(b"");
        empty.install(wp);
        let mut it = item(c"capture-pane -P -b cap", 1, &mut t);
        assert_eq!(exec(&mut it), CMD_RETURN_NORMAL);
        assert_eq!(captured(), None, "an empty queue makes an empty capture");

        let mut absent = Pending::without_buffer();
        absent.install(wp);
        let mut it = item(c"capture-pane -P -b cap", 2, &mut t);
        assert_eq!(exec(&mut it), CMD_RETURN_NORMAL);
        assert_eq!(captured(), None, "no pending queue captures nothing");
    }
}

#[test]
fn the_H_flag_overrides_P_and_sends_the_capture_down_the_history_path() {
    let _guard = globals();
    let _paste = Paste::new();
    let mut t = Target::new(10, 3);
    let wp = t.pane(0);
    unsafe {
        let mut pending = Pending::with_bytes(b"ZZZ");
        pending.install(wp);

        let mut it = item(c"capture-pane -P -b cap", 1, &mut t);
        assert_eq!(exec(&mut it), CMD_RETURN_NORMAL);
        assert_eq!(captured(), Some(b"ZZZ".to_vec()));

        write_row(wp, 0, "sss");
        let mut forced = item(c"capture-pane -P -H -b cap2", 2, &mut t);
        assert_eq!(exec(&mut forced), CMD_RETURN_NORMAL);
        let pb = paste_get_name(c"cap2".as_ptr());
        assert!(
            pb.is_null(),
            "with -H the pending queue is ignored; the history path finds \
             no hyperlinks and answers empty"
        );
    }
}

#[test]
fn printing_to_a_control_client_writes_one_line_per_capture() {
    let _guard = globals();
    let mut control = Control::new();
    let mut t = Target::new(10, 2);
    let wp = t.pane(0);
    unsafe {
        write_row(wp, 0, "ab");
        write_row(wp, 1, "cd");

        let mut plain = Item::with_client()
            .from_file(FILE, 1)
            .with_args(c"capture-pane -p")
            .targeting(&mut t);
        control.attach_to(&mut plain);
        assert_eq!(exec(&mut plain), CMD_RETURN_NORMAL);
        assert_eq!(
            control.written(),
            b"ab\ncd\n",
            "the capture's trailing newline is trimmed before writing"
        );

        mark_row(wp, 0, GRID_LINE_WRAPPED);
        let mut joined = Item::with_client()
            .from_file(FILE, 2)
            .with_args(c"capture-pane -J -p")
            .targeting(&mut t);
        control.attach_to(&mut joined);
        assert_eq!(exec(&mut joined), CMD_RETURN_NORMAL);
        assert_eq!(
            control.written(),
            b"abcd\n",
            "nothing is trimmed when the capture does not end in a newline, \
             and the control framing adds the break"
        );
    }
}

#[test]
fn printing_to_a_client_that_cannot_be_written_to_is_an_error() {
    let _guard = globals();
    let mut log = MessageLog::take();
    let mut t = Target::new(10, 3);
    let wp = t.pane(0);
    unsafe {
        write_row(wp, 0, "aaa");

        let mut it = Item::with_client()
            .from_file(FILE, 1)
            .with_args(c"capture-pane -p")
            .targeting(&mut t);
        (*it.client()).flags |= CLIENT_ATTACHED as uint64_t;
        assert_eq!(exec(&mut it), CMD_RETURN_ERROR, "can't write to client");
    }
    drop(log);
}

/// A turn at the server's message log, taken away for the length of a test so
/// that what the test records is all there is. Put back exactly as found.
struct MessageLog {
    saved: VecDeque<message_entry>,
}

impl MessageLog {
    fn take() -> MessageLog {
        MessageLog {
            saved: ::core::mem::take(crate::server::message_log.queue()),
        }
    }
}

impl Drop for MessageLog {
    fn drop(&mut self) {
        *crate::server::message_log.queue() = ::core::mem::take(&mut self.saved);
    }
}

#[test]
fn capturing_the_alternate_screen_reports_errors_and_honours_quiet() {
    let _guard = globals();
    let _paste = Paste::new();
    let mut t = Target::new(10, 3);
    unsafe {
        let mut loud = item(c"capture-pane -a", 1, &mut t);
        assert_eq!(
            exec(&mut loud),
            CMD_RETURN_ERROR,
            "no alternate screen has been saved"
        );

        let mut quiet = item(c"capture-pane -a -q -b cap", 2, &mut t);
        assert_eq!(exec(&mut quiet), CMD_RETURN_NORMAL);
        assert_eq!(captured(), None, "quietly answering an empty capture");
    }
}

#[test]
fn capturing_the_alternate_screen_reads_the_saved_grid() {
    let _guard = globals();
    let _paste = Paste::new();
    let mut t = Target::new(10, 3);
    let wp = t.pane(0);
    unsafe {
        write_row(wp, 0, "main");
        let mut saved = grid_create(10, 3, 100);
        for (i, byte) in b"XY".iter().enumerate() {
            let gc = ascii(*byte);
            grid_set_cell(&mut saved, i as u_int, 0, &gc);
        }
        (*wp).base.saved_grid = Some(saved);

        let mut it = item(c"capture-pane -a -b cap", 1, &mut t);
        assert_eq!(exec(&mut it), CMD_RETURN_NORMAL);
        assert_eq!(
            captured(),
            Some(b"XY\n\n\n".to_vec()),
            "the saved grid is captured, empty rows and all"
        );
    }
}

#[test]
fn the_M_flag_falls_back_to_the_base_screen_when_no_mode_has_one() {
    let _guard = globals();
    let _paste = Paste::new();
    let mut t = Target::new(10, 3);
    let wp = t.pane(0);
    unsafe {
        write_row(wp, 0, "mmm");

        let mut it = item(c"capture-pane -M -b cap", 1, &mut t);
        assert_eq!(exec(&mut it), CMD_RETURN_NORMAL);
        assert_eq!(
            captured(),
            Some(b"mmm\n\n\n".to_vec()),
            "the untouched rows come out empty"
        );
    }
}

#[test]
fn clear_history_empties_the_scrollback_and_H_resets_its_links() {
    let _guard = globals();
    let mut t = Target::new(10, 3);
    let wp = t.pane(0);
    unsafe {
        scroll(wp, 2);
        assert_eq!((*screen_grid_ptr(&mut (*wp).base)).hsize, 2);

        let hl = (*wp).base.hyperlinks_ref().expect("a hyperlink store");
        let inner = hyperlinks_put(hl, c"https://example.com/", Some(c"id"));
        let hl_ptr = hl.as_ptr();

        let mut clear = Item::new()
            .from_file(FILE, 1)
            .with_args(c"clear-history")
            .targeting(&mut t);
        assert_eq!(exec(&mut clear), CMD_RETURN_NORMAL);
        assert_eq!(
            (*screen_grid_ptr(&mut (*wp).base)).hsize,
            0,
            "the history is gone"
        );
        assert!(
            crate::grid::hyperlinks_get(&*hl_ptr, inner).is_some(),
            "without -H the stored links survive"
        );

        let mut with_h = Item::new()
            .from_file(FILE, 2)
            .with_args(c"clear-history -H")
            .targeting(&mut t);
        assert_eq!(exec(&mut with_h), CMD_RETURN_NORMAL);
        assert_eq!(
            (*wp).base.hyperlinks_ptr(),
            hl_ptr,
            "-H resets the screen's own store in place"
        );
        assert!(
            crate::grid::hyperlinks_get(&*hl_ptr, inner).is_none(),
            "and its links are gone afterwards"
        );
    }
}

/// A client's end of the connection to the process it prints through: a
/// zeroed peer whose message buffer sits on one side of a socket pair, with a
/// reading buffer on the other. `file_print_buffer` and `file_print` queue
/// their messages on it, and [`Peer::sent`] flushes them across and takes them
/// off again; nothing runs the event loop, so the read event the send arms
/// never fires and is taken down again on drop.
struct Peer {
    peer: Option<Box<tmuxpeer>>,
    reader: Box<imsgbuf>,
    fds: [c_int; 2],
}

impl Peer {
    fn new() -> Peer {
        ensure_reactor();
        let mut fds = [-1 as c_int; 2];
        unsafe {
            assert_eq!(
                ::libc::socketpair(::libc::AF_UNIX, ::libc::SOCK_STREAM, 0, fds.as_mut_ptr()),
                0,
                "no socket pair"
            );
        }
        let mut p = Peer {
            peer: Some(zeroed::<tmuxpeer>()),
            reader: zeroed::<imsgbuf>(),
            fds,
        };
        unsafe {
            assert_eq!(imsgbuf_init(&mut (*p.ptr()).ibuf, fds[0]), 0);
            assert_eq!(imsgbuf_init(&mut p.reader, fds[1]), 0);
        }
        p
    }

    fn ptr(&self) -> *mut tmuxpeer {
        peer_ptr(&self.peer)
    }

    /// Gives the item's client this connection to print down, which owns it
    /// until [`Peer::take_back`].
    fn attach_to(&mut self, item: &mut Item) {
        unsafe { (*item.client()).peer = self.peer.take() };
    }

    fn take_back(&mut self, item: &mut Item) {
        unsafe { self.peer = (*item.client()).peer.take() };
    }

    /// Sends everything queued and answers each message that came out, as its
    /// type and its payload.
    fn sent(&mut self) -> Vec<(msgtype, Vec<u8>)> {
        let mut out = Vec::new();
        unsafe {
            assert_eq!(imsgbuf_flush(&mut (*self.ptr()).ibuf), 0, "nothing sent");
            assert_eq!(imsgbuf_read(&mut self.reader), 1, "nothing arrived");
            loop {
                let mut msg = Box::new(imsg::default());
                match imsgbuf_get(&mut self.reader, &raw mut *msg) {
                    0 => break,
                    1 => {}
                    other => panic!("imsgbuf_get answered {other}"),
                }
                let len = imsg_get_len(&raw mut *msg);
                let data = if len == 0 {
                    Vec::new()
                } else {
                    slice::from_raw_parts(msg.data as *const u8, len).to_vec()
                };
                out.push((imsg_get_type(&raw mut *msg) as msgtype, data));
                imsg_free(&raw mut *msg);
            }
        }
        out
    }
}

impl Drop for Peer {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr().is_null() {
                (*self.ptr()).event.disable();
                imsgbuf_clear(&mut (*self.ptr()).ibuf);
            }
            imsgbuf_clear(&mut self.reader);
            ::libc::close(self.fds[0]);
            ::libc::close(self.fds[1]);
        }
    }
}

/// Frees the stdout stream the print path opened on a client, the way the
/// client's own teardown would, so nothing is left pointing at the test's
/// client once it goes.
unsafe fn close_files(c: *mut client) {
    unsafe {
        while let Some(cf) = (*c).files.values().next().cloned() {
            file_free(cf);
        }
    }
}

/// View mode opened on a pane, which is the mode `capture-pane -M` is for:
/// it keeps a backing screen of its own, separate from the pane's, and
/// [`window_copy_add`] writes the text a test wants captured into it.
struct Mode {
    wp: *mut window_pane,
}

impl Mode {
    fn open(wp: *mut window_pane, text: &CStr) -> Mode {
        unsafe {
            assert!((*wp).modes.is_empty(), "a mode is already open");
            assert_eq!(
                window_pane_set_mode(
                    wp,
                    ::core::ptr::null_mut::<window_pane>(),
                    WindowMode::View,
                    ::core::ptr::null_mut::<cmd_find_state>(),
                    None,
                ),
                0,
                "view mode did not open"
            );
            window_copy_add(wp, 0, text.as_ptr(), fmt_args![]);
        }
        Mode { wp }
    }
}

impl Drop for Mode {
    fn drop(&mut self) {
        unsafe { window_pane_reset_mode(self.wp) };
    }
}

#[test]
fn a_start_line_below_the_screen_clamps_to_the_last_visible_row() {
    let _guard = globals();
    let _paste = Paste::new();
    let mut t = Target::new(10, 3);
    let wp = t.pane(0);
    unsafe {
        write_row(wp, 0, "aaa");
        write_row(wp, 1, "bbb");
        write_row(wp, 2, "ccc");

        let mut it = item(c"capture-pane -S 100 -b cap", 1, &mut t);
        assert_eq!(exec(&mut it), CMD_RETURN_NORMAL);
        assert_eq!(
            captured(),
            Some(b"ccc\n".to_vec()),
            "a start past the bottom is pulled back to the last visible row"
        );
    }
}

#[test]
fn an_end_line_below_the_screen_clamps_to_the_last_visible_row() {
    let _guard = globals();
    let _paste = Paste::new();
    let mut t = Target::new(10, 3);
    let wp = t.pane(0);
    unsafe {
        write_row(wp, 0, "aaa");
        write_row(wp, 1, "bbb");
        write_row(wp, 2, "ccc");

        let mut it = item(c"capture-pane -S 1 -E 100 -b cap", 1, &mut t);
        assert_eq!(exec(&mut it), CMD_RETURN_NORMAL);
        assert_eq!(
            captured(),
            Some(b"bbb\nccc\n".to_vec()),
            "an end past the bottom is pulled back to the last visible row"
        );
    }
}

#[test]
fn an_E_dash_ends_at_the_last_visible_line() {
    let _guard = globals();
    let _paste = Paste::new();
    let mut t = Target::new(10, 3);
    let wp = t.pane(0);
    unsafe {
        scroll(wp, 2);
        write_row(wp, 0, "h0");
        write_row(wp, 1, "h1");
        write_row(wp, 2, "s0");
        write_row(wp, 3, "s1");
        write_row(wp, 4, "s2");

        let mut it = item(c"capture-pane -E - -b cap", 1, &mut t);
        assert_eq!(exec(&mut it), CMD_RETURN_NORMAL);
        assert_eq!(
            captured(),
            Some(b"s0\ns1\ns2\n".to_vec()),
            "the range runs from the top of the screen to the last visible row"
        );
    }
}

#[test]
fn the_M_flag_reads_the_screen_the_mode_hands_over() {
    let _guard = globals();
    let _paste = Paste::new();
    let mut t = Target::new(10, 3);
    let wp = t.pane(0);
    unsafe {
        write_row(wp, 0, "base");
        let _mode = Mode::open(wp, c"mode");

        let mut it = item(c"capture-pane -M -b cap", 1, &mut t);
        assert_eq!(exec(&mut it), CMD_RETURN_NORMAL);
        let text = String::from_utf8_lossy(&captured().expect("nothing was captured")).into_owned();
        assert!(
            text.contains("mode"),
            "the mode's own screen was not captured: {text:?}"
        );
        assert!(
            !text.contains("base"),
            "the pane's own screen was captured instead: {text:?}"
        );
    }
}

#[test]
fn the_H_flag_stops_collecting_at_a_screen_width_of_uris() {
    let _guard = globals();
    let _paste = Paste::new();
    let mut t = Target::new(10, 3);
    let wp = t.pane(0);
    unsafe {
        screen_reset_hyperlinks(&raw mut (*wp).base);
        let hl = (*wp).base.hyperlinks_ref().expect("a hyperlink store");
        let uris: Vec<CString> = (0..11)
            .map(|n| CString::new(format!("https://e/{n}")).expect("a uri"))
            .collect();
        let ids: Vec<CString> = (0..11)
            .map(|n| CString::new(format!("id{n}")).expect("an id"))
            .collect();
        let links: Vec<u_int> = (0..11)
            .map(|n| hyperlinks_put(hl, &uris[n], Some(&ids[n])))
            .collect();

        write_linked_row(wp, 0, &links[..10]);
        write_linked_row(wp, 1, &links[10..]);

        let mut it = item(c"capture-pane -H -b cap", 1, &mut t);
        assert_eq!(exec(&mut it), CMD_RETURN_NORMAL);
        assert_eq!(
            captured(),
            Some(
                b"https://e/0 https://e/1 https://e/2 https://e/3 https://e/4 \
                  https://e/5 https://e/6 https://e/7 https://e/8 https://e/9\n"
                    .to_vec()
            ),
            "the list holds one screen width of link ids, and the line \
             carrying the next one stops there and vanishes with it"
        );
    }
}

#[test]
fn printing_to_a_client_that_can_print_sends_the_capture_down_its_peer() {
    let _guard = globals();
    let mut peer = Peer::new();
    let mut t = Target::new(10, 2);
    let wp = t.pane(0);
    unsafe {
        write_row(wp, 0, "ab");
        write_row(wp, 1, "cd");

        let mut it = Item::with_client()
            .from_file(FILE, 1)
            .with_args(c"capture-pane -p")
            .targeting(&mut t);
        peer.attach_to(&mut it);
        assert_eq!(exec(&mut it), CMD_RETURN_NORMAL);

        peer.take_back(&mut it);
        let sent = peer.sent();
        assert_eq!(sent.len(), 2, "the stream is opened, then written");
        assert_eq!(sent[0].0, MSG_WRITE_OPEN);
        assert_eq!(sent[1].0, MSG_WRITE);
        let head = ::core::mem::size_of::<msg_write_data>();
        assert_eq!(
            &sent[1].1[head..],
            b"ab\ncd\n",
            "the capture goes over with its trailing newline trimmed and the \
             one file_print adds put back"
        );
        close_files(it.client());
    }
}

#[test]
fn an_empty_buffer_name_is_refused() {
    let _guard = globals();
    let _paste = Paste::new();
    let mut t = Target::new(10, 3);
    let wp = t.pane(0);
    unsafe {
        write_row(wp, 0, "aaa");

        let mut it = item(c"capture-pane -b ''", 1, &mut t);
        assert_eq!(
            exec(&mut it),
            CMD_RETURN_ERROR,
            "the paste store turns an empty buffer name down"
        );
        assert!(
            paste_walk(::core::ptr::null_mut()).is_null(),
            "and nothing was stored"
        );
    }
}

#[test]
fn the_lowest_start_line_there_is_starts_at_the_first_history_line() {
    let _guard = globals();
    let _paste = Paste::new();
    let mut t = Target::new(10, 3);
    let wp = t.pane(0);
    unsafe {
        scroll(wp, 1);
        write_row(wp, 0, "h0");
        write_row(wp, 1, "s0");
        write_row(wp, 2, "s1");
        write_row(wp, 3, "s2");

        let mut it = item(c"capture-pane -S -2147483648 -b cap", 1, &mut t);
        assert_eq!(exec(&mut it), CMD_RETURN_NORMAL);
        assert_eq!(
            captured(),
            Some(b"h0\ns0\ns1\ns2\n".to_vec()),
            "the lowest value the flag accepts negates to itself, which is \
             further back than any history"
        );
    }
}

#[test]
fn a_capture_that_does_not_end_in_a_newline_is_printed_whole() {
    let _guard = globals();
    let mut control = Control::new();
    let mut t = Target::new(10, 2);
    let wp = t.pane(0);
    unsafe {
        write_row(wp, 0, "ab");
        write_row(wp, 1, "cd");
        mark_row(wp, 0, GRID_LINE_WRAPPED);
        mark_row(wp, 1, GRID_LINE_WRAPPED);

        let mut it = Item::with_client()
            .from_file(FILE, 1)
            .with_args(c"capture-pane -J -p")
            .targeting(&mut t);
        control.attach_to(&mut it);
        assert_eq!(exec(&mut it), CMD_RETURN_NORMAL);
        assert_eq!(
            control.written(),
            b"abcd\n",
            "a last line that is itself wrapped closes nothing, so there is \
             no trailing newline to take off"
        );
    }
}
