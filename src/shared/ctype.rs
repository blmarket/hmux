//! Authoritative ctype declarations.

pub const _ISalnum: ctype_code = 8;

pub const _ISpunct: ctype_code = 4;

pub const _IScntrl: ctype_code = 2;

pub const _ISblank: ctype_code = 1;

pub const _ISgraph: ctype_code = 32768;

pub const _ISprint: ctype_code = 16384;

pub const _ISspace: ctype_code = 8192;

pub const _ISxdigit: ctype_code = 4096;

pub const _ISdigit: ctype_code = 2048;

pub const _ISalpha: ctype_code = 1024;

pub const _ISlower: ctype_code = 512;

pub const _ISupper: ctype_code = 256;

pub type ctype_code = ::core::ffi::c_uint;
