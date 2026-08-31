use crate::cmd::CMD_RETURN_NORMAL;
use crate::cmd::cmd_choose_tree::cmd_choose_buffer_entry;
use crate::cmd::{cmdq_free, cmdq_new, cmdq_next};
use crate::paste::{paste_add, paste_free, paste_walk};
use crate::status::{status_init, status_prompt_clear};
use crate::tests::test_fixtures::{Clients, Item, Target, globals, seen};
use crate::types::*;
use crate::window::window_pane_current_mode;
use crate::window::window_pane_reset_mode_all;
use ::core::ffi::CStr;
use ::core::ptr::null_mut;

const FILE: &CStr = c"test_coverage_window_buffer.rs";

unsafe fn clear_buffers() {
    unsafe {
        let mut pb = paste_walk(null_mut());
        while !pb.is_null() {
            let next = paste_walk(pb);
            paste_free(pb);
            pb = next;
        }
    }
}

#[test]
fn test_window_buffer_mode_lifecycle_and_keys() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut t = Target::new(80, 24);

    unsafe {
        clear_buffers();
        paste_add(null_mut(), b"first buffer text\n".to_vec());
        paste_add(null_mut(), b"second buffer line\n".to_vec());

        let c1 = clients.add("client-1", 80, 24);
        (*c1).session = t.session();
        (*c1).queue = Some(cmdq_new());
        status_init(c1);

        let wp = t.pane(0);

        let mut item = Item::with_client()
            .from_file(FILE, 1)
            .with_args(c"choose-buffer")
            .targeting(&mut t);

        let exec = cmd_choose_buffer_entry.exec;
        assert_eq!(exec(&*item.cmd(), item.ptr()), CMD_RETURN_NORMAL);

        let wme = window_pane_current_mode(wp);
        assert!(!wme.is_null());
        assert_eq!((*wme).mode(), WindowMode::Buffer);
        assert_eq!(seen((*wme).mode().name().as_ptr()), "buffer-mode");
        assert!((*wme).mode().default_format().is_some());

        // Update and resize
        (*wme).mode().update(wme);
        (*wme).mode().resize(wme, 90, 28);

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
            if !(*wp).modes.is_empty() {
                let cur_wme = window_pane_current_mode(wp);
                (*cur_wme)
                    .mode()
                    .key(cur_wme, c1, t.session(), t.winlink(0), key, null_mut());
            }
        }

        status_prompt_clear(c1);
        while cmdq_next(c1) != 0 {}
        cmdq_free((*c1).queue.take().expect("client carries its queue"));
        window_pane_reset_mode_all(wp);
        assert!((*wp).modes.is_empty());
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
        paste_add(null_mut(), b"first buffer text\n".to_vec());

        let c1 = clients.add("client-1", 80, 24);
        (*c1).session = t.session();
        (*c1).queue = Some(cmdq_new());
        status_init(c1);

        let wp = t.pane(0);
        let mut item = Item::with_client()
            .from_file(FILE, 1)
            .with_args(c"choose-buffer")
            .targeting(&mut t);
        let exec = cmd_choose_buffer_entry.exec;
        assert_eq!(exec(&*item.cmd(), item.ptr()), CMD_RETURN_NORMAL);

        let wme = window_pane_current_mode(wp);
        (*wme).mode().key(
            wme,
            c1,
            t.session(),
            t.winlink(0),
            b'f' as key_code,
            null_mut(),
        );
        assert_eq!((*c1).prompt, Prompt::ModeTreeFilter);
        let PromptData::ModeTree(held) = &(*c1).prompt_data else {
            panic!("the prompt gave up its handle to the tree");
        };
        assert!(held.upgrade().is_some(), "the prompt reaches no live tree");

        window_pane_reset_mode_all(wp);
        assert!(
            (*wp).modes.is_empty(),
            "buffer-mode outlived its pane entry"
        );
        let PromptData::ModeTree(held) = &(*c1).prompt_data else {
            panic!("the prompt gave up its handle to the tree");
        };
        assert!(
            held.upgrade().is_none(),
            "the tree outlived the mode entry that owned it"
        );

        let answered = (*c1)
            .prompt
            .input(c1, &mut (*c1).prompt_data, Some(c"text"), 1);
        assert_eq!(answered, 0, "the prompt asked to stay up without a tree");

        status_prompt_clear(c1);
        while cmdq_next(c1) != 0 {}
        cmdq_free((*c1).queue.take().expect("client carries its queue"));
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
        paste_add(null_mut(), b"alpha buffer\n".to_vec());
        paste_add(null_mut(), b"beta buffer\n".to_vec());

        let c1 = clients.add("client-1", 80, 24);
        (*c1).session = t.session();

        let wp = t.pane(0);

        let mut item = Item::with_client()
            .from_file(FILE, 1)
            .with_args(c"choose-buffer -F \"#{buffer_name}\" -K \"#{buffer_name}\" -r -O name")
            .targeting(&mut t);

        let exec = cmd_choose_buffer_entry.exec;
        assert_eq!(exec(&*item.cmd(), item.ptr()), CMD_RETURN_NORMAL);

        let wme = window_pane_current_mode(wp);
        assert!(!wme.is_null());

        window_pane_reset_mode_all(wp);
        clear_buffers();
    }
}
