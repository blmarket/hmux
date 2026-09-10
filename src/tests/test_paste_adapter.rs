use super::*;
use crate::options::OptionsRef;

use crate::tests::test_fixtures::globals;

fn events(store: &mut RustPasteBufferStore) -> Vec<(bool, CString)> {
    store
        .take_events()
        .into_iter()
        .map(|event| match event {
            PasteBufferEvent::Changed(name) => (false, name),
            PasteBufferEvent::Deleted(name) => (true, name),
        })
        .collect()
}

#[test]
fn server_events_keep_tmux_notification_order() {
    let mut store = RustPasteBufferStore::server();

    store.set_named(c"one", b"first".to_vec()).unwrap();
    assert_eq!(events(&mut store), [(false, c"one".to_owned())]);

    store.set_named(c"one", b"second".to_vec()).unwrap();
    assert_eq!(
        events(&mut store),
        [(true, c"one".to_owned()), (false, c"one".to_owned())]
    );

    store.add_automatic(None, b"a".to_vec(), 1);
    drop(events(&mut store));
    store.add_automatic(None, b"b".to_vec(), 1);
    assert_eq!(
        events(&mut store),
        [
            (true, c"buffer0".to_owned()),
            (false, c"buffer1".to_owned())
        ]
    );

    store.rename(c"buffer1", c"one").unwrap();
    assert_eq!(
        events(&mut store),
        [
            (true, c"one".to_owned()),
            (true, c"buffer1".to_owned()),
            (false, c"one".to_owned())
        ]
    );
}

#[test]
fn the_server_adapter_reads_the_live_buffer_limit() {
    let _guard = globals();
    unsafe {
        let before = (global_options
            .get()
            .as_ref()
            .expect("global options are initialized"))
        .number(c"buffer-limit");
        (global_options
            .get()
            .as_ref()
            .expect("global options are initialized"))
        .set_number(c"buffer-limit", 17);
        assert_eq!(paste_buffer_limit(), 17);
        (global_options
            .get()
            .as_ref()
            .expect("global options are initialized"))
        .set_number(c"buffer-limit", before);
    }
}

#[test]
fn scoped_access_keeps_observations_inside_the_borrow() {
    let _guard = globals();
    let old_names = with_paste_buffers(|store| {
        store
            .buffers()
            .map(|buffer| buffer.name.to_owned())
            .collect::<Vec<_>>()
    });
    for name in old_names {
        with_paste_buffers_mut(|store| store.remove(name.as_c_str()));
    }

    with_paste_buffers_mut(|store| store.set_named(c"scoped", b"data".to_vec())).unwrap();
    assert_eq!(
        with_paste_buffers(|store| store.get(c"scoped").map(|buffer| buffer.data.to_vec())),
        Some(b"data".to_vec())
    );
    assert!(with_paste_buffers_mut(|store| store.remove(c"scoped")));
}

#[test]
fn server_paste_stores_are_independent_between_threads() {
    let ready = std::sync::Barrier::new(2);
    std::thread::scope(|scope| {
        for data in [b"first".as_slice(), b"second".as_slice()] {
            let ready = &ready;
            scope.spawn(move || {
                assert!(with_paste_buffers(PasteBufferStore::is_empty));
                with_paste_buffers_mut(|store| {
                    *store = RustPasteBufferStore::new();
                    store.set_named(c"shared-name", data.to_vec()).unwrap();
                });
                ready.wait();
                assert_eq!(
                    with_paste_buffers(|store| store.get(c"shared-name").unwrap().data.to_vec()),
                    data
                );
                ready.wait();
                assert!(with_paste_buffers_mut(|store| store.remove(c"shared-name")));
            });
        }
    });
}

#[test]
fn nested_paste_access_checks_borrows_and_recovers_after_unwinding() {
    std::thread::spawn(|| {
        with_paste_buffers_mut(|store| *store = RustPasteBufferStore::new());
        let result = std::panic::catch_unwind(|| {
            with_paste_buffers(|_| {
                with_paste_buffers_mut(|store| store.set_named(c"conflict", b"data".to_vec()))
            })
        });
        assert!(result.is_err());
        assert!(with_paste_buffers(PasteBufferStore::is_empty));
        with_paste_buffers_mut(|store| store.set_named(c"after", b"data".to_vec())).unwrap();
        with_paste_buffers(|_| {
            assert_eq!(with_paste_buffers(|store| store.buffers().count()), 1);
        });
    })
    .join()
    .unwrap();
}
