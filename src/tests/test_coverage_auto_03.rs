//! Coverage for [`crate::file`] — constants and pure helpers.
//!
//! Exercises the message-number constants, [`file_find_ref`] / [`file_free`],
//! [`file_can_print`], [`file_write_left`] and the `file_fire_*` callbacks
//! through a fake client/peer map without touching the event loop's fatal
//! paths (`file_write_open` / `file_read_open` size checks, `proc_send`).

use crate::file::{
    CLIENT_ATTACHED, CLIENT_CONTROL, MSG_READ, MSG_READ_CANCEL, MSG_READ_DONE, MSG_READ_OPEN,
    MSG_WRITE, MSG_WRITE_CLOSE, MSG_WRITE_OPEN, MSG_WRITE_READY, file_can_print,
    file_create_with_client, file_create_with_peer, file_find_ref, file_fire_done, file_fire_read,
    file_free, file_write_left,
};
use crate::reactor::Buf;
use crate::reactor::Reactor;
use crate::tests::test_fixtures::{ensure_reactor, globals, zeroed_client};
use crate::types::*;
use ::core::ptr::null_mut;
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
        let mut files: client_files_t = ::std::collections::BTreeMap::new();
        assert!(file_find_ref(&raw mut files, 10).is_none());
        let cf = file_create_with_peer(
            null_mut::<tmuxpeer>(),
            &raw mut files,
            10,
            None,
            ClientFileData::None,
        );
        assert!(!cf.as_ptr().is_null());
        assert_eq!(
            file_find_ref(&raw mut files, 10).unwrap().as_ptr(),
            cf.as_ptr()
        );
        assert!(file_find_ref(&raw mut files, 11).is_none());
        file_free(cf);
        assert!(file_find_ref(&raw mut files, 10).is_none());
    }
}

#[test]
fn file_create_with_client_detaches_attached_client() {
    let _guard = globals();
    unsafe {
        ensure_reactor();
        let mut c = zeroed_client();
        c.flags = CLIENT_ATTACHED as u64;
        let cf = file_create_with_client(&raw mut *c, 42, None, ClientFileData::None);
        assert!(!cf.as_ptr().is_null());
        assert!(
            (*cf.as_ptr()).client().is_null(),
            "attached client should be detached"
        );
        assert_eq!((*cf.as_ptr()).stream, 42);
        file_free(cf);
    }
}

#[test]
fn file_create_with_client_is_removed_from_the_client_tree_on_free() {
    let _guard = globals();
    unsafe {
        ensure_reactor();
        let mut c = zeroed_client();
        c.flags = 0;
        c.peer = None;
        c.files = ::std::collections::BTreeMap::new();
        let cf = file_create_with_client(&raw mut *c, 7, None, ClientFileData::None);
        assert_eq!(
            file_find_ref(&raw mut c.files, 7).unwrap().as_ptr(),
            cf.as_ptr()
        );
        file_free(cf);
        assert!(file_find_ref(&raw mut c.files, 7).is_none());
    }
}

#[test]
fn file_free_unlinks_without_invalidating_other_strong_handles() {
    let _guard = globals();
    unsafe {
        ensure_reactor();
        let mut files: client_files_t = ::std::collections::BTreeMap::new();
        let cf = file_create_with_peer(
            null_mut::<tmuxpeer>(),
            &raw mut files,
            99,
            None,
            ClientFileData::None,
        );
        let held = cf.clone();
        file_free(cf);
        assert_eq!((*held.as_ptr()).stream, 99);
        assert!(file_find_ref(&raw mut files, 99).is_none());
        drop(held);
        assert!(file_find_ref(&raw mut files, 99).is_none());
    }
}

// ---------------------------------------------------------------------------
// file_can_print
// ---------------------------------------------------------------------------

#[test]
fn file_can_print_covers_null_attached_control_and_normal() {
    let _guard = globals();
    unsafe {
        assert_eq!(file_can_print(null_mut::<client>()), 0);
        let mut c = zeroed_client();
        c.flags = CLIENT_ATTACHED as u64;
        assert_eq!(file_can_print(&raw mut *c), 0);
        c.flags = CLIENT_CONTROL as u64;
        assert_eq!(file_can_print(&raw mut *c), 0);
        c.flags = CLIENT_ATTACHED as u64 | CLIENT_CONTROL as u64;
        assert_eq!(file_can_print(&raw mut *c), 0);
        c.flags = 0;
        assert_eq!(file_can_print(&raw mut *c), 1);
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
        let mut files: client_files_t = ::std::collections::BTreeMap::new();
        assert_eq!(file_write_left(&raw mut files), 0);
        let cf = file_create_with_peer(
            null_mut::<tmuxpeer>(),
            &raw mut files,
            1,
            None,
            ClientFileData::None,
        );
        assert!((*cf.as_ptr()).event.is_none());
        assert_eq!(file_write_left(&raw mut files), 0);
        file_free(cf);
        assert_eq!(file_write_left(&raw mut files), 0);
    }
}

// ---------------------------------------------------------------------------
// file_fire_read / file_fire_done
// ---------------------------------------------------------------------------

static FIRE_READ_SEEN: AtomicI32 = AtomicI32::new(0);
static FIRE_READ_CLOSED: AtomicI32 = AtomicI32::new(-1);

unsafe fn fire_read_cb(
    _c: *mut client,
    _path: *const ::core::ffi::c_char,
    _error: ::core::ffi::c_int,
    closed: ::core::ffi::c_int,
    buffer: *mut Buf,
    _data: ClientFileData,
) {
    FIRE_READ_SEEN.fetch_add(1, Ordering::SeqCst);
    FIRE_READ_CLOSED.store(closed, Ordering::SeqCst);
    // buffer should be the file's buffer
    assert!(!buffer.is_null());
}

#[test]
fn file_fire_read_invokes_callback_with_buffer() {
    let _guard = globals();
    unsafe {
        ensure_reactor();
        FIRE_READ_SEEN.store(0, Ordering::SeqCst);
        FIRE_READ_CLOSED.store(-1, Ordering::SeqCst);
        let mut files: client_files_t = ::std::collections::BTreeMap::new();
        let cf = file_create_with_peer(
            null_mut::<tmuxpeer>(),
            &raw mut files,
            50,
            Some(fire_read_cb),
            ClientFileData::None,
        );
        // add some bytes so buffer is non-empty (not required but exercises path)
        let msg = b"hello";
        (*cf.as_ptr()).buffer.as_mut().append(msg);
        assert_eq!((*cf.as_ptr()).buffer.as_ref().len(), 5);
        file_fire_read(&cf);
        assert_eq!(FIRE_READ_SEEN.load(Ordering::SeqCst), 1);
        assert_eq!(FIRE_READ_CLOSED.load(Ordering::SeqCst), 0);
        file_free(cf);
    }
}

#[test]
fn file_fire_done_defers_without_crashing() {
    let _guard = globals();
    unsafe {
        ensure_reactor();
        let mut files: client_files_t = ::std::collections::BTreeMap::new();
        let cf = file_create_with_peer(
            null_mut::<tmuxpeer>(),
            &raw mut files,
            51,
            None,
            ClientFileData::None,
        );
        file_fire_done(cf);
        crate::reactor::current().run_once();
        assert!(file_find_ref(&raw mut files, 51).is_none());
    }
}

#[test]
fn file_fire_done_only_schedules_one_callback() {
    let _guard = globals();
    unsafe {
        ensure_reactor();
        FIRE_READ_SEEN.store(0, Ordering::SeqCst);
        let mut files: client_files_t = ::std::collections::BTreeMap::new();
        let cf = file_create_with_peer(
            null_mut::<tmuxpeer>(),
            &raw mut files,
            52,
            Some(fire_read_cb),
            ClientFileData::None,
        );
        file_fire_done(cf.clone());
        file_fire_done(cf);
        crate::reactor::current().run_once();
        assert_eq!(FIRE_READ_SEEN.load(Ordering::SeqCst), 1);
        assert!(file_find_ref(&raw mut files, 52).is_none());
    }
}
