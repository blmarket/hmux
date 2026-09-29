use hmux2::src::cmd::queue::{cmdq_append, cmdq_get_callback_owned, cmdq_new, cmdq_next};
use hmux2::src::cmd::queue::{cmdq_get_event, cmdq_get_state_owned, cmdq_new_state};
use hmux2::src::shared::client::client;
use hmux2::src::shared::key::key_event;
use std::rc::Rc;

#[test]
fn event_snapshots_observe_clients_and_never_redirect_expired_targets() {
    use hmux2::src::shared::client::client;
    use hmux2::src::shared::window::window;
    unsafe {
        let explicit = client::new();
        let fallback = client::new();
        let mut event = key_event::new(1, Default::default(), None);
        assert!(Rc::ptr_eq(
            &event.resolve_client(|| Some(fallback.clone())).unwrap(),
            &fallback
        ));
        event.client = Rc::downgrade(&explicit);
        let snapshot = event.metadata_snapshot();
        assert_eq!(Rc::strong_count(&explicit), 1, "snapshots are observers");
        let resolved = snapshot
            .resolve_client(|| panic!("explicit target must win"))
            .unwrap();
        assert!(Rc::ptr_eq(&resolved, &explicit));
        let mut window = window::default();
        window.latest = Rc::downgrade(&explicit);
        assert!(!window.latest.ptr_eq(&Rc::downgrade(&fallback)));
        drop(explicit);
        assert!(window.latest.upgrade().is_some(), "dispatch retains client");
        drop(resolved);
        assert!(window.latest.upgrade().is_none());
        assert!(snapshot
            .resolve_client(|| panic!("expired target must not fall back"))
            .is_none());
        assert!(window.latest.ptr_eq(&snapshot.client));
        drop(fallback);
    }
}
