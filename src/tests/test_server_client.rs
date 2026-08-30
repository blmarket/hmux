use super::*;
use crate::tests::test_fixtures::{Clients, Target, globals};

/// A client attached to `t`'s session that keeps a pane of its own, which is
/// what makes [`server_client_get_pane`] consult the per-window entry rather
/// than answer the window's active pane.
unsafe fn client_with_its_own_pane(attached: &mut Clients, t: &mut Target) -> *mut client {
    unsafe {
        let c = attached.add("client", 80, 24);
        (*c).session = t.session();
        (*c).flags |= CLIENT_ACTIVEPANE as uint64_t;
        c
    }
}

/// Drops the per-window sizes a test left on `c`, since nothing here takes
/// the client down through `server_destroy`.
unsafe fn forget_client_windows(c: *mut client) {
    unsafe { drop(::core::mem::take(&mut (*c).windows)) };
}

#[test]
fn a_client_finds_the_pane_it_made_active() {
    let _guard = globals();
    let mut attached = Clients::new();
    let mut t = Target::new(80, 24);
    unsafe {
        let c = client_with_its_own_pane(&mut attached, &mut t);
        let wp = t.pane(0);

        server_client_set_pane(c, wp);

        assert_eq!(server_client_get_pane(c), wp);
        forget_client_windows(c);
    }
}

#[test]
fn a_client_that_has_made_no_pane_active_finds_none() {
    let _guard = globals();
    let mut attached = Clients::new();
    let mut t = Target::new(80, 24);
    unsafe {
        let c = client_with_its_own_pane(&mut attached, &mut t);
        server_client_add_client_window(c, (*t.window(0)).id);

        assert!(server_client_get_pane(c).is_null());
        forget_client_windows(c);
    }
}

#[test]
fn taking_a_pane_away_forgets_the_client_that_had_it_active() {
    let _guard = globals();
    let mut attached = Clients::new();
    let mut t = Target::new(80, 24);
    unsafe {
        let c = client_with_its_own_pane(&mut attached, &mut t);
        let wp = t.pane(0);
        server_client_set_pane(c, wp);

        server_client_remove_pane(wp);

        assert!((*c).windows.is_empty(), "the entry naming the pane is gone");
    }
}
