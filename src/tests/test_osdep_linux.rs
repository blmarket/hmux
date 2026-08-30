use super::*;
use ::core::ffi::{c_char, c_int};
use ::core::ptr::null_mut;

/// Both lookups start by asking the descriptor which process group is in
/// front of the terminal behind it, and answer nothing when it is no
/// terminal at all. The other half of each — the walk through `/proc` for
/// the command line or the working directory — wants a pseudo-terminal
/// that is the *controlling* terminal of the process asking, which a unit
/// test cannot stand up without a session of its own; the conformance
/// suite is what exercises it.
#[test]
fn nothing_is_known_about_a_descriptor_that_is_no_terminal() {
    unsafe {
        let mut fds = [-1 as c_int; 2];
        assert_eq!(::libc::pipe(fds.as_mut_ptr()), 0, "no pipe");
        assert!(osdep_get_name(fds[0], null_mut::<c_char>()).is_none());
        assert!(osdep_get_cwd(fds[0]).is_none());
        assert!(osdep_get_name(-1, null_mut::<c_char>()).is_none());
        assert!(osdep_get_cwd(-1).is_none());
        ::libc::close(fds[0]);
        ::libc::close(fds[1]);
    }
}

#[test]
fn test_osdep_event_init() {
    let base = osdep_event_init();
    let _ = base;
}
