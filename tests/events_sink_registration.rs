use hmux::src::events::{events_add_sink, events_fire, events_remove_sink};
use hmux::src::events_payload::event_payload_create;
use hmux::src::shared::events::{events_callback, EventSinkId};
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn sink_ids_survive_self_removal_and_do_not_remove_replacements() {
    unsafe {
        let first_id = Rc::new(Cell::new(EventSinkId::default()));
        let second_id = Rc::new(Cell::new(EventSinkId::default()));
        let first_calls = Rc::new(Cell::new(0));
        let second_calls = Rc::new(Cell::new(0));
        let name = c"sink-id-lifecycle";

        let self_id = first_id.clone();
        let replacement_id = second_id.clone();
        let first_count = first_calls.clone();
        let second_count = second_calls.clone();
        first_id.set(events_add_sink(
            name,
            events_callback(move |_, _| {
                first_count.set(first_count.get() + 1);
                events_remove_sink(self_id.get());
                let count = second_count.clone();
                let id = events_add_sink(
                    c"sink-id-lifecycle",
                    events_callback(move |_, _| count.set(count.get() + 1)),
                );
                replacement_id.set(id);
            }),
        ));

        events_fire(&*name, event_payload_create());
        assert_eq!(first_calls.get(), 1);
        assert_eq!(second_calls.get(), 0);
        assert_ne!(first_id.get(), second_id.get());

        events_remove_sink(first_id.get());
        events_fire(&*name, event_payload_create());
        assert_eq!(first_calls.get(), 1);
        assert_eq!(second_calls.get(), 1);

        events_remove_sink(second_id.get());
        events_remove_sink(EventSinkId::default());
        events_fire(&*name, event_payload_create());
        assert_eq!(second_calls.get(), 1);
    }
}
