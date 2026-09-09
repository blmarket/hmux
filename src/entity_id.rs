use std::cell::Cell;
use std::thread::LocalKey;

/// Allocates an entity identity once, including the final representable value.
/// Exhaustion stays recorded so stale observations can never resolve to a new
/// entity through counter wraparound.
pub(crate) fn next_entity_id(counter: &'static LocalKey<Cell<Option<u32>>>) -> u32 {
    try_next_entity_id(counter).expect("entity IDs exhausted")
}

pub(crate) fn try_next_entity_id(counter: &'static LocalKey<Cell<Option<u32>>>) -> Option<u32> {
    let id = counter.get()?;
    counter.set(id.checked_add(1));
    Some(id)
}
