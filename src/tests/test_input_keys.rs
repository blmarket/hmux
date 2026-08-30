use super::*;
use crate::text::key_string_lookup_string;
use crate::log::log_add_level;
use crate::options::options_set_number;
use crate::resize::WINDOW_ZOOMED;
use crate::tests::test_fixtures::{Pane, Screen, StreamBuffer, Window, globals};
use crate::window::window_set_active;
use ::core::ffi::{CStr, c_int};
use ::core::ptr::null_mut;
use ::std::sync::MutexGuard;

/// A screen to read the terminal modes from, a buffer event to write the
/// answer into, and the turn at the globals the key table and the options
/// live in.
struct Keys {
    screen: Screen,
    bev: StreamBuffer,
    _guard: MutexGuard<'static, ()>,
}

impl Keys {
    fn new() -> Keys {
        static BUILD: ::std::sync::Once = ::std::sync::Once::new();
        let guard = globals();
        BUILD.call_once(input_key_build);
        unsafe {
            options_set_number(
                global_options,
                c"backspace".as_ptr(),
                key_string_lookup_string(c"C-?".as_ptr()) as ::core::ffi::c_longlong,
            );
            options_set_number(global_options, c"extended-keys-format".as_ptr(), 0);
        }
        Keys {
            screen: Screen::new(10, 2, 0),
            bev: StreamBuffer::new(),
            _guard: guard,
        }
    }

    fn mode(&mut self, mode: c_int) -> &mut Keys {
        self.screen.mode = mode;
        self
    }

    /// What `input_key` writes for the key named `s`, and what it answered.
    fn key(&mut self, s: &CStr) -> (c_int, String) {
        let key = unsafe { key_string_lookup_string(s.as_ptr()) };
        assert_ne!(key, KEYC_UNKNOWN as key_code, "{s:?} is not a key");
        self.code(key)
    }

    /// The same for a key code worked out by hand.
    fn code(&mut self, key: key_code) -> (c_int, String) {
        let answer = unsafe { input_key(self.screen.ptr(), self.bev.ptr(), key) };
        (answer, shown(&self.bev.written()))
    }

    /// The backspace the terminal is told to send.
    fn backspace(&mut self, s: &CStr) -> &mut Keys {
        unsafe {
            options_set_number(
                global_options,
                c"backspace".as_ptr(),
                key_string_lookup_string(s.as_ptr()) as ::core::ffi::c_longlong,
            );
        }
        self
    }

    fn extended_format(&mut self, format: ::core::ffi::c_longlong) -> &mut Keys {
        unsafe { options_set_number(global_options, c"extended-keys-format".as_ptr(), format) };
        self
    }
}

/// Bytes as text, with the ones that are not printable written out.
fn shown(bytes: &[u8]) -> String {
    let mut out = String::new();
    for &byte in bytes {
        match byte {
            0x1b => out.push_str("<esc>"),
            0x20..=0x7e => out.push(byte as char),
            _ => out.push_str(&format!("<{byte:02x}>")),
        }
    }
    out
}

#[test]
fn a_mouse_key_is_not_a_key_the_pane_is_told_about() {
    let mut keys = Keys::new();
    assert_eq!(keys.code(KEYC_MOUSE as key_code), (0, String::new()));
    assert_eq!(
        keys.code((KEYC_TYPE_MOUSEMOVE as key_code) << 32),
        (0, String::new())
    );
    assert_eq!(
        keys.code((KEYC_TYPE_TRIPLECLICK as key_code) << 32),
        (0, String::new())
    );
}

#[test]
fn a_literal_key_is_written_as_the_one_byte_it_is() {
    let mut keys = Keys::new();
    assert_eq!(keys.code(b'a' as key_code | KEYC_LITERAL), (0, "a".into()));
    assert_eq!(keys.code(0x1b | KEYC_LITERAL), (0, "<esc>".into()));
}

#[test]
fn a_printable_key_is_written_as_itself() {
    let mut keys = Keys::new();
    assert_eq!(keys.key(c"a"), (0, "a".into()));
    assert_eq!(keys.key(c"Space"), (0, " ".into()));
    assert_eq!(keys.key(c"~"), (0, "~".into()));
    assert_eq!(keys.key(c"Tab"), (0, "<09>".into()));
    assert_eq!(keys.key(c"Enter"), (0, "<0d>".into()));
    assert_eq!(keys.key(c"Escape"), (0, "<esc>".into()));
}

#[test]
fn a_character_of_more_than_one_byte_is_written_whole() {
    let mut keys = Keys::new();
    assert_eq!(keys.key(c"\u{4e2d}"), (0, "<e4><b8><ad>".into()));
}

#[test]
fn backspace_is_whatever_the_option_says_it_is() {
    let mut keys = Keys::new();
    assert_eq!(keys.key(c"BSpace"), (0, "<7f>".into()));
    assert_eq!(keys.backspace(c"C-h").key(c"BSpace"), (0, "<08>".into()));
    assert_eq!(keys.backspace(c"C-@").key(c"BSpace"), (0, "<00>".into()));
    assert_eq!(keys.backspace(c"C-_").key(c"BSpace"), (0, "<1f>".into()));
    assert_eq!(keys.backspace(c"b").key(c"BSpace"), (0, "b".into()));
}

/// A backspace the option names with a modifier the one-byte forms do not
/// cover is written as nothing at all.
#[test]
fn a_backspace_with_a_modifier_of_its_own_is_written_as_nothing() {
    let mut keys = Keys::new();
    assert_eq!(keys.backspace(c"M-b").key(c"BSpace"), (0, String::new()));
    assert_eq!(keys.backspace(c"C-1").key(c"BSpace"), (0, String::new()));
}

#[test]
fn a_backspace_carrying_modifiers_is_looked_up_as_the_key_it_stands_for() {
    let mut keys = Keys::new();
    assert_eq!(keys.backspace(c"C-h").key(c"C-BSpace"), (0, "<08>".into()));
}

#[test]
fn a_back_tab_loses_its_modifiers_unless_the_terminal_asked_for_them() {
    let mut keys = Keys::new();
    assert_eq!(keys.key(c"BTab"), (0, "<esc>[Z".into()));
    assert_eq!(
        keys.mode(MODE_KEYS_EXTENDED_2).key(c"BTab"),
        (0, "<esc>[9;2u".into())
    );
}

#[test]
fn a_cursor_key_follows_the_terminals_cursor_mode() {
    let mut keys = Keys::new();
    assert_eq!(keys.key(c"Up"), (0, "<esc>[A".into()));
    assert_eq!(keys.mode(MODE_KCURSOR).key(c"Up"), (0, "<esc>OA".into()));
}

#[test]
fn a_keypad_key_follows_the_terminals_keypad_mode() {
    let mut keys = Keys::new();
    assert_eq!(keys.key(c"KP*"), (0, "*".into()));
    assert_eq!(keys.mode(MODE_KKEYPAD).key(c"KP*"), (0, "<esc>Oj".into()));
}

#[test]
fn a_function_key_is_looked_up_in_the_table() {
    let mut keys = Keys::new();
    assert_eq!(keys.key(c"F1"), (0, "<esc>OP".into()));
    assert_eq!(keys.key(c"S-F1"), (0, "<esc>[1;2P".into()));
    assert_eq!(keys.key(c"C-F1"), (0, "<esc>[1;5P".into()));
    assert_eq!(keys.key(c"Home"), (0, "<esc>[1~".into()));
    assert_eq!(keys.key(c"PPage"), (0, "<esc>[5~".into()));
}

#[test]
fn a_paste_key_is_only_written_when_the_terminal_asked_for_them() {
    let mut keys = Keys::new();
    assert_eq!(keys.code(KEYC_PASTE_START as key_code), (0, String::new()));
    assert_eq!(
        keys.mode(MODE_BRACKETPASTE)
            .code(KEYC_PASTE_START as key_code),
        (0, "<esc>[200~".into())
    );
    assert_eq!(
        keys.code(KEYC_PASTE_END as key_code),
        (0, "<esc>[201~".into())
    );
}

#[test]
fn a_key_with_meta_the_table_does_not_carry_is_written_after_an_escape() {
    let mut keys = Keys::new();
    assert_eq!(keys.key(c"M-Up"), (0, "<esc>[1;3A".into()));
    let up = unsafe { key_string_lookup_string(c"Up".as_ptr()) };
    assert_eq!(keys.code(up | KEYC_META), (0, "<esc><esc>[A".into()));
}

#[test]
fn a_key_of_a_kind_no_terminal_has_is_left_alone() {
    let mut keys = Keys::new();
    assert_eq!(
        keys.code((KEYC_TYPE_USER as key_code) << 32),
        (0, String::new())
    );
    assert_eq!(
        keys.code((KEYC_TYPE_FUNCTION as key_code) << 32),
        (0, String::new())
    );
}

#[test]
fn a_control_key_is_written_as_the_byte_it_stands_for() {
    let mut keys = Keys::new();
    assert_eq!(keys.key(c"C-a"), (0, "<01>".into()));
    assert_eq!(keys.key(c"C-Space"), (0, "<00>".into()));
    assert_eq!(keys.key(c"C-/"), (0, "<1f>".into()));
    assert_eq!(keys.key(c"C-4"), (0, "<1c>".into()));
    assert_eq!(keys.key(c"C-1"), (0, "1".into()));
    assert_eq!(keys.key(c"C-9"), (0, "9".into()));
    assert_eq!(keys.key(c"M-a"), (0, "<esc>a".into()));
    assert_eq!(keys.key(c"C-M-a"), (0, "<esc><01>".into()));
}

/// Return, newline and tab are written as themselves even under control,
/// which is what strips the modifier off them.
#[test]
fn a_control_return_or_tab_loses_the_control() {
    let mut keys = Keys::new();
    assert_eq!(keys.key(c"C-Enter"), (0, "<0d>".into()));
    assert_eq!(keys.key(c"C-Tab"), (0, "<09>".into()));
}

#[test]
fn a_control_key_with_no_byte_of_its_own_is_written_as_nothing() {
    let mut keys = Keys::new();
    assert_eq!(keys.code(b'#' as key_code | KEYC_CTRL), (-1, String::new()));
}

/// A key above ASCII is written as the character it stands for even under
/// control, since the control is only read after the character is.
#[test]
fn a_wide_character_under_control_is_written_as_the_character() {
    let mut keys = Keys::new();
    assert_eq!(keys.code(0xa9 | KEYC_CTRL), (0, String::new()));
}

#[test]
fn a_wide_character_under_meta_is_written_after_an_escape() {
    let mut keys = Keys::new();
    assert_eq!(keys.key(c"M-\u{4e2d}"), (0, "<esc><e4><b8><ad>".into()));
}

#[test]
fn the_terminal_can_ask_for_every_key_in_the_extended_form() {
    let mut keys = Keys::new();
    keys.mode(MODE_KEYS_EXTENDED_2);
    assert_eq!(keys.key(c"S-a"), (0, "<esc>[97;2u".into()));
    assert_eq!(keys.key(c"M-a"), (0, "<esc>[97;3u".into()));
    assert_eq!(keys.key(c"S-M-a"), (0, "<esc>[97;4u".into()));
    assert_eq!(keys.key(c"C-a"), (0, "<esc>[97;5u".into()));
    assert_eq!(keys.key(c"S-C-a"), (0, "<esc>[97;6u".into()));
    assert_eq!(keys.key(c"C-M-a"), (0, "<esc>[97;7u".into()));
    assert_eq!(keys.key(c"S-C-M-a"), (0, "<esc>[97;8u".into()));
    assert_eq!(keys.code(b'a' as key_code), (0, "a".into()));
}

#[test]
fn the_extended_form_can_be_the_other_one() {
    let mut keys = Keys::new();
    keys.mode(MODE_KEYS_EXTENDED_2).extended_format(1);
    assert_eq!(keys.key(c"C-a"), (0, "<esc>[27;5;97~".into()));
}

#[test]
fn a_wide_character_in_the_extended_form_is_named_by_its_codepoint() {
    let mut keys = Keys::new();
    keys.mode(MODE_KEYS_EXTENDED_2);
    assert_eq!(keys.key(c"C-\u{4e2d}"), (0, "<esc>[20013;5u".into()));
}

#[test]
fn the_terminal_can_ask_for_the_extended_form_only_where_it_is_needed() {
    let mut keys = Keys::new();
    keys.mode(MODE_KEYS_EXTENDED);
    assert_eq!(keys.key(c"C-a"), (0, "<01>".into()));
    assert_eq!(keys.key(c"M-a"), (0, "<esc>a".into()));
    assert_eq!(keys.key(c"C-Space"), (0, "<00>".into()));
    assert_eq!(keys.key(c"C-/"), (0, "<1f>".into()));
    assert_eq!(keys.key(c"S-a"), (0, "<esc>[97;2u".into()));
    assert_eq!(keys.key(c"C-1"), (0, "<esc>[49;5u".into()));
}

#[test]
fn the_key_tree_holds_every_key_the_table_names() {
    let mut keys = Keys::new();
    let _ = &mut keys;
    assert!(input_key_get(KEYC_PASTE_START as key_code).is_some());
    assert!(input_key_get(0xdeadbeef).is_none());
    let mut count = 0;
    let mut last = 0;
    for key in super::keys().keys() {
        assert!(*key > last || count == 0, "the tree is out of order");
        last = *key;
        count += 1;
    }
    assert_eq!(count, 217);
}

/// A mouse report the terminal has not asked for is not written.
#[test]
fn a_mouse_report_needs_the_terminal_to_have_asked_for_one() {
    let _guard = globals();
    let mut s = Screen::new(10, 2, 0);
    let mut m = Box::new(mouse_event::default());
    m.sgr_type = b' ' as u_int;
    assert_eq!(mouse(&mut s, &mut m, 0, 0), None);
    s.mode = MODE_MOUSE_STANDARD;
    assert_eq!(mouse(&mut s, &mut m, 0, 0), Some("<esc>[M !!".into()));
}

/// What `input_key_get_mouse` answers, or `None` if it wrote nothing.
fn mouse(s: &mut Screen, m: &mut mouse_event, x: u_int, y: u_int) -> Option<String> {
    unsafe { input_key_get_mouse(s.ptr(), &raw mut *m, x, y).map(|report| shown(&report)) }
}

#[test]
fn a_drag_is_only_reported_when_the_terminal_follows_them() {
    let _guard = globals();
    let mut s = Screen::new(10, 2, 0);
    let mut m = Box::new(mouse_event::default());
    m.sgr_type = b' ' as u_int;
    m.b = MOUSE_MASK_DRAG as u_int;
    s.mode = MODE_MOUSE_STANDARD;
    assert_eq!(mouse(&mut s, &mut m, 0, 0), None);
    s.mode = MODE_MOUSE_BUTTON;
    assert_eq!(mouse(&mut s, &mut m, 0, 0), Some("<esc>[M@!!".into()));
}

#[test]
fn a_drag_with_no_button_down_is_only_reported_to_a_terminal_wanting_all() {
    let _guard = globals();
    let mut s = Screen::new(10, 2, 0);
    let mut m = Box::new(mouse_event::default());
    m.sgr_type = b' ' as u_int;
    m.b = (MOUSE_MASK_DRAG | 3) as u_int;
    m.lb = 3;
    s.mode = MODE_MOUSE_BUTTON;
    assert_eq!(mouse(&mut s, &mut m, 0, 0), None);
    s.mode = MODE_MOUSE_ALL;
    assert_eq!(mouse(&mut s, &mut m, 0, 0), Some("<esc>[MC!!".into()));
}

#[test]
fn an_sgr_report_names_the_button_and_the_place_in_full() {
    let _guard = globals();
    let mut s = Screen::new(10, 2, 0);
    let mut m = Box::new(mouse_event::default());
    m.sgr_type = b'M' as u_int;
    m.sgr_b = 0;
    s.mode = MODE_MOUSE_STANDARD | MODE_MOUSE_SGR;
    assert_eq!(mouse(&mut s, &mut m, 4, 9), Some("<esc>[<0;5;10M".into()));
    m.sgr_b = (MOUSE_MASK_DRAG | 3) as u_int;
    assert_eq!(mouse(&mut s, &mut m, 4, 9), None);
    s.mode = MODE_MOUSE_ALL | MODE_MOUSE_SGR;
    assert_eq!(mouse(&mut s, &mut m, 4, 9), Some("<esc>[<35;5;10M".into()));
}

#[test]
fn a_utf8_report_carries_places_two_bytes_wide() {
    let _guard = globals();
    let mut s = Screen::new(10, 2, 0);
    let mut m = Box::new(mouse_event::default());
    m.sgr_type = b' ' as u_int;
    s.mode = MODE_MOUSE_STANDARD | MODE_MOUSE_UTF8;
    assert_eq!(mouse(&mut s, &mut m, 0, 0), Some("<esc>[M !!".into()));
    assert_eq!(
        mouse(&mut s, &mut m, 200, 0),
        Some("<esc>[M <c3><a9>!".into())
    );
    assert_eq!(mouse(&mut s, &mut m, 2048, 0), None);
    m.b = 2048;
    assert_eq!(mouse(&mut s, &mut m, 0, 0), None);
}

#[test]
fn a_report_of_a_place_too_far_out_stops_at_the_edge() {
    let _guard = globals();
    let mut s = Screen::new(10, 2, 0);
    let mut m = Box::new(mouse_event::default());
    m.sgr_type = b' ' as u_int;
    s.mode = MODE_MOUSE_STANDARD;
    assert_eq!(
        mouse(&mut s, &mut m, 300, 300),
        Some("<esc>[M <ff><ff>".into())
    );
    s.mode = MODE_MOUSE_BUTTON;
    m.b = 224;
    assert_eq!(mouse(&mut s, &mut m, 0, 0), None);
}

#[test]
fn a_pane_is_given_the_bytes_of_a_key() {
    let _guard = globals();
    let mut pane = Pane::new(1, 10, 2, 0);
    let bev = StreamBuffer::new();
    unsafe {
        (*pane.ptr()).event = bev.ptr();
        assert_eq!(input_key_pane(pane.ptr(), b'a' as key_code, null_mut()), 0);
    }
    assert_eq!(shown(&bev.written()), "a");
}

#[test]
fn a_pane_is_given_a_mouse_report_only_for_a_click_inside_it() {
    let _guard = globals();
    let mut window = Window::new(1, "keys", 10, 2);
    let mut pane = Pane::new(1, 10, 2, 0);
    window.add_pane(&mut pane);
    let bev = StreamBuffer::new();
    let mut m = Box::new(mouse_event::default());
    m.sgr_type = b' ' as u_int;
    m.wp = 1;
    unsafe {
        (*pane.ptr()).event = bev.ptr();
        (*pane.screen()).mode = MODE_MOUSE_STANDARD;

        let key = KEYC_MOUSE as key_code;
        assert_eq!(input_key_pane(pane.ptr(), key, &raw mut *m), 0);
        assert_eq!(shown(&bev.written()), "<esc>[M !!");

        m.ignore = 1;
        assert_eq!(input_key_pane(pane.ptr(), key, &raw mut *m), 0);
        assert_eq!(shown(&bev.written()), "");

        m.ignore = 0;
        m.x = 20;
        assert_eq!(input_key_pane(pane.ptr(), key, &raw mut *m), 0);
        assert_eq!(shown(&bev.written()), "");

        m.x = 0;
        m.wp = 2;
        assert_eq!(input_key_pane(pane.ptr(), key, &raw mut *m), 0);
        assert_eq!(shown(&bev.written()), "");

        m.wp = 1;
        (*window.ptr()).flags |= WINDOW_ZOOMED;
        window_set_active(window.ptr(), null_mut::<window_pane>());
        assert_eq!(input_key_pane(pane.ptr(), key, &raw mut *m), 0);
        assert_eq!(shown(&bev.written()), "");
    }
}

/// A pane told about a mouse key it has no report for writes nothing, and
/// the log names every key on its way through.
#[test]
fn a_pane_names_the_key_in_the_log_and_takes_a_mouse_type_key_too() {
    let _guard = globals();
    let mut pane = Pane::new(1, 10, 2, 0);
    let bev = StreamBuffer::new();
    unsafe {
        log_add_level();
        assert_ne!(log_get_level(), 0);
        (*pane.ptr()).event = bev.ptr();
        assert_eq!(input_key_pane(pane.ptr(), b'a' as key_code, null_mut()), 0);
        assert_eq!(shown(&bev.written()), "a");
        assert_eq!(
            input_key_pane(
                pane.ptr(),
                (KEYC_TYPE_MOUSEMOVE as key_code) << 32,
                null_mut()
            ),
            0
        );
        assert_eq!(shown(&bev.written()), "");
    }
}

/// A mouse report the pane cannot make is not written, though the key was
/// one it would have taken.
#[test]
fn a_pane_writes_nothing_for_a_report_its_terminal_turned_down() {
    let _guard = globals();
    let mut window = Window::new(1, "keys", 10, 2);
    let mut pane = Pane::new(1, 10, 2, 0);
    window.add_pane(&mut pane);
    let bev = StreamBuffer::new();
    let mut m = Box::new(mouse_event::default());
    m.sgr_type = b' ' as u_int;
    m.wp = 1;
    m.b = MOUSE_MASK_DRAG as u_int;
    unsafe {
        (*pane.ptr()).event = bev.ptr();
        (*pane.screen()).mode = MODE_MOUSE_STANDARD;
        assert_eq!(
            input_key_pane(pane.ptr(), KEYC_MOUSE as key_code, &raw mut *m),
            0
        );
    }
    assert_eq!(shown(&bev.written()), "");
}

/// A key of no modifiers at all that reaches the extended form has nothing
/// to name, and neither does one whose character cannot be read back.
#[test]
fn a_key_the_extended_form_cannot_name_is_written_as_nothing() {
    let mut keys = Keys::new();
    keys.mode(MODE_KEYS_EXTENDED_2);
    assert_eq!(keys.code(0x01), (-1, String::new()));
    assert_eq!(keys.code(0x1fffff | KEYC_CTRL), (-1, String::new()));
}

/// A key carrying the cursor or keypad flag that the table has no entry
/// for under that flag is looked up again without it.
#[test]
fn a_cursor_or_keypad_key_the_table_does_not_carry_is_looked_up_plain() {
    let mut keys = Keys::new();
    let up = unsafe { key_string_lookup_string(c"Up".as_ptr()) };
    let kp = unsafe { key_string_lookup_string(c"KP*".as_ptr()) };
    assert_eq!(
        keys.mode(MODE_KCURSOR).code(up | KEYC_CURSOR | KEYC_SHIFT),
        (0, "<esc>[1;2A".into())
    );
    assert_eq!(
        keys.mode(MODE_KKEYPAD).code(kp | KEYC_KEYPAD | KEYC_SHIFT),
        (0, String::new())
    );
}

/// The backspace option can name a mouse key, which the pane is then never
/// told about.
#[test]
fn a_backspace_option_naming_a_mouse_key_is_ignored() {
    let mut keys = Keys::new();
    unsafe {
        options_set_number(
            global_options,
            c"backspace".as_ptr(),
            (((KEYC_TYPE_MOUSEMOVE as key_code) << 32) | 1) as ::core::ffi::c_longlong,
        );
    }
    assert_eq!(
        keys.code(KEYC_BSPACE as key_code | KEYC_CTRL),
        (0, String::new())
    );
}

#[test]
fn a_pane_told_about_a_mouse_key_with_no_report_behind_it_writes_nothing() {
    let _guard = globals();
    let mut pane = Pane::new(1, 10, 2, 0);
    let bev = StreamBuffer::new();
    unsafe {
        (*pane.ptr()).event = bev.ptr();
        assert_eq!(
            input_key_pane(pane.ptr(), KEYC_MOUSE as key_code, null_mut()),
            0
        );
    }
    assert_eq!(shown(&bev.written()), "");
}
