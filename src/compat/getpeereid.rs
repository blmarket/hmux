//! `getpeereid`, which BSD has and Linux does not: who is at the other end of
//! a Unix socket.
//!
//! Linux answers the same question through `SO_PEERCRED`, which writes a
//! `struct ucred` — a process as well as a user and a group, of which only the
//! last two are wanted here. tmux asks this of a client that has just
//! connected, to decide whether the socket may be shared with it.
//!
//! Coverage exemptions: none.
use crate::ffi::getsockopt;
pub use crate::types::*;
use ::core::ffi::{c_int, c_void};

/// What `SO_PEERCRED` writes: the process, the user and the group at the other
/// end of the socket.
#[repr(C)]
struct ucred {
    pid: pid_t,
    uid: uid_t,
    gid: gid_t,
}

const SOL_SOCKET: c_int = 1;
const SO_PEERCRED: c_int = 17;

/// The user and the group at the other end of `s`, or nothing if it has no
/// other end — which is anything that is not a connected socket, an unopened
/// descriptor included.
///
/// The length handed to `getsockopt` is an `int` rather than the `socklen_t`
/// the call is declared with, which is what the C writes and what every
/// platform tmux builds this shim for makes the same size.
fn peer_of(s: c_int) -> Option<(uid_t, gid_t)> {
    let mut uc = ucred {
        pid: 0,
        uid: 0,
        gid: 0,
    };
    let mut len = ::core::mem::size_of::<ucred>() as c_int;
    let asked = unsafe {
        getsockopt(
            s,
            SOL_SOCKET,
            SO_PEERCRED,
            (&raw mut uc).cast::<c_void>(),
            (&raw mut len).cast::<socklen_t>(),
        )
    };
    if asked == -1 {
        return None;
    }
    Some((uc.uid, uc.gid))
}

pub fn getpeereid(s: c_int, uid: &mut uid_t, gid: &mut gid_t) -> c_int {
    match peer_of(s) {
        Some((peer_uid, peer_gid)) => {
            *uid = peer_uid;
            *gid = peer_gid;
            0
        }
        None => -1,
    }
}

#[cfg(test)]
#[path = "../tests/test_compat_getpeereid.rs"]
mod tests;
