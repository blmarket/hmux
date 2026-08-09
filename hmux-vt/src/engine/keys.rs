//! Encoding input for the program in the pane.
//!
//! Key encoding is already hmux's own: the server's `input_keys` module is
//! the port of tmux's `input-keys.c`, and every key a pane receives goes
//! through it. What is left here is the mouse, whose encoding depends on which
//! reporting mode and which wire format the program asked for — screen state,
//! and so the screen's to answer.

use super::screen::Screen;
use crate::input::{Key, KeyEvent, MouseAction, MouseButton, MouseEvent};
use crate::screen::mode;

/// The largest offset field the UTF-8 mouse form carries: a two-byte UTF-8
/// sequence. A report with a field past this is not sent at all.
const MOUSE_UTF8_PARAM_MAX: u32 = 0x7ff;

/// The wire button number, before the modifier and motion bits.
///
/// The three ordinary buttons are 0–2; the wheel is 64 upwards; the extra
/// buttons are 128 upwards. This is xterm's numbering, which tmux and every
/// terminal that reports the mouse share.
fn button_code(button: MouseButton) -> u32 {
    match button {
        MouseButton::Left => 0,
        MouseButton::Middle => 1,
        MouseButton::Right => 2,
        MouseButton::WheelUp => 64,
        MouseButton::WheelDown => 65,
        MouseButton::Six => 66,
        MouseButton::Seven => 67,
        MouseButton::Eight => 128,
        MouseButton::Nine => 129,
        MouseButton::Ten => 130,
        MouseButton::Eleven => 131,
    }
}

/// Whether this event is reported at all under the screen's current mode.
///
/// Motion only exists for the two motion modes. Button-event mode follows a
/// *particular* button, so an event that names none — hmux produces one when
/// the button was one it does not recognise — has nothing to do with it;
/// standard and any-event mode report such an event under the wire's
/// "any button" code instead.
///
/// [`MouseEvent::any_button_pressed`] deliberately plays no part: a motion
/// event that names a button plainly has one down, and one that does not is
/// decided by the mode alone.
fn reportable(screen: &Screen, mouse: MouseEvent) -> bool {
    if screen.mode & mode::ALL_MOUSE == 0 {
        return false;
    }
    match mouse.action {
        MouseAction::Motion => {
            if screen.mode & mode::MOUSE_ALL != 0 {
                true
            } else {
                screen.mode & mode::MOUSE_BUTTON != 0 && mouse.button.is_some()
            }
        }
        _ => mouse.button.is_some() || screen.mode & mode::MOUSE_BUTTON == 0,
    }
}

/// Encode one mouse event, or nothing when the program has not asked for it.
pub fn encode_mouse(screen: &Screen, mouse: MouseEvent) -> Vec<u8> {
    if !reportable(screen, mouse) {
        return Vec::new();
    }
    let sgr = screen.mode & mode::MOUSE_SGR != 0;
    let released = mouse.action == MouseAction::Release;

    // Without SGR a release does not say which button was let go: it is
    // reported as button 3, the "any button up" code.
    let mut code = match mouse.button {
        Some(button) if sgr || !released => button_code(button),
        _ => 3,
    };
    if mouse.action == MouseAction::Motion {
        code += 32;
    }
    if mouse.shift {
        code += 4;
    }
    if mouse.alt {
        code += 8;
    }
    if mouse.control {
        code += 16;
    }

    // The wire is one-based.
    let column = u32::from(mouse.column) + 1;
    let row = u32::from(mouse.row) + 1;

    if sgr {
        let end = if released { 'm' } else { 'M' };
        return format!("\x1b[<{code};{column};{row}{end}").into_bytes();
    }
    let mut out = b"\x1b[M".to_vec();
    if screen.mode & mode::MOUSE_UTF8 != 0 {
        // The UTF-8 form encodes each field as a character rather than a byte,
        // which is what lets it address past column 223. Its own ceiling is a
        // two-byte sequence: a field that would not fit drops the whole report
        // rather than sending a coordinate the program cannot read back.
        if [code, column, row]
            .iter()
            .any(|value| value + 32 > MOUSE_UTF8_PARAM_MAX)
        {
            return Vec::new();
        }
        let mut buffer = [0u8; 4];
        for value in [code, column, row] {
            let character = char::from_u32(value + 32).unwrap_or('\u{fffd}');
            out.extend_from_slice(character.encode_utf8(&mut buffer).as_bytes());
        }
        return out;
    }
    // The original form is one byte per field, offset by 32, and simply cannot
    // address past column 223.
    for value in [code, column, row] {
        out.push(u8::try_from(value + 32).unwrap_or(u8::MAX));
    }
    out
}

/// Encode one key press for the program in the pane.
///
/// The pane's own key path does not come through here: the server encodes a
/// key against the pane's options with its `input_keys::encode`, which knows
/// about `extended-keys` and the terminal's key tables. This
/// covers the remaining caller, which needs the plain forms a terminal in its
/// default modes would send.
///
/// Only the mode word matters to the answer, so that is all it takes.
pub fn encode_key(mode_word: u32, key: KeyEvent<'_>) -> Vec<u8> {
    let application = mode_word & mode::KCURSOR != 0;
    let cursor = |final_byte: char| -> Vec<u8> {
        let introducer = if application { "\x1bO" } else { "\x1b[" };
        format!("{introducer}{final_byte}").into_bytes()
    };
    let tilde = |number: u8| -> Vec<u8> { format!("\x1b[{number}~").into_bytes() };

    let plain = match key.key {
        Key::ARROW_UP => cursor('A'),
        Key::ARROW_DOWN => cursor('B'),
        Key::ARROW_RIGHT => cursor('C'),
        Key::ARROW_LEFT => cursor('D'),
        Key::HOME => cursor('H'),
        Key::END => cursor('F'),
        Key::INSERT => tilde(2),
        Key::DELETE => tilde(3),
        Key::PAGE_UP => tilde(5),
        Key::PAGE_DOWN => tilde(6),
        Key::ENTER => b"\r".to_vec(),
        Key::TAB if key.shift => b"\x1b[Z".to_vec(),
        Key::TAB => b"\t".to_vec(),
        Key::BACKSPACE => b"\x7f".to_vec(),
        Key::ESCAPE => b"\x1b".to_vec(),
        other if other.function_number().is_some() => {
            // xterm's function keys, which is what tmux's `input-keys.c`
            // sends: SS3 for the first four and a numbered tilde for the rest.
            // The numbers skip 16 and 22, as they always have.
            match other.function_number().unwrap_or(1) {
                1 => b"\x1bOP".to_vec(),
                2 => b"\x1bOQ".to_vec(),
                3 => b"\x1bOR".to_vec(),
                4 => b"\x1bOS".to_vec(),
                5 => tilde(15),
                6 => tilde(17),
                7 => tilde(18),
                8 => tilde(19),
                9 => tilde(20),
                10 => tilde(21),
                11 => tilde(23),
                _ => tilde(24),
            }
        }
        _ => {
            let Some(text) = key.text else {
                return Vec::new();
            };
            if key.control {
                // Control folds the character onto its C0 counterpart.
                let Some(character) = text.chars().next() else {
                    return Vec::new();
                };
                let byte = (character as u8) & 0x1f;
                vec![byte]
            } else {
                text.as_bytes().to_vec()
            }
        }
    };
    if key.alt && !plain.is_empty() && plain[0] != 0x1b {
        // Alt prefixes an escape, which is how a terminal spells meta.
        let mut out = vec![0x1b];
        out.extend_from_slice(&plain);
        return out;
    }
    plain
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::dispatch::Engine;
    use crate::parser::tokenize;

    fn screen(input: &[u8]) -> Engine {
        let mut engine = Engine::new(80, 24, 100);
        for token in tokenize(input) {
            engine.apply(&token.kind);
        }
        engine
    }

    #[test]
    fn the_function_keys_are_xterms() {
        let engine = screen(b"");
        let send = |number: u8| {
            encode_key(
                engine.screen.mode,
                KeyEvent {
                    key: Key::function(number).expect("function key"),
                    shift: false,
                    control: false,
                    alt: false,
                    text: None,
                    unshifted_codepoint: None,
                },
            )
        };
        // The first four are SS3; the rest are numbered, and the numbering
        // skips 16 and 22 as xterm's always has.
        assert_eq!(send(1), b"\x1bOP");
        assert_eq!(send(4), b"\x1bOS");
        assert_eq!(send(5), b"\x1b[15~");
        assert_eq!(send(6), b"\x1b[17~");
        assert_eq!(send(10), b"\x1b[21~");
        assert_eq!(send(11), b"\x1b[23~");
        assert_eq!(send(12), b"\x1b[24~");
    }

    fn press(column: u16, row: u16) -> MouseEvent {
        MouseEvent {
            action: MouseAction::Press,
            button: Some(MouseButton::Left),
            shift: false,
            control: false,
            alt: false,
            column,
            row,
            any_button_pressed: true,
        }
    }

    #[test]
    fn nothing_is_reported_until_the_program_asks() {
        let engine = screen(b"");
        assert!(encode_mouse(&engine.screen, press(2, 3)).is_empty());
    }

    /// hmux hands the encoder a button-less press when the button was one it
    /// does not recognise. tmux never produces one, so there is no oracle; the
    /// requirement is that both backends treat it the same way.
    #[test]
    fn a_button_less_event_belongs_to_the_modes_that_can_report_it() {
        let event = MouseEvent {
            button: None,
            ..press(2, 3)
        };
        for setup in [&b"\x1b[?1000h\x1b[?1006h"[..], b"\x1b[?1003h\x1b[?1006h"] {
            let screen = screen(setup);
            assert_eq!(
                encode_mouse(&screen.screen, event),
                b"\x1b[<3;3;4M".to_vec(),
                "reported under the any-button code"
            );
        }
        let button = screen(b"\x1b[?1002h\x1b[?1006h");
        for action in [
            MouseAction::Press,
            MouseAction::Release,
            MouseAction::Motion,
        ] {
            assert!(
                encode_mouse(&button.screen, MouseEvent { action, ..event }).is_empty(),
                "button-event mode follows a button, and there is none"
            );
        }
    }

    #[test]
    fn sgr_reports_carry_the_button_and_the_direction() {
        let engine = screen(b"\x1b[?1000h\x1b[?1006h");
        assert_eq!(
            encode_mouse(&engine.screen, press(2, 3)),
            b"\x1b[<0;3;4M".to_vec()
        );
        assert_eq!(
            encode_mouse(
                &engine.screen,
                MouseEvent {
                    action: MouseAction::Release,
                    any_button_pressed: false,
                    ..press(2, 3)
                }
            ),
            b"\x1b[<0;3;4m".to_vec()
        );
    }

    #[test]
    fn modifiers_and_the_wheel_add_their_own_bits() {
        let engine = screen(b"\x1b[?1002h\x1b[?1006h");
        let event = MouseEvent {
            button: Some(MouseButton::WheelUp),
            shift: true,
            control: true,
            column: 0,
            row: 0,
            any_button_pressed: false,
            ..press(0, 0)
        };
        assert_eq!(
            encode_mouse(&engine.screen, event),
            b"\x1b[<84;1;1M".to_vec(),
            "wheel up is 64, shift adds 4 and control adds 16"
        );
    }

    #[test]
    fn the_original_form_offsets_each_field_by_thirty_two() {
        let engine = screen(b"\x1b[?1000h");
        assert_eq!(
            encode_mouse(&engine.screen, press(2, 3)),
            vec![0x1b, b'[', b'M', 32, 35, 36]
        );
    }

    #[test]
    fn the_utf8_form_drops_a_report_past_the_protocol_limit() {
        let engine = screen(b"\x1b[?1000h\x1b[?1005h");
        // 0x7ff - 32 is the last column the two-byte form can name.
        assert_eq!(
            encode_mouse(&engine.screen, press(0x7ff - 33, 3)),
            "\x1b[M\u{20}\u{7ff}\u{24}".as_bytes(),
            "the last addressable column is still reported"
        );
        assert!(
            encode_mouse(&engine.screen, press(0x7ff - 32, 3)).is_empty(),
            "one column past it drops the whole report"
        );
        assert!(
            encode_mouse(&engine.screen, press(3, 0x7ff - 32)).is_empty(),
            "the row has the same ceiling"
        );
    }

    #[test]
    fn a_release_without_sgr_does_not_say_which_button() {
        let engine = screen(b"\x1b[?1000h");
        let release = MouseEvent {
            action: MouseAction::Release,
            any_button_pressed: false,
            ..press(0, 0)
        };
        assert_eq!(
            encode_mouse(&engine.screen, release),
            vec![0x1b, b'[', b'M', 35, 33, 33],
            "button 3 is the any-button-up code"
        );
    }

    #[test]
    fn motion_is_reported_only_by_the_motion_modes() {
        let moving = MouseEvent {
            action: MouseAction::Motion,
            ..press(1, 1)
        };
        let standard = screen(b"\x1b[?1000h\x1b[?1006h");
        assert!(encode_mouse(&standard.screen, moving).is_empty());

        let button = screen(b"\x1b[?1002h\x1b[?1006h");
        assert_eq!(
            encode_mouse(&button.screen, moving),
            b"\x1b[<32;2;2M".to_vec(),
            "motion adds thirty-two to the button"
        );

        let all = screen(b"\x1b[?1003h\x1b[?1006h");
        assert_eq!(
            encode_mouse(
                &all.screen,
                MouseEvent {
                    button: None,
                    ..moving
                }
            ),
            b"\x1b[<35;2;2M".to_vec(),
            "any-event mode reports motion with no button held"
        );
    }

    #[test]
    fn the_mouse_modes_are_mutually_exclusive() {
        let engine = screen(b"\x1b[?1003h\x1b[?1000h\x1b[?1006h");
        let motion = MouseEvent {
            action: MouseAction::Motion,
            any_button_pressed: true,
            ..press(1, 1)
        };
        assert!(
            encode_mouse(&engine.screen, motion).is_empty(),
            "asking for 1000 turns 1003 off"
        );
    }

    #[test]
    fn the_cursor_keys_follow_the_application_mode() {
        let event = KeyEvent {
            key: Key::ARROW_UP,
            shift: false,
            control: false,
            alt: false,
            text: None,
            unshifted_codepoint: None,
        };
        let normal = screen(b"");
        assert_eq!(encode_key(normal.screen.mode, event), b"\x1b[A".to_vec());
        let application = screen(b"\x1b[?1h");
        assert_eq!(
            encode_key(application.screen.mode, event),
            b"\x1bOA".to_vec()
        );
    }

    #[test]
    fn control_and_alt_fold_a_character_the_way_a_terminal_does() {
        let engine = screen(b"");
        let base = KeyEvent {
            key: Key::from_ascii('c'),
            shift: false,
            control: false,
            alt: false,
            text: Some("c"),
            unshifted_codepoint: Some('c'),
        };
        assert_eq!(encode_key(engine.screen.mode, base), b"c".to_vec());
        assert_eq!(
            encode_key(
                engine.screen.mode,
                KeyEvent {
                    control: true,
                    ..base
                }
            ),
            vec![0x03]
        );
        assert_eq!(
            encode_key(engine.screen.mode, KeyEvent { alt: true, ..base }),
            vec![0x1b, b'c']
        );
    }
}
