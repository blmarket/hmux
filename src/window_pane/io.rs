use super::*;
use core::ffi::c_char;
use crate::window::{PANE_STYLECHANGED, PANE_THEMECHANGED, PANE_REDRAW};
use crate::text::{RustUtf8VisModel, Utf8VisModel};

impl RustWindowPaneWeak {
    /// # Safety
    /// Exclude conflicting palette and input access while resetting the terminal.
    pub(crate) unsafe fn reset_terminal(&self) -> bool {
        use crate::style::ColourEngine;
        let context = {
            let Some(mut owner) = self.upgrade() else {
                return false;
            };
            let pane = unsafe { &mut *owner.0.pane.get() };
            pane.clear_palette();
            pane.ictx.clone()
        };
        unsafe {
            context.as_ref().expect("pane input initialized").reset(1);
            self.appearance_changed();
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
        let pane = unsafe { &*owner.0.pane.get() };
        if *pane.flags() & crate::window::PANE_INPUTOFF != 0 {
            return true;
        }
        let wrap = bracket && pane.screen_ref().mode() & crate::window::MODE_BRACKETPASTE != 0;
        let output = pane.event;
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

}

impl RustWindowPaneWeak {
    /// # Safety
    /// Exclude conflicting pane and parser access while reading pending bytes.
    pub(crate) unsafe fn capture_pending(&self, escape: bool) -> Option<Vec<u8>> {
        let owner = self.upgrade()?;
        let wp = unsafe { &*owner.0.pane.get() };
        let mut ictx = crate::input::ictx_mut(&wp.ictx);
        let Some(pending) = ictx.pending() else {
            return Some(Vec::new());
        };
        let line = pending.as_slice();
        let linelen = line.len();
        if linelen == 0 {
            return Some(Vec::new());
        }
        if !escape {
            return Some(line.to_vec());
        }
        let mut buf = Vec::with_capacity(linelen);
        for &byte in line {
            if byte as c_char >= b' ' as c_char && byte != b'\\' {
                buf.push(byte);
            } else {
                buf.push(b'\\');
                buf.push(b'0' + (byte >> 6));
                buf.push(b'0' + ((byte >> 3) & 7));
                buf.push(b'0' + (byte & 7));
            }
        }
        Some(buf)
    }

}

impl RustWindowPaneWeak {
    /// Writes a mode selection to clients, restoring content redraw after temporary
    /// line-number suppression and notifying only after the writer finishes.
    /// # Safety
    /// Exclude conflicting pane/screen access. Writer callbacks must not retain a
    /// pane borrow; this operation keeps only an allocation observation across them.
    pub(crate) unsafe fn write_clipboard_selection(&self, display: &ScreenRef, bytes: &[u8], line_numbers: bool) {
        use crate::screen::{RustScreenWriteCtx, ScreenWriteCtx};
        unsafe {
            let restore = {
                let Some(mut owner) = self.upgrade() else { return; };
                let pane = &mut *owner.0.pane.get();
                let restore = line_numbers && pane.flags & PANE_REDRAW != 0;
                if restore { pane.flags &= !PANE_REDRAW; }
                restore
            };
            let mut writer = RustScreenWriteCtx::on_shared_pane_screen(display, Some(self.clone()));
            writer.setselection(c"", bytes);
            writer.finish();
            if restore { self.request_redraw(); }
            crate::notify::notify_pane(c"pane-set-clipboard", self.get());
        }
    }
}

impl RustWindowPaneWeak {
    /// Invalidates pane content, style and theme after an appearance-context change.
    /// # Safety
    /// Exclude conflicting pane access while scheduling the refresh.
    pub(crate) unsafe fn appearance_changed(&self) {
        if let Some(owner) = self.allocation.upgrade() {
            unsafe { (*owner.pane.get()).flags |= PANE_STYLECHANGED | PANE_THEMECHANGED | PANE_REDRAW; }
        }
    }
    /// Delivers focus protocol and notification before recording the window's focus state.
    /// # Safety
    /// Exclude conflicting pane/client access; callbacks must not retain pane borrows.
    pub(crate) unsafe fn update_focus(&self, w: &WindowRef, focused: bool) {
        use crate::consts::MODE_FOCUSON;
        use crate::window::{PANE_FOCUSED, PANE_EXITED};
        unsafe {
            let pane = self;
            let Some(wp) = pane.get() else { return; };
            if *wp.flags() & PANE_EXITED != 0 { return; }
            if focused == (*wp.flags() & PANE_FOCUSED != 0) {
                log_debug(
                    c"%s: %%%u focus unchanged",
                    fmt_args![c"window_pane_update_focus", pane.id()],
                );
                return;
            }
            let (description, event, sequence): (&CStr, &CStr, &[u8]) = if focused {
                (c"focus in", c"pane-focus-in", b"\x1B[I")
            } else {
                (c"focus out", c"pane-focus-out", b"\x1B[O")
            };
            log_debug(
                c"%s: %%%u %s",
                fmt_args![c"window_pane_update_focus", pane.id(), description],
            );
            if wp.base().mode() & MODE_FOCUSON != 0 {
                wp.write_terminal(sequence);
            }
            crate::notify::notify_pane_in_window(event, wp, w);
            if let Some(owner) = pane.allocation.upgrade() {
                let wp = &mut *owner.pane.get();
                if focused {
                    wp.flags |= PANE_FOCUSED;
                } else {
                    wp.flags &= !PANE_FOCUSED;
                }
            }
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
            self.appearance_changed();
        }
    }
}
