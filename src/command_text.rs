//! Encoding of command argument vectors for the wire and display paths.

use core::ffi::c_int;
use std::ffi::CString;

/// Command argument-vector encoding shared by clients, servers, and formats.
pub trait CommandTextCodec {
    /// Packs arguments as consecutive nul-terminated strings.
    fn pack(&self, argv: &[CString], out: &mut [u8]) -> bool;

    /// Unpacks `argc` nul-terminated strings from a protocol buffer.
    fn unpack(&self, packed: &mut [u8], argc: c_int) -> Option<Vec<CString>>;

    /// Quotes an argument vector as command text.
    fn stringify(&self, argv: &[CString]) -> CString;
}

/// Command argument-vector encoding implemented by hmux.
#[derive(Clone, Copy, Debug, Default)]
pub struct RustCommandTextCodec;

impl CommandTextCodec for RustCommandTextCodec {
    fn pack(&self, argv: &[CString], out: &mut [u8]) -> bool {
        crate::cmd::cmd_pack_argv_impl(argv, out) == 0
    }

    fn unpack(&self, packed: &mut [u8], argc: c_int) -> Option<Vec<CString>> {
        crate::cmd::cmd_unpack_argv_impl(packed, argc)
    }

    fn stringify(&self, argv: &[CString]) -> CString {
        crate::cmd::cmd_stringify_argv_impl(argv)
    }
}
