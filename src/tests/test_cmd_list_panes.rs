use super::*;
use crate::tests::test_fixtures::{
    Format, Item, Pane, Registry, Session, Window, globals, link, unlink,
};

#[test]
fn option_is_the_flag_text_or_nothing() {
    let _guard = globals();
    let mut item = Item::new().with_args(c"list-panes -F abc");
    unsafe {
        let args = cmd_get_args(&*item.cmd());
        assert_eq!(option(args, b'F'), Some(c"abc"));
        assert_eq!(option(args, b'f'), None);
    }
}

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
    let _guard = globals();
    let mut registry = Registry::new();
    let mut bee = Session::new(71, "b");
    let mut ay = Session::new(72, "a");
    assert_eq!(
        each_session().count(),
        0,
        "an empty server walks to nothing"
    );
    registry.add_session(&mut bee);
    registry.add_session(&mut ay);

    assert_eq!(
        each_session().collect::<Vec<_>>(),
        vec![ay.ptr(), bee.ptr()]
    );
}

#[test]
fn windows_of_hands_over_the_sessions_winlinks_in_index_order() {
    let _guard = globals();
    let mut s = Session::new(73, "walked");
    let mut first = Window::new(74, "first", 80, 24);
    let mut second = Window::new(75, "second", 80, 24);
    assert_eq!(windows_of(s.ptr()).count(), 0);
    let wl3 = link(&mut s, &mut second, 3);
    let wl1 = link(&mut s, &mut first, 1);

    assert_eq!(windows_of(s.ptr()).collect::<Vec<_>>(), vec![wl1, wl3]);

    unlink(&mut s, wl1);
    unlink(&mut s, wl3);
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
    registry.add_pane(&mut first);
    registry.add_pane(&mut second);

    unsafe {
        let mut crit = sort_criteria_t {
            order: SORT_INDEX,
            reversed: 0,
            order_seq: None,
        };
        assert_eq!(
            sorted_panes(w.ptr(), &mut crit),
            &[first.ptr(), second.ptr()]
        );
        crit.reversed = 1;
        assert_eq!(
            sorted_panes(w.ptr(), &mut crit),
            &[second.ptr(), first.ptr()]
        );
    }
}
