use crate::src::shared::prompt::*;
use crate::src::shared::abi::*;
use ::c2rust_bitfields;
extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;
    pub type options;
    fn __errno_location() -> *mut ::core::ffi::c_int;
    fn memmove(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strerror(__errnum: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn strsep(
        __stringp: *mut *mut ::core::ffi::c_char,
        __delim: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn fclose(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn fopen(
        __filename: *const ::core::ffi::c_char,
        __modes: *const ::core::ffi::c_char,
    ) -> *mut FILE;
    fn fputc(__c: ::core::ffi::c_int, __stream: *mut FILE) -> ::core::ffi::c_int;
    fn __getdelim(
        __lineptr: *mut *mut ::core::ffi::c_char,
        __n: *mut size_t,
        __delimiter: ::core::ffi::c_int,
        __stream: *mut FILE,
    ) -> __ssize_t;
    fn fputs(__s: *const ::core::ffi::c_char, __stream: *mut FILE) -> ::core::ffi::c_int;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xreallocarray(_: *mut ::core::ffi::c_void, _: size_t, _: size_t)
        -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xasprintf(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    static mut global_options: *mut options;
    fn find_home() -> *const ::core::ffi::c_char;
    fn options_get_string(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn options_get_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn prompt_type(_: *const ::core::ffi::c_char) -> prompt_type;
    fn prompt_type_string(_: prompt_type) -> *const ::core::ffi::c_char;
    fn log_debug(_: *const ::core::ffi::c_char, ...);
}
pub type __off_t = ::core::ffi::c_long;
pub type __off64_t = ::core::ffi::c_long;
pub type __ssize_t = ::core::ffi::c_long;
pub type ssize_t = isize;
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
#[inline]
unsafe extern "C" fn getline(
    mut __lineptr: *mut *mut ::core::ffi::c_char,
    mut __n: *mut size_t,
    mut __stream: *mut FILE,
) -> __ssize_t {
    return __getdelim(__lineptr, __n, '\n' as i32, __stream);
}
pub const PROMPT_NTYPES: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
static mut prompt_hlist: [*mut *mut ::core::ffi::c_char; 2] =
    [::core::ptr::null::<*mut ::core::ffi::c_char>() as *mut *mut ::core::ffi::c_char; 2];
static mut prompt_hsize: [u_int; 2] = [0; 2];
unsafe extern "C" fn prompt_find_history_file() -> *mut ::core::ffi::c_char {
    let mut home: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut history_file: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut path: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    history_file = options_get_string(
        global_options,
        b"history-file\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if *history_file as ::core::ffi::c_int == '\0' as i32 {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if *history_file as ::core::ffi::c_int == '/' as i32 {
        return xstrdup(history_file);
    }
    if *history_file.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '~' as i32
        || *history_file.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != '/' as i32
    {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    home = find_home();
    if home.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    xasprintf(
        &raw mut path,
        b"%s%s\0" as *const u8 as *const ::core::ffi::c_char,
        home,
        history_file.offset(1 as ::core::ffi::c_int as isize),
    );
    return path;
}
unsafe extern "C" fn prompt_add_typed_history(mut line: *mut ::core::ffi::c_char) {
    let mut typestr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut type_0: prompt_type = PROMPT_TYPE_INVALID;
    typestr = strsep(
        &raw mut line,
        b":\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if !line.is_null() {
        type_0 = prompt_type(typestr);
    }
    if type_0 as ::core::ffi::c_uint
        == PROMPT_TYPE_INVALID as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if !line.is_null() {
            line = line.offset(-1);
            *line = ':' as i32 as ::core::ffi::c_char;
        }
        prompt_add_history(typestr, PROMPT_TYPE_COMMAND as ::core::ffi::c_int as u_int);
    } else {
        prompt_add_history(line, type_0 as u_int);
    };
}
#[no_mangle]
pub unsafe extern "C" fn prompt_load_history() {
    let mut f: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut history_file: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut length: size_t = 0 as size_t;
    let mut got: ssize_t = 0;
    history_file = prompt_find_history_file();
    if history_file.is_null() {
        return;
    }
    log_debug(
        b"loading history from %s\0" as *const u8 as *const ::core::ffi::c_char,
        history_file,
    );
    f = fopen(
        history_file,
        b"r\0" as *const u8 as *const ::core::ffi::c_char,
    ) as *mut FILE;
    if f.is_null() {
        log_debug(
            b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            history_file,
            strerror(*__errno_location()),
        );
        free(history_file as *mut ::core::ffi::c_void);
        return;
    }
    free(history_file as *mut ::core::ffi::c_void);
    loop {
        got = getline(&raw mut line, &raw mut length, f) as ssize_t;
        if !(got != -(1 as ::core::ffi::c_int) as ssize_t) {
            break;
        }
        if got > 0 as ssize_t
            && *line.offset((got - 1 as ssize_t) as isize) as ::core::ffi::c_int == '\n' as i32
        {
            *line.offset((got - 1 as ssize_t) as isize) = '\0' as i32 as ::core::ffi::c_char;
        }
        if got > 0 as ssize_t {
            prompt_add_typed_history(line);
        }
    }
    free(line as *mut ::core::ffi::c_void);
    fclose(f);
}
#[no_mangle]
pub unsafe extern "C" fn prompt_save_history() {
    let mut f: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut i: u_int = 0;
    let mut type_0: u_int = 0;
    let mut history_file: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    history_file = prompt_find_history_file();
    if history_file.is_null() {
        return;
    }
    log_debug(
        b"saving history to %s\0" as *const u8 as *const ::core::ffi::c_char,
        history_file,
    );
    f = fopen(
        history_file,
        b"w\0" as *const u8 as *const ::core::ffi::c_char,
    ) as *mut FILE;
    if f.is_null() {
        log_debug(
            b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            history_file,
            strerror(*__errno_location()),
        );
        free(history_file as *mut ::core::ffi::c_void);
        return;
    }
    free(history_file as *mut ::core::ffi::c_void);
    type_0 = 0 as u_int;
    while type_0 < PROMPT_NTYPES as u_int {
        i = 0 as u_int;
        while i < prompt_hsize[type_0 as usize] {
            fputs(prompt_type_string(type_0 as prompt_type), f);
            fputc(':' as i32, f);
            fputs(*prompt_hlist[type_0 as usize].offset(i as isize), f);
            fputc('\n' as i32, f);
            i = i.wrapping_add(1);
        }
        type_0 = type_0.wrapping_add(1);
    }
    fclose(f);
}
#[no_mangle]
pub unsafe extern "C" fn prompt_up_history(
    mut idx: *mut u_int,
    mut type_0: u_int,
) -> *const ::core::ffi::c_char {
    if type_0 >= PROMPT_NTYPES as u_int {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    if prompt_hsize[type_0 as usize] == 0 as u_int
        || *idx.offset(type_0 as isize) == prompt_hsize[type_0 as usize]
    {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    let ref mut fresh0 = *idx.offset(type_0 as isize);
    *fresh0 = (*fresh0).wrapping_add(1);
    return *prompt_hlist[type_0 as usize]
        .offset(prompt_hsize[type_0 as usize].wrapping_sub(*idx.offset(type_0 as isize)) as isize);
}
#[no_mangle]
pub unsafe extern "C" fn prompt_down_history(
    mut idx: *mut u_int,
    mut type_0: u_int,
) -> *const ::core::ffi::c_char {
    if type_0 >= PROMPT_NTYPES as u_int {
        return b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if prompt_hsize[type_0 as usize] == 0 as u_int || *idx.offset(type_0 as isize) == 0 as u_int {
        return b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    let ref mut fresh1 = *idx.offset(type_0 as isize);
    *fresh1 = (*fresh1).wrapping_sub(1);
    if *idx.offset(type_0 as isize) == 0 as u_int {
        return b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return *prompt_hlist[type_0 as usize]
        .offset(prompt_hsize[type_0 as usize].wrapping_sub(*idx.offset(type_0 as isize)) as isize);
}
#[no_mangle]
pub unsafe extern "C" fn prompt_add_history(
    mut line: *const ::core::ffi::c_char,
    mut type_0: u_int,
) {
    let mut i: u_int = 0;
    let mut oldsize: u_int = 0;
    let mut newsize: u_int = 0;
    let mut freecount: u_int = 0;
    let mut hlimit: u_int = 0;
    let mut new: u_int = 1 as u_int;
    let mut movesize: size_t = 0;
    if type_0 >= PROMPT_NTYPES as u_int {
        return;
    }
    oldsize = prompt_hsize[type_0 as usize];
    if oldsize > 0 as u_int
        && strcmp(
            *prompt_hlist[type_0 as usize].offset(oldsize.wrapping_sub(1 as u_int) as isize),
            line,
        ) == 0 as ::core::ffi::c_int
    {
        new = 0 as u_int;
    }
    hlimit = options_get_number(
        global_options,
        b"prompt-history-limit\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    if hlimit > oldsize {
        if new == 0 as u_int {
            return;
        }
        newsize = oldsize.wrapping_add(new);
    } else {
        newsize = hlimit;
        freecount = oldsize.wrapping_add(new).wrapping_sub(newsize);
        if freecount > oldsize {
            freecount = oldsize;
        }
        if freecount == 0 as u_int {
            return;
        }
        i = 0 as u_int;
        while i < freecount {
            free(*prompt_hlist[type_0 as usize].offset(i as isize) as *mut ::core::ffi::c_void);
            i = i.wrapping_add(1);
        }
        movesize = (oldsize.wrapping_sub(freecount) as usize)
            .wrapping_mul(::core::mem::size_of::<*mut ::core::ffi::c_char>() as usize)
            as size_t;
        if movesize > 0 as size_t {
            memmove(
                (*(&raw mut prompt_hlist as *mut *mut *mut ::core::ffi::c_char)
                    .offset(type_0 as isize))
                .offset(0 as ::core::ffi::c_int as isize)
                    as *mut *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                (*(&raw mut prompt_hlist as *mut *mut *mut ::core::ffi::c_char)
                    .offset(type_0 as isize))
                .offset(freecount as isize) as *mut *mut ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                movesize,
            );
        }
    }
    if newsize == 0 as u_int {
        free(prompt_hlist[type_0 as usize] as *mut ::core::ffi::c_void);
        prompt_hlist[type_0 as usize] = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    } else if newsize != oldsize {
        prompt_hlist[type_0 as usize] = xreallocarray(
            prompt_hlist[type_0 as usize] as *mut ::core::ffi::c_void,
            newsize as size_t,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>() as size_t,
        ) as *mut *mut ::core::ffi::c_char;
    }
    if new == 1 as u_int && newsize > 0 as u_int {
        let ref mut fresh2 =
            *prompt_hlist[type_0 as usize].offset(newsize.wrapping_sub(1 as u_int) as isize);
        *fresh2 = xstrdup(line);
    }
    prompt_hsize[type_0 as usize] = newsize;
}
#[no_mangle]
pub unsafe extern "C" fn prompt_history_size(mut type_0: prompt_type) -> u_int {
    if type_0 as ::core::ffi::c_uint >= PROMPT_NTYPES as ::core::ffi::c_uint {
        return 0 as u_int;
    }
    return prompt_hsize[type_0 as usize];
}
#[no_mangle]
pub unsafe extern "C" fn prompt_history_get(
    mut type_0: prompt_type,
    mut idx: u_int,
) -> *const ::core::ffi::c_char {
    if type_0 as ::core::ffi::c_uint >= PROMPT_NTYPES as ::core::ffi::c_uint {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    if idx >= prompt_hsize[type_0 as usize] {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    return *prompt_hlist[type_0 as usize].offset(idx as isize);
}
#[no_mangle]
pub unsafe extern "C" fn prompt_history_clear(mut type_0: prompt_type) {
    let mut idx: u_int = 0;
    if type_0 as ::core::ffi::c_uint >= PROMPT_NTYPES as ::core::ffi::c_uint {
        return;
    }
    idx = 0 as u_int;
    while idx < prompt_hsize[type_0 as usize] {
        free(*prompt_hlist[type_0 as usize].offset(idx as isize) as *mut ::core::ffi::c_void);
        idx = idx.wrapping_add(1);
    }
    free(prompt_hlist[type_0 as usize] as *mut ::core::ffi::c_void);
    prompt_hlist[type_0 as usize] = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    prompt_hsize[type_0 as usize] = 0 as u_int;
}
