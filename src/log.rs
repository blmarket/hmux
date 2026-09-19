use ::c2rust_bitfields;
extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;
    fn __errno_location() -> *mut ::core::ffi::c_int;
    fn fclose(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn fflush(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn fopen(
        __filename: *const ::core::ffi::c_char,
        __modes: *const ::core::ffi::c_char,
    ) -> *mut FILE;
    fn setvbuf(
        __stream: *mut FILE,
        __buf: *mut ::core::ffi::c_char,
        __modes: ::core::ffi::c_int,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn fprintf(
        __stream: *mut FILE,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn vasprintf(
        __ptr: *mut *mut ::core::ffi::c_char,
        __f: *const ::core::ffi::c_char,
        __arg: ::core::ffi::VaList,
    ) -> ::core::ffi::c_int;
    fn exit(__status: ::core::ffi::c_int) -> !;
    fn strerror(__errnum: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn getpid() -> __pid_t;
    fn gettimeofday(__tv: *mut timeval, __tz: *mut ::core::ffi::c_void) -> ::core::ffi::c_int;
    fn event_set_log_callback(cb: event_log_cb);
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn stravis(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn xasprintf(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
}
pub type __builtin_va_list = [__va_list_tag; 1];
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __va_list_tag {
    pub gp_offset: ::core::ffi::c_uint,
    pub fp_offset: ::core::ffi::c_uint,
    pub overflow_arg_area: *mut ::core::ffi::c_void,
    pub reg_save_area: *mut ::core::ffi::c_void,
}
pub type __uint64_t = u64;
pub type __off_t = ::core::ffi::c_long;
pub type __off64_t = ::core::ffi::c_long;
pub type __pid_t = ::core::ffi::c_int;
pub type __time_t = ::core::ffi::c_long;
pub type __suseconds_t = ::core::ffi::c_long;
pub type size_t = usize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct timeval {
    pub tv_sec: __time_t,
    pub tv_usec: __suseconds_t,
}
pub type __gnuc_va_list = __builtin_va_list;
#[derive(Copy, Clone, BitfieldStruct)]
#[repr(C)]
pub struct _IO_FILE {
    pub _flags: ::core::ffi::c_int,
    pub _IO_read_ptr: *mut ::core::ffi::c_char,
    pub _IO_read_end: *mut ::core::ffi::c_char,
    pub _IO_read_base: *mut ::core::ffi::c_char,
    pub _IO_write_base: *mut ::core::ffi::c_char,
    pub _IO_write_ptr: *mut ::core::ffi::c_char,
    pub _IO_write_end: *mut ::core::ffi::c_char,
    pub _IO_buf_base: *mut ::core::ffi::c_char,
    pub _IO_buf_end: *mut ::core::ffi::c_char,
    pub _IO_save_base: *mut ::core::ffi::c_char,
    pub _IO_backup_base: *mut ::core::ffi::c_char,
    pub _IO_save_end: *mut ::core::ffi::c_char,
    pub _markers: *mut _IO_marker,
    pub _chain: *mut _IO_FILE,
    pub _fileno: ::core::ffi::c_int,
    #[bitfield(name = "_flags2", ty = "::core::ffi::c_int", bits = "0..=23")]
    pub _flags2: [u8; 3],
    pub _short_backupbuf: [::core::ffi::c_char; 1],
    pub _old_offset: __off_t,
    pub _cur_column: ::core::ffi::c_ushort,
    pub _vtable_offset: ::core::ffi::c_schar,
    pub _shortbuf: [::core::ffi::c_char; 1],
    pub _lock: *mut ::core::ffi::c_void,
    pub _offset: __off64_t,
    pub _codecvt: *mut _IO_codecvt,
    pub _wide_data: *mut _IO_wide_data,
    pub _freeres_list: *mut _IO_FILE,
    pub _freeres_buf: *mut ::core::ffi::c_void,
    pub _prevchain: *mut *mut _IO_FILE,
    pub _mode: ::core::ffi::c_int,
    pub _unused3: ::core::ffi::c_int,
    pub _total_written: __uint64_t,
    pub _unused2: [::core::ffi::c_char; 8],
}
pub type _IO_lock_t = ();
pub type FILE = _IO_FILE;
pub type va_list = __gnuc_va_list;
pub type event_log_cb =
    Option<unsafe extern "C" fn(::core::ffi::c_int, *const ::core::ffi::c_char) -> ()>;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const _IOLBF: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const VIS_OCTAL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const VIS_CSTYLE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const VIS_TAB: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const VIS_NL: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
static mut log_file: *mut FILE = ::core::ptr::null::<FILE>() as *mut FILE;
static mut log_level: ::core::ffi::c_int = 0;
unsafe extern "C" fn log_event_cb(
    mut severity: ::core::ffi::c_int,
    mut msg: *const ::core::ffi::c_char,
) {
    log_debug(b"%s\0" as *const u8 as *const ::core::ffi::c_char, msg);
}
#[no_mangle]
pub unsafe extern "C" fn log_add_level() {
    log_level += 1;
}
#[no_mangle]
pub unsafe extern "C" fn log_get_level() -> ::core::ffi::c_int {
    return log_level;
}
#[no_mangle]
pub unsafe extern "C" fn log_open(mut name: *const ::core::ffi::c_char) {
    let mut path: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if log_level == 0 as ::core::ffi::c_int {
        return;
    }
    log_close();
    xasprintf(
        &raw mut path,
        b"tmux-%s-%ld.log\0" as *const u8 as *const ::core::ffi::c_char,
        name,
        getpid() as ::core::ffi::c_long,
    );
    log_file = fopen(path, b"a\0" as *const u8 as *const ::core::ffi::c_char) as *mut FILE;
    free(path as *mut ::core::ffi::c_void);
    if log_file.is_null() {
        return;
    }
    setvbuf(
        log_file,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        _IOLBF,
        0 as size_t,
    );
    event_set_log_callback(Some(
        log_event_cb as unsafe extern "C" fn(::core::ffi::c_int, *const ::core::ffi::c_char) -> (),
    ));
}
#[no_mangle]
pub unsafe extern "C" fn log_toggle(mut name: *const ::core::ffi::c_char) {
    if log_level == 0 as ::core::ffi::c_int {
        log_level = 1 as ::core::ffi::c_int;
        log_open(name);
        log_debug(b"log opened\0" as *const u8 as *const ::core::ffi::c_char);
    } else {
        log_debug(b"log closed\0" as *const u8 as *const ::core::ffi::c_char);
        log_level = 0 as ::core::ffi::c_int;
        log_close();
    };
}
#[no_mangle]
pub unsafe extern "C" fn log_close() {
    if !log_file.is_null() {
        fclose(log_file);
    }
    log_file = ::core::ptr::null_mut::<FILE>();
    event_set_log_callback(None);
}
unsafe extern "C" fn log_vwrite(
    mut msg: *const ::core::ffi::c_char,
    mut ap: ::core::ffi::VaList,
    mut prefix: *const ::core::ffi::c_char,
) {
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    if log_file.is_null() {
        return;
    }
    if vasprintf(&raw mut s, msg, ap) == -(1 as ::core::ffi::c_int) {
        return;
    }
    if stravis(&raw mut out, s, VIS_OCTAL | VIS_CSTYLE | VIS_TAB | VIS_NL)
        == -(1 as ::core::ffi::c_int)
    {
        free(s as *mut ::core::ffi::c_void);
        return;
    }
    free(s as *mut ::core::ffi::c_void);
    gettimeofday(&raw mut tv, NULL);
    if fprintf(
        log_file,
        b"%lld.%06d %s%s\n\0" as *const u8 as *const ::core::ffi::c_char,
        tv.tv_sec as ::core::ffi::c_longlong,
        tv.tv_usec as ::core::ffi::c_int,
        prefix,
        out,
    ) != -(1 as ::core::ffi::c_int)
    {
        fflush(log_file);
    }
    free(out as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn log_debug(mut msg: *const ::core::ffi::c_char, mut args: ...) {
    let mut ap: ::core::ffi::VaList;
    if log_file.is_null() {
        return;
    }
    ap = args.clone();
    log_vwrite(
        msg,
        ap,
        b"\0" as *const u8 as *const ::core::ffi::c_char,
    );
}
#[no_mangle]
pub unsafe extern "C" fn fatal(mut msg: *const ::core::ffi::c_char, mut args: ...) -> ! {
    let mut tmp: [::core::ffi::c_char; 256] = [0; 256];
    let mut ap: ::core::ffi::VaList;
    if snprintf(
        &raw mut tmp as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        b"fatal: %s: \0" as *const u8 as *const ::core::ffi::c_char,
        strerror(*__errno_location()),
    ) < 0 as ::core::ffi::c_int
    {
        exit(1 as ::core::ffi::c_int);
    }
    ap = args.clone();
    log_vwrite(
        msg,
        ap,
        &raw mut tmp as *mut ::core::ffi::c_char,
    );
    exit(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn fatalx(mut msg: *const ::core::ffi::c_char, mut args: ...) -> ! {
    let mut ap: ::core::ffi::VaList;
    ap = args.clone();
    log_vwrite(
        msg,
        ap,
        b"fatal: \0" as *const u8 as *const ::core::ffi::c_char,
    );
    exit(1 as ::core::ffi::c_int);
}
