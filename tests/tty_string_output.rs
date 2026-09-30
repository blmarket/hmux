use hmux2::src::reactor::{evbuffer_drain, evbuffer_new, evbuffer_pullup};
use hmux2::src::server_client::Client as _;
use hmux2::src::shared::client::client;
use hmux2::src::shared::client::ClientRef;
use hmux2::src::shared::tty::{tty, tty_code, tty_term, TTYC_FSL, TTYC_SWD, TTYC_TSL};
use hmux2::src::tty::{tty_puts, tty_set_path, tty_set_title};
use std::ffi::CStr;

#[test]
fn strings_preserve_bytes_and_title_and_path_require_both_delimiters() {
    unsafe {
        let client_owner = client::new();
        let mut term = tty_term::empty();
        term.codes = vec![tty_code::None; TTYC_TSL.max(TTYC_SWD).max(TTYC_FSL) as usize + 1]
            .into_boxed_slice();
        *client_owner.borrow_terminal_mut() = tty {
            client: std::rc::Rc::downgrade(&client_owner),
            term: Some(Box::new(term)),
            out: Some(evbuffer_new()),
            cx: 4,
            cy: 2,
            ..Default::default()
        };
        let text = CStr::from_bytes_until_nul(b"raw\xff\0trailing").unwrap();
        tty_puts(&client_owner, text);
        tty_puts(&client_owner, c"");
        {
            let mut terminal = client_owner.borrow_terminal_mut();
            assert_eq!(
                evbuffer_pullup(terminal.out.as_deref_mut().unwrap(), -1).unwrap(),
                b"raw\xff"
            );
            evbuffer_drain(terminal.out.as_deref_mut().unwrap(), 4);
            assert_eq!((terminal.cx, terminal.cy), (4, 2));
            terminal.term.as_deref_mut().unwrap().codes[TTYC_TSL as usize] =
                tty_code::String(c"\x1b]2;".to_owned());
            terminal.term.as_deref_mut().unwrap().codes[TTYC_SWD as usize] =
                tty_code::String(c"\x1b]7;".to_owned());
        }
        tty_set_title(&client_owner, text);
        tty_set_path(&client_owner, c"file:///tmp");
        {
            let mut terminal = client_owner.borrow_terminal_mut();
            assert!(evbuffer_pullup(terminal.out.as_deref_mut().unwrap(), -1).is_none());
            terminal.term.as_deref_mut().unwrap().codes[TTYC_FSL as usize] =
                tty_code::String(c"\x07".to_owned());
        }
        tty_set_title(&client_owner, text);
        tty_set_path(&client_owner, c"file:///tmp");
        tty_set_title(&client_owner, c"");
        let expected = b"\x1b]2;raw\xff\x07\x1b]7;file:///tmp\x07\x1b]2;\x07";
        {
            let mut terminal = client_owner.borrow_terminal_mut();
            assert_eq!(
                evbuffer_pullup(terminal.out.as_deref_mut().unwrap(), -1).unwrap(),
                expected
            );
            assert_eq!((terminal.cx, terminal.cy), (4, 2));
        }
        let expected_written = 4 + expected.len();
        assert_eq!(written(&client_owner), expected_written);
    }
}

unsafe fn written(owner: &ClientRef) -> usize {
    let mut context = hmux2::src::shared::format::format_tree {
        c: std::rc::Rc::downgrade(owner),
        ..Default::default()
    };
    let Some(hmux2::src::format::FormatValue::String(value)) =
        owner.format_value(c"client_written", &mut context)
    else {
        panic!("client output accounting builtin");
    };
    value.to_str().unwrap().parse().unwrap()
}
