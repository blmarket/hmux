use crate::WindowPane;
use crate::cmd::CMD_RETURN_NORMAL;
use crate::tests::test_fixtures::{Clients, Item, Target, globals, seen};
use crate::types::*;
use ::core::ffi::CStr;

const FILE: &CStr = c"test_coverage_window_client.rs";

#[test]
fn test_window_client_mode_lifecycle_and_keys() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut t = Target::new(80, 24);

    unsafe {
        let c1 = clients.add("client-1", 80, 24);
        (*c1).set_attached_session(Some(t.session_handle()));
        let c2 = clients.add("client-2", 80, 24);
        (*c2).set_attached_session(Some(t.session_handle()));

        let wp = t.pane(0);

        let item = Item::with_client()
            .with_file(FILE, 1)
            .with_args(c"choose-client")
            .targeting(&mut t);

        let exec = crate::cmd::cmd_find(c"choose-client").unwrap().exec;
        assert_eq!(
            item.with_command(|command, item| exec(command, item)),
            CMD_RETURN_NORMAL
        );

        let wme = (&mut *wp).active_mode_mut().map(|mode| mode.into_entry()).expect("pane is in a mode");
        assert_eq!(wme.mode(), WindowMode::Client);
        assert_eq!(seen(wme.mode().name().as_ptr()), "client-mode");
        assert!(wme.mode().default_format().is_some());

        // Update and resize
        wme.update_target().unwrap().dispatch();
        let wme = (&mut *wp).active_mode_mut().map(|mode| mode.into_entry()).expect("pane is in a mode");
        wme.mode().resize(wme, 90, 28);

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
            if !(*wp).active_mode().is_none() {
                let cur_wme = (&mut *wp).active_mode_mut().map(|mode| mode.into_entry()).expect("pane is in a mode");
                cur_wme.key_target().unwrap().dispatch(&mut *c1, key, None);
            }
        }

        (&mut *wp).reset_modes();
        assert!((*wp).active_mode().is_none());
    }
}

#[test]
fn test_window_client_custom_format_and_detach() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut t = Target::new(80, 24);

    unsafe {
        let c1 = clients.add("client-a", 80, 24);
        (*c1).set_attached_session(Some(t.session_handle()));

        let wp = t.pane(0);

        let item = Item::with_client()
            .with_file(FILE, 1)
            .with_args(c"choose-client -F \"#{client_name}\" -K \"#{client_name}\" -r -O name")
            .targeting(&mut t);

        let exec = crate::cmd::cmd_find(c"choose-client").unwrap().exec;
        assert_eq!(
            item.with_command(|command, item| exec(command, item)),
            CMD_RETURN_NORMAL
        );

        let wme = (&mut *wp).active_mode_mut().map(|mode| mode.into_entry()).expect("pane is in a mode");

        wme.key_target()
            .unwrap()
            .dispatch(&mut *c1, b'd' as key_code, None);

        (&mut *wp).reset_modes();
    }
}
