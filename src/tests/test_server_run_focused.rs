use super::*;
use crate::tests::test_fixtures::{globals, zeroed_client};
use std::os::unix::fs::FileTypeExt;

#[test]
fn socket_bind_failure_restores_the_process_umask() {
    if crate::test_process::run() {
        return;
    }
    let _guard = globals();
    unsafe {
        let saved_path = socket_path.take();
        // /dev/null is a file, so binding a socket beneath it must fail.
        socket_path.set(Some(c"/dev/null/hmux.sock".to_owned()));
        let original_mask = umask(0o022);
        for flags in [0, CLIENT_DEFAULTSOCKET as u64] {
            let mut cause = None;
            let fd = server_create_socket(flags, &mut cause);
            let restored_mask = umask(original_mask);
            socket_path.set(saved_path.clone());
            assert_eq!(fd, -1);
            assert!(cause.is_some());
            assert_eq!(restored_mask, 0o022);
            socket_path.set(Some(c"/dev/null/hmux.sock".to_owned()));
            umask(0o022);
        }
        umask(original_mask);
        socket_path.set(saved_path);
    }
}

#[test]
fn socket_creation_reports_long_paths_and_builds_both_permission_modes() {
    if crate::test_process::run() {
        return;
    }
    let _guard = globals();
    unsafe {
        let saved = socket_path.take();
        socket_path.set(Some(
            CString::new(format!("/tmp/{}", "x".repeat(120))).unwrap(),
        ));
        let mut cause = None;
        assert_eq!(server_create_socket(0, &mut cause), -1);
        assert!(cause.unwrap().to_bytes().starts_with(b"error creating"));

        for flags in [0, CLIENT_DEFAULTSOCKET as u64] {
            let path = format!("/tmp/hmux-sr-{}-{flags}.sock", std::process::id());
            socket_path.set(Some(CString::new(path.clone()).unwrap()));
            let mut cause = None;
            let fd = server_create_socket(flags, &mut cause);
            assert!(fd >= 0, "{cause:?}");
            assert_eq!(close(fd), 0);
            let metadata = fs::metadata(&path).unwrap();
            assert!(metadata.file_type().is_socket());
            fs::remove_file(path).unwrap();
        }
        socket_path.set(saved);
    }
}

#[test]
fn client_walk_marked_and_accept_helpers_cover_empty_safe_states() {
    let _guard = globals();
    unsafe {
        let saved_clients = with_clients_mut(core::mem::take);
        assert!(first_client().is_none());
        assert_eq!(client_walk().count(), 0);
        let first = zeroed_client();
        let second = zeroed_client();
        let third = zeroed_client();
        with_clients_mut(|clients| {
            clients.extend([first.clone(), second.clone()]);
        });
        assert!(first_client().is_some_and(|held| held.ptr_eq(&first)));
        let mut walk = client_walk();
        assert!(walk.next().is_some_and(|held| held.ptr_eq(&first)));
        with_clients_mut(|clients| {
            clients.remove(0);
        });
        with_clients_mut(|clients| {
            clients.push(third.clone());
        });
        assert!(walk.next().is_some_and(|held| held.ptr_eq(&second)));
        assert!(walk.next().is_some_and(|held| held.ptr_eq(&third)));
        assert!(walk.next().is_none());
        with_clients_mut(|clients| {
            clients.clear();
        });
        with_clients_mut(|clients| *clients = saved_clients);

        server_clear_marked();
        assert_eq!(server_check_marked(), 0);
        assert_eq!(
            server_is_marked(None, None, None::<&dyn crate::WindowPane>),
            0
        );
        server_set_marked(None, None, None::<&dyn crate::WindowPane>);
        assert_eq!(server_check_marked(), 0);

        let old_fd = server_fd.get();
        server_fd.set(-1);
        server_add_accept(0);
        server_add_accept(1);
        server_stop_accept();
        server_accept_timer();
        server_fd.set(old_fd);
    }
}

#[test]
fn borrowed_clients_allow_nested_queries_and_reject_registry_mutation() {
    let _guard = globals();
    assert!(with_clients(|clients| clients.is_empty()));
    let first = zeroed_client();
    with_clients_mut(|clients| clients.push(first.clone()));
    with_clients(|clients| {
        assert!(clients[0].ptr_eq(&first));
        with_clients(|nested| assert!(core::ptr::eq(&clients[0], &nested[0])));
        let mutation = std::panic::catch_unwind(|| {
            with_clients_mut(|clients| clients.clear());
        });
        assert!(mutation.is_err());
        assert_eq!(clients.len(), 1);
    });
    with_clients_mut(|clients| clients.clear());
    assert!(with_clients(|clients| clients.is_empty()));
}

#[test]
fn client_walk_observes_membership_changes_without_retaining_unvisited_clients() {
    let _guard = globals();
    let first = zeroed_client();
    let second = zeroed_client();
    let removed = second.downgrade();
    with_clients_mut(|clients| clients.extend([first, second]));
    let mut walk = client_walk();
    let first = walk.next().unwrap();
    with_clients_mut(|clients| clients.truncate(1));
    assert!(removed.upgrade().is_none());
    let third = zeroed_client();
    with_clients_mut(|clients| clients.push(third.clone()));
    assert!(walk.next().unwrap().ptr_eq(&third));
    assert!(walk.next().is_none());
    drop(first);
    with_clients_mut(|clients| clients.clear());
}

#[test]
fn safe_client_walk_saves_next_before_removal_and_stops_at_the_original_tail() {
    let _guard = globals();
    let first = zeroed_client();
    let second = zeroed_client();
    with_clients_mut(|clients| clients.extend([first.clone(), second.clone()]));
    let mut walk = client_walk_safe();
    assert!(walk.next().unwrap().ptr_eq(&first));
    with_clients_mut(|clients| {
        clients.remove(0);
    });
    assert!(walk.next().unwrap().ptr_eq(&second));
    with_clients_mut(|clients| clients.push(zeroed_client()));
    assert!(walk.next().is_none());
    with_clients_mut(|clients| clients.clear());
}
