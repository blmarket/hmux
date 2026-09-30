use hmux2::src::shared::client::ClientRef;
use hmux2::src::reactor::{evbuffer_drain, evbuffer_new, evbuffer_pullup, shutdown_runtime};
use hmux2::src::server_client::Client;
use hmux2::src::shared::tty::{
    tty, tty_code, tty_command_data, tty_ctx, tty_term, TTYC_MS, TTY_NOBLOCK, TTY_STARTED,
};
use hmux2::src::tty::tty_set_selection;
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;

#[test]
fn selection_output_copies_binary_payloads_and_honours_terminal_capabilities() {
    unsafe {
        let (writer, _reader) = UnixStream::pair().unwrap();
        writer.set_nonblocking(true).unwrap();
        let client_owner = ClientRef::allocate();
        let mut term = tty_term::empty();
        term.codes = vec![tty_code::None; TTYC_MS as usize + 1].into_boxed_slice();
        let capability = c"\x1b]52;%p1%s;%p2%s\x07";
        {
            let mut terminal = client_owner.borrow_terminal_mut();
            *terminal = tty {
                client: std::rc::Rc::downgrade(&client_owner),
                term: Some(Box::new(term)),
                out: Some(evbuffer_new()),
                io_fd: Some(writer.as_raw_fd()),
                ..Default::default()
            };
            terminal.term.as_deref_mut().unwrap().codes[TTYC_MS as usize] =
                tty_code::String(capability.to_owned());
        }
        tty_set_selection(&client_owner, c"c", b"ignored while stopped");
        {
            let mut terminal = client_owner.borrow_terminal_mut();
            assert!(evbuffer_pullup(terminal.out.as_deref_mut().unwrap(), -1).is_none());
            terminal.flags = TTY_STARTED;
            terminal.term.as_deref_mut().unwrap().codes[TTYC_MS as usize] = tty_code::None;
        }
        tty_set_selection(&client_owner, c"c", b"ignored without capability");
        {
            let mut terminal = client_owner.borrow_terminal_mut();
            assert!(evbuffer_pullup(terminal.out.as_deref_mut().unwrap(), -1).is_none());
            terminal.term.as_deref_mut().unwrap().codes[TTYC_MS as usize] =
                tty_code::String(capability.to_owned());
        }
        for (selector, input, expected) in [
            (c"c", b"A\0B".as_slice(), b"\x1b]52;c;QQBC\x07".as_slice()),
            (c"", b"hi".as_slice(), b"\x1b]52;;aGk=\x07".as_slice()),
            (
                c"ps7",
                b"\xff\x80\0\x01".as_slice(),
                b"\x1b]52;ps7;/4AAAQ==\x07".as_slice(),
            ),
            (c"c", b"".as_slice(), b"\x1b]52;c;\x07".as_slice()),
        ] {
            let mut bytes = input.to_vec();
            {
                let ctx = tty_ctx {
                    data: tty_command_data::Selection {
                        clip: selector,
                        data: &bytes,
                    },
                    ..Default::default()
                };
                let (clip, data) = ctx.data.selection();
                tty_set_selection(&client_owner, clip, data);
            }
            bytes.fill(b'X');
            let mut terminal = client_owner.borrow_terminal_mut();
            assert_ne!(terminal.flags & TTY_NOBLOCK, 0);
            let output = terminal.out.as_deref_mut().unwrap();
            assert_eq!(evbuffer_pullup(output, -1).unwrap(), expected);
            evbuffer_drain(output, expected.len());
            terminal.write_task.cancel();
        }
        client_owner.borrow_terminal_mut().write_task.cancel();
        shutdown_runtime();
    }
}
