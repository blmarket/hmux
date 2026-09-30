use hmux2::src::events::{events_add_sink, events_remove_sink};
use hmux2::src::events_payload::event_payload_get_string;
use hmux2::src::options::{options_create_owned, options_default, options_free};
use hmux2::src::paste::{paste_buffer_data, paste_free, paste_get_name};
use hmux2::src::server_client::Client;
use hmux2::src::shared::client::client;
use hmux2::src::shared::events::events_callback;
use hmux2::src::shared::tty::TTY_OSC52QUERY;
use hmux2::src::tty::TerminalInput;
use hmux2::src::tty_keys::tty_keys_next;
use std::io::Write;
use std::os::fd::AsRawFd;

#[test]
fn clipboard_listener_can_replace_input_after_decoding() {
    unsafe {
        let mut options = options_create_owned(None);
        let definition = hmux2::src::options_table::options_table
            .iter()
            .find(|definition| definition.name == Some(c"buffer-limit"))
            .unwrap();
        options_default(&mut *options, definition);
        let previous = std::ptr::replace(&raw mut hmux2::src::tmux::global_options, &mut *options);
        let owner = client::new();
        let observer = std::rc::Rc::downgrade(&owner);
        let (reader, mut writer) = std::os::unix::net::UnixStream::pair().unwrap();
        let mut input = TerminalInput::default();
        // The returned clipboard bytes include NUL and non-UTF-8 data.
        writer.write_all(b"\x1b]52;c;YQD/\x07").unwrap();
        assert_eq!(input.read(reader.as_raw_fd()), 12);
        {
            let terminal = owner.borrow_terminal_mut();
            terminal.in_0 = Some(Box::new(input));
            terminal.flags |= TTY_OSC52QUERY;
        }
        let sink = events_add_sink(
            c"paste-buffer-changed",
            events_callback(move |_, payload| {
                let name = event_payload_get_string(payload).unwrap();
                let buffer = paste_get_name(name).unwrap();
                assert_eq!(paste_buffer_data(&buffer.borrow()).unwrap(), b"a\0\xff");
                // Discard the decoder's original allocation during dispatch.
                // The listener sees input still present until the final drain.
                let owner = observer.upgrade().unwrap();
                {
                    let terminal = owner.borrow_terminal_mut();
                    assert_eq!(terminal.in_0.as_deref().unwrap().len(), 12);
                    terminal.in_0 = Some(Box::new(TerminalInput::default()));
                    terminal.sx = 123;
                }
                paste_free(&buffer);
            }),
        );
        assert_eq!(tty_keys_next(&owner), 1);
        assert_eq!(owner.borrow_terminal().sx, 123);
        assert!(owner.borrow_terminal().in_0.as_deref().unwrap().is_empty());
        assert_eq!(owner.borrow_terminal().flags & TTY_OSC52QUERY, 0);
        events_remove_sink(sink);
        owner.borrow_terminal_mut().in_0 = None;
        hmux2::src::tmux::global_options = previous;
        options_free(options);
    }
}
