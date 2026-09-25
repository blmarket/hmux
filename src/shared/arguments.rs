//! Authoritative argument storage, parsing callbacks, and scalar domains.

use super::abi::{u_char, u_int};
use super::command::{cmd, cmd_list, cmd_parse_input};
use std::collections::BTreeMap;
use std::ffi::CString;
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

#[repr(C)]
pub struct args {
    pub tree: args_tree,
    pub count: u_int,
    pub values: Vec<args_value>,
}

impl args {
    pub fn empty() -> Self {
        Self {
            tree: unsafe { ::core::mem::zeroed() },
            count: unsafe { ::core::mem::zeroed() },
            values: Vec::new(),
        }
    }
}

pub struct ArgsCommand(pub *mut cmd_list);

impl Drop for ArgsCommand {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { crate::src::cmd::cmd_list_free(self.0) };
        }
    }
}

pub enum ArgsPayload {
    None,
    String(CString),
    Command(ArgsCommand),
    BorrowedString(*const ::core::ffi::c_char),
    BorrowedCommand(*mut cmd_list),
}

/// Stored values own their payload; parser inputs may use borrowed variants.
pub struct args_value {
    pub payload: ArgsPayload,
    pub cached: Option<CString>,
    pub entry: args_value_entry,
}

impl args_value {
    pub fn new(payload: ArgsPayload) -> Self {
        Self {
            payload,
            cached: None,
            entry: args_value_entry {
                owner: ::core::ptr::null_mut(),
                index: 0,
            },
        }
    }

    pub fn empty() -> Self {
        Self::new(ArgsPayload::None)
    }

    pub fn string(value: CString) -> Self {
        Self::new(ArgsPayload::String(value))
    }

    /// Takes over one existing command-list reference.
    pub unsafe fn commands(value: *mut cmd_list) -> Self {
        Self::new(ArgsPayload::Command(ArgsCommand(value)))
    }

    /// The pointer must remain valid until this temporary parser value is dropped.
    pub unsafe fn borrowed_string(value: *const ::core::ffi::c_char) -> Self {
        Self::new(ArgsPayload::BorrowedString(value))
    }

    /// The command list must remain valid until this temporary parser value is dropped.
    pub unsafe fn borrowed_commands(value: *mut cmd_list) -> Self {
        Self::new(ArgsPayload::BorrowedCommand(value))
    }

    pub fn type_0(&self) -> args_type {
        match self.payload {
            ArgsPayload::None => ARGS_NONE,
            ArgsPayload::String(_) | ArgsPayload::BorrowedString(_) => ARGS_STRING,
            ArgsPayload::Command(_) | ArgsPayload::BorrowedCommand(_) => ARGS_COMMANDS,
        }
    }

    pub fn string_ptr(&self) -> *const ::core::ffi::c_char {
        match &self.payload {
            ArgsPayload::String(value) => value.as_ptr(),
            ArgsPayload::BorrowedString(value) => *value,
            _ => ::core::ptr::null(),
        }
    }

    pub fn cmdlist(&self) -> *mut cmd_list {
        match &self.payload {
            ArgsPayload::Command(value) => value.0,
            ArgsPayload::BorrowedCommand(value) => *value,
            _ => ::core::ptr::null_mut(),
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct args_value_entry {
    /// Owner collection used by args_next_value; this is not a neighbor link.
    pub owner: *mut args_values_storage,
    /// Stable position in the owner's append-only value collection.
    pub index: usize,
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
/// Box-owned by the enclosing args tree. Its value-list tail may point into
/// this stable record; `args_free` unlinks values before dropping the Box.
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
/// ABI-sized head view for an argument entry's flag values. The head stores a
/// first-element view and owns the collection through its storage pointer.
pub struct args_values {
    pub first: *mut args_value,
    pub storage: *mut args_values_storage,
}

/// Owns stable flag-value records for one args_entry.
#[derive(Default)]
pub struct args_values_storage {
    pub(crate) values: Vec<Box<args_value>>,
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

#[repr(C)]
pub struct args_command_state {
    pub cmdlist: *mut cmd_list,
    pub cmd: Option<std::ffi::CString>,
    pub pi: cmd_parse_input,
    pub(crate) file: Option<std::ffi::CString>,
}

impl args_command_state {
    pub fn empty() -> Self {
        Self {
            cmdlist: unsafe { ::core::mem::zeroed() },
            cmd: Default::default(),
            pi: unsafe { ::core::mem::zeroed() },
            file: Default::default(),
        }
    }
}
