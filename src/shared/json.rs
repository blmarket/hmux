//! Authoritative json model declarations.

use super::abi::int64_t;
use std::collections::BTreeMap;
use std::collections::HashMap;

pub struct json_node {
    pub key: Option<std::ffi::CString>,
    pub parent: *mut json_node,
    pub value: JsonValue,
}

impl json_node {
    pub fn empty() -> Self {
        Self {
            key: None,
            parent: std::ptr::null_mut(),
            value: JsonValue::String(Default::default()),
        }
    }
    pub fn type_0(&self) -> json_node_type {
        self.value.kind()
    }
}

pub enum JsonValue {
    String(std::ffi::CString),
    Number(int64_t),
    Boolean(::core::ffi::c_int),
    Object(json_fields),
    Array(json_members_storage),
}

impl JsonValue {
    pub fn kind(&self) -> json_node_type {
        match self {
            Self::String(_) => 0,
            Self::Number(_) => 1,
            Self::Boolean(_) => 2,
            Self::Object(_) => 3,
            Self::Array(_) => 4,
        }
    }
    pub fn string_ptr(&self) -> *const ::core::ffi::c_char {
        let Self::String(value) = self else {
            panic!("not a JSON string")
        };
        value.as_ptr()
    }
    pub fn number(&self) -> int64_t {
        let Self::Number(value) = self else {
            panic!("not a JSON number")
        };
        *value
    }
    pub fn boolean(&self) -> ::core::ffi::c_int {
        let Self::Boolean(value) = self else {
            panic!("not a JSON boolean")
        };
        *value
    }
    pub fn fields(&self) -> &json_fields {
        let Self::Object(value) = self else {
            panic!("not a JSON object")
        };
        value
    }
    pub fn fields_mut(&mut self) -> &mut json_fields {
        let Self::Object(value) = self else {
            panic!("not a JSON object")
        };
        value
    }
    pub fn members(&self) -> &json_members_storage {
        let Self::Array(value) = self else {
            panic!("not a JSON array")
        };
        value
    }
    pub fn members_mut(&mut self) -> &mut json_members_storage {
        let Self::Array(value) = self else {
            panic!("not a JSON array")
        };
        value
    }
}

/// Array order belongs to the array node. Child nodes do not need intrusive links.
#[derive(Default)]
pub struct json_members_storage {
    pub(crate) members: Vec<*mut json_node>,
    pub(crate) indices: HashMap<*mut json_node, usize>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct json_fields {
    pub entries: *mut json_fields_storage,
}

/// Rust-owned ordering storage for an object's Box-owned field nodes.
#[derive(Default)]
pub struct json_fields_storage {
    pub(crate) entries: BTreeMap<Vec<u8>, *mut json_node>,
}

pub type json_node_type = ::core::ffi::c_uint;
