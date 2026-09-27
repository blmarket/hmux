use hmux2::src::reactor::{
    evbuffer_drain, evbuffer_new, evbuffer_pullup, event_del, event_set, shutdown_runtime,
};
use hmux2::src::shared::client::client;
use hmux2::src::shared::event::EV_WRITE;
use hmux2::src::shared::tty::{tty, tty_code, tty_term, TTYC_MS, TTY_NOBLOCK, TTY_STARTED};
use hmux2::src::tty::tty_set_selection;
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;

#[test]
fn selection_output_copies_binary_payloads_and_honours_terminal_capabilities() {
    unsafe {
        let (writer, _reader) = UnixStream::pair().unwrap();
        let mut client = client::empty();
        let mut term = tty_term::empty();
        term.codes = vec![tty_code::None; TTYC_MS as usize + 1].into_boxed_slice();
        let mut terminal = tty {
            client: &raw mut client,
            term: &raw mut term,
            out: Some(evbuffer_new()),
            ..Default::default()
        };
        event_set(
            &raw mut terminal.event_out,
            writer.as_raw_fd(),
            EV_WRITE as i16,
            |_, _| {},
        );
        let capability = c"\x1b]52;%p1%s;%p2%s\x07";
        term.codes[TTYC_MS as usize] = tty_code::String(capability.to_owned());
        tty_set_selection(&raw mut terminal, c"c", b"ignored while stopped");
        assert!(evbuffer_pullup(terminal.out.as_deref_mut().unwrap(), -1).is_none());
        terminal.flags = TTY_STARTED;
        term.codes[TTYC_MS as usize] = tty_code::None;
        tty_set_selection(&raw mut terminal, c"c", b"ignored without capability");
        assert!(evbuffer_pullup(terminal.out.as_deref_mut().unwrap(), -1).is_none());
        term.codes[TTYC_MS as usize] = tty_code::String(capability.to_owned());
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
            tty_set_selection(&raw mut terminal, selector, &bytes);
            bytes.fill(b'X');
            assert_ne!(terminal.flags & TTY_NOBLOCK, 0);
            let output = terminal.out.as_deref_mut().unwrap();
            assert_eq!(evbuffer_pullup(output, -1).unwrap(), expected);
            evbuffer_drain(output, expected.len());
            event_del(&raw mut terminal.event_out);
        }
        event_del(&raw mut terminal.event_out);
        shutdown_runtime();
    }
}
