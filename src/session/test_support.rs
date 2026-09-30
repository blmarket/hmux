//! Narrow setup operations for behavior tests outside the Session owner.
//! Fixture resources retain their explicit cleanup requirements.
use super::*;
use crate::src::shared::session::SessionRef;
use std::time::SystemTime;

pub(crate) trait SessionFixture {
    unsafe fn fixture_metadata(
        &self,
        name: Option<CString>,
        id: Option<u32>,
        attached: Option<u32>,
    );
    unsafe fn fixture_current(&self, link: refbox::Weak<winlink>);
    unsafe fn fixture_add_link(&self, index: i32) -> refbox::Weak<winlink>;
    unsafe fn fixture_remove_link(&self, link: refbox::Weak<winlink>);
    unsafe fn fixture_reindex(&self, link: refbox::Weak<winlink>, index: i32);
    unsafe fn fixture_activity(&self, time: SystemTime);
    unsafe fn fixture_environment(&self, environment: Option<Box<environ>>);
}

impl SessionFixture for SessionRef {
    unsafe fn fixture_metadata(
        &self,
        name: Option<CString>,
        id: Option<u32>,
        attached: Option<u32>,
    ) {
        let state = &mut *self.get();
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
    unsafe fn fixture_current(&self, link: refbox::Weak<winlink>) {
        (*self.get()).curw = link;
    }
    unsafe fn fixture_add_link(&self, index: i32) -> refbox::Weak<winlink> {
        let mut link = winlink_add(&mut (*self.get()).windows, index);
        link.get_mut_unchecked().session = Rc::downgrade(self);
        link
    }
    unsafe fn fixture_remove_link(&self, link: refbox::Weak<winlink>) {
        winlink_remove(&raw mut (*self.get()).windows, link);
    }
    unsafe fn fixture_reindex(&self, link: refbox::Weak<winlink>, index: i32) {
        crate::src::window::winlinks_reindex(&mut (*self.get()).windows, link, index);
    }
    unsafe fn fixture_activity(&self, time: SystemTime) {
        (*self.get()).activity_time = time;
    }
    unsafe fn fixture_environment(&self, environment: Option<Box<environ>>) {
        (*self.get()).environ = environment;
    }
}
