//! Prompt history keeps stable borrowed C strings while entries remain live.

use hmux2::src::format::bytes::write_cstr;
use hmux2::src::options::{
    options_create, options_default, options_free, options_set_number, options_set_string,
};
use hmux2::src::options_table::options_table;
use hmux2::src::prompt_history::{
    prompt_add_history, prompt_down_history, prompt_history_clear, prompt_history_get,
    prompt_history_size, prompt_load_history, prompt_save_history, prompt_up_history,
};
use hmux2::src::shared::prompt::{PROMPT_TYPE_COMMAND, PROMPT_TYPE_SEARCH};
use hmux2::src::tmux::global_options;
use std::ffi::{CStr, CString};
use std::os::unix::ffi::OsStrExt;

#[test]
fn history_owns_entries_and_preserves_order_pruning_and_navigation() {
    unsafe {
        let previous_options = global_options;
        let options = options_create(std::ptr::null_mut());
        for name in [c"prompt-history-limit", c"history-file"] {
            let definition = options_table
                .iter()
                .find(|entry| !entry.name.is_null() && CStr::from_ptr(entry.name) == name)
                .unwrap();
            options_default(options, definition);
        }
        global_options = options;

        prompt_history_clear(PROMPT_TYPE_COMMAND);
        prompt_history_clear(PROMPT_TYPE_SEARCH);
        assert_eq!(prompt_history_size(PROMPT_TYPE_COMMAND), 0);
        assert!(prompt_history_get(PROMPT_TYPE_COMMAND, 0).is_null());
        assert!(prompt_history_get(255, 0).is_null());

        options_set_number(options, c"prompt-history-limit".as_ptr(), 200);
        prompt_add_history(c"\xff-first".as_ptr(), PROMPT_TYPE_COMMAND);
        let first = prompt_history_get(PROMPT_TYPE_COMMAND, 0);
        for n in 0..128 {
            let text = CString::new(format!("entry-{n}")).unwrap();
            prompt_add_history(text.as_ptr(), PROMPT_TYPE_COMMAND);
        }
        assert_eq!(prompt_history_get(PROMPT_TYPE_COMMAND, 0), first);
        assert_eq!(CStr::from_ptr(first).to_bytes(), b"\xff-first");
        assert_eq!(prompt_history_size(PROMPT_TYPE_COMMAND), 129);
        prompt_add_history(c"entry-127".as_ptr(), PROMPT_TYPE_COMMAND);
        assert_eq!(prompt_history_size(PROMPT_TYPE_COMMAND), 129);

        prompt_history_clear(PROMPT_TYPE_COMMAND);
        options_set_number(options, c"prompt-history-limit".as_ptr(), 3);
        for text in [c"one", c"two", c"three"] {
            prompt_add_history(text.as_ptr(), PROMPT_TYPE_COMMAND);
        }
        let middle = prompt_history_get(PROMPT_TYPE_COMMAND, 1);
        prompt_add_history(c"four".as_ptr(), PROMPT_TYPE_COMMAND);
        assert_eq!(prompt_history_size(PROMPT_TYPE_COMMAND), 3);
        assert_eq!(prompt_history_get(PROMPT_TYPE_COMMAND, 0), middle);
        assert_eq!(CStr::from_ptr(middle), c"two");
        assert_eq!(
            CStr::from_ptr(prompt_history_get(PROMPT_TYPE_COMMAND, 2)),
            c"four"
        );

        // A duplicate at the limit does not add a row, but reducing the
        // limit still prunes the oldest rows.
        options_set_number(options, c"prompt-history-limit".as_ptr(), 2);
        prompt_add_history(c"four".as_ptr(), PROMPT_TYPE_COMMAND);
        assert_eq!(prompt_history_size(PROMPT_TYPE_COMMAND), 2);
        assert_eq!(
            CStr::from_ptr(prompt_history_get(PROMPT_TYPE_COMMAND, 0)),
            c"three"
        );

        let mut indexes = [0_u32; 2];
        assert_eq!(
            CStr::from_ptr(prompt_up_history(indexes.as_mut_ptr(), PROMPT_TYPE_COMMAND)),
            c"four"
        );
        assert_eq!(
            CStr::from_ptr(prompt_up_history(indexes.as_mut_ptr(), PROMPT_TYPE_COMMAND)),
            c"three"
        );
        assert!(prompt_up_history(indexes.as_mut_ptr(), PROMPT_TYPE_COMMAND).is_null());
        assert_eq!(
            CStr::from_ptr(prompt_down_history(
                indexes.as_mut_ptr(),
                PROMPT_TYPE_COMMAND
            )),
            c"four"
        );
        assert_eq!(
            CStr::from_ptr(prompt_down_history(
                indexes.as_mut_ptr(),
                PROMPT_TYPE_COMMAND
            )),
            c""
        );

        // The input can itself be a borrowed entry. Copy it before the old
        // entry is pruned, so the new entry gets the same bytes.
        let oldest = prompt_history_get(PROMPT_TYPE_COMMAND, 0);
        prompt_add_history(oldest, PROMPT_TYPE_COMMAND);
        assert_eq!(
            CStr::from_ptr(prompt_history_get(PROMPT_TYPE_COMMAND, 0)),
            c"four"
        );
        assert_eq!(
            CStr::from_ptr(prompt_history_get(PROMPT_TYPE_COMMAND, 1)),
            c"three"
        );

        let path =
            std::env::temp_dir().join(format!("hmux2-prompt-history-{}", std::process::id()));
        let path_string = CString::new(path.as_os_str().as_bytes()).unwrap();
        options_set_string(options, c"history-file".as_ptr(), 0, |out| {
            write_cstr(out, path_string.as_ptr())
        });
        prompt_save_history();
        assert_eq!(
            std::fs::read(&path).unwrap(),
            b"command:four\ncommand:three\n"
        );
        prompt_history_clear(PROMPT_TYPE_COMMAND);
        prompt_load_history();
        assert_eq!(prompt_history_size(PROMPT_TYPE_COMMAND), 2);
        assert_eq!(
            CStr::from_ptr(prompt_history_get(PROMPT_TYPE_COMMAND, 0)),
            c"four"
        );
        assert_eq!(
            CStr::from_ptr(prompt_history_get(PROMPT_TYPE_COMMAND, 1)),
            c"three"
        );
        prompt_history_clear(PROMPT_TYPE_COMMAND);
        std::fs::write(&path, b"command:one\0ignored\ncommand:two").unwrap();
        prompt_load_history();
        assert_eq!(prompt_history_size(PROMPT_TYPE_COMMAND), 2);
        assert_eq!(
            CStr::from_ptr(prompt_history_get(PROMPT_TYPE_COMMAND, 0)),
            c"one"
        );
        assert_eq!(
            CStr::from_ptr(prompt_history_get(PROMPT_TYPE_COMMAND, 1)),
            c"two"
        );

        prompt_history_clear(PROMPT_TYPE_COMMAND);
        prompt_history_clear(PROMPT_TYPE_SEARCH);
        options_set_number(options, c"prompt-history-limit".as_ptr(), 10);
        std::fs::write(
            &path,
            b"plain\nunknown:value\n:empty-prefix\ncommand:\nsearch:a:b\ncommand:first:second\n",
        )
        .unwrap();
        prompt_load_history();
        let command_history = [
            b"plain".as_slice(),
            b"unknown:value".as_slice(),
            b":empty-prefix".as_slice(),
            b"".as_slice(),
            b"first:second".as_slice(),
        ];
        assert_eq!(
            prompt_history_size(PROMPT_TYPE_COMMAND),
            command_history.len() as u32
        );
        for (index, expected) in command_history.into_iter().enumerate() {
            assert_eq!(
                CStr::from_ptr(prompt_history_get(PROMPT_TYPE_COMMAND, index as u32)).to_bytes(),
                expected
            );
        }
        assert_eq!(prompt_history_size(PROMPT_TYPE_SEARCH), 1);
        assert_eq!(
            CStr::from_ptr(prompt_history_get(PROMPT_TYPE_SEARCH, 0)).to_bytes(),
            b"a:b"
        );
        std::fs::remove_file(path).unwrap();

        options_set_number(options, c"prompt-history-limit".as_ptr(), 0);
        prompt_add_history(c"ignored".as_ptr(), PROMPT_TYPE_COMMAND);
        assert_eq!(prompt_history_size(PROMPT_TYPE_COMMAND), 0);
        prompt_add_history(c"search".as_ptr(), PROMPT_TYPE_SEARCH);
        assert_eq!(prompt_history_size(PROMPT_TYPE_SEARCH), 0);

        prompt_history_clear(PROMPT_TYPE_COMMAND);
        prompt_history_clear(PROMPT_TYPE_SEARCH);
        global_options = previous_options;
        options_free(options);
    }
}
