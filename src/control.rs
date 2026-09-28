use crate::src::cmd::parse::cmd_parse_and_append;
use crate::src::cmd::queue::{
    cmdq_append, cmdq_get_callback_owned, cmdq_get_client, cmdq_guard,
    cmdq_new_state,
};
use crate::src::ffi::libc::{__errno_location, close, memset, poll, strcmp, strlen};
use crate::src::ffi::libc::{nfds_t, pollfd};
use crate::src::format::bytes::format_message_with;
use crate::src::format::bytes::write_cstr;
use crate::src::log::{fatalx, log_cstr, log_cstr_n, log_debug};
use crate::src::monitor::{monitor_add, monitor_create_client_owned, monitor_remove};
use crate::src::reactor::{
    bufferevent_disable, bufferevent_enable, bufferevent_new,
    bufferevent_setwatermark, bufferevent_write, bufferevent_write_buffer, evbuffer_add,
    evbuffer_add_formatted, evbuffer_get_length, evbuffer_new, evbuffer_pullup,
    evbuffer_read, evbuffer_readln,
};
use crate::src::server_client::server_client_set_exit_message;
use crate::src::shared::abi::ssize_t;
use crate::src::shared::abi::*;
use crate::src::shared::client::client;
use crate::src::shared::client::{
    CLIENT_CONTROLCONTROL, CLIENT_CONTROL_DISCARD, CLIENT_CONTROL_NOOUTPUT,
    CLIENT_CONTROL_PAUSEAFTER, CLIENT_EXIT, CLIENT_UNATTACHEDFLAGS,
};
use crate::src::shared::command::cmd_parse_input;
use crate::src::shared::command::CMDQ_STATE_CONTROL;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd_find_state, cmdq_item, cmdq_state};
use crate::src::shared::control::{
    control_block, control_pane, control_panes, control_state, control_window,
    control_windows,
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
use crate::src::window::{
    window_pane_find_by_id, window_pane_get_new_data, window_pane_update_used_data,
    winlink_find_by_window,
};
use std::collections::VecDeque;
use std::cell::UnsafeCell;
use std::rc::Rc;
use std::ffi::{CStr, CString};

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

    fn add_block(&mut self, owner: refbox::RefBox<control_block>) -> *mut control_block {
        let block = owner.as_ptr().cast_mut();
        self.all_blocks.push_back(owner);
        block
    }

    fn block(&self, index: usize) -> *mut control_block {
        self.all_blocks
            .get(index)
            .map_or(std::ptr::null_mut(), |owner| owner.as_ptr().cast_mut())
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

fn control_first_block(cs: &control_state) -> *mut control_block {
    cs.block(0)
}

fn control_first_pane_block(cp: &control_pane) -> *mut control_block {
    cp.blocks.front().map_or(std::ptr::null_mut(), control_block_ptr)
}

fn control_block_ptr(block: &refbox::Weak<control_block>) -> *mut control_block {
    assert!(
        block.is_alive(),
        "pane block must be owned by its control state"
    );
    block.as_ptr().cast_mut()
}

fn control_remove_pane_block(cp: &mut control_pane, block: *mut control_block) {
    let index = cp
        .blocks
        .iter()
        .position(|candidate| std::ptr::eq(candidate.as_ptr(), block))
        .expect("control pane block must be queued on its pane");
    cp.blocks.remove(index).expect("located pane block");
}

fn control_add_block(
    cs: &mut control_state,
    owner: refbox::RefBox<control_block>,
) -> *mut control_block {
    cs.add_block(owner)
}

fn control_add_block_with_weak(
    cs: &mut control_state,
    owner: refbox::RefBox<control_block>,
) -> (*mut control_block, refbox::Weak<control_block>) {
    let observer = owner.downgrade();
    (control_add_block(cs, owner), observer)
}

#[cfg(test)]
mod control_queue_tests {
    use super::*;

    #[test]
    fn subscription_callback_releases_temporary_client_guard_on_early_exit() {
        unsafe {
            for dead in [false, true] {
                let client = client::new();
                let observed = Rc::downgrade(&client);
                if dead {
                    (*client.get()).flags |= crate::src::shared::client::CLIENT_DEAD as uint64_t;
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
                std::ffi::CStr::from_ptr(
                    ((*reply).line)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
                )
                .to_bytes(),
                b"reply-\xff"
            );

            control_free_block(cs, output);
            assert_eq!(control_first_block(cs), reply);
            assert_eq!(cs.queued_reply_bytes, 8);
            control_free_block(cs, reply);
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
            let (first, first_weak) = control_add_block_with_weak(cs, control_block::new(None, 12));
            let (middle, middle_weak) =
                control_add_block_with_weak(cs, control_block::new(None, 23));
            let (last, last_weak) = control_add_block_with_weak(cs, control_block::new(None, 34));
            let mut pane = control_pane {
                pane: 7,
                offset: window_pane_offset { used: 0 },
                queued: window_pane_offset { used: 0 },
                flags: 0,
                pending_flag: 0,
                blocks: VecDeque::from([first_weak, middle_weak.clone(), last_weak]),
            };

            assert_eq!(control_first_pane_block(&pane), first);
            control_remove_pane_block(&mut pane, middle);
            control_free_block(cs, middle);
            assert!(!middle_weak.is_alive());
            assert_eq!(control_first_pane_block(&pane), first);
            assert_eq!((*last).size, 34);
            assert_eq!(cs.block(0), first);
            assert_eq!(cs.block(1), last);
            control_remove_pane_block(&mut pane, first);
            control_free_block(cs, first);
            assert_eq!(control_first_pane_block(&pane), last);
            assert_eq!(control_first_block(cs), last);
            control_remove_pane_block(&mut pane, last);
            control_free_block(cs, last);
            assert!(control_first_block(cs).is_null());
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
        let mut index = control_panes { storage: None };
        let mut wp = window_pane::empty();
        wp.id = 4;
        wp.offset.used = 123;
        let first = control_add_pane(&mut index, &wp);
        assert_eq!((first.offset.used, first.queued.used), (123, 123));
        first.flags = CONTROL_PANE_OFF;
        let address = first as *mut control_pane as usize;
        for id in 5..100 {
            wp.id = id;
            control_add_pane(&mut index, &wp);
        }
        wp.id = 4;
        wp.offset.used = 456;
        let first = control_add_pane(&mut index, &wp);
        assert_eq!(first as *mut control_pane as usize, address);
        assert_eq!(
            (first.offset.used, first.queued.used, first.flags),
            (123, 123, CONTROL_PANE_OFF)
        );
        let moved = index;
        assert_eq!(moved.get(9).unwrap().pane, 9);
        assert!(moved.get(100).is_none());
        drop(moved);
    }

    #[test]
    fn resetting_panes_drops_output_and_pending_ids_but_keeps_replies() {
        unsafe {
            let mut client = client::empty();
            client.control_state = Some(Box::new(control_state::new()));
            let owner = client.control_state.as_deref_mut().unwrap();
            let mut wp = window_pane::empty();
            let reply = control_add_block(
                owner,
                control_block::new(Some(CString::new("reply").unwrap()), 0),
            );
            owner.queued_reply_bytes = 6;
            for id in [4, 9] {
                wp.id = id;
                wp.offset.used = 123;
                let (_, block) = control_add_block_with_weak(owner, control_block::new(None, 10));
                let pane = control_add_pane(&mut owner.panes, &wp);
                pane.blocks.push_back(block);
                pane.pending_flag = 1;
                owner.pending_panes.push_back(id);
                owner.pending_count += 1;
            }
            assert_eq!(owner.pending_snapshot(), [4, 9]);
            control_reset_offsets(&mut client);
            let owner = client.control_state.as_deref_mut().unwrap();
            assert!(owner.panes.storage.is_none());
            assert!(owner.pending_panes.is_empty());
            assert_eq!(owner.pending_count, 0);
            assert_eq!(owner.all_blocks.len(), 1);
            assert_eq!(owner.block(0), reply);
            assert_eq!(owner.queued_reply_bytes, 6);
            control_reset_offsets(&mut client);
            let owner = client.control_state.as_deref_mut().unwrap();
            wp.id = 4;
            wp.offset.used = 456;
            let pane = control_add_pane(&mut owner.panes, &wp);
            assert_eq!(pane.offset.used, 456);
            assert_eq!(pane.pending_flag, 0);
            assert!(pane.blocks.is_empty());
            assert!(owner.pending_snapshot().is_empty());
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
            assert_ne!((*owner.get()).flags & CLIENT_EXIT as uint64_t, 0);
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
            client: std::rc::Weak<std::cell::UnsafeCell<client>>,
            label: &'static str,
            order: Rc<RefCell<Vec<&'static str>>>,
        }
        impl Drop for CleanupProbe {
            fn drop(&mut self) {
                let owner = self.client.upgrade().expect("client retained during control stop");
                unsafe { assert!((*owner.get()).control_state.is_some()); }
                self.order.borrow_mut().push(self.label);
            }
        }

        unsafe {
            for shared_stream in [false, true] {
                let owner = client::new();
                let client = &mut *owner.get();
                if shared_stream {
                    client.flags = CLIENT_CONTROLCONTROL as uint64_t;
                }
                client.control_state = Some(Box::new(control_state::new()));
                let c = owner.get();
                let order = Rc::new(RefCell::new(Vec::new()));
                let probe = CleanupProbe {
                    client: Rc::downgrade(&owner),
                    label: "monitor",
                    order: order.clone(),
                };
                let subs = monitor_create_client_owned(
                    (c).as_ref().and_then(|model| model.observer.upgrade()).as_ref(),
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
                let cs = client.control_state.as_deref_mut().unwrap();
                cs.subs = Some(subs);
                cs.read_event = crate::src::reactor::StreamHandle::from_ptr(read_event);
                cs.write_event = crate::src::reactor::StreamHandle::from_ptr(write_event);
                cs.windows.set(4, 80, 24);
                cs.deferred.push_back(CString::new("deferred").unwrap());
                control_add_block(
                    cs,
                    control_block::new(Some(CString::new("reply").unwrap()), 0),
                );
                let (_, output) = control_add_block_with_weak(cs, control_block::new(None, 10));
                let wp = window_pane::empty();
                let pane = control_add_pane(&mut cs.panes, &wp);
                pane.blocks.push_back(output);
                pane.pending_flag = 1;
                cs.pending_panes.push_back(wp.id);
                cs.pending_count = 1;
                let read_observer = cs.read_event.clone();
                let write_observer = cs.write_event.clone();

                control_stop(&(*(c)).observer.upgrade().expect("live client"));
                assert!(client.control_state.is_none());
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
                control_stop(&(*(c)).observer.upgrade().expect("live client"));
                control_read_callback(&owner);
                control_write_callback(&owner);
            }
        }
    }

    #[test]
    fn formatting_can_reenter_notifications_or_stop_the_owned_state() {
        unsafe {
            let owner = client::new();
            let c = owner.get();
            (*c).control_state = Some(Box::new(control_state::new()));
            (*c).control_state.as_deref_mut().unwrap().guard_depth = 1;
            control_notify_write(&owner, |out| {
                control_notify_write(&owner, |out| out.write_all(b"inner"));
                out.write_all(b"outer")
            });
            let cs = (*c).control_state.as_deref().unwrap();
            assert_eq!(
                cs.deferred
                    .iter()
                    .map(|line| line.as_bytes())
                    .collect::<Vec<_>>(),
                [b"inner".as_slice(), b"outer".as_slice()]
            );
            control_write(&owner, |out| {
                control_stop(&owner);
                out.write_all(b"stopped while formatting reply")
            });
            assert!((*c).control_state.is_none());
            (*c).control_state = Some(Box::new(control_state::new()));
            control_notify_write(&owner, |out| {
                control_stop(&owner);
                out.write_all(b"stopped while formatting notification")
            });
            assert!((*c).control_state.is_none());
        }
    }

    #[test]
    fn window_index_owns_overrides_and_borrows_stable_entries() {
        let mut index = control_windows { storage: None };
        assert!(index.get(4).is_none());
        index.remove(4);
        assert!(index.storage.is_none());

        index.set(4, 80, 24);
        let address = index.get(4).unwrap() as *const control_window as usize;
        for id in 5..100 {
            index.set(id, id, id + 1);
        }
        index.set(4, 120, 40);
        let mut moved = index;
        let first = moved.get(4).unwrap();
        assert_eq!(first as *const control_window as usize, address);
        assert_eq!((first.sx, first.sy), (120, 40));
        moved.remove(4);
        assert!(moved.get(4).is_none());
        assert_eq!(moved.get(9).unwrap().sx, 9);
        // Dropping a populated index owns cleanup for all remaining overrides.
        drop(moved);

        let mut index = control_windows { storage: None };
        index.set(4, 80, 24);
        index.remove(4);
        assert!(index.storage.is_none());
        index.set(4, 90, 30);
        assert_eq!(index.get(4).unwrap().sy, 30);
    }
}

unsafe fn control_free_block(cs: &mut control_state, cb: *mut control_block) {
    control_release_block(&mut cs.all_blocks, &mut cs.queued_reply_bytes, cb);
}

unsafe fn control_release_block(
    blocks: &mut VecDeque<refbox::RefBox<control_block>>,
    queued_reply_bytes: &mut size_t,
    cb: *mut control_block,
) {
    let mut size: size_t = 0;
    if (*cb).size == 0 as size_t && !(*cb).line.is_none() {
        size = (*cb)
            .line
            .as_ref()
            .expect("reply block has an owned line")
            .as_bytes_with_nul()
            .len();
        if *queued_reply_bytes > size {
            *queued_reply_bytes = queued_reply_bytes.wrapping_sub(size);
        } else {
            *queued_reply_bytes = 0 as size_t;
        }
    }
    let index = blocks
        .iter()
        .position(|owner| std::ptr::eq(owner.as_ptr(), cb))
        .expect("control block must be owned by its state");
    drop(blocks.remove(index).expect("located control block"));
}

fn control_add_pane<'a>(panes: &'a mut control_panes, wp: &window_pane) -> &'a mut control_pane {
    let map = panes.storage.get_or_insert_with(Box::default);
    map.entry(wp.id)
        .or_insert_with(|| {
            Box::new(control_pane {
                pane: wp.id,
                offset: window_pane_offset {
                    used: wp.offset.used,
                },
                queued: window_pane_offset {
                    used: wp.offset.used,
                },
                flags: 0,
                pending_flag: 0,
                blocks: VecDeque::new(),
            })
        })
        .as_mut()
}
pub unsafe fn control_set_window_size(c: &mut client, window: u_int, sx: u_int, sy: u_int) {
    if let Some(cs) = c.control_state.as_mut() {
        cs.windows.set(window, sx, sy);
    }
}

pub unsafe fn control_get_window_size(
    c: &client,
    window: u_int,
    sx: *mut u_int,
    sy: *mut u_int,
) -> ::core::ffi::c_int {
    let Some(cw) = c
        .control_state
        .as_ref()
        .and_then(|cs| cs.windows.get(window))
    else {
        return 0;
    };
    *sx = cw.sx;
    *sy = cw.sy;
    1
}

pub unsafe fn control_clear_window_size(c: &mut client, window: u_int) {
    if let Some(cs) = c.control_state.as_mut() {
        cs.windows.remove(window);
    }
}

unsafe fn control_discard_pane(cs: &mut control_state, pane: u_int) {
    let Some(cp) = cs.panes.get_mut(pane) else {
        return;
    };
    while let Some(block) = cp.blocks.pop_front() {
        control_release_block(
            &mut cs.all_blocks,
            &mut cs.queued_reply_bytes,
            control_block_ptr(&block),
        );
    }
}

unsafe fn control_window_pane(c: &client, pane: u_int) -> Option<Rc<UnsafeCell<window_pane>>> {
    let session = c.session.upgrade()?;
    let pane = window_pane_find_by_id(pane)?;
    if winlink_find_by_window(&raw mut (*session.get()).windows, &(*((*pane.get()).window_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()))).observer.upgrade().expect("live window")).is_null() {
        return None;
    }
    Some(pane)
}
pub unsafe fn control_reset_offsets(c: &mut client) {
    let cs = c
        .control_state
        .as_deref_mut()
        .expect("control client state");
    if let Some(panes) = cs.panes.storage.take() {
        for (_, mut pane) in *panes {
            while let Some(block) = pane.blocks.pop_front() {
                control_free_block(cs, control_block_ptr(&block));
            }
        }
    }
    cs.pending_panes.clear();
    cs.pending_count = 0;
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
    let cp = cs.panes.get_mut(pane)?;
    if cp.flags & CONTROL_PANE_PAUSED != 0 {
        return None;
    }
    if cp.flags & CONTROL_PANE_OFF != 0 {
        *off = 1;
        return None;
    }
    *off = (cs.write_event.with_ptr(|stream| unsafe {
        evbuffer_get_length(&*(*stream).output)
    }).unwrap_or(0) >= CONTROL_BUFFER_LOW as size_t)
        as ::core::ffi::c_int;
    Some(&mut cp.offset)
}
pub unsafe fn control_set_pane_on(c_owner: &Rc<UnsafeCell<client>>, wp_owner: &Rc<UnsafeCell<window_pane>>) {
    let c = c_owner.get();
    let wp = wp_owner.get();

    let Some(cp) = (*c)
        .control_state
        .as_deref_mut()
        .expect("control client state")
        .panes
        .get_mut((*wp).id)
    else {
        return;
    };
    if cp.flags & CONTROL_PANE_OFF != 0 {
        cp.flags &= !CONTROL_PANE_OFF;
        cp.offset.used = (*wp).offset.used;
        cp.queued.used = (*wp).offset.used;
    }
}
pub unsafe fn control_set_pane_off(c_owner: &Rc<UnsafeCell<client>>, wp_owner: &Rc<UnsafeCell<window_pane>>) {
    let c = c_owner.get();
    let wp = wp_owner.get();

    let cs = (*c)
        .control_state
        .as_deref_mut()
        .expect("control client state");
    control_add_pane(&mut cs.panes, &*wp);
    control_discard_pane(cs, (*wp).id);
    let cp = cs.panes.get_mut((*wp).id).expect("indexed control pane");
    cp.offset.used = (*wp).offset.used;
    cp.queued.used = (*wp).offset.used;
    cp.flags |= CONTROL_PANE_OFF;
}
pub unsafe fn control_continue_pane(c_owner: &Rc<UnsafeCell<client>>, wp_owner: &Rc<UnsafeCell<window_pane>>) {
    let c = c_owner.get();
    let wp = wp_owner.get();

    let Some(cp) = (*c)
        .control_state
        .as_deref_mut()
        .expect("control client state")
        .panes
        .get_mut((*wp).id)
    else {
        return;
    };
    if cp.flags & CONTROL_PANE_PAUSED != 0 {
        cp.flags &= !CONTROL_PANE_PAUSED;
        cp.offset.used = (*wp).offset.used;
        cp.queued.used = (*wp).offset.used;
        control_notify_write(c_owner, |out| write!(out, "%continue %{}", (*wp).id));
    }
}
pub unsafe fn control_pause_pane(c_owner: &Rc<UnsafeCell<client>>, wp_owner: &Rc<UnsafeCell<window_pane>>) {
    let c = c_owner.get();
    let wp = wp_owner.get();

    let cs = (*c)
        .control_state
        .as_deref_mut()
        .expect("control client state");
    let cp = control_add_pane(&mut cs.panes, &*wp);
    if cp.flags & CONTROL_PANE_PAUSED == 0 {
        cp.flags |= CONTROL_PANE_PAUSED;
        control_discard_pane(cs, (*wp).id);
        control_notify_write(c_owner, |out| write!(out, "%pause %{}", (*wp).id));
    }
}
pub unsafe fn control_reset_pane(c_owner: &Rc<UnsafeCell<client>>, wp_owner: &Rc<UnsafeCell<window_pane>>) {
    let c = c_owner.get();
    let wp = wp_owner.get();

    let Some(cs) = (*c).control_state.as_deref_mut() else {
        return;
    };
    control_discard_pane(cs, (*wp).id);
    if let Some(cp) = cs.panes.get_mut((*wp).id) {
        cp.offset.used = (*wp).offset.used;
        cp.queued.used = (*wp).offset.used;
    }
}
unsafe fn control_check_reply_buffer(c_owner: &Rc<UnsafeCell<client>>, mut added: size_t) -> ::core::ffi::c_int {
    let c = c_owner.get();

    let Some(cs) = (*c).control_state.as_deref_mut() else {
        return 1;
    };
    let mut size: size_t = 0;
    if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_CONTROL_DISCARD != 0 {
        return 1 as ::core::ffi::c_int;
    }
    size = cs.write_event.with_ptr(|stream| unsafe {
        evbuffer_get_length(&*(*stream).output)
    }).unwrap_or(0);
    size = size.wrapping_add(cs.queued_reply_bytes);
    size = size.wrapping_add(added);
    if size < CONTROL_MAXIMUM_REPLY_BUFFER as size_t {
        return 0 as ::core::ffi::c_int;
    }
    log_debug(format_args!(
        "{}: {}: {} bytes of replies buffered",
        "control_check_reply_buffer",
        log_cstr(
            (((*c).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                as *const _
        ),
        (size) as usize
    ));
    if !(*c).flags & CLIENT_EXIT as uint64_t != 0 {
        server_client_set_exit_message(&mut *c, Some(CString::new("too far behind").unwrap()));
        (*c).flags |= CLIENT_EXIT as uint64_t;
        control_discard(&mut *c);
    }
    (*c).flags = ((*c).flags as ::core::ffi::c_ulonglong | CLIENT_CONTROL_DISCARD) as uint64_t;
    return 1 as ::core::ffi::c_int;
}
unsafe fn control_write_line(c_owner: &Rc<UnsafeCell<client>>, line: CString) {
    let c = c_owner.get();

    let mut cb: *mut control_block = ::core::ptr::null_mut::<control_block>();
    let size = line.as_bytes_with_nul().len() as size_t;
    if control_check_reply_buffer(c_owner, size) != 0 {
        return;
    }
    let Some(cs) = (*c).control_state.as_deref_mut() else {
        return;
    };
    if control_first_block(cs).is_null() {
        log_debug(format_args!(
            "{}: {}: writing line: {}",
            "control_write_line",
            log_cstr(
                (((*c).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            ),
            log_cstr((line.as_ptr()) as *const _)
        ));
        let _ = cs.write_event.with_ptr(|stream| unsafe {
            bufferevent_write(
                stream,
                line.as_ptr() as *const ::core::ffi::c_void,
                size.wrapping_sub(1 as size_t),
            );
            bufferevent_write(
                stream,
                b"\n\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                1 as size_t,
            );
            bufferevent_enable(stream, EV_WRITE as ::core::ffi::c_short)
        });
        return;
    }
    cb = control_add_block(cs, control_block::new(Some(line), 0));
    cs.queued_reply_bytes = cs.queued_reply_bytes.wrapping_add(size);
    (*cb).t = get_timer();
    log_debug(format_args!(
        "{}: {}: storing line: {}",
        "control_write_line",
        log_cstr(
            (((*c).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                as *const _
        ),
        log_cstr(
            (((*cb).line)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                as *const _
        )
    ));
    let _ = cs.write_event.with_ptr(|stream| unsafe {
        bufferevent_enable(stream, EV_WRITE as ::core::ffi::c_short)
    });
}
unsafe fn control_flush_deferred(c_owner: &Rc<UnsafeCell<client>>) {
    let c = c_owner.get();

    loop {
        let line = (*c)
            .control_state
            .as_deref_mut()
            .and_then(|cs| cs.deferred.pop_front());
        let Some(line) = line else {
            break;
        };
        control_write_line(c_owner, line);
    }
}
pub unsafe fn control_write(
    c_owner: &Rc<UnsafeCell<client>>,
    write: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
) {
    let c = c_owner.get();

    if (*c).control_state.is_none() {
        return;
    }
    let line = format_message_with(write);
    control_write_line(c_owner, line);
}
pub unsafe fn control_write_guard(
    c_owner: &Rc<UnsafeCell<client>>,
    mut guard: *const ::core::ffi::c_char,
    mut t: ::core::ffi::c_long,
    mut number: u_int,
    mut flags: ::core::ffi::c_int,
) {
    let c = c_owner.get();

    let Some(cs) = (*c).control_state.as_deref_mut() else {
        return;
    };
    if strcmp(guard, b"begin\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        cs.guard_depth += 1;
    }
    control_write(c_owner, |out| {
        out.write_all(b"%")?;
        write_cstr(out, guard)?;
        write!(
            out,
            " {} {} {}",
            (t) as ::core::ffi::c_long,
            (number) as u32,
            (flags) as i32
        )
    });
    let Some(cs) = (*c).control_state.as_deref_mut() else {
        return;
    };
    if strcmp(guard, b"begin\0" as *const u8 as *const ::core::ffi::c_char)
        != 0 as ::core::ffi::c_int
        && cs.guard_depth > 0 as ::core::ffi::c_int
        && {
            cs.guard_depth -= 1;
            cs.guard_depth == 0 as ::core::ffi::c_int
        }
    {
        control_flush_deferred(c_owner);
    }
}
pub unsafe fn control_notify_write(
    c_owner: &Rc<UnsafeCell<client>>,
    write: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
) {
    let c = c_owner.get();

    if (*c).control_state.is_none() {
        return;
    }
    let line = format_message_with(write);
    let Some(cs) = (*c).control_state.as_deref_mut() else {
        return;
    };
    if cs.guard_depth == 0 as ::core::ffi::c_int {
        control_write_line(c_owner, line);
        return;
    }
    log_debug(format_args!(
        "{}: {}: deferring notification: {}",
        "control_notify_write",
        log_cstr(
            (((*c).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                as *const _
        ),
        log_cstr((line.as_ptr()) as *const _)
    ));
    cs.deferred.push_back(line);
}
unsafe fn control_check_age(
    c_owner: &Rc<UnsafeCell<client>>,
    wp_owner: &Rc<UnsafeCell<window_pane>>,
    pane: u_int,
) -> ::core::ffi::c_int {
    let c = c_owner.get();
    let wp = wp_owner.get();

    let mut cb: *mut control_block = ::core::ptr::null_mut::<control_block>();
    let mut t: uint64_t = 0;
    let mut age: uint64_t = 0;
    let cs = (*c)
        .control_state
        .as_deref_mut()
        .expect("control client state");
    cb = control_first_pane_block(cs.panes.get(pane).expect("indexed control pane"));
    if cb.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    t = get_timer();
    if (*cb).t >= t {
        return 0 as ::core::ffi::c_int;
    }
    age = t.wrapping_sub((*cb).t);
    log_debug(format_args!(
        "{}: {}: %{} is {} behind",
        "control_check_age",
        log_cstr(
            (((*c).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                as *const _
        ),
        ((*wp).id) as u32,
        age as ::core::ffi::c_ulonglong
    ));
    if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_CONTROL_PAUSEAFTER != 0 {
        if age < (*c).pause_age as uint64_t {
            return 0 as ::core::ffi::c_int;
        }
        let cp = cs.panes.get_mut(pane).expect("indexed control pane");
        cp.flags |= CONTROL_PANE_PAUSED;
        control_discard_pane(cs, pane);
        control_notify_write(c_owner, |out| write!(out, "%pause %{}", ((*wp).id) as u32));
    } else {
        if age < CONTROL_MAXIMUM_AGE as uint64_t {
            return 0 as ::core::ffi::c_int;
        }
        server_client_set_exit_message(&mut *c, Some(CString::new("too far behind").unwrap()));
        (*c).flags |= CLIENT_EXIT as uint64_t;
        control_discard(&mut *c);
    }
    return 1 as ::core::ffi::c_int;
}
pub unsafe fn control_write_output(c_owner: &Rc<UnsafeCell<client>>, wp_owner: &Rc<UnsafeCell<window_pane>>) {
    let c = c_owner.get();
    let wp = wp_owner.get();

    let cs = (*c)
        .control_state
        .as_deref_mut()
        .expect("control client state");
    let pane = (*wp).id;
    let mut cb: *mut control_block = ::core::ptr::null_mut::<control_block>();
    let mut new_size: size_t = 0;
    if winlink_find_by_window(
        &raw mut (*(*c).session_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get())).windows,
        &(*((*wp).window_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()))).observer.upgrade().expect("live window"),
    )
    .is_null()
    {
        return;
    }
    if (*c).flags & (CONTROL_IGNORE_FLAGS | CLIENT_EXIT) as uint64_t != 0 {
        if cs.panes.get(pane).is_none() {
            return;
        }
    } else {
        let cp = control_add_pane(&mut cs.panes, &*wp);
        if !(cp.flags & (CONTROL_PANE_OFF | CONTROL_PANE_PAUSED) != 0) {
            if control_check_age(c_owner, wp_owner, pane) != 0 {
                return;
            }
            let cs = (*c)
                .control_state
                .as_deref_mut()
                .expect("control client state");
            let cp = cs.panes.get_mut(pane).expect("indexed control pane");
            new_size = (*wp).event.with_ptr(|event| unsafe {
                window_pane_get_new_data(&mut *(*event).input, (*wp).base_offset, &cp.queued).len()
            }).unwrap_or(0);
            if new_size == 0 as size_t {
                return;
            }
            window_pane_update_used_data(&(*(wp)).observer.upgrade().expect("live window_pane"), &raw mut cp.queued, new_size);
            let (new_block, block) =
                control_add_block_with_weak(cs, control_block::new(None, new_size));
            cb = new_block;
            (*cb).t = get_timer();
            let cp = cs.panes.get_mut(pane).expect("indexed control pane");
            cp.blocks.push_back(block);
            log_debug(format_args!(
                "{}: {}: new output block of {} for %{}",
                "control_write_output",
                log_cstr(
                    (((*c).name)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                        as *const _
                ),
                ((*cb).size) as usize,
                ((*wp).id) as u32
            ));
            if cp.pending_flag == 0 {
                log_debug(format_args!(
                    "{}: {}: %{} now pending",
                    "control_write_output",
                    log_cstr(
                        (((*c).name)
                            .as_ref()
                            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                            as *const _
                    ),
                    ((*wp).id) as u32
                ));
                cs.pending_panes.push_back(pane);
                cp.pending_flag = 1 as ::core::ffi::c_int;
                cs.pending_count = cs.pending_count.wrapping_add(1);
            }
            let _ = cs.write_event.with_ptr(|stream| unsafe {
                bufferevent_enable(stream, EV_WRITE as ::core::ffi::c_short)
            });
            return;
        }
    }
    log_debug(format_args!(
        "{}: {}: ignoring pane %{}",
        "control_write_output",
        log_cstr(
            (((*c).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                as *const _
        ),
        ((*wp).id) as u32
    ));
    let cp = cs.panes.get_mut(pane).expect("indexed control pane");
    window_pane_update_used_data(&(*(wp)).observer.upgrade().expect("live window_pane"), &raw mut cp.offset, SIZE_MAX as size_t);
    window_pane_update_used_data(&(*(wp)).observer.upgrade().expect("live window_pane"), &raw mut cp.queued, SIZE_MAX as size_t);
}
unsafe fn control_error(item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>, error: Option<CString>) -> cmd_retval {
    let item = item_handle.get();
    let c_owner = cmdq_get_client((item).as_ref());
    let mut c: *mut client = c_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    cmdq_guard(
        item_handle,
        b"begin\0" as *const u8 as *const ::core::ffi::c_char,
        1 as ::core::ffi::c_int,
    );
    control_write(&(*(c)).observer.upgrade().expect("live client"), |out| {
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
unsafe fn control_error_callback(owner: &Rc<UnsafeCell<client>>) {
    let c = owner.get();
    (*c).flags |= CLIENT_EXIT as uint64_t;
}
unsafe fn control_read_callback(owner: &Rc<UnsafeCell<client>>) {
    let c = owner.get();
    loop {
        let Some(cs) = (*c).control_state.as_deref_mut() else {
            break;
        };
        let Some(line) = cs.read_event.with_ptr(|stream| unsafe {
            evbuffer_readln(&mut *(*stream).input)
        }).flatten() else {
            break;
        };
        log_debug(format_args!(
            "{}: {}: {}",
            "control_read_callback",
            log_cstr(
                (((*c).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            ),
            log_cstr((line.as_ptr().cast::<::core::ffi::c_char>()) as *const _)
        ));
        if line[0] == 0 {
            (*c).flags |= CLIENT_EXIT as uint64_t;
            break;
        } else {
            let state = cmdq_new_state(
                ::core::ptr::null_mut::<cmd_find_state>(),
                ::core::ptr::null_mut::<key_event>(),
                CMDQ_STATE_CONTROL,
            );
            match cmd_parse_and_append(
                CStr::from_ptr(line.as_ptr().cast::<::core::ffi::c_char>()),
                Some(owner), Some(&state),
            ) {
                Err(error) => {
                    let error_item_allocation = cmdq_get_callback_owned(
                        b"control_error\0" as *const u8 as *const ::core::ffi::c_char,
                        Some(Box::new(move |item| unsafe {
                            control_error(item, error)
                        })),
                    );
                    cmdq_append(Some(owner), error_item_allocation);
                }
                Ok(_) => {}
            }

        }
    }
}
pub unsafe fn control_all_done(c: &client) -> ::core::ffi::c_int {
    let cs = (*c)
        .control_state
        .as_deref()
        .expect("control client state");
    if !control_first_block(cs).is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (cs.write_event.with_ptr(|stream| unsafe {
        evbuffer_get_length(&*(*stream).output)
    }).unwrap_or(0) == 0 as size_t)
        as ::core::ffi::c_int;
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
unsafe fn control_flush_all_blocks(c_owner: &Rc<UnsafeCell<client>>) {
    let c = c_owner.get();

    let cs = (*c)
        .control_state
        .as_deref_mut()
        .expect("control client state");
    loop {
        let cb = control_first_block(cs);
        if cb.is_null() || (*cb).size != 0 as size_t {
            break;
        }
        log_debug(format_args!(
            "{}: {}: flushing line: {}",
            "control_flush_all_blocks",
            log_cstr(
                (((*c).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            ),
            log_cstr(
                (((*cb).line)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            )
        ));
        let Some(()) = cs.write_event.with_ptr(|stream| unsafe {
            bufferevent_write(
                stream,
                ((*cb).line)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
                    as *const ::core::ffi::c_void,
                strlen(
                    ((*cb).line)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                ),
            );
            bufferevent_write(
                stream,
                b"\n\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                1 as size_t,
            );
        }) else { break };
        control_free_block(cs, cb);
    }
}
unsafe fn control_append_data(
    c_owner: &Rc<UnsafeCell<client>>,
    cp: &mut control_pane,
    mut age: uint64_t,
    message: Option<Box<evbuffer>>,
    wp_owner: &Rc<UnsafeCell<window_pane>>,
    mut size: size_t,
) -> Box<evbuffer> {
    let c = c_owner.get();
    let wp = wp_owner.get();

    let mut new_data: *mut u_char = ::core::ptr::null_mut::<u_char>();
    let mut new_size: size_t = 0;
    let mut start: size_t = 0;
    let mut i: u_int = 0;
    let mut message = message.unwrap_or_else(|| {
        let mut message = evbuffer_new();
        if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_CONTROL_PAUSEAFTER != 0 {
            evbuffer_add_formatted(&mut *message, |out| {
                write!(
                    out,
                    "%extended-output %{} {} : ",
                    ((*wp).id) as u32,
                    (age as ::core::ffi::c_ulonglong) as u64
                )
            });
        } else {
            evbuffer_add_formatted(&mut *message, |out| {
                write!(out, "%output %{} ", ((*wp).id) as u32)
            });
        }
        message
    });
    let data = (*wp).event.with_ptr(|event| unsafe {
        window_pane_get_new_data(&mut *(*event).input, (*wp).base_offset, &cp.offset).to_vec()
    }).unwrap_or_default();
    new_size = data.len();
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
    window_pane_update_used_data(&(*(wp)).observer.upgrade().expect("live window_pane"), &raw mut cp.offset, size);
    return message;
}
unsafe fn control_write_data(c_owner: &Rc<UnsafeCell<client>>, mut message: Box<evbuffer>) {
    let c = c_owner.get();

    let cs = (*c)
        .control_state
        .as_deref_mut()
        .expect("control client state");
    log_debug(format_args!(
        "{}: {}: {}",
        "control_write_data",
        log_cstr(
            (((*c).name)
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
    let _ = cs.write_event.with_ptr(|stream| unsafe {
        bufferevent_write_buffer(stream, &mut *message)
    });
}
unsafe fn control_write_pending(
    c_owner: &Rc<UnsafeCell<client>>,
    pane: u_int,
    mut limit: size_t,
) -> ::core::ffi::c_int {
    let c = c_owner.get();

    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut message: Option<Box<evbuffer>> = None;
    let mut used: size_t = 0 as size_t;
    let mut size: size_t = 0;
    let mut cb: *mut control_block = ::core::ptr::null_mut::<control_block>();
    let mut age: uint64_t = 0;
    let mut t: uint64_t = get_timer();
    let pane_owner = control_window_pane(&*c, pane);
    wp = pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    if wp.is_null() || (*wp).fd == -(1 as ::core::ffi::c_int) {
        control_discard_pane(
            (*c).control_state
                .as_deref_mut()
                .expect("control client state"),
            pane,
        );
        control_flush_all_blocks(c_owner);
        return 0 as ::core::ffi::c_int;
    }
    while used != limit
        && !(*c)
            .control_state
            .as_deref()
            .expect("control client state")
            .panes
            .get(pane)
            .expect("indexed control pane")
            .blocks
            .is_empty()
    {
        if control_check_age(c_owner, pane_owner.as_ref().expect("live control pane"), pane) != 0 {
            message = None;
            break;
        } else {
            let cs = (*c)
                .control_state
                .as_deref_mut()
                .expect("control client state");
            cb = control_first_pane_block(cs.panes.get(pane).expect("indexed control pane"));
            if (*cb).t < t {
                age = t.wrapping_sub((*cb).t);
            } else {
                age = 0 as uint64_t;
            }
            log_debug(format_args!(
                "{}: {}: output block {} (age {}) for %{} (used {}/{})",
                "control_write_pending",
                log_cstr(
                    (((*c).name)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                        as *const _
                ),
                ((*cb).size) as usize,
                age as ::core::ffi::c_ulonglong,
                (pane) as u32,
                (used) as usize,
                (limit) as usize
            ));
            size = (*cb).size;
            if size > limit.wrapping_sub(used) {
                size = limit.wrapping_sub(used);
            }
            used = used.wrapping_add(size);
            message = Some(control_append_data(
                c_owner,
                cs.panes.get_mut(pane).expect("indexed control pane"),
                age,
                message,
                pane_owner.as_ref().expect("live control pane"),
                size,
            ));
            (*cb).size = (*cb).size.wrapping_sub(size);
            if (*cb).size == 0 as size_t {
                control_remove_pane_block(
                    cs.panes.get_mut(pane).expect("indexed control pane"),
                    cb,
                );
                control_free_block(cs, cb);
                cb = control_first_block(cs);
                if !cb.is_null() && (*cb).size == 0 as size_t {
                    if !wp.is_null() {
                        if let Some(message) = message.take() {
                            control_write_data(c_owner, message);
                        }
                    }
                    control_flush_all_blocks(c_owner);
                }
            }
        }
    }
    if let Some(message) = message {
        control_write_data(c_owner, message);
    }
    return !(*c)
        .control_state
        .as_deref()
        .expect("control client state")
        .panes
        .get(pane)
        .expect("indexed control pane")
        .blocks
        .is_empty() as ::core::ffi::c_int;
}
unsafe fn control_write_callback(owner: &Rc<UnsafeCell<client>>) {
    let c = owner.get();
    if (*c).control_state.is_none() {
        return;
    }
    control_flush_all_blocks(owner);
    loop {
        let Some(cs) = (*c).control_state.as_deref_mut() else {
            return;
        };
        let Some(buffered) = cs.write_event.with_ptr(|stream| unsafe {
            evbuffer_get_length(&*(*stream).output)
        }) else { return };
        if buffered >= CONTROL_BUFFER_HIGH as size_t || cs.pending_count == 0 {
            break;
        }
        let space = (CONTROL_BUFFER_HIGH as size_t).wrapping_sub(buffered);
        log_debug(format_args!(
            "{}: {}: {} bytes available, {} panes",
            "control_write_callback",
            log_cstr(
                (*c).name
                    .as_ref()
                    .map_or(std::ptr::null(), |name| name.as_ptr())
            ),
            space,
            cs.pending_count
        ));
        let limit = (space / cs.pending_count as size_t / 3).max(CONTROL_WRITE_MINIMUM as size_t);
        let pending = cs.pending_snapshot();
        for pane in pending {
            let Some(cs) = (*c).control_state.as_deref_mut() else {
                return;
            };
            let Some(buffered) = cs.write_event.with_ptr(|stream| unsafe {
                evbuffer_get_length(&*(*stream).output)
            }) else { return };
            if buffered >= CONTROL_BUFFER_HIGH as size_t {
                break;
            }
            if !cs.pending_panes.contains(&pane) {
                continue;
            }
            if control_write_pending(owner, pane, limit) == 0 {
                let Some(cs) = (*c).control_state.as_deref_mut() else {
                    return;
                };
                if cs.remove_pending(pane) {
                    cs.panes
                        .get_mut(pane)
                        .expect("indexed control pane")
                        .pending_flag = 0;
                    cs.pending_count = cs.pending_count.wrapping_sub(1);
                }
            }
        }
    }
    if let Some(cs) = (*c).control_state.as_deref() {
        if cs.write_event.with_ptr(|stream| unsafe {
            evbuffer_get_length(&*(*stream).output)
        }) == Some(0) {
            let _ = cs.write_event.with_ptr(|stream| unsafe {
                bufferevent_disable(stream, EV_WRITE as ::core::ffi::c_short)
            });
        }
    }
}
unsafe fn control_sub_change(change: &monitor_change) {
    let Some(client_owner) = change.c.upgrade() else { return };
    let c = crate::src::shared::rc::as_ptr(&client_owner);
    if (*c).flags & crate::src::shared::client::CLIENT_DEAD as uint64_t != 0 {
        drop(client_owner);
        return;
    }
    let Some(session_owner) = change.s.upgrade() else {
        drop(client_owner);
        return;
    };
    let s = crate::src::shared::rc::as_ptr(&session_owner);
    let mut link = change.wl.try_borrow_mut().ok();
    if !change.wl.is_empty() && link.is_none() {
        drop(session_owner);
        drop(client_owner);
        return;
    }
    let wl = link.as_mut().map_or(std::ptr::null_mut(), |link| &raw mut **link);
    let pane_owner = crate::src::window::window_pane_upgrade(&change.wp);
    if !std::rc::Weak::ptr_eq(&change.wp, &std::rc::Weak::new()) && pane_owner.is_none() {
        drop(session_owner);
        drop(client_owner);
        return;
    }
    let wp = pane_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    if !wp.is_null() && !wl.is_null() {
        w = (*wp).window_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        control_notify_write(&(*(c)).observer.upgrade().expect("live client"), |out| {
            out.write_all(b"%subscription-changed ")?;
            out.write_all(change.name.to_bytes())?;
            write!(
                out,
                " ${} @{} {} %{} : ",
                ((*s).id) as u32,
                ((*w).id) as u32,
                ((*wl).idx) as u32,
                ((*wp).id) as u32
            )?;
            out.write_all(change.value.to_bytes())
        });
    } else if !wl.is_null() {
        w = (*wl).window_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
        control_notify_write(&(*(c)).observer.upgrade().expect("live client"), |out| {
            out.write_all(b"%subscription-changed ")?;
            out.write_all(change.name.to_bytes())?;
            write!(
                out,
                " ${} @{} {} - : ",
                ((*s).id) as u32,
                ((*w).id) as u32,
                ((*wl).idx) as u32
            )?;
            out.write_all(change.value.to_bytes())
        });
    } else {
        control_notify_write(&(*(c)).observer.upgrade().expect("live client"), |out| {
            out.write_all(b"%subscription-changed ")?;
            out.write_all(change.name.to_bytes())?;
            write!(out, " ${} - - - : ", ((*s).id) as u32)?;
            out.write_all(change.value.to_bytes())
        });
    };
    drop(session_owner);
    drop(client_owner);
}
fn control_stream_callbacks(owner: &Rc<UnsafeCell<client>>) -> (bufferevent_data_cb, bufferevent_data_cb, bufferevent_event_cb) {
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

pub unsafe fn control_start(owner: &Rc<UnsafeCell<client>>) {
    let c = owner.get();
    if (*c).flags & CLIENT_CONTROLCONTROL as uint64_t != 0 {
        close((*c).out_fd);
        (*c).out_fd = -(1 as ::core::ffi::c_int);
    } else {
        setblocking((*c).out_fd, 0 as ::core::ffi::c_int);
    }
    setblocking((*c).fd, 0 as ::core::ffi::c_int);
    (*c).control_state = Some(Box::new(control_state::new()));
    let subs = monitor_create_client_owned(
        (c).as_ref().and_then(|model| model.observer.upgrade()).as_ref(),
        monitor_callback(|change| unsafe { control_sub_change(change) }),
    );
    let cs = (*c)
        .control_state
        .as_deref_mut()
        .expect("control client state");
    cs.subs = Some(subs);
    let (read, write, error) = control_stream_callbacks(owner);
    let read_stream = bufferevent_new(
        (*c).fd,
        read,
        write.clone(),
        error.clone(),
    );
    if read_stream.is_null() {
        fatalx(|out| out.write_all(b"out of memory"));
    }
    cs.read_event = crate::src::reactor::StreamHandle::from_ptr(read_stream);
    if (*c).flags & CLIENT_CONTROLCONTROL as uint64_t != 0 {
        cs.write_event = cs.read_event.clone();
    } else {
        let write_stream = bufferevent_new(
            (*c).out_fd,
            None,
            write,
            error,
        );
        if write_stream.is_null() {
            fatalx(|out| out.write_all(b"out of memory"));
        }
        cs.write_event = crate::src::reactor::StreamHandle::from_ptr(write_stream);
    }
    let _ = cs.write_event.with_ptr(|stream| unsafe {
        bufferevent_setwatermark(stream)
    });
    if (*c).flags & CLIENT_CONTROLCONTROL as uint64_t != 0 {
        let _ = cs.write_event.with_ptr(|stream| unsafe {
            bufferevent_write(
                stream,
                b"\x1BP1000p\0" as *const u8 as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                7 as size_t,
            );
            bufferevent_enable(stream, EV_WRITE as ::core::ffi::c_short)
        });
    }
}
pub unsafe fn control_ready(c_owner: &Rc<UnsafeCell<client>>) {
    let c = c_owner.get();

    let cs = (*c).control_state.as_deref_mut().expect("control client state");
    let _ = cs.read_event.with_ptr(|stream| unsafe {
        bufferevent_enable(stream, EV_READ as ::core::ffi::c_short)
    });
}
pub unsafe fn control_discard(c: &mut client) {
    let cs = (*c)
        .control_state
        .as_deref_mut()
        .expect("control client state");
    if let Some(panes) = cs.panes.storage.as_mut() {
        for cp in panes.values_mut() {
            while let Some(block) = cp.blocks.pop_front() {
                control_release_block(
                    &mut cs.all_blocks,
                    &mut cs.queued_reply_bytes,
                    control_block_ptr(&block),
                );
            }
        }
    }
    let _ = cs.read_event.with_ptr(|stream| unsafe {
        bufferevent_disable(stream, EV_READ as ::core::ffi::c_short)
    });
}

pub unsafe fn control_discard_all(c: &mut client) {
    control_discard(c);
    let cs = (*c)
        .control_state
        .as_deref_mut()
        .expect("control client state");
    loop {
        let cb = control_first_block(cs);
        if cb.is_null() {
            break;
        }
        control_free_block(cs, cb);
    }
    cs.queued_reply_bytes = 0 as size_t;
    let _ = cs.write_event.with_ptr(|stream| unsafe {
        bufferevent_disable(stream, EV_WRITE as ::core::ffi::c_short)
    });
}
pub unsafe fn control_stop(c_owner: &Rc<UnsafeCell<client>>) {
    let c = c_owner.get();

    let Some(cs) = (*c).control_state.as_deref_mut() else {
        return;
    };
    let subs = cs.subs.take();
    // Keep the owner published until callbacks and external resources are gone.
    if let Some(subs) = subs {
        crate::src::monitor::monitor_destroy(subs);
    }
    if (*c).flags & CLIENT_CONTROLCONTROL as uint64_t == 0 {
        cs.write_event.free();
    } else {
        cs.write_event = Default::default();
    }
    cs.read_event.free();
    control_reset_offsets(&mut *(c));
    let cs = (*c)
        .control_state
        .as_deref_mut()
        .expect("control client state");
    cs.windows.storage = None;
    loop {
        let cb = control_first_block(cs);
        if cb.is_null() {
            break;
        }
        control_free_block(cs, cb);
    }
    drop((*c).control_state.take());
}
pub unsafe fn control_add_sub(
    c_owner: &Rc<UnsafeCell<client>>,
    mut name: *const ::core::ffi::c_char,
    mut type_0: monitor_type,
    mut id: ::core::ffi::c_int,
    mut format: *const ::core::ffi::c_char,
) {
    let c = c_owner.get();

    let subs = (*c)
        .control_state
        .as_deref_mut()
        .expect("control client state")
        .subs.as_deref_mut().expect("control subscriptions") as *mut _;
    monitor_add(subs, name, type_0, id, format, MONITOR_NOTIFY_INITIAL);
}
pub unsafe fn control_remove_sub(c_owner: &Rc<UnsafeCell<client>>, mut name: *const ::core::ffi::c_char) {
    let c = c_owner.get();

    let subs = (*c)
        .control_state
        .as_deref_mut()
        .expect("control client state")
        .subs.as_deref_mut().expect("control subscriptions") as *mut _;
    monitor_remove(subs, name);
}

impl control_panes {
    fn get(&self, pane: u_int) -> Option<&control_pane> {
        self.storage.as_ref()?.get(&pane).map(Box::as_ref)
    }

    fn get_mut(&mut self, pane: u_int) -> Option<&mut control_pane> {
        self.storage.as_mut()?.get_mut(&pane).map(Box::as_mut)
    }
}

impl control_windows {
    fn get(&self, window: u_int) -> Option<&control_window> {
        self.storage.as_ref()?.get(&window).map(Box::as_ref)
    }

    fn set(&mut self, window: u_int, sx: u_int, sy: u_int) {
        let map = self.storage.get_or_insert_with(Box::default);
        let entry = map
            .entry(window)
            .or_insert_with(|| Box::new(control_window { window, sx, sy }));
        entry.sx = sx;
        entry.sy = sy;
    }

    fn remove(&mut self, window: u_int) {
        let Some(map) = self.storage.as_mut() else {
            return;
        };
        map.remove(&window);
        if map.is_empty() {
            self.storage = None;
        }
    }
}
