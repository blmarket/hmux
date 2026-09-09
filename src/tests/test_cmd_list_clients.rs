use super::*;
use crate::server::CLIENT_ATTACHED;
use crate::tests::test_fixtures::{Clients, Format, globals};

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
fn sorted_clients_is_only_the_attached_ones_in_the_sorted_order() {
    let _guard = globals();
    let mut clients = Clients::new();
    let c1 = clients.add("/dev/pts/1", 80, 24);
    let c2 = clients.add("/dev/pts/2", 80, 24);
    let _c3 = clients.add("/dev/pts/3", 80, 24);
    unsafe {
        (*c1).flags = CLIENT_ATTACHED as uint64_t;
        (*c2).flags = CLIENT_ATTACHED as uint64_t;
        let mut crit = RustSortCriteria::new(SORT_NAME, false);
        assert_eq!(
            sorted_clients(&mut crit)
                .iter()
                .map(ClientRef::as_ptr)
                .collect::<Vec<_>>(),
            [c1, c2]
        );
        crit.set_reversed(true);
        assert_eq!(
            sorted_clients(&mut crit)
                .iter()
                .map(ClientRef::as_ptr)
                .collect::<Vec<_>>(),
            [c2, c1]
        );
    }
}
