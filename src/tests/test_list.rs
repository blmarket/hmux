use super::*;

fn nodes(n: usize) -> Vec<*mut u8> {
    (1..=n).map(|i| i as *mut u8).collect()
}

#[test]
fn foreach_walks_every_element_in_order() {
    let mut list = nodes(4);
    let walked: Vec<*mut u8> = unsafe { foreach(&raw mut list) }.collect();
    assert_eq!(walked, nodes(4));
}

#[test]
fn foreach_walks_an_empty_list_not_at_all() {
    let mut list: Vec<*mut u8> = Vec::new();
    assert_eq!(unsafe { foreach(&raw mut list) }.count(), 0);
}

#[test]
fn foreach_walks_into_what_the_body_appends() {
    let mut list = nodes(2);
    let mut walked = Vec::new();
    let mut walk = unsafe { foreach(&raw mut list) };
    for item in walk {
        walked.push(item);
        if item == 2 as *mut u8 {
            list.push(3 as *mut u8);
        }
    }
    assert_eq!(walked, nodes(3));
}

#[test]
fn foreach_walks_past_an_element_the_body_takes_out() {
    let mut list = nodes(4);
    let mut walked = Vec::new();
    let mut walk = unsafe { foreach(&raw mut list) };
    for item in walk {
        walked.push(item);
        if item == 2 as *mut u8 {
            list.retain(|&listed| listed != item);
        }
    }
    assert_eq!(walked, nodes(4));
}

#[test]
fn foreach_skips_a_later_element_the_body_takes_out() {
    let mut list = nodes(4);
    let mut walked = Vec::new();
    let mut walk = unsafe { foreach(&raw mut list) };
    for item in walk {
        walked.push(item);
        if item == std::ptr::dangling_mut::<u8>() {
            list.retain(|&listed| listed != 3 as *mut u8);
        }
    }
    assert_eq!(
        walked,
        vec![std::ptr::dangling_mut::<u8>(), 2 as *mut u8, 4 as *mut u8]
    );
}

#[test]
fn foreach_ends_where_a_reversed_list_puts_the_element_last() {
    let mut list = nodes(4);
    let mut walked = Vec::new();
    let mut walk = unsafe { foreach(&raw mut list) };
    for item in walk {
        walked.push(item);
        if item == std::ptr::dangling_mut::<u8>() {
            list.reverse();
        }
    }
    assert_eq!(walked, vec![std::ptr::dangling_mut::<u8>()]);
}

#[test]
fn foreach_safe_walks_every_element_in_order() {
    let mut list = nodes(4);
    let walked: Vec<*mut u8> = unsafe { foreach_safe(&raw mut list) }.collect();
    assert_eq!(walked, nodes(4));
}

#[test]
fn foreach_safe_walks_an_empty_list_not_at_all() {
    let mut list: Vec<*mut u8> = Vec::new();
    assert_eq!(unsafe { foreach_safe(&raw mut list) }.count(), 0);
}

#[test]
fn foreach_safe_walks_on_when_the_body_takes_the_element_out() {
    let mut list = nodes(4);
    let mut walked = Vec::new();
    let mut walk = unsafe { foreach_safe(&raw mut list) };
    for item in walk {
        walked.push(item);
        list.retain(|&listed| listed != item);
    }
    assert_eq!(walked, nodes(4));
    assert!(list.is_empty());
}

#[test]
fn foreach_safe_does_not_walk_into_what_the_body_appends() {
    let mut list = nodes(2);
    let mut walked = Vec::new();
    let mut walk = unsafe { foreach_safe(&raw mut list) };
    for item in walk {
        walked.push(item);
        if item == 2 as *mut u8 {
            list.push(3 as *mut u8);
        }
    }
    assert_eq!(walked, nodes(2));
}

#[test]
fn foreach_safe_ends_when_the_body_takes_the_successor_out() {
    let mut list = nodes(4);
    let mut walked = Vec::new();
    let mut walk = unsafe { foreach_safe(&raw mut list) };
    for item in walk {
        walked.push(item);
        if item == std::ptr::dangling_mut::<u8>() {
            list.retain(|&listed| listed != 2 as *mut u8);
        }
    }
    assert_eq!(walked, vec![std::ptr::dangling_mut::<u8>()]);
}

#[test]
fn foreach_safe_takes_out_every_other_element() {
    let mut list = nodes(6);
    let mut walked = Vec::new();
    let mut walk = unsafe { foreach_safe(&raw mut list) };
    for item in walk {
        walked.push(item);
        if item as usize % 2 == 1 {
            list.retain(|&listed| listed != item);
        }
    }
    assert_eq!(walked, nodes(6));
    assert_eq!(list, vec![2 as *mut u8, 4 as *mut u8, 6 as *mut u8]);
}

#[test]
fn foreach_queued_safe_after_starts_at_the_next_element() {
    let mut queue: VecDeque<*mut u8> = nodes(4).into();
    let walked: Vec<*mut u8> =
        unsafe { foreach_queued_safe_after(&raw mut queue, 2 as *mut u8) }.collect();
    assert_eq!(walked, vec![3 as *mut u8, 4 as *mut u8]);
}

#[test]
fn foreach_queued_safe_after_the_last_element_walks_nothing() {
    let mut queue: VecDeque<*mut u8> = nodes(4).into();
    assert_eq!(
        unsafe { foreach_queued_safe_after(&raw mut queue, 4 as *mut u8) }.count(),
        0
    );
}

#[test]
fn foreach_queued_safe_after_an_element_not_listed_walks_nothing() {
    let mut queue: VecDeque<*mut u8> = nodes(4).into();
    assert_eq!(
        unsafe { foreach_queued_safe_after(&raw mut queue, 9 as *mut u8) }.count(),
        0
    );
}

#[test]
fn foreach_queued_safe_after_takes_from_the_front_as_it_goes() {
    let mut queue: VecDeque<*mut u8> = nodes(5).into();
    let mut walked = Vec::new();
    let mut walk =
        unsafe { foreach_queued_safe_after(&raw mut queue, std::ptr::dangling_mut::<u8>()) };
    for item in walk {
        walked.push(item);
        queue.retain(|&listed| listed != item);
    }
    assert_eq!(
        walked,
        vec![2 as *mut u8, 3 as *mut u8, 4 as *mut u8, 5 as *mut u8]
    );
    assert_eq!(queue, vec![std::ptr::dangling_mut::<u8>()]);
}
