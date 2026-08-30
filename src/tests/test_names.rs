use super::*;
use crate::options::{options_create_boxed, options_default, options_set_number};
use crate::options::options_table;
use crate::tests::test_fixtures::{globals, zeroed_pane, zeroed_window};
use ::core::ffi::{CStr, c_int};
use ::std::ffi::CString;

fn parse(input: &CStr) -> String {
    unsafe {
        parse_window_name(input.as_ptr())
            .to_str()
            .unwrap()
            .to_owned()
    }
}

fn blank_window() -> Box<window> {
    zeroed_window()
}

fn blank_pane() -> Box<window_pane> {
    zeroed_pane()
}

#[test]
fn a_plain_command_is_its_own_name() {
    assert_eq!(parse(c"vi"), "vi");
    assert_eq!(parse(c"vi file"), "vi");
}

#[test]
fn surrounding_quotes_are_dropped() {
    assert_eq!(parse(c"\"vi\""), "vi");
    assert_eq!(parse(c"\"vi file\""), "vi");
    assert_eq!(parse(c"vi\"x"), "vi");
}

#[test]
fn an_exec_prefix_is_skipped() {
    assert_eq!(parse(c"exec vi"), "vi");
    assert_eq!(parse(c"\"exec vi -R\""), "vi");
    assert_eq!(parse(c"execvi"), "execvi");
}

#[test]
fn leading_spaces_and_dashes_are_skipped() {
    assert_eq!(parse(c"  vi"), "vi");
    assert_eq!(parse(c"-vi"), "vi");
    assert_eq!(parse(c"- -vi"), "vi");
    assert_eq!(parse(c"-bash"), "bash");
}

#[test]
fn trailing_non_alphanumeric_non_punctuation_is_stripped() {
    assert_eq!(parse(c"vi\t"), "vi");
    assert_eq!(parse(c"vi\t\t"), "vi");
    assert_eq!(parse(c"vi."), "vi.");
    assert_eq!(parse(c"\t"), "");
}

#[test]
fn an_absolute_path_is_reduced_to_its_last_component() {
    assert_eq!(parse(c"/usr/bin/vi"), "vi");
    assert_eq!(parse(c"/usr/bin/"), "bin");
    assert_eq!(parse(c"/"), "/");
    assert_eq!(parse(c"./usr/bin/vi"), "./usr/bin/vi");
}

#[test]
fn an_empty_name_stays_empty() {
    assert_eq!(parse(c""), "");
    assert_eq!(parse(c"\""), "");
    assert_eq!(parse(c" "), "");
    assert_eq!(parse(c"exec "), "");
}

#[test]
fn a_name_with_control_characters_is_rejected_and_left_empty() {
    assert_eq!(parse(c"\x07vi"), "");
}

#[test]
fn invalid_utf8_falls_back_to_the_empty_name() {
    let bad = CString::new(b"\xc3\x28".as_slice()).unwrap();
    assert_eq!(parse(&bad), "");
}

#[test]
fn a_window_without_an_active_pane_has_an_empty_default_name() {
    let mut w = blank_window();
    unsafe {
        let p = default_window_name(&raw mut *w);
        assert_eq!(p.as_bytes(), b"");
    }
}

#[test]
fn the_default_name_comes_from_the_active_pane_command() {
    let mut w = blank_window();
    let mut p = blank_pane();
    let arg0 = CString::new("/usr/bin/vi").unwrap();
    let arg1 = CString::new("file").unwrap();
    p.argv = vec![arg0, arg1];
    w.active_id = Some(p.id);
    w.panes.push(p);
    unsafe {
        let name = default_window_name(&raw mut *w);
        assert_eq!(name.as_bytes(), b"vi");
    }
}

#[test]
fn the_default_name_falls_back_to_the_pane_shell() {
    let mut w = blank_window();
    let mut p = blank_pane();
    let shell = CString::new("/bin/bash").unwrap();
    p.argv = Vec::new();
    p.shell = Some(shell.to_owned());
    w.active_id = Some(p.id);
    w.panes.push(p);
    unsafe {
        let name = default_window_name(&raw mut *w);
        assert_eq!(name.as_bytes(), b"bash");
    }
}

fn expired(name_time: timeval, now: timeval) -> c_int {
    let mut w = blank_window();
    w.name_time = name_time;
    let mut now = now;
    unsafe { name_time_expired(&raw mut *w, &raw mut now) }
}

#[test]
fn the_rename_interval_reports_the_time_left() {
    let base = timeval {
        tv_sec: 100,
        tv_usec: 0,
    };
    assert_eq!(
        expired(
            base,
            timeval {
                tv_sec: 100,
                tv_usec: 0
            }
        ),
        NAME_INTERVAL,
    );
    assert_eq!(
        expired(
            base,
            timeval {
                tv_sec: 100,
                tv_usec: 200000
            }
        ),
        NAME_INTERVAL - 200000,
    );
}

#[test]
fn the_rename_interval_reports_zero_once_it_has_passed() {
    let base = timeval {
        tv_sec: 100,
        tv_usec: 0,
    };
    assert_eq!(
        expired(
            base,
            timeval {
                tv_sec: 100,
                tv_usec: 600000
            }
        ),
        0,
    );
    assert_eq!(
        expired(
            base,
            timeval {
                tv_sec: 102,
                tv_usec: 0
            }
        ),
        0,
    );
}

#[test]
fn the_rename_interval_borrows_across_a_second() {
    assert_eq!(
        expired(
            timeval {
                tv_sec: 100,
                tv_usec: 900000
            },
            timeval {
                tv_sec: 101,
                tv_usec: 100000
            },
        ),
        NAME_INTERVAL - 200000,
    );
    assert_eq!(
        expired(
            timeval {
                tv_sec: 100,
                tv_usec: 900000
            },
            timeval {
                tv_sec: 101,
                tv_usec: 800000
            },
        ),
        0,
    );
}

#[test]
fn the_name_timer_callback_only_logs() {
    let _guard = globals();
    let mut w = blank_window();
    w.id = 7;
    unsafe {
        name_time_callback(&raw mut *w);
    }
}

#[test]
fn checking_a_window_without_an_active_pane_does_nothing() {
    let _guard = globals();
    let mut w = blank_window();
    unsafe { check_window_name(&raw mut *w) };
}

/// A standalone window options set holding just `automatic-rename`, taken
/// straight from the options table so it has the right table entry.
fn rename_options(value: ::core::ffi::c_longlong) -> *mut options {
    unsafe {
        let oo = Box::into_raw(options_create_boxed(::core::ptr::null_mut::<options>()));
        for oe in &options_table {
            if oe.name == c"automatic-rename" {
                options_default(oo, oe);
                options_set_number(oo, oe.name.as_ptr(), value);
                break;
            }
        }
        oo
    }
}

#[test]
fn checking_a_window_with_automatic_rename_off_does_nothing() {
    let _guard = globals();
    let mut w = blank_window();
    let p = blank_pane();
    w.active_id = Some(p.id);
    w.panes.push(p);
    unsafe {
        w.options = Some(Box::from_raw(rename_options(0)));
        check_window_name(&raw mut *w);
    }
}

#[test]
fn checking_an_unchanged_active_pane_does_nothing() {
    let _guard = globals();
    let mut w = blank_window();
    let mut p = blank_pane();
    p.flags = 0;
    w.active_id = Some(p.id);
    w.panes.push(p);
    unsafe {
        w.options = Some(Box::from_raw(rename_options(1)));
        check_window_name(&raw mut *w);
    }
}
