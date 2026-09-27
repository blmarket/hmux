use hmux2::src::environ::{environ_create};
use hmux2::src::options::{options_create, options_default, options_free};
use hmux2::src::options_table::options_table;
use hmux2::src::prompt::{
    prompt_create, prompt_free, prompt_incremental_start, prompt_set_options, prompt_update,
};
use hmux2::src::shared::command::cmd_find_state;
use hmux2::src::shared::prompt::*;
use hmux2::src::tmux::{global_environ, global_options, global_s_options, global_w_options};
use std::{
    cell::RefCell,
    ffi::{CStr, CString},
    rc::Rc,
};

fn input_bytes(prompt: &prompt) -> Vec<u8> {
    prompt
        .buffer
        .iter()
        .take_while(|cell| cell.size != 0)
        .flat_map(|cell| cell.data[..cell.size as usize].iter().copied())
        .collect()
}

#[test]
fn creation_preserves_input_expansion_incremental_state_and_owned_resources() {
    unsafe {
        let saved = (
            global_environ.take(),
            global_options,
            global_s_options,
            global_w_options,
        );
        let environment = environ_create();
        global_environ = Some(environment);
        let server = options_create(std::ptr::null_mut());
        let session = options_create(std::ptr::null_mut());
        let window = options_create(std::ptr::null_mut());
        global_options = server;
        global_s_options = session;
        global_w_options = window;
        for name in [
            c"message-style",
            c"message-command-style",
            c"prompt-cursor-style",
            c"prompt-command-cursor-style",
            c"prompt-cursor-colour",
            c"prompt-command-cursor-colour",
            c"message-format",
            c"status-keys",
            c"word-separators",
        ] {
            let definition = (&*std::ptr::addr_of!(options_table))
                .iter()
                .find(|entry| !entry.name.is_null() && CStr::from_ptr(entry.name) == name)
                .unwrap();
            options_default(session, definition);
        }
        let events = Rc::new(RefCell::new(Vec::new()));
        let mut state = cmd_find_state {
            flags: 77,
            idx: 9,
            ..Default::default()
        };
        state.current = &raw mut state;
        for flags in [
            0,
            PROMPT_NOFORMAT,
            PROMPT_INCREMENTAL,
            PROMPT_INCREMENTAL | PROMPT_NOFORMAT,
        ] {
            events.borrow_mut().clear();
            let label = CString::new("label: ").unwrap();
            let input = CString::new("#{==:a,a}漢").unwrap();
            let mut pd = prompt_create_data {
                fs: Some(&state),
                prompt: &label,
                input: Some(&input),
                flags,
                ..Default::default()
            };
            prompt_set_options(&mut pd, None);
            let option_addresses = (
                pd.style_str.as_ptr(),
                pd.command_style_str.as_ptr(),
                pd.message_format.as_ptr(),
                pd.word_separators.as_ptr(),
            );
            let inputs = events.clone();
            pd.inputcb = Some(Box::new(move |input, _| {
                inputs.borrow_mut().push(input.unwrap().to_owned());
                PROMPT_CONTINUE
            }));
            let freed = events.clone();
            pd.freecb = Some(Box::new(move || {
                freed.borrow_mut().push(c"freed".to_owned())
            }));
            let prompt = prompt_create(pd);
            drop(label);
            drop(input);
            assert_eq!(prompt.borrow().string.as_c_str(), c"label: ");
            assert_eq!(prompt.borrow().state.idx, 9);
            assert_eq!(prompt.borrow().state.flags, 0);
            assert!(prompt.borrow().state.current.is_null());
            assert_eq!(
                (
                    prompt.borrow().style_str.as_ptr(),
                    prompt.borrow().command_style_str.as_ptr(),
                    prompt.borrow().message_format.as_ptr(),
                    prompt.borrow().word_separators.as_ptr()
                ),
                option_addresses
            );
            let expected = if flags & PROMPT_NOFORMAT != 0 {
                c"#{==:a,a}漢"
            } else {
                c"1漢"
            };
            if flags & PROMPT_INCREMENTAL != 0 {
                assert_eq!(prompt.borrow().last.as_deref(), Some(expected));
                assert_eq!(prompt.borrow().index, 0);
                assert_eq!(input_bytes(&prompt.borrow()), b"");
            } else {
                assert!(prompt.borrow().last.is_none());
                assert_eq!(
                    prompt.borrow().index,
                    expected.to_str().unwrap().chars().count()
                );
                assert_eq!(input_bytes(&prompt.borrow()), expected.to_bytes());
            }
            assert!(events.borrow().is_empty());
            prompt_incremental_start(&prompt);
            let expected_events = if flags & PROMPT_INCREMENTAL != 0 {
                vec![c"=".to_owned()]
            } else {
                vec![]
            };
            assert_eq!(*events.borrow(), expected_events);
            {
                let mut state = prompt.borrow_mut();
                state.closed = 1;
                state.hindex = [2, 3];
                state.completion.names.push(c"stale".to_owned());
                state.completion.display = Some(c"stale".to_owned());
                prompt_update(&mut state, c"updated: ", Some(c"#{==:b,b}é"));
                let expected = if flags & PROMPT_NOFORMAT != 0 {
                    c"#{==:b,b}é"
                } else {
                    c"1é"
                };
                assert_eq!(input_bytes(&state), expected.to_bytes());
                assert_eq!(state.index, expected.to_str().unwrap().chars().count());
                assert_eq!(state.string.as_c_str(), c"updated: ");
                assert_eq!(state.hindex, [0, 0]);
                assert_eq!(state.closed, 0);
                assert!(state.completion.names.is_empty());
                assert!(state.completion.display.is_none());
                prompt_update(&mut state, c"empty: ", None);
                assert_eq!(input_bytes(&state), b"");
                assert_eq!(state.index, 0);
            }
            assert_eq!(*events.borrow(), expected_events);
            prompt_free(&prompt);
            let mut expected_events = expected_events;
            expected_events.push(c"freed".to_owned());
            assert_eq!(*events.borrow(), expected_events);
            assert_eq!(Rc::strong_count(&events), 1);
        }
        let prompt = prompt_create(prompt_create_data {
            flags: PROMPT_NOFORMAT,
            ..Default::default()
        });
        assert_eq!(prompt.borrow().state.idx, -1);
        assert_eq!(prompt.borrow().index, 0);
        assert_eq!(input_bytes(&prompt.borrow()), b"");
        prompt_free(&prompt);
        (
            global_environ,
            global_options,
            global_s_options,
            global_w_options,
        ) = saved;
        options_free(server);
        options_free(session);
        options_free(window);
    }
}
