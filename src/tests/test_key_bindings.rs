use super::*;
use std::panic::{AssertUnwindSafe, catch_unwind};

#[test]
fn cloned_table_handles_enforce_shared_and_exclusive_access() {
    let table = KeyTableRef::new(key_table::new(c"checked".to_owned()));
    let clone = table.clone();
    {
        let read = table.borrow();
        assert_eq!(clone.borrow().name, read.name);
        assert!(catch_unwind(AssertUnwindSafe(|| clone.borrow_mut())).is_err());
    }
    {
        let mut write = table.borrow_mut();
        assert!(catch_unwind(AssertUnwindSafe(|| clone.borrow())).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| clone.borrow_mut())).is_err());
        write.activity_time.tv_sec = 123;
    }
    assert_eq!(clone.borrow().activity_time.tv_sec, 123);
}

#[test]
fn table_registry_access_can_reenter_after_a_lookup() {
    std::thread::spawn(|| {
        let table = key_bindings_get_table_ref(c"retained", 1).unwrap();
        let retained =
            key_tables.with_borrow(|tables| tables.get(c"retained".as_ref()).unwrap().clone());
        let removed = key_tables.with_borrow_mut(|tables| tables.remove(c"retained"));
        drop(removed);
        assert!(key_bindings_get_table_ref(c"retained", 0).is_none());
        let replacement = key_bindings_get_table_ref(c"retained", 1).unwrap();
        assert!(!replacement.ptr_eq(&table));
        assert!(retained.ptr_eq(&table));
        table.borrow_mut().activity_time.tv_sec = 456;
        assert_eq!(retained.borrow().activity_time.tv_sec, 456);
        assert_eq!(replacement.borrow().activity_time.tv_sec, 0);
    })
    .join()
    .unwrap();
}

#[test]
fn key_table_registries_are_independent_between_threads() {
    let ready = std::sync::Barrier::new(2);
    std::thread::scope(|scope| {
        for stamp in [123, 456] {
            let ready = &ready;
            scope.spawn(move || {
                assert!(key_tables.with_borrow(|tables| tables.is_empty()));
                let table = key_bindings_get_table_ref(c"shared-name", 1).unwrap();
                table.borrow_mut().activity_time.tv_sec = stamp;
                ready.wait();
                let found = key_bindings_get_table_ref(c"shared-name", 0).unwrap();
                assert!(found.ptr_eq(&table));
                assert_eq!(found.borrow().activity_time.tv_sec, stamp);
            });
        }
    });
}

/// The commands the key runs, as a handle that keeps them alive past the
/// binding itself.
pub(crate) fn key_binding_cmdlist_ref(bd: &key_binding) -> Option<CmdListRef> {
    bd.cmdlist.clone()
}

pub(crate) unsafe fn key_bindings_reset_table(name: &CStr) {
    let Some(table_ref) = key_bindings_get_table_ref(name, 0) else {
        return;
    };
    let (has_defaults, keys) = {
        let table = table_ref.borrow();
        (
            (&table).has_defaults(),
            table.key_bindings.keys().copied().collect::<Vec<_>>(),
        )
    };
    if !has_defaults {
        unsafe { key_bindings_remove_table(name) };
        return;
    }
    for key in keys {
        key_bindings_reset(name, key);
    }
}

pub(crate) fn key_bindings_get(table: &key_table, key: key_code) -> Option<key_binding> {
    table.binding(key).cloned()
}

pub(crate) fn key_bindings_get_default(table: &key_table, key: key_code) -> Option<key_binding> {
    table.default_binding(key).cloned()
}
