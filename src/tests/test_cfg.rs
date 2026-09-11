use crate::grid::Grid as _;
use super::*;
use crate::WindowPane;
use crate::server::client_ref_of;
use crate::server::server_client_get_cwd;
use crate::tests::test_fixtures::{Clients, globals};

/// A load under way against `c`, the way [`start_cfg`] leaves one once it has
/// the first client. This puts the client and completion flag back as they
/// were on the way out, even if the test panics.
struct Loading(Option<ClientWeak>, bool);

impl Loading {
    unsafe fn for_client(c: *mut client) -> Loading {
        unsafe {
            let held = Loading(
                with_config(|config| config.client.take()),
                replace_configuration_finished(false),
            );
            with_config(|config| config.client = client_ref_of(&*c).map(|c| c.downgrade()));
            held
        }
    }
}

impl Drop for Loading {
    fn drop(&mut self) {
        {
            with_config(|config| config.client = self.0.take());
            replace_configuration_finished(self.1);
        }
    }
}

#[test]
fn a_load_takes_its_directory_from_the_client_it_runs_for() {
    let _guard = globals();
    let mut attached = Clients::new();
    unsafe {
        let c = attached.add("loader", 80, 24);
        (*c).cwd = Some(c"/from/the/client".to_owned());
        let _loading = Loading::for_client(c);

        let cwd = server_client_get_cwd(None, None);

        assert_eq!(cwd.as_c_str(), c"/from/the/client");
    }
}

#[test]
fn a_load_names_no_client_once_the_one_it_ran_for_has_gone() {
    let _guard = globals();
    unsafe {
        let _loading = {
            let mut attached = Clients::new();
            let c = attached.add("loader", 80, 24);
            let loading = Loading::for_client(c);
            assert_eq!(
                cfg_client()
                    .expect("the client is there while it lives")
                    .as_ptr(),
                c
            );
            loading
        };

        assert!(cfg_client().is_none(), "the client has gone");
    }
}

#[test]
fn causes_wait_for_an_active_pane_and_append_to_view_mode() {
    
    use crate::tests::test_fixtures::{Target, ensure_reactor};

    let _guard = globals();
    ensure_reactor();
    let mut target = Target::new(80, 24);
    let window = target.state().window().unwrap();
    let pane = target.state().pane_list_ref().unwrap();
    unsafe {
        take_causes();
        let session = target.session_handle().clone();
        session.add_attached();
        window.as_window_mut().active = None;
        cfg_add_cause(c"pending configuration error", fmt_args![]);
        cfg_show_causes(Some(session.as_session()));
        assert_eq!(with_config(|config| config.causes.len()), 1);
        assert!(pane.get().unwrap().active_mode().is_none());

        {
            let mut payload = window.as_window_mut();

            payload.active = payload
                .panes
                .iter()
                .find(|pane| pane.pane_id() == pane.pane_id())
                .map(|pane| pane.downgrade());
        }
        cfg_show_causes(Some(session.as_session()));
        cfg_add_cause(c"another configuration error", fmt_args![]);
        cfg_show_causes(Some(session.as_session()));
        assert!(with_config(|config| config.causes.is_empty()));
        let active = pane.get().unwrap();
        let mode = active.active_mode().unwrap();
        assert_eq!(mode.mode(), WindowMode::View);
        let WindowModeState::View(data) = &mode.state else {
            panic!("configuration errors use view mode");
        };
        let backing = data.backing.as_deref().unwrap();
        assert_eq!(
            (backing.grid()).string_cells(0, 0, 80, None, 0, None).as_c_str(),
            c"pending configuration error"
        );
        assert_eq!(
            (backing.grid()).string_cells(0, 1, 80, None, 0, None).as_c_str(),
            c"another configuration error"
        );
        drop(target);
    }
}
