use ::c2rust_bitfields;
extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;
    pub type event_base;
    fn readlink(
        __path: *const ::core::ffi::c_char,
        __buf: *mut ::core::ffi::c_char,
        __len: size_t,
    ) -> ssize_t;
    fn tcgetpgrp(__fd: ::core::ffi::c_int) -> __pid_t;
    fn fclose(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn fopen(
        __filename: *const ::core::ffi::c_char,
        __modes: *const ::core::ffi::c_char,
    ) -> *mut FILE;
    fn fgetc(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn setenv(
        __name: *const ::core::ffi::c_char,
        __value: *const ::core::ffi::c_char,
        __replace: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn unsetenv(__name: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn ioctl(__fd: ::core::ffi::c_int, __request: ::core::ffi::c_ulong, ...) -> ::core::ffi::c_int;
    fn event_init() -> *mut event_base;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xrealloc(_: *mut ::core::ffi::c_void, _: size_t) -> *mut ::core::ffi::c_void;
    fn xasprintf(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
}
pub type __uint64_t = u64;
pub type __off_t = ::core::ffi::c_long;
pub type __off64_t = ::core::ffi::c_long;
pub type __pid_t = ::core::ffi::c_int;
pub type pid_t = __pid_t;
pub type ssize_t = isize;
pub type size_t = usize;
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
pub const MAXPATHLEN: ::core::ffi::c_int = PATH_MAX;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const PATH_MAX: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const EOF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const TIOCGSID: ::core::ffi::c_int = 0x5429 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn osdep_get_name(
    mut fd: ::core::ffi::c_int,
    mut tty: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut f: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut path: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    let mut ch: ::core::ffi::c_int = 0;
    let mut pgrp: pid_t = 0;
    pgrp = tcgetpgrp(fd) as pid_t;
    if pgrp == -(1 as ::core::ffi::c_int) {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    xasprintf(
        &raw mut path,
        b"/proc/%lld/cmdline\0" as *const u8 as *const ::core::ffi::c_char,
        pgrp as ::core::ffi::c_longlong,
    );
    f = fopen(path, b"r\0" as *const u8 as *const ::core::ffi::c_char) as *mut FILE;
    if f.is_null() {
        free(path as *mut ::core::ffi::c_void);
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    free(path as *mut ::core::ffi::c_void);
    len = 0 as size_t;
    buf = ::core::ptr::null_mut::<::core::ffi::c_char>();
    loop {
        ch = fgetc(f);
        if !(ch != EOF) {
            break;
        }
        if ch == '\0' as i32 {
            break;
        }
        buf = xrealloc(
            buf as *mut ::core::ffi::c_void,
            len.wrapping_add(2 as size_t),
        ) as *mut ::core::ffi::c_char;
        let fresh0 = len;
        len = len.wrapping_add(1);
        *buf.offset(fresh0 as isize) = ch as ::core::ffi::c_char;
    }
    if !buf.is_null() {
        *buf.offset(len as isize) = '\0' as i32 as ::core::ffi::c_char;
    }
    fclose(f);
    return buf;
}
#[no_mangle]
pub unsafe extern "C" fn osdep_get_cwd(mut fd: ::core::ffi::c_int) -> *mut ::core::ffi::c_char {
    static mut target: [::core::ffi::c_char; 4097] = [0; 4097];
    let mut path: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut pgrp: pid_t = 0;
    let mut sid: pid_t = 0;
    let mut n: ssize_t = 0;
    pgrp = tcgetpgrp(fd) as pid_t;
    if pgrp == -(1 as ::core::ffi::c_int) {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    xasprintf(
        &raw mut path,
        b"/proc/%lld/cwd\0" as *const u8 as *const ::core::ffi::c_char,
        pgrp as ::core::ffi::c_longlong,
    );
    n = readlink(
        path,
        &raw mut target as *mut ::core::ffi::c_char,
        MAXPATHLEN as size_t,
    );
    free(path as *mut ::core::ffi::c_void);
    if n == -(1 as ::core::ffi::c_int) as ssize_t
        && ioctl(fd, TIOCGSID as ::core::ffi::c_ulong, &raw mut sid) != -(1 as ::core::ffi::c_int)
    {
        xasprintf(
            &raw mut path,
            b"/proc/%lld/cwd\0" as *const u8 as *const ::core::ffi::c_char,
            sid as ::core::ffi::c_longlong,
        );
        n = readlink(
            path,
            &raw mut target as *mut ::core::ffi::c_char,
            MAXPATHLEN as size_t,
        );
        free(path as *mut ::core::ffi::c_void);
    }
    if n > 0 as ssize_t {
        target[n as usize] = '\0' as i32 as ::core::ffi::c_char;
        return &raw mut target as *mut ::core::ffi::c_char;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn osdep_event_init() -> *mut event_base {
    let mut base: *mut event_base = ::core::ptr::null_mut::<event_base>();
    setenv(
        b"EVENT_NOEPOLL\0" as *const u8 as *const ::core::ffi::c_char,
        b"1\0" as *const u8 as *const ::core::ffi::c_char,
        1 as ::core::ffi::c_int,
    );
    base = event_init();
    unsetenv(b"EVENT_NOEPOLL\0" as *const u8 as *const ::core::ffi::c_char);
    return base;
}
