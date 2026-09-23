pub use crate::src::ffi::libc::__ssize_t;
use crate::src::ffi::libc::{
    __errno_location, __getdelim, fclose, fopen, fputc, fputs, free, strcmp, strerror, strsep,
};
use crate::src::log::log_debug;
use crate::src::options::{options_get_number, options_get_string};
use crate::src::prompt::{prompt_type, prompt_type_string};
use crate::src::shared::abi::*;
pub use crate::src::shared::abi::{__off64_t, __off_t, ssize_t};
pub use crate::src::shared::options::options;
pub use crate::src::shared::prompt::PROMPT_NTYPES;
use crate::src::shared::prompt::*;
pub use crate::src::shared::stdio::{
    _IO_codecvt, _IO_lock_t, _IO_marker, _IO_wide_data, _IO_FILE, FILE,
};
use crate::src::tmux::{find_home, global_options};
use crate::src::xmalloc::{xasprintf, xstrdup};
use std::ffi::{CStr, CString};

#[inline]
unsafe extern "C" fn getline(
    mut __lineptr: *mut *mut ::core::ffi::c_char,
    mut __n: *mut size_t,
    mut __stream: *mut FILE,
) -> __ssize_t {
    return __getdelim(__lineptr, __n, '\n' as i32, __stream);
}
// The C API borrows each string until that entry is pruned or cleared. Moving
// CString values within the vector does not move their NUL-terminated buffers.
static mut prompt_hlist: [Vec<CString>; PROMPT_NTYPES as usize] = [Vec::new(), Vec::new()];
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
        let history = &*(&raw const prompt_hlist[type_0 as usize]);
        while i < history.len() as u_int {
            fputs(prompt_type_string(type_0 as prompt_type), f);
            fputc(':' as i32, f);
            fputs(history[i as usize].as_ptr(), f);
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
    let history = &*(&raw const prompt_hlist[type_0 as usize]);
    if history.is_empty() || *idx.offset(type_0 as isize) == history.len() as u_int {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    let ref mut fresh0 = *idx.offset(type_0 as isize);
    *fresh0 = (*fresh0).wrapping_add(1);
    return history[history.len() - *idx.offset(type_0 as isize) as usize].as_ptr();
}
#[no_mangle]
pub unsafe extern "C" fn prompt_down_history(
    mut idx: *mut u_int,
    mut type_0: u_int,
) -> *const ::core::ffi::c_char {
    if type_0 >= PROMPT_NTYPES as u_int {
        return b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    let history = &*(&raw const prompt_hlist[type_0 as usize]);
    if history.is_empty() || *idx.offset(type_0 as isize) == 0 as u_int {
        return b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    let ref mut fresh1 = *idx.offset(type_0 as isize);
    *fresh1 = (*fresh1).wrapping_sub(1);
    if *idx.offset(type_0 as isize) == 0 as u_int {
        return b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return history[history.len() - *idx.offset(type_0 as isize) as usize].as_ptr();
}
#[no_mangle]
pub unsafe extern "C" fn prompt_add_history(
    mut line: *const ::core::ffi::c_char,
    mut type_0: u_int,
) {
    if type_0 >= PROMPT_NTYPES as u_int {
        return;
    }
    let history = &*(&raw const prompt_hlist[type_0 as usize]);
    let oldsize = history.len() as u_int;
    let new = !history
        .last()
        .is_some_and(|last| strcmp(last.as_ptr(), line) == 0);
    let hlimit = options_get_number(
        global_options,
        b"prompt-history-limit\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    if hlimit > oldsize {
        if !new {
            return;
        }
    } else if oldsize + new as u_int - hlimit == 0 {
        return;
    }

    // `line` may borrow an existing entry. Copy it before pruning can drop
    // that entry, including when the new entry comes from the oldest slot.
    let added = (new && hlimit != 0).then(|| CStr::from_ptr(line).to_owned());
    let history = &mut *(&raw mut prompt_hlist[type_0 as usize]);
    if hlimit <= oldsize {
        let freecount = (oldsize + new as u_int - hlimit).min(oldsize) as usize;
        history.drain(..freecount);
    }
    if hlimit == 0 {
        // The old implementation freed the pointer list as well as its items.
        *history = Vec::new();
    } else if let Some(added) = added {
        history.push(added);
    }
}
#[no_mangle]
pub unsafe extern "C" fn prompt_history_size(mut type_0: prompt_type) -> u_int {
    if type_0 as ::core::ffi::c_uint >= PROMPT_NTYPES as ::core::ffi::c_uint {
        return 0 as u_int;
    }
    return (&*(&raw const prompt_hlist[type_0 as usize])).len() as u_int;
}
#[no_mangle]
pub unsafe extern "C" fn prompt_history_get(
    mut type_0: prompt_type,
    mut idx: u_int,
) -> *const ::core::ffi::c_char {
    if type_0 as ::core::ffi::c_uint >= PROMPT_NTYPES as ::core::ffi::c_uint {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    let history = &*(&raw const prompt_hlist[type_0 as usize]);
    if idx >= history.len() as u_int {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    return history[idx as usize].as_ptr();
}
#[no_mangle]
pub unsafe extern "C" fn prompt_history_clear(mut type_0: prompt_type) {
    if type_0 as ::core::ffi::c_uint >= PROMPT_NTYPES as ::core::ffi::c_uint {
        return;
    }
    *(&raw mut prompt_hlist[type_0 as usize]) = Vec::new();
}
