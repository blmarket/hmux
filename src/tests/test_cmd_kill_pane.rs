use crate::WindowPane;
use super::*;
use crate::tests::test_fixtures::{Item, Target, ensure_reactor, globals};
use crate::window::window_active_pane;
use crate::window::{window_add_pane, window_count_panes};

/// Runs the item's parsed command through the entry's exec hook, the way
/// the command queue would.
fn run(item: &mut Item) -> cmd_retval {
    unsafe {
        let e = &cmd_kill_pane_entry;
        item.with_command(|command, item| (e.exec)(command, item))
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
    let wl = t.state().winlink_ref().unwrap();
    let w = t.state().window().unwrap();
    let target = t.state().pane_list_ref().unwrap();

    let mut item = Item::new().with_args(c"kill-pane -a").targeting(&mut t);
    unsafe {
        let other = window_add_pane(&mut w.as_window_mut(), Some(&target), 100, 0).id();
        assert!(
            w.as_window()
                .panes
                .iter()
                .any(|pane| pane.pane_id() == other)
        );
        assert_eq!(window_count_panes(&w.as_window(), 1), 2);
        assert!(wl.get().unwrap().window_handle().unwrap().ptr_eq(&w));
        assert!(
            w.active_pane().is_some_and(|pane| pane.ptr_eq(&target)),
            "the fixture pane is the active one"
        );

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert_eq!(
            window_count_panes(&w.as_window(), 1),
            1,
            "the sibling was removed from the window"
        );
        assert!(
            w.as_window()
                .panes
                .first()
                .is_some_and(|owner| owner.downgrade().ptr_eq(&target))
        );
        assert!(
            w.as_window()
                .panes
                .last()
                .is_some_and(|owner| owner.downgrade().ptr_eq(&target))
        );
        assert_eq!(
            w.as_window()
                .z_index
                .iter()
                .map(|pane| pane.id())
                .collect::<Vec<_>>(),
            vec![target.id()]
        );
        assert!(
            w.active_pane().is_some_and(|pane| pane.ptr_eq(&target)),
            "the target kept the window"
        );
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
        let doomed = window_add_pane(
            &mut *w,
            (&*kept).observation().as_ref(),
            100,
            0,
        )
        .id();
        assert!((*w).panes.iter().any(|pane| pane.pane_id() == doomed));
        assert_eq!(window_count_panes(&mut *w, 1), 2);

        let mut fs = t.state();
        fs.wp = crate::window::window_pane_find_by_id(doomed);
        (*item.ptr()).target = fs;

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert_eq!(window_count_panes(&mut *w, 1), 1);
        assert!((*w).panes.first().is_some_and(|owner| {
            owner
                .get()
                .is_some_and(|pane| core::ptr::addr_eq(pane, kept))
        }));
        assert!((*w).panes.last().is_some_and(|owner| {
            owner
                .get()
                .is_some_and(|pane| core::ptr::addr_eq(pane, kept))
        }));
        assert_eq!(
            (*w).z_index
                .iter()
                .map(|pane| pane.id())
                .collect::<Vec<_>>(),
            vec![(*kept).pane_id()]
        );
        assert!(
            window_active_pane(&*w).is_some_and(|pane| pane
                .get()
                .is_some_and(|active| core::ptr::addr_eq(active, kept))),
            "the window kept its active pane"
        );
    }
}
