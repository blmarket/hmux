//! Unit tests for [`crate::tty`]: the tree of terminal escape
//! sequences [`tty_keys_build`] assembles from tmux's built-in tables, the
//! capabilities a term reports and the server's `user-keys` option, plus the
//! OSC 10/11 colour-response parser [`tty_keys_colours`] and the module's
//! constant metadata.
//!
//! [`tty_keys_next`] and the mouse, clipboard, device-attributes and palette
//! helpers behind it consume a live client's input evbuffer and hand keys on
//! through the event loop, so they want a connected client and stay
//! unexercised here; so do the private add/find walkers, which build drives
//! on their behalf. Every sequence below comes straight from a byte slice in
//! memory: nothing reads or writes a descriptor, no timer runs and no event
//! loop turns.
//!
//! The tree is owned by `tty` as an [`Option<Box<tty_key>>`]. Tests borrow the
//! root while inspecting it, and [`tty_keys_free`] takes the root when a
//! fixture is done with it.

use crate::options::options_array_set;
use crate::options::options_get_ptr;
use crate::style::COLOUR_FLAG_RGB;
use crate::terminfo::{TTYC_KF1, TtyCode};
use crate::tests::test_fixtures::{Tty, globals};
use crate::tmux::global_options;
use crate::tty::{
    KEYC_CTRL, KEYC_CURSOR, KEYC_DC, KEYC_DOUBLECLICK_PANE, KEYC_DOWN, KEYC_F1, KEYC_F5,
    KEYC_IMPLIED_META, KEYC_KEYPAD, KEYC_KP_ZERO, KEYC_LEFT, KEYC_MASK_KEY, KEYC_META,
    KEYC_PASTE_END, KEYC_PASTE_START, KEYC_RIGHT, KEYC_SECONDCLICK_PANE, KEYC_SHIFT,
    KEYC_TRIPLECLICK_PANE, KEYC_TYPE_DOUBLECLICK, KEYC_TYPE_FUNCTION, KEYC_TYPE_MOUSEDOWN,
    KEYC_TYPE_MOUSEDRAG, KEYC_TYPE_MOUSEDRAGEND, KEYC_TYPE_MOUSEMOVE, KEYC_TYPE_MOUSEUP,
    KEYC_TYPE_NOTYPE, KEYC_TYPE_SECONDCLICK, KEYC_TYPE_TRIPLECLICK, KEYC_TYPE_UNICODE,
    KEYC_TYPE_USER, KEYC_TYPE_WHEELDOWN, KEYC_TYPE_WHEELUP, KEYC_UNKNOWN, KEYC_UP, KEYC_USER,
    TTY_BRACKETPASTE, TTY_WAITBG, TTY_WAITFG, TTY_WINSIZEQUERY, tty_key, tty_keys_build,
    tty_keys_colours, tty_keys_free,
};
use crate::types::{key_code, size_t};
use ::core::ffi::c_int;
use ::std::ffi::CString;

/// The key a stored binding answers with, walking the tree the way the
/// module's own finder does over plain bytes. [`KEYC_UNKNOWN`] when nothing
/// covers `seq`.
fn lookup(mut tk: Option<&tty_key>, seq: &[u8]) -> key_code {
    let mut i = 0usize;
    while let Some(node) = tk {
        if node.ch as u8 == seq[i] {
            i += 1;
            if i == seq.len() {
                return node.key;
            }
            tk = node.next.as_deref();
        } else if (seq[i] as c_int) < (node.ch as u8) as c_int {
            tk = node.left.as_deref();
        } else {
            tk = node.right.as_deref();
        }
    }
    KEYC_UNKNOWN
}

/// How many nodes the tree hangs together, counting each branch.
fn count_nodes(tk: Option<&tty_key>) -> usize {
    let Some(tk) = tk else {
        return 0;
    };
    1 + count_nodes(tk.next.as_deref())
        + count_nodes(tk.left.as_deref())
        + count_nodes(tk.right.as_deref())
}

/// Reports capability `code` as carrying the string `s`, the way a terminfo
/// entry would. Only ever given a key capability a string: a number there is
/// answered by the module under test with a fatal error.
unsafe fn bind_capability(t: &mut Tty, code: usize, s: &CString) {
    unsafe {
        t.term_mut().codes[code] = TtyCode::String(s.clone());
    }
}

/// Runs one colour response through the parser against `t`, answering what it
/// said and how much of the input it consumed.
unsafe fn parse_osc(t: &mut Tty, seq: &[u8], fg: &mut c_int, bg: &mut c_int) -> (c_int, usize) {
    unsafe {
        let mut size: size_t = 0;
        let rc = tty_keys_colours(t.ptr(), seq, &mut size, fg, bg);
        (rc, size as usize)
    }
}

#[test]
fn the_key_types_are_numbered_in_decode_order() {
    assert_eq!(KEYC_TYPE_UNICODE, 0);
    assert_eq!(KEYC_TYPE_USER, 1);
    assert_eq!(KEYC_TYPE_FUNCTION, 2);
    assert_eq!(KEYC_TYPE_MOUSEMOVE, 3);
    assert_eq!(KEYC_TYPE_MOUSEDOWN, 4);
    assert_eq!(KEYC_TYPE_MOUSEUP, 5);
    assert_eq!(KEYC_TYPE_MOUSEDRAG, 6);
    assert_eq!(KEYC_TYPE_MOUSEDRAGEND, 7);
    assert_eq!(KEYC_TYPE_WHEELDOWN, 8);
    assert_eq!(KEYC_TYPE_WHEELUP, 9);
    assert_eq!(KEYC_TYPE_SECONDCLICK, 10);
    assert_eq!(KEYC_TYPE_DOUBLECLICK, 11);
    assert_eq!(KEYC_TYPE_TRIPLECLICK, 12);
    assert_eq!(KEYC_TYPE_NOTYPE, 13);
    let ordered = [
        KEYC_TYPE_UNICODE,
        KEYC_TYPE_USER,
        KEYC_TYPE_FUNCTION,
        KEYC_TYPE_MOUSEMOVE,
        KEYC_TYPE_MOUSEDOWN,
        KEYC_TYPE_MOUSEUP,
        KEYC_TYPE_MOUSEDRAG,
        KEYC_TYPE_MOUSEDRAGEND,
        KEYC_TYPE_WHEELDOWN,
        KEYC_TYPE_WHEELUP,
        KEYC_TYPE_SECONDCLICK,
        KEYC_TYPE_DOUBLECLICK,
        KEYC_TYPE_TRIPLECLICK,
        KEYC_TYPE_NOTYPE,
    ];
    for pair in ordered.windows(2) {
        assert!(pair[0] < pair[1]);
    }
}

#[test]
fn the_click_families_step_one_user_key_apart_and_share_their_key_bits() {
    assert_eq!(KEYC_USER, 0x1_0000_0000);
    assert_eq!(KEYC_SECONDCLICK_PANE, 10 * KEYC_USER);
    assert_eq!(KEYC_DOUBLECLICK_PANE, 11 * KEYC_USER);
    assert_eq!(KEYC_TRIPLECLICK_PANE, 12 * KEYC_USER);
    assert_eq!(KEYC_DOUBLECLICK_PANE - KEYC_SECONDCLICK_PANE, KEYC_USER);
    assert_eq!(KEYC_TRIPLECLICK_PANE - KEYC_DOUBLECLICK_PANE, KEYC_USER);
    let pane_key = KEYC_SECONDCLICK_PANE & 0xffff_ffff;
    assert_eq!(pane_key, 0);
    assert_eq!(KEYC_DOUBLECLICK_PANE & 0xffff_ffff, pane_key);
    assert_eq!(KEYC_TRIPLECLICK_PANE & 0xffff_ffff, pane_key);
    assert_eq!(KEYC_UNKNOWN, 2 * KEYC_USER + 1);
    assert_eq!(KEYC_PASTE_START, KEYC_UNKNOWN + 4);
    assert_eq!(KEYC_PASTE_END, KEYC_PASTE_START + 1);
}

#[test]
fn the_modifier_and_tty_flags_pack_distinct_bits() {
    let flags = [
        KEYC_SHIFT,
        KEYC_META,
        KEYC_CTRL,
        KEYC_IMPLIED_META,
        KEYC_KEYPAD,
        KEYC_CURSOR,
    ];
    for flag in flags {
        assert_eq!(flag & KEYC_MASK_KEY, 0);
    }
    for (i, a) in flags.iter().enumerate() {
        for b in &flags[i + 1..] {
            assert_eq!(a & b, 0, "{a:#x} overlaps {b:#x}");
        }
    }
    assert_eq!(
        TTY_WAITFG | TTY_WAITBG | TTY_BRACKETPASTE | TTY_WINSIZEQUERY,
        0xf000
    );
    assert_eq!(TTY_WAITFG & TTY_WAITBG, 0);
    assert_eq!(TTY_BRACKETPASTE & TTY_WINSIZEQUERY, 0);
}

#[test]
fn building_binds_the_builtin_sequences_and_a_rebuild_leaves_the_same_tree() {
    let _guard = globals();
    let mut t = Tty::new();
    unsafe {
        assert!((*t.ptr()).key_tree.is_none());
        tty_keys_build(t.ptr());
        let tree = (*t.ptr()).key_tree.as_deref();
        assert!(tree.is_some());

        assert_eq!(lookup(tree, b"\x1bOA"), (KEYC_UP | KEYC_CURSOR));
        assert_eq!(lookup(tree, b"\x1b[B"), (KEYC_DOWN | KEYC_CURSOR));
        assert_eq!(lookup(tree, b"\x1b[D"), (KEYC_LEFT | KEYC_CURSOR));
        assert_eq!(lookup(tree, b"\x1b[C"), (KEYC_RIGHT | KEYC_CURSOR));
        assert_eq!(lookup(tree, b"\x1b[15~"), KEYC_F5);
        assert_eq!(lookup(tree, b"\x1bOp"), (KEYC_KP_ZERO | KEYC_KEYPAD));
        assert_eq!(
            lookup(tree, b"\x1b[201~"),
            (KEYC_PASTE_END | KEYC_IMPLIED_META)
        );

        assert_eq!(lookup(tree, b"\x1b[1;5A"), (KEYC_UP | KEYC_CTRL));
        assert_eq!(
            lookup(tree, b"\x1b[3;3~"),
            (KEYC_DC | KEYC_META | KEYC_IMPLIED_META)
        );

        assert_eq!(lookup(tree, b"\x1b[999999~"), KEYC_UNKNOWN);
        assert_eq!(lookup(tree, b"\x1bz"), KEYC_UNKNOWN);

        let nodes = count_nodes(tree);
        assert!(nodes > 300, "only {nodes} nodes built");

        tty_keys_build(t.ptr());
        let rebuilt = (*t.ptr()).key_tree.as_deref();
        assert!(rebuilt.is_some());
        assert_eq!(count_nodes(rebuilt), nodes);
        assert_eq!(lookup(rebuilt, b"\x1bOA"), (KEYC_UP | KEYC_CURSOR));
        assert_eq!(lookup(rebuilt, b"\x1b[15~"), KEYC_F5);

        tty_keys_free(t.ptr());
    }
}

#[test]
fn a_capability_the_term_reports_is_bound_and_nothing_without_one() {
    let _guard = globals();
    let mut t = Tty::new();
    let kf1 = CString::new(b"\x1b[?u".as_slice()).expect("no NUL");
    unsafe {
        tty_keys_build(t.ptr());
        let bare = (*t.ptr()).key_tree.as_deref();
        assert_eq!(lookup(bare, kf1.as_bytes()), KEYC_UNKNOWN);
        assert_eq!(lookup(bare, b"\x1bOA"), (KEYC_UP | KEYC_CURSOR));

        bind_capability(&mut t, TTYC_KF1 as usize, &kf1);
        tty_keys_build(t.ptr());
        let bound = (*t.ptr()).key_tree.as_deref();
        assert_eq!(lookup(bound, kf1.as_bytes()), KEYC_F1);
        assert_eq!(lookup(bound, b"\x1bOA"), (KEYC_UP | KEYC_CURSOR));

        tty_keys_free(t.ptr());
    }
}

#[test]
fn the_user_keys_option_extends_the_default_tree_by_index() {
    let _guard = globals();
    let mut t = Tty::new();
    let custom = CString::new(b"\x1b]u;fixture\x07".as_slice()).expect("no NUL");
    unsafe {
        let o = options_get_ptr(global_options, c"user-keys".as_ptr());
        assert!(
            !o.is_null(),
            "user-keys is not defaulted into global options"
        );

        tty_keys_build(t.ptr());
        let bare = (*t.ptr()).key_tree.as_deref();
        assert_eq!(lookup(bare, custom.as_bytes()), KEYC_UNKNOWN);

        assert_eq!(options_array_set(o, 0, custom.as_ptr(), 0, &mut None), 0);
        tty_keys_build(t.ptr());
        let extended = (*t.ptr()).key_tree.as_deref();
        assert_eq!(lookup(extended, custom.as_bytes()), KEYC_USER);
        assert_eq!(lookup(extended, b"\x1bOA"), (KEYC_UP | KEYC_CURSOR));

        assert_eq!(
            options_array_set(o, 0, ::core::ptr::null(), 0, &mut None),
            0
        );
        tty_keys_free(t.ptr());
    }
}

#[test]
fn an_osc_background_reply_sets_bg_and_consumes_itself() {
    let _guard = globals();
    let mut t = Tty::new();
    unsafe {
        (*t.ptr()).flags |= TTY_WAITBG;
        let seq = b"\x1b]11;rgb:0000/0000/0000\x07";
        let mut fg = -42;
        let mut bg = -42;
        let (rc, size) = parse_osc(&mut t, seq, &mut fg, &mut bg);
        assert_eq!(rc, 0);
        assert_eq!(size, seq.len());
        assert_eq!(bg, COLOUR_FLAG_RGB);
        assert_eq!(fg, -42);
        assert_eq!((*t.ptr()).flags & TTY_WAITBG, 0);
    }
}

#[test]
fn an_osc_foreground_reply_sets_fg_and_clears_its_wait_flag() {
    let _guard = globals();
    let mut t = Tty::new();
    unsafe {
        (*t.ptr()).flags |= TTY_WAITFG;
        let seq = b"\x1b]10;rgb:ffff/ffff/ffff\x1b\\";
        let mut fg = -42;
        let mut bg = -42;
        let (rc, size) = parse_osc(&mut t, seq, &mut fg, &mut bg);
        assert_eq!(rc, 0);
        assert_eq!(size, seq.len());
        assert_eq!(fg, COLOUR_FLAG_RGB | 0xffffff);
        assert_eq!(bg, -42);
        assert_eq!((*t.ptr()).flags & TTY_WAITFG, 0);
    }
}

#[test]
fn truncated_colour_replies_ask_for_more_input() {
    let _guard = globals();
    let mut t = Tty::new();
    for seq in [
        b"\x1b".as_slice(),
        b"\x1b]".as_slice(),
        b"\x1b]1".as_slice(),
        b"\x1b]11".as_slice(),
        b"\x1b]11;".as_slice(),
        b"\x1b]10;rgb:00".as_slice(),
    ] {
        let mut fg = -42;
        let mut bg = -42;
        let (rc, size) = unsafe { parse_osc(&mut t, seq, &mut fg, &mut bg) };
        assert_eq!(rc, 1, "{seq:?} should want more input");
        assert_eq!(size, 0);
        assert_eq!(fg, -42);
        assert_eq!(bg, -42);
    }
}

#[test]
fn malformed_introducers_are_rejected_out_of_hand() {
    let _guard = globals();
    let mut t = Tty::new();
    for seq in [
        b"\x1bX".as_slice(),
        b"\x1b[10;".as_slice(),
        b"\x1b]9;".as_slice(),
        b"\x1b]12;rgb:00/00/00".as_slice(),
        b"\x1b]10Xrgb:00/00/00\x07".as_slice(),
    ] {
        let mut fg = -42;
        let mut bg = -42;
        let (rc, size) = unsafe { parse_osc(&mut t, seq, &mut fg, &mut bg) };
        assert_eq!(rc, -1, "{seq:?} should be rejected");
        assert_eq!(size, 0);
        assert_eq!(fg, -42);
        assert_eq!(bg, -42);
    }
}

#[test]
fn an_empty_or_unparsable_payload_leaves_the_colours_alone() {
    let _guard = globals();
    let mut t = Tty::new();
    unsafe {
        let empty = b"\x1b]10;\x07";
        let mut fg = -42;
        let mut bg = -42;
        let (rc, size) = parse_osc(&mut t, empty, &mut fg, &mut bg);
        assert_eq!(rc, 0);
        assert_eq!(size, empty.len());
        assert_eq!(fg, -42);
        assert_eq!(bg, -42);

        let junk = b"\x1b]11;nonsense\x07";
        let (rc, size) = parse_osc(&mut t, junk, &mut fg, &mut bg);
        assert_eq!(rc, 0);
        assert_eq!(size, junk.len());
        assert_eq!(fg, -42);
        assert_eq!(bg, -42);
    }
}

#[test]
fn an_oversize_payload_is_rejected_rather_than_parsed() {
    let _guard = globals();
    let mut t = Tty::new();
    let mut seq = Vec::from(&b"\x1b]10;"[..]);
    seq.extend(::std::iter::repeat_n(b'a', 200));
    let mut fg = -42;
    let mut bg = -42;
    let (rc, size) = unsafe { parse_osc(&mut t, &seq, &mut fg, &mut bg) };
    assert_eq!(rc, -1);
    assert_eq!(size, 0);
    assert_eq!(fg, -42);
    assert_eq!(bg, -42);
}
