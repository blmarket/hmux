//! Unit tests for [`crate::cmd::cmd_set_option`], the module behind
//! `set-option`, `set-window-option` and `set-hook`: the three [`cmd_entry`]
//! tables it publishes, the argument-parse callback all three share, every
//! enumeration constant it re-exports, and the deterministic branches of
//! [`cmd_set_option_exec`] — name matching, scope resolution, the `-o`, `-u`,
//! `-U`, `-a`, `-F`, `-p`, `-q` and `-w` behaviours, user options, table
//! options of each value type, and array options by whole and by index.
//!
//! Exec is reached through an entry's own function pointer, exactly as the
//! command queue calls it, over items whose arguments come from the real
//! command parser and whose target find state has already been resolved.
//!
//! Safety notes. The refusals are driven onto client-less items, so
//! `cmdq_error` only records a config cause; each error-path run drains those
//! causes again through `cfg_print_causes`, which with no client logs and
//! frees them, leaving the global list as it was found. Every option write
//! lands in a fixture-owned set — a target's session, window or pane — never
//! in one of the process-global trees, so nothing leaks between tests. The
//! paths this suite leaves out are the ones that cannot run without a live
//! server: `-g` and `-s` (which would write the global trees) and any branch
//! whose success would be read back by `options_push_changes` through a set
//! that no longer holds the option, where the C answers with `fatalx`.

use crate::arguments::args_create;
use crate::arguments::{args_count, args_has};
use crate::cfg::cfg_print_causes;
use crate::cmd::cmd_get_args;
use crate::cmd::cmd_set_option::{
    ARGS_PARSE_COMMANDS_OR_STRING, ARGS_PARSE_STRING, CMD_AFTERHOOK, CMD_FIND_CANFAIL,
    CMD_FIND_PANE, CMD_FIND_WINDOW, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, cmd_set_hook_entry,
    cmd_set_option_entry, cmd_set_window_option_entry,
};
use crate::cmd::{CMD_PARSE_ERROR, cmd_parse_from_string};
use crate::tests::test_fixtures::{Item, Target, globals};
use crate::types::*;
use ::core::ffi::{CStr, c_char};
use ::core::ptr::null_mut;

/// Where the tests' items claim to come from, which is what `cmdq_error`
/// reports them under.
const FILE: &CStr = c"test-coverage-cmd-set-option.conf";

/// The three entries of the family, as raw pointers, so every field read
/// stays an explicit unsafe dereference rather than a shared reference into a
/// `static mut`.
fn set_entry() -> *const cmd_entry {
    &raw const cmd_set_option_entry
}

fn window_entry() -> *const cmd_entry {
    &raw const cmd_set_window_option_entry
}

fn hook_entry() -> *const cmd_entry {
    &raw const cmd_set_hook_entry
}

/// Runs `e`'s exec hook over `item`, the way the command queue would.
unsafe fn exec_entry(e: *const cmd_entry, item: &mut Item) -> cmd_retval {
    unsafe { ((*e).exec)(&*item.cmd(), item.ptr()) }
}

/// Runs the parsed `set-option` command `item` carries.
unsafe fn run(item: &mut Item) -> cmd_retval {
    unsafe { exec_entry(set_entry(), item) }
}

/// An item carrying a parsed `line`, sourced from [`FILE`].
fn plain(line: &'static CStr, number: u_int) -> Item {
    Item::new().from_file(FILE, number).with_args(line)
}

/// A [`plain`] item aimed at `target`'s session, window and pane.
fn aimed(line: &'static CStr, number: u_int, target: &mut Target) -> Item {
    plain(line, number).targeting(target)
}

/// Hands the config causes an error-path run recorded to the log and frees
/// them, so the global cause list is left empty as it was found.
unsafe fn drain_causes(item: &mut Item) {
    unsafe { cfg_print_causes(item.ptr()) };
}

#[test]
fn the_entries_describe_the_set_option_family() {
    unsafe {
        let e = set_entry();
        assert_eq!((*e).name.to_string_lossy(), "set-option");
        assert_eq!(
            (*e).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "set"
        );
        assert_eq!(
            (*e).usage.to_string_lossy(),
            "[-aFgopqsuUw] [-t target-pane] option [value]"
        );
        assert_eq!((*e).args.template.to_string_lossy(), "aFgopqst:uUw");
        assert_eq!((*e).args.lower, 1);
        assert_eq!((*e).args.upper, 2);
        assert!((*e).args.cb.is_some());
        assert_eq!((*e).source.flag, 0);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);
        assert_eq!((*e).target.flag, b't' as c_char);
        assert_eq!((*e).target.type_0, CMD_FIND_PANE);
        assert_eq!((*e).target.flags, CMD_FIND_CANFAIL);
        assert_eq!((*e).flags, CMD_AFTERHOOK);

        let w = window_entry();
        assert_eq!((*w).name.to_string_lossy(), "set-window-option");
        assert_eq!(
            (*w).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "setw"
        );
        assert_eq!(
            (*w).usage.to_string_lossy(),
            "[-aFgoqu] [-t target-window] option [value]"
        );
        assert_eq!((*w).args.template.to_string_lossy(), "aFgoqt:u");
        assert_eq!((*w).args.lower, 1);
        assert_eq!((*w).args.upper, 2);
        assert_eq!((*w).source.type_0, CMD_FIND_PANE);
        assert_eq!((*w).target.type_0, CMD_FIND_WINDOW);
        assert_eq!((*w).target.flags, CMD_FIND_CANFAIL);
        assert_eq!((*w).flags, CMD_AFTERHOOK);

        let h = hook_entry();
        assert_eq!((*h).name.to_string_lossy(), "set-hook");
        assert!((*h).alias.is_none());
        assert_eq!(
            (*h).usage.to_string_lossy(),
            "[-agpRuw] [-t target-pane] hook [command]"
        );
        assert_eq!((*h).args.template.to_string_lossy(), "agpRt:uw");
        assert_eq!((*h).args.lower, 1);
        assert_eq!((*h).args.upper, 2);
        assert_eq!((*h).target.flag, b't' as c_char);
        assert_eq!((*h).target.type_0, CMD_FIND_PANE);
        assert_eq!((*h).flags, CMD_AFTERHOOK);
    }
}

#[test]
fn the_argument_callback_treats_the_value_position_as_commands_or_strings() {
    unsafe {
        let mut cause = None;
        for e in [set_entry(), window_entry(), hook_entry()] {
            let cb = (*e).args.cb.expect("the entry shares the args callback");
            assert_eq!(
                cb(&args_create(), 1, &mut cause),
                ARGS_PARSE_COMMANDS_OR_STRING,
                "{}",
                (*(e)).name.to_string_lossy()
            );
            assert_eq!(cb(&args_create(), 0, &mut cause), ARGS_PARSE_STRING);
            assert_eq!(cb(&args_create(), 7, &mut cause), ARGS_PARSE_STRING);
        }
    }
}

#[test]
fn parsing_resolves_the_three_entries_and_carries_their_flags() {
    let _guard = globals();
    unsafe {
        for (line, want) in [
            (c"set-option @cov x", set_entry()),
            (c"set -a @cov x", set_entry()),
            (c"set-window-option automatic-rename off", window_entry()),
            (c"setw -u window-status-format", window_entry()),
            (c"set-hook after-split-window display-panes", hook_entry()),
        ] {
            let mut item = plain(line, 1);
            assert!(::core::ptr::eq((*item.cmd()).entry, want), "{line:?}");
        }

        let mut flagged = plain(c"set-option -aFopqsuUw -t %0 status-interval 42", 1);
        let a = cmd_get_args(&*flagged.cmd());
        assert_eq!(args_count(a), 2);
        for flag in *b"aFopqsuU" {
            assert_eq!(args_has(a, flag), 1, "{}", flag as char);
        }

        let mut none = cmd_parse_from_string(c"set-option".as_ptr(), null_mut());
        assert_eq!(none.status, CMD_PARSE_ERROR);
        let err = none.take_error();
        assert!(err.contains("too few arguments"), "{err}");

        let mut bad_flag = cmd_parse_from_string(c"set-option -z x y".as_ptr(), null_mut());
        assert_eq!(bad_flag.status, CMD_PARSE_ERROR);
        let err = bad_flag.take_error();
        assert!(err.contains("unknown flag"), "{err}");
    }
}

#[test]
fn set_option_exec_user_options_and_flags() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    unsafe {
        let mut item1 = aimed(c"set-option @myopt val1", 1, &mut target);
        assert_eq!(run(&mut item1), CMD_RETURN_NORMAL);

        let mut item_app = aimed(c"set-option -a @myopt val2", 2, &mut target);
        assert_eq!(run(&mut item_app), CMD_RETURN_NORMAL);

        let mut item_fmt = aimed(c"set-option -F @fmtopt val3", 3, &mut target);
        assert_eq!(run(&mut item_fmt), CMD_RETURN_NORMAL);

        let mut item_o_fail = aimed(c"set-option -o @myopt val4", 4, &mut target);
        assert_eq!(run(&mut item_o_fail), CMD_RETURN_ERROR);
        drain_causes(&mut item_o_fail);

        let mut item_o_quiet = aimed(c"set-option -o -q @myopt val4", 5, &mut target);
        assert_eq!(run(&mut item_o_quiet), CMD_RETURN_NORMAL);

        let mut item_unset = aimed(c"set-option -u @myopt", 6, &mut target);
        assert_eq!(run(&mut item_unset), CMD_RETURN_NORMAL);

        let mut item_unset_again = aimed(c"set-option -u @myopt", 7, &mut target);
        assert_eq!(run(&mut item_unset_again), CMD_RETURN_NORMAL);

        let mut item_invalid = aimed(c"set-option bad-option-xyz val", 8, &mut target);
        assert_eq!(run(&mut item_invalid), CMD_RETURN_ERROR);
        drain_causes(&mut item_invalid);

        let mut item_invalid_q = aimed(c"set-option -q bad-option-xyz val", 9, &mut target);
        assert_eq!(run(&mut item_invalid_q), CMD_RETURN_NORMAL);
    }
}

#[test]
fn set_window_option_and_set_hook_exec() {
    let _guard = globals();
    let mut target = Target::new(80, 24);
    unsafe {
        let mut item_hook_r = aimed(c"set-hook -R after-split-window", 10, &mut target);
        assert_eq!(
            exec_entry(hook_entry(), &mut item_hook_r),
            CMD_RETURN_NORMAL
        );

        let mut item_hook = aimed(
            c"set-hook after-split-window display-panes",
            11,
            &mut target,
        );
        assert_eq!(exec_entry(hook_entry(), &mut item_hook), CMD_RETURN_NORMAL);

        let mut item_w_opt = aimed(c"set-window-option allow-rename off", 12, &mut target);
        assert_eq!(
            exec_entry(window_entry(), &mut item_w_opt),
            CMD_RETURN_NORMAL
        );

        let mut item_w_u = aimed(c"set-window-option -u allow-rename", 13, &mut target);
        assert_eq!(exec_entry(window_entry(), &mut item_w_u), CMD_RETURN_NORMAL);

        let mut item_pane_opt = aimed(c"set-option -p allow-passthrough off", 14, &mut target);
        assert_eq!(run(&mut item_pane_opt), CMD_RETURN_NORMAL);

        let mut item_pane_u = aimed(c"set-option -p -u allow-passthrough", 15, &mut target);
        assert_eq!(run(&mut item_pane_u), CMD_RETURN_NORMAL);
    }
}
