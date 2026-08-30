use super::*;
use ::core::ffi::c_int;

#[test]
fn a_socket_knows_who_is_at_the_other_end_of_it() {
    unsafe {
        let mut fds = [-1 as c_int; 2];
        assert_eq!(
            ::libc::socketpair(::libc::AF_UNIX, ::libc::SOCK_STREAM, 0, fds.as_mut_ptr()),
            0,
            "no socket pair"
        );
        let mut uid: uid_t = 12345;
        let mut gid: gid_t = 12345;
        assert_eq!(getpeereid(fds[0], &raw mut uid, &raw mut gid), 0);
        assert_eq!(uid, ::libc::geteuid());
        assert_eq!(gid, ::libc::getegid());
        ::libc::close(fds[0]);
        ::libc::close(fds[1]);
    }
}

/// Only a socket has anybody at the other end, so a pipe — and a
/// descriptor that is not open at all — is refused, and neither the user
/// nor the group is written.
#[test]
fn anything_that_is_not_a_socket_has_nobody_at_the_other_end() {
    unsafe {
        let mut fds = [-1 as c_int; 2];
        assert_eq!(::libc::pipe(fds.as_mut_ptr()), 0, "no pipe");
        let mut uid: uid_t = 12345;
        let mut gid: gid_t = 12345;
        assert_eq!(getpeereid(fds[0], &raw mut uid, &raw mut gid), -1);
        assert_eq!(getpeereid(-1, &raw mut uid, &raw mut gid), -1);
        assert_eq!((uid, gid), (12345, 12345));
        ::libc::close(fds[0]);
        ::libc::close(fds[1]);
    }
}
