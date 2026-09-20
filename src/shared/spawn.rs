//! Authoritative spawn declarations, shared by the C translation units.

pub const SPAWN_BEFORE: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const SPAWN_FULLSIZE: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const SPAWN_HORIZONTAL: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const SPAWN_KILL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const SPAWN_DETACHED: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const SPAWN_EMPTY: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const SPAWN_RESPAWN: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const SPAWN_ZOOM: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const SPAWN_FLOATING: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const SPAWN_SPLIT: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const SPAWN_MODAL: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const SPAWN_FLOATOVERZOOM: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const SPAWN_NONOTIFY: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
