use super::*;
use crate::server::{server_clear_marked, server_set_marked};
use crate::tests::test_fixtures::{Clients, Item, Target, globals};

unsafe fn resolve(
    target: &mut Target,
    text: &CStr,
    kind: cmd_find_type,
    flags: core::ffi::c_int,
) -> (core::ffi::c_int, cmd_find_state) {
    unsafe {
        let mut item = Item::new().targeting(target);
        let mut fs = cmd_find_state::default();
        let result = cmd_find_target(&mut fs, &mut *item.ptr(), Some(text), kind, flags);
        (result, fs)
    }
}

#[test]
fn compound_targets_resolve_session_window_and_pane_components() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    target.add_window(3, 80, 24);
    unsafe {
        let (rc, fs) = resolve(&mut target, c"0:3.0", CMD_FIND_PANE, CMD_FIND_QUIET);
        assert_eq!(rc, 0);
        assert_eq!(fs.session().unwrap().as_ptr(), target.session());
        assert!(core::ptr::eq(
            fs.winlink_ref().unwrap().get().unwrap(),
            target.winlink(1)
        ));
        assert!(core::ptr::addr_eq(
            fs.pane_list_ref().unwrap().get().unwrap(),
            target.pane(1)
        ));

        let (rc, fs) = resolve(&mut target, c"0:3", CMD_FIND_WINDOW, CMD_FIND_QUIET);
        assert_eq!(rc, 0);
        assert!(core::ptr::eq(
            fs.winlink_ref().unwrap().get().unwrap(),
            target.winlink(1)
        ));
        let (rc, fs) = resolve(&mut target, c"3.0", CMD_FIND_PANE, CMD_FIND_QUIET);
        assert_eq!(rc, 0);
        assert!(core::ptr::addr_eq(
            fs.pane_list_ref().unwrap().get().unwrap(),
            target.pane(1)
        ));
        let (rc, _) = resolve(&mut target, c"0:99.99", CMD_FIND_PANE, CMD_FIND_CANFAIL);
        assert_eq!(rc, 0);
    }
}

#[test]
fn aliases_empty_and_exact_flags_cover_current_and_failure_paths() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    unsafe {
        for alias in [c"" as &CStr, c"{last}", c"{start}"] {
            let (rc, fs) = resolve(&mut target, alias, CMD_FIND_PANE, CMD_FIND_QUIET);
            if rc == 0 {
                assert!(core::ptr::addr_eq(
                    fs.pane_list_ref().unwrap().get().unwrap(),
                    target.pane(0)
                ));
            }
        }
        let (rc, _) = resolve(
            &mut target,
            c"does-not-exist",
            CMD_FIND_SESSION,
            CMD_FIND_EXACT_SESSION | CMD_FIND_QUIET,
        );
        assert_eq!(rc, -1);
        let (rc, _) = resolve(
            &mut target,
            c"target",
            CMD_FIND_WINDOW,
            CMD_FIND_EXACT_WINDOW | CMD_FIND_QUIET,
        );
        assert_eq!(rc, 0);
        let (rc, _) = resolve(
            &mut target,
            c"999",
            CMD_FIND_WINDOW,
            CMD_FIND_WINDOW_INDEX | CMD_FIND_QUIET,
        );
        assert_eq!(rc, 0);
    }
}

#[test]
fn marked_target_and_default_marked_select_the_registered_pane() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    unsafe {
        server_set_marked(
            Some(&*target.session()),
            Some(&*target.winlink(0)),
            Some(&*target.pane(0)),
        );
        for text in [c"~" as &CStr, c"{marked}"] {
            let (rc, fs) = resolve(&mut target, text, CMD_FIND_PANE, CMD_FIND_QUIET);
            assert_eq!(rc, 0);
            assert!(core::ptr::addr_eq(
                fs.pane_list_ref().unwrap().get().unwrap(),
                target.pane(0)
            ));
        }
        let mut item = Item::new().targeting(&mut target);
        let mut fs = cmd_find_state::default();
        assert_eq!(
            cmd_find_target(
                &mut fs,
                &mut *item.ptr(),
                None,
                CMD_FIND_PANE,
                CMD_FIND_DEFAULT_MARKED
            ),
            0
        );
        assert!(core::ptr::addr_eq(
            fs.pane_list_ref().unwrap().get().unwrap(),
            target.pane(0)
        ));
        server_clear_marked();
        let (rc, _) = resolve(&mut target, c"~", CMD_FIND_PANE, CMD_FIND_QUIET);
        assert_eq!(rc, -1);
    }
}

#[test]
fn client_selection_covers_exact_prefix_current_and_best_session_paths() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    let mut clients = Clients::new();
    unsafe {
        let alpha = clients.add("alpha-client", 80, 24);
        (*alpha).set_attached_session(Some(target.session_handle()));
        (*alpha).ttyname = Some(std::ffi::CString::new("").unwrap());
        let beta = clients.add("beta-client", 80, 24);
        (*beta).set_attached_session(Some(target.session_handle()));
        (*beta).ttyname = Some(std::ffi::CString::new("").unwrap());
        assert!(
            cmd_find_best_client(&*target.session())
                .is_some_and(|client| core::ptr::eq(client.as_ptr(), alpha))
        );
        assert!(
            cmd_find_client(None, Some(c"alpha-client"), 1)
                .is_some_and(|client| core::ptr::eq(client.as_ptr(), alpha))
        );
        assert!(cmd_find_client(None, Some(c"alpha"), 1).is_none());
        assert!(cmd_find_client(None, Some(c"missing"), 1).is_none());
        let mut fs = cmd_find_state::default();
        assert_eq!(
            cmd_find_from_client(&mut fs, crate::server::client_ref_of(&*alpha).as_ref(), 0),
            0
        );
        assert!(core::ptr::addr_eq(
            fs.pane_list_ref().unwrap().get().unwrap(),
            target.pane(0)
        ));
    }
}

#[test]
fn last_pane_target_only_resolves_the_first_history_reference() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    let _ = target.add_window(1, 80, 24);
    unsafe {
        let mut fs = target.state();
        let pane = fs.pane_ref().unwrap();
        let window = fs.window().unwrap();
        window.set_pane_history_for_test(vec![pane.clone()]);
        assert_eq!(cmd_find_get_pane_with_window(&mut fs, c"!"), 0);
        assert!(fs.pane_ref().unwrap().ptr_eq(&pane));
        for history in [vec![u_int::MAX, pane.id()], Vec::new()] {
            window.set_pane_history_for_test(
                history
                    .into_iter()
                    .map(|id| {
                        crate::window::window_pane_find_by_id(id).unwrap_or_else(|| {
                            (crate::tests::test_fixtures::PaneAllocation::default()).into_owner()
                                .downgrade()
                        })
                    })
                    .collect(),
            );
            assert_eq!(cmd_find_get_pane_with_window(&mut fs, c"!"), -1);
            assert!(fs.pane_ref().is_none());
        }
    }
}

#[test]
fn state_validation_requires_a_live_link_and_its_own_pane() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    target.add_window(1, 80, 24);
    unsafe {
        let mut fs = target.state();
        assert_eq!(cmd_find_valid_state(&fs), 1);
        let session = fs.session().unwrap();
        let other_window = session
            .as_session()
            .windows
            .get(&1)
            .unwrap()
            .window_handle()
            .unwrap()
            .clone();
        fs.wl = Some(1);
        assert_eq!(cmd_find_valid_state(&fs), 0);
        fs.set_window_ref(Some(&other_window));
        assert_eq!(cmd_find_valid_state(&fs), 0);
        fs.wp = Some(other_window.panes()[0].clone());
        assert_eq!(cmd_find_valid_state(&fs), 1);
        fs.wp = None;
        assert_eq!(cmd_find_valid_state(&fs), 0);
        fs.wp = Some(other_window.panes()[0].clone());
        fs.wl = Some(99);
        assert_eq!(cmd_find_valid_state(&fs), 0);
        fs.wl = Some(1);
        fs.set_session_ref(None);
        assert_eq!(cmd_find_valid_state(&fs), 0);
    }
}

#[test]
fn compound_targets_preserve_session_dots_window_colons_and_empty_components() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    unsafe {
        let state = target.state();
        let session = state.session().unwrap();
        session.rename(c"work.main".to_owned());
        let window = state.window().unwrap();
        window.set_window_name(Some(c"notes:extra"));
        let pane = state.pane_ref().unwrap();
        for text in [
            c"work.main:notes:extra.0",
            c"=work.main:=notes:extra.0",
            c":notes:extra.0",
            c"work.main:.0",
            c"work.main:notes:extra.",
            c"=:=.0",
            c"work.main:.",
            c":.",
        ] {
            let (rc, found) = resolve(&mut target, text, CMD_FIND_PANE, CMD_FIND_QUIET);
            assert_eq!(rc, 0, "{text:?}");
            assert!(found.pane_ref().unwrap().ptr_eq(&pane), "{text:?}");
        }
        let (rc, _) = resolve(
            &mut target,
            c"work.main:notes:extra.0.more",
            CMD_FIND_PANE,
            CMD_FIND_QUIET,
        );
        assert_eq!(rc, -1);
    }
}

#[test]
fn window_names_prefer_exact_matches_and_reject_ambiguous_prefixes_and_patterns() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    target.add_window(3, 80, 24);
    target.add_window(7, 80, 24);
    unsafe {
        let mut windows: Vec<_> = target
            .state()
            .session()
            .unwrap()
            .as_session()
            .windows
            .values()
            .map(|link| link.window_handle().unwrap().clone())
            .collect();
        for (window, name) in windows.iter_mut().zip([c"alpha", c"alphabet", c"beta"]) {
            window.set_window_name(Some(name));
        }
        for (text, expected) in [
            (c"alpha", 0),
            (c"=alpha", 0),
            (c"alph?", 0),
            (c"bet", 2),
            (c"b?ta", 2),
        ] {
            let (rc, found) = resolve(&mut target, text, CMD_FIND_WINDOW, CMD_FIND_QUIET);
            assert_eq!(rc, 0, "{text:?}");
            assert!(
                found.window().unwrap().ptr_eq(&windows[expected]),
                "{text:?}"
            );
        }
        for text in [c"alph", c"a*", c"=alph"] {
            let (rc, _) = resolve(&mut target, text, CMD_FIND_WINDOW, CMD_FIND_QUIET);
            assert_eq!(rc, -1, "{text:?}");
        }
        windows[1].set_window_name(Some(c"alpha"));
        let (rc, _) = resolve(&mut target, c"alpha", CMD_FIND_WINDOW, CMD_FIND_QUIET);
        assert_eq!(rc, -1);
    }
}

#[test]
fn window_targets_preserve_aliases_wraparound_and_unallocated_indices() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    target.add_window(3, 80, 24);
    target.add_window(7, 80, 24);
    unsafe {
        let mut session = target.state().session().unwrap();
        session.as_session_mut().curw = Some(3);
        session.as_session_mut().lastw = vec![0];
        for (text, index) in [
            (c"!", 0),
            (c"^", 0),
            (c":$", 7),
            (c"+", 7),
            (c"-", 0),
            (c"+2", 0),
            (c"-2", 7),
            (c"+bogus", 3),
        ] {
            let (rc, found) = resolve(&mut target, text, CMD_FIND_WINDOW, CMD_FIND_QUIET);
            assert_eq!(rc, 0, "{text:?}");
            assert_eq!(found.winlink_ref().unwrap().index(), index, "{text:?}");
        }
        let flags = CMD_FIND_QUIET | CMD_FIND_WINDOW_INDEX;
        for (text, index, link) in [(c"5", 5, None), (c"+2", 5, Some(3)), (c"-2", 1, Some(3))] {
            let (rc, found) = resolve(&mut target, text, CMD_FIND_WINDOW, flags);
            assert_eq!(rc, 0, "{text:?}");
            assert_eq!(found.idx, index, "{text:?}");
            assert_eq!(
                found.winlink_ref().map(|link| link.index()),
                link,
                "{text:?}"
            );
        }
        let (rc, _) = resolve(&mut target, c"-4", CMD_FIND_WINDOW, flags);
        assert_eq!(rc, -1);
    }
}

#[test]
fn best_window_link_prefers_current_then_lowest_matching_index() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    target.add_window(3, 80, 24);
    target.add_window(7, 80, 24);
    unsafe {
        let mut state = target.state();
        let window = state.window().unwrap();
        let mut session = state.session().unwrap();
        let mut other = crate::window::WinlinkRef::new(session.clone(), 7).unwrap();
        let unlinked = other.get().unwrap().window_handle().unwrap().clone();
        other.set_window(window).unwrap();
        session.as_session_mut().curw = Some(7);
        assert_eq!(cmd_find_best_winlink_with_window(&mut state), 0);
        assert_eq!(state.idx, 7);
        session.as_session_mut().curw = Some(3);
        assert_eq!(cmd_find_best_winlink_with_window(&mut state), 0);
        assert_eq!(state.idx, 0);
        state.set_window_ref(Some(&unlinked));
        assert_eq!(cmd_find_best_winlink_with_window(&mut state), -1);
        assert_eq!(state.idx, 0);
    }
}

#[test]
fn target_state_copy_preserves_list_membership_and_clears_stale_observations() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    unsafe {
        let mut source = target.state();
        let window = source.window().unwrap();
        let id = source.wp.as_ref().map(|pane| pane.id()).unwrap();
        window.clear_pane_membership_for_test(source.wp.as_ref().unwrap());
        assert_eq!(source.pane_ref().unwrap().id(), id);
        assert_eq!(source.pane_list_ref().unwrap().id(), id);
        let mut copied = cmd_find_state {
            flags: 0x99,
            ..Default::default()
        };
        cmd_find_copy_state(&mut copied, &source);
        assert_eq!(copied.wp.as_ref().map(|pane| pane.id()), Some(id));
        assert_eq!(copied.flags, 0x99);
        {
            let session = source.session().unwrap();
            let mut constructed = cmd_find_state::default();
            cmd_find_from_session(&mut constructed, session.as_session(), 0x11);
            assert_eq!(constructed.wp.as_ref().map(|pane| pane.id()), Some(id));
            assert_eq!(constructed.idx, -1);
            assert_eq!(constructed.flags, 0x11);
            window.set_active_pane_id_for_test(u32::MAX);
            cmd_find_from_session(&mut constructed, session.as_session(), 0x11);
            assert_eq!(constructed.wp.as_ref().map(|pane| pane.id()), None);
            window.set_active_pane_id_for_test(id);
        }
        for (text, index) in [(c"0:", -1), (c"0:0", 0), (c":0", 0)] {
            let (rc, found) = resolve(&mut target, text, CMD_FIND_PANE, CMD_FIND_QUIET);
            assert_eq!(rc, 0, "{text:?}");
            assert_eq!(found.idx, index, "{text:?}");
            assert_eq!(found.pane_list_ref().unwrap().id(), id, "{text:?}");
            assert_eq!(found.pane_ref().unwrap().id(), id, "{text:?}");
        }
        source.wl = Some(999);
        source.wp = None;
        cmd_find_copy_state(&mut copied, &source);
        assert_eq!(copied.wl, None);
        assert_eq!(copied.wp.as_ref().map(|pane| pane.id()), None);
        drop(window);
        drop(target);
        cmd_find_copy_state(&mut copied, &source);
        assert_eq!(cmd_find_empty_state(&copied), 1);
        assert_eq!(copied.flags, 0x99);
    }
}

#[test]
fn session_names_prefer_exact_matches_and_reject_ambiguous_prefixes_and_patterns() {
    let _guard = globals();
    let mut registry = crate::tests::test_fixtures::Registry::new();
    let mut sessions = [
        crate::tests::test_fixtures::Session::new(0, "alpha"),
        crate::tests::test_fixtures::Session::new(1, "alphabet"),
        crate::tests::test_fixtures::Session::new(2, "beta"),
    ];
    for session in &mut sessions {
        registry.add_session(session);
    }
    for (text, expected) in [(c"alpha", 0), (c"alph?", 0), (c"bet", 2), (c"b?ta", 2)] {
        let mut found = cmd_find_state::default();
        assert_eq!(
            unsafe { cmd_find_get_session(&mut found, text) },
            0,
            "{text:?}"
        );
        assert!(
            found.session().unwrap().ptr_eq(sessions[expected].handle()),
            "{text:?}"
        );
    }
    for (text, flags) in [
        (c"alph", 0),
        (c"a*", 0),
        (c"missing", 0),
        (c"bet", CMD_FIND_EXACT_SESSION),
    ] {
        let mut found = cmd_find_state {
            flags,
            ..Default::default()
        };
        assert_eq!(
            unsafe { cmd_find_get_session(&mut found, text) },
            -1,
            "{text:?}"
        );
        assert!(found.session().is_none(), "{text:?}");
    }
}

#[test]
fn detached_client_targets_the_sessions_current_window_after_finding_its_origin_pane() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    let mut clients = Clients::new();
    unsafe {
        let state = target.state();
        let origin = state.pane_ref().unwrap();
        let mut session = state.session().unwrap();
        target.add_window(3, 80, 24);
        session.as_session_mut().curw = Some(3);
        let current = target.state().pane_ref().unwrap();
        let client = clients.add("detached-client", 80, 24);
        (*client).environ = Some(crate::environ::new_environment_box());
        (*client).environ_mut().set(
            c"TMUX_PANE",
            0,
            &CString::new(format!("%{}", origin.id())).unwrap(),
        );
        let mut found = cmd_find_state::default();
        assert_eq!(
            cmd_find_from_client(
                &mut found,
                crate::server::client_ref_of(&*client).as_ref(),
                CMD_FIND_QUIET
            ),
            0
        );
        assert!(found.pane_ref().unwrap().ptr_eq(&current));
        assert_eq!(found.winlink_ref().unwrap().index(), 3);
        assert_eq!(found.idx, 0);
        assert_eq!(found.flags, CMD_FIND_QUIET);
    }
}

#[test]
fn explicit_pane_ids_preserve_physical_membership_and_reject_other_windows() {
    let _guard = globals();
    let mut target = Target::new(20, 6);
    target.add_window(1, 20, 6);
    unsafe {
        let source = target.state();
        let pane = source.pane_list_ref().unwrap();
        let window = source.window().unwrap();
        let text = CString::new(format!("%{}", pane.id())).unwrap();
        let other = crate::window::window_ref_of(&*target.window(1)).unwrap();
        for physical_only in [false, true] {
            if physical_only {
                window.clear_pane_membership_for_test(&pane);
                assert!(source.pane_ref().is_some());
            }
            let mut found = cmd_find_state::default();
            assert_eq!(cmd_find_get_pane(&source, &mut found, &text, 0), 0);
            assert!(found.pane_list_ref().unwrap().ptr_eq(&pane));
            assert!(found.window().unwrap().ptr_eq(&window));
            let mut found = cmd_find_state::default();
            found.set_session_ref(source.session().as_ref());
            assert_eq!(cmd_find_get_pane_with_session(&mut found, &text), 0);
            assert!(found.pane_list_ref().unwrap().ptr_eq(&pane));
            let mut found = source.clone();
            assert_eq!(cmd_find_get_pane_with_window(&mut found, &text), 0);
            assert!(found.pane_list_ref().unwrap().ptr_eq(&pane));
            found.set_window_ref(Some(&other));
            assert_eq!(cmd_find_get_pane_with_window(&mut found, &text), -1);
        }
        for text in [c"%99999", c"%invalid"] {
            let mut found = cmd_find_state::default();
            assert_eq!(cmd_find_get_pane(&source, &mut found, text, 0), -1);
            assert!(found.pane_list_ref().is_none());
        }
    }
}
