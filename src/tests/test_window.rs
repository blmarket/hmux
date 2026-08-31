use super::*;
use crate::tests::test_fixtures::{Session, Window, ensure_reactor, globals, link, unlink};
use ::std::sync::MutexGuard;

/// A turn at the server-wide state these tests reach — the window tree,
/// the tree of every pane and the id the next window is given — starting
/// from empty trees and leaving them empty.
fn server() -> MutexGuard<'static, ()> {
    let guard = globals();
    ensure_reactor();
    assert!(windows.map().is_empty(), "the window tree is not empty");
    assert!(pane_walk().next().is_none(), "the pane tree is not empty");
    guard
}

#[test]
fn winlinks_in_hands_over_the_sessions_winlinks_in_index_order() {
    let _guard = globals();
    let mut s = Session::new(50, "walked");
    let mut first = Window::new(51, "first", 80, 24);
    let mut second = Window::new(52, "second", 80, 24);
    let mut third = Window::new(53, "third", 80, 24);
    assert_eq!(
        unsafe { winlinks_in(s.ptr()) }.count(),
        0,
        "an empty session walks to nothing"
    );
    let wl2 = link(&mut s, &mut third, 2);
    let wl0 = link(&mut s, &mut first, 0);
    let wl1 = link(&mut s, &mut second, 1);

    assert_eq!(
        unsafe { winlinks_in(s.ptr()) }.collect::<Vec<_>>(),
        vec![wl0, wl1, wl2],
        "the tree is walked by index, not by the order they were linked"
    );

    for wl in [wl0, wl1, wl2] {
        unlink(&mut s, wl);
    }
    assert_eq!(unsafe { winlinks_in(s.ptr()) }.count(), 0);
}
