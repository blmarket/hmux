use super::*;
use crate::tests::test_fixtures::{Format, Pane, Registry, Session, Window, globals};

#[test]
fn passes_is_true_without_a_filter_and_follows_the_filter_with_one() {
    let _guard = globals();
    let mut ft = Format::new();
    unsafe {
        assert!(passes(ft.tree(), None));
        assert!(passes(ft.tree(), Some(c"1")));
        assert!(passes(ft.tree(), Some(c"#{==:a,a}")));
        assert!(!passes(ft.tree(), Some(c"0")));
        assert!(!passes(ft.tree(), Some(c"")));
    }
}

#[test]
fn each_level_carries_the_template_that_names_it() {
    assert!(
        Level::Window
            .template()
            .to_bytes()
            .starts_with(b"#{pane_index}: ")
    );
    assert!(
        Level::Session
            .template()
            .to_bytes()
            .starts_with(b"#{window_index}.#{pane_index}: ")
    );
    assert!(
        Level::Server
            .template()
            .to_bytes()
            .starts_with(b"#{session_name}:#{window_index}.#{pane_index}: ")
    );
    let tail = b"[history #{history_size}/#{history_limit}, #{history_bytes} bytes] #{pane_id}#{?pane_active, (active),}#{?pane_dead, (dead),}";
    for level in [Level::Window, Level::Session, Level::Server] {
        assert!(level.template().to_bytes().ends_with(tail), "{level:?}");
    }
}

#[test]
fn each_session_hands_over_every_registered_session_in_name_order() {
    let SESSIONS = SESSIONS_FIELD.get();

    let _guard = globals();
    let mut registry = Registry::new();
    let mut bee = Session::new(71, "b");
    let mut ay = Session::new(72, "a");
    assert_eq!(
        SESSIONS.read().values().count(),
        0,
        "an empty server walks to nothing"
    );
    registry.add_session(&mut bee);
    registry.add_session(&mut ay);

    assert_eq!(
        SESSIONS
            .read()
            .values()
            .map(|s| s.as_ptr())
            .collect::<Vec<_>>(),
        vec![ay.ptr(), bee.ptr()]
    );
}

#[test]
fn sorted_panes_is_the_windows_panes_in_the_order_asked_for() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut w = Window::new(76, "sorted", 80, 24);
    let mut first = Pane::new(77, 80, 24, 100);
    let mut second = Pane::new(78, 80, 24, 100);
    w.add_pane(&mut first);
    w.add_pane(&mut second);
    registry.add_window(&mut w);

    {
        let mut crit = RustSortCriteria::new(SORT_INDEX, false);
        assert_eq!(
            (w.handle())
                .sorted_panes(&crit)
                .iter()
                .map(|pane| pane.id())
                .collect::<Vec<_>>(),
            [77, 78]
        );
        crit.set_reversed(true);
        assert_eq!(
            (w.handle())
                .sorted_panes(&crit)
                .iter()
                .map(|pane| pane.id())
                .collect::<Vec<_>>(),
            [78, 77]
        );
    }
}
