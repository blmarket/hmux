use super::*;
use crate::tests::test_fixtures::{Item, Target, ensure_reactor, globals};
use crate::window::window_get_active;
use crate::window::{window_add_pane, window_count_panes, window_panes_first, window_panes_last};

/// Runs the item's parsed command through the entry's exec hook, the way
/// the command queue would.
fn run(item: &mut Item) -> cmd_retval {
    unsafe {
        let e = &raw const cmd_kill_pane_entry;
        ((*e).exec)(&*item.cmd(), item.ptr())
    }
}

/// `-a` over a window holding a sibling beside the target: the sibling is
/// taken off every client, out of the layout and out of the window, which
/// frees it, while the target itself is skipped and stays.
///
/// The sibling is a real `window_add_pane` pane rather than a fixture one,
/// because `window_remove_pane` ends in `window_pane_destroy`, which frees
/// the pane, its options and its screens outright — only a pane the pane
/// code allocated itself can be handed to it.
#[test]
fn with_a_every_other_pane_of_the_window_is_removed_and_freed() {
    let _guard = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    let wl = t.winlink(0);
    let w = t.window(0);
    let target = t.pane(0);

    let mut item = Item::new().with_args(c"kill-pane -a").targeting(&mut t);
    unsafe {
        let other = window_add_pane(w, target, 100, 0);
        assert!(!other.is_null());
        assert_eq!(window_count_panes(w, 1), 2);
        assert_eq!((*wl).window(), w);
        assert_eq!(
            window_get_active(w),
            target,
            "the fixture pane is the active one"
        );

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert_eq!(
            window_count_panes(w, 1),
            1,
            "the sibling was removed from the window"
        );
        assert_eq!(window_panes_first(w), target);
        assert_eq!(window_panes_last(w), target);
        assert_eq!((*w).z_index, vec![(*target).id]);
        assert_eq!(window_get_active(w), target, "the target kept the window");
    }
}

/// Without `-a` the target pane alone is handed to `server_kill_pane`,
/// which for a window that has another pane behind it takes the target out
/// of the layout and the window and frees it, leaving the window and its
/// session where they were.
#[test]
fn without_a_the_target_pane_alone_is_killed() {
    let _guard = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    let w = t.window(0);
    let kept = t.pane(0);

    let mut item = Item::new().with_args(c"kill-pane");
    unsafe {
        let doomed = window_add_pane(w, kept, 100, 0);
        assert!(!doomed.is_null());
        assert_eq!(window_count_panes(w, 1), 2);

        let mut fs = t.state();
        fs.set_pane(doomed);
        (*item.ptr()).target = fs;

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert_eq!(window_count_panes(w, 1), 1);
        assert_eq!(window_panes_first(w), kept);
        assert_eq!(window_panes_last(w), kept);
        assert_eq!((*w).z_index, vec![(*kept).id]);
        assert_eq!(
            window_get_active(w),
            kept,
            "the window kept its active pane"
        );
    }
}
