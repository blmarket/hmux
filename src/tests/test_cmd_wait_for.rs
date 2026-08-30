use super::*;
use crate::cmd::CMDQ_WAITING;
use crate::tests::test_fixtures::{Args, Item, globals};
use ::std::sync::MutexGuard;

/// The channel set is a module global and the command parser reads more of
/// them, so the tests take turns and each one starts from an empty set.
fn exclusive() -> MutexGuard<'static, ()> {
    let guard = globals();
    channels().clear();
    guard
}

/// The channel names in iteration order.
fn channel_names() -> Vec<String> {
    channels()
        .keys()
        .map(|name| name.to_str().unwrap().to_owned())
        .collect()
}

fn channel(name: &CStr) -> &'static WaitChannel {
    channels().get(name).expect("channel is missing")
}

/// An item the command queue would hand to the command, with a client
/// behind it: `cmdq_get_client` reads that client and `cmdq_continue`
/// clears the waiting flag, and those are the only fields this module
/// touches.
fn waiting_item() -> Item {
    let mut item = Item::with_client();
    item.set_flags(CMDQ_WAITING);
    item
}

/// The same item with nothing behind it, which is the error path, named
/// for where the error is reported from.
fn clientless_item(file: &'static CStr) -> Item {
    let mut item = Item::new().from_file(file, 4);
    item.set_flags(CMDQ_WAITING);
    item
}

#[test]
fn adding_a_channel_puts_it_in_the_tree_under_its_own_copy_of_the_name() {
    let _guard = exclusive();
    let name = CString::new("one").unwrap();
    let wc = channel_for(&name);
    assert!(!wc.locked);
    assert!(!wc.woken);
    assert!(wc.waiters.is_empty());
    assert!(wc.lockers.is_empty());
    assert!(channels().contains_key(c"one"));
    assert!(!channels().contains_key(c"two"));
    assert_eq!(channel_names(), ["one"]);

    channel_for(&name).woken = true;
    assert!(channel(c"one").woken);
    assert_eq!(channel_names(), ["one"]);
}

#[test]
fn a_channel_is_only_removed_once_it_is_woken_free_and_unwaited() {
    let _guard = exclusive();
    let mut item = waiting_item();
    channel_for(c"chan");

    remove_if_idle(c"chan");
    assert_eq!(channel_names(), ["chan"]);

    channel_for(c"chan").locked = true;
    channel_for(c"chan").woken = true;
    remove_if_idle(c"chan");
    assert_eq!(channel_names(), ["chan"]);

    channel_for(c"chan").locked = false;
    channel_for(c"chan").waiters.push_back(item.ptr());
    remove_if_idle(c"chan");
    assert_eq!(channel_names(), ["chan"]);

    channel_for(c"chan").waiters.clear();
    remove_if_idle(c"chan");
    assert!(channel_names().is_empty());
}

#[test]
fn signalling_an_unknown_channel_creates_it_already_woken() {
    let _guard = exclusive();
    unsafe {
        let retval = cmd_wait_for_signal(::core::ptr::null_mut(), c"chan");
        assert_eq!(retval, CMD_RETURN_NORMAL);
        assert!(channel(c"chan").woken);
    }
}

#[test]
fn signalling_an_already_woken_channel_drops_it() {
    let _guard = exclusive();
    channel_for(c"chan").woken = true;
    unsafe {
        let retval = cmd_wait_for_signal(::core::ptr::null_mut(), c"chan");
        assert_eq!(retval, CMD_RETURN_NORMAL);
        assert!(channel_names().is_empty());
    }
}

#[test]
fn waiting_on_a_fresh_channel_blocks_the_item() {
    let _guard = exclusive();
    let mut first = waiting_item();
    let mut second = waiting_item();
    unsafe {
        assert_eq!(cmd_wait_for_wait(first.ptr(), c"chan"), CMD_RETURN_WAIT);
        assert_eq!(cmd_wait_for_wait(second.ptr(), c"chan"), CMD_RETURN_WAIT);
        assert_eq!(channel(c"chan").waiters, [first.ptr(), second.ptr()]);
        assert_eq!(first.flags(), CMDQ_WAITING);
    }
}

#[test]
fn waiting_on_a_woken_channel_returns_at_once_and_drops_it() {
    let _guard = exclusive();
    let mut item = waiting_item();
    channel_for(c"chan").woken = true;
    unsafe {
        assert_eq!(cmd_wait_for_wait(item.ptr(), c"chan"), CMD_RETURN_NORMAL);
        assert!(channel_names().is_empty());
    }
}

#[test]
fn signalling_continues_every_waiter_but_leaves_the_channel() {
    let _guard = exclusive();
    let mut first = waiting_item();
    let mut second = waiting_item();
    unsafe {
        cmd_wait_for_wait(first.ptr(), c"chan");
        cmd_wait_for_wait(second.ptr(), c"chan");
        assert_eq!(
            cmd_wait_for_signal(::core::ptr::null_mut(), c"chan"),
            CMD_RETURN_NORMAL
        );
        assert_eq!(first.flags() & CMDQ_WAITING, 0);
        assert_eq!(second.flags() & CMDQ_WAITING, 0);
        assert!(channel(c"chan").waiters.is_empty());
        assert!(!channel(c"chan").woken);
        assert_eq!(channel_names(), ["chan"]);
    }
}

#[test]
fn waiting_without_a_client_is_an_error() {
    let _guard = exclusive();
    let mut item = clientless_item(c"wait.conf");
    unsafe {
        assert_eq!(cmd_wait_for_wait(item.ptr(), c"chan"), CMD_RETURN_ERROR);
        assert!(channel_names().is_empty());
    }
}

#[test]
fn locking_a_free_channel_takes_the_lock() {
    let _guard = exclusive();
    let mut item = waiting_item();
    unsafe {
        assert_eq!(cmd_wait_for_lock(item.ptr(), c"chan"), CMD_RETURN_NORMAL);
        assert!(channel(c"chan").locked);
        assert!(channel(c"chan").lockers.is_empty());
    }
}

#[test]
fn locking_a_locked_channel_queues_behind_it() {
    let _guard = exclusive();
    let mut holder = waiting_item();
    let mut waiter = waiting_item();
    unsafe {
        cmd_wait_for_lock(holder.ptr(), c"chan");
        assert_eq!(cmd_wait_for_lock(waiter.ptr(), c"chan"), CMD_RETURN_WAIT);
        assert_eq!(channel(c"chan").lockers, [waiter.ptr()]);
    }
}

#[test]
fn locking_without_a_client_is_an_error() {
    let _guard = exclusive();
    let mut item = clientless_item(c"lock.conf");
    unsafe {
        assert_eq!(cmd_wait_for_lock(item.ptr(), c"chan"), CMD_RETURN_ERROR);
        assert!(channel_names().is_empty());
    }
}

#[test]
fn unlocking_a_channel_that_is_not_locked_is_an_error() {
    let _guard = exclusive();
    let mut item = clientless_item(c"unlock.conf");
    unsafe {
        assert_eq!(cmd_wait_for_unlock(item.ptr(), c"chan"), CMD_RETURN_ERROR);
        channel_for(c"chan");
        assert_eq!(cmd_wait_for_unlock(item.ptr(), c"chan"), CMD_RETURN_ERROR);
    }
}

#[test]
fn unlocking_hands_the_lock_to_the_next_locker() {
    let _guard = exclusive();
    let mut holder = waiting_item();
    let mut next = waiting_item();
    let mut last = waiting_item();
    unsafe {
        cmd_wait_for_lock(holder.ptr(), c"chan");
        cmd_wait_for_lock(next.ptr(), c"chan");
        cmd_wait_for_lock(last.ptr(), c"chan");
        assert_eq!(
            cmd_wait_for_unlock(holder.ptr(), c"chan"),
            CMD_RETURN_NORMAL
        );
        assert_eq!(next.flags() & CMDQ_WAITING, 0);
        assert_eq!(last.flags(), CMDQ_WAITING);
        assert!(channel(c"chan").locked);
        assert_eq!(channel(c"chan").lockers, [last.ptr()]);

        assert_eq!(cmd_wait_for_unlock(next.ptr(), c"chan"), CMD_RETURN_NORMAL);
        assert_eq!(last.flags() & CMDQ_WAITING, 0);
        assert!(channel(c"chan").locked);
        assert!(channel(c"chan").lockers.is_empty());
    }
}

#[test]
fn unlocking_the_last_locker_leaves_an_unwoken_channel_behind() {
    let _guard = exclusive();
    let mut holder = waiting_item();
    unsafe {
        cmd_wait_for_lock(holder.ptr(), c"chan");
        assert_eq!(
            cmd_wait_for_unlock(holder.ptr(), c"chan"),
            CMD_RETURN_NORMAL
        );
        assert!(!channel(c"chan").locked);
        assert_eq!(channel_names(), ["chan"]);

        channel_for(c"chan").woken = true;
        assert_eq!(
            cmd_wait_for_signal(::core::ptr::null_mut(), c"chan"),
            CMD_RETURN_NORMAL
        );
        assert!(channel_names().is_empty());
    }
}

#[test]
fn flushing_wakes_everyone_and_empties_the_tree() {
    let _guard = exclusive();
    let mut waiter = waiting_item();
    let mut second_waiter = waiting_item();
    let mut holder = waiting_item();
    let mut locker = waiting_item();
    let mut second_locker = waiting_item();
    unsafe {
        cmd_wait_for_wait(waiter.ptr(), c"one");
        cmd_wait_for_wait(second_waiter.ptr(), c"one");
        cmd_wait_for_lock(holder.ptr(), c"two");
        cmd_wait_for_lock(locker.ptr(), c"two");
        cmd_wait_for_lock(second_locker.ptr(), c"two");
        channel_for(c"three");
        assert_eq!(channel_names(), ["one", "three", "two"]);

        cmd_wait_for_flush();

        assert_eq!(waiter.flags() & CMDQ_WAITING, 0);
        assert_eq!(second_waiter.flags() & CMDQ_WAITING, 0);
        assert_eq!(locker.flags() & CMDQ_WAITING, 0);
        assert_eq!(second_locker.flags() & CMDQ_WAITING, 0);
        assert!(channel_names().is_empty());
    }
}

#[test]
fn the_channel_tree_stays_sorted_as_channels_come_and_go() {
    let _guard = exclusive();
    let names: Vec<CString> = (0..32)
        .map(|i| CString::new(format!("chan{:02}", i * 7 % 32)).unwrap())
        .collect();
    for name in &names {
        channel_for(name);
    }
    let mut sorted: Vec<String> = names
        .iter()
        .map(|name| name.to_str().unwrap().to_owned())
        .collect();
    sorted.sort();
    assert_eq!(channel_names(), sorted);

    for (n, name) in names.iter().enumerate() {
        channel_for(name).woken = true;
        remove_if_idle(name);
        let mut left = sorted.clone();
        left.retain(|kept| {
            !names[..=n]
                .iter()
                .any(|removed| removed.to_str().unwrap() == kept)
        });
        assert_eq!(channel_names(), left);
    }
}

#[test]
fn exec_dispatches_on_the_flag() {
    let _guard = exclusive();
    let mut item = waiting_item();
    unsafe {
        let signal = Args::parse(c"wait-for -S chan");
        assert_eq!(
            cmd_wait_for_exec(&*signal.cmd(), item.ptr()),
            CMD_RETURN_NORMAL
        );
        assert!(channel(c"chan").woken);

        let lock = Args::parse(c"wait-for -L other");
        assert_eq!(
            cmd_wait_for_exec(&*lock.cmd(), item.ptr()),
            CMD_RETURN_NORMAL
        );
        assert!(channel(c"other").locked);

        let unlock = Args::parse(c"wait-for -U other");
        assert_eq!(
            cmd_wait_for_exec(&*unlock.cmd(), item.ptr()),
            CMD_RETURN_NORMAL
        );
        assert!(!channel(c"other").locked);

        let wait = Args::parse(c"wait-for chan");
        assert_eq!(
            cmd_wait_for_exec(&*wait.cmd(), item.ptr()),
            CMD_RETURN_NORMAL
        );
        assert!(!channels().contains_key(c"chan"));
    }
}
