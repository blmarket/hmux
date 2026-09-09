//! Stable access to decoded key and mouse events.

use crate::text::key_code;
use crate::types::u_int;
use core::ffi::c_int;

/// One decoded mouse event and its routing metadata.
pub trait MouseEvent {
    /// Builds an invalid, empty mouse event.
    fn from_mouse_event() -> Self
    where
        Self: Sized;

    /// Returns the validity marker.
    fn mouse_valid(&self) -> c_int;

    /// Replaces the validity marker.
    fn set_mouse_valid(&mut self, valid: c_int);

    /// Returns the ignore marker.
    fn mouse_ignore(&self) -> c_int;

    /// Replaces the ignore marker.
    fn set_mouse_ignore(&mut self, ignore: c_int);

    /// Returns the decoded mouse key.
    fn mouse_key(&self) -> key_code;

    /// Replaces the decoded mouse key.
    fn set_mouse_key(&mut self, key: key_code);

    /// Returns the status-line position and height.
    fn mouse_status(&self) -> (c_int, u_int);

    /// Replaces the status-line position and height.
    fn set_mouse_status(&mut self, position: c_int, lines: u_int);

    /// Returns the current terminal position.
    fn mouse_position(&self) -> (u_int, u_int);

    /// Replaces the current terminal position.
    fn set_mouse_position(&mut self, x: u_int, y: u_int);

    /// Returns the current button bits.
    fn mouse_button(&self) -> u_int;

    /// Replaces the current button bits.
    fn set_mouse_button(&mut self, button: u_int);

    /// Returns the previous terminal position.
    fn mouse_last_position(&self) -> (u_int, u_int);

    /// Replaces the previous terminal position.
    fn set_mouse_last_position(&mut self, x: u_int, y: u_int);

    /// Returns the previous button bits.
    fn mouse_last_button(&self) -> u_int;

    /// Replaces the previous button bits.
    fn set_mouse_last_button(&mut self, button: u_int);

    /// Returns the terminal origin offset.
    fn mouse_offset(&self) -> (u_int, u_int);

    /// Replaces the terminal origin offset.
    fn set_mouse_offset(&mut self, x: u_int, y: u_int);

    /// Returns the session, window, and pane routing identifiers.
    fn mouse_target(&self) -> (c_int, c_int, c_int);

    /// Replaces the session, window, and pane routing identifiers.
    fn set_mouse_target(&mut self, session: c_int, window: c_int, pane: c_int);

    /// Returns the SGR event type and button bits.
    fn mouse_sgr(&self) -> (u_int, u_int);

    /// Replaces the SGR event type and button bits.
    fn set_mouse_sgr(&mut self, event_type: u_int, button: u_int);
}

/// One decoded key event, with its optional mouse payload and raw bytes.
pub trait KeyEvent {
    /// The mouse payload implementation carried by the event.
    type Mouse: MouseEvent;

    /// Builds an event from a key, mouse payload, and raw bytes.
    fn from_key_event(key: key_code, mouse: Self::Mouse, buffer: Vec<u8>) -> Self
    where
        Self: Sized;

    /// Returns the decoded key.
    fn key_event_key(&self) -> key_code;

    /// Replaces the decoded key.
    fn set_key_event_key(&mut self, key: key_code);

    /// Returns the mouse payload.
    fn key_event_mouse(&self) -> &Self::Mouse;

    /// Returns the mutable mouse payload.
    fn key_event_mouse_mut(&mut self) -> &mut Self::Mouse;

    /// Returns the raw bytes attached to the event.
    fn key_event_buffer(&self) -> &[u8];

    /// Replaces the raw bytes attached to the event.
    fn set_key_event_buffer(&mut self, buffer: Vec<u8>);

    /// Removes and returns the raw bytes attached to the event.
    fn take_key_event_buffer(&mut self) -> Vec<u8>;
}

impl MouseEvent for crate::types::mouse_event {
    fn from_mouse_event() -> Self {
        Self::default()
    }

    fn mouse_valid(&self) -> c_int {
        self.valid
    }
    fn set_mouse_valid(&mut self, valid: c_int) {
        self.valid = valid;
    }
    fn mouse_ignore(&self) -> c_int {
        self.ignore
    }
    fn set_mouse_ignore(&mut self, ignore: c_int) {
        self.ignore = ignore;
    }
    fn mouse_key(&self) -> key_code {
        self.key
    }
    fn set_mouse_key(&mut self, key: key_code) {
        self.key = key;
    }
    fn mouse_status(&self) -> (c_int, u_int) {
        (self.statusat, self.statuslines)
    }
    fn set_mouse_status(&mut self, position: c_int, lines: u_int) {
        self.statusat = position;
        self.statuslines = lines;
    }
    fn mouse_position(&self) -> (u_int, u_int) {
        (self.x, self.y)
    }
    fn set_mouse_position(&mut self, x: u_int, y: u_int) {
        self.x = x;
        self.y = y;
    }
    fn mouse_button(&self) -> u_int {
        self.b
    }
    fn set_mouse_button(&mut self, button: u_int) {
        self.b = button;
    }
    fn mouse_last_position(&self) -> (u_int, u_int) {
        (self.lx, self.ly)
    }
    fn set_mouse_last_position(&mut self, x: u_int, y: u_int) {
        self.lx = x;
        self.ly = y;
    }
    fn mouse_last_button(&self) -> u_int {
        self.lb
    }
    fn set_mouse_last_button(&mut self, button: u_int) {
        self.lb = button;
    }
    fn mouse_offset(&self) -> (u_int, u_int) {
        (self.ox, self.oy)
    }
    fn set_mouse_offset(&mut self, x: u_int, y: u_int) {
        self.ox = x;
        self.oy = y;
    }
    fn mouse_target(&self) -> (c_int, c_int, c_int) {
        (self.s, self.w, self.wp)
    }
    fn set_mouse_target(&mut self, session: c_int, window: c_int, pane: c_int) {
        self.s = session;
        self.w = window;
        self.wp = pane;
    }
    fn mouse_sgr(&self) -> (u_int, u_int) {
        (self.sgr_type, self.sgr_b)
    }
    fn set_mouse_sgr(&mut self, event_type: u_int, button: u_int) {
        self.sgr_type = event_type;
        self.sgr_b = button;
    }
}

impl KeyEvent for crate::types::key_event {
    type Mouse = crate::types::mouse_event;

    fn from_key_event(key: key_code, mouse: Self::Mouse, buffer: Vec<u8>) -> Self {
        Self {
            key,
            m: mouse,
            buf: buffer,
        }
    }

    fn key_event_key(&self) -> key_code {
        self.key
    }
    fn set_key_event_key(&mut self, key: key_code) {
        self.key = key;
    }
    fn key_event_mouse(&self) -> &Self::Mouse {
        &self.m
    }
    fn key_event_mouse_mut(&mut self) -> &mut Self::Mouse {
        &mut self.m
    }
    fn key_event_buffer(&self) -> &[u8] {
        &self.buf
    }
    fn set_key_event_buffer(&mut self, buffer: Vec<u8>) {
        self.buf = buffer;
    }
    fn take_key_event_buffer(&mut self) -> Vec<u8> {
        core::mem::take(&mut self.buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{key_event, mouse_event};

    #[test]
    fn grouped_event_surface_round_trips_every_value() {
        let mut mouse = mouse_event::from_mouse_event();
        mouse.set_mouse_valid(1);
        mouse.set_mouse_ignore(2);
        mouse.set_mouse_key(3);
        mouse.set_mouse_status(-1, 4);
        mouse.set_mouse_position(5, 6);
        mouse.set_mouse_button(7);
        mouse.set_mouse_last_position(8, 9);
        mouse.set_mouse_last_button(10);
        mouse.set_mouse_offset(11, 12);
        mouse.set_mouse_target(13, 14, 15);
        mouse.set_mouse_sgr(16, 17);

        let mut event = key_event::from_key_event(18, mouse, vec![19, 20]);
        assert_eq!(event.key_event_key(), 18);
        assert_eq!(event.key_event_mouse().mouse_valid(), 1);
        assert_eq!(event.key_event_mouse().mouse_ignore(), 2);
        assert_eq!(event.key_event_mouse().mouse_key(), 3);
        assert_eq!(event.key_event_mouse().mouse_status(), (-1, 4));
        assert_eq!(event.key_event_mouse().mouse_position(), (5, 6));
        assert_eq!(event.key_event_mouse().mouse_button(), 7);
        assert_eq!(event.key_event_mouse().mouse_last_position(), (8, 9));
        assert_eq!(event.key_event_mouse().mouse_last_button(), 10);
        assert_eq!(event.key_event_mouse().mouse_offset(), (11, 12));
        assert_eq!(event.key_event_mouse().mouse_target(), (13, 14, 15));
        assert_eq!(event.key_event_mouse().mouse_sgr(), (16, 17));
        assert_eq!(event.key_event_buffer(), [19, 20]);
        assert_eq!(event.take_key_event_buffer(), vec![19, 20]);
        assert!(event.key_event_buffer().is_empty());
    }
}
