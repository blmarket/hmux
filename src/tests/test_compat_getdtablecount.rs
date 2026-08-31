//! What is left uncovered here is the `fatal` arm for a path that did not
//! fit, which ends the process.

use super::*;

/// The count is read out of `/proc`, so it is whatever the whole process
/// has open when it is asked. What a test can say about that is a floor:
/// the three descriptors the C library opens for every program, and the
/// two ends of a pipe it holds open itself. Cargo's other test threads
/// open descriptors of their own, so an exact count — or a count taken
/// twice and compared — is a race, not a test.
#[test]
fn the_count_holds_the_descriptors_this_process_has_open() {
    unsafe {
        assert!(getdtablecount() >= 3);
        let mut fds = [-1 as ::core::ffi::c_int; 2];
        assert_eq!(::libc::pipe(fds.as_mut_ptr()), 0, "no pipe");
        let n = getdtablecount();
        ::libc::close(fds[0]);
        ::libc::close(fds[1]);
        assert!(n >= 5, "{n} descriptors open");
    }
}
