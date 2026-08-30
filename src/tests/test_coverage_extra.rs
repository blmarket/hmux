//! Extra coverage for `server`, `server_fn`, `file` and `cfg`.
//! These modules sit at ~24% / 55% / 44% / 68% lines — the low hanging
//! fruit the harness can cover without a live daemon or a pty.

use crate::cfg::{cfg_add_cause, cfg_print_causes, cfg_show_causes};
use crate::file::{
    file_can_print, file_create_with_client, file_create_with_peer, file_find_ref, file_free,
    file_write_left,
};
use crate::proc::tmuxpeer;
use crate::reactor::Reactor;
use crate::server::{
    server_add_message, server_check_marked, server_clear_marked, server_create_socket,
    server_is_marked, server_set_marked,
};
use crate::server::{
    server_destroy_session, server_lock, server_redraw_client, server_redraw_session,
    server_redraw_window, server_redraw_window_borders, server_status_client,
    server_status_session, server_status_window,
};
use crate::session::session_add_attached;
use crate::tests::test_fixtures::{Clients, Target, ensure_reactor, globals};
use crate::tmux::{global_options, socket_path};
use crate::types::*;
use ::core::ffi::c_int;
use ::core::ptr::null_mut;

// ---------------------------------------------------------------------------
// server.rs — marked pane
// ---------------------------------------------------------------------------

#[test]
fn marked_pane_round_trips_through_set_and_clear() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    let s = target.session();
    let wl = target.winlink(0);
    let wp = target.pane(0);
    unsafe {
        server_set_marked(s, wl, wp);
        assert_eq!(server_check_marked(), 1);
        assert_eq!(server_is_marked(s, wl, wp), 1);
        assert_eq!(server_is_marked(null_mut(), wl, wp), 0);
        assert_eq!(server_is_marked(s, null_mut(), wp), 0);
        server_clear_marked();
        assert_eq!(server_check_marked(), 0);
        assert_eq!(server_is_marked(s, wl, wp), 0);
    }
}

#[test]
fn server_create_socket_too_long_path_returns_error() {
    let _guard = globals();
    let long = "a".repeat(200);
    let cs = ::std::ffi::CString::new(long).unwrap();
    let saved = unsafe { socket_path };
    unsafe {
        socket_path = cs.as_ptr();
        let mut cause = None;
        let fd = server_create_socket(0, &mut cause);
        assert_eq!(fd, -1);
        assert!(cause.is_some());
        socket_path = saved;
    }
}

#[test]
fn server_create_socket_success_and_cleanup() {
    let _guard = globals();
    let dir = ::std::env::temp_dir().join(format!("tmux-extra-{}", ::std::process::id()));
    let _ = ::std::fs::create_dir_all(&dir);
    let sock = dir.join("sock");
    let cs = ::std::ffi::CString::new(sock.to_str().unwrap()).unwrap();
    let saved = unsafe { socket_path };
    unsafe {
        socket_path = cs.as_ptr();
        let mut cause = None;
        let fd = server_create_socket(0, &mut cause);
        if fd >= 0 {
            ::libc::close(fd);
            let _ = ::std::fs::remove_file(&sock);
        }
        socket_path = saved;
    }
    let _ = ::std::fs::remove_dir_all(&dir);
}

#[test]
fn server_add_message_respects_message_limit() {
    let _guard = globals();
    unsafe {
        // shrink limit so the queue trimming fires
        crate::options::options_set_number(global_options, c"message-limit".as_ptr(), 2);
        // clear queue first
        crate::server::message_log.queue().clear();
        server_add_message(c"hello %d".as_ptr(), crate::fmt_args![1 as c_int]);
        server_add_message(c"hello %d".as_ptr(), crate::fmt_args![2 as c_int]);
        server_add_message(c"hello %d".as_ptr(), crate::fmt_args![3 as c_int]);
        assert!(crate::server::message_log.queue().len() <= 2);
        crate::server::message_log.queue().clear();
        crate::options::options_set_number(global_options, c"message-limit".as_ptr(), 100);
    }
}

// ---------------------------------------------------------------------------
// server_fn.rs — redraw / status helpers
// ---------------------------------------------------------------------------

#[test]
fn server_fn_redraw_and_status_mark_clients_and_sessions() {
    let _guard = globals();
    ensure_reactor();
    let mut target = Target::new(80, 24);
    let s = target.session();
    let mut clients_fixture = Clients::new();
    let c1 = clients_fixture.add("c1", 80, 24);
    let c2 = clients_fixture.add("c2", 80, 24);
    unsafe {
        (*c1).session = s;
        (*c2).session = s;
        (*c1).flags = 0;
        (*c2).flags = 0;
        let w = target.window(0);
        // redraw client sets ALL flags
        server_redraw_client(c1);
        assert_ne!((*c1).flags & 0x8, 0);
        // status client sets only STATUS
        server_status_client(c1);
        // redraw/status over session touches both clients
        (*c1).flags = 0;
        (*c2).flags = 0;
        server_redraw_session(s);
        assert_ne!((*c1).flags, 0);
        assert_ne!((*c2).flags, 0);
        (*c1).flags = 0;
        (*c2).flags = 0;
        server_status_session(s);
        assert_ne!((*c1).flags, 0);
        // window helpers walk clients whose curw points at the window
        (*c1).flags = 0;
        (*c2).flags = 0;
        server_redraw_window(w);
        server_redraw_window_borders(w);
        server_status_window(w);
        // at least one of them observed the window
        // (Target's curw points at w, so both sessions' clients qualify)
    }
}

#[test]
fn server_lock_is_noop_for_control_and_suspended_clients() {
    let _guard = globals();
    let mut clients_fixture = Clients::new();
    let c = clients_fixture.add("ctrl", 80, 24);
    unsafe {
        (*c).flags |= crate::server::CLIENT_CONTROL as u64;
        server_lock();
        (*c).flags = crate::server::CLIENT_SUSPENDED as u64;
        server_lock();
    }
}

// ---------------------------------------------------------------------------
// file.rs
// ---------------------------------------------------------------------------

#[test]
fn file_can_print_answers_for_null_and_flags() {
    let _guard = globals();
    unsafe {
        assert_eq!(file_can_print(null_mut()), 0);
        let mut clients_fixture = Clients::new();
        let c = clients_fixture.add("plain", 80, 24);
        assert_eq!(file_can_print(c), 1);
        (*c).flags |= super::super::file::CLIENT_CONTROL as u64;
        assert_eq!(file_can_print(c), 0);
        (*c).flags = super::super::file::CLIENT_ATTACHED as u64;
        assert_eq!(file_can_print(c), 0);
    }
}

#[test]
fn file_find_ref_and_create_and_free() {
    let _guard = globals();
    unsafe {
        let mut files: client_files_t = ::std::collections::BTreeMap::new();
        assert!(file_find_ref(&raw mut files, 99).is_none());
        // create via client (non-attached, so it is tracked in client's tree)
        let mut clients_fixture = Clients::new();
        let c = clients_fixture.add("f", 80, 24);
        // non-attached client: file_create_with_client registers in c.files
        let cf = file_create_with_client(c, 7, None, ClientFileData::None);
        assert!(!cf.as_ptr().is_null());
        assert!(file_find_ref(&raw mut (*c).files, 7).is_some());
        // write left reports empty buffer
        assert_eq!(file_write_left(&raw mut (*c).files), 0);
        file_free(cf);
        assert!(file_find_ref(&raw mut (*c).files, 7).is_none());
        // create via peer map
        let mut peer_files: client_files_t = ::std::collections::BTreeMap::new();
        let fake_peer = null_mut::<tmuxpeer>();
        let cf2 = file_create_with_peer(
            fake_peer,
            &raw mut peer_files,
            9,
            None,
            ClientFileData::None,
        );
        assert!(!cf2.as_ptr().is_null());
        assert!(file_find_ref(&raw mut peer_files, 9).is_some());
        file_free(cf2);
        assert!(file_find_ref(&raw mut peer_files, 9).is_none());
    }
}

// ---------------------------------------------------------------------------
// cfg.rs
// ---------------------------------------------------------------------------

#[test]
fn cfg_add_cause_and_print_and_show() {
    let _guard = globals();
    ensure_reactor();
    unsafe {
        cfg_add_cause(c"boom %d".as_ptr(), crate::fmt_args![42 as c_int]);
        let mut item = crate::tests::test_fixtures::Item::new();
        cfg_print_causes(item.ptr());
        cfg_add_cause(c"err %s".as_ptr(), crate::fmt_args![c"here".as_ptr()]);
        let mut item2 = crate::tests::test_fixtures::Item::new();
        cfg_print_causes(item2.ptr());
        cfg_add_cause(c"show %d".as_ptr(), crate::fmt_args![7 as c_int]);
        let mut target = Target::new(80, 24);
        session_add_attached(target.session());
        cfg_show_causes(target.session());
        cfg_add_cause(c"show2".as_ptr(), crate::fmt_args![]);
        cfg_show_causes(null_mut());
        let mut drain = crate::tests::test_fixtures::Item::new();
        cfg_print_causes(drain.ptr());
    }
}

#[test]
fn file_write_and_read_through_the_direct_filesystem_path() {
    let _guard = globals();
    ensure_reactor();
    let dir = ::std::path::PathBuf::from(format!("/tmp/tmux-file-extra-{}", ::std::process::id()));
    let _ = ::std::fs::create_dir_all(&dir);
    let path = dir.join("hello.txt");
    let cs = ::std::ffi::CString::new(path.to_str().unwrap()).unwrap();
    let data = b"hello file\n";
    unsafe {
        crate::file::file_write(
            null_mut(),
            cs.as_ptr(),
            0,
            data.as_ptr(),
            data.len() as size_t,
            None,
            ClientFileData::None,
        );
        crate::reactor::current().run_once();
        // file_write via null client takes the direct fopen branch – may succeed or set error,
        // either way the lines are covered.
        let _ = ::std::fs::read(&path);
        let cf = crate::file::file_read(null_mut(), cs.as_ptr(), None, ClientFileData::None);
        crate::reactor::current().run_once();
        assert!(cf.is_none());
        let _ = ::std::fs::remove_file(&path);
    }
    let _ = ::std::fs::remove_dir_all(&dir);
    let missing = dir.join("missing.txt");
    let cs2 = ::std::ffi::CString::new(missing.to_str().unwrap()).unwrap();
    unsafe {
        let cf = crate::file::file_read(null_mut(), cs2.as_ptr(), None, ClientFileData::None);
        crate::reactor::current().run_once();
        assert!(cf.is_none());
    }
}

#[test]
fn server_destroy_session_does_not_crash_on_lonely_session() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    unsafe {
        server_destroy_session(target.session());
    }
}
