use crate::WindowPane;
use crate::cmd::CMD_RETURN_NORMAL;
use crate::cmd::{CmdqListOps, cmdq_next};
use crate::paste::{PasteBufferStore, with_paste_buffers, with_paste_buffers_mut};
use crate::status::{status_init, status_prompt_clear};
use crate::tests::test_fixtures::{Clients, Item, Target, globals, seen};
use crate::types::*;
use crate::window::window_pane_reset_mode_all;
use crate::window::{window_pane_current_mode, window_pane_current_mode_mut};
use ::core::ffi::CStr;

const FILE: &CStr = c"test_coverage_window_buffer.rs";

unsafe fn clear_buffers() {
    let names = with_paste_buffers(|buffers| {
        buffers
            .buffers()
            .map(|buffer| buffer.name.to_owned())
            .collect::<Vec<_>>()
    });
    for name in names {
        with_paste_buffers_mut(|buffers| buffers.remove(name.as_c_str()));
    }
}

#[test]
fn test_window_buffer_mode_lifecycle_and_keys() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut t = Target::new(80, 24);

    unsafe {
        clear_buffers();
        with_paste_buffers_mut(|buffers| {
            buffers.add_automatic(None, b"first buffer text\n".to_vec(), 50);
            buffers.add_automatic(None, b"second buffer line\n".to_vec(), 50);
        });

        let c1 = clients.add("client-1", 80, 24);
        (*c1).set_attached_session(Some(t.session_handle()));
        (*c1).queue = Some(CmdqListRef::empty());
        status_init(&mut *c1);

        let wp = t.pane(0);

        let item = Item::with_client()
            .with_file(FILE, 1)
            .with_args(c"choose-buffer")
            .targeting(&mut t);

        let exec = crate::cmd::cmd_find(c"choose-buffer").unwrap().exec;
        assert_eq!(
            item.with_command(|command, item| exec(command, item)),
            CMD_RETURN_NORMAL
        );

        let wme = window_pane_current_mode_mut(&mut *wp).expect("pane is in a mode");
        assert_eq!(wme.mode(), WindowMode::Buffer);
        assert_eq!(seen(wme.mode().name().as_ptr()), "buffer-mode");
        assert!(wme.mode().default_format().is_some());

        // Update and resize
        wme.update_target().unwrap().dispatch();
        let wme = window_pane_current_mode_mut(&mut *wp).expect("pane is in a mode");
        wme.mode().resize(wme, 90, 28);

        // Key interactions
        for key in [
            b'j' as key_code,
            b'k' as key_code,
            b't' as key_code,
            b' ' as key_code,
            b'?' as key_code,
            b'v' as key_code,
            b'P' as key_code,
            b'D' as key_code,
            b'p' as key_code,
            b'd' as key_code,
            b'\r' as key_code,
            b'q' as key_code,
        ] {
            if !(*wp).modes().is_empty() {
                let cur_wme = window_pane_current_mode_mut(&mut *wp).expect("pane is in a mode");
                cur_wme.key_target().unwrap().dispatch(&mut *c1, key, None);
            }
        }

        status_prompt_clear(&mut *c1);
        while cmdq_next(
            (c1.as_mut())
                .as_deref()
                .and_then(crate::server::client_ref_of)
                .as_ref(),
        ) != 0
        {}
        let queue = (*c1).queue.take().expect("client carries its queue");
        assert!(queue.is_empty());
        drop(queue);
        window_pane_reset_mode_all(&mut *wp);
        assert!((*wp).modes().is_empty());
        clear_buffers();
    }
}

/// A filter prompt outlives the mode it was opened from. Closing the mode
/// releases the tree behind it, so the answer the prompt still carries finds
/// nothing to act on and reports the prompt finished.
#[test]
fn a_filter_prompt_outliving_buffer_mode_answers_without_the_tree() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut t = Target::new(80, 24);

    unsafe {
        clear_buffers();
        with_paste_buffers_mut(|buffers| {
            buffers.add_automatic(None, b"first buffer text\n".to_vec(), 50)
        });

        let c1 = clients.add("client-1", 80, 24);
        (*c1).set_attached_session(Some(t.session_handle()));
        (*c1).queue = Some(CmdqListRef::empty());
        status_init(&mut *c1);

        let wp = t.pane(0);
        let item = Item::with_client()
            .with_file(FILE, 1)
            .with_args(c"choose-buffer")
            .targeting(&mut t);
        let exec = crate::cmd::cmd_find(c"choose-buffer").unwrap().exec;
        assert_eq!(
            item.with_command(|command, item| exec(command, item)),
            CMD_RETURN_NORMAL
        );

        let wme = window_pane_current_mode_mut(&mut *wp).expect("pane is in a mode");
        wme.key_target()
            .unwrap()
            .dispatch(&mut *c1, b'f' as key_code, None);
        assert_eq!((*c1).prompt, Prompt::ModeTreeFilter);
        let PromptData::ModeTree(held) = &(*c1).prompt_data else {
            panic!("the prompt gave up its handle to the tree");
        };
        assert!(held.upgrade().is_some(), "the prompt reaches no live tree");

        window_pane_reset_mode_all(&mut *wp);
        assert!(
            (*wp).modes().is_empty(),
            "buffer-mode outlived its pane entry"
        );
        let PromptData::ModeTree(held) = &(*c1).prompt_data else {
            panic!("the prompt gave up its handle to the tree");
        };
        assert!(
            held.upgrade().is_none(),
            "the tree outlived the mode entry that owned it"
        );

        let answered = (*c1).prompt.input(&mut *c1, Some(c"text"), 1);
        assert_eq!(answered, 0, "the prompt asked to stay up without a tree");

        status_prompt_clear(&mut *c1);
        while cmdq_next(
            (c1.as_mut())
                .as_deref()
                .and_then(crate::server::client_ref_of)
                .as_ref(),
        ) != 0
        {}
        let queue = (*c1).queue.take().expect("client carries its queue");
        assert!(queue.is_empty());
        drop(queue);
        clear_buffers();
    }
}

#[test]
fn test_window_buffer_custom_format_and_sort() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut t = Target::new(80, 24);

    unsafe {
        clear_buffers();
        with_paste_buffers_mut(|buffers| {
            buffers.add_automatic(None, b"alpha buffer\n".to_vec(), 50);
            buffers.add_automatic(None, b"beta buffer\n".to_vec(), 50);
        });

        let c1 = clients.add("client-1", 80, 24);
        (*c1).set_attached_session(Some(t.session_handle()));

        let wp = t.pane(0);

        let item = Item::with_client()
            .with_file(FILE, 1)
            .with_args(c"choose-buffer -F \"#{buffer_name}\" -K \"#{buffer_name}\" -r -O name")
            .targeting(&mut t);

        let exec = crate::cmd::cmd_find(c"choose-buffer").unwrap().exec;
        assert_eq!(
            item.with_command(|command, item| exec(command, item)),
            CMD_RETURN_NORMAL
        );

        assert!(window_pane_current_mode(&*wp).is_some());

        window_pane_reset_mode_all(&mut *wp);
        clear_buffers();
    }
}
