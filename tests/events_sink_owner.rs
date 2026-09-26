use hmux2::src::events::{events_add_sink, events_fire, events_remove_sink};
use hmux2::src::events_payload::event_payload_create;
use hmux2::src::shared::events::{events_callback, events_sink};
use std::ffi::CStr;

struct State {
    first: *mut events_sink,
    second: *mut events_sink,
    later: *mut events_sink,
    calls: Vec<u8>,
}

#[test]
fn sink_owns_its_name_and_defers_removal_until_dispatch_ends() {
    unsafe {
        let state = std::rc::Rc::new(std::cell::RefCell::new(State {
            first: std::ptr::null_mut(),
            second: std::ptr::null_mut(),
            later: std::ptr::null_mut(),
            calls: Vec::new(),
        }));
        let mut name = b"owner-event\0".to_vec();
        let first_state = state.clone();
        state.borrow_mut().first = events_add_sink(
            CStr::from_ptr(name.as_ptr().cast()),
            events_callback(move |_, _| {
                let (first, second) = {
                    let mut state = first_state.borrow_mut();
                    state.calls.push(b'A');
                    (state.first, state.second)
                };
                {
                    events_remove_sink(first);
                    assert_eq!((*first).dead, 1);
                    events_remove_sink(second);
                }
                let later_state = first_state.clone();
                let later = {
                    events_add_sink(
                        c"owner-event",
                        events_callback(move |_, _| later_state.borrow_mut().calls.push(b'C')),
                    )
                };
                first_state.borrow_mut().later = later;
            }),
        );
        let second_state = state.clone();
        state.borrow_mut().second = events_add_sink(
            CStr::from_ptr(name.as_ptr().cast()),
            events_callback(move |_, _| second_state.borrow_mut().calls.push(b'B')),
        );

        // The caller's name buffer may disappear or change after registration.
        name[0] = b'X';
        events_fire(c"owner-event".as_ptr(), event_payload_create());
        assert_eq!(state.borrow().calls, b"A");

        events_fire(c"owner-event".as_ptr(), event_payload_create());
        assert_eq!(state.borrow().calls, b"AC");

        events_remove_sink(state.borrow().later);
    }
}
