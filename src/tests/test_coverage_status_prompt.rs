//! Unit tests for [`crate::status::status_prompt_key`] and the buffer editing
//! it drives — the paths that read and rewrite a client's `prompt_buffer`.
//!
//! `status_prompt_key` is reachable from a fixture client: it needs a session
//! for `status-keys`, an input callback to answer, and the active status
//! screen pointed at the client's own, which is the invariant
//! [`crate::status::status_prompt_set`] pushes and pops against. A fresh
//! fixture client has a zeroed status line, so [`Asker`] points that pointer
//! at the embedded screen once, on the way in; every prompt after that leaves
//! it as it found it, because opening one over another pops the screen it
//! pushed. Everything else it touches — the buffer, the cursor index, the
//! saved cut text — is plain client state these tests can read back.
//!
//! Each test drives the prompt through [`press`] with the same key strings a
//! user's `bind-key` would name, and reads the result back as a string with
//! [`buffer`]. That keeps the assertions on observable prompt behaviour rather
//! than on how the buffer happens to be allocated, so they hold across a
//! change of its representation.
//!
//! The prompt history is a set of process globals, so the test that walks it
//! empties all four slots on the way in and on the way out.
//!
//! What stays uncovered here is `status_prompt_redraw` and the completion
//! menu, both of which want a terminal.

use crate::text::{KEYC_UNKNOWN, key_string_lookup_string};
use crate::options::options_set_number;
use crate::session::session_options;
use crate::status::{
    MODEKEY_VI, PROMPT_COMMAND, PROMPT_ENTRY, PROMPT_INCREMENTAL, PROMPT_KEY, PROMPT_NTYPES,
    PROMPT_NUMERIC, PROMPT_SINGLE, PROMPT_TYPE_COMMAND, PROMPT_TYPE_SEARCH, status_prompt_clear,
    status_prompt_hlist, status_prompt_key, status_prompt_set,
};
use crate::tests::test_fixtures::{
    Clients, Target, ensure_reactor, globals, prompt_answers, prompt_answers_clear,
};
use crate::types::*;
use crate::text::utf8_vec_tocstr;
use ::core::ffi::{CStr, c_int};
use ::core::ptr::null_mut;
use ::std::ffi::CString;

/// The answers recorded since the last prompt was opened.
fn answers() -> Vec<(String, c_int)> {
    prompt_answers()
}

/// Opens a prompt on `c` holding `input`, with the recording callback and no
/// data of its own. Also clears the recorded answers, so a test reads only
/// what its own prompt reported.
unsafe fn prompt(c: *mut client, input: &CStr, flags: c_int) {
    unsafe {
        prompt_answers_clear();
        status_prompt_set(
            c,
            null_mut(),
            c":",
            Some(input),
            Prompt::Recorder,
            PromptData::None,
            flags,
            PROMPT_TYPE_COMMAND,
        );
    }
}

/// What the prompt is holding, as the string it would be accepted as.
unsafe fn buffer(c: *mut client) -> String {
    unsafe {
        utf8_vec_tocstr(&(*c).prompt_buffer)
            .to_string_lossy()
            .into_owned()
    }
}

/// Whether the prompt is still up. Accepting or cancelling takes it down.
unsafe fn prompting(c: *mut client) -> bool {
    unsafe { (*c).prompt_string.is_some() }
}

/// Sends one key, named the way `bind-key` would name it.
unsafe fn press(c: *mut client, key: &str) {
    unsafe {
        let name = CString::new(key).expect("a key name has no NUL");
        let k = key_string_lookup_string(name.as_ptr());
        assert_ne!(k, KEYC_UNKNOWN, "unknown key {key}");
        status_prompt_key(c, k);
    }
}

/// Sends each key in turn.
unsafe fn press_all(c: *mut client, keys: &[&str]) {
    unsafe {
        for key in keys {
            press(c, key);
        }
    }
}

/// Types `text` one printable ASCII character at a time.
unsafe fn type_text(c: *mut client, text: &str) {
    unsafe {
        for ch in text.chars() {
            assert!(ch.is_ascii_graphic(), "{ch:?} is not a plain key");
            press(c, &ch.to_string());
        }
    }
}

/// The text a cut left behind for pasting back.
unsafe fn saved(c: *mut client) -> Option<String> {
    unsafe {
        (*c).prompt_saved.as_ref().map(|v| {
            let mut out = String::new();
            for ud in v.iter() {
                out.push_str(&String::from_utf8_lossy(&ud.data[..ud.size as usize]));
            }
            out
        })
    }
}

/// Empties all four history slots, the way a fresh server has them.
unsafe fn drain_history() {
    unsafe {
        for t in 0..PROMPT_NTYPES as usize {
            hlists()[t].clear();
        }
    }
}

/// The four history slots, reached the way every caller reaches them.
unsafe fn hlists() -> &'static mut [Vec<::std::ffi::CString>; 4] {
    unsafe { &mut status_prompt_hlist }
}

/// A client with a session behind it, which is where the prompt reads
/// `status-keys` from.
struct Asker {
    _target: Target,
    _clients: Clients,
    c: *mut client,
}

impl Asker {
    fn new() -> Asker {
        let mut target = Target::new(80, 24);
        let mut clients = Clients::new();
        let c = clients.add("asker", 80, 24);
        unsafe {
            ensure_reactor();
            (*c).session = target.session();
            (*c).status.active = crate::types::StatusActive::Own;
        }
        Asker {
            _target: target,
            _clients: clients,
            c,
        }
    }

    /// Picks the vi editor for the prompt, which upstream reads per keypress.
    unsafe fn vi_keys(&mut self) {
        unsafe {
            options_set_number(
                session_options((*self.c).session),
                c"status-keys".as_ptr(),
                MODEKEY_VI as ::core::ffi::c_longlong,
            );
        }
    }
}

/// A fresh prompt holds the input it was given, with the cursor past its last
/// character and nothing cut yet.
#[test]
fn a_prompt_holds_its_input_with_the_cursor_at_the_end() {
    let _guard = globals();
    let mut a = Asker::new();
    let c = a.c;
    unsafe {
        prompt(c, c"hello", 0);
        assert_eq!(buffer(c), "hello");
        assert_eq!((*c).prompt_index, 5);
        assert!(saved(c).is_none());
        status_prompt_clear(c);
    }
}

/// Typing puts characters where the cursor is and moves it past them, both at
/// the end of the line and in the middle of it.
#[test]
fn typing_inserts_at_the_cursor_and_moves_it_on() {
    let _guard = globals();
    let mut a = Asker::new();
    let c = a.c;
    unsafe {
        prompt(c, c"", 0);
        type_text(c, "abc");
        assert_eq!(buffer(c), "abc");
        assert_eq!((*c).prompt_index, 3);

        press_all(c, &["C-a", "C-f"]);
        type_text(c, "XY");
        assert_eq!(buffer(c), "aXYbc");
        assert_eq!((*c).prompt_index, 3);
        status_prompt_clear(c);
    }
}

/// The buffer grows as it is typed into: it starts at the size of its input
/// and takes far more than that without losing anything.
#[test]
fn typing_grows_the_buffer_past_its_first_allocation() {
    let _guard = globals();
    let mut a = Asker::new();
    let c = a.c;
    unsafe {
        prompt(c, c"", 0);
        for i in 0..200 {
            press(c, &char::from(b'a' + (i % 26) as u8).to_string());
        }
        let want: String = (0..200)
            .map(|i| char::from(b'a' + (i % 26) as u8))
            .collect();
        assert_eq!(buffer(c), want);
        assert_eq!((*c).prompt_index, 200);

        press(c, "C-a");
        type_text(c, "Z");
        assert_eq!(buffer(c), format!("Z{want}"));
        assert_eq!((*c).prompt_index, 1);
        status_prompt_clear(c);
    }
}

/// A character that takes more than one byte still takes one place in the
/// buffer, and one step of the cursor.
#[test]
fn a_multibyte_character_takes_one_place_in_the_buffer() {
    let _guard = globals();
    let mut a = Asker::new();
    let c = a.c;
    unsafe {
        prompt(c, c"", 0);
        press(c, "\u{e9}");
        press(c, "x");
        assert_eq!(buffer(c), "\u{e9}x");
        assert_eq!((*c).prompt_index, 2);

        press(c, "BSpace");
        press(c, "BSpace");
        assert_eq!(buffer(c), "");
        assert_eq!((*c).prompt_index, 0);
        status_prompt_clear(c);
    }
}

/// The cursor steps one character at a time and jumps to either end, and
/// stops where the line does rather than running off it.
#[test]
fn the_cursor_walks_by_character_and_stops_at_both_ends() {
    let _guard = globals();
    let mut a = Asker::new();
    let c = a.c;
    unsafe {
        prompt(c, c"abcd", 0);
        press(c, "C-a");
        assert_eq!((*c).prompt_index, 0);
        press(c, "C-b");
        assert_eq!((*c).prompt_index, 0, "the cursor stops at the start");

        press(c, "C-f");
        assert_eq!((*c).prompt_index, 1);
        press(c, "Right");
        assert_eq!((*c).prompt_index, 2);
        press(c, "Left");
        assert_eq!((*c).prompt_index, 1);

        press(c, "C-e");
        assert_eq!((*c).prompt_index, 4);
        press(c, "C-f");
        assert_eq!((*c).prompt_index, 4, "the cursor stops at the end");

        press(c, "Home");
        assert_eq!((*c).prompt_index, 0);
        press(c, "End");
        assert_eq!((*c).prompt_index, 4);
        assert_eq!(buffer(c), "abcd", "walking leaves the line alone");
        status_prompt_clear(c);
    }
}

/// The cursor steps a word at a time, forward to the start of the next word
/// and back to the start of this one.
#[test]
fn the_cursor_walks_by_word() {
    let _guard = globals();
    let mut a = Asker::new();
    let c = a.c;
    unsafe {
        prompt(c, c"one two three", 0);
        press(c, "C-a");
        press(c, "M-f");
        assert_eq!((*c).prompt_index, 3, "to the end of the first word");
        press(c, "M-f");
        assert_eq!((*c).prompt_index, 7, "to the end of the second");

        press(c, "M-b");
        assert_eq!((*c).prompt_index, 4, "back to the start of the second");
        press(c, "M-b");
        assert_eq!((*c).prompt_index, 0, "back to the start of the first");
        press(c, "M-b");
        assert_eq!((*c).prompt_index, 0, "and no further");

        press(c, "C-e");
        press(c, "M-f");
        assert_eq!((*c).prompt_index, 13, "and no further the other way");
        assert_eq!(buffer(c), "one two three");
        status_prompt_clear(c);
    }
}

/// Backspace takes the character before the cursor and delete takes the one
/// under it; at either end of the line there is nothing to take.
#[test]
fn characters_go_away_before_and_at_the_cursor() {
    let _guard = globals();
    let mut a = Asker::new();
    let c = a.c;
    unsafe {
        prompt(c, c"axbc", 0);
        press_all(c, &["C-a", "C-f", "C-d"]);
        assert_eq!(buffer(c), "abc");
        assert_eq!((*c).prompt_index, 1);

        press_all(c, &["C-e", "BSpace"]);
        assert_eq!(buffer(c), "ab");
        assert_eq!((*c).prompt_index, 2);

        press(c, "DC");
        assert_eq!(buffer(c), "ab", "there is nothing under the cursor");

        press(c, "C-a");
        press(c, "BSpace");
        assert_eq!(buffer(c), "ab", "there is nothing before the cursor");
        assert_eq!((*c).prompt_index, 0);
        status_prompt_clear(c);
    }
}

/// Cutting to the end of the line, and cutting the whole line, drop what they
/// take: neither is put by for pasting back.
#[test]
fn cutting_the_line_keeps_nothing() {
    let _guard = globals();
    let a = Asker::new();
    let c = a.c;
    unsafe {
        prompt(c, c"one two three", 0);
        press_all(c, &["C-a", "M-f", "C-k"]);
        assert_eq!(buffer(c), "one");
        assert_eq!((*c).prompt_index, 3);
        assert!(saved(c).is_none());

        prompt(c, c"one two three", 0);
        press(c, "C-u");
        assert_eq!(buffer(c), "");
        assert_eq!((*c).prompt_index, 0);
        assert!(saved(c).is_none());
        status_prompt_clear(c);
    }
}

/// An empty prompt has no character anywhere for a key to read: walking or
/// cutting by word, deleting, and transposing all leave it as it was, and
/// leave it still up.
#[test]
fn keys_that_walk_the_line_have_nothing_to_walk_on_an_empty_prompt() {
    let _guard = globals();
    let mut a = Asker::new();
    let c = a.c;
    unsafe {
        prompt(c, c"", 0);
        press_all(
            c,
            &["M-f", "M-b", "C-w", "C-d", "BSpace", "C-t", "C-k", "C-u"],
        );
        assert_eq!(buffer(c), "");
        assert_eq!((*c).prompt_index, 0);
        assert!(prompting(c), "none of them accepted or cancelled");

        a.vi_keys();
        press_all(c, &["Escape", "w", "b", "e", "x", "D"]);
        assert_eq!(buffer(c), "");
        assert_eq!((*c).prompt_index, 0);
        assert!(prompting(c));
        status_prompt_clear(c);
    }
}

/// Deleting the last character of the line, and deleting until there is none
/// left, both stop at the end rather than running past it.
#[test]
fn deleting_stops_at_the_end_of_the_line() {
    let _guard = globals();
    let mut a = Asker::new();
    let c = a.c;
    unsafe {
        prompt(c, c"ab", 0);
        press_all(c, &["C-b", "C-d"]);
        assert_eq!(buffer(c), "a", "the last character goes");
        assert_eq!((*c).prompt_index, 1);

        press_all(c, &["C-a", "C-d"]);
        assert_eq!(buffer(c), "");
        assert_eq!((*c).prompt_index, 0);

        press(c, "C-d");
        assert_eq!(buffer(c), "", "there is nothing left to take");
        assert!(prompting(c));
        status_prompt_clear(c);
    }
}

/// Cutting the word behind the cursor does put it by, and cutting again puts
/// the newer word by in its place.
#[test]
fn cutting_the_word_before_the_cursor_keeps_it() {
    let _guard = globals();
    let a = Asker::new();
    let c = a.c;
    unsafe {
        prompt(c, c"one two three", 0);
        press(c, "C-w");
        assert_eq!(buffer(c), "one two ");
        assert_eq!((*c).prompt_index, 8);
        assert_eq!(saved(c).as_deref(), Some("three"));

        press(c, "C-w");
        assert_eq!(buffer(c), "one ");
        assert_eq!((*c).prompt_index, 4);
        assert_eq!(saved(c).as_deref(), Some("two "));
        status_prompt_clear(c);
    }
}

/// What was cut goes back in wherever the cursor is, at the end of the line
/// and in the middle of it, and can go in more than once.
#[test]
fn what_was_cut_is_pasted_back_at_the_cursor() {
    let _guard = globals();
    let a = Asker::new();
    let c = a.c;
    unsafe {
        prompt(c, c"one two three", 0);
        press(c, "C-w");
        assert_eq!(buffer(c), "one two ");

        press(c, "C-y");
        assert_eq!(buffer(c), "one two three", "back where it came from");
        assert_eq!((*c).prompt_index, 13);

        press(c, "C-a");
        press(c, "C-y");
        assert_eq!(buffer(c), "threeone two three", "and in at the front");
        assert_eq!((*c).prompt_index, 5);
        status_prompt_clear(c);
    }
}

/// Transposing swaps the two characters behind the cursor and steps past
/// them, and does nothing when there are not two to swap.
#[test]
fn transposing_swaps_the_two_characters_before_the_cursor() {
    let _guard = globals();
    let mut a = Asker::new();
    let c = a.c;
    unsafe {
        prompt(c, c"ab", 0);
        press(c, "C-t");
        assert_eq!(buffer(c), "ba");
        assert_eq!((*c).prompt_index, 2);

        prompt(c, c"a", 0);
        press(c, "C-t");
        assert_eq!(buffer(c), "a", "one character has nothing to swap with");
        status_prompt_clear(c);
    }
}

/// Accepting hands the line to the callback as the final answer and takes the
/// prompt down.
#[test]
fn accepting_hands_the_answer_to_the_callback() {
    let _guard = globals();
    let mut a = Asker::new();
    let c = a.c;
    unsafe {
        drain_history();
        prompt(c, c"", 0);
        type_text(c, "answer");
        press(c, "Enter");
        assert_eq!(answers(), vec![("answer".to_string(), 1)]);
        assert!(!prompting(c), "the prompt is taken down");
        drain_history();
    }
}

/// Cancelling takes the prompt down without an answer, whichever of the three
/// keys does it.
#[test]
fn cancelling_hands_nothing_to_the_callback() {
    let _guard = globals();
    let mut a = Asker::new();
    let c = a.c;
    unsafe {
        for key in ["Escape", "C-c", "C-g"] {
            prompt(c, c"typed", 0);
            press(c, key);
            assert_eq!(answers(), vec![("<none>".to_string(), 1)], "after {key}");
            assert!(!prompting(c), "the prompt is taken down by {key}");
        }
    }
}

/// Walking the history puts an older line up in place of the one being
/// edited, and walking back down comes off it again.
#[test]
fn walking_the_history_replaces_the_line() {
    let _guard = globals();
    let mut a = Asker::new();
    let c = a.c;
    unsafe {
        drain_history();

        for line in ["first", "second"] {
            prompt(c, c"", 0);
            type_text(c, line);
            press(c, "Enter");
        }
        assert_eq!(hlists()[PROMPT_TYPE_COMMAND as usize].len(), 2);

        prompt(c, c"", 0);
        press(c, "Up");
        assert_eq!(buffer(c), "second");
        assert_eq!((*c).prompt_index, 6);
        press(c, "Up");
        assert_eq!(buffer(c), "first");
        press(c, "Down");
        assert_eq!(buffer(c), "second");
        press(c, "Down");
        assert_eq!(buffer(c), "", "off the bottom is the empty line again");

        status_prompt_clear(c);
        drain_history();
    }
}

/// Each prompt type keeps its own history, so a line accepted at one does not
/// come up at another.
#[test]
fn each_prompt_type_keeps_its_own_history() {
    let _guard = globals();
    let mut a = Asker::new();
    let c = a.c;
    unsafe {
        drain_history();

        prompt(c, c"", 0);
        type_text(c, "command-line");
        press(c, "Enter");

        status_prompt_set(
            c,
            null_mut(),
            c":",
            Some(c""),
            Prompt::Recorder,
            PromptData::None,
            0,
            PROMPT_TYPE_SEARCH,
        );
        press(c, "Up");
        assert_eq!(buffer(c), "", "the search history is still empty");

        assert_eq!(hlists()[PROMPT_TYPE_COMMAND as usize].len(), 1);
        assert_eq!(hlists()[PROMPT_TYPE_SEARCH as usize].len(), 0);

        status_prompt_clear(c);
        drain_history();
    }
}

/// With the vi editor picked, escape leaves entry mode for command mode and
/// the motion and edit keys work there; `i` goes back to entry.
#[test]
fn vi_keys_move_and_edit_in_command_mode() {
    let _guard = globals();
    let mut a = Asker::new();
    let c = a.c;
    unsafe {
        a.vi_keys();
        prompt(c, c"one two three", 0);
        assert_eq!((*c).prompt_mode, PROMPT_ENTRY);

        press(c, "Escape");
        assert_eq!((*c).prompt_mode, PROMPT_COMMAND);
        assert_eq!((*c).prompt_index, 12, "the cursor comes off the end");

        press(c, "0");
        assert_eq!((*c).prompt_index, 0);
        press(c, "l");
        assert_eq!((*c).prompt_index, 1);
        press(c, "h");
        assert_eq!((*c).prompt_index, 0);
        press(c, "w");
        assert_eq!((*c).prompt_index, 4, "to the next word");
        press(c, "b");
        assert_eq!((*c).prompt_index, 0, "back a word");
        press(c, "e");
        assert_eq!((*c).prompt_index, 2, "to the end of this word");

        press(c, "0");
        press(c, "x");
        assert_eq!(buffer(c), "ne two three");

        press(c, "i");
        assert_eq!((*c).prompt_mode, PROMPT_ENTRY);
        type_text(c, "o");
        assert_eq!(buffer(c), "one two three", "typing works again");
        status_prompt_clear(c);
    }
}

/// A numeric prompt takes digits and nothing else: the first key that is not
/// one answers with what has been typed so far.
#[test]
fn a_numeric_prompt_takes_only_digits() {
    let _guard = globals();
    let mut a = Asker::new();
    let c = a.c;
    unsafe {
        prompt(c, c"", PROMPT_NUMERIC);
        type_text(c, "12");
        assert_eq!(buffer(c), "12");
        assert!(prompting(c));

        press(c, "x");
        assert_eq!(answers(), vec![("12".to_string(), 1)]);
        assert!(!prompting(c), "anything else ends it");
    }
}

/// A single-key prompt answers with the one character typed at it and goes
/// away again.
#[test]
fn a_single_key_prompt_answers_with_one_character() {
    let _guard = globals();
    let mut a = Asker::new();
    let c = a.c;
    unsafe {
        prompt(c, c"", PROMPT_SINGLE);
        press(c, "y");
        assert_eq!(answers(), vec![("y".to_string(), 1)]);
        assert!(!prompting(c));
    }
}

/// A key prompt answers with the name of whatever key was pressed, without
/// the key ever reaching the buffer.
#[test]
fn a_key_prompt_answers_with_the_key_name() {
    let _guard = globals();
    let mut a = Asker::new();
    let c = a.c;
    unsafe {
        prompt(c, c"", PROMPT_KEY);
        press(c, "C-a");
        assert_eq!(answers(), vec![("C-a".to_string(), 1)]);
        assert!(!prompting(c));
    }
}

/// An incremental prompt reports the line as it is typed, each time prefixed
/// the way the search that opened it expects, and never as final.
#[test]
fn an_incremental_prompt_reports_every_change() {
    let _guard = globals();
    let mut a = Asker::new();
    let c = a.c;
    unsafe {
        prompt(c, c"seed", PROMPT_INCREMENTAL);
        assert_eq!(
            answers(),
            vec![("=".to_string(), 0)],
            "opening reports the empty line"
        );
        assert_eq!(buffer(c), "", "the input is kept aside, not edited");

        type_text(c, "ab");
        assert_eq!(
            answers(),
            vec![
                ("=".to_string(), 0),
                ("=a".to_string(), 0),
                ("=ab".to_string(), 0),
            ]
        );

        press(c, "BSpace");
        assert_eq!(answers().last().cloned(), Some(("=a".to_string(), 0)));
        status_prompt_clear(c);
    }
}

/// A prompt starts with nothing put by for pasting, even the first one a
/// client is ever shown. Yanking then has nothing to put in rather than
/// reading from a vector that was never built: a client is handed out zeroed,
/// and a zeroed `Option<Vec<_>>` is a `Some` holding a null pointer, not the
/// `None` a zeroed C pointer stood for.
#[test]
fn a_first_prompt_has_nothing_put_by_to_paste() {
    let _guard = globals();
    let a = Asker::new();
    let c = a.c;
    unsafe {
        assert!(
            (*c).prompt_saved.is_none() || (*c).prompt_saved.as_ref().unwrap().is_empty(),
            "a fresh client has not cut anything"
        );

        prompt(c, c"abc", 0);
        assert!(
            (*c).prompt_saved.is_none(),
            "and neither has its first prompt"
        );

        press(c, "C-y");
        assert_eq!(buffer(c), "abc", "there is nothing to paste in");
        assert_eq!((*c).prompt_index, 3);
        status_prompt_clear(c);
    }
}

/// Completing a command name fills in the rest of it and leaves the cursor
/// past the space it adds, from wherever in the word the cursor stood.
#[test]
fn completing_fills_in_the_rest_of_a_command() {
    let _guard = globals();
    let a = Asker::new();
    let c = a.c;
    unsafe {
        prompt(c, c"kill-ser", 0);
        press(c, "Tab");
        assert_eq!(buffer(c), "kill-server ");
        assert_eq!((*c).prompt_index, 12);

        prompt(c, c"kill-ser", 0);
        press_all(c, &["C-a", "Tab"]);
        assert_eq!(buffer(c), "kill-server ");
        assert_eq!((*c).prompt_index, 12);

        prompt(c, c"zzzz", 0);
        press(c, "Tab");
        assert_eq!(buffer(c), "zzzz", "nothing completes it");
        status_prompt_clear(c);
    }
}

/// With the vi editor picked, each command-mode key does its own thing to the
/// line and leaves the prompt in entry or command mode as it should.
#[test]
fn vi_command_mode_keys_edit_and_move() {
    let _guard = globals();
    let mut a = Asker::new();
    let c = a.c;
    let cases: &[(&str, &str, usize, client_prompt_mode)] = &[
        ("A", "one two three", 13, PROMPT_ENTRY),
        ("I", "one two three", 0, PROMPT_ENTRY),
        ("C", "o", 1, PROMPT_ENTRY),
        ("D", "o", 1, PROMPT_COMMAND),
        ("S", "", 0, PROMPT_ENTRY),
        ("X", "ne two three", 0, PROMPT_COMMAND),
        ("$", "one two three", 13, PROMPT_COMMAND),
        ("^", "one two three", 0, PROMPT_COMMAND),
        ("d", "", 0, PROMPT_COMMAND),
    ];
    unsafe {
        a.vi_keys();
        for &(key, want, index, mode) in cases {
            prompt(c, c"one two three", 0);
            press_all(c, &["Escape", "0", "l"]);
            press(c, key);
            assert_eq!(buffer(c), want, "vi {key} leaves the line");
            assert_eq!((*c).prompt_index, index, "vi {key} leaves the cursor");
            assert_eq!((*c).prompt_mode, mode, "vi {key} leaves the mode");
        }
        status_prompt_clear(c);
    }
}

/// In vi command mode the history walks with `k` and `j` the way it does with
/// the arrow keys in entry mode.
#[test]
fn vi_command_mode_walks_the_history() {
    let _guard = globals();
    let mut a = Asker::new();
    let c = a.c;
    unsafe {
        drain_history();
        a.vi_keys();

        prompt(c, c"", 0);
        type_text(c, "remembered");
        press(c, "Enter");

        prompt(c, c"", 0);
        press(c, "Escape");
        press(c, "k");
        assert_eq!(buffer(c), "remembered");
        press(c, "j");
        assert_eq!(buffer(c), "");

        status_prompt_clear(c);
        drain_history();
    }
}
