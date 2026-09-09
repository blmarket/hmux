//! Coverage for [`crate::file`] — constants and pure helpers.
//!
//! Exercises the message-number constants, [`file_find_ref`] / [`ClientFileRef::close`],
//! [`file_can_print`], [`file_write_left`] and the `file_fire_*` callbacks
//! through a fake client/peer map without touching the event loop's fatal
//! paths (`file_write_open` / `file_read_open` size checks, `proc_send`).

use crate::file::{
    CLIENT_ATTACHED, CLIENT_CONTROL, MSG_READ, MSG_READ_CANCEL, MSG_READ_DONE, MSG_READ_OPEN,
    MSG_WRITE, MSG_WRITE_CLOSE, MSG_WRITE_OPEN, MSG_WRITE_READY, file_can_print, file_find_ref,
    file_write_left,
};
use crate::reactor::Reactor;
use crate::tests::test_fixtures::{ensure_reactor, globals, zeroed_client};
use crate::types::*;
use ::std::sync::atomic::{AtomicI32, Ordering};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

#[test]
fn message_constants_keep_their_upstream_values() {
    assert_eq!(MSG_READ_OPEN, 300);
    assert_eq!(MSG_READ, 301);
    assert_eq!(MSG_READ_DONE, 302);
    assert_eq!(MSG_WRITE_OPEN, 303);
    assert_eq!(MSG_WRITE, 304);
    assert_eq!(MSG_WRITE_READY, 305);
    assert_eq!(MSG_WRITE_CLOSE, 306);
    assert_eq!(MSG_READ_CANCEL, 307);
}

// ---------------------------------------------------------------------------
// file_find_ref / file_create_with_peer
// ---------------------------------------------------------------------------

#[test]
fn file_find_ref_on_peer_map_covers_missing_and_present() {
    let _guard = globals();
    unsafe {
        ensure_reactor();
        let files = std::rc::Rc::new(std::cell::RefCell::new(client_files_t::new()));
        assert!(file_find_ref(&files.borrow(), 10).is_none());
        let cf = ClientFileRef::create_with_peer(
            None,
            &FileOwner::Shared(std::rc::Rc::downgrade(&files)),
            10,
            None,
            ClientFileData::None,
        );
        assert!(file_find_ref(&files.borrow(), 10).unwrap().ptr_eq(&cf));
        assert!(file_find_ref(&files.borrow(), 11).is_none());
        cf.close();
        assert!(file_find_ref(&files.borrow(), 10).is_none());
    }
}

#[test]
fn file_create_with_client_detaches_attached_client() {
    let _guard = globals();
    unsafe {
        ensure_reactor();
        let mut c = zeroed_client();
        *c.flags_mut() = CLIENT_ATTACHED as u64;
        let cf = ClientFileRef::create_with_client(
            Some(c.as_client_mut()),
            42,
            None,
            ClientFileData::None,
        );
        assert!(
            cf.borrow().client().is_none(),
            "attached client should be detached"
        );
        assert_eq!(cf.borrow().stream, 42);
        cf.close();
    }
}

#[test]
fn file_create_with_client_is_removed_from_the_client_tree_on_free() {
    let _guard = globals();
    unsafe {
        ensure_reactor();
        let mut c = zeroed_client();
        *c.flags_mut() = 0;
        c.as_client_mut().peer = None;
        c.as_client_mut().files = std::collections::BTreeMap::new();
        let cf = ClientFileRef::create_with_client(
            Some(c.as_client_mut()),
            7,
            None,
            ClientFileData::None,
        );
        assert!(file_find_ref(&c.as_client().files, 7).unwrap().ptr_eq(&cf));
        cf.close();
        assert!(file_find_ref(&c.as_client().files, 7).is_none());
    }
}

#[test]
fn file_free_unlinks_without_invalidating_other_strong_handles() {
    let _guard = globals();
    unsafe {
        ensure_reactor();
        let files = std::rc::Rc::new(std::cell::RefCell::new(client_files_t::new()));
        let cf = ClientFileRef::create_with_peer(
            None,
            &FileOwner::Shared(std::rc::Rc::downgrade(&files)),
            99,
            None,
            ClientFileData::None,
        );
        let held = cf.clone();
        cf.close();
        assert_eq!(held.borrow().stream, 99);
        assert!(file_find_ref(&files.borrow(), 99).is_none());
        drop(held);
        assert!(file_find_ref(&files.borrow(), 99).is_none());
    }
}

// ---------------------------------------------------------------------------
// file_can_print
// ---------------------------------------------------------------------------

#[test]
fn file_can_print_covers_null_attached_control_and_normal() {
    let _guard = globals();
    {
        assert_eq!(file_can_print(None), 0);
        let mut c = zeroed_client();
        *unsafe { c.flags_mut() } = CLIENT_ATTACHED as u64;
        assert_eq!(file_can_print(Some(&*(unsafe { c.as_client() }))), 0);
        *unsafe { c.flags_mut() } = CLIENT_CONTROL as u64;
        assert_eq!(file_can_print(Some(&*(unsafe { c.as_client() }))), 0);
        *unsafe { c.flags_mut() } = CLIENT_ATTACHED as u64 | CLIENT_CONTROL as u64;
        assert_eq!(file_can_print(Some(&*(unsafe { c.as_client() }))), 0);
        *unsafe { c.flags_mut() } = 0;
        assert_eq!(file_can_print(Some(&*(unsafe { c.as_client() }))), 1);
    }
}

// ---------------------------------------------------------------------------
// file_write_left
// ---------------------------------------------------------------------------

#[test]
fn file_write_left_is_zero_when_empty_or_event_is_none() {
    let _guard = globals();
    unsafe {
        ensure_reactor();
        let files = std::rc::Rc::new(std::cell::RefCell::new(client_files_t::new()));
        assert_eq!(file_write_left(&files.borrow()), 0);
        let cf = ClientFileRef::create_with_peer(
            None,
            &FileOwner::Shared(std::rc::Rc::downgrade(&files)),
            1,
            None,
            ClientFileData::None,
        );
        assert!(cf.borrow().event.is_none());
        assert_eq!(file_write_left(&files.borrow()), 0);
        cf.close();
        assert_eq!(file_write_left(&files.borrow()), 0);
    }
}

// ---------------------------------------------------------------------------
// file_fire_read / file_fire_done
// ---------------------------------------------------------------------------

static FIRE_READ_SEEN: AtomicI32 = AtomicI32::new(0);
static FIRE_READ_CLOSED: AtomicI32 = AtomicI32::new(-1);

fn fire_read_cb(event: ClientFileEvent<'_>) {
    FIRE_READ_SEEN.fetch_add(1, Ordering::SeqCst);
    match event {
        ClientFileEvent::Read { buffer, .. } => {
            FIRE_READ_CLOSED.store(0, Ordering::SeqCst);
            assert_eq!(buffer.len(), 5);
        }
        ClientFileEvent::Done { buffer, .. } => {
            FIRE_READ_CLOSED.store(1, Ordering::SeqCst);
            assert!(buffer.is_empty());
        }
        ClientFileEvent::CheckExit => {
            FIRE_READ_CLOSED.store(-1, Ordering::SeqCst);
        }
    }
}

#[test]
fn file_fire_read_invokes_callback_with_buffer() {
    let _guard = globals();
    unsafe {
        ensure_reactor();
        FIRE_READ_SEEN.store(0, Ordering::SeqCst);
        FIRE_READ_CLOSED.store(-1, Ordering::SeqCst);
        let files = std::rc::Rc::new(std::cell::RefCell::new(client_files_t::new()));
        let cf = ClientFileRef::create_with_peer(
            None,
            &FileOwner::Shared(std::rc::Rc::downgrade(&files)),
            50,
            Some(std::rc::Rc::new(fire_read_cb)),
            ClientFileData::None,
        );
        cf.borrow_mut().path = Some(c"test".to_owned());
        // add some bytes so buffer is non-empty (not required but exercises path)
        let msg = b"hello";
        cf.borrow_mut().buffer.as_mut().append(msg);
        assert_eq!(cf.borrow().buffer.as_ref().len(), 5);
        cf.fire_read();
        assert_eq!(FIRE_READ_SEEN.load(Ordering::SeqCst), 1);
        assert_eq!(FIRE_READ_CLOSED.load(Ordering::SeqCst), 0);
        cf.close();
    }
}

#[test]
fn file_fire_done_defers_without_crashing() {
    let _guard = globals();
    unsafe {
        ensure_reactor();
        let files = std::rc::Rc::new(std::cell::RefCell::new(client_files_t::new()));
        let cf = ClientFileRef::create_with_peer(
            None,
            &FileOwner::Shared(std::rc::Rc::downgrade(&files)),
            51,
            None,
            ClientFileData::None,
        );
        cf.fire_done();
        crate::reactor::current().run_once();
        assert!(file_find_ref(&files.borrow(), 51).is_none());
    }
}

#[test]
fn file_fire_done_only_schedules_one_callback() {
    let _guard = globals();
    unsafe {
        ensure_reactor();
        FIRE_READ_SEEN.store(0, Ordering::SeqCst);
        let files = std::rc::Rc::new(std::cell::RefCell::new(client_files_t::new()));
        let cf = ClientFileRef::create_with_peer(
            None,
            &FileOwner::Shared(std::rc::Rc::downgrade(&files)),
            52,
            Some(std::rc::Rc::new(fire_read_cb)),
            ClientFileData::None,
        );
        cf.borrow_mut().path = Some(c"test".to_owned());
        (cf.clone()).fire_done();
        cf.fire_done();
        crate::reactor::current().run_once();
        assert_eq!(FIRE_READ_SEEN.load(Ordering::SeqCst), 1);
        assert!(file_find_ref(&files.borrow(), 52).is_none());
    }
}
