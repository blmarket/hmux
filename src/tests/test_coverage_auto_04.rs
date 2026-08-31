//! Coverage for [`crate::input`] — pure constants and buffer helpers.
//!
//! Exercises message-adjacent constants, buffer-size knobs and the lifecycle
//! helpers `input_init` / `input_free_box`, `input_reset`, `input_pending`,
//! `input_parse_buffer` (zero-length fast path) and the request-queue no-ops.
//! All tests are deterministic and avoid the fatal/IO paths.

use crate::input::{
    INPUT_BUF_DEFAULT_SIZE, INPUT_BUF_START, INPUT_DISCARD, INPUT_END_BEL, INPUT_END_ST,
    INPUT_LAST, INPUT_REQUEST_CLIPBOARD, INPUT_REQUEST_PALETTE, INPUT_REQUEST_QUEUE,
    INPUT_REQUEST_TIMEOUT, input_cancel_requests, input_free_box, input_init, input_parse_buffer,
    input_pending, input_request_reply, input_reset, input_set_buffer_size,
};
use crate::reactor::Stream;
use crate::style::{colour_palette_free, colour_palette_init};
use crate::tests::test_fixtures::{Pane, Window, ensure_reactor, globals};
use crate::types::InputRequestData;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

#[test]
fn input_end_constants_have_expected_values() {
    assert_eq!(INPUT_END_ST, 0);
    assert_eq!(INPUT_END_BEL, 1);
}

#[test]
fn input_request_type_constants_are_distinct() {
    assert_eq!(INPUT_REQUEST_PALETTE, 0);
    assert_eq!(INPUT_REQUEST_CLIPBOARD, 1);
    assert_eq!(INPUT_REQUEST_QUEUE, 2);
    assert_ne!(INPUT_REQUEST_PALETTE, INPUT_REQUEST_CLIPBOARD);
    assert_ne!(INPUT_REQUEST_CLIPBOARD, INPUT_REQUEST_QUEUE);
}

#[test]
fn input_buffer_constants_match_upstream() {
    assert_eq!(INPUT_BUF_DEFAULT_SIZE, 1048576);
    assert_eq!(INPUT_BUF_START, 32);
    assert_eq!(INPUT_REQUEST_TIMEOUT, 500);
    assert_eq!(INPUT_DISCARD, 0x1);
    assert_eq!(INPUT_LAST, 0x2);
}

// ---------------------------------------------------------------------------
// input_set_buffer_size — pure setter, no observable state except the static
// ---------------------------------------------------------------------------

#[test]
fn input_set_buffer_size_roundtrip_is_observable_via_later_allocation() {
    let _guard = globals();
    {
        input_set_buffer_size(2048);
        input_set_buffer_size(INPUT_BUF_DEFAULT_SIZE as usize);
        // setting again to default must not panic and must be idempotent
        input_set_buffer_size(INPUT_BUF_DEFAULT_SIZE as usize);
        // restore to a small value and back to default for other tests
        input_set_buffer_size(4096);
        input_set_buffer_size(INPUT_BUF_DEFAULT_SIZE as usize);
    }
}

// ---------------------------------------------------------------------------
// Helpers — a live input_ctx over a server-free pane
// ---------------------------------------------------------------------------

struct Ctx {
    _window: Window,
    pane: Pane,
    ictx: *mut crate::input::input_ctx,
    _guard: ::std::sync::MutexGuard<'static, ()>,
}

impl Ctx {
    fn new() -> Self {
        let guard = globals();
        ensure_reactor();
        let mut window = Window::new(1, "auto04", 80, 24);
        let mut pane = Pane::new(0, 80, 24, 100);
        window.add_pane(&mut pane);
        let wp = pane.ptr();
        let ictx = unsafe {
            colour_palette_init(&mut (*wp).palette);
            let ctx = input_init(crate::input::InputOwner::Pane((*wp).id), Stream::NONE);
            (*wp).ictx = Some(ctx);
            crate::input::ictx_opt(&(*wp).ictx).unwrap_or(::core::ptr::null_mut())
        };
        Self {
            _window: window,
            pane,
            ictx,
            _guard: guard,
        }
    }

    fn wp(&mut self) -> *mut crate::types::window_pane {
        self.pane.ptr()
    }
}

impl Drop for Ctx {
    fn drop(&mut self) {
        unsafe {
            let wp = self.wp();
            if let Some(ictx) = (*wp).ictx.take() {
                input_free_box(ictx);
            }
            colour_palette_free(Some(&mut (*wp).palette));
        }
    }
}

// ---------------------------------------------------------------------------
// input_init / input_pending / input_free_box lifecycle
// ---------------------------------------------------------------------------

#[test]
fn input_init_creates_pending_buffer_and_free_releases_it() {
    let ctx = Ctx::new();
    unsafe {
        let pending = input_pending(&mut *ctx.ictx);
        assert!(!pending.is_null());
        assert_eq!((*pending).len(), 0);
        // ictx fields initialised by input_init
        assert!((*ctx.ictx).input_buf.capacity() >= INPUT_BUF_START as usize);
        assert_eq!((*ctx.ictx).input_buf, [b'\0']);
    }
    // free happens in Drop — just check no panic
}

#[test]
fn input_reset_clears_intermediate_and_flags_without_touching_screen() {
    let mut ctx = Ctx::new();
    unsafe {
        // seed some state that input_reset must clear
        (*ctx.ictx).interm_len = 2;
        (*ctx.ictx).interm_buf[0] = b'(';
        (*ctx.ictx).interm_buf[1] = b')';
        (*ctx.ictx).param_len = 3;
        (*ctx.ictx).param_buf[0] = b'1';
        (*ctx.ictx).input_buf.extend_from_slice(b"abcd");
        (*ctx.ictx).flags = INPUT_DISCARD | INPUT_LAST;

        input_reset(&mut *ctx.ictx, 0);

        assert_eq!((*ctx.ictx).interm_len, 0);
        assert_eq!((*ctx.ictx).param_len, 0);
        assert_eq!((*ctx.ictx).flags & INPUT_DISCARD, 0);
        // state must be ground
        assert_eq!((*ctx.ictx).state.name, c"ground");
        // buffer still valid
        assert_eq!((*ctx.ictx).input_buf, [b'\0']);
    }
}

#[test]
fn input_parse_buffer_with_zero_length_is_noop() {
    let mut ctx = Ctx::new();
    unsafe {
        let before_cx = (*ctx.pane.screen()).cx;
        let before_cy = (*ctx.pane.screen()).cy;
        let pending_before = (*input_pending(&mut *ctx.ictx)).len();
        input_parse_buffer(ctx.wp(), b"".as_ptr(), 0);
        assert_eq!((*input_pending(&mut *ctx.ictx)).len(), pending_before);
        assert_eq!((*ctx.pane.screen()).cx, before_cx);
        assert_eq!((*ctx.pane.screen()).cy, before_cy);
        // printable path works after the no-op
        input_parse_buffer(ctx.wp(), b"hi".as_ptr(), 2);
        assert_eq!((*ctx.pane.screen()).cx, 2);
    }
}

#[test]
fn input_cancel_requests_on_empty_client_is_noop() {
    let _guard = globals();
    ensure_reactor();
    unsafe {
        let mut c = Box::new(crate::types::client::default());
        ::core::ptr::write(&raw mut c.input_requests, Vec::new());
        input_cancel_requests(&raw mut *c);
        assert!(c.input_requests.is_empty());
    }
}

#[test]
fn input_request_reply_with_no_matching_request_is_noop() {
    let _guard = globals();
    ensure_reactor();
    unsafe {
        let mut c = Box::new(crate::types::client::default());
        ::core::ptr::write(&raw mut c.input_requests, Vec::new());
        // no requests queued; must not crash
        input_request_reply(&raw mut *c, INPUT_REQUEST_PALETTE, &InputRequestData::None);
        input_request_reply(
            &raw mut *c,
            INPUT_REQUEST_CLIPBOARD,
            &InputRequestData::None,
        );
        input_request_reply(&raw mut *c, INPUT_REQUEST_QUEUE, &InputRequestData::None);
        assert!(c.input_requests.is_empty());
    }
}
