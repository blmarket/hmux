use super::*;
use crate::tests::test_fixtures::{Format, Registry, Session, globals};

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
fn sorted_sessions_is_every_registered_session_in_the_sorted_order() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut aaa = Session::new(21, "aaa");
    let mut bbb = Session::new(22, "bbb");
    registry.add_session(&mut aaa);
    registry.add_session(&mut bbb);
    {
        let mut crit = RustSortCriteria::new(SORT_NAME, false);
        assert_eq!(
            sorted_sessions(&mut crit)
                .iter()
                .map(SessionRef::as_ptr)
                .collect::<Vec<_>>(),
            [aaa.ptr(), bbb.ptr()]
        );
        crit.set_reversed(true);
        assert_eq!(
            sorted_sessions(&mut crit)
                .iter()
                .map(SessionRef::as_ptr)
                .collect::<Vec<_>>(),
            [bbb.ptr(), aaa.ptr()]
        );
    }
}
