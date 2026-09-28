use hmux2::src::cmd::queue::{cmdq_get_event, cmdq_get_state_owned, cmdq_new_state};
use hmux2::src::shared::command::cmdq_item;
use hmux2::src::shared::key::key_event;
use std::rc::Rc;

#[test]
fn shared_current_is_checked_and_event_snapshots_outlive_queue_items() {
    unsafe {
        let mut event = key_event::new(42, Default::default(), Some(vec![1, 2, 3]));
        event.m.valid = 1;
        let state = cmdq_new_state(std::ptr::null_mut(), &mut *event, 7);
        let observed = Rc::downgrade(&state);
        let mut item = cmdq_item::empty();
        item.state = Some(state.clone());
        let retained = cmdq_get_state_owned(&mut item);
        assert!(Rc::ptr_eq(&retained, &state));
        {
            let mut target = retained.current.borrow_mut();
            target.idx = 19;
            assert!(state.current.try_borrow().is_err());
            // Other state fields remain accessible without a whole-state borrow.
            let snapshot = cmdq_get_event(&mut item);
            assert_eq!(snapshot.key, 42);
            assert!(snapshot.bytes.is_none());
        }
        assert_eq!(state.current.borrow().idx, 19);
        let saved = state.current_snapshot();
        state.current.borrow_mut().idx = 23;
        assert_eq!(saved.idx, 19);
        let mut snapshot = cmdq_get_event(&mut item);
        snapshot.m.valid = 0;
        assert_eq!(cmdq_get_event(&mut item).m.valid, 1);
        drop(state);
        drop(item.state.take());
        assert!(observed.upgrade().is_some());
        assert_eq!(retained.flags, 7);
        drop(retained);
        assert!(observed.upgrade().is_none());
        assert_eq!(snapshot.key, 42);
    }
}

#[test]
fn event_snapshots_observe_clients_and_never_redirect_expired_targets() {
    use hmux2::src::shared::client::client;
    use hmux2::src::shared::window::window;
    unsafe {
        let explicit = client::new();
        let fallback = client::new();
        let mut event = key_event::new(1, Default::default(), None);
        assert!(Rc::ptr_eq(&event.resolve_client(|| Some(fallback.clone())).unwrap(), &fallback));
        event.client = Some(Rc::downgrade(&explicit));
        let snapshot = event.metadata_snapshot();
        assert_eq!(Rc::strong_count(&explicit), 1, "snapshots are observers");
        let resolved = snapshot.resolve_client(|| panic!("explicit target must win")).unwrap();
        assert!(Rc::ptr_eq(&resolved, &explicit));
        let mut window = window::default();
        window.latest = Rc::downgrade(&explicit);
        assert!(!window.latest.ptr_eq(&Rc::downgrade(&fallback)));
        drop(explicit);
        assert!(window.latest.upgrade().is_some(), "dispatch retains client");
        drop(resolved);
        assert!(window.latest.upgrade().is_none());
        assert!(snapshot.resolve_client(|| panic!("expired target must not fall back")).is_none());
        assert!(window.latest.ptr_eq(snapshot.client.as_ref().unwrap()));
        drop(fallback);
    }
}
