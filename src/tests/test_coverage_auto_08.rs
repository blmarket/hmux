//! Coverage for [`crate::tty`] and [`crate::terminfo`] — constants
//! and lightweight helpers reachable from the [`Tty`] fixture.
//!
//! `tty.rs` at 14.39% and `tty_term.rs` at 28% are dominated by terminfo
//! and ensure_reactor paths that want a live descriptor. The constants block at
//! the top of each file, `tty_set_size`, the `tty_term_*` accessors over a
//! zeroed `Tty`, `tty_term_ncodes`/`tty_term_describe` and `tty_fake_bce`
//! are the deterministic surface a unit test can reach without touching a
//! terminal, spawning a child or hitting `fatal`.

use crate::grid::grid_default_cell;
use crate::tests::test_fixtures::{Tty, globals, seen};
use crate::tty::{
    CLIENT_REDRAWSTATUS, CLIENT_REDRAWWINDOW, CLIENT_TERMINAL, MODE_CURSOR, MODE_MOUSE_ALL,
    MODE_MOUSE_BUTTON, MODE_MOUSE_STANDARD, TERM_DECFRA, TERM_DECSLRM, TERM_NOAM, TERM_RGBCOLOURS,
    TERM_VT100LIKE, TTY_BLOCK, TTY_NOCURSOR, TTY_STARTED, TTYC_ACSC, TTYC_BCE, TTYC_CLEAR,
    TTYC_CUP, TTYC_KMOUS, TTYC_XT, tty_fake_bce, tty_set_size,
};
use crate::terminfo::{
    TTYC_AM, TTYC_AX, TTYC_COLORS, TTYC_CSR, TTYCODE_FLAG, TTYCODE_NONE, TTYCODE_NUMBER,
    TTYCODE_STRING, TtyCode, tty_term_describe, tty_term_flag, tty_term_has, tty_term_ncodes,
    tty_term_number, tty_term_string, tty_term_string_i, tty_term_string_ii,
};
use crate::types::{tty, u_int};

// ---------------------------------------------------------------------------
// Constants — stable values copied from the C headers
// ---------------------------------------------------------------------------

#[test]
fn tty_and_tty_term_constants_keep_their_values() {
    assert_eq!(TTYC_ACSC, 0);
    assert_eq!(TTYC_BCE, 3);
    assert_eq!(TTYC_CLEAR, 9);
    assert_eq!(TTYC_CUP, 23);
    assert_eq!(TTYC_KMOUS, 165);
    assert_eq!(TTYC_XT, 232);

    assert_eq!(TTYC_AM, 1);
    assert_eq!(TTYC_AX, 2);
    assert_eq!(TTYC_COLORS, 13);
    assert_eq!(TTYC_CSR, 16);

    assert_eq!(TTYCODE_NONE, 0);
    assert_eq!(TTYCODE_STRING, 1);
    assert_eq!(TTYCODE_NUMBER, 2);
    assert_eq!(TTYCODE_FLAG, 3);

    assert_eq!(TERM_NOAM, 0x2);
    assert_eq!(TERM_DECSLRM, 0x4);
    assert_eq!(TERM_DECFRA, 0x8);
    assert_eq!(TERM_RGBCOLOURS, 0x10);
    assert_eq!(TERM_VT100LIKE, 0x20);

    assert_eq!(TTY_NOCURSOR, 0x1);
    assert_eq!(TTY_STARTED, 0x10);
    assert_eq!(TTY_BLOCK, 0x80);

    assert_eq!(MODE_CURSOR, 0x1);
    assert_eq!(MODE_MOUSE_STANDARD, 0x20);
    assert_eq!(MODE_MOUSE_BUTTON, 0x40);
    assert_eq!(MODE_MOUSE_ALL, 0x1000);

    assert_eq!(CLIENT_TERMINAL, 0x1);
    assert_eq!(CLIENT_REDRAWWINDOW, 0x8);
    assert_eq!(CLIENT_REDRAWSTATUS, 0x10);
}

#[test]
fn tty_term_ncodes_covers_the_full_table() {
    {
        assert_eq!(tty_term_ncodes(), 233);
    }
}

// ---------------------------------------------------------------------------
// tty_set_size — pure field store, no descriptor
// ---------------------------------------------------------------------------

#[test]
fn tty_set_size_stores_dimensions() {
    let _guard = globals();
    let mut t = Tty::new();
    unsafe {
        tty_set_size(t.ptr(), 80, 24, 10, 20);
        assert_eq!((*t.ptr()).sx, 80);
        assert_eq!((*t.ptr()).sy, 24);
        assert_eq!((*t.ptr()).xpixel, 10);
        assert_eq!((*t.ptr()).ypixel, 20);

        tty_set_size(t.ptr(), 132, 50, 0, 0);
        assert_eq!((*t.ptr()).sx, 132);
        assert_eq!((*t.ptr()).sy, 50);
        assert_eq!((*t.ptr()).xpixel, 0);
        assert_eq!((*t.ptr()).ypixel, 0);
    }
}

// ---------------------------------------------------------------------------
// tty_term accessors over the Tty fixture — zeroed codes start as missing
// ---------------------------------------------------------------------------

#[test]
fn tty_term_has_and_string_on_missing_code() {
    let _guard = globals();
    let mut t = Tty::new();
    unsafe {
        // every slot starts as TTYCODE_NONE
        assert_eq!(tty_term_has(t.term(), TTYC_CLEAR), 0);
        assert_eq!(tty_term_has(t.term(), TTYC_CUP), 0);
        assert_eq!(tty_term_has(t.term(), TTYC_KMOUS), 0);
        // missing string returns an empty C string
        let s = tty_term_string(t.term(), TTYC_CLEAR);
        assert_eq!(seen(s), "");
        // parametric wrappers also return empty when the capability is absent
        let si = tty_term_string_i(t.term(), TTYC_CUP, 1);
        assert_eq!(si.to_bytes(), b"");
        let sii = tty_term_string_ii(t.term(), TTYC_CSR, 0, 23);
        assert_eq!(sii.to_bytes(), b"");
        // number/flag on missing returns 0 without fatal
        assert_eq!(tty_term_number(t.term(), TTYC_COLORS), 0);
        assert_eq!(tty_term_flag(t.term(), TTYC_AM), 0);
    }
}

#[test]
fn tty_term_number_and_flag_roundtrip_via_tyy_fixture() {
    let _guard = globals();
    let mut t = Tty::new();
    // give the fixture a number and a flag
    t.set_number(TTYC_COLORS, 256);
    assert_eq!(tty_term_has(t.term(), TTYC_COLORS), 1);
    assert_eq!(tty_term_number(t.term(), TTYC_COLORS), 256);

    // flag: AM is a boolean capability — set via raw slot to keep test deterministic
    // Tty::set_number is for numbers; for flags write the slot directly
    t.set_flag(TTYC_AM, 1);
    assert_eq!(tty_term_has(t.term(), TTYC_AM), 1);
    assert_eq!(tty_term_flag(t.term(), TTYC_AM), 1);

    // a second flag set to 0 still counts as present but answers 0
    t.set_flag(TTYC_AX, 0);
    assert_eq!(tty_term_has(t.term(), TTYC_AX), 1);
    assert_eq!(tty_term_flag(t.term(), TTYC_AX), 0);
}

#[test]
fn tty_term_string_present_and_missing_after_mutation() {
    let _guard = globals();
    let mut t = Tty::new();
    unsafe {
        t.set_string(TTYC_CLEAR, c"hello");
        assert_eq!(tty_term_has(t.term(), TTYC_CLEAR), 1);
        assert_eq!(seen(tty_term_string(t.term(), TTYC_CLEAR)), "hello");
        // a different code is still missing
        assert_eq!(tty_term_has(t.term(), TTYC_CUP), 0);
        assert_eq!(seen(tty_term_string(t.term(), TTYC_CUP)), "");
    }
}

#[test]
fn tty_term_describe_mentions_missing_and_typed_values() {
    let _guard = globals();
    let mut t = Tty::new();
    unsafe {
        let missing = tty_term_describe(t.term(), TTYC_CLEAR as u_int)
            .to_string_lossy()
            .into_owned();
        assert!(missing.contains("[missing]"), "got {missing:?}");
        assert!(missing.contains("clear"));

        t.set_number(TTYC_COLORS, 256);
        let number = tty_term_describe(t.term(), TTYC_COLORS as u_int)
            .to_string_lossy()
            .into_owned();
        assert!(number.contains("(number)"), "got {number:?}");
        assert!(number.contains("256"));

        t.set_flag(TTYC_AM, 1);
        let flag = tty_term_describe(t.term(), TTYC_AM as u_int)
            .to_string_lossy()
            .into_owned();
        assert!(flag.contains("(flag)"), "got {flag:?}");
        assert!(flag.contains("true"));

        t.set_string(TTYC_CUP, c"foo");
        let string = tty_term_describe(t.term(), TTYC_CUP as u_int)
            .to_string_lossy()
            .into_owned();
        assert!(string.contains("(string)"), "got {string:?}");
    }
}

// ---------------------------------------------------------------------------
// tty_fake_bce — BCE fast-path vs fake-BCE fallback
// ---------------------------------------------------------------------------

#[test]
fn tty_fake_bce_with_and_without_bce_capability() {
    let _guard = globals();
    let mut t = Tty::new();
    let mut gc = unsafe { grid_default_cell };
    unsafe {
        // plain bg/fg (0) -> fake BCE needed when no BCE flag
        gc.bg = 0;
        gc.fg = 0;
        let bg: u_int = 0;
        // no BCE advertised -> must fake
        assert_eq!(tty_fake_bce(t.ptr() as *const tty, &raw const gc, bg), 1);

        // advertise BCE
        t.set_flag(TTYC_BCE, 1);
        assert_eq!(tty_fake_bce(t.ptr() as *const tty, &raw const gc, bg), 0);

        // even without BCE, default colours (bg 8/9) do not need faking
        // reset to no-BCE
        t.clear_code(TTYC_BCE);
        gc.bg = 8;
        let bg2: u_int = 8;
        assert_eq!(tty_fake_bce(t.ptr() as *const tty, &raw const gc, bg2), 0);

        gc.bg = 9;
        let bg3: u_int = 9;
        assert_eq!(tty_fake_bce(t.ptr() as *const tty, &raw const gc, bg3), 0);

        // mismatched: gc default but bg not default -> still fake
        gc.bg = 8;
        let bg4: u_int = 0;
        assert_eq!(tty_fake_bce(t.ptr() as *const tty, &raw const gc, bg4), 1);
    }
}

#[test]
fn tyy_fixture_starts_zeroed_and_client_flags_are_settable() {
    let _guard = globals();
    let mut t = Tty::new();
    unsafe {
        assert_eq!((*t.ptr()).sx, 0);
        assert_eq!((*t.ptr()).sy, 0);
        let codes: &[TtyCode] = &t.term().codes;
        assert_eq!(codes.len(), tty_term_ncodes() as usize);
        // all 233 slots start as NONE
        for i in 0..tty_term_ncodes() {
            assert_eq!(tty_term_has(t.term(), i as u_int), 0);
        }
        t.set_client_flags(CLIENT_TERMINAL as u64);
        // acs table starts empty
        assert_eq!(t.term().acs[0][0], 0);
        t.set_acs(b'a', "X");
        assert_eq!(t.term().acs[b'a' as usize][0] as u8, b'X');
    }
    let mut t2 = Tty::new();
    unsafe {
        assert_eq!(tty_term_has(t2.term(), TTYC_XT), 0);
    }
}
