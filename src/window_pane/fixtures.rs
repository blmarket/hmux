//! Narrow setup and component access for unit tests outside the pane owner.
//! These operations never lend a pane record. Fixture cleanup remains explicit.

use super::*;
use crate::src::shared::colour::colour_palette;
use crate::src::shared::prompt::prompt;

pub(crate) trait PaneFixture {
    unsafe fn fixture_id(&self, id: u32);
    unsafe fn fixture_register(&self);
    unsafe fn fixture_parent(&self, window: Option<&WindowRef>);
    unsafe fn fixture_geometry(&self, size: (u32, u32), offset: (i32, i32));
    unsafe fn fixture_scrollbar(&self, width: i32, pad: i32);
    unsafe fn fixture_flags(&self) -> i32;
    unsafe fn fixture_set_flags(&self, flags: i32);
    unsafe fn fixture_set_options(&self, options: Box<options>);
    unsafe fn fixture_take_options(&self) -> Option<Box<options>>;
    /// Only component operations may run here. End the scope before querying
    /// models, formatting, dispatching callbacks or replacing pane screens.
    unsafe fn fixture_base<R>(&self, edit: impl FnOnce(&mut screen) -> R) -> R;
    unsafe fn fixture_palette<R>(&self, edit: impl FnOnce(&mut colour_palette) -> R) -> R;
    unsafe fn fixture_sync_dirty<R>(&self, read: impl FnOnce(Option<&[u8]>, u32) -> R) -> R;
    unsafe fn fixture_add_mode(&self, entry: refbox::RefBox<window_mode_entry>);
    unsafe fn fixture_take_mode(&self) -> Option<refbox::RefBox<window_mode_entry>>;
    /// The caller must explicitly run required cleanup callbacks first. Entries
    /// with no resource cleanup callback may be cleared directly.
    unsafe fn fixture_clear_modes(&self);
    unsafe fn fixture_prompt(&self, prompt: refbox::RefBox<prompt>, x: u32);
    unsafe fn fixture_stream(&self, stream: crate::src::reactor::StreamHandle);
    unsafe fn fixture_take_stream(&self) -> crate::src::reactor::StreamHandle;
}

impl PaneFixture for Rc<UnsafeCell<window_pane>> {
    unsafe fn fixture_id(&self, id: u32) {
        (*self.get()).id = id;
    }

    unsafe fn fixture_register(&self) {
        assert!(window_pane_tree_insert(&mut all_window_panes, self.clone()).is_none());
    }

    unsafe fn fixture_parent(&self, window: Option<&WindowRef>) {
        (*self.get()).window = window.map_or_else(std::rc::Weak::new, Rc::downgrade);
    }

    unsafe fn fixture_geometry(&self, size: (u32, u32), offset: (i32, i32)) {
        let pane = &mut *self.get();
        (pane.sx, pane.sy) = size;
        (pane.xoff, pane.yoff) = offset;
    }

    unsafe fn fixture_scrollbar(&self, width: i32, pad: i32) {
        let pane = &mut *self.get();
        (pane.scrollbar_style.width, pane.scrollbar_style.pad) = (width, pad);
    }

    unsafe fn fixture_flags(&self) -> i32 {
        (*self.get()).flags
    }

    unsafe fn fixture_set_flags(&self, flags: i32) {
        (*self.get()).flags = flags;
    }

    unsafe fn fixture_set_options(&self, options: Box<options>) {
        let pane = &mut *self.get();
        assert!(
            pane.options.is_none(),
            "fixture must retire old options first"
        );
        pane.options = Some(options);
    }

    unsafe fn fixture_take_options(&self) -> Option<Box<options>> {
        (*self.get()).options.take()
    }

    unsafe fn fixture_base<R>(&self, edit: impl FnOnce(&mut screen) -> R) -> R {
        edit(&mut (*self.get()).base)
    }

    unsafe fn fixture_palette<R>(&self, edit: impl FnOnce(&mut colour_palette) -> R) -> R {
        edit(&mut (*self.get()).palette)
    }

    unsafe fn fixture_sync_dirty<R>(&self, read: impl FnOnce(Option<&[u8]>, u32) -> R) -> R {
        let pane = &*self.get();
        read(pane.sync_dirty.as_deref(), pane.sync_dirty_size)
    }

    unsafe fn fixture_add_mode(&self, entry: refbox::RefBox<window_mode_entry>) {
        (*self.get()).modes.push(entry);
    }

    unsafe fn fixture_take_mode(&self) -> Option<refbox::RefBox<window_mode_entry>> {
        (*self.get()).modes.pop()
    }

    unsafe fn fixture_clear_modes(&self) {
        let pane = &mut *self.get();
        pane.modes.clear();
        pane.screen_source = PaneScreenSource::Base;
    }

    unsafe fn fixture_prompt(&self, prompt: refbox::RefBox<prompt>, x: u32) {
        let pane = &mut *self.get();
        assert!(
            pane.prompt.is_none(),
            "fixture must retire old prompt first"
        );
        pane.prompt = Some(prompt);
        pane.prompt_cx = x;
    }

    unsafe fn fixture_stream(&self, stream: crate::src::reactor::StreamHandle) {
        let pane = &mut *self.get();
        assert!(
            !pane.event.is_alive(),
            "fixture must close old stream first"
        );
        pane.fd = -1;
        pane.pipe_fd = -1;
        pane.event = stream;
    }

    unsafe fn fixture_take_stream(&self) -> crate::src::reactor::StreamHandle {
        std::mem::take(&mut (*self.get()).event)
    }
}
