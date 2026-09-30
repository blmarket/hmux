use hmux2::src::reactor::{evbuffer_new, evbuffer_pullup};
use hmux2::src::server_client::Client as _;
use hmux2::src::shared::client::ClientRef;
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
            let client_owner = ClientRef::allocate();
            let mut term = tty_term::empty();
            term.flags = if no_auto_margin { TERM_NOAM } else { 0 };
            *client_owner.borrow_terminal_mut() = tty {
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
            tty_putn(&client_owner, data, width);
            assert_eq!(
                (
                    client_owner.borrow_terminal().cx,
                    client_owner.borrow_terminal().cy
                ),
                cursor
            );
            assert_eq!(
                evbuffer_pullup(
                    client_owner
                        .borrow_terminal_mut()
                        .out
                        .as_deref_mut()
                        .unwrap(),
                    -1
                )
                .unwrap_or_default(),
                expected
            );
            assert_eq!(written(&client_owner), expected.len());
            assert_eq!(
                client_owner.borrow_terminal().discarded,
                if blocked { data.len() } else { 0 }
            );
        }
    }
}

#[test]
fn repeating_spaces_preserves_chunk_boundaries_and_total_width() {
    unsafe {
        for count in [0, 1, 499, 500, 501, 1001] {
            let client_owner = ClientRef::allocate();
            let mut term = tty_term::empty();
            *client_owner.borrow_terminal_mut() = tty {
                client: std::rc::Rc::downgrade(&client_owner),
                term: Some(Box::new(term)),
                out: Some(evbuffer_new()),
                sx: 2000,
                sy: 2,
                ..Default::default()
            };
            tty_repeat_space(&client_owner, count);
            let mut terminal = client_owner.borrow_terminal_mut();
            let bytes =
                evbuffer_pullup(terminal.out.as_deref_mut().unwrap(), -1).unwrap_or_default();
            assert_eq!(bytes, vec![b' '; count as usize]);
            drop(terminal);
            assert_eq!(
                (
                    client_owner.borrow_terminal().cx,
                    client_owner.borrow_terminal().cy
                ),
                (count, 0)
            );
            assert_eq!(written(&client_owner), count as usize);
        }
    }
}

#[test]
fn raw_commands_consume_borrowed_binary_payloads_before_returning() {
    unsafe {
        let client_owner = ClientRef::allocate();
        *client_owner.borrow_terminal_mut() = tty {
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
            tty_cmd_rawstring(&client_owner, &ctx);
        }
        bytes.fill(b'X');
        let ctx = tty_ctx {
            data: tty_command_data::Bytes(&[]),
            ..Default::default()
        };
        tty_cmd_rawstring(&client_owner, &ctx);
        assert_eq!(
            evbuffer_pullup(
                client_owner
                    .borrow_terminal_mut()
                    .out
                    .as_deref_mut()
                    .unwrap(),
                -1
            )
            .unwrap(),
            b"raw\0\xff"
        );
        assert_eq!(
            (
                client_owner.borrow_terminal().cx,
                client_owner.borrow_terminal().cy
            ),
            (u32::MAX, u32::MAX)
        );
        assert_ne!(client_owner.borrow_terminal().flags & TTY_NOBLOCK, 0);
        assert_eq!(written(&client_owner), 5);
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
