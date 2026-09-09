use super::*;
use crate::tests::test_fixtures::{Format, Registry, Session, Window, globals, link, unlink};

#[test]
fn passes_is_true_without_a_filter_and_follows_the_filter_with_one() {
    let _guard = globals();
    let mut ft = Format::new();
    unsafe {
        assert!(passes(ft.tree(), None));
        assert!(passes(ft.tree(), Some(c"1")));
        assert!(passes(ft.tree(), Some(c"#{==:a,a}")));
        assert!(!passes(ft.tree(), Some(c"0")));
        assert!(!passes(ft.tree(), Some(c"")));
    }
}

#[test]
fn the_session_walk_is_that_session_windows_in_the_sorted_order() {
    let _guard = globals();
    let mut s = Session::new(31, "one");
    let mut other = Session::new(32, "two");
    let mut first = Window::new(41, "aaa", 80, 24);
    let mut second = Window::new(42, "bbb", 80, 24);
    let mut elsewhere = Window::new(43, "ccc", 80, 24);
    let wl0 = link(&mut s, &mut first, 0);
    let wl5 = link(&mut s, &mut second, 5);
    let wlo = link(&mut other, &mut elsewhere, 0);
    {
        let mut crit = RustSortCriteria::new(SORT_NAME, false);
        assert_eq!(
            link_keys((s.handle()).sorted_winlinks(&crit)),
            [(31, 0), (31, 5)]
        );
        crit.set_reversed(true);
        assert_eq!(
            link_keys((s.handle()).sorted_winlinks(&crit)),
            [(31, 5), (31, 0)]
        );
        crit.set_reversed(false);
        assert_eq!(
            link_keys((other.handle()).sorted_winlinks(&crit)),
            [(32, 0)]
        );
    }
    unlink(&mut other, wlo);
    unlink(&mut s, wl5);
    unlink(&mut s, wl0);
}

#[test]
fn the_all_walk_is_every_registered_session_windows_in_the_sorted_order() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut s = Session::new(33, "one");
    let mut other = Session::new(34, "two");
    registry.add_session(&mut s);
    registry.add_session(&mut other);
    let mut first = Window::new(44, "aaa", 80, 24);
    let mut second = Window::new(45, "bbb", 80, 24);
    let wl0 = link(&mut s, &mut first, 0);
    let wlo = link(&mut other, &mut second, 0);
    {
        let mut crit = RustSortCriteria::new(SORT_NAME, false);
        assert_eq!(link_keys(sort_get_winlinks(&crit)), [(33, 0), (34, 0)]);
        crit.set_reversed(true);
        assert_eq!(link_keys(sort_get_winlinks(&crit)), [(34, 0), (33, 0)]);
    }
    unlink(&mut other, wlo);
    unlink(&mut s, wl0);
}

#[test]
fn the_session_walk_default_template_is_the_upstream_one() {
    assert_eq!(
        LIST_WINDOWS_TEMPLATE.to_bytes(),
        b"#{window_index}: #{window_name}#{window_raw_flags} (#{window_panes} panes) [#{window_width}x#{window_height}] [layout #{window_layout}] #{window_id}#{?window_active, (active),}"
    );
}

fn link_keys(links: Vec<crate::window::WinlinkRef>) -> Vec<(u_int, core::ffi::c_int)> {
    links
        .iter()
        .map(|link| ({ link.session().id() }, link.index()))
        .collect()
}
