//! Coverage for screen initialization and owned teardown.

use crate::grid::Grid as _;
use crate::grid::Hyperlinks;
use crate::screen::{RustScreen, Screen as ScreenBoundary};
use crate::tests::test_fixtures::{Screen, globals, zeroed_screen};

// ---------------------------------------------------------------------------
// Construction via Screen fixture
// ---------------------------------------------------------------------------

#[test]
fn screen_init_creates_grid_with_requested_dimensions() {
    let _g = globals();
    let s = Screen::new(80, 24, 100);
    {
        assert_eq!((*s.grid()).width(), 80);
        assert_eq!((*s.grid()).height(), 24);
        assert_eq!((*s.grid()).history_limit(), 100);
        assert_eq!((*s.grid()).history_size(), 0);
        assert!(s.is_initialized());
    }
}

#[test]
fn screen_init_sets_default_fields() {
    let _g = globals();
    let s = Screen::new(10, 5, 50);
    assert_eq!(s.cursor(), (0, 0));
    assert_eq!(s.region(), (0, 4));
    assert_eq!(s.cursor_style(), crate::screen::SCREEN_CURSOR_DEFAULT);
    assert_eq!(
        s.default_cursor_style(),
        crate::screen::SCREEN_CURSOR_DEFAULT
    );
    assert_eq!(s.cursor_colour(), -1);
    assert_eq!(s.default_cursor_colour(), -1);
    assert!(!s.is_alternate());
    assert!(!s.has_selection());
    assert!(!s.is_collecting());
    assert!(s.hyperlinks().get(u32::MAX).is_none());
    assert!(!s.has_titles());
    assert!(s.title().is_some());
    assert_eq!(
        s.title()
            .expect("a new screen has a title")
            .to_bytes()
            .len(),
        0
    );
    assert_eq!(s.path(), None);
    // mode should be cursor | wrap
    assert_eq!(
        s.mode() & crate::screen::MODE_CURSOR,
        crate::screen::MODE_CURSOR
    );
    assert_eq!(
        s.mode() & crate::screen::MODE_WRAP,
        crate::screen::MODE_WRAP
    );
}

#[test]
fn screen_init_single_cell_screen_is_valid() {
    let _g = globals();
    let s = Screen::new(1, 1, 0);
    {
        assert_eq!((*s.grid()).width(), 1);
        assert_eq!((*s.grid()).height(), 1);
        assert_eq!(s.region(), (0, 0));
        assert_eq!(s.cursor(), (0, 0));
        // tabs is allocated even for 1 column
        assert!(s.has_tabs());
    }
}

#[test]
fn screen_init_with_zero_hlimit_has_no_history() {
    let _g = globals();
    let s = Screen::new(20, 10, 0);
    {
        assert_eq!((*s.grid()).history_limit(), 0);
        assert!(!s.grid().history_enabled());
    }
    let s2 = Screen::new(20, 10, 200);
    {
        assert_eq!((*s2.grid()).history_limit(), 200);
    }
}

#[test]
fn screen_init_tabs_every_eight_columns() {
    let _g = globals();
    let s = Screen::new(24, 5, 0);
    {
        for i in 0..(*s.grid()).width() {
            let is_set = s.tab_is_set(i);
            assert_eq!(is_set, i != 0 && i % 8 == 0, "column {i}");
        }
    }
}

// ---------------------------------------------------------------------------
// Owned replacement and teardown
// ---------------------------------------------------------------------------

#[test]
fn screen_init_then_manual_free_roundtrip() {
    let _g = globals();
    let mut s: Box<RustScreen> = zeroed_screen();
    {
        *s = RustScreen::new_with_server_options(40, 10, 100);
        assert!(s.is_initialized());
        assert!(s.hyperlinks().get(u32::MAX).is_none());
        assert!(s.has_tabs());
        assert!(s.title().is_some());
        *s = RustScreen::default();
    }
}

#[test]
fn screen_init_free_multiple_cycles_no_leak() {
    let _g = globals();
    for (sx, sy, hlimit) in [(10, 5, 0), (80, 24, 100), (1, 1, 10), (100, 50, 500)] {
        let mut s: Box<RustScreen> = zeroed_screen();
        {
            *s = RustScreen::new_with_server_options(sx, sy, hlimit);
            assert_eq!(RustScreen::grid(&s).width(), sx);
            assert_eq!(RustScreen::grid(&s).height(), sy);
            assert_eq!(RustScreen::grid(&s).history_limit(), hlimit);
            *s = RustScreen::default();
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
        {
            assert_eq!((*s.grid()).width(), sx);
            assert_eq!((*s.grid()).height(), sy);
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
        (&mut *s.ptr()).set_title(c"hello", 0);
        (&mut *s.ptr()).push_title();
        assert!(s.has_titles());
        let gc = crate::grid::grid_default_cell;
        (*s.ptr()).set_selection(0, 0, 2, 0, false, 0, 0, &gc);
        assert!(s.has_selection());
        // Dropping `s` must handle titles and the selection.
    }
}
