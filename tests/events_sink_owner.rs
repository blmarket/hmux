use hmux2::src::events::{events_add_sink, events_fire, events_remove_sink};
use hmux2::src::events_payload::event_payload_create;
use hmux2::src::shared::events::{event_payload, events_sink};
use std::ffi::{c_char, c_void};

struct State {
    first: *mut events_sink,
    second: *mut events_sink,
    later: *mut events_sink,
    calls: Vec<u8>,
}

unsafe extern "C" fn first(_: *const c_char, _: *mut event_payload, data: *mut c_void) {
    let state = &mut *data.cast::<State>();
    state.calls.push(b'A');
    events_remove_sink(state.first);
    assert_eq!((*state.first).dead, 1);
    events_remove_sink(state.second);
    state.later = events_add_sink(c"owner-event".as_ptr(), Some(later), data);
}

unsafe extern "C" fn second(_: *const c_char, _: *mut event_payload, data: *mut c_void) {
    (*data.cast::<State>()).calls.push(b'B');
}

unsafe extern "C" fn later(_: *const c_char, _: *mut event_payload, data: *mut c_void) {
    (*data.cast::<State>()).calls.push(b'C');
}

#[test]
fn sink_owns_its_name_and_defers_removal_until_dispatch_ends() {
    unsafe {
        let mut state = State {
            first: std::ptr::null_mut(),
            second: std::ptr::null_mut(),
            later: std::ptr::null_mut(),
            calls: Vec::new(),
        };
        let data = (&raw mut state).cast();
        let mut name = b"owner-event\0".to_vec();
        state.first = events_add_sink(name.as_ptr().cast(), Some(first), data);
        state.second = events_add_sink(name.as_ptr().cast(), Some(second), data);

        // The caller's name buffer may disappear or change after registration.
        name[0] = b'X';
        events_fire(c"owner-event".as_ptr(), event_payload_create());
        assert_eq!(state.calls, b"A");

        events_fire(c"owner-event".as_ptr(), event_payload_create());
        assert_eq!(state.calls, b"AC");

        events_remove_sink(state.later);
    }
}
