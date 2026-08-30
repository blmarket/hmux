//! Unit tests for [`crate::cmd::cmd_kill_server`] — the two entries the
//! server's life cycle hangs on, `kill-server` and `start-server`, the
//! argument-parsing, target-finding and return-value constants they are
//! written against, and [`crate::cmd::cmd_kill_server::cmd_kill_server_exec`]
//! behind both their `exec` pointers.
//!
//! That one function is shared by both commands and picks its behaviour by
//! comparing the running command's entry against the `kill-server` entry:
//! `kill-server` sends `SIGTERM` to the process itself, `start-server` sends
//! nothing anywhere. The signalling branch is reached under a counting
//! handler swapped in with `sigaction` and put back again when the test ends,
//! so the signal arrives as usual but only ticks an atomic, whichever thread
//! the kernel picks to run it on, and the disposition the process had before
//! the test keeps standing even if an assertion fails. The two signal tests
//! take turns at the swap through one mutex, since cargo runs the tests on
//! parallel threads and a signal disposition belongs to the whole process.

use crate::cmd::cmd_kill_server::{
    ARGS_PARSE_COMMANDS, ARGS_PARSE_COMMANDS_OR_STRING, ARGS_PARSE_INVALID, ARGS_PARSE_STRING,
    CMD_FIND_PANE, CMD_FIND_SESSION, CMD_FIND_WINDOW, CMD_RETURN_ERROR, CMD_RETURN_NORMAL,
    CMD_RETURN_STOP, CMD_RETURN_WAIT, CMD_STARTSERVER, SIGTERM, cmd_kill_server_entry,
    cmd_start_server_entry,
};
use crate::ffi::sigemptyset;
use crate::tests::test_fixtures::{Args, Item, globals, zeroed};
use ::core::ffi::c_int;
use ::core::ptr::null_mut;
use ::libc::sigaction;
use ::std::sync::Mutex;
use ::std::sync::MutexGuard;
use ::std::sync::atomic::{AtomicUsize, Ordering};
use ::std::time::{Duration, Instant};

/// How many `SIGTERM`s have been delivered since the counting handler went
/// in. Ticked from whatever thread the kernel hands the signal to, so it is
/// the only thing the handler touches.
static SIGNALLED: AtomicUsize = AtomicUsize::new(0);

unsafe extern "C" fn count_sigterm(_sig: c_int) {
    SIGNALLED.fetch_add(1, Ordering::SeqCst);
}

/// A turn at the `SIGTERM` disposition, replaced by [`count_sigterm`] for the
/// length of the test and given back — the old one, whatever it was — even if
/// the test panics.
struct CountingSigterm {
    saved: Box<sigaction>,
}

impl CountingSigterm {
    unsafe fn install() -> CountingSigterm {
        unsafe {
            let mut act = zeroed::<sigaction>();
            assert_eq!(sigemptyset(&raw mut act.sa_mask), 0);
            act.sa_flags = ::libc::SA_RESTART;
            act.sa_sigaction = count_sigterm as *const () as usize;
            let mut saved = zeroed::<sigaction>();
            assert_eq!(
                crate::ffi::sigaction(SIGTERM, &raw const *act, &raw mut *saved),
                0
            );
            CountingSigterm { saved }
        }
    }
}

impl Drop for CountingSigterm {
    fn drop(&mut self) {
        unsafe { crate::ffi::sigaction(SIGTERM, &raw const *self.saved, null_mut()) };
    }
}

/// A turn at the signal-disposition swap itself, so two signal tests cannot
/// race each other's handlers.
fn signal_turn() -> MutexGuard<'static, ()> {
    static TURN: Mutex<()> = Mutex::new(());
    TURN.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Blocks until the counting handler has ticked at least once, answering how
/// many times it has.
fn waits_for_the_signal() -> usize {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let seen = SIGNALLED.load(Ordering::SeqCst);
        if seen > 0 {
            return seen;
        }
        assert!(Instant::now() < deadline, "the SIGTERM never arrived");
        ::std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn the_entries_describe_two_no_argument_commands_sharing_one_exec() {
    unsafe {
        let kill_e = &raw const cmd_kill_server_entry;
        let start_e = &raw const cmd_start_server_entry;
        assert_ne!(kill_e, start_e);

        assert_eq!((*kill_e).name.to_bytes(), b"kill-server");
        assert!((*kill_e).alias.is_none());
        assert_eq!((*start_e).name.to_bytes(), b"start-server");
        assert_eq!(
            (*start_e).alias.expect("the entry has an alias").to_bytes(),
            b"start"
        );

        for e in [kill_e, start_e] {
            assert_eq!((*e).args.template.to_bytes(), b"");
            assert_eq!((*e).args.lower, 0);
            assert_eq!((*e).args.upper, 0);
            assert!((*e).args.cb.is_none());

            assert!((*e).usage.to_bytes().is_empty());

            for flag in [&raw const (*e).source, &raw const (*e).target] {
                assert_eq!((*flag).flag, 0);
                assert_eq!((*flag).type_0, CMD_FIND_PANE);
                assert_eq!((*flag).flags, 0);
            }
        }

        assert_eq!((*kill_e).flags, 0);
        assert_eq!((*start_e).flags, CMD_STARTSERVER);

        assert!(::core::ptr::fn_addr_eq((*kill_e).exec, (*start_e).exec));
    }
}

#[test]
fn the_constants_pin_the_values_the_command_table_and_queue_read_back() {
    assert_eq!(ARGS_PARSE_INVALID, 0);
    assert_eq!(ARGS_PARSE_STRING, 1);
    assert_eq!(ARGS_PARSE_COMMANDS_OR_STRING, 2);
    assert_eq!(ARGS_PARSE_COMMANDS, 3);

    assert_eq!(CMD_FIND_PANE, 0);
    assert_eq!(CMD_FIND_WINDOW, 1);
    assert_eq!(CMD_FIND_SESSION, 2);

    assert_eq!(CMD_RETURN_ERROR, -1);
    assert_eq!(CMD_RETURN_NORMAL, 0);
    assert_eq!(CMD_RETURN_WAIT, 1);
    assert_eq!(CMD_RETURN_STOP, 2);

    assert_eq!(SIGTERM, 15);
    assert_eq!(CMD_STARTSERVER, 0x1);
}

#[test]
fn parsing_resolves_both_names_and_the_alias_to_these_entries() {
    let _guard = globals();
    unsafe {
        let kill_args = Args::parse(c"kill-server");
        assert!(::core::ptr::eq(
            (*kill_args.cmd()).entry,
            &cmd_kill_server_entry
        ));

        let start_args = Args::parse(c"start-server");
        assert!(::core::ptr::eq(
            (*start_args.cmd()).entry,
            &cmd_start_server_entry
        ));

        let alias_args = Args::parse(c"start");
        assert!(::core::ptr::eq(
            (*alias_args.cmd()).entry,
            &cmd_start_server_entry
        ));
    }
}

/// `start-server` runs the comparison against its own entry, loses, and comes
/// back normal having sent nothing at all — the grace period gives any stray
/// delivery time to show up in the count before the test believes in it.
#[test]
fn exec_of_start_server_answers_normal_without_signalling_anything() {
    let _guard = globals();
    let _turn = signal_turn();
    let _disposition = unsafe { CountingSigterm::install() };
    SIGNALLED.store(0, Ordering::SeqCst);

    let mut item = Item::new().with_args(c"start-server");
    let rv = unsafe { (cmd_start_server_entry.exec)(&*item.cmd(), item.ptr()) };
    assert_eq!(rv, CMD_RETURN_NORMAL);

    ::std::thread::sleep(Duration::from_millis(100));
    assert_eq!(SIGNALLED.load(Ordering::SeqCst), 0);
}

/// `kill-server` runs the same comparison, wins, and signals its own process
/// with `SIGTERM` before answering normal — exactly once, and to this
/// process, which is what the counting handler's tick bears out.
#[test]
fn exec_of_kill_server_answers_normal_after_signalling_itself_once() {
    let _guard = globals();
    let _turn = signal_turn();
    let _disposition = unsafe { CountingSigterm::install() };
    SIGNALLED.store(0, Ordering::SeqCst);

    let mut item = Item::new().with_args(c"kill-server");
    let rv = unsafe { (cmd_kill_server_entry.exec)(&*item.cmd(), item.ptr()) };
    assert_eq!(rv, CMD_RETURN_NORMAL);

    assert_eq!(waits_for_the_signal(), 1);
}
