use crate::cmd::CMD_RETURN_NORMAL;
use crate::cmd::cmd_choose_tree::cmd_choose_client_entry;
use crate::tests::test_fixtures::{Clients, Item, Target, globals, seen};
use crate::types::*;
use crate::window::window_pane_current_mode;
use crate::window::window_pane_reset_mode_all;
use ::core::ffi::CStr;
use ::core::ptr::null_mut;

const FILE: &CStr = c"test_coverage_window_client.rs";

#[test]
fn test_window_client_mode_lifecycle_and_keys() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut t = Target::new(80, 24);

    unsafe {
        let c1 = clients.add("client-1", 80, 24);
        (*c1).session = t.session();
        let c2 = clients.add("client-2", 80, 24);
        (*c2).session = t.session();

        let wp = t.pane(0);

        let mut item = Item::with_client()
            .from_file(FILE, 1)
            .with_args(c"choose-client")
            .targeting(&mut t);

        let exec = cmd_choose_client_entry.exec;
        assert_eq!(exec(&*item.cmd(), item.ptr()), CMD_RETURN_NORMAL);

        let wme = window_pane_current_mode(wp);
        assert!(!wme.is_null());
        assert_eq!((*wme).mode(), WindowMode::Client);
        assert_eq!(seen((*wme).mode().name().as_ptr()), "client-mode");
        assert!((*wme).mode().default_format().is_some());

        // Update and resize
        (*wme).mode().update(wme);
        (*wme).mode().resize(wme, 90, 28);

        // Key interactions
        for key in [
            b'j' as key_code,
            b'k' as key_code,
            b't' as key_code,
            b' ' as key_code,
            b'?' as key_code,
            b'v' as key_code,
            b'D' as key_code,
            b'X' as key_code,
            b'Z' as key_code,
            b'z' as key_code,
            b'x' as key_code,
            b'd' as key_code,
            b'\r' as key_code,
            b'q' as key_code,
        ] {
            if !(*wp).modes.is_empty() {
                let cur_wme = window_pane_current_mode(wp);
                (*cur_wme)
                    .mode()
                    .key(cur_wme, c1, t.session(), t.winlink(0), key, null_mut());
            }
        }

        window_pane_reset_mode_all(wp);
        assert!((*wp).modes.is_empty());
    }
}

#[test]
fn test_window_client_custom_format_and_detach() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut t = Target::new(80, 24);

    unsafe {
        let c1 = clients.add("client-a", 80, 24);
        (*c1).session = t.session();

        let wp = t.pane(0);

        let mut item = Item::with_client()
            .from_file(FILE, 1)
            .with_args(c"choose-client -F \"#{client_name}\" -K \"#{client_name}\" -r -O name")
            .targeting(&mut t);

        let exec = cmd_choose_client_entry.exec;
        assert_eq!(exec(&*item.cmd(), item.ptr()), CMD_RETURN_NORMAL);

        let wme = window_pane_current_mode(wp);
        assert!(!wme.is_null());

        (*wme).mode().key(
            wme,
            c1,
            t.session(),
            t.winlink(0),
            b'd' as key_code,
            null_mut(),
        );

        window_pane_reset_mode_all(wp);
    }
}
