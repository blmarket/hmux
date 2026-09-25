use crate::src::cmd::cmd_stringify_argv_cstring;
use crate::src::ffi::libc::{
    __ctype_b_loc, __xpg_basename, gettimeofday, memcpy, strchr, strcmp, strcspn, strlen, strncmp,
};
use crate::src::format::{
    format_create, format_defaults_pane, format_defaults_window, format_expand_cstring, format_free,
};
use crate::src::log::log_debug;
use crate::src::options::{options_get_number, options_get_string};
use crate::src::reactor::{event_add, event_del, event_initialized, event_pending, event_set};
use crate::src::server_fn::{server_redraw_window_borders, server_status_window};
use crate::src::shared::abi::*;
use crate::src::shared::client::client;
use crate::src::shared::command::cmdq_item;
use crate::src::shared::ctype::{_ISalnum, _ISpunct};
use crate::src::shared::event::EV_TIMEOUT;
use crate::src::shared::format::format_tree;
use crate::src::shared::format::FORMAT_WINDOW;
use crate::src::shared::pane::PANE_CHANGED;
use crate::src::shared::window::window;
use crate::src::tmux::clean_name_cstring;
use crate::src::window::window_set_name;
use std::ffi::{CStr, CString};

pub const NAME_INTERVAL: ::core::ffi::c_int = 500000 as ::core::ffi::c_int;

unsafe extern "C" fn name_time_callback(
    _fd: ::core::ffi::c_int,
    _events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut w: *mut window = arg as *mut window;
    log_debug(
        b"@%u name timer expired\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).id,
    );
}
unsafe extern "C" fn name_time_expired(
    mut w: *mut window,
    mut tv: *mut timeval,
) -> ::core::ffi::c_int {
    let mut offset: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    offset.tv_sec = (*tv).tv_sec - (*w).name_time.tv_sec;
    offset.tv_usec = (*tv).tv_usec - (*w).name_time.tv_usec;
    if offset.tv_usec < 0 as __suseconds_t {
        offset.tv_sec -= 1;
        offset.tv_usec += 1000000 as __suseconds_t;
    }
    if offset.tv_sec != 0 as __time_t || offset.tv_usec > NAME_INTERVAL as __suseconds_t {
        return 0 as ::core::ffi::c_int;
    }
    return (NAME_INTERVAL as __suseconds_t - offset.tv_usec) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn check_window_name(mut w: *mut window) {
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut next: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut left: ::core::ffi::c_int = 0;
    if (*w).active.is_null() {
        return;
    }
    if options_get_number(
        (*w).options,
        b"automatic-rename\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0
    {
        return;
    }
    if !(*(*w).active).flags & PANE_CHANGED != 0 {
        log_debug(
            b"@%u active pane not changed\0" as *const u8 as *const ::core::ffi::c_char,
            (*w).id,
        );
        return;
    }
    log_debug(
        b"@%u active pane changed\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).id,
    );
    gettimeofday(&raw mut tv, NULL);
    left = name_time_expired(w, &raw mut tv);
    if left != 0 as ::core::ffi::c_int {
        if event_initialized(&(*w).name_event) == 0 {
            event_set(
                &raw mut (*w).name_event,
                -(1 as ::core::ffi::c_int),
                0 as ::core::ffi::c_short,
                Some(
                    name_time_callback
                        as unsafe extern "C" fn(
                            ::core::ffi::c_int,
                            ::core::ffi::c_short,
                            *mut ::core::ffi::c_void,
                        ) -> (),
                ),
                w as *mut ::core::ffi::c_void,
            );
        }
        if event_pending(
            &raw mut (*w).name_event,
            EV_TIMEOUT as ::core::ffi::c_short,
            ::core::ptr::null_mut::<timeval>(),
        ) == 0
        {
            log_debug(
                b"@%u name timer queued (%d left)\0" as *const u8 as *const ::core::ffi::c_char,
                (*w).id,
                left,
            );
            next.tv_usec = 0 as __suseconds_t;
            next.tv_sec = next.tv_usec as __time_t;
            next.tv_usec = left as __suseconds_t;
            event_add(&raw mut (*w).name_event, &raw mut next);
        } else {
            log_debug(
                b"@%u name timer already queued (%d left)\0" as *const u8
                    as *const ::core::ffi::c_char,
                (*w).id,
                left,
            );
        }
        return;
    }
    memcpy(
        &raw mut (*w).name_time as *mut ::core::ffi::c_void,
        &raw mut tv as *const ::core::ffi::c_void,
        ::core::mem::size_of::<timeval>() as size_t,
    );
    if event_initialized(&(*w).name_event) != 0 {
        event_del(&raw mut (*w).name_event);
    }
    (*(*w).active).flags &= !PANE_CHANGED;
    let name = format_window_name(w);
    if strcmp(name.as_ptr().cast_mut(), (*w).name) != 0 as ::core::ffi::c_int {
        log_debug(
            b"@%u new name %s (was %s)\0" as *const u8 as *const ::core::ffi::c_char,
            (*w).id,
            name.as_ptr(),
            (*w).name,
        );
        window_set_name(w, name.as_ptr().cast_mut(), 1 as ::core::ffi::c_int);
        server_redraw_window_borders(w);
        server_status_window(w);
    } else {
        log_debug(
            b"@%u name not changed (still %s)\0" as *const u8 as *const ::core::ffi::c_char,
            (*w).id,
            (*w).name,
        );
    }
}

pub(crate) unsafe fn default_window_name_cstring(w: &window) -> CString {
    if w.active.is_null() {
        return c"".to_owned();
    }
    let cmd = cmd_stringify_argv_cstring(&(*w.active).argv);
    if let Some(cmd) = cmd.as_ref().filter(|text| !text.as_bytes().is_empty()) {
        parse_window_name_cstring(cmd)
    } else {
        parse_window_name_cstring(CStr::from_ptr((*w.active).shell))
    }
}
unsafe fn format_window_name(w: *mut window) -> CString {
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut fmt: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    ft = format_create(
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<cmdq_item>(),
        (FORMAT_WINDOW | (*w).id) as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    format_defaults_window(ft, w);
    format_defaults_pane(ft, (*w).active);
    fmt = options_get_string(
        (*w).options,
        b"automatic-rename-format\0" as *const u8 as *const ::core::ffi::c_char,
    );
    let name = format_expand_cstring(ft, fmt);
    format_free(ft);
    name
}

pub unsafe fn parse_window_name_cstring(in_0: &CStr) -> CString {
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    // Keep the writable copy alive through basename and name cleaning.
    let mut copy = in_0.to_bytes_with_nul().to_vec();
    name = copy.as_mut_ptr().cast();
    if *name as ::core::ffi::c_int == '"' as i32 {
        name = name.offset(1);
    }
    *name.offset(strcspn(name, b"\"\0" as *const u8 as *const ::core::ffi::c_char) as isize) =
        '\0' as i32 as ::core::ffi::c_char;
    if strncmp(
        name,
        b"exec \0" as *const u8 as *const ::core::ffi::c_char,
        (::core::mem::size_of::<[::core::ffi::c_char; 6]>() as size_t).wrapping_sub(1 as size_t),
    ) == 0 as ::core::ffi::c_int
    {
        name = name
            .offset(::core::mem::size_of::<[::core::ffi::c_char; 6]>() as usize as isize)
            .offset(-(1 as ::core::ffi::c_int as isize));
    }
    while *name as ::core::ffi::c_int == ' ' as i32 || *name as ::core::ffi::c_int == '-' as i32 {
        name = name.offset(1);
    }
    ptr = strchr(name, ' ' as i32);
    if !ptr.is_null() {
        *ptr = '\0' as i32 as ::core::ffi::c_char;
    }
    if *name as ::core::ffi::c_int != '\0' as i32 {
        ptr = name
            .offset(strlen(name) as isize)
            .offset(-(1 as ::core::ffi::c_int as isize));
        while ptr > name
            && *(*__ctype_b_loc()).offset(*ptr as u_char as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                & _ISalnum as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                == 0
            && *(*__ctype_b_loc()).offset(*ptr as u_char as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                & _ISpunct as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                == 0
        {
            let fresh0 = ptr;
            ptr = ptr.offset(-1);
            *fresh0 = '\0' as i32 as ::core::ffi::c_char;
        }
    }
    if *name as ::core::ffi::c_int == '/' as i32 {
        name = __xpg_basename(name);
    }
    clean_name_cstring(CStr::from_ptr(name), 0).unwrap_or_else(|| c"".to_owned())
}

#[cfg(test)]
mod owned_name_tests {
    use super::*;

    #[test]
    fn window_without_active_pane_has_empty_owned_and_c_names() {
        unsafe {
            // Only the active field is read in this fixture.
            let w = std::mem::MaybeUninit::<window>::zeroed();
            let w = w.as_ptr().cast_mut();
            assert_eq!(default_window_name_cstring(&*w), c"");
        }
    }
}
