//! Coverage for pure helpers in [`crate::cmd`] and [`crate::args`].
//!
//! `cmd.rs` contributes `cmd_*_argv` packing, copying and stringifying plus
//! `cmd_template_replace`; `arguments.rs` contributes `args_escape` and the
//! lightweight `args_*` accessors. All paths here are exercised without a live
//! server or the command table, so no [`globals`] guard is needed except where
//! noted.

use crate::args::{
    args_create, args_escape, args_free, args_get_str, args_has, args_print, args_set,
};
use crate::cmd::{cmd_pack_argv, cmd_stringify_argv, cmd_template_replace, cmd_unpack_argv};
use crate::tests::test_fixtures::seen;
use crate::tests::test_fixtures::seen_str;
use ::core::ffi::c_int;
use ::std::ffi::{CStr, CString};

// ---------------------------------------------------------------------------
// cmd_append_argv / cmd_prepend_argv
// ---------------------------------------------------------------------------

#[test]
fn cmd_append_argv_builds_array_in_order() {
    let mut argv: Vec<CString> = Vec::new();
    cmd_append_argv(&mut argv, c"first");
    cmd_append_argv(&mut argv, c"second");
    cmd_append_argv(&mut argv, c"third");
    assert_eq!(argv.len(), 3);
    assert_eq!(argv[0].as_bytes(), b"first");
    assert_eq!(argv[1].as_bytes(), b"second");
    assert_eq!(argv[2].as_bytes(), b"third");
}

#[test]
fn cmd_prepend_argv_puts_arg_at_front() {
    let mut argv: Vec<CString> = Vec::new();
    cmd_append_argv(&mut argv, c"second");
    cmd_append_argv(&mut argv, c"third");
    cmd_prepend_argv(&mut argv, c"first");
    assert_eq!(argv.len(), 3);
    assert_eq!(argv[0].as_bytes(), b"first");
    assert_eq!(argv[1].as_bytes(), b"second");
    assert_eq!(argv[2].as_bytes(), b"third");
}

#[test]
fn vec_cstring_clone_duplicates_strings_independently() {
    let mut argv: Vec<CString> = Vec::new();
    cmd_append_argv(&mut argv, c"alpha");
    cmd_append_argv(&mut argv, c"beta");
    let copy = argv.clone();
    assert_eq!(copy[0].as_bytes(), b"alpha");
    assert_eq!(copy[1].as_bytes(), b"beta");
}

// ---------------------------------------------------------------------------
// cmd_pack_argv / cmd_unpack_argv round-trip
// ---------------------------------------------------------------------------

#[test]
fn cmd_pack_and_unpack_roundtrip_single_and_multiple() {
    unsafe {
        // single arg
        let mut argv: Vec<CString> = Vec::new();
        cmd_append_argv(&mut argv, c"hello");
        let mut buf = vec![0; 64];
        let rc = cmd_pack_argv(&argv, &mut buf);
        assert_eq!(rc, 0);
        // packed is NUL-terminated "hello\0"
        assert_eq!(seen(buf.as_mut_ptr().cast()), "hello");
        let unpacked = cmd_unpack_argv(&mut buf, argv.len() as c_int);
        let unpacked = unpacked.unwrap();
        assert_eq!(unpacked[0].as_bytes(), b"hello");

        // multiple args with empty in middle
        let mut argv2: Vec<CString> = Vec::new();
        cmd_append_argv(&mut argv2, c"a");
        cmd_append_argv(&mut argv2, c"");
        cmd_append_argv(&mut argv2, c"c");
        let mut buf2 = vec![0; 64];
        let rc3 = cmd_pack_argv(&argv2, &mut buf2);
        assert_eq!(rc3, 0);
        let unpacked2 = cmd_unpack_argv(&mut buf2, argv2.len() as c_int);
        let unpacked2 = unpacked2.unwrap();
        assert_eq!(unpacked2[0].as_bytes(), b"a");
        assert_eq!(unpacked2[1].as_bytes(), b"");
        assert_eq!(unpacked2[2].as_bytes(), b"c");

        // zero argc is a no-op
        let mut buf0 = vec![0; 8];
        assert_eq!(cmd_pack_argv(&[], &mut buf0), 0);
        let out = cmd_unpack_argv(&mut buf0, 0);
        assert_eq!(out, Some(Vec::new()));
    }
}

#[test]
fn cmd_pack_fails_when_buffer_too_small_and_unpack_rejects_bad_len() {
    let mut argv: Vec<CString> = Vec::new();
    cmd_append_argv(&mut argv, c"hello");
    cmd_append_argv(&mut argv, c"world");
    // need 12 bytes (6+6), give 5
    let mut tiny = vec![0; 5];
    let rc = cmd_pack_argv(&argv, &mut tiny);
    assert_eq!(rc, -1);
    // packed ok but truncated len for unpack -> None
    let mut buf = vec![0; 64];
    assert_eq!(cmd_pack_argv(&argv, &mut buf), 0);
    // len == 0 with argc>0
    assert_eq!(cmd_unpack_argv(&mut buf[..0], argv.len() as c_int), None);
    // argc out of range
    assert_eq!(cmd_unpack_argv(&mut buf, 1001), None);
    assert_eq!(cmd_unpack_argv(&mut buf, -1), None);
}

// ---------------------------------------------------------------------------
// cmd_stringify_argv
// ---------------------------------------------------------------------------

#[test]
fn cmd_stringify_argv_joins_with_escaping() {
    // empty
    let s = cmd_stringify_argv(&[]);
    let s = s.to_string_lossy();
    assert_eq!(s, "");
    // simple
    let mut argv: Vec<CString> = Vec::new();
    cmd_append_argv(&mut argv, c"foo");
    cmd_append_argv(&mut argv, c"bar");
    let s2 = cmd_stringify_argv(&argv);
    let s2 = s2.to_string_lossy();
    assert_eq!(s2, "foo bar");
    // needs escaping (space)
    let mut argv2: Vec<CString> = Vec::new();
    cmd_append_argv(&mut argv2, c"a b");
    let s3 = cmd_stringify_argv(&argv2);
    let s3 = s3.to_string_lossy();
    // args_escape puts quotes around strings with spaces
    assert!(s3.contains("a b"), "got {s3:?}");
    assert!(s3.contains('"') || s3.contains('\''), "got {s3:?}");
}

// ---------------------------------------------------------------------------
// cmd_template_replace — %1 and %% expansions
// ---------------------------------------------------------------------------

#[test]
fn cmd_template_replace_numbered_and_double_percent() {
    {
        // no percent -> copy
        let s = cmd_template_replace(c"nothing", c"arg", 1)
            .to_string_lossy()
            .into_owned();
        assert_eq!(s, "nothing");
        // %1 replaced when idx matches
        let s1 = cmd_template_replace(c"echo %1", c"hello", 1)
            .to_string_lossy()
            .into_owned();
        assert_eq!(s1, "echo hello");
        let s2 = cmd_template_replace(c"echo %1", c"hello", 2)
            .to_string_lossy()
            .into_owned();
        assert_eq!(s2, "echo %1");
        // %% replaced once
        let s3 = cmd_template_replace(c"echo %%", c"hello", 1)
            .to_string_lossy()
            .into_owned();
        assert_eq!(s3, "echo hello");
        let s4 = cmd_template_replace(c"%% and %%", c"x", 1)
            .to_string_lossy()
            .into_owned();
        assert_eq!(s4, "x and %%");
        // quoted form escapes special chars
        let s5 = cmd_template_replace(c"echo %1%", c"a;b", 1)
            .to_string_lossy()
            .into_owned();
        assert_eq!(s5, "echo a\\;b");
        let s6 = cmd_template_replace(c"echo %1%", c"a\"b", 1)
            .to_string_lossy()
            .into_owned();
        assert!(s6.contains("\\\""), "got {s6:?}");
    }
}

// ---------------------------------------------------------------------------
// arguments.rs — args_escape and lightweight accessors
// ---------------------------------------------------------------------------

#[test]
fn args_escape_empty_tilde_and_quotes() {
    // empty string -> ''
    let e0 = args_escape(c"").to_string_lossy().into_owned();
    assert_eq!(e0, "''");
    // single tilde needs escaping
    let e1 = args_escape(c"~").to_string_lossy().into_owned();
    assert_eq!(e1, "\\~");
    // leading tilde with more content also escapes tilde
    let e2 = args_escape(c"~/foo").to_string_lossy().into_owned();
    assert!(e2.starts_with("\\~") || e2.starts_with('"'), "got {e2:?}");
    // string with space needs quoting
    let e3 = args_escape(c"a b").to_string_lossy().into_owned();
    assert!(e3.contains("a b"));
    assert!(e3.starts_with('"') || e3.starts_with('\''), "got {e3:?}");
    // string needing double quotes
    let e4 = args_escape(c"a#b").to_string_lossy().into_owned();
    assert!(e4.starts_with('"'), "got {e4:?}");
    assert!(e4.contains("a#b"));
    // plain word no quoting
    let e5 = args_escape(c"hello").to_string_lossy().into_owned();
    assert_eq!(e5, "hello");
}

#[test]
fn args_has_get_and_print_roundtrip() {
    unsafe {
        let args = Box::into_raw(args_create());
        assert!(!args.is_null());
        assert_eq!(args_has(&*args, b'a'), 0);
        assert!(args_get_str(&*args, b'a').is_none());
        assert_eq!(args_print(&*args).to_string_lossy(), "");
        // set flag -a without value
        args_set(&mut *args, b'a', None, 0);
        assert_eq!(args_has(&*args, b'a'), 1);
        assert!(args_get_str(&*args, b'a').is_none());
        let printed1 = args_print(&*args).to_string_lossy().into_owned();
        assert_eq!(printed1, "-a");
        // set flag -b with string value
        let mut v = Box::new(crate::types::args_value_t::default());
        v.value = crate::types::ArgsValue::String(CString::new("val").unwrap());
        args_set(&mut *args, b'b', Some(v), 0);
        assert_eq!(args_has(&*args, b'b'), 1);
        assert_eq!(seen_str(args_get_str(&*args, b'b')), "val");
        let printed2 = args_print(&*args).to_string_lossy().into_owned();
        assert!(printed2.contains("-b"), "got {printed2:?}");
        assert!(printed2.contains("val"), "got {printed2:?}");
        assert_eq!(crate::args::args_count(&*args), 0);
        assert!(crate::args::args_value(&*args, 0).is_none());
        args_free(Box::from_raw(args));
    }
}

#[test]
fn args_print_with_positional_and_multiple_flags() {
    unsafe {
        let args = Box::into_raw(Box::new(
            crate::RustArguments::from_strings(&[c"pos"]),
        ));
        let printed = args_print(&*args).to_string_lossy().into_owned();
        assert!(printed.contains("pos"), "got {printed:?}");
        assert_eq!(crate::args::args_count(&*args), 1);
        assert_eq!(
            seen_str(crate::args::args_string_str(&*args, 0)),
            "pos"
        );
        assert!(crate::args::args_value(&*args, 1).is_none());
        args_free(Box::from_raw(args));
    }
}

pub(crate) fn cmd_prepend_argv(argv: &mut Vec<CString>, arg: &CStr) {
    argv.insert(0, arg.to_owned());
}

pub(crate) fn cmd_append_argv(argv: &mut Vec<CString>, arg: &CStr) {
    argv.push(arg.to_owned());
}
