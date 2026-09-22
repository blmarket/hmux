use hmux2::src::events::{events_add_sink, events_remove_sink};
use hmux2::src::events_payload::event_payload_get_string;
use hmux2::src::shared::events::event_payload;
use hmux2::src::window::{window, window_set_name};
use hmux2::src::xmalloc::xstrdup;
use std::ffi::{c_char, c_void, CStr};

struct RenameState {
    window: *mut window,
    events: Vec<(Vec<u8>, Vec<u8>)>,
    reenter: bool,
}

unsafe extern "C" fn on_rename(_: *const c_char, payload: *mut event_payload, data: *mut c_void) {
    let state = data.cast::<RenameState>();
    let old = CStr::from_ptr(event_payload_get_string(payload, c"old_name".as_ptr()))
        .to_bytes()
        .to_vec();
    let new = CStr::from_ptr(event_payload_get_string(payload, c"new_name".as_ptr()))
        .to_bytes()
        .to_vec();
    (*state).events.push((old, new));
    if (*state).reenter {
        (*state).reenter = false;
        window_set_name((*state).window, c"inner".as_ptr(), 0);
    }
}

#[test]
fn rename_keeps_old_name_through_reentrant_notification() {
    unsafe {
        // No session owns this fixture window, so the event payload is its only
        // temporary reference. Keep one reference to prevent window_destroy.
        let mut w: window = std::mem::zeroed();
        w.name = xstrdup(c"before".as_ptr());
        w.references = 1;
        let mut state = RenameState {
            window: &raw mut w,
            events: Vec::new(),
            reenter: true,
        };
        let sink = events_add_sink(
            c"window-renamed".as_ptr(),
            Some(on_rename),
            (&raw mut state).cast(),
        );

        window_set_name(&raw mut w, c"outer".as_ptr(), 0);
        assert_eq!(
            state.events,
            vec![
                (b"before".to_vec(), b"outer".to_vec()),
                (b"outer".to_vec(), b"inner".to_vec()),
            ]
        );
        assert_eq!(CStr::from_ptr(w.name), c"inner");
        assert_eq!(w.references, 1);

        // Invalid UTF-8 must leave the name and notification count unchanged.
        window_set_name(&raw mut w, c"\xff".as_ptr(), 0);
        assert_eq!(CStr::from_ptr(w.name), c"inner");
        assert_eq!(state.events.len(), 2);

        events_remove_sink(sink);
        libc::free(w.name.cast());
    }
}
