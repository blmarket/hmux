//! Unit tests for [`crate::tmux`] helpers that the daemon reaches from every
//! corner — name cleaning, shell validation, `argv0` building and the small
//! wrappers around timers and descriptors. None of them needs a server, a
//! session or a pane; the checks are pure string or descriptor work, so an
//! ordinary unit test can pin the exact branch each input takes and the
//! absence of side effects elsewhere.

use crate::compat::getprogname;
use crate::osdep_linux::osdep_event_init;
use crate::tmux::{
    check_name, checkshell, clean_name, find_cwd, find_home, get_timer, getversion, setblocking,
    shell_argv0, sig2name,
};
use ::core::ffi::{CStr, c_char, c_int};
use ::core::ptr::null;
use ::std::ffi::CString;

unsafe fn clean(s: &CStr, untrusted: c_int) -> Option<String> {
    unsafe { clean_name(s.as_ptr(), untrusted) }.map(|p| p.to_string_lossy().into_owned())
}

#[test]
fn checkshell_refuses_null_relative_and_missing_paths() {
    unsafe {
        assert_eq!(checkshell(null::<c_char>()), 0);
        assert_eq!(checkshell(c"".as_ptr()), 0);
        assert_eq!(checkshell(c"relative/bin/sh".as_ptr()), 0);
        assert_eq!(checkshell(c"/no/such/file/xyz".as_ptr()), 0);
    }
}

#[test]
fn checkshell_accepts_an_executable_absolute_path() {
    unsafe {
        assert_eq!(checkshell(c"/bin/sh".as_ptr()), 1);
        let has_bash = ::std::path::Path::new("/bin/bash").exists();
        if has_bash {
            assert_eq!(checkshell(c"/bin/bash".as_ptr()), 1);
        }
    }
}

#[test]
fn checkshell_refuses_its_own_program() {
    unsafe {
        let prog = CStr::from_ptr(getprogname());
        let prog_bytes = prog.to_bytes();
        let prog_name = prog_bytes
            .rsplit(|&b| b == b'/')
            .next()
            .unwrap_or(prog_bytes);
        let fake = CString::new(format!("/tmp/{}", String::from_utf8_lossy(prog_name))).unwrap();
        assert_eq!(checkshell(fake.as_ptr()), 0);
    }
}

#[test]
fn check_name_validates_utf8_only() {
    unsafe {
        assert_eq!(check_name(c"hello".as_ptr()), 1);
        assert_eq!(check_name(c"".as_ptr()), 1);
        assert_eq!(check_name(c"hello world".as_ptr()), 1);
        let bad = CString::new(b"\xff\xfe".as_slice()).unwrap();
        assert_eq!(check_name(bad.as_ptr()), 0);
    }
}

#[test]
fn clean_name_rejects_invalid_utf8() {
    unsafe {
        let bad = CString::new(b"\xff\xfe".as_slice()).unwrap();
        assert!(clean(&bad, 0).is_none());
        assert!(clean(&bad, 1).is_none());
    }
}

#[test]
fn clean_name_escapes_control_bytes() {
    unsafe {
        assert!(clean(c"a\nb\tc\x07d", 0).is_none());
        assert!(clean(c"hello", 0).is_some());
        assert_eq!(clean(c"hello", 0).unwrap(), "hello");
        assert!(clean(c"hi\x01there", 0).is_none());
    }
}

#[test]
fn clean_name_disarms_hash_paren_when_untrusted() {
    unsafe {
        let escaped = clean(c"#(echo hi)", 1).unwrap();
        assert_eq!(escaped, "_(echo hi)");
        let trusted = clean(c"#(echo hi)", 0).unwrap();
        assert_eq!(trusted, "#(echo hi)");
        assert_eq!(clean(c"#(a) #(b)", 1).unwrap(), "_(a) _(b)");
        assert_eq!(clean(c"foo #(bar)", 1).unwrap(), "foo _(bar)");
    }
}

#[test]
fn clean_name_keeps_other_hash_uses() {
    unsafe {
        assert_eq!(clean(c"#foo", 1).unwrap(), "#foo");
        assert_eq!(clean(c"foo#bar", 1).unwrap(), "foo#bar");
        assert_eq!(clean(c"#", 1).unwrap(), "#");
    }
}

#[test]
fn shell_argv0_builds_login_and_plain_names() {
    unsafe {
        assert_eq!(
            shell_argv0(c"/bin/bash".as_ptr(), 0).to_str().unwrap(),
            "bash"
        );
        assert_eq!(
            shell_argv0(c"/bin/bash".as_ptr(), 1).to_str().unwrap(),
            "-bash"
        );

        assert_eq!(shell_argv0(c"bash".as_ptr(), 0).to_str().unwrap(), "bash");
        assert_eq!(shell_argv0(c"bash".as_ptr(), 1).to_str().unwrap(), "-bash");

        assert_eq!(
            shell_argv0(c"/usr/local/bin/zsh".as_ptr(), 0)
                .to_str()
                .unwrap(),
            "zsh"
        );
        assert_eq!(
            shell_argv0(c"/usr/local/bin/zsh".as_ptr(), 1)
                .to_str()
                .unwrap(),
            "-zsh"
        );

        assert_eq!(shell_argv0(c"/bin/".as_ptr(), 0).to_str().unwrap(), "/bin/");
        assert_eq!(
            shell_argv0(c"/bin/".as_ptr(), 1).to_str().unwrap(),
            "-/bin/"
        );

        assert_eq!(shell_argv0(c"/".as_ptr(), 0).to_str().unwrap(), "/");
    }
}

#[test]
fn getversion_is_three_seven_b() {
    unsafe {
        assert_eq!(CStr::from_ptr(getversion()).to_str().unwrap(), "3.7b");
    }
}

#[test]
fn sig2name_writes_the_number_in_decimal() {
    assert_eq!(sig2name(9), c"9".to_owned());
    assert_eq!(sig2name(15), c"15".to_owned());
    assert_eq!(sig2name(0), c"0".to_owned());
    assert_eq!(sig2name(2), c"2".to_owned());
}

#[test]
fn find_cwd_returns_a_directory() {
    let p = find_cwd().expect("a working directory");
    let s = p.to_str().unwrap();
    assert!(s.starts_with('/'));
    assert!(::std::path::Path::new(s).is_dir());
    assert_eq!(find_cwd(), Some(p));
}

#[test]
fn find_home_returns_home_when_available() {
    let p = find_home().expect("a home directory");
    assert!(!p.to_bytes().is_empty());
}

#[test]
fn get_timer_is_monotonic() {
    {
        let a = get_timer();
        ::std::thread::sleep(::std::time::Duration::from_millis(5));
        let b = get_timer();
        assert!(b >= a);
        assert!(b - a < 10_000);
    }
}

#[test]
fn setblocking_toggles_nonblock() {
    unsafe {
        let mut fds = [-1 as c_int; 2];
        assert_eq!(::libc::pipe(fds.as_mut_ptr()), 0);
        let fd = fds[0];
        let before = ::libc::fcntl(fd, ::libc::F_GETFL);
        assert_ne!(before, -1);
        setblocking(fd, 0);
        let nonblock = ::libc::fcntl(fd, ::libc::F_GETFL);
        assert_ne!(nonblock & ::libc::O_NONBLOCK, 0);
        setblocking(fd, 1);
        let block = ::libc::fcntl(fd, ::libc::F_GETFL);
        assert_eq!(block & ::libc::O_NONBLOCK, 0);
        setblocking(-1, 0);
        setblocking(-1, 1);
        ::libc::close(fds[0]);
        ::libc::close(fds[1]);
    }
}

#[test]
fn osdep_event_init_initialises_reactor_and_cleans_env() {
    unsafe {
        ::std::env::remove_var("EVENT_NOEPOLL");
        let _base = osdep_event_init();
        assert!(::std::env::var_os("EVENT_NOEPOLL").is_none());
    }
}

#[test]
fn check_name_and_clean_name_agree_on_validity() {
    unsafe {
        let cases: &[&CStr] = &[c"ok", c"", c"with space", c"a#b"];
        for c in cases {
            assert_eq!(check_name(c.as_ptr()), 1);
            assert!(clean(c, 0).is_some());
        }
        let bad = CString::new(b"\x80bad".as_slice()).unwrap();
        assert_eq!(check_name(bad.as_ptr()), 0);
        assert!(clean(&bad, 0).is_none());
    }
}
