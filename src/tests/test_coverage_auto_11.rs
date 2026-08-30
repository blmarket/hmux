//! Coverage for [`crate::client`] — constants and pure helpers.
//!
//! `client.rs` is at 0% line coverage and is dominated by `proc`/`imsg`
//! machinery that needs a live server or a real socket. The deterministic
//! surface is its long block of `pub const` definitions: message protocol
//! numbers, client flags, exit-reason ladders and the surrounding
//! socket/termios enumerations. All tests here are pure value checks that
//! pin the upstream wire values and ordering invariants without touching the
//! server.

use crate::client::{
    AF_UNIX, CLIENT_CONTROL, CLIENT_CONTROL_WAITEXIT, CLIENT_CONTROLCONTROL, CLIENT_EXIT_DETACHED,
    CLIENT_EXIT_DETACHED_HUP, CLIENT_EXIT_EXITED, CLIENT_EXIT_LOST_SERVER, CLIENT_EXIT_LOST_TTY,
    CLIENT_EXIT_MESSAGE_PROVIDED, CLIENT_EXIT_NONE, CLIENT_EXIT_SERVER_EXITED,
    CLIENT_EXIT_TERMINATED, CLIENT_LOGIN, CLIENT_NOSTARTSERVER, CLIENT_STARTSERVER,
    CMD_PARSE_ERROR, CMD_PARSE_SUCCESS, CMD_STARTSERVER, ECONNREFUSED, ENAMETOOLONG,
    IMSG_HEADER_SIZE, MAX_IMSGSIZE, MSG_COMMAND, MSG_DETACH, MSG_DETACHKILL, MSG_EXEC, MSG_EXIT,
    MSG_EXITED, MSG_EXITING, MSG_FLAGS, MSG_IDENTIFY_CLIENTPID, MSG_IDENTIFY_CWD,
    MSG_IDENTIFY_DONE, MSG_IDENTIFY_ENVIRON, MSG_IDENTIFY_FEATURES, MSG_IDENTIFY_FLAGS,
    MSG_IDENTIFY_LONGFLAGS, MSG_IDENTIFY_STDIN, MSG_IDENTIFY_STDOUT, MSG_IDENTIFY_TERM,
    MSG_IDENTIFY_TERMINFO, MSG_IDENTIFY_TTYNAME, MSG_LOCK, MSG_OLDSTDERR, MSG_OLDSTDIN,
    MSG_OLDSTDOUT, MSG_READ, MSG_READ_CANCEL, MSG_READ_DONE, MSG_READ_OPEN, MSG_READY, MSG_RESIZE,
    MSG_SHELL, MSG_SHUTDOWN, MSG_SUSPEND, MSG_UNLOCK, MSG_VERSION, MSG_WAKEUP, MSG_WRITE,
    MSG_WRITE_CLOSE, MSG_WRITE_OPEN, MSG_WRITE_READY, PANE_LINES_DOUBLE, PANE_LINES_HEAVY,
    PANE_LINES_NUMBER, PANE_LINES_SIMPLE, PANE_LINES_SINGLE, PANE_LINES_SPACES, PF_LOCAL,
    PROTOCOL_VERSION, SIGCHLD, SOCK_DGRAM, SOCK_STREAM, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO,
    STYLE_ALIGN_ABSOLUTE_CENTRE, STYLE_ALIGN_CENTRE, STYLE_ALIGN_DEFAULT, STYLE_ALIGN_LEFT,
    STYLE_ALIGN_RIGHT, STYLE_DEFAULT_BASE, STYLE_DEFAULT_POP, STYLE_DEFAULT_PUSH,
    STYLE_DEFAULT_SET, THEME_DARK, THEME_LIGHT, THEME_UNKNOWN,
};

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn consecutive(family: &[u32], base: u32) {
    for (i, &v) in family.iter().enumerate() {
        assert_eq!(
            v,
            base + i as u32,
            "index {i}: expected {} got {v}",
            base + i as u32
        );
    }
}

// ---------------------------------------------------------------------------
// client flags — stable bit assignments from tmux.h
// ---------------------------------------------------------------------------

#[test]
fn client_flag_constants_match_upstream_values() {
    assert_eq!(CMD_STARTSERVER, 0x1);
    assert_eq!(CLIENT_LOGIN, 0x2);
    assert_eq!(CLIENT_NOSTARTSERVER, 0x1000);
    assert_eq!(CLIENT_CONTROL, 0x2000);
    assert_eq!(CLIENT_CONTROLCONTROL, 0x4000);
    assert_eq!(CLIENT_STARTSERVER, 0x10000000);
    assert_eq!(CLIENT_CONTROL_WAITEXIT, 0x200000000);

    // all distinct bits (cast to u64 to avoid i32/u64 mismatch)
    assert_eq!((CLIENT_LOGIN as u64) & (CLIENT_NOSTARTSERVER as u64), 0);
    assert_eq!((CLIENT_CONTROL as u64) & (CLIENT_CONTROLCONTROL as u64), 0);
    assert_eq!((CLIENT_STARTSERVER as u64) & CLIENT_CONTROL_WAITEXIT, 0);
    assert_ne!(CLIENT_CONTROL, CLIENT_CONTROLCONTROL);
    // ordering
    assert!((CLIENT_LOGIN as u64) < (CLIENT_NOSTARTSERVER as u64));
    assert!((CLIENT_NOSTARTSERVER as u64) < (CLIENT_CONTROL as u64));
    assert!((CLIENT_CONTROL as u64) < (CLIENT_STARTSERVER as u64));
    assert!((CLIENT_STARTSERVER as u64) < CLIENT_CONTROL_WAITEXIT);
}

// ---------------------------------------------------------------------------
// protocol version and imsg framing
// ---------------------------------------------------------------------------

#[test]
fn protocol_and_imsg_constants_match_upstream() {
    assert_eq!(PROTOCOL_VERSION, 8);
    assert_eq!(MAX_IMSGSIZE, 16384);
    assert!(IMSG_HEADER_SIZE > 0);
    assert!(IMSG_HEADER_SIZE < MAX_IMSGSIZE as usize);
    // MSG_VERSION is the only value in the 12 slot
    assert_eq!(MSG_VERSION, 12);
    assert!(MSG_VERSION < MSG_IDENTIFY_FLAGS);
}

// ---------------------------------------------------------------------------
// MSG identify block — contiguous 100..112
// ---------------------------------------------------------------------------

#[test]
fn msg_identify_block_is_contiguous_from_100() {
    consecutive(
        &[
            MSG_IDENTIFY_FLAGS,
            MSG_IDENTIFY_TERM,
            MSG_IDENTIFY_TTYNAME,
            103, // MSG_IDENTIFY_OLDCWD
            MSG_IDENTIFY_STDIN,
            MSG_IDENTIFY_ENVIRON,
            MSG_IDENTIFY_DONE,
            MSG_IDENTIFY_CLIENTPID,
            MSG_IDENTIFY_CWD,
            MSG_IDENTIFY_FEATURES,
            MSG_IDENTIFY_STDOUT,
            MSG_IDENTIFY_LONGFLAGS,
            MSG_IDENTIFY_TERMINFO,
        ],
        100,
    );
    // spot checks against names
    assert_eq!(MSG_IDENTIFY_FLAGS, 100);
    assert_eq!(MSG_IDENTIFY_TERM, 101);
    assert_eq!(MSG_IDENTIFY_TTYNAME, 102);
    assert_eq!(MSG_IDENTIFY_DONE, 106);
    assert_eq!(MSG_IDENTIFY_TERMINFO, 112);
}

// ---------------------------------------------------------------------------
// MSG command block — contiguous 200..218
// ---------------------------------------------------------------------------

#[test]
fn msg_command_block_is_contiguous_from_200() {
    consecutive(
        &[
            MSG_COMMAND,
            MSG_DETACH,
            MSG_DETACHKILL,
            MSG_EXIT,
            MSG_EXITED,
            MSG_EXITING,
            MSG_LOCK,
            MSG_READY,
            MSG_RESIZE,
            MSG_SHELL,
            MSG_SHUTDOWN,
            MSG_OLDSTDERR,
            MSG_OLDSTDIN,
            MSG_OLDSTDOUT,
            MSG_SUSPEND,
            MSG_UNLOCK,
            MSG_WAKEUP,
            MSG_EXEC,
            MSG_FLAGS,
        ],
        200,
    );
    assert_eq!(MSG_COMMAND, 200);
    assert_eq!(MSG_FLAGS, 218);
    assert!(MSG_COMMAND < MSG_FLAGS);
    assert!(MSG_FLAGS < MSG_READ_OPEN);
}

// ---------------------------------------------------------------------------
// MSG file-transfer block — contiguous 300..307
// ---------------------------------------------------------------------------

#[test]
fn msg_file_transfer_block_is_contiguous_from_300() {
    consecutive(
        &[
            MSG_READ_OPEN,
            MSG_READ,
            MSG_READ_DONE,
            MSG_WRITE_OPEN,
            MSG_WRITE,
            MSG_WRITE_READY,
            MSG_WRITE_CLOSE,
            MSG_READ_CANCEL,
        ],
        300,
    );
    assert_eq!(MSG_READ_OPEN, 300);
    assert_eq!(MSG_READ_CANCEL, 307);
}

// ---------------------------------------------------------------------------
// CLIENT_EXIT_* reasons — consecutive ladder 0..8
// ---------------------------------------------------------------------------

#[test]
fn client_exit_reason_constants_form_ladder_from_zero() {
    consecutive(
        &[
            CLIENT_EXIT_NONE,
            CLIENT_EXIT_DETACHED,
            CLIENT_EXIT_DETACHED_HUP,
            CLIENT_EXIT_LOST_TTY,
            CLIENT_EXIT_TERMINATED,
            CLIENT_EXIT_LOST_SERVER,
            CLIENT_EXIT_EXITED,
            CLIENT_EXIT_SERVER_EXITED,
            CLIENT_EXIT_MESSAGE_PROVIDED,
        ],
        0,
    );
    assert_eq!(CLIENT_EXIT_NONE, 0);
    assert_eq!(CLIENT_EXIT_MESSAGE_PROVIDED, 8);
    assert!(CLIENT_EXIT_NONE < CLIENT_EXIT_DETACHED);
    assert!(CLIENT_EXIT_DETACHED < CLIENT_EXIT_LOST_TTY);
}

// ---------------------------------------------------------------------------
// socket / address / errno constants
// ---------------------------------------------------------------------------

#[test]
fn socket_and_errno_constants_match_linux_values() {
    assert_eq!(SOCK_STREAM, 1);
    assert_eq!(SOCK_DGRAM, 2);
    assert!(SOCK_STREAM < SOCK_DGRAM);
    assert_eq!(PF_LOCAL, 1);
    assert_eq!(AF_UNIX, PF_LOCAL);
    assert_eq!(STDIN_FILENO, 0);
    assert_eq!(STDOUT_FILENO, 1);
    assert_eq!(STDERR_FILENO, 2);
    assert_eq!(ENAMETOOLONG, 36);
    assert_eq!(ECONNREFUSED, 111);
    assert_eq!(SIGCHLD, 17);
}

// ---------------------------------------------------------------------------
// pane / style / theme ladders re-exported through client.rs
// ---------------------------------------------------------------------------

#[test]
fn pane_style_and_theme_ladders_are_consecutive() {
    consecutive(
        &[
            PANE_LINES_SINGLE,
            PANE_LINES_DOUBLE,
            PANE_LINES_HEAVY,
            PANE_LINES_SIMPLE,
            PANE_LINES_NUMBER,
            PANE_LINES_SPACES,
        ],
        0,
    );
    consecutive(
        &[
            STYLE_ALIGN_DEFAULT,
            STYLE_ALIGN_LEFT,
            STYLE_ALIGN_CENTRE,
            STYLE_ALIGN_RIGHT,
            STYLE_ALIGN_ABSOLUTE_CENTRE,
        ],
        0,
    );
    consecutive(
        &[
            STYLE_DEFAULT_BASE,
            STYLE_DEFAULT_PUSH,
            STYLE_DEFAULT_POP,
            STYLE_DEFAULT_SET,
        ],
        0,
    );
    consecutive(&[THEME_UNKNOWN, THEME_LIGHT, THEME_DARK], 0);
    assert_eq!(THEME_UNKNOWN, 0);
    assert_eq!(THEME_DARK, 2);
}

// ---------------------------------------------------------------------------
// cmd_parse sentinels
// ---------------------------------------------------------------------------

#[test]
fn cmd_parse_status_constants_match_headers() {
    assert_eq!(CMD_PARSE_ERROR, 0);
    assert_eq!(CMD_PARSE_SUCCESS, 1);
    assert_ne!(CMD_PARSE_ERROR, CMD_PARSE_SUCCESS);
}
