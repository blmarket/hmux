use super::*;
use crate::tests::test_fixtures::{Clients, Session, globals, zeroed_client};

#[test]
fn direct_client_redraw_and_status_flags_are_exact_and_idempotent() {
    let _guard = globals();
    let mut c = zeroed_client();
    unsafe {
        *c.flags_mut() = 0;
        server_status_client(c.as_client_mut());
        assert_eq!(c.flags(), CLIENT_REDRAWSTATUS as u64);
        server_redraw_client(c.as_client_mut());
        assert_eq!(c.flags() & CLIENT_ALLREDRAWFLAGS, CLIENT_ALLREDRAWFLAGS);
        let flags = c.flags();
        server_redraw_client(c.as_client_mut());
        assert_eq!(c.flags(), flags);
    }
}

#[test]
fn session_redraw_status_and_lock_skip_unrelated_control_and_suspended_clients() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut selected = Session::new(801, "selected");
    let mut other = Session::new(802, "other");
    let selected_client = clients.add("selected-client", 80, 24);
    let other_client = clients.add("other-client", 80, 24);
    unsafe {
        (*selected_client).set_attached_session(Some(selected.handle()));
        (*other_client).set_attached_session(Some(other.handle()));
        server_status_session(&mut *selected.ptr());
        assert_ne!((*selected_client).flags & CLIENT_REDRAWSTATUS as u64, 0);
        assert_eq!((*other_client).flags & CLIENT_REDRAWSTATUS as u64, 0);
        (*selected_client).flags = 0;
        server_redraw_session_group(&mut *selected.ptr());
        assert_ne!((*selected_client).flags & CLIENT_REDRAWWINDOW as u64, 0);
        assert_eq!((*other_client).flags & CLIENT_REDRAWWINDOW as u64, 0);

        (*selected_client).flags = CLIENT_CONTROL as u64;
        server_lock_client(&mut *selected_client);
        assert_eq!((*selected_client).flags, CLIENT_CONTROL as u64);
        (*selected_client).flags = CLIENT_SUSPENDED as u64;
        server_lock_client(&mut *selected_client);
        assert_eq!((*selected_client).flags, CLIENT_SUSPENDED as u64);
    }
}

#[test]
fn window_redraw_skips_unrelated_and_missing_current_windows_but_status_tracks_links() {
    use crate::tests::test_fixtures::{Target, Window};

    let _guard = globals();
    let mut target = Target::new(80, 24);
    let mut session = target.session_handle().clone();
    let window = target.state().window().unwrap();
    let unrelated = Window::new(999, "unrelated", 80, 24);
    let mut clients = Clients::new();
    clients.add("attached", 80, 24);
    clients.add("detached", 80, 24);
    let mut owners: Vec<_> = client_walk().collect();
    unsafe {
        owners[0].set_attached_session(Some(&session));
        for client in &mut owners {
            *client.flags_mut() = 0;
        }
        (unrelated.handle()).redraw();
        (unrelated.handle()).redraw_borders();
        (unrelated.handle()).redraw_status();
        assert!(owners.iter().all(|client| client.flags() == 0));

        let payload = window.as_window_mut();
        window.redraw_borders();
        assert_eq!(owners[0].flags(), CLIENT_REDRAWBORDERS as u64);
        assert_eq!(owners[1].flags(), 0);
        window.redraw();
        assert_eq!(owners[0].flags(), CLIENT_ALLREDRAWFLAGS);
        drop(payload);
        assert_eq!(owners[1].flags(), 0);

        *owners[0].flags_mut() = 0;
        let current = session.as_session_mut().curw_idx.take();
        window.redraw();
        window.redraw_borders();
        assert!(owners.iter().all(|client| client.flags() == 0));
        window.redraw_status();
        assert_eq!(owners[0].flags(), CLIENT_REDRAWSTATUS as u64);
        assert_eq!(owners[1].flags(), 0);
        session.as_session_mut().curw_idx = current;
    }
}
