//! Authoritative monitor declarations, shared by the C translation units.

pub type monitor_type = ::core::ffi::c_uint;
pub const MONITOR_ALL_WINDOWS: monitor_type = 4;
pub const MONITOR_WINDOW: monitor_type = 3;
pub const MONITOR_ALL_PANES: monitor_type = 2;
pub const MONITOR_PANE: monitor_type = 1;
pub const MONITOR_SESSION: monitor_type = 0;
pub const MONITOR_NOTIFY_TRUE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MONITOR_NOTIFY_INITIAL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
