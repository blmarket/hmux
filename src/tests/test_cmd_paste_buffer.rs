use super::*;
use crate::tests::test_fixtures::{Args, Item, Paste, StreamBuffer, Target, globals};

/// A pane the exec routine will write to: not exited, with its writes
/// going to `bev`. The descriptor number is only ever read.
unsafe fn attach(wp: *mut window_pane, bev: &StreamBuffer) {
    unsafe {
        (*wp).fd = 1000;
        (*wp).event = bev.ptr();
    }
}

#[test]
fn the_separator_is_s_before_r_before_a_carriage_return() {
    let _guard = globals();
    unsafe {
        assert_eq!(separator(&*Args::parse(c"paste-buffer").ptr()), c"\r");
        assert_eq!(separator(&*Args::parse(c"paste-buffer -r").ptr()), c"\n");
        assert_eq!(separator(&*Args::parse(c"paste-buffer -s XY").ptr()), c"XY");
        assert_eq!(
            separator(&*Args::parse(c"paste-buffer -r -s XY").ptr()),
            c"XY",
            "-s is read first, so it wins over -r"
        );
    }
}

#[test]
fn buffer_bytes_is_what_the_store_holds_newlines_and_all() {
    let _guard = globals();
    let store = Paste::new();
    unsafe {
        assert_eq!(buffer_bytes(store.add(c"lines", "ab\ncd\n")), b"ab\ncd\n");
        assert_eq!(buffer_bytes(store.add(c"one", "x")), b"x");
    }
}

#[test]
fn send_line_escapes_unless_it_is_asked_for_the_bytes_raw() {
    let _guard = globals();
    let bev = StreamBuffer::new();
    let mut t = Target::new(10, 3);
    unsafe {
        let wp = t.pane(0);
        attach(wp, &bev);

        send_line(wp, b"a\x01b", true);
        assert_eq!(bev.written(), b"a\x01b");

        send_line(wp, b"a\x01b", false);
        assert_eq!(bev.written(), b"a^Ab", "the control byte is made visible");

        send_line(wp, b"", false);
        assert!(
            bev.written().is_empty(),
            "an empty line puts nothing in front of its separator"
        );
    }
}

#[test]
fn a_run_of_newlines_sends_a_separator_for_each_of_the_empty_lines() {
    let _guard = globals();
    let store = Paste::new();
    let bev = StreamBuffer::new();
    let mut t = Target::new(10, 3);
    unsafe {
        store.add(c"gappy", "a\n\n\nb");
        attach(t.pane(0), &bev);

        let mut it = Item::new()
            .with_args(c"paste-buffer -b gappy")
            .targeting(&mut t);
        let run = cmd_paste_buffer_entry.exec;
        assert_eq!(run(&*it.cmd(), it.ptr()), CMD_RETURN_NORMAL);

        assert_eq!(
            bev.written(),
            b"a\r\r\rb",
            "each newline closes a line, and the two empty ones send only \
             their separators"
        );
    }
}

#[test]
fn a_buffer_ending_in_a_newline_sends_nothing_after_its_last_separator() {
    let _guard = globals();
    let store = Paste::new();
    let bev = StreamBuffer::new();
    let mut t = Target::new(10, 3);
    unsafe {
        store.add(c"closed", "ab\n");
        attach(t.pane(0), &bev);

        let mut it = Item::new()
            .with_args(c"paste-buffer -b closed")
            .targeting(&mut t);
        let run = cmd_paste_buffer_entry.exec;
        assert_eq!(run(&*it.cmd(), it.ptr()), CMD_RETURN_NORMAL);

        assert_eq!(bev.written(), b"ab\r");
    }
}
