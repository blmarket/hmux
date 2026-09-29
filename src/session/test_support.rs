//! Minimal private-state fixture operations for other modules' behavior tests.
//! No production API or raw Session/component view is exposed.
use super::*;

pub(crate) unsafe fn metadata(
    owner: &Rc<UnsafeCell<session>>,
    name: Option<CString>,
    id: Option<u32>,
    attached: Option<u32>,
) {
    let state = &mut *owner.get();
    if let Some(name) = name {
        state.name = name;
    }
    if let Some(id) = id {
        state.id = id;
    }
    if let Some(attached) = attached {
        state.attached = attached;
    }
}
pub(crate) unsafe fn current(owner: &Rc<UnsafeCell<session>>, link: refbox::Weak<winlink>) {
    (*owner.get()).curw = link;
}
pub(crate) unsafe fn add_link(
    owner: &Rc<UnsafeCell<session>>,
    index: i32,
) -> refbox::Weak<winlink> {
    let mut link = winlink_add(&mut (*owner.get()).windows, index);
    link.get_mut_unchecked().session = Rc::downgrade(owner);
    link
}
pub(crate) unsafe fn remove_link(owner: &Rc<UnsafeCell<session>>, link: refbox::Weak<winlink>) {
    winlink_remove(&raw mut (*owner.get()).windows, link);
}
pub(crate) unsafe fn reindex(
    owner: &Rc<UnsafeCell<session>>,
    link: refbox::Weak<winlink>,
    index: i32,
) {
    crate::src::window::winlinks_reindex(&mut (*owner.get()).windows, link, index);
}

pub(crate) unsafe fn activity(owner: &Rc<UnsafeCell<session>>, time: timeval) {
    (*owner.get()).activity_time = time;
}
