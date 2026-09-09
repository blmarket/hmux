use crate::args::RustArguments;
#[cfg(test)]
use crate::PaneCommandState;
use crate::grid::Grid;
use crate::screen::Screen;
use crate::text::{RustUtf8VisModel, Utf8VisModel};
use crate::types::*;
use crate::window::{PANE_REDRAW, PANE_STYLECHANGED, PANE_THEMECHANGED};
use crate::{PaneGeometryState, WindowPane};
use std::ffi::{CStr, CString};

impl RustWindowPaneWeak {
    /// # Safety
    /// Exclude conflicting pane access while taking this owned snapshot.
    pub(crate) unsafe fn geometry(&self) -> Option<crate::PaneGeometry> {
        let owner = self.upgrade()?;
        let pane = unsafe { owner.as_pane() };
        Some(pane.geometry())
    }

    /// # Safety
    /// Exclude conflicting pane access while taking this owned snapshot.
    #[cfg(test)]
    pub(crate) unsafe fn command(&self) -> Option<crate::PaneCommand> {
        let owner = self.upgrade()?;
        let pane = unsafe { owner.as_pane() };
        Some(pane.pane_command())
    }

    /// # Safety
    /// Exclude conflicting pane access while taking this owned snapshot.
    pub(crate) unsafe fn options(&self) -> Option<RustOptionsRef> {
        let owner = self.upgrade()?;
        let pane = unsafe { owner.as_pane() };
        Some(pane.options_ref().clone())
    }
}

impl RustWindowPaneWeak {
    /// # Safety
    /// Exclude other pane flag access during the update.
    pub(crate) unsafe fn add_flags(&self, flags: core::ffi::c_int) -> bool {
        let Some(mut owner) = self.upgrade() else {
            return false;
        };
        unsafe { *owner.as_pane_mut().flags_mut() |= flags };
        true
    }

    /// # Safety
    /// Exclude other pane flag access during the update.
    pub(crate) unsafe fn remove_flags(&self, flags: core::ffi::c_int) -> bool {
        let Some(mut owner) = self.upgrade() else {
            return false;
        };
        unsafe { *owner.as_pane_mut().flags_mut() &= !flags };
        true
    }

    /// # Safety
    /// Exclude mutation or destruction of this pane during client cleanup.
    pub(crate) unsafe fn remove_from_clients(&self) -> bool {
        if !self.is_alive() {
            return false;
        };
        unsafe { crate::server::server_client_remove_pane(self.get().expect("live pane")) };
        true
    }

    /// # Safety
    /// Exclude conflicting pane access during mode destruction and its callbacks.
    pub(crate) unsafe fn reset_modes(&self) -> bool {
        let mut observed = self.clone();
        let Some(pane) = (unsafe { observed.get_mut() }) else {
            return false;
        };
        unsafe { crate::window::window_pane_reset_mode_all(pane) };
        true
    }

    /// # Safety
    /// Exclude conflicting pane, mode, and layout access during this transition.
    pub(crate) unsafe fn set_mode(
        &self,
        source: Option<RustWindowPaneWeak>,
        mode: WindowMode,
        target: Option<&cmd_find_state>,
        args: Option<&RustArguments>,
    ) -> Option<core::ffi::c_int> {
        let mut observed = self.clone();
        let pane = unsafe { observed.get_mut()? };
        Some(unsafe { crate::window::window_pane_set_mode(pane, source, mode, target, args) })
    }

    /// # Safety
    /// Exclude other pane geometry access during this update.
    #[cfg(test)]
    pub(crate) unsafe fn set_position(&self, x: core::ffi::c_int, y: core::ffi::c_int) -> bool {
        let Some(mut owner) = self.upgrade() else {
            return false;
        };
        unsafe { owner.as_pane_mut().set_position(x, y) };
        true
    }

    /// # Safety
    /// Exclude mutable pane access while constructing the notification.
    pub(crate) unsafe fn notify(&self, name: &CStr) -> bool {
        if !self.is_alive() {
            return false;
        };
        unsafe { crate::notify::notify_pane(name, self.get()) };
        true
    }

    /// # Safety
    /// Exclude mutable pane access while querying process state.
    pub(crate) unsafe fn has_exited(&self) -> Option<bool> {
        let owner = self.upgrade()?;
        Some(unsafe { crate::window::window_pane_exited(owner.as_pane()) != 0 })
    }
}

pub(crate) enum PaneDirection {
    Left,
    Right,
    Up,
    Down,
}

impl RustWindowPaneWeak {
    /// # Safety
    /// Exclude mutation of pane membership, layout, and geometry during this search.
    pub(crate) unsafe fn neighbor(&self, direction: PaneDirection) -> Option<Self> {
        unsafe {
            let owner = self.upgrade()?;
            let pane = Some(owner.as_pane());
            match direction {
                PaneDirection::Left => crate::window::window_pane_find_left(pane),
                PaneDirection::Right => crate::window::window_pane_find_right(pane),
                PaneDirection::Up => crate::window::window_pane_find_up(pane),
                PaneDirection::Down => crate::window::window_pane_find_down(pane),
            }
        }
    }
}

impl RustWindowPaneWeak {
    /// # Safety
    /// Exclude conflicting screen access while replacing the title.
    pub(crate) unsafe fn set_title(&self, title: &CStr) -> Option<bool> {
        let mut owner = self.upgrade()?;
        Some(unsafe { owner.as_pane_mut().base_mut().set_title(title, 0) != 0 })
    }

    /// # Safety
    /// Exclude conflicting palette and input access while resetting the terminal.
    pub(crate) unsafe fn reset_terminal(&self) -> bool {
        use crate::style::ColourEngine;
        let context = {
            let Some(mut owner) = self.upgrade() else {
                return false;
            };
            let pane = unsafe { owner.as_pane_mut() };
            crate::style::RustColourEngine.clear_palette(Some(pane.palette_mut()));
            pane.ictx().clone()
        };
        unsafe {
            context.as_ref().expect("pane input initialized").reset(1);
            self.add_flags(PANE_STYLECHANGED | PANE_THEMECHANGED | PANE_REDRAW);
        }
        true
    }
}

/// Sends one line to the pane's output stream: `-S` hands the bytes over as
/// they are, and without it `utf8_stravisx` makes control bytes visible.
pub(crate) fn send_line(output: crate::reactor::Stream, line: &[u8], raw: bool) {
    if raw {
        output.write(line);
    } else {
        let visible = RustUtf8VisModel.encode_utf8(line, 0x20 | 0x40);
        output.write(visible.as_bytes());
    }
}

impl RustWindowPaneWeak {
    /// # Safety
    /// Exclude conflicting pane and stream access while sending the paste.
    pub(crate) unsafe fn paste_buffer(
        &self,
        bytes: &[u8],
        separator: &[u8],
        raw: bool,
        bracket: bool,
    ) -> bool {
        let Some(owner) = self.upgrade() else {
            return false;
        };
        let pane = unsafe { owner.as_pane() };
        if *pane.flags() & crate::window::PANE_INPUTOFF != 0 {
            return true;
        }
        let wrap = bracket && pane.screen_ref().mode() & crate::window::MODE_BRACKETPASTE != 0;
        let output = *pane.event();
        drop(owner);
        if wrap {
            output.write(b"\x1b[200~");
        }
        for line in bytes.split_inclusive(|&b| b == b'\n') {
            match line.strip_suffix(b"\n") {
                Some(head) => {
                    send_line(output, head, raw);
                    output.write(separator);
                }
                None => send_line(output, line, raw),
            }
        }
        if wrap {
            output.write(b"\x1b[201~");
        }
        true
    }

    /// # Safety
    /// Exclude conflicting pane state access until the file-input operation is created.
    pub(crate) unsafe fn start_input(
        &self,
        item: &crate::cmd::CmdqItemRef,
    ) -> Result<core::ffi::c_int, CString> {
        unsafe {
            let Some(pane) = self.get() else { return Ok(1) };
            item.with_item(|item| crate::window::window_pane_start_input(pane, item))
        }
    }
}

mod capture;
pub(crate) use capture::{CapturePaneEdge, PaneCapture};

mod pipe;
pub(crate) use pipe::PanePipePair;

impl RustWindowPaneWeak {
    /// # Safety
    /// Exclude conflicting pane and screen access while bringing history into view.
    pub(crate) unsafe fn trim_unused_screen(&self) -> bool {
        let Some(mut owner) = self.upgrade() else {
            return false;
        };
        let pane = unsafe { owner.as_pane_mut() };
        if !pane.modes().is_empty() {
            return true;
        }
        let (cx, cy) = pane.base().cursor();
        let grid = pane.base_mut().grid_mut();
        let adjust = grid.sy.wrapping_sub(1).wrapping_sub(cy).min(grid.hsize);
        grid.remove_history(adjust);
        pane.base_mut().set_cursor(cx, cy.wrapping_add(adjust));
        *pane.flags_mut() |= PANE_REDRAW;
        true
    }
}

impl RustWindowPaneWeak {
    /// # Safety
    /// Exclude conflicting pane output access during this snapshot.
    #[cfg(test)]
    pub(crate) unsafe fn output_offset(&self) -> Option<crate::RustPaneOutputOffset> {
        let owner = self.upgrade()?;
        Some(*unsafe { owner.as_pane() }.offset())
    }
}

#[cfg(test)]
mod lifetime_tests {
    use super::*;

    #[test]
    fn owned_snapshots_do_not_pin_the_pane_or_follow_later_mutation() {
        let mut owner = RustWindowPaneRef::from_pane(Box::default());
        let pane = owner.downgrade();
        unsafe {
            owner.as_pane_mut().set_pane_command(&crate::PaneCommand {
                argv: vec![c"before".to_owned()],
                ..Default::default()
            });
            pane.set_position(2, 4);
            let geometry = pane.geometry().unwrap();
            let command = pane.command().unwrap();
            pane.set_position(8, 9);
            owner.as_pane_mut().clear_pane_command();
            assert_eq!((geometry.x, geometry.y), (2, 4));
            assert_eq!(command.argv, [c"before".to_owned()]);
            let payload = owner.into_pane();
            assert!(!pane.is_alive());
            assert!(pane.geometry().is_none());
            assert!(pane.output_offset().is_none());
            assert!(!pane.set_position(0, 0));
            assert!(!pane.reset_modes());
            drop(payload);
        }
    }
}

impl RustWindowPaneWeak {
    /// Sets both pane window styles and invalidates its appearance.
    ///
    /// # Safety
    /// Resolve a live pane immediately before this call and exclude conflicting
    /// pane and option access. This operation dispatches no callbacks.
    pub(crate) unsafe fn set_window_style(&self, style: &CStr) {
        unsafe {
            let mut observed = self.clone();
            let pane = observed.get_mut().expect("the styled pane is present");
            pane.options_ref()
                .set_string(c"window-style", 0, c"%s", crate::fmt_args![style]);
            pane.options_ref().set_string(
                c"window-active-style",
                0,
                c"%s",
                crate::fmt_args![style],
            );
            *pane.flags_mut() |= PANE_REDRAW | PANE_STYLECHANGED | PANE_THEMECHANGED;
        }
    }
}

impl RustWindowPaneWeak {
    /// Appends a line to view mode, opening it if necessary. A missing pane is a
    /// no-op; mode entry and output use the existing implementation.
    ///
    /// # Safety
    /// Exclude conflicting pane/mode/screen access during entry and its callbacks.
    pub(crate) unsafe fn append_view_line(&self, message: &CStr) {
        unsafe {
            let Some(pane) = self.get() else { return };
            if pane
                .modes()
                .first()
                .is_none_or(|mode| mode.mode() != WindowMode::View)
            {
                self.set_mode(None, WindowMode::View, None, None);
            }
            let mut observed = self.clone();
            if let Some(pane) = observed.get_mut() {
                crate::modes::window_copy_add(pane, 1, c"%s", crate::fmt_args![message]);
            }
        }
    }
}

impl RustWindowPaneWeak {
    /// Marks the pane screen for redraw without changing window border/status policy.
    ///
    /// # Safety
    /// Exclude conflicting pane flag access during this synchronous operation.
    pub(crate) unsafe fn request_redraw(&self) -> bool {
        unsafe { self.add_flags(PANE_REDRAW) }
    }
}
