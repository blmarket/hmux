use hmux2::src::cmd::queue::{cmdq_append, cmdq_get_callback_owned, cmdq_new, cmdq_next};
use hmux2::src::cmd::queue::{cmdq_get_event, cmdq_get_state_owned, cmdq_new_state};
use hmux2::src::server_client::Client as _;
use hmux2::src::shared::client::ClientRef;
use hmux2::src::shared::key::key_event;
use hmux2::src::window::Window;
use std::rc::Rc;

#[test]
fn event_snapshots_observe_clients_and_never_redirect_expired_targets() {
    use hmux2::src::shared::window::window;
    unsafe {
        let explicit = ClientRef::allocate();
        let fallback = ClientRef::allocate();
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
        let window = hmux2::src::shared::window::WindowRef::empty();
        assert!(window.set_latest_client(Some(&explicit)));
        assert!(window.is_latest_client(&resolved));
        assert!(!window.is_latest_client(&fallback));
        drop(explicit);
        assert!(
            snapshot.client.upgrade().is_some(),
            "dispatch retains client"
        );
        assert!(!window.set_latest_client(Some(&resolved)));
        drop(resolved);
        assert!(snapshot.client.upgrade().is_none());
        assert!(snapshot
            .resolve_client(|| panic!("expired target must not fall back"))
            .is_none());
        // Expiration does not silently switch to a fallback identity.
        assert!(!window.is_latest_client(&fallback));
        assert!(window.set_latest_client(None));
        assert!(!window.set_latest_client(None));
        window.release(c"event snapshot window");
        drop(fallback);
    }
}
