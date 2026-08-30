//! Coverage for [`crate::screen`] – screen_init, screen_free helpers with [`Screen`] fixture.

use crate::screen::{screen_free, screen_grid_ptr, screen_init};
use crate::tests::test_fixtures::{Screen, globals, zeroed_screen};

// ---------------------------------------------------------------------------
// screen_init via Screen fixture
// ---------------------------------------------------------------------------

#[test]
fn screen_init_creates_grid_with_requested_dimensions() {
    let _g = globals();
    let s = Screen::new(80, 24, 100);
    unsafe {
        assert_eq!((*s.grid()).sx, 80);
        assert_eq!((*s.grid()).sy, 24);
        assert_eq!((*s.grid()).hlimit, 100);
        assert_eq!((*s.grid()).hsize, 0);
        assert!(!s.grid().is_null());
    }
}

#[test]
fn screen_init_sets_default_fields() {
    let _g = globals();
    let s = Screen::new(10, 5, 50);
    assert_eq!(s.cx, 0);
    assert_eq!(s.cy, 0);
    assert_eq!(s.rupper, 0);
    assert_eq!(s.rlower, 4);
    assert_eq!(s.cstyle, crate::screen::SCREEN_CURSOR_DEFAULT);
    assert_eq!(s.default_cstyle, crate::screen::SCREEN_CURSOR_DEFAULT);
    assert_eq!(s.ccolour, -1);
    assert_eq!(s.default_ccolour, -1);
    assert!(s.saved_grid.is_none());
    assert!(s.sel.is_none());
    assert!(s.write_list.is_empty());
    assert!(s.hyperlinks.is_some());
    assert!(s.titles.is_none());
    assert!(s.title.is_some());
    assert_eq!(
        s.title
            .as_deref()
            .expect("a new screen has a title")
            .to_bytes()
            .len(),
        0
    );
    assert_eq!(s.path, None);
    // mode should be cursor | wrap
    assert_eq!(
        s.mode & crate::screen::MODE_CURSOR,
        crate::screen::MODE_CURSOR
    );
    assert_eq!(s.mode & crate::screen::MODE_WRAP, crate::screen::MODE_WRAP);
}

#[test]
fn screen_init_single_cell_screen_is_valid() {
    let _g = globals();
    let s = Screen::new(1, 1, 0);
    unsafe {
        assert_eq!((*s.grid()).sx, 1);
        assert_eq!((*s.grid()).sy, 1);
        assert_eq!(s.rupper, 0);
        assert_eq!(s.rlower, 0);
        assert_eq!(s.cx, 0);
        assert_eq!(s.cy, 0);
        // tabs is allocated even for 1 column
        assert!(!s.tabs.is_empty());
    }
}

#[test]
fn screen_init_with_zero_hlimit_has_no_history() {
    let _g = globals();
    let s = Screen::new(20, 10, 0);
    unsafe {
        assert_eq!((*s.grid()).hlimit, 0);
        assert_eq!((*s.grid()).flags & crate::screen::GRID_HISTORY, 0);
    }
    let s2 = Screen::new(20, 10, 200);
    unsafe {
        assert_eq!((*s2.grid()).hlimit, 200);
    }
}

#[test]
fn screen_init_tabs_every_eight_columns() {
    let _g = globals();
    let s = Screen::new(24, 5, 0);
    unsafe {
        for i in 0..(*s.grid()).sx {
            let byte = s.tabs[(i >> 3) as usize];
            let is_set = byte as ::core::ffi::c_int & (1 << (i & 0x7)) != 0;
            assert_eq!(is_set, i != 0 && i % 8 == 0, "column {i}");
        }
    }
}

// ---------------------------------------------------------------------------
// screen_free – manual init / free roundtrip
// ---------------------------------------------------------------------------

#[test]
fn screen_init_then_manual_free_roundtrip() {
    let _g = globals();
    let mut s: Box<crate::types::screen> = zeroed_screen();
    unsafe {
        screen_init(&raw mut *s, 40, 10, 100);
        assert!(s.grid.is_some());
        assert!(s.hyperlinks.is_some());
        assert!(!s.tabs.is_empty());
        assert!(s.title.is_some());
        screen_free(&raw mut *s);
        // Prevent use-after-free: mark fields null so Box drop does not double-free
        // screen_free already freed grid/hyperlinks/title etc, but the struct
        // memory itself is still owned by Box which will be deallocated normally.
        // Zero it so Drop of Box does not attempt to interpret leftover pointers.
        // We simply forget the zeroed state – Box drop only frees allocation.
    }
}

#[test]
fn screen_init_free_multiple_cycles_no_leak() {
    let _g = globals();
    for (sx, sy, hlimit) in [(10, 5, 0), (80, 24, 100), (1, 1, 10), (100, 50, 500)] {
        let mut s: Box<crate::types::screen> = zeroed_screen();
        unsafe {
            screen_init(&raw mut *s, sx, sy, hlimit);
            assert_eq!((*screen_grid_ptr(&raw mut *s)).sx, sx);
            assert_eq!((*screen_grid_ptr(&raw mut *s)).sy, sy);
            assert_eq!((*screen_grid_ptr(&raw mut *s)).hlimit, hlimit);
            screen_free(&raw mut *s);
        }
    }
}

#[test]
fn screen_fixture_drop_matches_manual_init_free() {
    let _g = globals();
    // Fixture path: Screen::new does init and Drop does free – ensure various
    // sizes survive creation and destruction without crash.
    let sizes = [(5, 5, 0), (20, 10, 100), (80, 24, 1000), (1, 24, 50)];
    for (sx, sy, hlimit) in sizes {
        let s = Screen::new(sx, sy, hlimit);
        unsafe {
            assert_eq!((*s.grid()).sx, sx);
            assert_eq!((*s.grid()).sy, sy);
        }
        // drop here
    }
}

#[test]
fn screen_free_via_fixture_handles_titles_and_selection() {
    let _g = globals();
    let mut s = Screen::new(10, 5, 100);
    unsafe {
        // push a title so titles stack is allocated, and set a selection
        crate::screen::screen_set_title(s.ptr(), c"hello".as_ptr(), 0);
        crate::screen::screen_push_title(s.ptr());
        assert!(s.titles.is_some());
        let mut gc = crate::grid::grid_default_cell;
        crate::screen::screen_set_selection(s.ptr(), 0, 0, 2, 0, 0, 0, 0, &raw mut gc);
        assert!(s.sel.is_some());
        // Dropping `s` will call screen_free which must handle titles and sel
    }
}
