use super::*;
use crate::WindowPane;
use crate::options::OptionsRef;
use crate::pane_geometry::PaneGeometryState;
use crate::pane_identity::PaneIdentity;
use crate::tests::test_fixtures::{Pane, Target, Window, globals};
use crate::window_dimensions::WindowPixelSize;
use crate::window_fill_character::WindowFillCharacterState;
use crate::window_name::WindowNameState;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

#[test]
fn window_handle_snapshots_release_borrows_and_do_not_own_panes() {
    let _guard = globals();
    let fixture = Window::new(42, "before", 80, 24);
    let owner = fixture.handle();
    let pane = RustWindowPaneRef::from_pane(Box::new(window_pane::default()));
    let weak = pane.downgrade();
    owner.as_window_mut().panes.push(pane);
    owner.as_window_mut().active = Some(weak.clone());

    let name = owner.window_name();
    let options = owner.options();
    let panes = owner.panes();
    let active = owner.active_pane().unwrap();
    owner.set_window_name(Some(c"after"));
    owner.as_window_mut().panes.clear();

    assert_eq!(name.as_deref(), Some(c"before"));
    assert_eq!(owner.window_name().as_deref(), Some(c"after"));
    assert_eq!(owner.pane_count(), 0);
    assert_eq!(panes.len(), 1);
    assert!(panes[0].ptr_eq(&weak));
    assert!(!panes[0].is_alive());
    assert!(!active.is_alive());
    assert!(owner.active_pane().is_none());
    assert!(owner.active_pane_id().is_none());
    let _payload = owner.as_window_mut();
    unsafe {
        options.set_number(c"automatic-rename", 0);
        assert_eq!(options.number(c"automatic-rename"), 0);
    }
}

#[test]
fn moved_window_payload_cannot_recover_its_previous_owner() {
    let original = WindowRef::new(window::default());
    let moved = std::mem::take(&mut *original.as_window_mut());
    assert!(window_ref_of(&moved).is_none());
    let replacement = WindowRef::new(moved);
    assert!(
        window_ref_of(&replacement.as_window())
            .unwrap()
            .ptr_eq(&replacement)
    );
    assert!(!replacement.ptr_eq(&original));
}

#[test]
fn window_borrows_are_checked_across_cloned_handles() {
    let owner = WindowRef::new(window::default());
    owner.register_id();
    let other = owner.clone();
    let read = owner.as_window();
    let second_read = other.as_window();
    assert_eq!(read.window_id(), second_read.window_id());
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            other.as_window_mut();
        }))
        .is_err()
    );
    drop(read);
    drop(second_read);
    let mut write = owner.as_window_mut();
    write.flags = WINDOW_ZOOMED;
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            other.as_window();
        }))
        .is_err()
    );
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            other.as_window_mut();
        }))
        .is_err()
    );
    assert!(window_ref_of(&write).unwrap().ptr_eq(&owner));
    assert!(
        WindowRef::find_by_id(write.window_id())
            .unwrap()
            .ptr_eq(&owner)
    );
    drop(write);
    assert_eq!(other.as_window().flags, WINDOW_ZOOMED);
    other.as_window_mut().flags = 0;
}

#[test]
fn shown_screen_borrows_handle_pending_modes_and_base_fallback() {
    let mut pane = window_pane::default();
    assert!(core::ptr::eq(&*pane.try_screen_ref().unwrap(), pane.base()));
    *pane.shown_mut() = PaneScreen::Mode;
    assert!(core::ptr::eq(&*pane.try_screen_ref().unwrap(), pane.base()));
    pane.modes_mut().push(Box::new(window_mode_entry {
        wp: None,
        swp: None,
        state: WindowModeState::None,
        screen: None,
        prefix: 1,
    }));
    assert!(pane.try_screen_ref().is_none());
    pane.modes_mut()[0].state =
        WindowModeState::Clock(Box::new(crate::modes::window_clock_mode_data {
            screen: RustScreen::default(),
            tim: 0,
            timer: TimerHandle::ZERO,
        }));
    assert!(pane.try_screen_ref().is_none());
    pane.modes_mut()[0].screen = Some(ModeScreen::Clock);
    let WindowModeState::Clock(data) = &pane.modes()[0].state else {
        unreachable!()
    };
    assert!(core::ptr::eq(&*pane.screen_ref(), &data.screen));
    *pane.shown_mut() = PaneScreen::Base;
    assert!(core::ptr::eq(&*pane.screen_ref(), pane.base()));
    *pane.shown_mut() = PaneScreen::Mode;
    pane.modes_mut().clear();
    assert!(core::ptr::eq(&*pane.screen_ref(), pane.base()));
}

#[test]
fn window_observer_expires_when_its_last_owner_drops() {
    std::thread::spawn(|| {
        let reference = WindowRef::new(window::default());
        let observer = reference.downgrade();
        assert!(window_ref_of(&reference.as_window()).is_some());
        drop(reference);
        assert!(observer.upgrade().is_none());
    })
    .join()
    .unwrap();
}

#[test]
fn window_cleanup_works_during_thread_local_teardown() {
    thread_local! {
        static LAST_WINDOW: std::cell::RefCell<Option<WindowRef>> = const {
            std::cell::RefCell::new(None)
        };
    }
    std::thread::spawn(|| {
        LAST_WINDOW.with_borrow_mut(|last| {
            let reference = WindowRef::new(window::default());
            reference.register_id();
            {
                window_panes_insert_tail(
                    &mut reference.as_window_mut(),
                    RustWindowPaneRef::new(detached_pane()),
                );
            }
            *last = Some(reference);
        });
    })
    .join()
    .unwrap();
}

#[test]
fn entity_id_exhaustion_never_recycles_a_previous_identity() {
    std::thread::spawn(|| {
        for counter in [&next_window_id, &next_window_pane_id] {
            assert_eq!(next_entity_id(counter), 0);
            counter.set(Some(u_int::MAX - 1));
            assert_eq!(next_entity_id(counter), u_int::MAX - 1);
            assert_eq!(next_entity_id(counter), u_int::MAX);
            for _ in 0..2 {
                assert!(std::panic::catch_unwind(|| next_entity_id(counter)).is_err());
                assert_eq!(counter.get(), None);
            }
        }
        assert!(std::panic::catch_unwind(|| WindowRef::create(80, 24, 0, 0)).is_err());
    })
    .join()
    .unwrap();
}

#[test]
fn reserving_pane_ids_cannot_rewind_or_revive_an_exhausted_counter() {
    std::thread::spawn(|| {
        window_pane_reserve_id(42);
        assert_eq!(next_entity_id(&next_window_pane_id), 43);
        window_pane_reserve_id(1);
        assert_eq!(next_entity_id(&next_window_pane_id), 44);
        window_pane_reserve_id(u_int::MAX);
        window_pane_reserve_id(0);
        assert_eq!(next_window_pane_id.get(), None);
        assert!(std::panic::catch_unwind(|| next_entity_id(&next_window_pane_id)).is_err());
    })
    .join()
    .unwrap();
}

#[test]
fn registry_string_lookup_and_names_cover_valid_invalid_and_untrusted_paths() {
    let _guard = globals();
    let mut target = Target::new(40, 12);
    unsafe {
        let w = target.window(0);
        let wp = target.pane(0);
        assert!(
            WindowRef::find_by_id((*w).window_id())
                .is_some_and(|owner| core::ptr::eq(owner.as_ptr(), w))
        );
        assert_eq!(
            WindowRef::find_by_id_str(c"@0").map(|window| window.window_id()),
            Some(0)
        );
        assert!(WindowRef::find_by_id_str(c"0").is_none());
        assert!(WindowRef::find_by_id_str(c"@bad").is_none());
        assert_eq!(window_pane_find_by_id_str(c"%0").unwrap().as_mut_ptr(), wp);
        assert!(window_pane_find_by_id_str(c"0").is_none());
        assert!(window_pane_find_by_id_str(c"%bad").is_none());

        (window_ref_of(&*w).unwrap()).set_name(c"renamed", 0);
        assert_eq!((*w).window_name(), Some(c"renamed"));
        (window_ref_of(&*w).unwrap()).set_name(c"unsafe/name", 1);
        assert!((*w).window_name().is_some());
    }
}

#[test]
fn pane_order_active_last_used_zindex_and_direction_helpers_cover_boundaries() {
    let _guard = globals();
    let mut window = Window::new(50, "many", 40, 12);
    let mut p0 = Pane::new(50, 20, 12, 10);
    let mut p1 = Pane::new(51, 20, 12, 10);
    window.add_pane(&mut p0);
    window.add_pane(&mut p1);
    unsafe {
        let w = window.ptr();
        let a = p0.ptr();
        let b = p1.ptr();
        (*a).set_position(0, (*a).geometry().yoff);
        (*b).set_position(20, (*b).geometry().yoff);
        let payload = window.handle().as_window();
        let mut panes = payload.panes.iter();
        assert!(core::ptr::addr_eq(panes.next().unwrap().get().unwrap(), a));
        assert!(core::ptr::addr_eq(
            panes.next_back().unwrap().get().unwrap(),
            b
        ));
        assert!(panes.next().is_none());
        drop(payload);
        assert_eq!(
            window_pane_at_index(&*w, 0).map(|pane| pane.id()),
            Some((*a).pane_id())
        );
        window.options().set_number(c"pane-base-index", 5);
        assert!(window_pane_at_index(&*w, 0).is_none());
        assert_eq!(
            window_pane_at_index(&*w, 5).map(|pane| pane.id()),
            Some((*a).pane_id())
        );
        assert_eq!(
            window_pane_at_index(&*w, 6).map(|pane| pane.id()),
            Some((*b).pane_id())
        );
        assert!(window_pane_at_index(&*w, 7).is_none());
        window.options().set_number(c"pane-base-index", 0);
        assert_eq!(
            window
                .handle()
                .pane_by_id((*b).pane_id())
                .unwrap()
                .as_mut_ptr(),
            b
        );
        assert_eq!(window_panes_position(&*w, Some(&*b)), Some(1));
        let panes = window.handle().panes();
        for (n, forward, backward) in [(0, 0, 1), (1, 1, 0), (2, 0, 1), (3, 1, 0)] {
            assert!(
                window_pane_next_by_number(&*w, Some(&panes[0]), n)
                    .unwrap()
                    .ptr_eq(&panes[forward])
            );
            assert!(
                window_pane_previous_by_number(&*w, Some(&panes[1]), n)
                    .unwrap()
                    .ptr_eq(&panes[backward])
            );
        }
        assert!(window_pane_next_by_number(&*w, None, 0).is_none());
        assert!(
            window_pane_next_by_number(&*w, None, 1)
                .unwrap()
                .ptr_eq(&panes[0])
        );
        assert!(
            window_pane_previous_by_number(&*w, None, 1)
                .unwrap()
                .ptr_eq(&panes[1])
        );
        let empty = crate::types::window::default();
        assert!(
            window_pane_next_by_number(&empty, Some(&panes[0]), 0)
                .unwrap()
                .ptr_eq(&panes[0])
        );
        assert!(window_pane_next_by_number(&empty, Some(&panes[0]), 1).is_none());
        assert!(window_pane_previous_by_number(&empty, Some(&panes[0]), 1).is_none());
        assert_eq!(window_pane_index(&*w, &*a).0, 0);
        assert_eq!(window_pane_zindex(&panes[1]).0, 0);
        assert_eq!(window_count_panes(&mut *w, 1), 2);

        let selected_id = (*b).pane_id();
        assert_eq!(
            (window.reference()).set_active_pane(
                &crate::window::window_pane_find_by_id(selected_id)
                    .expect("the selected pane exists"),
                0
            ),
            1
        );
        assert!(window_active_pane(&*w).is_some_and(|pane| {
            pane.get()
                .is_some_and(|active| core::ptr::addr_eq(active, b))
        }));
        assert_eq!(
            (*w).last_panes.first().map(|pane| pane.id()),
            Some((*a).pane_id())
        );
        let selected_id = (*b).pane_id();
        assert_eq!(
            (window.reference()).set_active_pane(
                &crate::window::window_pane_find_by_id(selected_id)
                    .expect("the selected pane exists"),
                0
            ),
            0
        );
        assert_eq!(
            window_find_string(&*w, c"left").map(|pane| pane.id()),
            Some((*a).pane_id())
        );
        assert_eq!(
            window_find_string(&*w, c"right").map(|pane| pane.id()),
            Some((*b).pane_id())
        );
        assert!(window_find_string(&*w, c"unknown").is_none());
        assert!(window_pane_find_left(Some(&*b)).is_none());
        assert!(window_pane_find_right(Some(&*a)).is_none());

        window_pane_zindex_remove(&mut *w, &*a);
        window_pane_zindex_insert_head(&mut *w, &*a);
        window_pane_zindex_remove(&mut *w, &*b);
        window_pane_zindex_insert_tail(&mut *w, &*b);
        assert_eq!(
            (*w).z_index
                .iter()
                .map(|pane| pane.id())
                .collect::<Vec<_>>(),
            [(*a).pane_id(), (*b).pane_id()]
        );
    }
}

#[test]
fn resize_modes_visibility_flags_and_fill_character_cover_safe_state_paths() {
    let _guard = globals();
    let mut target = Target::new(40, 12);
    unsafe {
        let w = target.window(0);
        let wp = target.pane(0);
        assert_eq!(window_pane_mode(&*wp), 0);
        assert!(window_pane_current_mode(&*wp).is_none());
        let source_pane_id = (*wp).pane_id();
        assert_eq!(
            window_pane_set_mode(
                &mut *wp,
                crate::window::window_pane_find_by_id(source_pane_id),
                WindowMode::Copy,
                None,
                None
            ),
            0
        );
        assert_ne!(window_pane_mode(&*wp), 0);
        assert!(window_pane_current_mode(&*wp).is_some());
        assert_eq!(
            window_pane_set_mode(
                &mut *wp,
                crate::window::window_pane_find_by_id(source_pane_id),
                WindowMode::Copy,
                None,
                None
            ),
            1
        );
        window_pane_resize(&mut *wp, 30, 8);
        assert_eq!(((*wp).geometry().sx, (*wp).geometry().sy), (30, 8));
        window_pane_reset_mode_all(&mut *wp);
        assert!(window_pane_current_mode(&*wp).is_none());

        assert_eq!(window_pane_visible(&*w, &*wp), 1);
        *(*wp).flags_mut() |= PANE_EXITED;
        assert_eq!(window_pane_exited(&*wp), 1);
        *(*wp).flags_mut() &= !PANE_EXITED;
        assert_eq!(
            window_pane_is_floating(
                &*w,
                &{ crate::window::window_pane_ref_of(&(*wp)) }.expect("the pane allocation exists")
            ),
            0
        );
        assert_eq!(window_has_floating_panes(&mut *w), 0);
        assert_eq!(
            window_pane_printable_flags(&target.state().pane_ref().unwrap())
                .unwrap()
                .as_bytes(),
            b"*"
        );

        ((*w).options_ref()).set_string(c"fill-character", 0, c"%s", fmt_args![c"#".as_ptr()]);
        window_set_fill_character(&mut *w);
        assert!(w.as_ref().unwrap().fill_character().is_some());
        ((*w).options_ref()).set_string(c"fill-character", 0, c"%s", fmt_args![c"".as_ptr()]);
        window_set_fill_character(&mut *w);
        assert!(w.as_ref().unwrap().fill_character().is_none());
    }
}

#[test]
fn window_link_lookup_borrows_the_first_matching_allocation() {
    let _guard = globals();
    let first = WindowRef::new(window::default());
    let second = WindowRef::new(window::default());
    let mut links = winlinks::new();
    for (index, owner) in [
        (2, Some(first.clone())),
        (4, Some(second.clone())),
        (6, Some(first.clone())),
        (0, None),
    ] {
        links.insert(
            index,
            Box::new(winlink {
                idx: index,
                session: None,
                window: owner,
                flags: 0,
            }),
        );
    }
    {
        assert_eq!(first.window_id(), second.window_id());
        let first_link = winlink_find_by_window(&links, &first.as_window()).unwrap();
        let second_link = winlink_find_by_window(&links, &second.as_window()).unwrap();
        assert_eq!((first_link.idx, second_link.idx), (2, 4));
        links.remove(&2);
        assert_eq!(
            winlink_find_by_window(&links, &first.as_window()).map(|link| link.idx),
            Some(6)
        );
        links.remove(&6);
        assert!(winlink_find_by_window(&links, &first.as_window()).is_none());
        assert_eq!(
            winlink_find_by_window(&links, &second.as_window()).map(|link| link.idx),
            Some(4)
        );
    }
}

#[test]
fn winlink_iteration_lookup_numbering_stack_and_flags_cover_edge_cases() {
    let _guard = globals();
    let mut target = Target::new(40, 12);
    target.add_window(3, 40, 12);
    unsafe {
        let s = target.session();
        let first = target.winlink(0);
        let second = target.winlink(1);
        let mut links = target.session_handle().as_session().windows.values();
        assert!(core::ptr::eq(links.next().unwrap().as_ref(), first));
        assert!(core::ptr::eq(links.next_back().unwrap().as_ref(), second));
        assert!(links.next().is_none());
        assert_eq!(winlink_count(&(*s).windows), 2);
        assert!(
            (*s).windows
                .get(&3)
                .map(Box::as_ref)
                .is_some_and(|link| core::ptr::eq(link, second))
        );
        assert!(
            winlink_find_by_window(&(*s).windows, &*target.window(1))
                .is_some_and(|link| core::ptr::eq(link, second))
        );
        assert_eq!(
            winlink_find_by_window_id(&(*s).windows, (*target.window(1)).window_id())
                .map(|link| link.idx),
            Some(3)
        );
        for (n, index) in [(-1, 0), (0, 0), (1, 3), (2, 0), (3, 3)] {
            assert_eq!(
                winlink_next_by_number(&*first, &*s, n).map(|link| link.idx),
                Some(index)
            );
        }
        for (n, index) in [(-1, 3), (0, 3), (1, 0), (2, 3), (3, 0)] {
            assert_eq!(
                winlink_previous_by_number(&*second, &*s, n).map(|link| link.idx),
                Some(index)
            );
        }

        winlink_stack_push(&mut (*s).lastw, Some(&mut *first));
        assert_eq!((*s).lastw.first(), Some(&(*first).idx));
        winlink_stack_remove(&mut (*s).lastw, Some(&mut *first));
        assert!((*s).lastw.is_empty());
        (*first).flags = WINLINK_ACTIVITY | WINLINK_BELL | WINLINK_SILENCE;
        assert_eq!(window_printable_flags(&*first, 0).as_bytes(), b"#!~*");
        assert_eq!(window_printable_flags(&*first, 1).as_bytes(), b"##!~*");
        ((*first)
            .window_handle()
            .expect("a link owns its window")
            .clone())
        .clear_alerts();
        assert_eq!(
            (*first).flags & (WINLINK_ACTIVITY | WINLINK_BELL | WINLINK_SILENCE),
            0
        );
    }
}

#[test]
fn window_id_registration_is_removed_with_last_owner() {
    let _guard = globals();
    let reference = WindowRef::new(window::default());
    let id = unsafe { (*reference.as_ptr()).window_id() };
    reference.register_id();
    drop(reference);
    assert!(!window_ids().contains(&id));
}

#[test]
fn window_id_registration_is_thread_local_and_retains_replacements() {
    std::thread::spawn(|| {
        let old = WindowRef::new(window::default());
        old.register_id();
        let id = unsafe { (*old.as_ptr()).window_id() };
        std::thread::spawn(move || {
            assert!(WindowRef::find_by_id(id).is_none());
            assert!(window_ids().is_empty());
        })
        .join()
        .unwrap();
        let current = WindowRef::new(window::default());
        current.register_id();
        let retained = current.clone();
        drop(old);
        drop(current);
        assert_eq!(
            WindowRef::find_by_id(id).unwrap().as_ptr(),
            retained.as_ptr()
        );
        drop(retained);
        assert!(!window_ids().contains(&id));
    })
    .join()
    .unwrap();
}

#[test]
fn pane_registration_cleanup_outlives_the_thread_local_index() {
    thread_local! {
        static LAST_PANE: std::cell::RefCell<Option<RustWindowPaneRef>> = const {
            std::cell::RefCell::new(None)
        };
    }
    std::thread::spawn(|| {
        LAST_PANE.with_borrow_mut(|last| {
            *last = Some(RustWindowPaneRef::new(detached_pane()));
        });
    })
    .join()
    .unwrap();
}

#[test]
fn pane_registration_removes_only_its_own_entry() {
    let index = GlobalPaneIndex {
        panes: HandleRegistry::new(),
    };
    let old = index.register(detached_pane());
    let replacement = index.register(detached_pane());
    let id = replacement.pane_id();
    drop(old);
    assert!(index.find(id).is_some());
    drop(replacement);
    assert!(index.ids().is_empty());
}

#[test]
fn pane_walk_skips_removed_ids_and_defers_new_registrations() {
    let _guard = globals();
    let registered = |id| {
        let mut pane = detached_pane();
        pane.set_pane_id(id);
        RustWindowPaneRef::new(pane)
    };
    let first = registered(1);
    let removed = registered(2);
    let last = registered(3);
    let mut walk = pane_walk();
    assert_eq!(walk.next(), Some(first.downgrade()));
    drop(removed);
    let added = registered(4);
    assert_eq!(walk.collect::<Vec<_>>(), [last.downgrade()]);
    assert_eq!(
        pane_walk().collect::<Vec<_>>(),
        [first.downgrade(), last.downgrade(), added.downgrade()]
    );
}

#[test]
fn pane_owner_transfer_preserves_address_and_expires_deferred_observers() {
    let _guard = globals();
    let mut allocation = detached_pane();
    let pointer = core::ptr::from_mut(&mut *allocation);
    let owner = RustWindowPaneRef::new(allocation);
    let id = owner.pane_id();
    assert_eq!(owner.as_ptr(), pointer);
    let calls = std::rc::Rc::new(std::cell::Cell::new(0));
    let observed = calls.clone();
    let callback = on_pane(id, move |_| observed.set(observed.get() + 1));
    let source = WindowRef::new(window::default());
    let destination = WindowRef::new(window::default());
    unsafe {
        window_panes_insert_tail(&mut source.as_window_mut(), owner);
        let mut transferred = window_panes_take(
            &mut source.as_window_mut(),
            &crate::window::window_pane_find_by_id(id).expect("the pane exists"),
        )
        .unwrap();
        assert!(source.as_window().panes.is_empty());
        assert!(
            window_pane_find_by_id(id)
                .unwrap()
                .ptr_eq(&transferred.downgrade())
        );
        callback(Stream::NONE);
        assert_eq!(calls.get(), 0);
        window_pane_set_window_ref(transferred.as_pane_mut(), Some(&destination));
        assert_eq!(
            window_panes_insert_tail(&mut destination.as_window_mut(), transferred).id(),
            id
        );
        assert_eq!(window_pane_find_by_id(id).unwrap().as_mut_ptr(), pointer);
        assert!(
            window_pane_find_by_id(id)
                .unwrap()
                .window()
                .unwrap()
                .ptr_eq(&destination)
        );
    }
    callback(Stream::NONE);
    assert_eq!(calls.get(), 1);
    drop(destination);
    assert!(window_pane_find_by_id(id).is_none());
    callback(Stream::NONE);
    assert_eq!(calls.get(), 1);
}

#[test]
fn pane_lookup_follows_swapped_owners_after_exclusive_borrows() {
    let _guard = globals();
    let mut source = WindowRef::new(window::default());
    let mut destination = WindowRef::new(window::default());
    let pane = |id| {
        let mut value = detached_pane();
        value.set_pane_id(id);
        RustWindowPaneRef::new(value)
    };
    unsafe {
        window_panes_insert_tail(&mut source.as_window_mut(), pane(1));
        window_panes_insert_tail(&mut destination.as_window_mut(), pane(2));
        let mut previous = window_pane_find_by_id(1).unwrap();
        *previous.get_mut().unwrap().flags_mut() = PANE_REDRAW;
        assert_eq!(
            *window_pane_find_by_id(1).unwrap().get().unwrap().flags(),
            PANE_REDRAW
        );
        source.swap_panes(
            &previous,
            &mut destination,
            &window_pane_find_by_id(2).unwrap(),
        );
        assert!(previous.get().is_some());
        assert!(previous.window().unwrap().ptr_eq(&destination));
        let mut moved = window_pane_find_by_id(1).unwrap();
        assert!(previous.ptr_eq(&moved));
        assert!(moved.window().unwrap().ptr_eq(&destination));
        assert!(
            moved
                .get()
                .unwrap()
                .window_context()
                .unwrap()
                .ptr_eq(&destination)
        );
        *moved.get_mut().unwrap().flags_mut() |= PANE_CHANGED;
        assert_eq!(
            *window_pane_find_by_id(1).unwrap().get().unwrap().flags(),
            PANE_REDRAW | PANE_CHANGED
        );
        let other = window_pane_find_by_id(2).unwrap();
        assert!(other.window().unwrap().ptr_eq(&source));
        assert!(
            other
                .get()
                .unwrap()
                .window_context()
                .unwrap()
                .ptr_eq(&source)
        );
        drop(previous);
        drop(other);
        drop(source);
        assert!(window_pane_find_by_id(2).is_none());
        assert!(window_pane_find_by_id(1).is_some());
    }
}

#[test]
fn moving_an_old_pane_cannot_rebind_a_replacement_registration() {
    let _guard = globals();
    let source = WindowRef::new(window::default());
    let destination = WindowRef::new(window::default());
    {
        let old = RustWindowPaneRef::new(detached_pane());
        let id = old.pane_id();
        let original = old.downgrade();
        window_panes_insert_tail(&mut source.as_window_mut(), old);
        let replacement = RustWindowPaneRef::new(detached_pane());
        window_panes_insert_tail(&mut destination.as_window_mut(), replacement);
        let detached = window_panes_take(&mut source.as_window_mut(), &original).unwrap();
        window_panes_insert_tail(&mut source.as_window_mut(), detached);
        assert!(
            window_pane_find_by_id(id)
                .unwrap()
                .window()
                .unwrap()
                .ptr_eq(&destination)
        );
        drop(source);
        assert!(
            window_pane_find_by_id(id)
                .unwrap()
                .window()
                .unwrap()
                .ptr_eq(&destination)
        );
        drop(destination);
        assert!(window_pane_find_by_id(id).is_none());
    }
}

#[test]
fn target_window_observation_uses_owners_without_borrowing_the_payload() {
    let reference = WindowRef::new(window::default());
    let mut target = cmd_find_state::default();
    target.set_window_ref(Some(&reference));
    assert!(target.window().unwrap().ptr_eq(&reference));
    drop(reference);
    assert!(target.window().is_none());
}

#[test]
fn losing_the_active_pane_uses_history_then_owned_neighbors() {
    let _g = globals();
    for (lost, history, expected) in [
        (1, vec![], 2),
        (2, vec![], 1),
        (3, vec![1], 1),
        (3, vec![999, 1], 2),
    ] {
        let mut fixture = Window::new(10, "lost-pane", 80, 24);
        let mut panes = (1..=3)
            .map(|id| Pane::new(id, 80, 24, 100))
            .collect::<Vec<_>>();
        for pane in &mut panes {
            fixture.add_pane(pane);
        }
        let owner = fixture.reference();
        unsafe {
            let mut w = owner.as_window_mut();
            w.active = w
                .panes
                .iter()
                .find(|pane| pane.pane_id() == lost)
                .map(|pane| pane.downgrade());
            w.last_panes = history
                .into_iter()
                .map(|id| {
                    crate::window::window_pane_find_by_id(id).unwrap_or_else(|| {
                        RustWindowPaneRef::from_pane(Box::new(window_pane::default())).downgrade()
                    })
                })
                .collect();
            for pane in &mut w.panes {
                *pane.as_pane_mut().flags_mut() |= PANE_VISITED;
            }
            let lost_pane = w
                .panes
                .iter()
                .find(|pane| pane.id() == lost)
                .unwrap()
                .downgrade();
            drop(w);
            owner.lost_pane(&lost_pane);
            let w = owner.as_window();
            assert_eq!(w.active_pane_id(), Some(expected));
            assert!(!w.last_panes.iter().any(|pane| pane.pane_id() == Some(lost)));
            assert!(!w.last_panes.iter().any(|pane| pane.pane_id() == Some(expected)));
            let selected = w
                .panes
                .iter()
                .find(|pane| pane.pane_id() == expected)
                .unwrap()
                .as_pane();
            assert_eq!(*selected.flags() & PANE_VISITED, 0);
            assert_ne!(*selected.flags() & PANE_CHANGED, 0);
        }
    }
}

#[test]
fn pane_index_observers_do_not_retain_the_allocation() {
    let index = GlobalPaneIndex {
        panes: HandleRegistry::new(),
    };
    let original = index.register(detached_pane());
    let id = original.id();
    let weak = original.downgrade();
    let retained = index.find(id).unwrap();
    assert!(original.downgrade().ptr_eq(&retained));
    drop(original);
    assert!(weak.upgrade().is_none());
    assert!(unsafe { retained.get() }.is_none());
    assert!(index.find(id).is_none());
    drop(retained);
    assert!(weak.upgrade().is_none());
    assert!(index.find(id).is_none());
    assert!(index.ids().is_empty());
}

#[test]
fn pane_removal_tears_down_resources_before_the_last_reference_drops() {
    use std::os::fd::{AsRawFd, IntoRawFd};
    use std::os::unix::net::UnixStream;

    let _guard = globals();
    let owner = WindowRef::create(20, 8, 0, 0);
    unsafe {
        let id = window_add_pane(&mut owner.as_window_mut(), None, 0, 0).id();
        let mut retained = window_pane_find_by_id(id).unwrap();
        let weak = retained.clone();
        let calls = std::rc::Rc::new(std::cell::Cell::new(0));
        let observed = calls.clone();
        let callback = on_pane(id, move |_| observed.set(observed.get() + 1));
        let (pipe, peer) = UnixStream::pair().unwrap();
        let fd = pipe.as_raw_fd();
        *retained.get_mut().unwrap().pipe_fd_mut() = pipe.into_raw_fd();
        screen_write_start_sync(retained.get_mut());
        let timer = *retained.get().unwrap().sync_timer();
        assert!(timer.is_armed());
        callback(Stream::NONE);
        assert_eq!(calls.get(), 1);

        owner.remove_pane(&crate::window::window_pane_find_by_id(id).expect("the pane exists"));
        assert!(window_pane_find_by_id(id).is_none());
        assert!(!pane_walk().any(|pane| pane.id() == id));
        assert!(retained.get().is_none());
        assert!(retained.get_mut().is_none());
        assert!(retained.window().is_none());
        assert!(weak.upgrade().is_none());
        assert!(retained.as_ptr().is_null());
        assert!(!timer.is_armed());
        assert_eq!(libc::fcntl(fd, libc::F_GETFD), -1);
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::EBADF)
        );
        callback(Stream::NONE);
        assert_eq!(calls.get(), 1);
        drop(peer);
        drop(retained);
        assert!(weak.upgrade().is_none());
    }
}

#[test]
fn retained_pane_does_not_keep_its_window_alive_or_delay_window_teardown() {
    let _guard = globals();
    let owner = WindowRef::create(20, 8, 0, 0);
    let weak_window = owner.downgrade();
    let id = unsafe { window_add_pane(&mut owner.as_window_mut(), None, 0, 0).id() };
    let mut pane = window_pane_find_by_id(id).unwrap();
    unsafe {
        assert_eq!(
            window_pane_set_mode(
                pane.as_pane_mut(),
                crate::window::window_pane_find_by_id(id),
                WindowMode::Copy,
                None,
                None
            ),
            0
        );
    }
    let weak_pane = pane.clone();
    let last_owner = owner.clone();
    drop(owner);
    assert!(weak_window.upgrade().is_some());
    assert!(window_pane_find_by_id(id).is_some());
    assert!(unsafe { (*pane.as_ptr()).options().is_some() });
    drop(last_owner);
    assert!(weak_window.upgrade().is_none());
    assert!(window_pane_find_by_id(id).is_none());
    assert!(pane.window().is_none());
    assert!(unsafe { pane.get() }.is_none());
    assert!(weak_pane.upgrade().is_none());
    assert!(pane.as_ptr().is_null());
    drop(pane);
    assert!(weak_pane.upgrade().is_none());
}

#[test]
fn fixture_cleanup_retires_panes_without_closing_borrowed_descriptors() {
    use std::os::fd::AsRawFd;
    use std::os::unix::net::UnixStream;

    let _guard = globals();
    let (borrowed, peer) = UnixStream::pair().unwrap();
    let mut fixture = Window::new(952, "borrowed-io", 20, 4);
    let mut pane = Pane::new(953, 20, 4, 0);
    fixture.add_pane(&mut pane);
    let owner = fixture.reference();
    let retained = window_pane_find_by_id(953).unwrap();
    unsafe {
        *(*pane.ptr()).fd_mut() = borrowed.as_raw_fd();
        *(*pane.ptr()).pipe_fd_mut() = peer.as_raw_fd();
    }
    drop(fixture);
    assert!(window_pane_find_by_id(953).is_none());
    assert!(retained.window().is_none());
    assert!(unsafe { retained.get() }.is_none());
    assert!(retained.as_ptr().is_null());
    drop(owner);
    drop(retained);
    assert_ne!(
        unsafe { libc::fcntl(borrowed.as_raw_fd(), libc::F_GETFD) },
        -1
    );
    assert_ne!(unsafe { libc::fcntl(peer.as_raw_fd(), libc::F_GETFD) }, -1);
}

#[test]
fn synchronized_key_and_paste_skip_the_source_and_unavailable_destinations() {
    use crate::tests::test_fixtures::StreamBuffer;

    let _guard = globals();
    let mut window = Window::new(901, "synchronized", 20, 4);
    let mut fixtures: Vec<_> = (0..5)
        .map(|index| Pane::new(901 + index, 20, 4, 0))
        .collect();
    let streams: Vec<_> = (0..5).map(|_| StreamBuffer::new()).collect();
    for pane in &mut fixtures {
        window.add_pane(pane);
    }
    unsafe {
        let mut panes = window.handle().panes();
        for (pane, stream) in panes.iter_mut().zip(&streams) {
            let wp = pane.as_pane_mut();
            *wp.fd_mut() = 1000;
            *wp.event_mut() = stream.ptr();
            wp.options_ref().set_number(c"synchronize-panes", 1);
        }
        *panes[2].as_pane_mut().flags_mut() |= PANE_INPUTOFF;
        panes[3]
            .as_pane()
            .options_ref()
            .set_number(c"synchronize-panes", 0);
        *panes[4].as_pane_mut().fd_mut() = -1;
        window_pane_copy_key(panes[0].as_pane(), b'x' as key_code);
        window_pane_copy_paste(panes[0].as_pane(), ByteBuffer::from(b"paste".to_vec()));
        assert!(streams[0].written().is_empty());
        assert_eq!(streams[1].written(), b"xpaste");
        assert!(
            streams[2..]
                .iter()
                .all(|stream| stream.written().is_empty())
        );
        window_pane_paste(
            panes[0].as_pane(),
            KEYC_NONE,
            ByteBuffer::from(b"own".to_vec()),
        );
        assert_eq!(streams[0].written(), b"own");
        assert_eq!(streams[1].written(), b"own");
        assert!(
            streams[2..]
                .iter()
                .all(|stream| stream.written().is_empty())
        );
    }
}

#[test]
fn directional_selection_retains_the_most_recent_candidate_and_preserves_first_ties() {
    use crate::pane_activity::PaneActivityState;
    use crate::pane_geometry::PaneGeometry;

    let _guard = globals();
    let mut window = Window::new(950, "direction", 40, 20);
    let mut fixtures: Vec<_> = (0..3).map(|id| Pane::new(950 + id, 20, 20, 0)).collect();
    for pane in &mut fixtures {
        window.add_pane(pane);
    }
    unsafe {
        let mut panes = window.handle().panes();
        panes[0].as_pane_mut().set_geometry(PaneGeometry {
            xoff: 0,
            yoff: 0,
            sx: 19,
            sy: 20,
        });
        panes[1].as_pane_mut().set_geometry(PaneGeometry {
            xoff: 20,
            yoff: 0,
            sx: 20,
            sy: 9,
        });
        panes[2].as_pane_mut().set_geometry(PaneGeometry {
            xoff: 20,
            yoff: 10,
            sx: 20,
            sy: 10,
        });
        panes[1].as_pane_mut().mark_active_at(7);
        panes[2].as_pane_mut().mark_active_at(9);
        let selected = window_pane_find_right(Some(panes[0].as_pane())).unwrap();
        assert!(selected.ptr_eq(&panes[2]));
        panes[1].as_pane_mut().mark_active_at(9);
        let selected = window_pane_find_right(Some(panes[0].as_pane())).unwrap();
        assert!(selected.ptr_eq(&panes[1]));
        assert!(window_pane_find_up(None::<&crate::types::window_pane>).is_none());
        assert!(window_pane_find_down(None::<&crate::types::window_pane>).is_none());
        assert!(window_pane_find_left(None::<&crate::types::window_pane>).is_none());
        assert!(window_pane_find_right(None::<&crate::types::window_pane>).is_none());
    }
}

fn detached_pane() -> Box<window_pane> {
    let mut pane = Box::new(window_pane::default());
    *pane.fd_mut() = -1;
    *pane.pipe_fd_mut() = -1;
    pane
}

#[test]
fn window_pane_lookup_checks_the_list_while_transfers_retain_the_backlink() {
    let _guard = globals();
    let source = WindowRef::new(window::default());
    let destination = WindowRef::new(window::default());
    let pane = RustWindowPaneRef::new(detached_pane());
    let id = pane.pane_id();
    unsafe {
        window_panes_insert_tail(&mut source.as_window_mut(), pane);
        let lookup = source.clone();
        assert!(lookup.pane_by_id(id).is_some());
        assert!(lookup.pane_by_id(id + 1).is_none());
        assert!(destination.pane_by_id(id).is_none());
        let mut payload = source.as_window_mut();
        payload.active = payload
            .panes
            .iter()
            .find(|pane| pane.pane_id() == id)
            .map(|pane| pane.downgrade());
        drop(payload);
        let retained = source.pane_by_id(id).unwrap();
        let moved = window_panes_take(
            &mut source.as_window_mut(),
            &crate::window::window_pane_find_by_id(id).expect("the pane exists"),
        )
        .unwrap();
        assert!(source.pane_by_id(id).is_none());
        assert!(destination.pane_by_id(id).is_none());
        assert!(moved.window().unwrap().ptr_eq(&source));
        assert!(moved.as_pane().window_context().unwrap().ptr_eq(&source));
        assert!(window_pane_find_by_id(id).unwrap().ptr_eq(&retained));
        window_panes_insert_tail(&mut destination.as_window_mut(), moved);
        assert!(source.pane_by_id(id).is_none());
        assert!(destination.pane_by_id(id).unwrap().ptr_eq(&retained));
        assert!(
            retained
                .as_pane()
                .window_context()
                .unwrap()
                .ptr_eq(&destination)
        );
        destination
            .remove_pane(&crate::window::window_pane_find_by_id(id).expect("the pane exists"));
        assert!(destination.pane_by_id(id).is_none());
        assert!(window_pane_find_by_id(id).is_none());
        assert!(retained.get().is_none());
    }
}

#[test]
fn registry_lookup_requires_registration_and_list_membership() {
    let _guard = globals();
    let owner = WindowRef::new(window::default());
    let unregistered = RustWindowPaneRef::from_pane(detached_pane());
    let id = unregistered.pane_id();
    let pointer = unregistered.as_mut_ptr();
    unsafe {
        window_panes_insert_tail(&mut owner.as_window_mut(), unregistered);
        assert!(owner.pane_by_id(id).is_none());
        assert!(
            owner
                .as_window()
                .panes
                .iter()
                .any(|pane| pane.pane_id() == id)
        );
        let mut target = cmd_find_state::default();
        target.set_window_ref(Some(&owner));
        target.wp = Some(owner.as_window().panes[0].downgrade());
        assert!(core::ptr::addr_eq(
            target.pane_list_ref().unwrap().get().unwrap(),
            pointer
        ));
        assert!(target.pane_ref().is_some());
        owner.remove_pane(&target.pane_list_ref().unwrap());
        assert!(target.pane_list_ref().is_none());

        let mut pane = RustWindowPaneRef::new(detached_pane());
        window_pane_set_window_ref(pane.as_pane_mut(), Some(&owner));
        owner.as_window_mut().panes.push(pane);
        assert!(owner.pane_by_id(id).is_some());
        assert!(window_pane_find_by_id(id).is_some());
        let pane = window_panes_take(
            &mut owner.as_window_mut(),
            &crate::window::window_pane_find_by_id(id).expect("the pane exists"),
        )
        .unwrap();
        window_panes_insert_tail(&mut owner.as_window_mut(), pane);
        assert!(owner.pane_by_id(id).is_some());
        assert!(target.pane_ref().is_none());
    }
}

#[test]
fn focus_updates_emit_only_transitions_and_preserve_exited_panes() {
    use crate::tests::test_fixtures::{Clients, StreamBuffer};

    let _guard = globals();
    let mut target = Target::new(20, 6);
    let mut clients = Clients::new();
    let output = StreamBuffer::new();
    unsafe {
        let client = &mut *clients.add("focused", 20, 6);
        client.set_attached_session(Some(target.session_handle()));
        client.flags |= CLIENT_FOCUSED as uint64_t;
        crate::session::session_ref_of(&mut *target.session())
            .expect("session owner")
            .add_attached();
        let window = window_ref_of(&*target.window(0)).unwrap();
        let mut pane = window.as_window().panes[0].downgrade();
        {
            let wp = pane.get_mut().unwrap();
            *wp.event_mut() = output.ptr();
            let pane_screen_mode = wp.base().mode() | MODE_FOCUSON;
            wp.base_mut().set_mode(pane_screen_mode);
        }
        (window).update_focus();
        assert_ne!(*pane.get().unwrap().flags() & PANE_FOCUSED, 0);
        assert_eq!(output.written(), b"\x1b[I");
        (window).update_focus();
        assert!(output.written().is_empty());
        client.flags &= !(CLIENT_FOCUSED as uint64_t);
        (window).update_focus();
        assert_eq!(*pane.get().unwrap().flags() & PANE_FOCUSED, 0);
        assert_eq!(output.written(), b"\x1b[O");
        {
            let wp = pane.get_mut().unwrap();
            let pane_screen_mode = wp.base().mode() & !MODE_FOCUSON;
            wp.base_mut().set_mode(pane_screen_mode);
        }
        client.flags |= CLIENT_FOCUSED as uint64_t;
        (window).update_focus();
        assert_ne!(*pane.get().unwrap().flags() & PANE_FOCUSED, 0);
        assert!(output.written().is_empty());
        *pane.get_mut().unwrap().flags_mut() |= PANE_EXITED;
        client.flags &= !(CLIENT_FOCUSED as uint64_t);
        (window).update_focus();
        assert_ne!(*pane.get().unwrap().flags() & PANE_FOCUSED, 0);
        assert!(output.written().is_empty());
    }
}

#[test]
fn focus_requires_a_client_viewing_the_panes_current_window() {
    use crate::tests::test_fixtures::Clients;

    let _guard = globals();
    let mut target = Target::new(20, 6);
    let other = target.add_window(1, 20, 6);
    let mut clients = Clients::new();
    unsafe {
        let client = &mut *clients.add("focused", 20, 6);
        client.set_attached_session(Some(target.session_handle()));
        client.flags |= CLIENT_FOCUSED as uint64_t;
        crate::session::session_ref_of(&mut *target.session())
            .expect("session owner")
            .add_attached();
        let window = window_ref_of(&*target.window(other)).unwrap();
        let pane = window.as_window().panes[0].downgrade();
        (window).update_focus();
        assert_eq!(*pane.get().unwrap().flags() & PANE_FOCUSED, 0);
        (*target.session()).curw = Some(1);
        (window).update_focus();
        assert_ne!(*pane.get().unwrap().flags() & PANE_FOCUSED, 0);
        (*target.session()).curw = None;
        (window).update_focus();
        assert_eq!(*pane.get().unwrap().flags() & PANE_FOCUSED, 0);
        client.set_attached_session(None);
        (window).update_focus();
        assert_eq!(*pane.get().unwrap().flags() & PANE_FOCUSED, 0);
        window_update_focus(None);
    }
}

#[test]
fn client_colours_use_the_first_known_value_from_any_linked_window() {
    use crate::tests::test_fixtures::Clients;

    let _guard = globals();
    let mut target = Target::new(20, 6);
    target.add_window(1, 20, 6);
    let mut clients = Clients::new();
    unsafe {
        let detached = &mut *clients.add("detached", 20, 6);
        detached.tty.fg = 3;
        detached.tty.bg = 4;
        let first = &mut *clients.add("first", 20, 6);
        first.set_attached_session(Some(target.session_handle()));
        first.tty.fg = 1;
        first.tty.bg = -1;
        let second = &mut *clients.add("second", 20, 6);
        second.set_attached_session(Some(target.session_handle()));
        second.tty.fg = 2;
        second.tty.bg = 7;
        (*target.session()).curw = Some(1);
        let pane = &*target.pane(0);
        assert_eq!(window_pane_get_fg(pane), 1);
        assert_eq!(window_get_bg_client(pane), 7);
        first.flags |= CLIENT_SUSPENDED as uint64_t;
        assert_eq!(window_pane_get_fg(pane), 2);
        second.set_attached_session(None);
        assert_eq!(window_pane_get_fg(pane), -1);
        assert_eq!(window_get_bg_client(pane), -1);
    }
}

#[test]
fn pane_theme_uses_client_consensus_only_without_a_known_background() {
    use crate::tests::test_fixtures::Clients;

    let _guard = globals();
    let mut target = Target::new(20, 6);
    let mut clients = Clients::new();
    unsafe {
        let first = &mut *clients.add("light", 20, 6);
        first.set_attached_session(Some(target.session_handle()));
        first.theme = THEME_LIGHT;
        first.tty.bg = -1;
        let second = &mut *clients.add("unknown", 20, 6);
        second.set_attached_session(Some(target.session_handle()));
        second.theme = THEME_UNKNOWN;
        second.tty.bg = -1;
        let pane = &mut *target.pane(0);
        *pane.flags_mut() &= !PANE_STYLECHANGED;
        pane.set_styles(PaneStyleCells {
            cached_gc: grid_default_cell,
            cached_active_gc: grid_default_cell,
        });
        assert_eq!(window_pane_get_theme(Some(pane)), THEME_LIGHT);
        second.theme = THEME_DARK;
        assert_eq!(window_pane_get_theme(Some(pane)), THEME_UNKNOWN);
        first.flags |= CLIENT_SUSPENDED as uint64_t;
        assert_eq!(window_pane_get_theme(Some(pane)), THEME_DARK);
        let light = grid_cell {
            bg: 7,
            ..grid_default_cell
        };
        pane.set_styles(PaneStyleCells {
            cached_gc: light,
            cached_active_gc: light,
        });
        assert_eq!(window_pane_get_theme(Some(pane)), THEME_LIGHT);
        pane.set_styles(PaneStyleCells {
            cached_gc: grid_default_cell,
            cached_active_gc: grid_default_cell,
        });
        second.set_attached_session(None);
        assert_eq!(window_pane_get_theme(Some(pane)), THEME_UNKNOWN);
        assert_eq!(
            window_pane_get_theme(None::<&mut crate::types::window_pane>),
            THEME_UNKNOWN
        );
    }
}

#[test]
fn theme_notifications_follow_the_shown_screen_and_only_emit_changed_themes() {
    use crate::tests::test_fixtures::StreamBuffer;

    let _guard = globals();
    let mut target = Target::new(20, 6);
    let output = StreamBuffer::new();
    unsafe {
        let pane = &mut *target.pane(0);
        *pane.fd_mut() = 1000;
        *pane.event_mut() = output.ptr();
        let pane_screen_mode = pane.base().mode() | MODE_THEME_UPDATES;
        pane.base_mut().set_mode(pane_screen_mode);
        assert_eq!(
            window_pane_set_mode(pane, None, WindowMode::View, None, None),
            0
        );
        let shown = pane.modes()[0]
            .state
            .copy_mode_data_ref()
            .unwrap()
            .screen
            .clone();
        {
            let mut screen = shown.borrow_mut();
            let mode = screen.mode() & !MODE_THEME_UPDATES;
            screen.set_mode(mode);
        }
        *pane.flags_mut() = (*pane.flags() & !PANE_STYLECHANGED) | PANE_THEMECHANGED;
        let light = grid_cell {
            bg: 7,
            ..grid_default_cell
        };
        pane.set_styles(PaneStyleCells {
            cached_gc: light,
            cached_active_gc: light,
        });
        window_pane_send_theme_update(Some(pane));
        assert!(output.written().is_empty());
        assert_eq!(pane.theme(), THEME_UNKNOWN);
        assert_ne!(*pane.flags() & PANE_THEMECHANGED, 0);
        {
            let mut screen = shown.borrow_mut();
            let mode = screen.mode() | MODE_THEME_UPDATES;
            screen.set_mode(mode);
        }
        window_pane_send_theme_update(Some(pane));
        assert_eq!(output.written(), b"\x1b[?997;2n");
        assert_eq!(pane.theme(), THEME_LIGHT);
        assert_eq!(*pane.flags() & PANE_THEMECHANGED, 0);
        *pane.flags_mut() |= PANE_THEMECHANGED;
        window_pane_send_theme_update(Some(pane));
        assert!(output.written().is_empty());
        assert_ne!(*pane.flags() & PANE_THEMECHANGED, 0);
        let dark = grid_cell {
            bg: 0,
            ..grid_default_cell
        };
        pane.set_styles(PaneStyleCells {
            cached_gc: dark,
            cached_active_gc: dark,
        });
        window_pane_send_theme_update(Some(pane));
        assert_eq!(output.written(), b"\x1b[?997;1n");
        assert_eq!(pane.theme(), THEME_DARK);
        *pane.flags_mut() |= PANE_THEMECHANGED | PANE_EXITED;
        pane.set_styles(PaneStyleCells {
            cached_gc: light,
            cached_active_gc: light,
        });
        window_pane_send_theme_update(Some(pane));
        assert!(output.written().is_empty());
        assert_eq!(pane.theme(), THEME_DARK);
        *pane.fd_mut() = -1;
    }
}

#[test]
fn default_cursor_updates_borrow_each_modes_display_screen() {
    let _guard = globals();
    let mut target = Target::new(40, 12);
    unsafe {
        let state = target.state();
        let pane = &mut *target.pane(0);
        let options = pane.options_ref().clone();
        options.set_number(c"cursor-colour", 2);
        options.set_number(c"cursor-style", 3);
        window_pane_default_cursor(pane);
        assert_eq!(pane.base().default_cursor_colour(), 2);
        let base_style = pane.base().default_cursor_style();
        for mode in [
            WindowMode::Clock,
            WindowMode::Copy,
            WindowMode::View,
            WindowMode::Buffer,
            WindowMode::Client,
            WindowMode::Tree,
            WindowMode::Customize,
        ] {
            let source = pane.pane_id();
            assert_eq!(
                window_pane_set_mode(
                    pane,
                    crate::window::window_pane_find_by_id(source),
                    mode,
                    Some(&state),
                    None
                ),
                0
            );
            let backing_defaults = if matches!(mode, WindowMode::Copy | WindowMode::View) {
                pane.modes()[0]
                    .state
                    .copy_mode_data_ref()
                    .unwrap()
                    .backing
                    .as_ref()
                    .map(|screen| {
                        (
                            screen.default_cursor_colour(),
                            screen.default_cursor_style(),
                            screen.default_cursor_mode(),
                        )
                    })
            } else {
                None
            };
            options.set_number(c"cursor-colour", 5);
            options.set_number(c"cursor-style", 5);
            window_pane_default_cursor(pane);
            let shown = pane.screen_ref();
            assert_eq!(shown.default_cursor_colour(), 5);
            assert_eq!(shown.default_cursor_style(), SCREEN_CURSOR_BAR);
            assert_eq!(pane.base().default_cursor_colour(), 2);
            assert_eq!(pane.base().default_cursor_style(), base_style);
            if let Some(expected) = backing_defaults {
                let backing = pane.modes()[0]
                    .state
                    .copy_mode_data_ref()
                    .unwrap()
                    .backing
                    .as_ref()
                    .unwrap();
                assert_eq!(
                    (
                        backing.default_cursor_colour(),
                        backing.default_cursor_style(),
                        backing.default_cursor_mode()
                    ),
                    expected
                );
            }
            drop(shown);
            window_pane_reset_mode_all(pane);
            options.set_number(c"cursor-colour", 2);
            options.set_number(c"cursor-style", 3);
        }
        *pane.shown_mut() = PaneScreen::Mode;
        options.set_number(c"cursor-colour", 6);
        window_pane_default_cursor(pane);
        assert_eq!(pane.base().default_cursor_colour(), 6);
    }
}

#[test]
fn shown_mode_screen_reads_hold_the_shared_screen_borrow() {
    let _guard = globals();
    let mut target = Target::new(40, 12);
    let state = target.state();
    let mut pane = state.pane_ref().unwrap();
    unsafe {
        for mode in [
            WindowMode::Copy,
            WindowMode::View,
            WindowMode::Buffer,
            WindowMode::Client,
            WindowMode::Tree,
            WindowMode::Customize,
        ] {
            let source = (mode == WindowMode::Copy).then_some(pane.id());
            assert_eq!(
                window_pane_set_mode(
                    pane.get_mut().unwrap(),
                    source.and_then(crate::window::window_pane_find_by_id),
                    mode,
                    Some(&state),
                    None
                ),
                0
            );
            let entry = window_pane_current_mode(pane.get().unwrap()).unwrap();
            let display = match &entry.state {
                WindowModeState::Copy(data) | WindowModeState::View(data) => data.screen.clone(),
                _ => entry
                    .screen.as_ref().unwrap().shared().unwrap()
                    .clone(),
            };
            let shown = pane.get().unwrap().screen_ref();
            assert!(core::ptr::eq(&*shown, &*display.borrow()));
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    display.borrow_mut().set_cursor(1, 1);
                }))
                .is_err()
            );
            drop(shown);
            display.borrow_mut().set_cursor(1, 1);
            assert_eq!(pane.get().unwrap().screen_ref().cursor(), (1, 1));
            window_pane_reset_mode_all(pane.get_mut().unwrap());
        }
    }
}

#[test]
fn floating_checks_and_counts_read_the_supplied_layout_without_pane_back_references() {
    let mut w = window::default();
    for id in [3, 7] {
        let mut pane = Box::new(window_pane::default());
        pane.set_pane_id(id);
        w.panes.push(RustWindowPaneRef::from_pane(pane));
    }
    w.layout_root = Some(Box::new(layout_cell {
        cells: vec![
            Box::new(layout_cell {
                wp: Some(w.panes[0].downgrade()),
                flags: LAYOUT_CELL_FLOATING,
                ..Default::default()
            }),
            Box::new(layout_cell {
                wp: Some(w.panes[1].downgrade()),
                ..Default::default()
            }),
        ],
        ..Default::default()
    }));
    assert_eq!(window_pane_is_floating(&w, &w.panes[0].downgrade()), 1);
    assert_eq!(window_pane_is_floating(&w, &w.panes[1].downgrade()), 0);
    assert_eq!(
        window_pane_is_floating(
            &w,
            &RustWindowPaneRef::from_pane(Box::new(window_pane::default())).downgrade()
        ),
        0
    );
    assert_eq!(window_has_floating_panes(&w), 1);
    assert_eq!(window_count_panes(&w, 0), 1);
    assert_eq!(window_count_panes(&w, 1), 2);
    let mut other = window::default();
    assert_eq!(window_pane_is_floating(&other, &w.panes[0].downgrade()), 0);
    other.layout_root = w.layout_root.take();
    assert_eq!(window_pane_is_floating(&other, &w.panes[0].downgrade()), 1);
    assert_eq!(window_has_floating_panes(&w), 0);
    assert_eq!(window_count_panes(&w, 0), 2);
}

#[test]
fn pane_stacking_and_flags_follow_physical_owners_and_skip_retired_targets() {
    let _guard = globals();
    let mut target = Target::new(40, 12);
    let pane = target.state().pane_ref().unwrap();
    let window = pane.window().unwrap();
    unsafe {
        let mut payload = Box::new(window_pane::default());
        payload.set_pane_id(99);
        *payload.options_mut() = Some(pane.as_pane().options_ref().clone());
        *payload.flags_mut() = PANE_ZOOMED;
        let listed = RustWindowPaneRef::from_pane(payload);
        let observed = listed.downgrade();
        window_panes_insert_tail(&mut window.as_window_mut(), listed);
        let listed = observed;
        window.as_window_mut().layout_root = Some(Box::new(layout_cell {
            wp: Some(pane.clone()),
            flags: LAYOUT_CELL_FLOATING,
            ..Default::default()
        }));
        window.as_window_mut().z_index = vec![pane.clone(), listed.clone()];
        {
            let mut payload = window.as_window_mut();
            payload.active = payload
                .panes
                .iter()
                .find(|pane| pane.pane_id() == listed.id())
                .map(|pane| pane.downgrade());
        }
        window.as_window_mut().last_panes = vec![pane.clone()];
        assert!(window_pane_find_by_id(listed.id()).is_none());
        assert_eq!(window_pane_zindex(&pane), (0, 0));
        assert_eq!(window_pane_zindex(&listed), (0, 2));
        assert_eq!(window_pane_printable_flags(&pane).as_deref(), Some(c"-F"));
        assert_eq!(window_pane_printable_flags(&listed).as_deref(), Some(c"*Z"));
        window.as_window_mut().z_index = vec![listed.clone(), pane.clone()];
        assert_eq!(window_pane_zindex(&listed), (0, 1));
        assert_eq!(window_pane_zindex(&pane), (0, 0));
        let mut impostor = Box::new(window_pane::default());
        impostor.set_pane_id(listed.id());
        impostor.set_window_context(Some(&window));
        let impostor = RustWindowPaneRef::from_pane(impostor);
        assert_eq!(window_pane_zindex(&impostor.downgrade()), (-1, 1));
        assert_eq!(
            window_pane_printable_flags(&impostor.downgrade()).as_deref(),
            Some(c"")
        );
        window.as_window_mut().z_index = vec![
            RustWindowPaneRef::from_pane(Box::new(window_pane::default())).downgrade(),
            pane.clone(),
            listed.clone(),
        ];
        assert_eq!(window_pane_zindex(&pane), (-1, 0));
        window.as_window_mut().z_index = vec![listed.clone(), pane.clone()];
        window.remove_pane(
            &crate::window::window_pane_find_by_id(pane.id()).expect("the pane exists"),
        );
        assert_eq!(window_pane_zindex(&pane), (-1, 0));
        assert!(window_pane_printable_flags(&pane).is_none());
        drop(target);
    }
}

#[test]
fn terminal_resize_uses_the_panes_current_window_pixels_and_skips_missing_owners() {
    let _guard = globals();
    let mut target = Target::new(40, 12);
    target.add_window(1, 40, 12);
    let mut pane = target.state().pane_ref().unwrap();
    let original = pane.window().unwrap();
    unsafe {
        let destination = window_ref_of(&*target.window(1)).unwrap();
        original.set_pixels(WindowPixelSize {
            width: 8,
            height: 16,
        });
        destination.set_pixels(WindowPixelSize {
            width: 10,
            height: 20,
        });
        let mut master = -1;
        let mut slave = -1;
        assert_eq!(
            libc::openpty(
                &mut master,
                &mut slave,
                core::ptr::null_mut(),
                core::ptr::null(),
                core::ptr::null()
            ),
            0
        );
        let master = OwnedFd::from_raw_fd(master);
        let slave = OwnedFd::from_raw_fd(slave);
        let pane_fd = libc::dup(master.as_raw_fd());
        assert!(pane_fd >= 0);
        *pane.as_pane_mut().fd_mut() = pane_fd;
        let size = || {
            let mut size = libc::winsize {
                ws_col: 0,
                ws_row: 0,
                ws_xpixel: 0,
                ws_ypixel: 0,
            };
            assert_eq!(
                libc::ioctl(slave.as_raw_fd(), libc::TIOCGWINSZ, &mut size),
                0
            );
            (size.ws_col, size.ws_row, size.ws_xpixel, size.ws_ypixel)
        };
        window_pane_send_resize(&pane, 10, 4);
        assert_eq!(size(), (10, 4, 80, 64));
        let transferred = window_panes_take(
            &mut original.as_window_mut(),
            &crate::window::window_pane_find_by_id(pane.id()).expect("the pane exists"),
        )
        .unwrap();
        window_pane_send_resize(&pane, 11, 5);
        assert_eq!(size(), (10, 4, 80, 64));
        window_panes_insert_tail(&mut destination.as_window_mut(), transferred);
        window_pane_send_resize(&pane, 11, 5);
        assert_eq!(size(), (11, 5, 110, 100));
        destination.remove_pane(
            &crate::window::window_pane_find_by_id(pane.id()).expect("the pane exists"),
        );
        window_pane_send_resize(&pane, 12, 6);
        assert_eq!(size(), (11, 5, 110, 100));
        drop(target);
    }
}

#[test]
fn pane_indices_read_the_supplied_list_and_options_without_rebinding_duplicate_ids() {
    let _guard = globals();
    let mut target = Target::new(40, 12);
    let pane = target.state().pane_ref().unwrap();
    let original = pane.window().unwrap();
    unsafe {
        let options = original.options();
        let mut supplied = window::default();
        supplied.options = Some(options.clone());
        let mut unregistered = Box::new(window_pane::default());
        unregistered.set_pane_id(99);
        options.set_number(c"pane-base-index", 10);
        assert_eq!(
            window_pane_index(&original.as_window(), pane.as_pane()),
            (0, 10)
        );
        let owned = original.as_window_mut().panes.remove(0);
        supplied.panes = vec![RustWindowPaneRef::from_pane(unregistered), owned];
        assert_eq!(window_pane_index(&supplied, pane.as_pane()), (0, 11));
        assert_eq!(
            window_pane_index(&supplied, supplied.panes[0].as_pane()),
            (0, 10)
        );
        let mut duplicate = window_pane::default();
        duplicate.set_pane_id(pane.id());
        assert_eq!(window_pane_index(&supplied, &duplicate), (-1, 12));
        options.set_number(c"pane-base-index", -1);
        assert_eq!(window_pane_index(&supplied, pane.as_pane()), (0, 0));
        assert_eq!(window_pane_index(&supplied, &duplicate), (-1, 1));
        original
            .as_window_mut()
            .panes
            .push(supplied.panes.remove(1));
    }
}

#[test]
fn visibility_uses_the_supplied_zoom_state_and_physical_pane_identity() {
    let mut w = window::default();
    for id in [3, 7] {
        let mut pane = Box::new(window_pane::default());
        pane.set_pane_id(id);
        w.panes.push(RustWindowPaneRef::from_pane(pane));
    }
    let first = w.panes[0].downgrade();
    let second = w.panes[1].downgrade();
    unsafe {
        assert_eq!(window_pane_visible(&w, first.as_pane()), 1);
        assert_eq!(window_pane_visible(&w, second.as_pane()), 1);
        w.flags |= WINDOW_ZOOMED;
        w.active = w
            .panes
            .iter()
            .find(|pane| pane.pane_id() == first.id())
            .map(|pane| pane.downgrade());
        assert_eq!(window_pane_visible(&w, first.as_pane()), 1);
        assert_eq!(window_pane_visible(&w, second.as_pane()), 0);
        let mut duplicate = window_pane::default();
        duplicate.set_pane_id(first.id());
        assert_eq!(window_pane_visible(&w, &duplicate), 0);
        w.active = w
            .panes
            .iter()
            .find(|pane| pane.pane_id() == second.id())
            .map(|pane| pane.downgrade());
        assert_eq!(window_pane_visible(&w, first.as_pane()), 0);
        assert_eq!(window_pane_visible(&w, second.as_pane()), 1);
        w.active = w
            .panes
            .iter()
            .find(|pane| pane.pane_id() == 99)
            .map(|pane| pane.downgrade());
        assert_eq!(window_pane_visible(&w, first.as_pane()), 0);
        w.flags &= !WINDOW_ZOOMED;
        assert_eq!(window_pane_visible(&w, first.as_pane()), 1);
    }
}

#[test]
fn a_recorded_window_context_expires_when_its_last_owner_drops() {
    let _guard = globals();
    let window = WindowRef::new(window::default());
    let weak = window.downgrade();
    let mut pane = window_pane::default();
    window_pane_set_window_ref(&mut pane, Some(&window));
    let retained = pane.window_context().unwrap();
    assert!(retained.ptr_eq(&window));
    drop(window);
    assert!(weak.upgrade().is_some());
    drop(retained);
    assert!(weak.upgrade().is_none());
    assert!(pane.window_context().is_none());
}

#[test]
fn a_stream_error_callback_does_not_delay_pane_destruction() {
    let _guard = globals();
    let window = WindowRef::create(20, 8, 0, 0);
    unsafe {
        let id = window_add_pane(&mut window.as_window_mut(), None, 0, 0).id();
        let observed = window_pane_find_by_id(id).unwrap();
        let callback_window = window.clone();
        let callback = on_pane_error(id, move |pane| {
            let window = callback_window.clone();
            window.remove_pane(
                &crate::window::window_pane_find_by_id(pane.pane_id()).expect("the pane exists"),
            );
        });
        callback(Stream::NONE, 0);
        assert!(!observed.is_alive());
        assert!(observed.upgrade().is_none());
        assert!(observed.get().is_none());
        assert!(window.as_window().panes.is_empty());
        callback(Stream::NONE, 0);
    }
}
