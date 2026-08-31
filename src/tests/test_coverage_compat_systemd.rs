//! Unit tests for [`crate::compat`].
//!
//! The module is tmux's systemd glue: the socket-activation shim
//! ([`systemd_activated`](crate::compat::systemd_activated) and
//! [`systemd_create_socket`](crate::compat::systemd_create_socket))
//! and the D-Bus call that moves a spawned pane into a transient scope,
//! [`systemd_move_to_new_cgroup`](crate::compat::systemd_move_to_new_cgroup),
//! together with the `sd-bus` shapes it needs — the opaque bus types, the
//! `sd_bus_error` value type, the 128-bit id union, the job-watch record the
//! signal callback fills, the Linux socket-type and errno numbers, and the
//! `JobRemoved` handler itself.
//!
//! What runs here is everything that is deterministic without a session bus,
//! an inherited descriptor or a fabricated D-Bus message: the constants
//! against their Linux ABI values, the layouts of the three concrete records,
//! the one branch of
//! [`job_removed_handler`](crate::compat::job_removed_handler)
//! that answers before any bus traffic (a watch with no path returns zero
//! without reading the message), and the activation probe in its natural
//! state — a plain test process carries no `LISTEN_PID`/`LISTEN_FDS`, so
//! libsystemd answers zero without looking at descriptors.
//!
//! What does not run, by design: `systemd_create_socket` falls through to
//! `server_create_socket` whenever no fd was inherited, which unlinks, binds
//! and listens on the real socket path; `systemd_move_to_new_cgroup` opens
//! the user's session bus; the remaining branches of the handler need a real
//! `sd_bus_message` for libsystemd to read. Both entry points are pinned at
//! compile time by function-pointer coercions instead, and the limitation is
//! recorded here as with the other process-spawning suites. None of the
//! process-wide statics the server keeps are touched, so no turn at the
//! [`crate::tests::test_fixtures::globals`] mutex is wanted.

use crate::compat::{
    E2BIG, EPFNOSUPPORT, SD_BUS_ERROR_NULL, SD_LISTEN_FDS_START, SOCK_CLOEXEC, SOCK_DCCP,
    SOCK_DGRAM, SOCK_NONBLOCK, SOCK_PACKET, SOCK_RAW, SOCK_RDM, SOCK_SEQPACKET, SOCK_STREAM,
    job_removed_handler, sd_bus_error, sd_bus_message_handler_t, sd_id128, systemd_activated,
    systemd_create_socket, systemd_job_watch, systemd_move_to_new_cgroup,
};
use crate::types::*;
use ::core::ffi::{CStr, c_char, c_int};
use ::core::ptr::{null, null_mut};
use ::std::ffi::CString;

/// The shape `systemd_create_socket` offers its callers: the client flags
/// word and the `cause` out-parameter of the ordinary socket path.
type CreateSocket = unsafe fn(c_int, &mut Option<CString>) -> c_int;

/// The shape `systemd_move_to_new_cgroup` offers its callers: only the
/// `cause` out-parameter, answering the last sd-bus result code.
type MoveToNewCgroup = unsafe fn(&mut Option<CString>) -> c_int;

#[test]
fn socket_type_constants_hold_the_linux_protocol_numbers() {
    assert_eq!(SOCK_STREAM, 1);
    assert_eq!(SOCK_DGRAM, 2);
    assert_eq!(SOCK_RAW, 3);
    assert_eq!(SOCK_RDM, 4);
    assert_eq!(SOCK_SEQPACKET, 5);
    assert_eq!(SOCK_DCCP, 6);
    assert_eq!(SOCK_PACKET, 10);
    assert_eq!(SOCK_STREAM as u64, ::libc::SOCK_STREAM as u64);
    assert_eq!(SOCK_DGRAM as u64, ::libc::SOCK_DGRAM as u64);
    assert_eq!(SOCK_RAW as u64, ::libc::SOCK_RAW as u64);
    assert_eq!(SOCK_RDM as u64, ::libc::SOCK_RDM as u64);
    assert_eq!(SOCK_SEQPACKET as u64, ::libc::SOCK_SEQPACKET as u64);
    assert_eq!(SOCK_DCCP as u64, ::libc::SOCK_DCCP as u64);
    // assert_eq!(SOCK_PACKET as u64, ::libc::SOCK_PACKET as u64);
    let all = [
        SOCK_STREAM,
        SOCK_DGRAM,
        SOCK_RAW,
        SOCK_RDM,
        SOCK_SEQPACKET,
        SOCK_DCCP,
        SOCK_PACKET,
        SOCK_NONBLOCK,
        SOCK_CLOEXEC,
    ];
    for i in 0..all.len() {
        for j in (i + 1)..all.len() {
            assert_ne!(all[i], all[j], "constants {i} and {j} collide");
        }
    }
}

#[test]
fn nonblock_and_cloexec_carry_the_open_flag_bits_the_callers_or_in() {
    assert_eq!(SOCK_NONBLOCK, 2048);
    assert_eq!(SOCK_CLOEXEC, 524288);
    assert_eq!(SOCK_NONBLOCK as u64, ::libc::O_NONBLOCK as u64);
    assert_eq!(SOCK_CLOEXEC as u64, ::libc::O_CLOEXEC as u64);
    let composed = SOCK_STREAM | SOCK_NONBLOCK | SOCK_CLOEXEC;
    assert_eq!(composed & SOCK_STREAM, SOCK_STREAM);
    assert_eq!(composed & SOCK_NONBLOCK, SOCK_NONBLOCK);
    assert_eq!(composed & SOCK_CLOEXEC, SOCK_CLOEXEC);
    assert_eq!(
        composed & !(SOCK_STREAM | SOCK_NONBLOCK | SOCK_CLOEXEC),
        0,
        "the flag bits reach outside the fields they are ored into"
    );
}

#[test]
fn errno_constants_hold_the_linux_values_the_shim_reports() {
    assert_eq!(E2BIG, 7);
    assert_eq!(EPFNOSUPPORT, 96);
    assert_eq!(E2BIG, ::libc::E2BIG);
    assert_eq!(EPFNOSUPPORT, ::libc::EPFNOSUPPORT);
    assert_ne!(E2BIG, EPFNOSUPPORT);
}

#[test]
fn sd_listen_fds_start_is_past_the_standard_descriptors() {
    assert_eq!(SD_LISTEN_FDS_START, 3);
    assert!(SD_LISTEN_FDS_START > ::libc::STDERR_FILENO);
    assert_eq!(SD_LISTEN_FDS_START, ::libc::STDERR_FILENO + 1);
}

#[test]
fn the_null_bus_error_is_all_zeroes_and_copies_by_value() {
    let e = SD_BUS_ERROR_NULL;
    assert!(e.name.is_null());
    assert!(e.message.is_null());
    assert_eq!(e._need_free, 0);
    let mut copy = e;
    copy._need_free = 1;
    assert_eq!(e._need_free, 0, "SD_BUS_ERROR_NULL was mutated by a copy");
    assert_eq!(copy.name, e.name);
    assert_eq!(copy.message, e.message);
}

#[test]
fn a_bus_error_carries_a_name_a_message_and_its_need_free_flag() {
    let e = sd_bus_error {
        name: c"org.freedesktop.systemd.Error".as_ptr(),
        message: c"boom".as_ptr(),
        _need_free: 1,
    };
    unsafe {
        assert_eq!(
            CStr::from_ptr(e.name).to_bytes(),
            b"org.freedesktop.systemd.Error"
        );
        assert_eq!(CStr::from_ptr(e.message).to_bytes(), b"boom");
    }
    assert_eq!(e._need_free, 1);
}

#[test]
fn a_did128_reads_its_bytes_and_its_qwords_the_same_way() {
    let mut id = sd_id128 { bytes: [0; 16] };
    unsafe {
        id.qwords[0] = 0x0123456789abcdef;
        id.qwords[1] = 0xfedcba9876543210;
        let (q0, q1) = (id.qwords[0], id.qwords[1]);
        let mut little_endian_view = [0u8; 16];
        little_endian_view[..8].copy_from_slice(&q0.to_le_bytes());
        little_endian_view[8..].copy_from_slice(&q1.to_le_bytes());
        assert_eq!(id.bytes, little_endian_view);

        let mut other = sd_id128 { bytes: [0; 16] };
        for i in 0..16usize {
            other.bytes[i] = i as uint8_t;
        }
        let q0 = uint64_t::from_le_bytes(other.bytes[..8].try_into().unwrap());
        let q1 = uint64_t::from_le_bytes(other.bytes[8..].try_into().unwrap());
        assert_eq!(other.qwords[0], q0);
        assert_eq!(other.qwords[1], q1);
        assert_ne!(id.qwords[0], other.qwords[0]);
    }
}

#[test]
fn a_job_watch_holds_a_path_and_a_done_flag() {
    let mut w = systemd_job_watch {
        path: c"/org/freedesktop/systemd1/job/41".as_ptr(),
        done: 0,
    };
    assert!(!w.path.is_null());
    w.done = 1;
    assert_eq!(w.done, 1);
    w.done = 0;
    assert_eq!(w.done, 0);
}

#[test]
fn job_removed_handler_matches_the_message_handler_shape() {
    let handler: sd_bus_message_handler_t = Some(job_removed_handler);
    assert!(handler.is_some());
    let none: sd_bus_message_handler_t = None;
    assert!(none.is_none());
}

#[test]
fn job_removed_handler_answers_zero_when_the_watch_has_no_path() {
    unsafe {
        let mut watch = systemd_job_watch {
            path: null::<c_char>(),
            done: 7,
        };
        let mut err = SD_BUS_ERROR_NULL;
        let r = job_removed_handler(null_mut(), &raw mut watch, &raw mut err);
        assert_eq!(r, 0);
        assert_eq!(watch.done, 7, "a pathless watch must stay untouched");
    }
}

#[test]
fn the_entry_points_keep_their_server_shaped_signatures() {
    let _create_socket_must_compile: CreateSocket = systemd_create_socket;
    let _move_to_new_cgroup_must_compile: MoveToNewCgroup = systemd_move_to_new_cgroup;
}

#[test]
fn systemd_activated_answers_no_without_an_inherited_listener() {
    for name in ["LISTEN_PID", "LISTEN_FDS"] {
        assert!(
            ::std::env::var(name).is_err(),
            "{name} is set in the harness environment, so this probe would read inherited fds"
        );
    }
    {
        assert_eq!(systemd_activated(), 0);
    }
}
