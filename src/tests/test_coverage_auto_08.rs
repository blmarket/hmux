//! Coverage for [`crate::tty`] and [`crate::terminfo`] — constants
//! and lightweight helpers reachable from the [`Tty`] fixture.
//!
//! `tty.rs` at 14.39% and `tty_term.rs` at 28% are dominated by terminfo
//! and ensure_reactor paths that want a live descriptor. The constants block at
//! the top of each file, `tty_set_size` and `tty_fake_bce` are the
//! deterministic surface a unit test can reach without touching a
//! terminal, spawning a child or hitting `fatal`.

use crate::grid::grid_default_cell;
use crate::terminfo::TerminalCapabilities;
use crate::tests::test_fixtures::{Tty, globals};
use crate::tty::{
    CLIENT_REDRAWSTATUS, CLIENT_REDRAWWINDOW, CLIENT_TERMINAL, MODE_CURSOR, MODE_MOUSE_ALL,
    MODE_MOUSE_BUTTON, MODE_MOUSE_STANDARD, TERM_DECFRA, TERM_DECSLRM, TERM_NOAM, TERM_RGBCOLOURS,
    TERM_VT100LIKE, TTY_BLOCK, TTY_NOCURSOR, TTY_STARTED, TTYC_ACSC, TTYC_BCE, TTYC_CLEAR,
    TTYC_CUP, TTYC_KMOUS, TTYC_XT, tty_fake_bce, tty_set_size,
};
use crate::types::u_int;

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

// ---------------------------------------------------------------------------
// tty_set_size — pure field store, no descriptor
// ---------------------------------------------------------------------------

#[test]
fn tty_set_size_stores_dimensions() {
    let _guard = globals();
    let mut t = Tty::new();
    unsafe {
        tty_set_size(&mut *t.ptr(), 80, 24, 10, 20);
        assert_eq!((*t.ptr()).sx, 80);
        assert_eq!((*t.ptr()).sy, 24);
        assert_eq!((*t.ptr()).xpixel, 10);
        assert_eq!((*t.ptr()).ypixel, 20);

        tty_set_size(&mut *t.ptr(), 132, 50, 0, 0);
        assert_eq!((*t.ptr()).sx, 132);
        assert_eq!((*t.ptr()).sy, 50);
        assert_eq!((*t.ptr()).xpixel, 0);
        assert_eq!((*t.ptr()).ypixel, 0);
    }
}

// ---------------------------------------------------------------------------
// tty_fake_bce — BCE fast-path vs fake-BCE fallback
// ---------------------------------------------------------------------------

#[test]
fn tty_fake_bce_with_and_without_bce_capability() {
    let _guard = globals();
    let mut t = Tty::new();
    let mut gc = { grid_default_cell };
    unsafe {
        // plain bg/fg (0) -> fake BCE needed when no BCE flag
        gc.bg = 0;
        gc.fg = 0;
        let bg: u_int = 0;
        // no BCE advertised -> must fake
        assert_eq!(tty_fake_bce(&*t.ptr(), &gc, bg), 1);

        // advertise BCE
        t.set_flag(TTYC_BCE, 1);
        assert_eq!(tty_fake_bce(&*t.ptr(), &gc, bg), 0);

        // even without BCE, default colours (bg 8/9) do not need faking
        // reset to no-BCE
        t.clear_code(TTYC_BCE);
        gc.bg = 8;
        let bg2: u_int = 8;
        assert_eq!(tty_fake_bce(&*t.ptr(), &gc, bg2), 0);

        gc.bg = 9;
        let bg3: u_int = 9;
        assert_eq!(tty_fake_bce(&*t.ptr(), &gc, bg3), 0);

        // mismatched: gc default but bg not default -> still fake
        gc.bg = 8;
        let bg4: u_int = 0;
        assert_eq!(tty_fake_bce(&*t.ptr(), &gc, bg4), 1);
    }
}

#[test]
fn tyy_fixture_starts_zeroed_and_client_flags_are_settable() {
    let _guard = globals();
    let mut t = Tty::new();
    unsafe {
        assert_eq!((*t.ptr()).sx, 0);
        assert_eq!((*t.ptr()).sy, 0);
        assert_eq!(t.term().capability_count(), 233);
        for i in 0..t.term().capability_count() {
            assert!(!t.term().has(i as u_int));
        }
        t.set_client_flags(CLIENT_TERMINAL as u64);
        assert_eq!(t.term().acs(0), None);
        t.set_acs(b'a', "X");
        assert_eq!(t.term().acs(b'a'), Some(c"X"));
    }
    let t2 = Tty::new();
    assert!(!t2.term().has(TTYC_XT));
}
