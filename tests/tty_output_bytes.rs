use hmux2::src::reactor::{evbuffer_new, evbuffer_pullup};
use hmux2::src::shared::client::client;
use hmux2::src::shared::tty::{
    tty, tty_command_data, tty_ctx, tty_term, TERM_NOAM, TTY_BLOCK, TTY_NOBLOCK,
};
use hmux2::src::tty::{tty_cmd_rawstring, tty_putn, tty_repeat_space};

#[test]
fn byte_lengths_and_display_widths_have_distinct_clipping_and_cursor_rules() {
    unsafe {
        for (no_auto_margin, blocked, x, y, data, width, expected, cursor) in [
            (
                false,
                false,
                2,
                0,
                b"A\0B".as_slice(),
                3,
                b"A\0B".as_slice(),
                (5, 0),
            ),
            (
                false,
                false,
                6,
                0,
                "漢".as_bytes(),
                2,
                "漢".as_bytes(),
                (8, 0),
            ),
            (
                false,
                false,
                7,
                0,
                b"abc".as_slice(),
                3,
                b"abc".as_slice(),
                (2, 1),
            ),
            (
                true,
                false,
                6,
                2,
                "漢".as_bytes(),
                2,
                b"\xe6".as_slice(),
                (8, 2),
            ),
            (
                true,
                false,
                7,
                2,
                b"X".as_slice(),
                1,
                b"".as_slice(),
                (8, 2),
            ),
            (
                false,
                true,
                2,
                0,
                b"A\0B".as_slice(),
                3,
                b"".as_slice(),
                (5, 0),
            ),
            (
                false,
                false,
                2,
                0,
                b"".as_slice(),
                0,
                b"".as_slice(),
                (2, 0),
            ),
            (
                false,
                false,
                7,
                0,
                b"x".as_slice(),
                20,
                b"x".as_slice(),
                (u32::MAX, u32::MAX),
            ),
        ] {
            let client_owner = client::new();
            let client = &mut *client_owner.get();
            let mut term = tty_term::empty();
            term.flags = if no_auto_margin { TERM_NOAM } else { 0 };
            let mut terminal = tty {
                client: std::rc::Rc::downgrade(&client_owner),
                term: Some(Box::new(term)),
                out: Some(evbuffer_new()),
                sx: 8,
                sy: 3,
                cx: x,
                cy: y,
                flags: if blocked { TTY_BLOCK } else { 0 },
                ..Default::default()
            };
            tty_putn(&raw mut terminal, data, width);
            assert_eq!((terminal.cx, terminal.cy), cursor);
            assert_eq!(
                evbuffer_pullup(terminal.out.as_deref_mut().unwrap(), -1).unwrap_or_default(),
                expected
            );
            assert_eq!(client.written, expected.len());
            assert_eq!(terminal.discarded, if blocked { data.len() } else { 0 });
        }
    }
}

#[test]
fn repeating_spaces_preserves_chunk_boundaries_and_total_width() {
    unsafe {
        for count in [0, 1, 499, 500, 501, 1001] {
            let client_owner = client::new();
            let client = &mut *client_owner.get();
            let mut term = tty_term::empty();
            let mut terminal = tty {
                client: std::rc::Rc::downgrade(&client_owner),
                term: Some(Box::new(term)),
                out: Some(evbuffer_new()),
                sx: 2000,
                sy: 2,
                ..Default::default()
            };
            tty_repeat_space(&raw mut terminal, count);
            let bytes =
                evbuffer_pullup(terminal.out.as_deref_mut().unwrap(), -1).unwrap_or_default();
            assert_eq!(bytes, vec![b' '; count as usize]);
            assert_eq!((terminal.cx, terminal.cy), (count, 0));
            assert_eq!(client.written, count as usize);
        }
    }
}

#[test]
fn raw_commands_consume_borrowed_binary_payloads_before_returning() {
    unsafe {
        let client_owner = client::new();
        let client = &mut *client_owner.get();
        let mut terminal = tty {
            client: std::rc::Rc::downgrade(&client_owner),
            out: Some(evbuffer_new()),
            cx: 3,
            cy: 1,
            ..Default::default()
        };
        let mut bytes = *b"raw\0\xff";
        {
            let ctx = tty_ctx {
                data: tty_command_data::Bytes(&bytes),
                ..Default::default()
            };
            tty_cmd_rawstring(&raw mut terminal, &ctx);
        }
        bytes.fill(b'X');
        let ctx = tty_ctx {
            data: tty_command_data::Bytes(&[]),
            ..Default::default()
        };
        tty_cmd_rawstring(&raw mut terminal, &ctx);
        assert_eq!(
            evbuffer_pullup(terminal.out.as_deref_mut().unwrap(), -1).unwrap(),
            b"raw\0\xff"
        );
        assert_eq!((terminal.cx, terminal.cy), (u32::MAX, u32::MAX));
        assert_ne!(terminal.flags & TTY_NOBLOCK, 0);
        assert_eq!(client.written, 5);
    }
}
