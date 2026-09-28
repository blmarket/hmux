use crate::src::options::options_owner_ptr;
use crate::src::cmd::cmd_stringify_argv_cstring;
use crate::src::ffi::libc::{
    __ctype_b_loc, __xpg_basename, gettimeofday, memcpy, strchr, strcmp, strcspn, strlen, strncmp,
};
use crate::src::format::{
    format_create, format_defaults_pane, format_defaults_window, format_expand_cstring, format_free,
};
use crate::src::log::{log_cstr, log_debug};
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

unsafe fn name_time_callback(w_owner: &std::rc::Rc<std::cell::UnsafeCell<window>>) {
    let mut w = w_owner.get();
    log_debug(format_args!("@{} name timer expired", ((*w).id) as u32));
}
unsafe fn name_time_expired(w_value: &window, mut tv: *mut timeval) -> ::core::ffi::c_int {
    let w: *mut window = w_value as *const _ as *mut _;
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
pub unsafe fn check_window_name(w_owner: &std::rc::Rc<std::cell::UnsafeCell<window>>) {
    let mut w = w_owner.get();
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut next: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut left: ::core::ffi::c_int = 0;
    if (*w).active_pane().is_none() {
        return;
    }
    if options_get_number(
        options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
        b"automatic-rename\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0
    {
        return;
    }
    if !(*(*w).active_pane().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get())).flags & PANE_CHANGED != 0 {
        log_debug(format_args!(
            "@{} active pane not changed",
            ((*w).id) as u32
        ));
        return;
    }
    log_debug(format_args!("@{} active pane changed", ((*w).id) as u32));
    gettimeofday(&raw mut tv, NULL);
    left = name_time_expired(&*(w), &raw mut tv);
    if left != 0 as ::core::ffi::c_int {
        if event_initialized(&(*w).name_event) == 0 {
            event_set(
                &raw mut (*w).name_event,
                -(1 as ::core::ffi::c_int),
                0 as ::core::ffi::c_short,
                {
                let observer = std::rc::Rc::downgrade(w_owner);
                move |_, _| unsafe {
                    if let Some(owner) = observer.upgrade() {
                        name_time_callback(&owner);
                    }
                }
            },
            );
        }
        if event_pending(
            &raw mut (*w).name_event,
            EV_TIMEOUT as ::core::ffi::c_short,
            ::core::ptr::null_mut::<timeval>(),
        ) == 0
        {
            log_debug(format_args!(
                "@{} name timer queued ({} left)",
                ((*w).id) as u32,
                (left) as i32
            ));
            next.tv_usec = 0 as __suseconds_t;
            next.tv_sec = next.tv_usec as __time_t;
            next.tv_usec = left as __suseconds_t;
            event_add(&raw mut (*w).name_event, &raw mut next);
        } else {
            log_debug(format_args!(
                "@{} name timer already queued ({} left)",
                ((*w).id) as u32,
                (left) as i32
            ));
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
    (*(*w).active_pane().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get())).flags &= !PANE_CHANGED;
    let name = format_window_name(w_owner);
    if strcmp(name.as_ptr().cast_mut(), (*w).name.as_ptr().cast_mut()) != 0 as ::core::ffi::c_int {
        log_debug(format_args!(
            "@{} new name {} (was {})",
            ((*w).id) as u32,
            crate::src::log::log_bytes(name.as_bytes()),
            crate::src::log::log_bytes((*w).name.as_bytes())
        ));
        window_set_name(w_owner, name.as_ptr().cast_mut(), 1 as ::core::ffi::c_int);
        server_redraw_window_borders(&*(w));
        server_status_window(&*(w));
    } else {
        log_debug(format_args!(
            "@{} name not changed (still {})",
            ((*w).id) as u32,
            crate::src::log::log_bytes((*w).name.as_bytes())
        ));
    }
}

pub(crate) unsafe fn default_window_name_cstring(w: &window) -> CString {
    if w.active_pane().is_none() {
        return c"".to_owned();
    }
    let cmd = cmd_stringify_argv_cstring(&(*w.active_pane().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get())).argv);
    if let Some(cmd) = cmd.as_ref().filter(|text| !text.as_bytes().is_empty()) {
        parse_window_name_cstring(cmd)
    } else {
        parse_window_name_cstring(CStr::from_ptr(
            (*w.active_pane().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()))
                .shell
                .as_ref()
                .map_or(::core::ptr::null(), |value| value.as_ptr()),
        ))
    }
}
unsafe fn format_window_name(w_owner: &std::rc::Rc<std::cell::UnsafeCell<window>>) -> CString {
    let mut w = w_owner.get();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut fmt: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut ft_owner = format_create(
        None,
        None,
        (FORMAT_WINDOW | (*w).id) as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    ft = &raw mut *ft_owner;
    format_defaults_window(ft, Some(w_owner));
    format_defaults_pane(ft, &(*((*w).active_pane().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()))).observer.upgrade().expect("live window_pane"));
    fmt = options_get_string(
        options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
        b"automatic-rename-format\0" as *const u8 as *const ::core::ffi::c_char,
    );
    let name = format_expand_cstring(ft, fmt);
    format_free(ft_owner);
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
            // Only the active field is read; other fields need valid defaults.
            let mut w = window::default();
            let w = &raw mut w;
            assert_eq!(default_window_name_cstring(&*w), c"");
        }
    }
}
