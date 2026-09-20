//! Application event handles; no foreign event-loop layout is retained.
pub use hmux_rt::ByteBuffer as evbuffer;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct event {
    pub timer: usize,
    pub io: usize,
    pub signal: usize,
    pub deferred: usize,
    pub fd: ::core::ffi::c_int,
    pub flags: ::core::ffi::c_short,
    pub initialized: bool,
    pub callback: Option<
        unsafe extern "C" fn(::core::ffi::c_int, ::core::ffi::c_short, *mut ::core::ffi::c_void),
    >,
    pub arg: *mut ::core::ffi::c_void,
}
impl event {
    pub const ZERO: Self = Self {
        timer: 0,
        io: 0,
        signal: 0,
        deferred: 0,
        fd: 0,
        flags: 0,
        initialized: false,
        callback: None,
        arg: std::ptr::null_mut(),
    };
}
#[repr(C)]
pub struct event_base {
    _private: u8,
}
#[repr(C)]
pub struct bufferevent {
    pub id: usize,
    pub input: *mut evbuffer,
    pub output: *mut evbuffer,
}
pub type bufferevent_data_cb =
    Option<unsafe extern "C" fn(*mut bufferevent, *mut ::core::ffi::c_void)>;
pub type bufferevent_event_cb =
    Option<unsafe extern "C" fn(*mut bufferevent, ::core::ffi::c_short, *mut ::core::ffi::c_void)>;

pub const EV_TIMEOUT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const EV_READ: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const EV_WRITE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const EV_SIGNAL: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const EV_PERSIST: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;

pub type evbuffer_eol_style = ::core::ffi::c_uint;
pub const EVBUFFER_EOL_NUL: evbuffer_eol_style = 4;
pub const EVBUFFER_EOL_LF: evbuffer_eol_style = 3;
pub const EVBUFFER_EOL_CRLF_STRICT: evbuffer_eol_style = 2;
pub const EVBUFFER_EOL_CRLF: evbuffer_eol_style = 1;
pub const EVBUFFER_EOL_ANY: evbuffer_eol_style = 0;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn host_handles_are_zero_initializable_and_have_no_drop() {
        assert!(!std::mem::needs_drop::<event>());
        let zero: event = unsafe { std::mem::zeroed() };
        assert!(zero.callback.is_none());
        assert_eq!(zero.timer, 0);
        assert_eq!(std::mem::size_of::<event>(), 56);
        assert_eq!(std::mem::size_of::<bufferevent>(), 24);
    }
}
