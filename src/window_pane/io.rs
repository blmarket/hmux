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
            crate::style::RustColourEngine.clear_palette(Some(pane.palette_mut()));
            pane.ictx.clone()
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
