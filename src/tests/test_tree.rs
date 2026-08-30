use super::*;

#[test]
fn global_tree_operations() {
    let tree: GlobalTree<i32, &'static str> = GlobalTree::default();
    let map = tree.map();
    assert!(map.is_empty());
    map.insert(1, "one");
    map.insert(2, "two");
    assert_eq!(map.len(), 2);
    assert_eq!(map.get(&1), Some(&"one"));
    assert_eq!(map.get(&2), Some(&"two"));
    assert_eq!(map.remove(&1), Some("one"));
    assert_eq!(map.len(), 1);
}

#[test]
fn global_queue_operations() {
    let queue: GlobalQueue<i32> = GlobalQueue::new();
    let q = queue.queue();
    assert!(q.is_empty());
    q.push_back(10);
    q.push_back(20);
    assert_eq!(q.len(), 2);
    assert_eq!(q.pop_front(), Some(10));
    assert_eq!(q.pop_front(), Some(20));
    assert_eq!(q.pop_front(), None);
}
