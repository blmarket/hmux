//! Unit fixture setup through retained windows. Cleanup remains explicit.
use super::*;

pub(crate) trait WindowFixture {
    fn fixture_with_id(id: u32) -> Self
    where
        Self: Sized;
    /// Geometry-only setup without layout or resize callbacks.
    fn fixture_with_size(sx: u32, sy: u32) -> Self
    where
        Self: Sized;
    fn fixture_with_options() -> Self
    where
        Self: Sized;
    unsafe fn fixture_zoomed() -> Self
    where
        Self: Sized;
    /// Deliberately change visibility without repairing the layout.
    unsafe fn fixture_set_zoomed(&self, enabled: bool);
}

impl WindowFixture for WindowRef {
    fn fixture_with_id(id: u32) -> Self {
        window::with_id_for_test(id)
    }
    fn fixture_with_size(sx: u32, sy: u32) -> Self {
        window::with_size_for_test(sx, sy)
    }
    fn fixture_with_options() -> Self {
        window::with_options_for_test()
    }
    unsafe fn fixture_zoomed() -> Self {
        super::test_support::zoomed_window()
    }
    unsafe fn fixture_set_zoomed(&self, enabled: bool) {
        super::test_support::set_zoomed(self, enabled);
    }
}
