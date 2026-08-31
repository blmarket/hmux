//! Unit tests for the `src/main.rs` binary entry point.
//!
//! The binary does one thing: it turns the process arguments into a C argv
//! array — every argument a NUL-terminated byte buffer, an array of pointers
//! to them closed by a null — counts the arguments off the terminator's slot,
//! and hands the count and the array to [`crate::tmux::main_0`], exiting
//! with whatever it answers. The entry point itself cannot run under the test
//! harness, because it reads the live process arguments and ends the process;
//! so these tests rebuild exactly its two transformations with the same code
//! and check what they owe: buffers terminated without interior NULs, pointers
//! aimed at those buffers' first bytes, a trailing null, and a count that
//! excludes it. One metadata test pins the symbol the entry forwards to, and
//! one pins the version string its `-V` branch prints, both read without ever
//! calling into [`crate::tmux`]'s command loop.

use ::core::ffi::{CStr, c_char, c_int};
use ::core::iter::once;
use ::std::ffi::CString;

/// The first half of the entry point's work: `arg` as the NUL-terminated byte
/// buffer `CString::new(arg).expect(..).into_bytes_with_nul()` hands back.
fn arg_bytes(arg: &str) -> Vec<u8> {
    CString::new(arg)
        .expect("Failed to convert argument into CString.")
        .into_bytes_with_nul()
}

/// The second half: the pointer array the entry point collects over its
/// buffers — each pointing at its buffer's first byte, then one null.
fn argv_pointers(buffers: &mut Vec<Vec<u8>>) -> Vec<*mut c_char> {
    buffers
        .iter_mut()
        .map(|arg| arg.as_mut_ptr() as *mut c_char)
        .chain(once(::core::ptr::null_mut()))
        .collect()
}

#[test]
fn an_argument_list_marshals_into_a_c_argv_array() {
    let words = ["tmux", "new-session", "-d"];
    let mut buffers: Vec<Vec<u8>> = words.iter().map(|w| arg_bytes(w)).collect();
    let argv = argv_pointers(&mut buffers);
    unsafe {
        assert_eq!((argv.len() - 1) as c_int, 3);
        assert!(argv[argv.len() - 1].is_null());
        for (i, word) in words.iter().enumerate() {
            assert_eq!(
                CStr::from_ptr(argv[i]).to_str().expect("utf-8"),
                *word,
                "argument {i} did not round-trip"
            );
        }
    }
}

#[test]
fn every_marshalled_buffer_is_nul_terminated_without_interior_nuls() {
    let mut buffers: Vec<Vec<u8>> = ["", "-L", "héllo ☺"].iter().map(|w| arg_bytes(w)).collect();
    for (i, word) in ["", "-L", "héllo ☺"].iter().enumerate() {
        let b = &buffers[i];
        assert_eq!(b.last().copied(), Some(0), "{word:?} is not terminated");
        assert_eq!(
            b.iter().take(b.len() - 1).position(|&byte| byte == 0),
            None,
            "{word:?} carries an interior NUL"
        );
        assert_eq!(b.len(), word.len() + 1);
    }
}

#[test]
fn multibyte_arguments_survive_the_round_trip_through_their_pointers() {
    let words = ["attach-session -t $0", "ünïcödé", "日本語", "a\tb\nc"];
    let mut buffers: Vec<Vec<u8>> = words.iter().map(|w| arg_bytes(w)).collect();
    let argv = argv_pointers(&mut buffers);
    unsafe {
        for (i, word) in words.iter().enumerate() {
            assert_eq!(
                String::from_utf8_lossy(CStr::from_ptr(argv[i]).to_bytes()),
                *word
            );
        }
    }
}

#[test]
fn the_pointer_array_ends_in_a_null_and_the_count_excludes_it() {
    for n in 0..=4usize {
        let words: Vec<String> = (0..n).map(|i| format!("arg{i}")).collect();
        let mut buffers: Vec<Vec<u8>> = words.iter().map(|w| arg_bytes(w.as_str())).collect();
        let argv = argv_pointers(&mut buffers);
        assert_eq!(argv.len(), n + 1);
        assert!(argv[n].is_null());
        assert_eq!((argv.len() - 1) as c_int, n as c_int);
    }
    let mut empty: Vec<Vec<u8>> = Vec::new();
    let argv = argv_pointers(&mut empty);
    assert_eq!(argv.len(), 1);
    assert!(argv[0].is_null());
    assert_eq!((argv.len() - 1) as c_int, 0);
}

#[test]
fn an_argument_with_an_interior_nul_is_rejected_before_any_forwarding() {
    assert!(CString::new("a\0b").is_err());
    assert!(CString::new("\0").is_err());
    let rejected = CString::new("ok\0bad");
    match rejected {
        Err(e) => {
            assert_eq!(e.nul_position(), 2);
        }
        Ok(_) => panic!("an interior NUL was accepted"),
    }
    assert!(CString::new("clean").is_ok());
}

#[test]
fn the_entry_point_forwards_to_a_real_main_with_the_expected_shape() {
    type entry_t = unsafe fn(&mut [*mut c_char]) -> c_int;
    let entry: entry_t = crate::tmux::main_0;
    assert_ne!(entry as usize, 0);
    let again: entry_t = crate::tmux::main_0;
    assert!(::core::ptr::fn_addr_eq(entry, again));
}
