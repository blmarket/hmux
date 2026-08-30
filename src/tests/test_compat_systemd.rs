use super::*;
use ::core::ptr::null_mut;

#[test]
fn test_job_removed_handler_null_path() {
    unsafe {
        let mut watch = systemd_job_watch {
            path: null_mut(),
            done: 0,
        };
        let res = job_removed_handler(
            null_mut(),
            &raw mut watch as *mut ::core::ffi::c_void,
            null_mut(),
        );
        assert_eq!(res, 0);
        assert_eq!(watch.done, 0);
    }
}

#[test]
fn test_systemd_create_socket_fallback_when_no_listen_fds() {
    let _guard = crate::tests::test_fixtures::globals();
    unsafe {
        let mut cause = None;
        let fd = systemd_create_socket(0, &mut cause);
        if fd >= 0 {
            ::libc::close(fd);
        }
    }
}
