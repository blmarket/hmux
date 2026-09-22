use hmux2::src::file::*;
use std::ptr::null_mut;

#[test]
fn stream_lookup_order_reference_release_and_double_removal() {
    unsafe {
        let mut files = client_files {
            storage: null_mut(),
        };
        let mut nodes = Vec::new();
        for stream in [i32::MAX, -1, 42, 0, i32::MIN] {
            nodes.push(file_create_with_peer(
                null_mut(),
                &mut files,
                stream,
                None,
                null_mut(),
            ));
        }
        let mut probe: client_file = std::mem::zeroed();
        probe.stream = 1;
        assert_eq!((*client_files_nfind(&mut files, &mut probe)).stream, 42);
        assert!(client_files_find(&mut files, &mut probe).is_null());
        probe.stream = 42;
        assert_eq!(client_files_find(&mut files, &mut probe), nodes[2]);
        assert_eq!((*client_files_prev(nodes[2])).stream, 0);
        let mut node = client_files_minmax(&mut files, -1);
        for stream in [i32::MIN, -1, 0, 42, i32::MAX] {
            assert_eq!((*node).stream, stream);
            node = client_files_next(node);
        }
        assert!(node.is_null());
        (*nodes[2]).references += 1;
        file_free(nodes[2]);
        assert_eq!(client_files_find(&mut files, &mut probe), nodes[2]);
        // Read completion removes from the index before file_free does so again.
        client_files_remove(&mut files, nodes[2]);
        file_free(nodes[2]);
        for (i, node) in nodes.into_iter().enumerate() {
            if i != 2 {
                file_free(node);
            }
        }
        assert!(files.storage.is_null());
        let again = file_create_with_peer(null_mut(), &mut files, 7, None, null_mut());
        file_free(again);
        assert!(files.storage.is_null());
    }
}
