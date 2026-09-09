use super::*;
use crate::tests::test_fixtures::{Format, Paste, globals};

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
fn sorted_buffers_is_the_whole_store_in_the_sorted_order() {
    let _guard = globals();
    let paste = Paste::new();
    let b = paste.add(c"b_buffer", "second");
    let a = paste.add(c"a_buffer", "first");
    {
        let mut crit = RustSortCriteria::new(SORT_NAME, false);
        assert_eq!(
            sorted_buffers(&mut crit)
                .into_iter()
                .map(|buffer| buffer.name)
                .collect::<Vec<_>>(),
            [a.clone(), b.clone()]
        );
        crit.set_reversed(true);
        assert_eq!(
            sorted_buffers(&mut crit)
                .into_iter()
                .map(|buffer| buffer.name)
                .collect::<Vec<_>>(),
            [b, a]
        );
    }
}
