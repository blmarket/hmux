use super::*;
use crate::server::server_client_get_cwd;
use crate::tests::test_fixtures::{Clients, globals, seen};

/// A load under way against `c`, the way [`start_cfg`] leaves one once it has
/// the first client. Both are process-wide, so this puts them back as they
/// were on the way out, even if the test panics.
struct Loading(Option<ClientWeak>, ::core::ffi::c_int);

impl Loading {
    unsafe fn for_client(c: *mut client) -> Loading {
        unsafe {
            let held = Loading(CFG_CLIENT.take(), cfg_finished);
            CFG_CLIENT = client_ref_from_ptr(c).map(|c| c.downgrade());
            cfg_finished = 0;
            held
        }
    }
}

impl Drop for Loading {
    fn drop(&mut self) {
        unsafe {
            CFG_CLIENT = self.0.take();
            cfg_finished = self.1;
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

        let cwd = server_client_get_cwd(::core::ptr::null_mut(), ::core::ptr::null_mut());

        assert_eq!(seen(cwd), "/from/the/client");
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
            assert_eq!(cfg_client(), c, "the client is there while it lives");
            loading
        };

        assert!(cfg_client().is_null(), "the client has gone");
    }
}
