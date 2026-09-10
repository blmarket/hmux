use super::*;
use ::core::ptr::null_mut;

#[test]
fn test_job_removed_handler_null_path() {
    unsafe {
        let mut watch = systemd_job_watch {
            path: None,
            done: 0,
        };
        let res = job_removed_handler(null_mut(), (&raw mut watch).cast(), null_mut());
        assert_eq!(res, 0);
        assert_eq!(watch.done, 0);
    }
}

#[test]
fn test_systemd_create_socket_fallback_when_no_listen_fds() {
    if crate::test_process::run() {
        return;
    }
    let _guard = crate::tests::test_fixtures::globals();
    unsafe {
        let mut cause = None;
        let fd = systemd_create_socket(0, &mut cause);
        if fd >= 0 {
            libc::close(fd);
        }
    }
}

#[test]
fn bus_error_snapshot_outlives_native_error_cleanup() {
    let mut native = NativeBusError::default();
    unsafe {
        sd_bus_error_set(
            &mut native,
            c"org.example.Error".as_ptr(),
            c"detail".as_ptr(),
        );
    }
    let mut snapshot = native.snapshot();
    drop(native);
    assert_eq!(
        snapshot.systemd_bus_error_name(),
        Some(c"org.example.Error")
    );
    assert_eq!(snapshot.systemd_bus_error_message(), Some(c"detail"));
    snapshot.set_systemd_bus_error_need_free(0);
    assert_eq!(snapshot.systemd_bus_error_message(), Some(c"detail"));
}
