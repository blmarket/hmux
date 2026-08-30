use super::*;
use crate::tests::test_fixtures::{Format, Item, Paste, globals};

#[test]
fn option_is_the_flag_text_or_nothing() {
    let _guard = globals();
    let mut item = Item::new().with_args(c"list-buffers -F abc");
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
fn sorted_buffers_is_the_whole_store_in_the_sorted_order() {
    let _guard = globals();
    let paste = Paste::new();
    let b = paste.add(c"b_buffer", "second");
    let a = paste.add(c"a_buffer", "first");
    {
        let mut crit = sort_criteria_t {
            order: SORT_NAME,
            reversed: 0,
            order_seq: None,
        };
        assert_eq!(sorted_buffers(&mut crit), &[a, b]);
        crit.reversed = 1;
        assert_eq!(sorted_buffers(&mut crit), &[b, a]);
    }
}
