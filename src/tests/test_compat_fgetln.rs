//! What is left uncovered here is the two allocator arms — the first
//! buffer that could not be made and the doubling that could not be done.
//! The C allocator is what hands those out, and a test cannot make it
//! refuse.

use super::*;
use crate::ffi::{fclose, fdopen};
use ::core::ffi::{c_int, c_void};
use ::core::ptr::null_mut;
use ::std::sync::MutexGuard;

/// A turn at the buffer every line is read into, which is this module's
/// own static and grows as long a line needs it to. Cargo runs the tests
/// on parallel threads, so every test that reads a line holds this.
fn buffer() -> MutexGuard<'static, ()> {
    static BUFFER: ::std::sync::Mutex<()> = ::std::sync::Mutex::new(());
    BUFFER
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// A file over the read end of a pipe holding `text`, with the write end
/// closed already so that reading past the text reaches the end of the
/// file. A pipe holds far more than any of these tests writes.
struct Reader(*mut FILE);

impl Reader {
    fn new(text: &[u8]) -> Reader {
        unsafe {
            let mut fds = [-1 as c_int; 2];
            assert_eq!(::libc::pipe(fds.as_mut_ptr()), 0, "no pipe");
            assert_eq!(
                ::libc::write(fds[1], text.as_ptr() as *const c_void, text.len()),
                text.len() as ::libc::ssize_t
            );
            ::libc::close(fds[1]);
            let f = fdopen(fds[0], c"r".as_ptr());
            assert!(!f.is_null(), "no file");
            Reader(f)
        }
    }

    /// The next line, or nothing at the end of the file. What comes back
    /// is not a C string — it is as long as the length says and carries no
    /// terminator — so it is read as bytes.
    fn line(&self) -> Option<Vec<u8>> {
        unsafe {
            let mut len: size_t = 0;
            let p = fgetln(self.0, &raw mut len);
            if p.is_null() {
                assert_eq!(len, 0);
                return None;
            }
            Some(::core::slice::from_raw_parts(p as *const u8, len as usize).to_vec())
        }
    }
}

impl Drop for Reader {
    fn drop(&mut self) {
        unsafe { fclose(self.0) };
    }
}

#[test]
fn lines_come_back_one_at_a_time_and_keep_their_newlines() {
    let _guard = buffer();
    let reader = Reader::new(b"one\ntwo\n");
    assert_eq!(reader.line().as_deref(), Some(&b"one\n"[..]));
    assert_eq!(reader.line().as_deref(), Some(&b"two\n"[..]));
    assert_eq!(reader.line(), None);
}

/// A file whose last line has no newline hands that line over as it
/// stands, and only the read after it reaches the end.
#[test]
fn a_last_line_with_no_newline_comes_back_as_it_stands() {
    let _guard = buffer();
    let reader = Reader::new(b"one\ntwo");
    assert_eq!(reader.line().as_deref(), Some(&b"one\n"[..]));
    assert_eq!(reader.line().as_deref(), Some(&b"two"[..]));
    assert_eq!(reader.line(), None);
}

#[test]
fn a_file_holding_nothing_answers_nothing() {
    let _guard = buffer();
    assert_eq!(Reader::new(b"").line(), None);
}

/// An empty line is one byte long, so it is a line like any other; it is
/// only a line of no length at all that answers nothing.
#[test]
fn an_empty_line_is_still_a_line() {
    let _guard = buffer();
    let reader = Reader::new(b"\n\n");
    assert_eq!(reader.line().as_deref(), Some(&b"\n"[..]));
    assert_eq!(reader.line().as_deref(), Some(&b"\n"[..]));
    assert_eq!(reader.line(), None);
}

/// The buffer starts at `BUFSIZ` bytes and doubles whenever a line fills
/// it, so a line of any length comes back whole.
#[test]
fn a_line_longer_than_the_buffer_grows_it() {
    let _guard = buffer();
    let mut text = vec![b'a'; 3 * BUFSIZ as usize];
    text.push(b'\n');
    text.extend_from_slice(b"after\n");
    let reader = Reader::new(&text);
    let line = reader.line().expect("a line");
    assert_eq!(line.len(), 3 * BUFSIZ as usize + 1);
    assert!(line.iter().take(line.len() - 1).all(|b| *b == b'a'));
    assert_eq!(reader.line().as_deref(), Some(&b"after\n"[..]));
    assert_eq!(reader.line(), None);
}

#[test]
fn no_file_and_nowhere_to_put_the_length_are_both_refused() {
    let _guard = buffer();
    let reader = Reader::new(b"one\n");
    unsafe {
        let mut len: size_t = 7;
        *__errno_location() = 0;
        assert!(fgetln(null_mut::<FILE>(), &raw mut len).is_null());
        assert_eq!(*__errno_location(), EINVAL);
        assert_eq!(len, 7);

        *__errno_location() = 0;
        assert!(fgetln(reader.0, null_mut::<size_t>()).is_null());
        assert_eq!(*__errno_location(), EINVAL);
    }
}
