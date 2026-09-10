use super::*;

#[test]
fn global_tree_operations() {
    let tree: GlobalTree<i32, &'static str> = GlobalTree::default();
    let mut map = tree.map();
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
fn global_tree_reads_can_nest_without_copying_entries() {
    let tree = GlobalTree::new();
    tree.map().insert(1, String::from("one"));
    let first = tree.read();
    let second = tree.read();
    assert!(std::ptr::eq(&first[&1], &second[&1]));
}

#[test]
fn safe_tree_walk_saves_only_the_next_entry_before_mutation() {
    let tree = GlobalTree::new();
    let first = std::rc::Rc::new(1);
    let second = std::rc::Rc::new(3);
    let last = std::rc::Rc::new(5);
    let removed = std::rc::Rc::downgrade(&last);
    tree.map().extend([(1, first), (3, second), (5, last)]);
    let mut walk = tree.walk_safe();
    assert_eq!(*walk.next().unwrap(), 1);
    tree.map().remove(&1);
    tree.map().remove(&5);
    assert!(removed.upgrade().is_none());
    tree.map().insert(2, std::rc::Rc::new(2));
    assert_eq!(*walk.next().unwrap(), 3);
    tree.map().insert(4, std::rc::Rc::new(4));
    assert!(walk.next().is_none());
}

#[test]
#[should_panic(expected = "already borrowed")]
fn a_global_tree_rejects_overlapping_mutable_borrows() {
    let tree: GlobalTree<i32, i32> = GlobalTree::new();
    let _first = tree.map();
    let _second = tree.map();
}

#[test]
fn global_queue_operations() {
    let queue: GlobalQueue<i32> = GlobalQueue::new();
    let mut q = queue.queue();
    assert!(q.is_empty());
    q.push_back(10);
    q.push_back(20);
    assert_eq!(q.len(), 2);
    assert_eq!(q.pop_front(), Some(10));
    assert_eq!(q.pop_front(), Some(20));
    assert_eq!(q.pop_front(), None);
}

#[test]
#[should_panic(expected = "already borrowed")]
fn a_global_queue_rejects_overlapping_mutable_borrows() {
    let queue: GlobalQueue<i32> = GlobalQueue::new();
    let _first = queue.queue();
    let _second = queue.queue();
}
