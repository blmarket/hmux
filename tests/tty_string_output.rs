use hmux2::src::reactor::{evbuffer_drain, evbuffer_new, evbuffer_pullup};
use hmux2::src::shared::client::client;
use hmux2::src::shared::tty::{tty, tty_code, tty_term, TTYC_FSL, TTYC_SWD, TTYC_TSL};
use hmux2::src::tty::{tty_puts, tty_set_path, tty_set_title};
use std::ffi::CStr;

#[test]
fn strings_preserve_bytes_and_title_and_path_require_both_delimiters() {
    unsafe {
        let mut client = client::empty();
        let mut term = tty_term::empty();
        term.codes = vec![tty_code::None; TTYC_TSL.max(TTYC_SWD).max(TTYC_FSL) as usize + 1]
            .into_boxed_slice();
        let mut terminal = tty {
            client: &raw mut client,
            term: &raw mut term,
            out: Some(evbuffer_new()),
            cx: 4,
            cy: 2,
            ..Default::default()
        };
        let text = CStr::from_bytes_until_nul(b"raw\xff\0trailing").unwrap();
        tty_puts(&raw mut terminal, text);
        tty_puts(&raw mut terminal, c"");
        assert_eq!(
            evbuffer_pullup(terminal.out.as_deref_mut().unwrap(), -1).unwrap(),
            b"raw\xff"
        );
        evbuffer_drain(terminal.out.as_deref_mut().unwrap(), 4);
        assert_eq!((terminal.cx, terminal.cy), (4, 2));

        term.codes[TTYC_TSL as usize] = tty_code::String(c"\x1b]2;".to_owned());
        term.codes[TTYC_SWD as usize] = tty_code::String(c"\x1b]7;".to_owned());
        tty_set_title(&raw mut terminal, text);
        tty_set_path(&raw mut terminal, c"file:///tmp");
        assert!(evbuffer_pullup(terminal.out.as_deref_mut().unwrap(), -1).is_none());

        term.codes[TTYC_FSL as usize] = tty_code::String(c"\x07".to_owned());
        tty_set_title(&raw mut terminal, text);
        tty_set_path(&raw mut terminal, c"file:///tmp");
        tty_set_title(&raw mut terminal, c"");
        let expected = b"\x1b]2;raw\xff\x07\x1b]7;file:///tmp\x07\x1b]2;\x07";
        assert_eq!(
            evbuffer_pullup(terminal.out.as_deref_mut().unwrap(), -1).unwrap(),
            expected
        );
        assert_eq!(client.written, 4 + expected.len());
        assert_eq!((terminal.cx, terminal.cy), (4, 2));
    }
}
