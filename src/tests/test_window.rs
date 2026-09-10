use super::*;
use crate::tests::test_fixtures::{Session, Window, globals, link, unlink};

#[test]
fn winlinks_in_hands_over_the_sessions_winlinks_in_index_order() {
    let _guard = globals();
    let mut s = Session::new(50, "walked");
    let mut first = Window::new(51, "first", 80, 24);
    let mut second = Window::new(52, "second", 80, 24);
    let mut third = Window::new(53, "third", 80, 24);
    assert_eq!(
        winlinks_in(s.handle()).count(),
        0,
        "an empty session walks to nothing"
    );
    let wl2 = link(&mut s, &mut third, 2);
    let wl0 = link(&mut s, &mut first, 0);
    let wl1 = link(&mut s, &mut second, 1);

    assert_eq!(
        winlinks_in(s.handle())
            .map(|wl| wl.index())
            .collect::<Vec<_>>(),
        vec![0, 1, 2],
        "the tree is walked by index, not by the order they were linked"
    );

    for wl in [wl0, wl1, wl2] {
        unlink(&mut s, wl);
    }
    assert_eq!(winlinks_in(s.handle()).count(), 0);
}

#[test]
fn session_link_snapshot_skips_removals_and_defers_new_indices() {
    let _guard = globals();
    let mut session = Session::new(50, "snapshot");
    let mut first = Window::new(51, "first", 80, 24);
    let mut second = Window::new(52, "second", 80, 24);
    let mut third = Window::new(53, "third", 80, 24);
    let wl0 = link(&mut session, &mut first, 0);
    let wl1 = link(&mut session, &mut second, 1);
    let snapshot = winlinks_in(session.handle());
    unlink(&mut session, wl1);
    let wl2 = link(&mut session, &mut third, 2);
    let held: Vec<_> = snapshot.collect();
    assert_eq!(held.iter().map(|wl| wl.index()).collect::<Vec<_>>(), [0]);
    assert!(held[0].get().is_some());
    unlink(&mut session, wl0);
    assert!(held[0].get().is_none());
    assert_eq!(
        winlinks_in(session.handle())
            .map(|wl| wl.index())
            .collect::<Vec<_>>(),
        [2]
    );
    unlink(&mut session, wl2);
}

#[test]
fn inserting_links_preserves_occupied_entries_and_wraps_automatic_indexes() {
    let mut links = winlinks::new();
    winlink_insert(&mut links, INT_MAX).unwrap().flags = WINLINK_BELL;
    assert!(winlink_insert(&mut links, INT_MAX).is_none());
    assert_eq!(links.get(&INT_MAX).unwrap().flags, WINLINK_BELL);
    assert_eq!(
        winlink_insert(&mut links, core::ffi::c_int::MIN)
            .unwrap()
            .idx,
        0
    );
    assert_eq!(winlink_insert(&mut links, -1).unwrap().idx, 1);
    assert_eq!(links.keys().copied().collect::<Vec<_>>(), [0, 1, INT_MAX]);
}

#[test]
fn removing_a_link_by_index_detaches_only_that_window_membership() {
    let _guard = globals();
    let mut session = Session::new(60, "remove");
    let mut window = Window::new(61, "window", 80, 24);
    link(&mut session, &mut window, 4);
    link(&mut session, &mut window, 2);
    let mut session = session.reference();
    let window = window.reference();
    let removed = WinlinkRef::new(session.clone(), 4).unwrap();
    unsafe {
        assert!(winlink_remove(&mut session.as_session_mut().windows, 4));
        assert!(removed.get().is_none());
        assert!(!winlink_remove(&mut session.as_session_mut().windows, 4));
        assert_eq!(
            window
                .winlinks()
                .map(|link| link.index())
                .collect::<Vec<_>>(),
            [2]
        );
        assert!(winlink_remove(&mut session.as_session_mut().windows, 2));
        assert!(window.as_window().winlinks.is_empty());
        session.as_session_mut().curw_idx = None;
    }
}

#[test]
fn shuffling_links_preserves_selection_history_and_owner_observations() {
    let _guard = globals();
    let mut s = Session::new(50, "shuffle");
    let mut first = Window::new(51, "first", 80, 24);
    let mut second = Window::new(52, "second", 80, 24);
    let mut distant = Window::new(53, "distant", 80, 24);
    link(&mut s, &mut first, 0);
    link(&mut s, &mut second, 1);
    link(&mut s, &mut distant, 3);
    let mut owner = s.reference();
    unsafe {
        owner.as_session_mut().curw_idx = Some(1);
        owner.as_session_mut().lastw = vec![0, 3];
        let shift = winlink_shuffle_up(owner.as_session_mut(), Some(0), 1).unwrap();
        assert_eq!(shift.index, 0);
        assert_eq!(owner.as_session().curw_idx, Some(2));
        assert_eq!(owner.as_session().lastw, vec![1, 3]);
        for (original, window) in [
            (0, first.reference()),
            (1, second.reference()),
            (3, distant.reference()),
        ] {
            let index = shift.remap(original);
            assert!(
                owner.as_session().windows[&index]
                    .window_handle()
                    .unwrap()
                    .ptr_eq(&window)
            );
            assert!(
                window
                    .winlinks()
                    .any(|link| link.index() == index && link.session().ptr_eq(&owner))
            );
        }
        assert!(winlink_shuffle_up(owner.as_session_mut(), Some(INT_MAX), 0).is_none());
        assert!(winlink_shuffle_up(owner.as_session_mut(), None, 1).is_none());
    }
    crate::tests::test_fixtures::unlink_all(&mut s);
}

pub(crate) fn window_registry_clear() {
    WINDOWS.with(HandleRegistry::clear);
}

pub(crate) fn window_has_floating_panes(w: &window) -> core::ffi::c_int {
    w.panes
        .iter()
        .any(|pane| window_pane_is_floating(w, &pane.downgrade()) != 0) as core::ffi::c_int
}

pub(crate) fn screen_write_sync_callback_for_test(wp: &mut impl crate::WindowPane) {
    screen_write_sync_callback(wp)
}

/// Keeps `id` out of the ids the server hands out, so a pane a test builds
/// by hand is never given the same id as one the server makes.
pub(crate) fn window_pane_reserve_id(id: u_int) {
    if next_window_pane_id.get().is_some_and(|next| next <= id) {
        next_window_pane_id.set(id.checked_add(1));
    }
}

/// Makes `wp` the window's active pane, or gives up having one when it is
/// null.
pub(crate) unsafe fn window_set_active(w: &mut window, wp: Option<&impl crate::WindowPane>) {
    w.active_pane = wp.and_then(|wp| {
        w.panes
            .iter()
            .find(|pane| core::ptr::addr_eq(unsafe { pane.as_pane() }, wp))
            .map(|pane| pane.downgrade())
    });
}

/// Where `wp` sits among `w`'s panes, or `None` when the window does not hold
/// it.
pub(crate) fn window_panes_position(
    w: &window,
    wp: Option<&impl crate::WindowPane>,
) -> Option<usize> {
    let wp = wp?;
    w.panes
        .iter()
        .position(|pane| core::ptr::addr_eq(pane.as_ptr(), wp))
}

/// Puts `wp` on top of a stacking order.
pub(crate) fn window_pane_zindex_insert_head(w: &mut window, wp: &impl crate::WindowPane) {
    let pane = w
        .panes
        .iter()
        .find(|pane| pane.pane_id() == wp.pane_id())
        .unwrap()
        .downgrade();
    pane_stack(&mut *w, PaneStack::ZIndex).insert(0, pane);
}
