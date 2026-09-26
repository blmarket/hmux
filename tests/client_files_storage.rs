use hmux2::src::file::*;
use hmux2::src::reactor::{event_init, event_loop, shutdown_runtime};
use hmux2::src::shared::client::{client_file, client_files};
use std::cell::Cell;
use std::ptr::null_mut;
use std::rc::Rc;

#[test]
fn terminal_file_completion_is_scheduled_once() {
    unsafe {
        shutdown_runtime();
        event_init();
        let calls = Rc::new(Cell::new(0_usize));
        let callback_calls = calls.clone();
        let mut files = client_files { storage: None };
        let file = file_create_with_peer(
            null_mut(),
            &mut files,
            42,
            Some(Box::new(move |_| {
                callback_calls.set(callback_calls.get() + 1)
            })),
        );
        // Completion and teardown can both arrive before the event runs.
        file_fire_done(file);
        file_fire_done(file);
        event_loop(2);
        event_loop(2);
        assert_eq!(calls.get(), 1);
        assert!(files.storage.is_none());
        shutdown_runtime();
    }
}

#[test]
fn stream_lookup_order_reference_release_and_double_removal() {
    unsafe {
        let mut files = client_files { storage: None };
        let mut nodes = Vec::new();
        for stream in [i32::MAX, -1, 42, 0, i32::MIN] {
            nodes.push(file_create_with_peer(null_mut(), &mut files, stream, None));
        }
        let mut probe: client_file = client_file::empty();
        probe.stream = 1;
        assert_eq!((*client_files_nfind(&files, &probe)).stream, 42);
        assert!(client_files_find(&files, &probe).is_null());
        probe.stream = 42;
        assert_eq!(client_files_find(&files, &probe), nodes[2]);
        assert_eq!((*client_files_prev(&*nodes[2])).stream, 0);
        let mut node = client_files_minmax(&files, -1);
        for stream in [i32::MIN, -1, 0, 42, i32::MAX] {
            assert_eq!((*node).stream, stream);
            node = client_files_next(&*node);
        }
        assert!(node.is_null());
        (*nodes[2]).references += 1;
        file_free(nodes[2]);
        assert_eq!(client_files_find(&files, &probe), nodes[2]);
        // Read completion removes from the index before file_free does so again.
        client_files_remove(&mut files, nodes[2]);
        file_free(nodes[2]);
        for (i, node) in nodes.into_iter().enumerate() {
            if i != 2 {
                file_free(node);
            }
        }
        assert!(files.storage.is_none());
        let again = file_create_with_peer(null_mut(), &mut files, 7, None);
        file_free(again);
        assert!(files.storage.is_none());
    }
}
