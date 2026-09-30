use crate::src::cmd::parse::cmd_parse_and_append;
use crate::src::cmd::queue::{
    cmdq_append, cmdq_get_callback_owned, cmdq_get_client, cmdq_guard, cmdq_new_state,
};
use crate::src::ffi::libc::{__errno_location, close, memset, poll, strcmp, strlen};
use crate::src::ffi::libc::{nfds_t, pollfd};
use crate::src::format::bytes::format_message_with;
use crate::src::format::bytes::write_cstr;
use crate::src::log::{fatalx, log_cstr, log_cstr_n, log_debug};
use crate::src::monitor::{monitor_add, monitor_create_client_owned, monitor_remove};
use crate::src::reactor::BufferEvent;
use crate::src::reactor::{
    bufferevent_disable, bufferevent_enable, bufferevent_get_input, bufferevent_new,
    bufferevent_setwatermark, bufferevent_write, bufferevent_write_buffer, evbuffer_add,
    evbuffer_add_formatted, evbuffer_get_length, evbuffer_new, evbuffer_pullup, evbuffer_read,
    evbuffer_readln,
};
use crate::src::server_client::Client as _;
use crate::src::server_client::Client;
use crate::src::session::Session;
use crate::src::shared::abi::ssize_t;
use crate::src::shared::abi::*;
use crate::src::shared::client::client;
use crate::src::shared::client::{ClientRef, ClientWeak};
use crate::src::shared::client::{
    CLIENT_CONTROLCONTROL, CLIENT_CONTROL_DISCARD, CLIENT_CONTROL_NOOUTPUT,
    CLIENT_CONTROL_PAUSEAFTER, CLIENT_EXIT, CLIENT_UNATTACHEDFLAGS,
};
use crate::src::shared::command::cmd_parse_input;
use crate::src::shared::command::CMDQ_STATE_CONTROL;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd_find_state, cmdq_item, cmdq_state};
use crate::src::shared::control::{
    control_block, control_pane, control_panes, control_state, control_window, control_windows,
};
use crate::src::shared::errno::{EAGAIN, EINTR};
use crate::src::shared::event::*;
use crate::src::shared::event::{EV_READ, EV_WRITE};
use crate::src::shared::key::key_event;
use crate::src::shared::limits::SIZE_MAX;
use crate::src::shared::monitor::{monitor_callback, monitor_change};
use crate::src::shared::monitor::{monitor_type, MONITOR_NOTIFY_INITIAL};
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::window_pane_offset;
use crate::src::shared::posix_io::STDIN_FILENO;
use crate::src::shared::session::session;
use crate::src::shared::window::{window, winlink};
use crate::src::tmux::{get_timer, setblocking};
use crate::src::window::{window_pane_find_by_id, winlink_find_by_window, Window, WindowPane};
use hmux_buffer::SegmentedBuf;
use std::cell::UnsafeCell;
use std::collections::VecDeque;
use std::ffi::{CStr, CString};
use std::rc::Rc;

pub const POLLIN: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const INFTIM: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const CONTROL_PANE_OFF: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CONTROL_PANE_PAUSED: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CONTROL_BUFFER_LOW: ::core::ffi::c_int = 512 as ::core::ffi::c_int;
pub const CONTROL_BUFFER_HIGH: ::core::ffi::c_int = 8192 as ::core::ffi::c_int;
pub const CONTROL_WRITE_MINIMUM: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const CONTROL_MAXIMUM_AGE: ::core::ffi::c_int = 300000 as ::core::ffi::c_int;
pub const CONTROL_MAXIMUM_REPLY_BUFFER: ::core::ffi::c_int =
    64 as ::core::ffi::c_int * 1024 as ::core::ffi::c_int * 1024 as ::core::ffi::c_int;
pub const CONTROL_IGNORE_FLAGS: ::core::ffi::c_int =
    CLIENT_CONTROL_NOOUTPUT | CLIENT_UNATTACHEDFLAGS;

impl control_block {
    fn new(line: Option<CString>, size: size_t) -> refbox::RefBox<Self> {
        refbox::RefBox::new(control_block {
            size: size,
            line: line,
            t: 0,
        })
    }
}

impl control_state {
    fn new() -> Self {
        control_state {
            deferred: VecDeque::new(),
            all_blocks: VecDeque::new(),
            pending_panes: VecDeque::new(),
            ..control_state::empty()
        }
    }

    fn add_block(&mut self, owner: refbox::RefBox<control_block>) -> refbox::Weak<control_block> {
        let block = owner.downgrade();
        self.all_blocks.push_back(owner);
        block
    }

    fn block(&self, index: usize) -> refbox::Weak<control_block> {
        self.all_blocks
            .get(index)
            .map_or_else(refbox::Weak::new, refbox::RefBox::downgrade)
    }

    fn pending_snapshot(&self) -> Vec<u_int> {
        self.pending_panes.iter().copied().collect()
    }

    fn remove_pending(&mut self, pane: u_int) -> bool {
        let Some(index) = self.pending_panes.iter().position(|queued| *queued == pane) else {
            return false;
        };
        self.pending_panes.remove(index).is_some()
    }
}

fn control_first_block(cs: &control_state) -> refbox::Weak<control_block> {
    cs.block(0)
}

fn control_first_pane_block(cp: &control_pane) -> refbox::Weak<control_block> {
    cp.blocks.front().cloned().unwrap_or_default()
}

fn control_remove_pane_block(cp: &mut control_pane, block: &refbox::Weak<control_block>) {
    let index = cp
        .blocks
        .iter()
        .position(|candidate| candidate == block)
        .expect("control pane block must be queued on its pane");
    cp.blocks.remove(index).expect("located pane block");
}

fn control_add_block(
    cs: &mut control_state,
    owner: refbox::RefBox<control_block>,
) -> refbox::Weak<control_block> {
    cs.add_block(owner)
}

#[cfg(test)]
mod control_queue_tests {
    use super::*;

    #[test]
    fn pane_output_trait_keeps_binary_escaping_and_consumes_only_the_block() {
        unsafe {
            let client = client::new();
            let pane = window_pane::new();
            (*pane.get()).id = 7;
            (*pane.get()).fd = -1;
            (*pane.get()).pipe_fd = -1;
            let stream = bufferevent_new(-1, None, None, None);
            (*pane.get()).event = crate::src::reactor::StreamHandle::from_ptr(stream);
            let input = b"a\0\\tail";
            evbuffer_add(&mut (*stream).input, input.as_ptr().cast(), input.len());
            let mut panes = Default::default();
            let consumer = control_add_pane(&mut panes, pane.id(), pane.output_offset());
            let mut message = control_append_data(&client, &mut consumer.offset, 0, None, &pane, 3);
            assert_eq!(
                evbuffer_pullup(&mut message, -1).unwrap(),
                b"%output %7 a\\000\\134"
            );
            let mut rest = [0; 4];
            assert_eq!(pane.copy_output(&consumer.offset, &mut rest), 4);
            assert_eq!(&rest, b"tail");
            std::mem::take(&mut (*pane.get()).event).free();
            drop(pane);
            drop(client);
            crate::src::reactor::shutdown_runtime();
        }
    }

    #[test]
    fn subscription_callback_releases_temporary_client_guard_on_early_exit() {
        unsafe {
            for dead in [false, true] {
                let client = client::new();
                let observed = Rc::downgrade(&client);
                if dead {
                    client.update_flags(crate::src::shared::client::CLIENT_DEAD as uint64_t, 0);
                }
                let change = monitor_change {
                    name: c"subscription",
                    value: c"value",
                    last: None,
                    c: observed.clone(),
                    s: std::rc::Weak::new(),
                    wl: refbox::Weak::new(),
                    wp: std::rc::Weak::new(),
                };
                control_sub_change(&change);
                assert_eq!(Rc::strong_count(&client), 1);
                drop(client);
                assert!(observed.upgrade().is_none());
            }
        }
    }

    #[test]
    fn state_block_owner_preserves_order_addresses_and_reply_accounting() {
        unsafe {
            let mut owner = control_state::new();
            let cs = &mut owner;

            let output = control_add_block(cs, control_block::new(None, 10));
            let reply_text = CString::new(b"reply-\xff".as_slice()).unwrap();
            let reply = control_add_block(cs, control_block::new(Some(reply_text), 0));
            cs.queued_reply_bytes = 8;
            for size in 1..=64 {
                control_add_block(cs, control_block::new(None, size));
            }

            assert_eq!(control_first_block(cs), output);
            assert_eq!(cs.block(1), reply);
            assert_eq!(
                reply
                    .try_borrow_mut()
                    .expect("live control block")
                    .line
                    .as_deref()
                    .expect("reply text")
                    .to_bytes(),
                b"reply-\xff"
            );

            control_free_block(cs, &output);
            assert_eq!(control_first_block(cs), reply);
            assert_eq!(cs.queued_reply_bytes, 8);
            control_free_block(cs, &reply);
            assert_eq!(cs.block(0), control_first_block(cs));
            assert_eq!(cs.queued_reply_bytes, 0);
        }
    }

    #[test]
    fn deferred_line_transfer_keeps_bytes_alive() {
        let mut owner = control_state::new();
        owner
            .deferred
            .push_back(CString::new(b"first-\xff".as_slice()).unwrap());
        owner.deferred.push_back(CString::new("second").unwrap());
        let line = owner.deferred.pop_front().unwrap();
        assert_eq!(line.as_bytes(), b"first-\xff");
        let line = owner.deferred.pop_front().unwrap();
        assert_eq!(line.as_bytes(), b"second");
    }

    #[test]
    fn pane_block_handles_keep_order_and_addresses_while_state_owns_blocks() {
        unsafe {
            let mut owner = control_state::new();
            let cs = &mut owner;
            let first = control_add_block(cs, control_block::new(None, 12));
            let first_weak = first.clone();
            let middle = control_add_block(cs, control_block::new(None, 23));
            let middle_weak = middle.clone();
            let last = control_add_block(cs, control_block::new(None, 34));
            let last_weak = last.clone();
            let mut pane = control_pane {
                pane: 7,
                offset: window_pane_offset { used: 0 },
                queued: window_pane_offset { used: 0 },
                flags: 0,
                pending_flag: 0,
                blocks: VecDeque::from([first_weak, middle_weak.clone(), last_weak]),
            };

            assert_eq!(control_first_pane_block(&pane), first);
            control_remove_pane_block(&mut pane, &middle);
            control_free_block(cs, &middle);
            assert!(!middle_weak.is_alive());
            assert_eq!(control_first_pane_block(&pane), first);
            assert_eq!(last.try_borrow_mut().expect("live control block").size, 34);
            assert_eq!(cs.block(0), first);
            assert_eq!(cs.block(1), last);
            control_remove_pane_block(&mut pane, &first);
            control_free_block(cs, &first);
            assert_eq!(control_first_pane_block(&pane), last);
            assert_eq!(control_first_block(cs), last);
            control_remove_pane_block(&mut pane, &last);
            control_free_block(cs, &last);
            assert!(!control_first_block(cs).is_alive());
        }
    }

    #[test]
    fn pending_snapshot_defers_reentrant_appends_to_the_next_pass() {
        let mut owner = control_state::new();
        owner.pending_panes.extend([4, 9]);
        let pass = owner.pending_snapshot();
        owner.pending_panes.push_back(12);
        assert_eq!(pass, [4, 9]);
        assert!(owner.remove_pending(4));
        assert!(!owner.remove_pending(4));
        assert_eq!(owner.pending_snapshot(), [9, 12]);
    }

    #[test]
    fn pane_index_owns_pane_boxes_and_keeps_addresses_stable() {
        let mut index = Default::default();
        let mut wp = window_pane::empty();
        wp.id = 4;
        wp.offset.used = 123;
        let first = control_add_pane(&mut index, wp.id, wp.offset);
        assert_eq!((first.offset.used, first.queued.used), (123, 123));
        first.flags = CONTROL_PANE_OFF;
        let address = first as *mut control_pane as usize;
        for id in 5..100 {
            wp.id = id;
            control_add_pane(&mut index, wp.id, wp.offset);
        }
        wp.id = 4;
        wp.offset.used = 456;
        let first = control_add_pane(&mut index, wp.id, wp.offset);
        assert_eq!(first as *mut control_pane as usize, address);
        assert_eq!(
            (first.offset.used, first.queued.used, first.flags),
            (123, 123, CONTROL_PANE_OFF)
        );
        let moved = index;
        assert_eq!(moved.get(&9).unwrap().pane, 9);
        assert!(moved.get(&100).is_none());
        drop(moved);
    }

    #[test]
    fn resetting_panes_drops_output_and_pending_ids_but_keeps_replies() {
        unsafe {
            let client = client::with_control_for_test(None, None);
            let reply = {
                let mut owner = client.borrow_control_mut().unwrap();
                let reply = control_add_block(
                    &mut owner,
                    control_block::new(Some(CString::new("reply").unwrap()), 0),
                );
                owner.queued_reply_bytes = 6;
                for id in [4, 9] {
                    let block = control_add_block(&mut owner, control_block::new(None, 10));
                    let pane = control_add_pane(
                        &mut owner.panes,
                        id,
                        window_pane_offset {
                            used: 123,
                            ..Default::default()
                        },
                    );
                    pane.blocks.push_back(block);
                    pane.pending_flag = 1;
                    owner.pending_panes.push_back(id);
                }
                assert_eq!(owner.pending_snapshot(), [4, 9]);
                reply
            };
            control_reset_offsets(&client);
            {
                let owner = client.borrow_control_mut().unwrap();
                assert!(owner.panes.is_empty());
                assert!(owner.pending_panes.is_empty());
                assert_eq!(owner.all_blocks.len(), 1);
                assert_eq!(owner.block(0), reply);
                assert_eq!(owner.queued_reply_bytes, 6);
            }
            control_reset_offsets(&client);
            {
                let mut owner = client.borrow_control_mut().unwrap();
                let pane = control_add_pane(
                    &mut owner.panes,
                    4,
                    window_pane_offset {
                        used: 456,
                        ..Default::default()
                    },
                );
                assert_eq!(pane.offset.used, 456);
                assert_eq!(pane.pending_flag, 0);
                assert!(pane.blocks.is_empty());
                assert!(owner.pending_snapshot().is_empty());
            }
            control_stop(&client);
        }
    }

    #[test]
    fn detached_stream_callbacks_do_not_retain_or_access_expired_clients() {
        unsafe {
            let owner = client::new();
            let observer = Rc::downgrade(&owner);
            let (read, write, error) = control_stream_callbacks(&owner);
            let mut stream = bufferevent::default();
            let stream = std::ptr::NonNull::from(&mut stream);
            assert_eq!(Rc::strong_count(&owner), 1);
            // Read/write tolerate stopped control state; error marks a live client.
            read.as_ref().unwrap().borrow_mut()(stream);
            write.as_ref().unwrap().borrow_mut()(stream);
            error.as_ref().unwrap().borrow_mut()(stream, 0);
            assert_ne!(owner.flags() & CLIENT_EXIT as uint64_t, 0);
            drop(owner);
            assert!(observer.upgrade().is_none());
            read.as_ref().unwrap().borrow_mut()(stream);
            write.as_ref().unwrap().borrow_mut()(stream);
            error.as_ref().unwrap().borrow_mut()(stream, 0);
        }
    }

    #[test]
    fn stop_keeps_state_owned_until_monitor_and_stream_callbacks_are_released() {
        use std::cell::RefCell;
        use std::rc::Rc;

        struct CleanupProbe {
            client: ClientWeak,
            label: &'static str,
            order: Rc<RefCell<Vec<&'static str>>>,
        }
        impl Drop for CleanupProbe {
            fn drop(&mut self) {
                let owner = self
                    .client
                    .upgrade()
                    .expect("client retained during control stop");
                unsafe {
                    assert!(owner.borrow_control_mut().is_some());
                }
                self.order.borrow_mut().push(self.label);
            }
        }

        unsafe {
            for shared_stream in [false, true] {
                let owner = client::with_control_for_test(None, None);
                if shared_stream {
                    owner.update_flags(CLIENT_CONTROLCONTROL as uint64_t, 0);
                }
                let order = Rc::new(RefCell::new(Vec::new()));
                let probe = CleanupProbe {
                    client: Rc::downgrade(&owner),
                    label: "monitor",
                    order: order.clone(),
                };
                let subs = monitor_create_client_owned(
                    Some(&owner),
                    monitor_callback(move |_| {
                        let _ = &probe;
                    }),
                );
                let probe = CleanupProbe {
                    client: Rc::downgrade(&owner),
                    label: "read",
                    order: order.clone(),
                };
                let read_event = bufferevent_new(
                    -1,
                    bufferevent_data_callback(move |_| {
                        let _ = &probe;
                    }),
                    None,
                    None,
                );
                let write_event = if shared_stream {
                    read_event
                } else {
                    let probe = CleanupProbe {
                        client: Rc::downgrade(&owner),
                        label: "write",
                        order: order.clone(),
                    };
                    bufferevent_new(
                        -1,
                        None,
                        bufferevent_data_callback(move |_| {
                            let _ = &probe;
                        }),
                        None,
                    )
                };
                let (read_observer, write_observer) = {
                    let mut cs = owner.borrow_control_mut().unwrap();
                    cs.subs = Some(subs);
                    cs.read_event = crate::src::reactor::StreamHandle::from_ptr(read_event);
                    cs.write_event = crate::src::reactor::StreamHandle::from_ptr(write_event);
                    control_windows_set(&mut cs.windows, 4, 80, 24);
                    cs.deferred.push_back(CString::new("deferred").unwrap());
                    control_add_block(
                        &mut cs,
                        control_block::new(Some(CString::new("reply").unwrap()), 0),
                    );
                    let output = control_add_block(&mut cs, control_block::new(None, 10));
                    let wp = window_pane::empty();
                    let pane = control_add_pane(&mut cs.panes, wp.id, wp.offset);
                    pane.blocks.push_back(output);
                    pane.pending_flag = 1;
                    cs.pending_panes.push_back(wp.id);
                    (cs.read_event.clone(), cs.write_event.clone())
                };

                control_stop(&owner);
                assert!(owner.borrow_control_mut().is_none());
                assert!(!read_observer.is_alive());
                assert!(!write_observer.is_alive());
                assert_eq!(
                    *order.borrow(),
                    if shared_stream {
                        vec!["monitor", "read"]
                    } else {
                        vec!["monitor", "write", "read"]
                    }
                );
                control_stop(&owner);
                control_read_callback(&owner);
                control_write_callback(&owner);
            }
        }
    }

    #[test]
    fn reply_limit_discards_only_when_starting_exit() {
        unsafe {
            for exiting in [false, true] {
                let owner = client::with_control_for_test(None, None);
                if exiting {
                    owner.update_flags(CLIENT_EXIT as u64, 0);
                }
                let block = {
                    let mut cs = owner.borrow_control_mut().unwrap();
                    cs.queued_reply_bytes = CONTROL_MAXIMUM_REPLY_BUFFER as usize - 1;
                    let block = control_add_block(&mut cs, control_block::new(None, 10));
                    control_add_pane(&mut cs.panes, 7, window_pane_offset::default())
                        .blocks
                        .push_back(block.clone());
                    block
                };
                assert!(owner.accept_control_reply(0));
                assert!(!owner.accept_control_reply(1));
                assert_ne!(owner.flags() & CLIENT_EXIT as u64, 0);
                assert_ne!(owner.flags() & CLIENT_CONTROL_DISCARD, 0);
                assert_eq!(block.is_alive(), exiting);
                assert!(!owner.accept_control_reply(0));
                control_stop(&owner);
            }
            crate::src::reactor::shutdown_runtime();
        }
    }

    #[test]
    fn formatting_can_reenter_notifications_or_stop_the_owned_state() {
        unsafe {
            let owner = client::with_control_for_test(None, None);
            owner.borrow_control_mut().unwrap().guard_depth = 1;
            owner.notify(|out| {
                owner.notify(|out| out.write_all(b"inner"));
                out.write_all(b"outer")
            });
            {
                let cs = owner.borrow_control_mut().unwrap();
                assert_eq!(
                    cs.deferred
                        .iter()
                        .map(|line| line.as_bytes())
                        .collect::<Vec<_>>(),
                    [b"inner".as_slice(), b"outer".as_slice()]
                );
            }
            control_write(&owner, |out| {
                control_stop(&owner);
                out.write_all(b"stopped while formatting reply")
            });
            assert!(owner.borrow_control_mut().is_none());
            let owner = client::with_control_for_test(None, None);
            owner.notify(|out| {
                control_stop(&owner);
                out.write_all(b"stopped while formatting notification")
            });
            assert!(owner.borrow_control_mut().is_none());
        }
    }

    #[test]
    fn window_index_owns_overrides_and_borrows_stable_entries() {
        let mut index: control_windows = Default::default();
        assert!(index.get(&4).is_none());
        index.remove(&4);
        assert!(index.is_empty());

        control_windows_set(&mut index, 4, 80, 24);
        let address = index.get(&4).unwrap().as_ref() as *const control_window as usize;
        for id in 5..100 {
            control_windows_set(&mut index, id, id, id + 1);
        }
        control_windows_set(&mut index, 4, 120, 40);
        let mut moved = index;
        let first = moved.get(&4).unwrap();
        assert_eq!(first.as_ref() as *const control_window as usize, address);
        assert_eq!((first.sx, first.sy), (120, 40));
        moved.remove(&4);
        assert!(moved.get(&4).is_none());
        assert_eq!(moved.get(&9).unwrap().sx, 9);
        // Dropping a populated index owns cleanup for all remaining overrides.
        drop(moved);

        let mut index = Default::default();
        control_windows_set(&mut index, 4, 80, 24);
        index.remove(&4);
        assert!(index.is_empty());
        control_windows_set(&mut index, 4, 90, 30);
        assert_eq!(index.get(&4).unwrap().sy, 30);
    }
}

unsafe fn control_free_block(cs: &mut control_state, cb: &refbox::Weak<control_block>) {
    control_release_block(&mut cs.all_blocks, &mut cs.queued_reply_bytes, cb);
}

unsafe fn control_release_block(
    blocks: &mut VecDeque<refbox::RefBox<control_block>>,
    queued_reply_bytes: &mut size_t,
    cb: &refbox::Weak<control_block>,
) {
    let size = {
        let block = cb.try_borrow_mut().expect("live control block");
        (block.size == 0)
            .then(|| {
                block
                    .line
                    .as_ref()
                    .map(|line| line.as_bytes_with_nul().len())
            })
            .flatten()
    };
    if let Some(size) = size {
        *queued_reply_bytes = queued_reply_bytes.saturating_sub(size);
    }
    let index = blocks
        .iter()
        .position(|owner| cb.is(owner))
        .expect("control block must be owned by its state");
    drop(blocks.remove(index).expect("located control block"));
}

fn control_add_pane(
    panes: &mut control_panes,
    id: u32,
    offset: window_pane_offset,
) -> &mut control_pane {
    panes
        .entry(id)
        .or_insert_with(|| {
            Box::new(control_pane {
                pane: id,
                offset,
                queued: offset,
                flags: 0,
                pending_flag: 0,
                blocks: VecDeque::new(),
            })
        })
        .as_mut()
}
pub unsafe fn control_set_window_size(c: &ClientRef, window: u_int, sx: u_int, sy: u_int) {
    if let Some(mut cs) = c.borrow_control_mut() {
        control_windows_set(&mut cs.windows, window, sx, sy);
    }
}

pub unsafe fn control_get_window_size(
    c: &ClientRef,
    window: u_int,
    sx: *mut u_int,
    sy: *mut u_int,
) -> ::core::ffi::c_int {
    let Some(cs) = c.borrow_control_mut() else {
        return 0;
    };
    let Some(cw) = cs.windows.get(&window) else {
        return 0;
    };
    *sx = cw.sx;
    *sy = cw.sy;
    1
}

pub unsafe fn control_clear_window_size(c: &ClientRef, window: u_int) {
    if let Some(mut cs) = c.borrow_control_mut() {
        cs.windows.remove(&window);
    }
}

unsafe fn control_discard_pane(cs: &mut control_state, pane: u_int) {
    let Some(cp) = cs.panes.get_mut(&pane) else {
        return;
    };
    while let Some(block) = cp.blocks.pop_front() {
        control_release_block(&mut cs.all_blocks, &mut cs.queued_reply_bytes, &block);
    }
}

unsafe fn control_session_has_pane(
    session: &Rc<UnsafeCell<session>>,
    pane: &Rc<UnsafeCell<window_pane>>,
) -> bool {
    let parent = pane.window_observer().upgrade().expect("live pane parent");
    let present = session.with_winlinks(|links| winlink_find_by_window(links, &parent).is_alive());
    parent.release(c"control_session_has_pane");
    present
}
unsafe fn control_window_pane(c: &ClientRef, pane: u_int) -> Option<Rc<UnsafeCell<window_pane>>> {
    let session = c.attached_session().upgrade()?;
    let pane = window_pane_find_by_id(pane)?;
    control_session_has_pane(&session, &pane).then_some(pane)
}
pub unsafe fn control_reset_offsets(c: &ClientRef) {
    let mut state = c.borrow_control_mut().expect("control client state");
    let cs = &mut *state;
    for (_, mut pane) in std::mem::take(&mut cs.panes) {
        while let Some(block) = pane.blocks.pop_front() {
            control_free_block(cs, &block);
        }
    }
    cs.pending_panes.clear();
}
pub unsafe fn control_pane_offset<'a>(
    cs: &'a mut control_state,
    client_flags: uint64_t,
    pane: u_int,
    off: &mut ::core::ffi::c_int,
) -> Option<&'a mut window_pane_offset> {
    *off = 0;
    if client_flags & CLIENT_CONTROL_NOOUTPUT as uint64_t != 0 {
        return None;
    }
    let cp = cs.panes.get_mut(&pane)?;
    if cp.flags & CONTROL_PANE_PAUSED != 0 {
        return None;
    }
    if cp.flags & CONTROL_PANE_OFF != 0 {
        *off = 1;
        return None;
    }
    *off = (cs
        .write_event
        .with_ptr(|stream| unsafe { evbuffer_get_length(&*(*stream).output) })
        .unwrap_or(0)
        >= CONTROL_BUFFER_LOW as size_t) as ::core::ffi::c_int;
    Some(&mut cp.offset)
}
pub unsafe fn control_set_pane_on(c: &ClientRef, pane: &Rc<UnsafeCell<window_pane>>) {
    let (id, offset) = (pane.id(), pane.output_offset());
    let mut state = c.borrow_control_mut().expect("control client state");
    let Some(cp) = state.panes.get_mut(&id) else {
        return;
    };
    if cp.flags & CONTROL_PANE_OFF != 0 {
        cp.flags &= !CONTROL_PANE_OFF;
        cp.offset.used = offset.used;
        cp.queued.used = offset.used;
    }
}

pub unsafe fn control_set_pane_off(c: &ClientRef, pane: &Rc<UnsafeCell<window_pane>>) {
    let (id, offset) = (pane.id(), pane.output_offset());
    let mut state = c.borrow_control_mut().expect("control client state");
    let cs = &mut *state;
    control_add_pane(&mut cs.panes, id, offset);
    control_discard_pane(cs, id);
    let cp = cs.panes.get_mut(&id).expect("indexed control pane");
    cp.offset.used = offset.used;
    cp.queued.used = offset.used;
    cp.flags |= CONTROL_PANE_OFF;
}

pub unsafe fn control_continue_pane(c: &ClientRef, pane: &Rc<UnsafeCell<window_pane>>) {
    let (id, offset) = (pane.id(), pane.output_offset());
    {
        let mut state = c.borrow_control_mut().expect("control client state");
        let Some(cp) = state.panes.get_mut(&id) else {
            return;
        };
        if cp.flags & CONTROL_PANE_PAUSED == 0 {
            return;
        }
        cp.flags &= !CONTROL_PANE_PAUSED;
        cp.offset.used = offset.used;
        cp.queued.used = offset.used;
    }
    control_notify_write(c, |out| write!(out, "%continue %{}", pane.id()));
}

pub unsafe fn control_pause_pane(c: &ClientRef, pane: &Rc<UnsafeCell<window_pane>>) {
    let (id, offset) = (pane.id(), pane.output_offset());
    {
        let mut state = c.borrow_control_mut().expect("control client state");
        let cs = &mut *state;
        let cp = control_add_pane(&mut cs.panes, id, offset);
        if cp.flags & CONTROL_PANE_PAUSED != 0 {
            return;
        }
        cp.flags |= CONTROL_PANE_PAUSED;
        control_discard_pane(cs, id);
    }
    control_notify_write(c, |out| write!(out, "%pause %{}", pane.id()));
}

pub unsafe fn control_reset_pane(c: &ClientRef, pane: &Rc<UnsafeCell<window_pane>>) {
    let (id, offset) = (pane.id(), pane.output_offset());
    let Some(mut state) = c.borrow_control_mut() else {
        return;
    };
    let cs = &mut *state;
    control_discard_pane(cs, id);
    if let Some(cp) = cs.panes.get_mut(&id) {
        cp.offset.used = offset.used;
        cp.queued.used = offset.used;
    }
}

unsafe fn control_write_line(c_owner: &ClientRef, line: CString) {
    let size = line.as_bytes_with_nul().len();
    if !c_owner.accept_control_reply(size) {
        return;
    }
    let name_owner = c_owner.name();
    let name = name_owner
        .as_ref()
        .map_or(std::ptr::null(), |name| name.as_ptr());
    let (stream, immediate) = {
        let Some(mut cs) = c_owner.borrow_control_mut() else {
            return;
        };
        let stream = cs.write_event.clone();
        if !control_first_block(&cs).is_alive() {
            log_debug(format_args!(
                "control_write_line: {}: writing line: {}",
                log_cstr(name),
                log_cstr(line.as_ptr())
            ));
            (stream, Some(line))
        } else {
            let cb = control_add_block(&mut cs, control_block::new(Some(line), 0));
            cs.queued_reply_bytes = cs.queued_reply_bytes.wrapping_add(size);
            cb.try_borrow_mut().expect("live control block").t = get_timer();
            log_debug(format_args!(
                "control_write_line: {}: storing line: {}",
                log_cstr(name),
                log_cstr(
                    cb.try_borrow_mut()
                        .expect("live control block")
                        .line
                        .as_ref()
                        .map_or(std::ptr::null(), |line| line.as_ptr())
                )
            ));
            (stream, None)
        }
    };
    let _ = stream.with_ptr(|stream| {
        if let Some(line) = immediate {
            bufferevent_write(stream, line.as_ptr().cast(), size - 1);
            bufferevent_write(stream, b"\n".as_ptr().cast(), 1);
        }
        bufferevent_enable(stream, EV_WRITE as i16)
    });
}
unsafe fn control_flush_deferred(c_owner: &ClientRef) {
    loop {
        let line = c_owner
            .borrow_control_mut()
            .and_then(|mut cs| cs.deferred.pop_front());
        let Some(line) = line else { break };
        control_write_line(c_owner, line);
    }
}
pub unsafe fn control_write(
    c_owner: &ClientRef,
    write: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
) {
    if c_owner.borrow_control_mut().is_none() {
        return;
    }
    let line = format_message_with(write);
    control_write_line(c_owner, line);
}
pub unsafe fn control_write_guard(
    c_owner: &ClientRef,
    guard: *const ::core::ffi::c_char,
    t: ::core::ffi::c_long,
    number: u_int,
    flags: ::core::ffi::c_int,
) {
    let begin = CStr::from_ptr(guard) == c"begin";
    {
        let Some(mut cs) = c_owner.borrow_control_mut() else {
            return;
        };
        if begin {
            cs.guard_depth += 1;
        }
    }
    control_write(c_owner, |out| {
        out.write_all(b"%")?;
        write_cstr(out, guard)?;
        write!(out, " {} {} {}", t, number, flags)
    });
    let flush = {
        let Some(mut cs) = c_owner.borrow_control_mut() else {
            return;
        };
        if !begin && cs.guard_depth > 0 {
            cs.guard_depth -= 1;
            cs.guard_depth == 0
        } else {
            false
        }
    };
    if flush {
        control_flush_deferred(c_owner);
    }
}
pub unsafe fn control_notify_write(
    c_owner: &ClientRef,
    write: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
) {
    if c_owner.borrow_control_mut().is_none() {
        return;
    }
    let line = format_message_with(write);
    let name = c_owner.name();
    {
        let Some(mut cs) = c_owner.borrow_control_mut() else {
            return;
        };
        if cs.guard_depth != 0 {
            log_debug(format_args!(
                "control_notify_write: {}: deferring notification: {}",
                log_cstr(name.as_ref().map_or(std::ptr::null(), |name| name.as_ptr())),
                log_cstr(line.as_ptr())
            ));
            cs.deferred.push_back(line);
            return;
        }
    }
    control_write_line(c_owner, line);
}
unsafe fn control_check_age(
    client: &ClientRef,
    window_pane: &Rc<UnsafeCell<window_pane>>,
    pane: u_int,
) -> i32 {
    let block = {
        let state = client.borrow_control_mut().expect("control client state");
        control_first_pane_block(state.panes.get(&pane).expect("indexed control pane"))
    };
    if !block.is_alive() {
        return 0;
    }
    let now = get_timer();
    let timestamp = block.try_borrow_mut().expect("live control block").t;
    if timestamp >= now {
        return 0;
    }
    let age = now.wrapping_sub(timestamp);
    let name = client.name();
    log_debug(format_args!(
        "control_check_age: {}: %{} is {} behind",
        log_cstr(name.as_ref().map_or(std::ptr::null(), |name| name.as_ptr())),
        window_pane.id(),
        age
    ));
    if let Some(limit) = client.control_pause_after() {
        if age < limit {
            return 0;
        }
        {
            let mut state = client.borrow_control_mut().expect("control client state");
            state
                .panes
                .get_mut(&pane)
                .expect("indexed control pane")
                .flags |= CONTROL_PANE_PAUSED;
            control_discard_pane(&mut state, pane);
        }
        control_notify_write(client, |out| write!(out, "%pause %{}", window_pane.id()));
    } else {
        if age < CONTROL_MAXIMUM_AGE as u64 {
            return 0;
        }
        client.exit_with_message(c"too far behind".to_owned(), None);
        control_discard(client);
    }
    1
}

pub unsafe fn control_write_output(client: &ClientRef, wp: &Rc<UnsafeCell<window_pane>>) {
    let pane = wp.id();
    let session = client
        .attached_session()
        .upgrade()
        .expect("attached control client");
    if !control_session_has_pane(&session, wp) {
        return;
    }
    drop(session);
    let ignored = if client.flags() & (CONTROL_IGNORE_FLAGS | CLIENT_EXIT) as u64 != 0 {
        let state = client.borrow_control_mut().expect("control client state");
        if !state.panes.contains_key(&pane) {
            return;
        }
        true
    } else {
        let offset = wp.output_offset();
        let mut state = client.borrow_control_mut().expect("control client state");
        let cp = control_add_pane(&mut state.panes, pane, offset);
        cp.flags & (CONTROL_PANE_OFF | CONTROL_PANE_PAUSED) != 0
    };
    if !ignored {
        if control_check_age(client, wp, pane) != 0 {
            return;
        }
        let mut queued = {
            let state = client.borrow_control_mut().expect("control client state");
            state.panes.get(&pane).expect("indexed control pane").queued
        };
        let previous = queued.used;
        wp.advance_output(&mut queued, usize::MAX);
        let size = queued.used.wrapping_sub(previous);
        let name = client.name();
        let stream = {
            let mut state = client.borrow_control_mut().expect("control client state");
            let cs = &mut *state;
            cs.panes
                .get_mut(&pane)
                .expect("indexed control pane")
                .queued = queued;
            if size == 0 {
                return;
            }
            let block = control_add_block(cs, control_block::new(None, size));
            block.try_borrow_mut().expect("live control block").t = get_timer();
            let cp = cs.panes.get_mut(&pane).expect("indexed control pane");
            cp.blocks.push_back(block);
            log_debug(format_args!(
                "control_write_output: {}: new output block of {} for %{}",
                log_cstr(name.as_ref().map_or(std::ptr::null(), |name| name.as_ptr())),
                size,
                pane
            ));
            if cp.pending_flag == 0 {
                log_debug(format_args!(
                    "control_write_output: {}: %{} now pending",
                    log_cstr(name.as_ref().map_or(std::ptr::null(), |name| name.as_ptr())),
                    pane
                ));
                cs.pending_panes.push_back(pane);
                cp.pending_flag = 1;
            }
            cs.write_event.clone()
        };
        let _ = stream.with_ptr(|stream| bufferevent_enable(stream, EV_WRITE as i16));
        return;
    }
    let name = client.name();
    log_debug(format_args!(
        "control_write_output: {}: ignoring pane %{}",
        log_cstr(name.as_ref().map_or(std::ptr::null(), |name| name.as_ptr())),
        pane
    ));
    let (mut offset, mut queued) = {
        let state = client.borrow_control_mut().expect("control client state");
        let cp = state.panes.get(&pane).expect("indexed control pane");
        (cp.offset, cp.queued)
    };
    wp.advance_output(&mut offset, usize::MAX);
    wp.advance_output(&mut queued, usize::MAX);
    let mut state = client.borrow_control_mut().expect("control client state");
    let cp = state.panes.get_mut(&pane).expect("indexed control pane");
    cp.offset = offset;
    cp.queued = queued;
}
unsafe fn control_error(
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
    error: Option<CString>,
) -> cmd_retval {
    let item = item_handle.get();
    let c_owner = cmdq_get_client((item).as_ref());
    let mut c: Option<ClientRef> = c_owner.clone();
    cmdq_guard(
        item_handle,
        b"begin\0" as *const u8 as *const ::core::ffi::c_char,
        1 as ::core::ffi::c_int,
    );
    control_write(&c.clone().expect("live client"), |out| {
        out.write_all(b"parse error: ")?;
        write_cstr(
            out,
            error
                .as_ref()
                .map_or(::core::ptr::null(), |cause| cause.as_ptr()),
        )
    });
    cmdq_guard(
        item_handle,
        b"error\0" as *const u8 as *const ::core::ffi::c_char,
        1 as ::core::ffi::c_int,
    );
    return CMD_RETURN_NORMAL;
}
unsafe fn control_error_callback(owner: &ClientRef) {
    let mut c: Option<ClientRef> = Some(owner.clone());
    c.as_ref()
        .expect("live client")
        .update_flags(CLIENT_EXIT as uint64_t, 0);
}
unsafe fn control_read_callback(owner: &ClientRef) {
    let mut c: Option<ClientRef> = Some(owner.clone());
    loop {
        let stream = {
            let Some(state) = owner.borrow_control_mut() else {
                break;
            };
            state.read_event.clone()
        };
        let Some(line) = stream
            .with_ptr(|stream| unsafe { evbuffer_readln(bufferevent_get_input(&mut *stream)) })
            .flatten()
        else {
            break;
        };
        log_debug(format_args!(
            "{}: {}: {}",
            "control_read_callback",
            log_cstr(
                ((c.as_ref().expect("live client").name())
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            ),
            log_cstr((line.as_ptr().cast::<::core::ffi::c_char>()) as *const _)
        ));
        if line[0] == 0 {
            c.as_ref()
                .expect("live client")
                .update_flags(CLIENT_EXIT as uint64_t, 0);
            break;
        } else {
            let state = cmdq_new_state(
                ::core::ptr::null_mut::<cmd_find_state>(),
                ::core::ptr::null_mut::<key_event>(),
                CMDQ_STATE_CONTROL,
            );
            match cmd_parse_and_append(
                CStr::from_ptr(line.as_ptr().cast::<::core::ffi::c_char>()),
                Some(owner),
                Some(&state),
            ) {
                Err(error) => {
                    let error_item_allocation = cmdq_get_callback_owned(
                        c"control_error",
                        Some(Box::new(move |item| unsafe { control_error(item, error) })),
                    );
                    cmdq_append(Some(owner), error_item_allocation);
                }
                Ok(_) => {}
            }
        }
    }
}
pub unsafe fn control_all_done(c: &ClientRef) -> ::core::ffi::c_int {
    let state = c.borrow_control_mut().expect("control client state");
    let cs = &*state;
    if control_first_block(cs).is_alive() {
        return 0 as ::core::ffi::c_int;
    }
    return (cs
        .write_event
        .with_ptr(|stream| unsafe { evbuffer_get_length(&*(*stream).output) })
        .unwrap_or(0)
        == 0 as size_t) as ::core::ffi::c_int;
}
pub unsafe fn control_wait_exit() {
    let mut fd: ::core::ffi::c_int = STDIN_FILENO;
    let mut pfd: pollfd = pollfd {
        fd: 0,
        events: 0,
        revents: 0,
    };
    let mut n: ::core::ffi::c_int = 0;
    let mut evb = evbuffer_new();
    loop {
        if let Some(line) = evbuffer_readln(&mut *evb) {
            if line[0] == 0 {
                break;
            }
        } else {
            memset(
                &raw mut pfd as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<pollfd>() as size_t,
            );
            pfd.fd = fd;
            pfd.events = POLLIN as ::core::ffi::c_short;
            if poll(&raw mut pfd, 1 as nfds_t, INFTIM) == -(1 as ::core::ffi::c_int) {
                if !(*__errno_location() == EINTR) {
                    break;
                }
            } else {
                n = evbuffer_read(&mut *evb, fd, -(1 as ::core::ffi::c_int));
                if n == 0 as ::core::ffi::c_int {
                    break;
                }
                if n == -(1 as ::core::ffi::c_int)
                    && *__errno_location() != EAGAIN
                    && *__errno_location() != EINTR
                {
                    break;
                }
            }
        }
    }
}
unsafe fn control_flush_all_blocks(client: &ClientRef) {
    loop {
        let (block, line, stream) = {
            let state = client.borrow_control_mut().expect("control client state");
            let block = control_first_block(&state);
            if !block.is_alive() {
                break;
            }
            let data = block.try_borrow_mut().expect("live control block");
            if data.size != 0 {
                break;
            }
            let line = data.line.clone().expect("reply block has a line");
            drop(data);
            (block, line, state.write_event.clone())
        };
        let name = client.name();
        log_debug(format_args!(
            "control_flush_all_blocks: {}: flushing line: {}",
            log_cstr(name.as_ref().map_or(std::ptr::null(), |name| name.as_ptr())),
            log_cstr(line.as_ptr())
        ));
        let Some(()) = stream.with_ptr(|stream| {
            bufferevent_write(stream, line.as_ptr().cast(), line.as_bytes().len());
            bufferevent_write(stream, c"\n".as_ptr().cast(), 1);
        }) else {
            break;
        };
        let mut state = client.borrow_control_mut().expect("control client state");
        control_free_block(&mut state, &block);
    }
}
unsafe fn control_append_data(
    c_owner: &ClientRef,
    offset: &mut window_pane_offset,
    mut age: uint64_t,
    message: Option<Box<SegmentedBuf>>,
    wp_owner: &Rc<UnsafeCell<window_pane>>,
    mut size: size_t,
) -> Box<SegmentedBuf> {
    let mut c: Option<ClientRef> = Some(c_owner.clone());

    let mut new_data: *mut u_char = ::core::ptr::null_mut::<u_char>();
    let mut new_size: size_t = 0;
    let mut start: size_t = 0;
    let mut i: u_int = 0;
    let mut message = message.unwrap_or_else(|| {
        let mut message = evbuffer_new();
        if c.as_ref().expect("live client").flags() as ::core::ffi::c_ulonglong
            & CLIENT_CONTROL_PAUSEAFTER
            != 0
        {
            evbuffer_add_formatted(&mut *message, |out| {
                write!(
                    out,
                    "%extended-output %{} {} : ",
                    (wp_owner.id()) as u32,
                    (age as ::core::ffi::c_ulonglong) as u64
                )
            });
        } else {
            evbuffer_add_formatted(&mut *message, |out| {
                write!(out, "%output %{} ", (wp_owner.id()) as u32)
            });
        }
        message
    });
    let mut data = vec![0; size];
    new_size = wp_owner.copy_output(offset, &mut data);
    new_data = data.as_ptr().cast_mut();
    if new_size < size {
        fatalx(|out| {
            write!(
                out,
                "not enough data: {} < {}",
                (new_size) as usize,
                (size) as usize
            )
        });
    }
    i = 0 as u_int;
    while (i as size_t) < size {
        if (*new_data.offset(i as isize) as ::core::ffi::c_int) < ' ' as i32
            || *new_data.offset(i as isize) as ::core::ffi::c_int == '\\' as i32
        {
            evbuffer_add_formatted(&mut *message, |out| {
                write!(
                    out,
                    "\\{:03o}",
                    (*new_data.offset(i as isize) as ::core::ffi::c_int) as u32
                )
            });
        } else {
            start = i as size_t;
            while (i.wrapping_add(1 as u_int) as size_t) < size
                && *new_data.offset(i.wrapping_add(1 as u_int) as isize) as ::core::ffi::c_int
                    >= ' ' as i32
                && *new_data.offset(i.wrapping_add(1 as u_int) as isize) as ::core::ffi::c_int
                    != '\\' as i32
            {
                i = i.wrapping_add(1);
            }
            evbuffer_add(
                &mut *message,
                new_data.offset(start as isize) as *const ::core::ffi::c_void,
                (i as size_t).wrapping_sub(start).wrapping_add(1 as size_t),
            );
        }
        i = i.wrapping_add(1);
    }
    wp_owner.advance_output(offset, size);
    return message;
}
unsafe fn control_write_data(c_owner: &ClientRef, mut message: Box<SegmentedBuf>) {
    let mut c: Option<ClientRef> = Some(c_owner.clone());

    log_debug(format_args!(
        "{}: {}: {}",
        "control_write_data",
        log_cstr(
            ((c.as_ref().expect("live client").name())
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                as *const _
        ),
        log_cstr_n(
            (evbuffer_pullup(&mut *message, -1)
                .map_or(std::ptr::null_mut(), |bytes| bytes.as_mut_ptr())) as *const _,
            evbuffer_get_length(&message) as ::core::ffi::c_int
        )
    ));
    evbuffer_add(
        &mut *message,
        b"\n\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
        1 as size_t,
    );
    let stream = {
        c_owner
            .borrow_control_mut()
            .expect("control client state")
            .write_event
            .clone()
    };
    let _ = stream.with_ptr(|stream| unsafe { bufferevent_write_buffer(stream, &mut *message) });
}
unsafe fn control_write_pending(client: &ClientRef, pane: u_int, limit: size_t) -> i32 {
    let now = get_timer();
    let owner = control_window_pane(client, pane);
    if owner.as_ref().is_none_or(|pane| !pane.has_tty()) {
        {
            let mut state = client.borrow_control_mut().expect("control client state");
            control_discard_pane(&mut state, pane);
        }
        control_flush_all_blocks(client);
        return 0;
    }
    let owner = owner.expect("live control pane");
    let mut message = None;
    let mut used = 0;
    while used != limit {
        let empty = {
            let state = client.borrow_control_mut().expect("control client state");
            state
                .panes
                .get(&pane)
                .expect("indexed control pane")
                .blocks
                .is_empty()
        };
        if empty {
            break;
        }
        if control_check_age(client, &owner, pane) != 0 {
            message = None;
            break;
        }
        let (block, mut offset) = {
            let state = client.borrow_control_mut().expect("control client state");
            let cp = state.panes.get(&pane).expect("indexed control pane");
            (control_first_pane_block(cp), cp.offset)
        };
        let (size, timestamp) = {
            let data = block.try_borrow_mut().expect("live control block");
            (data.size, data.t)
        };
        let age = if timestamp < now {
            now.wrapping_sub(timestamp)
        } else {
            0
        };
        let name = client.name();
        log_debug(format_args!(
            "control_write_pending: {}: output block {} (age {}) for %{} (used {}/{})",
            log_cstr(name.as_ref().map_or(std::ptr::null(), |name| name.as_ptr())),
            size,
            age,
            pane,
            used,
            limit
        ));
        let size = size.min(limit.wrapping_sub(used));
        used = used.wrapping_add(size);
        message = Some(control_append_data(
            client,
            &mut offset,
            age,
            message,
            &owner,
            size,
        ));
        let flush = {
            let mut state = client.borrow_control_mut().expect("control client state");
            let cs = &mut *state;
            cs.panes
                .get_mut(&pane)
                .expect("indexed control pane")
                .offset = offset;
            let left = {
                let mut data = block.try_borrow_mut().expect("live control block");
                data.size = data.size.wrapping_sub(size);
                data.size
            };
            if left == 0 {
                control_remove_pane_block(
                    cs.panes.get_mut(&pane).expect("indexed control pane"),
                    &block,
                );
                control_free_block(cs, &block);
                let first = control_first_block(cs);
                first.is_alive() && first.try_borrow_mut().expect("live control block").size == 0
            } else {
                false
            }
        };
        if flush {
            if let Some(message) = message.take() {
                control_write_data(client, message);
            }
            control_flush_all_blocks(client);
        }
    }
    if let Some(message) = message {
        control_write_data(client, message);
    }
    let state = client.borrow_control_mut().expect("control client state");
    (!state
        .panes
        .get(&pane)
        .expect("indexed control pane")
        .blocks
        .is_empty()) as i32
}

unsafe fn control_write_callback(owner: &ClientRef) {
    if owner.borrow_control_mut().is_none() {
        return;
    }
    control_flush_all_blocks(owner);
    loop {
        let (space, pending) = {
            let Some(state) = owner.borrow_control_mut() else {
                return;
            };
            let Some(buffered) = state
                .write_event
                .with_ptr(|stream| evbuffer_get_length(&*(*stream).output))
            else {
                return;
            };
            if buffered >= CONTROL_BUFFER_HIGH as usize || state.pending_panes.is_empty() {
                break;
            }
            (
                CONTROL_BUFFER_HIGH as usize - buffered,
                state.pending_snapshot(),
            )
        };
        let name = owner.name();
        log_debug(format_args!(
            "control_write_callback: {}: {} bytes available, {} panes",
            log_cstr(name.as_ref().map_or(std::ptr::null(), |name| name.as_ptr())),
            space,
            pending.len()
        ));
        let limit = (space / pending.len() / 3).max(CONTROL_WRITE_MINIMUM as usize);
        for pane in pending {
            {
                let Some(state) = owner.borrow_control_mut() else {
                    return;
                };
                let Some(buffered) = state
                    .write_event
                    .with_ptr(|stream| evbuffer_get_length(&*(*stream).output))
                else {
                    return;
                };
                if buffered >= CONTROL_BUFFER_HIGH as usize {
                    break;
                }
                if !state.pending_panes.contains(&pane) {
                    continue;
                }
            }
            if control_write_pending(owner, pane, limit) == 0 {
                let Some(mut state) = owner.borrow_control_mut() else {
                    return;
                };
                if state.remove_pending(pane) {
                    state
                        .panes
                        .get_mut(&pane)
                        .expect("indexed control pane")
                        .pending_flag = 0;
                }
            }
        }
    }
    let stream = {
        let Some(state) = owner.borrow_control_mut() else {
            return;
        };
        state.write_event.clone()
    };
    if stream.with_ptr(|stream| evbuffer_get_length(&*(*stream).output)) == Some(0) {
        let _ = stream.with_ptr(|stream| bufferevent_disable(stream, EV_WRITE as i16));
    }
}
unsafe fn control_sub_change(change: &monitor_change) {
    let Some(client_owner) = change.c.upgrade() else {
        return;
    };
    if client_owner.is_dead() {
        drop(client_owner);
        return;
    }
    let Some(session_owner) = change.s.upgrade() else {
        drop(client_owner);
        return;
    };
    let mut link = change.wl.try_borrow_mut().ok();
    if !change.wl.is_empty() && link.is_none() {
        drop(session_owner);
        drop(client_owner);
        return;
    }
    let wl = link
        .as_mut()
        .map_or(std::ptr::null_mut(), |link| &raw mut **link);
    let pane_owner = crate::src::window::window_pane_upgrade(&change.wp);
    if !std::rc::Weak::ptr_eq(&change.wp, &std::rc::Weak::new()) && pane_owner.is_none() {
        drop(session_owner);
        drop(client_owner);
        return;
    }
    let session_id = session_owner.id();
    let location = if !wl.is_null() {
        let (window_id, pane_id) = if let Some(pane) = pane_owner.as_ref() {
            let window = pane
                .window_observer()
                .upgrade()
                .expect("live subscription pane parent");
            let id = window.id();
            window.release(c"control_sub_change");
            (id, Some(pane.id()))
        } else {
            (
                (*wl)
                    .window_handle()
                    .expect("live subscription window")
                    .id(),
                None,
            )
        };
        Some((window_id, (*wl).idx as u32, pane_id))
    } else {
        None
    };
    // Output may reenter and unlink this window; only copied location data is
    // needed by the writer after this point.
    drop(link);
    control_notify_write(&client_owner, |out| {
        out.write_all(b"%subscription-changed ")?;
        out.write_all(change.name.to_bytes())?;
        match location {
            Some((window, index, Some(pane))) => {
                write!(out, " ${session_id} @{window} {index} %{pane} : ")?
            }
            Some((window, index, None)) => write!(out, " ${session_id} @{window} {index} - : ")?,
            None => write!(out, " ${session_id} - - - : ")?,
        }
        out.write_all(change.value.to_bytes())
    });
    drop(session_owner);
    drop(client_owner);
}
pub(crate) fn control_stream_callbacks(
    owner: &ClientRef,
) -> (
    bufferevent_data_cb,
    bufferevent_data_cb,
    bufferevent_event_cb,
) {
    let read = Rc::downgrade(owner);
    let write = read.clone();
    let error = read.clone();
    (
        bufferevent_data_callback(move |_| {
            if let Some(owner) = read.upgrade() {
                unsafe { control_read_callback(&owner) };
            }
        }),
        bufferevent_data_callback(move |_| {
            if let Some(owner) = write.upgrade() {
                unsafe { control_write_callback(&owner) };
            }
        }),
        bufferevent_event_callback(move |_, _| {
            if let Some(owner) = error.upgrade() {
                unsafe { control_error_callback(&owner) };
            }
        }),
    )
}

pub unsafe fn control_start(owner: &ClientRef) {
    owner.start_control();
}

pub(crate) unsafe fn control_subscriptions(
    owner: &ClientRef,
) -> crate::src::shared::monitor::MonitorRef {
    monitor_create_client_owned(
        Some(owner),
        monitor_callback(|change| unsafe { control_sub_change(change) }),
    )
}

pub unsafe fn control_ready(c: &ClientRef) {
    let stream = {
        c.borrow_control_mut()
            .expect("control client state")
            .read_event
            .clone()
    };
    let _ = stream.with_ptr(|stream| unsafe { bufferevent_enable(stream, EV_READ as i16) });
}
pub unsafe fn control_discard(c: &ClientRef) {
    let mut state = c.borrow_control_mut().expect("control client state");
    control_discard_pane_output(&mut state);
}

/// Component-only work: no Client access and no synchronous model callbacks.
pub(crate) unsafe fn control_discard_pane_output(cs: &mut control_state) {
    for cp in cs.panes.values_mut() {
        while let Some(block) = cp.blocks.pop_front() {
            control_release_block(&mut cs.all_blocks, &mut cs.queued_reply_bytes, &block);
        }
    }
    let _ = cs
        .read_event
        .with_ptr(|stream| unsafe { bufferevent_disable(stream, EV_READ as ::core::ffi::c_short) });
}

pub unsafe fn control_discard_all(c: &ClientRef) {
    control_discard(c);
    let mut state = c.borrow_control_mut().expect("control client state");
    let cs = &mut *state;
    loop {
        let cb = control_first_block(cs);
        if !cb.is_alive() {
            break;
        }
        control_free_block(cs, &cb);
    }
    cs.queued_reply_bytes = 0 as size_t;
    let _ = cs.write_event.with_ptr(|stream| unsafe {
        bufferevent_disable(stream, EV_WRITE as ::core::ffi::c_short)
    });
}
pub unsafe fn control_stop(owner: &ClientRef) {
    owner.stop_control();
}

pub(crate) unsafe fn control_clear_remaining(cs: &mut control_state) {
    cs.windows.clear();
    loop {
        let block = control_first_block(cs);
        if !block.is_alive() {
            break;
        }
        control_free_block(cs, &block);
    }
}

pub unsafe fn control_add_sub(
    c_owner: &ClientRef,
    mut name: *const ::core::ffi::c_char,
    mut type_0: monitor_type,
    mut id: ::core::ffi::c_int,
    mut format: *const ::core::ffi::c_char,
) {
    let subscriptions = {
        let state = c_owner.borrow_control_mut().expect("control client state");
        state.subs.as_ref().expect("control subscriptions").clone()
    };
    // Retain only the independently owned monitor while publishing its timer.
    // The Client component borrow has ended before monitor operations begin.
    monitor_add(
        &subscriptions,
        name,
        type_0,
        id,
        format,
        MONITOR_NOTIFY_INITIAL,
    );
}
pub unsafe fn control_remove_sub(c_owner: &ClientRef, mut name: *const ::core::ffi::c_char) {
    let subscriptions = {
        let state = c_owner.borrow_control_mut().expect("control client state");
        state.subs.as_ref().expect("control subscriptions").clone()
    };
    monitor_remove(&subscriptions, name);
}

fn control_windows_set(windows: &mut control_windows, window: u_int, sx: u_int, sy: u_int) {
    let entry = windows
        .entry(window)
        .or_insert_with(|| Box::new(control_window { window, sx, sy }));
    entry.sx = sx;
    entry.sy = sy;
}
