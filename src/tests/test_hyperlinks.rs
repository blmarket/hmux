use super::*;

#[test]
fn eviction_is_shared_by_sets_on_the_same_thread() {
    std::thread::spawn(|| {
        let first = RustHyperlinks::new();
        let oldest = first.put(c"https://first", None);
        let second = RustHyperlinks::new();
        for _ in 1..MAX_HYPERLINKS {
            second.put(c"https://second", None);
        }
        assert!(first.get(oldest).is_none());
        assert!(second.get(1).is_some());
        first.with(|hl| assert_eq!(hl.registry.borrow().links.len(), 4999));
        second.reset();
        first.with(|hl| assert!(hl.registry.borrow().links.is_empty()));
    })
    .join()
    .unwrap();
}

#[test]
fn eviction_and_id_allocation_are_thread_local() {
    let ready = std::sync::Barrier::new(2);
    std::thread::scope(|scope| {
        scope.spawn(|| {
            let first = RustHyperlinks::new();
            let id = first.put(c"https://retained", None);
            ready.wait();
            ready.wait();
            assert_eq!(first.get(id).unwrap().2.as_c_str(), c"tmux1");
        });
        scope.spawn(|| {
            let other = RustHyperlinks::new();
            ready.wait();
            for _ in 0..MAX_HYPERLINKS {
                other.put(c"https://evicted", None);
            }
            assert!(other.get(1).is_none());
            ready.wait();
        });
    });
}

#[test]
fn last_owner_unregisters_links_without_retaining_a_cycle() {
    std::thread::spawn(|| {
        let store = RustHyperlinks::new();
        store.put(c"https://owned", None);
        let clone = store.clone();
        let weak = store.downgrade();
        let registry = store.with(|hl| Rc::clone(&hl.registry));
        drop(store);
        assert_eq!(registry.borrow().links.len(), 1);
        drop(clone);
        assert!(weak.upgrade().is_none());
        assert!(registry.borrow().links.is_empty());
    })
    .join()
    .unwrap();
}

#[test]
fn a_set_can_outlive_the_thread_local_registry_owner() {
    thread_local! {
        static LAST_SET: RefCell<Option<RustHyperlinks>> = const { RefCell::new(None) };
    }
    std::thread::spawn(|| {
        LAST_SET.with(|last| {
            let store = RustHyperlinks::new();
            store.put(c"https://thread-exit", None);
            *last.borrow_mut() = Some(store);
        });
    })
    .join()
    .unwrap();
}
