//! Authoritative argument storage, parsing callbacks, and scalar domains.

use super::abi::{u_char, u_int};
use super::command::{cmd, cmd_list, cmd_parse_input};
use std::collections::BTreeMap;
pub type args_type = ::core::ffi::c_uint;
pub const ARGS_COMMANDS: args_type = 2;
pub const ARGS_STRING: args_type = 1;
pub const ARGS_NONE: args_type = 0;

pub type args_parse_type = ::core::ffi::c_uint;
pub const ARGS_PARSE_COMMANDS: args_parse_type = 3;
pub const ARGS_PARSE_COMMANDS_OR_STRING: args_parse_type = 2;
pub const ARGS_PARSE_STRING: args_parse_type = 1;
pub const ARGS_PARSE_INVALID: args_parse_type = 0;

#[cfg(test)]
mod tests {
    use super::*;
    use ::core::mem::{align_of, size_of};

    #[test]
    fn argument_domains_match_translated_c_baseline() {
        assert_eq!(size_of::<args_type>(), 4);
        assert_eq!(align_of::<args_type>(), 4);
        assert_eq!(size_of::<args_parse_type>(), 4);
        assert_eq!(align_of::<args_parse_type>(), 4);
        assert_eq!(ARGS_NONE, 0);
        assert_eq!(ARGS_COMMANDS, 2);
        assert_eq!(ARGS_PARSE_INVALID, 0);
        assert_eq!(ARGS_PARSE_COMMANDS, 3);
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct args {
    pub tree: args_tree,
    pub count: u_int,
    pub values: *mut args_value,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct args_value {
    pub type_0: args_type,
    pub c2rust_unnamed: args_value_c2rust_unnamed,
    pub cached: *mut ::core::ffi::c_char,
    pub entry: args_value_entry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct args_value_entry {
    pub tqe_next: *mut args_value,
    pub tqe_prev: *mut *mut args_value,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union args_value_c2rust_unnamed {
    pub string: *mut ::core::ffi::c_char,
    pub cmdlist: *mut cmd_list,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct args_tree {
    pub entries: *mut args_tree_storage,
}

/// Rust-owned ordering storage for an argument tree's C-allocated entries.
#[derive(Default)]
pub struct args_tree_storage {
    pub(crate) entries: BTreeMap<u_char, *mut args_entry>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct args_entry {
    pub flag: u_char,
    pub values: args_values,
    pub count: u_int,
    pub flags: ::core::ffi::c_int,
    pub entry: args_entry_entry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct args_entry_entry {
    pub rbe_left: *mut args_entry,
    pub rbe_right: *mut args_entry,
    pub rbe_parent: *mut args_entry,
    pub rbe_color: ::core::ffi::c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct args_values {
    pub tqh_first: *mut args_value,
    pub tqh_last: *mut *mut args_value,
}

pub type args_parse_cb = Option<
    unsafe extern "C" fn(*mut args, u_int, *mut *mut ::core::ffi::c_char) -> args_parse_type,
>;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct args_parse {
    pub template: *const ::core::ffi::c_char,
    pub lower: ::core::ffi::c_int,
    pub upper: ::core::ffi::c_int,
    pub cb: args_parse_cb,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct args_command_state {
    pub cmdlist: *mut cmd_list,
    pub cmd: *mut ::core::ffi::c_char,
    pub pi: cmd_parse_input,
}
