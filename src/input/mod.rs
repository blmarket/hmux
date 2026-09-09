//! The VT parser: the state machine a pane's output is fed through, and the
//! keys sent back the other way.
//!
//! Everything below this module is private. What the rest of the crate may
//! use is exactly what is re-exported here.

mod keys;
mod parser;

pub use keys::{
    KeyEncoding, KeyInputEncoder, MouseInputEncoder, RustKeyInputEncoder, RustMouseInputEncoder,
    input_key, input_key_build, input_key_get_mouse, input_key_pane,
};

pub use parser::{
    InputCtxRef, InputOwner, ictx_mut, input_cancel_requests, input_ctx, input_parse_buffer,
    input_parse_pane, input_request, input_request_handle, input_set_buffer_size,
};

#[cfg(test)]
pub(crate) use keys::{KEYC_CTRL, KEYC_LITERAL, KEYC_META};
#[cfg(test)]
pub(crate) use parser::{
    GRID_LINE_START_OUTPUT, GRID_LINE_START_PROMPT, INPUT_BUF_DEFAULT_SIZE, INPUT_BUF_START,
    INPUT_DISCARD, INPUT_END_BEL, INPUT_END_ST, INPUT_LAST, INPUT_REQUEST_CLIPBOARD,
    INPUT_REQUEST_PALETTE, INPUT_REQUEST_QUEUE, INPUT_REQUEST_TIMEOUT, MODE_FOCUSON, ictx_opt,
};
