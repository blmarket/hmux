//! `getdtablecount`, which OpenBSD has and Linux does not: how many
//! descriptors this process has open.
//!
//! Linux answers it through `/proc`, where every open descriptor is a link
//! under the process's own `fd` directory, so the count is what a glob over
//! that directory matches. A `/proc` that is not mounted matches nothing and
//! is answered as no descriptors at all rather than as a failure.
//!
//! Coverage exemptions: the `fatal` arm for a path that did not fit, which
//! ends the process.
use crate::ffi::{getpid, glob, globfree, snprintf};
use crate::fmt_args;
use crate::log::fatal;
pub use crate::types::*;
use ::core::ffi::{c_char, c_int, c_long};

/// How long a path may be, which is what the C sizes its buffer to.
const PATH_MAX: usize = 4096;

pub fn getdtablecount() -> c_int {
    unsafe {
        let mut path = [0 as c_char; PATH_MAX];
        if snprintf(
            path.as_mut_ptr(),
            PATH_MAX as size_t,
            c"/proc/%ld/fd/*".as_ptr(),
            getpid() as c_long,
        ) < 0
        {
            fatal(c"snprintf overflow".as_ptr(), fmt_args![]);
        }
        let mut g = glob_t::default();
        let mut n: c_int = 0;
        if glob(path.as_ptr(), 0, None, &raw mut g) == 0 {
            n = g.gl_pathc as c_int;
        }
        globfree(&raw mut g);
        n
    }
}

#[cfg(test)]
#[path = "../tests/test_compat_getdtablecount.rs"]
mod tests;
