use super::*;
use crate::WindowPane;
use crate::options::OptionsRef;
use crate::pane_command::{PaneCommand, PaneCommandState};
use crate::tests::test_fixtures::{Target, globals, zeroed_pane, zeroed_window};
use crate::window_timestamps::WindowTimestampState;
use ::core::ffi::{CStr, c_int};
use ::std::ffi::CString;

fn parse(input: &CStr) -> String {
    parse_window_name(input).to_str().unwrap().to_owned()
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
fn repeated_slashes_in_an_absolute_path_are_not_part_of_the_name() {
    assert_eq!(parse(c"//"), "/");
    assert_eq!(parse(c"///"), "/");
    assert_eq!(parse(c"/usr//bin//"), "bin");
    assert_eq!(parse(c"/usr/bin/vi//"), "vi");
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
        let p = default_window_name(&WindowRef::new(*w));
        assert_eq!(p.as_bytes(), b"");
    }
}

#[test]
fn the_default_name_comes_from_the_active_pane_command() {
    let mut w = blank_window();
    let mut p = blank_pane();
    let arg0 = CString::new("/usr/bin/vi").unwrap();
    let arg1 = CString::new("file").unwrap();
    p.set_pane_command(&PaneCommand {
        argv: vec![arg0, arg1],
        ..Default::default()
    });
    let pane = RustWindowPaneRef::new(p);
    w.active_pane = Some(pane.downgrade());
    w.panes.push(pane);
    unsafe {
        let name = default_window_name(&WindowRef::new(*w));
        assert_eq!(name.as_bytes(), b"vi");
    }
}

#[test]
fn the_default_name_falls_back_to_the_pane_shell() {
    let mut w = blank_window();
    let mut p = blank_pane();
    let shell = CString::new("/bin/bash").unwrap();
    p.set_pane_command(&PaneCommand {
        shell: Some(shell.to_owned()),
        ..Default::default()
    });
    let pane = RustWindowPaneRef::new(p);
    w.active_pane = Some(pane.downgrade());
    w.panes.push(pane);
    unsafe {
        let name = default_window_name(&WindowRef::new(*w));
        assert_eq!(name.as_bytes(), b"bash");
    }
}

fn expired(name_time: timeval, now: timeval) -> c_int {
    let mut w = blank_window();
    w.set_name_update_time(name_time);
    (WindowRef::new(*w)).name_time_expired(now)
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
    let w_ref = WindowRef::new(*w);
    w_ref.on_name_timer();
}

#[test]
fn checking_a_window_without_an_active_pane_does_nothing() {
    let _guard = globals();
    let mut target = Target::new(20, 6);
    unsafe {
        let mut window = target.state().window().unwrap();
        window.as_window_mut().active_pane = None;
        window.check_name();
        assert_eq!(window.window_name().as_deref(), Some(c"target"));
        assert!(!window.as_window().name_event.is_armed());
    }
}

#[test]
fn checking_a_window_with_automatic_rename_off_does_nothing() {
    let _guard = globals();
    let mut target = Target::new(20, 6);
    unsafe {
        let state = target.state();
        let window = state.window().unwrap();
        let mut pane = state.pane_ref().unwrap();
        *pane.get_mut().unwrap().flags_mut() |= PANE_CHANGED;
        window.options().set_number(c"automatic-rename", 0);
        window.check_name();
        assert_eq!(window.window_name().as_deref(), Some(c"target"));
        assert_ne!(*pane.get().unwrap().flags() & PANE_CHANGED, 0);
        assert!(!window.as_window().name_event.is_armed());
    }
}

#[test]
fn checking_an_unchanged_active_pane_does_nothing() {
    let _guard = globals();
    let mut target = Target::new(20, 6);
    unsafe {
        let state = target.state();
        let window = state.window().unwrap();
        *state.pane_ref().unwrap().get_mut().unwrap().flags_mut() &= !PANE_CHANGED;
        window.options().set_number(c"automatic-rename", 1);
        window.check_name();
        assert_eq!(window.window_name().as_deref(), Some(c"target"));
        assert!(!window.as_window().name_event.is_armed());
    }
}

#[test]
fn automatic_rename_formats_the_active_pane_and_defers_the_next_change() {
    let _guard = globals();
    let mut target = Target::new(20, 6);
    unsafe {
        let state = target.state();
        let mut window = state.window().unwrap();
        let mut pane = state.pane_ref().unwrap();
        let options = window.options();
        options.set_number(c"automatic-rename", 1);
        options.set_string(
            c"automatic-rename-format",
            0,
            c"pane-#{pane_id}",
            fmt_args![],
        );
        *pane.get_mut().unwrap().flags_mut() |= PANE_CHANGED;
        window.set_name_update_time(timeval::default());
        window.check_name();
        assert_eq!(window.window_name().as_deref(), Some(c"pane-%0"));
        assert_eq!(*pane.get().unwrap().flags() & PANE_CHANGED, 0);
        options.set_string(c"automatic-rename-format", 0, c"deferred", fmt_args![]);
        let now = timeval::now();
        window.set_name_update_time(now);
        *pane.get_mut().unwrap().flags_mut() |= PANE_CHANGED;
        window.check_name();
        assert!(window.as_window().name_event.is_armed());
        assert_eq!(window.window_name().as_deref(), Some(c"pane-%0"));
        assert_ne!(*pane.get().unwrap().flags() & PANE_CHANGED, 0);
        window.set_name_update_time(timeval::default());
        window.check_name();
        assert!(!window.as_window().name_event.is_armed());
        assert_eq!(window.window_name().as_deref(), Some(c"deferred"));
        assert_eq!(*pane.get().unwrap().flags() & PANE_CHANGED, 0);
    }
}
