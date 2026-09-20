//! Authoritative environment declarations.

pub const ENVIRON_HIDDEN: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct environ {
    pub rbh_root: *mut environ_entry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct environ_entry {
    pub name: *mut ::core::ffi::c_char,
    pub value: *mut ::core::ffi::c_char,
    pub flags: ::core::ffi::c_int,
    pub entry: environ_entry_entry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct environ_entry_entry {
    pub rbe_left: *mut environ_entry,
    pub rbe_right: *mut environ_entry,
    pub rbe_parent: *mut environ_entry,
    pub rbe_color: ::core::ffi::c_int,
}
