use crate::src::cmd::find::cmd_find_from_window;
use crate::src::control::control_get_window_size;
use crate::src::events::{events_fire, events_fire_window};
use crate::src::events_payload::{
    event_payload_create, event_payload_set_target, event_payload_set_uint,
    event_payload_set_window,
};
use crate::src::ffi::libc::sscanf;
use crate::src::layout::layout_resize;
use crate::src::log::{log_cstr, log_debug};
use crate::src::options::options_owner_ptr;
use crate::src::options::{options_get_number, options_get_string};
use crate::src::server::clients;
use crate::src::server_fn::server_redraw_window;
use crate::src::session::sessions;
use crate::src::session::{session_has, sessions_minmax, sessions_next};
use crate::src::shared::events::event_payload;
use crate::src::status::{status_line_size, status_update_cache};
use crate::src::tmux::global_w_options;
use crate::src::tty::tty_update_window_offset;
use crate::src::window::windows;
use crate::src::window::{
    window_has_pane, window_resize, window_unzoom, window_zoom, window_zoomed_pane, windows_minmax,
    windows_next,
};

use crate::src::shared::abi::*;
use crate::src::shared::client::client;
use crate::src::shared::client::{
    CLIENT_CONTROL, CLIENT_IGNORESIZE, CLIENT_NOSIZEFLAGS, CLIENT_SIZECHANGED, CLIENT_STATUSOFF,
    CLIENT_UNATTACHEDFLAGS, CLIENT_WINDOWSIZECHANGED,
};
use crate::src::shared::command::cmd_find_state;
use crate::src::shared::limits::UINT_MAX;
use crate::src::shared::pane::window_pane;
use crate::src::shared::session::session;
use crate::src::shared::tree::RB_NEGINF;
use crate::src::shared::window::{
    WINDOW_MAXIMUM, WINDOW_MINIMUM, WINDOW_RESIZE, WINDOW_SIZE_LARGEST, WINDOW_SIZE_LATEST,
    WINDOW_SIZE_MANUAL,
};
use crate::src::shared::window::{window, winlink};

unsafe fn resize_fire_window_resized(
    w_owner: &std::rc::Rc<std::cell::UnsafeCell<window>>,
    mut old_sx: u_int,
    mut old_sy: u_int,
) {
    unsafe {
        let mut w = w_owner.get();
        let mut fs: cmd_find_state = cmd_find_state {
            flags: 0,
            s: Default::default(),
            wl: Default::default(),
            w: Default::default(),
            wp: Default::default(),
            idx: 0,
        };
        let mut ep = event_payload_create();
        cmd_find_from_window(&raw mut fs, w_owner, 0 as ::core::ffi::c_int);
        event_payload_set_target(&mut *ep, &fs);
        event_payload_set_window(
            &mut *ep,
            b"window\0" as *const u8 as *const ::core::ffi::c_char,
            (*(w)).observer.upgrade().expect("live window"),
        );
        event_payload_set_uint(
            &mut *ep,
            b"width\0" as *const u8 as *const ::core::ffi::c_char,
            (*w).sx,
        );
        event_payload_set_uint(
            &mut *ep,
            b"height\0" as *const u8 as *const ::core::ffi::c_char,
            (*w).sy,
        );
        event_payload_set_uint(
            &mut *ep,
            b"old_width\0" as *const u8 as *const ::core::ffi::c_char,
            old_sx,
        );
        event_payload_set_uint(
            &mut *ep,
            b"old_height\0" as *const u8 as *const ::core::ffi::c_char,
            old_sy,
        );
        events_fire(
            b"window-resized\0" as *const u8 as *const ::core::ffi::c_char,
            ep,
        );
    }
}
pub unsafe fn resize_window(
    w_owner: &std::rc::Rc<std::cell::UnsafeCell<window>>,
    mut sx: u_int,
    mut sy: u_int,
    mut xpixel: ::core::ffi::c_int,
    mut ypixel: ::core::ffi::c_int,
) {
    unsafe {
        let mut w = w_owner.get();
        let mut old_sx: u_int = (*w).sx;
        let mut old_sy: u_int = (*w).sy;
        if sx < WINDOW_MINIMUM as u_int {
            sx = WINDOW_MINIMUM as u_int;
        }
        if sx > WINDOW_MAXIMUM as u_int {
            sx = WINDOW_MAXIMUM as u_int;
        }
        if sy < WINDOW_MINIMUM as u_int {
            sy = WINDOW_MINIMUM as u_int;
        }
        if sy > WINDOW_MAXIMUM as u_int {
            sy = WINDOW_MAXIMUM as u_int;
        }
        let zoomed_owner = window_zoomed_pane(&*w);
        if zoomed_owner.is_some() {
            window_unzoom(w_owner, 1 as ::core::ffi::c_int);
        }
        layout_resize(w_owner, sx, sy);
        if sx
            < (*(*w)
                .layout_root_ptr()
                .map_or(std::ptr::null_mut(), |root| root))
            .g
            .sx
        {
            sx = (*(*w)
                .layout_root_ptr()
                .map_or(std::ptr::null_mut(), |root| root))
            .g
            .sx;
        }
        if sy
            < (*(*w)
                .layout_root_ptr()
                .map_or(std::ptr::null_mut(), |root| root))
            .g
            .sy
        {
            sy = (*(*w)
                .layout_root_ptr()
                .map_or(std::ptr::null_mut(), |root| root))
            .g
            .sy;
        }
        window_resize(w_owner, sx, sy, xpixel, ypixel);
        log_debug(format_args!(
            "{}: @{} resized to {}x{}; layout {}x{}",
            "resize_window",
            ((*w).id) as u32,
            (sx) as u32,
            (sy) as u32,
            ((*(*w)
                .layout_root_ptr()
                .map_or(std::ptr::null_mut(), |root| root))
            .g
            .sx) as u32,
            ((*(*w)
                .layout_root_ptr()
                .map_or(std::ptr::null_mut(), |root| root))
            .g
            .sy) as u32
        ));
        if let Some(zoomed_owner) = zoomed_owner {
            if window_has_pane(&*w, &(*zoomed_owner.get()).observer) {
                window_zoom(&zoomed_owner);
            }
        }
        tty_update_window_offset(&(*(w)).observer.upgrade().expect("live window"));
        server_redraw_window(&*(w));
        events_fire_window(
            b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
            (*(w)).observer.upgrade().expect("live window"),
        );
        resize_fire_window_resized(w_owner, old_sx, old_sy);
        (*w).flags &= !WINDOW_RESIZE;
    }
}
unsafe fn ignore_client_size(c_value: &client) -> ::core::ffi::c_int {
    unsafe {
        let c: *mut client = c_value as *const _ as *mut _;
        let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
        if (*c).session_handle().is_none() {
            return 1 as ::core::ffi::c_int;
        }
        if (*c).flags & CLIENT_NOSIZEFLAGS as uint64_t != 0 {
            return 1 as ::core::ffi::c_int;
        }
        if (*c).flags & CLIENT_IGNORESIZE as uint64_t != 0 {
            let mut registry_loop_0_owner = clients.first();
            loop_0 = registry_loop_0_owner
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get());
            while !loop_0.is_null() {
                if !(*loop_0).session_handle().is_none() {
                    if !((*loop_0).flags & CLIENT_NOSIZEFLAGS as uint64_t != 0) {
                        if !(*loop_0).flags & CLIENT_IGNORESIZE as uint64_t != 0 {
                            return 1 as ::core::ffi::c_int;
                        }
                    }
                }
                registry_loop_0_owner = clients.next(
                    registry_loop_0_owner
                        .as_ref()
                        .expect("current registry client"),
                );
                loop_0 = registry_loop_0_owner
                    .as_ref()
                    .map_or(std::ptr::null_mut(), |owner| owner.get());
            }
        }
        if (*c).flags & CLIENT_CONTROL as uint64_t != 0
            && !(*c).flags & CLIENT_SIZECHANGED as uint64_t != 0
            && !(*c).flags as ::core::ffi::c_ulonglong & CLIENT_WINDOWSIZECHANGED != 0
        {
            return 1 as ::core::ffi::c_int;
        }
        return 0 as ::core::ffi::c_int;
    }
}
unsafe fn clients_with_window(w_owner: &std::rc::Rc<std::cell::UnsafeCell<window>>) -> u_int {
    unsafe {
        let mut w = w_owner.get();
        let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
        let mut n: u_int = 0 as u_int;
        let mut registry_loop_0_owner = clients.first();
        loop_0 = registry_loop_0_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
        while !loop_0.is_null() {
            if !(ignore_client_size(&*(loop_0)) != 0
                || session_has(
                    &*(*loop_0)
                        .session_handle()
                        .as_ref()
                        .map_or(std::ptr::null_mut(), |owner| owner.get()),
                    &*w,
                ) == 0)
            {
                n = n.wrapping_add(1);
                if n > 1 as u_int {
                    break;
                }
            }
            registry_loop_0_owner = clients.next(
                registry_loop_0_owner
                    .as_ref()
                    .expect("current registry client"),
            );
            loop_0 = registry_loop_0_owner
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get());
        }
        return n;
    }
}
unsafe fn clients_calculate_size(
    mut type_0: ::core::ffi::c_int,
    c_owner: Option<&std::rc::Rc<std::cell::UnsafeCell<client>>>,
    w_owner: Option<&std::rc::Rc<std::cell::UnsafeCell<window>>>,
    mut skip_client: impl FnMut(&client) -> bool,
    mut sx: *mut u_int,
    mut sy: *mut u_int,
    mut xpixel: *mut u_int,
    mut ypixel: *mut u_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut c = c_owner.map_or(std::ptr::null_mut(), |owner| owner.get());
        let mut w = w_owner.map_or(std::ptr::null_mut(), |owner| owner.get());
        let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
        let mut cx: u_int = 0;
        let mut cy: u_int = 0;
        let mut n: u_int = 0 as u_int;
        if type_0 == WINDOW_SIZE_LARGEST {
            *sx = 0 as u_int;
            *sy = 0 as u_int;
        } else if !w.is_null() && type_0 == WINDOW_SIZE_MANUAL {
            *sx = (*w).manual_sx;
            *sy = (*w).manual_sy;
            log_debug(format_args!(
                "{}: manual size {}x{}",
                "clients_calculate_size",
                (*sx) as u32,
                (*sy) as u32
            ));
        } else {
            *sx = UINT_MAX as u_int;
            *sy = UINT_MAX as u_int;
        }
        *ypixel = 0 as u_int;
        *xpixel = *ypixel;
        if type_0 == WINDOW_SIZE_LATEST && !w.is_null() {
            n = clients_with_window(&(*(w)).observer.upgrade().expect("live window"));
        }
        if !(type_0 == WINDOW_SIZE_MANUAL) {
            let mut registry_loop_0_owner = clients.first();
            loop_0 = registry_loop_0_owner
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get());
            while !loop_0.is_null() {
                if loop_0 != c && ignore_client_size(&*(loop_0)) != 0 {
                    log_debug(format_args!(
                        "{}: ignoring {} (1)",
                        "clients_calculate_size",
                        log_cstr(
                            (((*loop_0).name)
                                .as_ref()
                                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                                as *const _
                        )
                    ));
                } else if loop_0 != c && skip_client(&*loop_0) {
                    log_debug(format_args!(
                        "{}: skipping {} (1)",
                        "clients_calculate_size",
                        log_cstr(
                            (((*loop_0).name)
                                .as_ref()
                                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                                as *const _
                        )
                    ));
                } else if type_0 == WINDOW_SIZE_LATEST
                    && n > 1 as u_int
                    && !(*w).latest.ptr_eq(&(*loop_0).observer)
                {
                    log_debug(format_args!(
                        "{}: {} is not latest",
                        "clients_calculate_size",
                        log_cstr(
                            (((*loop_0).name)
                                .as_ref()
                                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                                as *const _
                        )
                    ));
                } else {
                    if w.is_null()
                        || control_get_window_size(&*(loop_0), (*w).id, &raw mut cx, &raw mut cy)
                            == 0
                        || cx == 0 as u_int
                        || cy == 0 as u_int
                    {
                        cx = (*loop_0).tty.sx;
                        cy = (*loop_0).tty.sy.wrapping_sub(status_line_size(&*loop_0));
                    }
                    if type_0 == WINDOW_SIZE_LARGEST {
                        if cx > *sx {
                            *sx = cx;
                        }
                        if cy > *sy {
                            *sy = cy;
                        }
                    } else {
                        if cx < *sx {
                            *sx = cx;
                        }
                        if cy < *sy {
                            *sy = cy;
                        }
                    }
                    if (*loop_0).tty.xpixel > *xpixel && (*loop_0).tty.ypixel > *ypixel {
                        *xpixel = (*loop_0).tty.xpixel;
                        *ypixel = (*loop_0).tty.ypixel;
                    }
                    log_debug(format_args!(
                        "{}: after {} ({}x{}), size is {}x{}",
                        "clients_calculate_size",
                        log_cstr(
                            (((*loop_0).name)
                                .as_ref()
                                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                                as *const _
                        ),
                        (cx) as u32,
                        (cy) as u32,
                        (*sx) as u32,
                        (*sy) as u32
                    ));
                }
                registry_loop_0_owner = clients.next(
                    registry_loop_0_owner
                        .as_ref()
                        .expect("current registry client"),
                );
                loop_0 = registry_loop_0_owner
                    .as_ref()
                    .map_or(std::ptr::null_mut(), |owner| owner.get());
            }
            if *sx != UINT_MAX && *sy != UINT_MAX {
                log_debug(format_args!(
                    "{}: calculated size {}x{}",
                    "clients_calculate_size",
                    (*sx) as u32,
                    (*sy) as u32
                ));
            } else {
                log_debug(format_args!(
                    "{}: no calculated size",
                    "clients_calculate_size"
                ));
            }
        }
        if !w.is_null() {
            let mut registry_loop_0_owner = clients.first();
            loop_0 = registry_loop_0_owner
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get());
            while !loop_0.is_null() {
                if !(loop_0 != c && ignore_client_size(&*(loop_0)) != 0) {
                    if !(loop_0 != c && skip_client(&*loop_0)) {
                        if !(!(*loop_0).flags as ::core::ffi::c_ulonglong
                            & CLIENT_WINDOWSIZECHANGED
                            != 0)
                        {
                            if !(control_get_window_size(
                                &*(loop_0),
                                (*w).id,
                                &raw mut cx,
                                &raw mut cy,
                            ) == 0)
                            {
                                log_debug(format_args!(
                                    "{}: {} size for @{} is {}x{}",
                                    log_cstr(
                                        (b"clients_calculate_size\0" as *const u8
                                            as *const ::core::ffi::c_char)
                                            as *const _
                                    ),
                                    log_cstr(
                                        (((*loop_0).name)
                                            .as_ref()
                                            .map_or(::core::ptr::null_mut(), |value| {
                                                value.as_ptr().cast_mut()
                                            })) as *const _
                                    ),
                                    ((*w).id) as u32,
                                    (cx) as u32,
                                    (cy) as u32
                                ));
                                if cx != 0 as u_int && *sx > cx {
                                    *sx = cx;
                                }
                                if cy != 0 as u_int && *sy > cy {
                                    *sy = cy;
                                }
                            }
                        }
                    }
                }
                registry_loop_0_owner = clients.next(
                    registry_loop_0_owner
                        .as_ref()
                        .expect("current registry client"),
                );
                loop_0 = registry_loop_0_owner
                    .as_ref()
                    .map_or(std::ptr::null_mut(), |owner| owner.get());
            }
        }
        if *sx != UINT_MAX && *sy != UINT_MAX {
            log_debug(format_args!(
                "{}: calculated size {}x{}",
                "clients_calculate_size",
                (*sx) as u32,
                (*sy) as u32
            ));
        } else {
            log_debug(format_args!(
                "{}: no calculated size",
                "clients_calculate_size"
            ));
        }
        if type_0 == WINDOW_SIZE_MANUAL {
            log_debug(format_args!("{}: type is manual", "clients_calculate_size"));
            return (w != NULL as *mut window) as ::core::ffi::c_int;
        }
        if type_0 == WINDOW_SIZE_LARGEST {
            log_debug(format_args!(
                "{}: type is largest",
                "clients_calculate_size"
            ));
            return (*sx != 0 as u_int && *sy != 0 as u_int) as ::core::ffi::c_int;
        }
        if type_0 == WINDOW_SIZE_LATEST {
            log_debug(format_args!("{}: type is latest", "clients_calculate_size"));
        } else {
            log_debug(format_args!(
                "{}: type is smallest",
                "clients_calculate_size"
            ));
        }
        return (*sx != UINT_MAX && *sy != UINT_MAX) as ::core::ffi::c_int;
    }
}
pub unsafe fn default_window_size(
    c_owner: Option<&std::rc::Rc<std::cell::UnsafeCell<client>>>,
    s_owner: &std::rc::Rc<std::cell::UnsafeCell<session>>,
    w_owner: Option<&std::rc::Rc<std::cell::UnsafeCell<window>>>,
    mut sx: *mut u_int,
    mut sy: *mut u_int,
    mut xpixel: *mut u_int,
    mut ypixel: *mut u_int,
    mut type_0: ::core::ffi::c_int,
) {
    unsafe {
        let mut c = c_owner.map_or(std::ptr::null_mut(), |owner| owner.get());
        let mut s = s_owner.get();
        let mut w = w_owner.map_or(std::ptr::null_mut(), |owner| owner.get());
        let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        if type_0 == -(1 as ::core::ffi::c_int) {
            type_0 = options_get_number(
                global_w_options,
                b"window-size\0" as *const u8 as *const ::core::ffi::c_char,
            ) as ::core::ffi::c_int;
        }
        if type_0 == WINDOW_SIZE_LATEST && !c.is_null() && ignore_client_size(&*(c)) == 0 {
            *sx = (*c).tty.sx;
            *sy = (*c).tty.sy.wrapping_sub(status_line_size(&*c));
            *xpixel = (*c).tty.xpixel;
            *ypixel = (*c).tty.ypixel;
            log_debug(format_args!(
                "{}: using {}x{} from {}",
                "default_window_size",
                (*sx) as u32,
                (*sy) as u32,
                log_cstr(
                    (((*c).name)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                        as *const _
                )
            ));
        } else {
            if !c.is_null() && (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
                c = ::core::ptr::null_mut::<client>();
            }
            if clients_calculate_size(
                type_0,
                (c).as_ref()
                    .and_then(|model| model.observer.upgrade())
                    .as_ref(),
                w_owner,
                |candidate| unsafe {
                    (!w.is_null()
                        && session_has(
                            &*candidate
                                .session_handle()
                                .as_ref()
                                .map_or(std::ptr::null_mut(), |owner| owner.get()),
                            &*w,
                        ) == 0)
                        || (w.is_null()
                            && candidate
                                .session_handle()
                                .as_ref()
                                .map_or(std::ptr::null_mut(), |owner| owner.get())
                                != s)
                },
                sx,
                sy,
                xpixel,
                ypixel,
            ) == 0
            {
                value = options_get_string(
                    options_owner_ptr(&mut (*s).options)
                        .map_or(std::ptr::null_mut(), |options| options),
                    b"default-size\0" as *const u8 as *const ::core::ffi::c_char,
                );
                if sscanf(
                    value,
                    b"%ux%u\0" as *const u8 as *const ::core::ffi::c_char,
                    sx,
                    sy,
                ) != 2 as ::core::ffi::c_int
                {
                    *sx = 80 as u_int;
                    *sy = 24 as u_int;
                }
                log_debug(format_args!(
                    "{}: using {}x{} from default-size",
                    "default_window_size",
                    (*sx) as u32,
                    (*sy) as u32
                ));
            }
        }
        if *sx < WINDOW_MINIMUM as u_int {
            *sx = WINDOW_MINIMUM as u_int;
        }
        if *sx > WINDOW_MAXIMUM as u_int {
            *sx = WINDOW_MAXIMUM as u_int;
        }
        if *sy < WINDOW_MINIMUM as u_int {
            *sy = WINDOW_MINIMUM as u_int;
        }
        if *sy > WINDOW_MAXIMUM as u_int {
            *sy = WINDOW_MAXIMUM as u_int;
        }
        log_debug(format_args!(
            "{}: resulting size is {}x{}",
            "default_window_size",
            (*sx) as u32,
            (*sy) as u32
        ));
    }
}
pub unsafe fn recalculate_size(
    w_owner: &std::rc::Rc<std::cell::UnsafeCell<window>>,
    mut now: ::core::ffi::c_int,
) {
    unsafe {
        let mut w = w_owner.get();
        let mut sx: u_int = 0;
        let mut sy: u_int = 0;
        let mut xpixel: u_int = 0 as u_int;
        let mut ypixel: u_int = 0 as u_int;
        let mut type_0: ::core::ffi::c_int = 0;
        let mut current: ::core::ffi::c_int = 0;
        let mut changed: ::core::ffi::c_int = 0;
        if (*w).active_pane().is_none() {
            return;
        }
        log_debug(format_args!(
            "{}: @{} is {}x{}",
            "recalculate_size",
            ((*w).id) as u32,
            ((*w).sx) as u32,
            ((*w).sy) as u32
        ));
        type_0 = options_get_number(
            options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
            b"window-size\0" as *const u8 as *const ::core::ffi::c_char,
        ) as ::core::ffi::c_int;
        current = options_get_number(
            options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
            b"aggressive-resize\0" as *const u8 as *const ::core::ffi::c_char,
        ) as ::core::ffi::c_int;
        changed = clients_calculate_size(
            type_0,
            None,
            Some(w_owner),
            |candidate| unsafe {
                let session = candidate
                    .session_handle()
                    .as_ref()
                    .map_or(std::ptr::null_mut(), |owner| owner.get());
                if session.is_null() || !(*session).current_winlink().is_alive() {
                    return true;
                }
                if current != 0 {
                    ((*session).current_winlink())
                        .get_unchecked()
                        .window_handle()
                        .as_ref()
                        .map_or(std::ptr::null_mut(), |owner| owner.get())
                        != w
                } else {
                    session_has(&*session, &*w) == 0
                }
            },
            &raw mut sx,
            &raw mut sy,
            &raw mut xpixel,
            &raw mut ypixel,
        );
        if (*w).flags & WINDOW_RESIZE != 0 {
            if now == 0 && changed != 0 && (*w).new_sx == sx && (*w).new_sy == sy {
                changed = 0 as ::core::ffi::c_int;
            }
        } else if now == 0 && changed != 0 && (*w).sx == sx && (*w).sy == sy {
            changed = 0 as ::core::ffi::c_int;
        }
        if changed == 0 {
            log_debug(format_args!(
                "{}: @{} no size change",
                "recalculate_size",
                ((*w).id) as u32
            ));
            tty_update_window_offset(&(*(w)).observer.upgrade().expect("live window"));
            return;
        }
        log_debug(format_args!(
            "{}: @{} new size {}x{}",
            "recalculate_size",
            ((*w).id) as u32,
            (sx) as u32,
            (sy) as u32
        ));
        if now != 0 || type_0 == WINDOW_SIZE_MANUAL {
            resize_window(
                w_owner,
                sx,
                sy,
                xpixel as ::core::ffi::c_int,
                ypixel as ::core::ffi::c_int,
            );
        } else {
            (*w).new_sx = sx;
            (*w).new_sy = sy;
            (*w).new_xpixel = xpixel;
            (*w).new_ypixel = ypixel;
            (*w).flags |= WINDOW_RESIZE;
            tty_update_window_offset(&(*(w)).observer.upgrade().expect("live window"));
        };
    }
}
pub unsafe fn recalculate_sizes() {
    unsafe {
        recalculate_sizes_now(0 as ::core::ffi::c_int);
    }
}
pub unsafe fn recalculate_sizes_now(mut now: ::core::ffi::c_int) {
    unsafe {
        let mut s: *mut session = ::core::ptr::null_mut::<session>();
        let mut c: *mut client = ::core::ptr::null_mut::<client>();
        let mut w: *mut window = ::core::ptr::null_mut::<window>();
        let mut s_owner = sessions_minmax(&*std::ptr::addr_of!(sessions));
        s = s_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
        while !s.is_null() {
            (*s).attached = 0 as u_int;
            status_update_cache(&mut *(s));
            s_owner = sessions_next(&*s);
            s = s_owner
                .as_ref()
                .map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
        }
        let mut registry_c_owner = clients.first();
        c = registry_c_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
        while !c.is_null() {
            s = (*c)
                .session_handle()
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get());
            if !s.is_null() && (*c).flags & CLIENT_UNATTACHEDFLAGS as uint64_t == 0 {
                (*s).attached = (*s).attached.wrapping_add(1);
            }
            if !(ignore_client_size(&*(c)) != 0) {
                if (*c).tty.sy <= (*s).statuslines || (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
                    (*c).flags |= CLIENT_STATUSOFF as uint64_t;
                } else {
                    (*c).flags &= !CLIENT_STATUSOFF as uint64_t;
                }
            }
            registry_c_owner =
                clients.next(registry_c_owner.as_ref().expect("current registry client"));
            c = registry_c_owner
                .as_ref()
                .map_or(std::ptr::null_mut(), |owner| owner.get());
        }
        let mut window_cursor = windows_minmax(&*std::ptr::addr_of!(windows));
        while let Some(window_owner) = window_cursor.take() {
            w = window_owner.get();
            recalculate_size(&(*(w)).observer.upgrade().expect("live window"), now);
            window_cursor = windows_next(&*w);
            crate::src::window::window_remove_ref(window_owner, c"window traversal".as_ptr());
        }
    }
}
