use crate::src::ffi::libc::{
    __ctype_b_loc, __xpg_basename, gettimeofday, strchr, strcspn, strlen, strncmp,
};
use crate::src::format::{
    format_create, format_defaults_pane, format_defaults_window, format_expand_cstring, format_free,
};
use crate::src::log::log_debug;
use crate::src::options::options_get_number;
use crate::src::server_fn::{server_redraw_window_borders, server_status_window};
use crate::src::shared::abi::*;
use crate::src::shared::ctype::{_ISalnum, _ISpunct};
use crate::src::shared::format::FORMAT_WINDOW;
use crate::src::shared::window::window;
use crate::src::shared::window::WindowRef;
use crate::src::tmux::clean_name_cstring;
use crate::src::window::{Window as _, WindowPane as _};
use std::ffi::{CStr, CString};

pub const NAME_INTERVAL: ::core::ffi::c_int = 500000 as ::core::ffi::c_int;

pub(crate) fn name_time_left(previous: timeval, now: timeval) -> i32 {
    let mut seconds = now.tv_sec - previous.tv_sec;
    let mut micros = now.tv_usec - previous.tv_usec;
    if micros < 0 {
        seconds -= 1;
        micros += 1_000_000;
    }
    if seconds != 0 || micros > NAME_INTERVAL as _ {
        return 0;
    }
    (NAME_INTERVAL as __suseconds_t - micros) as i32
}

pub unsafe fn check_window_name(owner: &WindowRef) {
    if owner.active_pane().is_none() {
        return;
    }
    if owner.with_options_mut(|options| options_get_number(options, c"automatic-rename".as_ptr()))
        == 0
    {
        return;
    }
    let pane = owner.active_pane().expect("active pane");
    if !pane.has_pending_change() {
        log_debug(format_args!("@{} active pane not changed", owner.id()));
        return;
    }
    log_debug(format_args!("@{} active pane changed", owner.id()));
    let mut now = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    gettimeofday(&mut now, NULL);
    if !owner.begin_name_check(now) {
        return;
    }
    pane.acknowledge_change();
    drop(pane);
    let name = format_window_name(owner);
    let previous = owner.name();
    if name != previous {
        log_debug(format_args!(
            "@{} new name {} (was {})",
            owner.id(),
            crate::src::log::log_bytes(name.as_bytes()),
            crate::src::log::log_bytes(previous.as_bytes())
        ));
        owner.rename(&name, true);
        server_redraw_window_borders(owner);
        server_status_window(owner);
    } else {
        log_debug(format_args!(
            "@{} name not changed (still {})",
            owner.id(),
            crate::src::log::log_bytes(previous.as_bytes())
        ));
    }
}

pub(crate) unsafe fn default_window_name_cstring(owner: &WindowRef) -> CString {
    owner
        .active_pane()
        .map_or_else(|| c"".to_owned(), |pane| pane.default_window_name())
}
unsafe fn format_window_name(owner: &WindowRef) -> CString {
    let mut context = format_create(None, None, (FORMAT_WINDOW | owner.id()) as i32, 0);
    format_defaults_window(&mut *context, Some(owner));
    format_defaults_pane(&mut *context, &owner.active_pane().expect("active pane"));
    let format = owner.with_options_mut(|options| {
        crate::src::options::options_get_string_optional(
            options,
            c"automatic-rename-format".as_ptr(),
        )
    });
    let name = format_expand_cstring(
        &mut *context,
        format.as_deref().map_or(std::ptr::null(), CStr::as_ptr),
    );
    format_free(context);
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
    fn automatic_name_delay_handles_second_rollover_and_exact_deadline() {
        let previous = timeval {
            tv_sec: 10,
            tv_usec: 800_000,
        };
        for (seconds, micros, expected) in [
            (10, 800_000, NAME_INTERVAL),
            (10, 900_000, 400_000),
            (11, 0, 300_000),
            (11, 299_999, 1),
            (11, 300_000, 0),
            (11, 300_001, 0),
            (12, 0, 0),
        ] {
            assert_eq!(
                name_time_left(
                    previous,
                    timeval {
                        tv_sec: seconds,
                        tv_usec: micros
                    }
                ),
                expected
            );
        }
    }

    #[test]
    fn window_without_active_pane_has_empty_owned_and_c_names() {
        unsafe {
            let window = window::new();
            assert_eq!(default_window_name_cstring(&window), c"");
            window.release(c"empty default name test");
        }
    }
}
