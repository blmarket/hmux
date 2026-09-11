use super::*;
use crate::window::{PANE_EXITED, PANE_CHANGED, PANE_UNSEENCHANGES};
use crate::screen::RustScreenWriteCtx;
use crate::log::log_get_level;
use crate::control::{control_pane_offset, control_pane_offset_mut, control_write_output};
use crate::server::{client_walk, server_destroy_pane};
use crate::reactor::Interest;
use crate::pane_output::PaneOutputOffset;
use crate::fmt_args;
use crate::log::log_debug;
use crate::consts::{CLIENT_CONTROL, SIZE_MAX};

pub(super) unsafe fn maintain_output(wp: &mut window_pane) {
    unsafe {
        let mut minimum: size_t;
        let mut off: core::ffi::c_int = 1 as core::ffi::c_int;
        let mut attached_clients: u_int = 0 as u_int;
        let mut new_size: size_t;
        minimum = wp.offset.position();
        if wp.pipe_fd != -(1 as core::ffi::c_int) && wp.pipe_offset.position() < minimum {
            minimum = wp.pipe_offset.position();
        }
        for c in client_walk() {
            if !c.attached_session().is_none() {
                attached_clients = attached_clients.wrapping_add(1);
                if !c.flags() & CLIENT_CONTROL as uint64_t != 0 {
                    off = 0 as core::ffi::c_int;
                } else {
                    let (offset, flag) = control_pane_offset(c.as_client(), wp);
                    if flag == 0 {
                        off = 0;
                    }
                    if let Some(wpo) = offset {
                        new_size = wp.unread_output_len(wpo);
                        log_debug(
                            c"%s: %s has %zu bytes used and %zu left for %%%u",
                            fmt_args![
                                c"server_client_check_pane_buffer".as_ptr(),
                                c.name(),
                                wpo.position().wrapping_sub(wp.base_offset),
                                new_size,
                                wp.pane_id()
                            ],
                        );
                        if wpo.position() < minimum {
                            minimum = wpo.position();
                        }
                    }
                }
            }
        }
        if attached_clients == 0 as u_int {
            off = 0 as core::ffi::c_int;
        }
        minimum = minimum.wrapping_sub(wp.base_offset);
        if !(minimum == 0 as size_t) {
            log_debug(
                c"%s: %%%u has %zu minimum (of %zu) bytes used",
                fmt_args![
                    c"server_client_check_pane_buffer".as_ptr(),
                    wp.pane_id(),
                    minimum,
                    wp.event.input_len()
                ],
            );
            wp.event.with_input(|buffer| buffer.drain(minimum));
            let output_base = wp.base_offset;
            if output_base > (SIZE_MAX as size_t).wrapping_sub(minimum) {
                log_debug(
                    c"%s: %%%u base offset has wrapped",
                    fmt_args![c"server_client_check_pane_buffer".as_ptr(), wp.pane_id()],
                );
                wp.offset.rebase(output_base);
                if wp.pipe_fd != -(1 as core::ffi::c_int) {
                    wp.pipe_offset.rebase(output_base);
                }
                for mut c in client_walk() {
                    if !(c.attached_session().is_none()
                        || !c.flags() & CLIENT_CONTROL as uint64_t != 0)
                    {
                        let (offset, flag) = control_pane_offset_mut(c.as_client_mut(), wp);
                        if let Some(offset) = offset.filter(|_| flag == 0) {
                            offset.rebase(output_base);
                        }
                    }
                }
                wp.base_offset = minimum;
            } else {
                wp.base_offset = output_base.wrapping_add(minimum);
            }
        }
        log_debug(
            c"%s: pane %%%u is %s",
            fmt_args![
                c"server_client_check_pane_buffer".as_ptr(),
                wp.pane_id(),
                if off != 0 {
                    c"off".as_ptr()
                } else {
                    c"on".as_ptr()
                }
            ],
        );
        if off != 0 {
            wp.event.disable(Interest::Read);
        } else {
            wp.event.enable(Interest::Read);
        };
    }
}
pub(super) fn window_pane_read_callback(wp: &mut window_pane) {
    unsafe {
        let size = wp.event.input_len();
        let new_size: size_t;
        if wp.pipe_fd != -(1 as core::ffi::c_int) {
            let mut new_data = wp.unread_output(&wp.pipe_offset);
            new_size = new_data.len();
            if new_size > 0 as size_t {
                wp.pipe_event.write_buffer(&mut new_data);
                let mut pipe_offset = wp.pipe_offset;
                wp.advance_output(&mut pipe_offset, new_size);
                wp.pipe_offset = pipe_offset;
            }
        }
        log_debug(c"%%%u has %zu bytes", fmt_args![wp.pane_id(), size]);
        for mut c in client_walk() {
            if !c.attached_session().is_none() && c.flags() & CLIENT_CONTROL as uint64_t != 0 {
                control_write_output(c.as_client_mut(), wp);
            }
        }
        wp.parse_output();
        wp.event.disable(Interest::Read);
    }
}
pub(super) fn window_pane_error_callback(wp: &mut window_pane) {
    unsafe {
        log_debug(c"%%%u error", fmt_args![wp.pane_id()]);
        *wp.flags_mut() |= PANE_EXITED;
        if wp.destroy_ready() {
            server_destroy_pane(
                &(wp).observation().expect("the pane is owned"),
                1 as core::ffi::c_int,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RustPaneOutputOffset;
    use crate::tests::test_fixtures::{StreamBuffer, globals};

    #[test]
    fn retention_waits_for_pipe_and_clamps_independent_readers() {
        let _guard = globals();
        let stream = StreamBuffer::new();
        stream.ptr().with_input(|input| input.append(b"abcdef"));
        let mut pane = window_pane { event: stream.ptr(), fd: -1, pipe_fd: 0,
            offset: RustPaneOutputOffset::at(5), pipe_offset: RustPaneOutputOffset::at(2),
            ..Default::default() };
        let mut reader = RustPaneOutputOffset::at(2);
        assert_eq!(pane.unread_output(&reader).as_slice(), b"cdef");
        pane.advance_output(&mut reader, usize::MAX);
        assert_eq!(reader.position(), 6);
        unsafe { pane.maintain_output() };
        assert_eq!(stream.ptr().input_len(), 4);
        assert_eq!(pane.unread_output(&pane.output_position()).as_slice(), b"f");
        pane.pipe_fd = -1;
        unsafe { pane.maintain_output() };
        assert_eq!(stream.ptr().input_len(), 1);
        assert_eq!(pane.unread_output_len(&reader), 0);
    }

    #[test]
    fn wrapped_output_rebases_the_parser_without_losing_unread_bytes() {
        let _guard = globals();
        let stream = StreamBuffer::new();
        stream.ptr().with_input(|input| input.append(b"abcde"));
        let mut pane = window_pane { event: stream.ptr(), fd: -1, pipe_fd: -1,
            base_offset: usize::MAX - 2, offset: RustPaneOutputOffset::at(1),
            ..Default::default() };
        unsafe { pane.maintain_output() };
        assert_eq!(pane.output_position().position(), 4);
        assert_eq!(pane.unread_output(&pane.output_position()).as_slice(), b"e");
    }
}

pub(super) unsafe fn parse_bytes(wp: &mut window_pane, mut input: ByteBuffer) {
    unsafe {
        let owner = wp.ictx.clone().expect("a pane being parsed has a parser");
        let mut ictx = owner.borrow_mut();
        let len = input.len();
        if len == 0 {
            return;
        }
        let window = wp
            .window_context()
            .expect("a parsed pane has a window context");
        window.update_activity();
        crate::plugin::note_pane_output(wp.pane_id());
        *wp.flags_mut() |= PANE_CHANGED;
        if !wp.modes.is_empty() {
            *wp.flags_mut() |= PANE_UNSEENCHANGES;
        }
        if log_get_level() != 0 {
            let data = input.as_slice();
            log_debug(
                c"%s: %%%u %s, %zu bytes: %.*s",
                fmt_args![
                    c"input_parse_buffer".as_ptr(),
                    wp.pane_id(),
                    ictx.state.name.as_ptr(),
                    len,
                    len as core::ffi::c_int,
                    data.as_ptr()
                ],
            );
        }
        let mut base_guard;
        let mut sctx = if wp.modes.is_empty() {
            RustScreenWriteCtx::on_pane_base(wp)
        } else {
            base_guard = wp.base_mut();
            RustScreenWriteCtx::on_borrowed_screen(&mut base_guard)
        };
        crate::input::parser::input_parse(&mut ictx, &mut sctx, input);
    }
}

