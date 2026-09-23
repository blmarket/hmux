//! Frozen pre-migration sizes, alignments, and every named field offset.
use std::mem::{align_of, offset_of, size_of};
#[test]
fn original_copies_match() {
    let mut records = Vec::new();
    macro_rules! record {
        ($label:literal, $ty:ty, [$($field:ident),*]) => {
            records.push(format!("{} {} {} {:?}", $label, size_of::<$ty>(), align_of::<$ty>(),
                &[$(offset_of!($ty, $field)),*] as &[usize]));
        };
    }
    record!(
        "src/cmd_queue.rs::event_payload",
        *mut hmux2::src::cmd_queue::event_payload,
        []
    );
    record!(
        "src/cmd_rename_session.rs::event_payload",
        *mut hmux2::src::cmd_rename_session::event_payload,
        []
    );
    record!(
        "src/cmd_select_pane.rs::event_payload",
        *mut hmux2::src::cmd_select_pane::event_payload,
        []
    );
    record!(
        "src/cmd_set_option.rs::event_payload",
        *mut hmux2::src::cmd_set_option::event_payload,
        []
    );
    record!(
        "src/cmd_split_window.rs::event_payload",
        *mut hmux2::src::cmd_split_window::event_payload,
        []
    );
    record!(
        "src/cmd_wait_for.rs::event_payload",
        *mut hmux2::src::cmd_wait_for::event_payload,
        []
    );
    record!(
        "src/control_notify.rs::event_payload",
        *mut hmux2::src::control_notify::event_payload,
        []
    );
    record!(
        "src/events.rs::event_payload",
        *mut hmux2::src::events::event_payload,
        []
    );
    record!(
        "src/events_payload.rs::event_payload",
        hmux2::src::events_payload::event_payload,
        [items, target]
    );
    record!(
        "src/hooks.rs::event_payload",
        *mut hmux2::src::hooks::event_payload,
        []
    );
    record!(
        "src/input.rs::event_payload",
        *mut hmux2::src::input::event_payload,
        []
    );
    record!(
        "src/paste.rs::event_payload",
        *mut hmux2::src::paste::event_payload,
        []
    );
    record!(
        "src/resize.rs::event_payload",
        *mut hmux2::src::resize::event_payload,
        []
    );
    record!(
        "src/server_client.rs::event_payload",
        *mut hmux2::src::server_client::event_payload,
        []
    );
    record!(
        "src/server_fn.rs::event_payload",
        *mut hmux2::src::server_fn::event_payload,
        []
    );
    record!(
        "src/session.rs::event_payload",
        *mut hmux2::src::session::event_payload,
        []
    );
    record!(
        "src/spawn.rs::event_payload",
        *mut hmux2::src::spawn::event_payload,
        []
    );
    record!(
        "src/window.rs::event_payload",
        *mut hmux2::src::window::event_payload,
        []
    );
    record!(
        "src/cmd_queue.rs::event_payload_free_cb",
        hmux2::src::cmd_queue::event_payload_free_cb,
        []
    );
    record!(
        "src/events_payload.rs::event_payload_free_cb",
        hmux2::src::events_payload::event_payload_free_cb,
        []
    );
    record!(
        "src/hooks.rs::event_payload_free_cb",
        hmux2::src::hooks::event_payload_free_cb,
        []
    );
    record!(
        "src/cmd_wait_for.rs::event_payload_item",
        *mut hmux2::src::cmd_wait_for::event_payload_item,
        []
    );
    record!(
        "src/events_payload.rs::event_payload_item",
        hmux2::src::events_payload::event_payload_item,
        [name, type_0, c2rust_unnamed, entry]
    );
    record!(
        "src/events_payload.rs::C2RustUnnamed_36",
        hmux2::src::events_payload::event_payload_item_c2rust_unnamed,
        [
            string,
            time,
            number,
            unsigned_number,
            client,
            session,
            window,
            pane,
            pointer
        ]
    );
    record!(
        "src/events_payload.rs::C2RustUnnamed_37",
        hmux2::src::events_payload::event_payload_item_c2rust_unnamed_pointer,
        [ptr, free_cb, print_cb]
    );
    record!(
        "src/events_payload.rs::C2RustUnnamed_35",
        hmux2::src::events_payload::event_payload_item_entry,
        [rbe_left, rbe_right, rbe_parent, rbe_color]
    );
    record!(
        "src/cmd_queue.rs::event_payload_print_cb",
        hmux2::src::cmd_queue::event_payload_print_cb,
        []
    );
    record!(
        "src/events_payload.rs::event_payload_print_cb",
        hmux2::src::events_payload::event_payload_print_cb,
        []
    );
    record!(
        "src/hooks.rs::event_payload_print_cb",
        hmux2::src::hooks::event_payload_print_cb,
        []
    );
    record!(
        "src/events_payload.rs::event_payload_tree",
        hmux2::src::events_payload::event_payload_tree,
        [entries]
    );
    record!(
        "src/events_payload.rs::event_payload_type",
        hmux2::src::events_payload::event_payload_type,
        []
    );
    record!(
        "src/cmd_wait_for.rs::events_cb",
        hmux2::src::cmd_wait_for::events_cb,
        []
    );
    record!(
        "src/control_notify.rs::events_cb",
        hmux2::src::control_notify::events_cb,
        []
    );
    record!(
        "src/events.rs::events_cb",
        hmux2::src::events::events_cb,
        []
    );
    record!("src/hooks.rs::events_cb", hmux2::src::hooks::events_cb, []);
    record!(
        "src/cmd_wait_for.rs::events_sink",
        *mut hmux2::src::cmd_wait_for::events_sink,
        []
    );
    record!(
        "src/control_notify.rs::events_sink",
        *mut hmux2::src::control_notify::events_sink,
        []
    );
    record!(
        "src/events.rs::events_sink",
        hmux2::src::events::events_sink,
        [name, cb, data, dead, generation]
    );
    record!(
        "src/hooks.rs::events_sink",
        *mut hmux2::src::hooks::events_sink,
        []
    );
    let actual = records.join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/remaining-events.txt"));
}
