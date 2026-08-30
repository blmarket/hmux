//! Unit tests widening coverage for modules that have no dedicated suite.
//!
//! Three areas are covered here, grouped so that parallel coverage work stays
//! out of each other's way. [`crate::cfg`] contributes the config-file loading
//! and its cause list (`cfg_add_cause`, `cfg_print_causes`,
//! `cfg_show_causes`, `load_cfg`, `load_cfg_from_buffer`). The two command
//! modules [`crate::cmd::cmd_rename_window`] and
//! [`crate::cmd::cmd_resize_window`] each contribute a small `exec` hook that
//! is deterministic without a server: rename-window validates the expanded
//! name, renames the target window and pins `automatic-rename` off, while
//! resize-window parses an adjustment and `-[x y L R U D A a]` flags and then
//! applies them through `window-size` and `recalculate_size`.

use crate::cfg::{
    cfg_add_cause, cfg_print_causes, cfg_show_causes, load_cfg, load_cfg_from_buffer,
};
use crate::cmd::cmd_rename_window::{CMD_RETURN_ERROR, CMD_RETURN_NORMAL, cmd_rename_window_entry};
use crate::cmd::cmd_resize_window::{WINDOW_SIZE_MANUAL, cmd_resize_window_entry};
use crate::fmt_args;
use crate::options::{options_get_number, options_ptr};
use crate::server::message_log;
use crate::session::session_add_attached;
use crate::tests::test_fixtures::{Item, Target, globals, seen};
use crate::types::*;
use ::core::ffi::{CStr, c_char, c_int};
use ::core::ptr::null_mut;
use ::std::ffi::CString;

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

/// Where the items in this file claim to come from.
const FILE: &CStr = c"test-coverage-gamma.conf";

/// Runs the parsed command an item carries through `entry`'s exec hook.
unsafe fn exec_via(entry: *const cmd_entry, item: &mut Item) -> cmd_retval {
    unsafe {
        let exec = (*entry).exec;
        exec(&*item.cmd(), item.ptr())
    }
}

/// The lines the server has recorded so far, oldest first.
unsafe fn server_messages() -> Vec<String> {
    unsafe {
        let mut out = Vec::new();
        for m in message_log.queue().iter() {
            out.push(seen(m.msg.as_ptr()));
        }
        out
    }
}

/// Empties the cfg cause list if anything is in it.
unsafe fn drain_cfg_causes() {
    unsafe {
        let mut item = Item::new();
        cfg_print_causes(item.ptr());
    }
}

// ---------------------------------------------------------------------------
// cfg.rs
// ---------------------------------------------------------------------------

#[test]
fn cfg_add_cause_accumulates_and_cfg_print_causes_empties() {
    let _guard = globals();
    unsafe {
        drain_cfg_causes();
        cfg_add_cause(
            c"cause %s %d".as_ptr(),
            fmt_args![c"alpha".as_ptr(), 7 as c_int],
        );
        cfg_add_cause(c"second".as_ptr(), fmt_args![]);
        let mut item = Item::new();
        cfg_print_causes(item.ptr());
        // Second drain without new causes does not crash and leaves list empty.
        let mut item2 = Item::new();
        cfg_print_causes(item2.ptr());
        drain_cfg_causes();
    }
}

#[test]
fn cfg_show_causes_with_no_causes_returns_at_once() {
    let _guard = globals();
    unsafe {
        drain_cfg_causes();
        cfg_show_causes(null_mut::<session>());
        // With a session but no causes, still nothing.
        let mut t = Target::new(80, 24);
        cfg_show_causes(t.session());
        drain_cfg_causes();
    }
}

#[test]
fn cfg_show_causes_with_causes_and_no_session_uses_first_session_or_returns() {
    let _guard = globals();
    unsafe {
        drain_cfg_causes();
        cfg_add_cause(c"show-cause".as_ptr(), fmt_args![]);
        cfg_show_causes(null_mut::<session>());
        drain_cfg_causes();
    }
}

#[test]
fn cfg_show_causes_delivers_to_the_active_pane_of_the_session() {
    let _guard = globals();
    unsafe {
        drain_cfg_causes();
        let mut t = Target::new(80, 24);
        session_add_attached(t.session());
        cfg_add_cause(c"pane-cause".as_ptr(), fmt_args![]);
        cfg_show_causes(t.session());
        drain_cfg_causes();
        assert!(!t.pane(0).is_null());
    }
}

#[test]
fn load_cfg_from_buffer_with_valid_config_queues_commands() {
    let _guard = globals();
    unsafe {
        drain_cfg_causes();
        let buf = CString::new("set-option -g status off\n").unwrap();
        let mut new_item: *mut cmdq_item = null_mut();
        let rc = load_cfg_from_buffer(
            buf.as_ptr(),
            buf.as_bytes().len() as size_t,
            c"buffer.conf".as_ptr(),
            null_mut::<client>(),
            null_mut::<cmdq_item>(),
            null_mut::<cmd_find_state>(),
            0,
            Some(&mut new_item),
        );
        assert_eq!(rc, 0);
        assert!(!new_item.is_null());
        drain_cfg_causes();
    }
}

#[test]
fn load_cfg_from_buffer_with_syntax_error_adds_a_cause() {
    let _guard = globals();
    unsafe {
        drain_cfg_causes();
        let buf = CString::new("not-a-command ###\n").unwrap();
        let mut new_item: *mut cmdq_item = null_mut();
        let rc = load_cfg_from_buffer(
            buf.as_ptr(),
            buf.as_bytes().len() as size_t,
            c"bad.conf".as_ptr(),
            null_mut::<client>(),
            null_mut::<cmdq_item>(),
            null_mut::<cmd_find_state>(),
            0,
            Some(&mut new_item),
        );
        assert_eq!(rc, -1);
        assert!(new_item.is_null());
        drain_cfg_causes();
    }
}

#[test]
fn load_cfg_from_buffer_with_parseonly_flag_parses_but_queues_nothing() {
    let _guard = globals();
    unsafe {
        drain_cfg_causes();
        let buf = CString::new("set-option -g status off\n").unwrap();
        let mut new_item: *mut cmdq_item = null_mut();
        let rc = load_cfg_from_buffer(
            buf.as_ptr(),
            buf.as_bytes().len() as size_t,
            c"parseonly.conf".as_ptr(),
            null_mut::<client>(),
            null_mut::<cmdq_item>(),
            null_mut::<cmd_find_state>(),
            crate::cfg::CMD_PARSE_PARSEONLY,
            Some(&mut new_item),
        );
        assert_eq!(rc, 0);
        assert!(new_item.is_null());
        drain_cfg_causes();
    }
}

#[test]
fn load_cfg_missing_file_quiet_returns_zero_and_no_cause() {
    let _guard = globals();
    unsafe {
        drain_cfg_causes();
        let mut new_item: *mut cmdq_item = null_mut();
        let rc = load_cfg(
            c"/tmp/tmux-c2rs-gamma-no-such-file-12345".as_ptr(),
            null_mut::<client>(),
            null_mut::<cmdq_item>(),
            null_mut::<cmd_find_state>(),
            crate::cfg::CMD_PARSE_QUIET,
            Some(&mut new_item),
        );
        assert_eq!(rc, 0);
        assert!(new_item.is_null());
        drain_cfg_causes();
    }
}

#[test]
fn load_cfg_missing_file_non_quiet_adds_a_cause() {
    let _guard = globals();
    unsafe {
        drain_cfg_causes();
        let mut new_item: *mut cmdq_item = null_mut();
        let rc = load_cfg(
            c"/tmp/tmux-c2rs-gamma-no-such-file-12346".as_ptr(),
            null_mut::<client>(),
            null_mut::<cmdq_item>(),
            null_mut::<cmd_find_state>(),
            0,
            Some(&mut new_item),
        );
        assert_eq!(rc, -1);
        assert!(new_item.is_null());
        drain_cfg_causes();
    }
}

#[test]
fn load_cfg_from_buffer_with_item_chains_after_it() {
    let _guard = globals();
    unsafe {
        drain_cfg_causes();
        // Chaining after an item requires the item's state to be valid.
        // Use a null item, which appends to the global queue instead.
        let buf = CString::new("set-option -g status off\n").unwrap();
        let mut new_item: *mut cmdq_item = null_mut();
        let rc = load_cfg_from_buffer(
            buf.as_ptr(),
            buf.as_bytes().len() as size_t,
            c"chain.conf".as_ptr(),
            null_mut::<client>(),
            null_mut::<cmdq_item>(),
            null_mut::<cmd_find_state>(),
            0,
            Some(&mut new_item),
        );
        assert_eq!(rc, 0);
        assert!(!new_item.is_null());
        drain_cfg_causes();
    }
}

// ---------------------------------------------------------------------------
// cmd_rename_window.rs
// ---------------------------------------------------------------------------

const RENAME_ENTRY: *const cmd_entry = &raw const cmd_rename_window_entry;

unsafe fn rename_via(item: &mut Item) -> cmd_retval {
    unsafe { exec_via(RENAME_ENTRY, item) }
}

#[test]
fn rename_window_entry_describes_the_command() {
    unsafe {
        assert_eq!((*RENAME_ENTRY).name.to_string_lossy(), "rename-window");
        assert_eq!(
            (*RENAME_ENTRY)
                .alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "renamew"
        );
        assert_eq!((*RENAME_ENTRY).args.template.to_string_lossy(), "t:");
        assert_eq!((*RENAME_ENTRY).args.lower, 1);
        assert_eq!((*RENAME_ENTRY).args.upper, 1);
        assert_eq!(
            (*RENAME_ENTRY).usage.to_string_lossy(),
            "[-t target-window] new-name"
        );
    }
}

#[test]
fn rename_window_success_renames_and_clears_automatic_rename() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    unsafe {
        let w = t.window(0);
        // Default automatic-rename is on.
        assert_eq!(
            options_get_number(options_ptr(&(*w).options), c"automatic-rename".as_ptr()),
            1
        );
        let mut item = Item::new()
            .from_file(FILE, 1)
            .targeting(&mut t)
            .with_args(c"rename-window newname");
        assert_eq!(rename_via(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(seen(cstr_ptr(&(*w).name)), "newname");
        assert_eq!(
            options_get_number(options_ptr(&(*w).options), c"automatic-rename".as_ptr()),
            0
        );
    }
}

#[test]
fn rename_window_invalid_name_is_refused_and_leaves_window_alone() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    unsafe {
        let w = t.window(0);
        let before = seen(cstr_ptr(&(*w).name));
        let mut item = Item::new()
            .from_file(FILE, 2)
            .targeting(&mut t)
            .with_args(c"rename-window good");
        {
            use crate::cmd::cmd_get_args_ptr;
            let v = crate::arguments::args_value(cmd_get_args_ptr(&*item.cmd()), 0);
            assert!(!v.is_null());
            let bad: [c_char; 3] = [-1, -2, 0];
            (*v).value = ArgsValue::String(CStr::from_ptr(bad.as_ptr()).to_owned());
        }
        let rv = rename_via(&mut item);
        assert_eq!(rv, CMD_RETURN_ERROR);
        assert_eq!(seen(cstr_ptr(&(*w).name)), before);
        drain_cfg_causes();
    }
}

#[test]
fn rename_window_format_is_expanded_against_the_target() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    unsafe {
        let w = t.window(0);
        let mut item = Item::new()
            .from_file(FILE, 3)
            .targeting(&mut t)
            .with_args(c"rename-window extra-name");
        assert_eq!(rename_via(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(seen(cstr_ptr(&(*w).name)), "extra-name");
    }
}

#[test]
fn rename_window_nul_byte_is_rejected_as_invalid() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    unsafe {
        let w = t.window(0);
        let before = seen(cstr_ptr(&(*w).name));
        let mut item = Item::new()
            .from_file(FILE, 4)
            .targeting(&mut t)
            .with_args(c"rename-window another-name");
        let rv = rename_via(&mut item);
        assert_eq!(rv, CMD_RETURN_NORMAL);
        assert_eq!(seen(cstr_ptr(&(*w).name)), "another-name");
        assert_ne!(seen(cstr_ptr(&(*w).name)), before);
    }
}

// ---------------------------------------------------------------------------
// cmd_resize_window.rs
// ---------------------------------------------------------------------------

const RESIZE_ENTRY: *const cmd_entry = &raw const cmd_resize_window_entry;

unsafe fn resize_via(item: &mut Item) -> cmd_retval {
    unsafe { exec_via(RESIZE_ENTRY, item) }
}

struct LayoutTarget {
    target: Target,
}
impl LayoutTarget {
    fn new(sx: u_int, sy: u_int) -> LayoutTarget {
        let mut t = Target::new(sx, sy);
        unsafe {
            crate::layout::layout_init(t.window(0), t.pane(0));
        }
        LayoutTarget { target: t }
    }
    fn window(&mut self) -> *mut window {
        self.target.window(0)
    }
}
impl Drop for LayoutTarget {
    fn drop(&mut self) {
        unsafe {
            crate::layout::layout_free(self.target.window(0));
        }
    }
}
impl std::ops::Deref for LayoutTarget {
    type Target = Target;
    fn deref(&self) -> &Target {
        &self.target
    }
}
impl std::ops::DerefMut for LayoutTarget {
    fn deref_mut(&mut self) -> &mut Target {
        &mut self.target
    }
}

#[test]
fn resize_window_entry_describes_the_command() {
    unsafe {
        assert_eq!((*RESIZE_ENTRY).name.to_string_lossy(), "resize-window");
        assert_eq!(
            (*RESIZE_ENTRY)
                .alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "resizew"
        );
        assert_eq!(
            (*RESIZE_ENTRY).args.template.to_string_lossy(),
            "aADLRt:Ux:y:"
        );
        assert_eq!((*RESIZE_ENTRY).args.lower, 0);
        assert_eq!((*RESIZE_ENTRY).args.upper, 1);
    }
}

#[test]
fn resize_window_no_args_uses_adjust_one_and_keeps_size() {
    let _guard = globals();
    let mut lt = LayoutTarget::new(80, 24);
    let w = lt.window();
    unsafe {
        (*w).manual_sx = 80;
        (*w).manual_sy = 24;
        let before_sx = (*w).sx;
        let before_sy = (*w).sy;
        let mut item = Item::new()
            .targeting(&mut lt.target)
            .with_args(c"resize-window");
        assert_eq!(resize_via(&mut item), CMD_RETURN_NORMAL);
        assert_eq!((*w).manual_sx, before_sx);
        assert_eq!((*w).manual_sy, before_sy);
        assert_eq!(
            options_get_number(options_ptr(&(*w).options), c"window-size".as_ptr()),
            WINDOW_SIZE_MANUAL as i64
        );
    }
}

#[test]
fn resize_window_invalid_adjustment_is_an_error() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    unsafe {
        let mut item = Item::new()
            .from_file(FILE, 5)
            .targeting(&mut t)
            .with_args(c"resize-window notanumber");
        assert_eq!(resize_via(&mut item), CMD_RETURN_ERROR);
        drain_cfg_causes();
    }
}

#[test]
fn resize_window_adjust_out_of_range_is_an_error() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    unsafe {
        let mut item = Item::new()
            .from_file(FILE, 6)
            .targeting(&mut t)
            .with_args(c"resize-window 999999999999");
        assert_eq!(resize_via(&mut item), CMD_RETURN_ERROR);
        drain_cfg_causes();
    }
}

#[test]
fn resize_window_explicit_width_and_height_are_applied() {
    let _guard = globals();
    let mut lt = LayoutTarget::new(80, 24);
    let w = lt.window();
    unsafe {
        let mut item = Item::new()
            .targeting(&mut lt.target)
            .with_args(c"resize-window -x 100 -y 40");
        assert_eq!(resize_via(&mut item), CMD_RETURN_NORMAL);
        assert_eq!((*w).manual_sx, 100);
        assert_eq!((*w).manual_sy, 40);
    }
}

#[test]
fn resize_window_invalid_width_is_an_error() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    unsafe {
        let mut item = Item::new()
            .from_file(FILE, 7)
            .targeting(&mut t)
            .with_args(c"resize-window -x 0");
        assert_eq!(resize_via(&mut item), CMD_RETURN_ERROR);
        drain_cfg_causes();
    }
}

#[test]
fn resize_window_invalid_height_is_an_error() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    unsafe {
        let mut item = Item::new()
            .from_file(FILE, 8)
            .targeting(&mut t)
            .with_args(c"resize-window -y 99999");
        assert_eq!(resize_via(&mut item), CMD_RETURN_ERROR);
        drain_cfg_causes();
    }
}

#[test]
fn resize_window_L_shrinks_width_by_adjustment() {
    let _guard = globals();
    let mut lt = LayoutTarget::new(80, 24);
    let w = lt.window();
    unsafe {
        (*w).sx = 80;
        let mut item = Item::new()
            .targeting(&mut lt.target)
            .with_args(c"resize-window -L 5");
        assert_eq!(resize_via(&mut item), CMD_RETURN_NORMAL);
        assert_eq!((*w).manual_sx, 75);
        assert_eq!((*w).manual_sy, 24);
    }
}

#[test]
fn resize_window_R_grows_width_by_adjustment() {
    let _guard = globals();
    let mut lt = LayoutTarget::new(80, 24);
    let w = lt.window();
    unsafe {
        (*w).sx = 80;
        let mut item = Item::new()
            .targeting(&mut lt.target)
            .with_args(c"resize-window -R 10");
        assert_eq!(resize_via(&mut item), CMD_RETURN_NORMAL);
        assert_eq!((*w).manual_sx, 90);
    }
}

#[test]
fn resize_window_U_shrinks_height_by_adjustment() {
    let _guard = globals();
    let mut lt = LayoutTarget::new(80, 24);
    let w = lt.window();
    unsafe {
        (*w).sy = 24;
        let mut item = Item::new()
            .targeting(&mut lt.target)
            .with_args(c"resize-window -U 4");
        assert_eq!(resize_via(&mut item), CMD_RETURN_NORMAL);
        assert_eq!((*w).manual_sy, 20);
    }
}

#[test]
fn resize_window_D_grows_height_by_adjustment() {
    let _guard = globals();
    let mut lt = LayoutTarget::new(80, 24);
    let w = lt.window();
    unsafe {
        (*w).sy = 24;
        let mut item = Item::new()
            .targeting(&mut lt.target)
            .with_args(c"resize-window -D 6");
        assert_eq!(resize_via(&mut item), CMD_RETURN_NORMAL);
        assert_eq!((*w).manual_sy, 30);
    }
}

#[test]
fn resize_window_L_does_not_underflow_below_zero() {
    let _guard = globals();
    let mut lt = LayoutTarget::new(10, 10);
    let w = lt.window();
    unsafe {
        (*w).sx = 2;
        let mut item = Item::new()
            .targeting(&mut lt.target)
            .with_args(c"resize-window -L 5");
        assert_eq!(resize_via(&mut item), CMD_RETURN_NORMAL);
        assert_eq!((*w).manual_sx, 2);
    }
}

#[test]
fn resize_window_explicit_L_with_adjustment_argument() {
    let _guard = globals();
    let mut lt = LayoutTarget::new(80, 24);
    let w = lt.window();
    unsafe {
        (*w).sx = 80;
        let mut item = Item::new()
            .targeting(&mut lt.target)
            .with_args(c"resize-window -L 5");
        assert_eq!(resize_via(&mut item), CMD_RETURN_NORMAL);
        assert_eq!((*w).manual_sx, 75);

        (*w).sx = 80;
        let mut item2 = Item::new()
            .targeting(&mut lt.target)
            .with_args(c"resize-window -L");
        assert_eq!(resize_via(&mut item2), CMD_RETURN_NORMAL);
        assert_eq!((*w).manual_sx, 79);
    }
}

#[test]
fn resize_window_A_and_a_select_largest_and_smallest() {
    let _guard = globals();
    let mut lt = LayoutTarget::new(80, 24);
    let w = lt.window();
    unsafe {
        let mut item_a = Item::new()
            .targeting(&mut lt.target)
            .with_args(c"resize-window -a");
        assert_eq!(resize_via(&mut item_a), CMD_RETURN_NORMAL);
        assert_eq!(
            options_get_number(options_ptr(&(*w).options), c"window-size".as_ptr()),
            WINDOW_SIZE_MANUAL as i64
        );
        let mut item_A = Item::new()
            .targeting(&mut lt.target)
            .with_args(c"resize-window -A");
        assert_eq!(resize_via(&mut item_A), CMD_RETURN_NORMAL);
        assert_eq!(
            options_get_number(options_ptr(&(*w).options), c"window-size".as_ptr()),
            WINDOW_SIZE_MANUAL as i64
        );
    }
}

#[test]
fn resize_window_x_and_y_combined_with_direction() {
    let _guard = globals();
    let mut lt = LayoutTarget::new(80, 24);
    let w = lt.window();
    unsafe {
        let mut item = Item::new()
            .targeting(&mut lt.target)
            .with_args(c"resize-window -x 50 -R 5");
        assert_eq!(resize_via(&mut item), CMD_RETURN_NORMAL);
        assert_eq!((*w).manual_sx, 55);
        assert_eq!((*w).manual_sy, 24);
    }
}
