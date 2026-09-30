//! Unit fixture setup through retained clients. Cleanup remains explicit.
use super::*;

pub(crate) trait ClientFixture {
    unsafe fn fixture_with_session(session: Option<&SessionRef>) -> Self
    where
        Self: Sized;
    unsafe fn fixture_with_control(name: Option<&CStr>, session: Option<&SessionRef>) -> Self
    where
        Self: Sized;
    unsafe fn fixture_with_names(name: Option<&CStr>, tty_name: Option<&CStr>) -> Self
    where
        Self: Sized;
    unsafe fn fixture_with_queue() -> Self
    where
        Self: Sized;
    unsafe fn fixture_activity(&self, seconds: u64, micros: u64);
}

impl ClientFixture for ClientRef {
    unsafe fn fixture_with_session(session: Option<&SessionRef>) -> Self {
        client::with_session_for_test(session)
    }
    unsafe fn fixture_with_control(name: Option<&CStr>, session: Option<&SessionRef>) -> Self {
        client::with_control_for_test(name, session)
    }
    unsafe fn fixture_with_names(name: Option<&CStr>, tty_name: Option<&CStr>) -> Self {
        client::with_names_for_test(name, tty_name)
    }
    unsafe fn fixture_with_queue() -> Self {
        client::with_queue_for_test()
    }
    unsafe fn fixture_activity(&self, seconds: u64, micros: u64) {
        client::activity_for_test(self, seconds, micros)
    }
}
