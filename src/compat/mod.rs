//! The shims: what the C had from libbsd, imsg, utf8proc and systemd,
//! kept here so the rest of the crate can call it by its C name.
//!
//! Everything below this module is private. What the rest of the crate may
//! use is exactly what is re-exported here.

mod fdforkpty;
mod fgetln;
mod freezero;
mod getdtablecount;
mod getopt_long;
mod getpeereid;
mod getprogname;
mod htonll;
mod imsg;
mod imsg_buffer;
mod ntohll;
mod recallocarray;
mod setproctitle;
mod strtonum;
mod systemd;
mod unvis;
mod utf8proc;
mod vis;

pub use fdforkpty::{fdforkpty, getptmfd};
pub use getopt_long::{BSDgetopt, BSDoptarg, BSDoptind};
pub use getpeereid::getpeereid;
pub use getprogname::getprogname;
pub use imsg::{
    imsg_compose, imsg_free, imsg_get, imsg_get_fd, imsgbuf_allow_fdpass, imsgbuf_clear,
    imsgbuf_flush, imsgbuf_init, imsgbuf_queuelen, imsgbuf_read, imsgbuf_write,
};
pub use imsg_buffer::{ibufqueue, msgbuf, msghdr};
pub use recallocarray::recallocarray;
pub use setproctitle::setproctitle;
pub use strtonum::strtonum;
pub use systemd::{systemd_activated, systemd_create_socket, systemd_move_to_new_cgroup};
pub use unvis::strunvis;
pub use utf8proc::{utf8proc_mbtowc, utf8proc_wctomb, utf8proc_wcwidth};
pub use vis::{stravis, strnvis, vis};

#[cfg(test)]
pub(crate) use fdforkpty::{__INT_MAX__, INT_MAX};
#[cfg(test)]
pub(crate) use getopt_long::{BADCH, FLAG_ALLARGS, FLAG_LONGONLY, FLAG_PERMUTE, INORDER};
#[cfg(test)]
pub(crate) use htonll::htonll;
#[cfg(test)]
pub(crate) use imsg::{imsg_get_len, imsg_get_type, imsgbuf_get};
#[cfg(test)]
pub(crate) use imsg_buffer::*;
#[cfg(test)]
pub(crate) use ntohll::ntohll;
#[cfg(test)]
pub(crate) use strtonum::{EINVAL, ERANGE, LLONG_MAX, LLONG_MIN};
#[cfg(test)]
pub(crate) use systemd::{
    E2BIG, EPFNOSUPPORT, SD_BUS_ERROR_NULL, SD_LISTEN_FDS_START, SOCK_CLOEXEC, SOCK_DCCP,
    SOCK_DGRAM, SOCK_NONBLOCK, SOCK_PACKET, SOCK_RAW, SOCK_RDM, SOCK_SEQPACKET, SOCK_STREAM,
    job_removed_handler, sd_bus_error, sd_bus_message_handler_t, sd_id128, systemd_job_watch,
};
#[cfg(test)]
pub(crate) use vis::{
    VIS_ALL, VIS_CSTYLE, VIS_DQ, VIS_GLOB, VIS_NL, VIS_NOSLASH, VIS_OCTAL, VIS_SAFE, VIS_SP,
    VIS_TAB, strvis, strvisx,
};
